Before you run the example make sure local redis state store is running by executing:
```
docker ps
```

1. To run the example we need to first create the state store component yaml.
```bash
mkdir components 
touch components/statestore.yaml
```

2. The statestore.yaml should contain the connection information for your statestore. Paste the follwoing content into it:
```yaml
apiVersion: dapr.io/v1alpha1
kind: Component
metadata:
  name: statestore
spec:
  type: state.redis
  metadata:
  - name: redisHost
    value: localhost:6379
  - name: redisPassword
    value: ""
  - name: actorStateStore
    value: "true"
```

3. Now run daprd with the following command:
```
daprd -app-id my-app -app-port 8080 -dapr-grpc-port 3800
```

4. In a new terminal run the example client from the projects root:
```
cargo run --example client
```

If everything went well you should see the following output:
```
Successfully saved!
Value is "world"
```

