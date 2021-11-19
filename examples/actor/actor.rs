use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::handler::Handler;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::{routing::get, AddExtensionLayer, Router};
use axum_debug::debug_handler;
use std::sync::Arc;

use dapr::actors::*;

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

    fn register(manager: &mut dyn ActorManager) {
        manager.register("HelloActor", || Arc::new(Box::new(HelloActor {})));
    }
}

#[debug_handler]
async fn invoke_method(
    Path((actor_type, actor_id, method_name)): Path<(String, String, String)>,
    Extension(actor_manager): Extension<DynActorManager>,
) -> String {
    let response = actor_manager.invoke(&actor_type, &actor_id, &method_name);
    match response {
        None => "".into(), //TODO: Error
        Some(result) => result,
    }
}

#[debug_handler]
async fn get_registered_actors(Extension(actor_manager): Extension<DynActorManager>) -> String {
    actor_manager.registered_actors()
}
#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug")
    }
    let mut manager = ActorManagerImpl::new();
    HelloActor::register(&mut manager);

    println!("trying to say something");
    match manager.invoke("HelloActor", "1", "say_hi") {
        Ok(response) => println!("Say Hi: {}", response),
        Err(err) => eprint!("Could not invoke actor ☹️ {:?}", err),
    }

    println!("trying to say something again");
    match manager.invoke("HelloActor", "2", "say_ho") {
        Ok(response) => println!("Say ho: {}", response),
        Err(err) => eprint!("Could not invoke actor ☹️ {:?}", err),
    }

    let manager: DynActorManager = Arc::new(manager);

    let app = Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/dapr/config", get(get_registered_actors))
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route(
            "/actors/:actor_type/:actor_id/method/:method_name",
            get(invoke_method).put(invoke_method),
        )
        .layer(AddExtensionLayer::new(manager.clone()));
    let app = app.fallback(handler_404.into_service());
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3500".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler_404(request: Request<Body>) -> impl IntoResponse {
    println!("404-ing!! {:?}", request);
    (StatusCode::NOT_FOUND, "nothing to see here")
}
