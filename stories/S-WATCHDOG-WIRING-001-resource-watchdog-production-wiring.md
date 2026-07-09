---
document_type: story
story_id: S-WATCHDOG-WIRING-001
title: "prism-bin + prism-query + prism-mcp + prism-storage: ResourceWatchdog Production Wiring — Boot Construction, Query-Start Denylist Gate (E-QUERY-008), force_execute, watchdog_status Unstub"
# wave: NOT wave-scheduled — post-demo backlog per orchestrator sequencing directive
# (2026-06-10 review cycle dispatch): NOT demo-blocking because the per-query
# GreedyMemoryPool (BC-2.11.006 → E-WATCHDOG-001) and the 30s query timeout
# (engine.rs tokio::time::timeout → E-QUERY-004) are already wired and protect the
# demo. Placed alongside S-CACHE-SPEC-COMPLIANCE-001 in the post-demo backlog.
wave: post-demo-backlog
epic_id: maintenance
priority: P1
# Priority rationale: P1 (not P2 like sibling S-CACHE-SPEC-COMPLIANCE-001) because two
# ACTIVE P0 behavioral contracts (BC-2.15.007 process-RSS kill path, BC-2.15.008
# denylisting) are wholly unreachable in production — a POL-15-class active-BC-without-
# production-path condition discovered by the QRY fix-burst STOP report
# (fix/review-2026-06-10-query-core cycle, 2026-06-10). The process-level RSS safety
# net and the crash-loop denylist are absent from the running binary.
# Status rationale: draft (NOT ready) despite non-empty behavioral_contracts —
# (1) orchestrator-directed sequencing: post-demo backlog; must not be dispatched
#     before the live-demo objective (T5 Story B → T6 → T8) completes;
# (2) remove-uncertainty must run against the post-demo develop baseline (boot.rs
#     step numbering, engine.rs execute() shape, and server.rs QueryToolParams are
#     all active surfaces in the in-flight demo stories and fix branches).
status: draft
version: "1.3"
level: "L4"
producer: story-writer
timestamp: "2026-06-10T00:00:00Z"
created: "2026-06-10"
modified: "2026-06-10T14:00:00Z"
phase: 3
tdd_mode: strict
subsystems: [SS-15, SS-11, SS-10, SS-22]
# Subsystem anchor justifications (per ARCH-INDEX Subsystem Registry):
#   SS-15 (Storage Layer) owns the ResourceWatchdog + denylist library this story
#     activates (prism-storage watchdog.rs / denylist.rs, `watchdog` CF) and the
#     storage-value extension (reason + denylisted_at fields, BC-2.15.008
#     postcondition 2). BC-2.15.00x are SS-15 contracts.
#   SS-11 (Query Execution) owns the execution-path integration: query-start
#     denylist gate, token register/deregister lifecycle, cancellation observation,
#     failure-record/success-reset hooks, and the SHA-256 normalized-query
#     fingerprint helper in prism-query (engine.rs).
#   SS-10 (MCP Interface) owns the `force_execute` QueryToolParams parameter and
#     the `watchdog_status` tool unstub (prism-mcp server.rs).
#   SS-22 (Process Lifecycle) owns the boot-sequence wiring contract (ADR-022 §B):
#     where in the 11-step sequence the watchdog is constructed, monitor-spawned,
#     and threaded into QueryEngine/PrismServer.
target_module: prism-query
crates_touched: [prism-bin, prism-query, prism-mcp, prism-storage]
behavioral_contracts: [BC-2.15.005, BC-2.15.007, BC-2.15.008]
# BC anchor note: boot-activation ACs trace to BC-2.15.007 precondition 2 ("The
# resource watchdog is active and monitoring (BC-2.15.006)") — this story makes that
# precondition satisfiable in production. Full BC-2.15.006 graduated-level config
# compliance (normal/restrictive/permissive profiles, TOML overrides, clamping,
# E-WATCH-001/002) is OUT OF SCOPE — see §Out of Scope for the routing flag.
# BC-2.15.005 anchor justification (added v1.1, QRY cascade P3-01 realignment):
# BC-2.15.008 v1.4's THIRD trigger — `crash` = process crash detected via a
# CrashRecovery marker on restart (BC-2.15.005) — requires the dirty-bit lifecycle
# in production. The primitives are merged (S-2.01, PR #43: set_dirty / clear_dirty /
# check_dirty_on_startup in dirty_bits.rs; VP-057-proven advance_crash_counter in
# recovery.rs) but have ZERO production callers — the same POL-15-class
# constructed-but-unreachable defect this story exists to close. Existing-anchor
# check: BC-2.15.005 stories S-2.01 (MERGED PR #43, primitives only) and S-6.05
# (migrate-storage CLI) — neither claims the wiring. AC-014 is the anchor.
verification_properties: [VP-058]
# VP-058 (watchdog memory grace period two-check policy, proptest + Kani-shaped
# pure fn `should_terminate_for_memory`) — anchor UNCHANGED (S-2.02 owns the proof
# and tests). This story is the PRODUCTION-WIRING vehicle: the proven function is
# currently dead code relative to the monitor loop (`spawn_monitor` cancels on a
# SINGLE Kill-level poll, violating DI-027). AC-004 wires it in.
depends_on:
  - S-DEMO-DTU-LIVE-SCENARIO-001-B
  - S-DEMO-MULTI-TENANT-DTU-001
  - S-DEMO-004
  # Dependency anchors — ORCHESTRATOR-DIRECTED SEQUENCING, not build-order:
  #   These three stories constitute the live-demo objective (T5 Story B → T6 →
  #   T8). The 2026-06-10 dispatch directs this wiring story into the post-demo
  #   backlog (NOT demo-blocking: per-query pool + query timeout already protect
  #   the demo). There is no compile-time dependency on any of the three; the
  #   edges exist so the wave scheduler cannot pull this story ahead of the demo.
  #   Same pattern as sibling S-CACHE-SPEC-COMPLIANCE-001.
blocks: []
points: 9
# Points justification:
#   1. Boot Arc-DI construction + spawn_monitor + QueryEngine/PrismServer threading
#      (ADR-022 §B sub-step, INFO limits log): 1.5 pts
#   2. Execution-path lifecycle: register/deregister + cancellation observation +
#      DI-027 grace-period wiring into the monitor loop: 2 pts
#   3. Denylist plumbing: fingerprint helper, storage value-format extension
#      (reason/denylisted_at), query-start gate + E-QUERY-008 construction,
#      failure-record (ratified trigger set) + success-reset: 2.5 pts
#   4. Crash-trigger wiring (added v1.1, BC-2.15.008 v1.4 third trigger): dirty-bit
#      set/clear at the execute entry point + boot crash-recovery sub-step
#      (check_dirty_on_startup + advance_crash_counter → reason `crash`) +
#      E-QUERY-008 Display alignment to taxonomy v1.70 (prism-core): 1 pt
#   5. MCP surface: force_execute param + watchdog_status unstub (status snapshot +
#      capability-gated denylist clear): 1 pt
#   6. Red Gate suite (~14 tests incl. restart-persistence integration test) +
#      config surface ([watchdog] denylist_threshold / denylist_expiry_secs): 1 pt
estimated_days: 3.5
risk: MEDIUM
# Risk justification: the cancellation-observation point in QueryEngine::execute
# (tokio::select! on the registered token vs the execution future) interacts with
# the existing 30s tokio::time::timeout wrapper and the partial-failure envelope
# (BC-2.11.011) — getting all-or-nothing termination (BC-2.15.007 invariant 3)
# right across the fan-out abort path is the risky part. The denylist gate itself
# is low-risk (pure pre-check). Mitigated by the StaticProbe/FixedClock test seams
# already shipped in S-2.02.
acceptance_criteria_count: 14
red_gate_tests: 14
estimated_passes: "2-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Reuse the S-2.02 test seams: StaticProbe (fixed RSS) + FixedClock (fixed unix
    time) — no real-RSS or 24h-sleep tests. The two-consecutive-check grace period
    is tested by stepping the monitor logic with injected probe values (SID-1
    in-process unit seams), not by waiting on real poll intervals."
  - "The cancellation-observation select! must preserve the existing E-QUERY-004
    timeout path BYTE-IDENTICALLY (BC-2.11.006 30s wrapper stays; the watchdog token
    is an ADDITIONAL cancellation source, not a replacement). Regression-run the
    existing BC-2.11.006 timeout suite unchanged."
  - "Ratified trigger set (BC-2.15.008 v1.4; BC-2.15.007 v1.6; DI-027 invariants
    v1.7; error-taxonomy v2.26; architect adjudication D2,
    proposals/cache-envelope-adjudication-2026-06-10.md): denylist-eligible failures
    are EXACTLY {timeout = query timeout E-QUERY-004, rss_kill = process-RSS kill
    E-WATCHDOG-002, crash = CrashRecovery-marker-detected process crash per
    BC-2.15.005}, with reason ∈ {timeout, rss_kill, crash}. Per-query
    GreedyMemoryPool trips (E-WATCHDOG-001, PrismError::QueryMemoryBudgetExceeded)
    AND record-cap violations (E-QUERY-005) MUST NOT increment the counter — and,
    as non-terminated executions for counter purposes, they RESET any prior
    watchdog-termination/crash streak to 0 (EC-15-043, BOTH halves). Tests must
    prove both halves: counter unchanged AND streak reset."
  - "Fingerprint normalization reuses the PrismQL parser's canonical statement
    rendering (EC-07-040 precedent: syntax-different-but-canonically-identical
    queries share the hash). Do NOT hash the raw query string — whitespace variants
    of the same runaway query would evade the denylist."
  - "DenylistEntry is a pub type in prism-storage: adding reason/denylisted_at fields
    requires #[non_exhaustive] discipline (compile-fail gate EXPECTED bump in ci.yml
    if newly annotated). Check before extending."
  - "prism-mcp must NOT gain a prism-storage dependency: watchdog status reaches
    PrismServer via an Arc-DI seam wired at boot step 9 (mirror the existing
    Option<Arc<QueryEngine>> pattern), keeping the MCP layer storage-agnostic."
