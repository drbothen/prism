---
document_type: story
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
title: "Scenario Progression + Enrichment Correlation — Unfolding-Attack Live Demo"
wave: 5
epic_id: E-DEMO
priority: P2
status: ready
version: "2.17"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-09"
modified: "2026-06-13T12:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates including prism-dtu-demo-server,
#   prism-dtu-common, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty,
#   prism-dtu-cyberint, prism-dtu-threatintel, and prism-dtu-nvd per ARCH-INDEX Subsystem
#   Registry. The scenario progression engine is demo infrastructure entirely within SS-01.
#   Decision anchor: ADR-036 v2.2 subsystems_affected: [SS-01].
target_module: prism-dtu-common
crates_touched: [prism-dtu-common, prism-dtu-demo-server, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-threatintel, prism-dtu-nvd]
behavioral_contracts: [BC-2.06.019, BC-2.06.020]
verification_properties: [VP-019-A, VP-019-B, VP-019-C, VP-019-D, VP-019-E, VP-019-F, VP-019-G, VP-019-H, VP-019-I, VP-020-A, VP-020-B, VP-020-C, VP-020-D, VP-020-E, VP-020-F, VP-020-G, VP-020-H, VP-020-I, VP-020-J, VP-020-K, VP-020-L]
depends_on:
  - S-DEMO-DTU-LIVE-SCENARIO-001-A
  # Dependency anchor: Story A delivers new_with_seed constructors + generated_records in state
  # for all 4 generator-backed clones. Story B's new_with_scenario constructors project a
  # StageMask over the generated_records introduced by Story A. Without generated_records in
  # the state struct (Story A scope), there is no substrate to project over. This is a hard
  # build-order dependency: Story B cannot compile without Story A's state field additions.
blocks: []
points: 7
# Points justification (ADR-036 v2.2 §8 Story B estimate):
#   1. IncidentTimeline, IncidentStage, StageMask, current_stage_index pure fn in
#      prism-dtu-common/src/scenario/ (extends Story A stub module): 1.5 pts
#   2. Per-clone new_with_scenario constructors (CrowdStrike, Armis, Claroty, Cyberint):
#      adds timeline: Option<Arc<IncidentTimeline>> to state; builds on Story A's new_with_seed
#      substrate: 1.5 pts
#   3. Route handler stage-mask projection (4 clones × 1-2 routes each): 1.5 pts
#   4. ThreatIntelClone::new_with_scenario (infallible) + NvdClone::new_with_scenario
#      (fallible): 0.5 pts each = 1 pt
#   5. build_clone_pairs scenario coordination (catalog derivation, Arc<IncidentTimeline>
#      threading, E-DEMO-002/003): 0.5 pts
#   6. Red Gate test suite (~16 tests, FAIL-first): 1 pt
#   Total: 7 pts
estimated_days: 3
risk: HIGH
# Risk justification:
#   Depends on Story A substrate being in place. IncidentTimeline threading via Arc to 4
#   operational clones adds complexity to build_clone_pairs. Stage-mask filtering must be
#   a pure function with no shared mutable state (ADR-036 §3.1 — no background mutator).
#   Cross-DTU entity coherence requires ScenarioEntityCatalog to produce the same primary
#   device IDs for the same (seed, org_id) regardless of which clone queries it.
#   NvdClone::new_with_scenario is fallible (like NvdClone::new()). CVSS path is
#   CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score (f64), NOT a flat field.
#   stage_duration_secs array has 4 entries (for stages 1-4 activation thresholds);
#   stage 0 always activates at 0 — no array entry needed.
acceptance_criteria_count: 19
red_gate_tests: 23
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "stage_duration_secs 4-entry array: [60, 180, 360, 600] for stages 1-4; stage 0 always at 0. BC-2.06.019 §Postcondition 2 and ADR-036 v2.3 §2.2 are authoritative. Any test using 5-entry arrays or different thresholds is wrong."
  - "current_stage_index is a pure function: no side effects, no shared mutable state, no tokio::spawn, no Arc<AtomicU64> progression counter. ADR-036 v2.3 §2.1 mandates this; Architecture Compliance Rules section is binding."
  - "NvdClone::new_with_scenario returns anyhow::Result<Self> (fallible, like NvdClone::new()). ArmisClone::new_with_scenario returns anyhow::Result<Self> (fallible — new_with_seed_anchored is fallible for Armis). CyberintClone::new_with_scenario returns anyhow::Result<Self> (fallible, like CyberintClone::new()). ThreatIntelClone::new_with_scenario is infallible (like ThreatIntelClone::new()). CrowdstrikeClone::new_with_scenario and ClarotyClone::new_with_scenario are infallible (-> Self). Tests must handle Result for NVD, Armis, and Cyberint."
  - "CVSS path is CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score (f64) >= 7.0, NOT metrics.score or any flat field. cvss_metric_v31 is Option<Vec<CvssMetricV31>> — test code must unwrap/as_ref the Option (pre-flight task at ~line 404 already documents this). Implementer MUST read crates/prism-dtu-nvd/src/types.rs before writing the constructor."
  - "Cross-DTU entity coherence: primary_device_id_cs and primary_device_id_armis in ScenarioEntityCatalog use the same org_slug derivation (hex of org_id.as_bytes()[0..4]). They MUST match across Armis and CrowdStrike queries. ADR-036 v2.3 §2.2."
  - "Secondary RNG stream independence (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001): seeded_rng(seed.wrapping_add(1), org_id) for catalog derivation must be a SEPARATE ChaCha20Rng instance from the primary generator stream. Implementing build_clone_pairs must NOT advance the primary stream before catalog derivation."
  - "StageMask must NOT carry #[non_exhaustive] — it is internal to prism-dtu-common and must be exhaustively constructible within the crate (BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001). NOTE: ADR-036 v2.3 §2.2 code snippet erroneously shows StageMask as #[non_exhaustive]; BC-2.06.019 wins on contract semantics per CLAUDE.md Source-of-Truth Precedence."
  - "#[non_exhaustive] EXPECTED bump: new pub types IncidentTimeline, IncidentStage (minimum +2) added in this story; implementer must read live EXPECTED= from ci.yml and increment by exact new-type count (ADR-036 v2.3 §2.5: 49 was pre-Story-A baseline; Story A incremented for ScenarioEntityCatalog; implementer reads current ci.yml value before incrementing)."
  - "reqwest::Client timeout: .timeout(Duration::from_secs(30)) in all new integration test HTTP clients per CLAUDE.md conventions."
  - "INV-PERIMETER-001: ThreatIntel and NVD new_with_scenario constructors must not import prism-spec-engine/prism-sensors/prism-query. prism-dtu-common dep (fixture-gen feature) added by Story A; no new cross-DTU perimeter changes needed beyond adding fixture-gen feature to prism-dtu-threatintel/Cargo.toml and prism-dtu-nvd/Cargo.toml."
  - "NIT-1 E-DEMO-004 trigger reconciliation: Story A fires E-DEMO-004 when new_with_seed is called for a non-default fixture_set + missing org_id. Story B adds the scenario.enabled path which also requires org_id. The E-DEMO-004 guard already present from Story A (guards org_id absent before any constructor) covers both trigger cases. The message 'scenario.enabled requires org_id' in the error-taxonomy is accurate for the scenario-enabled path; Story B does NOT need to add a separate guard — it inherits the Story A guard and the message is correct."
  - "NIT-2 ScenarioConfig fields (enabled/archetype/scenario_start_secs/stage_duration_secs) were deserialized in Story A but unconsumed. Story B is the sole consumer of all four fields — this is the core implementation scope of this story."
  - "time_anchor wiring (ADR-036 v2.3 §2.3): the 3-arg new_with_seed anchors generated record timestamps at demo_time_anchor() = 2026-01-01, which is stale for a June 2026 demo. Story B's new_with_scenario MUST internally call the 4-arg new_with_seed_anchored(seed, archetype, org_id, time_anchor) (NOT new_with_seed), then set timeline = Some(Arc::clone(&timeline)). time_anchor is passed in from build_clone_pairs (derived ONCE from scenario_start_epoch_secs via DateTime::from_timestamp). When scenario_start_secs = None, Utc::now() is called ONCE and used for BOTH scenario_start_epoch_secs and time_anchor — do NOT call Utc::now() twice."
  - "5-arg new_with_scenario signature (ADR-036 v2.3 §2.4): new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>). Per-clone return types: CrowdstrikeClone -> Self (infallible); ClarotyClone -> Self (infallible); ArmisClone -> anyhow::Result<Self> (fallible — new_with_seed_anchored is fallible for Armis); CyberintClone -> anyhow::Result<Self> (fallible). ThreatIntelClone::new_with_scenario(entities) -> Self (infallible, different signature). NvdClone::new_with_scenario(entities) -> anyhow::Result<Self> (fallible). The 4-arg form without time_anchor is FORBIDDEN in the scenario-enabled path — it would produce dead code for new_with_seed_anchored and stale record timestamps."
traces_to: [D-1077, D-1090, ADR-036]
supersedes: []
---

# S-DEMO-DTU-LIVE-SCENARIO-001-B: Scenario Progression + Enrichment Correlation

Add the `IncidentTimeline` temporal layer on top of Story A's seeded generator substrate.
Implements BC-2.06.019 (pure-function-of-time stage engine with 5 stages) and BC-2.06.020
(ThreatIntel IOC injection + NVD CVE injection from the shared `ScenarioEntityCatalog`).
Together with Story A, this delivers the complete multi-client SOC demo live-scenario layer.

**ADR-036 v2.3 amendment (time_anchor wiring):** `new_with_scenario` for generator-backed clones
MUST internally call `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` (4-arg), NOT
the 3-arg `new_with_seed` (which anchors at `demo_time_anchor()` = 2026-01-01). The 5-arg
constructor signature is `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>)`.
`time_anchor` is derived ONCE at `build_clone_pairs` from `scenario_start_epoch_secs` via
`DateTime::from_timestamp(scenario_start_epoch_secs, 0)`. This ensures generated record
timestamps are era-coherent with the scenario clock (a June 2026 demo gets June 2026 timestamps).

