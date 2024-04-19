const CONFIGSTORE_NAME: &str = "configstore";
type DaprClient = dapr::Client<dapr::client::TonicClient>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set the Dapr address
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = match DaprClient::connect(addr).await {
        Ok(client) => {
            println!("connected to dapr sidecar");
            client
        }
        Err(error) => {
            panic!("failed to connect to dapr sidecar: {:?}", error)
        }
    };
    println!("debug");

    let key = String::from("hello");

    // get key-value pair in the state store
    let response = client
        .get_configuration(CONFIGSTORE_NAME, vec![(&key)], None)
        .await?;
    let val = response.items.get("hello").unwrap();
    println!("Configuration value: {val:?}");

    Ok(())
}
