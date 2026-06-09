---
document_type: story
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
title: "Scenario Progression + Enrichment Correlation — Unfolding-Attack Live Demo"
wave: 5
epic_id: E-DEMO
priority: P2
# BC status: ready pending Story A merge + remove-uncertainty run
# Set status to ready once S-DEMO-DTU-LIVE-SCENARIO-001-A merges to develop AND
# remove-uncertainty confirms no substrate changes introduced by Story A implementation.
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-06-09T00:00:00Z"
created: "2026-06-09"
modified: "2026-06-09T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates including prism-dtu-demo-server,
#   prism-dtu-common, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty,
#   prism-dtu-cyberint, prism-dtu-threatintel, and prism-dtu-nvd per ARCH-INDEX Subsystem
#   Registry. The scenario progression engine is demo infrastructure entirely within SS-01.
#   Decision anchor: ADR-036 subsystems_affected: [SS-01].
target_module: prism-dtu-common
crates_touched: [prism-dtu-common, prism-dtu-demo-server, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-threatintel, prism-dtu-nvd]
behavioral_contracts: [BC-2.06.019, BC-2.06.020]
verification_properties: [VP-019-A, VP-019-B, VP-019-C, VP-019-D, VP-019-E, VP-019-F, VP-019-G, VP-019-H, VP-020-A, VP-020-B, VP-020-C, VP-020-D, VP-020-E, VP-020-F, VP-020-G, VP-020-H]
depends_on:
  - S-DEMO-DTU-LIVE-SCENARIO-001-A
  # Dependency anchor: Story A delivers new_with_seed constructors + generated_records in state
  # for all 4 generator-backed clones. Story B's new_with_scenario constructors project a
  # StageMask over the generated_records introduced by Story A. Without generated_records in
  # the state struct (Story A scope), there is no substrate to project over. This is a hard
  # build-order dependency: Story B cannot compile without Story A's state field additions.
blocks: []
points: 7
# Points justification (ADR-036 §8 Story B estimate):
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
acceptance_criteria_count: 16
red_gate_tests: 16
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "stage_duration_secs 4-entry array: [60, 180, 360, 600] for stages 1-4; stage 0 always at 0. BC-2.06.019 and ADR-036 §2.2 are authoritative. Any test using 5-entry arrays or different thresholds is wrong."
  - "current_stage_index is a pure function: no side effects, no shared mutable state, no tokio::spawn, no Arc<AtomicU64> progression counter. ADR-036 §2.1 mandates this."
  - "NvdClone::new_with_scenario returns anyhow::Result<Self> (fallible, like NvdClone::new()). ThreatIntelClone::new_with_scenario is infallible (like ThreatIntelClone::new()). Test must handle Result for NVD."
  - "CVSS path is CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score (f64) >= 7.0, NOT metrics.score or any flat field. Implementer MUST read crates/prism-dtu-nvd/src/types.rs before writing the constructor."
  - "Cross-DTU entity coherence: primary_device_id_cs and primary_device_id_armis in ScenarioEntityCatalog use the same org_slug derivation (hex of org_id.as_bytes()[0..4]). They MUST match across Armis and CrowdStrike queries. ADR-036 §2.2."
  - "Secondary RNG stream independence (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001): seeded_rng(seed.wrapping_add(1), org_id) for catalog derivation must be a SEPARATE ChaCha20Rng instance from the primary generator stream. Implementing build_clone_pairs must NOT advance the primary stream before catalog derivation."
  - "StageMask must NOT carry #[non_exhaustive] — it is internal to prism-dtu-common and must be exhaustively constructible within the crate (BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001)."
  - "#[non_exhaustive] EXPECTED bump: new pub types IncidentTimeline, IncidentStage, StageMask (if exported) added in this story; implementer must read live EXPECTED= from ci.yml and increment by exact new-type count."
  - "reqwest::Client timeout: .timeout(Duration::from_secs(30)) in all new integration test HTTP clients per CLAUDE.md conventions."
  - "INV-PERIMETER-001: ThreatIntel and NVD new_with_scenario constructors must not import prism-spec-engine/prism-sensors/prism-query. prism-dtu-common dep added by Story A; no new cross-DTU perimeter changes needed."
traces_to: [D-1077, ADR-036]
supersedes: []
---

# S-DEMO-DTU-LIVE-SCENARIO-001-B: Scenario Progression + Enrichment Correlation

Add the `IncidentTimeline` temporal layer on top of Story A's seeded generator substrate.
Implements BC-2.06.019 (pure-function-of-time stage engine with 5 stages) and BC-2.06.020
(ThreatIntel IOC injection + NVD CVE injection from the shared `ScenarioEntityCatalog`).
Together with Story A, this delivers the complete multi-client SOC demo live-scenario layer.

