---
document_type: adr
adr_id: ADR-013
title: "Schedule Execution Semantics"
status: PROPOSED
version: "0.2"
date: 2026-05-02
wave: 4
phase: 4.A
authors: [architect]
producer: architect
timestamp: 2026-05-02T00:00:00Z
inputs:
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/architect-adr-identification.md
  - .factory/cycles/wave-4-operations/preflight-findings/research-findings.md
  - .factory/STATE.md (D-207..D-213)
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
  - .factory/specs/architecture/decisions/ADR-012-src-convention.md
anchor_stories:
  - S-4.01
  - S-4.08
aligns_with:
  - ADR-006
  - ADR-008
  - ADR-010
  - ADR-012
supersedes: []
superseded_by: null
amendments: []
related_decisions:
  - D-207
  - D-208
  - D-209
  - D-211
  - D-212
locked_decisions:
  - D-209
references_decisions:
  - D-211
verification_properties:
  - VP-026
  - VP-030
  - VP-137
subsystems_affected: [SS-12]
traces_to: specs/architecture/ARCH-INDEX.md
---

# ADR-013: Schedule Execution Semantics

## Status

PROPOSED 2026-05-02, v0.2. Pending review and acceptance prior to story remediation and BC authoring.

---

## 1. Context

### 1.1 The Schedule Executor

The `prism-operations` crate (introduced per ADR-012 `src/` convention, SS-04) will include a schedule executor that drives tick-based query execution. S-4.01 defines the CRUD surface (create/list/delete schedule MCP tools) and the execution loop. S-4.08 defines action delivery, which is a sibling subsystem sharing the same crate but using a distinct execution pathway.

The executor's job: on each tick, evaluate which persisted schedules are fire-eligible given the current wall-clock time plus their splay offset, then execute the matching schedules subject to an in-flight concurrency cap. Schedules are persisted in RocksDB, org-scoped per D-208, and executed on behalf of a specific `(OrgId, ScheduleId)` tuple.

### 1.2 Why This ADR Exists

The S-4.01 story draft as of 2026-04-16 contained several under-specified or conflicting design points that, if left to per-story implementation choice, would produce inconsistent behavior or verifiability gaps:

- **Semaphore sharing conflict.** S-4.01 specified a 16-permit shared semaphore between schedule execution and action delivery. D-209 (LOCKED 2026-05-02) overrides this: independent 8-permit semaphores per subsystem, no sharing. The story text must be remediated; this ADR is the authoritative record.
- **Cron library gap.** Story drafts cited `cron 0.12.x`. R-2 (research-findings.md) found `croner 3.0.1` is the correct choice for DST/timezone correctness. Story drafts must be updated.
- **Missed-fire policy unspecified.** Neither S-4.01 nor any BC defines behavior when a tick is missed due to system pause or semaphore exhaustion. This ADR establishes skip-not-queue.
- **Schedule-change notification.** D-211 requires that dedup-window resolutions be invalidated on schedule change. The notification mechanism (reload hook) must be established here; its consumers are ADR-015's responsibility.

### 1.3 Scope of This ADR

This ADR is scoped to:
- Tick interval and configurability
- Splay strategy and hash function
- Per-subsystem semaphore allocation (D-209)
- Missed-fire policy
- In-flight skip determinism
- Schedule store CF design
- Schedule-change reload hook (notification only; dedup mechanics deferred to ADR-015)
- Cron expression library selection

This ADR does NOT define:
- Alert dedup-window resolution mechanics (ADR-015)
- Action delivery semaphore behavior beyond citing D-209 (ADR-016)
- SIEM output formats (ADR-019)

---

## 2. Decision

### 2.1 Tick Interval

**Decision:** The scheduler tick interval defaults to 60 seconds. It is configurable via the environment variable `PRISM_SCHEDULER_TICK_SECS`, accepting integer values in the range [10, 3600]. Values outside this range are rejected at startup with a clear configuration error message citing the valid range.

