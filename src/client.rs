use crate::dapr::dapr::proto::{common::v1 as common_v1, runtime::v1 as dapr_v1};
use prost_types::Any;
use std::collections::HashMap;
use tonic::Streaming;
use tonic::{transport::Channel as TonicChannel, Request};

use crate::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
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
    /// * `pubsub_name` - Name of the pubsub component
    /// * `topic` - Pubsub topic.
    /// * `data` - The data which will be published to topic.
    pub async fn publish_event<S>(
        &mut self,
        pubsub_name: S,
        topic: S,
        data_content_type: S,
        data: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let mut mdata = HashMap::<String, String>::new();
        if let Some(m) = metadata {
            mdata = m;
        }
        self.0
            .publish_event(PublishEventRequest {
                pubsub_name: pubsub_name.into(),
                topic: topic.into(),
                data_content_type: data_content_type.into(),
                data,
                metadata: mdata,
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

    /// Get all secrets for a given store
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the secret store.
    pub async fn get_bulk_secret<S>(
        &mut self,
        store_name: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetBulkSecretResponse, Error>
    where
        S: Into<String>,
    {
        self.0
            .get_bulk_secret(GetBulkSecretRequest {
                store_name: store_name.into(),
                metadata: metadata.unwrap_or_default(),
            })
            .await
    }

    /// Get the state for a specific key.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of state store.
    /// * `key` - The key of the desired state.
    pub async fn get_state<S>(
        &mut self,
        store_name: S,
        key: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetStateResponse, Error>
    where
        S: Into<String>,
    {
        let mut mdata = HashMap::<String, String>::new();
        if let Some(m) = metadata {
            mdata = m;
        }

        self.0
            .get_state(GetStateRequest {
                store_name: store_name.into(),
                key: key.into(),
                metadata: mdata,
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

    /// Delete an array of state objects.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of state store.
    /// * `states` - The array of the state key values.
    pub async fn delete_bulk_state<I, K>(&mut self, store_name: K, states: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, Vec<u8>)>,
        K: Into<String>,
    {
        self.0
            .delete_bulk_state(DeleteBulkStateRequest {
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
    pub async fn delete_state<S>(
        &mut self,
        store_name: S,
        key: S,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let mut mdata = HashMap::<String, String>::new();
        if let Some(m) = metadata {
            mdata = m;
        }

        self.0
            .delete_state(DeleteStateRequest {
                store_name: store_name.into(),
                key: key.into(),
                metadata: mdata,
                ..Default::default()
            })
            .await
    }

    /// Set sidecar Metadata
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    pub async fn set_metadata<S>(&mut self, key: S, value: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .set_metadata(SetMetadataRequest {
                key: key.into(),
                value: value.into(),
            })
            .await
    }

    /// Set sidecar Metadata
    ///
    pub async fn get_metadata(&mut self) -> Result<GetMetadataResponse, Error> {
        self.0.get_metadata().await
    }

    /// Invoke a method in a Dapr actor.
    ///
    /// # Arguments
    ///
    /// * `actor_type` - Type of the actor.
    /// * `actor_id` - Id of the actor.
    /// * `method_name` - Name of the method to invoke.
    /// * `input` - Required. Data required to invoke service, should be json serializable.
    pub async fn invoke_actor<I, M, TInput, TOutput>(
        &mut self,
        actor_type: I,
        actor_id: I,
        method_name: M,
        input: TInput,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<TOutput, Error>
    where
        I: Into<String>,
        M: Into<String>,
        TInput: Serialize,
        TOutput: for<'a> Deserialize<'a>,
    {
        let mut mdata = HashMap::<String, String>::new();
        if let Some(m) = metadata {
            mdata = m;
        }

        mdata.insert("Content-Type".to_string(), "application/json".to_string());

        let data = match serde_json::to_vec(&input) {
            Ok(data) => data,
            Err(_e) => return Err(Error::SerializationError),
        };

        let res = self
            .0
            .invoke_actor(InvokeActorRequest {
                actor_type: actor_type.into(),
                actor_id: actor_id.into(),
                method: method_name.into(),
                data,
                metadata: mdata,
            })
            .await?;

        match serde_json::from_slice::<TOutput>(&res.data) {
            Ok(output) => Ok(output),
            Err(_e) => Err(Error::SerializationError),
        }
    }

    /// Get the configuration for a specific key
    /// ///
    /// # Arguments
    ///
    /// * `store_name` - The name of config store.
    /// * `keys` - The key of the desired configuration.
    pub async fn get_configuration<S, K>(
        &mut self,
        store_name: S,
        keys: Vec<K>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<GetConfigurationResponse, Error>
    where
        S: Into<String>,
        K: Into<String>,
    {
        let request = GetConfigurationRequest {
            store_name: store_name.into(),
            keys: keys.into_iter().map(|key| key.into()).collect(),
            metadata: metadata.unwrap_or_default(),
        };
        self.0.get_configuration(request).await
    }

    /// Subscribe to configuration changes
    pub async fn subscribe_configuration<S>(
        &mut self,
        store_name: S,
        keys: Vec<S>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Streaming<SubscribeConfigurationResponse>, Error>
    where
        S: Into<String>,
    {
        let request = SubscribeConfigurationRequest {
            store_name: store_name.into(),
            keys: keys.into_iter().map(|key| key.into()).collect(),
            metadata: metadata.unwrap_or_default(),
        };
        self.0.subscribe_configuration(request).await
    }

    /// Unsubscribe from configuration changes
    pub async fn unsubscribe_configuration<S>(
        &mut self,
        store_name: S,
        id: S,
    ) -> Result<UnsubscribeConfigurationResponse, Error>
    where
        S: Into<String>,
    {
        let request = UnsubscribeConfigurationRequest {
            id: id.into(),
            store_name: store_name.into(),
        };
        self.0.unsubscribe_configuration(request).await
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
    async fn get_bulk_secret(
        &mut self,
        request: GetBulkSecretRequest,
    ) -> Result<GetBulkSecretResponse, Error>;
    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error>;
    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error>;
    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error>;
    async fn delete_bulk_state(&mut self, request: DeleteBulkStateRequest) -> Result<(), Error>;
    async fn set_metadata(&mut self, request: SetMetadataRequest) -> Result<(), Error>;
    async fn get_metadata(&mut self) -> Result<GetMetadataResponse, Error>;
    async fn invoke_actor(
        &mut self,
        request: InvokeActorRequest,
    ) -> Result<InvokeActorResponse, Error>;
    async fn get_configuration(
        &mut self,
        request: GetConfigurationRequest,
    ) -> Result<GetConfigurationResponse, Error>;
    async fn subscribe_configuration(
        &mut self,
        request: SubscribeConfigurationRequest,
    ) -> Result<Streaming<SubscribeConfigurationResponse>, Error>;
    async fn unsubscribe_configuration(
        &mut self,
        request: UnsubscribeConfigurationRequest,
    ) -> Result<UnsubscribeConfigurationResponse, Error>;
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
        self.publish_event(Request::new(request))
            .await?
            .into_inner();
        Ok(())
    }

    async fn get_secret(&mut self, request: GetSecretRequest) -> Result<GetSecretResponse, Error> {
        Ok(self.get_secret(Request::new(request)).await?.into_inner())
    }

    async fn get_bulk_secret(
        &mut self,
        request: GetBulkSecretRequest,
    ) -> Result<GetBulkSecretResponse, Error> {
        Ok(self
            .get_bulk_secret(Request::new(request))
            .await?
            .into_inner())
    }

    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error> {
        Ok(self.get_state(Request::new(request)).await?.into_inner())
    }

    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error> {
        self.save_state(Request::new(request)).await?.into_inner();
        Ok(())
    }

    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error> {
        self.delete_state(Request::new(request)).await?.into_inner();
        Ok(())
    }

    async fn delete_bulk_state(&mut self, request: DeleteBulkStateRequest) -> Result<(), Error> {
        self.delete_bulk_state(Request::new(request))
            .await?
            .into_inner();
        Ok(())
    }

    async fn set_metadata(&mut self, request: SetMetadataRequest) -> Result<(), Error> {
        self.set_metadata(Request::new(request)).await?.into_inner();
        Ok(())
    }

    async fn get_metadata(&mut self) -> Result<GetMetadataResponse, Error> {
        Ok(self.get_metadata(GetMetadataRequest {}).await?.into_inner())
    }

    async fn invoke_actor(
        &mut self,
        request: InvokeActorRequest,
    ) -> Result<InvokeActorResponse, Error> {
        Ok(self.invoke_actor(Request::new(request)).await?.into_inner())
    }

    async fn get_configuration(
        &mut self,
        request: GetConfigurationRequest,
    ) -> Result<GetConfigurationResponse, Error> {
        Ok(self
            .get_configuration(Request::new(request))
            .await?
            .into_inner())
    }

    async fn subscribe_configuration(
        &mut self,
        request: SubscribeConfigurationRequest,
    ) -> Result<Streaming<SubscribeConfigurationResponse>, Error> {
        Ok(self
            .subscribe_configuration(Request::new(request))
            .await?
            .into_inner())
    }

    async fn unsubscribe_configuration(
        &mut self,
        request: UnsubscribeConfigurationRequest,
    ) -> Result<UnsubscribeConfigurationResponse, Error> {
        Ok(self
            .unsubscribe_configuration(Request::new(request))
            .await?
            .into_inner())
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

/// A request for deleting bulk state
pub type DeleteBulkStateRequest = dapr_v1::DeleteBulkStateRequest;

/// A request for getting secret
pub type GetSecretRequest = dapr_v1::GetSecretRequest;

/// A response from getting secret
pub type GetSecretResponse = dapr_v1::GetSecretResponse;

/// A request for getting bulk secrets
pub type GetBulkSecretRequest = dapr_v1::GetBulkSecretRequest;

/// A response for getting bulk secrets
pub type GetBulkSecretResponse = dapr_v1::GetBulkSecretResponse;

/// A response from getting metadata
pub type GetMetadataResponse = dapr_v1::GetMetadataResponse;

/// A request for getting metadata
pub type GetMetadataRequest = dapr_v1::GetMetadataRequest;

/// A request for setting metadata
pub type SetMetadataRequest = dapr_v1::SetMetadataRequest;

/// A request for invoking an actor
pub type InvokeActorRequest = dapr_v1::InvokeActorRequest;

/// A response from invoking an actor
pub type InvokeActorResponse = dapr_v1::InvokeActorResponse;
/// A request for getting configuration
pub type GetConfigurationRequest = dapr_v1::GetConfigurationRequest;

/// A response from getting configuration
pub type GetConfigurationResponse = dapr_v1::GetConfigurationResponse;

/// A request for subscribing to configuration changes
pub type SubscribeConfigurationRequest = dapr_v1::SubscribeConfigurationRequest;

/// A response from subscribing tto configuration changes
pub type SubscribeConfigurationResponse = dapr_v1::SubscribeConfigurationResponse;

/// A request for unsubscribing from configuration changes
pub type UnsubscribeConfigurationRequest = dapr_v1::UnsubscribeConfigurationRequest;

/// A response from unsubscribing from configuration changes
pub type UnsubscribeConfigurationResponse = dapr_v1::UnsubscribeConfigurationResponse;

/// A tonic based gRPC client
pub type TonicClient = dapr_v1::dapr_client::DaprClient<TonicChannel>;

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
