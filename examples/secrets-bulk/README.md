Before you run the example make sure local redis state store is running by executing:
```
docker ps
```

1. To run the example we need to first build the examples using the following command:

```
cargo build --examples
```

2. Run the example with dapr using the following command:

<!-- STEP
name: Run app example
output_match_mode: substring
match_order: none
expected_stdout_lines:
  - '== APP == Found secret1 with value: TestSecret1'
  - '== APP == Found secret2 with value: TestSecret2'
  - '== APP == Found secret3 with value: TestSecret3'
background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=rustapp --dapr-grpc-port 3500 --resources-path ./resources/ cargo run -- --example secrets-bulk
```

<!-- END_STEP -->

If everything went well you should see the following output along with dapr logs:
```
== APP == Found secret1 with value: TestSecret1
== APP == Found secret2 with value: TestSecret2
== APP == Found secret3 with value: TestSecret3
```
_Note: The order of the secrets returned is not ordered_
