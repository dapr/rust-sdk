# App API token example

Demonstrates inbound auth on a Dapr app-callback server using
`dapr::client::AppApiTokenLayer`.

When `APP_API_TOKEN` is set, the layer rejects every incoming gRPC request
that does not carry a matching `dapr-api-token` metadata header. When
unset, the layer is a no-op.

Configure the matching token on the Dapr sidecar with the standard
[`appApiToken` mechanism](https://docs.dapr.io/operations/security/app-api-token/).

## Run

The app installs `AppApiTokenLayer::from_env()` on its tonic server and
advertises a single `cron` input binding (`probe`, defined in
`./config/cron.yaml`). When launched under `dapr run` with `APP_API_TOKEN`
set in the environment, the sidecar inherits the same token and signs every
callback to the app with a matching `dapr-api-token` metadata header. The
sidecar delivers the first cron tick to the app's `on_binding_event`,
proving the layer authenticates real data-plane callbacks end-to-end. The
app then shuts down gracefully.

<!-- STEP
name: Run app-api-token example
expected_stdout_lines:
  - '== APP == sidecar callback received: on_binding_event(probe) (auth ok)'
  - '== APP == app-api-token example: ok'
background: false
sleep: 5
timeout_seconds: 120
-->

```bash
APP_API_TOKEN=expected-token dapr run \
    --app-id app-api-token \
    --app-protocol grpc \
    --app-port 50051 \
    --resources-path ./config \
    -- cargo run --example app-api-token
```

<!-- END_STEP -->

## Result

`AppApiTokenLayer` rejects any sidecar callback that arrives without the
expected `dapr-api-token` metadata. The matching token flows in through
`APP_API_TOKEN`, so the cron binding's tick lands authenticated, the app
logs the callback, then shuts down. If the token were missing or
mismatched the sidecar would log an `Unauthenticated` error and no
callback line would be printed.
