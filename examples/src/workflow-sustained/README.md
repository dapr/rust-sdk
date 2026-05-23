# Dapr Sustained Workflow Example with rust-sdk

This example stress-tests Dapr workflow scheduling by starting many concurrent `SustainedWorkflow` instances. Each workflow calls one `DoWork` activity, which sleeps for a short varied interval and returns `input * 2`.

## Configuration

Environment variables:

- `WORKFLOW_COUNT`: number of workflow instances to schedule. Defaults to `100` and is capped at `10000` for safety.
- `WORKFLOW_CONCURRENCY`: maximum number of in-flight scheduling/waiting tasks. Defaults to `WORKFLOW_COUNT` so all workflows start in parallel.

Dapr workflows pin to a Redis state store named `wf-store`. If running multiple examples at once, give them distinct app IDs; they can share the store.

## Run

Start Redis locally on port 6379, then run:

<!-- STEP
name: Run Workflow
output_match_mode: substring
expected_stdout_lines:
  - 'Workflow(s) and activities registered.'
  - 'Worker initialized'
  - 'Sustained workflow summary'
  - 'succeeded:'
  - 'throughput:'
  - 'latency avg:'

background: true
sleep: 30
timeout_seconds: 120
-->

```bash
WORKFLOW_COUNT=5 \
dapr run --app-id workflow-sustained \
         --resources-path ./config \
         -- cargo run --example workflow-sustained
```

<!-- END_STEP -->

Example with explicit load settings:

```bash
WORKFLOW_COUNT=1000 WORKFLOW_CONCURRENCY=100 \
dapr run --app-id workflow-sustained \
         --resources-path ./config \
         -- cargo run --example workflow-sustained
```

The program prints total count, succeeded/failed counts, total elapsed time, throughput, and latency average/min/p50/p95/p99/max.
