use std::collections::HashMap;

use dapr::proto::{common::v1 as common_v1, runtime::v1 as dapr_v1};

use crate::dapr::*;

/// InvokeRequest is the message to invoke a method with the data.
pub type InvokeRequest = common_v1::InvokeRequest;

/// InvokeResponse is the response message inclduing data and its content type
/// from app callback.
pub type InvokeResponse = common_v1::InvokeResponse;

/// ListTopicSubscriptionsResponse is the message including the list of the subscribing topics.
pub type ListTopicSubscriptionsResponse = dapr_v1::ListTopicSubscriptionsResponse;

/// TopicSubscription represents a topic and it's metadata (session id etc.)
pub type TopicSubscription = dapr_v1::TopicSubscription;

/// TopicEventRequest message is compatiable with CloudEvent spec v1.0.
pub type TopicEventRequest = dapr_v1::TopicEventRequest;

/// TopicEventResponse is response from app on published message
pub type TopicEventResponse = dapr_v1::TopicEventResponse;

/// ListInputBindingsResponse is the message including the list of input bindings.
pub type ListInputBindingsResponse = dapr_v1::ListInputBindingsResponse;

/// BindingEventRequest represents input bindings event.
pub type BindingEventRequest = dapr_v1::BindingEventRequest;

/// BindingEventResponse includes operations to save state or
/// send data to output bindings optionally.
pub type BindingEventResponse = dapr_v1::BindingEventResponse;

impl ListTopicSubscriptionsResponse {
    /// Create `ListTopicSubscriptionsResponse` with a topic.
    pub fn topic(pubsub_name: String, topic: String) -> Self {
        let topic_subscription = TopicSubscription::new(pubsub_name, topic, None);

        Self {
            subscriptions: vec![topic_subscription],
        }
    }
}

impl TopicSubscription {
    /// Create a new `TopicSubscription` for a give topic.
    pub fn new(
        pubsub_name: String,
        topic: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        let mut topic_subscription = TopicSubscription {
            pubsub_name,
            topic,
            ..Default::default()
        };

        if let Some(metadata) = metadata {
            topic_subscription.metadata = metadata;
        }

        topic_subscription
    }
}

impl ListInputBindingsResponse {
    pub fn binding(binding_name: String) -> Self{
        Self{
            bindings: vec![binding_name]
        }
    }
}