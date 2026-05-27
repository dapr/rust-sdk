use std::time::Duration;

use hello_world::{HelloRequest, greeter_client::GreeterClient};

use tonic::metadata::MetadataValue;

pub mod hello_world {
    include!("../protos/helloworld.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Resolve the Dapr sidecar address from the standard env vars
    // (`DAPR_GRPC_ENDPOINT`, `DAPR_GRPC_PORT`).
    let address = dapr::client::default_sidecar_address();

    let mut client = GreeterClient::connect(address).await?;

    let request = HelloRequest {
        name: "Test".to_string(),
    };
    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "dapr-app-id",
        MetadataValue::from_static("invoke-grpc-server"),
    );

    let response = client.say_hello(request).await.unwrap();
    let hello_reply = response.into_inner();

    println!("Response: {hello_reply:#?}");

    Ok(())
}
