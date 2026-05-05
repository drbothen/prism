---
document_type: holdout-scenario
level: L3
id: "HS-012"
category: "action-delivery"
must_pass: true
priority: P0
epic_id: "E-4"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-04T00:00:00Z
phase: 1b
inputs: []
input-hash: null
traces_to: prd.md
behavioral_contracts:
  - BC-2.18.001
  - BC-2.18.002
  - BC-2.18.003
  - BC-2.18.004
  - BC-2.18.005
  - BC-2.18.006
  - BC-2.18.007
  - BC-2.18.008
  - BC-2.18.009
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Wave 4 holdout group — BC-anchored per D-216."
---

# HS-012: Action Delivery

**Group:** Independent 8-permit action delivery semaphore (D-209 split), VP-045 try_acquire non-blocking semantics, SemaphoreExhausted audit event, action_state CF discriminator state machine (ADR-016 §2.5), terminal state enforcement, and multi-tenant action delivery isolation for the S-4.08 action delivery subsystem.
**Date:** 2026-05-04
**Priority:** P0

---

## Scenario

Six sub-scenarios covering the Wave 4 action delivery framework (S-4.08). These scenarios validate the three most safety-critical architectural decisions for this subsystem. First, D-209 LOCKED independence: the `action_delivery_semaphore` is an 8-permit pool module-private to `action/delivery.rs`, completely independent of the `schedule_executor_semaphore` owned by the schedule executor (BC-2.18.004). Second, VP-045 non-blocking semantics: `try_acquire()` on an exhausted semaphore ABORTS the tick immediately (within 10ms) rather than blocking, preventing the cron tick loop from stalling (BC-2.18.004, INV-ACTION-004). Third, the `action_state` column family 5-row state machine: key discriminators `\x00` (PENDING_ACTION), `\x01` (DELIVERY_ATTEMPT), `\x02` (DELIVERY_SUCCESS), `\x03` (DELIVERY_FAILURE / dead-letter), `\x04` (retry-state row) govern all action lifecycle state (ADR-016 §2.5); once a DELIVERY_TERMINAL state is reached, no further transitions occur.

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.18.001 | Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry | HS-012-04, HS-012-05 |
| BC-2.18.002 | Schedule Action Triggers — Best-Effort, Retry on Next Cron Tick | HS-012-02 |
| BC-2.18.003 | Manual Action Triggers — Fire-and-Forget, Result Returned Immediately | HS-012-06 |
| BC-2.18.004 | Action Delivery Semaphore — 8-Permit Independent Pool, try_acquire() Skip-If-Unavailable | HS-012-01, HS-012-02, HS-012-03 |
| BC-2.18.005 | Partial Report Failure — Failed Sections Include Error Note, Others Delivered | HS-012-04 |
| BC-2.18.006 | Action Template Variables — Injection-Scanned Before Interpolation | HS-012-04 |
| BC-2.18.007 | Action Credentials Must Use AI-Opaque Reference Model — No Inline Values | HS-012-06 |
| BC-2.18.008 | All Action Executions Are Audit-Logged — Success, Failure, and Suppression | HS-012-01, HS-012-03, HS-012-04, HS-012-05 |
| BC-2.18.009 | `${case.alert_ids_quoted}` Values Validated as UUID v7 Before Interpolation | HS-012-04 |

## Verification Approach

All sub-scenarios run in the DTU harness with the `ActionDeliveryEngine` initialized under test. For HS-012-01 (semaphore independence), the harness inspects the in-memory object graph at runtime and asserts that the `Arc<Semaphore>` inside `ActionDeliveryEngine` is a distinct pointer from the `Arc<Semaphore>` inside the schedule executor — they do not share the same allocation. For HS-012-02 (VP-045 non-blocking), the harness saturates all 8 `action_delivery_semaphore` permits, then fires a cron tick and measures wall-time for the `try_acquire()` path; it must return within 10ms.

