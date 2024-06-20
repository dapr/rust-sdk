#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set the Dapr address
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let secret_store = "localsecretstore";

    let secrets_response = client.get_bulk_secret(secret_store, None).await?;

    for (secret_name, secret_content) in &secrets_response.data {
        println!(
            "Found {} with value: {}",
            secret_name,
            &secret_content.secrets.get(secret_name).unwrap()
        );
    }

    Ok(())
}
