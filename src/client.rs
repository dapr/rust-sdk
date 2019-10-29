use crate::error::Error;
use prost_types::Any;

pub struct Client(internal::client::DaprClient<tonic::transport::Channel>);

impl Client {
    /// Connect to a Dapr enabled app.
    pub fn connect(addr: String) -> Result<Self, Error> {
        Ok(Client(internal::client::DaprClient::connect(addr)?))
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

mod internal {
    tonic::include_proto!("dapr");
}

/// A response from invoking a service
pub type InvokeServiceResponse = internal::InvokeServiceResponseEnvelope;
