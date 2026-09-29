# Dapr Vector (alpha1) Example with the Rust-SDK

This example upserts pre-computed embeddings into a Meilisearch collection, then runs single, by-ID and batched similarity queries against it.

The vector building block is alpha and requires a Dapr runtime that includes it.

## Step

### Prepare

- Dapr installed
- Meilisearch running locally:

<!-- STEP
name: Start Meilisearch
background: false
sleep: 5
timeout: 60
-->

```bash
docker run -d --name meilisearch-vector -p 7700:7700 -e MEILI_MASTER_KEY=masterKey getmeili/meilisearch:latest
```

<!-- END_STEP -->

### Run Vector Example

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

2. Run the example using the Dapr CLI

<!-- STEP
name: Run Vector
output_match_mode: substring
expected_stdout_lines:
  - 'collection created: cars'
  - 'query match: roadster'
  - 'batch query 1 match: hatchback'
  - 'got vector: pickup'
  - 'collection deleted: cars'

background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=vector --resources-path ./config --dapr-grpc-port 50001 -- cargo run --example vector
```

<!-- END_STEP -->

3. Stop Meilisearch

<!-- STEP
name: Stop Meilisearch
background: false
sleep: 1
timeout: 30
-->

```bash
docker rm -f meilisearch-vector
```

<!-- END_STEP -->

## Result

```
  - 'collection created: cars'
  - 'query match: roadster'
  - 'batch query 1 match: hatchback'
  - 'got vector: pickup'
  - 'collection deleted: cars'
```
