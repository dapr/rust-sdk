//! Demonstrates wiring [`dapr::client::AppApiTokenLayer`] into a tonic
//! [`Server`] to enforce the `APP_API_TOKEN` env var on inbound requests
//! from the Dapr sidecar.
//!
//! When `APP_API_TOKEN` is set, the layer requires every incoming gRPC
//! request to carry a matching `dapr-api-token` metadata header — otherwise
//! the request is rejected with `Unauthenticated`. When the env var is
//! unset the layer is a no-op, so it is safe to install unconditionally.
//!
//! This example is designed to be run under `dapr run` with `APP_API_TOKEN`
//! set in the environment. The sidecar inherits the env var and injects the
//! matching `dapr-api-token` metadata on every callback to the app. The app
//! installs `AppApiTokenLayer::from_env()` on its tonic server and advertises
//! a single `cron` input binding (`probe`, defined in `./config/cron.yaml`).
//! The sidecar delivers the first cron tick to the app's `on_binding_event`,
//! which proves authenticated callbacks succeed end-to-end and triggers a
//! graceful shutdown.

use std::sync::Arc;

use dapr::appcallback::*;
use dapr::client::AppApiTokenLayer;
use dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer};
use tokio::sync::Notify;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

/// Minimal `AppCallback` impl that logs each authenticated sidecar call
/// and signals the server to shut down on the first one.
struct LoggingCallback {
    shutdown: Arc<Notify>,
}

impl LoggingCallback {
    fn signal_auth_ok(&self, method: &str) {
        println!("sidecar callback received: {method} (auth ok)");
        self.shutdown.notify_one();
    }
}

#[tonic::async_trait]
impl AppCallback for LoggingCallback {
    async fn on_invoke(
        &self,
        _request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        Err(Status::unimplemented("on_invoke"))
    }

    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        Ok(Response::new(ListTopicSubscriptionsResponse {
            subscriptions: vec![],
        }))
    }

    async fn on_topic_event(
        &self,
        _request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        Err(Status::unimplemented("on_topic_event"))
    }

    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        Ok(Response::new(ListInputBindingsResponse {
            bindings: vec!["probe".to_string()],
        }))
    }

    async fn on_binding_event(
        &self,
        request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        self.signal_auth_ok(&format!("on_binding_event({})", request.into_inner().name));
        Ok(Response::new(BindingEventResponse::default()))
    }

    async fn on_bulk_topic_event(
        &self,
        _request: Request<TopicEventBulkRequest>,
    ) -> Result<Response<TopicEventBulkResponse>, Status> {
        Err(Status::unimplemented("on_bulk_topic_event"))
    }

    async fn on_job_event(
        &self,
        _request: Request<JobEventRequest>,
    ) -> Result<Response<JobEventResponse>, Status> {
        Err(Status::unimplemented("on_job_event"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let shutdown = Arc::new(Notify::new());
    let callback = LoggingCallback {
        shutdown: shutdown.clone(),
    };

    println!("AppCallback server listening on {addr}");

    Server::builder()
        // `from_env()` reads `APP_API_TOKEN`. When unset, this is a no-op.
        .layer(AppApiTokenLayer::from_env())
        .add_service(AppCallbackServer::new(callback))
        .serve_with_shutdown(addr, async move { shutdown.notified().await })
        .await?;

    println!("app-api-token example: ok");
    Ok(())
}
