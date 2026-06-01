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
    // Create the client
    let mut client = dapr::Client::new().await?;

    // Retry with exponential backoff until the actor-server is registered.
    let mut resp: Result<MyResponse, dapr::error::Error> =
        Err(dapr::error::Error::SerializationError);
    for attempt in 1..=10u32 {
        let data = MyRequest {
            name: "test".to_string(),
        };

        resp = client
            .invoke_actor("MyActor", "a1", "do_stuff", data, None)
            .await;

        if resp.is_ok() || attempt == 10 {
            break;
        }

        eprintln!("Actor not ready (attempt {attempt}/10), retrying in 2s…");
        tokio::time::sleep(std::time::Duration::new(2, 0)).await;
    }

    println!("Response: {resp:#?}");

    Ok(())
}
