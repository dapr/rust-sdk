use dapr_macros::topic;
use tonic::transport::Server;

use dapr::serde::{Deserialize, Serialize};
use dapr::{appcallback::*, dapr::proto::runtime::v1::app_callback_server::AppCallbackServer};

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    pub order_number: i32,
    pub order_details: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Refund {
    pub order_number: i32,
    pub refund_amount: i32,
}

#[topic(pub_sub_name = "pubsub", topic = "A")]
async fn handle_a_event(order: Order) {
    println!("Topic A - {order:#?}")
}

#[topic(pub_sub_name = "pubsub", topic = "B")]
async fn handle_b_event(refund: Refund) {
    println!("Topic B - {refund:#?}")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse().unwrap();

    let mut callback_service = AppCallbackService::new();

    callback_service.add_handler(HandleAEvent.get_handler());

    callback_service.add_handler(HandleBEvent.get_handler());

    println!("AppCallback server listening on: {addr}");

    // Create a gRPC server with the callback_service.
    Server::builder()
        .add_service(AppCallbackServer::new(callback_service))
        .serve(addr)
        .await?;

    Ok(())
}
