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

# BC-2.18.004: Scheduled Report Queries — try_acquire() on 16-Permit Semaphore, Skip If Unavailable

## Description

Scheduled report actions that execute PrismQL queries (via `QueryEngine`) MUST acquire
the 16-permit shared schedule semaphore using `try_acquire()` before executing report
queries. If all 16 permits are held (by detection scheduled queries or other report
actions), the tick is skipped for this action — not blocked. This prevents scheduled
report actions from starving detection queries. This is INV-ACTION-004.

## Preconditions

- A `trigger = "schedule"` action with `[action.destination.report]` configured has
  matched its cron expression
- The `ActionEngine` holds a reference to the shared 16-permit `Arc<Semaphore>` (same
  semaphore used by S-4.01 detection scheduled queries)
- The semaphore has zero permits available (`all 16 held by other executions`)

## Postconditions

- **Semaphore available:** `try_acquire()` succeeds; report queries execute via
  `QueryEngine::execute(query, client_id, 200MB_budget, 30s_timeout)`; each query
  section is rendered; full report assembled and delivered; semaphore released.
- **Semaphore unavailable:** `try_acquire()` returns immediately with `Err(TryAcquireError::NoPermits)`
  - Log: `"action report '{action_id}' skipped: schedule semaphore unavailable"`
  - The tick is skipped; delivery does not occur for this tick
  - Normal cron tick cycle resumes; next tick will attempt `try_acquire()` again
  - No error returned to MCP; no audit event for the skip (log is sufficient)

## Invariants

- INV-ACTION-004: Scheduled report queries MUST use `try_acquire()`, never `acquire()`
- Using `acquire()` (blocking) would freeze the cron tick loop if all permits are held,
  preventing ALL scheduled actions from evaluating — this is a safety-critical constraint
- The semaphore is shared with S-4.01 (detection scheduled queries); `ActionEngine`
  receives the `Arc<Semaphore>` at construction time, does NOT create its own semaphore
- Report query execution is scoped within the semaphore permit lifetime; permit is
  released after all report sections are assembled (not after delivery)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | All 16 semaphore permits held | Tick skipped; `INFO` log; no error or audit event |
| — | Report query execution fails (timeout, OOM, error) | Section contains error note per BC-2.18.005; other sections still rendered; report delivered with partial content |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-012 | Semaphore has 1 permit available; 2 concurrent report ticks fire | First `try_acquire()` succeeds; second gets `NoPermits`; second tick skipped |
| EC-18-013 | Report action with 10 queries; semaphore acquired; query 5 times out | Sections 1-4 rendered normally; section 5 contains error note; sections 6-10 execute normally; partial report delivered (INV-ACTION-005) |
| EC-18-014 | Detection scheduled query holds 16 permits during morning burst | All concurrent report action ticks are skipped; detection queries complete; permits released; next report tick fires normally |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-004-happy | Semaphore has permits; cron fires; 3-section report | All 3 sections rendered; report delivered | Baseline |
| TV-18-004-skip | All 16 permits held; cron fires | Tick skipped; INFO log; no delivery | Error row 1 |
| TV-18-004-contention | 2 concurrent ticks; 1 permit available | First succeeds; second skipped with `NoPermits` | EC-18-012 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-045 | When all 16 semaphore permits are held, `ActionEngine::fire_schedule()` returns immediately (within 10ms) without acquiring a permit; it does not block or await on the semaphore | Proptest |

## Related BCs

- BC-2.18.002 — Schedule Best-Effort (governs delivery guarantee for schedule triggers)
- BC-2.18.005 — Partial Report Failure Handling (governs behavior when individual queries fail)
- BC-2.12.004 — Schedule Execution Loop (owns the 16-permit semaphore; actions share it)

## Architecture Anchors

- AD-021: Actions — schedule semaphore `try_acquire()`
- `specs/architecture/actions.md` — schedule semaphore, report execution
- S-4.08 Architecture Compliance: "ActionEngine::fire_schedule MUST use `try_acquire()` on the 16-permit schedule semaphore, NOT `acquire()`"
- S-4.08 Task 8: `action/report.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-004, AC-6)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Given all 16 semaphore permits held → verify report tick skipped with log."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-004 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-045); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
