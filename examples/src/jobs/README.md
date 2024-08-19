# Jobs Example

This is a simple example that demonstrates Dapr's job scheduling capabilities.

## Running

To run this example:

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

2. Run the multi-app run template:

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

3. Stop with `ctrl + c`
