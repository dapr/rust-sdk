# Dapr Conversation Example with the Rust-SDK

This example uses the echo component to send a request and the component response will be the exact message received.

## Step

### Prepare

- Dapr installed

### Run Conversation Example

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

2. Run the example using the Dapr CLI

<!-- STEP
name: Run Conversation
output_match_mode: substring
expected_stdout_lines:
  - 'conversation input: "hello world"'
  - 'conversation output: "hello world"'

background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=conversation --resources-path ./config --dapr-grpc-port 3500 -- cargo run --example conversation
```

<!-- END_STEP -->

## Result

```
  - 'conversation input: hello world'
  - 'conversation output: hello world'
```
