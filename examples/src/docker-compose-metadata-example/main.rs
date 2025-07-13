use std::time::Duration;

type DaprClient = dapr::Client<dapr::client::TonicClient>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    let address = "https://127.0.0.1".to_string();
    let port = "50002".to_string();

    let mut client = DaprClient::connect_with_port(address, port).await?;

    let metadata = client.get_metadata().await?;

    println!("retrieved data: {:?}", metadata);

    Ok(())
}
