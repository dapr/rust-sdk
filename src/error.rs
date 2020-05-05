#[derive(Debug)]
pub enum Error {
    TransportError,
    GrpcError(GrpcError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl std::convert::From<tonic::transport::Error> for Error {
    fn from(error: tonic::transport::Error) -> Self {
        Error::TransportError
    }
}

impl std::convert::From<tonic::Status> for Error {
    fn from(_error: tonic::Status) -> Self {
        Error::GrpcError(GrpcError {})
    }
}

#[derive(Debug)]
pub struct GrpcError {}

impl std::fmt::Display for GrpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
