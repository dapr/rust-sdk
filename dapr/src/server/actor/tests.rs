#[cfg(test)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::body::{Body as AxumBody, to_bytes as axum_to_bytes};
use axum::http::Request as AxumRequest;
use axum::{Json, Router};
use dapr::server::{
    DaprHttpServer,
    actor::{Actor, ActorError, runtime::ActorTypeRegistration},
};
use dapr_macros::actor;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MyResponse {
    pub actor_id: String,
    pub name: String,
    pub available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub name: String,
}

#[actor]
struct MyActor {
    id: String,
}

#[async_trait]
impl Actor for MyActor {
    async fn on_activate(&self) -> Result<(), ActorError> {
        TEST_STATE.increment_on_activate(&self.id).await;
        Ok(())
    }

    async fn on_deactivate(&self) -> Result<(), ActorError> {
        TEST_STATE.increment_on_deactivate(&self.id).await;
        Ok(())
    }

    async fn on_reminder(&self, _reminder_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }

    async fn on_timer(&self, _timer_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }
}

impl MyActor {
    async fn do_stuff(&self, Json(req): Json<MyRequest>) -> Json<MyResponse> {
        Json(MyResponse {
            actor_id: self.id.clone(),
            name: req.name,
            available: true,
        })
    }
}

#[tokio::test]
async fn test_actor_invoke() {
    let dapr_port = get_available_port().await.unwrap();

    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        let address = format!("127.0.0.1:{dapr_port}");
        let listener = TcpListener::bind(address).await.unwrap();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });
    tokio::task::yield_now().await;

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port).await;

    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<MyActor>(
                "MyActor",
                Box::new(|_actor_type, actor_id, _context| {
                    Arc::new(MyActor {
                        id: actor_id.to_string(),
                    })
                }),
            )
            .register_method("do_stuff", MyActor::do_stuff),
        )
        .await;

    let actor_id = Uuid::new_v4().to_string();

    let app = dapr_server.build_test_router().await;

    // First invoke
    let req_body = serde_json::to_string(&json!({ "name": "foo" })).unwrap();
    let req = AxumRequest::builder()
        .method("PUT")
        .uri(format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .header("content-type", "application/json")
        .body(AxumBody::from(req_body))
        .unwrap();

    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let bytes = axum_to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let resp_json: MyResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        resp_json,
        MyResponse {
            actor_id: actor_id.clone(),
            name: "foo".to_string(),
            available: true,
        }
    );

    assert_eq!(
        TEST_STATE
            .get_actor_state(&actor_id)
            .await
            .unwrap()
            .on_activate,
        1
    );

    // Second invoke (should not call on_activate again)
    let req_body2 = serde_json::to_string(&json!({ "name": "foo" })).unwrap();
    let req2 = AxumRequest::builder()
        .method("PUT")
        .uri(format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .header("content-type", "application/json")
        .body(AxumBody::from(req_body2))
        .unwrap();

    let resp2 = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req2)
        .await
        .unwrap();
    assert!(resp2.status().is_success());

    assert_eq!(
        TEST_STATE
            .get_actor_state(&actor_id)
            .await
            .unwrap()
            .on_activate,
        1
    );

    fake_sidecar.abort();
}

#[tokio::test]
async fn test_actor_deactivate() {
    let dapr_port = get_available_port().await.unwrap();

    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        let address = format!("127.0.0.1:{dapr_port}");
        let listener = TcpListener::bind(address).await.unwrap();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });
    tokio::task::yield_now().await;

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port).await;

    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<MyActor>(
                "MyActor",
                Box::new(|_actor_type, actor_id, _context| {
                    Arc::new(MyActor {
                        id: actor_id.to_string(),
                    })
                }),
            )
            .register_method("do_stuff", MyActor::do_stuff),
        )
        .await;

    let app = dapr_server.build_test_router().await;

    let actor_id = Uuid::new_v4().to_string();

    // Invoke to activate
    let req_body = serde_json::to_string(&json!({ "name": "foo" })).unwrap();
    let req = AxumRequest::builder()
        .method("PUT")
        .uri(format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .header("content-type", "application/json")
        .body(AxumBody::from(req_body))
        .unwrap();
    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Deactivate - first should succeed
    let delete_req = AxumRequest::builder()
        .method("DELETE")
        .uri(format!("/actors/MyActor/{actor_id}"))
        .body(AxumBody::empty())
        .unwrap();
    let delete_resp1 =
        tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), delete_req)
            .await
            .unwrap();
    assert!(delete_resp1.status().is_success());

    // Deactivate again - should be not found
    let delete_req2 = AxumRequest::builder()
        .method("DELETE")
        .uri(format!("/actors/MyActor/{actor_id}"))
        .body(AxumBody::empty())
        .unwrap();
    let delete_resp2 =
        tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), delete_req2)
            .await
            .unwrap();
    assert_eq!(delete_resp2.status().as_u16(), 404);

    assert_eq!(
        TEST_STATE
            .get_actor_state(&actor_id)
            .await
            .unwrap()
            .on_deactivate,
        1
    );

    fake_sidecar.abort();
}

