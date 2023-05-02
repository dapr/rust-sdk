use std::{sync::{Arc, Mutex}, str::from_utf8};
use dapr::{server::actor::{self, ActorError, context_client::{ActorContextClient, DaprActorInterface}}, dapr::dapr::proto::runtime::v1::dapr_client::DaprClient, client::{DaprInterface, TonicClient}};
use serde::{Serialize, Deserialize};
use tonic::transport::Channel;

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResponse {
    pub available: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyRequest {
    pub name: String,
}

struct MyActor {
    id: String,
    client: Box<ActorContextClient<TonicClient>>
}

impl actor::Actor for MyActor {
    
    fn on_activate(&mut self) -> Result<(), ActorError> {
        println!("on_activate {}", self.id);
        Ok(())
    }

    fn on_deactivate(&mut self) -> Result<(), ActorError> {
        println!("on_deactivate");
        Ok(())
    }

    fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, actor::ActorError> {
        println!("on_invoke {}", method);

        match method {
            "do_stuff" => actor::invoke_method_json(self, &MyActor::do_stuff, data),
            _ => Err(actor::ActorError::MethodNotFound)
        }
    }

    fn on_reminder(&mut self, reminder_name: &str, data: Vec<u8>) -> Result<(), actor::ActorError> {
        println!("on_reminder {} {:?}", reminder_name, from_utf8(&data));
        Ok(())
    }

    fn on_timer(&mut self, timer_name: &str, data: Vec<u8>) -> Result<(), actor::ActorError> {
        println!("on_timer {} {:?}", timer_name, from_utf8(&data));
        Ok(())
    }

}

impl MyActor {
    fn new(id: &str, actor_type: &str, client: Box<ActorContextClient<TonicClient>>) -> Self {
        println!("creating actor {} {}", id, actor_type);
        MyActor {
            id: id.to_string(),
            client
        }
    }
    async fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, actor::ActorError> {        
        println!("doing stuff with {}", data.name);
        let r = self.client.get_actor_state("key1").await?;
        println!("get_actor_state {:?}", r);
        Ok(MyResponse { available: true })
    }    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let mut dapr_server = dapr::server::DaprHttpServer::new().await;
    dapr_server.register_actor("MyActor", Box::new(|id, actor_type, client| Arc::new(Mutex::new(MyActor::new(id, actor_type, client)))));
    dapr_server.start(None, None).await?;
        
    Ok(())
}
