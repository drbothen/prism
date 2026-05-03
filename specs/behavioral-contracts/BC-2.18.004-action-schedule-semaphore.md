---
document_type: behavioral-contract
level: L3
version: "1.5"
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

# BC-2.18.004: Action Delivery Semaphore — 8-Permit Independent Pool, try_acquire() Skip-If-Unavailable

> **Supersedes note:** Earlier draft mandated a 16-permit semaphore shared with S-4.01
> (schedule executor), with `ActionDeliveryEngine` receiving the `Arc<Semaphore>` at construction
> time rather than owning it. Current spec reflects D-209 LOCKED per-subsystem 8-permit
> independent semaphores: `action_delivery_semaphore` is module-private to
> `action/delivery.rs` and OWNS its own `Arc<Semaphore>` constructed in
> `ActionDeliveryEngine::init` (per ADR-016 §2.11 + D-209). NOT shared with schedule executor.

## Description

Scheduled report actions that execute PrismQL queries (via `QueryEngine`) MUST acquire
the **8-permit `action_delivery_semaphore`** (module-private to `action/delivery.rs`;
per ADR-016 §2.11 and D-209) using `try_acquire()` before executing report queries.
If all 8 permits are held (by other concurrent action deliveries), the tick is skipped
for this action — not blocked. This prevents the cron tick loop from blocking.
The `ActionDeliveryEngine` OWNS its own `Arc<Semaphore>` constructed in `ActionDeliveryEngine::init`;
it does NOT share the semaphore with the schedule executor. This is INV-ACTION-004.

## Preconditions

- A `trigger = "schedule"` action with `[action.destination.report]` configured has
  matched its cron expression
- The `ActionDeliveryEngine` owns the module-private 8-permit `action_delivery_semaphore`
  (`Arc<Semaphore>` constructed in `ActionDeliveryEngine::init`; per ADR-016 §2.11 + D-209)
- The semaphore has zero permits available (all 8 held by other concurrent action deliveries)

## Postconditions

- **Semaphore available:** `try_acquire()` succeeds; report queries execute via
  `QueryEngine::execute(query, client_id, 200MB_budget, 30s_timeout)`; each query
  section is rendered; full report assembled and delivered; semaphore released.
- **Semaphore unavailable:** `try_acquire()` returns immediately with `Err(TryAcquireError::NoPermits)`
  - Log: `"action report '{action_id}' skipped: action delivery semaphore unavailable"`
  - The tick is skipped; delivery does not occur for this tick
  - Normal cron tick cycle resumes; next tick will attempt `try_acquire()` again
  - No error returned to MCP; no audit event for the skip (log is sufficient)

## Invariants

- INV-ACTION-004: Scheduled report queries MUST use `try_acquire()`, never `acquire()`
  on the `action_delivery_semaphore`
- Using `acquire()` (blocking) would freeze the cron tick loop if all permits are held,
  preventing ALL scheduled actions from evaluating — this is a safety-critical constraint
- The `action_delivery_semaphore` is module-private to `action/delivery.rs`; `ActionDeliveryEngine`
  OWNS its own `Arc<Semaphore>` constructed in `ActionDeliveryEngine::init` — it does NOT receive
  a shared semaphore from outside (per D-209)
- The `action_delivery_semaphore` is NOT shared with the schedule executor
  (`schedule_executor_semaphore`); the two pools are fully independent (per D-209)
- Report query execution is scoped within the semaphore permit lifetime; permit is
  released after all report sections are assembled (not after delivery)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | All 8 `action_delivery_semaphore` permits held (per ADR-016 §2.11 + D-209) | Tick skipped; `INFO` log; no error or audit event |
