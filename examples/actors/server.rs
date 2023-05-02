use std::{sync::{Arc, Mutex}, str::from_utf8};
use async_trait::async_trait;
use dapr::{server::actor::{self, ActorError, context_client::{ActorContextClient, GrpcDaprClient}, ActorFactory, ActorInstance}, client::{DaprInterface, TonicClient}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse {
    pub available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub name: String,
}

struct MyActor {
    actor_type: String,
    id: String,
    client: Box<ActorContextClient<TonicClient>>
}

#[async_trait]
impl actor::Actor for MyActor {
    
    async fn on_activate(&mut self) -> Result<(), ActorError> {
        println!("on_activate {}", self.id);
        Ok(())
    }

    async fn on_deactivate(&mut self) -> Result<(), ActorError> {
        println!("on_deactivate");
        Ok(())
    }

    async fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, actor::ActorError> {
        println!("on_invoke {} {:?}", method, from_utf8(&data));
               
        match method {
            "do_stuff" => {
                let args = serde_json::from_slice::<MyRequest>(&data);
                if args.is_err() {
                    return Err(ActorError::SerializationError());
                }
                
                match self.do_stuff(args.unwrap()).await {
                    Ok(r) => Ok(serde_json::to_vec(&r).unwrap()),
                    Err(e) => Err(e)
                }
            }
            _ => Err(actor::ActorError::MethodNotFound)
        }
    }

    async fn on_reminder(&mut self, reminder_name: &str, data: Vec<u8>) -> Result<(), actor::ActorError> {
        println!("on_reminder {} {:?}", reminder_name, from_utf8(&data));
        Ok(())
    }

    async fn on_timer(&mut self, timer_name: &str, data: Vec<u8>) -> Result<(), actor::ActorError> {
        println!("on_timer {} {:?}", timer_name, from_utf8(&data));
        Ok(())
    }

}

impl MyActor {
    async fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, actor::ActorError> {
        println!("doing stuff with {}", data.name);
        let r = self.client.get_actor_state("key1").await;
        println!("get_actor_state {:?}", r);
        Ok(MyResponse { available: true })
    }    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let mut dapr_server = dapr::server::DaprHttpServer::new().await;
    dapr_server.register_actor("MyActor", Box::new(
        |actor_type, id, client| Arc::new(Mutex::new(MyActor{
            actor_type: actor_type.to_string(), 
            id: id.to_string(), 
            client
        }))));
    
    dapr_server.start(None, None).await?;
        
    Ok(())
}
