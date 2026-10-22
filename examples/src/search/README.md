# Dapr Search (alpha1) Example with the Rust-SDK

This example indexes documents in a Meilisearch index, then runs a filtered and sorted full-text search against it.

The search building block is alpha and requires a Dapr runtime that includes it.

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
docker run -d --name meilisearch-search -p 7700:7700 -e MEILI_MASTER_KEY=masterKey getmeili/meilisearch:latest
```

<!-- END_STEP -->

### Run Search Example

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
name: Run Search
output_match_mode: substring
expected_stdout_lines:
  - 'index created: cars'
  - 'search hit: 2 electric hatchback'
  - 'search hit: 1 electric roadster'
  - 'got document: 3 diesel pickup'
  - 'index deleted: cars'

background: true
sleep: 15
timeout_seconds: 30
-->

```bash
dapr run --app-id=search --resources-path ./config --dapr-grpc-port 50001 -- cargo run --example search
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
docker rm -f meilisearch-search
```

<!-- END_STEP -->

## Result

```
  - 'index created: cars'
  - 'search hit: 2 electric hatchback'
  - 'search hit: 1 electric roadster'
  - 'got document: 3 diesel pickup'
  - 'index deleted: cars'
```
