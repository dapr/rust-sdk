//! gRPC interceptors / tower layers for Dapr API tokens.
//!
//! - [`ApiTokenInterceptor`] is an outbound [`tonic::service::Interceptor`]
//!   that adds the configured `dapr-api-token` metadata to every request the
//!   client makes. It is wired in automatically by
//!   [`crate::Client::new`] / [`crate::Client::from_options`].
//! - [`AppApiTokenLayer`] is an inbound [`tower::Layer`] that enforces the
//!   `APP_API_TOKEN` env var on incoming gRPC requests against the
//!   app-callback server. It is opt-in.

#![warn(missing_docs)]

use std::task::{Context, Poll};

use tonic::{
    Status,
    metadata::{Ascii, MetadataValue},
};
use tower::{Layer, Service};

use super::config::{API_TOKEN_METADATA_KEY, APP_API_TOKEN_ENV};

/// Outbound interceptor that adds the Dapr API token metadata to each call.
///
/// Construct via [`ApiTokenInterceptor::new`]. When the token is `None` or
/// empty, the interceptor is a no-op, so it is safe to install
/// unconditionally.
///
/// # Examples
///
/// ```
/// use dapr::client::ApiTokenInterceptor;
///
/// let interceptor = ApiTokenInterceptor::new(Some("my-token".to_string()));
/// assert!(interceptor.has_token());
/// ```
#[derive(Clone, Debug, Default)]
pub struct ApiTokenInterceptor {
    token: Option<MetadataValue<Ascii>>,
}

impl ApiTokenInterceptor {
    /// Create an interceptor that injects the given token. Passing `None`
    /// (or `Some("")`) yields a no-op interceptor.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::InvalidMetadata`] when the token
    /// contains characters that are not valid HTTP/2 ASCII metadata.
    pub fn new(token: Option<String>) -> Self {
        Self::try_new(token).unwrap_or_default()
    }

    /// Fallible variant of [`ApiTokenInterceptor::new`] returning the
    /// metadata parse error.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::InvalidMetadata`] when the token
    /// contains characters that are not valid HTTP/2 ASCII metadata.
    pub fn try_new(token: Option<String>) -> Result<Self, crate::error::Error> {
        let token = match token {
            Some(t) if !t.is_empty() => Some(t.parse()?),
            _ => None,
        };
        Ok(Self { token })
    }

    /// Returns `true` when a non-empty token has been configured.
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }
}

impl tonic::service::Interceptor for ApiTokenInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(token) = &self.token {
            request
                .metadata_mut()
                .insert(API_TOKEN_METADATA_KEY, token.clone());
        }
        Ok(request)
    }
}

/// A tower [`Layer`] that enforces the `APP_API_TOKEN` env var (or an
/// explicit token) on incoming gRPC requests.
///
/// Wrap your tonic [`Server`](https://docs.rs/tonic/latest/tonic/transport/server/struct.Server.html)
/// or a specific service with `AppApiTokenLayer` to require that callers
/// present a matching `dapr-api-token` metadata value. Missing or mismatched
/// tokens result in `Unauthenticated`.
///
/// When the layer is constructed via [`AppApiTokenLayer::from_env`] and the
/// `APP_API_TOKEN` env var is unset or empty, the layer is permissive — every
/// request passes. This makes it safe to install unconditionally.
///
/// # Examples
///
/// ```
/// use dapr::client::AppApiTokenLayer;
///
/// // Read APP_API_TOKEN from the environment (permissive if unset):
/// let layer = AppApiTokenLayer::from_env();
///
/// // Or specify an explicit expected token:
/// let strict = AppApiTokenLayer::new(Some("expected".to_string()));
/// assert!(strict.is_enforcing());
/// ```
#[derive(Clone, Debug, Default)]
pub struct AppApiTokenLayer {
    expected: Option<String>,
}

impl AppApiTokenLayer {
    /// Construct a layer that enforces the given token. `None` (or an empty
    /// string) yields a permissive layer.
    pub fn new(expected: Option<String>) -> Self {
        let expected = match expected {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        };
        Self { expected }
    }

    /// Construct a layer that reads the expected token from `APP_API_TOKEN`.
    /// Permissive when the env var is unset or empty.
    pub fn from_env() -> Self {
        Self::new(std::env::var(APP_API_TOKEN_ENV).ok())
    }

