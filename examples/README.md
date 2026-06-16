# Dapr Rust SDK - Examples

These examples demonstrate how to use the Dapr Rust SDK.

* [actors](src/actors)
  * Demonstrates Dapr actor hosting and actor client calls
* [app-api-token](src/app-api-token)
  * Demonstrates inbound auth on an app-callback server with `AppApiTokenLayer`
* [bindings](src/bindings)
  * Demonstrates Dapr input and output bindings
* [client](src/client)
  * Saves, gets, and deletes state with a Dapr client
* [client-config](src/client-config)
  * Demonstrates client construction from environment variables, options, and explicit addresses
* [configuration](src/configuration)
  * Reads and subscribes to Dapr configuration values
* [conversation-alpha1](src/conversation-alpha1)
  * Sends a request to the Dapr conversation alpha1 echo component
* [conversation-alpha2](src/conversation-alpha2)
  * Sends a request to the Dapr conversation alpha2 echo component
* [crypto](src/crypto)
  * Demonstrates Dapr cryptography operations
* [invoke/grpc](src/invoke/grpc)
  * Demonstrates service invocation over gRPC
* [invoke/grpc-proxying](src/invoke/grpc-proxying)
  * Demonstrates service invocation through Dapr gRPC proxying
* [jobs](src/jobs)
  * Demonstrates Dapr job scheduling
* [jobs-failurepolicy](src/jobs-failurepolicy)
  * Demonstrates Dapr job scheduling with a failure policy
* [pubsub](src/pubsub)
  * Publishes and subscribes to events
* [query_state](src/query_state)
  * Demonstrates state queries against a query-capable state store
* [secrets-bulk](src/secrets-bulk)
  * Retrieves secrets in bulk from a local secret store
* [workflow](src/workflow)
  * Demonstrates registering, running, waiting for, and purging a workflow
* [workflow-history-propagation](src/workflow-history-propagation)
  * Demonstrates propagating workflow history to child workflows and activities
* [workflow-parallel](src/workflow-parallel)
  * Demonstrates parallel workflow activity execution
* [workflow-sustained](src/workflow-sustained)
  * Demonstrates sustained concurrent workflow scheduling
* [workflow-taskexecutionid](src/workflow-taskexecutionid)
  * Demonstrates activity retry task execution IDs

## Adding new examples

To add new examples, update `examples/Cargo.toml` with an entry like:

```toml
[[example]]
name = "example-name"
path = "src/example-name/main.rs"
```