For HS-012-04 (discriminator state machine), the harness directly reads RocksDB `action_state` CF byte keys and verifies that: (a) the discriminator byte (`\x00`, `\x01`, `\x02`, `\x03`, `\x04`) matches the action's lifecycle phase, and (b) transitions follow the canonical ADR-016 §2.5 success path (`\x00→\x01→\x02`) or failure path (`\x00→\x01→\x03`). For HS-012-05 (terminal state), the harness writes a `\x03` (DELIVERY_FAILURE) terminal row and then attempts to trigger further delivery; it asserts no transition out of the terminal row occurs.

## Evaluation Rubric

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Semaphore independence (D-209 split — distinct Arc allocations, no sharing) | 25% | 100% — must-pass (D-209 LOCKED) |
| VP-045 non-blocking: try_acquire returns within 10ms on exhaustion | 25% | 100% — must-pass (INV-ACTION-004) |
| SemaphoreExhausted audit event on saturation | 10% | ≥90% |
| action_state CF discriminator transitions — success + failure paths correct | 25% | 100% — must-pass (ADR-016 §2.5) |
| DELIVERY_TERMINAL state: no further transitions | 10% | ≥95% |
| Multi-tenant action delivery isolation (per-org permit accounting) | 5% | ≥90% |

Total: 100%. Overall PASS threshold: all must-pass criteria at 100%, aggregate weighted score ≥87%.

## Edge Conditions

- 8 concurrent action deliveries in-flight when a 9th cron tick fires: 9th must skip within 10ms (VP-045); no blocking; SemaphoreExhausted emitted (BC-2.18.004 EC-18-014).
- Report action with 10 query sections; query 5 times out: sections 1-4 and 6-10 render normally; section 5 carries error note; partial report delivered (BC-2.18.005 EC-18-013).
- Action template variable containing a `${...}` expression with shell metacharacters: injection scan must reject the template before interpolation (BC-2.18.006).
- Non-retryable HTTP 4xx delivery failure (not 429): no retry; dead-letter row written with discriminator `\x03`; terminal state reached immediately (BC-2.18.001).
- Action credential configured with inline plaintext value: must be rejected at config load with E-ACTION-001; credential reference model enforced (BC-2.18.007).

## Failure Guidance

If HS-012-01 fails (shared semaphore detected): enforce D-209 by verifying that `ActionDeliveryEngine::init` calls `Arc::new(Semaphore::new(8))` internally and NEVER accepts an `Arc<Semaphore>` parameter from outside. The semaphore must NOT be constructed in `main.rs` and passed in. If HS-012-02 fails (blocking on try_acquire): locate all `semaphore.acquire()` calls in `action/delivery.rs` and replace with `try_acquire()`. VP-045 verifies this property formally. If HS-012-04 fails (wrong discriminator byte): cross-reference ADR-016 §2.5 key discriminator table and verify the bincode serialization of `action_state` CF rows uses the correct byte prefix for each state. If HS-012-05 fails (transition out of terminal): add a terminal-state guard in the delivery engine that checks for `\x02` or `\x03` discriminator before any new write. Open TDs: S-4.08 is the implementing story.

## Category: architectural-invariant

Source: D-209 LOCKED (8/8 independent semaphore split), VP-045 (try_acquire non-blocking), ADR-016 §2.5 (discriminator key schema), ADR-008 (org-prefixed action_state CF keys). Must-pass: true.

---

## HS-012-01: Action Delivery Uses Independent 8-Permit Semaphore — NOT Shared with Scheduler (D-209)

**Title:** The `action_delivery_semaphore` is a module-private 8-permit pool owned by `ActionDeliveryEngine::init`; it shares no allocation with the `schedule_executor_semaphore`.

**Preconditions:**
- Prism server started with `ActionDeliveryEngine` initialized (BC-2.18.004)
- Schedule executor also initialized with its own `schedule_executor_semaphore`
- Both subsystems active and accepting triggers

