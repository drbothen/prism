---
document_type: story
story_id: S-DEMO-DTU-LIVE-SCENARIO-001
title: "prism-dtu-common + demo-server + 6 DTU crates: Deterministic Live-Scenario Progression Engine (CompromisedEndpoint multi-client SOC demo)"
wave: 5
epic_id: E-DEMO
priority: P2
status: superseded
superseded_by: [S-DEMO-DTU-LIVE-SCENARIO-001-A, S-DEMO-DTU-LIVE-SCENARIO-001-B]
version: "1.1"
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
#   Registry. The scenario-progression engine is test/demo infrastructure that lives entirely
#   within the SS-01 DTU clone family. No other subsystem touches these crates.
#   Decision anchor: ADR-036 subsystems_affected: [SS-01].
crates_touched: [prism-dtu-common, prism-dtu-demo-server, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-threatintel, prism-dtu-nvd]
target_module: prism-dtu-common
behavioral_contracts: [BC-2.06.018, BC-2.06.019, BC-2.06.020]
verification_properties: [VP-018-A, VP-018-B, VP-018-C, VP-018-D, VP-018-E, VP-019-A, VP-019-B, VP-019-C, VP-019-D, VP-019-E, VP-019-F, VP-019-G, VP-019-H, VP-020-A, VP-020-B, VP-020-C, VP-020-D, VP-020-E, VP-020-F, VP-020-G, VP-020-H]
depends_on:
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001
  # Dependency anchor: S-CONFIG-MULTI-TENANT-OVERRIDE-001 delivers the per-org overlay plumbing
  # in prism-spec-engine and prism-sensors (merged, develop@3e822522). This story's integration
  # tests exercise multi-client scenario correctness end-to-end within the DTU layer; the overlay
  # plumbing being in place ensures the base_url routing for per-client demo instances works.
  # S-DEMO-MULTI-TENANT-DTU-001 is NOT a hard dependency: it delivers per-DTU multi-address binding
  # (BC-2.06.017), which is COMPLEMENTARY infrastructure — a demo operator needs both stories for
  # a full multi-client demo, but the scenario engine (this story) is independently testable in a
  # single-address harness. Declaring S-DEMO-MULTI-TENANT-DTU-001 as depends_on would create a
  # spurious serial dependency that delays Wave 5 parallelism. The two stories can be delivered
  # in parallel or in either order. Rationale: D-1077 decision note, ADR-036 §8 "single story"
  # recommendation.
blocks: []
points: 13
# Points justification (ADR-036 §8 estimate):
#   1. prism-dtu-common/src/scenario/ module: ScenarioEntityCatalog, IncidentTimeline,
#      IncidentStage, StageMask, current_stage_index pure fn — ~200 lines pure Rust: 2 pts
#   2. CloneConfig extension: ScenarioConfig struct + TOML deserialization + defaults: 1 pt
#   3. build_clone_pairs coordination: catalog derivation, E-DEMO-002/003 guards,
#      Arc<IncidentTimeline> threading to 4 generator-backed clones + entity catalog to 2
#      enrichment clones (~60 lines): 2 pts
#   4. Per-DTU state extension: Option<Arc<IncidentTimeline>> in 4 state structs: 1 pt
#   5. Per-DTU route projection: stage-mask filtering in device + alert/detection routes
#      for Armis, CrowdStrike, Claroty, Cyberint (8 route files modified): 3 pts
#   6. ThreatIntel::new_with_scenario + NVD::new_with_scenario constructors: 1 pt
#   7. #[non_exhaustive] on new pub types + ci.yml EXPECTED bump: 0.5 pts
#   8. Red Gate test suite (~24 tests, FAIL-first): 2 pts
#   9. Integration gate (just check): 0.5 pts
#   Total: ~13 pts — at ceiling; split not recommended per ADR-036 §8
estimated_days: 5
risk: HIGH
# Risk justification:
#   This story touches 8 crates and introduces a new cross-crate coordination pattern
#   (ScenarioEntityCatalog shared from prism-dtu-common to 6 consuming crates). The per-DTU
#   route projection adds two code paths to 8+ route handlers. The secondary RNG stream
#   independence (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001) must not shift generator output,
#   which would break BC-2.06.018 §PC-4 backward compat. The #[non_exhaustive] ci.yml EXPECTED
#   bump must be computed precisely or the compile-fail gate will trip.
acceptance_criteria_count: 24
red_gate_tests: 24
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Backward compatibility (INV-SCENARIO-DISABLED-COMPAT-001): regression test TV-019-007 must pass before PR can merge."
  - "Secondary RNG stream independence (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001): seeded_rng(seed.wrapping_add(1), org_id) must be a separate ChaCha20Rng instance; implementer MUST NOT call next() on the primary stream before building the catalog."
  - "reqwest::Client timeout: any new HTTP client in integration tests uses .timeout(Duration::from_secs(30)) per CLAUDE.md conventions."
  - "#[non_exhaustive] EXPECTED bump: implementer must grep the current EXPECTED=N value in ci.yml and increment by the count of new #[non_exhaustive] pub types added before committing."
  - "INV-PERIMETER-001: new_with_scenario constructors in prism-dtu-threatintel and prism-dtu-nvd must not import from prism-spec-engine, prism-sensors, or prism-query. Verified by existing compile-fail gate."
traces_to: [D-1077, ADR-036]
---

> **SUPERSEDED — ADR-036 v2.0 Story Split (D-1077, 2026-06-09)**
>
> This story has been split into two focused stories following the ADR-036 v2.0
> substrate-reconciliation correction. The remove-uncertainty scan (U-01..U-09) confirmed
> that BC-2.06.018 baseline seeding is entirely unimplemented in the current codebase
> (demo-server clones serve static JSON; `CloneConfig.seed` is never forwarded;
> no `generate()` call exists in `build_clone_pairs()`). A single-story delivery at
> production-grade is unrealistic given this substrate gap.
>
> - **S-DEMO-DTU-LIVE-SCENARIO-001-A** (status: ready, 8 pts, BC-2.06.018) — Baseline
>   Seeding Retrofit: wire seeded generators into demo-server clone serving paths.
>   This is the immediate deliverable and the prerequisite for Story B.
> - **S-DEMO-DTU-LIVE-SCENARIO-001-B** (status: draft, 7 pts, BC-2.06.019 + BC-2.06.020) —
>   Scenario Progression + Enrichment Correlation: IncidentTimeline, StageMask projection,
>   ThreatIntel/NVD lookup injection. Depends on Story A merge.
>
> The content of this file is preserved as the authoritative basis from which Story B
> was derived. Do not delete this file; it is referenced by the STORY-INDEX supersession row.

