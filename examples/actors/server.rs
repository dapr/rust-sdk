use std::str::from_utf8;
use dapr::{server::actor::{self, ActorError, context_client::{ActorContextClient}, Actor, runtime::ActorTypeRegistration}, client::{TonicClient}};
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

impl MyActor {
    
    fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, ActorError> {        
        println!("doing stuff with {}", data.name);
        let r = self.client.get_actor_state("key1");
        println!("get_actor_state {:?}", r);
        Ok(MyResponse { available: true })
    }
}


impl Actor for MyActor {
    
    fn on_activate(&mut self) -> Result<(), ActorError> {
        println!("on_activate {}", self.id);
        Ok(())
    }

    fn on_deactivate(&mut self) -> Result<(), ActorError> {
        println!("on_deactivate");
        Ok(())
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let mut dapr_server = dapr::server::DaprHttpServer::new().await;
    
    dapr_server.register_actor(ActorTypeRegistration::new("MyActor", |actor_type, id, client| Box::new(MyActor{
            actor_type, 
            id, 
            client}))
        .register_method("do_stuff", MyActor::do_stuff)
        .register_method("do_stuff2", MyActor::do_stuff));
        
    
    dapr_server.start(None, None).await?;
        
    Ok(())
}
