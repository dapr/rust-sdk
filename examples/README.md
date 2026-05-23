# Dapr Rust SDK - Examples

These examples demonstrates how to use Dapr rust sdk.

* [client](src/client)
  * Simple dapr client example that saves, gets, and deletes state from the state stores
* [pubsub](src/pubsub)
  * Publishes and subscribes to events
* [workflow](src/workflow)
  * Demonstrates registering, running, waiting for, and purging a workflow
* [workflow-parallel](src/workflow-parallel)
  * Demonstrates parallel workflow activity execution
* [workflow-taskexecutionid](src/workflow-taskexecutionid)
  * Demonstrates activity retry task execution IDs
* [workflow-sustained](src/workflow-sustained)
  * Demonstrates sustained concurrent workflow scheduling
* [workflow-history-propagation](src/workflow-history-propagation)
  * Demonstrates propagating workflow history to child workflows and activities

## Adding new examples

To add new examples, `Cargo.toml` would have to be updated as follows:

```rust
[[example]]
name = "example-name"
path = "examples/example-name/example.rs"
```