inputs:
  - .factory/specs/behavioral-contracts/BC-2.15.005-crash-recovery-dirty-bits.md
  - .factory/specs/behavioral-contracts/BC-2.15.007-watchdog-query-termination.md
  - .factory/specs/behavioral-contracts/BC-2.15.008-query-denylisting.md
  - .factory/specs/verification-properties/vp-058-watchdog-memory-grace-period-two-check-policy.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/proposals/cache-envelope-adjudication-2026-06-10.md
  - crates/prism-storage/src/watchdog.rs
  - crates/prism-storage/src/denylist.rs
  - crates/prism-storage/src/dirty_bits.rs
  - crates/prism-storage/src/recovery.rs
  - crates/prism-storage/src/proofs/watchdog_memory.rs
  - crates/prism-query/src/engine.rs
  - crates/prism-bin/src/boot.rs
  - crates/prism-mcp/src/server.rs
traces_to: ["QRY fix-burst STOP report (fix/review-2026-06-10-query-core cycle, 2026-06-10): ResourceWatchdog zero production wiring — POL-15-class active-BC-unreachable discovery"]
supersedes: []
---

# S-WATCHDOG-WIRING-001: ResourceWatchdog Production Wiring — Boot Construction, Query-Start Denylist Gate (E-QUERY-008), force_execute, watchdog_status Unstub

