# Pub/Sub Example

This is a simple example that demonstrates Dapr's pub/sub capabilities. To implement pub/sub in your rust application, you need to implement `AppCallback` server for subscribing to events. Specifically, the following two methods need to be implemented for pub/sub to work:

1. `list_topic_subscriptions` - Dapr runtime calls this method to get list of topics the application is subscribed to.
2. `on_topic_event` - Defines how the application handles the topic event. 

> **Note:** Make sure to use latest version of proto bindings.

## Running

> Before you run the example make sure local redis state store is running by executing:
> ```
> docker ps
> ```

To run this example:

1. Start Subscriber (expose gRPC server receiver on port 50051):
```bash
dapr run --app-id rust-subscriber --app-protocol grpc --app-port 50051 cargo run -- --example subscriber
```

2. Start Publisher:
```bash
dapr run --app-id rust-publisher --app-protocol grpc cargo run -- --example publisher
```
