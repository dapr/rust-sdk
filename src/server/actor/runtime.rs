use std::{collections::HashMap};
use super::{ActorFactory, ActorInstance, ActorError, context_client::{DaprActorInterface, ActorContextClient}, ActorBuilder};

pub struct ActorRuntime<TClient> 
where 
    TClient: DaprActorInterface,
    TClient: Clone,
{
    inner_channel: TClient,
    client_factory: Box<dyn Fn(TClient, &str, &str) -> ActorContextClient<TClient>>,
    registered_actors_types: HashMap<String, ActorFactory<TClient>>,
    active_actors: HashMap<(String, String), ActorInstance>
}

unsafe impl<TClient: DaprActorInterface> Send for ActorRuntime<TClient>
where 
    TClient: DaprActorInterface,
    TClient: Clone
{
}

impl<TClient> ActorRuntime<TClient> 
where 
    TClient: DaprActorInterface,
    TClient: Clone,
{
    pub fn new(channel: TClient, client_factory: Box<dyn Fn(TClient, &str, &str) -> ActorContextClient<TClient>>) -> Self {        
        ActorRuntime {
            inner_channel: channel,
            client_factory,
            registered_actors_types: HashMap::new(),
            active_actors: HashMap::new()
        }
    }

    pub fn register_actor(&mut self, name: &str, factory: ActorFactory<TClient>) {
        self.registered_actors_types.insert(name.to_string(), factory);
    }

    pub async fn invoke_actor(&mut self, name: &str, id: &str, method: &str, data: Vec<u8>) -> Result<Vec<u8>, ActorError> {
        let actor = self.get_or_create_actor(name, id).await?;
        let mut actor = actor.lock().unwrap();
        actor.on_invoke(method, data).await
    }

    pub async fn deactivate_actor(&mut self, name: &str, id: &str) -> Result<(), ActorError> {
      let actor = match self.active_actors.remove(&(name.to_string(), id.to_string())) {
        Some(actor_ref) => actor_ref,
        None => return Err(ActorError::ActorNotFound)
      };
      let mut actor = actor.lock().unwrap();      
      actor.on_deactivate().await?;
      drop(actor);
      Ok(())
    }

    pub fn deactivate_all(&mut self) {
        for actor in self.active_actors.values() {
            let mut actor = actor.lock().unwrap();
            let fut = actor.on_deactivate();
            _ = futures::executor::block_on(fut);
        }
        self.active_actors.clear();
    }

    pub async fn invoke_reminder(&mut self, name: &str, id: &str, reminder_name: &str, data : Vec<u8>) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id).await?;
        let mut actor = actor.lock().unwrap();
        actor.on_reminder(reminder_name, data).await?;
        Ok(())
    }

    pub async fn invoke_timer(&mut self, name: &str, id: &str, timer_name: &str, data : Vec<u8>) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id).await?;
        let mut actor = actor.lock().unwrap();
        actor.on_timer(timer_name, data).await?;
        Ok(())
    }

    pub fn list_registered_actors(&self) -> Vec<String> {
        self.registered_actors_types.keys().map(|k| k.to_string()).collect()
    }


    async fn get_or_create_actor(&mut self, actor_type: &str, id: &str) -> Result<ActorInstance, ActorError> {
        match self.active_actors.get(&(actor_type.to_string(), id.to_string())) {
            Some(actor_ref) => Ok(actor_ref.clone()),
            None => self.activate_actor(actor_type, id).await
        }
    }       

    async fn activate_actor(&mut self, actor_type: &str, id: &str) -> Result<ActorInstance, ActorError> {
        let actor = match self.registered_actors_types.get(actor_type) {
            Some(f) => {
              let cc = self.client_factory.as_ref();
              let client = Box::new(cc(self.inner_channel.clone(), actor_type, id));
              f(id, actor_type, client)
            },
            None => Err(ActorError::NotRegistered)?
        };

        let actor_key = (actor_type.to_string(), id.to_string());
        self.active_actors.insert(actor_key, actor.clone());

        match actor.lock() {
            Ok(mut a) => a.on_activate().await?,
            Err(_) => Err(ActorError::CorruptedState)?
        };

        Ok(actor)
    }

}

impl<TClient> Drop for ActorRuntime<TClient> 
where 
    TClient: DaprActorInterface,
    TClient: Clone,
{
    fn drop(&mut self) {
        self.deactivate_all();
    }
}


/*
#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde::{Serialize, Deserialize};
    use crate::server::actor::{invoke_method_json, Actor};
    use super::*;

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

    impl Actor for MyActor {
        fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, ActorError> {
            match method {
                "do_stuff" => invoke_method_json(self, &MyActor::do_stuff, data),
                _ => Err(ActorError::MethodNotFound)
            }
        }
    }

    impl MyActor {
        fn new(id: &str) -> Self {
            MyActor {
                id: id.to_string(),
            }
        }
        fn do_stuff(&mut self, data: MyRequest) -> Result<MyResponse, ActorError> {        
            println!("doing stuff with {}", data.name);
            Ok(MyResponse { available: true })
        }    
    }

    #[test]
    fn test_actor_invoke() {
        let mut runtime = ActorRuntime::new();
        runtime.register_actor("MyActor", Box::new(|id| Arc::new(Mutex::new(MyActor::new(id)))));
        let data_str = r#"{ "name": "foo" }"#;
        let data = data_str.as_bytes().to_vec();
        
        match runtime.invoke_actor("MyActor", "1", "do_stuff", data) {
            Ok(response) => {
                let response: MyResponse = serde_json::from_slice(&response).unwrap();
                assert_eq!(response.available, true);
            },
            Err(e) => panic!("error: {:?}", e)
        };
    }

    #[test]
    fn test_actor_deactivate() {
        let mut runtime = ActorRuntime::new();
        runtime.register_actor("MyActor", Box::new(|id| Arc::new(Mutex::new(MyActor::new(id)))));
        let data_str = r#"{ "name": "foo" }"#;
        let data = data_str.as_bytes().to_vec();
        _ = runtime.invoke_actor("MyActor", "1", "do_stuff", data);

        match runtime.deactivate_actor("MyActor", "1") {
            Ok(_) => (),
            Err(e) => panic!("error: {:?}", e)
        };
        
        match runtime.deactivate_actor("MyActor", "1") {
            Ok(_) => panic!("should not be able to deactivate twice"),
            Err(e) => match e {
                ActorError::ActorNotFound => (),
                _ => panic!("wrong error: {:?}", e)
            }
        };
    }

}
*/