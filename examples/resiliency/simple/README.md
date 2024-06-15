This example validates the resiliency and does not demonstrate any extra
functionality. It is based off the configuration example to connect to the
sidecar and make a call for a configuration item stored in redis.

1. Insert a key with the value `hello` to redis using the following command:


<!-- STEP
name: Insert test configuration item
output_match_mode: substring
expected_stdout_lines:
  - 'OK'
background: false
sleep: 5
timeout_seconds: 5
-->

```bash
docker exec dapr_redis redis-cli MSET hello "world"
```

<!-- END_STEP -->

2. Run the example without the sidecar using the following command:

<!-- STEP
name: Run configuration app (expecting a fail)
env:
  DAPR_GRPC_PORT: "3500"
  DAPR_API_MAX_RETRIES: "1"
  DAPR_API_TIMEOUT_MILLISECONDS: "10000"
output_match_mode: substring
expected_stdout_lines:
  - ''
expected_stderr_lines:
  - 'ConnectError'
expected_return_code: 101
background: false
sleep: 30
timeout_seconds: 30
-->

```bash
cargo run --example resiliency-simple
```

<!-- END_STEP -->

The result should be that the request will fail.

3. Run the example without the sidecar (this time in the background)

<!-- STEP
name: Run configuration app (expecting a success eventually)
env:
  DAPR_GRPC_PORT: "3500"
  DAPR_API_MAX_RETRIES: "10"
  DAPR_API_TIMEOUT_MILLISECONDS: "10000"
output_match_mode: substring
expected_stdout_lines:
  - 'Configuration value: ConfigurationItem { value: "world"'
background: true
sleep: 30
timeout_seconds: 30
-->

```bash
cargo run --example resiliency-simple
```

<!-- END_STEP -->



4. Run the Dapr sidecar

<!-- STEP
name: Run Dapr sidecar
output_match_mode: substring
expected_stdout_lines:
  - ''
background: true
sleep: 15
timeout_seconds: 15
-->

```bash
dapr run --app-id=rustapp --resources-path .../components --dapr-grpc-port 3500
```

<!-- END_STEP -->

The example app should make contact with the Dapr sidecar and the result should
be returned from the configuration request successfully.

```
Configuration value: ConfigurationItem { value: "world", version: "", metadata: {} }
```
