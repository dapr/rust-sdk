use std::collections::HashMap;

use crate::dapr::*;

/// InvokeRequest is the message to invoke a method with the data.
pub type InvokeRequest = dapr::proto::common::v1::InvokeRequest;

/// InvokeResponse is the response message inclduing data and its content type
/// from app callback.
pub type InvokeResponse = dapr::proto::common::v1::InvokeResponse;

/// ListTopicSubscriptionsResponse is the message including the list of the subscribing topics.
pub type ListTopicSubscriptionsResponse = dapr::proto::runtime::v1::ListTopicSubscriptionsResponse;

/// TopicSubscription represents a topic and it's metadata (session id etc.)
pub type TopicSubscription = dapr::proto::runtime::v1::TopicSubscription;

/// TopicEventRequest message is compatiable with CloudEvent spec v1.0.
pub type TopicEventRequest = dapr::proto::runtime::v1::TopicEventRequest;

/// TopicEventResponse is response from app on published message
pub type TopicEventResponse = dapr::proto::runtime::v1::TopicEventResponse;

/// ListInputBindingsResponse is the message including the list of input bindings.
pub type ListInputBindingsResponse = dapr::proto::runtime::v1::ListInputBindingsResponse;

/// BindingEventRequest represents input bindings event.
pub type BindingEventRequest = dapr::proto::runtime::v1::BindingEventRequest;

/// BindingEventResponse includes operations to save state or
/// send data to output bindings optionally.
pub type BindingEventResponse = dapr::proto::runtime::v1::BindingEventResponse;

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
