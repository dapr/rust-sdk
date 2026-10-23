use std::{collections::HashMap, time::Duration};

const MAX_INVOKE_ATTEMPTS: usize = 10;

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

        let mut attempts = 0;
        loop {
            attempts += 1;
            match client
                .invoke_binding(
                    binding_name,
                    message.clone(),
                    "create",
                    Some(metadata.clone()),
                )
                .await
            {
                Ok(_) => break,
                Err(err) if attempts < MAX_INVOKE_ATTEMPTS => {
                    eprintln!("Failed to invoke binding on attempt {attempts}; retrying: {err}");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(err) => return Err(err.into()),
            }
        }

        // sleep for 500ms to simulate delay b/w two events
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
