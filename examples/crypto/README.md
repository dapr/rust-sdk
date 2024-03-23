# Crypto Example

This is a simple example that demonstrates Dapr's Cryptography capabilities.

> **Note:** Make sure to use latest version of proto bindings.

## Running

> Before you run the example make sure generate keys in examples/crypto/keys directory:
> ```
> mkdir -p keys
> # Generate a private RSA key, 4096-bit keys
> openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:4096 -out keys/rsa-private-key.pem
> # Generate a 256-bit key for AES
> openssl rand -out keys/symmetric-key-256 32
> ```

To run this example:

1. Run the multi-app run template:

<!-- STEP
name: Run Subscriber
output_match_mode: substring
match_order: none
expected_stdout_lines:
  - '== APP - crypto-example == Successfully Decrypted String'
  - '== APP - crypto-example == Successfully Decrypted Image'
background: true
sleep: 30
timeout_seconds: 90
-->

```bash
mkdir -p keys
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:4096 -out keys/rsa-private-key.pem
openssl rand -out keys/symmetric-key-256 32
dapr run -f .
```

<!-- END_STEP -->

2. Stop with `ctrl + c`
