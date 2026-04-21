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
input-hash: "365fb25"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.008: All Action Executions Are Audit-Logged — Success, Failure, and Suppression

## Description

Every action execution outcome — whether delivered successfully, failed, suppressed by
rate limiting, or suppressed by deduplication — is recorded as an audit log entry. The
audit entry includes: `action_id`, `trigger` type, `client_id`, `destination_type`,
`status`, `delivery_latency_ms` (for deliveries), `suppression_reason` (for
suppressions), `_safety_flags` (for injection-scanned templates). This is INV-ACTION-008.

## Preconditions

- An action trigger has fired (alert, case, schedule, or manual)
- The `ActionEngine` has evaluated rate limits, deduplication, and/or attempted delivery

## Postconditions

- An audit entry is emitted for EVERY action execution outcome:
  - **Delivered:** `{ event_type: "action_delivered", action_id, trigger, client_id, destination_type, status: "success", delivery_latency_ms, _safety_flags }`
  - **Failed:** `{ event_type: "action_delivery_failed", action_id, trigger, client_id, destination_type, status: "failed", error, attempt_number, _safety_flags }`
  - **Suppressed (rate limit):** `{ event_type: "action_suppressed", action_id, trigger, client_id, reason: "rate_limit", hour_bucket, count }`
  - **Suppressed (cooldown):** `{ event_type: "action_suppressed", action_id, trigger, client_id, reason: "cooldown", last_fired_at }`
  - **Suppressed (dedup):** `{ event_type: "action_suppressed", action_id, trigger, client_id, reason: "dedup", content_hash, original_delivery_at }`
  - **Dead-lettered:** `{ event_type: "action_dead_lettered", action_id, trigger, client_id, alert_id, attempts: 5, final_error }`
- Audit entries are emitted to the standard `tracing` subscriber (stdout/JSON) AND
  to the RocksDB `AuditBuffer` CF (BC-2.15.003)
- Audit entries are emitted AFTER the delivery attempt (or suppression decision), not before

## Invariants

- INV-ACTION-008: All action executions (success, failure, suppression) are audit-logged
- Audit entries are NOT omitted for suppressed deliveries — suppression is a notable event
- The `_safety_flags` field from injection scanning (BC-2.18.006) is always included
  in delivery audit entries (empty array when no flags)
- Credential values are NEVER included in audit entries (BC-2.05.003 applies)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-002` | Audit buffer RocksDB write fails | Log to stderr as fallback; action delivery proceeds regardless of audit failure |
| — | Audit entry exceeds maximum size (e.g., very large template body in error) | Audit entry body truncated at 64KB; `truncated: true` field added |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-027 | 1000 alerts/minute firing the same action | 1000 audit entries emitted (suppression entries count); RocksDB buffer handles burst |
| EC-18-028 | Schedule action suppressed every tick for an hour (semaphore unavailable) | Each suppressed tick generates a `INFO` log (not an audit entry — skip is not a notable suppression); audit entry only for actual delivery attempts or rate-limit suppressions |
| EC-18-029 | Dead-letter after 5 retries | Single `action_dead_lettered` audit entry; the 5 `action_delivery_failed` entries are also present (one per attempt) |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-008-happy | Action delivered successfully | `action_delivered` audit entry with `status: "success"` | Baseline |
| TV-18-008-fail | Delivery fails | `action_delivery_failed` audit entry with error details | Error row 1 |
| TV-18-008-suppress | Rate limit exceeded | `action_suppressed` audit entry with `reason: "rate_limit"` | AC-2 |
| TV-18-008-deadletter | 5 retry exhaustions | 1 `action_dead_lettered` + 5 `action_delivery_failed` entries | EC-18-029 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | Audit completeness is an integration test concern covering all ActionEngine code paths; credential-absence in audit entries covered by BC-2.05.003 policy and VP-046 credential rejection proving values never enter the system as bare strings | — |

## Related BCs

- BC-2.05.001 — Every MCP Tool Invocation Produces Exactly One Audit Entry (covers the `fire_action` tool call; BC-2.18.008 covers the delivery outcome)
- BC-2.05.003 — Credential Values Never in Audit Entries
- BC-2.18.006 — Template Injection Scanning (`_safety_flags` field in audit entry)
- BC-2.15.003 — Buffered Audit Log Persistence (RocksDB buffer for external delivery)

## Architecture Anchors

- AD-021: Actions — audit logging for all outcomes
- AD-016: Write-audit ordering (audit before return where applicable)
- `specs/architecture/actions.md` — audit events, suppression logging
- S-4.08 Task 6: `action/rate_limit.rs` — suppression audit events
- S-4.08 Task 7: `action/retry.rs` — failure and dead-letter audit events

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-008, AC-2)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Exceed `max_per_hour` → verify delivery suppressed with audit log (reason: 'rate_limit')."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-008 |
| ADR | AD-016, AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
