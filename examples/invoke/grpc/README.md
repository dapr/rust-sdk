Before you run the example make sure local redis state store is running by executing:
```
docker ps
```

1. To run the example we need to first build the examples using the following command:

```
cargo build --examples
```

2. Run the example with dapr using the following command:

```
dapr run --app-id=invoke-grpc-server --app-protocol grpc --app-port 50052 -- cargo run --example invoke-grpc-server
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