**Depends on:** S-DEMO-DTU-LIVE-SCENARIO-001-A (merged PR #181 develop@c287b00d — SATISFIED).
Story A delivered: `new_with_seed` constructors + `generated_records`/`generated_devices`/
`generated_detections` in state for all 4 generator-backed clones; `ScenarioEntityCatalog`,
`org_slug_from_org_id`, `build_scenario_entity_catalog` in `prism-dtu-common/src/scenario/`;
`ScenarioConfig` deserialized in `CloneConfig` (fields consumed in Story B). Status is
`ready` — Story A is merged; this story is fully specifiable against the delivered substrate.

**NIT-1 (E-DEMO-004 reconciliation):** Story A fires E-DEMO-004 when `new_with_seed` is
called for a non-default fixture_set archetype + missing `org_id`. Story B's `scenario.enabled`
path also requires `org_id`. The Story A guard already covers the scenario-enabled path: when
`scenario.enabled = true`, `build_clone_pairs` will attempt `new_with_scenario(seed, archetype,
org_id, timeline)`, which requires `org_id` — the existing E-DEMO-004 guard fires correctly.
No message change needed; the error-taxonomy message "scenario.enabled requires org_id" is
accurate for this path. Implementer must verify the guard is hit before any clone constructor
is called in the scenario path (not just the non-default fixture_set path).

**NIT-2 (ScenarioConfig consumption):** Story A deserialized all four `ScenarioConfig` fields
(`enabled`, `archetype`, `scenario_start_secs`, `stage_duration_secs`) but left them unconsumed.
Story B consumes all four: `enabled` gates scenario vs. static path; `archetype` is validated
and mapped to E-DEMO-003 on unrecognized value; `scenario_start_secs` sets
`IncidentTimeline.scenario_start_epoch_secs`; `stage_duration_secs` provides operator-override
thresholds (defaulting to [60, 180, 360, 600] when empty). All four fields must be read from
`CloneConfig.scenario` in `build_clone_pairs` — no field may remain unread after Story B merges.

---

## Narrative

As a SOC analyst running a multi-client live demo, I want each demo client's incident to
unfold across all six DTU clones in a reproducible temporal sequence — Baseline, Recon (60s),
LateralMovement (180s), Exfil (360s), Containment (600s) — so that I can show a complete
attack lifecycle that tells a coherent story a prospect immediately understands.

**Goal:** Given `scenario.enabled = true`, a shared `scenario_start_secs`, and `seed` + `org_id`
in `CloneConfig`, every HTTP request to any operational DTU reflects the current stage
deterministically. The same `scenario_start_secs` + same elapsed time → same stage → same
data. Enrichment DTUs (ThreatIntel, NVD) resolve scenario IOCs and CVEs from construction
time, always ready regardless of which stage the operational DTUs are at. Two clients with
different seeds produce coherent but disjoint entity catalogs (INV-CROSS-DTU-ENTITY-COHERENCE-001).

---

## Behavioral Contracts

| BC | Title | Key Invariants |
|----|-------|----------------|
| BC-2.06.019 v1.7 | Demo-Server Scenario Progression — Pure-Function Temporal Stage Advancement | INV-PROGRESSION-REPRODUCIBILITY-001, INV-STAGE-MONOTONICITY-001, INV-STAGE-MASK-COMPLETENESS-001, INV-SCENARIO-DISABLED-COMPAT-001, INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 |
| BC-2.06.020 v1.6 | Demo-Server Enrichment Correlation — Scenario IOCs/CVEs Resolve in ThreatIntel/NVD; Cyberint Alert CVEs Use Catalog IDs (Collision-Safe in All Modes) | INV-THREATINTEL-IOC-CORRELATION-001, INV-NVD-CVE-CORRELATION-001, INV-CYBERINT-ALERT-CVE-CORRELATION-001, INV-CROSS-DTU-ENTITY-COHERENCE-001, INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001, INV-PERIMETER-COMPLIANCE-001, INV-CONSTRUCTION-TIME-INJECTION-001 |

---

## Acceptance Criteria

### Group A — Scenario Progression Types (BC-2.06.019)

**AC-001 — IncidentTimeline + IncidentStage + StageMask types defined in prism-dtu-common**
(traces to BC-2.06.019 precondition 3 and ADR-036 v2.2 §2.2)

Given `prism-dtu-common` compiled with `feature = "fixture-gen"`,
when the types `IncidentTimeline`, `IncidentStage`, and `StageMask` are imported,
then:
- `IncidentTimeline` is `#[non_exhaustive]`, `#[derive(Clone, Debug)]`, with fields `entities: ScenarioEntityCatalog`, `stages: Vec<IncidentStage>`, `scenario_start_epoch_secs: i64`
- `IncidentStage` is `#[non_exhaustive]`, `#[derive(Clone, Debug)]`, with fields `name: &'static str`, `activates_after_secs: u64`, `visible_entity_mask: StageMask`
- `StageMask` is NOT `#[non_exhaustive]` (internal struct, exhaustively constructible within the crate per INV-STAGE-MASK-COMPLETENESS-001), `#[derive(Clone, Debug)]`, with 6 bool fields: `primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`
- Default `CompromisedEndpoint` timeline has 5 stages: Baseline (0s), Recon (60s), LateralMovement (180s), Exfil (360s), Containment (600s); `stage_duration_secs` config array has exactly 4 entries (one per non-zero stage activation threshold; stage 0 is always 0s and has no array entry)
- NOTE: ADR-036 v2.2 §2.2 code snippet erroneously marks `StageMask` as `#[non_exhaustive]`; BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 is authoritative and StageMask must NOT be `#[non_exhaustive]`

Red Gate: `test_BC_2_06_019_timeline_types_non_exhaustive_and_structure`

**AC-002 — current_stage_index is a pure function of (timeline, now_epoch_secs)**
(traces to BC-2.06.019 invariant INV-PROGRESSION-REPRODUCIBILITY-001 and postcondition 3)

Given a `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize` function,
when called multiple times with the same `(timeline, now_epoch_secs)` pair from any number of concurrent callers,
then all invocations return the same `usize` with no shared mutable state, no locks, no tokio spawn, and no side effects.

The implementation MUST match the ADR-036 v2.2 §2.2 formula verbatim:
```
elapsed = (now_epoch_secs - timeline.scenario_start_epoch_secs).max(0) as u64;
stage = last index i where timeline.stages[i].activates_after_secs <= elapsed
```

Red Gate: `test_BC_2_06_019_stage_index_pure_function_reproducible`

**AC-003 — Stage boundary correctness: 5 stages at default thresholds**
(traces to BC-2.06.019 postcondition 2 and postcondition 3)

Given `scenario_start_epoch_secs = T` and default `CompromisedEndpoint` stage thresholds `[60, 180, 360, 600]` (4-entry `stage_duration_secs` array, canonical per BC-2.06.019 §Postcondition 2 table):
- `now = T + 0s` → stage 0 (Baseline; `activates_after_secs = 0`)
- `now = T + 30s` → stage 0 (Baseline; elapsed 30 < 60)
- `now = T + 90s` → stage 1 (Recon; elapsed 90 >= 60)
- `now = T + 200s` → stage 2 (LateralMovement; elapsed 200 >= 180)
- `now = T + 400s` → stage 3 (Exfil; elapsed 400 >= 360)
- `now = T + 700s` → stage 4 (Containment; elapsed 700 >= 600; saturates at max stage)

Test vectors source: BC-2.06.019 TV-019-001 through TV-019-005.

Red Gate: `test_BC_2_06_019_stage_boundary_5_thresholds_correct`

**AC-004 — INV-STAGE-MONOTONICITY-001: stage index never decreases over increasing time**
(traces to BC-2.06.019 invariant INV-STAGE-MONOTONICITY-001)

Given a timeline with default thresholds, when `current_stage_index` is evaluated at a monotonically increasing sequence of `now_epoch_secs` values spanning all stage boundaries, then the returned stage index is non-decreasing across the entire sequence.

Red Gate: `test_BC_2_06_019_stage_index_monotonic_over_time`

**AC-005 — Clock-skew / future start: elapsed clamped to 0 returns stage 0**
(traces to BC-2.06.019 edge case EC-019-003)

Given `now_epoch_secs < scenario_start_epoch_secs` (clock skew or future start),
when `current_stage_index` is called,
then it returns stage 0 without panic; `elapsed = max(0, now - start)` clamps negative to 0.

Test vector source: BC-2.06.019 TV-019-006.

Red Gate: `test_BC_2_06_019_clock_skew_clamped_to_baseline`

**AC-006 — INV-STAGE-MASK-COMPLETENESS-001: all 6 StageMask fields explicitly set in every stage**
(traces to BC-2.06.019 invariant INV-STAGE-MASK-COMPLETENESS-001)

Given the default `CompromisedEndpoint` `IncidentTimeline` (5 stages),
when each `IncidentStage.visible_entity_mask` is inspected,
then every stage has explicit bool values for all 6 fields (`primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`) — no field is left uninitialized or implicitly defaulted.

The expected per-stage mask values (authoritative: BC-2.06.019 §Postcondition 2 table):
- Stage 0 (Baseline): `primary_device=true`; all others `false`
- Stage 1 (Recon): `primary_device=true`; `lateral_devices=false`; IOC/CVE `false`
- Stage 2 (LateralMovement): `primary_device=true`; `lateral_devices=true`; `ioc_hashes=true`; IP/domain/CVE `false`
- Stage 3 (Exfil): `primary_device=true`; `lateral_devices=true`; `ioc_ips=true`; `ioc_domains=true`; `ioc_hashes=true`; `device_cves=false`
- Stage 4 (Containment): all 6 fields `true`

Red Gate: `test_BC_2_06_019_stage_mask_completeness_all_6_fields`

**AC-007 — Armis new_with_scenario: primary_device not visible at stage 0; visible at stage 1+**
(traces to BC-2.06.019 postcondition 4 and TV-019-009, TV-019-010)

Given an Armis clone constructed with `ArmisClone::new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor)` (5-arg fallible, `-> anyhow::Result<Self>`, ADR-036 v2.3 §2.4) and `scenario_start_secs = T`:
- At `now = T + 30s` (stage 0 / Baseline): `GET /api/v1/devices` response does NOT contain `catalog.primary_device_id_armis`
- At `now = T + 90s` (stage 1 / Recon): `GET /api/v1/devices` response CONTAINS `catalog.primary_device_id_armis`; lateral device IDs are NOT present

The route handler acquires `Arc<IncidentTimeline>` from state, calls `current_stage_index(&timeline, Utc::now().timestamp())`, retrieves `StageMask` for the computed index, and filters `generated_records` by mask. No re-generation occurs per request.

Red Gate: `test_BC_2_06_019_armis_primary_device_stage_visibility`

**AC-008 — CrowdStrike new_with_scenario: containment_status = "contained" only at stage 4**
(traces to BC-2.06.019 postcondition 4 and TV-019-011)

Given a CrowdStrike clone with `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor)` (5-arg, ADR-036 v2.3 §2.4) and `scenario_start_secs = T`:
- At `now = T + 200s` (stage 2 / LateralMovement): the device record for `primary_device_id_cs` shows `containment_status = "normal"` (or equivalent non-contained value)
- At `now = T + 700s` (stage 4 / Containment): the same device record shows `containment_status = "contained"`

The pre-built `FixtureSet` record already carries `containment_status = "contained"` for the CompromisedEndpoint primary device; the Containment stage mask makes it visible.

Red Gate: `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only`

**AC-009 — E-DEMO-002: mismatched seeds across scenario-enabled clones rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-002 and precondition 5)

Given `clones.crowdstrike.seed = 100` and `clones.armis.seed = 200`, both with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-002"` and both clone names and seed values, before any clone constructor is called.

Verbatim message format (error-taxonomy.md E-DEMO-002):
`"demo-server: E-DEMO-002: scenario clones '{clone_a}' (seed={seed_a}) and '{clone_b}' (seed={seed_b}) have different seeds; cross-DTU coherence requires all scenario-enabled clones to share the same seed"`

Red Gate: `test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones`

**AC-010 — E-DEMO-003: unrecognized scenario archetype rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-003 and precondition 7)

Given `scenario.archetype = "unknown_archetype_value"` for any clone with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` and the clone name and the invalid archetype string.

Also applies when `stage_duration_secs` has wrong length: `"demo-server: E-DEMO-003: clone '{clone_name}': stage_duration_secs has {provided} entries but archetype '{archetype}' requires exactly {expected}"` (CompromisedEndpoint requires exactly 4 entries).

Both E-DEMO-003 variants fire before any clone constructor is called.

Red Gate: `test_BC_2_06_019_e_demo_003_unrecognized_archetype`

**AC-011 — INV-SCENARIO-DISABLED-COMPAT-001: scenario.enabled=false is byte-identical to BC-2.06.018 seeded path**
(traces to BC-2.06.019 invariant INV-SCENARIO-DISABLED-COMPAT-001 and TV-019-007)

Given a clone constructed with `scenario.enabled = false` (or absent `[clones.*.scenario]` block) and `seed = 42`, `fixture_set = "default"`, when queried at any fixed request path, then responses are byte-identical to the Story A `new_with_seed(42, HealthyOtEnvironment, default_org)` responses; `timeline: Option<Arc<IncidentTimeline>>` is `None` in the clone state.

NOTE on determinism (ADR-036 v2.3 §2.3): for the scenario-enabled path, determinism means fixed `(seed, org_id, scenario_start_secs)` config inputs → identical output across runs. When `scenario_start_secs = None`, `Utc::now()` is called ONCE at `build_clone_pairs` entry and used for both `scenario_start_epoch_secs` and `time_anchor` — this path is deterministic per-run but NOT cross-run (expected behavior). The `scenario.enabled = false` path delegates to `new_with_seed` which internally calls `new_with_seed_anchored(demo_time_anchor())` — this IS cross-run deterministic (static 2026-01-01 anchor), which is correct for static-snapshot use where no operator `scenario_start_secs` exists.

Red Gate: `test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path`

**AC-012 — INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001: catalog derivation does not shift generator output**
(traces to BC-2.06.019 invariant INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 and postcondition 1)

Given two harness constructions with same `seed = 100, org_id = "<uuid>"`: one with `scenario.enabled = false` and one with `scenario.enabled = true`, when both are queried at the same device endpoint, then the underlying `FixtureSet` device records (from Story A `generated_records`) are byte-identical — the catalog derivation via `gen_seeded_rng(seed.wrapping_add(1), &org_id)` (two-arg re-export; ADR-036 v2.1 U-A-01) has NOT consumed state from the primary generator stream `gen_seeded_rng(seed, &org_id)`.

NOTE: the catalog derivation call is `gen_seeded_rng(seed.wrapping_add(1), &org_id)` (the two-arg re-export alias in `prism-dtu-common::lib`), NOT the one-arg legacy `seeded_rng`. See ADR-036 v2.1 U-A-01 for the distinction.

Red Gate: `test_BC_2_06_019_secondary_rng_independence_no_primary_shift`

---

### Group B — Enrichment Correlation (BC-2.06.020)

**AC-013 — INV-THREATINTEL-IOC-CORRELATION-001: all scenario IOCs resolve as Malicious in ThreatIntel**
(traces to BC-2.06.020 invariant INV-THREATINTEL-IOC-CORRELATION-001, postconditions 1 and 2)

Given `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` (infallible),
when lookup requests are issued for each of `entities.ioc_ips[0]`, `entities.ioc_domains[0]`, and `entities.ioc_hashes[0]`,
then each response contains `threat_is_known_malicious = true` and `threat_score >= 75`;
and `ThreatIntelState.fixture_registry` (a `Mutex<HashMap<String, FixtureKey>>`) has all IOC entries with `FixtureKey::Malicious` pre-populated at construction time.

The constructor: calls existing `with_admin_token` init, then pre-populates `fixture_registry` by locking the Mutex and inserting all IOC strings from `entities.ioc_ips`, `entities.ioc_domains`, and `entities.ioc_hashes` with `FixtureKey::Malicious`. Lock is released before construction returns. This is the ONLY mutation — no further mutation of `fixture_registry` in the scenario injection path after construction.

The `prism-dtu-threatintel/Cargo.toml` requires adding `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature (ADR-036 v2.2 §2.3).

