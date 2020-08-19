use tonic::{transport::Server, Request, Response, Status};

use dapr::{
    appcallback::*,
    dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer},
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
    ///
    /// NOTE: Dapr runtime will call this method to get
    /// the list of topics the app wants to subscribe to.
    /// In this example, the app is subscribing to topic `A`.
    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        let topic = "A".to_string();
        let pubsub_name = "pubsub".to_string();

        let list_subscriptions = ListTopicSubscriptionsResponse::topic(pubsub_name, topic);

        Ok(Response::new(list_subscriptions))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
        let data = &request.into_inner().data;

        let message = String::from_utf8_lossy(&data);

        println!("Message: {}", &message);

        Ok(Response::new(TopicEventResponse::default()))
    }

    /// Lists all input bindings subscribed by this app.
    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        Ok(Response::new(ListInputBindingsResponse::default()))
    }

    /// Listens events from the input bindings.
    async fn on_binding_event(
        &self,
        _request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
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