# S-DEMO-DTU-LIVE-SCENARIO-001: Deterministic Live-Scenario Progression Engine

Multi-client SOC demo live-scenario layer: `CompromisedEndpoint` archetype evolves through
Baseline → Recon (60s) → LateralMovement (180s) → Exfil (360s) → Containment (600s) coherently
across Armis, CrowdStrike, Claroty, Cyberint (operational), and ThreatIntel, NVD (enrichment).
Implements ADR-036 (D-1077) as a pure-function-of-wall-clock-time stage engine in `prism-dtu-common`
with per-DTU StageMask projection and construction-time lookup injection.

---

## Narrative

As a SOC analyst running a multi-client live demo, I want each demo client's incident to
unfold across all six DTU clones in a reproducible temporal sequence, so that I can show
a complete attack lifecycle — from initial device visibility (Recon), through lateral movement
detections (CrowdStrike/Armis), to exfiltration IOC resolution (ThreatIntel), to CVE
enrichment (NVD), and finally device containment (CrowdStrike) — that tells a coherent story
a prospect understands.

**Goal:** Given `scenario.enabled = true` and a shared `scenario_start_secs` across DTU clones,
every HTTP request to any operational DTU reflects the current stage deterministically. The same
demo replay (same `scenario_start_secs`, same elapsed time) produces byte-identical responses.
Enrichment DTUs (ThreatIntel, NVD) are always-ready: scenario IOCs and CVEs resolve from
construction time regardless of stage. Multi-client distinctness is preserved: two clients with
different seeds produce disjoint `ScenarioEntityCatalog` IDs, so one client's incident cannot
be confused with another's.

---

## Behavioral Contracts

| BC | Title | Key Invariants |
|----|-------|----------------|
| BC-2.06.018 | Demo-Server Config-Time Data Seeding — Per-Clone seed + fixture_set Wire-Up | INV-DISTINCT-DATA-001, INV-FIXTURE-SET-ARCHETYPE-MAP-001, INV-CONSTRUCTION-TIME-FAILURE-001 |
| BC-2.06.019 | Demo-Server Scenario Progression — Pure-Function Temporal Stage Advancement | INV-PROGRESSION-REPRODUCIBILITY-001, INV-STAGE-MONOTONICITY-001, INV-STAGE-MASK-COMPLETENESS-001, INV-SCENARIO-DISABLED-COMPAT-001, INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 |
| BC-2.06.020 | Demo-Server Enrichment Correlation — Scenario IOCs/CVEs Resolve in ThreatIntel/NVD | INV-THREATINTEL-IOC-CORRELATION-001, INV-NVD-CVE-CORRELATION-001, INV-CROSS-DTU-ENTITY-COHERENCE-001, INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001, INV-PERIMETER-COMPLIANCE-001, INV-CONSTRUCTION-TIME-INJECTION-001 |

---

## Acceptance Criteria

### Group A — Config-Time Seeding (BC-2.06.018)

**AC-001 — Config-time seed forwarded to generator-backed clones**
(traces to BC-2.06.018 postcondition 1)

Given a `demo.toml` with `clones.claroty.seed = 100` and `fixture_set = "compromised"`,
when `build_clone_pairs` runs, then `ClarotyClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id)` is called and the resulting device/alert responses differ from those of a clone seeded with `seed = 200`.

Red Gate: `test_BC_2_06_018_seed_forwarded_to_claroty`

**AC-002 — INV-DISTINCT-DATA-001: disjoint ID sets for distinct seeds**
(traces to BC-2.06.018 invariant INV-DISTINCT-DATA-001)

Given two Armis clone instances constructed with `seed_A = 100` and `seed_B = 200` (same org_id),
when both are queried at `/api/v1/devices`,
then `response_A.ids ∩ response_B.ids = ∅` — no device ID appears in both responses.

Red Gate: `test_BC_2_06_018_distinct_seeds_disjoint_ids`

**AC-003 — fixture_set → Archetype canonical mapping (INV-FIXTURE-SET-ARCHETYPE-MAP-001)**
(traces to BC-2.06.018 invariant INV-FIXTURE-SET-ARCHETYPE-MAP-001)

Given each of the 8 canonical `fixture_set` strings (`"default"`, `"compromised"`, `"auth_outage"`, `"large_scale"`, `"pagination_edges"`, `"schema_drift"`, `"high_churn"`, `"dormant"`),
when `build_clone_pairs` constructs the clone,
then the correct `Archetype` variant is selected with no construction-time error;
and given `fixture_set = "xyzzy_unknown"`, then `build_clone_pairs` returns `Err` containing `"E-DEMO-001"`.

Red Gate: `test_BC_2_06_018_fixture_set_archetype_mapping`

**AC-004 — E-DEMO-001 propagates at construction, not request time (INV-CONSTRUCTION-TIME-FAILURE-001)**
(traces to BC-2.06.018 invariant INV-CONSTRUCTION-TIME-FAILURE-001)

Given `fixture_set = "bad_value"` for a clone,
when `build_clone_pairs` is called,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-001"` and the clone name and the invalid value; and the process does not panic at request handling time.

Red Gate: `test_BC_2_06_018_e_demo_001_at_construction_time`

**AC-005 — Backward compat: seed=42 + fixture_set="default" is byte-identical to pre-seeding path**
(traces to BC-2.06.018 postcondition 4)

Given `CloneConfig.seed = 42` and `CloneConfig.fixture_set = "default"` for all generator-backed clones,
when both the new `new_with_seed(42, HealthyOtEnvironment, default_org)` path and the legacy path are exercised,
then all existing integration tests that passed against the pre-seeding `CloneType::new()` constructor continue to pass without modification.

Red Gate: `test_BC_2_06_018_backward_compat_seed42_default`

---

### Group B — Scenario Progression (BC-2.06.019)

**AC-006 — ScenarioEntityCatalog constructed once per client from (seed, org_id) via secondary RNG stream**
(traces to BC-2.06.019 postcondition 1)

Given a client config with `scenario.enabled = true` and `seed = 100`, `org_id = "acme"`,
when `build_clone_pairs` runs,
then a single `ScenarioEntityCatalog` is derived via `seeded_rng(seed.wrapping_add(1), org_id)` (secondary RNG), with `primary_device_id = "dev-acme-100-0"`, non-empty `ioc_ips`, `ioc_domains`, `ioc_hashes`, and `device_cves`; and the secondary RNG stream does not consume state from the primary stream `seeded_rng(100, "acme")`.

