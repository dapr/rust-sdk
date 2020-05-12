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
dapr run --app-id=rustapp --grpc-port 3500 cargo run -- --example client
```

If everything went well you should see the following output along with dapr logs:
```
Successfully saved!
Value is "world"
```

