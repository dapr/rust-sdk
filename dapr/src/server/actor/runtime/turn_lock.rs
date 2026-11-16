use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub const REENTRANCY_ID_HEADER: &str = "dapr-reentrancy-id";

/// Coordinates turn-based execution and reentrancy per actor ID.
///
/// In the virtual actor model, an actor executes sequentially. This lock ensures
/// that calls to an actor ID are executed one at a time (FIFO), while permitting
/// matching reentrant calls when reentrancy is enabled for the actor type.
///
/// Crucially, if a caller context or HTTP connection is canceled during execution,
/// local turn ownership is retained until execution returns, preventing subsequent
/// requests from overlapping with lingering executions.
#[derive(Clone, Default)]
pub struct ActorTurnLock {
    inner: Arc<Mutex<HashMap<(String, String), TurnEntry>>>,
}

struct TurnWaiter {
    reentrancy_id: Option<String>,
    notify: Arc<Notify>,
}

struct TurnEntry {
    /// Total references to this entry (active executions + queued waiters).
    /// When refs drops to 0, the entry is automatically removed from the map.
    refs: usize,
    /// Number of active calls currently running within the turn.
    /// In non-reentrant execution, this is 0 or 1.
    /// In reentrant execution, nested calls sharing the same reentrancy ID increment this.
    active_calls: usize,
    /// Active reentrancy ID for the running turn, if any.
    reentrancy_id: Option<String>,
    /// Whether reentrancy is permitted for the active turn.
    /// Set to true once activation has completed.
    reentrancy_allowed: bool,
    /// Waiters waiting to acquire the turn, ordered FIFO.
    waiters: VecDeque<Arc<TurnWaiter>>,
}

impl TurnEntry {
    fn try_enter(&mut self, req_reentrancy_id: Option<&str>) -> bool {
        if self.active_calls > 0 {
            // Check if reentrancy is permitted and matches the active reentrancy ID
            let is_matching_reentrant = self.reentrancy_allowed
                && req_reentrancy_id.is_some()
                && req_reentrancy_id == self.reentrancy_id.as_deref()
                && req_reentrancy_id != Some("");
            if is_matching_reentrant {
                self.active_calls += 1;
                return true;
            }
            return false;
        }

        self.active_calls = 1;
        self.reentrancy_id = req_reentrancy_id.map(|s| s.to_string());
        self.reentrancy_allowed = false;
        true
    }

    fn wake_next_or_reentrant(&mut self) {
        if self.active_calls != 0 {
            return;
        }
        if let Some(waiter) = self.waiters.pop_front() {
            waiter.notify.notify_one();
        }
    }

    fn allow_reentrancy(&mut self) {
        if self.reentrancy_allowed || self.reentrancy_id.is_none() {
            return;
        }
        self.reentrancy_allowed = true;
        let active_reentrancy_id = self.reentrancy_id.as_deref();
        let mut remaining = VecDeque::new();
        while let Some(w) = self.waiters.pop_front() {
            let is_matching = w.reentrancy_id.as_deref().is_some()
                && w.reentrancy_id.as_deref() == active_reentrancy_id
                && w.reentrancy_id.as_deref() != Some("");
            if is_matching {
                w.notify.notify_one();
                continue;
            }
            remaining.push_back(w);
        }
        self.waiters = remaining;
    }
}

/// An active lease on an actor turn.
///
/// Dropping or releasing this lease notifies the next waiter in the FIFO queue
/// or cleans up the entry if no further calls are active or waiting.
pub struct ActorTurnLease {
    actor_key: (String, String),
    lock: ActorTurnLock,
    released: Mutex<bool>,
}

