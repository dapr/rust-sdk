use std::{thread, time::Duration};

use hello_world::{greeter_client::GreeterClient, HelloRequest};

use tonic::metadata::MetadataValue;

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    thread::sleep(Duration::from_secs(5));

    // Set the Dapr address
    let address = "https://127.0.0.1".to_string();

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

    println!("Response: {:#?}", hello_reply);

    Ok(())
}
