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

1. Run the multi-app run template:

<!-- STEP
name: Run Subscriber
output_match_mode: substring
match_order: none
expected_stdout_lines:
  - '== APP - rust-subscriber == Message: 0 => hello from rust!'
  - '== APP - rust-subscriber == Content-Type: text/plain'
  - '== APP - rust-subscriber == Message: 1 => hello from rust!'
  - '== APP - rust-subscriber == Content-Type: text/plain'
  - '== APP - rust-subscriber == Message: 2 => hello from rust!'
  - '== APP - rust-subscriber == Content-Type: text/plain'
  - '== APP - rust-publisher == messages published'
background: true
sleep: 30
timeout_seconds: 90
-->


```bash
dapr run -f .
```

<!-- END_STEP -->

2. Stop with `ctrl + c`

### Running without multi-app

1. Run the subscriber with dapr
```bash
dapr run --app-id rust-subscriber --app-protocol grpc --app-port 50051 cargo run -- --example subscriber
```

2. Run the publisher with dapr
```bash
dapr run --app-id rust-publisher --app-protocol grpc cargo run -- --example publisher
```