Red Gate: `test_BC_2_06_019_catalog_constructed_secondary_rng`

**AC-007 — INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001: catalog derivation does not shift generator output**
(traces to BC-2.06.019 invariant INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001)

Given two harness constructions with `seed = 100, org_id = "acme"`: one with `scenario.enabled = false` (static path) and one with `scenario.enabled = true` (scenario path),
when both are queried at the same device endpoint,
then the underlying `FixtureSet` device records are byte-identical in both cases — the catalog derivation has not shifted the generator's RNG output.

Red Gate: `test_BC_2_06_019_secondary_rng_stream_independence`

**AC-008 — INV-PROGRESSION-REPRODUCIBILITY-001: current_stage_index is pure function of (timeline, now)**
(traces to BC-2.06.019 invariant INV-PROGRESSION-REPRODUCIBILITY-001)

Given two independent `IncidentTimeline` instances built from the same `(seed, org_id, scenario_start_secs)`,
when `current_stage_index` is called on both with the same `now_epoch_secs`,
then both return the same `usize` stage index, across independent process restarts and concurrent invocations.

Red Gate: `test_BC_2_06_019_stage_index_reproducible`

**AC-009 — Stage boundary correctness: 5 stages at default thresholds**
(traces to BC-2.06.019 postcondition 2 and postcondition 3)

Given `scenario_start_epoch_secs = T` and the default `CompromisedEndpoint` stage thresholds [0, 60, 180, 360, 600]:
- `now = T + 0s` → stage 0 (Baseline)
- `now = T + 30s` → stage 0 (Baseline; < 60s threshold)
- `now = T + 90s` → stage 1 (Recon)
- `now = T + 200s` → stage 2 (LateralMovement)
- `now = T + 400s` → stage 3 (Exfil)
- `now = T + 700s` → stage 4 (Containment; saturates at max stage)

Red Gate: `test_BC_2_06_019_stage_boundary_all_5_thresholds`

**AC-010 — INV-STAGE-MONOTONICITY-001: stage index never decreases**
(traces to BC-2.06.019 invariant INV-STAGE-MONOTONICITY-001)

Given a timeline with default thresholds,
when `current_stage_index` is evaluated at a monotonically increasing sequence of `now_epoch_secs` values spanning all stage boundaries,
then the returned stage index is non-decreasing across the entire sequence.

Red Gate: `test_BC_2_06_019_stage_monotonicity`

**AC-011 — Clock-skew / future start handled: elapsed clamped to 0**
(traces to BC-2.06.019 edge case EC-019-003)

Given `now_epoch_secs < scenario_start_epoch_secs` (clock skew or `scenario_start_secs` set in the future),
when `current_stage_index` is called,
then it returns stage 0 (Baseline) without panic — `elapsed = max(0, now - start)` clamps to 0.

Red Gate: `test_BC_2_06_019_clock_skew_clamped_to_baseline`

**AC-012 — INV-STAGE-MASK-COMPLETENESS-001: all 6 StageMask fields explicitly set for every stage**
(traces to BC-2.06.019 invariant INV-STAGE-MASK-COMPLETENESS-001)

Given the default `CompromisedEndpoint` `IncidentTimeline` (5 stages),
when each `IncidentStage.visible_entity_mask` is inspected,
then every stage has explicit bool values for all 6 fields: `primary_device`, `lateral_devices`, `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves` — no field is left uninitialized or implicitly defaulted.

Red Gate: `test_BC_2_06_019_stage_mask_completeness_all_6_fields`

**AC-013 — Per-DTU StageMask projection: Armis primary_device not visible at stage 0, visible at stage 1+**
(traces to BC-2.06.019 postcondition 4 and test vectors TV-019-009, TV-019-010)

Given an Armis clone constructed with `scenario.enabled = true` and a timeline with `scenario_start_secs = T`:
- At `now = T + 30s` (stage 0 / Baseline), `GET /api/v1/devices` response does NOT contain `catalog.primary_device_id`.
- At `now = T + 90s` (stage 1 / Recon), `GET /api/v1/devices` response CONTAINS `catalog.primary_device_id`; lateral device IDs are NOT present.

Red Gate: `test_BC_2_06_019_armis_primary_device_stage_visibility`

**AC-014 — Per-DTU StageMask projection: CrowdStrike containment_status visible at stage 4 only**
(traces to BC-2.06.019 postcondition 4 and test vector TV-019-011)

Given a CrowdStrike clone with `scenario.enabled = true` and a timeline at `scenario_start_secs = T`:
- At `now = T + 200s` (stage 2 / LateralMovement), the device record for `primary_device_id` shows `containment_status = "normal"` (or equivalent non-contained value).
- At `now = T + 700s` (stage 4 / Containment), the same device record shows `containment_status = "contained"`.

Red Gate: `test_BC_2_06_019_crowdstrike_containment_stage4_only`

**AC-015 — E-DEMO-002: mismatched seeds across scenario-enabled clones rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-002)

Given `clones.crowdstrike.seed = 100` and `clones.armis.seed = 200`, both with `scenario.enabled = true`,
when `build_clone_pairs` runs,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-002"` and both clone names and seed values, before any clone constructor is called.

Red Gate: `test_BC_2_06_019_e_demo_002_seed_mismatch`

**AC-016 — E-DEMO-003: unrecognized scenario archetype rejected at construction**
(traces to BC-2.06.019 error code E-DEMO-003)

Given `scenario.archetype = "unknown_val"` for any clone with `scenario.enabled = true`,
when `build_clone_pairs` runs,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-003"` and the clone name and the invalid archetype string.

Red Gate: `test_BC_2_06_019_e_demo_003_unrecognized_archetype`

**AC-017 — INV-SCENARIO-DISABLED-COMPAT-001: scenario.enabled=false path is byte-identical to BC-2.06.018**
(traces to BC-2.06.019 invariant INV-SCENARIO-DISABLED-COMPAT-001 and postcondition 6)

Given a clone constructed with `scenario.enabled = false` (or absent `[clones.*.scenario]` block) and `seed = 42`, `fixture_set = "default"`,
when queried at any fixed request path,
then responses are byte-identical to the pre-ADR-036 `new_with_seed(42, HealthyOtEnvironment, default_org)` responses; `timeline: Option<Arc<IncidentTimeline>>` is `None` in the clone state.

Red Gate: `test_BC_2_06_019_scenario_disabled_byte_identical_compat`

**AC-018 — Concurrent requests at same now produce identical responses (no shared mutable state)**
(traces to BC-2.06.019 postcondition 3 and test vector TV-019-014)

