#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    tokio::time::sleep(std::time::Duration::new(2, 0)).await;

    // Set the Dapr address
    let addr = "https://127.0.0.1".to_string();

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    let key = String::from("hello");

    let val = String::from("world").into_bytes();

    let store_name = String::from("statestore");

    // save key-value pair in the state store
    client
        .save_state(store_name, key, val, None, None, None)
        .await?;

    println!("Successfully saved!");

    let get_response = client.get_state("statestore", "hello", None).await?;
    println!("Value is {:?}", String::from_utf8_lossy(&get_response.data));

    // delete a value from the state store
    client.delete_state("statestore", "hello", None).await?;

    // validate if the value was successfully deleted
    let del_result = client.get_state("statestore", "hello", None).await?;

    // should print "[]" upon successful delete
    println!("Deleted value: {:?}", del_result.data);

    Ok(())
}
