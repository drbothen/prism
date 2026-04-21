---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-18"
capability: "CAP-033"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "ac6b633"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.001: Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry

## Description

When an alert or case event triggers an action, the delivery is guaranteed at-least-once.
Failed deliveries are retried with exponential backoff (maximum 5 attempts, base 2 seconds,
maximum 60 seconds per attempt). Retry state is persisted to RocksDB `action_state` CF
before each attempt. After 5 failures, a dead-letter record is written and an audit event
is emitted. The source alert is NOT lost (it remains in the `alerts` CF). This is INV-ACTION-001.

## Preconditions

- `ActionEngine` is initialized with a registered `ActionSpec` with `trigger = "alert"` or
  `trigger = "case"`
- An alert or case event matching the action's `clients` and `filter` criteria is broadcast
- The destination (webhook, email, syslog, or plugin) is temporarily unavailable or returns
  a retryable error

## Postconditions

- **Retryable failure (HTTP 429, 5xx, network error):**
  - Retry state is written to `action_state` CF:
    `"{action_id}:retry:{alert_id}"` → `{ attempt: u32, next_retry_at: Timestamp, last_error: String }`
  - Backoff schedule: attempt 1 = 2s, 2 = 4s, 3 = 8s, 4 = 30s, 5 = 60s
  - Retry is executed via `tokio::time::sleep` (does NOT block the trigger evaluation loop)
  - On success: retry key is deleted from `action_state` CF
- **Non-retryable failure (HTTP 4xx except 429):**
  - No retry; audit-logged as `action_delivery_failed` with `retryable: false`
  - Dead-letter record written: `"{action_id}:dead_letter:{alert_id}"` → JSON with final error
- **Exhausted retries (5 failures):**
  - Dead-letter record written to `action_state` CF
  - Audit event: `action_delivery_failed` with `attempts: 5`, `final_error: String`
  - The source alert record in the `alerts` CF is NOT modified or deleted

## Invariants

- INV-ACTION-001: Alert and case triggers deliver at-least-once with retry (max 5, exponential backoff 2s base, 60s max)
- Retry state persistence MUST complete before the retry delay begins
- The trigger evaluation loop (subscribing to the alert broadcast channel) MUST NOT block
  during retry delays — retries run in separate tokio tasks
- Dead-letter state is append-only; it does not replace the source alert or case record

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ACTION-003` | All 5 retry attempts exhausted | Dead-letter + `action_delivery_failed` audit event; alert intact in `alerts` CF |
| `E-ACTION-004` | `action_state` CF write fails during retry state persistence | Log `ERROR`; proceed with retry anyway (best-effort persistence; at-least-once guarantee remains on crash via dirty bit) |
| `E-STORE-002` | RocksDB unavailable entirely | Log `ERROR`; retry will be attempted without persisted state (in-memory only for this session) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-001 | Webhook returns 503 on all 5 attempts | Dead-letter written; audit event; alert preserved |
| EC-18-002 | Webhook returns 200 on 3rd attempt | Retry key deleted; success audit event with `attempts: 3` |
| EC-18-003 | Prism restarts after 2 failed attempts (retry state in RocksDB) | On restart, retry state is re-read; remaining 3 attempts executed |
| EC-18-004 | Broadcast channel lagged (`RecvError::Lagged`) during alert consumption | Log `WARN "action engine lagged; skipping N alerts"`; resume from latest; missed alerts not retried (broadcast limitation) |
| EC-18-005 | 100 concurrent alert triggers for the same action | Each creates an independent retry task; `action_state` keys are keyed by `alert_id` so no collision |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-001-happy | Webhook returns 200 on first attempt | Delivery success; retry key not written | Baseline |
| TV-18-001-retry | Webhook returns 503 on attempts 1-2, 200 on attempt 3 | Success after 3 attempts; retry key deleted | EC-18-002 |
| TV-18-001-exhaust | Webhook returns 500 on all 5 attempts | Dead-letter written; `action_delivery_failed` audit event | EC-18-001 |
| TV-18-001-restart | 2 failures; Prism restarts; RocksDB state present | Remaining 3 attempts executed post-restart | EC-18-003 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-044 | The action delivery retry state machine never exceeds 5 attempts; the dead-letter transition fires exactly once after the 5th failure; the state is terminal after dead-lettering | Kani |
| (none) | Source alert preserved in `alerts` CF after dead-letter — integration behavior; integration test in tests/action_tests.rs | — |

## Related BCs

- BC-2.18.002 — Schedule Action Best-Effort Delivery (different delivery guarantee for schedule triggers)
- BC-2.18.003 — Manual Action Fire-and-Forget (different guarantee for manual triggers)
- BC-2.12.011 — RETIRED; this BC (BC-2.18.001) is the normative replacement
- BC-2.05.001 — Audit Entry per Tool Invocation (covers `action_delivery_failed` audit)

## Architecture Anchors

- AD-021: Actions — at-least-once delivery
- `specs/architecture/actions.md` — retry logic, exponential backoff, dead-letter
- S-4.08 Task 7: `action/retry.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-001, AC-1, AC-3)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Simulate webhook returning 500 → verify retry with backoff → verify dead-letter after 5 failures."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-001 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-044); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