**Depends on:** S-DEMO-DTU-LIVE-SCENARIO-001-A (must merge to develop first). Status remains
`draft` until Story A merges and `remove-uncertainty` confirms the Story A implementation
matches the substrate assumptions in this spec.

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
| BC-2.06.019 | Demo-Server Scenario Progression — Pure-Function Temporal Stage Advancement | INV-PROGRESSION-REPRODUCIBILITY-001, INV-STAGE-MONOTONICITY-001, INV-STAGE-MASK-COMPLETENESS-001, INV-SCENARIO-DISABLED-COMPAT-001, INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 |
| BC-2.06.020 | Demo-Server Enrichment Correlation — Scenario IOCs/CVEs Resolve in ThreatIntel/NVD | INV-THREATINTEL-IOC-CORRELATION-001, INV-NVD-CVE-CORRELATION-001, INV-CROSS-DTU-ENTITY-COHERENCE-001, INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001, INV-PERIMETER-COMPLIANCE-001, INV-CONSTRUCTION-TIME-INJECTION-001 |

---

## Acceptance Criteria

### Group A — Scenario Progression Types (BC-2.06.019)

**AC-001 — IncidentTimeline + IncidentStage + StageMask types defined in prism-dtu-common**
(traces to BC-2.06.019 precondition 1 and ADR-036 §2.2)

Given `prism-dtu-common` compiled with `feature = "fixture-gen"`,
when the types `IncidentTimeline`, `IncidentStage`, and `StageMask` are imported,
then:
- `IncidentTimeline` is `#[non_exhaustive]`, `#[derive(Clone, Debug)]`, with fields `entities: ScenarioEntityCatalog`, `stages: Vec<IncidentStage>`, `scenario_start_epoch_secs: i64`
- `IncidentStage` is `#[non_exhaustive]`, `#[derive(Clone, Debug)]`, with fields `name: &'static str`, `activates_after_secs: u64`, `visible_entity_mask: StageMask`
- `StageMask` is NOT `#[non_exhaustive]` (it is internal and must be exhaustively constructible within the crate per INV-STAGE-MASK-COMPLETENESS-001), with 6 bool fields: `primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`
- Default `CompromisedEndpoint` timeline has 5 stages: Baseline (0s), Recon (60s), LateralMovement (180s), Exfil (360s), Containment (600s); `stage_duration_secs` array has 4 entries

Red Gate: `test_BC_2_06_019_timeline_types_non_exhaustive_and_structure`

**AC-002 — current_stage_index is a pure function of (timeline, now_epoch_secs)**
(traces to BC-2.06.019 invariant INV-PROGRESSION-REPRODUCIBILITY-001 and postcondition 2)

Given a `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize` function,
when called multiple times with the same `(timeline, now_epoch_secs)` pair from any number of concurrent callers,
then all invocations return the same `usize` with no shared mutable state, no locks, no tokio spawn, and no side effects.

Red Gate: `test_BC_2_06_019_stage_index_pure_function_reproducible`

**AC-003 — Stage boundary correctness: 5 stages at default thresholds**
(traces to BC-2.06.019 postcondition 2 and postcondition 3)

Given `scenario_start_epoch_secs = T` and default `CompromisedEndpoint` stage thresholds `[60, 180, 360, 600]` (4-entry `stage_duration_secs` array):
- `now = T + 0s` → stage 0 (Baseline; `activates_after_secs = 0`)
- `now = T + 30s` → stage 0 (Baseline; elapsed 30 < 60)
- `now = T + 90s` → stage 1 (Recon; elapsed 90 >= 60)
- `now = T + 200s` → stage 2 (LateralMovement; elapsed 200 >= 180)
- `now = T + 400s` → stage 3 (Exfil; elapsed 400 >= 360)
- `now = T + 700s` → stage 4 (Containment; elapsed 700 >= 600; saturates at max stage)

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

Red Gate: `test_BC_2_06_019_clock_skew_clamped_to_baseline`

**AC-006 — INV-STAGE-MASK-COMPLETENESS-001: all 6 StageMask fields explicitly set in every stage**
(traces to BC-2.06.019 invariant INV-STAGE-MASK-COMPLETENESS-001)

Given the default `CompromisedEndpoint` `IncidentTimeline` (5 stages),
when each `IncidentStage.visible_entity_mask` is inspected,
then every stage has explicit bool values for all 6 fields (`primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`) — no field is left uninitialized or implicitly defaulted.

Red Gate: `test_BC_2_06_019_stage_mask_completeness_all_6_fields`

**AC-007 — Armis new_with_scenario: primary_device not visible at stage 0; visible at stage 1+**
(traces to BC-2.06.019 postcondition 4)

Given an Armis clone constructed with `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline))` and `scenario_start_secs = T`:
- At `now = T + 30s` (stage 0 / Baseline): `GET /api/v1/devices` response does NOT contain `catalog.primary_device_id_armis`
- At `now = T + 90s` (stage 1 / Recon): `GET /api/v1/devices` response CONTAINS `catalog.primary_device_id_armis`; lateral device IDs are NOT present

Red Gate: `test_BC_2_06_019_armis_primary_device_stage_visibility`

**AC-008 — CrowdStrike new_with_scenario: containment_status = "contained" only at stage 4**
(traces to BC-2.06.019 postcondition 4)

Given a CrowdStrike clone with `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline))` and `scenario_start_secs = T`:
- At `now = T + 200s` (stage 2 / LateralMovement): the device record for `primary_device_id_cs` shows `containment_status = "normal"` (or equivalent non-contained value)
- At `now = T + 700s` (stage 4 / Containment): the same device record shows `containment_status = "contained"`