Given 3 concurrent HTTP requests to the same DTU route at the same `now_epoch_secs = T + 200s`,
when all 3 responses are received,
then they are byte-identical — `current_stage_index` is a pure function with no shared mutable progression state, so no lock contention or race condition can produce divergent results.

Red Gate: `test_BC_2_06_019_concurrent_requests_same_now_identical`

---

### Group C — Enrichment Correlation (BC-2.06.020)

**AC-019 — INV-THREATINTEL-IOC-CORRELATION-001: all scenario IOCs resolve as Malicious in ThreatIntel**
(traces to BC-2.06.020 invariant INV-THREATINTEL-IOC-CORRELATION-001)

Given `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog)` constructed with a catalog from `(seed=100, org_id="acme")`,
when lookup requests are issued for each of `entities.ioc_ips[0]`, `entities.ioc_domains[0]`, and `entities.ioc_hashes[0]`,
then each response contains `threat_is_known_malicious = true` and `threat_score >= 75`.

Red Gate: `test_BC_2_06_020_threatintel_ioc_correlation_all_types`

**AC-020 — INV-NVD-CVE-CORRELATION-001: all scenario CVEs resolve with HIGH CVSS in NVD**
(traces to BC-2.06.020 invariant INV-NVD-CVE-CORRELATION-001)

Given `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog)` constructed with a catalog from `(seed=100, org_id="acme")`,
when a CVE lookup request is issued for `entities.device_cves[0]`,
then the response contains a `CveRecord` with `cvss_metric_v31[0].cvss_data.base_score >= 7.0` and does not return 404.

Red Gate: `test_BC_2_06_020_nvd_cve_correlation_high_cvss`

**AC-021 — INV-CROSS-DTU-ENTITY-COHERENCE-001: primary_device_id consistent across Armis, CrowdStrike, Claroty at stage >= 1**
(traces to BC-2.06.020 invariant INV-CROSS-DTU-ENTITY-COHERENCE-001 and postcondition 5)

Given three clones (Armis, CrowdStrike, Claroty) all constructed with `(seed=100, org_id="acme", scenario.enabled=true)` and `scenario_start_secs = T`,
when all three are queried at `now = T + 90s` (stage 1 / Recon),
then `catalog.primary_device_id = "dev-acme-100-0"` appears in the device response of each clone.

Red Gate: `test_BC_2_06_020_cross_dtu_entity_coherence_stage1`

**AC-022 — INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001: non-scenario indicators return default registry result**
(traces to BC-2.06.020 invariant INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 and postconditions 6)

Given a ThreatIntel clone constructed with `new_with_scenario` and a non-scenario IP `"192.0.2.1"` (not in `ioc_ips`),
when a lookup is issued for `"192.0.2.1"`,
then the response is identical to `ThreatIntelClone::new().lookup("192.0.2.1")` — scenario injection is strictly additive.
Likewise, for NVD with CVE ID `"CVE-2020-99999"` not in `device_cves`, the response is identical to `NvdClone::new()`.

Red Gate: `test_BC_2_06_020_non_scenario_lookup_passthrough`

**AC-023 — INV-CONSTRUCTION-TIME-INJECTION-001: enrichment registries pre-populated at construction, not at request time**
(traces to BC-2.06.020 invariant INV-CONSTRUCTION-TIME-INJECTION-001)

Given a ThreatIntel clone constructed with `new_with_scenario(entities)`,
when the first lookup for `entities.ioc_ips[0]` arrives immediately after construction (no warm-up delay),
then the response is `threat_is_known_malicious = true` — the registry is fully populated before the server starts serving, with no deferred-injection race window.

Red Gate: `test_BC_2_06_020_enrichment_injection_at_construction_not_request_time`

**AC-024 — INV-PERIMETER-COMPLIANCE-001: new_with_scenario constructors do not depend on prism-spec-engine / prism-sensors / prism-query**
(traces to BC-2.06.020 invariant INV-PERIMETER-COMPLIANCE-001)

Given `ThreatIntelClone::new_with_scenario` and `NvdClone::new_with_scenario` exist in the codebase,
when the compile-fail gate in `tests/external/perimeter-violation/` is run,
then it passes with zero new violations — no import from `prism-spec-engine`, `prism-sensors`, or `prism-query` appears in `prism-dtu-threatintel` or `prism-dtu-nvd`.
The `ci.yml` `EXPECTED` count is bumped by the count of new `#[non_exhaustive]` public types added in this story.

Red Gate: `test_BC_2_06_020_perimeter_compile_fail_gate_still_passes` (compile-fail test, not a runtime test)

---

## Red Gate Test Plan

All tests below are written FAIL-first (stub → red → implement → green) per SID-1.
Tests are unit tests in `#[cfg(test)] mod tests` blocks or integration tests in `crates/<crate>/tests/`,
NOT `#[ignore]`'d unless a specific DTU-EXT dependency is cited.

