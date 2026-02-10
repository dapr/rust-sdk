use crate::dapr::proto::runtime::v1::app_callback_server::AppCallback;
use crate::dapr::proto::{common, runtime};
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

/// InvokeRequest is the message to invoke a method with the data.
pub type InvokeRequest = common::v1::InvokeRequest;

/// InvokeResponse is the response message inclduing data and its content type
/// from app callback.
pub type InvokeResponse = common::v1::InvokeResponse;

/// ListTopicSubscriptionsResponse is the message including the list of the subscribing topics.
pub type ListTopicSubscriptionsResponse = runtime::v1::ListTopicSubscriptionsResponse;

/// TopicSubscription represents a topic and it's metadata (session id etc.)
pub type TopicSubscription = runtime::v1::TopicSubscription;

/// TopicEventRequest message is compatiable with CloudEvent spec v1.0.
pub type TopicEventRequest = runtime::v1::TopicEventRequest;

/// TopicEventResponse is response from app on published message
pub type TopicEventResponse = runtime::v1::TopicEventResponse;

/// ListInputBindingsResponse is the message including the list of input bindings.
pub type ListInputBindingsResponse = runtime::v1::ListInputBindingsResponse;

/// BindingEventRequest represents input bindings event.
pub type BindingEventRequest = runtime::v1::BindingEventRequest;

/// BindingEventResponse includes operations to save state or
/// send data to output bindings optionally.
pub type BindingEventResponse = runtime::v1::BindingEventResponse;

/// TopicEventBulkRequest is the message for bulk topic event request.
/// It includes multiple events in one request.
pub type TopicEventBulkRequest = runtime::v1::TopicEventBulkRequest;

/// TopicEventBulkResponse is the message for bulk topic event response.
/// It includes the result for each event in the request.
pub type TopicEventBulkResponse = runtime::v1::TopicEventBulkResponse;

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
    pub fn binding(binding_name: String) -> Self {
        Self {
            bindings: vec![binding_name],
        }
    }
}

pub struct AppCallbackService {
    handlers: Vec<Handler>,
}

pub struct Handler {
    pub pub_sub_name: String,
    pub topic: String,
    pub handler: Box<dyn HandlerMethod>,
}

#[tonic::async_trait]
impl AppCallback for AppCallbackService {
    async fn on_invoke(
        &self,
        _request: Request<common::v1::InvokeRequest>,
    ) -> Result<Response<common::v1::InvokeResponse>, Status> {
        todo!("on_invoke is not implemented yet")
    }

    async fn list_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<runtime::v1::ListTopicSubscriptionsResponse>, Status> {
        let topics = self
            .handlers
            .iter()
            .fold(Vec::new(), |mut topics, handler| {
                topics.push(TopicSubscription::new(
                    handler.pub_sub_name.clone(),
                    handler.topic.clone(),
                    None,
                ));
                topics
            });
        Ok(Response::new(ListTopicSubscriptionsResponse {
            subscriptions: topics,
        }))
    }

    async fn on_topic_event(
        &self,
        request: Request<runtime::v1::TopicEventRequest>,
    ) -> Result<Response<runtime::v1::TopicEventResponse>, Status> {
        let request_inner = request.into_inner();
        let pub_sub_name = request_inner.pubsub_name.clone();
        let topic_name = request_inner.topic.clone();
        let handler = self
            .handlers
            .iter()
            .find(|x| x.pub_sub_name == pub_sub_name && x.topic == topic_name);
        if let Some(handler) = handler {
            return handler.handler.handler(request_inner).await;
        }
        Err(Status::new(Code::Internal, "Handler Not Found"))
    }

    async fn list_input_bindings(
        &self,
        _request: Request<()>,
    ) -> Result<Response<runtime::v1::ListInputBindingsResponse>, Status> {
        todo!("list_input_bindings is not implemented yet")
    }

    async fn on_binding_event(
        &self,
        _request: Request<BindingEventRequest>,
    ) -> Result<Response<BindingEventResponse>, Status> {
        todo!("on_binding_event is not implemented yet")
    }

    async fn on_bulk_topic_event(
        &self,
        _request: Request<TopicEventBulkRequest>,
    ) -> Result<Response<TopicEventBulkResponse>, Status> {
        todo!("on_bulk_topic_event is not implemented yet")
    }
}

impl Default for AppCallbackService {
    fn default() -> Self {
        Self::new()
    }
}

impl AppCallbackService {
    pub fn new() -> AppCallbackService {
        AppCallbackService { handlers: vec![] }
    }

    pub fn add_handler(&mut self, handler: Handler) {
        self.handlers.push(handler)
    }
}

#[tonic::async_trait]
pub trait HandlerMethod: Send + Sync + 'static {
    async fn handler(
        &self,
        request: runtime::v1::TopicEventRequest,
    ) -> Result<Response<runtime::v1::TopicEventResponse>, Status>;
}
