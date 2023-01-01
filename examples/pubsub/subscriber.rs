use dapr::proc_macros::topic;
use dapr::{
    appcallback::*,
    dapr::dapr::proto::runtime::v1::app_callback_server::{AppCallback, AppCallbackServer},
};
use tonic::{transport::Server, Request, Response, Status};

use dapr::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    pub order_number: i32,
    pub order_details: String,
}

#[topic(pub_sub_name = "pubsub", topic = "A")]
async fn handle_event(order: Order) {
    println!("{:#?}", order)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();

    let callback_service = HandleEvent::default();

    println!("AppCallback server listening on: {}", addr);

    // Create a gRPC server with the callback_service.
    Server::builder()
        .add_service(AppCallbackServer::new(callback_service))
        .serve(addr)
        .await?;

    Ok(())
}
