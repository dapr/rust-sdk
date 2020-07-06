use std::{thread, time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    thread::sleep(time::Duration::new(2, 0));

    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let mut count = 0;

    // topic to publish message to
    let topic = "A".to_string();

    loop {
        if count == 100 {
            break;
        }

        let message = format!("{} => hello from rust!", &count)
            .as_bytes()
            .to_vec();

        client.publish_event(&topic, message).await?;

        count = count + 1;

        // sleep for 2 secs to simulate delay b/w two events
        thread::sleep(time::Duration::new(2, 0));
    }

    Ok(())
}
