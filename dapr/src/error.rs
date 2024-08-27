use std::{convert::From, env::VarError, fmt, fmt::Display, num::ParseIntError};

use tonic::{transport::Error as TonicError, Status as TonicStatus};

#[derive(Debug)]
pub enum Error {
    TransportError,
    GrpcError(GrpcError),
    ParseIntError,
    VarError,
    SerializationError,
    UnimplementedError,
    InitializationError,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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

#[derive(Debug)]
pub struct GrpcError {
    _status: TonicStatus,
}

impl Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