Red Gate: `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only`

**AC-009 — E-DEMO-002: mismatched seeds across scenario-enabled clones rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-002)

Given `clones.crowdstrike.seed = 100` and `clones.armis.seed = 200`, both with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-002"` and both clone names and seed values, before any clone constructor is called.

Red Gate: `test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones`

**AC-010 — E-DEMO-003: unrecognized scenario archetype rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-003)

Given `scenario.archetype = "unknown_archetype_value"` for any clone with `scenario.enabled = true`, when `build_clone_pairs` runs, then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` and the clone name and the invalid archetype string.

Red Gate: `test_BC_2_06_019_e_demo_003_unrecognized_archetype`

**AC-011 — INV-SCENARIO-DISABLED-COMPAT-001: scenario.enabled=false is byte-identical to BC-2.06.018 seeded path**
(traces to BC-2.06.019 invariant INV-SCENARIO-DISABLED-COMPAT-001)

Given a clone constructed with `scenario.enabled = false` (or absent `[clones.*.scenario]` block) and `seed = 42`, `fixture_set = "default"`, when queried at any fixed request path, then responses are byte-identical to the Story A `new_with_seed(42, HealthyOtEnvironment, default_org)` responses; `timeline: Option<Arc<IncidentTimeline>>` is `None` in the clone state.

Red Gate: `test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path`

**AC-012 — INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001: catalog derivation does not shift generator output**
(traces to BC-2.06.019 invariant INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001)

Given two harness constructions with same `seed = 100, org_id = "<uuid>"`: one with `scenario.enabled = false` and one with `scenario.enabled = true`, when both are queried at the same device endpoint, then the underlying `FixtureSet` device records (from Story A `generated_records`) are byte-identical — the catalog derivation via `seeded_rng(seed.wrapping_add(1), org_id)` has NOT consumed state from the primary generator stream `seeded_rng(seed, org_id)`.

Red Gate: `test_BC_2_06_019_secondary_rng_independence_no_primary_shift`

---

### Group B — Enrichment Correlation (BC-2.06.020)

**AC-013 — INV-THREATINTEL-IOC-CORRELATION-001: all scenario IOCs resolve as Malicious in ThreatIntel**
(traces to BC-2.06.020 invariant INV-THREATINTEL-IOC-CORRELATION-001 and postcondition 1)

Given `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` (infallible),
when lookup requests are issued for each of `entities.ioc_ips[0]`, `entities.ioc_domains[0]`, and `entities.ioc_hashes[0]`,
then each response contains `threat_is_known_malicious = true` and `threat_score >= 75`;
and `ThreatIntelState.fixture_registry` (a `Mutex<HashMap<String, FixtureKey>>`) has all IOC entries with `FixtureKey::Malicious` pre-populated at construction time.

Red Gate: `test_BC_2_06_020_threatintel_ioc_correlation_all_types`

**AC-014 — INV-NVD-CVE-CORRELATION-001: all scenario CVEs resolve with HIGH CVSS in NVD**
(traces to BC-2.06.020 invariant INV-NVD-CVE-CORRELATION-001 and postcondition 3)

Given `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (fallible),
when a CVE lookup is issued for `entities.device_cves[0]`,
then the response contains a `CveRecord` where `metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0` (f64) and `metrics.cvss_metric_v31[0].cvss_data.base_severity` is `"HIGH"`;
and `NvdState.cve_registry` is an immutable `HashMap<String, CveRecord>` (NOT Mutex-wrapped) built entirely at construction time.

Red Gate: `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score`

**AC-015 — INV-CROSS-DTU-ENTITY-COHERENCE-001: primary_device_id consistent across Armis, CrowdStrike, Claroty at stage >= 1**
(traces to BC-2.06.020 invariant INV-CROSS-DTU-ENTITY-COHERENCE-001 and postcondition 5)

Given three clones (Armis, CrowdStrike, Claroty) all constructed with the same `(seed=100, org_id="<uuid-with-bytes-starting-deadbeef>", scenario.enabled=true)` and `scenario_start_secs = T`, when all three are queried at `now = T + 90s` (stage 1 / Recon), then:
- Armis `/api/v1/devices` response contains a device with ID `catalog.primary_device_id_armis = "dev-deadbeef-100-0"`
- CrowdStrike `/devices/entities/devices/v2` response contains a device with ID `catalog.primary_device_id_cs = "dev-deadbeef-100-0"`
- Claroty device/asset response contains a device with ID following the same canonical format

Red Gate: `test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones`

**AC-016 — INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001**
(traces to BC-2.06.020 invariants INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 and INV-PERIMETER-COMPLIANCE-001)

Given a ThreatIntel clone constructed with `new_with_scenario` and a non-scenario IP `"192.0.2.1"` (not in `ioc_ips`),
when a lookup is issued for `"192.0.2.1"`,
then the response is identical to `ThreatIntelClone::new().lookup("192.0.2.1")` — scenario injection is strictly additive.
AND the compile-fail gate `tests/external/perimeter-violation/` passes with zero new violations after `new_with_scenario` constructors are added.

Red Gate: `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate`

---

## Red Gate Test Plan