Red Gate: `test_BC_2_06_020_threatintel_ioc_correlation_all_types`

**AC-014 — INV-NVD-CVE-CORRELATION-001: all scenario CVEs resolve with HIGH CVSS in NVD**
(traces to BC-2.06.020 invariant INV-NVD-CVE-CORRELATION-001, postconditions 3 and 4)

Given `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (fallible, mirrors `NvdClone::new() -> anyhow::Result<Self>`),
when `NvdState::lookup_and_count(&state, &entities.device_cves[0])` is called (NOT `NvdClone::lookup()` — this method does not exist),
then it returns `Some(record)` where:
- `record.metrics.cvss_metric_v31` is `Option<Vec<CvssMetricV31>>` — test code MUST call `.as_ref().and_then(|v| v.first())` (or equivalent) to unwrap the Option before accessing `.cvss_data`
- `record.metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0` (type: `f64`; exact path per ADR-036 v2.3 §2.3 and `crates/prism-dtu-nvd/src/types.rs`)
- `record.metrics.cvss_metric_v31[0].cvss_data.base_severity == "HIGH"` (type: `String`; field is `base_severity`, NOT `severity`)
- Default values for construction: `base_score = 8.1`, `base_severity = "HIGH".to_string()`

`NvdState.cve_registry` is an IMMUTABLE `HashMap<String, CveRecord>` (NOT Mutex-wrapped). Built at construction: load base fixtures from `fixtures/cves.json` (same as `new()`), then insert synthetic `CveRecord` entries for each CVE ID in `entities.device_cves`. No post-construction mutation.

The `prism-dtu-nvd/Cargo.toml` requires adding `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature (ADR-036 v2.2 §2.3).

Implementer MUST read `crates/prism-dtu-nvd/src/types.rs` before implementing to confirm the exact `CveRecord`, `CveMetrics`, `CvssMetricV31`, `CvssData` struct field names and types.

Red Gate: `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score`

**AC-015 — INV-CROSS-DTU-ENTITY-COHERENCE-001: primary_device_id consistent across Armis, CrowdStrike, Claroty at stage >= 1**
(traces to BC-2.06.020 invariant INV-CROSS-DTU-ENTITY-COHERENCE-001 and postcondition 5)

Given three clones (Armis, CrowdStrike, Claroty) all constructed with the same `(seed=100, org_id="<uuid-with-first-4-bytes-0xde-0xad-0xbe-0xef>", scenario.enabled=true)` and `scenario_start_secs = T`, when all three are queried at `now = T + 90s` (stage 1 / Recon), then:
- Armis `/api/v1/devices` response contains a device with ID `catalog.primary_device_id_armis = "dev-deadbeef-100-0"`
- CrowdStrike `/devices/entities/devices/v2` (or equivalent) response contains a device with ID `catalog.primary_device_id_cs = "dev-deadbeef-100-0"`
- Claroty device/asset response contains a device with ID following the same canonical format `"dev-deadbeef-100-0"` (Claroty uses the same `org_slug_from_org_id()` derivation; harness passes `&catalog.org_slug` consistently)

The canonical org_slug formula: `hex(org_id.as_bytes()[0..4])` → 8 hex chars (ADR-036 v2.2 §2.2 `org_slug_from_org_id`).

Red Gate: `test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones`

**AC-016 — INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001**
(traces to BC-2.06.020 invariants INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 and INV-PERIMETER-COMPLIANCE-001)

Given a ThreatIntel clone constructed with `new_with_scenario` and a non-scenario IP `"192.0.2.1"` (not in `ioc_ips`),
when a lookup is issued for `"192.0.2.1"`,
then the response is identical to `ThreatIntelClone::new().lookup("192.0.2.1")` — scenario injection is strictly additive.

AND neither `prism-dtu-threatintel` nor `prism-dtu-nvd` may import from `prism-spec-engine`, `prism-sensors`, or `prism-query` after `new_with_scenario` constructors are added (`prism-core` is on the allow-list per BC-2.06.020 INV-PERIMETER-COMPLIANCE-001). This constraint is enforced STRUCTURALLY: `prism-dtu-threatintel` and `prism-dtu-nvd` declare no dependency on the forbidden crates in their `Cargo.toml` entries, so any forbidden `use` statement is an ordinary E0432 compile error caught by the standard `cargo build`. The compile-fail gate at `tests/external/perimeter-violation/` enforces the `prism-query` pub-API perimeter (BC-2.11.006) only — it has no dependency on or knowledge of the DTU crates and does NOT enforce the DTU perimeter.

Red Gate: `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate`

**AC-017 — E-DEMO-003: archetype/fixture_set contradiction rejected at construction**
(traces to BC-2.06.019 edge case EC-019-012)

Given `scenario.enabled = true`, when `build_clone_pairs` encounters an archetype/fixture_set contradiction, it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` — before any clone constructor is called.

Two contradiction directions are covered (both named by EC-019-012):

**Direction 1 — archetype that does not support scenario progression:**
Given `scenario.archetype = "healthy"` (HealthyOtEnvironment does not support 5-stage progression) with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` and the clone name and the archetype string.

Message shape (verbatim taxonomy E-DEMO-003 unrecognized-archetype variant, error-taxonomy.md §DEMO):
`"demo-server: E-DEMO-003: clone '{clone_name}': unrecognized scenario archetype '{value}'; valid values: compromised_endpoint, healthy"`

NOTE: `"healthy"` IS a recognized archetype string (it is in the `valid values` list), yet a `healthy` archetype with `scenario.enabled = true` is a contradiction — the `healthy` archetype does not support the 5-stage `IncidentTimeline`. `build_clone_pairs` must detect this coherence failure (archetype does not support progression) and return E-DEMO-003. The existing unrecognized-archetype message variant subsumes this case: the archetype value IS syntactically valid but is semantically incompatible with `scenario.enabled = true`. If the BC sanctions a distinct message variant for this direction (e.g., `"archetype 'healthy' does not support scenario progression"`), the implementer must use that variant; absent a BC-sanctioned variant, the existing message shape with the incompatible archetype string in `{value}` is the correct default.

**Direction 2 — archetype/fixture_set incoherence (compromised_endpoint × DormantTenant):**
Given `scenario.archetype = "compromised_endpoint"` (explicitly set) but `fixture_set = "dormant"` (which maps to the `DormantTenant` archetype internally), when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` and the clone name. The `fixture_set`-derived archetype and the `scenario.archetype`-declared archetype disagree; `build_clone_pairs` detects the contradiction because a `CompromisedEndpoint` scenario archetype MUST be coherent with the fixture_set-derived archetype for every clone in scope. A `DormantTenant` fixture_set produces empty generated records — driving a 5-stage `CompromisedEndpoint` timeline over an empty dataset is incoherent.

The `scenario.archetype` field is consumed, not decorative: for every clone with `scenario.enabled = true`, `build_clone_pairs` MUST verify that `scenario.archetype` agrees with the archetype derived from `fixture_set`. If they contradict, E-DEMO-003 fires before any constructor is called.

Guard placement: within the ordered pre-construction validation block, the archetype/fixture_set contradiction check at the E-DEMO-003 position — AFTER the E-DEMO-002 seed-mismatch check and E-DEMO-006 org_id-mismatch check, and BEFORE the E-DEMO-004 missing-org_id check:
`E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype / archetype×fixture_set contradiction) → E-DEMO-004 (missing org_id)`

Red Gate: `test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction`

**AC-018 — E-DEMO-006: mismatched org_ids across scenario-enabled clones rejected at construction**
(traces to BC-2.06.019 precondition 6 and EC-019-013; TV-019-015)

Given two or more scenario-enabled clones in the same client config block that share the same `seed` but have different `org_id` values (e.g., `clones.crowdstrike.org_id = "<uuid-A>"` and `clones.armis.org_id = "<uuid-B>"` where uuid-B ≠ uuid-A), both with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-006"` and both clone names and both org_id values — BEFORE any clone constructor is called.

Verbatim message format (error-taxonomy.md v1.78 E-DEMO-006):
`"demo-server: E-DEMO-006: scenario clones '{clone_a}' (org_id={org_id_a}) and '{clone_b}' (org_id={org_id_b}) have different org_ids; cross-DTU coherence requires all scenario-enabled clones to share the same org_id"`

Guard placement: AFTER the E-DEMO-002 seed-mismatch check and BEFORE the E-DEMO-003 archetype check. The full canonical order is:
`E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype / archetype×fixture_set contradiction) → E-DEMO-004 (missing org_id)`

Rationale (BC-2.06.019 PRE-6): without this guard, `ScenarioEntityCatalog` is derived from the first clone's `(seed, org_id_A)` pair, producing `primary_device_id = "dev-{slug_A}-{seed}-0"`. A second clone using `org_id_B` generates devices as `"dev-{slug_B}-{seed}-0"` (different slug). INV-CROSS-DTU-ENTITY-COHERENCE-001 (BC-2.06.020) cross-DTU join returns empty with no diagnostic — a SOUL.md §4 silent partial-failure. The guard prevents this silent incoherence.

Test vector: BC-2.06.019 TV-019-015 (`seed_A = 100` for crowdstrike org_id=uuid-A and armis org_id=uuid-B ≠ uuid-A, both scenario.enabled=true → Err containing E-DEMO-006, no clone constructed).

Red Gate: `test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones`

**AC-019 — Cyberint alert CVEs correlate to NVD in scenario mode; collision-safe synthetic CVE namespace in all modes**
(traces to BC-2.06.020 postcondition 8 (INV-CYBERINT-ALERT-CVE-CORRELATION-001 scenario-mode clause), postcondition 9 (baseline collision-safety), and INV-CYBERINT-ALERT-CVE-CORRELATION-001)

**Scenario mode (CyberintClone constructed via `new_with_scenario`):**

Given a `CyberintClone` constructed via `CyberintClone::new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor, &catalog)` where `catalog.device_cves` contains exactly 3 entries (e.g., `["CVE-9999-00001", "CVE-9999-00002", "CVE-9999-00003"]` — drawn from `gen_device_cves` per SEC-001), when the generated CVE-surface alert records are inspected:
- Every record's `cve_id` field MUST be a member of `catalog.device_cves`. No `cve_id` may reference a CVE outside `catalog.device_cves`.
- When `generate_cves` produces more than 3 records (e.g., `CompromisedEndpoint` baseline = 10 records), the mapping is cyclic: record at index `i` uses `catalog.device_cves[i % catalog.device_cves.len()]`. This ensures all 10 records' `cve_id` values cycle over the 3 catalog entries — no out-of-catalog CVE ID is introduced regardless of record count (BC-2.06.020 §PC-8 / EC-020-012).
- For every `cve_id` on every generated CVE record, `NvdState::lookup_and_count(&state, cve_id)` returns `Some(record)` where `record.metrics.cvss_metric_v31.as_ref().and_then(|v| v.first()).unwrap().cvss_data.base_score >= 7.0` — the end-to-end pivot chain `Cyberint alert cve_id → NVD lookup → HIGH CVSS record` resolves without exception (BC-2.06.020 §INV-CYBERINT-ALERT-CVE-CORRELATION-001 scenario-mode clause; requires NVD `new_with_scenario` to have pre-populated all `catalog.device_cves` per PC-3).
- Note: `catalog.device_cves` entries are `CVE-9999-{:05}` format (SEC-001; `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs` uses this sentinel). The `CVE-9999-` year is the same collision-safe namespace used for baseline mode — the difference in scenario mode is that these specific `CVE-9999-*` IDs are also present in the NVD registry via `NvdClone::new_with_scenario`.

**Baseline/non-scenario mode (`new()`, `new_with_seed()`, or `new_with_access_token()` — no `ScenarioEntityCatalog` available):**

