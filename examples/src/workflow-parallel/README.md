# Dapr Parallel Workflow Example with rust-sdk

## Step

### Prepare

- Dapr installed
- Redis running locally on port 6379

### Run Workflow

<!-- STEP
name: Run Workflow
output_match_mode: substring
expected_stdout_lines:
  - 'Workflow(s) and activities registered.'
  - 'Processing work item: 9'
  - 'Work item 9 processed. Result: 18'
  - 'Final result: 90'
  - 'workflow status: COMPLETED'
  - 'workflow terminated'
  - 'workflow purged'

background: true
sleep: 30
timeout_seconds: 60
-->

```bash
dapr run --app-id workflow-parallel \
         --dapr-grpc-port 50001 \
         --log-level debug \
         --resources-path ./config \
         -- cargo run -p examples --example workflow-parallel
```

<!-- END_STEP -->

## Result

```
Workflow(s) and activities registered.
Processing work item: 9
Work item 9 processed. Result: 18
Final result: 90
workflow status: COMPLETED
workflow terminated
workflow purged
```
