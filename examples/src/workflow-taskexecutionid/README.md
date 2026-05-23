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

The workflow schedules the `RetryN` activity twice with a retry policy. `RetryN`
uses the activity task execution ID as a deterministic key so replay does not
increment the retry counter, fails until the final retry attempt, and then
completes successfully.
