use std::{collections::HashMap, sync::Arc, sync::Mutex, error::Error};
use serde::{Serialize, Deserialize};

pub type ActorInstance = Arc<Mutex<dyn Actor>>;
pub type ActorFactory = Box<dyn Fn(&str) -> ActorInstance>;

#[derive(Debug)]
pub enum ActorError {
    NotRegistered,
    CorruptedState,
    MethodNotFound,
    ActorNotFound,
    MethodError(Box<dyn Error>),
    SerializationError()
}

pub trait Actor {
    fn on_activate(&mut self) -> Result<(), ActorError> { Ok(()) }
    fn on_deactivate(&mut self) -> Result<(), ActorError> { Ok(()) }
    fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, ActorError>;
    fn on_reminder(&mut self, _reminder_name: &str) {}
    fn on_timer(&mut self, _timer_name: &str) {}
}

pub fn invoke_method_json<TActor, TInput, TOutput>(actor: &mut TActor, method: &dyn Fn(&mut TActor, TInput) -> Result<TOutput, ActorError>, data: Vec<u8>) -> Result<Vec<u8>, ActorError> 
    where TActor: Actor, TInput: for<'a> Deserialize<'a>, TOutput: Serialize
{    
    let args = serde_json::from_slice::<TInput>(&data);
    if args.is_err() {
        return Err(ActorError::SerializationError());
    }
    match method(actor, args.unwrap()) {
        Ok(r) => {
            let serialized = serde_json::to_vec(&r).unwrap();
            Ok(serialized)
        },
        Err(e) => Err(e)
    }
}

pub struct ActorRuntime {
    registered_actors_types: HashMap<String, ActorFactory>,
    active_actors: HashMap<(String, String), ActorInstance>
}

unsafe impl Send for ActorRuntime {
}

impl ActorRuntime {
    pub fn new() -> Self {
        ActorRuntime {
            registered_actors_types: HashMap::new(),
            active_actors: HashMap::new()
        }
    }

    pub fn register_actor(&mut self, name: &str, factory: ActorFactory) {
        
        self.registered_actors_types.insert(name.to_string(), factory);
    }

    pub fn invoke_actor(&mut self, name: &str, id: &str, method: &str, data: Vec<u8>) -> Result<Vec<u8>, ActorError> {
        let actor = self.get_or_create_actor(name, id)?;
        let mut actor = actor.lock().unwrap();
        actor.on_invoke(method, data)
    }

    pub fn deactivate_actor(&mut self, name: &str, id: &str) -> Result<(), ActorError> {
      let actor = match self.active_actors.remove(&(name.to_string(), id.to_string())) {
        Some(actor_ref) => actor_ref,
        None => return Err(ActorError::ActorNotFound)
      };
      let mut actor = actor.lock().unwrap();      
      actor.on_deactivate()?;
      drop(actor);
      Ok(())
    }

    pub fn deactivate_all(&mut self) {
        for actor in self.active_actors.values() {
            let mut actor = actor.lock().unwrap();
            let _ = actor.on_deactivate();
        }
        self.active_actors.clear();
    }

    pub fn invoke_reminder(&mut self, name: &str, id: &str, reminder_name: &str) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id)?;
        let mut actor = actor.lock().unwrap();
        actor.on_reminder(reminder_name);
        Ok(())
    }

    pub fn invoke_timer(&mut self, name: &str, id: &str, timer_name: &str) -> Result<(), ActorError> {
        let actor = self.get_or_create_actor(name, id)?;
        let mut actor = actor.lock().unwrap();
        actor.on_timer(timer_name);
        Ok(())
    }

    pub fn list_registered_actors(&self) -> Vec<String> {
        self.registered_actors_types.keys().map(|k| k.to_string()).collect()
    }


    fn get_or_create_actor(&mut self, name: &str, id: &str) -> Result<ActorInstance, ActorError> {
        
        match self.active_actors.get(&(name.to_string(), id.to_string())) {
            Some(actor_ref) => Ok(actor_ref.clone()),
            None => self.activate_actor(name, id)
        }
    }       

    fn activate_actor(&mut self, name: &str, id: &str) -> Result<ActorInstance, ActorError> {
        let actor = match self.registered_actors_types.get(name) {
            Some(f) => f(id),
            None => Err(ActorError::NotRegistered)?
        };

        let actor_key = (name.to_string(), id.to_string());
        self.active_actors.insert(actor_key, actor.clone());

        match actor.lock() {
            Ok(mut a) => a.on_activate()?,
            Err(_) => Err(ActorError::CorruptedState)?
        };

        Ok(actor)
    }

}

impl Drop for ActorRuntime {
    fn drop(&mut self) {
        self.deactivate_all();
    }
}


#[cfg(test)]
mod tests {
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