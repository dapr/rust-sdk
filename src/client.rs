use crate::error::Error;
use prost_types::Any;
use std::fmt;

pub struct Client(internal::client::DaprClient<tonic::transport::Channel>);

impl Client {
    /// Connect to a Dapr enabled app.
    pub async fn connect(addr: String) -> Result<Self, Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(|| {
            tx.send(internal::client::DaprClient::connect(addr).map(Client)).unwrap();
        });
        Ok(rx.await.unwrap()?)
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
        let res = self
            .0
            .invoke_service(tonic::Request::new(internal::InvokeServiceEnvelope {
                id: app_id.into(),
                method: method_name.into(),
                data,
                ..Default::default()
            }))
            .await?;

        Ok(res.into_inner())
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client")
    }
}


mod internal {
    tonic::include_proto!("dapr");
}

/// A response from invoking a service
pub type InvokeServiceResponse = internal::InvokeServiceResponseEnvelope;
