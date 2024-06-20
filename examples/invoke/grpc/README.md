Before you run the example make sure local redis state store is running by executing:
```
docker ps
```

1. To run the example we need to first build the examples using the following command:

```
cargo build --examples
```

2. Run the example with dapr using the following command to start the multi-app run:

<!-- STEP
name: Run Multi-app
output_match_mode: substring
match_order: sequential
expected_stdout_lines:
  - '== APP - invoke-grpc-server == Method: say_hello'
  - '== APP - invoke-grpc-server == Name: "Test"'
  - '== APP - invoke-grpc-client == Message: "Hello World!"'
  - '== APP - invoke-grpc-client == Response: InvokeResponse {'
  - '== APP - invoke-grpc-client ==     data: Some('
  - '== APP - invoke-grpc-client ==         Any {'
  - '== APP - invoke-grpc-client ==             type_url: "",'
  - '== APP - invoke-grpc-client ==             value: ['
  - '== APP - invoke-grpc-client ==                 10,'
  - '== APP - invoke-grpc-client ==                 12,'
  - '== APP - invoke-grpc-client ==                 72,'
  - '== APP - invoke-grpc-client ==                 101,'
  - '== APP - invoke-grpc-client ==                 108,'
  - '== APP - invoke-grpc-client ==                 108,'
  - '== APP - invoke-grpc-client ==                 111,'
  - '== APP - invoke-grpc-client ==                 32,'
  - '== APP - invoke-grpc-client ==                 87,'
  - '== APP - invoke-grpc-client ==                 111,'
  - '== APP - invoke-grpc-client ==                 114,'
  - '== APP - invoke-grpc-client ==                 108,'
  - '== APP - invoke-grpc-client ==                 100,'
  - '== APP - invoke-grpc-client ==                 33,'
  - '== APP - invoke-grpc-client ==             ],'
  - '== APP - invoke-grpc-client ==         },'
  - '== APP - invoke-grpc-client ==     ),'
  - '== APP - invoke-grpc-client ==     content_type: "application/json",'
  - '== APP - invoke-grpc-client == }'
background: true
sleep: 30
timeout_seconds: 90
-->
== APP - invoke-grpc-server == Method: say_hello
		== APP - invoke-grpc-server == Name: "Test"
```bash
dapr run -f .
```

<!-- END_STEP -->

The multi-app run is the equivalent of running: 
1. The server application with dapr
```bash
dapr run --app-id=invoke-grpc-server --app-protocol grpc --app-port 50051 -- cargo run --example invoke-grpc-server
```

2. The client application
```bash
dapr run --app-id=invoke-grpc-client -- cargo run --example invoke-grpc-client
```

If everything went well you should see the following output along with dapr logs:
```
Message: "Hello World!"
Response: InvokeResponse {
    data: Some(
        Any {
            type_url: "",
            value: [
                10,
                12,
                72,
                101,
                108,
                108,
                111,
                32,
                87,
                111,
                114,
                108,
                100,
                33,
            ],
        },
    ),
    content_type: "application/json",
}
```

