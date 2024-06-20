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
name: Run Multi-app
output_match_mode: substring
match_order: none
expected_stdout_lines:
  - '== APP - invoke-grpc-server == AppCallback server listening on: [::]:50051'
  - '== APP - invoke-grpc-client == Response: HelloReply {'
  - '== APP - invoke-grpc-client ==     message: "Hello Test!",'
  - '== APP - invoke-grpc-client == }'
background: true
sleep: 30
timeout_seconds: 90
-->

```bash
dapr run -f .
```

<!-- END_STEP -->

What the multi-run step effectively runs for you:
1. Runs the invoke-grpc-server:
```bash
dapr run --app-id=invoke-grpc-server --app-protocol grpc --app-port 50051 -- cargo run --example invoke-grpc-proxying-server
```

2. Runs the invoke-grpc-client:
```bash
dapr run --app-id=invoke-grpc-client -- cargo run --example invoke-grpc-proxying-client
```

If everything went well you should see the following output along with dapr logs:
```
Response: HelloReply {
    message: "Hello Test!",
}
```