Given a `CyberintClone` constructed via any non-scenario constructor, when the generated CVE-surface alert records are inspected:
- Every record's `cve_id` field MUST match `^CVE-9999-\d{4}$` (format: `"CVE-9999-{:04}"` with `rng.gen_range(0u32..10000)`). The `CVE-9999-` prefix uses year 9999, which is never used by the real NVD advisory database.
- No `cve_id` may use a real calendar year pattern (`CVE-20xx-*`, `CVE-19xx-*`, `CVE-200x-*`). The pre-fix behavior of generating `CVE-2024-{:04}` (line ~340 of `prism-dtu-cyberint/src/generator.rs`) is a VIOLATION of this AC; the implementer fix changes this to `CVE-9999-{:04}` unconditionally for all non-scenario paths (BC-2.06.020 §PC-9; SEC-001 directive).
- These baseline CVEs are intentionally non-pivotable: `NvdState::lookup_and_count(&state, cve_id)` returns `None` for `CVE-9999-*` IDs in a non-scenario NVD clone (which holds only static fixture entries). A 404/"not found" NVD response is the correct and expected outcome in baseline mode — no error or exception in the demo flow.
- The existing `test_sec_001_device_cves_use_unambiguous_synthetic_year` test in `prism-dtu-common` covers the `gen_device_cves` function (catalog CVEs); AC-019 baseline tests cover the Cyberint generator path (`prism-dtu-cyberint/src/generator.rs` line ~340) separately.

**Universal collision-safety (ALL modes):**

Given any `CyberintClone` in any mode, when all generated CVE-surface alert records are collected, then no `cve_id` matches the pattern `^CVE-(202\d|201\d|200\d|199\d)-` — no real calendar year is used in any Cyberint-generated CVE ID, whether scenario or baseline. This invariant is statically guaranteed: scenario mode draws from `catalog.device_cves` (which are `CVE-9999-*` per SEC-001); baseline mode generates `CVE-9999-*` directly after the implementer fix (BC-2.06.020 §INV-CYBERINT-ALERT-CVE-CORRELATION-001 universal collision-safety clause).

Red Gate tests (verbatim names from implementer commit f0b6b8c7):
- `test_BC_2_06_020_cyberint_baseline_cve_uses_cve_9999_namespace` — crate: `prism-dtu-cyberint`; traces to BC-2.06.020 PC-9 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 baseline mode / TV-020-011 / VP-020-I
- `test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog` — crate: `prism-dtu-cyberint`; traces to BC-2.06.020 PC-8 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 scenario mode / TV-020-012 / VP-020-J
- `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` — crate: `prism-dtu-demo-server`; test file: `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs`; traces to BC-2.06.020 PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001 + INV-NVD-CVE-CORRELATION-001 / TV-020-013 / VP-020-K
- `test_BC_2_06_020_cyberint_scenario_cyclic_catalog_assignment` — crate: `prism-dtu-cyberint`; traces to BC-2.06.020 EC-020-012 / TV-020-014 / VP-020-L

SEC-001 note: the `catalog.device_cves` field (source: `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs`) already uses `CVE-9999-{:05}` format per SEC-001. The existing test `test_sec_001_device_cves_use_unambiguous_synthetic_year` in `prism-dtu-common` verifies the catalog side. AC-019 covers the complementary Cyberint generator side: (a) scenario path draws from that catalog, (b) baseline path independently uses `CVE-9999-` on line ~340 of `generator.rs`.

---

## Red Gate Test Plan

All tests written FAIL-first per SID-1 (CLAUDE.md §SID-1). Unit tests in `#[cfg(test)] mod tests` blocks or integration tests in `crates/<crate>/tests/`. No `#[ignore]` unless blocking on a live external service (in-process harness tests are NOT `#[ignore]`'d per SID-1).

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_06_019_timeline_types_non_exhaustive_and_structure` | prism-dtu-common | BC-2.06.019 PRE-3 / ADR-036 v2.2 §2.2 | unit |
| 2 | `test_BC_2_06_019_stage_index_pure_function_reproducible` | prism-dtu-common | BC-2.06.019 INV-PROGRESSION-REPRODUCIBILITY-001 / PC-3 | unit |
| 3 | `test_BC_2_06_019_stage_boundary_5_thresholds_correct` | prism-dtu-common | BC-2.06.019 PC-2, PC-3 / TV-019-001..005 | unit |
| 4 | `test_BC_2_06_019_stage_index_monotonic_over_time` | prism-dtu-common | BC-2.06.019 INV-STAGE-MONOTONICITY-001 | unit |
| 5 | `test_BC_2_06_019_clock_skew_clamped_to_baseline` | prism-dtu-common | BC-2.06.019 EC-019-003 / TV-019-006 | unit |
| 6 | `test_BC_2_06_019_stage_mask_completeness_all_6_fields` | prism-dtu-common | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 / PC-2 table | unit |
| 7 | `test_BC_2_06_019_armis_primary_device_stage_visibility` | prism-dtu-armis | BC-2.06.019 PC-4 / TV-019-009, TV-019-010 | integration |
| 8 | `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only` | prism-dtu-crowdstrike | BC-2.06.019 PC-4 / TV-019-011 | integration |
| 9 | `test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-002 / TV-019-012 | unit |
| 10 | `test_BC_2_06_019_e_demo_003_unrecognized_archetype` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-003 / TV-019-013 | unit |
| 11 | `test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path` | prism-dtu-demo-server | BC-2.06.019 INV-SCENARIO-DISABLED-COMPAT-001 / TV-019-007 | regression |
| 12 | `test_BC_2_06_019_secondary_rng_independence_no_primary_shift` | prism-dtu-common | BC-2.06.019 INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 / PC-1 | unit |
| 13 | `test_BC_2_06_020_threatintel_ioc_correlation_all_types` | prism-dtu-threatintel | BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / PC-1, PC-2 | unit |
| 14 | `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score` | prism-dtu-nvd | BC-2.06.020 INV-NVD-CVE-CORRELATION-001 / PC-3, PC-4 | unit |
| 15 | `test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones` | prism-dtu-demo-server | BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 / PC-5 | integration |
| 16 | `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate` | prism-dtu-threatintel | BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001 / PC-6 — passthrough assertion is unit test; perimeter compliance verified structurally via `cargo build` (no forbidden `Cargo.toml` dep → E0432 on violation; `tests/external/perimeter-violation/` covers `prism-query` perimeter only) | unit |
| 17 | `test_dormant_tenant_seeded_empty_records_not_static_fallback` | prism-dtu-armis (or prism-dtu-crowdstrike) | DormantTenant regression: `fixture_gen_seeded=true + generated_records=[]` must NOT fall back to static JSON — it must return empty response, not static-fixture data | unit |
| 18 | `test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction` | prism-dtu-demo-server | BC-2.06.019 EC-019-012: archetype/fixture_set contradiction (`compromised_endpoint` × `DormantTenant`, and `healthy` archetype with scenario enabled) returns E-DEMO-003 before any constructor called; guard position: E-DEMO-002 → E-DEMO-006 → **E-DEMO-003** → E-DEMO-004 | unit |
| 19 | `test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones` | prism-dtu-demo-server | BC-2.06.019 PRE-6 / EC-019-013 / TV-019-015: two scenario-enabled clones with same seed but different org_ids returns E-DEMO-006 containing both clone names and org_id values before any constructor called; guard position: E-DEMO-002 → **E-DEMO-006** → E-DEMO-003 → E-DEMO-004 | unit |
| 20 | `test_BC_2_06_020_cyberint_baseline_cve_uses_cve_9999_namespace` | prism-dtu-cyberint | BC-2.06.020 PC-9 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 (baseline mode) / TV-020-011 / VP-020-I: all generated CVE records from non-scenario CyberintClone use `CVE-9999-` namespace; no real-year CVE IDs emitted (regression against pre-fix `CVE-2024-*` behavior) | unit |
| 21 | `test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog` | prism-dtu-cyberint | BC-2.06.020 PC-8 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 (scenario mode) / TV-020-012 / VP-020-J: all scenario-mode Cyberint CVE records' `cve_id` values are members of `catalog.device_cves`; no out-of-catalog CVE ID introduced | unit |
| 22 | `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` | prism-dtu-demo-server | BC-2.06.020 PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001 + INV-NVD-CVE-CORRELATION-001 / TV-020-013 / VP-020-K: for each `cve_id` in scenario Cyberint records, `NvdState::lookup_and_count(cve_id)` returns `Some(record)` with `base_score >= 7.0`; end-to-end pivot chain; test file: `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs` | integration |
| 23 | `test_BC_2_06_020_cyberint_scenario_cyclic_catalog_assignment` | prism-dtu-cyberint | BC-2.06.020 EC-020-012 / TV-020-014 / VP-020-L: `CompromisedEndpoint` archetype produces 10 CVE records against 3-entry catalog; all `cve_id` values are in catalog; cyclic assignment distributes records over 3 catalog entries without overflow or repetition of an out-of-catalog ID | unit |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file, v2.16) | ~9 000 |
| ADR-036 v2.3 (full) | ~5 800 |
| BC-2.06.019 v1.7 (full) | ~3 200 |
| BC-2.06.020 v1.6 (full) | ~3 600 |
| Story A spec (substrate context; confirmed merged) | ~3 000 |
| prism-dtu-common/src/scenario/mod.rs (from Story A + extensions) | ~1 500 |
| prism-dtu-demo-server/src/{harness,config}.rs (post-Story-A state) | ~2 000 |
| prism-dtu-armis/src/{state,clone}.rs (post-Story-A state) | ~1 500 |
| prism-dtu-crowdstrike/src/{state,clone}.rs (post-Story-A state) | ~1 500 |
| prism-dtu-claroty/src/{state,clone}.rs (post-Story-A state) | ~1 000 |
| prism-dtu-cyberint/src/{state,clone,generator}.rs (post-Story-A state) | ~1 200 |
| prism-dtu-threatintel/src/{state,clone}.rs | ~900 |
| prism-dtu-nvd/src/{state,clone,types}.rs | ~1 000 |
| ci.yml (EXPECTED line) | ~200 |
| Test files (23 stubs × ~40 lines each) | ~2 800 |
| Tool outputs (nextest, clippy, compile-fail) | ~2 000 |
| BC files (2 BCs: BC-2.06.019, BC-2.06.020) | included above |
| **Total estimate** | **~40 200** |

At ~200k context window, this is ~20.1% — within the 20-30% ceiling.

---

## Tasks

Implementation checklist (TDD order — write failing tests before each implementation step per SID-1):

**Pre-flight: read substrate before writing anything**

- [ ] Read `crates/prism-dtu-common/src/scenario/mod.rs` (Story A substrate) fully before editing — confirm `ScenarioEntityCatalog`, `org_slug_from_org_id`, `build_scenario_entity_catalog`, and `gen_seeded_rng` are present
- [ ] Read `crates/prism-dtu-demo-server/src/config.rs` — confirm `ScenarioConfig { enabled, archetype, scenario_start_secs, stage_duration_secs }` fields are deserialized (NIT-2: all four fields present, none consumed yet)
- [ ] Read `crates/prism-dtu-demo-server/src/harness.rs` — confirm E-DEMO-004/005 guards from Story A are in place; identify where E-DEMO-002/003 and scenario coordination must be added
- [ ] Read `crates/prism-dtu-nvd/src/types.rs` fully — confirm exact struct path: `CveRecord.metrics: CveMetrics` → `.cvss_metric_v31: Option<Vec<CvssMetricV31>>` → `[0].cvss_data: CvssData` → `.base_score: f64` and `.base_severity: String`
- [ ] Read `crates/prism-dtu-nvd/src/state.rs` — confirm `cve_registry: HashMap<String, CveRecord>` is immutable (NOT Mutex-wrapped); confirm `NvdState::lookup_and_count(&self, cve_id: &str) -> Option<CveRecord>`
- [ ] Read `crates/prism-dtu-threatintel/src/state.rs` — confirm `fixture_registry: Mutex<HashMap<String, FixtureKey>>` and `FixtureKey::Malicious` variant
- [ ] Read ci.yml EXPECTED= value (live; do NOT assume 51 from ADR-036 estimate)
- [ ] Run SAP-2 pre-check (CLAUDE.md §SAP-2): if this story modifies any TOML sensor specs, read corresponding DTU clone types.rs; flag any TOML column without a DTU struct field equivalent

**Phase 1: prism-dtu-common scenario module — IncidentTimeline layer**

- [ ] Write failing test 1 (FAIL first): `test_BC_2_06_019_timeline_types_non_exhaustive_and_structure`
- [ ] Add `IncidentTimeline` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`) with fields: `entities: ScenarioEntityCatalog`, `stages: Vec<IncidentStage>`, `scenario_start_epoch_secs: i64`
- [ ] Add `IncidentStage` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`) with fields: `name: &'static str`, `activates_after_secs: u64`, `visible_entity_mask: StageMask`
- [ ] Add `StageMask` (NOT `#[non_exhaustive]`; internal struct; `#[derive(Clone, Debug)]`) with 6 bool fields: `primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`
- [ ] Write failing tests 2-6 (FAIL first): pure function, stage boundaries, monotonicity, clock skew, mask completeness
- [ ] Implement `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize` per ADR-036 v2.2 §2.2: `elapsed = (now - start).max(0) as u64`; iterate stages; return last index with `activates_after_secs <= elapsed`
- [ ] Implement `build_default_incident_timeline(catalog: ScenarioEntityCatalog, start_secs: i64, stage_duration_secs: &[u64]) -> IncidentTimeline`: if empty, use default `[60, 180, 360, 600]`; construct 5 `IncidentStage` instances with explicit `StageMask` per BC-2.06.019 PC-2 table; stage 0 `activates_after_secs = 0` always
- [ ] Verify tests 1-6 pass
- [ ] Update `ci.yml EXPECTED=` by exact count of new `#[non_exhaustive]` pub types in this story (at minimum +2: `IncidentTimeline`, `IncidentStage`); update `tests/external/non-exhaustive-violation/` violation rows atomically

