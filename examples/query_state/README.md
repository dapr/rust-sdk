# Query state Example
To run this example, the default local redis state store will not work as it does not support redis-json server. You will encounter the following error
```
 GrpcError(GrpcError { _status: Status { code: Internal, message: "failed query in state store statestore: redis-json server support is required for query capability", metadata: MetadataMap { headers: {"content-type": "application/grpc", "grpc-trace-bin": "AABniqIo9TrSF6TepfB0yzgNAZzAwpG45zK0AgE"} }, source: None } })
```

Instead, we will be following the query state example in the [Dapr docs](https://docs.dapr.io/developing-applications/building-blocks/state-management/howto-state-query-api/#example-data-and-query) and will be using mongo instead.

To setup MongoDB, execute the following command:
```
docker run -d --rm -p 27017:27017 --name mongodb mongo:5
```

You can then apply the statestore configuration using the `statestore/mongodb.yaml` file. 

Then, execute the following commands to populate the state data in the statestore:
```
dapr run --app-id demo --dapr-http-port 3500 --resources-path statestore/
```
In a new terminal:

```
curl -X POST -H "Content-Type: application/json" http://localhost:3500/v1.0/state/statestore -d @./statestore/dataset.json
``````

1. To run the example we need to first build the examples using the following command:

```
cargo build --examples
```

2. Executing the first query
Query:
```json
{
    "filter": {
        "EQ": { "state": "CA" }
    },
    "sort": [
        {
            "key": "person.id",
            "order": "DESC"
        }
    ]
}

```
Execute the first state query using the following command:
```
dapr run --app-id=rustapp --dapr-grpc-port 3501  cargo run -- --example query_state_q1
```
Expected result:
```
Query results: [Object {"id": String("3"), "value": String("{\"city\":\"Sacramento\",\"state\":\"CA\",\"person\":{\"org\":\"Finance\",\"id\":1071.0}}")}, Object {"id": String("7"), "value": String("{\"person\":{\"org\":\"Dev Ops\",\"id\":1015.0},\"city\":\"San Francisco\",\"state\":\"CA\"}")}, Object {"id": String("5"), "value": String("{\"person\":{\"org\":\"Hardware\",\"id\":1007.0},\"city\":\"Los Angeles\",\"state\":\"CA\"}")}, Object {"id": String("9"), "value": String("{\"person\":{\"org\":\"Finance\",\"id\":1002.0},\"city\":\"San Diego\",\"state\":\"CA\"}")}]
```

3. Executing the second query
Query:
```json
{
    "filter": {
        "IN": { "person.org": [ "Dev Ops", "Hardware" ] }
    }
}
```
Execute the second state query using the following command:
```
dapr run --app-id=rustapp --dapr-grpc-port 3501  cargo run -- --example query_state_q1
```
Expected result:
```


 Query results: [Object {"id": String("5"), "value": String("{\"person\":{\"org\":\"Hardware\",\"id\":1007.0},\"city\":\"Los Angeles\",\"state\":\"CA\"}")}, Object {"id": String("2"), "value": String("{\"person\":{\"id\":1028.0,\"org\":\"Hardware\"},\"city\":\"Portland\",\"state\":\"OR\"}")}, Object {"id": String("4"), "value": String("{\"person\":{\"org\":\"Dev Ops\",\"id\":1042.0},\"city\":\"Spokane\",\"state\":\"WA\"}")}, Object {"id": String("7"), "value": String("{\"person\":{\"org\":\"Dev Ops\",\"id\":1015.0},\"city\":\"San Francisco\",\"state\":\"CA\"}")}, Object {"id": String("8"), "value": String("{\"city\":\"Redmond\",\"state\":\"WA\",\"person\":{\"id\":1077.0,\"org\":\"Hardware\"}}")}, Object {"id": String("10"), "value": String("{\"person\":{\"org\":\"Dev Ops\",\"id\":1054.0},\"city\":\"New York\",\"state\":\"NY\"}")}, Object {"id": String("1"), "value": String("{\"person\":{\"org\":\"Dev Ops\",\"id\":1036.0},\"city\":\"Seattle\",\"state\":\"WA\"}")}]
```