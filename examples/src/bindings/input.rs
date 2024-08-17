use tonic::{transport::Server, Request, Response, Status};

use dapr::dapr::proto::common::v1::{InvokeRequest, InvokeResponse};
use dapr::dapr::proto::runtime::v1::{app_callback_server::{AppCallback, AppCallbackServer},
    BindingEventRequest, BindingEventResponse, ListInputBindingsResponse,
    ListTopicSubscriptionsResponse, TopicEventRequest, TopicEventResponse,
};

#[derive(Default)]
pub struct AppCallbackService {}

#[tonic::async_trait]
impl AppCallback for AppCallbackService {
    /// Invokes service method with InvokeRequest.
    async fn on_invoke(
        &self,
        _request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        Ok(Response::new(InvokeResponse::default()))
    }

    /// Lists all topics subscribed by this app.
    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        Ok(Response::new(ListTopicSubscriptionsResponse::default()))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        _request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        Ok(Response::new(TopicEventResponse::default()))
    }

    /// Lists all input bindings subscribed by this app.
    /// NOTE: Dapr runtime will call this method to get
    /// the list of bindings the app wants to subscribe to.
    /// In this example, the app is subscribing to a local pubsub binding named "binding-example"

    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        let list_bindings = ListInputBindingsResponse {
            bindings: vec![String::from("binding-example")],
        };

        Ok(Response::new(list_bindings))
    }

    /// Listens events from the input bindings.
    async fn on_binding_event(
        &self,
        request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        let r = request.into_inner();
        let name = &r.name;
        let data = &r.data;

        let message = String::from_utf8_lossy(&data);
        println!("Binding Name: {}", &name);
        println!("Message: {}", &message);

        Ok(Response::new(BindingEventResponse::default()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse().unwrap();

    let callback_service = AppCallbackService::default();

    println!("AppCallback server listening on: {}", addr);

    // Create a gRPC server with the callback_service.
    Server::builder()
        .add_service(AppCallbackServer::new(callback_service))
        .serve(addr)
        .await?;

    Ok(())
}
