---
document_type: story
story_id: S-WATCHDOG-CONFIG-PROFILE-001
title: "prism-bin + prism-storage + prism-query + prism-mcp: Graduated Watchdog Config Profiles (normal/restrictive/permissive) — [watchdog] level + Per-Limit TOML Overrides with Clamping (E-WATCH-001/002) + Resource History Exposure"
# wave: NOT wave-scheduled — post-demo backlog per orchestrator sequencing directive,
# same pattern as its dependency S-WATCHDOG-WIRING-001 and sibling
# S-CACHE-SPEC-COMPLIANCE-001. NOT demo-blocking: the demo runs on the BC-correct
# normal-level defaults already hardcoded at every enforcement point (512MB SI RSS
# budget in watchdog.rs, 200MB pool + 30s timeout + 10K records + fan-out 10 in
# QueryEngineConfig::default()) — the profile system changes WHICH values are used,
# not whether limits exist.
wave: post-demo-backlog
epic_id: maintenance
priority: P1
# Priority rationale: P1 — BC-2.15.006 is an ACTIVE P0 behavioral contract whose
# defining surface (graduated level profiles, per-limit TOML overrides, clamping,
# E-WATCH-001/002 startup errors) was NEVER implemented by merged owner S-2.02
# (which shipped hardcoded normal-level constants only: SI 512MB budget + fixed
# 0.70/0.85/0.95 thresholds) and is NOT in S-WATCHDOG-WIRING-001's dispatch scope
# (explicitly flagged in that story's §Out of Scope item 1 with routing
# recommendation for THIS story ID). This is the second POL-15-class
# (runtime_wiring_required_for_accepted_adrs / active-contract-without-
# production-path) gap on the watchdog subsystem, surfaced by the
# S-WATCHDOG-WIRING-001 authoring burst (2026-06-10). Same P1 rationale as that
# sibling: active P0 BC clauses unreachable in production.
# Status rationale: draft (NOT ready) despite non-empty behavioral_contracts —
# (1) orchestrator-directed sequencing: post-demo backlog; must not be dispatched
#     before the live-demo objective (T5 Story B → T6 → T8) completes;
# (2) hard build-order dependency on S-WATCHDOG-WIRING-001 (unmerged): this story
#     parameterizes the boot construction sub-step, the [watchdog] TOML section,
#     the QueryEngine/PrismServer Arc-DI seams, the INFO limits log, and the
#     watchdog_status live snapshot — none of which exist on develop today;
# (3) remove-uncertainty must run against the post-wiring develop baseline
#     (WatchdogConfig section shape, boot sub-step location, watchdog_status
#     snapshot schema are all surfaces S-WATCHDOG-WIRING-001 creates).
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-10T00:00:00Z"
created: "2026-06-10"
modified: "2026-06-10T00:00:00Z"
phase: 3
tdd_mode: strict
subsystems: [SS-15, SS-22, SS-11, SS-10]
# Subsystem anchor justifications (per ARCH-INDEX Subsystem Registry):
#   SS-15 (Storage Layer) owns this story's core scope because BC-2.15.006 is an
#     SS-15 contract and the profile-resolution domain logic (WatchdogProfile,
#     EffectiveLimits, clamping) plus the per-query resource history live in
#     prism-storage's watchdog module (the ResourceWatchdog home per S-2.02).
#   SS-22 (Process Lifecycle) owns the boot-sequence integration: [watchdog]
#     config parsing at step 2 (BC-2.06.011 surface), E-WATCH-001 fatal-startup
#     mapping to the ADR-022 §A exit-code table, and threading resolved limits
#     into the construction sub-step S-WATCHDOG-WIRING-001 creates.
#   SS-11 (Query Execution) owns the engine-side enforcement-point threading:
#     QueryEngineConfig (timeout_secs, memory_pool_bytes, max_materialized_records,
#     max_fan_out_concurrency) stops being Default::default() at boot step 8 and
#     becomes profile-derived; the RSS-delta snapshot hooks wrap execute().
#   SS-10 (MCP Interface) owns the watchdog_status response extension (configured
#     level + effective limits + resource history) via the Arc-DI seam
#     S-WATCHDOG-WIRING-001 wires (no prism-storage dep in prism-mcp).
target_module: prism-storage
crates_touched: [prism-bin, prism-storage, prism-query, prism-mcp]
behavioral_contracts: [BC-2.15.006]
# BC anchor note: BC-2.15.006 (Resource Watchdog Initialization — graduated level,
# TOML overrides, clamping, E-WATCH-001/002, startup INFO log, watchdog_status
# exposure of limits/denylist/resource-history). The denylist-exposure and
# clear-path clauses of the watchdog_status tool are owned by S-WATCHDOG-WIRING-001
# (BC-2.15.008 anchored there); THIS story covers the BC-2.15.006-owned portions:
# configured level + effective limits + resource history. BC-2.15.007/008 are
# regression surface only (see §Preserved Contracts), not re-anchored here.
verification_properties: []
# VP note (deliberate empty array, NOT an omission): BC-2.15.006's §Verification
# Properties table is ratified "(none)" — "Override-takes-precedence is a trivial
# Option::unwrap_or merge, unit test coverage sufficient; watchdog-cannot-be-disabled
# is a hardcoded constant, code review property; enforcement side covered by
# VP-014/015." Inventing a VP reference here would contradict the converged BC.
# VP-058 anchor stays at S-2.02 (proof owner) per S-WATCHDOG-WIRING-001 precedent.
depends_on:
  - S-WATCHDOG-WIRING-001
  # Dependency anchor — REAL BUILD-ORDER dependency (unlike the sibling stories'
  # sequencing-only edges): S-WATCHDOG-CONFIG-PROFILE-001 depends on
  # S-WATCHDOG-WIRING-001 because profiles configure a watchdog that must exist —
  # that story creates (a) the boot construction sub-step + spawn_monitor this
  # story parameterizes with profile-derived budget_bytes, (b) the [watchdog]
  # TOML section (denylist_threshold/denylist_expiry_secs) this story extends
  # with level + 4 override keys, (c) the Arc-DI seams into QueryEngine and
  # PrismServer this story threads EffectiveLimits through, (d) the INFO
  # limits-at-startup log + watchdog.limits_configured catalog row this story
  # makes profile-derived, and (e) the unstubbed watchdog_status tool this story
  # extends with level/limits/resource-history fields. Transitively inherits the
  # post-demo sequencing edges (S-DEMO-DTU-LIVE-SCENARIO-001-B,
  # S-DEMO-MULTI-TENANT-DTU-001, S-DEMO-004) via S-WATCHDOG-WIRING-001's
  # depends_on — no need to duplicate them here.
