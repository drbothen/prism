---
document_type: holdout-scenario
level: L3
id: "HS-009"
category: "scheduler-operations"
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
  - BC-2.12.001
  - BC-2.12.002
  - BC-2.12.003
  - BC-2.12.004
  - BC-2.12.006
  - BC-2.12.008
  - BC-2.12.009
  - BC-2.12.010
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

# HS-009: Scheduler Operations

**Group:** Schedule CRUD lifecycle, 60-second tick semantics, ScheduleFireMissed audit event, pack rotation, and multi-tenant isolation for the S-4.01 scheduler subsystem.
**Date:** 2026-05-04
**Priority:** P0

---

## Scenario

Six sub-scenarios covering the Wave 4 scheduler subsystem (S-4.01). These scenarios validate that schedules are correctly created, listed, deleted, and persisted across restarts (BC-2.12.001/002/003/010); that the execution loop fires on the **60-second default tick** (ADR-013 §2.1) rather than the erroneously-spec'd 1-second value (HIGH-001 from Pass 6); that ScheduleFireMissed audit events are emitted when a tick overruns its deadline; that retiring a pack invalidates pending fires for that pack; that org-prefixed CF keys (ADR-008) enforce multi-tenant schedule isolation; and that disabling and re-enabling a schedule preserves the schedule_id while pausing tick fires.

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.12.001 | `create_schedule` MCP Tool — Create a Scheduled Query | HS-009-01, HS-009-06 |
| BC-2.12.002 | `list_schedules` MCP Tool — List Active Schedules with Next Run Times | HS-009-01, HS-009-05 |
| BC-2.12.003 | `delete_schedule` MCP Tool — Remove a Schedule (Confirmation Required) | HS-009-01 |
| BC-2.12.004 | Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip | HS-009-02, HS-009-03, HS-009-04 |
| BC-2.12.006 | Epoch/Counter Tracking — Exactly-Once Semantics, Persist After Each Run | HS-009-02, HS-009-03 |
| BC-2.12.008 | Pack Loading and Discovery — Load Packs from Config, Conditional Execution | HS-009-04 |
| BC-2.12.009 | Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack` | HS-009-04 |
| BC-2.12.010 | Schedule State Persistence — RocksDB Domain for Scheduling Metadata | HS-009-01, HS-009-06 |

## Verification Approach

All sub-scenarios run within the DTU harness in logical isolation mode. Two org contexts (OrgA / OrgB) are instantiated for multi-tenant sub-scenarios (HS-009-05). The scheduler tick interval is driven by a tokio test runtime with time acceleration so that 60-second virtual ticks elapse within milliseconds of wall time. Audit event emission is verified by subscribing to the structured log stream and asserting the presence of `ScheduleFireMissed` entries with correct `org_id` and `schedule_id` fields. CF key prefixes are validated by directly inspecting RocksDB key bytes to confirm `{org_id}:` prefix per ADR-008.

For HS-009-02, the test harness explicitly asserts that zero tick fires occur during the first 59 virtual seconds and that the first fire occurs at or after the 60-second mark, verifying the ADR-013 §2.1 default. For HS-009-03, a simulated slow query (>60s) is injected to trigger the ScheduleFireMissed audit condition. For HS-009-06, the schedule disable/re-enable roundtrip is verified by asserting schedule_id invariance across the pause/resume cycle via `list_schedules`.

## Evaluation Rubric

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Schedule CRUD correctness (create/list/delete persist and retrieve) | 25% | 100% — must-pass |
| 60-second default tick (no 1-second fires, first fire at ≥60s) | 25% | 100% — must-pass (ADR-013 §2.1) |
| ScheduleFireMissed audit event emitted on tick overrun | 15% | ≥90% |
| Pack rotation invalidates pending fires for retired pack | 15% | ≥90% |
| Multi-tenant CF key isolation (Org A keys invisible to Org B) | 15% | 100% — must-pass (ADR-008) |
| Schedule disable/re-enable preserves schedule_id | 5% | ≥80% |

Total: 100%. Overall PASS threshold: all must-pass criteria at 100%, aggregate weighted score ≥85%.

## Edge Conditions

- Scheduler restart with `next_run` in the past: must fire on the next tick, not immediately at startup (EC-12-009 per BC-2.12.004).
- 100+ schedules all due on the same tick: at most 8 concurrent executions (schedule_executor_semaphore cap per ADR-013 §2.3 + D-209); remaining schedules emit E-SCHED-004 and retry next tick (EC-12-011 per BC-2.12.004).
- Malformed org_slug used as org_id prefix: path traversal prevention must reject slugs containing directory separators.
- Schedule deleted mid-execution: in-flight execution for the deleted schedule must complete (no panic); subsequent ticks skip the deleted schedule.
- Deserialization failure of persisted schedule state: that schedule is disabled with a warning (E-STORE-003 per BC-2.12.010); other schedules continue unaffected.

## Failure Guidance

If HS-009-02 fails (tick firing at wrong interval): verify `PRISM_SCHEDULER_TICK_SECS` default is 60, not 1. Examine S-4.01 task 2 (tick loop implementation) and confirm alignment with ADR-013 §2.1. If HS-009-03 fails (ScheduleFireMissed not emitted): check audit emitter integration in the schedule executor loop and verify the overrun detection condition compares `completion_time > next_run_at` correctly. If HS-009-05 fails (cross-org leakage): verify all RocksDB CF keys begin with `{org_id}:` prefix per ADR-008 universal re-keying. Open TDs: none; this is a clean-sheet Wave 4 story (S-4.01).

## Category: architectural-invariant

Source: ADR-013 §2.1 (60-second tick), D-209 (8-permit semaphore split), ADR-008 (org_id-prefixed CF keys). Must-pass: true.

---

## HS-009-01: Schedule CRUD Lifecycle with Org-Prefixed CF Keys

**Title:** Create, list, update, and delete a schedule; verify RocksDB keys carry `{org_id}:` prefix per ADR-008.

**Preconditions:**
- Prism server initialized with RocksDB `schedules` column family
- OrgA is registered; OrgA's MCP session is active
- No pre-existing schedules for OrgA

**Steps:**
1. Call `create_schedule` with `{ name: "s1", query: "SELECT * FROM crowdstrike_alerts", interval: 300, org_id: "org-a" }`. Capture returned `schedule_id`.
2. Call `list_schedules` for OrgA. Assert `schedule_id` and `name` appear in the response.
3. Assert the RocksDB key for this schedule matches the pattern `"org-a:sched:s1"` (ADR-008 org_id prefix + BC-2.12.010 key schema).
4. Call `delete_schedule` with `{ schedule_id: <captured_id>, confirmation: true }`.
5. Call `list_schedules` again. Assert the schedule is absent.
6. Assert RocksDB key `"org-a:sched:s1"` has been deleted (key read returns `None`).

**Expected Outcome:**
- `create_schedule` returns a valid `schedule_id` (UUID v7).
- `list_schedules` shows the schedule with correct `name`, `interval`, and `next_run_at`.
- RocksDB key carries `"org-a:"` prefix confirming ADR-008 compliance.
- `delete_schedule` returns success; subsequent `list_schedules` omits the schedule.
- RocksDB key is deleted after `delete_schedule`.
- All operations emit audit entries with `org_id: "org-a"` as a structured field.

**Repos Tested:** prism-operations (S-4.01 schedule CRUD, BC-2.12.001/002/003/010), prism-storage (RocksDB `schedules` CF, ADR-008 key prefix)

---

## HS-009-02: 60-Second Default Tick Fires Correctly per ADR-013 §2.1

**Title:** Scheduler executes the first tick at exactly 60 seconds, not at 1 second (ADR-013 §2.1 regression guard).

**Preconditions:**
- Prism server started with default configuration (no `PRISM_SCHEDULER_TICK_SECS` override)
- One enabled schedule with `interval: 300` (5-minute query interval) for OrgA
- Tokio test runtime with time acceleration enabled; virtual time controllable

**Steps:**
1. Start the schedule execution loop. Record `t0` (virtual start time).
2. Advance virtual time by 59 seconds. Assert no tick has fired (no query execution, epoch counter still 0).
3. Advance virtual time to 60 seconds. Assert exactly one tick fires.
4. Verify the schedule's epoch counter increments from 0 to 1 after the tick (BC-2.12.006).
5. Advance virtual time to 119 seconds. Assert no second tick fires.
6. Advance to 120 seconds. Assert second tick fires; epoch increments to 2.

**Expected Outcome:**
- Zero tick fires before the 60-second mark.
- First tick fires at the 60-second mark (within ±100ms wall time with acceleration).
- Epoch counter increments exactly once per tick (BC-2.12.006 exactly-once semantics).
- No tick fires at 1-second intervals (confirming HIGH-001 from Pass 6 is fully resolved).
- `next_run_at` is updated to `t_fire + 60s` after each tick.

**Repos Tested:** prism-operations (S-4.01 execution loop, ADR-013 §2.1), prism-storage (epoch persistence, BC-2.12.006)

---

## HS-009-03: ScheduleFireMissed Audit Event Emitted on Tick Overrun

**Title:** When a query execution overruns its 60-second slot, a ScheduleFireMissed audit event is emitted.

**Preconditions:**
- One enabled schedule for OrgA with `interval: 300`
- Injected slow-query stub that takes 75 virtual seconds to complete (exceeds 60-second tick window)
- Audit log subscriber listening for structured events

**Steps:**
1. Start the execution loop (tick = 60s default).
2. First tick fires at t=60s. The injected stub executes for 75 virtual seconds (completes at t=135s).
3. At t=120s (when the second tick would normally fire), assert the second tick is NOT fired immediately (query is still in-flight; in-flight skip per BC-2.12.004).
4. At t=135s (query completes), assert the audit log contains a `ScheduleFireMissed` entry with `{ org_id: "org-a", schedule_id: <id>, expected_at: t120, completed_at: t135, drift_ms: 15000 }`.
5. Assert epoch counter was incremented at t=135s (exactly once for the first fire).
6. Assert next tick fires at t=195s (next_run_at based on time drift compensation per BC-2.12.004).

**Expected Outcome:**
- `ScheduleFireMissed` event is present in the structured audit log.
- Event fields: `org_id`, `schedule_id`, `expected_at`, `completed_at`, `drift_ms` all populated.
- In-flight skip prevents concurrent execution for the same `(OrgId, ScheduleId)` (BC-2.12.004 invariant).
- Epoch incremented exactly once despite the overrun.
- Time drift compensation adjusts `next_run_at` correctly per ADR-013 §2.1.

**Repos Tested:** prism-operations (S-4.01 execution loop, in-flight guard, audit event emission), prism-audit (ScheduleFireMissed event schema)

---

## HS-009-04: Schedule Pack Rotation Invalidates Pending Fires for Retired Packs

**Title:** Retiring a pack via `delete_pack` causes scheduled executions referencing that pack to be skipped.

**Preconditions:**
- Pack "pack-alpha" loaded and active; schedules for OrgA reference `pack_id: "pack-alpha"`
- One pending scheduled fire for "pack-alpha" at t=60s
- RocksDB `schedules` CF initialized with pack reference metadata

**Steps:**
1. At t=55s (before tick fires), call `delete_pack` with `{ pack_id: "pack-alpha", confirmation: true }`.
2. At t=60s, the tick fires for the schedule referencing "pack-alpha".
3. Assert the execution for "pack-alpha" is SKIPPED (pack_id not found in pack registry).
4. Assert an appropriate warning log entry referencing the retired pack_id.
5. Assert epoch is NOT incremented for the skipped execution (pack_id absent = no work to do).
6. Assert other schedules (not referencing "pack-alpha") still execute normally on the same tick.

**Expected Outcome:**
- Execution is skipped for retired pack; no panic or crash.
- Warning log contains `pack_id: "pack-alpha"` and `reason: "pack_not_found"`.
- Epoch counter is not incremented for the skipped schedule.
- Other schedules on the same tick execute normally (independent of retired pack).
- `delete_pack` audit entry is present in the log prior to the skipped fire.

**Repos Tested:** prism-operations (S-4.01 pack-rotation guard, BC-2.12.008/009), prism-storage (pack_id lookup, RocksDB diff_results CF)

---

## HS-009-05: Multi-Tenant Schedule Isolation

**Title:** OrgA's schedules and schedule state are invisible to OrgB; cross-org leakage is prevented.

**Preconditions:**
- OrgA has 2 active schedules; OrgB has 1 active schedule
- Both orgs initialized in the same Prism server instance
- ADR-008 org_id-prefixed CF keys in effect

**Steps:**
1. OrgA calls `list_schedules`. Assert only OrgA's 2 schedules appear.
2. OrgB calls `list_schedules`. Assert only OrgB's 1 schedule appears.
3. Directly enumerate RocksDB `schedules` CF keys. Assert all OrgA keys begin with `"org-a:"` and all OrgB keys begin with `"org-b:"`. No key lacks an org prefix.
4. OrgA's execution loop fires at t=60s. Assert OrgB's epoch counters are NOT incremented.
5. OrgB's execution loop fires at t=60s. Assert OrgA's epoch counters are NOT incremented by OrgB's fire.
6. Attempt cross-org `get_schedule` for OrgA schedule_id via OrgB session. Assert 404 / not-found response.

**Expected Outcome:**
- `list_schedules` returns only the calling org's schedules in all cases.
- All RocksDB keys carry correct `{org_id}:` prefix per ADR-008.
- Epoch counters remain per-org isolated; cross-org fires do not increment foreign counters.
- Cross-org schedule lookup returns not-found (no data leakage of schedule metadata).

**Repos Tested:** prism-operations (S-4.01 org-scoped schedule dispatch, BC-2.12.001-004), prism-storage (ADR-008 key prefix enforcement, BC-2.12.010)

---

## HS-009-06: Schedule Disable/Re-Enable Preserves schedule_id and Pauses Tick Fires

**Title:** Disabling a schedule pauses tick fires; re-enabling resumes them; schedule_id is invariant throughout.

**Preconditions:**
- Schedule "s2" active for OrgA, `schedule_id` = UUID-S2
- Execution loop running at 60-second default tick

**Steps:**
1. At t=60s, tick fires for "s2". Assert epoch increments to 1.
2. At t=90s, call `update_schedule` (or equivalent disable operation) to disable "s2". Assert response contains the same `schedule_id` = UUID-S2.
3. At t=120s, tick fires. Assert "s2" is NOT executed (disabled status check skips it). Epoch remains at 1.
4. At t=150s, re-enable "s2" via `update_schedule`. Assert `schedule_id` is still UUID-S2 (invariant).
5. At t=180s, tick fires. Assert "s2" executes again. Epoch increments to 2.
6. Call `list_schedules`. Assert `schedule_id` = UUID-S2 with `enabled: true` and `epoch: 2`.

**Expected Outcome:**
- `schedule_id` UUID-S2 is unchanged throughout disable/re-enable cycle.
- Tick fires are paused during disabled period; no executions occur for "s2" at t=120s.
- Re-enable resumes tick fires at next tick boundary.
- Epoch counter resumes from last value (1 → 2), not reset.
- Audit log contains `schedule_disabled` and `schedule_enabled` entries with `schedule_id: UUID-S2`.

**Repos Tested:** prism-operations (S-4.01 schedule enable/disable, BC-2.12.001/002/004), prism-storage (schedule enabled flag persistence, BC-2.12.010)

---

## State Checkpoint

```yaml
scenario_group: HS-009
title: Scheduler Operations
scenarios: 6
priority: P0
must_pass: true
wave: 4
stories_covered: [S-4.01]
bcs_anchored:
  - BC-2.12.001
  - BC-2.12.002
  - BC-2.12.003
  - BC-2.12.004
  - BC-2.12.006
  - BC-2.12.008
  - BC-2.12.009
  - BC-2.12.010
key_invariants:
  - ADR-013-2.1-60s-default-tick
  - ADR-008-org-id-prefixed-cf-keys
  - D-209-8-permit-schedule-semaphore
status: draft
introduced: cycle-1
```