All tests written FAIL-first per SID-1. Unit tests in `#[cfg(test)] mod tests` blocks or
integration tests in `crates/<crate>/tests/`. No `#[ignore]` unless external-service
dependency cited.

| # | Test Name | Crate | BC | Type |
|---|-----------|-------|-----|------|
| 1 | `test_BC_2_06_019_timeline_types_non_exhaustive_and_structure` | prism-dtu-common | BC-2.06.019 PRE-1 / ADR-036 §2.2 | unit |
| 2 | `test_BC_2_06_019_stage_index_pure_function_reproducible` | prism-dtu-common | BC-2.06.019 INV-PROGRESSION-REPRODUCIBILITY-001 | unit |
| 3 | `test_BC_2_06_019_stage_boundary_5_thresholds_correct` | prism-dtu-common | BC-2.06.019 PC-2, PC-3 | unit |
| 4 | `test_BC_2_06_019_stage_index_monotonic_over_time` | prism-dtu-common | BC-2.06.019 INV-STAGE-MONOTONICITY-001 | unit |
| 5 | `test_BC_2_06_019_clock_skew_clamped_to_baseline` | prism-dtu-common | BC-2.06.019 EC-019-003 | unit |
| 6 | `test_BC_2_06_019_stage_mask_completeness_all_6_fields` | prism-dtu-common | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 | unit |
| 7 | `test_BC_2_06_019_armis_primary_device_stage_visibility` | prism-dtu-armis | BC-2.06.019 PC-4 | integration |
| 8 | `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only` | prism-dtu-crowdstrike | BC-2.06.019 PC-4 | integration |
| 9 | `test_BC_2_06_019_e_demo_002_seed_mismatch_across_scenario_clones` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-002 | unit |
| 10 | `test_BC_2_06_019_e_demo_003_unrecognized_archetype` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-003 | unit |
| 11 | `test_BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path` | prism-dtu-demo-server | BC-2.06.019 INV-SCENARIO-DISABLED-COMPAT-001 | regression |
| 12 | `test_BC_2_06_019_secondary_rng_independence_no_primary_shift` | prism-dtu-common | BC-2.06.019 INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 | unit |
| 13 | `test_BC_2_06_020_threatintel_ioc_correlation_all_types` | prism-dtu-threatintel | BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 | unit |
| 14 | `test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score` | prism-dtu-nvd | BC-2.06.020 INV-NVD-CVE-CORRELATION-001 | unit |
| 15 | `test_BC_2_06_020_cross_dtu_entity_coherence_stage1_all_three_clones` | prism-dtu-demo-server | BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 | integration |
| 16 | `test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate` | prism-dtu-threatintel + tests/external/perimeter-violation | BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001 | unit + compile-fail |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5 500 |
| ADR-036 v2.0 (full) | ~5 500 |
| BC-2.06.019 (full) | ~3 000 |
| BC-2.06.020 (full) | ~3 000 |
| Story A spec (substrate context) | ~3 000 |
| prism-dtu-common/src/scenario/mod.rs (from Story A + extensions) | ~1 500 |
| prism-dtu-demo-server/src/{harness,config}.rs (post-Story-A state) | ~2 000 |
| prism-dtu-armis/src/{state,clone}.rs (post-Story-A state) | ~1 500 |
| prism-dtu-crowdstrike/src/{state,clone}.rs (post-Story-A state) | ~1 500 |
| prism-dtu-threatintel/src/{state,clone}.rs | ~900 |
| prism-dtu-nvd/src/{state,clone,types}.rs | ~1 000 |
| ci.yml (EXPECTED line) | ~200 |
| Test files (16 stubs × ~40 lines each) | ~2 000 |
| Tool outputs (nextest, clippy, compile-fail) | ~2 000 |
| **Total estimate** | **~32 600** |

At ~200k context window, this is ~16% — within the 20-30% ceiling.

---

## Tasks

Implementation checklist (TDD order — write failing tests before each implementation step):

**Phase 1: prism-dtu-common scenario module — IncidentTimeline layer**

- [ ] Read `crates/prism-dtu-common/src/scenario/mod.rs` (Story A substrate) before editing
- [ ] Add `IncidentTimeline` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`) with fields: `entities: ScenarioEntityCatalog`, `stages: Vec<IncidentStage>`, `scenario_start_epoch_secs: i64`
- [ ] Add `IncidentStage` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`) with fields: `name: &'static str`, `activates_after_secs: u64`, `visible_entity_mask: StageMask`
- [ ] Add `StageMask` (NOT `#[non_exhaustive]`; internal struct, `#[derive(Clone, Debug)]`) with 6 bool fields: `primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves`
- [ ] Implement `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize` pure function per ADR-036 §2.2: `elapsed = max(0, now - start)` as `u64`; iterate stages; return index of last stage with `activates_after_secs <= elapsed`
- [ ] Implement `build_default_incident_timeline(catalog: ScenarioEntityCatalog, start_secs: i64, stage_duration_secs: &[u64]) -> IncidentTimeline`: if `stage_duration_secs` is empty, use default `[60, 180, 360, 600]`; construct 5 `IncidentStage` instances with `activates_after_secs` from array + stage 0 at 0; set `StageMask` per ADR-036 §2.2 stage table (Baseline: primary only, no alerts; Recon: primary + low alerts; LateralMovement: primary + lateral + ioc_hashes; Exfil: all devices + ioc_ips + ioc_domains; Containment: primary contained + all IOCs + device_cves)
- [ ] Write unit tests 1-6 (FAIL first): types structure, pure function, boundaries, monotonicity, clock skew, mask completeness