**Phase 2: Per-clone new_with_scenario constructors**

- [ ] Write failing test 8 (FAIL first): `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only`
- [ ] Read CrowdStrike `state.rs`, `clone.rs` (post-Story-A) — confirm `generated_devices: Vec<serde_json::Value>` and `generated_detections: Vec<serde_json::Value>` present
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `CrowdstrikeState`
- [ ] Add `CrowdstrikeClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> Self` under `#[cfg(feature = "fixture-gen")]` (5-arg, ADR-036 v2.3 §2.4): calls `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` internally (NOT the 3-arg `new_with_seed` — ADR-036 v2.3 §2.3), then sets `timeline = Some(Arc::clone(&timeline))`. `time_anchor` is passed in from `build_clone_pairs` (derived ONCE from `scenario_start_epoch_secs` via `DateTime::from_timestamp`).
- [ ] Modify `routes/hosts.rs`: when `fixture_gen_seeded == true && timeline.is_some()` (scenario path), call `current_stage_index` and apply `StageMask` filter on `generated_devices`; when `fixture_gen_seeded == true && timeline.is_none()` (Story A seeded path), serve all `generated_devices` without mask filter; when `fixture_gen_seeded == false` (static path), serve unchanged static JSON — three-way composition, NOT `generated_devices.is_empty()` branching (DormantTenant guard: seeded=true produces records=[] for some archetypes — DO NOT branch on `generated_devices.is_empty()`). NOTE: if CrowdstrikeState uses `fixture_gen_seeded: bool` flag (see state.rs MUST-level doc comment), branch on that flag, not record count.
- [ ] Verify test 8 passes

- [ ] Write failing test 7 (FAIL first): `test_BC_2_06_019_armis_primary_device_stage_visibility`
- [ ] Read Armis `state.rs`, `clone.rs` (post-Story-A) — confirm `generated_records: Vec<serde_json::Value>` present
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `ArmisState`
- [ ] Add `ArmisClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> anyhow::Result<Self>` (5-arg fallible, ADR-036 v2.3 §2.4): calls `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` internally (NOT 3-arg `new_with_seed`), propagates its `Result`, then sets `timeline = Some(Arc::clone(&timeline))`.
- [ ] Modify Armis `routes/devices.rs` `paginate_devices`: three-way composition on `fixture_gen_seeded` flag — scenario path (`fixture_gen_seeded == true && timeline.is_some()`): apply `StageMask` filter on `generated_records`; seeded path (`fixture_gen_seeded == true && timeline.is_none()`): serve all `generated_records`; static path (`fixture_gen_seeded == false`): serve `devices_ordered` unchanged. DO NOT branch on `generated_records.is_empty()` (DormantTenant guard).
- [ ] Verify test 7 passes

- [ ] Read Claroty `state.rs`, `clone.rs` (post-Story-A) before editing
- [ ] IMPORTANT — chrono feature gate in Claroty: `chrono` is gated behind `dep:chrono` in `crates/prism-dtu-claroty/Cargo.toml` under `[features] fixture-gen = [...]` (unlike armis/crowdstrike where chrono is unconditional). The `timeline: Option<Arc<IncidentTimeline>>` state field, `time_anchor: DateTime<Utc>` constructor parameter, and all `Utc::now()` / chrono call sites in handlers MUST be `#[cfg(feature = "fixture-gen")]`-gated. Non-gated code must not reference `chrono`. Verification task: `cargo check -p prism-dtu-claroty` WITHOUT `--features fixture-gen` must compile with zero errors.
- [ ] Add `#[cfg(feature = "fixture-gen")] timeline: Option<Arc<IncidentTimeline>>` to `ClarotyState`
- [ ] Add `#[cfg(feature = "fixture-gen")] ClarotyClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> Self` (5-arg, ADR-036 v2.3 §2.4): calls `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` internally (NOT 3-arg `new_with_seed`), then sets `timeline = Some(Arc::clone(&timeline))`.
- [ ] Modify Claroty route handlers: three-way composition on `fixture_gen_seeded` (same DormantTenant guard as armis/crowdstrike); scenario branch gated `#[cfg(feature = "fixture-gen")]`. Verify `cargo check -p prism-dtu-claroty` (without fixture-gen) passes.

- [ ] Read Cyberint `state.rs`, `clone.rs` (post-Story-A) before editing
- [ ] IMPORTANT — chrono feature gate in Cyberint: `chrono` is gated behind `dep:chrono` in `crates/prism-dtu-cyberint/Cargo.toml` under `[features] fixture-gen = [...]` (like Claroty; unlike armis/crowdstrike). Same constraint applies: `timeline` state field, `time_anchor` constructor parameter, and all chrono call sites in handlers MUST be `#[cfg(feature = "fixture-gen")]`-gated. Verification task: `cargo check -p prism-dtu-cyberint` WITHOUT `--features fixture-gen` must compile with zero errors.
- [ ] Add `#[cfg(feature = "fixture-gen")] timeline: Option<Arc<IncidentTimeline>>` to `CyberintState`
- [ ] Add `#[cfg(feature = "fixture-gen")] CyberintClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>, catalog: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (6-arg fallible; BC-2.06.020 PC-8): calls `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` internally (NOT 3-arg `new_with_seed`), then sets `timeline = Some(Arc::clone(&timeline))`. The `catalog` parameter is threaded to `generate_cves` / `generate_with_catalog` so that every CVE-surface alert record's `cve_id` ∈ `catalog.device_cves` — enabling the end-to-end AC-019 / VP-020-K pivot chain `Cyberint alert cve_id → NVD lookup → HIGH CVSS record`.
- [ ] Modify Cyberint route handlers: three-way composition on `fixture_gen_seeded` (same DormantTenant guard); scenario branch gated `#[cfg(feature = "fixture-gen")]`. Verify `cargo check -p prism-dtu-cyberint` (without fixture-gen) passes.

**Phase 3: Enrichment clone constructors**

- [ ] Write failing tests 13, 16 (FAIL first): IOC correlation + passthrough
- [ ] Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to `crates/prism-dtu-threatintel/Cargo.toml`
- [ ] Add `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` (infallible): replicates `ThreatIntelClone::new()` body (clone.rs:48-60) — generates an `admin_token` uuid, constructs `ThreatIntelState::with_admin_token(admin_token.clone())` (state.rs:38 — this is a `ThreatIntelState` method, NOT a `ThreatIntelClone` method), stores in `Arc::new(...)`, fills in the other `ThreatIntelClone` fields — then locks `fixture_registry` on the resulting state and inserts all `ioc_ips`, `ioc_domains`, `ioc_hashes` as `FixtureKey::Malicious`; releases lock before returning; must NOT import `prism-spec-engine`/`prism-sensors`/`prism-query`
- [ ] Verify tests 13, 16 pass

- [ ] Write failing test 14 (FAIL first): `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score`
- [ ] Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to `crates/prism-dtu-nvd/Cargo.toml`
- [ ] Add `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (fallible, matches `new()` return type): load base fixtures from `fixtures/cves.json` (same as `new()`), then insert synthetic `CveRecord` entries for each CVE ID in `entities.device_cves` with `base_score = 8.1`, `base_severity = "HIGH".to_string()` filling all required `CvssData`/`CvssMetricV31` struct fields from `types.rs`; build immutable `HashMap` at construction (NOT Mutex-wrapped)
- [ ] Verify test 14 passes

**Phase 4: build_clone_pairs scenario coordination**

- [ ] Read `crates/prism-dtu-demo-server/src/harness.rs` (post-Story-A) before editing
- [ ] Write failing tests 9, 10, 11, 12, 18, 19 (FAIL first): E-DEMO-002, E-DEMO-003 (unrecognized archetype), E-DEMO-003 (archetype×fixture_set contradiction, AC-017), scenario-disabled compat, secondary RNG independence, E-DEMO-006 (org_id mismatch, AC-018)
- [ ] Add E-DEMO-002 guard: before any constructor, if multiple scenario-enabled clones have different `seed` values, return `Err(anyhow!("demo-server: E-DEMO-002: ..."))`
- [ ] Add E-DEMO-006 guard (position: AFTER E-DEMO-002, BEFORE E-DEMO-003): if multiple scenario-enabled clones have different `org_id` values, return `Err(anyhow!("demo-server: E-DEMO-006: scenario clones '{clone_a}' (org_id={org_id_a}) and '{clone_b}' (org_id={org_id_b}) have different org_ids; cross-DTU coherence requires all scenario-enabled clones to share the same org_id"))` — before any clone constructor is called (AC-018, BC-2.06.019 PRE-6, EC-019-013, TV-019-015). Both clone names and both org_id values must appear in the error string.
- [ ] Add E-DEMO-003 guard (position: AFTER E-DEMO-006, BEFORE E-DEMO-004): (a) if `scenario.archetype` is not a recognized string (`"compromised_endpoint"`, `"healthy"`), return E-DEMO-003; (b) if `stage_duration_secs` length != 4 for `compromised_endpoint`; (c) if `scenario.archetype = "healthy"` with `scenario.enabled = true` (archetype does not support progression); (d) if `scenario.archetype = "compromised_endpoint"` but `fixture_set` maps to `DormantTenant` (archetype/fixture_set incoherence, EC-019-012 Direction 2). All four sub-cases return E-DEMO-003 before any clone constructor is called (AC-017).
- [ ] Fix stale doc comment at `crates/prism-dtu-demo-server/src/config.rs:98` — currently reads "Only `\"compromised_endpoint\"` is supported in v1"; update to list both recognized values: "Valid values: `\"compromised_endpoint\"`, `\"healthy\"`" (or equivalent accurate prose); must be fixed in the SAME commit as the E-DEMO-003 guard implementation (D item from remove-uncertainty findings)
- [ ] Consume `scenario.scenario_start_secs` (NIT-2): derive ONCE — `let scenario_start_epoch_secs = config.scenario.scenario_start_secs.unwrap_or_else(|| Utc::now().timestamp()); let time_anchor = DateTime::from_timestamp(scenario_start_epoch_secs, 0).expect("scenario_start_epoch_secs always in-range");` (ADR-036 v2.3 §2.4 step 4). When `scenario_start_secs = None`, `Utc::now()` is called AT MOST ONCE; both `scenario_start_epoch_secs` and `time_anchor` share the same captured epoch so they cannot diverge by milliseconds.
- [ ] Consume `scenario.stage_duration_secs` (NIT-2): pass to `build_default_incident_timeline`; empty vec uses defaults `[60, 180, 360, 600]`
- [ ] Consume `scenario.enabled` and `scenario.archetype` (NIT-2): gate scenario path on `enabled = true`; validate `archetype` string and emit E-DEMO-003 if invalid
- [ ] When `scenario.enabled = true`: derive `org_slug = org_slug_from_org_id(&org_id)` (already in Story A); build `ScenarioEntityCatalog` via `build_scenario_entity_catalog(seed, &org_id)` using `gen_seeded_rng(seed.wrapping_add(1), &org_id)` secondary stream; build `IncidentTimeline` from catalog + `stage_duration_secs`; wrap in `Arc::new(timeline)`; call `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor)` (5-arg, ADR-036 v2.3 §2.4) for CrowdStrike, Armis, and Claroty; call `CyberintClone::new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor, &catalog)` (6-arg, BC-2.06.020 PC-8 — `&catalog` is the 6th arg that threads CVE IDs into the Cyberint generator enabling the AC-019/VP-020-K pivot chain); call `ThreatIntelClone::new_with_scenario(&catalog)` and `NvdClone::new_with_scenario(&catalog)?`
- [ ] Verify NIT-1: the E-DEMO-004 guard (from Story A) fires correctly when `scenario.enabled = true` but `org_id = None`; add a test assertion confirming the guard order: E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype) → E-DEMO-004 (missing org_id) — all before any constructor is called
- [ ] Verify tests 9-12, 18-19 pass

**Phase 5: Cross-DTU coherence integration test**

- [ ] Write failing test 15 (FAIL first): `test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones`
- [ ] Implement: construct Armis, CrowdStrike, Claroty via `new_with_scenario` with same `(seed=100, org_id)` where first 4 bytes are `[0xde, 0xad, 0xbe, 0xef]`; query all three at `now = scenario_start_secs + 90s` (stage 1); assert each response contains `"dev-deadbeef-100-0"`
- [ ] Verify test 15 passes