#[derive(Clone, Debug)]
struct TestActorState {
    pub on_activate: u32,
    pub on_deactivate: u32,
}

struct TestState {
    actors: Arc<Mutex<HashMap<String, TestActorState>>>,
}

impl TestState {
    pub fn new() -> Self {
        TestState {
            actors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_actor_state(&self, actor_id: &str) -> Option<TestActorState> {
        let actors = self.actors.lock().await;
        actors.get(actor_id).cloned()
    }

    pub async fn increment_on_activate(&self, actor_id: &str) {
        let mut actors = self.actors.lock().await;
        let actor_state = actors
            .entry(actor_id.to_string())
            .or_insert(TestActorState {
                on_activate: 0,
                on_deactivate: 0,
            });
        actor_state.on_activate += 1;
    }

    pub async fn increment_on_deactivate(&self, actor_id: &str) {
        let mut actors = self.actors.lock().await;
        let actor_state = actors
            .entry(actor_id.to_string())
            .or_insert(TestActorState {
                on_activate: 0,
                on_deactivate: 0,
            });
        actor_state.on_deactivate += 1;
    }
}

static TEST_STATE: Lazy<TestState> = Lazy::new(TestState::new);

async fn get_available_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").await.ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener);
    Some(port)
}

#[tokio::test]
async fn test_actor_server_enforces_app_api_token() {
    use dapr::client::AppApiTokenLayer;

    let dapr_port = get_available_port().await.unwrap();

    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        let address = format!("127.0.0.1:{dapr_port}");
        let listener = TcpListener::bind(address).await.unwrap();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });
    tokio::task::yield_now().await;

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port)
        .await
        .with_app_api_token_layer(AppApiTokenLayer::new(Some("expected".to_string())));

    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<MyActor>(
                "MyActor",
                Box::new(|_actor_type, actor_id, _context| {
                    Arc::new(MyActor {
                        id: actor_id.to_string(),
                    })
                }),
            )
            .register_method("do_stuff", MyActor::do_stuff),
        )
        .await;

    let app = dapr_server.build_test_router().await;
    let actor_id = Uuid::new_v4().to_string();
    let path = format!("/actors/MyActor/{actor_id}/method/do_stuff");
    let body = serde_json::to_string(&json!({ "name": "foo" })).unwrap();

    // (1) Missing token → 401.
    let req = AxumRequest::builder()
        .method("PUT")
        .uri(&path)
        .header("content-type", "application/json")
        .body(AxumBody::from(body.clone()))
        .unwrap();
    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);

    // (2) Wrong token → 401.
    let req = AxumRequest::builder()
        .method("PUT")
        .uri(&path)
        .header("content-type", "application/json")
        .header("dapr-api-token", "wrong")
        .body(AxumBody::from(body.clone()))
        .unwrap();
    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);

    // (3) Correct token → 200.
    let req = AxumRequest::builder()
        .method("PUT")
        .uri(&path)
        .header("content-type", "application/json")
        .header("dapr-api-token", "expected")
        .body(AxumBody::from(body))
        .unwrap();
    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app.clone(), req)
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // (4) /healthz is always open even with enforcement enabled.
    let req = AxumRequest::builder()
        .method("GET")
        .uri("/healthz")
        .body(AxumBody::empty())
        .unwrap();
    let resp = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app, req)
        .await
        .unwrap();
    assert!(resp.status().is_success());

    fake_sidecar.abort();
}

#[derive(Clone, Default)]
struct ActorConcurrencyStats {
    active_turns: Arc<std::sync::atomic::AtomicUsize>,
    max_overlap: Arc<std::sync::atomic::AtomicUsize>,
    completed_slow: Arc<std::sync::atomic::AtomicBool>,
}

#[actor]
struct ConcurrencyActor {
    id: String,
    stats: ActorConcurrencyStats,
}

