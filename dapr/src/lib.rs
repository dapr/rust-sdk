#![doc = include_str!("../README.md")]

pub use serde;
pub use serde_json;

pub use client::Client;

/// Module containing the Dapr Callback SDK.
pub mod appcallback;
/// Module containing the 'Client' implementation.
pub mod client;

/// Module importing the Dapr runtime implementation.
pub mod dapr {
    #![allow(clippy::large_enum_variant)]
    pub mod proto {
        pub mod common {
            pub mod v1 {
                include!("dapr/dapr.proto.common.v1.rs");
            }
        }
        pub mod runtime {
            pub mod v1 {
                include!("dapr/dapr.proto.runtime.v1.rs");
            }
        }
    }
}
/// Module defining the error implementations.
pub mod error;
/// Module containing the 'Server' implementation.
pub mod server;
