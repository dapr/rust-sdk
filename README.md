# Dapr SDK for Rust (Alpha)

[![Crates.io][crates-badge]][crates-url] [![Build Status][actions-badge]][actions-url] [![License: Apache 2.0][apache-badge]][apache-url] [![FOSSA Status][fossa-badge]][fossa-url] [![Good First Issues][gfi-badge]][gfi-url] [![discord][discord-badge]][discord-url] [![YouTube][youtube-badge]][youtube-link] [![X/Twitter][x-badge]][x-link]

[crates-badge]: https://img.shields.io/crates/v/dapr.svg?style=flat
[crates-url]: https://crates.io/crates/dapr
[apache-badge]: https://img.shields.io/github/license/dapr/rust-sdk?style=flat&label=License&logo=github
[apache-url]: https://github.com/dapr/rust-sdk/blob/master/LICENSE
[actions-badge]: https://github.com/dapr/rust-sdk/workflows/dapr-rust-sdk/badge.svg
[actions-url]: https://github.com/dapr/rust-sdk/actions?query=workflow%3Adapr-rust-sdk
[fossa-badge]: https://app.fossa.com/api/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk.svg?type=shield
[fossa-url]: https://app.fossa.com/projects/custom%2B162%2Fgithub.com%2Fdapr%2Frust-sdk?ref=badge_shield
[gfi-badge]:https://img.shields.io/github/issues-search/dapr/rust-sdk?query=type%3Aissue%20is%3Aopen%20label%3A%22good%20first%20issue%22&label=Good%20first%20issues&style=flat&logo=github
[gfi-url]:https://github.com/dapr/rust-sdk/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[discord-badge]: https://img.shields.io/discord/778680217417809931?label=Discord&style=flat&logo=discord
[discord-url]: http://bit.ly/dapr-discord
[youtube-badge]:https://img.shields.io/youtube/channel/views/UCtpSQ9BLB_3EXdWAUQYwnRA?style=flat&label=YouTube%20views&logo=youtube
[youtube-link]:https://youtube.com/@daprdev
[x-badge]:https://img.shields.io/twitter/follow/daprdev?logo=x&style=flat
[x-link]:https://twitter.com/daprdev

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

Ensure you have Rust version 1.56 or higher installed. If not, install Rust [here](https://www.rust-lang.org/tools/install).

You will also need to install [protoc](https://github.com/protocolbuffers/protobuf#protobuf-compiler-installation).

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

### Contact Us
Reach out with any questions you may have and we'll be sure to answer them as
soon as possible!

[![Discord Banner](https://discord.com/api/guilds/778680217417809931/widget.png?style=banner2)](https://aka.ms/dapr-discord)