#[async_trait]
impl Actor for ConcurrencyActor {
    async fn on_activate(&self) -> Result<(), ActorError> {
        Ok(())
    }
    async fn on_deactivate(&self) -> Result<(), ActorError> {
        Ok(())
    }
    async fn on_reminder(&self, _reminder_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }
    async fn on_timer(&self, _timer_name: &str, _data: Vec<u8>) -> Result<(), ActorError> {
        Ok(())
    }
}

impl ConcurrencyActor {
    async fn slow_call(&self, Json(_req): Json<MyRequest>) -> Json<MyResponse> {
        let current = self
            .stats
            .active_turns
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        self.stats
            .max_overlap
            .fetch_max(current, std::sync::atomic::Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.stats
            .active_turns
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        self.stats
            .completed_slow
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Json(MyResponse {
            actor_id: self.id.clone(),
            name: "slow".to_string(),
            available: true,
        })
    }

    async fn fast_call(&self, Json(_req): Json<MyRequest>) -> Json<MyResponse> {
        let current = self
            .stats
            .active_turns
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        self.stats
            .max_overlap
            .fetch_max(current, std::sync::atomic::Ordering::SeqCst);
        self.stats
            .active_turns
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        Json(MyResponse {
            actor_id: self.id.clone(),
            name: "fast".to_string(),
            available: true,
        })
    }
}

#[tokio::test]
async fn test_actor_turn_concurrency_serializes_calls() {
    let stats = ActorConcurrencyStats::default();
    let stats_clone = stats.clone();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dapr_port = listener.local_addr().unwrap().port();
    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port).await;
    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<ConcurrencyActor>(
                "ConcurrencyActor",
                Box::new(move |_actor_type, actor_id, _context| {
                    Arc::new(ConcurrencyActor {
                        id: actor_id.to_string(),
                        stats: stats_clone.clone(),
                    })
                }),
            )
            .register_method("slow_call", ConcurrencyActor::slow_call),
        )
        .await;

    let actor_id = Uuid::new_v4().to_string();
    let app = dapr_server.build_test_router().await;

    let req_body = serde_json::to_string(&json!({ "name": "test" })).unwrap();
    let app1 = app.clone();
    let uri1 = format!("/actors/ConcurrencyActor/{actor_id}/method/slow_call");
    let b1 = req_body.clone();
    let h1 = tokio::spawn(async move {
        let req = AxumRequest::builder()
            .method("PUT")
            .uri(uri1)
            .header("content-type", "application/json")
            .body(AxumBody::from(b1))
            .unwrap();
        tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app1, req)
            .await
            .unwrap()
    });

    let app2 = app.clone();
    let uri2 = format!("/actors/ConcurrencyActor/{actor_id}/method/slow_call");
    let b2 = req_body.clone();
    let h2 = tokio::spawn(async move {
        let req = AxumRequest::builder()
            .method("PUT")
            .uri(uri2)
            .header("content-type", "application/json")
            .body(AxumBody::from(b2))
            .unwrap();
        tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app2, req)
            .await
            .unwrap()
    });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.unwrap().status().is_success());
    assert!(r2.unwrap().status().is_success());

    assert_eq!(
        stats.max_overlap.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "Concurrent calls to the same actor ID must be strictly serialized"
    );

    fake_sidecar.abort();
}

