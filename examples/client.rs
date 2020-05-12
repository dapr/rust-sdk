
extern crate async_trait;
extern crate dapr;
use prost_types::Any;



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
    let val = Some( Any{
        type_url: String::from("string"),
        value: String::from("world").as_bytes().to_vec()});

    let store_name = String::from("statestore");
    let _res = client.save_state(store_name, vec![(key, val)]).await?;

    println!("Successfully saved!");

    let get_response = client.get_state("statestore", "hello").await?;
    println!("Value is {:?}", String::from_utf8_lossy(&get_response.data.unwrap().value));

    Ok(())
}
