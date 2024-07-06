# Input and Output Bindings Example

This is a simple example that demonstrates Dapr's binding capabilities. To implement input bindings in your rust application, you need to implement `AppCallback` server for subscribing to events. Specifically, the following two methods need to be implemented for input bindings to work:

1. `list_input_bindings` - Dapr runtime calls this method to get list of bindings the application is subscribed to.
2. `on_binding_event` - Defines how the application handles the input binding event. 

> **Note:** Make sure to use latest version of proto bindings.

In order to have both examples working with the same binding configuration ServiceBus was used here. If you don't have it available you can change to a binding that works for both Input and Output from [this list](https://docs.dapr.io/reference/components-reference/supported-bindings/)


## Running

To run this example:

1. Run a kafka container

<!-- STEP
name: Run kafka instance
background: true
sleep: 90
timeout_seconds: 120
expected_stderr_lines:
-->

```bash
docker run -p 9092:9092 apache/kafka:3.7.1
```

<!-- END_STEP -->

2. Run the multi-app run template (`dapr.yaml`)

<!-- STEP
name: Run Multi-app Run
output_match_mode: substring
match_order: sequential
expected_stdout_lines:
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 0 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 1 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 2 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 3 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 4 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 5 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 6 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 7 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 8 => hello from rust!'
  - '== APP - rust-input-b == Binding Name: binding-example'
  - '== APP - rust-input-b == Message: 9 => hello from rust!'
background: true
sleep: 30
timeout_seconds: 90
-->

```bash
dapr run -f .
```

<!-- END_STEP -->