use std::{convert::From, fmt, fmt::Display, sync::PoisonError};

use crate::error::ActorErrorType::InternalError;
use tonic::{transport::Error as TonicError, Status as TonicStatus};
#[derive(Debug)]
pub enum Error {
    TransportError,
    GrpcError(GrpcError),
    ActorError(ActorError),
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
        Error::GrpcError(GrpcError { status: error })
    }
}

#[derive(Debug)]
pub struct GrpcError {
    status: TonicStatus,
}

impl Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum ActorErrorType {
    NoSuchMethod,
    NoSuchActorType,
    MethodInvocation,
    InternalError,
    HttpError,
}

#[derive(Debug)]
pub struct ActorError {
    internal_error: ActorErrorType,
}

impl Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ActorErrorType> for Error {
    fn from(error: ActorErrorType) -> Self {
        Error::ActorError(ActorError {
            internal_error: error,
        })
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_error: PoisonError<T>) -> Error {
        Error::ActorError(ActorError {
            internal_error: InternalError,
        })
    }
}
