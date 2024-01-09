use async_trait::async_trait;
use axum::{extract::rejection::PathRejection, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display, sync::Arc};

use self::context_client::ActorContextClient;

pub mod context_client;
pub mod runtime;

pub type ActorFactory = Box<dyn Fn(&str, &str, ActorContextClient) -> Arc<dyn Actor> + Send + Sync>;

#[async_trait]
pub trait Actor: Send + Sync {
    async fn on_activate(&self) -> Result<(), ActorError>;
    async fn on_deactivate(&self) -> Result<(), ActorError>;
    async fn on_reminder(&self, _reminder_name: &str, _data: Vec<u8>) -> Result<(), ActorError>;
    async fn on_timer(&self, _timer_name: &str, _data: Vec<u8>) -> Result<(), ActorError>;
}

#[derive(Debug)]
pub enum ActorError {
    NotRegistered,
    CorruptedState,
    MethodNotFound,
    ActorNotFound,
    MethodError(Box<dyn Error>),
    SerializationError(),
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(self.to_string()),
        )
            .into_response()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActorPath {
    pub actor_id: String,
}

pub enum ActorRejection {
    ActorError(String),
    Path(PathRejection),
}

impl IntoResponse for ActorRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            ActorRejection::ActorError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(e)).into_response()
            }
            ActorRejection::Path(e) => {
                (StatusCode::BAD_REQUEST, axum::Json(e.body_text())).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests;
