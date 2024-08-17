use dapr::{
    appcallback::*,
    dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer},
};
use tonic::{transport::Server, Request, Response, Status};

use prost::Message;

use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    include!("../protos/helloworld.rs");
}

pub struct AppCallbackService {}

#[tonic::async_trait]
impl AppCallback for AppCallbackService {
    /// Invokes service method with InvokeRequest.
    async fn on_invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let r = request.into_inner();

        let method = &r.method;
        println!("Method: {method}");
        let data = &r.data;

        if let Some(any) = data {
            let data = &any.value;
            let resp = HelloRequest::decode(&data[..]).unwrap();
            println!("Name: {:#?}", &resp.name);

            let response = HelloReply {
                message: "Hello World!".to_string(),
            };
            let data = response.encode_to_vec();

            let data = prost_types::Any {
                type_url: "".to_string(),
                value: data,
            };

            let invoke_response = InvokeResponse {
                content_type: "application/json".to_string(),
                data: Some(data),
            };

            return Ok(Response::new(invoke_response));
        };

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
        let list_subscriptions = ListTopicSubscriptionsResponse::default();
        Ok(Response::new(list_subscriptions))
    }

    /// Subscribes events from Pubsub.
    async fn on_topic_event(
        &self,
        _request: Request<TopicEventRequest>,
    ) -> Result<Response<TopicEventResponse>, Status> {
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
    let server_address = "[::]:50051".parse().unwrap();

    let callback_service = AppCallbackService {};

    println!("AppCallback server listening on: {}", server_address);
    // Create a gRPC server with the callback_service.
    Server::builder()
        .add_service(AppCallbackServer::new(callback_service))
        .serve(server_address)
        .await?;

    Ok(())
}
