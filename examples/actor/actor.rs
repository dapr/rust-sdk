use axum::{Json, Router, body, extract::rejection::JsonRejection, routing::get};
use serde::Serialize;
use std::collections::HashMap;
use dapr::actors::*;
use std::sync::{Arc, RwLock};
use axum::extract::Path;

pub trait Actor {
    fn invoke(&self, method: &str, args: &str) -> String;
    fn register(manager: &mut ActorManager)
    where
        Self: Sized;
}

struct HelloActor {}

impl HelloActor {
    pub fn say_hi(&self) -> String {
        "hi there".to_string()
    }

    pub fn say_ho(&self) -> String {
        "ho there".to_string()
    }
}

impl Actor for HelloActor {
    fn invoke(&self, method: &str, _args: &str) -> String {
        match method {
            "say_hi" => self.say_hi(),
            "say_ho" => self.say_ho(),
            _ => panic!("No such method"),
        }
    }

    fn register(manager: &mut ActorManager) {
        manager.register("HelloActor", || Arc::new(Box::new(HelloActor {})));
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActorConfig {
    pub actor_types: Vec<String>,
    pub actor_idle_timeout: DaprDuration,
    pub actor_scan_interval: DaprDuration,
    pub drain_ongoing_call_timeout: DaprDuration,
    pub drain_rebalanced_actors: bool,
}

impl ActorConfig{
    
    fn new(actor_types: Vec<String>) -> Self {
        Self {
            actor_types,
            actor_idle_timeout: DaprDuration::from(std::time::Duration::from_secs(3600)),
            actor_scan_interval: DaprDuration::from(std::time::Duration::from_secs(30)),
            drain_ongoing_call_timeout: DaprDuration::from(std::time::Duration::from_secs(30)),
            drain_rebalanced_actors: true,
        }
    }
}

pub struct ActorManager {
    pub registered_types: Arc<RwLock<HashMap<String, Callback>>>,
    pub activated_actors: Arc<RwLock<HashMap<String, BoxedActor>>>,
}

impl ActorManager {
    pub fn new() -> Self {
        Self {
            registered_types: Arc::new(RwLock::new(HashMap::new())),
            activated_actors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&mut self, name: &str, callback: Callback) {
        let mut types_map = self.registered_types.write().unwrap();
        types_map.insert(name.to_string(), callback);
    }

    pub fn registered_actors(&self) -> String {
        let types_map = self.registered_types.read().unwrap();
        let types = types_map.keys().map(|k| k.clone()).collect::<Vec<String>>();
        let config  = ActorConfig::new(types);
        serde_json::to_string(&config).unwrap()
    }

    pub fn invoke(&self, actor_type: &str, _actor_id: &str, method: &str) -> Option<String> {
        let maybe_actor = {
            self.activated_actors.read().unwrap().get(actor_type).cloned()
        };
        let actor = match maybe_actor {
            Some(actor) => {
                actor.clone()
            },
            None => {
                let type_map = self.registered_types.read().unwrap();
                let creator = type_map.get(actor_type).unwrap();                
                let actor = creator();
                {
                    self.activated_actors.write().unwrap().insert(actor_type.to_string(), actor.clone());
                }
                actor
            },
        };
        
        Some(actor.invoke(method, ""))
    }
}

type BoxedActor = Arc<Box<dyn Actor + Sync + Send>>;
type Callback = fn() -> BoxedActor;

async fn invoke_method (
    Path((actor_type, actor_id, method_name)) : Path<(String, String, String)>,
    body: Result<Json<Value>, JsonRejection>) -> String {

    let result = format!("type {} actor {} method {}", actor_type, actor_id, method_name);
    println!("{}", result);
    println!("{:?}", body);
    result
}

#[tokio::main]
async fn main() {
    let mut manager = ActorManager::new();
    HelloActor::register(&mut manager);

    println!("trying to say something");
    match manager.invoke("HelloActor", "", "say_hi") {
        Some(response) => println!("Say Hi: {}", response),
        None => eprint!("Could not invoke actor ☹️"),
    }

    println!("trying to say something again");
    match manager.invoke("HelloActor", "", "say_ho") {
        Some(response) => println!("Say ho: {}", response),
        None => eprint!("Could not invoke actor ☹️"),
    }

    //PUT http://localhost:<appPort>/actors/<actorType>/<actorId>/method/<methodName>

    let manager = Arc::new(manager);
    let app = Router::new()
        .route("/", get(||  async { "hello world"}))
        .route("/dapr/config", get(|| async move { manager.clone().registered_actors()}))
        .route("/actors/:actor_type/:actor_id/method/:method_name", put(invoke_method));
        // .route("/actors/:actor_type/:actor_id/method/:method_name", get(|Path((actor_type, actor_id, method_name)): Path<(String, String, String)>| async move { invoke_method(actor_type, actor_id, method_name, manager.clone()) } ));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3500".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
