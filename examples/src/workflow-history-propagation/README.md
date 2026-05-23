# Dapr Workflow History Propagation Example

This example demonstrates how workflows can propagate their execution history to child workflows and activities, enabling downstream consumers to inspect the full (or partial) execution context of their caller.

## Workflow Architecture

```text
MerchantCheckout (workflow)
├── ValidateMerchant (activity, no propagation)
└── ProcessPayment (child workflow, Lineage)
    ├── ValidateCard (activity, no propagation)
    ├── CheckSpendingLimits (activity, no propagation)
    ├── FraudDetection (child workflow, Lineage)
    │     → sees MerchantCheckout + ProcessPayment events
    └── SettlePayment (activity, OwnHistory)
          → sees ProcessPayment events only
```

## Propagation Scope

| Mode | What it sends | Use case |
|------|--------------|----------|
| `HistoryPropagationScope::Lineage` | Caller's own events + any ancestor events it received | Full chain-of-custody verification |
| `HistoryPropagationScope::OwnHistory` | Caller's own events only (no ancestor chain) | Trust boundary — downstream only sees the immediate caller |

## Running the Example

First-time setup: run `dapr init` if you haven't already. From this directory, run:

<!-- STEP
name: Run Workflow
output_match_mode: substring
expected_stdout_lines:
  - 'WORKFLOW HISTORY PROPAGATION DEMO'
  - '[MerchantCheckout] Starting checkout'
  - '[ValidateMerchant] Validating merchant'
  - '[ProcessPayment] Starting payment'
  - '[FraudDetection] Checking payment'
  - '= COMPLETE ='

background: true
sleep: 30
timeout_seconds: 60
-->

```bash
dapr run --app-id workflow-history-propagation --resources-path ./config -- cargo run --example workflow-history-propagation
```

<!-- END_STEP -->

Expected output includes:

```text
WORKFLOW HISTORY PROPAGATION DEMO
[MerchantCheckout] Starting checkout
[ValidateMerchant] Validating merchant
[ProcessPayment] Starting payment
events (scope: LINEAGE)
[FraudDetection] APPROVED
scope=OWN_HISTORY
[SettlePayment] SETTLED
= COMPLETE =
```

Standalone propagation may log warnings that chunks are unsigned. Those warnings are expected without Kubernetes mTLS and workflow history signing.

## Troubleshooting

- **FraudDetection reports 0 events** — confirm the parent calls use `.with_history_propagation(HistoryPropagationScope::Lineage)`.
- **SettlePayment sees MerchantCheckout events** — confirm it uses `HistoryPropagationScope::OwnHistory` rather than `Lineage`.
- **Sidecar connection errors** — start through `dapr run` so `WorkflowClient::new()` can connect to the Dapr gRPC endpoint.

## Files

```text
workflow-history-propagation/
├── README.md
├── main.rs
└── config/
    └── redis.yaml
```
