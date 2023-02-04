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
dapr run --app-id=invoke-grpc-server --app-protocol grpc --app-port 50052 -- cargo run --example invoke-grpc-proxying-server
dapr run --app-id=invoke-grpc-client -- cargo run --example invoke-grpc-proxying-client
```

If everything went well you should see the following output along with dapr logs:
```
Response: HelloReply {
    message: "Hello Test!",
}
```
