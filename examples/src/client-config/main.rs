//! Demonstrates the three idiomatic ways to construct a Dapr [`Client`]:
//!
//! 1. [`dapr::Client::new`] — reads `DAPR_GRPC_ENDPOINT`, `DAPR_GRPC_PORT`,
//!    `DAPR_API_TOKEN`, and `DAPR_CLIENT_TIMEOUT_SECONDS` from the
//!    environment. This is the recommended entry point for apps that run
//!    alongside a Dapr sidecar.
//! 2. [`dapr::Client::from_options`] — full programmatic control via
//!    [`dapr::client::ClientOptions`]. Useful for tests, embedded sidecars,
//!    or anywhere you need explicit configuration.
//! 3. [`dapr::Client::connect_with_address`] — convenience for the common
//!    case of "I just want to override the address".

use std::time::Duration;

use dapr::client::ClientOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow the sidecar a moment to come up when run under `dapr run`.
    tokio::time::sleep(Duration::from_secs(2)).await;

    // (1) Env-driven. Honors DAPR_GRPC_ENDPOINT / DAPR_GRPC_PORT /
    //     DAPR_API_TOKEN / DAPR_CLIENT_TIMEOUT_SECONDS.
    let mut client_env = dapr::Client::new().await?;
    println!("[env-driven] created via Client::new()");
    let _ = client_env.get_metadata().await;

    // (2) Explicit options via the builder.
    let opts = ClientOptions::new()
        .with_address("http://127.0.0.1:50001".to_string())
        .with_api_token("my-token")
        .with_timeout(Duration::from_secs(10));
    let mut client_opts = dapr::Client::from_options(opts).await?;
    println!("[options] created via Client::from_options(...)");
    let _ = client_opts.get_metadata().await;

    // (3) Explicit address only. Other settings still come from the
    //     environment (notably DAPR_API_TOKEN).
    let mut client_addr =
        dapr::Client::connect_with_address("http://127.0.0.1:50001".to_string()).await?;
    println!("[address] created via Client::connect_with_address(...)");
    let _ = client_addr.get_metadata().await;

    Ok(())
}
