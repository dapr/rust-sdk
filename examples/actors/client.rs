use std::{collections::HashMap, str::from_utf8};

use dapr::server::actor::context_client::ActorStateOperation;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("actor client");
    // TODO: Handle this issue in the sdk
    // Introduce delay so that dapr grpc port is assigned before app tries to connect
    std::thread::sleep(std::time::Duration::new(2, 0));

    // Get the Dapr port and create a connection
    let port: u16 = 60128; //std::env::var("DAPR_GRPC_PORT")?.parse()?;
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

    let addr = format!("https://127.0.0.1:{}", port);
    let mut actor_client = dapr::server::actor::context_client::ActorContextClient::<dapr::client::TonicClient>::connect(addr, "MyActor".to_string(), "a1".to_string()).await?;

    // let timer_data = "someData";
    // let timer_data = timer_data.as_bytes().to_vec();

    // //let period = Duration::from_secs(5);
    // match actor_client.register_actor_reminder("remind3", Some(std::time::Duration::from_secs(5)), Some(std::time::Duration::from_secs(10)), timer_data, None).await {
    //     Ok(r) => {
    //         println!("reminder registered: {:#?}", r);
    //     },
    //     Err(e) => {
    //         println!("Error: {:#?}", e);
    //     }
    // }

    //actor_client.unregister_actor_reminder("remind3").await;
    
    //let s = prost_types::Any::from("test".as_bytes().to_vec());
    
    // let ops = vec![
    //     ActorStateOperation::Upsert { key: "key1".to_string(), value: Some("value1".as_bytes().to_vec()) }
    // ];
    
    // match actor_client.execute_actor_state_transaction(ops).await {
    //     Ok(r) => {
    //         println!("state transaction: {:#?}", r);
    //     },
    //     Err(e) => {
    //         println!("Error: {:#?}", e);
    //     }
    // }

    match actor_client.get_actor_state("key1").await {
        Ok(r) => {
            
            let hs = String::from_utf8(r.data);
            println!("data: {:#?}", hs);
            //println!("state transaction: {:#?}", r);
        },
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }

    
    Ok(())
}
