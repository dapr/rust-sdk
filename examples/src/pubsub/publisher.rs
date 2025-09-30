use std::{collections::HashMap, time::Duration};

use tokio::time;

use dapr::serde::{Deserialize, Serialize};
use dapr::serde_json;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Set address for Dapr connection
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    // name of the pubsub component
    let pubsub_name = "pubsub".to_string();

    // content type of the pubsub data
    let data_content_type = "text/plain".to_string();

    // topic to publish message to
    let topic = "A".to_string();
    let topic_b = "B".to_string();

    // Delay to wait for the subscriber to fully start
    time::sleep(Duration::from_secs(5)).await;

    for count in 0..10 {
        let order = Order {
            order_number: count,
            order_details: format!("Count is {count}"),
        };
        // message metadata
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("count".to_string(), count.to_string());

        // message
        let message = serde_json::to_string(&order).unwrap().into_bytes();

        client
            .publish_event(
                &pubsub_name,
                &topic,
                &data_content_type,
                message,
                Some(metadata),
            )
            .await?;

        // sleep for 1 second to simulate delay between each event
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    for count in 0..10 {
        let refund = Refund {
            order_number: count,
            refund_amount: 1200,
        };
        // message metadata
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("count".to_string(), count.to_string());

        // message
        let message = serde_json::to_string(&refund).unwrap().into_bytes();

        client
            .publish_event(
                &pubsub_name,
                &topic_b,
                &data_content_type,
                message,
                Some(metadata),
            )
            .await?;

        // sleep for 2 seconds to simulate delay between two events
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    println!("messages published");

    Ok(())
}
