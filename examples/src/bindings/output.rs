use std::{collections::HashMap, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create the client
    let mut client = dapr::Client::new().await?;

    // name of the component
    let binding_name = "binding-example";

    for count in 0..10 {
        // message metadata
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("count".to_string(), count.to_string());

        // message
        let message = format!("{} => hello from rust!", &count).into_bytes();

        client
            .invoke_binding(binding_name, message, "create", Some(metadata))
            .await?;

        // sleep for 500ms to simulate delay b/w two events
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
