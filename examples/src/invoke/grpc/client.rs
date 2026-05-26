use crate::hello_world::HelloReply;
use std::time::Duration;

use prost::Message;

pub mod hello_world {
    include!("../protos/helloworld.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Connect to the Dapr sidecar.
    let mut client = dapr::Client::new().await?;

    let request = hello_world::HelloRequest {
        name: "Test".to_string(),
    };
    let data = request.encode_to_vec();
    let data = prost_types::Any {
        type_url: "".to_string(),
        value: data,
    };

    let response = client
        .invoke_service("invoke-grpc-server", "say_hello", Some(data))
        .await
        .unwrap();

    if let Some(any) = &response.data {
        let data = &any.value;
        let resp = HelloReply::decode(&data[..]).unwrap();
        println!("Message: {:#?}", &resp.message);
    };

    println!("Response: {response:#?}");

    Ok(())
}