| # | Test Name | Crate | BC | Type |
|---|-----------|-------|-----|------|
| 1 | `test_BC_2_06_018_seed_forwarded_to_claroty` | prism-dtu-demo-server | BC-2.06.018 PC-1 | unit |
| 2 | `test_BC_2_06_018_distinct_seeds_disjoint_ids` | prism-dtu-demo-server | BC-2.06.018 INV-DISTINCT-DATA-001 | integration |
| 3 | `test_BC_2_06_018_fixture_set_archetype_mapping` | prism-dtu-demo-server | BC-2.06.018 INV-FIXTURE-SET-ARCHETYPE-MAP-001 | unit |
| 4 | `test_BC_2_06_018_e_demo_001_at_construction_time` | prism-dtu-demo-server | BC-2.06.018 INV-CONSTRUCTION-TIME-FAILURE-001 | unit |
| 5 | `test_BC_2_06_018_backward_compat_seed42_default` | prism-dtu-demo-server | BC-2.06.018 PC-4 | regression |
| 6 | `test_BC_2_06_019_catalog_constructed_secondary_rng` | prism-dtu-common | BC-2.06.019 PC-1 | unit |
| 7 | `test_BC_2_06_019_secondary_rng_stream_independence` | prism-dtu-common | BC-2.06.019 INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 | unit |
| 8 | `test_BC_2_06_019_stage_index_reproducible` | prism-dtu-common | BC-2.06.019 INV-PROGRESSION-REPRODUCIBILITY-001 | unit |
| 9 | `test_BC_2_06_019_stage_boundary_all_5_thresholds` | prism-dtu-common | BC-2.06.019 PC-2, PC-3 | unit |
| 10 | `test_BC_2_06_019_stage_monotonicity` | prism-dtu-common | BC-2.06.019 INV-STAGE-MONOTONICITY-001 | unit |
| 11 | `test_BC_2_06_019_clock_skew_clamped_to_baseline` | prism-dtu-common | BC-2.06.019 EC-019-003 | unit |
| 12 | `test_BC_2_06_019_stage_mask_completeness_all_6_fields` | prism-dtu-common | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 | unit |
| 13 | `test_BC_2_06_019_armis_primary_device_stage_visibility` | prism-dtu-armis | BC-2.06.019 PC-4 | integration |
| 14 | `test_BC_2_06_019_crowdstrike_containment_stage4_only` | prism-dtu-crowdstrike | BC-2.06.019 PC-4 | integration |
| 15 | `test_BC_2_06_019_e_demo_002_seed_mismatch` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-002 | unit |
| 16 | `test_BC_2_06_019_e_demo_003_unrecognized_archetype` | prism-dtu-demo-server | BC-2.06.019 E-DEMO-003 | unit |
| 17 | `test_BC_2_06_019_scenario_disabled_byte_identical_compat` | prism-dtu-demo-server | BC-2.06.019 INV-SCENARIO-DISABLED-COMPAT-001 | regression |
| 18 | `test_BC_2_06_019_concurrent_requests_same_now_identical` | prism-dtu-demo-server | BC-2.06.019 PC-3 | concurrency |
| 19 | `test_BC_2_06_020_threatintel_ioc_correlation_all_types` | prism-dtu-threatintel | BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 | unit |
| 20 | `test_BC_2_06_020_nvd_cve_correlation_high_cvss` | prism-dtu-nvd | BC-2.06.020 INV-NVD-CVE-CORRELATION-001 | unit |
| 21 | `test_BC_2_06_020_cross_dtu_entity_coherence_stage1` | prism-dtu-demo-server | BC-2.06.020 INV-CROSS-DTU-ENTITY-COHERENCE-001 | integration |
| 22 | `test_BC_2_06_020_non_scenario_lookup_passthrough` | prism-dtu-threatintel / prism-dtu-nvd | BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 | unit |
| 23 | `test_BC_2_06_020_enrichment_injection_at_construction_not_request_time` | prism-dtu-threatintel | BC-2.06.020 INV-CONSTRUCTION-TIME-INJECTION-001 | unit |
| 24 | `test_BC_2_06_020_perimeter_compile_fail_gate_still_passes` | tests/external/perimeter-violation | BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 | compile-fail |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~6 000 |
| ADR-036 (full) | ~5 500 |
| BC-2.06.018, BC-2.06.019, BC-2.06.020 (all 3) | ~9 000 |
| prism-dtu-common/src/generator/{archetype,rng,opts,fixture}.rs | ~2 500 |
| prism-dtu-demo-server/src/{harness,config}.rs | ~2 000 |
| prism-dtu-armis/src/{state,clone}.rs | ~1 500 |
| prism-dtu-crowdstrike/src/{state,generator}.rs | ~1 500 |
| prism-dtu-threatintel/src/state.rs | ~800 |
| prism-dtu-nvd/src/state.rs | ~800 |
| ci.yml (EXPECTED line + feature flags) | ~200 |
| Test files (24 stubs × ~40 lines each) | ~3 000 |
| Tool outputs (nextest, clippy) | ~2 000 |
| **Total estimate** | **~34 800** |

At ~200k context window, this is ~17% — within the 20-30% ceiling. No split required per ADR-036 §8.

---

## Tasks

Implementation checklist (TDD order — write failing tests before each implementation step):

**Phase 1: prism-dtu-common scenario module**

- [ ] Create `crates/prism-dtu-common/src/scenario/mod.rs` (behind `feature = "fixture-gen"`)
- [ ] Define `ScenarioEntityCatalog` (non-exhaustive pub struct with all 6 fields per ADR-036 §2.2)
- [ ] Define `StageMask` (exhaustive internal struct — must NOT carry `#[non_exhaustive]` per BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001)
- [ ] Define `IncidentStage` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`)
- [ ] Define `IncidentTimeline` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`)
- [ ] Implement `current_stage_index(timeline: &IncidentTimeline, now_epoch_secs: i64) -> usize` pure function
- [ ] Implement `build_scenario_entity_catalog(seed: u64, org_id: &OrgId) -> ScenarioEntityCatalog` using secondary RNG stream `seeded_rng(seed.wrapping_add(1), org_id)`
- [ ] Implement default `CompromisedEndpoint` `IncidentTimeline` builder with 5 stages and default thresholds
- [ ] Write unit tests 6–12 (FAIL first): stage boundaries, monotonicity, clock skew, mask completeness, secondary RNG independence, reproducibility
- [ ] Export `scenario` module from `prism-dtu-common/src/lib.rs` (feature-gated)

**Phase 2: prism-dtu-demo-server config extension**

- [ ] Add `ScenarioConfig` struct to `crates/prism-dtu-demo-server/src/config.rs` (`#[derive(Debug, Clone, Deserialize, Default)]`)
- [ ] Add `scenario: Option<ScenarioConfig>` field to `CloneConfig`
- [ ] Add `scenario_start_secs: Option<i64>` to `CloneConfig` (or nest inside `ScenarioConfig` per ADR-036 §2.4)
- [ ] Write unit tests 1, 3, 4 (FAIL first): seed forwarding, fixture_set mapping, E-DEMO-001

**Phase 3: build_clone_pairs coordination**

- [ ] Add scenario coordination logic to `build_clone_pairs` in `crates/prism-dtu-demo-server/src/harness.rs`:
  - E-DEMO-002 seed-mismatch guard (before any clone constructor)
  - E-DEMO-003 unrecognized archetype guard
  - `ScenarioEntityCatalog` derivation from first scenario-enabled clone's `(seed, org_id)`
  - `IncidentTimeline` construction with operator-override `stage_duration_secs`
  - `Arc<IncidentTimeline>` threading to `new_with_scenario` for generator-backed clones
  - `&ScenarioEntityCatalog` threading to `new_with_scenario` for ThreatIntel + NVD
- [ ] Write unit tests 2, 5, 15, 16, 17 (FAIL first): distinct IDs, backward compat, E-DEMO-002, E-DEMO-003, scenario-disabled compat

**Phase 4: per-DTU state struct extension (4 generator-backed clones)**

- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` field to `ArmisState` (prism-dtu-armis/src/state.rs)
- [ ] Add `new_with_scenario(seed, archetype, org_id, timeline: Arc<IncidentTimeline>) -> Self` constructor to Armis clone
- [ ] Add `timeline: Option<Arc<IncidentTimeline>>` to `CrowdstrikeState` (prism-dtu-crowdstrike/src/state.rs)
- [ ] Add `new_with_scenario` constructor to CrowdStrike clone
- [ ] Repeat for Claroty (`ClarotyState`)
- [ ] Repeat for Cyberint (`CyberintState`)

**Phase 5: per-DTU route projection (stage-mask filtering)**

- [ ] Armis `/api/v1/devices` route: add scenario-path branch — call `current_stage_index`, apply StageMask to filter `primary_device` and `lateral_devices`
- [ ] Armis alerts route (if applicable): apply StageMask for `ioc_hashes`, `ioc_ips`, `ioc_domains`
- [ ] CrowdStrike device query route: apply StageMask; expose `containment_status = "contained"` only at Containment stage
- [ ] CrowdStrike detections route: apply StageMask for IOC-related alert records
- [ ] Claroty device/asset route: apply StageMask for `primary_device` and `lateral_devices`
- [ ] Cyberint alert/intelligence route: apply StageMask for IOC-related intelligence records
- [ ] Write integration tests 13, 14, 18 (FAIL first): Armis stage visibility, CrowdStrike containment, concurrent requests

**Phase 6: enrichment clone constructors**

- [ ] Add `ThreatIntelClone::new_with_scenario(entities: &ScenarioEntityCatalog)` to prism-dtu-threatintel
  - Insert all `ioc_ips`, `ioc_domains`, `ioc_hashes` with `FixtureKey::Malicious` into `fixture_registry`
  - Must NOT import from `prism-spec-engine`, `prism-sensors`, `prism-query`
- [ ] Add `NvdClone::new_with_scenario(entities: &ScenarioEntityCatalog)` to prism-dtu-nvd
  - Insert all `device_cves` as synthetic `CveRecord` with `base_score >= 7.0`
  - CVSS vector derived deterministically from CVE ID index
- [ ] Write unit tests 19–23 (FAIL first): IOC correlation, NVD CVE correlation, non-scenario passthrough, construction-time injection

**Phase 7: perimeter gate and #[non_exhaustive] compliance**

- [ ] Audit all new public types in `prism-dtu-common/src/scenario/`: add `#[non_exhaustive]` to `ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage`; confirm `StageMask` does NOT have `#[non_exhaustive]`
- [ ] Run compile-fail gate: `cargo test -p tests-external-perimeter-violation` (or equivalent)
- [ ] Count new `#[non_exhaustive]` pub types; update `EXPECTED=N` in `ci.yml` accordingly
- [ ] Write/verify compile-fail test 24

**Phase 8: final gate**

- [ ] Run `just check` — all 24 Red Gate tests pass; no clippy warnings; fmt clean
- [ ] Confirm BC-2.06.018 PC-4 backward compat regression (test 5) passes
- [ ] Confirm INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 test (test 7) passes
- [ ] Confirm perimeter compile-fail gate (test 24) still passes with zero new violations

---

## Previous Story Intelligence

This is the first story in the EPIC-DEMO live-scenario epic. No predecessor stories in this
exact epic. Relevant intelligence from sibling stories:

**From S-DEMO-MULTI-TENANT-DTU-001 (v1.2, complementary sibling):**
- `prism-dtu-demo-server/src/harness.rs::build_clone_pairs` returns `anyhow::Result<Vec<ClonePair>>`.
  Coordinate with this story's error propagation pattern: `E-DEMO-002` and `E-DEMO-003` use the
  same `anyhow::bail!` / `anyhow::anyhow!` pattern already established for `E-DEMO-001`.
- `#[non_exhaustive]` EXPECTED count in `ci.yml` was bumped to 36 as of S-PLUGIN-PREREQ-C.
  This story must grep `EXPECTED=` in `ci.yml`, note the current value, and increment by the
  exact count of new `#[non_exhaustive]` pub types (expected: 3 — `ScenarioEntityCatalog`,
  `IncidentTimeline`, `IncidentStage`).
- `reqwest::Client` in test HTTP clients must use `.timeout(Duration::from_secs(30))` per
  CLAUDE.md conventions and TD-S-PLUGIN-PREREQ-B-005 open gap record.
- The `feature = "fixture-gen"` flag in `prism-dtu-common` is the correct gating mechanism
  for all new scenario module types.

**From PLUGIN-MIGRATION-001-D lessons (lessons.md entries 16, 17, 24):**
- SAP-1: After implementation, run `rg 'event_type\s*=' crates/ --type rust` to verify any new
  `tracing::*!(event_type=...)` emissions have BC-2.16.002 Structured Event Catalog rows.
  If the scenario coordination logic adds `tracing::info!(event_type="scenario.stage_change", ...)`,
  that catalog row must be in the same commit.
- SID-1: Integration tests that depend on a running DTU HTTP server use `#[ignore]` only if
  the dependency is a live external service. In-process harness tests (spawning tokio tasks)
  are NOT `#[ignore]`'d — they run in CI without external dependencies.

---

## Architecture Compliance Rules

Extracted from ADR-036, ADR-009, ADR-002, ADR-022, and ARCH-INDEX. Non-negotiable.

| Rule | Source | Enforcement |
|------|--------|-------------|
| `current_stage_index` is a pure function: no side effects, no shared mutable state, no tokio `spawn`, no `Arc<AtomicU64>` progression counter | ADR-036 §2.1 | Adversary probe: no `Mutex<StageIndex>` in state structs |
| `StageMask` must NOT carry `#[non_exhaustive]` — it is internal to `prism-dtu-common` and must be exhaustively constructible within the crate | BC-2.06.019 INV-STAGE-MASK-COMPLETENESS-001 | Compile-fail test 24 + adversary |
| `ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage` MUST carry `#[non_exhaustive]` as public types in `prism-dtu-common` | CLAUDE.md #[non_exhaustive] discipline | ci.yml EXPECTED bump |
| Scenario types live ONLY in `prism-dtu-common/src/scenario/` behind `feature = "fixture-gen"` — no separate `prism-dtu-scenario` crate | ADR-036 §3.4 | Adversary: no new crates in Cargo.toml |
| `INV-PERIMETER-001`: `prism-dtu-threatintel` and `prism-dtu-nvd` must NOT gain deps on `prism-spec-engine`, `prism-sensors`, `prism-query` | ADR-036 §2.5, BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 | Compile-fail gate `tests/external/perimeter-violation/` |
| `await_holding_lock = "deny"` (ADR-002 §H1): no `.await` inside a `Mutex` lock guard in route handlers | ADR-002 | clippy deny list |
| Secondary RNG stream `seeded_rng(seed.wrapping_add(1), org_id)` must be a SEPARATE `ChaCha20Rng` instance, not a continuation of the primary stream | ADR-036 §3 cons, BC-2.06.019 INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 | Test 7 (regression) |
| `FixtureSet` is generated ONCE at construction time and filtered per-request — NO per-request re-generation | ADR-036 §2.3 | Adversary: no `generate(...)` call in route handler body |
| `scenario_start_epoch_secs` must be set from `CloneConfig.scenario_start_secs` when `Some`, or from `Utc::now().timestamp()` at `build_clone_pairs` call time when `None` | BC-2.06.019 PC-2 | Test 9, test 11 |
| Forbidden pattern: `Arc::new(SomeThing::placeholder())` in production boot path | ADR-022 §C, CLAUDE.md | Adversary |
| All tracing emission sites with `event_type =` must have BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |

