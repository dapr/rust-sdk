# Distributed Lock

This is a simple example that demonstrates Dapr's Distributed Lock capabilities.

> **Note:** Make sure to use latest version of proto bindings.

## Running

To run this example:

1. Run the multi-app run template:

<!-- STEP
name: Run multi-app
output_match_mode: substring
match_order: sequential
expected_stdout_lines:
  - '== APP - distributed-lock-example == Successfully acquired lock on: resource'
  - '== APP - distributed-lock-example == Unsuccessfully acquired lock on: resource'
  - '== APP - distributed-lock-example == Successfully released lock on: resource'
  - '== APP - distributed-lock-example == Successfully acquired lock on: resource'
background: true
sleep: 30
timeout_seconds: 90
-->

```bash
dapr run -f .
```

<!-- END_STEP -->

2. Stop with `ctrl + c`
