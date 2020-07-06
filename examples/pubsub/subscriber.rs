use std::collections::HashMap;

use tonic::{transport::Server, Request, Response, Status};

use dapr::dapr::dapr::proto::common::v1::*;
use dapr::dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer};
use dapr::dapr::dapr::proto::runtime::v1::*;

#[derive(Default)]
pub struct AppCallbackService {}

#[tonic::async_trait]
impl AppCallback for AppCallbackService {
    async fn on_invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        Ok(Response::new(InvokeResponse::default()))
    }

    async fn list_topic_subscriptions(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
        let topic = "A".to_string();

        let topic_subscription = TopicSubscription {
            topic,
            metadata: HashMap::new(),
        };

        let list_sub = ListTopicSubscriptionsResponse {
            subscriptions: vec![topic_subscription],
        };

        Ok(Response::new(list_sub))
    }

    async fn on_topic_event(
        &self,
        request: Request<TopicEventRequest>,
    ) -> Result<Response<()>, Status> {
        let data = &request.into_inner().data;
        let message = String::from_utf8_lossy(&data);
        println!("Message: {}", &message);
        Ok(Response::new(()))
    }

    async fn list_input_bindings(
        &self,
        request: Request<()>,
    ) -> Result<Response<ListInputBindingsResponse>, Status> {
        Ok(Response::new(ListInputBindingsResponse::default()))
    }

    async fn on_binding_event(
        &self,
        request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        Ok(Response::new(BindingEventResponse::default()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse().unwrap();

    let callbackservice = AppCallbackService::default();

    println!("Call back service listening on: {}", addr);

    Server::builder()
        .add_service(AppCallbackServer::new(callbackservice))
        .serve(addr)
        .await?;

    Ok(())
}
