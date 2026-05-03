---
document_type: behavioral-contract
level: L3
version: "1.8"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-18"
capability: "CAP-033"
lifecycle_status: active
introduced: cycle-1
modified: 2026-05-03
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "248b3b0"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.001: Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry

> **Supersedes note:** Earlier draft used a non-standard backoff sequence of `2s/4s/8s/30s/60s`.
> Current spec reflects ADR-016 §2.6 standard exponential backoff `2s, 4s, 8s, 16s, 32s`
> with ±10% jitter per attempt (cumulative range 55.8s–68.2s nominal-jittered; max 5 attempts).

## Description

When an alert or case event triggers an action, the delivery is guaranteed at-least-once.
Failed deliveries are retried with exponential backoff (maximum 5 attempts, schedule
**2s, 4s, 8s, 16s, 32s** with ±10% jitter per attempt; cumulative 55.8s–68.2s; per
ADR-016 §2.6). Retry state is persisted to RocksDB `action_state` CF before each attempt
using key discriminator `\x04` (retry-state row per ADR-016 §2.5), prefixed by OrgId per
ADR-008 universal re-keying rule. After 5 failures, a dead-letter record is written using
key discriminator `\x03` (dead-letter row per ADR-016 §2.5) and an audit event is emitted.
The source alert is NOT lost (it remains in the `alerts` CF). This is INV-ACTION-001.

## Preconditions

- `ActionDeliveryEngine` is initialized with a registered `ActionSpec` with `trigger = "alert"` or
  `trigger = "case"`
- An alert or case event matching the action's `clients` and `filter` criteria is broadcast
- The destination (webhook, email, syslog, or plugin) is temporarily unavailable or returns
  a retryable error

## Postconditions

- **Retryable failure (HTTP 429, 5xx, network error):**
  - Retry state is written to `action_state` CF:
    key `"{org_id}:\x04:{action_id}:{idempotency_key}"` → bincode-encoded `RetryState { attempt: u8, next_attempt_at: Timestamp, last_error: Option<String> }`; TTL 24h
    (`{idempotency_key}` abstract — alert→`alert_id`; case→`timeline_entry_id`; discriminator `\x04` = retry-state row per ADR-016 §2.5; OrgId-first prefix per ADR-008)
  - Backoff schedule (per ADR-016 §2.6): attempt 1 = 2s ±10%, 2 = 4s ±10%, 3 = 8s ±10%,
    4 = 16s ±10%, 5 = 32s ±10%; cumulative range 55.8s–68.2s
  - Retry is executed via `tokio::time::sleep` (does NOT block the trigger evaluation loop)
  - On success: retry key is deleted from `action_state` CF
- **Non-retryable failure (HTTP 4xx except 429):**
  - No retry; audit-logged as `action_delivery_failed` with `retryable: false`
  - Dead-letter record written: key `"{org_id}:\x03:{action_id}:{idempotency_key}"` → bincode-encoded `DeadLetterEntry { final_attempt: u8, terminal_error: String, dead_lettered_at: Timestamp }`
    (discriminator `\x03` = dead-letter row per ADR-016 §2.5 v0.6; OrgId-first prefix per ADR-008; `{idempotency_key}` abstract — alert→alert_id; case→timeline_entry_id; terminal — written only after max_attempts exhausted or non-retryable error)
- **Exhausted retries (5 failures):**
  - Dead-letter record written to `action_state` CF (same key format as non-retryable above)
  - Audit event: `action_delivery_failed` with `attempts: 5`, `final_error: String`
  - The source alert record in the `alerts` CF is NOT modified or deleted

## Invariants

- INV-ACTION-001: Alert and case triggers deliver at-least-once with retry (max 5 attempts,
  exponential backoff `2s, 4s, 8s, 16s, 32s` with ±10% jitter per ADR-016 §2.6; cap 32s
  per attempt)
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
| EC-18-005 | 100 concurrent alert triggers for the same action | Each creates an independent retry task; `action_state` keys are keyed by `idempotency_key` (= `alert_id` for alert triggers) so no collision |
| EC-18-005a | 100 concurrent case state-change events triggering the same action | Each creates an independent retry task; `action_state` keys are keyed by `idempotency_key` (= `timeline_entry_id` for case triggers) so no collision |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-001-happy | Webhook returns 200 on first attempt | Delivery success; retry key not written | Baseline |
| TV-18-001-retry | Webhook returns 503 on attempts 1-2, 200 on attempt 3 | Success after 3 attempts; retry key deleted; delays ~2s and ~4s (±10%) | EC-18-002 |
| TV-18-001-exhaust | Webhook returns 500 on all 5 attempts | Dead-letter written; `action_delivery_failed` audit event; total delay 55.8s–68.2s | EC-18-001 |
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

