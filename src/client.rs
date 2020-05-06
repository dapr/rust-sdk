use async_trait::async_trait;
use crate::error::Error;
use prost_types::Any;

pub struct Client<T>(T);

impl <T: DaprInterface> Client<T> {
    /// Connect to a Dapr enabled app.
    pub async fn connect(addr: String) -> Result<Self, Error> {
        Ok(Client(T::connect(addr).await?))
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
                message: common_v1::InvokeRequest{
                    method: method_name.into(),
                    data,
                    ..Default::default()
                }.into(),
                ..Default::default()
            })
            .await

    }

    /// Invoke an Dapr output binding.
    pub async fn invoke_binding<S>(&mut self, name: S, data: Option<Any>) -> Result<(), Error>
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
    pub async fn publish_event<S>(&mut self, topic: S, data: Option<Any>) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.0
            .publish_event(PublishEventRequest {
                topic: topic.into(),
                data
            })
            .await
    }

    /// Get the secret for a specific key.
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
    pub async fn save_state<I, K>(&mut self, store_name: K, requests: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, Option<Any>)>,
        K: Into<String>,
    {
        self.0
            .save_state(SaveStateRequest {
                store_name: store_name.into(),
                requests: requests.into_iter().map(|pair| pair.into()).collect(),
            })
            .await
    }

    /// Delete the state for a specific key.
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
pub trait DaprInterface: std::marker::Sized {
    async fn connect(addr: String) -> Result<Self, Error>;
    async fn publish_event(&mut self, request: PublishEventRequest) -> Result<(), Error>;
    async fn invoke_service(&mut self, request: InvokeServiceRequest) -> Result<InvokeServiceResponse, Error>;
    async fn invoke_binding(&mut self, request: InvokeBindingRequest) -> Result<(), Error>;
    async fn get_secret(&mut self, request: GetSecretRequest) -> Result<GetSecretResponse, Error>;
    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error>;
    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error>;
    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error>;
}


#[async_trait]
impl DaprInterface for dapr_v1::dapr_client::DaprClient<tonic::transport::Channel> {
    async fn connect(addr: String) -> Result<Self, Error> {
        Ok(dapr_v1::dapr_client::DaprClient::connect(addr).await?)
    }

    async fn invoke_service(&mut self, request: InvokeServiceRequest) -> Result<InvokeServiceResponse, Error> {
        Ok(self.invoke_service(tonic::Request::new(request)).await?.into_inner())
    }

    async fn invoke_binding(&mut self, request: InvokeBindingRequest) -> Result<(), Error> {
        Ok(self.invoke_binding(tonic::Request::new(request)).await?.into_inner())
    }

    async fn publish_event(&mut self, request: PublishEventRequest) -> Result<(), Error> {
        Ok(self.publish_event(tonic::Request::new(request)).await?.into_inner())
    }
    
    async fn get_secret(&mut self, request: GetSecretRequest) -> Result<GetSecretResponse, Error> {
        Ok(self.get_secret(tonic::Request::new(request)).await?.into_inner())
    }

    async fn get_state(&mut self, request: GetStateRequest) -> Result<GetStateResponse, Error> {
        Ok(self.get_state(tonic::Request::new(request)).await?.into_inner())
    }

    async fn save_state(&mut self, request: SaveStateRequest) -> Result<(), Error> {
        Ok(self.save_state(tonic::Request::new(request)).await?.into_inner())
    }

    async fn delete_state(&mut self, request: DeleteStateRequest) -> Result<(), Error> {
        Ok(self.delete_state(tonic::Request::new(request)).await?.into_inner())
    }
}

pub mod dapr {
    pub mod proto {
        pub mod common {
            pub mod v1 {
                tonic::include_proto!("dapr.proto.common.v1");
            }
        }
        pub mod dapr {
            pub mod v1 {
                tonic::include_proto!("dapr.proto.dapr.v1");
            }
        }
    }
}

use dapr::proto::common::v1 as common_v1;
use dapr::proto::dapr::v1 as dapr_v1;


/// A request from invoking a service
pub type InvokeServiceRequest = dapr_v1::InvokeServiceRequest;

/// A response from invoking a service
pub type InvokeServiceResponse = common_v1::InvokeResponse;

/// A request from invoking a binding
pub type InvokeBindingRequest = dapr_v1::InvokeBindingEnvelope;

/// A request for publishing event
pub type PublishEventRequest = dapr_v1::PublishEventEnvelope;

/// A request for getting state
pub type GetStateRequest = dapr_v1::GetStateEnvelope;

/// A response from getting state
pub type GetStateResponse = dapr_v1::GetStateResponseEnvelope;

/// A request for saving state
pub type SaveStateRequest = dapr_v1::SaveStateEnvelope;

/// A request for deleting state
pub type DeleteStateRequest = dapr_v1::DeleteStateEnvelope;

/// A request for getting secret
pub type GetSecretRequest = dapr_v1::GetSecretEnvelope;

/// A response from getting secret
pub type GetSecretResponse = dapr_v1::GetSecretResponseEnvelope;

/// A tonic based gRPC client
pub type TonicClient = dapr_v1::dapr_client::DaprClient<tonic::transport::Channel>;

impl<K> From<(K, Option<Any>)> for dapr_v1::StateRequest
where
    K: Into<String>,
{
    fn from((key, value): (K, Option<Any>)) -> Self {
        dapr_v1::StateRequest {
            key: key.into(),
            value: value,
            ..Default::default()
        }
    }
}

// impl<I> From<(I, Option<Any>)> for common_v1::InvokeRequest
// where
//     I: Into<String>,
// {
//     fn from((method, data): (I, Option<Any>)) -> Self {
//         common_v1::InvokeRequest {
//             method: method.into(),
//             data: data.unwrap_or_default()
//         }
//     }
// }
