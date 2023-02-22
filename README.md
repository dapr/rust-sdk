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

## Prerequsites

* [Install Rust > 1.40](https://www.rust-lang.org/tools/install)

## Usage

```toml
[dependencies]
dapr = "0.11.0"
```

A client can be created as follows:

```rust
use dapr;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;
```

## Try out examples

[Examples](./examples)

## Building

To build

```bash
cargo build
```

>Note: The proto buf client generation is built into `cargo build` process so updating the proto files under `dapr/` is enough to update the proto buf client.

## To refresh .proto files from upstream dapr

1. Just need to run update-protos.sh, which will basically fetch latest proto updates.
2. By default, it picks from master proto. To specify a particular release/version, please specify with a -v flag

```bash
./update-protos.sh -v v1.10.1
```
