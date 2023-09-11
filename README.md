# Dapr SDK for Rust

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]
[![License: Apache 2.0][apache-badge]][apache-url]
[![FOSSA Status][fossa-badge]][fossa-url]

[crates-badge]: https://img.shields.io/crates/v/dapr.svg
[crates-url]: https://crates.io/crates/dapr
[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[apache-url]: https://github.com/dapr/rust-sdk/blob/master/LICENSE
[actions-badge]: https://github.com/dapr/rust-sdk/workflows/dapr-rust-sdk/badge.svg
[actions-url]: https://github.com/dapr/rust-sdk/actions?query=workflow%3Adapr-rust-sdk
[fossa-badge]: https://app.fossa.com/api/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk.svg?type=shield
[fossa-url]: https://app.fossa.com/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk?ref=badge_shield

⚠ Work in Progress ⚠

Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.

- [dapr.io](https://dapr.io)
- [@DaprDev](https://twitter.com/DaprDev)

## Prerequisites

Ensure you have Rust version 1.40 or higher installed. If not, install Rust [here]((https://www.rust-lang.org/tools/install)).

## How to use

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
dapr = "0.13.0"
```

Here's a basic example to create a client:

```rust
use dapr;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;
```

## Explore more examples

Browse through more examples to understand the SDK better: [View examples](./examples)

## Building

To build the SDK run:

```bash
cargo build
```

>Note: The protobuf client generation is built into `cargo build` process so updating the proto files under `dapr/` is enough to update the protobuf client.

## Updating .proto files from upstream Dapr

To fetch the latest .proto files from Dapr execute the script `update-protos.sh`:

```bash
./update-protos.sh
```

By default, the script fetches the latest proto updates from the master branch of the Dapr repository. If you need to choose a specific release or version, use the -v flag:

```bash
./update-protos.sh -v v1.12.0
```