**Steps:**
1. Retrieve the `Arc<Semaphore>` pointer from `ActionDeliveryEngine` (via test accessor or Arc::ptr_eq).
2. Retrieve the `Arc<Semaphore>` pointer from the schedule executor's `schedule_executor_semaphore`.
3. Assert `Arc::ptr_eq(&action_delivery_semaphore, &schedule_executor_semaphore)` == FALSE (distinct allocations).
4. Assert both semaphores have exactly 8 initial permits (D-209: 8/8 split).
5. Saturate all 8 `action_delivery_semaphore` permits (8 concurrent action deliveries).
6. Attempt to fire a schedule executor tick. Assert the schedule executor succeeds in acquiring its own semaphore permit (not blocked by the saturated action delivery semaphore).
7. Release all 8 action delivery permits. Assert action delivery resumes on next tick.

**Expected Outcome:**
- `Arc::ptr_eq` returns false — distinct semaphore allocations confirmed (D-209 LOCKED).
- Both semaphores initialized with 8 permits (not 16, not shared).
- Schedule executor unaffected by action delivery saturation (independence verified).
- Action delivery resumes after permits released.
- Audit log confirms actions began delivery after permit release.

**Repos Tested:** prism-operations (S-4.08 ActionDeliveryEngine::init, BC-2.18.004, D-209), prism-operations (S-4.01 schedule_executor_semaphore, BC-2.12.004)

---

## HS-012-02: VP-045 try_acquire Non-Blocking — Tick Aborts Within 10ms if Permit Unavailable

**Title:** When all 8 `action_delivery_semaphore` permits are held, `ActionDeliveryEngine::fire_schedule()` returns immediately (within 10ms); it does not await or block.

**Preconditions:**
- All 8 `action_delivery_semaphore` permits held by concurrent action deliveries (EC-18-014 scenario)
- A cron-triggered action is due (cron expression matches current time)
- Wall-clock timer active for measuring `fire_schedule()` return latency

**Steps:**
1. Saturate all 8 permits by holding 8 async tasks inside the semaphore (not releasing).
2. Record `t_start` (wall clock).
3. Call `ActionDeliveryEngine::fire_schedule()` for the due cron action.
4. Record `t_end` immediately after the call returns.
5. Assert `t_end - t_start < 10ms` (VP-045 non-blocking invariant).
6. Assert the function returned `Err(SemaphoreUnavailable)` or equivalent skip result (not `Ok` / not panicked).
7. Assert log contains: `"action report '<action_id>' skipped: action delivery semaphore unavailable"` at INFO level.
8. Assert NO delivery attempt was made (no HTTP request, no audit event for delivery start).

**Expected Outcome:**
- `fire_schedule()` returns in <10ms when semaphore exhausted (VP-045).
- Return value indicates skip, not error (schedule should retry on next cron tick).
- INFO-level log entry present with `action_id` and reason.
- No delivery attempt recorded; no audit event for this skipped tick.
- Cron tick loop continues evaluating other due actions without blocking.

**Repos Tested:** prism-operations (S-4.08 fire_schedule try_acquire path, BC-2.18.004, VP-045, INV-ACTION-004)

---

## HS-012-03: SemaphoreExhausted Audit Event Emitted When Action Delivery Semaphore Saturated

**Title:** When `try_acquire()` fails due to semaphore exhaustion, a `SemaphoreExhausted` structured audit event is emitted with correct metadata.

**Preconditions:**
- All 8 `action_delivery_semaphore` permits held
- A cron-triggered action fires during saturation
- Audit log subscriber listening for `SemaphoreExhausted` events

**Steps:**
1. Saturate all 8 permits (simulate 8 concurrent report deliveries).
2. Fire a 9th cron action tick.
3. `try_acquire()` returns `Err(NoPermits)`.
4. Assert a `SemaphoreExhausted` audit event is written with fields:
   - `event_type: "SemaphoreExhausted"`
   - `subsystem: "action_delivery"`
   - `action_id: <id>`
   - `permits_available: 0`
   - `permits_total: 8`
   - `org_id: <org_id>`
5. Assert the event is at WARN level (not ERROR — this is a normal throttling condition).
6. Assert the `action_delivery_semaphore` permit count remains at 0 (not decremented).

