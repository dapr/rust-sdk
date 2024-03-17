use tokio_stream::StreamExt;

const CONFIGSTORE_NAME: &str = "configstore";
type DaprClient = dapr::Client<dapr::client::TonicClient>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    std::thread::sleep(std::time::Duration::new(2, 0));

    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = DaprClient::connect(addr).await?;

    let key = String::from("hello");

    // save key-value pair in the state store
    let response = client
        .get_configuration(CONFIGSTORE_NAME, vec![(&key)], None)
        .await?;
    let val = response.items.get("hello").unwrap();
    println!("Configuration value: {val:?}");

    // Subscribe for configuration changes
    let mut stream = client
        .subscribe_configuration(CONFIGSTORE_NAME, vec![(&key)], None)
        .await?;

    let mut subscription_id = String::new();
    while let Some(result) = stream.next().await {
        let subscribe = result.unwrap();
        if subscribe.items.is_empty() {
            // Update the subscription_id
            subscription_id = subscribe.id.clone();
            println!("App subscribed to config changes with subscription id: {subscription_id:?} ");
            continue;
        }
        println!("Configuration value: {:?}", subscribe.items);
        unsubscribe(&mut client, &subscription_id).await;
    }

    Ok(())
}

// Function to unsubscribe from configuration updates and exit the app
async fn unsubscribe(client: &mut DaprClient, subscription_id: &str) {
    match client
        .unsubscribe_configuration("CONFIGSTORE_NAME", subscription_id)
        .await
    {
        Ok(_) => println!("App unsubscribed from config changes"),
        Err(e) => println!("Error unsubscribing from config updates: {}", e),
    }
    std::process::exit(0);
}
