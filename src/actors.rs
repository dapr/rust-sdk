use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::handler::Handler;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{AddExtensionLayer, Router};
use axum_debug::debug_handler;
use serde::Serialize;

use crate::daprduration::DaprDuration;
use crate::error::{ActorErrorType, Error};

pub type BoxedActor = Arc<Box<dyn Actor + Sync + Send>>;
pub type Callback = fn() -> BoxedActor;
pub type DynActorManager = Arc<dyn ActorManager + Send + Sync>;

pub trait Actor {
    fn invoke(&self, method: &str, args: &str) -> Result<String, Error>;
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
                let creator = type_map
                    .get(&actor_type)
                    .ok_or(ActorErrorType::NoSuchActorType)?;
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

        actor.invoke(method, "")
    }

    fn register(&mut self, name: &str, callback: Callback) {
        let mut types_map = self.registered_types.write().unwrap();
        types_map.insert(name.to_lowercase().to_string(), callback);
    }
}

impl IntoResponse for crate::error::Error {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        let (body, status_code) = match self {
            Error::ActorError(_internal) => (
                Body::from("something went wrong"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            _ => (
                Body::from("something else went wrong"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };

        Response::builder().status(status_code).body(body).unwrap()
    }
}

#[debug_handler]
async fn invoke_method(
    Path((actor_type, actor_id, method_name)): Path<(String, String, String)>,
    Extension(actor_manager): Extension<DynActorManager>,
) -> Result<String, Error> {
    actor_manager.invoke(&actor_type, &actor_id, &method_name)
}

#[debug_handler]
async fn get_registered_actors(Extension(actor_manager): Extension<DynActorManager>) -> String {
    actor_manager.registered_actors()
}

pub async fn serve<M>(manager: M, url: &str) -> Result<(), Error>
where
    M: ActorManager + Sync + Send + 'static,
{
    let manager: DynActorManager = Arc::new(manager);

    let app = Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/dapr/config", get(get_registered_actors))
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route(
            "/actors/:actor_type/:actor_id/method/:method_name",
            get(invoke_method).put(invoke_method).post(invoke_method),
        )
        .layer(AddExtensionLayer::new(manager.clone()));
    let app = app.fallback(handler_404.into_service());
    // run it with hyper on localhost:3000
    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .map_err(|_e| Error::from(ActorErrorType::HttpError))
}

async fn handler_404(request: Request<Body>) -> impl IntoResponse {
    println!("404-ing!! {:?}", request);
    (StatusCode::NOT_FOUND, "nothing to see here")
}
