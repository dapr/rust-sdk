use std::{convert::From, fmt, fmt::Display};

use tonic::{transport::Error as TonicError, Status as TonicStatus};

#[derive(Debug)]
pub enum Error {
    TransportError,
    GrpcError(GrpcError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<TonicError> for Error {
    fn from(_error: TonicError) -> Self {
        Error::TransportError
    }
}

impl From<TonicStatus> for Error {
    fn from(_error: TonicStatus) -> Self {
        Error::GrpcError(GrpcError {})
    }
}

#[derive(Debug)]
pub struct GrpcError {}

impl Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
