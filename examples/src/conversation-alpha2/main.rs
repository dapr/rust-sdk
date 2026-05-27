use dapr::client::{
    ConversationInputAlpha2Builder, ConversationMessage, ConversationMessageContent,
    ConversationMessageOfUser, ConversationRequestAlpha2Builder,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep to allow for the server to become available
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Connect using env vars
    let mut client = dapr::Client::new().await?;

    // Build a user message
    let user_msg = ConversationMessageOfUser {
        name: None,
        content: vec![ConversationMessageContent {
            text: "hello world".to_string(),
        }],
    };

    // Build a conversation message from the user message
    let conversation_msg = ConversationMessage::from(user_msg.clone());

    // Build input
    let input = ConversationInputAlpha2Builder::new(vec![conversation_msg])
        .with_scrub_pii(false)
        .build();

    // Build request
    let request = ConversationRequestAlpha2Builder::new("echo", vec![input])
        .with_metadata(HashMap::new())
        .with_scrub_pii(false)
        .with_temperature(0.7)
        .build();

    // Call llm
    match client.converse_alpha2(request).await {
        Ok(response) => {
            println!("conversation input: {:?}", user_msg.content[0].text);
            if let Some(output) = response.outputs.first() {
                if let Some(choice) = output.choices.first() {
                    if let Some(message) = &choice.message {
                        println!("conversation output: {:?}", message.content);
                    } else {
                        eprintln!("No message found in first choice.");
                    }
                } else {
                    eprintln!("No choices found in first output.");
                }
            } else {
                eprintln!("No outputs found in response.");
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    Ok(())
}