**Phase 2: Per-clone new_with_scenario constructors**

- [ ] Read CrowdStrike `state.rs`, `clone.rs` (Story A state) before editing — confirm `generated_devices` + `generated_detections` fields present
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `CrowdstrikeState`
- [ ] Add `CrowdstrikeClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, timeline: Arc<IncidentTimeline>) -> Self` under `#[cfg(feature = "fixture-gen")]`: calls `new_with_seed(seed, org_id)` internally, then sets `timeline = Some(timeline)`
- [ ] Modify `routes/hosts.rs`: when `timeline.is_some()`, call `current_stage_index` and apply `StageMask` filter on top of `generated_devices`; when `timeline.is_none()`, use Story A dual-path (generated_devices when non-empty, else stateful-write-target)
- [ ] Write unit test 8 (FAIL first): CrowdStrike containment stage4 only

- [ ] Read Armis `state.rs`, `clone.rs` (Story A state) before editing
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `ArmisState`
- [ ] Add `ArmisClone::new_with_scenario(seed: u64, archetype: Archetype, org_id: OrgId, org_slug: &str, timeline: Arc<IncidentTimeline>) -> Self`
- [ ] Modify Armis `routes/devices.rs`: scenario projection (StageMask filter) when `timeline.is_some()`
- [ ] Write unit test 7 (FAIL first): Armis primary device stage visibility

- [ ] Read Claroty `state.rs`, `clone.rs` (Story A state) before editing
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `ClarotyState`
- [ ] Add `ClarotyClone::new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>) -> Self`
- [ ] Modify Claroty route handlers: scenario projection when `timeline.is_some()`

- [ ] Read Cyberint `state.rs`, `clone.rs` (Story A state) before editing
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `CyberintState`
- [ ] Add `CyberintClone::new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>) -> anyhow::Result<Self>` (fallible)
- [ ] Modify Cyberint route handlers: scenario projection

**Phase 3: Enrichment clone constructors**

- [ ] Read `crates/prism-dtu-threatintel/src/state.rs` + `clone.rs` fully before editing — confirm `fixture_registry: Mutex<HashMap<String, FixtureKey>>`
- [ ] Add `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` (infallible): calls `with_admin_token`, then pre-populates `fixture_registry` with all `entities.ioc_ips`, `entities.ioc_domains`, `entities.ioc_hashes` as `FixtureKey::Malicious`; must NOT import `prism-spec-engine`/`prism-sensors`/`prism-query`
- [ ] Write unit tests 13, 16 (FAIL first): IOC correlation + passthrough

- [ ] Read `crates/prism-dtu-nvd/src/state.rs` + `clone.rs` + `types.rs` fully before editing — confirm `cve_registry: HashMap<String, CveRecord>` (immutable after construction); confirm CVSS path: `CveRecord.metrics: CveMetrics` → `.cvss_metric_v31: Option<Vec<CvssMetricV31>>` → `[0].cvss_data: CvssData` → `.base_score: f64`, `.base_severity: String`
- [ ] Add `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` (fallible, same return type as `new()`): build `cve_registry` by loading base fixtures (same as `new()`), then insert synthetic `CveRecord` entries for each CVE in `entities.device_cves` with `base_score = 8.1`, `base_severity = "HIGH"`, filling all required `CvssData` / `CvssMetricV31` struct fields from the real type definitions
- [ ] Write unit test 14 (FAIL first): NVD CVE high CVSS

**Phase 4: build_clone_pairs scenario coordination**

- [ ] Read `crates/prism-dtu-demo-server/src/harness.rs` (Story A state) before editing
- [ ] Add E-DEMO-002 guard: if multiple scenario-enabled clones have different `seed` values, return `Err(E-DEMO-002)` before any constructor called
- [ ] Add E-DEMO-003 guard: if `scenario.archetype` is not `"compromised_endpoint"` (or other recognized value), return `Err(E-DEMO-003)` with clone name and invalid value
- [ ] When `scenario.enabled = true`: derive `org_slug = org_slug_from_org_id(&org_id)` (already done in Story A; confirm it exists); derive `ScenarioEntityCatalog` via `build_scenario_entity_catalog(seed, &org_id)` using secondary RNG `seeded_rng(seed.wrapping_add(1), org_id)`; construct `IncidentTimeline` from catalog + `stage_duration_secs` (operator override or defaults); wrap in `Arc::new(timeline)`; call `new_with_scenario(seed, archetype, org_id, Arc::clone(&timeline))` for 4 generator-backed clones; call `ThreatIntelClone::new_with_scenario(&catalog)` and `NvdClone::new_with_scenario(&catalog)?`
- [ ] Write unit tests 9, 10, 11, 12 (FAIL first): E-DEMO-002, E-DEMO-003, scenario-disabled compat, secondary RNG independence

