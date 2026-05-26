//! Client configuration: environment variables, [`ClientOptions`], and
//! endpoint parsing.
//!
//! The Rust SDK can be configured either entirely from environment variables
//! (the common case when running alongside a Dapr sidecar) or programmatically
//! via [`ClientOptions`] with builder-style setters.
//!
//! See the crate-level docs for a comparison and recommendation.

#![warn(missing_docs)]

use std::time::Duration;

use crate::error::Error;

/// Environment variable holding the full gRPC endpoint of the Dapr sidecar
/// (e.g. `http://127.0.0.1:50001`, `https://my-sidecar:443?tls=true`,
/// `unix:///var/run/dapr.sock`). When set, this overrides [`DAPR_GRPC_PORT_ENV`].
pub const DAPR_GRPC_ENDPOINT_ENV: &str = "DAPR_GRPC_ENDPOINT";

/// Environment variable holding the gRPC port of the Dapr sidecar on
/// `127.0.0.1`. Used when [`DAPR_GRPC_ENDPOINT_ENV`] is not set.
pub const DAPR_GRPC_PORT_ENV: &str = "DAPR_GRPC_PORT";

/// Environment variable holding the Dapr API token used for outbound calls.
/// When present, the SDK adds it to each gRPC request as the
/// `dapr-api-token` metadata key.
pub const DAPR_API_TOKEN_ENV: &str = "DAPR_API_TOKEN";

/// Environment variable holding the gRPC connect timeout in seconds. Matches
/// the go-sdk name for cross-SDK consistency. Defaults to
/// [`DEFAULT_CLIENT_TIMEOUT_SECONDS`].
pub const DAPR_CLIENT_TIMEOUT_SECONDS_ENV: &str = "DAPR_CLIENT_TIMEOUT_SECONDS";

/// Environment variable holding the expected API token on inbound calls
/// against the app-callback server. When set, requests without a matching
/// `dapr-api-token` metadata value are rejected.
pub const APP_API_TOKEN_ENV: &str = "APP_API_TOKEN";

/// gRPC metadata key used to carry the Dapr API token.
pub const API_TOKEN_METADATA_KEY: &str = "dapr-api-token";

/// Default gRPC port of the Dapr sidecar when no env var is set.
pub const DEFAULT_DAPR_GRPC_PORT: u16 = 50001;

/// Default connect timeout (seconds) for the gRPC channel. Matches go-sdk.
pub const DEFAULT_CLIENT_TIMEOUT_SECONDS: u64 = 5;

/// Build the default sidecar gRPC address from the environment, with this
/// resolution order:
///
/// 1. [`DAPR_GRPC_ENDPOINT_ENV`] if set.
/// 2. `http://127.0.0.1:$DAPR_GRPC_PORT` if [`DAPR_GRPC_PORT_ENV`] is set.
/// 3. `http://127.0.0.1:50001` otherwise.
///
/// This is the same algorithm used by `Client::new()` and is exposed so that
/// related components (workflow, HTTP server) can share it.
pub fn default_sidecar_address() -> String {
    if let Ok(endpoint) = std::env::var(DAPR_GRPC_ENDPOINT_ENV)
        && !endpoint.is_empty()
    {
        return endpoint;
    }
    match std::env::var(DAPR_GRPC_PORT_ENV) {
        Ok(port) if !port.is_empty() => format!("http://127.0.0.1:{port}"),
        _ => format!("http://127.0.0.1:{DEFAULT_DAPR_GRPC_PORT}"),
    }
}

/// Configuration for the gRPC [`crate::Client`].
///
/// `ClientOptions` is the idiomatic, programmatic alternative to relying on
/// environment variables. The [`Default`] implementation reads from the
/// environment (so `ClientOptions::default()` behaves identically to
/// `Client::new()` minus the actual connect), and builder-style `with_*`
/// methods let you override individual fields.
///
/// The struct is `#[non_exhaustive]` so future fields can be added without a
/// breaking change. Construct it via [`ClientOptions::default`] or
/// [`ClientOptions::new`] and chain setters.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use dapr::client::ClientOptions;
///
/// // Pure programmatic configuration:
/// let opts = ClientOptions::new()
///     .with_address("https://my-sidecar:443?tls=true")
///     .with_api_token("super-secret")
///     .with_timeout(Duration::from_secs(10));
/// assert_eq!(opts.address(), "https://my-sidecar:443?tls=true");
/// assert_eq!(opts.api_token(), Some("super-secret"));
/// assert_eq!(opts.timeout(), Duration::from_secs(10));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ClientOptions {
    address: String,
    api_token: Option<String>,
    timeout: Duration,
}

impl ClientOptions {
    /// Create a new `ClientOptions` populated from the environment.
    ///
    /// Equivalent to [`ClientOptions::default`]; see that method for the
    /// resolution rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a `ClientOptions` from environment variables only, returning an
    /// error if any present-but-invalid value cannot be parsed (e.g.
    /// `DAPR_CLIENT_TIMEOUT_SECONDS=abc`).
    ///
    /// [`ClientOptions::default`] uses this internally and falls back to
    /// defaults on parse failure; prefer `from_env` when you want to surface
    /// such errors explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ParseIntError`] when
    /// `DAPR_CLIENT_TIMEOUT_SECONDS` is set to a non-integer or non-positive
    /// value.
    pub fn from_env() -> Result<Self, Error> {
        let address = default_sidecar_address();
        let api_token = read_optional_env(DAPR_API_TOKEN_ENV);
        let timeout = read_timeout_env()?;
        Ok(Self {
            address,
            api_token,
            timeout,
        })
    }

