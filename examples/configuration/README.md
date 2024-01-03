Before you run the example make sure local redis state store is running by executing:
```bash
docker ps
```

1. To run the example we need to first build the examples using the following command:

```bash
cargo build --examples
```

2. Run the example with dapr using the following command:

```bash
dapr run --app-id=rustapp --resources-path ./examples/components --dapr-grpc-port 3500 -- cargo run --example configuration
```

3. Change the value of the key `hello` in redis using the following command:

```bash
docker exec dapr_redis redis-cli MSET hello "world"
```


If everything went well you should see the following output along with dapr logs:
```
Configuration value: ConfigurationItem { value: "world", version: "", metadata: {} }
App subscribed to config changes with subscription id: "d383169a-0893-4c64-adde-fc3145b56d07" 
Configuration value: {"hello": ConfigurationItem { value: "world", version: "", metadata: {} }}
App unsubscribed from config changes
```


