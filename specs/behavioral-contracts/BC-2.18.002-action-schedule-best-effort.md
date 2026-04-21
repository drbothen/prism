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
input-hash: "47125c0"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.002: Schedule Action Triggers — Best-Effort, Retry on Next Cron Tick

## Description

Actions with `trigger = "schedule"` use best-effort delivery semantics. If a scheduled
action fires and the delivery fails (or the tick is missed entirely), no retry is
attempted for that missed tick. The action will be attempted again at the next cron
evaluation. There is no catch-up for missed windows. This is INV-ACTION-002.

## Preconditions

- `ActionEngine` has a registered `ActionSpec` with `trigger = "schedule"` and a valid
  cron expression in `schedule`
- The `CronScheduler` tick loop fires and evaluates the cron expression for the current time
- The cron expression matches the current UTC time

## Postconditions

- **Delivery success:** The action fires and the report/notification is delivered.
  The cron scheduler records the last-fired timestamp in `action_state` CF.
- **Delivery failure:** An `ERROR`-level log is emitted:
  `"Schedule action '{action_id}' delivery failed at {tick_time}: {error}. Will retry at next tick."`
  No retry state is written. No dead-letter. The next cron tick evaluates normally.
- **Missed tick (Prism down at tick time):** The tick is silently skipped. Prism does
  not attempt to "catch up" missed scheduled actions on restart.
- **Semaphore unavailable (all 16 schedule permits held):** Tick is skipped with log
  `"Schedule action '{action_id}' skipped: schedule semaphore unavailable"`. This is
  covered separately in BC-2.18.004.

## Invariants

- INV-ACTION-002: Schedule triggers are best-effort — no catch-up for missed windows, no retry on failure
- The cron tick loop runs on a 1-second `tokio::time::interval`; it MUST NOT block
- Failed scheduled deliveries are logged but do not create retry or dead-letter state
- Schedule action delivery runs via `ActionEngine::fire_schedule`, not the same retry
  infrastructure as alert/case triggers (BC-2.18.001)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | Delivery fails at scheduled tick | Log `ERROR`; no retry; next tick fires normally |
| — | Cron expression no longer matches (e.g., monthly job, off-day) | No action fires; tick loop continues |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-006 | Prism restarts mid-scheduled-action delivery | Delivery is abandoned; next scheduled tick fires normally (no catch-up) |
| EC-18-007 | Cron fires every second (`* * * * * *`) and delivery takes 2 seconds | The 1-second ticker fires again while previous delivery is in progress; `try_acquire()` on semaphore prevents overlap (BC-2.18.004) |
| EC-18-008 | Schedule action with only one client; client removed from config | Next tick: `clients` list is empty; action skips with log; no delivery |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-002-happy | Cron expression matches; delivery succeeds | Last-fired timestamp written to `action_state` CF | Baseline |
| TV-18-002-fail | Cron tick fires; delivery returns HTTP 500 | ERROR log; no retry state; next tick evaluates normally | Error row 1 |
| TV-18-002-missed | Prism was down during cron tick; restarts | Missed tick silently skipped; no catch-up | EC-18-006 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | Best-effort/no-catch-up semantics are behavioral absence-of-write; integration test in tests/action_tests.rs; no pure-function invariant warranting formal verification | — |

## Related BCs

- BC-2.18.001 — Alert/Case At-Least-Once Delivery (stricter guarantee for alert/case triggers)
- BC-2.18.003 — Manual Fire-and-Forget (third trigger type)
- BC-2.18.004 — Schedule Semaphore Enforcement (semaphore governs concurrency, not delivery guarantee)

## Architecture Anchors

- AD-021: Actions — schedule trigger best-effort semantics
- `specs/architecture/actions.md` — schedule trigger, cron evaluation
- S-4.08 Task 9: `action/cron.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-002, AC-5)

## VP Anchors

No dedicated test for missed-tick behavior (non-deterministic timing). Covered implicitly by cron tick loop tests in `tests/action_tests.rs`.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-002 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
