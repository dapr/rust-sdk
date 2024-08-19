# Dapr Rust SDK - Examples

These examples demonstrates how to use Dapr rust sdk.

* [client](src/client)
  * Simple dapr client example that saves, gets, and deletes state from the state stores
* [pubsub](src/pubsub)
  * Publishes and subscribes to events

## Adding new examples

To add new examples, `Cargo.toml` would have to be updated as follows:

```rust
[[example]]
name = "example-name"
path = "examples/example-name/example.rs"
```