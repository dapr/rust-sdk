# Dapr Task Execution ID Workflow Example

## Step

### Prepare

- Dapr installed
- Redis available at `localhost:6379` for the workflow state store

### Build

```bash
cargo build -p examples --example workflow-taskexecutionid
```

### Run Workflow

<!-- STEP
name: Run Workflow
output_match_mode: substring
expected_stdout_lines:
  - 'Workflow(s) and activities registered.'
  - 'Worker initialized'
  - 'RetryN  1'
  - 'RetryN  2'
  - 'RetryN  3'
  - 'RetryN  4'
  - 'RetryN  5'
  - 'RetryN  1'
  - 'RetryN  2'
  - 'RetryN  3'
  - 'RetryN  4'
  - 'RetryN  5'
  - 'workflow status: Completed'
  - 'workflow terminated'
  - 'workflow purged'

background: true
sleep: 30
timeout_seconds: 60
-->

```bash
dapr run --app-id workflow-taskexecutionid \
         --dapr-grpc-port 50001 \
         --log-level debug \
         --resources-path ./config \
         -- cargo run --example workflow-taskexecutionid
```

<!-- END_STEP -->

## Result

The workflow schedules the `RetryN` activity twice with a retry policy. Each
invocation is tagged with a unique `invocation_id` that the workflow includes
in the activity input. Because Dapr replays the same input on every retry of a
given scheduled task, `RetryN` combines `orchestration_id` and `invocation_id`
into a deterministic deduplication key that is stable across retries of one logical
activity call but differs between distinct invocations. The activity fails
until the final retry attempt, then completes successfully — producing two
independent `RetryN 1..5` sequences.
