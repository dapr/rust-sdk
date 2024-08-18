Before you run the example make sure local redis state store is running by executing:
```
docker ps
```

1. To run the example we need to first build the examples using the following command:

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

2. Run the example with dapr using the following command:

<!-- STEP
name: Run client example
output_match_mode: substring
expected_stdout_lines:
  - '== APP == Successfully saved!'
  - '== APP == Value is "world"'
  - '== APP == Deleted value: []'
background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=rustapp --dapr-grpc-port 3500 --resources-path ./resources cargo run -- --example client
```

<!-- END_STEP -->

If everything went well you should see the following output along with dapr logs:
```
Successfully saved!
Value is "world"
Deleted value: []
```

