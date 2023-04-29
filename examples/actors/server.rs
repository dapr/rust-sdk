use std::sync::{Arc, Mutex};
use dapr::server::actor::{self, ActorError};
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
    id: String,
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

}

impl MyActor {
    fn new(id: &str) -> Self {
        println!("creating actor {}", id);
        MyActor {
            id: id.to_string(),
        }
    }
    fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, actor::ActorError> {        
        println!("doing stuff with {}", data.name);        
        Ok(MyResponse { available: true })
    }    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dapr_server = dapr::server::DaprHttpServer::new();
    dapr_server.register_actor("MyActor", Box::new(|id| Arc::new(Mutex::new(MyActor::new(id)))));
    dapr_server.start(None, None).await?;
        
    Ok(())
}