The tick rate governs the minimum fire-check frequency. It is not the cron expression precision — actual fire times are computed against the cron expression (see §2.8) and may fire at sub-minute precision if the tick is configured tightly enough.

Rationale: 60 seconds matches common operational cadence (default polling intervals, alert rule evaluation cycles). The configurable override follows the `PRISM_*` env-var convention established in ADR-010 for scheduler-related tuning parameters.

### 2.2 Splay Strategy

**Decision:** Splay offsets are computed per-schedule at schedule-load time, not per-tick. The formula is:

```
splay_seconds = u64::from_le_bytes(blake3_hash(schedule_id_bytes)[0..8]) % (interval_seconds / 4)
splay_seconds = min(splay_seconds, 900)  // cap at 15 minutes
```

The hash function is blake3 (per R-4: `blake3 = "1.8"`, no CVEs, workspace standard). Only the first 8 bytes of the 32-byte hash output are used via `u64::from_le_bytes` (little-endian, explicitly pinned) to derive the u64 modulus operand; the full entropy of schedule_id is mixed into the splay without bias from the low-order bits alone.

`interval_seconds` is a derived field computed at schedule-load time from the `ScheduleEntry.cron_expr` via `croner::Cron` shortest-period analysis (see §2.6 for the `ScheduleEntry` struct). It is not stored independently in the cron expression; it is computed once at load and cached alongside the splay offset in the in-memory splay-target cache.

Splay is stable for the lifetime of a schedule: the same `schedule_id` always maps to the same splay offset. Splay is recomputed only on schedule reload (see §2.7). This guarantees that VP-026 (splay computation deterministic per schedule_id) is satisfied.

The 15-minute cap ensures that no schedule is artificially deferred beyond a quarter-hour from its nominal fire time regardless of how large the interval is. The `interval/4` modulus ensures splay is proportionate: a 60-second interval splays up to 15 seconds; a 3600-second (1-hour) interval splays up to 900 seconds (15 minutes, capped).

### 2.3 Per-Subsystem Semaphore Allocation (D-209 — LOCKED)

**Decision (D-209 LOCKED):** Two independent `Arc<Semaphore>` instances are constructed at `prism-operations` startup:

| Semaphore | Owner Module | Permits | Purpose |
|---|---|---|---|
| `schedule_executor_semaphore` | `schedule/executor.rs` | 8 | Caps concurrent schedule executions |
| `action_delivery_semaphore` | `action/delivery.rs` | 8 | Caps concurrent action deliveries |

**There is no shared semaphore.** Each semaphore is constructed in its owning module's init function and is not visible to the other module. Cross-module handle passing is prohibited.

Rationale for the split: D-209 identified a cross-subsystem starvation hazard in the original 16-permit shared design. If action delivery claims all 16 permits during a burst (e.g., a policy violation wave), schedule execution is starved and vice versa. With independent 8-permit pools, each subsystem has a guaranteed floor regardless of the other's load. 8 permits per subsystem preserves the original total concurrency budget (16) while eliminating starvation.

**Liveness invariant (VP-137):** Schedule execution cannot be starved by action delivery; action delivery cannot be starved by schedule execution. Proof: the semaphores are disjoint; no code path holding `schedule_executor_semaphore` can block acquisition of `action_delivery_semaphore` or vice versa.

Note: the pre-existing S-4.01 story text specifying "16-permit shared semaphore" is superseded by this ADR. Story-writer must remediate S-4.01 and S-4.08 to reflect the 8/8 split. This is a story drift item tracked in the Wave 4 drift audit.

### 2.4 Missed-Fire Policy

**Decision:** skip-not-queue. When the executor tick detects that a schedule's splay-adjusted fire time fell in the past (e.g., the process was paused, the machine was suspended, or semaphore exhaustion prevented the fire from executing), the missed fire is:

