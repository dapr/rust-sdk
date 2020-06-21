use std::marker::Sized;

use async_trait::async_trait;
use dapr::proto::{common::v1 as common_v1, runtime::v1 as dapr_v1};
use prost_types::Any;
use tonic::{transport::Channel as TonicChannel, Request};

use crate::dapr::*;
use crate::error::Error;

pub struct Client<T>(T);

impl<T: DaprInterface> Client<T> {
    /// Connect to a Dapr enabled app.
    ///
    /// # Arguments
    ///
    /// * `addr` - Address of gRPC server to connect to.
    pub async fn connect(addr: String) -> Result<Self, Error> {
        Ok(Client(T::connect(addr).await?))
    }

    /// Invoke a method in a Dapr enabled app.
    ///
    /// # Arguments
    ///
    /// * `app_id` - Id of the application running.
    /// * `method_name` - Name of the method to invoke.
    /// * `data` - Required. Bytes value or data required to invoke service.
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
        self.0
            .invoke_service(InvokeServiceRequest {
                id: app_id.into(),
                message: common_v1::InvokeRequest {
                    method: method_name.into(),
                    data,
                    ..Default::default()
                }
                .into(),
            })
            .await
    }

    /// Invoke an Dapr output binding.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the output binding to invoke.
    /// * `data` - The data which will be sent to the output binding.
    pub async fn invoke_binding<S>(
        &mut self,
        name: S,
        data: Vec<u8>,
    ) -> Result<InvokeBindingResponse, Error>
    where
        S: Into<String>,
    {
        self.0
            .invoke_binding(InvokeBindingRequest {
                name: name.into(),
                data,
                ..Default::default()
            })
            .await
    }

    /// Publish a payload to multiple consumers who are listening on a topic.
    ///
    /// Dapr guarantees at least once semantics for this endpoint.
    ///
    /// # Arguments
    ///
    /// * `topic` - Pubsub topic.
    /// * `data` - The data which will be published to topic.
    pub async fn publish_event<S>(&mut self, topic: S, data: Vec<u8>) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .publish_event(PublishEventRequest {
                topic: topic.into(),
                data,
            })
            .await
    }

    /// Get the secret for a specific key.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of secret store.
    /// * `key` - The name of secret key.
    pub async fn get_secret<S>(&mut self, store_name: S, key: S) -> Result<GetSecretResponse, Error>
    where
        S: Into<String>,
    {
        self.0
            .get_secret(GetSecretRequest {
                store_name: store_name.into(),
                key: key.into(),
                ..Default::default()
            })
            .await
    }

    /// Get the state for a specific key.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of state store.
    /// * `key` - The key of the desired state.
    pub async fn get_state<S>(&mut self, store_name: S, key: S) -> Result<GetStateResponse, Error>
    where
        S: Into<String>,
    {
        self.0
            .get_state(GetStateRequest {
                store_name: store_name.into(),
                key: key.into(),
                ..Default::default()
            })
            .await
    }

    /// Save an array of state objects.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of state store.
    /// * `states` - The array of the state key values.
    pub async fn save_state<I, K>(&mut self, store_name: K, states: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, Vec<u8>)>,
        K: Into<String>,
    {
        self.0
            .save_state(SaveStateRequest {
                store_name: store_name.into(),
                states: states.into_iter().map(|pair| pair.into()).collect(),
            })
            .await
    }

    /// Delete the state for a specific key.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of state store.
    /// * `key` - The key of the desired state.
    pub async fn delete_state<S>(&mut self, store_name: S, key: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .delete_state(DeleteStateRequest {
                store_name: store_name.into(),
                key: key.into(),
                ..Default::default()
            })
            .await
    }
}

#[async_trait]
pub trait DaprInterface: Sized {
    async fn connect(addr: String) -> Result<Self, Error>;
    async fn publish_event(&mut self, request: PublishEventRequest) -> Result<(), Error>;
    async fn invoke_service(
        &mut self,
        request: InvokeServiceRequest,
    ) -> Result<InvokeServiceResponse, Error>;
    async fn invoke_binding(
        &mut self,
        request: InvokeBindingRequest,
    ) -> Result<InvokeBindingResponse, Error>;
    async fn get_secret(&mut self, request: GetSecretRequest) -> Result<GetSecretResponse, Error>;
    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error>;
    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error>;
    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error>;
}

#[async_trait]
impl DaprInterface for dapr_v1::dapr_client::DaprClient<TonicChannel> {
    async fn connect(addr: String) -> Result<Self, Error> {
        Ok(dapr_v1::dapr_client::DaprClient::connect(addr).await?)
    }

    async fn invoke_service(
        &mut self,
        request: InvokeServiceRequest,
    ) -> Result<InvokeServiceResponse, Error> {
        Ok(self
            .invoke_service(Request::new(request))
            .await?
            .into_inner())
    }

    async fn invoke_binding(
        &mut self,
        request: InvokeBindingRequest,
    ) -> Result<InvokeBindingResponse, Error> {
        Ok(self
            .invoke_binding(Request::new(request))
            .await?
            .into_inner())
    }

    async fn publish_event(&mut self, request: PublishEventRequest) -> Result<(), Error> {
        Ok(self
            .publish_event(Request::new(request))
            .await?
            .into_inner())
    }

    async fn get_secret(&mut self, request: GetSecretRequest) -> Result<GetSecretResponse, Error> {
        Ok(self.get_secret(Request::new(request)).await?.into_inner())
    }

    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error> {
        Ok(self.get_state(Request::new(request)).await?.into_inner())
    }

    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error> {
        Ok(self.save_state(Request::new(request)).await?.into_inner())
    }

    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error> {
        Ok(self.delete_state(Request::new(request)).await?.into_inner())
    }
}

/// A request from invoking a service
pub type InvokeServiceRequest = dapr_v1::InvokeServiceRequest;

/// A response from invoking a service
pub type InvokeServiceResponse = common_v1::InvokeResponse;

/// A request from invoking a binding
pub type InvokeBindingRequest = dapr_v1::InvokeBindingRequest;

/// A reponse from invoking a binding
pub type InvokeBindingResponse = dapr_v1::InvokeBindingResponse;

/// A request for publishing event
pub type PublishEventRequest = dapr_v1::PublishEventRequest;

/// A request for getting state
pub type GetStateRequest = dapr_v1::GetStateRequest;

/// A response from getting state
pub type GetStateResponse = dapr_v1::GetStateResponse;

/// A request for saving state
pub type SaveStateRequest = dapr_v1::SaveStateRequest;

/// A request for deleting state
pub type DeleteStateRequest = dapr_v1::DeleteStateRequest;

/// A request for getting secret
pub type GetSecretRequest = dapr_v1::GetSecretRequest;

/// A response from getting secret
pub type GetSecretResponse = dapr_v1::GetSecretResponse;

/// A tonic based gRPC client
pub type TonicClient = dapr_v1::dapr_client::DaprClient<tonic::transport::Channel>;

impl<K> From<(K, Vec<u8>)> for common_v1::StateItem
where
    K: Into<String>,
{
    fn from((key, value): (K, Vec<u8>)) -> Self {
        common_v1::StateItem {
            key: key.into(),
            value,
            ..Default::default()
        }
    }
}