    /// Returns `true` when this layer will reject requests without a valid
    /// token.
    pub fn is_enforcing(&self) -> bool {
        self.expected.is_some()
    }
}

impl<S> Layer<S> for AppApiTokenLayer {
    type Service = AppApiTokenService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AppApiTokenService {
            inner,
            expected: self.expected.clone(),
        }
    }
}

/// Tower service produced by [`AppApiTokenLayer`].
#[derive(Clone, Debug)]
pub struct AppApiTokenService<S> {
    inner: S,
    expected: Option<String>,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for AppApiTokenService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let expected = self.expected.clone();

        Box::pin(async move {
            if let Some(expected_token) = expected {
                let presented = req
                    .headers()
                    .get(API_TOKEN_METADATA_KEY)
                    .and_then(|v| v.to_str().ok());
                if presented != Some(expected_token.as_str()) {
                    let response = http::Response::builder()
                        .status(http::StatusCode::UNAUTHORIZED)
                        .header(
                            "grpc-status",
                            (tonic::Code::Unauthenticated as i32).to_string(),
                        )
                        .header("grpc-message", "invalid or missing dapr-api-token")
                        .body(ResBody::default())
                        .expect("static response is valid");
                    return Ok(response);
                }
            }
            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Request;
    use tonic::service::Interceptor;

    #[test]
    fn interceptor_no_token_is_noop() {
        let mut interceptor = ApiTokenInterceptor::new(None);
        let req = interceptor.call(Request::new(())).unwrap();
        assert!(req.metadata().get(API_TOKEN_METADATA_KEY).is_none());
    }

    #[test]
    fn interceptor_empty_token_is_noop() {
        let mut interceptor = ApiTokenInterceptor::new(Some(String::new()));
        assert!(!interceptor.has_token());
        let req = interceptor.call(Request::new(())).unwrap();
        assert!(req.metadata().get(API_TOKEN_METADATA_KEY).is_none());
    }

    #[test]
    fn interceptor_injects_token() {
        let mut interceptor = ApiTokenInterceptor::new(Some("abc".to_string()));
        let req = interceptor.call(Request::new(())).unwrap();
        assert_eq!(req.metadata().get(API_TOKEN_METADATA_KEY).unwrap(), "abc");
    }

    #[test]
    fn interceptor_rejects_invalid_metadata() {
        // \n is not a valid ASCII metadata character.
        assert!(matches!(
            ApiTokenInterceptor::try_new(Some("bad\nvalue".to_string())),
            Err(crate::error::Error::InvalidMetadata)
        ));
    }

    #[test]
    fn app_layer_permissive_when_no_token() {
        let layer = AppApiTokenLayer::new(None);
        assert!(!layer.is_enforcing());
    }

    #[test]
    fn app_layer_enforces_when_token_set() {
        let layer = AppApiTokenLayer::new(Some("token".to_string()));
        assert!(layer.is_enforcing());
    }

    // End-to-end check: applying the layer to an axum Router actually
    // rejects requests that don't carry the expected token, and lets
    // matching requests through.
    #[tokio::test]
    async fn app_layer_rejects_unauthenticated_axum_requests() {
        use axum::{Router, routing::get};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app: Router = Router::new()
            .route("/secret", get(|| async { "ok" }))
            .layer(AppApiTokenLayer::new(Some("expected".to_string())));

        // No token → 401.
        let resp = app
            .clone()
            .oneshot(
                http::Request::builder()
                    .uri("/secret")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        // Wrong token → 401.
        let resp = app
            .clone()
            .oneshot(
                http::Request::builder()
                    .uri("/secret")
                    .header(API_TOKEN_METADATA_KEY, "wrong")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        // Correct token → 200 + body.
        let resp = app
            .oneshot(
                http::Request::builder()
                    .uri("/secret")
                    .header(API_TOKEN_METADATA_KEY, "expected")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");
    }

    // Permissive layer (no env / explicit None) lets everything through.
    #[tokio::test]
    async fn app_layer_permissive_passes_through() {
        use axum::{Router, routing::get};
        use tower::ServiceExt;

        let app: Router = Router::new()
            .route("/open", get(|| async { "ok" }))
            .layer(AppApiTokenLayer::new(None));

        let resp = app
            .oneshot(
                http::Request::builder()
                    .uri("/open")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
