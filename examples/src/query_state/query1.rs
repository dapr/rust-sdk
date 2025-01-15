use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    tokio::time::sleep(std::time::Duration::new(5, 0)).await;

    // Set the Dapr address and create a connection
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let query_condition = json!({
        "filter": {
            "EQ": { "state": "CA" }
        },
        "sort": [
            {
                "key": "person.id",
                "order": "DESC"
            }
        ]
    });

    let response = match client
        .query_state_alpha1("statestore", query_condition, None)
        .await
    {
        Ok(response) => response.results,
        Err(e) => {
            println!("Error: {:?}", e);
            return Ok(());
        }
    };

    let mut results = Vec::new();
    for item in response {
        let value = String::from_utf8(item.data).unwrap();
        //push id and value as json
        results.push(json!({
            "id": item.key,
            "value": value
        }));
    }
    println!("Query results: {:?}", results);

    Ok(())
}
