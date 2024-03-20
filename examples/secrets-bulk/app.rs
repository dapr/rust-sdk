#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

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
