#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::connect(addr).await?;

    // Invoke a method called MyMethod on another Dapr enabled service with id client
    let res = client.invoke_service("client", "my_method", None).await?;

    Ok(())
}