#[tokio::test]
async fn test_actor_cancelled_call_holds_turn_until_finished() {
    let stats = ActorConcurrencyStats::default();
    let stats_clone = stats.clone();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dapr_port = listener.local_addr().unwrap().port();
    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port).await;
    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<ConcurrencyActor>(
                "ConcurrencyActor",
                Box::new(move |_actor_type, actor_id, _context| {
                    Arc::new(ConcurrencyActor {
                        id: actor_id.to_string(),
                        stats: stats_clone.clone(),
                    })
                }),
            )
            .register_method("slow_call", ConcurrencyActor::slow_call)
            .register_method("fast_call", ConcurrencyActor::fast_call),
        )
        .await;

    let actor_id = Uuid::new_v4().to_string();
    let app = dapr_server.build_test_router().await;

    // (1) Start slow call, but abort the caller's request future early (simulating cancellation)
    let app1 = app.clone();
    let uri1 = format!("/actors/ConcurrencyActor/{actor_id}/method/slow_call");
    let req_body = serde_json::to_string(&json!({ "name": "test" })).unwrap();
    let b1 = req_body.clone();
    let h1 = tokio::spawn(async move {
        let req = AxumRequest::builder()
            .method("PUT")
            .uri(uri1)
            .header("content-type", "application/json")
            .body(AxumBody::from(b1))
            .unwrap();
        _ = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app1, req).await;
    });

    // Abort the caller's HTTP request future while slow_call is still executing
    tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    h1.abort();

    // Verify slow_call is still running and hasn't finished yet
    assert!(
        !stats
            .completed_slow
            .load(std::sync::atomic::Ordering::SeqCst),
        "slow_call should still be running in the background"
    );

    // (2) Immediately send fast_call for the same actor ID.
    // fast_call must wait for slow_call to finish its turn before starting.
    let app2 = app.clone();
    let uri2 = format!("/actors/ConcurrencyActor/{actor_id}/method/fast_call");
    let req2 = AxumRequest::builder()
        .method("PUT")
        .uri(uri2)
        .header("content-type", "application/json")
        .body(AxumBody::from(req_body))
        .unwrap();

    let resp2 = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app2, req2)
        .await
        .unwrap();
    assert!(resp2.status().is_success());

    // By the time fast_call completed, slow_call must have completed without overlap
    assert!(
        stats
            .completed_slow
            .load(std::sync::atomic::Ordering::SeqCst),
        "slow_call must have completed before fast_call finished"
    );
    assert_eq!(
        stats.max_overlap.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "Cancelled call must hold the turn lock so subsequent calls never overlap"
    );

    fake_sidecar.abort();
}

#[tokio::test]
async fn test_actor_reentrancy_over_http() {
    let stats = ActorConcurrencyStats::default();
    let stats_clone = stats.clone();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dapr_port = listener.local_addr().unwrap().port();
    let fake_sidecar = tokio::spawn(async move {
        let sidecar = Router::new();
        _ = axum::serve(listener, sidecar.into_make_service()).await;
    });

    let mut dapr_server = DaprHttpServer::with_dapr_port(dapr_port).await;
    dapr_server
        .register_actor(
            ActorTypeRegistration::new::<ConcurrencyActor>(
                "ConcurrencyActor",
                Box::new(move |_actor_type, actor_id, _context| {
                    Arc::new(ConcurrencyActor {
                        id: actor_id.to_string(),
                        stats: stats_clone.clone(),
                    })
                }),
            )
            .with_reentrancy(dapr::server::models::ReentrancyConfig {
                enabled: true,
                max_stack_depth: None,
            })
            .register_method("slow_call", ConcurrencyActor::slow_call)
            .register_method("fast_call", ConcurrencyActor::fast_call),
        )
        .await;

    let actor_id = Uuid::new_v4().to_string();
    let app = dapr_server.build_test_router().await;

    let req_body = serde_json::to_string(&json!({ "name": "test" })).unwrap();

    // Call 1 with reentrancy ID "chain-1"
    let app1 = app.clone();
    let uri1 = format!("/actors/ConcurrencyActor/{actor_id}/method/slow_call");
    let b1 = req_body.clone();
    let h1 = tokio::spawn(async move {
        let req = AxumRequest::builder()
            .method("PUT")
            .uri(uri1)
            .header("content-type", "application/json")
            .header("dapr-reentrancy-id", "chain-1")
            .body(AxumBody::from(b1))
            .unwrap();
        tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app1, req)
            .await
            .unwrap()
    });

    // Wait for Call 1 to start and allow reentrancy
    tokio::time::sleep(std::time::Duration::from_millis(15)).await;

    // Call 2 with the same reentrancy ID "chain-1" can reenter
    let app2 = app.clone();
    let uri2 = format!("/actors/ConcurrencyActor/{actor_id}/method/fast_call");
    let b2 = req_body.clone();
    let req2 = AxumRequest::builder()
        .method("PUT")
        .uri(uri2)
        .header("content-type", "application/json")
        .header("dapr-reentrancy-id", "chain-1")
        .body(AxumBody::from(b2))
        .unwrap();

    let resp2 = tower::util::ServiceExt::<AxumRequest<AxumBody>>::oneshot(app2, req2)
        .await
        .unwrap();
    assert!(resp2.status().is_success());

    // Both executed within the reentrant chain
    assert_eq!(
        stats.max_overlap.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "Reentrant call with matching Dapr-Reentrancy-Id must be allowed to enter"
    );

    let resp1 = h1.await.unwrap();
    assert!(resp1.status().is_success());

    fake_sidecar.abort();
}
