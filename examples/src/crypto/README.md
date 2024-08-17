# Crypto Example

This is a simple example that demonstrates Dapr's Cryptography capabilities.

> **Note:** Make sure to use latest version of proto bindings.

## Running

To run this example:

1. Generate keys in examples/crypto/keys directory:
<!-- STEP
name: Generate keys
background: false
sleep: 5
timeout_seconds: 30
-->
```bash
mkdir -p keys
# Generate a private RSA key, 4096-bit keys
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:4096 -out keys/rsa-private-key.pem
# Generate a 256-bit key for AES
openssl rand -out keys/symmetric-key-256 32
```

<!-- END_STEP -->

2. Run the multi-app run template:

<!-- STEP
name: Run multi-app
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
dapr run -f .
```

<!-- END_STEP -->

2. Stop with `ctrl + c`
