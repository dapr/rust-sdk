# Dapr SDK for Rust

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]
[![License: MIT][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/dapr.svg
[crates-url]: https://crates.io/crates/dapr
[mit-badge]: https://img.shields.io/badge/License-MIT-yellow.svg
[mit-url]: https://github.com/dapr/rust-sdk/blob/master/LICENSE
[actions-badge]: https://github.com/gdhuper/rust-sdk/workflows/dapr-rust-sdk/badge.svg
[actions-url]: https://github.com/dapr/rust-sdk/actions?query=workflow%3Adapr-rust-sdk

⚠ Work in Progress ⚠

Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.

- [dapr.io](https://dapr.io)
- [@DaprDev](https://twitter.com/DaprDev)

## Prerequsites

* [Install Rust > 1.40](https://www.rust-lang.org/tools/install)

## Usage

```toml
[dependencies]
dapr = "0.0.9"
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
