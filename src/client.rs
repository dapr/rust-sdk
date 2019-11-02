use async_trait::async_trait;
use crate::error::Error;
use prost_types::Any;
use std::fmt;

pub struct Client<T>(T);

impl <T: DaprInterface> Client<T> {
    /// Connect to a Dapr enabled app.
    pub async fn connect(addr: String) -> Result<Self, Error> {
        T::connect(addr).await.map(Client)
    }

    /// Invoke a method in a Dapr enabled app.
    pub async fn invoke_service<I, M>(
        &mut self,
        app_id: I,
        method_name: M,
        data: Option<Any>,
    ) -> Result<InvokeServiceResponse, Error>
    where
        I: Into<String>,
        M: Into<String>,
    {
        self
            .0
            .invoke_service(InvokeServiceRequest {
                id: app_id.into(),
                method: method_name.into(),
                data,
                ..Default::default()
            })
            .await

    }
}

#[async_trait]
pub trait DaprInterface: std::marker::Sized {
    async fn connect(addr: String) -> Result<Self, Error>;
    async fn invoke_service(&mut self, request: InvokeServiceRequest) -> Result<InvokeServiceResponse, Error>;
}


#[async_trait]
impl DaprInterface for internal::client::DaprClient<tonic::transport::Channel> {
    async fn connect(addr: String) -> Result<Self, Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(|| {
            tx.send(internal::client::DaprClient::connect(addr)).unwrap_or_else(|_| panic!(""));
        });
        Ok(rx.await.unwrap_or_else(|_| panic!(""))?)

    }

    async fn invoke_service(&mut self, request: InvokeServiceRequest) -> Result<InvokeServiceResponse, Error> {
        Ok(self.invoke_service(tonic::Request::new(request)).await?.into_inner())
    }
}


mod internal {
    tonic::include_proto!("dapr");
}

/// A request from invoking a service
pub type InvokeServiceRequest = internal::InvokeServiceEnvelope;
/// A response from invoking a service
pub type InvokeServiceResponse = internal::InvokeServiceResponseEnvelope;

/// A tonic based gRPC client
pub type TonicClient = internal::client::DaprClient<tonic::transport::Channel>;