**Phase 6: Final gates**

- [ ] Run SAP-1 probe (CLAUDE.md §SAP-1): `rg 'event_type\s*=' crates/ --type rust` — verify any new `event_type` emissions have BC-2.16.002 catalog rows; if NO new emissions added, state explicitly in PR description
- [ ] Run `cargo check -p prism-dtu-claroty` and `cargo check -p prism-dtu-cyberint` WITHOUT `--features fixture-gen` — both must compile with zero errors (chrono feature-gate verification, MEDIUM-C item)
- [ ] Sibling-sweep for forbidden 3-arg path in scenario context: `grep -rn "new_with_seed\b" crates/prism-dtu-*/src/clone.rs` — any occurrence inside a `new_with_scenario` body is a violation (must use `new_with_seed_anchored`)
- [ ] Run `just check` — all 23 Red Gate tests pass; zero clippy warnings; fmt clean
- [ ] Verify DTU perimeter (test 16): `cargo build -p prism-dtu-threatintel -p prism-dtu-nvd` compiles with zero E0432 errors involving `prism-spec-engine`, `prism-sensors`, or `prism-query` (structural Cargo enforcement per INV-PERIMETER-COMPLIANCE-001; the `tests/external/perimeter-violation/` compile-fail gate covers `prism-query` perimeter only and is unrelated to this check)
- [ ] Confirm all 4 `ScenarioConfig` fields consumed in `build_clone_pairs` — zero dead code warnings on `scenario.enabled`, `scenario.archetype`, `scenario.scenario_start_secs`, `scenario.stage_duration_secs`

---

## Previous Story Intelligence

This is Story B of the E-DEMO live-scenario split. Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A)
is the direct predecessor — merged PR #181 develop@c287b00d (D-1089 2026-06-10).

**Confirmed substrate (read from Story A spec v1.5 + BC-2.06.018 v1.6 active):**

- `CrowdstrikeState`, `ArmisState`, `ClarotyState`, `CyberintState` each have `generated_devices`/`generated_records` fields from Story A
- `ScenarioEntityCatalog`, `org_slug_from_org_id`, `build_scenario_entity_catalog` are in `prism-dtu-common/src/scenario/` from Story A
- `gen_seeded_rng(seed, &org_id)` two-arg re-export is the canonical call (ADR-036 v2.1 U-A-01 correction); NOT one-arg `seeded_rng`
- `ThreatIntelState.fixture_registry` is `Mutex<HashMap<String, FixtureKey>>` — mutable at construction
- `NvdState.cve_registry` is an immutable `HashMap<String, CveRecord>` (NOT Mutex-wrapped)
- `NvdClone::new()` returns `anyhow::Result<Self>` (fallible); Story B's `new_with_scenario` must match
- CVSS path: `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` — read `types.rs` to confirm field names
- `fixture-gen` feature added to `prism-dtu-threatintel` and `prism-dtu-nvd` Cargo.toml by Story A (ADR-036 §2.5 item 5)
- `ScenarioConfig { enabled, archetype, scenario_start_secs, stage_duration_secs }` deserialized in `CloneConfig` but fields UNCONSUMED — Story B consumes all four (NIT-2)
- `E-DEMO-004/005` guards from Story A in `build_clone_pairs` cover the `org_id` absent case for the scenario path (NIT-1 reconciliation)

**Implementer MUST verify substrate assumptions above against actual crate source files before writing any implementation.**

**From PLUGIN-MIGRATION-001-D lessons (SAP-1, SAP-2, SID-1):**
- SAP-1: after implementation, run `rg 'event_type\s*=' crates/ --type rust`; any new `event_type` emissions require BC-2.16.002 catalog rows in the same commit
- SAP-2: if any TOML sensor specs change, read corresponding DTU clone `types.rs` + routes to verify field parity (no TOML column without DTU struct equivalent)
- SID-1: integration tests driving in-process clone HTTP servers use `#[ignore]` ONLY if blocking on a live external service; in-process harness tests are not `#[ignore]`'d

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `current_stage_index` is a pure function: no side effects, no shared mutable state, no tokio spawn, no Arc<AtomicU64> counter | ADR-036 v2.3 §2.1 + BC-2.06.019 PC-3 | Adversary probe: grep for Mutex<StageIndex> / Arc<AtomicU64> in state structs |
| `StageMask` must NOT carry `#[non_exhaustive]` — internal struct, exhaustively constructible within the crate | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 (wins over ADR-036 v2.3 §2.2 code snippet which erroneously shows it with #[non_exhaustive] — BC wins per CLAUDE.md Source-of-Truth Precedence for contract semantics) | Adversary + compile test |
| `IncidentTimeline`, `IncidentStage` MUST carry `#[non_exhaustive]` as public types in `prism-dtu-common` | CLAUDE.md §Conventions #[non_exhaustive] discipline | ci.yml EXPECTED bump + non-exhaustive-violation/ rows |
| `IncidentTimeline` threaded via `Arc` (NOT `Arc<Mutex<...>>`) — read-only after construction | ADR-036 v2.3 §2.3 | Adversary: grep for Mutex<IncidentTimeline> |
| `NvdState.cve_registry` is an immutable `HashMap` (NOT Mutex-wrapped); built entirely at construction time | ADR-036 v2.3 §2.3 + BC-2.06.020 PC-3 | Adversary: grep for Mutex<.*cve_registry> |
| `new_with_scenario` for ThreatIntel/NVD must NOT import `prism-spec-engine`, `prism-sensors`, or `prism-query` | BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 + ADR-036 v2.3 §2.5 | Structural Cargo enforcement: forbidden crates absent from `prism-dtu-threatintel`/`prism-dtu-nvd` `Cargo.toml`; any forbidden `use` is a standard E0432 compile error. The `tests/external/perimeter-violation/` gate covers `prism-query` perimeter only (BC-2.11.006). |
| CVSS path: `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` (field `cvss_metric_v31` is `Option<Vec<CvssMetricV31>>` — unwrap the Option) and `.base_severity: String` — implementer MUST read `crates/prism-dtu-nvd/src/types.rs` | ADR-036 v2.3 §1.3 + §2.3 | Adversary: read types.rs before review; check test assertions |
| `stage_duration_secs` config array has exactly 4 entries for the 5-stage timeline (stages 1-4 thresholds; stage 0 always 0) | ADR-036 v2.3 §2.2 + BC-2.06.019 PC-2 | Tests 3, 10 |
| E-DEMO-002, E-DEMO-006, E-DEMO-003, and E-DEMO-004 all detected BEFORE any clone constructor is called in `build_clone_pairs`; canonical guard order: E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype / archetype×fixture_set contradiction) → E-DEMO-004 (missing org_id) | ADR-036 v2.3 §2.4 + BC-2.06.019 PRE-5 / PRE-6 + error-taxonomy v2.26 | Tests 9, 10, 18, 19 |
| All 4 `ScenarioConfig` fields (`enabled`, `archetype`, `scenario_start_secs`, `stage_duration_secs`) must be consumed in `build_clone_pairs`; zero dead-code warnings | NIT-2 (from Story A) | Adversary: clippy dead-code sweep |
| Guard order: seed-mismatch (E-DEMO-002) → org_id-mismatch (E-DEMO-006) → bad-archetype (E-DEMO-003) → missing-org_id (E-DEMO-004) — all before any constructor | NIT-1 reconciliation + BC-2.06.019 PRE-6 | Tests 9, 10, 18, 19 + adversary guard-order probe |
| `await_holding_lock = "deny"` (ADR-002 §H1): no `.await` inside a Mutex lock guard in route handlers | ADR-002 | clippy deny list |
| All tracing emission sites with `event_type =` must have BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe post-implementation |
| Forbidden pattern: `Arc::new(SomeThing::placeholder())` in production boot path | ADR-022 §C + CLAUDE.md | Adversary |
| Do NOT use `gen_seeded_rng(seed.wrapping_add(1), ...)` with the one-arg legacy `seeded_rng`; use the two-arg re-export alias `gen_seeded_rng` in `prism-dtu-common::lib` | ADR-036 v2.1 U-A-01 | Adversary: grep for one-arg seeded_rng usage in scenario catalog derivation |
| `new_with_scenario` for generator-backed clones MUST internally call `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` (4-arg) — NOT the 3-arg `new_with_seed` (which anchors at `demo_time_anchor()` = 2026-01-01, producing stale timestamps for a June 2026 demo). The `time_anchor` argument is passed in from `build_clone_pairs` (derived once from `scenario_start_epoch_secs`). Using 3-arg `new_with_seed` in the scenario path is a FORBIDDEN pattern. | ADR-036 v2.3 §2.3 | Adversary: grep for `new_with_seed\b` inside `new_with_scenario` bodies; any occurrence is a violation |
| Route handlers must branch on `fixture_gen_seeded` flag (from state struct), NOT on `generated_records.is_empty()` / `generated_devices.is_empty()`. DormantTenant archetype: `fixture_gen_seeded=true` but records=[] is a VALID state that must NOT fall through to static-JSON path. Three-way composition: scenario path (seeded + timeline.is_some()), seeded path (seeded + timeline.is_none()), static path (not seeded). | Post-Story-A MUST-level doc comments in crates/prism-dtu-armis/src/state.rs:160-169 and crates/prism-dtu-crowdstrike/src/state.rs:154-176 | Adversary: read state.rs before review; grep for generated_records.is_empty() in handler bodies — any occurrence is a violation |
| `chrono` in claroty and cyberint is gated `dep:chrono` under `[features] fixture-gen`. The `timeline` state field, `time_anchor` parameter, and all chrono call sites in handlers for these two clones MUST be `#[cfg(feature = "fixture-gen")]`-gated. Verification: `cargo check -p prism-dtu-claroty` and `cargo check -p prism-dtu-cyberint` WITHOUT `--features fixture-gen` must compile. | crates/prism-dtu-claroty/Cargo.toml:15 + crates/prism-dtu-cyberint/Cargo.toml:18 | Pre-merge CI gate |

---

## Library & Framework Requirements

Versions pinned from `dependency-graph.md` and `rust-toolchain.toml`. Do NOT invent versions.

| Crate | Version | Usage |
|-------|---------|-------|
| `axum` | `0.7` | Route handlers in all prism-dtu-* crates |
| `tokio` | `1` (multi-threaded runtime) | Async runtime per ADR-002 / AD-013 |
| `chrono` | project-pinned | `Utc::now().timestamp()` for `now_epoch_secs` and `DateTime::from_timestamp` for `time_anchor`. Already a direct (unconditional) dependency in armis and crowdstrike. In claroty and cyberint, `chrono` is gated `dep:chrono` under `[features] fixture-gen` — any `timeline` field, `time_anchor` parameter, or `Utc::now()` / `DateTime` usage in these two crates MUST be `#[cfg(feature = "fixture-gen")]`-gated; non-gated code must not reference chrono. NOT added to threatintel/nvd (no chrono needed for their constructors). |
| `serde` / `serde_json` | project-pinned | `CloneConfig` / `ScenarioConfig` deserialization + `generated_records` JSON handling |
| `rand_chacha` (`ChaCha20Rng`) | project-pinned | Secondary RNG stream for `build_scenario_entity_catalog` via `gen_seeded_rng` |
| `anyhow` | project-pinned | Error propagation for E-DEMO-002/E-DEMO-006/E-DEMO-003; `NvdClone::new_with_scenario` return type; `CyberintClone::new_with_scenario` return type |
| `uuid` | project-pinned | `uuid::Uuid::parse_str()` for org_id (already used in Story A substrate) |
| `reqwest` | project-pinned | Integration test HTTP clients; `.timeout(Duration::from_secs(30))` mandatory per CLAUDE.md |