Wire the S-2.02 ResourceWatchdog + denylist library (merged PR #52, 2026-04-25) into
the production binary. The QRY fix-burst STOP report (2026-06-10) discovered the
subsystem has ZERO production wiring: `ResourceWatchdog` is never constructed in the
boot sequence, no query-start denylist gate exists, `PrismError::QueryDenylisted`
(E-QUERY-008) is unconstructible from any production code path, and the
`watchdog_status` MCP tool returns `-32003 not yet implemented`. Active P0 contracts
BC-2.15.007 (process-RSS kill path) and BC-2.15.008 (denylisting) are unreachable —
the same constructed-but-unreachable defect class as QRY-02 (cache wired but never
on the hot path).

**Sequencing (orchestrator-directed, 2026-06-10):** NOT demo-blocking — the per-query
GreedyMemoryPool (BC-2.11.006 → E-WATCHDOG-001) and the 30s query timeout
(E-QUERY-004) are already wired and protect the demo. Post-demo backlog alongside
S-CACHE-SPEC-COMPLIANCE-001.

---

## Narrative

As a Prism platform operator, I want the resource watchdog constructed at boot,
monitoring every executing query, killing the process-RSS-violating workload with a
structured retryable error, and denylisting repeat-offender queries with a manual
override — so that the production binary actually has the crash-loop and
memory-exhaustion safety net that BC-2.15.007/008 promise, instead of a fully tested
library that no production code path ever calls.

---

## Behavioral Contracts

| BC | Title | Key Clauses Implemented |
|----|-------|------------------------|
| BC-2.15.007 | Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error | Precondition 2 (watchdog active and monitoring) made satisfiable in production; postcondition 1 (monitors each running query — token register/deregister lifecycle); postcondition 2 memory bullet (process-RSS kill → `E-WATCHDOG-002`, retryable); DI-027 memory grace period (two consecutive checks) wired into the production monitor loop; "terminated query is recorded in watchdog state for denylist evaluation"; "audit entry emitted with violation type and query details"; invariants 1–3 (non-negotiable termination, clean termination, all-or-nothing) |
| BC-2.15.008 | Query Denylisting — After N Consecutive Failures, Denylist with Manual Override | Postcondition 1 (SHA-256 normalized-query fingerprint recorded per denylist-eligible failure); postcondition 2 (v1.4 trigger set — 3 consecutive watchdog terminations {`timeout` E-QUERY-004, `rss_kill` E-WATCHDOG-002} or process crashes {`crash`, CrashRecovery markers per BC-2.15.005}; threshold denylisting with `denylisted_at`/`reason`/`failure_count`/`expiry`; immediate rejection with `E-QUERY-008`); counter-exclusions postcondition (E-WATCHDOG-001 pool trips + E-QUERY-005 record-cap never feed the counter — DI-027); non-terminated-execution reset postcondition (success OR excluded failure resets the streak to 0 — EC-15-043); auto-expiry; `force_execute: true` manual override; RocksDB persistence across restarts; invariants 1–4 |
| BC-2.15.005 | Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart | Postcondition 1 (dirty bit set with `sync: true` before execution; E-STORE-009 fail-closed on write failure); postcondition 2 (v1.4 orderly-exit semantics — cleared on EVERY orderly exit, successful completion OR structured-error termination; only process death leaves a surviving marker); postconditions 3–4 (startup scan after schema-version check; each detected crash recorded with reason `crash` into the BC-2.15.008 counter; WARN per source; marker cleared after processing; E-STORE-010 retry-next-startup); invariants 1–3. VP-057 anchor UNCHANGED (S-2.01 owns the `advance_crash_counter` proof; this story is the production-wiring vehicle — same pattern as VP-058). |

### Preserved Contracts (regression surface — NOT in `behavioral_contracts`; no new ACs)

| BC | Surface this story touches | Preservation requirement |
|----|---------------------------|--------------------------|
| BC-2.11.006 | 30s `tokio::time::timeout` wrapper + GreedyMemoryPool in `engine.rs` | Timeout path (E-QUERY-004) and per-query pool trip (E-WATCHDOG-001) unchanged; the watchdog token is an ADDITIONAL cancellation source. Existing suite passes unchanged. |
| BC-2.11.011 | Partial-failure envelope on the fan-out path | Watchdog-cancelled queries return the structured error, never a partial envelope (BC-2.15.007 invariant 3) |
| BC-2.10.012 | `#[serde(deny_unknown_fields)]` + injection-scan discipline on tool params | `force_execute` added as a typed optional bool — no relaxation of `deny_unknown_fields`, no new string surface to scan |
| BC-2.16.002 | Structured Event Catalog | Every new `event_type` emission (watchdog kill, denylist add/hit/clear, limits-at-startup INFO) gets a same-commit catalog row (SAP-1) |

---

## Current State (verified against develop, 2026-06-10)

- `prism-storage/src/watchdog.rs`: `ResourceWatchdog` complete (graduated levels,
  `register_query`/`deregister_query`/`check_query`/`spawn_monitor`,
  `get_watchdog_status`, SI 512 MB budget, `MemoryProbe` seam) — referenced ONLY by
  prism-storage tests/proofs. Never constructed by prism-bin: `boot.rs` has zero
  watchdog mentions.
- `prism-storage/src/denylist.rs`: `record_failure` / `is_denylisted` /
  `clear_denylist` complete with `ClockProbe` seam and lazy expiry. NO production
  caller. Storage value format `{failure_count}:{last_failure_ts}:{expiry_ts}`
  carries NO `reason` and NO `denylisted_at` — BC-2.15.008 postcondition 2 fields
  missing, so a real `E-QUERY-008` error cannot be populated.
- No SHA-256 normalized-query fingerprint helper exists anywhere; `record_failure`
  takes `fingerprint: &str` that nothing computes.
- `prism-core/src/error.rs`: `PrismError::QueryDenylisted` (E-QUERY-008) exists with
  `failure_count`/`reason`/`expiry_ts` fields — constructed by ZERO production code
  paths (only the prism-mcp `error_mapping.rs` match arm references it). Its Display
  is the pre-ratification shape ("E-QUERY-008: query denylisted after {failure_count}
  consecutive failures (reason: {reason}); denylist expires at {expiry_ts}; use
  force_execute: true to override") — NOT the taxonomy v1.70 verbatim Message Format
  — and the `reason` field doc comment carries the RETIRED vocabulary
  "(timeout / memory / record_limit)" (QRY cascade P3-03).
- `prism-storage/src/dirty_bits.rs` + `prism-storage/src/recovery.rs`: the
  BC-2.15.005 crash-recovery primitives (`set_dirty` / `clear_dirty` /
  `check_dirty_on_startup`; VP-057-proven `advance_crash_counter` returning
  `RecoveryAction::{Warn, Denylist}`) are complete with Kani proof — and have ZERO
  production callers. No dirty bit is ever set before a query executes, and boot
  never scans the `dirty_bits` CF, so the BC-2.15.008 v1.4 `crash` trigger is wholly
  unreachable (same POL-15 class).
- `prism-query/src/engine.rs`: `QueryEngine::execute` enforces the 30s timeout and
  the GreedyMemoryPool, but never registers a `CancellationToken` with any watchdog,
  never checks a denylist, never records failures/successes. `QueryOptions` has no
  `force_execute`.
- `prism-mcp/src/server.rs`: `QueryToolParams { query, clients }` — no
  `force_execute`. `watchdog_status` tool emits its audit event then returns
  `not_yet_available_msg("watchdog")` (-32003); its doc string promises a
  capability-gated denylist clear that does not exist (Standing Rule 3 §3 surface).
- `prism-storage/src/proofs/watchdog_memory.rs`: VP-058
  `should_terminate_for_memory` (two-consecutive-check DI-027 policy) is proven —
  and NOT called by `spawn_monitor`, which cancels ALL tokens on a SINGLE Kill-level
  poll. The proven grace period is dead code relative to the monitor loop.
- Stale doc comment: `watchdog.rs::check_query` cites "E-WATCHDOG-001, BC-2.15.007"
  — superseded by taxonomy v1.68 D2 (`WatchdogKilled` = E-WATCHDOG-002; error.rs
  display already corrected). In-scope doc fix (TD-VSDD-060 sibling sweep).

### Interim protections in effect until this story ships

1. **Per-query memory budget** — DataFusion GreedyMemoryPool (BC-2.11.006 →
   E-WATCHDOG-001) is wired and kills per-query 200MB violations.
2. **Query timeout** — 30s `tokio::time::timeout` in `engine.rs` (E-QUERY-004).
3. **NOT protected:** process-level RSS growth across concurrent queries
   (E-WATCHDOG-002 path), crash-loop repeat-offender queries (denylist), and
   operator observability (`watchdog_status`).

---

## Acceptance Criteria

### Group A — Production activation + termination path (BC-2.15.007)

**AC-001 — Boot constructs, monitor-spawns, and threads the watchdog (Arc-DI)**
(traces to BC-2.15.007 precondition 2 "The resource watchdog is active and
monitoring (BC-2.15.006)" — this story makes the precondition satisfiable in
production)

Given the 11-step boot sequence (ADR-022 §B), when boot reaches the post-step-6
state (RocksDB open — the `watchdog` CF and denylist need the backend), then an
`Arc<ResourceWatchdog>` is constructed (production `SysinfoProbe`), its
`spawn_monitor` background task is started with the 3-second check interval
(BC-2.15.006 invariant: interval fixed at 3s; the stale 500ms code-comment claim is
corrected), the configured limits are logged once at INFO, and the Arc is threaded
into `QueryEngine` (boot step 9) and into `PrismServer` (for AC-013) via constructor
Arc-DI per ADR-022 — no placeholder-construct, no lazy global.

Red Gate: `test_BC_2_15_007_boot_constructs_and_spawns_watchdog`

**AC-002 — Every query execution registers and deregisters its token**
(traces to BC-2.15.007 postcondition 1 "The watchdog monitors each running query")

Given any call into `QueryEngine::execute` (the single entry point used by the MCP
`query` tool and any future scheduled-execution caller), when execution begins, then
a fresh `CancellationToken` is registered with the watchdog (`register_query`) and
is deregistered (`deregister_query`) on EVERY exit path — success, structured error,
timeout, and panic-unwind (drop-guard) — so the token registry cannot leak entries.

Red Gate: `test_BC_2_15_007_execute_registers_and_deregisters_token_all_paths`

**AC-003 — Process-RSS kill is observed end-to-end with E-WATCHDOG-002**
(traces to BC-2.15.007 postcondition 2 memory bullet + postcondition "structured
error … E-WATCHDOG-002" + invariants 1–3)

Given a registered executing query and an injected `StaticProbe` reporting RSS at or
above the kill threshold (95% of the 512 MB SI budget) on two consecutive monitor
checks, when the monitor cancels the registered tokens, then the executing query
observes the cancellation (select-on-token in the execution path), aborts in-flight
sensor API calls, drops materialized data, and returns
`PrismError::WatchdogKilled { budget_bytes }` (E-WATCHDOG-002, retryable) to the
caller — never a partial result set.

Red Gate: `test_BC_2_15_007_rss_kill_cancels_query_returns_e_watchdog_002`

**AC-004 — DI-027 grace period wired into the production monitor loop (VP-058)**
(traces to BC-2.15.007 postcondition "Memory grace period (DI-027): A single spike
above the process-RSS kill threshold does not immediately terminate the query")

Given the production `spawn_monitor` loop, when a SINGLE poll observes Kill-level
RSS, then NO tokens are cancelled; when TWO CONSECUTIVE polls observe Kill-level
RSS, then all registered tokens are cancelled — the loop consults the proven
VP-058 function `should_terminate_for_memory` (consecutive_over_limit ≥ 2) instead
of the current single-check cancel, and a sub-Kill poll resets the consecutive
counter.

Red Gate: `test_BC_2_15_007_monitor_grace_period_two_consecutive_checks`

**AC-005 — Denylist-eligible terminations recorded + audit entry emitted**
(traces to BC-2.15.007 v1.6 postcondition "Denylist eligibility (DI-027 scope): only
timeout (`E-QUERY-004`) and process-RSS kill (`E-WATCHDOG-002`) terminations are
recorded in watchdog state for denylist evaluation (BC-2.15.008)" and "An audit
entry is emitted with the violation type and query details")

Given a query terminated by the watchdog — timeout (E-QUERY-004) or process-RSS kill
(E-WATCHDOG-002) — when the structured error propagates, then
`denylist::record_failure` is invoked with the query's fingerprint and the violation
`reason` ∈ {`timeout`, `rss_kill`} (the third member of the BC-2.15.008 v1.4 reason
vocabulary, `crash`, is recorded by the restart path — AC-014), AND an audit entry
carrying the violation type and query details is written via the established audit
path. A record-cap termination (E-QUERY-005) is explicitly NOT recorded — it is NOT
denylist-eligible and, for counter purposes, is a non-terminated execution that
resets any prior streak (EC-15-043; both halves asserted by AC-007's Red Gate test).
All new `tracing::*!(event_type=…)` sites gain same-commit BC-2.16.002 catalog rows
(SAP-1).

Red Gate: `test_BC_2_15_007_termination_records_failure_and_audit_entry`

### Group B — Denylist activation (BC-2.15.008)

**AC-006 — SHA-256 normalized-query fingerprint helper**
(traces to BC-2.15.008 postcondition 1 "recorded … with the query hash (SHA-256 of
the normalized query string)")

Given two PrismQL query strings that parse to the same canonical statement (e.g.,
whitespace/case-insignificant syntax variants — EC-07-040 precedent), when the
fingerprint helper (prism-query, reusing the parser's canonical rendering + `sha2`)
computes their fingerprints, then both produce the same SHA-256 hex string; and
given two semantically different statements, the fingerprints differ. The helper is
the SINGLE binding used by the gate (AC-008), failure recording (AC-005/AC-007),
and success-reset (AC-009) — one fingerprint definition, no parallel hashers.

Red Gate: `test_BC_2_15_008_fingerprint_sha256_of_normalized_query`

**AC-007 — Ratified v1.4 trigger set: terminations and crashes count; excluded
failures do NOT increment AND reset the streak**
(traces to BC-2.15.008 v1.4 postcondition 2 "When a query hash accumulates
**3 consecutive** watchdog terminations (query timeout `E-QUERY-004` or process-RSS
kill `E-WATCHDOG-002`, per BC-2.15.007) or process crashes (detected via
CrashRecovery markers on restart, BC-2.15.005) — threshold configurable via
`watchdog.denylist_threshold`, default 3" + counter-exclusions postcondition
"per-query GreedyMemoryPool budget trips (`E-WATCHDOG-001`, BC-2.11.006) and
record-cap violations (`E-QUERY-005`, DI-019 10K materialization limit) do NOT feed
the denylist counter" + invariants 1–2; EC-15-043)

Given consecutive denylist-eligible failures of the same fingerprint — the trigger
set is exactly {`timeout` = query timeout E-QUERY-004, `rss_kill` = process-RSS kill
E-WATCHDOG-002, `crash` = CrashRecovery-detected process crash (BC-2.15.005,
AC-014)} — when the count reaches the configured threshold, then the fingerprint is
denylisted. And given a per-query GreedyMemoryPool trip (E-WATCHDOG-001,
`PrismError::QueryMemoryBudgetExceeded`, BC-2.11.006) OR a record-cap violation
(E-QUERY-005, DI-019 10K limit), when that error propagates, then BOTH halves of
EC-15-043 hold: the denylist counter for the fingerprint is NOT incremented, AND —
as a non-terminated execution for counter purposes — any prior
watchdog-termination/crash streak is RESET to 0. The Red Gate test exercises the
BC's canonical "Excluded reason — pool trip" vector: timeout, pool trip, timeout →
the pool trip resets the streak; the third event counts as failure 1 of 3; no
denylisting.

Red Gate: `test_BC_2_15_008_excluded_failures_do_not_increment_and_reset_streak`

**AC-008 — Query-start denylist gate rejects with a fully-populated E-QUERY-008**
(traces to BC-2.15.008 postcondition 2 "Future attempts to execute a query matching
this hash are immediately rejected with `E-QUERY-008`" + postcondition 2 entry
fields "`denylisted_at`, `reason` …, `failure_count`, `expiry`")

Given a denylisted, unexpired fingerprint, when an identical-canonical-form query is
submitted (without `force_execute`), then `QueryEngine::execute` rejects it BEFORE
any cache read or sensor fan-out with `PrismError::QueryDenylisted { failure_count,
reason, expiry_ts }` populated from the stored entry — which requires extending the
`watchdog` CF value encoding and `DenylistEntry` to carry `reason` and
`denylisted_at` (currently absent), and replacing the bare-bool lookup with a
status-returning lookup. Sensor APIs are NOT contacted (asserted via mock at the
dependency boundary, SID-1). The rejection's Display output is the error-taxonomy
v1.70 verbatim E-QUERY-008 Message Format — "Query has been denylisted after {N}
consecutive failures ({reason}). Denylist expires at {expiry}." — with the
`force_execute` hint living in recovery guidance (error_mapping / taxonomy Notes),
NOT in the Display string (see the Phase 2 error.rs task; P3-03).

Red Gate: `test_BC_2_15_008_gate_rejects_denylisted_query_e_query_008`

**AC-009 — Success resets the consecutive failure counter**
(traces to BC-2.15.008 postcondition "A non-terminated execution of the same query
hash resets the consecutive failure counter to 0" + invariant 1; EC-15-028)

Given a fingerprint with a below-threshold failure count (e.g., 2 of 3), when an
execution of the same canonical query completes without watchdog termination, then
the counter is reset to 0 (new `record_success` operation in
`prism-storage::denylist`), and a subsequent termination starts counting from 1 —
no denylisting on the third non-consecutive failure.

Red Gate: `test_BC_2_15_008_success_resets_consecutive_counter`

**AC-010 — `force_execute: true` bypasses the gate for a single execution**
(traces to BC-2.15.008 postcondition "Manual override: the `query` tool accepts a
`force_execute: true` parameter that bypasses the denylist for a single execution
(still subject to watchdog limits)" + invariant 2; EC-15-029)

Given a denylisted fingerprint, when the MCP `query` tool is invoked with
`force_execute: true` (new optional bool on `QueryToolParams`, serde default
`false`, `deny_unknown_fields` retained, plumbed through `QueryOptions`), then the
gate is bypassed for that execution ONLY, the query is still registered with and
monitored by the watchdog (AC-002/AC-003 apply), and — if the forced execution
completes without termination — the denylist entry for the fingerprint is cleared
(EC-15-029: success clears denylist for that hash).

Red Gate: `test_BC_2_15_008_force_execute_bypasses_gate_success_clears_entry`

**AC-011 — Auto-expiry and restart persistence on the production path**
(traces to BC-2.15.008 postconditions "Denylist entries automatically expire after
the configured duration" and "Denylist state is persisted in RocksDB and survives
restarts" + invariant 3)

Given a denylisted fingerprint with expiry T (injected `FixedClock`), when an
identical query is submitted at time > T through the production gate, then the
entry is lazily removed and the query executes normally; and given a denylisted
fingerprint, when the process restarts (subprocess integration test reopening the
same RocksDB path), then the gate still rejects with E-QUERY-008 — persistence
survives restart (this is the restart integration test BC-2.15.008's VP table
delegates to the watchdog test suite).

Red Gate: `test_BC_2_15_008_expiry_lazy_removal_and_restart_persistence`

**AC-012 — Config surface: threshold and expiry read from prism.toml at boot**
(traces to BC-2.15.008 postcondition 2 "configurable via
`watchdog.denylist_threshold`, default 3" and "expiry (configurable, default 86400
seconds / 24 hours)")

Given `[watchdog] denylist_threshold = N` and `[watchdog] denylist_expiry_secs = E`
in prism.toml, when boot wires the watchdog/denylist, then the gate and
`record_failure` use N and E (constants `DENYLIST_THRESHOLD = 3` /
`DENYLIST_EXPIRY_SECS = 86400` become the documented defaults; `record_failure`
gains an expiry parameter); `denylist_threshold = 1` denylists on the first failure
(EC-15-032); absent keys default to 3 / 86400.

Red Gate: `test_BC_2_15_008_denylist_threshold_and_expiry_configurable`

**AC-014 — Crash trigger: dirty-bit lifecycle wired; CrashRecovery markers on
restart feed the denylist with reason `crash`**
(traces to BC-2.15.008 v1.4 precondition 1 crash clause + postcondition 1 +
postcondition 2 third trigger {`crash` = CrashRecovery-detected process crash};
also traces to BC-2.15.005 postconditions 1–4 + invariants 1–3. AC ID appended per append-only AC
numbering — Group C's AC-013 pre-dates this AC, added in story v1.1 when the v1.4
trigger set's third member was found absent, QRY cascade P3-01.)

Given the production execution path, when `QueryEngine::execute` begins, then a
dirty bit is set in the `dirty_bits` CF with `sync: true` (key = the query's
canonical fingerprint per AC-006; value = `DirtyBitEntry`; a failed dirty-bit write
aborts the query fail-closed with E-STORE-009), and the bit is cleared on every
ORDERLY exit — success or structured error — so only a process crash leaves a
marker. (Per BC-2.15.005 v1.4 postcondition 2 — ratified by the product-owner
micro-burst 2026-06-10, resolving the wording-sharpen flag raised in story v1.1:
the dirty bit is cleared on EVERY orderly exit, successful completion OR
structured-error termination, so only genuine process death leaves a surviving
marker. Rationale recorded in the BC: markers persisting on structured-error exits
would double-count watchdog terminations as crashes on the next restart, breaking
BC-2.15.008 v1.4's single-streak semantics.) Given uncleared markers on startup, when
the boot crash-recovery sub-step runs (after the schema-version check per
BC-2.15.005; wiring the existing `check_dirty_on_startup` +
`advance_crash_counter` primitives, which currently have zero production callers),
then each detected crash is recorded against the fingerprint's consecutive failure
counter with reason `crash` — the SAME counter and store the gate (AC-008) reads,
because crashes and watchdog terminations share one consecutive streak per
BC-2.15.008 v1.4 postcondition 2 — the marker is cleared after processing
(E-STORE-010 on recovery failure: WARN, marker NOT cleared, retried next startup),
Scheduled-source markers log the BC-2.15.005 WARN ("schedule interrupted, will
retry on next tick"), and at threshold the hash is denylisted so the next
submission is rejected with E-QUERY-008 carrying `reason = "crash"`.

Red Gate: `test_BC_2_15_008_crash_marker_lifecycle_and_restart_feeds_denylist`

### Group C — Operator surface (BC-2.15.008 observability + override)

**AC-013 — `watchdog_status` tool unstubbed**
(traces to BC-2.15.008 postcondition "Denylist state is persisted in RocksDB" —
operator-visible denylist state — and invariant 2's manual-override surface; the
E-QUERY-008 taxonomy row's remediation path is "clear the denylist via
watchdog_status". Anchor note: the tool's existence clause lives in BC-2.15.006
postconditions, which is out of frontmatter scope — see §Out of Scope; the clauses
implemented HERE are the BC-2.15.008-owned denylist-state exposure and clear path.)

Given a booted server, when the `watchdog_status` MCP tool is invoked, then it
returns the live `get_watchdog_status` snapshot (graduated level, `budget_bytes`,
`current_bytes`, denylist entries with fingerprint/failure_count/reason/expiry)
wrapped in the safety envelope — the `-32003 not yet implemented` stub and the
"not yet wired" doc lines are removed; and when invoked with the optional
capability-gated `clear_denylist` parameter (the capability gate the existing tool
doc string already promises — Standing Rule 3 §3: implement or remove; we
implement), then `denylist::clear_denylist` runs for the given fingerprint (or all),
gated on the established capability mechanism and audit-logged. PrismServer reaches
the data via an Arc-DI seam wired at boot step 9 (no prism-storage dependency added
to prism-mcp).

Red Gate: `test_watchdog_status_returns_live_snapshot_and_gated_clear`

---

## Red Gate Test Plan

All tests written FAIL-first. External boundaries mocked in-process per SID-1
(StaticProbe for RSS, FixedClock for time, InMemoryBackend for storage, mock sensor
boundary for not-contacted assertions). One subprocess integration test (restart
persistence) is justified per the in-process-vs-subprocess discipline.

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_15_007_boot_constructs_and_spawns_watchdog` | prism-bin | BC-2.15.007 PC-2 (active+monitoring) | integration |
| 2 | `test_BC_2_15_007_execute_registers_and_deregisters_token_all_paths` | prism-query | BC-2.15.007 PC-1 | unit |
| 3 | `test_BC_2_15_007_rss_kill_cancels_query_returns_e_watchdog_002` | prism-query | BC-2.15.007 PC-2 memory + INV 1–3 | unit |
| 4 | `test_BC_2_15_007_monitor_grace_period_two_consecutive_checks` | prism-storage | BC-2.15.007 DI-027 PC (VP-058 wiring) | unit |
| 5 | `test_BC_2_15_007_termination_records_failure_and_audit_entry` | prism-query | BC-2.15.007 v1.6 denylist-eligibility + audit PCs | unit |
| 6 | `test_BC_2_15_008_fingerprint_sha256_of_normalized_query` | prism-query | BC-2.15.008 PC-1 | unit |
| 7 | `test_BC_2_15_008_excluded_failures_do_not_increment_and_reset_streak` | prism-query | BC-2.15.008 PC-2 v1.4 trigger set + counter-exclusions PC + reset PC (EC-15-043 both halves; canonical pool-trip vector) | unit |
| 8 | `test_BC_2_15_008_gate_rejects_denylisted_query_e_query_008` | prism-query | BC-2.15.008 PC-2 rejection + entry fields | unit |
| 9 | `test_BC_2_15_008_success_resets_consecutive_counter` | prism-storage | BC-2.15.008 success-reset PC + INV-1 | unit |
| 10 | `test_BC_2_15_008_force_execute_bypasses_gate_success_clears_entry` | prism-mcp | BC-2.15.008 manual-override PC + INV-2, EC-15-029 | unit |
| 11 | `test_BC_2_15_008_expiry_lazy_removal_and_restart_persistence` | prism-bin | BC-2.15.008 expiry + persistence PCs + INV-3 | integration (subprocess) |
| 12 | `test_BC_2_15_008_denylist_threshold_and_expiry_configurable` | prism-bin | BC-2.15.008 PC-2 config + EC-15-032 | unit |
| 13 | `test_watchdog_status_returns_live_snapshot_and_gated_clear` | prism-mcp | BC-2.15.008 persisted-state exposure + clear path | unit |
| 14 | `test_BC_2_15_008_crash_marker_lifecycle_and_restart_feeds_denylist` | prism-bin | BC-2.15.008 PC-2 crash trigger + BC-2.15.005 PCs 1–4 (set/clear lifecycle; seeded markers → recovery → reason `crash`; E-STORE-009/010 paths) | integration |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~9 000 |
| BC-2.15.007 v1.6 (full) | ~2 600 |
| BC-2.15.008 v1.4 (full) | ~2 100 |
| BC-2.15.005 v1.4 (full) | ~1 300 |
| VP-058 file + proofs/watchdog_memory.rs | ~1 800 |
| proposals/cache-envelope-adjudication-2026-06-10.md (D2 trigger-set context) | ~3 500 |
| prism-storage/src/watchdog.rs (full) | ~4 500 |
| prism-storage/src/denylist.rs (full) | ~3 000 |
| prism-storage/src/dirty_bits.rs + recovery.rs (full) | ~1 500 |
| prism-bin/src/boot.rs (step-6→step-9 regions only — NOT full file) | ~6 000 |
| prism-query/src/engine.rs (execute + QueryOptions + construction regions only) | ~5 500 |
| prism-mcp/src/server.rs (QueryToolParams, query tool, watchdog_status regions only) | ~4 500 |
| prism-core/src/error.rs (E-QUERY/E-WATCHDOG region) | ~1 000 |
| error-taxonomy.md (E-QUERY-008 / E-WATCHDOG rows) + ADR-022 §B table | ~2 000 |
| Test stubs (14 × ~50 lines) | ~3 500 |
| Tool outputs (nextest, clippy, proptest VP-058) | ~3 000 |
| **Total estimate** | **~55 000** |

At ~256k context window this is ~22% — within the 20-30% ceiling, contingent on the
partial-read discipline for boot.rs / engine.rs / server.rs (load only the listed
regions; server.rs is 4 000+ lines).

---

## Tasks

**Phase 0: Re-baseline**

- [ ] Run remove-uncertainty against the post-demo develop baseline: confirm
      §Current State still holds (boot.rs step shape, engine.rs execute wrapper,
      QueryToolParams fields, watchdog_status stub) — the demo stories and fix
      branches all touch these surfaces
- [ ] Story-writer collapses any drift and flips status draft → ready

**Phase 1: Storage-layer extensions (prism-storage)**

- [ ] Read `denylist.rs` + `watchdog.rs` + `dirty_bits.rs` + `recovery.rs` fully
      before editing
- [ ] Extend the `watchdog` CF value encoding + `DenylistEntry` with `reason`
      (vocabulary EXACTLY {timeout, rss_kill, crash} per BC-2.15.008 v1.4
      postcondition 2) and `denylisted_at` (BC-2.15.008 PC-2 fields);
      check/add `#[non_exhaustive]` on the pub struct (ci.yml EXPECTED bump if
      newly annotated); replace bare-bool `is_denylisted` callers with a
      status-returning lookup carrying the stored fields (AC-008)
- [ ] Add `record_success` (reset counter to 0; clear entry when called on a
      forced-execution success per EC-15-029) (AC-009/AC-010)
- [ ] Parameterize `record_failure` expiry (default `DENYLIST_EXPIRY_SECS`) (AC-012)
- [ ] Wire `should_terminate_for_memory` into `spawn_monitor` (two-consecutive-check
      DI-027 policy; sub-Kill poll resets the counter) (AC-004); fix the stale
      `check_query` "E-WATCHDOG-001" doc comment → E-WATCHDOG-002 (TD-VSDD-060
      sibling-sweep all sites)
- [ ] Write Red Gate tests 4, 9 (FAIL first)

**Phase 2: Execution-path integration (prism-query)**

- [ ] Read engine.rs execute/QueryOptions/construction regions before editing
- [ ] Implement the fingerprint helper: SHA-256 hex over the parser's canonical
      statement rendering; single binding shared by gate/record/reset (AC-006)
- [ ] Thread `Arc<ResourceWatchdog>` + storage backend into `QueryEngine`
      construction (Arc-DI; "wiring not redesign" — adding the parameters where
      they were missing is in-scope per Standing Rule 3 §4)
- [ ] Query-start gate: fingerprint → status lookup → `PrismError::QueryDenylisted`
      with stored fields, BEFORE cache read and fan-out; honor
      `QueryOptions.force_execute` bypass (AC-008/AC-010)
- [ ] Token lifecycle: register at execute start, drop-guard deregister on all
      exits; select-on-token cancellation observation alongside the existing 30s
      timeout wrapper (preserve E-QUERY-004 path byte-identically) (AC-002/AC-003)
- [ ] Failure-record hooks for the watchdog-termination triggers {E-QUERY-004 →
      reason `timeout`, E-WATCHDOG-002 → reason `rss_kill`}; explicitly NOT for
      E-WATCHDOG-001 pool trips NOR E-QUERY-005 record-cap violations — BOTH
      excluded failures additionally RESET the streak as non-terminated executions
      (EC-15-043, both halves); success-reset hook on non-terminated completion;
      audit entry with violation type + query details (AC-005/AC-007)
- [ ] Dirty-bit lifecycle at the execute entry point: `set_dirty` with `sync: true`
      (key = canonical fingerprint) before execution, fail-closed E-STORE-009 on
      write failure; clear on every orderly exit (success or structured error)
      (AC-014, BC-2.15.005 PCs 1–2 + invariants 1–2)
- [ ] Align `PrismError::QueryDenylisted` in `prism-core/src/error.rs` to taxonomy
      v1.70 (P3-03): Display becomes the verbatim E-QUERY-008 Message Format
      "Query has been denylisted after {N} consecutive failures ({reason}).
      Denylist expires at {expiry}." — the shipped "use force_execute: true to
      override" suffix moves OUT of Display into recovery guidance
      (error_mapping.rs / taxonomy Notes); fix the `reason` field doc comment —
      retire "(timeout / memory / record_limit)" → "(timeout / rss_kill / crash)";
      TD-VSDD-060 sibling-sweep error_mapping.rs + any Display-asserting tests
      (AC-008)
- [ ] Write Red Gate tests 2, 3, 5, 6, 7, 8 (FAIL first)

**Phase 3: Boot + MCP surface (prism-bin, prism-mcp)**

- [ ] Read boot.rs step-6→step-9 regions before editing
- [ ] Boot sub-step: construct `Arc<ResourceWatchdog>`, `spawn_monitor` at 3s
      (BC-2.15.006 invariant — correct the 500ms code-comment claim), INFO limits
      log, read `[watchdog] denylist_threshold` / `denylist_expiry_secs` from the
      step-4 config snapshot (defaults 3 / 86400), thread into QueryEngine (step 9)
      and PrismServer (AC-001/AC-012)
- [ ] Boot crash-recovery sub-step (after the schema-version check, before the
      server accepts queries): wire `check_dirty_on_startup` +
      `advance_crash_counter` (currently zero production callers); record each
      detected crash into the SHARED consecutive failure counter with reason
      `crash`; clear processed markers; E-STORE-010 on recovery failure (WARN,
      marker retained, retry next startup); BC-2.15.005 WARN per marker source
      (AC-014)
- [ ] `QueryToolParams.force_execute: Option<bool>`/`#[serde(default)]` → plumb to
      `QueryOptions` (deny_unknown_fields retained; no new injection-scan surface —
      typed bool) (AC-010)
- [ ] Unstub `watchdog_status`: live snapshot via Arc-DI seam (no prism-storage dep
      in prism-mcp); capability-gated `clear_denylist` param per the existing doc
      promise; remove "not yet wired" doc lines; audit-log the clear (AC-013)
- [ ] Write Red Gate tests 1, 10, 11, 12, 13, 14 (FAIL first)

**Phase 4: Closure**

- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — every new emission
      (kill, denylist add/hit/clear, startup limits) has a same-commit BC-2.16.002
      catalog row
- [ ] TD-VSDD-060 sibling sweep: `is_denylisted` callsites, `record_failure`
      signature change, `DenylistEntry` field additions, E-WATCHDOG-001/002 doc
      comments across prism-storage/prism-query/prism-bin
- [ ] BC backlink update: BC-2.15.005 + BC-2.15.007 + BC-2.15.008 Traceability
      "Stories" rows gain S-WATCHDOG-WIRING-001
- [ ] `just check` GREEN; `just iter prism-storage` + `just iter prism-query` during
      inner loop; non-exhaustive compile-fail gate EXPECTED count verified

---

## Previous Story Intelligence

- **S-2.02 (the library this story activates):** introduced the `MemoryProbe` /
  `ClockProbe` test seams specifically so RSS and 24h-expiry behavior are testable
  in-process — REUSE them; do not invent new seams. The 512 MB budget is SI
  (512,000,000 bytes), NOT MiB (ADR-S2.02-002 in PR #52) — threshold tests must use
  the SI base. The Kill action cancels ALL registered tokens (process RSS is
  process-wide; per-query attribution belongs to the GreedyMemoryPool layer) — do
  not "fix" that into single-token cancellation. Lazy expiry, no background reaper
  (Architecture Compliance Rule).
- **QRY-02 closure (review-2026-06-10 cycle):** the response cache was constructed
  but unreachable until a wiring fix — the EXACT defect class this story closes for
  the watchdog. Lesson: wiring claims must be verified by behavior tests (sensor API
  NOT contacted on denylist rejection; token actually cancelled on RSS kill), not by
  construction-site inspection.
- **D2 adjudication (taxonomy v1.68, proposals/cache-envelope-adjudication-2026-06-10.md):**
  E-WATCHDOG-001 = per-query GreedyMemoryPool trip (query at fault, not retryable);
  E-WATCHDOG-002 = process-RSS watchdog kill (retryable, query not necessarily at
  fault). The denylist counts watchdog-kill conditions and crashes, NOT pool trips.
  BC-2.15.007 v1.6 carries the ratified division of labor (v1.6 additionally carves
  record-cap E-QUERY-005 OUT of denylist eligibility — QRY cascade P3-02 companion
  to this story's P3-01 realignment); BC-2.15.008 v1.4 is the canonical trigger-set
  statement {timeout, rss_kill, crash} with EC-15-043 streak-reset semantics;
  `watchdog.rs::check_query` doc comment is stale on this (fix in Phase 1).
- **PLUGIN-MIGRATION-001-D lessons 16/17/24:** SAP-1 same-commit catalog rows;
  SID-1 — no `#[ignore]`-rationalized deferrals (the restart-persistence test is the
  ONE justified subprocess test; everything else mocks at the boundary); read the
  real types (`DenylistStatus`, `WatchdogStatus`, `QueryOptions`) before writing
  tests — do not infer shapes from BC prose.
- **S-DEMO-003 cascade lesson (behavioral audit):** story claims about code must be
  verified against the actual functions — every §Current State claim above was
  verified against develop on 2026-06-10 (function-name anchors per TD-VSDD-091).

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Watchdog/denylist deps wired via constructor Arc-DI; no placeholder-construct in the boot path | ADR-022 §C + Standing Rule 3 §4 | Red Gate test 1 + adversary probe |
| Memory measurement via `sysinfo`, never `/proc/self/status` | S-2.02 Architecture Compliance Rule | Existing watchdog.rs; no new probe code |
| Token registry stays `DashMap`, not Mutex; monitor cancels ALL tokens at Kill | S-2.02 EC-003/AC-4 design | Adversary probe: no single-token "fix" |
| DI-027 two-consecutive-check grace period via `should_terminate_for_memory` | BC-2.15.007 PC (VP-058) | Red Gate test 4 |
| Denylist expiry is LAZY (at query start); no background reaper | S-2.02 Architecture Compliance Rule | Red Gate test 11 |
| Pool trips (E-WATCHDOG-001) AND record-cap violations (E-QUERY-005) never increment the denylist counter — and reset the streak as non-terminated executions | BC-2.15.008 v1.4 + BC-2.15.007 v1.6 + DI-027 v1.7 (EC-15-043, both halves) | Red Gate test 7 |
| Crash trigger feeds the SAME consecutive counter as watchdog terminations (one streak, reason `crash`) | BC-2.15.008 v1.4 PC-2 + BC-2.15.005 | Red Gate test 14 |
| `QueryDenylisted` Display matches the taxonomy v1.70 verbatim E-QUERY-008 Message Format; force_execute hint in recovery guidance only | error-taxonomy.md v1.70 E-QUERY-008 row (P3-03) | Red Gate test 8 |
| Only taxonomy error codes used: E-QUERY-004/005/008, E-WATCHDOG-002, E-STORE-009/010 — no invented codes | error-taxonomy.md + CLAUDE.md | Red Gate tests 3, 5, 8, 14 + adversary |
| `deny_unknown_fields` retained on QueryToolParams; injection scan BEFORE domain logic unchanged | BC-2.09.001 / BC-2.10.012 | Red Gate test 10 + adversary |
| prism-mcp gains NO prism-storage dependency (Arc-DI seam instead) | Forbidden Dependencies below | Cargo.toml review + adversary |
| New/extended pub types in prism-storage/prism-query get `#[non_exhaustive]` + ci.yml EXPECTED bump | CLAUDE.md non-exhaustive discipline | Compile-fail gate `tests/external/non-exhaustive-violation/` |
| New `tracing::*!(event_type=…)` sites require same-commit BC-2.16.002 rows | SAP-1 | Adversary SAP-1 probe |
| No `unwrap()`/`expect()` on the gate/record/reset paths; storage errors propagate as structured `PrismError` | CLAUDE.md error handling | Clippy + adversary |
| Monitor check interval 3s fixed (not configurable) | BC-2.15.006 invariant 3 (spec wins over the 500ms code comment) | Red Gate test 1 |
| Spec content cites function names + behavioral anchors, not line numbers | TD-VSDD-091 | This story complies |

---

## Library & Framework Requirements

Versions pinned from the workspace `Cargo.toml`. No NEW external dependencies.

| Crate | Version | Usage |
|-------|---------|-------|
| `sha2` | workspace-pinned | Fingerprint helper (already a prism-query dep for cache keys) |
| `sysinfo` | workspace-pinned (prism-storage `watchdog` feature) | RSS probe (existing) |
| `dashmap` | workspace-pinned (prism-storage `watchdog` feature) | Token registry (existing) |
| `tokio-util` | workspace-pinned | `CancellationToken` (existing in prism-query + prism-storage) |
| `tokio` | `1` (multi-threaded) | Monitor task + select-on-token (existing) |
| `tracing` | workspace-pinned | Structured emissions (BC-2.16.002 rows required) |
| `serde` | workspace-pinned | `force_execute` field (typed bool; default false) |

**Forbidden patterns:**
- Do NOT hash the raw query string for the fingerprint (canonical rendering only)
- Do NOT add a second fingerprint definition anywhere (single binding, AC-006)
- Do NOT construct `PrismError::QueryDenylisted` with synthesized field values —
  fields come from the stored denylist entry

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-storage/src/denylist.rs` | MODIFY | Value encoding + `DenylistEntry` gain `reason`/`denylisted_at`; status-returning lookup; `record_success`; expiry param |
| `crates/prism-storage/src/watchdog.rs` | MODIFY | `spawn_monitor` DI-027 two-check wiring; stale E-WATCHDOG-001 doc fix; `WatchdogStatus.denylist` carries extended entries |
| `crates/prism-storage/src/recovery.rs` | MODIFY | Restart evaluation routine: scanned markers → `advance_crash_counter` → shared denylist counter with reason `crash`; E-STORE-010 retry semantics (AC-014) |
| `crates/prism-storage/src/dirty_bits.rs` | MODIFY (if needed) | Key/value alignment to BC-2.15.005 `DirtyBitEntry` (fingerprint key, `sync: true` put) — primitives reused; touch only where the shipped shape diverges from the BC struct |
| `crates/prism-core/src/error.rs` | MODIFY | `QueryDenylisted` Display → taxonomy v1.70 verbatim E-QUERY-008 Message Format (force_execute hint moves to recovery guidance); `reason` field doc vocabulary retired "(timeout / memory / record_limit)" → "(timeout / rss_kill / crash)" (P3-03; AC-008) |
| `crates/prism-query/src/engine.rs` | MODIFY | Watchdog Arc-DI constructor params; query-start gate; token lifecycle; dirty-bit set/clear at execute entry (AC-014); failure/success hooks; `QueryOptions.force_execute` |
| `crates/prism-query/src/` (new module, e.g. `fingerprint.rs`) | CREATE | SHA-256 normalized-query fingerprint helper (parser canonical rendering) |
| `crates/prism-bin/src/boot.rs` | MODIFY | Post-step-6 watchdog construction + spawn_monitor(3s) + INFO limits log + config read + step-9 threading + crash-recovery sub-step (markers → reason `crash`, AC-014) |
| `crates/prism-mcp/src/server.rs` | MODIFY | `QueryToolParams.force_execute`; `watchdog_status` unstub via Arc-DI seam; capability-gated clear |
| `crates/prism-storage/src/tests/` + `crates/prism-query/src/tests/` + `crates/prism-bin/tests/` + prism-mcp test modules | MODIFY/CREATE | 14 Red Gate tests per plan |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Same-commit catalog rows for new event_type emissions (SAP-1) |
| `.factory/specs/behavioral-contracts/BC-2.15.005-*.md`, `BC-2.15.007-*.md`, `BC-2.15.008-*.md` | MODIFY | Traceability "Stories" backlink rows gain S-WATCHDOG-WIRING-001 |
| `.github/workflows/ci.yml` | MODIFY (only if `#[non_exhaustive]` newly annotated) | `EXPECTED=` bump per non-exhaustive discipline |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.15.007 EC-15-024 | Query at 29.9s on 30s timeout; sensor call returns at 30.1s | Terminated at 30s (existing E-QUERY-004 path unchanged); partial response discarded; failure recorded with reason `timeout` |
| EC-002 | BC-2.15.007 EC-15-025 | Scheduled query terminated by watchdog | Failure recorded against its fingerprint; engine-level behavior identical for all callers (schedule continuation semantics owned by S-4.01's executor) |
| EC-003 | BC-2.15.007 EC-15-026 | Concurrent queries; one trips its per-query pool | E-WATCHDOG-001 to that query only (pool layer, BC-2.11.006); denylist counter UNCHANGED and any prior streak RESET (EC-15-043 both halves, AC-007); others continue |
| EC-004 | BC-2.15.007 EC-15-027 | Process RSS at Kill level with multiple registered queries | After two consecutive checks, ALL registered tokens cancelled (S-2.02 design); each receives E-WATCHDOG-002; each fingerprint records an `rss_kill` failure |
| EC-005 | BC-2.15.008 EC-15-028 | Timeout, timeout, success on third attempt | Counter reset to 0 by the success; no denylisting (AC-009) |
| EC-006 | BC-2.15.008 EC-15-029 | Denylisted, then `force_execute: true` succeeds | Entry cleared; subsequent normal execution allowed (AC-010) |
| EC-007 | BC-2.15.008 EC-15-030 | Scheduled query is denylisted | Engine-level gate rejects with E-QUERY-008 for ANY caller (this story); skip-with-warning + schedule-not-disabled handling is owned by S-4.01 (Schedule CRUD and Execution Loop) when its executor consumes the error |
| EC-008 | BC-2.15.008 EC-15-031 | SHA-256 collision between two queries | Both affected by the same entry (accepted, astronomically improbable) |
| EC-009 | BC-2.15.008 EC-15-032 | `watchdog.denylist_threshold: 1` | Denylisted on first watchdog termination (AC-012) |
| EC-010 | This story | `force_execute: true` on a NON-denylisted query | No-op bypass; executes normally; still monitored; success-reset semantics unchanged |
| EC-011 | This story | Denylist storage read fails at the gate (RocksDB error) | Structured `PrismError` storage variant propagates (fail-closed for observability); never a silent allow-with-swallowed-error, never panic |
| EC-012 | This story | Forced execution terminated by watchdog again | Failure recorded (counter increments from its current value); entry NOT cleared (only non-terminated execution clears, per BC invariant 1) |
| EC-013 | This story | Pre-existing denylist entries written by the old 3-field value encoding (no reason / denylisted_at) | Defensive decode: legacy values are lazily DELETED on first read with a WARN — no production path ever wrote them (`record_failure` had zero production callers, per §Current State), so no real denial state can be lost; never fabricate a `reason` outside the {timeout, rss_kill, crash} vocabulary; no parse panic on old CF contents |
| EC-014 | BC-2.15.005 EC-15-017 | Clean shutdown (SIGTERM) with in-flight queries completing | All orderly exits clear their dirty bits; next startup finds zero markers; no counter increments |
| EC-015 | BC-2.15.005 EC-15-018 | Crash during the dirty-bit write itself | Marker may or may not exist (atomic RocksDB put); at worst one crash goes undetected; no decode panic |
| EC-016 | BC-2.15.005 EC-15-019 | 100 dirty bits on startup | Processed sequentially in the boot sub-step; recovery may take seconds; boot proceeds after processing |
| EC-017 | BC-2.15.005 EC-15-020 | Marker from a previous Prism version / unknown source variant | WARN logged; marker cleared; no recovery attempted; no panic |
| EC-018 | BC-2.15.005 E-STORE-009 | Dirty-bit write fails before execution (disk full, I/O error) | Query aborted fail-closed with E-STORE-009 — without the marker a crashing query could never be denylisted |
| EC-019 | BC-2.15.005 E-STORE-010 | Recovery action fails during the startup scan | WARN logged; marker NOT cleared; recovery retried on next startup (idempotent per BC-2.15.005 invariant 3) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| Fingerprint helper (canonical rendering → SHA-256 hex) | `prism-query/src/fingerprint.rs` (new) | Pure | BC-2.15.008 PC-1 |
| Query-start denylist gate | `prism-query/src/engine.rs` (`execute` pre-check) | Effectful (CF read) | BC-2.15.008 PC-2 |
| Token register/deregister lifecycle + select-on-token | `prism-query/src/engine.rs` | Effectful | BC-2.15.007 PC-1/PC-2 |
| DI-027 two-check monitor policy | `prism-storage/src/watchdog.rs::spawn_monitor` + `proofs::should_terminate_for_memory` | Pure decision fn + effectful loop | BC-2.15.007 DI-027; VP-058 |
| Failure record / success reset | `prism-storage/src/denylist.rs` (`record_failure`, new `record_success`) | Effectful (CF write) | BC-2.15.008 PCs |
| Dirty-bit lifecycle (set `sync: true` / clear on orderly exit) | `prism-query/src/engine.rs` → `prism-storage/src/dirty_bits.rs` | Effectful (CF write) | BC-2.15.005 PCs 1–2 |
| Boot crash-recovery evaluation (markers → counter, reason `crash`) | `prism-bin/src/boot.rs` sub-step → `prism-storage/src/recovery.rs` (`check_dirty_on_startup` + `advance_crash_counter`) | Pure decision fn (VP-057) + effectful scan/write | BC-2.15.005 PCs 3–4; BC-2.15.008 PC-2 crash trigger |
| Boot construction + threading | `prism-bin/src/boot.rs` (post-step-6 sub-step → step 9) | Effectful | ADR-022 §B; BC-2.15.007 PC-2 (precondition) |
| `force_execute` param + `watchdog_status` unstub | `prism-mcp/src/server.rs` | Effectful (tool layer) | BC-2.15.008 override + state exposure |

---

## Forbidden Dependencies

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-mcp` | `prism-storage` | MCP layer stays storage-agnostic; watchdog status reaches PrismServer via an Arc-DI seam wired in prism-bin (mirror the `Option<Arc<QueryEngine>>` pattern). If prism-mcp gains a prism-storage dependency, the build review MUST fail it. |
| `prism-storage` | `prism-query` | Fingerprint computation (parser-dependent) lives in prism-query; prism-storage continues to accept `fingerprint: &str`. A reverse dependency would cycle the graph. |
| `prism-core` | `sha2`, `dashmap`, `sysinfo` (no new) | Core stays mechanism-free; the error variants it already owns are sufficient |

---

## SAP-1 Compliance (Structured Event Catalog)

Expected potential emissions (implementer must enumerate actual sites):
- `event_type = "watchdog.query_killed"` (RSS kill path, AC-003)
- `event_type = "watchdog.denylist_added"` / `"watchdog.denylist_hit"` /
  `"watchdog.denylist_cleared"` (AC-005/AC-008/AC-013)
- `event_type = "watchdog.limits_configured"` (boot INFO log, AC-001)
- `event_type = "watchdog.crash_recovered"` (boot crash-recovery sub-step: marker
  processed / counter incremented with reason `crash` / E-STORE-010 retry WARN,
  AC-014)

Each added emission gets its BC-2.16.002 catalog row (field schema, audit role,
recurrence policy) in the SAME commit. If any listed emission is NOT added (e.g.,
covered by the audit path instead), state so explicitly in the PR description
(D-765 precedent: `?` propagation / audit-entry coverage does not require a
catalog row).

---

## Out of Scope (explicit routing flags — NOT silent deferrals)

1. **BC-2.15.006 graduated-level config profile** (normal/restrictive/permissive
   profiles, `watchdog.level`, per-limit TOML overrides with clamping,
   E-WATCH-001/E-WATCH-002 startup errors): NOT implemented by S-2.02 (hardcoded SI
   512 MB + fixed thresholds) and NOT in this story's dispatch scope. This is a
   SECOND POL-15-class gap on an active P0 BC, surfaced to the orchestrator with
   proposed routing: product-owner/story-writer to anchor a follow-up story
   (suggested ID S-WATCHDOG-CONFIG-PROFILE-001) covering the BC-2.15.006 config
   surface. This story implements only the BC-2.15.006-adjacent items its dispatch
   scope requires (3s interval invariant, INFO limits log, denylist
   threshold/expiry keys).
2. **EC-15-030 skip-with-warning schedule semantics**: rejection is guaranteed here
   at the engine level for all callers; the warning + schedule-not-disabled
   handling belongs to S-4.01 (Schedule CRUD and Execution Loop), which consumes
   E-QUERY-008 from its executor.

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.3 | 2026-07-08 | **Reconciling pin round (pass-4 closures): error-taxonomy v1.70→v2.26. One live version-pin cite updated: frontmatter YAML Architecture Intelligence string `error-taxonomy v1.70`. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 1.2→1.3; updated 2026-07-08 (POL-23).** |
| v1.2 | 2026-06-10 | POL-8 propagation of BC-2.15.005 v1.4 (PO micro-burst handoff 2026-06-10; story-writer micro-burst). The v1.1 AC-014 wording-sharpen flag ("successful" → orderly completion, postcondition 2) is RESOLVED — the product-owner ratified the orderly-exit semantics in BC-2.15.005 v1.4 (cleared on EVERY orderly exit, successful completion OR structured-error termination; only process death leaves a marker). 3 pins advanced: (1) Token Budget BC-2.15.005 v1.3→v1.4; (2) BC table postcondition-2 summary rewritten to the v1.4 orderly-exit wording; (3) AC-014 parenthetical now cites BC-2.15.005 v1.4 directly — "flagged to product-owner" note retired, double-count rationale retained per the BC's recorded rationale. No AC count, Red Gate, points, scope, or trigger-set changes — wording-pin propagation only. |
| v1.1 | 2026-06-10 | QRY cascade P3-01 (HIGH) + P3-03 (MED) realignment to BC-2.15.008 v1.4 (story-writer fix-burst, 2026-06-10): the story was authored in parallel with the BC's v1.4 amendment and encoded the RETIRED trigger set ("timeout/memory/record_limit", E-QUERY-005 record-cap as a trigger, no crash trigger). **P3-01:** trigger set realigned to the ratified v1.4 set {`timeout` = E-QUERY-004, `rss_kill` = E-WATCHDOG-002, `crash` = CrashRecovery markers per BC-2.15.005} with reason vocabulary {timeout, rss_kill, crash} — AC-005 rescoped to the BC-2.15.007 v1.6 denylist-eligibility clause (E-QUERY-005 explicitly NOT recorded); AC-007 re-quoted to the v1.4 postcondition-2 verbatim text and extended to assert BOTH halves of EC-15-043 (excluded failures do not increment AND reset the streak; test 7 renamed `test_BC_2_15_008_excluded_failures_do_not_increment_and_reset_streak`, exercising the BC's canonical pool-trip vector); risk_mitigations §3 + Phase-1/Phase-2 tasks + EC-003/EC-004/EC-013 + Architecture Compliance rows rewritten; Token Budget pins advanced (BC-2.15.008 v1.3→v1.4, BC-2.15.007 v1.5→v1.6). NEW AC-014 + Red Gate test 14: the v1.4 third trigger (crash) was wholly absent — dirty-bit set/clear wired at the execute entry point (E-STORE-009 fail-closed) + boot crash-recovery sub-step (check_dirty_on_startup + advance_crash_counter → shared counter, reason `crash`, E-STORE-010 retry) — BC-2.15.005 ADDED to behavioral_contracts with full propagation (BC table row, AC trace, token budget, backlink task; anchor justification in frontmatter: primitives merged via S-2.01 PR #43 with zero production callers, no unmerged story claims the wiring; BC-2.15.005 PC-2 "successful" read as "orderly completion" — flagged to product-owner as wording sharpen). EC-014..EC-019 added (BC-2.15.005 EC-15-017..020 + E-STORE-009/010 coverage). **P3-03:** `crates/prism-core/src/error.rs` MODIFY row + Phase-2 task added — `QueryDenylisted` Display aligned to taxonomy v1.70 verbatim E-QUERY-008 Message Format ("Query has been denylisted after {N} consecutive failures ({reason}). Denylist expires at {expiry}."; force_execute hint moves to recovery guidance) and the `reason` field doc vocabulary retired (timeout/memory/record_limit → timeout/rss_kill/crash); AC-008 + Red Gate test 8 anchor the Display assertion. Counts: 13→14 ACs / 13→14 Red Gate tests; points 8→9 (crash-trigger wiring +1); estimated_days 3→3.5; SAP-1 expected emissions + `watchdog.crash_recovered`. |
| v1.0 | 2026-06-10 | Initial authoring per orchestrator story-writer dispatch (2026-06-10 review cycle): QRY fix-burst STOP report discovered ResourceWatchdog has zero production wiring (never constructed in boot; no query-start denylist gate; E-QUERY-008 unconstructible; watchdog_status stubbed; VP-058 grace period not wired into spawn_monitor) — active P0 BCs 2.15.007/008 unreachable, POL-15 class. Existing-anchor check: S-2.02 (sole BC-2.15.007/008 owner) is MERGED (PR #52); no unmerged wave-4/5 or follow-up story claims the wiring — new story required. 13 ACs; 13 Red Gate tests; 8 pts; ratified trigger set per taxonomy v1.68 D2 (pool trips excluded). Sequenced post-demo backlog alongside S-CACHE-SPEC-COMPLIANCE-001 (NOT demo-blocking: per-query pool + query timeout already wired). BC-2.15.006 config-profile gap flagged out-of-scope with routing recommendation (S-WATCHDOG-CONFIG-PROFILE-001 follow-up). |