---

## Library & Framework Requirements

Versions pinned from `dependency-graph.md` and `rust-toolchain.toml`. Use these exact versions.
Do NOT invent version numbers from training data.

| Crate | Version | Usage |
|-------|---------|-------|
| `axum` | `0.7` | Route handlers in all prism-dtu-* crates |
| `tokio` | `1` (multi-threaded runtime) | Async runtime per ADR-002 / AD-013 |
| `chrono` | project-pinned | `Utc::now().timestamp()` for `now_epoch_secs` in `current_stage_index` |
| `serde` / `serde_json` | project-pinned | `CloneConfig` / `ScenarioConfig` TOML deserialization |
| `arc-swap` | project-pinned | NOT used for stage progression (pure-function model requires no ArcSwap); use for existing config hot-reload only |
| `rand_chacha` (`ChaCha20Rng`) | project-pinned | `seeded_rng(seed, org_id)` primary + secondary RNG streams |
| `anyhow` | project-pinned | Error propagation in `build_clone_pairs` for E-DEMO-002 / E-DEMO-003 |
| `reqwest` | project-pinned | Integration test HTTP client; `.timeout(Duration::from_secs(30))` mandatory per CLAUDE.md |

**Forbidden versions / patterns:**
- Do NOT introduce `tokio::time::interval` or `tokio::spawn` for stage progression — pure function model only (ADR-036 §3.1)
- Do NOT use `once_cell::sync::Lazy<Mutex<StageIndex>>` or equivalent shared mutable progression state

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-common/src/scenario/mod.rs` | CREATE | `ScenarioEntityCatalog`, `IncidentTimeline`, `IncidentStage`, `StageMask`, `current_stage_index`, `build_scenario_entity_catalog` |
| `crates/prism-dtu-common/src/lib.rs` | MODIFY | Add `pub mod scenario;` under `#[cfg(feature = "fixture-gen")]` |
| `crates/prism-dtu-demo-server/src/config.rs` | MODIFY | Add `ScenarioConfig` struct + `scenario: Option<ScenarioConfig>` field on `CloneConfig` |
| `crates/prism-dtu-demo-server/src/harness.rs` | MODIFY | `build_clone_pairs`: add E-DEMO-002/003 guards, catalog derivation, `Arc<IncidentTimeline>` threading, enrichment catalog injection |
| `crates/prism-dtu-armis/src/state.rs` | MODIFY | Add `timeline: Option<Arc<IncidentTimeline>>` field; add `new_with_scenario` constructor |
| `crates/prism-dtu-armis/src/clone.rs` (or routes/) | MODIFY | Devices route: add scenario projection path with `current_stage_index` + `StageMask` filtering |
| `crates/prism-dtu-crowdstrike/src/state.rs` | MODIFY | Add `timeline: Option<Arc<IncidentTimeline>>` field; add `new_with_scenario` constructor |
| `crates/prism-dtu-crowdstrike/src/generator.rs` | READ-ONLY reference | Confirm `device[0].id` format and `containment_status` field name |
| `crates/prism-dtu-claroty/src/state.rs` | MODIFY | Add `timeline: Option<Arc<IncidentTimeline>>` field; add `new_with_scenario` constructor |
| `crates/prism-dtu-claroty/src/clone.rs` (or routes/) | MODIFY | Device/asset route: add scenario projection path |
| `crates/prism-dtu-cyberint/src/state.rs` | MODIFY | Add `timeline: Option<Arc<IncidentTimeline>>` field; add `new_with_scenario` constructor |
| `crates/prism-dtu-cyberint/src/clone.rs` (or routes/) | MODIFY | Alert/intelligence route: add scenario projection path |
| `crates/prism-dtu-threatintel/src/state.rs` | MODIFY | Add `new_with_scenario(entities: &ScenarioEntityCatalog)` constructor with IOC injection |
| `crates/prism-dtu-nvd/src/state.rs` | MODIFY | Add `new_with_scenario(entities: &ScenarioEntityCatalog)` constructor with CVE injection |
| `.github/workflows/ci.yml` | MODIFY | Bump `EXPECTED=N` by count of new `#[non_exhaustive]` pub types (expected +3: ScenarioEntityCatalog, IncidentTimeline, IncidentStage) |

