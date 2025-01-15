use dapr::client::{ConversationInputBuilder, ConversationRequestBuilder};
use std::time::Duration;

type DaprClient = dapr::Client<dapr::client::TonicClient>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Set the Dapr address
    let address = "https://127.0.0.1".to_string();

    let mut client = DaprClient::connect(address).await?;

    let input = ConversationInputBuilder::new("hello world").build();

    let conversation_component = "echo";

    let request =
        ConversationRequestBuilder::new(conversation_component, vec![input.clone()]).build();

    println!("conversation input: {:?}", input.message);

    let response = client.converse_alpha1(request).await?;

    println!("conversation output: {:?}", response.outputs[0].result);
    Ok(())
}
