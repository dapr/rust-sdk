---
type: docs
title: "Getting started with the Dapr client Rust SDK"
linkTitle: "Client"
weight: 20000
description: How to get up and running with the Dapr Rust SDK
no_list: true
---

The Dapr client package allows you to interact with other Dapr applications from a Go application.

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

## Related links
[Rust SDK Examples](https://github.com/dapr/rust-sdk/tree/master/examples)
