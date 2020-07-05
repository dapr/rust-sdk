# Dapr Rust SDK - Examples

These examples demonstrates how to use Dapr rust sdk.

* [client](./client)
  * Simple dapr cient example that saves, gets, and delete state from the state stores
* [pubsub](./pubsub)
  * Publishes and subscribes events

## Adding new examples

To add new examples, `Cargo.toml` would have to be updated as follows:

```rust
[[example]]
name = "example-name"
path = "examples/example-name/example.rs"

```