impl ActorTurnLock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquires the turn for the given actor type and actor ID.
    ///
    /// If another call is currently executing on this actor, this method awaits until
    /// the turn is released, unless `reentrancy_id` matches the active turn's reentrancy ID
    /// and reentrancy is permitted.
    pub async fn acquire(
        &self,
        actor_type: &str,
        actor_id: &str,
        reentrancy_id: Option<&str>,
    ) -> Arc<ActorTurnLease> {
        let key = (actor_type.to_string(), actor_id.to_string());
        let waiter_notify = {
            let mut map = self.inner.lock().unwrap();
            let entry = map.entry(key.clone()).or_insert_with(|| TurnEntry {
                refs: 0,
                active_calls: 0,
                reentrancy_id: None,
                reentrancy_allowed: false,
                waiters: VecDeque::new(),
            });
            entry.refs += 1;

            // Only enter immediately if no other waiters are ahead in line
            if entry.waiters.is_empty() && entry.try_enter(reentrancy_id) {
                return Arc::new(ActorTurnLease {
                    actor_key: key,
                    lock: self.clone(),
                    released: Mutex::new(false),
                });
            }

            // Must wait in FIFO queue
            let waiter = Arc::new(TurnWaiter {
                reentrancy_id: reentrancy_id.map(|s| s.to_string()),
                notify: Arc::new(Notify::new()),
            });
            entry.waiters.push_back(waiter.clone());
            waiter.notify.clone()
        };

        // Guard for cancellation while waiting in the queue
        struct WaiterGuard {
            key: (String, String),
            lock: ActorTurnLock,
            waiter_notify: Arc<Notify>,
            acquired: bool,
        }

        impl Drop for WaiterGuard {
            fn drop(&mut self) {
                if !self.acquired {
                    let mut map = self.lock.inner.lock().unwrap();
                    if let Some(entry) = map.get_mut(&self.key) {
                        entry
                            .waiters
                            .retain(|w| !Arc::ptr_eq(&w.notify, &self.waiter_notify));
                        entry.refs = entry.refs.saturating_sub(1);
                        if entry.refs == 0 {
                            map.remove(&self.key);
                        } else if entry.active_calls == 0 {
                            entry.wake_next_or_reentrant();
                        }
                    }
                }
            }
        }

        let mut guard = WaiterGuard {
            key: key.clone(),
            lock: self.clone(),
            waiter_notify,
            acquired: false,
        };

        loop {
            guard.waiter_notify.notified().await;
            let mut map = self.inner.lock().unwrap();
            if let Some(entry) = map.get_mut(&key) {
                if entry.try_enter(reentrancy_id) {
                    guard.acquired = true;
                    return Arc::new(ActorTurnLease {
                        actor_key: key,
                        lock: self.clone(),
                        released: Mutex::new(false),
                    });
                }
            } else {
                let _entry = map.entry(key.clone()).or_insert_with(|| TurnEntry {
                    refs: 1,
                    active_calls: 1,
                    reentrancy_id: reentrancy_id.map(|s| s.to_string()),
                    reentrancy_allowed: false,
                    waiters: VecDeque::new(),
                });
                guard.acquired = true;
                return Arc::new(ActorTurnLease {
                    actor_key: key,
                    lock: self.clone(),
                    released: Mutex::new(false),
                });
            }
        }
    }

    /// Queries the current lock state for tests.
    #[cfg(test)]
    pub fn get_state(&self, actor_type: &str, actor_id: &str) -> Option<(usize, usize)> {
        let map = self.inner.lock().unwrap();
        map.get(&(actor_type.to_string(), actor_id.to_string()))
            .map(|e| (e.active_calls, e.waiters.len()))
    }
}

impl ActorTurnLease {
    /// Allows calls sharing the active reentrancy ID to enter this turn.
    ///
    /// This should be called once actor activation has completed.
    pub fn allow_reentrancy(&self) {
        let released = self.released.lock().unwrap();
        if *released {
            return;
        }
        let mut map = self.lock.inner.lock().unwrap();
        if let Some(entry) = map.get_mut(&self.actor_key) {
            entry.allow_reentrancy();
        }
    }

    /// Releases the turn lease explicitly.
    pub fn release(&self) {
        let mut released = self.released.lock().unwrap();
        if *released {
            return;
        }
        *released = true;

        let mut map = self.lock.inner.lock().unwrap();
        if let Some(entry) = map.get_mut(&self.actor_key) {
            entry.active_calls = entry.active_calls.saturating_sub(1);
            if entry.active_calls == 0 {
                entry.reentrancy_id = None;
                entry.reentrancy_allowed = false;
                entry.wake_next_or_reentrant();
            }
            entry.refs = entry.refs.saturating_sub(1);
            if entry.refs == 0 {
                map.remove(&self.actor_key);
            }
        }
    }
}