1. Recorded in the audit log as a `ScheduleFireMissed` event with: `schedule_id`, `org_id`, `nominal_fire_time`, `actual_tick_time`, `miss_reason` (one of: `SemaphoreExhausted`, `ProcessPaused`, `InitialLoad`).
2. NOT enqueued for retry. The executor advances the next expected fire time to the next splay-adjusted target as computed from `now()`.

Rationale: queue-and-retry creates a thundering-herd hazard after a recovery event. If a process pauses for 10 minutes and 50 schedules missed their windows, queuing all 50 for immediate execution would saturate the semaphore and potentially flood downstream sensors. Skip-with-audit gives operators a clean, auditable signal for investigating missed fires without creating cascading load.

**Operator contract:** operators relying on at-least-once schedule semantics must configure an external supervisory alert on `ScheduleFireMissed` event rate.

### 2.5 In-Flight Skip Determinism

**Decision:** When a schedule's tick evaluation finds that schedule already has an in-flight execution (spawned by a prior tick that is still running), the tick skips that schedule deterministically. No second execution is spawned.

In-flight state is tracked in an in-memory `DashMap<(OrgId, ScheduleId), JoinHandle<()>>` inside `schedule/executor.rs`. On each tick:

1. For each fire-eligible schedule: check if `(org_id, schedule_id)` exists in the in-flight map.
2. If present and the `JoinHandle` reports `!is_finished()`: skip; emit `ScheduleFireSkipped { reason: InFlight }` to audit.
3. If present and `is_finished()` (task completed since last tick): remove the stale entry; proceed with normal fire evaluation.
4. If absent: acquire semaphore permit (try_acquire, non-blocking per VP-045); if acquired, spawn task, insert handle into map.

This guarantees VP-026: the fire-eligibility decision is deterministic for a given `(OrgId, ScheduleId)` at any tick. Two concurrent ticks (impossible by design — the tick loop is single-threaded) cannot both fire the same schedule.

The `DashMap` is not persisted. On process restart, the in-flight map is empty and all schedules are treated as idle. This is correct: the scheduler must re-evaluate fire eligibility from persisted `next_run_at` timestamps on startup.

**`next_run_at` write-ordering:** `next_run_at` is NOT updated at the moment of schedule fire. It is written to RocksDB only at fire-completion (after the query result is returned and diff computation finishes). This means `next_run_at` in the persisted `ScheduleEntry` lags the in-memory splay-target cache during active execution. On crash before `next_run_at` is updated, the schedule may double-fire on next startup; idempotency at the alert/case layer prevents duplicate-effect side-effects. `next_run_at` is held in a separate in-memory cache during the fire window and only flushed on completion.

### 2.6 Schedule Store Column Family Design

**Decision:** RocksDB column family for schedules:

| Field | Value |
|---|---|
| CF name | `schedules` |
| Key format | `{org_id_bytes}:{schedule_id_bytes}` |
| Key prefix | `{org_id_bytes}:` per ADR-008 universal re-keying rule |
| Value encoding | bincode 2.x with serde feature (workspace standard) |
| Value type | `ScheduleEntry` (Rust struct, includes: `schedule_id: ScheduleId`, `org_id: OrgId`, `cron_expr: String`, `next_run_at: DateTime<Utc>`, `splay_seconds: u64`, `created_at: DateTime<Utc>`, `client_scope: Vec<ClientId>`, `enabled: bool`, `interval_seconds: u64`) |

The `{org_id_bytes}:` prefix satisfies ADR-008's universal re-keying rule: every cross-tenant isolation guarantee that ADR-008 makes for DTU state also applies to schedule entries. Per-org `reset_for(org_id)` semantics work correctly because all org-A schedule keys share the same CF prefix and can be deleted by prefix-scan without touching org-B entries.

**`enabled` field:** Defaults to `true`. Capability-gated packs (per ADR-018 §2.5) set this to `false` for schedules belonging to packs whose required capability flag is disabled for the org. The schedule executor tick loop skips any `ScheduleEntry` where `enabled == false`.

