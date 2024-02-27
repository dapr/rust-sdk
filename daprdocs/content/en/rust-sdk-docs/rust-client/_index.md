---
type: docs
title: "Getting started with the Dapr client Rust SDK"
linkTitle: "Client"
weight: 20000
description: How to get up and running with the Dapr Rust SDK
no_list: true
---

The Dapr client package allows you to interact with other Dapr applications from a Rust application.

> **Note:** The Rust SDK is currently in Alpha/WIP state and is constantly evolving.

## Prerequisites

- [Dapr CLI]({{< ref install-dapr-cli.md >}}) installed
- Initialized [Dapr environment]({{< ref install-dapr-selfhost.md >}})
- [Rust installed](https://www.rust-lang.org/tools/install)


## Import the client package

Add Dapr to your `cargo.toml`

```toml
[dependencies]
# Other dependencies
dapr = "0.13.0"
```

You can either reference `dapr::Client` or bind the full path to a new name as follows:
```rust
use dapr::Client as DaprClient
```


## Building blocks

The Rust SDK allows you to interface with the [Dapr building blocks]({{< ref building-blocks >}}).

### Service Invocation

To invoke a specific method on another service running with Dapr sidecar, the Dapr client Go SDK provides two options:

Invoke a service
```rust
let response = client
    .invoke_service("service-to-invoke", "method-to-invoke", Some(data))
    .await
    .unwrap();
```


For a full guide on service invocation, visit [How-To: Invoke a service]({{< ref howto-invoke-discover-services.md >}}).

### State Management

The Dapr Client provides access to these state management methods:  `save_state`, `get_state`, `delete_state` that can be used like so:

```rust
let store_name = "store-name";
let state_key = "state-key";

let states = vec![(state_key, ("state-value").as_bytes().to_vec())];

// save state with the key "state-key" and value "state-value"
client.save_state(store_name, states).await?;

// get state for key "state-key"
let response = client.get_state(store_name, state_key, None).await.unwrap();

// delete state for key "state-key"
client.delete_state(store_name, state_key, None).await?;
```

> **Note:** The `save_state` method currently performs a 'bulk' save but this will be refactored


For a full guide on state management, visit [How-To: Save & get state]({{< ref howto-get-save-state.md >}}).

### Publish Messages
To publish data onto a topic, the Dapr Go client provides a simple method:

```rust
let pubsub_name = "pubsub-name".to_string();
let pubsub_topic = "topic-name".to_string();
let pubsub_content_type = "text/plain".to_string();

let data = "content".to_string().into_bytes();
client
    .publish_event(pubsub_name, pubsub_topic, pubsub_content_type, data, None)
    .await?;
```

For a full guide on pub/sub, visit [How-To: Publish & subscribe]({{< ref howto-publish-subscribe.md >}}).

## Related links
[Rust SDK Examples](https://github.com/dapr/rust-sdk/tree/master/examples)
