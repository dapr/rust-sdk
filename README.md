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

Ensure you have Rust version 1.88 or higher installed. If not, install Rust [here](https://www.rust-lang.org/tools/install).

These crates no longer require protoc unless to recompile the protobuf files.

## How to use

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
dapr = "0.19"
```

Here's a basic example to create a client:

```rust,no_run
use dapr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads DAPR_GRPC_ENDPOINT / DAPR_GRPC_PORT / DAPR_API_TOKEN /
    // DAPR_CLIENT_TIMEOUT_SECONDS from the environment, with sensible
    // defaults (`http://127.0.0.1:50001`, 5 s timeout, no token).
    let mut client = dapr::Client::new().await?;
    let _ = client.get_metadata().await?;
    Ok(())
}
```

## Configuration

The client honors the following environment variables, matching the
[other Dapr SDKs](https://docs.dapr.io/developing-applications/sdks/):

| Variable                      | Default | Purpose                                                         |
| ----------------------------- | ------- | --------------------------------------------------------------- |
| `DAPR_GRPC_ENDPOINT`          | (unset) | Full sidecar endpoint (scheme + host + port). Takes precedence. |
| `DAPR_GRPC_PORT`              | `50001` | Port on `127.0.0.1` when `DAPR_GRPC_ENDPOINT` is unset.         |
| `DAPR_API_TOKEN`              | (unset) | Outbound `dapr-api-token` metadata sent on every gRPC call.     |
| `DAPR_CLIENT_TIMEOUT_SECONDS` | `5`     | Connect timeout for the gRPC channel.                           |
| `APP_API_TOKEN`               | (unset) | Inbound auth token enforced by `AppApiTokenLayer` (see below).  |

You can configure the same settings programmatically via
[`dapr::client::ClientOptions`](https://docs.rs/dapr/latest/dapr/client/struct.ClientOptions.html):

```rust,no_run
use std::time::Duration;
use dapr::client::ClientOptions;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let opts = ClientOptions::new()
    .with_address("https://my-sidecar:443?tls=true".to_string())
    .with_api_token("my-token")
    .with_timeout(Duration::from_secs(10));
let mut client = dapr::Client::from_options(opts).await?;
# Ok(()) }
```

Or, if you just need to override the address:

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let mut client = dapr::Client::connect_with_address(
    "http://127.0.0.1:50001".to_string()
).await?;
# Ok(()) }
```

### Authentication

**Outbound** — set `DAPR_API_TOKEN` (or `ClientOptions::with_api_token`) and the
client automatically attaches the `dapr-api-token` metadata header to every
request. No further setup required.

**Inbound** — protect your app-callback gRPC server with `APP_API_TOKEN` by
adding `AppApiTokenLayer` to your tonic `Server`. When the env var is unset the
layer is a no-op, so it is safe to install unconditionally:

```rust,no_run
use dapr::appcallback::AppCallbackService;
use dapr::client::AppApiTokenLayer;
use dapr::dapr::proto::runtime::v1::app_callback_server::AppCallbackServer;
use tonic::transport::Server;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let addr = "127.0.0.1:50051".parse()?;
Server::builder()
    .layer(AppApiTokenLayer::from_env())
    .add_service(AppCallbackServer::new(AppCallbackService::new()))
    .serve(addr)
    .await?;
# Ok(()) }
```

See the [`client-config`](https://github.com/dapr/rust-sdk/tree/main/examples/src/client-config)
and [`app-api-token`](https://github.com/dapr/rust-sdk/tree/main/examples/src/app-api-token)
examples for end-to-end usage.

### Migrating from `Client::connect` / `Client::connect_with_port`

`Client::connect` and `Client::connect_with_port` are deprecated in `0.19.0`
and will be **removed in `0.20.0`**. Migrate as follows:

```rust,ignore
// Before (deprecated):
let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
let addr = format!("http://127.0.0.1:{port}");
let mut client = dapr::Client::<dapr::client::TonicClient>::connect(addr).await?;

// After:
let mut client = dapr::Client::new().await?;
```

## Workflows

Workflows are available through the default-on `workflow` cargo feature and the `dapr::workflow` module. See the [workflow](https://github.com/dapr/rust-sdk/tree/main/examples/src/workflow), [workflow-parallel](https://github.com/dapr/rust-sdk/tree/main/examples/src/workflow-parallel), [workflow-taskexecutionid](https://github.com/dapr/rust-sdk/tree/main/examples/src/workflow-taskexecutionid), [workflow-sustained](https://github.com/dapr/rust-sdk/tree/main/examples/src/workflow-sustained), and [workflow-history-propagation](https://github.com/dapr/rust-sdk/tree/main/examples/src/workflow-history-propagation) examples.

## Explore more examples

Browse through more examples to understand the SDK better: [View examples](https://github.com/dapr/rust-sdk/tree/main/examples)

## Building

To build the SDK run:

```bash
cargo build
```

## Developing (Updating .proto files from upstream Dapr)

To fetch the latest .proto files from Dapr execute the script `update-protos.sh`:

```bash
./update-protos.sh
```

By default, the script fetches the latest proto updates from the master branch of the Dapr repository. If you need to choose a specific release or version, use the -v flag:

```bash
./update-protos.sh -v v1.15.0
```

You will also need to install [protoc](https://github.com/protocolbuffers/protobuf#protobuf-compiler-installation).

Protos can then be compiled using:

```bash
cargo run proto-gen
```

### Contact Us

Reach out with any questions you may have and we'll be sure to answer them as
soon as possible!

[![Discord Banner](https://discord.com/api/guilds/778680217417809931/widget.png?style=banner2)](https://aka.ms/dapr-discord)
