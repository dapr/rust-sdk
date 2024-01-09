use std::{convert::From, fmt, fmt::Display};

use tonic::{transport::Error as TonicError, Status as TonicStatus};

#[derive(Debug)]
pub enum Error {
    TransportError,
    GrpcError(GrpcError),
    SerializationError,
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
    fn from(error: TonicStatus) -> Self {
        Error::GrpcError(GrpcError { _status: error })
    }
}

#[derive(Debug)]
pub struct GrpcError {
    _status: TonicStatus,
}

impl Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