**`interval_seconds` field:** Derived at schedule-load time from `croner::Cron` shortest-period analysis of `cron_expr`. Not stored independently — computed once at load and persisted in the `ScheduleEntry` for use by the splay formula (§2.2) and dedup-window resolution (ADR-015 §2.7) without re-parsing the cron expression on every tick.

**Per-org schedule cap:** The cap is **per-org**, default 500, overridable via `PRISM_MAX_SCHEDULES`. VP-030 harness range `(0, 10000]` is the enforcement bound. `create_schedule` performs a prefix-count scan before insertion; if the count meets or exceeds the cap, it returns `Err(ScheduleLimitExceeded)`.

### 2.7 Schedule-Change Reload Hook

**Decision:** When a schedule is created, updated, or deleted via the S-4.01 CRUD tools, the schedule executor receives a notification via a `tokio::sync::watch` channel. The channel carries a `ScheduleChangeNotification` enum:

```
enum ScheduleChangeNotification {
    Created(ScheduleId),
    Updated(ScheduleId),
    Deleted(ScheduleId),
}
```

On receipt, the executor:

1. Removes the affected `schedule_id` from its in-memory splay-target cache (`Vec<(DateTime<Utc>, ScheduleEntry)>`).
2. For `Created` and `Updated`: reloads the entry from RocksDB; recomputes splay; inserts into the cache.
3. For `Deleted`: the removal in step 1 is sufficient. Additionally, the `Deleted` handler MUST also remove the entry from the in-flight `DashMap<(OrgId, ScheduleId), JoinHandle<()>>` (step 4 of §2.5). If the schedule is currently in-flight, the in-flight task completes but no subsequent fire is scheduled and no stale DashMap entry remains.

This hook is the enabler for D-211: when a detection rule's associated schedule changes, the dedup-window resolution previously baked into the `RuleCondition` is invalidated. ADR-015 will define the precise invalidation semantics for dedup-window resolution. This ADR establishes only that the notification exists and that the executor's in-memory cache is invalidated on schedule change.

The `watch` channel sender is owned by the CRUD layer (`crud.rs`); the receiver is owned by the executor (`executor.rs`). The channel is constructed at module init and wired via dependency injection into both components. This is consistent with ADR-012's `src/` layout convention for `prism-operations`.

### 2.8 Cron Expression Library

**Decision:** Use `croner = "3"` (current: 3.0.1, R-2). Do not use the `cron` crate (0.15.0 or any earlier version).

Rationale (per R-2):
- `croner` 3.0.1 provides DST-correctness, timezone awareness via `chrono-tz`, and `L`/`#`/`W` Quartz-compatible extensions. MSSP schedule tenants span arbitrary timezones; DST transitions on naive UTC schedules produce silent misfires.
- `cron` 0.15.0 has no DST awareness and no timezone handling. It would silently mis-fire at DST boundaries for local-timezone schedules.
- `tokio-cron-scheduler` is rejected because it couples the scheduler lifecycle to its own runtime model and adds persistence dependencies (PostgreSQL, NATS) that conflict with Prism's RocksDB-first persistence architecture.

The cron expression is stored as a raw string in `ScheduleEntry.cron_expr` and parsed at schedule-load time into a `croner::Cron` instance. The parsed `Cron` instance is held in the in-memory splay-target cache alongside the splay offset. Re-parsing on every tick is rejected (CPU waste at scale with many schedules).

Story drafts referencing `cron 0.12.x` in S-4.01 and S-4.08 must be remediated. This is a story drift item tracked in the Wave 4 drift audit.

---

## Rationale

The eight decisions in Section 2 are jointly necessary. Each addresses a distinct failure mode that would otherwise manifest as either a correctness defect, a verifiability gap, or an operational hazard.

