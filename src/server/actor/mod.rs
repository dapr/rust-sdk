use std::{sync::Arc, sync::Mutex, error::Error};
use serde::{Serialize, Deserialize};

use self::context_client::{ActorContextClient};

pub mod context_client;
pub mod runtime;

pub type ActorInstance = Arc<Mutex<Box<dyn Actor>>>;
pub type ActorFactory<TActorClient> = Box<dyn Fn(String, String, Box<ActorContextClient<TActorClient>>) -> Box<dyn Actor>>;

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
    fn on_activate(&mut self) -> Result<(), ActorError>;
    fn on_deactivate(&mut self) -> Result<(), ActorError>;
    fn on_reminder(&mut self, _reminder_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
    fn on_timer(&mut self, _timer_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
}

pub type ActorMethod = Box<dyn Fn(&mut dyn Actor, Vec<u8>) -> Result<Vec<u8>, ActorError>>;

pub fn decorate_actor_method<TActor, TInput, TMethod, TOutput>(method: TMethod) -> ActorMethod
    where 
        TActor: Actor, 
        TInput: for<'a> Deserialize<'a>, 
        TOutput: Serialize,
        TMethod: Fn(&mut TActor, TInput) -> Result<TOutput, ActorError> + 'static
{       
    let f =  move |actor: &mut dyn Actor, data: Vec<u8>| {
        log::debug!("Invoking actor method with data: {:?}", data);        
        let args = serde_json::from_slice::<TInput>(&data);
        if args.is_err() {
            log::error!("Failed to deserialize actor method arguments - {:?}", args.err());
            return Err(ActorError::SerializationError());
        }
        
        let well_known_actor = unsafe { &mut *(actor as *mut dyn Actor as *mut TActor) };

        match method(well_known_actor, args.unwrap()) {
            Ok(r) => {
                let serialized = serde_json::to_vec(&r).unwrap();
                Ok(serialized)
            },
            Err(e) => Err(e)
        }
    };
    Box::new(f)
}