**Forbidden patterns:**
- Do NOT introduce `tokio::time::interval` or `tokio::spawn` for stage progression (pure function only, ADR-036 v2.2 §3.1)
- Do NOT wrap `cve_registry` in `Mutex<...>` — it is immutable after construction (ADR-036 v2.2 §2.3)
- Do NOT call `generate()` in route handler bodies — generation is construction-time only
- Do NOT use the one-arg legacy `seeded_rng` for catalog derivation — use `gen_seeded_rng` (two-arg re-export, ADR-036 v2.1 U-A-01)
- Do NOT hardcode `Archetype::CompromisedEndpoint` in `build_clone_pairs` — archetype comes from `fixture_set→Archetype` mapping via INV-FIXTURE-SET-ARCHETYPE-MAP-001 (ADR-036 v2.2)
- Do NOT add `#[non_exhaustive]` to `StageMask` — it must be exhaustively constructible within `prism-dtu-common` (BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001)

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-common/src/scenario/mod.rs` | MODIFY (Story A stub exists) | Add `IncidentTimeline`, `IncidentStage`, `StageMask`, `current_stage_index`, `build_default_incident_timeline` |
| `crates/prism-dtu-crowdstrike/src/state.rs` | MODIFY (Story A: `generated_devices` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | MODIFY (Story A: `new_with_seed_anchored` present) | Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> Self` (5-arg; internally calls `new_with_seed_anchored`, NOT `new_with_seed`) |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | MODIFY (Story A: dual-path present) | Add StageMask filter when `timeline.is_some()` |
| `crates/prism-dtu-armis/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-armis/src/clone.rs` | MODIFY (Story A: `new_with_seed_anchored` present) | Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> anyhow::Result<Self>` (5-arg fallible; internally calls `new_with_seed_anchored`, NOT `new_with_seed`; propagates Result) |
| `crates/prism-dtu-armis/src/routes/devices.rs` | MODIFY (Story A: dual-path present) | Add StageMask filter when `timeline.is_some()` |
| `crates/prism-dtu-claroty/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-claroty/src/clone.rs` | MODIFY | Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>) -> Self` (5-arg; #[cfg(feature="fixture-gen")]; calls `new_with_seed_anchored`; chrono gated) |
| `crates/prism-dtu-claroty/src/routes/` | MODIFY (Story A: dual-path present) | Add StageMask filter |
| `crates/prism-dtu-cyberint/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>, time_anchor: DateTime<Utc>, catalog: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (6-arg fallible; #[cfg(feature="fixture-gen")]; calls `new_with_seed_anchored`; threads `catalog` to `generate_with_catalog` for PC-8 CVE correlation; chrono gated) |
| `crates/prism-dtu-cyberint/src/routes/` | MODIFY | Add StageMask filter |
| `crates/prism-dtu-threatintel/src/clone.rs` | MODIFY | Add `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` |
| `crates/prism-dtu-threatintel/Cargo.toml` | MODIFY (if not done in Story A §5) | Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature; verify Story A already added it |
| `crates/prism-dtu-nvd/src/clone.rs` | MODIFY | Add `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` |
| `crates/prism-dtu-nvd/Cargo.toml` | MODIFY (if not done in Story A §5) | Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature; verify Story A already added it |
| `crates/prism-dtu-demo-server/src/harness.rs` | MODIFY (Story A: E-DEMO-004/005 present) | Add scenario coordination: E-DEMO-002/E-DEMO-006/E-DEMO-003 guards (in canonical order), `scenario.enabled/archetype/scenario_start_secs/stage_duration_secs` consumption, catalog derivation, IncidentTimeline construction, Arc threading |
| `.github/workflows/ci.yml` | MODIFY | Bump `EXPECTED=N` by count of new `#[non_exhaustive]` pub types (at minimum +2: `IncidentTimeline`, `IncidentStage`; read live value first) |
| `tests/external/non-exhaustive-violation/` | MODIFY | Add violation rows for new `#[non_exhaustive]` types |
| `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs` | CREATE | VP-020-K integration test: `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` — genuine end-to-end NvdState::lookup_and_count pivot; BPRL-P12-01 closure relocation from prism-dtu-cyberint |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.06.019 EC-019-003 | `now_epoch_secs < scenario_start_epoch_secs` (clock skew) | `elapsed = max(0, now - start) = 0`; stage 0; no panic (AC-005, TV-019-006) |
| EC-002 | BC-2.06.019 EC-019-004 | `now_epoch_secs` far past all thresholds (elapsed >> 600s) | Stage index saturates at `stages.len() - 1` (Containment); no index-out-of-bounds panic |
| EC-003 | BC-2.06.019 EC-019-005 | Two scenario-enabled clones with different seeds | `build_clone_pairs` returns `E-DEMO-002` before any constructor called (AC-009, TV-019-012) |
| EC-004 | BC-2.06.019 EC-019-006b/c | `stage_duration_secs` has wrong entry count (not 4 for compromised_endpoint) | `E-DEMO-003` variant with `"stage_duration_secs has N entries but archetype 'compromised_endpoint' requires exactly 4"` |
| EC-005 | BC-2.06.019 EC-019-007 | `scenario_start_secs = None` | `scenario_start_epoch_secs` set to `Utc::now().timestamp()` at `build_clone_pairs` call time; demo begins at stage 0 |
| EC-006 | BC-2.06.019 EC-019-009 | `scenario_start_secs` set to past epoch (mid-scenario start) | Correct: elapsed positive at startup; stage may start at Recon or LateralMovement; operator intentional |
| EC-007 | BC-2.06.019 EC-019-011 | Route handler called concurrently from multiple async tasks | `current_stage_index` is pure; concurrent calls safe without locking (TV-019-014) |
| EC-008 | BC-2.06.020 EC-020-002 | `entities.ioc_ips = []` (catalog produces no IPs for this seed) | No IP entries inserted; no error; non-empty catalog fields still injected normally |
| EC-009 | BC-2.06.020 EC-020-003 | Same IOC in catalog AND pre-existing registry as Benign | `HashMap::insert` overwrites; `FixtureKey::Malicious` wins; scenario injection takes priority |
| EC-010 | BC-2.06.020 EC-020-011 | `scenario.enabled = true` for operational DTUs, ThreatIntel uses static path | ThreatIntel uses `new()` (static default); scenario IOCs will NOT resolve as Malicious; valid operator config; no error |
| EC-011 | ADR-036 v2.2 §2.2 | `seed = u64::MAX` → `gen_seeded_rng(0, &org_id)` secondary stream | `wrapping_add(1) = 0`; valid; no panic |
| EC-012 | ADR-036 v2.2 §2.3 | Stage-mask filter applied when `generated_records` is empty (Story A produced empty set) | No records to filter; empty response; no panic; existing behavior preserved |
| EC-013 | ADR-036 v2.2 §2.3 | `NvdClone::new_with_scenario` returns `Err` (e.g., fixture file missing) | Error propagated through `build_clone_pairs -> anyhow::Result<Vec<ClonePair>>`; harness aborts cleanly |
| EC-014 | BC-2.06.019 EC-019-012 | `scenario.enabled = true` with `fixture_set = "dormant"` (DormantTenant archetype) — Direction 1: `scenario.archetype = "healthy"` with `scenario.enabled = true`; Direction 2: `scenario.archetype = "compromised_endpoint"` with `fixture_set = "dormant"` | `build_clone_pairs` returns `E-DEMO-003` before any clone constructor is called; guard at E-DEMO-003 position in the ordered check sequence (AC-017) |
| EC-015 | BC-2.06.019 EC-019-013 | Two scenario-enabled clones with same `seed` but different `org_ids` (e.g., CrowdStrike `org_id="<uuid-A>"`, Armis `org_id="<uuid-B>"`, both `scenario.enabled = true`) | `build_clone_pairs` returns `Err` containing `"E-DEMO-006"`, both clone names, and both org_id values before any constructor is called; guard position: after E-DEMO-002, before E-DEMO-003 (AC-018, TV-019-015) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `IncidentTimeline` | `prism-dtu-common/src/scenario/` | Pure (data struct, no I/O) | ADR-036 v2.2 §2.2 |
| `IncidentStage` | `prism-dtu-common/src/scenario/` | Pure (data struct) | ADR-036 v2.2 §2.2 |
| `StageMask` | `prism-dtu-common/src/scenario/` | Pure (data struct; internal, NOT non-exhaustive) | ADR-036 v2.2 §2.2 + BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 |
| `current_stage_index` | `prism-dtu-common/src/scenario/` | Pure (function: `(&IncidentTimeline, i64) -> usize`) | ADR-036 v2.2 §2.1 |
| `build_default_incident_timeline` | `prism-dtu-common/src/scenario/` | Pure (deterministic from catalog + thresholds) | ADR-036 v2.2 §2.2 |
| `build_clone_pairs` (scenario coordination additions) | `prism-dtu-demo-server/src/harness.rs` | Effectful (constructs clones, reads config, calls catalog + timeline builders; E-DEMO-002/E-DEMO-006/E-DEMO-003/E-DEMO-004 guards in canonical order) | ADR-036 v2.2 §2.4 + BC-2.06.019 PRE-5/PRE-6 |
| `ArmisState.timeline` / route projection | `prism-dtu-armis/src/state.rs` + routes | Effectful (HTTP handler calling pure `current_stage_index`) | ADR-036 v2.2 §2.3 |
| `CrowdstrikeState.timeline` / route projection | `prism-dtu-crowdstrike/src/state.rs` + routes | Effectful | ADR-036 v2.2 §2.3 |
| `ClarotyState.timeline` / route projection | `prism-dtu-claroty/src/state.rs` + routes | Effectful | ADR-036 v2.2 §2.3 |
| `CyberintState.timeline` / route projection | `prism-dtu-cyberint/src/state.rs` + routes | Effectful (fallible constructor) | ADR-036 v2.2 §2.3 |
| `ThreatIntelClone::new_with_scenario` | `prism-dtu-threatintel/src/clone.rs` | Effectful (constructor: populates `fixture_registry` Mutex<HashMap> at init; infallible) | ADR-036 v2.2 §2.3 + BC-2.06.020 PC-1 |
| `NvdClone::new_with_scenario` | `prism-dtu-nvd/src/clone.rs` | Effectful (fallible constructor: builds immutable `cve_registry` HashMap at init) | ADR-036 v2.2 §2.3 + BC-2.06.020 PC-3 |

---

## Forbidden Dependencies

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-dtu-threatintel` | `prism-spec-engine` | INV-PERIMETER-001 / ADR-036 v2.2 §2.5 |
| `prism-dtu-threatintel` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-threatintel` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-common` | `prism-spec-engine` | INV-PERIMETER-001 |
| Any new crate | `prism-dtu-scenario` (does not exist) | ADR-036 v2.2 §3.4 — no separate crate |

---

## SAP-1 Compliance (Structured Event Catalog)

Per CLAUDE.md §SAP-1, any `tracing::*!(event_type = "...")` emission site added in this
story requires a corresponding row in BC-2.16.002 Structured Event Catalog with event_type,
emitting module, field schema, audit role, and recurrence policy.

Expected potential emissions (implementer must enumerate actual sites):
- Potentially `event_type = "scenario.construction"` in `build_clone_pairs`
- Potentially `event_type = "scenario.stage_computed"` in route handlers

If NO new `event_type` emissions are added in this story, state explicitly in the PR description: "SAP-1: zero new event_type emissions in S-DEMO-DTU-LIVE-SCENARIO-001-B."

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v2.17 | 2026-07-08 | **Reconciling pin round (pass-4 closures): error-taxonomy v1.78→v2.26. One live version-pin cite updated: §Architecture Compliance Rules table middle column guard-order row. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.16→2.17; updated 2026-07-08 (POL-23).** |
| v2.16 | 2026-06-13 | BPRL-P24-01: AC-016 perimeter-enforcement prose corrected (structural Cargo/E0432, not the prism-query perimeter-violation gate); BC-2.06.020 v1.5→v1.6 pin-sync. Invariant requirement unchanged; counts unchanged (19 ACs / 23 RGT). |
| v2.15 | 2026-06-13 | Consistency-validator DRIFT-2/3: CyberintClone::new_with_scenario 5-arg→6-arg (+`catalog: &ScenarioEntityCatalog`) in three sites: (1) Phase-2 Cyberint constructor task — signature updated to 6-arg with note that `catalog` is threaded to `generate_with_catalog` for PC-8 CVE correlation (AC-019/VP-020-K pivot chain); (2) Phase-4 `build_clone_pairs` Cyberint call — `new_with_scenario(…, &catalog)` 6-arg with explicit note; (3) FSR table row for `crates/prism-dtu-cyberint/src/clone.rs` — description updated to 6-arg with `catalog: &ScenarioEntityCatalog` and PC-8 annotation. Aligns task/FSR with AC-019, BC-2.06.020 PC-8, STORY-INDEX D-1117 entry, and shipped code (`crates/prism-dtu-cyberint/src/clone.rs` new_with_scenario 6-arg). Other 4 operational clones (Armis/CrowdStrike/Claroty/ThreatIntel/NVD) 5-arg descriptions unchanged. No behavior/count change (19 ACs / 23 RGT). |
| v2.14 | 2026-06-13 | BC-2.06.020 v1.4→v1.5 pin-sync (BPRL-P22-01: VP Anchors prose A-H→A-L / 8→12 VPs; no behavior change). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget BC-2.06.020 context row. Story spec self-reference v2.12→v2.14. counts unchanged (19 ACs / 23 RGT). |
| v2.13 | 2026-06-13 | BPRL-P15-01 closure: Phase-6 gate instruction stale RGT count 19→23 (canonical count per frontmatter/table/STORY-INDEX). Exhaustive count-prose sweep (TD-VSDD-060): all other `\b19\b` hits classified as test-index labels, AC-count (correct), or historical changelog rows — no additional fixes required. No behavior/count change; red_gate_tests stays 23, acceptance_criteria_count stays 19. |
| v2.12 | 2026-06-13 | BPRL-P14-01 closure: AC-019 baseline RNG range literal 0..100000→0..10000 (matches ^CVE-9999-\d{4}$ invariant + code); BC-2.06.020 v1.3→v1.4 pin-sync. No behavior change; counts unchanged (19 ACs / 23 RGT). |
| v2.11 | 2026-06-13 | BPRL-P12-01 closure: VP-020-K integration test relocated cyberint→demo-server (genuine NvdState::lookup_and_count end-to-end pivot; redundant duplicate-named cyberint membership test removed); red_gate_tests 23 UNCHANGED. AC-019 bullet updated: crate cite changed from `prism-dtu-cyberint` (or `prism-dtu-demo-server`) → definitively `prism-dtu-demo-server` with test file `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs`. RGT table row 22 crate column: `prism-dtu-cyberint` → `prism-dtu-demo-server`; test file added inline. FSR table: new CREATE row for `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs`. Token Budget story spec v2.10→v2.11. No BC version change (BC-2.06.020 stays v1.3). version 2.10→2.11. |
| v2.10 | 2026-06-12 | D-1117 — AC-019 added: Cyberint alert CVE correlation + SEC-001 collision-safety; BC-2.06.020 v1.2→v1.3. AC-019 covers: (a) scenario mode — every Cyberint CVE-surface record's `cve_id` ∈ `catalog.device_cves` (cyclic when record count > catalog size); end-to-end pivot chain: `cve_id → NvdState::lookup_and_count → base_score >= 7.0`; (b) baseline/non-scenario mode — `cve_id` uses `CVE-9999-` namespace (collision-safe; intentionally non-pivotable; NVD 404 is correct); (c) universal: no real-year (`CVE-20xx-*`) CVE IDs emitted from any generation path. Four Red Gate tests added (verbatim names from implementer commit f0b6b8c7): `test_BC_2_06_020_cyberint_baseline_cve_uses_cve_9999_namespace` (TV-020-011/VP-020-I), `test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog` (TV-020-012/VP-020-J), `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` (TV-020-013/VP-020-K), `test_BC_2_06_020_cyberint_scenario_cyclic_catalog_assignment` (TV-020-014/VP-020-L). SEC-001 note: `catalog.device_cves` uses `CVE-9999-{:05}` per `gen_device_cves`; `test_sec_001_device_cves_use_unambiguous_synthetic_year` covers catalog side; AC-019 covers Cyberint generator side. BC table row + Token Budget updated v1.2→v1.3. verification_properties extended with VP-020-I..L. acceptance_criteria_count 18→19; red_gate_tests 19→23. version 2.9→2.10. |
| v2.9 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.6→v1.7 pin-sync (BPRL-P7-01 inventory-prose correction; POL-23). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget row. No AC changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED. version 2.8→2.9. |
| v2.8 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.5→v1.6 pin-sync (BPRL-P6-01 Claroty devices Route Coverage row + exhaustive inventory verification note; POL-23). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget row. No AC changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED. version 2.7→2.8. |
| v2.7 | 2026-06-12 | Micro-sweep — BC-2.06.019 v1.4→v1.5 pin-sync (BPRL-P5-01 Route Coverage Table corrections + PC-4 5-arg prose; POL-23). Two live pin sites updated: §Behavioral Contracts BC table row and §Token Budget row. No AC changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED. version 2.6→2.7. |
| v2.6 | 2026-06-12 | Pin-sync — BC-2.06.019 v1.3→v1.4 per D-1109. PC-4 amended: per-sensor IOC-surface matrix (Cyberint/CrowdStrike detections YES but deferred to S-DEMO-ENRICHMENT-PIVOT-003; Armis/Claroty permanently excluded); IOC-stamping deferred to S-DEMO-ENRICHMENT-PIVOT-003; detections/armis-alerts stage-guards added in-PR at bc0f36c5. Two live pin sites updated: §Behavioral Contracts BC table row (v1.3→v1.4) and §Token Budget row (v1.3→v1.4). No AC changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED. version 2.5→2.6. |
| v2.5 | 2026-06-12 | Pin-sync — BC-2.06.020 v1.1→v1.2 (B-P7-02 closure). OBS-2 micro-burst (BC-INDEX v6.29) bumped BC-2.06.020 v1.1→v1.2 after the story body was last pinned at v2.4; v2.4 propagated the BC-2.06.019 v1.2→v1.3 bump but missed the 020 bump. Both live pin sites updated: §Behavioral Contracts BC table row (line 147) and §Token Budget row. No AC changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED. version 2.4→2.5. STORY-INDEX row v2.4→v2.5. |
| v2.4 | 2026-06-12 | Micro-amendment — LOCAL pass-5 findings B-P5-02 (story half) + B-P5-04. B-P5-02: AC-010 BC cite corrected — PO burst a45130fd renumbered BC-2.06.019 preconditions contiguous 1-8 (PRE-6=org_id guard, PRE-7=archetype guard, PRE-8=build-before-start); AC-010 trace was `precondition 6` (archetype guard) → corrected to `precondition 7`. POL-29 sweep confirmed: all remaining `precondition 6` / `PRE-6` cites in story body correctly reference the org_id guard (unchanged). B-P5-04: ArmisClone::new_with_scenario corrected from `-> Self` (infallible) to `-> anyhow::Result<Self>` (fallible) — Armis new_with_seed_anchored is fallible; this flows through to new_with_scenario. Fixed everywhere: risk_mitigations line 69 (expanded fallible list: NVD + Armis + Cyberint), risk_mitigations line 80 (per-clone return-type table: CrowdStrike/Claroty `-> Self`; Armis/Cyberint/NVD `-> anyhow::Result<Self>`; ThreatIntel `-> Self`), AC-007 body (added `fallible, -> anyhow::Result<Self>` qualifier), Phase 2 task (Armis constructor return type), FSR row for prism-dtu-armis/src/clone.rs. CrowdStrike/Claroty/ThreatIntel `-> Self` are correct and unchanged. BC-2.06.019 version pin bumped v1.2→v1.3 in BC table and Token Budget (PO burst a45130fd). error-taxonomy pin bumped v1.77→v1.78 in AC-018 and Architecture Compliance Rules (PO burst a45130fd). acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED; version 2.3→2.4. |
| v2.3 | 2026-06-12 | Micro-amendment — PO burst 13c1b17a OBS-1 work-order: BC-2.06.019 v1.2 PRE-6 org_id-equality guard added. AC-018 (traced to BC-2.06.019 PRE-6 / EC-019-013 / TV-019-015): `build_clone_pairs` rejects scenario-enabled clones with mismatched `org_id` values → `Err` containing `"E-DEMO-006"` with both clone names and org_id values, BEFORE any clone constructor. Verbatim message from error-taxonomy v1.77: `"demo-server: E-DEMO-006: scenario clones '{clone_a}' (org_id={org_id_a}) and '{clone_b}' (org_id={org_id_b}) have different org_ids; cross-DTU coherence requires all scenario-enabled clones to share the same org_id"`. Full guard order updated everywhere it appears (AC-017, Architecture Compliance Rules, Phase-4 tasks, NIT-1 verification): `E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype) → E-DEMO-004 (missing org_id)`. Red Gate test 19 (`test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones`) added. EC-015 (EC-019-013) added to Edge Cases table. BC-2.06.019 version pin bumped v1.1→v1.2 in BC table. VP-019-I added to verification_properties frontmatter. Token Budget story-spec ~7 700→~8 200; BC-2.06.019 ~3 000→~3 200; test-stubs 18→19 × 40 lines; total ~36 400→~37 200. acceptance_criteria_count 17→18; red_gate_tests 18→19; version 2.2→2.3. STORY-INDEX row synced v2.2→v2.3. |
| v2.2 | 2026-06-12 | Micro-amendment — LOCAL pass-3 finding B-P3-01 scope closure. Added AC-017 traced to BC-2.06.019 EC-019-012: `build_clone_pairs` rejects archetype/fixture_set contradictions with E-DEMO-003 before any constructor. Two directions covered: (1) `scenario.archetype = "healthy"` with `scenario.enabled = true` (archetype does not support 5-stage progression); (2) `scenario.archetype = "compromised_endpoint"` × `fixture_set = "dormant"` (CompromisedEndpoint × DormantTenant incoherence). Guard placement documented: E-DEMO-002 → E-DEMO-003 → E-DEMO-004. EC table row EC-014 (EC-019-012). Red Gate test 18 (`test_BC_2_06_019_e_demo_003_archetype_fixture_set_contradiction`). Phase 4 E-DEMO-003 guard task extended with four sub-cases (a–d). Token Budget story-spec ~7 200→~7 700; test-stubs 16→18 × 40 lines; total ~36 000→~36 400. acceptance_criteria_count 16→17; red_gate_tests 17→18; version 2.1→2.2. STORY-INDEX row updated v2.1→v2.2. |
| v2.1 | 2026-06-12 | Amendment burst — remove-uncertainty findings closure (ADR-036 v2.3 work-order). HIGH: time_anchor wiring — all 4 generator-backed clone constructors updated from 4-arg to 5-arg `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline), time_anchor)`; `new_with_scenario` body must call `new_with_seed_anchored(seed, archetype, org_id, time_anchor)` (NOT 3-arg `new_with_seed`); `time_anchor` derived ONCE in `build_clone_pairs` from `scenario_start_epoch_secs`; `Utc::now()` called AT MOST ONCE for the None path; ADR-036 version bumped v2.2→v2.3 throughout; Architecture Compliance Rules + risk_mitigations extended; body intro ADR-036 v2.3 amendment block added; AC-007/AC-008 updated to 5-arg form; AC-011 None-path determinism note added. MEDIUM-B: DormantTenant regression guard — Phase 2 handler tasks updated with explicit three-way composition rule (fixture_gen_seeded flag, NOT generated_records.is_empty()); Red Gate test 17 added (dormant_tenant guard); red_gate_tests 16→17; Architecture Compliance Rules row added. MEDIUM-C: chrono feature-gating in claroty/cyberint — Phase 2 Claroty + Cyberint tasks updated with explicit `#[cfg(feature="fixture-gen")]` gating requirement; Phase 6 cargo-check verification tasks added; Library & Framework Requirements chrono row updated; Architecture Compliance Rules row added; FSR rows for claroty/cyberint updated. LOW-D1: E-DEMO-003 config.rs doc comment fix task added to Phase 4 E-DEMO-003 guard step. LOW-D2: with_admin_token phrasing clarified — it is `ThreatIntelState::with_admin_token` (state.rs:38) called from the clone constructor body, not a `ThreatIntelClone` method; full constructor body replication described. LOW-D3: AC-014 cvss_metric_v31 Option<Vec<>> unwrap note added. Token Budget ~35 100→~36 000. version 2.0→2.1; modified timestamp updated. |
| v2.0 | 2026-06-12 | T5 materialization burst (D-1090 full-autonomy grant). Story A MERGED (PR #181 develop@c287b00d) — status draft→ready. CONTRACT-COMPLETENESS FRONT-LOAD verified: all 4 mechanisms fully specified in BC-2.06.019 v1.1 + BC-2.06.020 v1.1 + ADR-036 v2.2. NIT-1 folded in: E-DEMO-004 trigger/message reconciliation documented in frontmatter risk_mitigations + story body + §Tasks (no change to error message needed; Story A guard covers scenario path). NIT-2 folded in: all 4 ScenarioConfig fields explicitly noted as consumed in Story B in body intro + §Tasks + frontmatter risk_mitigations. Architecture Compliance Rules extended with: ADR-036 v2.2 StageMask #[non_exhaustive] conflict note (BC-2.06.019 wins); E-DEMO-002/003/004 guard order constraint; NIT-1 E-DEMO-004 order rule; gen_seeded_rng two-arg alias rule (ADR-036 v2.1 U-A-01). AC-002 implementation formula added verbatim. AC-003 TV source cites (BC-2.06.019 TV-019-001..005). AC-012 gen_seeded_rng alias correction. AC-013 lock-and-insert-then-release description. AC-014 exact NvdState::lookup_and_count method name + fallibility note. AC-015 org_slug formula cited verbatim. Token Budget updated (+2 500 for additional context). Pre-flight §Tasks items added (substrate read checklist). version 1.0→2.0; modified timestamp updated; BC-INDEX Story anchor update pending state-manager. |
| v1.0 | 2026-06-09 | Initial authoring per ADR-036 v2.0 §8 story split (D-1077). Derived from S-DEMO-DTU-LIVE-SCENARIO-001 Group B+C ACs with substrate corrections: stage_duration_secs 4-entry array; NvdState::lookup_and_count → NvdState::cve_registry immutable HashMap; CVSS path metrics.cvss_metric_v31[0].cvss_data.base_score; NvdClone::new_with_scenario fallible; ThreatIntelClone::new_with_scenario infallible; canonical IDs "dev-{8hex}-{seed}-{n}" per ADR-036 §2.2. Depends on Story A merge. |
