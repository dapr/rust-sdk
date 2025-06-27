use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse {
    pub available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    tokio::time::sleep(std::time::Duration::new(2, 0)).await;

    // Define the Dapr address
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let data = MyRequest {
        name: "test".to_string(),
    };

    let resp: Result<MyResponse, dapr::error::Error> = client
        .invoke_actor("MyActor", "a1", "do_stuff", data, None)
        .await;

    println!("Response: {resp:#?}");

    Ok(())
}