**Expected Outcome:**
- `SemaphoreExhausted` event present in structured audit log with all required fields.
- Event at WARN severity level.
- `permits_total: 8` (confirming D-209 8-permit cap, not 16).
- `subsystem: "action_delivery"` (distinguishes from `schedule_executor` semaphore exhaustion which is E-SCHED-004).
- Permit count unchanged after the failed try_acquire.

**Repos Tested:** prism-operations (S-4.08 SemaphoreExhausted audit event, BC-2.18.004/008), prism-audit (SemaphoreExhausted event schema)

---

## HS-012-04: action_state CF Discriminator Transitions — Success and Failure Paths per ADR-016 §2.5

**Title:** The `action_state` CF 5-row state machine correctly transitions through `\x00→\x01→\x02` (success) and `\x00→\x01→\x03` (failure) discriminator paths; retry-state row (`\x04`) written on retryable failure.

**Preconditions:**
- One action configured for OrgA with `trigger: "alert"`; alert UUID-A1 triggers it
- Webhook destination temporarily unavailable (simulated HTTP 503)
- Action delivery engine with exponential backoff retry (ADR-016 §2.6: 2s, 4s, 8s, 16s, 32s per BC-2.18.001)

**Steps (Success path — separate fixture):**
1. Feed alert UUID-A1 to `ActionDeliveryEngine`. Assert `action_state` CF key `"{org_id}:\x00:{action_id}:{alert_id}"` written (PENDING_ACTION row).
2. Delivery attempt begins. Assert key `"{org_id}:\x01:{action_id}:{alert_id}"` written (DELIVERY_ATTEMPT row). Assert `\x00` key deleted.
3. Webhook returns HTTP 200. Assert key `"{org_id}:\x02:{action_id}:{alert_id}"` written (DELIVERY_SUCCESS row). Assert `\x01` key deleted.
4. Assert audit event `action_delivery_succeeded` with `discriminator: "\x02"` present.

**Steps (Failure path — separate fixture):**
5. Feed alert UUID-A2; webhook returns HTTP 503 (retryable). Assert `\x04` retry-state key written with `attempt: 1`.
6. After backoff (2s), second attempt returns HTTP 503. Assert retry-state key updated (`attempt: 2`).
7. After 5 failed attempts, assert dead-letter row: `"{org_id}:\x03:{action_id}:{alert_id}"` written (DELIVERY_FAILURE row).
8. Assert audit event `action_delivery_failed` with `attempts: 5`, `discriminator: "\x03"`.
9. Assert source alert UUID-A2 is NOT deleted from `alerts` CF (INV-ACTION-001).

**Expected Outcome:**
- Success path: `\x00` → `\x01` → `\x02`; all intermediate keys cleaned up.
- Failure path: `\x00` → `\x01` → `\x04` (retry × 5) → `\x03` (dead-letter).
- Discriminator bytes match ADR-016 §2.5 canonical 5-row schema exactly.
- All keys carry `{org_id}:` prefix per ADR-008.
- `idempotency_key` = `alert_id` (UUID v7) for alert-trigger actions.
- Source alert not modified after delivery failure (INV-ACTION-001).

**Repos Tested:** prism-operations (S-4.08 action_state CF writes, BC-2.18.001/008), prism-storage (action_state CF key schema, ADR-016 §2.5, ADR-008)

---

## HS-012-05: DELIVERY_TERMINAL State (`\x04` Semantics) — No Further Transitions

**Title:** Once an action_state CF entry reaches DELIVERY_FAILURE (`\x03`) or DELIVERY_SUCCESS (`\x02`), no further state transitions occur; the terminal state is immutable.

**Preconditions:**
- Action UUID-X has reached `\x03` dead-letter state (DELIVERY_FAILURE after 5 retries)
- A new alert trigger fires that would normally re-execute UUID-X's action

