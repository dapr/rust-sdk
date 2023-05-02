use std::{sync::Arc, sync::Mutex, error::Error};
use serde::{Serialize, Deserialize};

//use self::context_client::{DaprActorInterface};

pub mod context_client;
pub mod runtime;

pub type ActorInstance = Arc<Mutex<dyn Actor>>;
pub type ActorFactory<TActorClient> = Box<dyn Fn(&str, &str, Box<TActorClient>) -> ActorInstance>;

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
    fn on_reminder(&mut self, _reminder_name: &str, _data : Vec<u8>) -> Result<(), ActorError> { Ok(()) }
    fn on_timer(&mut self, _timer_name: &str, _data : Vec<u8>) -> Result<(), ActorError> { Ok(()) }
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


