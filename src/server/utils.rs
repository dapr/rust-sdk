use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use serde::de::DeserializeOwned;

/// Workaround for Dapr's JSON serialization not correcly setting Content-Type header

#[derive(Debug, Clone, Copy, Default)]
pub struct DaprJson<T>(pub T);

pub enum JsonRejection {
    JsonError(String),
}

#[async_trait]
impl<T, S> FromRequest<S> for DaprJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = match axum::body::Bytes::from_request(req, state).await {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Error getting bytes: {}", e);
                return Err(JsonRejection::JsonError(e.to_string()));
            }
        };
        let value = match serde_json::from_slice::<T>(&bytes) {
            Ok(value) => value,
            Err(e) => {
                log::error!("Error deserializing JSON: {}", e);
                return Err(JsonRejection::JsonError(e.to_string()));
            }
        };

        Ok(DaprJson(value))
    }
}

impl IntoResponse for JsonRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            JsonRejection::JsonError(e) => (StatusCode::BAD_REQUEST, axum::Json(e)).into_response(),
        }
    }
}