    /// Override the gRPC endpoint. Accepts the same shapes as
    /// `DAPR_GRPC_ENDPOINT`.
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }

    /// Set the Dapr API token used as the `dapr-api-token` metadata value on
    /// every outgoing call. Pass an empty string to clear.
    pub fn with_api_token(mut self, token: impl Into<String>) -> Self {
        let t = token.into();
        self.api_token = if t.is_empty() { None } else { Some(t) };
        self
    }

    /// Clear any previously-set Dapr API token.
    pub fn without_api_token(mut self) -> Self {
        self.api_token = None;
        self
    }

    /// Set the gRPC connect timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The configured gRPC endpoint.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// The configured Dapr API token, if any.
    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    /// The configured connect timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for ClientOptions {
    /// Read all options from the environment, silently falling back to
    /// defaults if any value is missing or invalid.
    ///
    /// Use [`ClientOptions::from_env`] if you want errors surfaced.
    fn default() -> Self {
        Self {
            address: default_sidecar_address(),
            api_token: read_optional_env(DAPR_API_TOKEN_ENV),
            timeout: read_timeout_env()
                .unwrap_or_else(|_| Duration::from_secs(DEFAULT_CLIENT_TIMEOUT_SECONDS)),
        }
    }
}

fn read_optional_env(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

fn read_timeout_env() -> Result<Duration, Error> {
    match std::env::var(DAPR_CLIENT_TIMEOUT_SECONDS_ENV) {
        Ok(v) if !v.is_empty() => {
            let secs: u64 = v.parse()?;
            if secs == 0 {
                return Err(Error::ParseIntError);
            }
            Ok(Duration::from_secs(secs))
        }
        _ => Ok(Duration::from_secs(DEFAULT_CLIENT_TIMEOUT_SECONDS)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // env vars are process-global; serialise env-mutating tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(pairs: &[(&str, Option<&str>)], f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev: Vec<(String, Option<String>)> = pairs
            .iter()
            .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
            .collect();
        for (k, v) in pairs {
            match v {
                Some(val) => unsafe { std::env::set_var(k, val) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        for (k, v) in prev {
            match v {
                Some(val) => unsafe { std::env::set_var(&k, val) },
                None => unsafe { std::env::remove_var(&k) },
            }
        }
        if let Err(e) = result {
            std::panic::resume_unwind(e);
        }
    }

    #[test]
    fn default_address_uses_built_in_default_when_unset() {
        with_env(
            &[(DAPR_GRPC_ENDPOINT_ENV, None), (DAPR_GRPC_PORT_ENV, None)],
            || {
                assert_eq!(default_sidecar_address(), "http://127.0.0.1:50001");
            },
        );
    }

    #[test]
    fn default_address_uses_port_env() {
        with_env(
            &[
                (DAPR_GRPC_ENDPOINT_ENV, None),
                (DAPR_GRPC_PORT_ENV, Some("12345")),
            ],
            || {
                assert_eq!(default_sidecar_address(), "http://127.0.0.1:12345");
            },
        );
    }

    #[test]
    fn default_address_prefers_endpoint_env() {
        with_env(
            &[
                (DAPR_GRPC_ENDPOINT_ENV, Some("https://sidecar:443?tls=true")),
                (DAPR_GRPC_PORT_ENV, Some("12345")),
            ],
            || {
                assert_eq!(default_sidecar_address(), "https://sidecar:443?tls=true");
            },
        );
    }

    #[test]
    fn options_from_env_reads_token_and_timeout() {
        with_env(
            &[
                (DAPR_GRPC_ENDPOINT_ENV, Some("http://1.2.3.4:50001")),
                (DAPR_API_TOKEN_ENV, Some("tok")),
                (DAPR_CLIENT_TIMEOUT_SECONDS_ENV, Some("17")),
            ],
            || {
                let opts = ClientOptions::from_env().unwrap();
                assert_eq!(opts.address(), "http://1.2.3.4:50001");
                assert_eq!(opts.api_token(), Some("tok"));
                assert_eq!(opts.timeout(), Duration::from_secs(17));
            },
        );
    }

    #[test]
    fn options_from_env_rejects_invalid_timeout() {
        with_env(
            &[(DAPR_CLIENT_TIMEOUT_SECONDS_ENV, Some("not-a-number"))],
            || {
                assert!(matches!(
                    ClientOptions::from_env(),
                    Err(Error::ParseIntError)
                ));
            },
        );
    }

    #[test]
    fn options_from_env_rejects_zero_timeout() {
        with_env(&[(DAPR_CLIENT_TIMEOUT_SECONDS_ENV, Some("0"))], || {
            assert!(matches!(
                ClientOptions::from_env(),
                Err(Error::ParseIntError)
            ));
        });
    }

    #[test]
    fn options_default_falls_back_on_invalid_timeout() {
        with_env(&[(DAPR_CLIENT_TIMEOUT_SECONDS_ENV, Some("nope"))], || {
            let opts = ClientOptions::default();
            assert_eq!(
                opts.timeout(),
                Duration::from_secs(DEFAULT_CLIENT_TIMEOUT_SECONDS)
            );
        });
    }

    #[test]
    fn builder_overrides_take_precedence() {
        let opts = ClientOptions::new()
            .with_address("http://override:1234")
            .with_api_token("abc")
            .with_timeout(Duration::from_secs(42));
        assert_eq!(opts.address(), "http://override:1234");
        assert_eq!(opts.api_token(), Some("abc"));
        assert_eq!(opts.timeout(), Duration::from_secs(42));
    }

    #[test]
    fn empty_api_token_clears() {
        let opts = ClientOptions::new()
            .with_api_token("abc")
            .with_api_token("");
        assert_eq!(opts.api_token(), None);
    }
}