- ADR-016 §2.5: `action_state` CF key discriminators (`\x03` dead-letter, `\x04` retry-state); OrgId-first prefix per ADR-008
- ADR-016 §2.6: Action delivery retry backoff schedule (`2s, 4s, 8s, 16s, 32s` ±10% jitter)
- ADR-008: Universal OrgId-first re-keying rule (applies to all `action_state` CF keys)
- AD-021: Actions — at-least-once delivery
- `specs/architecture/actions.md` — retry logic, exponential backoff, dead-letter
- S-4.08 Task 7: `action/retry.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-001, AC-1, AC-3)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Simulate webhook returning 500 → verify retry with backoff (2s/4s/8s/16s/32s ±10%) → verify dead-letter after 5 failures."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-001 |
| ADR | ADR-016 §2.5 (CF key discriminators), ADR-016 §2.6 (backoff schedule), ADR-008 (OrgId prefix) |
| Story | S-4.08 |
| Priority | P0 |

## Phase 4.A Pass 6 Remediation Notes

**Adversary finding:** HIGH-002 (Pass 6) — BC body specified non-standard backoff sequence
`2s/4s/8s/30s/60s`, contradicting locked ADR-016 §2.6.

**Changes made (2026-05-02):**
- Backoff schedule corrected: `2s/4s/8s/30s/60s` → **`2s, 4s, 8s, 16s, 32s`** (cap 32s per
  attempt) with **±10% jitter** per attempt per ADR-016 §2.6
- Cumulative range added: 55.8s–68.2s nominal-jittered
- Updated: Description, Postconditions backoff schedule row, INV-ACTION-001 invariant,
  Canonical Test Vectors (TV-18-001-retry and TV-18-001-exhaust notes), VP Anchors, Traceability ADR field
- Added Architecture Anchor for ADR-016 §2.6
- Added supersedes note at top of body

## Phase 4.A Pass 8 Remediation Notes

v1.5 (P8 fix): Retry/dead-letter CF keys aligned with ADR-016 §2.5 retry-state row (NEW `\x04`) and dead-letter row `\x03`; OrgId-first prefix per ADR-008 (P8-BC-2.18.001-A-H-002).

## Phase 4.A Pass 9 Remediation Notes

v1.6 (P9 fix): Dead-letter CF key aligned to `{idempotency_key}` per ADR-016 §2.5 v0.6 adjudication (F-P9-H-002).

## Phase 4.A Pass 10 Remediation Notes

v1.7 (P10 fix): Retry-state CF key aligned to `{idempotency_key}` per ADR-016 §2.5 v0.7 (F-P10-H-002 — Pass 9 sister-row sweep gap).
v1.7 (P10 fix): EC-18-005 + EC-18-005a — alert AND case trigger collision cases now both covered (F-P10-M-002).

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.8 | F-P20-L-002 | 2026-05-03 | product-owner | Pass 20 COSMETIC LOW: ActionEngine → ActionDeliveryEngine canonical type name (matches ADR-016 §1.1/§2.11 + S-4.08 Task 1). |
| 1.7 | wave4-pass10-bc-sweep | 2026-05-03 | product-owner | P10 fixes: retry-state key updated to `{idempotency_key}` per ADR-016 §2.5 v0.7 (F-P10-H-002); EC-18-005 updated + EC-18-005a added for case-trigger collision coverage (F-P10-M-002). |
| 1.6 | wave4-pass9-bc-sweep | 2026-05-03 | state-manager | P9 fix (F-P9-H-002): dead-letter key updated `{alert_id}` → `{idempotency_key}` per ADR-016 §2.5 v0.6 adjudication (abstract — alert→alert_id; case→timeline_entry_id). |
| 1.5 | wave4-pass8-bc-sweep | 2026-05-03 | product-owner | P8 fix (P8-BC-2.18.001-A-H-002): retry key `{org_id}:\x04:{action_id}:{alert_id}` + bincode RetryState per ADR-016 §2.5; dead-letter key `{org_id}:\x03:{action_id}:{alert_id}` + bincode DeadLetterEntry per ADR-016 §2.5; OrgId-first prefix per ADR-008; value types specified; Architecture Anchors and Traceability updated. |
| 1.4 | wave4-pass6-bc-sweep | 2026-05-02 | product-owner | Phase 4.A Pass 6 remediation (HIGH-002): corrected backoff to 2s/4s/8s/16s/32s ±10% jitter per ADR-016 §2.6; cumulative range 55.8s–68.2s; removed non-standard 30s/60s cap. |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-044); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
