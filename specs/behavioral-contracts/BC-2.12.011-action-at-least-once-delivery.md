---
document_type: behavioral-contract
level: L3
version: "1.0"
status: retired
lifecycle_status: retired
producer: product-owner
timestamp: 2026-04-16T22:00:00
phase: 2-patch
inputs: []
input-hash: "n/a"
traces_to: ""
extracted_from: ""
origin: greenfield
subsystem: "SS-12"
capability: "CAP-021"
introduced: cycle-1
modified: [phase-2-patch-burst-51]
deprecated: "2026-04-16"
deprecated_by: "v3-patch-burst-4b"
retired: "2026-04-16"
removed: null
removal_reason: null
replacement: "BC-2.18.001"
---

> **RETIRED (2026-04-16):** Superseded by BC-2.18.001 (Action Delivery Engine subsystem, INV-ACTION-001).
> BC-2.12.011 was a high-level cross-subsystem summary written before subsystem 18 was established.
> BC-2.18.001 is the normative specification. In all conflicts, BC-2.18.001 wins.
> This file is retained for historical traceability only.

# BC-2.12.011: Action At-Least-Once Delivery with Retry

## Description

RETIRED. This contract specified that action delivery (webhook, email, script) must be
attempted at least once per trigger event, with exponential-backoff retry on transient
failure and dead-letter persistence on permanent failure. It was written as a
cross-subsystem summary before SS-18 (Action Delivery Engine) was established.
The normative successor is BC-2.18.001.

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

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RETIRED — see BC-2.18.001 | n/a |

## Canonical Test Vectors

> RETIRED — see BC-2.18.001 for active test vectors.

| Input | Expected Output | Category |
|-------|----------------|----------|
| RETIRED | see BC-2.18.001 | n/a |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | RETIRED — see BC-2.18.001 | n/a |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 |
| L2 Domain Invariants | DI-002, DI-004 |
| Architecture Module | SS-12 (historical); normative owner: SS-18 Action Delivery Engine |
| Stories | — (retired before story assignment) |

## Changelog

| Version | Burst | Finding | Change |
|---------|-------|---------|--------|
| 1.0 | cycle-1 / Burst 4b | — | Created as cross-subsystem summary for Action Delivery; retired 2026-04-16 when SS-18 (Action Delivery Engine) established; superseded by BC-2.18.001 |
| 1.0 | Burst 51 | P3P50-A-MED-001 | Frontmatter `status: removed` corrected to `status: retired` — 3-way consistency fix (lifecycle_status, body RETIRED prose, and BC-INDEX Status col all canonical as `retired`); no semantic change. Template conformance fields added. |
