# Invoke HTTP Example

This example demonstrates how to use the Dapr HTTP proxying feature to proxy HTTP requests sent via the Rust SDK (using `reqwest`) through Dapr to reach another application.

1. Build the examples:
```bash
cargo build --examples
```

2. Run with Dapr in the example/invoke/http directory:
```bash
dapr run -f .
```

If everything worked, you should see the `invoke-http-client` successfully sending an HTTP request via the Dapr sidecar to the `invoke-http-server`, with `"Hello, test! (from HTTP server)"` printed to the stdout.
