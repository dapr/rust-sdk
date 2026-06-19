use axum::{
    Json, Router,
    extract::{OriginalUri, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
};
use futures::{Future, FutureExt};
use std::{pin::Pin, sync::Arc, time::Duration};
use tokio::net::TcpListener;

use super::super::client::{
    AppApiTokenLayer, TonicClient,
    config::{DAPR_GRPC_PORT_ENV, DEFAULT_DAPR_GRPC_PORT},
};
use super::actor::runtime::{ActorRuntime, ActorTypeRegistration};

/// The Dapr HTTP server.
///
/// Supports Http callbacks from the Dapr sidecar.
///
/// # Example:
/// ```ignore
/// # use std::sync::Arc;
/// # use dapr::server::actor::{context_client::ActorContextClient, Actor, ActorError, ActorFactory, runtime::ActorTypeRegistration};
/// # use dapr::server::utils::DaprJson;
/// # use dapr::actor;
/// # use axum::{Json, Router};
/// # use serde::{Deserialize, Serialize};
/// # #[actor]
/// # struct MyActor {
/// #     id: String,
/// #     client: ActorContextClient,
/// # }
/// #
/// # #[async_trait::async_trait]
/// # impl Actor for MyActor {
/// #    async fn on_activate(&self) -> Result<(), ActorError> {
/// #        todo!()
/// #    }
/// #    async fn on_deactivate(&self) -> Result<(), ActorError> {
/// #         todo!()
/// #    }
/// #    async fn on_reminder(&self, reminder_name: &str, data: Vec<u8>) -> Result<(), ActorError> {
/// #         todo!()
/// #    }
/// #    async fn on_timer(&self, timer_name: &str, data: Vec<u8>) -> Result<(), ActorError> {
/// #         todo!()
/// #    }
/// # }
/// ##[derive(Serialize, Deserialize)]
/// pub struct MyRequest {
/// pub name: String,
/// }
///
///##[derive(Serialize, Deserialize)]
///pub struct MyResponse {
///    pub available: bool,
///}   
///
///impl MyActor {
///    fn do_stuff(&self, DaprJson(data): DaprJson<MyRequest>) -> Json<MyResponse> {        
///        println!("doing stuff with {}", data.name);        
///        Json(MyResponse {
///            available: true
///        })
///    }    
///}
/// # async fn main_async() {
/// let mut dapr_server = dapr::server::DaprHttpServer::new().await;
///     
/// dapr_server.register_actor(ActorTypeRegistration::new::<MyActor>("MyActor", Box::new(|_actor_type, actor_id, context| {
///     Arc::new(MyActor {
///         id: actor_id.to_string(),
///         client: context,
///     })}))
///     .register_method("do_stuff", MyActor::do_stuff))
///     .await;
///
/// dapr_server.start(None).await;
/// # }
/// ```
pub struct DaprHttpServer {
    actor_runtime: Arc<ActorRuntime>,
    shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    app_api_token_layer: AppApiTokenLayer,
}

impl DaprHttpServer {
    /// Creates a new instance of the Dapr HTTP server with default options.
    ///
    /// # Panics
    ///
    /// This function panics if the Dapr Sidecar cannot be reached!
    /// For a non-panicking version that allows you to handle any errors yourself, see:
    /// [DaprHttpServer::try_new_with_dapr_port]
    pub async fn new() -> Self {
        Self::with_dapr_port(dapr_grpc_port_from_env()).await
    }

    /// Creates a new instance of the Dapr HTTP server that connects to the Dapr sidecar on the
    /// given dapr_port.
    ///
    /// # Panics
    ///
    /// This function panics if the Dapr Sidecar cannot be reached!
    /// For a non-panicking version that allows you to handle any errors yourself, see:
    /// [DaprHttpServer::try_new_with_dapr_port]
    pub async fn with_dapr_port(dapr_port: u16) -> Self {
        match Self::try_new_with_dapr_port(dapr_port).await {
            Ok(c) => c,
            Err(err) => panic!("failed to connect to dapr: {err}"),
        }
    }

    /// Creates a new instance of the Dapr HTTP server that connects to the Dapr sidecar on the
    /// given dapr_port.
    ///
    /// In contrast to the other functions that create a DaprHttpServer, this function does
    /// not panic, but instead returns a Result.
    ///
    /// The connection uses exponential-backoff retries in case the sidecar
    /// is not yet ready.
    pub async fn try_new_with_dapr_port(
        dapr_port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dapr_addr = format!("http://127.0.0.1:{dapr_port}");

        let cc = Self::connect_with_retry(&dapr_addr).await?;
        let rt = ActorRuntime::new(cc);

        Ok(DaprHttpServer {
            actor_runtime: Arc::new(rt),
            shutdown_signal: None,
            // Reads `APP_API_TOKEN` from the environment. Permissive if unset.
            app_api_token_layer: AppApiTokenLayer::from_env(),
        })
    }

    /// Connect to the Dapr sidecar with exponential-backoff retries.
    async fn connect_with_retry(
        dapr_addr: &str,
    ) -> Result<TonicClient, Box<dyn std::error::Error>> {
        const MAX_RETRIES: u32 = 10;
        let mut retry_delay = Duration::from_millis(500);
        let max_delay = Duration::from_secs(2);

        let mut last_err = None;
        for attempt in 1..=MAX_RETRIES {
            match TonicClient::connect(dapr_addr.to_string()).await {
                Ok(client) => return Ok(client),
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        log::warn!(
                            "Dapr sidecar not ready (attempt {attempt}/{MAX_RETRIES}), \
                             retrying in {retry_delay:?}…"
                        );
                        tokio::time::sleep(retry_delay).await;
                        retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                    }
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap().into())
    }

    /// Override the [`AppApiTokenLayer`] used to authenticate inbound
    /// requests from the Dapr sidecar.
    ///
    /// By default, the server reads `APP_API_TOKEN` from the environment;
    /// when the env var is unset the layer is a no-op. Use this method to
    /// supply an explicit expected token (e.g. read from a secret store) or
    /// to disable enforcement entirely with `AppApiTokenLayer::new(None)`.
    ///
    /// The `/healthz` endpoint is always exempt from the token check so
    /// that infrastructure liveness probes continue to work.
    /// @mikeee: reconsider this exemption in the future, as it may have security implications.
    pub fn with_app_api_token_layer(mut self, layer: AppApiTokenLayer) -> Self {
        self.app_api_token_layer = layer;
        self
    }

    pub fn with_graceful_shutdown<F>(self, signal: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        DaprHttpServer {
            shutdown_signal: Some(signal.boxed()),
            ..self
        }
    }

    /// Registers an actor type with the Dapr runtime.
    ///
    /// # Arguments:
    /// * `registration` - The [ActorTypeRegistration] struct, carries the methods that can be invoked on it and the factory to create instances of it.
    pub async fn register_actor(&self, registration: ActorTypeRegistration) {
        self.actor_runtime.register_actor(registration).await;
    }

    /// Starts the Dapr HTTP server.
    ///
    /// # Arguments:
    /// * `port` - The port to listen on. If not specified, the APP_PORT environment variable will be used. If that is not specified, 8080 will be used.
    pub async fn start(&mut self, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router().await;

        let default_port: u16 = std::env::var("APP_PORT")
            .unwrap_or(String::from("8080"))
            .parse()
            .unwrap_or(8080);

        let address = format!("127.0.0.1:{}", port.unwrap_or(default_port));
        let listener = TcpListener::bind(address).await?;

        let server = axum::serve(listener, app.into_make_service());

        let final_result = match self.shutdown_signal.take() {
            Some(signal) => {
                server
                    .with_graceful_shutdown(async move {
                        signal.await;
                    })
                    .await
            }
            None => server.await,
        };

        self.actor_runtime.deactivate_all().await;

        Ok(final_result?)
    }

    pub async fn build_test_router(&mut self) -> Router {
        self.build_router().await
    }

    async fn build_router(&mut self) -> Router {
        let rt = self.actor_runtime.clone();

        // All actor / config endpoints — protected by the APP_API_TOKEN
        // layer when configured. `/healthz` is intentionally excluded so
        // that infrastructure liveness probes don't need to present the
        // token.
        let protected = Router::new()
            .route(
                "/dapr/config",
                get(registered_actors).with_state(rt.clone()),
            )
            .route(
                "/actors/:actor_type/:actor_id",
                delete(deactivate_actor).with_state(rt.clone()),
            )
            .route(
                "/actors/:actor_type/:actor_id/method/remind/:reminder_name",
                put(invoke_reminder).with_state(rt.clone()),
            )
            .route(
                "/actors/:actor_type/:actor_id/method/timer/:timer_name",
                put(invoke_timer).with_state(rt.clone()),
            );

        let protected = self
            .actor_runtime
            .configure_method_routes(protected, rt.clone())
            .await
            .layer(self.app_api_token_layer.clone());

        Router::new()
            .route("/healthz", get(health_check))
            .merge(protected)
            .fallback(fallback_handler)
    }
}

async fn fallback_handler(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    log::warn!("Returning 404 for request: {uri}");
    (
        StatusCode::NOT_FOUND,
        format!("The URI '{uri}' could not be found!"),
    )
}

async fn health_check() -> impl IntoResponse {
    log::debug!("recieved health check request");
    StatusCode::OK
}

async fn registered_actors(State(runtime): State<Arc<ActorRuntime>>) -> impl IntoResponse {
    log::debug!("daprd requested registered actors");
    let ra = runtime.list_registered_actors().await;
    let result = super::models::RegisteredActorsResponse { entities: ra };

    Json(result)
}

async fn deactivate_actor(
    State(runtime): State<Arc<ActorRuntime>>,
    Path((actor_type, actor_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match runtime.deactivate_actor(&actor_type, &actor_id).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log::error!("invoke_actor: {err:?}");
            match err {
                super::actor::ActorError::ActorNotFound => StatusCode::NOT_FOUND,
                _ => {
                    log::error!("deactivate_actor: {err:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }
}

async fn invoke_reminder(
    State(runtime): State<Arc<ActorRuntime>>,
    Path((actor_type, actor_id, reminder_name)): Path<(String, String, String)>,
    Json(payload): Json<ReminderPayload>,
) -> impl IntoResponse {
    log::debug!("invoke_reminder: {actor_type} {actor_id} {reminder_name} {payload:?}");

    match runtime
        .invoke_reminder(
            &actor_type,
            &actor_id,
            &reminder_name,
            payload.data.unwrap_or_default().into_bytes(),
        )
        .await
    {
        Ok(_output) => StatusCode::OK,
        Err(err) => {
            log::error!("invoke_actor: {err:?}");
            match err {
                super::actor::ActorError::ActorNotFound => StatusCode::NOT_FOUND,
                _ => {
                    log::error!("invoke_reminder: {err:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }
}

async fn invoke_timer(
    State(runtime): State<Arc<ActorRuntime>>,
    Path((actor_type, actor_id, timer_name)): Path<(String, String, String)>,
    Json(payload): Json<TimerPayload>,
) -> impl IntoResponse {
    log::debug!("invoke_timer: {actor_type} {actor_id} {timer_name}, {payload:?}");

    match runtime
        .invoke_timer(
            &actor_type,
            &actor_id,
            &timer_name,
            payload.data.unwrap_or_default().into_bytes(),
        )
        .await
    {
        Ok(_output) => StatusCode::OK,
        Err(err) => {
            log::error!("invoke_actor: {err:?}");
            match err {
                super::actor::ActorError::ActorNotFound => StatusCode::NOT_FOUND,
                _ => {
                    log::error!("invoke_timer: {err:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct ReminderPayload {
    data: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct TimerPayload {
    data: Option<String>,
}

fn dapr_grpc_port_from_env() -> u16 {
    std::env::var(DAPR_GRPC_PORT_ENV)
        .unwrap_or_else(|_| DEFAULT_DAPR_GRPC_PORT.to_string())
        .parse()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(key: &str, value: Option<&str>, f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var(key).ok();
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        match prev {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }

        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn http_server_defaults_to_standard_dapr_grpc_port() {
        with_env(DAPR_GRPC_PORT_ENV, None, || {
            assert_eq!(dapr_grpc_port_from_env(), DEFAULT_DAPR_GRPC_PORT);
        });
    }

    #[test]
    fn http_server_honors_dapr_grpc_port_override() {
        with_env(DAPR_GRPC_PORT_ENV, Some("12345"), || {
            assert_eq!(dapr_grpc_port_from_env(), 12345);
        });
    }
}
