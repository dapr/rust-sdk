use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use crate::error::Error;

pub type BoxedActor = Arc<Box<dyn Actor + Sync + Send>>;
pub type Callback = fn() -> BoxedActor;
pub type DynActorManager = Arc<dyn ActorManager + Send + Sync>;

pub trait Actor {
    fn invoke(&self, method: &str, args: &str) -> String;
    fn register(manager: &mut dyn ActorManager)
    where
        Self: Sized;
}

pub trait ActorManager {
    fn registered_actors(&self) -> String;
    fn invoke(&self, actor_type: &str, _actor_id: &str, method: &str) -> Result<String, Error>;
    fn register(&mut self, name: &str, callback: Callback);
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActorConfig {
    pub entities: Vec<String>,
    pub actor_idle_timeout: DaprDuration,
    pub actor_scan_interval: DaprDuration,
    pub drain_ongoing_call_timeout: DaprDuration,
    pub drain_rebalanced_actors: bool,
}

impl ActorConfig {
    fn new(entities: Vec<String>) -> Self {
        Self {
            entities,
            actor_idle_timeout: DaprDuration::from(std::time::Duration::from_secs(3600)),
            actor_scan_interval: DaprDuration::from(std::time::Duration::from_secs(30)),
            drain_ongoing_call_timeout: DaprDuration::from(std::time::Duration::from_secs(30)),
            drain_rebalanced_actors: true,
        }
    }
}


pub struct ActorManagerImpl {
    pub registered_types: Arc<RwLock<HashMap<String, Callback>>>,
    pub activated_actors: Arc<RwLock<HashMap<String, BoxedActor>>>,
}

impl ActorManagerImpl {
    pub fn new() -> Self {
        Self {
            registered_types: Arc::new(RwLock::new(HashMap::new())),
            activated_actors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ActorManager for ActorManagerImpl {
    fn registered_actors(&self) -> String {
        let types_map = self.registered_types.read().unwrap();
        let types = types_map.keys().map(|k| k.clone()).collect::<Vec<String>>();
        let config = ActorConfig::new(types);
        let result = serde_json::to_string(&config).unwrap();
        result
    }
    fn invoke(&self, actor_type: &str, actor_id: &str, method: &str) -> Result<String, Error> {
        let actor_type = actor_type.to_lowercase();
        let maybe_actor = { self.activated_actors.read().unwrap().get(actor_id).cloned() };
        let actor = match maybe_actor {
            Some(actor) => actor.clone(),
            None => {
                let type_map = self.registered_types.read()?;
                let creator = type_map.get(&actor_type).ok_or(crate::error::Error::ActorErrorType::NoSuchMethod)?;
                let actor = creator();
                {
                    self.activated_actors
                        .write()
                        .unwrap()
                        .insert(actor_id.to_string(), actor.clone());
                }
                actor
            }
        };

        Ok(actor.invoke(method, ""))
    }

    fn register(&mut self, name: &str, callback: Callback) {
        let mut types_map = self.registered_types.write().unwrap();
        types_map.insert(name.to_lowercase().to_string(), callback);
    }
}

#[derive(Debug)]
pub struct DaprDuration {
    duration: Duration,
}

impl DaprDuration {
    pub fn from(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Serialize for DaprDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // write in format expected by Dapr, it only accepts h, m, s, ms, us(micro), ns
        let dapr_time_str = match self.duration.as_millis() {
            0 => "0s".to_string(),
            millis => {
                const ONE_HOUR: u128 = 1000 * 3600;
                const ONE_MIN: u128 = 1000 * 60;
                const ONE_SEC: u128 = 1000;
                let hours = millis / ONE_HOUR;
                let mins = (millis % ONE_HOUR) / ONE_MIN;
                let seconds = (millis % ONE_MIN) / ONE_SEC;
                let millis = millis % ONE_SEC;

                format!("{}h{}m{}s{}ms", hours, mins, seconds, millis)
            }
        };

        serializer.serialize_str(&dapr_time_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn works_with_content() {
        let hours = 45;
        let minutes = 32;
        let seconds = 23;
        let millis = 234;
        let duration = DaprDuration::from(std::time::Duration::from_secs_f64(
            hours as f64 * 3600f64
                + minutes as f64 * 60f64
                + seconds as f64
                + millis as f64 / 1000.0,
        ));
        let expected = format!("\"{}h {}m {}s {}ms\"", hours, minutes, seconds, millis);
        assert_eq!(expected, serde_json::to_string(&duration).unwrap());
    }

    #[test]
    fn works_with_empty_duration() {
        let duration = DaprDuration::from(std::time::Duration::from_secs(0));
        let expected = format!("\"0s\"");
        assert_eq!(expected, serde_json::to_string(&duration).unwrap());
    }
}