**Per-subsystem semaphores (§2.3) are required for formal liveness.** The core constraint driving D-209 is not performance tuning — it is VP-137's liveness proof. A shared semaphore cannot be proven starvation-free between two independent consumers unless one is given priority, which introduces its own complexity. Disjoint semaphores make the proof trivial: no code path in one subsystem can block the other. This satisfies the liveness invariant stated in VP-137 with a structural argument rather than a runtime one.

**`croner` (§2.8) is required for multi-tenant correctness.** The MSSP deployment model places schedules across customer tenants in arbitrary timezones (memory: `project_deployment_model.md`). A cron parser with no DST awareness silently misfires at spring/fall transitions — a failure mode that is operationally expensive to diagnose because the misfire is invisible at the application layer (no error is raised; the schedule simply does not fire). R-2 (research-findings.md) documents that `cron 0.15.0` has no DST handling. `croner 3.0.1` strictly dominates for this use case.

**Skip-not-queue (§2.4) is required for recovery safety.** The missed-fire policy must be chosen before implementation because it defines the audit contract. Queue-and-retry creates a well-known thundering-herd hazard after maintenance windows: a 10-minute pause with 50 active schedules produces 50 immediate catch-up executions against downstream sensors, which may themselves be under maintenance or rate-limited. Skip-with-audit preserves the audit trail without creating cascading load. This choice directly satisfies the BC-2.12.004 execution loop contract (post-restart steady-state behavior).

**`watch`-channel reload hook (§2.7) is required to satisfy D-211.** ADR-015 (alert dedup-window resolution) requires that cached dedup-window values be invalidated when a schedule changes. The invalidation path must exist before ADR-015 can be authored; establishing the channel here gives ADR-015 a concrete notification primitive to build on. The `watch` channel is the correct Tokio primitive for this: it delivers only the latest notification (no queuing), is single-producer/multi-consumer, and has no backpressure, which matches the invalidation semantics (only the most recent state matters).

**Blake3 splay (§2.2) satisfies VP-026's determinism requirement while being fast enough to recompute on reload.** The splay formula uses only the `schedule_id` bytes and the interval as inputs, both of which are stable for the lifetime of a schedule. Blake3 was chosen over FxHash or AHash because it is already a workspace-standard hash (R-4) and its 32-byte output provides uniform distribution across the modulus range without bias from low-order bit clustering.

---

## 3. Consequences

### 3.1 Positive

- **Formal liveness guarantee.** Independent semaphores (D-209) enable VP-137 to be proven: no cross-subsystem starvation is structurally possible. This is a stronger guarantee than a shared-semaphore design, which would require reasoning about priority inversion.
- **Deterministic, auditable scheduling.** Splay computed at load time (not per-tick) means every fire time is deterministic and stable across process restarts (splay is reloaded from the persisted `ScheduleEntry`). VP-026 proofs are straightforward.
- **Clean recovery semantics.** Skip-not-queue means post-restart behavior is identical to steady-state behavior. Thundering-herd hazards after maintenance windows are eliminated by design.
- **D-211 foundation.** The `watch`-channel reload hook gives ADR-015 a clean, already-wired notification path for dedup-window invalidation. No additional IPC mechanism is needed.
- **DST correctness.** `croner` eliminates a class of silent misfires that would be operationally expensive to diagnose in a multi-tenant MSSP environment.

### 3.2 Negative

- **Potential under-utilization.** With independent 8-permit pools, if schedule execution is idle but action delivery is saturated (or vice versa), the idle pool's permits are wasted. The original 16-permit shared pool would fully utilize capacity in this scenario. At 8 permits per subsystem, peak concurrency is halved vs. a shared-pool design when one subsystem dominates. Operators with high action-delivery load and low schedule load (or vice versa) cannot tune this asymmetrically without a code change. Mitigation: 8 permits is generous for the initial wave; this can be revisited in a future ADR amendment if performance profiling reveals systematic under-utilization.
- **Reload hook adds complexity.** The `watch` channel adds a coordination path between CRUD and executor that did not exist in a purely tick-driven design. Test coverage must verify that the channel is correctly wired and that the executor handles slow/buffered notifications without correctness regression. This is addressed by VP-026 (determinism test includes post-reload fire-time correctness).
- **In-memory `DashMap` lost on restart.** Schedules that were in-flight at process shutdown are not tracked on restart. The first tick after restart will evaluate fire eligibility from persisted `next_run_at` timestamps, potentially firing schedules that were "skipped" during shutdown. This is consistent with the skip-not-queue policy: any fire that missed its window is simply not retried.

