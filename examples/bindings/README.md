# Input and Output Bindings Example

This is a simple example that demonstrates Dapr's binding capabilities. To implement input bindings in your rust application, you need to implement `AppCallback` server for subscribing to events. Specifically, the following two methods need to be implemented for input bindings to work:

1. `list_input_bindings` - Dapr runtime calls this method to get list of bindings the application is subscribed to.
2. `on_binding_event` - Defines how the application handles the input binding event. 

> **Note:** Make sure to use latest version of proto bindings.

In order to have both examples working with the same binding configuration ServiceBus was used here. If you don't have it available you can change to a binding that works for both Input and Output from [this list](https://docs.dapr.io/reference/components-reference/supported-bindings/)


## Running

To run this example:

1. Start Input bindings listener (expose gRPC server receiver on port 50051):
```bash
dapr run --components-path ./examples/bindings/components --app-id rust-input-b --app-protocol grpc --app-port 50051 cargo run -- --example input-bindings
```

2. Start Output binding:
```bash
dapr run --components-path ./examples/bindings/components --app-id rust-output-b --app-protocol grpc cargo run -- --example output-bindings
```