**Phase 5: Cross-DTU coherence integration test**

- [ ] Write integration test 15 (FAIL first): 3-clone cross-DTU coherence at stage 1

**Phase 6: ci.yml EXPECTED bump**

- [ ] Audit new `#[non_exhaustive]` pub types added in this story: `IncidentTimeline`, `IncidentStage` (at minimum 2); verify by running the non-exhaustive compile-fail gate; update `EXPECTED=N` in `ci.yml` by the exact new count; add violation rows
- [ ] Run `just check` — all 16 Red Gate tests pass; no clippy warnings; fmt clean
- [ ] Confirm perimeter compile-fail gate (test 16) passes with zero new violations
- [ ] Run SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify any new emissions have BC-2.16.002 catalog rows

---

## Previous Story Intelligence

This is Story B of the E-DEMO live-scenario split. Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A)
is the direct predecessor and its merge is a hard prerequisite.

**Critical substrate facts to verify after Story A merges (run remove-uncertainty before dispatch):**

- Confirm `CrowdstrikeState`, `ArmisState`, `ClarotyState`, `CyberintState` each have `generated_records` (or `generated_devices`/`generated_detections`) field from Story A
- Confirm `ScenarioEntityCatalog`, `org_slug_from_org_id`, `build_scenario_entity_catalog` are in `prism-dtu-common/src/scenario/` from Story A
- Confirm `ThreatIntelState.fixture_registry` is `Mutex<HashMap<String, FixtureKey>>` (mutable at construction — Story A does NOT change this; Story B inserts into it)
- Confirm `NvdState.cve_registry` is an immutable `HashMap<String, CveRecord>` (Story A does NOT change this; Story B adds entries at construction time by building a new registry with scenario CVEs)
- Confirm `NvdClone::new()` returns `anyhow::Result<Self>` (fallible; Story B's `new_with_scenario` must match)
- Confirm CVSS path: `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` — read `crates/prism-dtu-nvd/src/types.rs` directly; do NOT infer from test descriptions

**From PLUGIN-MIGRATION-001-D lessons (entries 16, 17, 24):**
- SAP-1: after implementation, run `rg 'event_type\s*=' crates/ --type rust`; verify any new `event_type` emissions have BC-2.16.002 catalog rows in the same commit
- SID-1: integration tests that depend on a running DTU HTTP server use `#[ignore]` only if the dependency is a live external service; in-process harness tests (tokio tasks) are NOT `#[ignore]`'d
- SAP-2 (DTU↔TOML schema parity): any changes to TOML sensor specs in this story require reading the corresponding DTU clone's types.rs to verify column/field parity before committing

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `current_stage_index` is a pure function: no side effects, no shared mutable state, no tokio spawn, no Arc<AtomicU64> counter | ADR-036 §2.1 | Adversary probe: no Mutex<StageIndex> in state structs |
| `StageMask` must NOT carry `#[non_exhaustive]` — it is internal and must be exhaustively constructible within the crate | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 | Adversary + compile test |
| `IncidentTimeline`, `IncidentStage` MUST carry `#[non_exhaustive]` as public types in `prism-dtu-common` | CLAUDE.md #[non_exhaustive] discipline | ci.yml EXPECTED bump |
| `IncidentTimeline` threaded via `Arc` (NOT `Arc<Mutex<...>>`) — read-only after construction | ADR-036 §2.3 | Adversary: no Mutex on timeline field |
| `NvdState.cve_registry` is an immutable `HashMap` (NOT Mutex-wrapped); built entirely at construction time | ADR-036 §2.3, BC-2.06.020 | Adversary: no Mutex on cve_registry |
| `new_with_scenario` for ThreatIntel/NVD must NOT import `prism-spec-engine`, `prism-sensors`, or `prism-query` | INV-PERIMETER-001, ADR-036 §2.5, BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 | Compile-fail gate `tests/external/perimeter-violation/` |
| CVSS path: `CveRecord.metrics.cvss_metric_v31[0].cvss_data.base_score: f64` and `.base_severity: String` — implementer MUST read `crates/prism-dtu-nvd/src/types.rs` to confirm field names | ADR-036 §1.3 U-05 substrate correction | Adversary: read types.rs before review |
| `stage_duration_secs` config array has 4 entries for the 5-stage timeline (stages 1-4 activation thresholds; stage 0 always 0) | ADR-036 §2.2, BC-2.06.019 §9 correction | Test 3 (stage boundary 5 thresholds) |
| E-DEMO-002/003 detected BEFORE any clone constructor is called in `build_clone_pairs` | ADR-036 §6, INV-CONSTRUCTION-TIME-FAILURE-001 | Tests 9, 10 |
| `await_holding_lock = "deny"` (ADR-002 §H1): no `.await` inside a Mutex lock guard in route handlers | ADR-002 | clippy deny list |
| All tracing emission sites with `event_type =` must have BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| Forbidden pattern: `Arc::new(SomeThing::placeholder())` in production boot path | ADR-022 §C, CLAUDE.md | Adversary |

---

## Library & Framework Requirements

Versions pinned from `dependency-graph.md` and `rust-toolchain.toml`. Do NOT invent versions.

| Crate | Version | Usage |
|-------|---------|-------|
| `axum` | `0.7` | Route handlers in all prism-dtu-* crates |
| `tokio` | `1` (multi-threaded runtime) | Async runtime per ADR-002 / AD-013 |
| `chrono` | project-pinned | `Utc::now().timestamp()` for `now_epoch_secs`; already present in armis/crowdstrike; NOT added to threatintel/nvd |
| `serde` / `serde_json` | project-pinned | `CloneConfig` / `ScenarioConfig` deserialization |
| `rand_chacha` (`ChaCha20Rng`) | project-pinned | Secondary RNG stream for `build_scenario_entity_catalog` |
| `anyhow` | project-pinned | Error propagation for E-DEMO-002/003; `NvdClone::new_with_scenario` return type |
| `uuid` | project-pinned | `uuid::Uuid::parse_str()` for org_id string (already used in Story A; confirm it exists post-Story-A merge) |
| `reqwest` | project-pinned | Integration test HTTP clients; `.timeout(Duration::from_secs(30))` mandatory |

**Forbidden patterns:**
- Do NOT introduce `tokio::time::interval` or `tokio::spawn` for stage progression (pure function only, ADR-036 §3.1)
- Do NOT wrap `cve_registry` in `Mutex<...>` — it is immutable after construction (ADR-036 §2.3 substrate correction)
- Do NOT call `generate()` in route handler bodies — generation is construction-time only

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-common/src/scenario/mod.rs` | MODIFY (Story A stub exists) | Add `IncidentTimeline`, `IncidentStage`, `StageMask`, `current_stage_index`, `build_default_incident_timeline` |
| `crates/prism-dtu-crowdstrike/src/state.rs` | MODIFY (Story A: `generated_devices` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | MODIFY (Story A: `new_with_seed` present) | Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>) -> Self` |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | MODIFY (Story A: dual-path present) | Add StageMask filter when `timeline.is_some()` |
| `crates/prism-dtu-armis/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-armis/src/clone.rs` | MODIFY (Story A: `new_with_seed` present) | Add `new_with_scenario(seed, archetype, org_id, org_slug: &str, timeline: Arc<IncidentTimeline>) -> Self` |
| `crates/prism-dtu-armis/src/routes/devices.rs` | MODIFY (Story A: dual-path present) | Add StageMask filter when `timeline.is_some()` |
| `crates/prism-dtu-claroty/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-claroty/src/clone.rs` | MODIFY | Add `new_with_scenario` |
| `crates/prism-dtu-claroty/src/routes/` | MODIFY (Story A: dual-path present) | Add StageMask filter |
| `crates/prism-dtu-cyberint/src/state.rs` | MODIFY (Story A: `generated_records` present) | Add `timeline: Option<Arc<IncidentTimeline>>` |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Add `new_with_scenario(…) -> anyhow::Result<Self>` (fallible) |
| `crates/prism-dtu-cyberint/src/routes/` | MODIFY | Add StageMask filter |
| `crates/prism-dtu-threatintel/src/state.rs` | MODIFY | Add `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> Self` |
| `crates/prism-dtu-nvd/src/state.rs` | MODIFY | Add `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog) -> anyhow::Result<Self>` |
| `crates/prism-dtu-demo-server/src/harness.rs` | MODIFY (Story A: E-DEMO-004/005 present) | Add scenario coordination: E-DEMO-002/003, catalog derivation, IncidentTimeline construction, Arc threading |
| `.github/workflows/ci.yml` | MODIFY | Bump `EXPECTED=N` by count of new `#[non_exhaustive]` pub types (at minimum +2: `IncidentTimeline`, `IncidentStage`; implementer verifies exact count) |
| `tests/external/non-exhaustive-violation/` | MODIFY | Add violation rows for new `#[non_exhaustive]` types |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.06.019 EC-019-003 | `now_epoch_secs < scenario_start_epoch_secs` (clock skew) | `elapsed = max(0, now - start) = 0`; stage 0; no panic (AC-005) |
| EC-002 | BC-2.06.019 EC-019-004 | `now_epoch_secs` far past all thresholds (elapsed >> 600s) | Stage index saturates at `stages.len() - 1` (Containment); no index-out-of-bounds panic |
| EC-003 | BC-2.06.019 EC-019-005 | Two scenario-enabled clones with different seeds | `build_clone_pairs` returns `E-DEMO-002` before any clone constructor called (AC-009) |
| EC-004 | BC-2.06.019 EC-019-006 | `stage_duration_secs` has wrong entry count (not 4) | `E-DEMO-003` (unrecognized archetype or stage-count mismatch); construction fails |
| EC-005 | BC-2.06.019 EC-019-007 | `scenario_start_secs = None` | `scenario_start_epoch_secs` set to `Utc::now().timestamp()` at `build_clone_pairs` call time; demo begins at stage 0 |
| EC-006 | BC-2.06.019 EC-019-009 | `scenario_start_secs` set to past epoch (mid-scenario start) | Correct: elapsed positive at startup; stage index may start at Recon or LateralMovement; operator intentional |
| EC-007 | BC-2.06.019 EC-019-011 | Route handler called concurrently from multiple async tasks | `current_stage_index` is pure; concurrent calls safe without locking (no shared mutable state) |
| EC-008 | BC-2.06.020 EC-020-002 | `entities.ioc_ips = []` (catalog produces no IPs for this seed) | No IP entries inserted; no error; non-empty catalog fields still injected normally |
| EC-009 | BC-2.06.020 EC-020-003 | Same IOC in catalog AND pre-existing registry as Benign | `HashMap::insert` overwrites; `FixtureKey::Malicious` wins; scenario injection takes priority |
| EC-010 | BC-2.06.020 EC-020-011 | `scenario.enabled = true` for operational DTUs, ThreatIntel uses static path | ThreatIntel uses `new()` (static default); scenario IOCs will NOT resolve as Malicious; valid (if incomplete) operator config; no error |
| EC-011 | ADR-036 §2.2 | `seed = u64::MAX` → `seeded_rng(0, org_id)` secondary stream | `wrapping_add(1) = 0`; valid; no panic |
| EC-012 | ADR-036 §2.3 | Stage-mask filter applied when `generated_records` is empty (Story A produced empty set) | No records to filter; empty response; no panic; existing behavior preserved |
| EC-013 | ADR-036 §2.3 | `NvdClone::new_with_scenario` returns `Err` (e.g., fixture file missing) | Error propagated through `build_clone_pairs -> anyhow::Result<Vec<ClonePair>>`; harness aborts cleanly |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `IncidentTimeline` | `prism-dtu-common/src/scenario/` | Pure (data struct, no I/O) | ADR-036 §2.2 |
| `IncidentStage` | `prism-dtu-common/src/scenario/` | Pure (data struct) | ADR-036 §2.2 |
| `StageMask` | `prism-dtu-common/src/scenario/` | Pure (data struct; internal, NOT non-exhaustive) | ADR-036 §2.2, BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 |
| `current_stage_index` | `prism-dtu-common/src/scenario/` | Pure (function: `(&IncidentTimeline, i64) -> usize`) | ADR-036 §2.1 |
| `build_default_incident_timeline` | `prism-dtu-common/src/scenario/` | Pure (deterministic from catalog + thresholds) | ADR-036 §2.2 |
| `build_clone_pairs` (scenario coordination additions) | `prism-dtu-demo-server/src/harness.rs` | Effectful (constructs clones, reads config, calls catalog + timeline builders) | ADR-036 §2.4 |
| `ArmisState.timeline` / route projection | `prism-dtu-armis/src/state.rs` + routes | Effectful (HTTP handler calling pure `current_stage_index`) | ADR-036 §2.3 |
| `CrowdstrikeState.timeline` / route projection | `prism-dtu-crowdstrike/src/state.rs` + routes | Effectful | ADR-036 §2.3 |
| `ClarotyState.timeline` / route projection | `prism-dtu-claroty/src/state.rs` + routes | Effectful | ADR-036 §2.3 |
| `CyberintState.timeline` / route projection | `prism-dtu-cyberint/src/state.rs` + routes | Effectful (fallible constructor) | ADR-036 §2.3 |
| `ThreatIntelClone::new_with_scenario` | `prism-dtu-threatintel/src/state.rs` | Effectful (constructor: populates `fixture_registry` Mutex<HashMap> at init) | ADR-036 §2.3, BC-2.06.020 PC-1 |
| `NvdClone::new_with_scenario` | `prism-dtu-nvd/src/state.rs` | Effectful (fallible constructor: builds `cve_registry` HashMap at init) | ADR-036 §2.3, BC-2.06.020 PC-3 |

---

## Forbidden Dependencies

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-dtu-threatintel` | `prism-spec-engine` | INV-PERIMETER-001 / ADR-036 §2.5 |
| `prism-dtu-threatintel` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-threatintel` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-common` | `prism-spec-engine` | INV-PERIMETER-001 |
| Any new crate | `prism-dtu-scenario` (does not exist) | ADR-036 §3.4 — no separate crate |

---

## SAP-1 Compliance (Structured Event Catalog)

Per CLAUDE.md §SAP-1, any `tracing::*!(event_type = "...")` emission site added in this
story requires a corresponding row in BC-2.16.002 Structured Event Catalog with event_type,
emitting module, field schema, audit role, and recurrence policy.

Expected potential emissions (implementer must enumerate actual sites):
- Potentially `event_type = "scenario.construction"` in `build_clone_pairs`
- Potentially `event_type = "scenario.stage_computed"` in route handlers (if logging stage transitions)

If NO new `event_type` emissions are added, state so explicitly in the PR description.

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-06-09 | Initial authoring per ADR-036 v2.0 §8 story split (D-1077). Derived from S-DEMO-DTU-LIVE-SCENARIO-001 Group B+C ACs with substrate corrections: stage_duration_secs 4-entry array; NvdState::lookup_and_count → NvdState::cve_registry immutable HashMap; CVSS path metrics.cvss_metric_v31[0].cvss_data.base_score; NvdClone::new_with_scenario fallible; ThreatIntelClone::new_with_scenario infallible; canonical IDs "dev-{8hex}-{seed}-{n}" per ADR-036 §2.2. Depends on Story A merge. |
