This example validates the resiliency of the instantiated client and does not 
demonstrate any extra functionality. It is based off the configuration example 
to connect to the sidecar and make a call for a configuration item stored in 
redis. 

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

2. Run the example without the sidecar

<!-- STEP
name: Run configuration app
env:
  DAPR_GRPC_PORT: "3500"
  DAPR_API_MAX_RETRIES: "10"
  DAPR_API_TIMEOUT_MILLISECONDS: "10000"
output_match_mode: substring
expected_stdout_lines:
  - 'Configuration value: ConfigurationItem { value: "world"'
  - 'Configuration value: ConfigurationItem { value: "world2"'
background: true
sleep: 30
timeout_seconds: 30
-->

```bash
cargo run --example resiliency-instance
```

<!-- END_STEP -->

3. Run the Dapr sidecar

<!-- STEP
name: Run Dapr sidecar
output_match_mode: substring
expected_stdout_lines:
  - ''
background: false
sleep: 10
timeout_seconds: 10
-->

```bash
dapr run --app-id=rustapp --resources-path ../../components --dapr-grpc-port 3500
```

<!-- END_STEP -->

4. Update the hello key with the value `world2` to redis using the following command:


<!-- STEP
name: Update test configuration item
output_match_mode: substring
expected_stdout_lines:
  - 'OK'
background: false
sleep: 5
timeout_seconds: 5
-->

```bash
docker exec dapr_redis redis-cli MSET hello "world2"
```

<!-- END_STEP -->

5. Run the Dapr sidecar (for the second time)

<!-- STEP
name: Run Dapr sidecar
output_match_mode: substring
expected_stdout_lines:
  - ''
background: true
sleep: 10
timeout_seconds: 10
-->

```bash
dapr run --app-id=rustapp --resources-path ../../components --dapr-grpc-port 3500
```

<!-- END_STEP -->
The example app should make contact with the Dapr sidecar and the result should
be returned from the configuration request successfully.

```
Configuration value: ConfigurationItem { value: "world", version: "", metadata: {} }
```
