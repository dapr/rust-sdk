use std::{convert::From, env::VarError, fmt, fmt::Display, num::ParseIntError};

use tonic::{
    Status as TonicStatus, metadata::errors::InvalidMetadataValue, transport::Error as TonicError,
};

/// Errors returned by the Dapr SDK client and helpers.
///
/// This enum is marked `#[non_exhaustive]` so that new variants can be added
/// in future minor releases without breaking downstream code that matches on
/// it. Downstream code performing exhaustive `match` must include a wildcard
/// arm.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The transport layer (tonic / hyper) failed to establish or maintain a
    /// connection.
    TransportError,
    /// A gRPC call returned a non-OK [`tonic::Status`].
    GrpcError(GrpcError),
    /// A value could not be parsed as an integer (for example,
    /// `DAPR_GRPC_PORT` or `DAPR_CLIENT_TIMEOUT_SECONDS`).
    ParseIntError,
    /// An environment variable lookup failed (typically because the variable
    /// is unset or contains non-Unicode data).
    VarError,
    /// A response payload could not be (de)serialized.
    SerializationError,
    /// A gRPC endpoint string could not be parsed (invalid scheme, malformed
    /// host/port, unsupported TLS query, etc.). The wrapped string is a
    /// sanitized endpoint (`scheme://host[:port]` when a scheme is present;
    /// otherwise `host[:port]`) for diagnostics.
    InvalidEndpoint(String),
    /// Establishing the gRPC connection exceeded the configured connect
    /// timeout (see `DAPR_CLIENT_TIMEOUT_SECONDS` or
    /// [`crate::client::ClientOptions::with_timeout`]).
    ConnectTimeout,
    /// A value supplied as gRPC metadata (e.g. the `dapr-api-token` header)
    /// contained characters that are not legal in HTTP/2 metadata.
    InvalidMetadata,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Return a diagnostics-safe endpoint string.
///
/// Keeps scheme when present and strips path/query/fragment and any userinfo
/// so that credentials never appear in logs or error messages.
pub(crate) fn sanitize_endpoint_for_diagnostics(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "<empty>".to_string();
    }

    let (scheme, rest) = match trimmed.split_once("://") {
        Some((s, r)) => (Some(s), r),
        None => (None, trimmed),
    };

    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);

    let host_port = authority.rsplit('@').next().unwrap_or(authority).trim();

    if host_port.is_empty() {
        "<invalid>".to_string()
    } else if let Some(scheme) = scheme {
        format!("{scheme}://{host_port}")
    } else {
        host_port.to_string()
    }
}

impl std::error::Error for Error {}

impl From<ParseIntError> for Error {
    fn from(_error: ParseIntError) -> Self {
        Error::ParseIntError
    }
}

impl From<VarError> for Error {
    fn from(_error: VarError) -> Self {
        Error::VarError
    }
}

impl From<TonicError> for Error {
    fn from(_error: TonicError) -> Self {
        Error::TransportError
    }
}

impl From<TonicStatus> for Error {
    fn from(error: TonicStatus) -> Self {
        Error::GrpcError(GrpcError { _status: error })
    }
}

impl From<InvalidMetadataValue> for Error {
    fn from(_error: InvalidMetadataValue) -> Self {
        Error::InvalidMetadata
    }
}

/// Wrapper around a [`tonic::Status`] produced by a failed gRPC call.
#[derive(Debug)]
pub struct GrpcError {
    _status: TonicStatus,
}

impl Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
