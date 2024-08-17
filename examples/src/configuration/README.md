Before you run the example make sure local redis state store is running by executing:
```bash
docker ps
```

1. To run the example we need to first build the examples using the following command:

```bash
cargo build --examples
```

2. Insert a key with the value `hello` to redis using the following command:


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

3. Run the example with dapr using the following command:

<!-- STEP
name: Run configuration app
output_match_mode: substring
expected_stdout_lines:
  - '== APP == Configuration value: ConfigurationItem { value: "world"'
  - '== APP == App subscribed to config changes with subscription id:'
  - '== APP == Configuration value: {"hello": ConfigurationItem { value: "world2"'
  - '== APP == App unsubscribed from config changes'
background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=rustapp --resources-path ../components --dapr-grpc-port 3500 -- cargo run --example configuration
```

<!-- END_STEP -->

4. Change the value of the key `hello` in redis using the following command:

<!-- STEP
name: Update test configuration item
output_match_mode: substring
expected_stdout_lines:
  - 'OK'
background: true
sleep: 5
timeout_seconds: 5
-->

```bash
docker exec dapr_redis redis-cli MSET hello "world2"
```

<!-- END_STEP -->


If everything went well you should see the following output along with dapr logs:
```
Configuration value: ConfigurationItem { value: "world", version: "", metadata: {} }
App subscribed to config changes with subscription id: "d383169a-0893-4c64-adde-fc3145b56d07" 
Configuration value: {"hello": ConfigurationItem { value: "world2", version: "", metadata: {} }}
App unsubscribed from config changes
```


