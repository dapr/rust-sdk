use std::{sync::Arc, sync::Mutex, error::Error, future::{Future}};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use self::context_client::{ActorContextClient, DaprActorInterface};

pub mod context_client;
pub mod runtime;

pub type ActorInstance = Arc<Mutex<dyn Actor>>;
pub type ActorFactory<TActorClient> = Box<dyn Fn(&str, &str, Box<ActorContextClient<TActorClient>>) -> ActorInstance>;

#[derive(Debug)]
pub enum ActorError {
    NotRegistered,
    CorruptedState,
    MethodNotFound,
    ActorNotFound,
    MethodError(Box<dyn Error>),
    SerializationError()
}

#[async_trait]
pub trait Actor {
    async fn on_activate(&mut self) -> Result<(), ActorError>;
    async fn on_deactivate(&mut self) -> Result<(), ActorError>;
    async fn on_invoke(&mut self, method: &str, data: Vec<u8>) -> Result<Vec<u8>, ActorError>;
    async fn on_reminder(&mut self, _reminder_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
    async fn on_timer(&mut self, _timer_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
}

pub trait ActorBuilder<T: DaprActorInterface> {
    fn build(&self, actor_type: &str, id: &str, client: Box<ActorContextClient<T>>) -> ActorInstance;
}

// pub async fn invoke_method_json<TActor, TInput, TOutput, TMethod, TFuture>(actor: &mut TActor, method: TMethod, data: Vec<u8>) -> Result<Vec<u8>, ActorError> 
//     where 
//         TActor: Actor, 
//         TInput: for<'a> Deserialize<'a>, 
//         TOutput: Serialize,
//         TMethod: Fn(&mut TActor, TInput) -> TFuture,
//         TFuture: Future<Output = Result<TOutput, ActorError>>
// {    
//     let args = serde_json::from_slice::<TInput>(&data);
//     if args.is_err() {
//         return Err(ActorError::SerializationError());
//     }
    
//     match method(actor, args.unwrap()).await {
//         Ok(r) => {
//             let serialized = serde_json::to_vec(&r).unwrap();
//             Ok(serialized)
//         },
//         Err(e) => Err(e)
//     }
// }