impl Drop for ActorTurnLease {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_turn_lock_serializes_calls() {
        let lock = ActorTurnLock::new();
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let lock1 = lock.clone();
        let order1 = execution_order.clone();
        let h1 = tokio::spawn(async move {
            let lease = lock1.acquire("test_type", "actor1", None).await;
            order1.lock().unwrap().push("start_1");
            sleep(Duration::from_millis(50)).await;
            order1.lock().unwrap().push("end_1");
            drop(lease);
        });

        sleep(Duration::from_millis(10)).await;

        let lock2 = lock.clone();
        let order2 = execution_order.clone();
        let h2 = tokio::spawn(async move {
            let lease = lock2.acquire("test_type", "actor1", None).await;
            order2.lock().unwrap().push("start_2");
            sleep(Duration::from_millis(10)).await;
            order2.lock().unwrap().push("end_2");
            drop(lease);
        });

        h1.await.unwrap();
        h2.await.unwrap();

        let order = execution_order.lock().unwrap().clone();
        assert_eq!(order, vec!["start_1", "end_1", "start_2", "end_2"]);
    }

    #[tokio::test]
    async fn test_turn_lock_parallel_across_different_actor_ids() {
        let lock = ActorTurnLock::new();
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        let lock1 = lock.clone();
        let order1 = execution_order.clone();
        let h1 = tokio::spawn(async move {
            let lease = lock1.acquire("test_type", "actor1", None).await;
            order1.lock().unwrap().push("start_1");
            sleep(Duration::from_millis(50)).await;
            order1.lock().unwrap().push("end_1");
            drop(lease);
        });

        sleep(Duration::from_millis(10)).await;

        let lock2 = lock.clone();
        let order2 = execution_order.clone();
        let h2 = tokio::spawn(async move {
            let lease = lock2.acquire("test_type", "actor2", None).await;
            order2.lock().unwrap().push("start_2");
            sleep(Duration::from_millis(10)).await;
            order2.lock().unwrap().push("end_2");
            drop(lease);
        });

        h1.await.unwrap();
        h2.await.unwrap();

        let order = execution_order.lock().unwrap().clone();
        // Since actor2 is a different actor ID, start_2 runs concurrently before end_1
        assert_eq!(order[0], "start_1");
        assert_eq!(order[1], "start_2");
    }

    #[tokio::test]
    async fn test_turn_lock_reentrancy() {
        let lock = ActorTurnLock::new();
        let lease1 = lock
            .acquire("test_type", "actor1", Some("reentrant-123"))
            .await;

        // Before allow_reentrancy, matching call should wait
        let lock_clone = lock.clone();
        let entered = Arc::new(Mutex::new(false));
        let entered_clone = entered.clone();
        let h = tokio::spawn(async move {
            let _lease2 = lock_clone
                .acquire("test_type", "actor1", Some("reentrant-123"))
                .await;
            *entered_clone.lock().unwrap() = true;
        });

        sleep(Duration::from_millis(20)).await;
        assert!(!*entered.lock().unwrap());

        // Call allow_reentrancy: matching reentrancy call should now enter immediately
        lease1.allow_reentrancy();
        sleep(Duration::from_millis(20)).await;
        assert!(*entered.lock().unwrap());

        h.await.unwrap();
    }

    #[tokio::test]
    async fn test_turn_lock_cancelled_waiter_cleans_up() {
        let lock = ActorTurnLock::new();
        let lease1 = lock.acquire("test_type", "actor1", None).await;

        let lock_clone = lock.clone();
        let h = tokio::spawn(async move {
            let _lease2 = lock_clone.acquire("test_type", "actor1", None).await;
        });

        sleep(Duration::from_millis(20)).await;
        assert_eq!(lock.get_state("test_type", "actor1"), Some((1, 1)));

        // Abort the waiting task
        h.abort();
        sleep(Duration::from_millis(20)).await;

        // Waiting task is removed from queue
        assert_eq!(lock.get_state("test_type", "actor1"), Some((1, 0)));

        drop(lease1);
        // After lease1 drops, entry is cleaned up
        assert_eq!(lock.get_state("test_type", "actor1"), None);
    }
}