**Forbidden new files:**
- Do NOT create `crates/prism-dtu-scenario/` — scenario types belong in `prism-dtu-common/src/scenario/` per ADR-036 §3.4
- Do NOT create a new `prism-dtu-harness` integration shim — use existing `prism-dtu-demo-server` harness patterns

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.06.019 EC-019-003 | `now_epoch_secs < scenario_start_epoch_secs` (clock skew / future start) | `elapsed = max(0, now - start) = 0`; stage 0 returned; no panic (AC-011) |
| EC-002 | BC-2.06.019 EC-019-004 | `now_epoch_secs` far past all stage thresholds (e.g., elapsed >> 600s) | Stage index saturates at `stages.len() - 1` (Containment); no index-out-of-bounds panic |
| EC-003 | BC-2.06.019 EC-019-005 | Two scenario-enabled clones with different seeds in same client config | `build_clone_pairs` returns `E-DEMO-002` before any clone constructor is called (AC-015) |
| EC-004 | BC-2.06.019 EC-019-006 | `stage_duration_secs` has fewer entries than archetype stage count | `E-DEMO-003` with message citing provided vs expected count |
| EC-005 | BC-2.06.019 EC-019-007 | `scenario_start_secs = None` | `scenario_start_epoch_secs` set to `Utc::now().timestamp()` at `build_clone_pairs` call time; demo begins at stage 0 |
| EC-006 | BC-2.06.019 EC-019-009 | `scenario_start_secs` set to a past epoch ("demo already in progress") | Correct: elapsed positive at startup; stage index may start at Recon or LateralMovement; operator uses this for mid-scenario start |
| EC-007 | BC-2.06.019 EC-019-011 | Route handler called concurrently from multiple async tasks | `current_stage_index` is pure with no shared mutable state; concurrent calls safe without locking (AC-018) |
| EC-008 | ADR-036 §4 cons | `scenario_start_secs` set to different values for CrowdStrike vs Armis | `E-DEMO-002` fires because seeds also differ (or if seeds are same, stages diverge — operator must set same `scenario_start_secs`; `build_clone_pairs` SHOULD warn if `scenario_start_secs` values differ across enabled clones) |
| EC-009 | BC-2.06.018 EC-018-004 | `fixture_set = "xyzzy_unknown"` for any clone | Construction-time `E-DEMO-001`; `build_clone_pairs` returns `Err` (AC-004) |
| EC-010 | BC-2.06.020 EC-020-002 | Catalog has `ioc_ips = []` (empty; e.g., seed produces zero IPs) | No entries inserted for IPs; no error; non-empty catalog fields still injected normally |
| EC-011 | BC-2.06.020 EC-020-003 | Same IOC appears in scenario catalog AND pre-existing default registry as Benign | HashMap::insert overwrites; `FixtureKey::Malicious` wins (scenario injection takes priority) |
| EC-012 | BC-2.06.020 EC-020-011 | `scenario.enabled = true` for operational DTUs, `scenario.enabled = false` for ThreatIntel | ThreatIntel uses static default; scenario IOCs will NOT resolve as Malicious; this is a valid (if incomplete) operator config — no `build_clone_pairs` error |
| EC-013 | ADR-036 §2.2 | `seed = u64::MAX` passed to `seeded_rng(seed.wrapping_add(1), org_id)` | `wrapping_add(1)` wraps to `0`; `seeded_rng(0, org_id)` is valid; no panic; secondary stream still independent of primary |
| EC-014 | ADR-036 §5 Option D rejected | ThreatIntel/NVD generator (new full generator) | NOT implemented — lookup injection is the correct mechanism per ADR-036 §3.2. If an implementer finds a reason to introduce a full generator, this must be escalated to architect. |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `ScenarioEntityCatalog` | `prism-dtu-common/src/scenario/` | Pure (data struct, no I/O) | ADR-036 §2.2 |
| `IncidentTimeline` | `prism-dtu-common/src/scenario/` | Pure (data struct, no I/O) | ADR-036 §2.2 |
| `current_stage_index` | `prism-dtu-common/src/scenario/` | Pure (function: `(&IncidentTimeline, i64) -> usize`) | ADR-036 §2.1 |
| `build_scenario_entity_catalog` | `prism-dtu-common/src/scenario/` | Pure (deterministic from seed+org_id) | ADR-036 §2.2 |
| `ScenarioConfig` | `prism-dtu-demo-server/src/config.rs` | Pure (TOML deserialization struct) | ADR-036 §2.4 |
| `build_clone_pairs` (scenario coordination logic) | `prism-dtu-demo-server/src/harness.rs` | Effectful (constructs clones, reads config) | ADR-036 §2.4 |
| `ArmisState.timeline` / route projection | `prism-dtu-armis/src/state.rs` + routes | Effectful (HTTP handler calling pure `current_stage_index`) | ADR-036 §2.3 |
| `CrowdstrikeState.timeline` / route projection | `prism-dtu-crowdstrike/src/state.rs` + routes | Effectful | ADR-036 §2.3 |
| `ClarotyState.timeline` / route projection | `prism-dtu-claroty/src/state.rs` + routes | Effectful | ADR-036 §2.3 |
| `CyberintState.timeline` / route projection | `prism-dtu-cyberint/src/state.rs` + routes | Effectful | ADR-036 §2.3 |
| `ThreatIntelClone::new_with_scenario` | `prism-dtu-threatintel/src/state.rs` | Effectful (constructor: mutates `fixture_registry` at init) | ADR-036 §2.3, BC-2.06.020 PC-1 |
| `NvdClone::new_with_scenario` | `prism-dtu-nvd/src/state.rs` | Effectful (constructor: mutates `cve_registry` at init) | ADR-036 §2.3, BC-2.06.020 PC-3 |

---

## Forbidden Dependencies

These dependencies MUST NOT appear in the `[dependencies]` section of the following crates'
`Cargo.toml` files. If they appear, the build MUST fail (enforced by the compile-fail gate
in `tests/external/perimeter-violation/`).

| Crate | Forbidden Dependency | Reason |
|-------|---------------------|--------|
| `prism-dtu-threatintel` | `prism-spec-engine` | INV-PERIMETER-001 / ADR-036 §2.5 |
| `prism-dtu-threatintel` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-threatintel` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-sensors` | INV-PERIMETER-001 |
| `prism-dtu-nvd` | `prism-query` | INV-PERIMETER-001 |
| `prism-dtu-common` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-armis` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-crowdstrike` | `prism-spec-engine` | INV-PERIMETER-001 |
| Any new crate | `prism-dtu-scenario` (does not exist) | ADR-036 §3.4 — no separate crate; scenario types live in `prism-dtu-common` |

The only permitted new cross-DTU dependency is: `prism-dtu-threatintel` → `prism-dtu-common` (feature = "fixture-gen") and `prism-dtu-nvd` → `prism-dtu-common` (feature = "fixture-gen"), if these dependencies do not already exist. All other `prism-dtu-*` crates already depend on `prism-dtu-common`.

---

## SAP-1 Compliance (Structured Event Catalog)

Per CLAUDE.md §SAP-1, any `tracing::*!(event_type = "...")` emission site added in this story
requires a corresponding row in BC-2.16.002 Structured Event Catalog with:
- event_type value
- emitting module
- field schema
- audit role
- recurrence policy

If the scenario coordination logic in `build_clone_pairs` or the route projection handlers add
`event_type` emissions, those catalog rows MUST be in the same commit. Removal of an emission
(e.g., replaced by `?` propagation) does NOT require a new catalog row per D-765 precedent.

Expected new emissions (implementer must enumerate actual sites):
- Potentially `event_type = "scenario.construction"` in `build_clone_pairs`
- Potentially `event_type = "scenario.stage_change"` in route handlers (if logging stage transitions)

If NO new `event_type` emissions are added, state so explicitly in the PR description.
