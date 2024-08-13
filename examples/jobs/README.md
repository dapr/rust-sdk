# Jobs Example

This is a simple example that demonstrates Dapr's job scheduling capabilities.

## Running

To run this example:

1. Run the multi-app run template:

<!-- STEP
name: Run multi-app
output_match_mode: substring
match_order: none
expected_stdout_lines:
  - 'job scheduled successfully'
  - 'job received'
  - 'job received'
  - 'job received'
  - 'received job on ping_pong_handler'
  - 'received job on ping_pong_handler'
  - 'received job on ping_pong_handler'
  - 'received job on ping_pong_handler'
  - 'received job on ping_pong_handler'
background: true
sleep: 30
timeout_seconds: 30
-->

```bash
dapr run -f .
```

<!-- END_STEP -->

2. Stop with `ctrl + c`
