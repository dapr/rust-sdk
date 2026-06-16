# Client configuration example

Demonstrates the three idiomatic ways to construct a Dapr `Client` in Rust:

| Constructor                              | Use when                                              |
|------------------------------------------|-------------------------------------------------------|
| `dapr::Client::new()`                    | Standard sidecar deployment — read everything from env vars. |
| `dapr::Client::from_options(opts)`       | You need explicit, programmatic configuration.       |
| `dapr::Client::connect_with_address(a)`  | You only need to override the address.               |

## Env vars honored

| Variable                       | Default                       |
|--------------------------------|-------------------------------|
| `DAPR_GRPC_ENDPOINT`           | (unset) — takes precedence    |
| `DAPR_GRPC_PORT`               | `50001`                       |
| `DAPR_API_TOKEN`               | (unset) — no auth header sent |
| `DAPR_CLIENT_TIMEOUT_SECONDS`  | `5`                           |

## Run

1. Build the examples:

<!-- STEP
name: Build
background: false
sleep: 30
timeout: 60
-->

```bash
cargo build --examples
```

<!-- END_STEP -->

2. Run the example with dapr:

<!-- STEP
name: Run client-config example
output_match_mode: substring
match_order: sequential
expected_stdout_lines:
  - '[env-driven] created via Client::new()'
  - '[options] created via Client::from_options(...)'
  - '[address] created via Client::connect_with_address(...)'
background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=rustapp --dapr-grpc-port 50001 -- cargo run --example client-config
```

<!-- END_STEP -->
