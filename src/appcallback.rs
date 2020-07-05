use std::marker::Sized;

use async_trait::async_trait;
use dapr::proto::{common::v1 as common_v1, runtime::v1 as dapr_v1};
use prost_types::Any;
use tonic::{transport::Server as CallBackServer, Request};

use crate::dapr::*;
use crate::error::Error;

pub struct Server<T>(T);

impl<T: AppCallback> Server<T> {
    async fn on_invoke(&mut self, request: InvokeRequest) -> Result<InvokeResponse, Error> {
        Ok(self.0.on_invoke(request).await?)
    }

    async fn list_topic_subscriptions(&mut self) -> Result<ListTopicSubscriptionsResponse, Error> {
        Ok(self.0.list_topic_subscriptions().await?)
    }

    async fn on_topic_event(&mut self, request: TopicEventRequest) -> Result<(), Error> {
        Ok(self.0.on_topic_event(request).await?)
    }

    async fn list_input_bindings(&mut self) -> Result<ListInputBindingsResponse, Error> {
        Ok(self.0.list_input_bindings().await?)
    }

    async fn on_binding_event(
        &mut self,
        request: BindingEventRequest,
    ) -> Result<BindingEventResponse, Error> {
        Ok(self.0.on_binding_event(request).await?)
    }
}

#[async_trait]
pub trait AppCallback: Sized {
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

pub type TopicSubscription = dapr_v1::TopicSubscription;

pub type TopicEventRequest = dapr_v1::TopicEventRequest;

pub type ListInputBindingsResponse = dapr_v1::ListInputBindingsResponse;

pub type BindingEventRequest = dapr_v1::BindingEventRequest;

pub type BindingEventResponse = dapr_v1::BindingEventResponse;

pub type TonicServer = dapr_v1::app_callback_server::AppCallbackServer<tonic::transport::Channel>;