---

## 4. Alternatives Considered

### 4.1 Shared 16-Permit Semaphore (Rejected — D-209)

The S-4.01 story draft proposed a single 16-permit `Arc<Semaphore>` shared between schedule execution and action delivery. This was rejected by D-209 for the starvation hazard documented in §2.3. No further analysis is needed; D-209 is LOCKED.

### 4.2 `cron` Crate 0.15.0 (Rejected — R-2)

The `cron` crate (zslayton/cron) 0.15.0 is the latest version of the original cron parser used in early story drafts. It is rejected because:
- No DST awareness: schedules expressed in local timezones mis-fire at spring/fall transitions.
- No timezone handling: all computations are UTC-only.
- No `L`/`#`/`W` Quartz extensions: limits expressibility for end-of-month / nth-weekday schedules common in MSSP operational patterns.

`croner 3.0.1` strictly dominates `cron 0.15.0` for this use case (R-2).

### 4.3 Queue-on-Missed-Fire (Rejected)

An alternative missed-fire policy would queue all missed fires for catch-up execution after recovery. This is rejected because:
- After a 10-minute maintenance window with 50 active schedules (a realistic MSSP deployment), 50 missed fires would all become immediately eligible, saturating the 8-permit semaphore and flooding downstream sensors with burst load.
- "Catch-up" semantics are inconsistent with the polling/sampling nature of MSSP sensor queries: a Claroty device inventory poll from 9 minutes ago is stale, not useful.
- The audit record of missed fires (skip-not-queue policy) gives operators the same observability with zero thundering-herd risk.

### 4.4 Per-Tick Splay Recomputation (Rejected)

Recomputing splay on every tick was considered as a simplification (no splay cache required). Rejected because:
- VP-026 requires determinism across ticks for a given schedule. If splay were recomputed from a time-dependent seed, determinism could not be proven.
- The in-memory splay cache is small (one `u64` per schedule). The overhead of caching is negligible.

### 4.5 `tokio-cron-scheduler` (Rejected — R-2)

The `tokio-cron-scheduler` crate provides a full async scheduler with optional PostgreSQL/NATS persistence. It is rejected because:
- Couples scheduler lifecycle to its own task management model, conflicting with Prism's single tick-loop architecture.
- Persistence backends (PostgreSQL, NATS) conflict with Prism's RocksDB-first persistence architecture (ADR-008).
- Adds significant dependency weight for capabilities Prism does not need (external storage, job registry API).

---

## Phase 4.A Pass 1 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 1 fix-burst (2026-05-02). Version bumped 0.1 → 0.2.

- **P1-ADR-013-A-H-001 fix:** `subsystems_affected` corrected from `[SS-04]` to `[SS-12]` (Scheduler). SS-04 = Feature Flags; schedule executor lives in SS-12 per ARCH-INDEX Subsystem Registry.
- **P1-ADR-013-A-M-002 fix:** Splay formula now references `interval_seconds` as an explicit derived field in `ScheduleEntry`, computed at load time from `croner::Cron` shortest-period analysis. §2.6 struct updated to include the field.
- **P1-ADR-013-A-M-003 fix:** Endianness explicitly pinned: `u64::from_le_bytes(hash[0..8])` in §2.2 formula.
- **P1-ADR-013-A-M-004 fix:** `enabled: bool` field added to `ScheduleEntry` in §2.6. Defaults `true`; capability-gated packs set `false` for disabled-capability schedules.
- **P1-ADR-013-A-M-005 fix:** `next_run_at` write-ordering documented in §2.5: written only at fire-completion, not at fire-start. Crash-before-update may cause double-fire; idempotency at alert/case layer is the mitigation.
- **P1-ADR-013-A-M-006 fix:** `Deleted` notification handler in §2.7 now explicitly requires removing the entry from the in-flight DashMap in addition to the splay-target cache.
- **P1-ADR-013-A-M-007 fix:** Cap source-of-truth in §2.6 clarified: per-org, default 500, override via `PRISM_MAX_SCHEDULES`, VP-030 harness range `(0, 10000]`.

