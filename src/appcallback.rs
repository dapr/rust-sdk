use std::marker::Sized;

use async_trait::async_trait;
use dapr::proto::{common::v1 as common_v1, runtime::v1 as dapr_v1};
use prost_types::Any;
use tonic::{transport::Server as TonicChannel, Request};

use crate::dapr::*;
use crate::error::Error;

pub struct Server<T>(T);

impl<T: AppCallbackServer> Server<T> {
    pub async fn serve(addr: String) -> Result<Self, Error> {
        Ok(Server(T::serve(addr).await?))
    }
}

#[async_trait]
pub trait AppCallbackServer: Sized {
    async fn serve(addr: String) -> Result<Self, Error>;
    async fn on_invoke(&mut self, request: InvokeRequest) -> Result<InvokeResponse, Error>;
    async fn list_topic_subscriptions(&mut self) -> Result<ListTopicSubscriptionsResponse, Error>;
    async fn on_topic_event(&mut self, request: TopicEventRequest) -> Result<(), Error>;
    async fn list_input_bindings(&mut self) -> Result<ListInputBindingsResponse, Error>;
    async fn on_binding_event(
        &mut self,
        request: BindingEventRequest,
    ) -> Result<BindingEventResponse, Error>;
}

pub type InvokeRequest = common_v1::InvokeRequest;

pub type InvokeResponse = common_v1::InvokeResponse;

pub type ListTopicSubscriptionsResponse = dapr_v1::ListTopicSubscriptionsResponse;

pub type TopicEventRequest = dapr_v1::TopicEventRequest;

pub type ListInputBindingsResponse = dapr_v1::ListInputBindingsResponse;

pub type BindingEventRequest = dapr_v1::BindingEventRequest;

pub type BindingEventResponse = dapr_v1::BindingEventResponse;
