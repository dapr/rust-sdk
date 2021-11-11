use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct InnerState {
    name: String
}
#[derive(Serialize, Deserialize, Debug)]
struct InternalState {
    foo: f64,
    bar: String,    
    inner: InnerState
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    std::thread::sleep(std::time::Duration::new(2, 0));

    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

    
    let key = String::from("hello");

    let data = InternalState {
        foo: 12.0,
        bar: "thomas".into(), 
        inner: InnerState { name: "inner_name".into()}
    };


    let store_name = String::from("statestore");

    // save key-value pair in the state store
    client.save_state(store_name, vec![(key, serde_json::to_string(&data).unwrap().into_bytes())]).await?;

    println!("Successfully saved!");

    let get_response = client.get_state("statestore", "hello", None).await?;
    let received_string = String::from_utf8_lossy(&get_response.data);
    println!("Value is {:?}", received_string);

    println!("Parsed as {:?}", serde_json::from_str::<InternalState>(&received_string).unwrap());
    // delete a value from the state store
    client.delete_state("statestore", "hello", None).await?;

    // validate if the value was successfully deleted
    let del_result = client.get_state("statestore", "hello", None).await?;

    // should print "[]" upon successful delete
    println!("Deleted value: {:?}", del_result.data);

    Ok(())
}