**Steps:**
1. Assert `action_state` CF contains key `"{org_id}:\x03:{action_id}:{alert_id}"` for UUID-X.
2. Inject a second trigger for the same `action_id` and same `alert_id`.
3. Assert the delivery engine recognizes the existing `\x03` terminal row and does NOT write a new `\x00` PENDING_ACTION row for the same `(action_id, alert_id)` pair.
4. Assert the `\x03` row is unchanged (no overwrite).
5. Assert no delivery attempt is made (no HTTP call, no `\x01` DELIVERY_ATTEMPT row).
6. Assert a WARN-level log entry: `"action {action_id} for alert {alert_id} already in terminal state \x03; skipping re-delivery"`.
7. Verify the same terminal immutability for a `\x02` DELIVERY_SUCCESS row (success is also terminal; cannot re-deliver).

**Expected Outcome:**
- Terminal states `\x02` and `\x03` are immutable — no transition out of them.
- Re-trigger for the same `(action_id, alert_id)` is a no-op (idempotent guard on terminal state).
- WARN log confirms the terminal skip for `\x03`.
- No new rows written; CF state unchanged after re-trigger.
- Terminal immutability applies symmetrically to both `\x02` (success) and `\x03` (failure) states.

**Repos Tested:** prism-operations (S-4.08 terminal-state guard, BC-2.18.001, ADR-016 §2.5), prism-storage (action_state CF, BC-2.18.001)

---

## HS-012-06: Multi-Tenant Action Delivery Isolation — Org A's Actions Invisible to Org B

**Title:** OrgA's action executions and `action_state` CF entries are invisible to OrgB; per-org permit accounting prevents cross-org interference.

**Preconditions:**
- OrgA has 3 active actions configured; OrgB has 2 active actions configured
- Both orgs have triggered alert-based actions simultaneously
- All `action_state` CF keys carry `{org_id}:` prefix per ADR-008

**Steps:**
1. Trigger OrgA's 3 actions and OrgB's 2 actions simultaneously.
2. Assert OrgA's `action_state` CF entries all begin with `"org-a:"` (direct key enumeration).
3. Assert OrgB's `action_state` CF entries all begin with `"org-b:"`.
4. Assert no `action_state` key exists without an org prefix.
5. Assert OrgA's audit log entries contain `org_id: "org-a"` and do NOT contain any `"org-b"` references.
6. Assert OrgB's audit log entries contain `org_id: "org-b"` and do NOT contain any `"org-a"` references.
7. Assert that OrgA's semaphore saturation (all 8 permits held by OrgA's actions) does NOT prevent OrgB's actions from attempting delivery (permit pools are NOT per-org — both orgs share the 8-permit pool; this sub-scenario verifies that partial OrgA saturation still allows OrgB through the same pool fairly, and that there is no cross-org state leakage in CF keys or audit events).

**Expected Outcome:**
- All `action_state` CF keys carry correct `{org_id}:` prefix per ADR-008.
- Zero cross-org CF key contamination.
- Audit events contain only the owning org's `org_id` field.
- The shared 8-permit `action_delivery_semaphore` is org-agnostic at the semaphore layer (fairness is not per-org; both orgs compete for the same pool); key-level isolation is purely CF-prefix based.
- No panic, no cross-org data reference in any log entry.

**Repos Tested:** prism-operations (S-4.08 org-scoped action dispatch, BC-2.18.008), prism-storage (action_state CF ADR-008 key prefix enforcement)

---

## State Checkpoint

```yaml
scenario_group: HS-012
title: Action Delivery
scenarios: 6
priority: P0
must_pass: true
wave: 4
stories_covered: [S-4.08]
bcs_anchored:
  - BC-2.18.001
  - BC-2.18.002
  - BC-2.18.003
  - BC-2.18.004
  - BC-2.18.005
  - BC-2.18.006
  - BC-2.18.007
  - BC-2.18.008
  - BC-2.18.009
key_invariants:
  - D-209-independent-8-permit-action-semaphore
  - VP-045-try-acquire-non-blocking-10ms
  - ADR-016-2.5-discriminator-state-machine
  - ADR-008-org-prefixed-action-state-cf
  - INV-ACTION-001-at-least-once-delivery
  - INV-ACTION-004-try-acquire-only
status: draft
introduced: cycle-1
```
