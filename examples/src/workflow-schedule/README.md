# Dapr Workflow Schedule Example

A workflow schedule starts a workflow on a cron schedule; each run is a new
workflow instance whose ID starts with `<schedule name>_`. This example
creates a schedule for `GreetingWorkflow`, triggers a run by hand, waits for
the schedule to complete, lists its runs and deletes it.

## Step

### Prepare

- Dapr 1.20 or later (dapr master until 1.20.0-rc.1 is released)
- Redis available at `localhost:6379` for the workflow state store

### Build

```bash
cargo build -p examples --example workflow-schedule --features dapr/workflow
```

### Run Workflow Schedule

<!-- STEP
name: Run Workflow Schedule
output_match_mode: substring
expected_stdout_lines:
  - 'GreetingWorkflow registered'
  - 'Worker started'
  - 'schedule created'
  - 'manual run triggered: true'
  - 'schedule completed after 2 runs'
  - 'schedule listed: true'
  - 'run output: "Hello Dapr!"'
  - 'run output: "Hello Dapr!"'
  - 'runs completed: 2'
  - 'schedule deleted'
  - 'workflow worker successfully shutdown'

background: true
sleep: 30
timeout_seconds: 60
-->

```bash
dapr run --app-id workflow-schedule \
         --resources-path ./config \
         -- cargo run --example workflow-schedule
```

<!-- END_STEP -->

## Result

The schedule is created with two repeats, triggered once by hand, completes
after the triggered run and one scheduled run, is listed, its runs are waited
for and purged, and it is deleted.
