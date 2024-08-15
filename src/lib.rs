#![doc = include_str!("../README.md")]

extern crate dapr_macros;

pub use dapr_macros::actor;
pub use serde;
pub use serde_json;

pub use client::Client;

/// Module containing the Dapr Callback SDK.
pub mod appcallback;
/// Module containing the 'Client' implementation.
pub mod client;
/// Module importing the Dapr runtime implementation.
pub mod dapr;
/// Module defining the error implementations.
pub mod error;
/// Module containing the 'Server' implementation.
pub mod server;
