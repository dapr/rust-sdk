# Dapr SDK for Rust (Alpha)

[![Crates.io][crates-badge]][crates-url]
[![Build Status][actions-badge]][actions-url]
[![discord][discord-badge]][discord-url]
[![License: Apache 2.0][apache-badge]][apache-url]
[![FOSSA Status][fossa-badge]][fossa-url]

[crates-badge]: https://img.shields.io/crates/v/dapr.svg
[crates-url]: https://crates.io/crates/dapr
[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[apache-url]: https://github.com/dapr/rust-sdk/blob/master/LICENSE
[actions-badge]: https://github.com/dapr/rust-sdk/workflows/dapr-rust-sdk/badge.svg
[actions-url]: https://github.com/dapr/rust-sdk/actions?query=workflow%3Adapr-rust-sdk+branch%3Amain
[fossa-badge]: https://app.fossa.com/api/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk.svg?type=shield
[fossa-url]: https://app.fossa.com/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk?ref=badge_shield
[discord-badge]: https://img.shields.io/discord/778680217417809931
[discord-url]: https://discord.com/channels/778680217417809931/778680217417809934

Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.

- [dapr.io](https://dapr.io)
- [@DaprDev](https://twitter.com/DaprDev)

## Alpha

This SDK is currently in Alpha. Work is underway to bring forward a stable
release and will likely involve breaking changes.
- Documentation is incomplete.
- Not all building blocks are currently implemented.
- There may be bugs.
- The SDK does not have complete test coverage.

The maintainers commit to resolving any issues that arise and bringing this SDK
to a stable release. With this in mind, the SDK will follow the norms and
conventions of a stable SDK so far as is possible.

This SDK will be accounted for as a part of the release process. Support for 
the latest runtime release is targeted but not guaranteed.

The main tenet of development will be stability and functionality that improves
resiliency.

## Prerequisites

Ensure you have Rust version 1.79 or higher installed. If not, install Rust [here](https://www.rust-lang.org/tools/install).

You will also need to install [protoc](https://github.com/protocolbuffers/protobuf#protobuf-compiler-installation).

## How to use

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
dapr = "0.15.0"
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

## Developing (Updating .proto files from upstream Dapr)

To fetch the latest .proto files from Dapr execute the script `update-protos.sh`:

```bash
./update-protos.sh
```

By default, the script fetches the latest proto updates from the master branch of the Dapr repository. If you need to choose a specific release or version, use the -v flag:

```bash
./update-protos.sh -v v1.14.0
```

Protos can then be compiled using:

```bash
cargo run proto-gen
```

### Contact Us
Reach out with any questions you may have and we'll be sure to answer them as
soon as possible!

[![Discord Banner](https://discord.com/api/guilds/778680217417809931/widget.png?style=banner2)](https://aka.ms/dapr-discord)
