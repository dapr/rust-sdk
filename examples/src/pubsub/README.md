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

1. To run the example we need to first build the examples using the following command:

<!-- STEP
name: Build
background: false
sleep: 30
timeout: 60
-->

```bash
cargo build --examples
```

<!-- END_STEP -->

2. Run the multi-app run template:

<!-- STEP
name: Run Subscriber
output_match_mode: substring
match_order: sequential
expected_stdout_lines:
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 0,'
  - '== APP - rust-subscriber ==     order_details: "Count is 0",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 1,'
  - '== APP - rust-subscriber ==     order_details: "Count is 1",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 2,'
  - '== APP - rust-subscriber ==     order_details: "Count is 2",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 3,'
  - '== APP - rust-subscriber ==     order_details: "Count is 3",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 4,'
  - '== APP - rust-subscriber ==     order_details: "Count is 4",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 5,'
  - '== APP - rust-subscriber ==     order_details: "Count is 5",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 6,'
  - '== APP - rust-subscriber ==     order_details: "Count is 6",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 7,'
  - '== APP - rust-subscriber ==     order_details: "Count is 7",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 8,'
  - '== APP - rust-subscriber ==     order_details: "Count is 8",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic A - Order {'
  - '== APP - rust-subscriber ==     order_number: 9,'
  - '== APP - rust-subscriber ==     order_details: "Count is 9",'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 0,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 1,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 2,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 3,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 4,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 5,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 6,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 7,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 8,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-subscriber == Topic B - Refund {'
  - '== APP - rust-subscriber ==     order_number: 9,'
  - '== APP - rust-subscriber ==     refund_amount: 1200,'
  - '== APP - rust-subscriber == }'
  - '== APP - rust-publisher == messages published'
background: false
sleep: 30
timeout_seconds: 60
-->


```bash
dapr run -f .
```

<!-- END_STEP -->

3. Stop with `ctrl + c`

### Running without multi-app

1. Run the subscriber with dapr
```bash
dapr run --app-id rust-subscriber --app-protocol grpc --app-port 50051 cargo run -- --example pubsub-subscriber
```

2. Run the publisher with dapr
```bash
dapr run --app-id rust-publisher --app-protocol grpc cargo run -- --example pubsub-publisher
```
