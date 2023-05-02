use std::{collections::HashMap};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("actor client");
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    std::thread::sleep(std::time::Duration::new(2, 0));

    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let data_str = r#"{ "name": "foo" }"#;
    let data = data_str.as_bytes().to_vec();

    let mut metadata = HashMap::new();
    metadata.insert("Content-Type".to_string(), "application/json".to_string());

    let resp = client.invoke_actor("MyActor", "a1", "do_stuff", data, Some(metadata)).await;

    match resp {
        Ok(r) => {
            let s = String::from_utf8(r.data);
            println!("Response: {:#?}", s);
        },
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }
    
    Ok(())
}