---

## Source / Origin

- **Architectural decisions (STATE.md §Wave 4 Decision Log):**
  - D-207: 6-ADR topology; ADR-013 scoped to schedule execution semantics (logged 2026-05-02).
  - D-209 (LOCKED): Independent 8-permit per-subsystem semaphores; no shared semaphore (logged 2026-05-02).
  - D-211: Dedup-window resolved at scheduling-time; invalidated on schedule change; notification hook established here (logged 2026-05-02).
- **Research findings (research-findings.md):**
  - R-2 §croner: `croner 3.0.1` recommended; `cron 0.15.0` rejected for DST/timezone deficiency (2026-05-02).
  - R-4 §blake3: `blake3 = "1.8.5"`, no CVEs, workspace standard (2026-05-02).
- **Story draft (S-4.01-schedule-crud.md):**
  - Tick interval, splay formula, semaphore design, and missed-fire behavior extracted from story implementation spec section; story text contains the pre-D-209 16-permit shared semaphore and the pre-R-2 `cron 0.12.x` pin — both superseded by this ADR.
- **Prior ADRs:**
  - ADR-006 §2.1: OrgId canonical routing key; schedules are org-scoped.
  - ADR-008: Universal `{org_id}:` CF key prefix rule; `schedules` CF key format derived here.
  - ADR-010: `PRISM_*` env-var convention; `PRISM_SCHEDULER_TICK_SECS` follows this pattern.
  - ADR-012: `prism-operations` crate layout; `src/schedule/` and `src/action/` as sibling modules.
- **Verification properties:**
  - VP-026 (vp-026-splay-deterministic.md): pre-existing; harness skeleton formalized in §5.1.
  - VP-030 (vp-030-schedule-rule-caps.md): pre-existing; enforcement mechanism specified in §2.6.
  - VP-137: proposed in this ADR; VP file and VP-INDEX update to be produced before Phase 4.B BC authoring.

---

## 5. Verification Plan

### 5.1 VP-026 — Splay Computation Determinism

**Property:** `compute_splay(schedule_id) == compute_splay(schedule_id)` for all inputs; splay is in `[0, min(interval/4, 900)]`.
**Method:** Kani (model checking).
**Harness skeleton:**

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_splay_determinism() {
    let id: [u8; 16] = kani::any();
    let interval: u64 = kani::any_where(|i| *i >= 10 && *i <= 3600);
    let s1 = compute_splay(&id, interval);
    let s2 = compute_splay(&id, interval);
    kani::assert(s1 == s2, "splay must be deterministic");
    kani::assert(s1 <= interval / 4, "splay must not exceed interval/4");
    kani::assert(s1 <= 900, "splay must not exceed 900 seconds");
}
```

**Status:** draft (VP-026 file exists; harness skeleton to be added by story-writer per S-4.01 remediation).

### 5.2 VP-030 — Per-Org Schedule Count Cap Enforcement

**Property:** `create_schedule` returns `Err(ScheduleLimitExceeded)` when the count of existing schedules for the org meets or exceeds the cap.
**Method:** Kani (model checking).
**Harness skeleton:**

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_schedule_cap_enforcement() {
    let cap: usize = kani::any_where(|c| *c > 0 && *c <= 10000);
    let existing: usize = kani::any_where(|e| *e <= cap);
    let result = check_cap_before_insert(existing, cap);
    if existing >= cap {
        kani::assert(result.is_err(), "must reject at or above cap");
    } else {
        kani::assert(result.is_ok(), "must allow below cap");
    }
}
```