| — | Report query execution fails (timeout, OOM, error) | Section contains error note per BC-2.18.005; other sections still rendered; report delivered with partial content |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-012 | Semaphore has 1 permit available; 2 concurrent report ticks fire | First `try_acquire()` succeeds; second gets `NoPermits`; second tick skipped |
| EC-18-013 | Report action with 10 queries; semaphore acquired; query 5 times out | Sections 1-4 rendered normally; section 5 contains error note; sections 6-10 execute normally; partial report delivered (INV-ACTION-005) |
| EC-18-014 | 8 concurrent action deliveries in-flight during burst | All additional report action ticks are skipped; in-flight deliveries complete; permits released; next report tick fires normally |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-004-happy | Semaphore has permits; cron fires; 3-section report | All 3 sections rendered; report delivered | Baseline |
| TV-18-004-skip | All 8 `action_delivery_semaphore` permits held; cron fires | Tick skipped; INFO log; no delivery | Error row 1 |
| TV-18-004-contention | 2 concurrent ticks; 1 permit available | First succeeds; second skipped with `NoPermits` | EC-18-012 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-045 | When all 8 `action_delivery_semaphore` permits are held, `ActionDeliveryEngine::fire_schedule()` returns immediately (within 10ms) without acquiring a permit; it does not block or await on the semaphore | Proptest |

## Related BCs

- BC-2.18.002 — Schedule Best-Effort (governs delivery guarantee for schedule triggers)
- BC-2.18.005 — Partial Report Failure Handling (governs behavior when individual queries fail)
- BC-2.12.004 — Schedule Execution Loop (owns the independent `schedule_executor_semaphore`; NOT shared with action delivery per D-209)

## Architecture Anchors

- ADR-016 §2.11: `action_delivery_semaphore` — 8-permit, module-private to `action/delivery.rs`
- D-209: Per-subsystem 8-permit independent semaphores; action delivery owns its own `Arc<Semaphore>` constructed in `ActionDeliveryEngine::init`; NOT shared with schedule executor
- AD-021: Actions — schedule semaphore `try_acquire()`
- `specs/architecture/actions.md` — schedule semaphore, report execution
- S-4.08 Architecture Compliance: "ActionDeliveryEngine::fire_schedule MUST use `try_acquire()` on the 8-permit `action_delivery_semaphore`, NOT `acquire()`; semaphore is module-private, NOT shared with schedule executor"
- S-4.08 Task 8: `action/report.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-004, AC-6)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Given all 8 `action_delivery_semaphore` permits held → verify report tick skipped with log."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-004 |
| ADR | ADR-016 §2.11, D-209 |
| Story | S-4.08 |
| Priority | P0 |

## Phase 4.A Pass 6 Remediation Notes

**Adversary finding:** HIGH-004 (Pass 6) — BC body mandated a 16-permit semaphore shared
with S-4.01 (schedule executor), with `ActionDeliveryEngine` receiving the semaphore at construction
rather than owning it. This directly contradicted D-209 and ADR-016 §2.11.

**Changes made (2026-05-02):**
- **H1 title changed:** "Scheduled Report Queries — try_acquire() on 16-Permit Semaphore,
  Skip If Unavailable" → **"Action Delivery Semaphore — 8-Permit Independent Pool,
  try_acquire() Skip-If-Unavailable"** (flag for BC-INDEX update by state-manager)
- All "16-permit" references → **"8-permit"** throughout Description, Preconditions,
  Postconditions, Invariants, Error Conditions, Edge Cases, Canonical Test Vectors,
  Verification Properties, VP Anchors
- All "shared with S-4.01" / "shared schedule semaphore" → **"module-private; NOT shared
  with schedule executor"** per D-209
- Ownership model corrected: "receives the `Arc<Semaphore>` at construction time, does NOT
  create its own semaphore" → **"OWNS its own `Arc<Semaphore>` constructed in
  `ActionDeliveryEngine::init`"** per D-209
- Semaphore name canonicalized: generic "schedule semaphore" →
  **"`action_delivery_semaphore`"** per ADR-016 §2.11
- EC-18-014 updated: "Detection scheduled query holds 16 permits" → "8 concurrent action
  deliveries in-flight"
- Added Architecture Anchors for ADR-016 §2.11 and D-209
- Updated Story Architecture Compliance note
- Updated Traceability ADR field
- Added supersedes note at top of body

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | F-P20-L-002 | 2026-05-03 | product-owner | Pass 20 COSMETIC LOW: ActionEngine → ActionDeliveryEngine canonical type name (matches ADR-016 §1.1/§2.11 + S-4.08 Task 1). |
| 1.4 | wave4-pass6-bc-sweep | 2026-05-02 | product-owner | Phase 4.A Pass 6 remediation (HIGH-004): H1 title updated; corrected to 8-permit module-private action_delivery_semaphore owned by ActionEngine::init (ADR-016 §2.11 + D-209); removed shared-with-S-4.01 model. |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-045); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
