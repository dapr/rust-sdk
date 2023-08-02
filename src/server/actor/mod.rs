use std::{sync::Arc, error::Error, fmt::Display};
use async_trait::async_trait;
use axum::{response::IntoResponse, extract::rejection::PathRejection, http::StatusCode};
use serde::{Serialize, Deserialize};

use self::context_client::ActorContextClient;

pub mod context_client;
pub mod runtime;


pub type ActorFactory = Box<dyn Fn(&str, &str, ActorContextClient) -> Arc<dyn Actor> + Send + Sync>;

#[async_trait]
pub trait Actor : Send + Sync {
    async fn on_activate(&self) -> Result<(), ActorError>;
    async fn on_deactivate(&self) -> Result<(), ActorError>;
    async fn on_reminder(&self, _reminder_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
    async fn on_timer(&self, _timer_name: &str, _data : Vec<u8>) -> Result<(), ActorError>;
}

#[derive(Debug)]
pub enum ActorError {
    NotRegistered,
    CorruptedState,
    MethodNotFound,
    ActorNotFound,
    MethodError(Box<dyn Error>),
    SerializationError()
}

impl Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorError::NotRegistered => write!(f, "Actor not registered"),
            ActorError::CorruptedState => write!(f, "Actor state corrupted"),
            ActorError::MethodNotFound => write!(f, "Method not found"),
            ActorError::ActorNotFound => write!(f, "Actor not found"),
            ActorError::MethodError(e) => write!(f, "Method error: {}", e),
            ActorError::SerializationError() => write!(f, "Serialization error"),
        }
    }
}

impl IntoResponse for ActorError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(self.to_string())).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActorPath {
    pub actor_id: String,
}

pub enum ActorRejection {
    ActorError(String),
    Path(PathRejection)
}

impl IntoResponse for ActorRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            ActorRejection::ActorError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(e)).into_response()
            },
            ActorRejection::Path(e) => {
                (StatusCode::BAD_REQUEST, axum::Json(e.body_text())).into_response()
            }
        }
    }
}

#[macro_export]
macro_rules! actor {
    ( $t:ident ) => {
        use axum::extract::{FromRequestParts, Path};
        use axum::http::request::Parts;
        use $crate::server::actor::{ActorPath, ActorRejection};
        use $crate::server::actor::runtime::ActorState;
        
        #[async_trait]
        impl FromRequestParts<ActorState> for &$t {
            
            type Rejection = ActorRejection;

            async fn from_request_parts(parts: &mut Parts, state: &ActorState) -> Result<Self, Self::Rejection> {                
                let path = match Path::<ActorPath>::from_request_parts(parts, state).await{
                    Ok(path) => path,
                    Err(e) => {
                        log::error!("Error getting path: {}", e);
                        return Err(ActorRejection::Path(e));
                    }
                };
                let actor_type = state.actor_type.clone();
                let actor_id = path.actor_id.clone();                
                log::info!("Request for actor_type: {}, actor_id: {}", actor_type, actor_id);
                let actor = match state.runtime.get_or_create_actor(&actor_type, &actor_id).await {
                    Ok(actor) => actor,
                    Err(e) => {
                        log::error!("Error getting actor: {}", e);
                        return Err(ActorRejection::ActorError(e.to_string()));
                    }
                };
                let actor = actor.as_ref();
                let well_known_actor = unsafe { & *(actor as *const dyn Actor as *const $t) };        
                Ok(well_known_actor)
            }
        }
    }
}

#[cfg(test)]
mod tests;