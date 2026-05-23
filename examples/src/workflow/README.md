# Dapr Workflow Example

## Step

### Prepare

- Dapr installed
- Redis available at `localhost:6379` for the workflow state store

### Build

```bash
cargo build -p examples --example workflow --features dapr/workflow
```

### Run Workflow

<!-- STEP
name: Run Workflow
output_match_mode: substring
expected_stdout_lines:
  - 'TestWorkflow registered'
  - 'TestActivity registered'
  - 'FailActivity registered'
  - 'Worker initialized'
  - 'runner started'
  - 'workflow started with id: a7a4168d-3a1c-41da-8a4f-e7f6d9c718d9'
  - 'workflow paused'
  - 'workflow resumed'
  - 'stage: 1'
  - 'workflow event raised'
  - 'stage: 2'
  - 'fail activity executions: 3'
  - 'workflow status: Completed'
  - 'workflow purged'
  - 'stage: 2'
  - 'workflow started with id: a7a4168d-3a1c-41da-8a4f-e7f6d9c718d9'
  - 'workflow status: Running'
  - 'workflow terminated'
  - 'workflow purged'
  - 'workflow worker successfully shutdown'

background: true
sleep: 60
timeout_seconds: 60
-->

```bash
dapr run --app-id workflow \
         --resources-path ./config \
         -- cargo run --example workflow
```

<!-- END_STEP -->

## Result

The workflow is scheduled, suspended, resumed, sent an external event, completed,
purged, then scheduled again, terminated, and purged.