**Status:** draft (VP-030 file exists; harness skeleton to be added per S-4.01 remediation).

### 5.3 VP-137 — Per-Subsystem Semaphore Liveness

**Property:** Schedule execution cannot be starved by action delivery; action delivery cannot be starved by schedule execution. Formally: acquiring `schedule_executor_semaphore` never depends on releasing `action_delivery_semaphore`, and vice versa.
**Method:** Proptest (structural property; the liveness invariant is a module-boundary enforcement check, not a runtime concurrency test).
**Approach:** The test verifies that no code path in `schedule/executor.rs` holds a reference to `action_delivery_semaphore`, and no code path in `action/delivery.rs` holds a reference to `schedule_executor_semaphore`. This is enforced by module visibility (each semaphore is module-private). The proptest confirms no cross-module access in the compiled artifact by inspecting the module dependency graph.

In addition, an integration test spawns a saturated action delivery scenario (8 concurrent action permits held indefinitely) and asserts that schedule execution fires within two tick intervals, confirming no real-world starvation.

**Status:** proposed; VP-137 assigned in this ADR. VP file to be created and VP-INDEX updated before Phase 4.B BC authoring begins.

**Module:** `prism-operations`
**Priority:** P1
**Anchor story:** S-4.01 (primary), S-4.08 (secondary)

---

## 6. Migration Path

Not applicable. The `prism-operations` crate's schedule executor is greenfield for Wave 4. There is no prior schedule executor to migrate from. The `schedules` CF does not exist in production RocksDB instances prior to Wave 4 deployment.

Upgrade note for Wave 4 deployment: the `schedules` CF must be created via the RocksDB `create_cf` call during process startup if it does not exist. A missing CF is not an error on first run; it is initialized on first `create_schedule` invocation or pre-created in the startup initialization sequence (to be specified in BC-2.12.010 by the story-writer).

---

## 7. References

### Research Findings

- **R-2** (`research-findings.md §R-2`): `croner 3.0.1` recommended for DST/timezone correctness; `cron 0.15.0` rejected for no DST awareness.
- **R-4** (`research-findings.md §R-4`): `blake3 = "1.8.5"`, no CVEs, no RustSec advisories, SIMD-accelerated, workspace standard.

### Architecture Decisions

- **D-207** (STATE.md §Wave 4 Decision Log): 6-ADR topology; ADR-013 scoped to schedule execution semantics; ADR-016 owns action delivery semaphore documentation.
- **D-208** (STATE.md §Wave 4 Decision Log): OrgId/ClientId dual hierarchy; all Wave 4 domain types gain `org_id: OrgId`; RocksDB CF keys gain `{org_id}:` prefix per ADR-008.
- **D-209** (STATE.md §Wave 4 Decision Log — LOCKED): Independent 8-permit semaphores; no shared semaphore; starvation hazard eliminated.
- **D-211** (STATE.md §Wave 4 Decision Log): Dedup window resolved at scheduling-time; invalidated on schedule change. Schedule-change hook established here; dedup mechanics owned by ADR-015.

### Prior ADRs

- **ADR-006 §2.1**: OrgId is canonical routing key; schedules are org-scoped and keyed by `(OrgId, ScheduleId)`.
- **ADR-008**: Universal re-keying rule; `schedules` CF key format `{org_id}:{schedule_id}` derives directly from this rule.
- **ADR-010**: `PRISM_*` env-var convention for configuration; `PRISM_SCHEDULER_TICK_SECS` follows this convention.
- **ADR-012**: `prism-operations` crate layout; `src/schedule/` and `src/action/` as sibling modules under `crates/prism-operations/src/`.
