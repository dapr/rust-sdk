#[cfg(test)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{Json, Router};
use axum_test::TestServer;
use dapr::server::{
    actor::{runtime::ActorTypeRegistration, Actor, ActorError},
    DaprHttpServer,
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
    let server = TestServer::new(app.into_make_service()).unwrap();

    let invoke_resp = server
        .put(&format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .json(&json!({ "name": "foo" }))
        .await;
    invoke_resp.assert_status_ok();

    invoke_resp.assert_json(&MyResponse {
        actor_id: actor_id.clone(),
        name: "foo".to_string(),
        available: true,
    });

    assert_eq!(
        TEST_STATE
            .get_actor_state(&actor_id)
            .await
            .unwrap()
            .on_activate,
        1
    );

    let invoke_resp2 = server
        .put(&format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .json(&json!({ "name": "foo" }))
        .await;
    invoke_resp2.assert_status_ok();

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
    let server = TestServer::new(app.into_make_service()).unwrap();

    let actor_id = Uuid::new_v4().to_string();

    let invoke_resp = server
        .put(&format!("/actors/MyActor/{actor_id}/method/do_stuff"))
        .json(&json!({ "name": "foo" }))
        .await;
    invoke_resp.assert_status_ok();

    let deactivate_resp1 = server.delete(&format!("/actors/MyActor/{actor_id}")).await;
    deactivate_resp1.assert_status_ok();

    let deactivate_resp2 = server.delete(&format!("/actors/MyActor/{actor_id}")).await;
    deactivate_resp2.assert_status_not_found();

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
    for port in 8000..9000 {
        if TcpListener::bind(format!("127.0.0.1:{port}")).await.is_ok() {
            return Some(port);
        }
    }
    None
}
