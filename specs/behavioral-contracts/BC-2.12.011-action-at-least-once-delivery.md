---
document_type: behavioral-contract
level: L3
version: "1.0"
status: removed
lifecycle_status: retired
deprecated_by: "v3-patch-burst-4b"
replacement: "BC-2.18.001"
deprecated: "2026-04-16"
producer: product-owner
timestamp: 2026-04-16T22:00:00
phase: 3
origin: greenfield
subsystem: "Scheduler"
capability: "CAP-021"
---

> **RETIRED (2026-04-16):** Superseded by BC-2.18.001 (Action Delivery Engine subsystem, INV-ACTION-001).
> BC-2.12.011 was a high-level cross-subsystem summary written before subsystem 18 was established.
> BC-2.18.001 is the normative specification. In all conflicts, BC-2.18.001 wins.
> This file is retained for historical traceability only.

# BC-2.12.011: Action At-Least-Once Delivery with Retry

## Preconditions
- An action trigger fires (alert, case event, schedule, or manual)
- The action spec is loaded and valid

## Postconditions
- Action delivery attempts at least once per trigger event
- On transient failure (HTTP 5xx, timeout, connection refused): retry with exponential backoff (base 2s, max 60s, max 5 attempts total)
- On permanent failure (HTTP 4xx, invalid template): no retry, write to dead-letter in action_state CF
- Successful delivery: write delivery receipt to action_state CF with timestamp and response summary
- Failed delivery after all retries: write dead-letter entry to action_state CF with error detail
- Every delivery attempt (success or failure) produces an audit entry (DI-004)

## Invariants
- At-least-once: a trigger event may produce duplicate deliveries (network ack lost) but never zero deliveries unless all retries exhausted
- Dead-letter entries are queryable via prism_action_state internal table
- Credential values used for action delivery (webhook tokens, SMTP passwords) are resolved via CredentialStore reference — never stored inline in action specs (DI-002)
