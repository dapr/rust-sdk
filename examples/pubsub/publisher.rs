use std::{collections::HashMap, thread, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    thread::sleep(Duration::from_secs(2));

    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    // name of the pubsub component
    let pubsub_name = "pubsub".to_string();

    // content type of the pubsub data
    let data_content_type = "text/plain".to_string();

    // topic to publish message to
    let topic = "A".to_string();

    for count in 0..100 {
        // message metadata
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("count".to_string(), count.to_string());

	// message
        let message = format!("{} => hello from rust!", &count).into_bytes();

        client.publish_event(&pubsub_name, &topic, &data_content_type, message, Some(metadata)).await?;

        // sleep for 2 secs to simulate delay b/w two events
        tokio::time::delay_for(Duration::from_secs(2)).await;
    }

    Ok(())
}