blocks: []
points: 8
# Points justification:
#   1. Pure profile-resolution core in prism-storage (WatchdogProfile enum,
#      EffectiveLimits, override merge, clamping with warning data): 1.5 pts
#   2. Config schema: [watchdog] section extension (level + 4 override keys,
#      deny_unknown_fields, E-WATCH-001 fatal mapping to exit 2): 1.5 pts
#   3. Enforcement-point threading: ResourceWatchdog budget_bytes +
#      QueryEngineConfig (pool/timeout/records/fan-out) profile-derived at boot
#      step 8 (replaces QueryEngineConfig::default()): 2 pts
#   4. Per-query resource history: RSS before/after snapshots around execute(),
#      bounded ring buffer, watchdog_status exposure: 1.5 pts
#   5. watchdog_status level/limits exposure + INFO log enrichment: 0.5 pt
#   6. Red Gate suite (~10 tests incl. exit-code subprocess test): 1 pt
estimated_days: 3
risk: MEDIUM
# Risk justification: the enforcement-point threading touches the same boot step 8
# / engine construction surfaces as S-WATCHDOG-WIRING-001 and the demo fix branches
# (re-baseline required); the RSS-delta hooks add a probe read on the query hot
# path (cheap, but ordering vs the wiring story's token lifecycle must be tested).
# The pure resolution core and the clamping are low-risk. Unit-convention hazard
# (SI process budget vs MiB per-query pool) is mitigated by an explicit
# Architecture Compliance Rule below.
acceptance_criteria_count: 10
red_gate_tests: 10
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Unit conventions are NOT homogenized: process-RSS budget stays SI
    (ADR-S2.02-002: 512 MB = 512_000_000 bytes) and the per-query pool stays
    binary MiB as-built (memory.rs QUERY_MEMORY_POOL_BYTES = 200*1024*1024,
    BC-2.11.006). Profile values map into each subsystem's existing convention;
    a test pins both conversions so neither silently flips."
  - "The clamp set implemented with E-WATCH-002 is exactly the three BC/taxonomy-
    specified minimums (64 MB memory, 5s timeout, 1000 records) plus ONE derived
    floor (max_concurrent_api_calls >= 1, see EC-007 flag) — no other invented
    minimums."
  - "Profile resolution is a pure function returning (EffectiveLimits,
    Vec<ClampWarning>) so boot (the effectful shell) owns the E-WATCH-002 warn
    logging and the E-WATCH-001 fatal exit — keeps the core unit-testable without
    subprocess tests (SID-1)."
  - "QueryEngineConfig is already doc-commented 'configurable via TOML'
    (engine.rs) but boot constructs it via Default::default() (boot.rs step 8) —
    this story closes that doc-vs-code gap (Standing Rule 3 §3 class) rather than
    adding a parallel config struct."
  - "RSS-delta history hooks must not contact the probe under lock or block the
    hot path: snapshot via the existing MemoryProbe Arc before/after execute,
    push into a bounded ring buffer (fixed capacity; O(1) push, oldest evicted)."
inputs:
  - .factory/specs/behavioral-contracts/BC-2.15.006-resource-watchdog-initialization.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/domain-spec/capabilities.md
  - .factory/stories/S-WATCHDOG-WIRING-001-resource-watchdog-production-wiring.md
  - .factory/specs/architecture/api-surface.md
  - crates/prism-storage/src/watchdog.rs
  - crates/prism-query/src/engine.rs
  - crates/prism-query/src/memory.rs
  - crates/prism-bin/src/boot.rs
  - crates/prism-mcp/src/server.rs
traces_to: ["S-WATCHDOG-WIRING-001 §Out of Scope item 1 (2026-06-10): BC-2.15.006 graduated-config-profile surface never implemented by merged owner S-2.02 — second POL-15-class active-P0-BC gap on the watchdog subsystem; routing recommendation S-WATCHDOG-CONFIG-PROFILE-001 accepted by orchestrator dispatch 2026-06-10"]
supersedes: []
---

# S-WATCHDOG-CONFIG-PROFILE-001: Graduated Watchdog Config Profiles — `[watchdog] level` + Per-Limit TOML Overrides with Clamping (E-WATCH-001/002) + Resource History Exposure

Implement the BC-2.15.006 configuration surface that S-2.02 never built and
S-WATCHDOG-WIRING-001 explicitly scoped out: the three-level graduated profile
(`normal` / `restrictive` / `permissive`) selected via `watchdog.level` in
prism.toml, per-limit TOML overrides that take precedence over profile defaults,
below-minimum clamping with `E-WATCH-002` warnings, fatal `E-WATCH-001` startup
rejection of invalid levels, profile-derived limits threaded into EVERY
enforcement point (process-RSS budget, per-query pool, query timeout, fan-out
concurrency, materialized-record cap), and the BC-2.15.006-owned portions of the
`watchdog_status` exposure (configured level, effective limits, per-query
resource history).

