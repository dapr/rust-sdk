# Dapr SDK for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

⚠ Work in Progress ⚠

Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.

- [dapr.io](https://dapr.io)
- [@DaprDev](https://twitter.com/DaprDev)

## Prerequsites

* [Install Rust > 1.40](https://www.rust-lang.org/tools/install)

## Usage

A client can be created as follows:

```
extern crate async_trait;
extern crate dapr;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

```

## Building

To build

```bash
cargo build
```

>Note: The proto buf client generation is built into `cargo build` process so updating the proto files under `dapr/` is enough to update the proto buf client.