**Gap provenance (POL-15 class):** S-2.02 (merged PR #52, 2026-04-25) shipped the
graduated `WatchdogLevel` RSS-threshold enum and hardcoded normal-level constants
(SI 512 MB budget; 0.70/0.85/0.95 fractions) but NO config profile system: there
is no `watchdog.level` key, no override keys, no clamping, and E-WATCH-001/002
are unconstructible from any production path. `QueryEngineConfig` carries the
other four limits and is doc-commented "configurable via TOML", yet boot
constructs it with `Default::default()` (boot.rs step 8) — the BC's restrictive
and permissive columns are unreachable in production. Surfaced by the
S-WATCHDOG-WIRING-001 authoring burst (2026-06-10) as a second POL-15-class gap
on an active P0 BC; that story's §Out of Scope item 1 routes the surface here.

**Sequencing (orchestrator-directed, 2026-06-10):** post-demo backlog, strictly
after S-WATCHDOG-WIRING-001 (real build-order dependency — see frontmatter anchor).
NOT demo-blocking: every enforcement point already runs on the BC-correct
normal-level defaults as hardcoded constants.

---

## Narrative

As a Prism platform operator deploying to environments with very different
resource envelopes (resource-constrained analyst laptops vs large MSSP servers),
I want to select a graduated watchdog profile in prism.toml and override
individual limits where my environment demands it — with unsafe values clamped
loudly rather than honored silently — so that one binary serves restrictive,
normal, and permissive deployments with finite, operator-visible resource bounds
that the watchdog enforces exactly as configured.

---

## Behavioral Contracts

| BC | Title | Key Clauses Implemented |
|----|-------|------------------------|
| BC-2.15.006 | Resource Watchdog Initialization — Set Memory/CPU/Timeout Limits Based on Graduated Level | Precondition 2 (`watchdog.level` ∈ {normal, restrictive, permissive}, default normal); postcondition 1 (full level-defaults table across all six limits); postcondition 2 (override keys `watchdog.memory_limit_mb` / `query_timeout_seconds` / `max_concurrent_api_calls` / `max_materialized_records`); postcondition 3 (override precedence); postcondition "limits logged at startup at INFO"; postcondition 5 (watchdog_status exposes current limits + resource history — the BC-2.15.006-owned portions); invariants 1–3 (always-active, hardcoded defaults, fixed 3s interval); error conditions E-WATCH-001 (fatal invalid level) and E-WATCH-002 (clamp + warn); EC-15-021/022/023; all four Canonical Test Vectors |

### Preserved Contracts (regression surface — NOT in `behavioral_contracts`; no new ACs)

| BC | Surface this story touches | Preservation requirement |
|----|---------------------------|--------------------------|
| BC-2.15.007 | `ResourceWatchdog::budget_bytes` becomes profile-derived | Kill threshold stays 95% of the (now-configurable) process budget; DI-027 two-check grace period and token-cancellation behavior wired by S-WATCHDOG-WIRING-001 unchanged. Existing suite passes with normal-level defaults byte-identically. |
| BC-2.15.008 | `[watchdog]` TOML section shared with denylist keys | `denylist_threshold` / `denylist_expiry_secs` parsing (S-WATCHDOG-WIRING-001 AC-012) unchanged; this story only ADDS keys to the same section. |
| BC-2.11.006 | `QueryEngineConfig` { timeout_secs, memory_pool_bytes, max_materialized_records } | Normal-level resolved values are EXACTLY the BC-2.11.006 defaults (30s / 200 MiB / 10K) — a config absent from TOML produces a byte-identical engine config to today's `Default::default()`. E-QUERY-004/E-WATCHDOG-001 error paths unchanged. |
| BC-2.11.005 | `QueryEngineConfig.max_fan_out_concurrency` | Normal default stays 10; fan-out semantics unchanged, only the bound becomes profile-derived. |
| BC-2.06.011 | prism.toml config load (boot step 2) | `[watchdog]` parse failures classify as config-invalid (exit 2, ADR-022 §A) like every other schema failure; no new exit codes. |
| BC-2.16.002 | Structured Event Catalog | `watchdog.limits_configured` row (created by S-WATCHDOG-WIRING-001) gains the level/override/clamp fields in the SAME commit that changes the emission; any NEW emission (e.g., clamp warning) gets a same-commit row (SAP-1). |

---

## Current State (verified against develop, 2026-06-10 — re-verify in Phase 0 after S-WATCHDOG-WIRING-001 merges)

- `prism-storage/src/watchdog.rs`: `WatchdogLevel` is the RSS-pressure level
  (Normal/Warn/Throttle/Kill) — NOT the config profile; do not conflate. Budget
  hardcoded `512 * 1_000 * 1_000` (SI, ADR-S2.02-002) in both `new()` and
  `with_probe()`. No profile type, no override merge, no clamping, no resource
  history (zero `history`/`VecDeque` hits).
- `prism-query/src/engine.rs`: `QueryEngineConfig { timeout_secs: 30,
  max_materialized_records: 10_000, memory_pool_bytes: 200 * 1024 * 1024,
  max_fan_out_concurrency: 10 }` — doc says "configurable via TOML"; nothing
  reads TOML into it.
- `prism-bin/src/boot.rs`: `let query_config = QueryEngineConfig::default();`
  at step 8 (`QueryEngine::new_full` construction site). `PrismConfig`
  (`#[non_exhaustive]`, NOT deny_unknown_fields at top level) has no watchdog
  field. `step2_load_config` maps TOML/schema failures to
  `BootError::ConfigInvalid` → `EXIT_CONFIG_INVALID` (exit 2).
- `prism-core` error surface: E-WATCH-001/002 exist ONLY as taxonomy rows
  (error-taxonomy.md v1.69 rows 365–366); no `PrismError` variant constructs
  them. E-WATCH-001 is a fatal startup message (BootError display carries it);
  E-WATCH-002 is a WARN-level log + clamp, not a returned error.
- After S-WATCHDOG-WIRING-001 (dependency, unmerged): boot constructs
  `Arc<ResourceWatchdog>` post-step-6, spawns the 3s monitor, logs limits at
  INFO, threads Arcs into QueryEngine/PrismServer, parses `[watchdog]
  denylist_threshold/denylist_expiry_secs`, and unstubs `watchdog_status`. This
  story builds on every one of those surfaces.

### Taxonomy verification (performed 2026-06-10 against error-taxonomy.md v1.69)

| Code | Row present | Severity/Category | Message template | Resolution |
|------|------------|-------------------|------------------|------------|
| E-WATCH-001 | YES (row in §Configuration) | broken / configuration | "Invalid watchdog level: '{value}'. Valid: normal, restrictive, permissive" | Fatal startup error; check `watchdog.level` in TOML config |
| E-WATCH-002 | YES (row in §Configuration) | cosmetic / configuration | "Watchdog override value below safe minimum: {param}={value} (minimum {min})" | Value clamped to minimum (64 MB memory, 5s timeout, 1000 records); warning logged |

Both rows match BC-2.15.006's Error Conditions table exactly. No taxonomy
amendment required for the core scope; the ONE derived floor (EC-007) adds
"1 call" to the E-WATCH-002 resolution enumeration — flagged below, same-commit
taxonomy touch, no new codes, no renumbering (POL-1 safe).

---

## Acceptance Criteria

### Group A — Profile system (level selection + defaults)

**AC-001 — `watchdog.level` parsed at boot; absent section/key defaults to `normal`**
(traces to BC-2.15.006 precondition 2 + EC-15-023)

Given prism.toml with `[watchdog] level = "restrictive"` (or `"normal"` /
`"permissive"`), when boot step 2 loads config, then the parsed `WatchdogConfig`
carries that profile; and given prism.toml with NO `[watchdog]` section or a
`[watchdog]` section without `level`, then the profile defaults to `normal` and
all six limits resolve to the normal column (EC-15-023) — producing a
`QueryEngineConfig` byte-identical to today's `Default::default()` and the
watchdog's existing SI 512 MB budget.

Red Gate: `test_BC_2_15_006_level_parse_and_absent_defaults_to_normal`

**AC-002 — Pure profile resolution implements the full level-defaults table**
(traces to BC-2.15.006 postcondition 1 table)

Given each of the three profiles with no overrides, when the pure resolution
function (`resolve_limits(profile, overrides) -> (EffectiveLimits,
Vec<ClampWarning>)` in prism-storage's watchdog module) runs, then the resolved
limits match the BC table exactly: normal = 512 MB RSS / 200 MB per-query / 30s /
10 calls / 10,000 records; restrictive = 256 MB / 100 MB / 15s / 5 / 5,000;
permissive = 2048 MB / 512 MB / 120s / 32 / 50,000; check interval 3s in all
three (not a resolvable field — hardcoded constant, invariant 3). Level defaults
are hardcoded in the resolution function (invariant 2 — they cannot be removed,
only overridden).

Red Gate: `test_BC_2_15_006_profile_defaults_match_bc_table_all_levels`

**AC-003 — Invalid `watchdog.level` is a fatal startup error (E-WATCH-001, exit 2)**
(traces to BC-2.15.006 error condition E-WATCH-001 + Canonical Test Vector "Invalid level")

Given prism.toml with `watchdog.level = "extreme"` (or any non-member value,
including case variants like `"Normal"` — exact lowercase match per the taxonomy
template's valid list), when boot runs, then startup fails before the watchdog
or QueryEngine is constructed, the error message is the taxonomy template
`Invalid watchdog level: 'extreme'. Valid: normal, restrictive, permissive`
(E-WATCH-001), and the process exits with `EXIT_CONFIG_INVALID` (exit 2) per the
ADR-022 §A config-invalid class via `BootError::ConfigInvalid` — no new exit
codes, no new PrismError variant needed.

Red Gate: `test_BC_2_15_006_invalid_level_e_watch_001_exit_2`

### Group B — Overrides + clamping

**AC-004 — Per-limit overrides take precedence; non-overridden limits keep level defaults**
(traces to BC-2.15.006 postconditions 2–3 + EC-15-021 + Canonical Test Vector "Override wins")

Given `[watchdog] level = "restrictive"` with `query_timeout_seconds = 60`, when
limits resolve, then timeout is 60s while ALL other limits remain the restrictive
defaults (256 MB RSS / 100 MB per-query / 5 calls / 5,000 records) — EC-15-021
verbatim. Each of the four BC override keys (`memory_limit_mb`,
`query_timeout_seconds`, `max_concurrent_api_calls`, `max_materialized_records`)
independently overrides exactly its own limit on any level. There is
deliberately NO override key for the per-query memory budget — it is
profile-set only per the BC's postcondition key list (see EC-009).

Red Gate: `test_BC_2_15_006_override_precedence_per_key_ec_15_021`

**AC-005 — Below-minimum overrides are clamped with an E-WATCH-002 warning**
(traces to BC-2.15.006 error condition E-WATCH-002 + Canonical Test Vector "Clamped override")

Given an override below its safe minimum — `memory_limit_mb = 16` (min 64 MB),
`query_timeout_seconds = 2` (min 5s), or `max_materialized_records = 100`
(min 1000) — when limits resolve, then the value is clamped to the minimum, the
resolution function returns a `ClampWarning` per clamped key, and boot logs each
warning at WARN with the taxonomy template `Watchdog override value below safe
minimum: {param}={value} (minimum {min})` (E-WATCH-002). Boundary: an override
EQUAL to the minimum (e.g., `memory_limit_mb = 64`) is accepted with no warning.
Startup proceeds (E-WATCH-002 is cosmetic, never fatal).

Red Gate: `test_BC_2_15_006_below_minimum_clamped_e_watch_002`

**AC-006 — Invariants mechanized: watchdog cannot be disabled; interval not configurable**
(traces to BC-2.15.006 invariants 1–3)

Given the extended `[watchdog]` section, when any disable-shaped or
interval-shaped key is present (`enabled`, `disable`, `check_interval`,
`check_interval_seconds`, or any other unknown key), then config load fails as
config-invalid (exit 2) — the `WatchdogConfig` struct carries
`#[serde(deny_unknown_fields)]` so no TOML path can disable the watchdog or
change the 3-second interval; and the resolved `EffectiveLimits` type has no
representation for "unlimited" (all fields are finite integers — even permissive
enforces finite bounds, invariant 1). Coordination note: S-WATCHDOG-WIRING-001
introduces the section without `deny_unknown_fields`; adding it here is the
invariant-3 mechanization and is in-scope.

Red Gate: `test_BC_2_15_006_unknown_watchdog_keys_rejected_invariants`

### Group C — Enforcement-point threading

**AC-007 — Resolved limits thread into every enforcement point at boot**
(traces to BC-2.15.006 postcondition 1 "The resource watchdog is initialized with limits based on the configured level")

Given a resolved `EffectiveLimits`, when boot constructs the runtime, then:
(a) `ResourceWatchdog` is constructed with `budget_bytes` = the resolved process
RSS limit in SI bytes (ADR-S2.02-002 convention; 0.70/0.85/0.95 threshold
fractions unchanged); (b) `QueryEngineConfig` at step 8 is built from the
resolved limits — `memory_pool_bytes` (per-query budget, binary-MiB convention
per memory.rs as-built), `timeout_secs`, `max_materialized_records`,
`max_fan_out_concurrency` (= max concurrent API calls per query, BC normal
default 10 matches the existing field default) — replacing
`QueryEngineConfig::default()`; and (c) with `level = "restrictive"` end-to-end,
a query exceeding 15s times out with the existing E-QUERY-004 path (proving the
profile value, not the constant, is enforced). No enforcement point retains a
hardcoded profile value reachable in production.

Red Gate: `test_BC_2_15_006_effective_limits_thread_into_watchdog_and_engine`

### Group D — Observability (BC-2.15.006-owned exposure)

**AC-008 — Startup INFO log carries level + effective (post-override, post-clamp) limits**
(traces to BC-2.15.006 postcondition "Current limits are logged at startup at INFO" + Canonical Test Vector "Happy path")

Given any valid configuration, when boot completes the watchdog sub-step, then
the existing `watchdog.limits_configured` INFO emission (created by
S-WATCHDOG-WIRING-001 AC-001) carries structured fields for the configured
level, all six effective limits (post-override, post-clamp values — a clamped
`memory_limit_mb = 16` logs 64), and which keys were overridden; the BC-2.16.002
catalog row for `watchdog.limits_configured` is updated with the extended field
schema in the SAME commit (SAP-1).

Red Gate: `test_BC_2_15_006_info_log_effective_limits_and_level`

**AC-009 — `watchdog_status` exposes configured level + current effective limits**
(traces to BC-2.15.006 postcondition 5 "The watchdog exposes current limits … via the dedicated `watchdog_status` MCP tool")

Given a booted server with `level = "permissive"` and an override, when the
`watchdog_status` tool (unstubbed by S-WATCHDOG-WIRING-001 AC-013) is invoked,
then the response includes the configured profile name and the six effective
limits actually enforced (matching AC-008's logged values), alongside the
denylist fields that story already exposes — reaching PrismServer via the
existing Arc-DI seam (no prism-storage dependency added to prism-mcp).

Red Gate: `test_BC_2_15_006_watchdog_status_exposes_level_and_limits`

**AC-010 — Per-query resource history recorded and exposed**
(traces to BC-2.15.006 postcondition 5 "… and resource history via the dedicated `watchdog_status` MCP tool"; CAP-024 "before and after each query execution, the watchdog snapshots process RSS and records the delta"; api-surface.md watchdog_status row)

Given executing queries, when each execution completes (success OR error), then
the watchdog records a resource-history entry — query fingerprint, RSS before,
RSS after, delta, completion timestamp — captured via the existing `MemoryProbe`
seam (StaticProbe-testable, SID-1) into a bounded ring buffer (fixed capacity
`RESOURCE_HISTORY_CAPACITY = 256` entries, oldest evicted — derived constant,
see EC-011 flag; BC/CAP specify the history but not its bound); and when
`watchdog_status` is invoked, then the response includes the recorded history
entries. The hook lives on the same execute() wrapper that S-WATCHDOG-WIRING-001
adds for token register/deregister — one lifecycle site, no second wrapper.

Red Gate: `test_BC_2_15_006_resource_history_recorded_bounded_and_exposed`

---

## Red Gate Test Plan

All tests written FAIL-first. External boundaries mocked in-process per SID-1
(StaticProbe for RSS, in-memory/temp config for TOML, mock at the sensor
boundary). One subprocess test (exit-code contract) is justified per the
in-process-vs-subprocess discipline — exit codes require a real process; it
joins the existing `exit_code_contract.rs` pattern.

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_15_006_level_parse_and_absent_defaults_to_normal` | prism-bin | precondition 2; EC-15-023 | unit |
| 2 | `test_BC_2_15_006_profile_defaults_match_bc_table_all_levels` | prism-storage | postcondition 1 table; invariant 2 | unit |
| 3 | `test_BC_2_15_006_invalid_level_e_watch_001_exit_2` | prism-bin | E-WATCH-001; ADR-022 §A | integration (subprocess) |
| 4 | `test_BC_2_15_006_override_precedence_per_key_ec_15_021` | prism-storage | postconditions 2–3; EC-15-021 | unit |
| 5 | `test_BC_2_15_006_below_minimum_clamped_e_watch_002` | prism-storage | E-WATCH-002 + boundary | unit |
| 6 | `test_BC_2_15_006_unknown_watchdog_keys_rejected_invariants` | prism-bin | invariants 1–3 | unit |
| 7 | `test_BC_2_15_006_effective_limits_thread_into_watchdog_and_engine` | prism-bin | postcondition 1 (initialization) | integration |
| 8 | `test_BC_2_15_006_info_log_effective_limits_and_level` | prism-bin | INFO-log postcondition | unit |
| 9 | `test_BC_2_15_006_watchdog_status_exposes_level_and_limits` | prism-mcp | postcondition 5 (limits) | unit |
| 10 | `test_BC_2_15_006_resource_history_recorded_bounded_and_exposed` | prism-query | postcondition 5 (history) | unit |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~7 500 |
| BC files (1 BC): BC-2.15.006 v1.3 (full) | ~1 200 |
| error-taxonomy.md (E-WATCH-001/002 + E-QUERY-004 rows only) | ~600 |
| domain-spec/capabilities.md CAP-024 row | ~1 000 |
| S-WATCHDOG-WIRING-001 (§Current State + AC-001/AC-012/AC-013 + §Out of Scope only) | ~3 500 |
| api-surface.md watchdog_status row | ~300 |
| prism-storage/src/watchdog.rs (full) | ~4 500 |
| prism-query/src/engine.rs (QueryEngineConfig + execute regions only) | ~4 000 |
| prism-query/src/memory.rs (full) | ~1 200 |
| prism-bin/src/boot.rs (step-2 config load, PrismConfig struct, step-8 regions only — NOT full file) | ~6 000 |
| prism-mcp/src/server.rs (watchdog_status region only) | ~2 500 |
| ADR-022 §A exit-code table | ~600 |
| Test stubs (10 × ~50 lines) | ~2 500 |
| Tool outputs (nextest, clippy) | ~3 000 |
| **Total estimate** | **~38 400** |

At ~256k context window this is ~15% — within the 20-30% ceiling, contingent on
the partial-read discipline for boot.rs / engine.rs / server.rs (load only the
listed regions).

---

## Tasks

**Phase 0: Re-baseline (after S-WATCHDOG-WIRING-001 merges)**

- [ ] Run remove-uncertainty against the post-wiring develop baseline: confirm
      the `WatchdogConfig` section shape, the boot watchdog sub-step location,
      the QueryEngine/PrismServer Arc-DI seams, the `watchdog.limits_configured`
      emission, and the watchdog_status snapshot schema all match this story's
      assumptions — adjust §Current State and ACs for drift
- [ ] Story-writer collapses any drift and flips status draft → ready

**Phase 1: Pure resolution core (prism-storage)**

- [ ] Read `watchdog.rs` fully before editing
- [ ] Add `WatchdogProfile` enum (Normal/Restrictive/Permissive — distinct type
      from the RSS-pressure `WatchdogLevel`; do NOT conflate or rename the
      existing enum), `EffectiveLimits`, `WatchdogOverrides`, `ClampWarning`
      (pub types → `#[non_exhaustive]` + ci.yml EXPECTED bump check)
- [ ] Implement `resolve_limits(profile, overrides) -> (EffectiveLimits,
      Vec<ClampWarning>)` — hardcoded BC defaults table, per-key override merge,
      clamp set {memory 64 MB, timeout 5s, records 1000, calls ≥1 (EC-007 flag)}
      (AC-002/AC-004/AC-005)
- [ ] Parameterize `ResourceWatchdog::new`/`with_probe` with `budget_bytes`
      (SI; existing callers updated — TD-VSDD-060 sibling sweep)
- [ ] Add bounded resource-history ring buffer + `record_query_resources(...)` +
      history in `WatchdogStatus` (AC-010)
- [ ] Write Red Gate tests 2, 4, 5 (FAIL first)

**Phase 2: Config schema + boot threading (prism-bin)**

- [ ] Read boot.rs step-2/PrismConfig/step-8 regions before editing
- [ ] Extend `WatchdogConfig` with `level` + 4 override keys; add
      `#[serde(deny_unknown_fields)]`; map invalid level to
      `BootError::ConfigInvalid` carrying the E-WATCH-001 taxonomy template
      (AC-001/AC-003/AC-006)
- [ ] Boot watchdog sub-step: resolve limits, WARN-log each ClampWarning with
      the E-WATCH-002 template, construct `ResourceWatchdog` with resolved
      `budget_bytes`, build `QueryEngineConfig` from `EffectiveLimits` at step 8
      (replacing `Default::default()`), enrich the
      `watchdog.limits_configured` INFO emission (AC-007/AC-008)
- [ ] Write Red Gate tests 1, 3, 6, 7, 8 (FAIL first)

**Phase 3: Execution-path history hooks (prism-query) + MCP exposure (prism-mcp)**

- [ ] RSS before/after snapshots around `QueryEngine::execute` on the same
      lifecycle wrapper as the wiring story's token register/deregister; record
      fingerprint/before/after/delta/timestamp on ALL exit paths (AC-010)
- [ ] Extend the watchdog_status response with profile name, effective limits,
      and resource history via the existing Arc-DI seam (AC-009/AC-010)
- [ ] Write Red Gate tests 9, 10 (FAIL first)

**Phase 4: Closure**

- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — the enriched
      `watchdog.limits_configured` row + any new emission has a same-commit
      BC-2.16.002 row
- [ ] Taxonomy companion touch: E-WATCH-002 resolution column gains "1 call"
      for the derived max_concurrent_api_calls floor (EC-007; flag to
      product-owner for BC-2.15.006 companion ratification — no new codes,
      no renumbering)
- [ ] TD-VSDD-060 sibling sweep: `ResourceWatchdog::new`/`with_probe` callsites,
      `QueryEngineConfig` construction sites, `WatchdogStatus` consumers
- [ ] BC backlink update: BC-2.15.006 Traceability "Stories" row gains
      S-WATCHDOG-CONFIG-PROFILE-001 (alongside S-2.02)
- [ ] `just check` GREEN; `just iter prism-storage` + `just iter prism-bin`
      during inner loop; non-exhaustive compile-fail gate EXPECTED count verified

---

## Previous Story Intelligence

- **S-WATCHDOG-WIRING-001 (direct dependency, unmerged at authoring):** creates
  every seam this story threads through — boot sub-step, `[watchdog]` section,
  Arc-DI into QueryEngine/PrismServer, `watchdog.limits_configured` emission,
  unstubbed watchdog_status. Its §Out of Scope item 1 is this story's charter.
  Its risk_mitigations note the prism-mcp/prism-storage firewall (Arc-DI seam,
  no direct dep) — this story must respect the same boundary when extending the
  status response. Re-verify all assumed shapes in Phase 0 (the wiring story
  itself mandates a remove-uncertainty re-baseline; drift is likely).
- **S-2.02 (merged owner of the watchdog library):** the 512 MB budget is SI
  (512,000,000 bytes), NOT MiB — ADR-S2.02-002 in PR #52; profile values for the
  process budget must use the SI base (256_000_000 / 2_048_000_000). The
  `MemoryProbe`/`StaticProbe` seams exist for exactly this kind of in-process
  RSS testing — reuse, do not invent new seams. `WatchdogLevel` in watchdog.rs
  is the RSS-pressure ladder (Normal/Warn/Throttle/Kill), NOT the config
  profile — the BC's "graduated level" naming collides; this story's new enum
  must be named distinctly (`WatchdogProfile`) to avoid the ColumnType-style
  shadow-type defect class (ADR-024 lesson).
- **QRY-02 / S-WATCHDOG-WIRING-001 defect-class lesson:** constructed-but-
  unreachable code is the project's most recurrent defect class. AC-007's
  end-to-end restrictive-timeout assertion exists so the profile system is
  proven ENFORCED, not merely parsed — verification by behavior, not by
  construction-site inspection.
- **D2 adjudication (taxonomy v1.68→v1.69 lineage):** E-WATCH-* (configuration
  namespace, this story) is disjoint from E-WATCHDOG-* (runtime enforcement
  namespace, BC-2.11.006/BC-2.15.007). Do not cross-cite them: an invalid level
  is E-WATCH-001; a memory kill is E-WATCHDOG-002.
- **PLUGIN-MIGRATION-001-D lessons 16/17/24:** SAP-1 same-commit catalog rows;
  SID-1 — no `#[ignore]`-rationalized deferrals (the exit-code subprocess test
  is the ONE justified subprocess test); read the real types
  (`QueryEngineConfig`, `WatchdogStatus`, `PrismConfig`) before writing tests —
  do not infer shapes from BC prose.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Profile resolution is a PURE function returning warnings as data; boot (effectful shell) owns logging and fatal exits | Pure-core/effectful-I/O boundary (architecture purity discipline) | Red Gate tests 2/4/5 run without tokio or I/O |
| Process-RSS budget in SI bytes (ADR-S2.02-002); per-query pool in binary MiB (memory.rs as-built); profile maps into EACH convention — no homogenization in this story | S-2.02 PR #52 + BC-2.11.006 as-built | Red Gate test 7 pins both conversions |
| `WatchdogProfile` (config) is a DISTINCT type from `WatchdogLevel` (RSS pressure) — no rename, no conflation | ADR-024 shadow-type lesson | Adversary probe + code review |
| Level defaults hardcoded in the resolution function; overridable, never removable | BC-2.15.006 invariant 2 | Red Gate test 2 |
| Check interval fixed at 3s; not present in `EffectiveLimits` or `WatchdogConfig` | BC-2.15.006 invariant 3 | Red Gate test 6 |
| `#[serde(deny_unknown_fields)]` on `WatchdogConfig`; invalid level → `BootError::ConfigInvalid` → exit 2 — no new exit codes | ADR-022 §A; BC-2.06.011 | Red Gate tests 3/6 |
| Only taxonomy error codes used: E-WATCH-001, E-WATCH-002 (config namespace) — never E-WATCHDOG-* for config conditions, no invented codes | error-taxonomy.md v1.69 + CLAUDE.md | Red Gate tests 3/5 + adversary |
| Exact taxonomy message templates for E-WATCH-001/002 (verbatim per ADR-035 canonical-row convention) | error-taxonomy.md rows 365–366 | Red Gate tests 3/5 assert message text |
| New pub types in prism-storage get `#[non_exhaustive]` + ci.yml EXPECTED bump | CLAUDE.md non-exhaustive discipline | Compile-fail gate `tests/external/non-exhaustive-violation/` |
| `watchdog.limits_configured` field-schema change + any new emission → same-commit BC-2.16.002 rows | SAP-1 | Adversary SAP-1 probe |
| Resource-history hooks use the existing `MemoryProbe` seam; bounded ring buffer; no probe call under lock on the hot path | S-2.02 seam discipline + risk_mitigations | Red Gate test 10 + adversary |
| prism-mcp gains NO prism-storage dependency (extend via the existing Arc-DI seam) | S-WATCHDOG-WIRING-001 Forbidden Dependencies | Cargo.toml review + adversary |
| No `unwrap()`/`expect()` on config/resolution paths; failures propagate as structured `BootError`/`PrismError` | CLAUDE.md error handling | Clippy + adversary |
| Spec content cites function names + behavioral anchors, not line numbers | TD-VSDD-091 | This story complies |

---

## Library & Framework Requirements

Versions pinned from the workspace `Cargo.toml`. No NEW external dependencies.

| Crate | Version | Usage |
|-------|---------|-------|
| `serde` | workspace-pinned | `WatchdogConfig` section deserialization (deny_unknown_fields) |
| `toml` | workspace-pinned | Already used by `step2_load_config`; no new parse path |
| `sysinfo` | workspace-pinned (prism-storage `watchdog` feature) | RSS probe for resource history (existing) |
| `tracing` | workspace-pinned | E-WATCH-002 WARN + enriched limits INFO (BC-2.16.002 rows required) |
| `tokio` | `1` (multi-threaded) | Existing execute() wrapper; no new runtime surface |

**Forbidden patterns:**
- Do NOT add a `watchdog.check_interval*` or `watchdog.enabled` config key (invariants 1/3)
- Do NOT add an override key for the per-query memory budget (BC postcondition key list is exhaustive; see EC-009)
- Do NOT reuse or rename `WatchdogLevel` for the config profile (shadow-type hazard)
- Do NOT emit E-WATCHDOG-001/002 for configuration conditions (namespace split per taxonomy)

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-storage/src/watchdog.rs` (or new sibling module `crates/prism-storage/src/watchdog_profile.rs`) | MODIFY/CREATE | `WatchdogProfile`, `EffectiveLimits`, `WatchdogOverrides`, `ClampWarning`, `resolve_limits` (pure); `ResourceWatchdog` budget parameterization; bounded resource-history buffer + `WatchdogStatus` extension |
| `crates/prism-bin/src/boot.rs` | MODIFY | `WatchdogConfig` section extension (level + 4 override keys, deny_unknown_fields); E-WATCH-001 ConfigInvalid mapping; resolve + clamp-warn at the watchdog sub-step; `QueryEngineConfig` built from `EffectiveLimits` at step 8; enriched limits INFO log |
| `crates/prism-query/src/engine.rs` | MODIFY | Resource-history snapshot hooks on the execute() lifecycle wrapper (probe before/after, record on all exit paths) |
| `crates/prism-mcp/src/server.rs` | MODIFY | watchdog_status response gains profile name + effective limits + resource history (via existing Arc-DI seam) |
| `crates/prism-storage/src/tests/` + `crates/prism-bin/tests/` + prism-query/prism-mcp test modules | MODIFY/CREATE | 10 Red Gate tests per plan (exit-code test joins the existing `exit_code_contract.rs` pattern) |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Same-commit catalog row update for `watchdog.limits_configured` field schema + any new emission (SAP-1) |
| `.factory/specs/prd-supplements/error-taxonomy.md` | MODIFY | E-WATCH-002 resolution column gains "1 call" derived floor (EC-007 companion; no new rows/codes — POL-1 safe) |
| `.factory/specs/behavioral-contracts/BC-2.15.006-*.md` | MODIFY | Traceability "Stories" backlink row gains S-WATCHDOG-CONFIG-PROFILE-001 |
| `tests/external/non-exhaustive-violation/` + `.github/workflows/ci.yml` | MODIFY (only if EXPECTED changes) | `EXPECTED=` bump for new `#[non_exhaustive]` pub types |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.15.006 EC-15-021 | `level: restrictive` + `query_timeout_seconds: 60` | Override wins; timeout 60s; all other limits stay restrictive (AC-004) |
| EC-002 | BC-2.15.006 EC-15-022 | System has less RAM than the configured memory limit (e.g., permissive 2048 MB on a 1 GB host) | Watchdog still monitors against the configured budget; OS may OOM-kill before the watchdog triggers — no special-casing, no host-RAM probe at config time |
| EC-003 | BC-2.15.006 EC-15-023 | No `[watchdog]` section in prism.toml | Normal level, all default limits; byte-identical engine config to pre-story `Default::default()` (AC-001) |
| EC-004 | E-WATCH-001 | `level: "Normal"` (case variant) | E-WATCH-001 fatal — valid set is exact lowercase {normal, restrictive, permissive} per the taxonomy template (AC-003) |
| EC-005 | E-WATCH-002 | Override exactly AT the minimum (`memory_limit_mb: 64`) | Accepted, no warning — clamp triggers strictly below the minimum (AC-005) |
| EC-006 | E-WATCH-002 | Multiple below-minimum overrides in one config | One E-WATCH-002 WARN per clamped key; all clamped; startup proceeds |
| EC-007 | This story (derived floor — FLAGGED) | `max_concurrent_api_calls: 0` | Clamped to 1 with E-WATCH-002 WARN. DERIVED minimum: BC-2.15.006/taxonomy enumerate only three minimums (64 MB / 5s / 1000 records); a floor of 1 follows from invariant 1 (always-active finite bounds — 0 permits would deadlock every fan-out, a de-facto disable). Taxonomy E-WATCH-002 resolution column gains "1 call" in the same commit; flagged to product-owner for BC-2.15.006 companion ratification during BC anchoring. NOT a new error code. |
| EC-008 | Invariant 3 | `check_interval_seconds: 10` (or `enabled: false`) in `[watchdog]` | Config-invalid (exit 2) via deny_unknown_fields — interval and existence are not configurable (AC-006) |
| EC-009 | BC postcondition 2 key list | `query_memory_budget_mb: 300` in `[watchdog]` | Config-invalid (exit 2): the per-query budget has NO override key per the BC's exhaustive key list — profile-set only. Rejected by deny_unknown_fields, not silently ignored |
| EC-010 | This story | Override ABOVE the profile default (e.g., restrictive + `memory_limit_mb: 1024`) | Honored — clamping is a floor only; the BC defines no ceilings. The value remains finite (invariant 1 holds) |
| EC-011 | This story (derived bound — FLAGGED) | Resource history under sustained load | Ring buffer holds the most recent `RESOURCE_HISTORY_CAPACITY = 256` entries, oldest evicted, O(1) push — DERIVED constant: BC/CAP-024 specify per-query history but no bound; unbounded growth would violate the watchdog's own memory-bounding purpose. Documented at the constant; flagged to product-owner alongside EC-007 |
| EC-012 | This story | Query terminated by watchdog/timeout (error exit path) | Resource-history entry still recorded (before/after/delta with the at-termination RSS) — history covers ALL exit paths, matching the wiring story's drop-guard lifecycle (AC-010) |
| EC-013 | This story | `[watchdog]` section present with ONLY denylist keys (S-WATCHDOG-WIRING-001 vintage config) | Valid: level defaults to normal; denylist keys parse per the wiring story; zero migration needed for existing configs |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `WatchdogProfile` + `EffectiveLimits` + `resolve_limits` (defaults table, override merge, clamping) | `prism-storage` watchdog module | Pure | BC-2.15.006 postconditions 1–3, E-WATCH-002 |
| `WatchdogConfig` TOML section (level + override keys, deny_unknown_fields) | `prism-bin/src/boot.rs` (PrismConfig) | Effectful (config I/O at step 2) | BC-2.15.006 precondition 2; invariants 1/3 |
| E-WATCH-001 fatal mapping; E-WATCH-002 WARN logging | `prism-bin/src/boot.rs` (`step2_load_config` + watchdog sub-step) | Effectful | BC-2.15.006 error conditions; ADR-022 §A |
| Enforcement-point threading (watchdog budget_bytes; QueryEngineConfig from EffectiveLimits) | `prism-bin/src/boot.rs` (watchdog sub-step + step 8) | Effectful | BC-2.15.006 postcondition 1 |
| Per-query RSS-delta snapshot hooks | `prism-query/src/engine.rs` (execute lifecycle wrapper) | Effectful (probe read) | BC-2.15.006 postcondition 5; CAP-024 |
| Bounded resource-history buffer + status extension | `prism-storage` watchdog module | Effectful (shared state) | BC-2.15.006 postcondition 5 |
| watchdog_status response extension (level/limits/history) | `prism-mcp/src/server.rs` | Effectful (tool layer) | BC-2.15.006 postcondition 5 |

---

## Forbidden Dependencies

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-mcp` | `prism-storage` | MCP layer stays storage-agnostic; level/limits/history reach PrismServer via the Arc-DI seam S-WATCHDOG-WIRING-001 wires. If prism-mcp gains a prism-storage dependency, the build review MUST fail it. |
| `prism-storage` | `prism-query`, `prism-bin` | Resolution core is a leaf domain library; config parsing stays in prism-bin, fingerprints stay in prism-query. A reverse dependency would cycle the graph. |
| `prism-core` | (no new deps) | E-WATCH-001/002 need no new PrismError variants — E-WATCH-001 rides BootError::ConfigInvalid; E-WATCH-002 is a WARN log, not an error value |

---

## SAP-1 Compliance (Structured Event Catalog)

Expected emission changes (implementer must enumerate actual sites):
- `event_type = "watchdog.limits_configured"` (EXISTING row from
  S-WATCHDOG-WIRING-001) — field schema EXTENDED: level, six effective limits,
  overridden-keys list (AC-008)
- `event_type = "watchdog.override_clamped"` (NEW, if implemented as a
  structured emission rather than a plain WARN with the E-WATCH-002 template) —
  one per clamped key (AC-005)

Each added/changed emission gets its BC-2.16.002 catalog row (field schema,
audit role, recurrence policy) in the SAME commit. If the clamp warning ships as
the bare taxonomy-template WARN without an `event_type` field, state so
explicitly in the PR description (D-765 precedent).

---

## Out of Scope (explicit routing flags — NOT silent deferrals)

1. **BC-2.15.007/BC-2.15.008 production wiring** (boot construction, token
   lifecycle, denylist gate, force_execute, watchdog_status unstub): owned by
   S-WATCHDOG-WIRING-001 (this story's direct dependency). This story only
   parameterizes and extends those surfaces.
2. **Hot-reload of `[watchdog]` changes** (AD-007 arc-swap config reload):
   BC-2.15.006 is an INITIALIZATION contract ("at startup"); limit changes take
   effect on restart. If the hot-reload story family (S-1.12 lineage) later
   wants live watchdog re-profiling, that is a NEW capability requiring a BC
   amendment — routed to product-owner, not silently absorbed here.
3. **CPU-time limits**: the BC title says "Memory/CPU/Timeout" but the BC's
   postcondition table defines no CPU-specific limit row (the timeout is the
   CPU-bounding mechanism per CAP-024's osquery-derived model). No CPU
   percentage knob is invented here; if one is wanted, that is a BC amendment
   (product-owner).

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-06-10 | Initial authoring per orchestrator story-writer micro-burst (2026-06-10): anchors the BC-2.15.006 implementation gap surfaced by the S-WATCHDOG-WIRING-001 authoring burst (§Out of Scope item 1 routing recommendation accepted — second POL-15-class gap on the watchdog subsystem; graduated profiles, TOML overrides, clamping, E-WATCH-001/002 never implemented by merged owner S-2.02). Taxonomy verification: E-WATCH-001/002 both present in error-taxonomy.md v1.69 (rows match BC-2.15.006 exactly; no new codes needed). 10 ACs (profile parse/defaults/E-WATCH-001 exit-2, override precedence/clamping/invariant mechanization, enforcement-point threading, INFO log + watchdog_status level/limits + resource history); 10 Red Gate tests; 8 pts. Two derived constants flagged for product-owner ratification: max_concurrent_api_calls floor of 1 (EC-007, taxonomy E-WATCH-002 resolution companion touch) and RESOURCE_HISTORY_CAPACITY=256 (EC-011). depends_on S-WATCHDOG-WIRING-001 (REAL build-order: profiles configure a watchdog that must exist); post-demo backlog sequencing inherited transitively. verification_properties deliberately empty per BC-2.15.006 ratified VP table "(none)". |
