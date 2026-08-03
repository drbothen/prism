---
document_type: story
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-A
title: "Baseline Seeding Retrofit — Wire Seeded Generators into Demo-Server Clones for Per-Client Distinct Data"
wave: 5
epic_id: E-DEMO
priority: P2
status: ready
version: "1.6"
level: "L4"
producer: story-writer
timestamp: "2026-06-09T00:00:00Z"
created: "2026-06-09"
modified: "2026-06-09T23:00:00Z"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns all prism-dtu-* crates including prism-dtu-demo-server,
#   prism-dtu-common, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty,
#   prism-dtu-cyberint, prism-dtu-threatintel, and prism-dtu-nvd per ARCH-INDEX Subsystem
#   Registry. The baseline seeding retrofit lives entirely within the SS-01 DTU clone family.
#   Decision anchor: ADR-036 subsystems_affected: [SS-01].
target_module: prism-dtu-common
crates_touched: [prism-dtu-common, prism-dtu-demo-server, prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty, prism-dtu-cyberint, prism-dtu-threatintel, prism-dtu-nvd]
behavioral_contracts: [BC-2.06.018]
verification_properties: [VP-018-A, VP-018-B, VP-018-C, VP-018-D, VP-018-E]
depends_on:
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001
  # Dependency anchor: S-CONFIG-MULTI-TENANT-OVERRIDE-001 delivers per-org overlay plumbing in
  # prism-spec-engine and prism-sensors (merged, develop@3e822522). This story's integration
  # tests exercise multi-client seeding correctness end-to-end; the overlay plumbing being in
  # place ensures base_url routing for per-client demo instances works.
blocks:
  - S-DEMO-DTU-LIVE-SCENARIO-001-B
  # Dependency anchor: Story B (scenario progression + enrichment correlation) depends on this
  # story because its new_with_scenario constructors build on top of the new_with_seed substrate
  # introduced here. Story B's stage-mask filtering over generated_records is impossible without
  # Story A's generator wiring into the serving path.
points: 8
# Points justification (ADR-036 §8 Story A estimate):
#   1. prism-dtu-common/src/scenario/ module stub: ScenarioEntityCatalog + org_slug_from_org_id
#      + secondary RNG stream catalog derivation (IOC IPs, domains, hashes, CVE IDs): 1.5 pts
#   2. DemoConfig/CloneConfig extension: org_id Option<String> + ScenarioConfig struct: 1 pt
#   3. Per-clone new_with_seed constructors (CrowdStrike, Armis, Claroty, Cyberint): 2 pts
#      Each calls generate() under fixture-gen, stores records in new state field
#   4. Route handler dual-path logic (4 clones × 1-2 routes each): 1.5 pts
#   5. build_clone_pairs seed-forwarding + E-DEMO-004/005 guards: 0.5 pts
#   6. Cargo.toml fixture-gen additions (threatintel, nvd) + ci.yml EXPECTED bump: 0.5 pts
#   7. Red Gate test suite (~14 tests, FAIL-first): 1 pt
#   Total: 8 pts
estimated_days: 3
risk: HIGH
# Risk justification:
#   Touches 8 crates; introduces new constructor paths to 4 generator-backed clones;
#   route-handler dual-path logic must not break existing static-JSON serving path;
#   INV-DISTINCT-DATA-001 requires exact canonical ID format per ADR-036 §2.2;
#   ci.yml EXPECTED count must be computed precisely; Cyberint new_with_seed is fallible
#   (Cyberint::new() is already fallible), requiring consistent error propagation.
acceptance_criteria_count: 14
red_gate_tests: 17
estimated_passes: "3-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "INV-DISTINCT-DATA-001: canonical device ID format is 'dev-{8hex}-{seed}-{n}' where 8hex = hex(org_id.as_bytes()[0..4]). Any test using 'dev-acme-...' is incorrect (ADR-036 §3.5 / BC-2.06.018 §Canonical Org Slug). Test vectors must use a real org UUID and derive expected IDs from it."
  - "Backward compat (BC-2.06.018 postcondition 4): new() static-JSON path must remain unchanged. new_with_seed(42, Archetype::HealthyOtEnvironment, default_org) must produce data semantically equivalent to pre-Story-A new() behavior. Regression test (RG-A-005) verifies this before PR can merge."
  - "Cyberint new_with_seed fallibility: CyberintClone::new() is already anyhow::Result<Self>; new_with_seed must also be anyhow::Result<Self>. build_clone_pairs propagates the error consistently."
  - "Archetype forwarding — NO hardcoded CompromisedEndpoint: build_clone_pairs maps fixture_set→Archetype via INV-FIXTURE-SET-ARCHETYPE-MAP-001 and forwards the variant to each new_with_seed(seed, archetype, org_id). The old 2-arg form new_with_seed(seed, org_id) and any call with literal Archetype::CompromisedEndpoint in harness.rs are SUPERSEDED by ADR-036 v2.2 (F-P6-HIGH-001 closure). The archetype arg reaches generate() and changes served data — differential Red Gate tests enforce this."
  - "Armis org_slug is derived INTERNALLY: ArmisClone::new_with_seed takes (seed, archetype, org_id) — NOT (seed, org_id, org_slug). org_slug_from_org_id(&org_id) is called inside new_with_seed before forwarding to generate(). The old 3-arg-with-org_slug form AC-004 v1.3 is superseded (ADR-036 v2.2)."
  - "reqwest::Client timeout: any new HTTP client in integration tests uses .timeout(Duration::from_secs(30)) per CLAUDE.md conventions."
  - "#[non_exhaustive] EXPECTED bump: implementer must read the current EXPECTED=N value in ci.yml, count new #[non_exhaustive] pub types added (ScenarioEntityCatalog = 1), and increment EXPECTED atomically. ScenarioEntityCatalog is the only new public #[non_exhaustive] type in this story; IncidentTimeline/IncidentStage/StageMask are Story B scope."
  - "INV-PERIMETER-001: fixture-gen feature additions to prism-dtu-threatintel and prism-dtu-nvd Cargo.toml must only add prism-dtu-common/fixture-gen as transitive dep. No prism-spec-engine/prism-sensors/prism-query dependency may be introduced. Verified by existing compile-fail gate."
  - "gen_seeded_rng signature: use `gen_seeded_rng(seed: u64, org_id: &OrgId)` (2-arg XOR-formula function, re-exported from prism-dtu-common lib.rs; Cyberint already imports it). The bare 1-arg `seeded_rng` is a legacy helper — do NOT use it. Takes &OrgId ([u8;16]), NOT &str. All four generators derive org_slug internally (via org_slug_from_org_id); org_slug is NOT a new_with_seed constructor argument for any clone (ADR-036 v2.2)."
  - "ArmisClone::new_with_seed fallibility: ArmisClone::new() is already fallible (crates/prism-dtu-armis/src/clone.rs:58); new_with_seed must also return anyhow::Result<Self>. build_clone_pairs propagates the error via ? consistently with Cyberint."
traces_to: [D-1077, ADR-036, F-P6-HIGH-001]
supersedes: []
---

# S-DEMO-DTU-LIVE-SCENARIO-001-A: Baseline Seeding Retrofit

Wire the deterministic generator into demo-server clone serving paths so each demo client
receives data from a seeded `FixtureSet`, producing disjoint device/alert/detection ID sets
across clients. Implements BC-2.06.018 (config-time seed + org_id forwarding), which is
currently **unimplemented** per ADR-036 v2.0 §1.3 substrate correction. This is the prerequisite
for Story B (scenario progression), which requires generated_records to be in the serving path.

---

## Authority

ADR-036 is the authoritative design document for this story. Read it before implementing:
`.factory/specs/architecture/decisions/ADR-036-*.md`.

ADR-036 §2.2 defines the canonical `org_slug_from_org_id` formula (`hex(org_id.as_bytes()[0..4])`),
`ScenarioEntityCatalog` structure, and the `gen_seeded_rng` two-arg secondary RNG stream. ADR-036
§2.3 defines the generator-backed clone constructor pattern (`new_with_seed`) and the
`generated_records` substrate in clone state. ADR-036 §2.4 defines `ScenarioConfig` struct fields.
ADR-036 §2.5 defines perimeter rules (INV-PERIMETER-001: no `prism-spec-engine`, `prism-sensors`,
or `prism-query` deps in DTU crates). ADR-036 §3.2 prohibits per-request `generate()` calls.
ADR-036 §3.4 prohibits a separate scenario crate.

ADR-036 `status: ACCEPTED`. `superseded_by: null`.

BC-2.06.018 §Postconditions governs config-time seed and org_id forwarding (all 14 ACs). Invariants
INV-DISTINCT-DATA-001, INV-FIXTURE-SET-ARCHETYPE-MAP-001, INV-CONSTRUCTION-TIME-FAILURE-001, and
INV-CONFIGURE-ENDPOINT-SECONDARY-001 are binding behavioral invariants.

Parent story note: S-DEMO-DTU-LIVE-SCENARIO-001 is `status: superseded` (superseded by this story
and S-DEMO-DTU-LIVE-SCENARIO-001-B). This story covers BC-2.06.018 scope.

---

## Narrative

As a demo operator configuring a multi-client SOC demo, I want each client's DTU clones to
serve data derived from a unique seed and org UUID, so that two analysts investigating different
demo clients see completely different device IDs, alert IDs, and detection IDs that cannot be
confused with each other.

**Goal:** After this story, `demo.toml` entries with `seed = N`, `fixture_set = "<archetype-name>"`, and `org_id = "<uuid>"` in
`CloneConfig` cause `build_clone_pairs` to map `fixture_set→Archetype` (per INV-FIXTURE-SET-ARCHETYPE-MAP-001) and call `new_with_seed(N, mapped_archetype, org_id)` on each generator-backed
clone, which calls `generate(..., mapped_archetype, ...)` and stores the resulting records in a new `generated_records`
field in state. Route handlers serve from `generated_records` when present, falling back to the
existing static-JSON path when absent (backward compat). Two clients with different seeds
produce pairwise-disjoint canonical ID sets per INV-DISTINCT-DATA-001. The `mapped_archetype` drives
the content of served records — `dormant` → empty responses, `large_scale` → large record sets, etc.
No hardcoded `Archetype::CompromisedEndpoint` exists anywhere in harness.rs or constructors.

---

## Behavioral Contracts

| BC | Title | Key Invariants |
|----|-------|----------------|
| BC-2.06.018 v1.5 | Demo-Server Config-Time Data Seeding — Per-Clone seed + fixture_set Wire-Up | INV-DISTINCT-DATA-001, INV-FIXTURE-SET-ARCHETYPE-MAP-001, INV-CONSTRUCTION-TIME-FAILURE-001, INV-CONFIGURE-ENDPOINT-SECONDARY-001 |

---

## Acceptance Criteria

### AC-001 — ScenarioEntityCatalog stub constructed from (seed, org_id) via secondary RNG stream
(traces to BC-2.06.018 precondition 4 and ADR-036 §2.2)

Given a call to `build_scenario_entity_catalog(seed: u64, org_id: &OrgId)`,
when the function executes,
then it produces a `ScenarioEntityCatalog` with:
- `org_slug = hex(org_id.as_bytes()[0..4])` — 8 lowercase hex chars (e.g., `"deadbeef"` for org bytes `[0xde, 0xad, 0xbe, 0xef, ...]`)
- `primary_device_id_cs = "dev-{org_slug}-{seed}-0"` (e.g., `"dev-deadbeef-42-0"`)
- `primary_device_id_armis = "dev-{org_slug}-{seed}-0"` (same formula; Armis generator receives org_slug as `&str` arg)
- non-empty `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves` derived via `gen_seeded_rng(seed.wrapping_add(1), &org_id)` (secondary RNG stream, independent of primary stream; `gen_seeded_rng` is the 2-arg XOR-formula function re-exported from `prism-dtu-common::rng` — takes `(u64, &OrgId)`, NOT the 1-arg legacy `seeded_rng`)

Red Gate: `test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids`

### AC-002 — org_slug_from_org_id produces canonical 8-hex-char slug
(traces to BC-2.06.018 §Canonical Org Slug and ADR-036 §2.2)

Given an `OrgId` whose first 4 bytes are `[0xde, 0xad, 0xbe, 0xef]`,
when `org_slug_from_org_id(&org_id)` is called,
then it returns `"deadbeef"` (8 lowercase hex characters);
and for any `OrgId`, the returned string is exactly 8 characters of `[0-9a-f]`.

Red Gate: `test_BC_2_06_018_org_slug_from_org_id_canonical_format`

### AC-003 — new_with_seed(seed, archetype, org_id) forwarded from build_clone_pairs to CrowdStrike clone
(traces to BC-2.06.018 postcondition 1 and ADR-036 v2.2 canonical 3-arg constructor)

Given `demo.toml` with `clones.crowdstrike.seed = 100`, `clones.crowdstrike.org_id = "<uuid>"`, and `fixture_set = "compromised"`,
when `build_clone_pairs` runs,
then `CrowdstrikeClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id) -> Self` is called (archetype forwarded from the INV-FIXTURE-SET-ARCHETYPE-MAP-001 mapping, not hardcoded in harness.rs),
and the resulting `/devices/entities/devices/v2` responses contain device IDs in `"dev-{8hex}-100-{n}"` format,
and these IDs are different from those of a clone seeded with `seed = 200`.

Red Gate: `test_BC_2_06_018_crowdstrike_new_with_seed_forwarded`

### AC-004 — new_with_seed(seed, archetype, org_id) forwarded to Armis clone; org_slug derived internally (fallible)
(traces to BC-2.06.018 postcondition 1 and ADR-036 v2.2 canonical 3-arg constructor)

Given `demo.toml` with `clones.armis.seed = 100`, `clones.armis.org_id = "<uuid>"`, and `fixture_set = "compromised"`,
when `build_clone_pairs` runs,
then `ArmisClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id) -> anyhow::Result<Self>` is called
  (mirrors `ArmisClone::new()` which is fallible per `crates/prism-dtu-armis/src/clone.rs:58`);
  and org_slug is derived INTERNALLY inside new_with_seed via `org_slug_from_org_id(&org_id)` before being forwarded to `generate()` — org_slug is NOT a constructor argument (ADR-036 v2.2 supersedes the old 3-arg-with-org_slug form);
  and the resulting `/api/v1/devices` responses contain device records with IDs in `"dev-{8hex}-100-{n}"` format;
  and a construction error propagates through `build_clone_pairs`'s `anyhow::Result<Vec<ClonePair>>` return via `?`.

Red Gate: `test_BC_2_06_018_armis_new_with_seed_canonical_3arg`

### AC-005 — INV-DISTINCT-DATA-001: disjoint ID sets for distinct seeds
(traces to BC-2.06.018 invariant INV-DISTINCT-DATA-001)

Given two demo clients with `seed_A = 100, org_id_A = "<uuid-A>"` and `seed_B = 200, org_id_B = "<uuid-B>"` (distinct seeds),
when both clients' Armis (or CrowdStrike) clones are constructed and queried,
then the sets of device IDs in their responses are pairwise-disjoint: `ids_A ∩ ids_B = ∅`;
and both ID sets follow the canonical `"dev-{8hex}-{seed}-{n}"` format.

Red Gate: `test_BC_2_06_018_distinct_seeds_disjoint_ids`

### AC-006 — Backward compat: new() static-JSON path unchanged; seed=42 + fixture_set="default" is byte-identical
(traces to BC-2.06.018 postcondition 4)

Given `CloneConfig.seed = 42` (default) and `CloneConfig.fixture_set = "default"` (default) for all clones,
when both the legacy `CloneType::new()` path (no seed, no org_id) and the new `new_with_seed(42, HealthyOtEnvironment, default_org)` path are exercised,
then all existing integration tests that passed against the pre-seeding constructor continue to pass without modification;
and a clone constructed with `new()` (absent `org_id`) uses the static-JSON fallback path byte-identically to pre-Story-A behavior;
and for `ArmisClone`, the fallible `new_with_seed(...) -> anyhow::Result<Self>` must not regress the existing fallible `new() -> anyhow::Result<Self>` behavior (both propagate errors consistently via `?` in `build_clone_pairs`).

Red Gate: `test_BC_2_06_018_backward_compat_seed42_default`

### AC-007 — INV-FIXTURE-SET-ARCHETYPE-MAP-001: fixture_set → Archetype canonical mapping, archetype reaches generator and changes served data
(traces to BC-2.06.018 invariant INV-FIXTURE-SET-ARCHETYPE-MAP-001 and ADR-036 v2.2 §4)

Given each of the 8 canonical `fixture_set` strings (`"default"`, `"compromised"`, `"auth_outage"`, `"large_scale"`, `"pagination_edges"`, `"schema_drift"`, `"high_churn"`, `"dormant"`),
when `build_clone_pairs` constructs the clone with that `fixture_set`,
then the correct `Archetype` variant is selected with no construction-time error (construction `is_ok()` for all 8);
AND the mapped archetype actually reaches `generate()` and changes the served route output — verified by differential assertions:
  - `fixture_set = "dormant"` → served device/alert response bodies are EMPTY (zero records) per BC-2.06.018 EC-018-003 / TV-018-006;
  - `fixture_set = "large_scale"` → served record count matches the `LargeScale` archetype baseline (implementer reads the generator's `LargeScale` branch to get the canonical expected count — do NOT assume 10 000 if the generator uses a different scale value);
  - same seed + org_id, `fixture_set = "compromised"` vs `fixture_set = "dormant"` → DIFFERENT served route output (proves archetype drives output, not merely construction);
and given `fixture_set = "xyzzy_unknown"`, then `build_clone_pairs` returns `Err` containing `"E-DEMO-001"`.

NOTE: The differential assertions in this AC are load-bearing. An implementation that passes `is_ok()` but does NOT forward the archetype to `generate()` (e.g., hardcoded `Archetype::CompromisedEndpoint`) will fail the dormant-empty-response and compromised-vs-dormant differential assertions. This is the TD-VSDD-059 paper-test upgrade per F-P6-HIGH-001.

Red Gate: `test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error`
Additional differential Red Gates — see §Red Gate Test Plan rows 15, 16, 17 below.

### AC-008 — E-DEMO-001 propagates at construction, not request time (INV-CONSTRUCTION-TIME-FAILURE-001)
(traces to BC-2.06.018 invariant INV-CONSTRUCTION-TIME-FAILURE-001)

Given `fixture_set = "bad_value"` for any clone,
when `build_clone_pairs` is called,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-001"`, the clone name, and the invalid value; and the process does not panic at request-handling time.

Red Gate: `test_BC_2_06_018_e_demo_001_at_construction_not_request_time`

### AC-009 — E-DEMO-004: missing org_id when new_with_seed called fails at construction
(traces to BC-2.06.018 §Error Codes E-DEMO-004 and ADR-036 §6)

Given a clone config with `fixture_set = "compromised"` (which maps to `CompromisedEndpoint` archetype) but `org_id` absent (`None`),
when `build_clone_pairs` attempts to construct it via `new_with_seed`,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-004"` and the clone name; the error surfaces before any clone constructor is called.

Red Gate: `test_BC_2_06_018_e_demo_004_absent_org_id_at_construction`

### AC-010 — E-DEMO-005: invalid UUID in org_id fails at construction
(traces to BC-2.06.018 §Error Codes E-DEMO-005 and ADR-036 §6)

Given a clone config with `org_id = "not-a-valid-uuid"`,
when `build_clone_pairs` parses the org_id,
then it returns `Err(e)` where `e.to_string()` contains `"E-DEMO-005"`, the clone name, and the invalid value; no clone constructor is called.

Red Gate: `test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction`

### AC-011 — new_with_seed(seed, archetype, org_id) forwarded to Claroty clone (infallible)
(traces to BC-2.06.018 postcondition 1 and ADR-036 v2.2 canonical 3-arg constructor)

Given `demo.toml` with `clones.claroty.seed = 100`, `clones.claroty.org_id = "<uuid>"`, and `fixture_set = "compromised"`,
when `build_clone_pairs` runs,
then `ClarotyClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id) -> Self` is called (archetype forwarded from INV-FIXTURE-SET-ARCHETYPE-MAP-001 mapping, NOT hardcoded in harness.rs; org_slug derived internally in new_with_seed);
and the route handlers serve from `generated_records` when non-empty.

Red Gate: `test_BC_2_06_018_claroty_new_with_seed_forwarded`

### AC-012 — new_with_seed(seed, archetype, org_id) forwarded to Cyberint clone (fallible)
(traces to BC-2.06.018 postcondition 1 and ADR-036 v2.2 canonical 3-arg constructor)

Given `demo.toml` with `clones.cyberint.seed = 100`, `clones.cyberint.org_id = "<uuid>"`, and `fixture_set = "compromised"`,
when `build_clone_pairs` runs,
then `CyberintClone::new_with_seed(100, Archetype::CompromisedEndpoint, org_id) -> anyhow::Result<Self>` is called (archetype forwarded from INV-FIXTURE-SET-ARCHETYPE-MAP-001 mapping, NOT hardcoded in harness.rs; mirrors existing `CyberintClone::new() -> anyhow::Result<Self>` fallibility);
and a construction error propagates through `build_clone_pairs`'s `anyhow::Result<Vec<ClonePair>>` return via `?`.

Red Gate: `test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible`

### AC-013 — fixture-gen feature additions compile cleanly for threatintel + nvd
(traces to BC-2.06.018 precondition 3 and ADR-036 §2.3 enrichment clones + §2.5 perimeter)

Given `fixture-gen = ["prism-dtu-common/fixture-gen"]` added to `prism-dtu-threatintel/Cargo.toml` and `prism-dtu-nvd/Cargo.toml`,
when `cargo build -p prism-dtu-threatintel --features fixture-gen` and `cargo build -p prism-dtu-nvd --features fixture-gen` run,
then both compile without error; and the compile-fail gate in `tests/external/perimeter-violation/` passes with zero new violations (INV-PERIMETER-001: no dep on prism-spec-engine/prism-sensors/prism-query introduced).

Red Gate: `test_BC_2_06_018_perimeter_compile_fail_gate_passes_after_feature_additions` (compile-fail)

### AC-014 — ci.yml EXPECTED bumped atomically for new #[non_exhaustive] types
(traces to BC-2.06.018 and CLAUDE.md #[non_exhaustive] discipline)

Given `ScenarioEntityCatalog` is a new public `#[non_exhaustive]` type in `prism-dtu-common/src/scenario/`,
when the implementer runs the non-exhaustive compile-fail gate (`tests/external/non-exhaustive-violation/`),
then `EXPECTED` in `ci.yml` is incremented by the exact count of new `#[non_exhaustive]` pub types added in this story (at minimum 1: `ScenarioEntityCatalog`; implementer must verify by running the gate and reading the violation count);
and `tests/external/non-exhaustive-violation/Cargo.toml` adds `prism-dtu-common = { path = "../../../crates/prism-dtu-common", features = ["fixture-gen"] }` under `[dependencies]` (the violation crate currently has no prism-dtu-* dependency; this dep addition is required for the `ScenarioEntityCatalog` import row to resolve);
and the non-exhaustive-violation crate includes an import row for `prism-dtu-common::scenario::ScenarioEntityCatalog`.

Red Gate: (compile-fail gate output — implementer verifies EXPECTED count matches actual violations before committing)

---

## Red Gate Test Plan

All tests written FAIL-first (stub → red → implement → green) per SID-1. Unit tests in
`#[cfg(test)] mod tests` blocks; integration tests in `crates/<crate>/tests/`. No `#[ignore]`
unless a specific external-service dependency is cited.

| # | Test Name | Crate | BC | Type |
|---|-----------|-------|-----|------|
| 1 | `test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids` | prism-dtu-common | BC-2.06.018 PC-4 / ADR-036 §2.2 | unit |
| 2 | `test_BC_2_06_018_org_slug_from_org_id_canonical_format` | prism-dtu-common | BC-2.06.018 §Canonical Org Slug | unit |
| 3 | `test_BC_2_06_018_crowdstrike_new_with_seed_forwarded` | prism-dtu-crowdstrike | BC-2.06.018 PC-1 | unit |
| 4 | `test_BC_2_06_018_armis_new_with_seed_canonical_3arg` | prism-dtu-armis | BC-2.06.018 PC-1 / ADR-036 v2.2 | unit |
| 5 | `test_BC_2_06_018_distinct_seeds_disjoint_ids` | prism-dtu-demo-server | BC-2.06.018 INV-DISTINCT-DATA-001 | integration |
| 6 | `test_BC_2_06_018_backward_compat_seed42_default` | prism-dtu-demo-server | BC-2.06.018 PC-4 | regression |
| 7 | `test_BC_2_06_018_fixture_set_archetype_mapping_all_8_valid_plus_error` | prism-dtu-demo-server | BC-2.06.018 INV-FIXTURE-SET-ARCHETYPE-MAP-001 | unit |
| 8 | `test_BC_2_06_018_e_demo_001_at_construction_not_request_time` | prism-dtu-demo-server | BC-2.06.018 INV-CONSTRUCTION-TIME-FAILURE-001 | unit |
| 9 | `test_BC_2_06_018_e_demo_004_absent_org_id_at_construction` | prism-dtu-demo-server | BC-2.06.018 E-DEMO-004 | unit |
| 10 | `test_BC_2_06_018_e_demo_005_invalid_uuid_at_construction` | prism-dtu-demo-server | BC-2.06.018 E-DEMO-005 | unit |
| 11 | `test_BC_2_06_018_claroty_new_with_seed_forwarded` | prism-dtu-claroty | BC-2.06.018 PC-1 | unit |
| 12 | `test_BC_2_06_018_cyberint_new_with_seed_forwarded_fallible` | prism-dtu-cyberint | BC-2.06.018 PC-1 | unit |
| 13 | `test_BC_2_06_018_perimeter_compile_fail_gate_passes_after_feature_additions` | tests/external/perimeter-violation | BC-2.06.018 / INV-PERIMETER-001 | compile-fail |
| 14 | (ci.yml EXPECTED gate) | .github/workflows + tests/external/non-exhaustive-violation | CLAUDE.md discipline | compile-fail |
| 15 | `test_BC_2_06_018_dormant_archetype_empty_served_response` | prism-dtu-demo-server | BC-2.06.018 EC-018-003 / TV-018-006 / INV-FIXTURE-SET-ARCHETYPE-MAP-001 | route-output |
| 16 | `test_BC_2_06_018_large_scale_archetype_record_count` | prism-dtu-demo-server | BC-2.06.018 EC-018-005 / INV-FIXTURE-SET-ARCHETYPE-MAP-001 | route-output |
| 17 | `test_BC_2_06_018_archetype_drives_served_output_differential` | prism-dtu-demo-server | BC-2.06.018 INV-FIXTURE-SET-ARCHETYPE-MAP-001 / ADR-036 v2.2 | route-output (differential) |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~5 500 |
| ADR-036 v2.2 (full) | ~5 500 |
| BC-2.06.018 v1.5 (full) | ~3 500 |
| prism-dtu-common/src/generator/{archetype,rng,opts,fixture}.rs | ~2 500 |
| prism-dtu-demo-server/src/{harness,config}.rs | ~2 000 |
| prism-dtu-armis/src/{state,clone,generator}.rs | ~1 800 |
| prism-dtu-crowdstrike/src/{state,clone,generator}.rs | ~1 800 |
| prism-dtu-claroty/src/{state,clone}.rs | ~1 200 |
| prism-dtu-cyberint/src/{state,clone}.rs | ~1 200 |
| prism-dtu-threatintel/src/{state,clone}.rs + Cargo.toml | ~900 |
| prism-dtu-nvd/src/{state,clone,types}.rs + Cargo.toml | ~1 000 |
| ci.yml (EXPECTED line + feature flags) | ~200 |
| Test files (17 stubs × ~40 lines each) | ~2 200 |
| Tool outputs (nextest, clippy, compile-fail) | ~2 000 |
| **Total estimate** | **~31 300** |

At ~200k context window, this is ~16% — well within the 20-30% ceiling.

---

## Tasks

Implementation checklist (TDD order — write failing tests before each implementation step):

**Phase 1: prism-dtu-common scenario stub module**

- [ ] Create `crates/prism-dtu-common/src/scenario/mod.rs` (behind `#[cfg(feature = "fixture-gen")]`)
- [ ] Define `ScenarioEntityCatalog` (`#[non_exhaustive]`, `#[derive(Clone, Debug)]`) with all fields per ADR-036 §2.2
- [ ] Implement `org_slug_from_org_id(org_id: &OrgId) -> String` — formula: `hex(org_id.as_bytes()[0..4])`
- [ ] Implement `build_scenario_entity_catalog(seed: u64, org_id: &OrgId) -> ScenarioEntityCatalog` using `gen_seeded_rng(seed.wrapping_add(1), &org_id)` (secondary RNG stream; completely separate `ChaCha20Rng` instance from primary; `gen_seeded_rng` is the 2-arg XOR-formula function re-exported from `prism-dtu-common::rng`, NOT the 1-arg legacy `seeded_rng`)
- [ ] Export `scenario` module from `prism-dtu-common/src/lib.rs` (under `#[cfg(feature = "fixture-gen")]`)
- [ ] Write unit tests 1-2 (FAIL first): catalog secondary RNG + canonical IDs, org_slug format

**Phase 2: DemoConfig/CloneConfig extension**

- [ ] Read `crates/prism-dtu-demo-server/src/config.rs` fully before editing
- [ ] Add `org_id: Option<String>` field to `CloneConfig` (UUID string; parses to `OrgId` in `build_clone_pairs`)
- [ ] Add `ScenarioConfig` struct (`#[derive(Debug, Clone, Deserialize, Default)]`) per ADR-036 §2.4 fields: `enabled`, `archetype`, `scenario_start_secs`, `stage_duration_secs`
- [ ] Add `scenario: Option<ScenarioConfig>` field to `CloneConfig`

**Phase 3: Per-clone new_with_seed constructors (FAIL-first for each)**

- [ ] Read CrowdStrike `state.rs`, `clone.rs`, `generator.rs` before editing
- [ ] Add `generated_devices: Vec<serde_json::Value>` and `generated_detections: Vec<serde_json::Value>` to `CrowdstrikeState`
- [ ] Add `CrowdstrikeClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> Self` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg signature): accepts `archetype: Archetype` parameter and forwards it to `generate(org_id, archetype, GenOpts { seed, ..GenOpts::default() })` (use `..GenOpts::default()` — NOT bare `..`; GenOpts has 4 fields and is not `#[non_exhaustive]`, so bare `..` is a syntax error); stores records in `generated_devices` / `generated_detections` by `_record_type` filter. DO NOT hardcode `Archetype::CompromisedEndpoint` here; the caller (build_clone_pairs) provides the mapped variant.
- [ ] Modify `routes/hosts.rs`: serve `generated_devices` when non-empty; fall back to `load_host_ids()` / `load_host_details()` (static embedded JSON in routes/hosts.rs — the READ fallback, NOT `containment_store` which is a write-target overlay) when `generated_devices` is empty
- [ ] Modify `routes/detections.rs`: serve `generated_detections` when non-empty; fall back to the existing static detection path when `generated_detections` is empty
- [ ] Write unit test 3 (FAIL first): CrowdStrike new_with_seed forwarded

- [ ] Read Armis `state.rs`, `clone.rs`, `generator.rs` before editing
- [ ] Add `generated_records: Vec<serde_json::Value>` to `ArmisState` (alongside existing `devices_ordered`, `alert_fixture`)
- [ ] Add `ArmisClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> anyhow::Result<Self>` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg signature; fallible — mirrors existing `new() -> anyhow::Result<Self>`; `build_clone_pairs` uses `?` to propagate): INTERNALLY derive `org_slug = org_slug_from_org_id(&org_id)`, then call `generate(org_id, &org_slug, archetype, &GenOpts { seed, ..GenOpts::default() })` (Armis generator takes org_id BY VALUE as `OrgId`, org_slug as `&str`, archetype by value, opts BY REF as `&GenOpts`); stores FixtureSet records in `generated_records`. DO NOT add org_slug as a constructor parameter — it is derived internally (ADR-036 v2.2 supersedes the old 3-arg-with-org_slug form). DO NOT hardcode `Archetype::CompromisedEndpoint` here.
- [ ] Modify `routes/devices.rs` `paginate_devices`: serve from `generated_records` (deserialized as `DeviceRecord`) when non-empty; fall back to `devices_ordered` when empty
- [ ] Write unit test 4 (FAIL first): Armis new_with_seed with org_slug

- [ ] Read Claroty `state.rs`, `clone.rs` (and generator if present) before editing
- [ ] Add `generated_records: Vec<serde_json::Value>` to `ClarotyState`
- [ ] Add `ClarotyClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> Self` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg signature; infallible — returns `Self`): forwards `archetype` parameter to Claroty generator per its actual signature (implementer reads generator.rs to confirm — Claroty/Cyberint use org_id BY REF `&OrgId`, opts BY REF `&GenOpts`, org_slug derived internally); stores records in `generated_records`. DO NOT hardcode `Archetype::CompromisedEndpoint` here.
- [ ] Modify Claroty route handlers: dual-path (generated vs static JSON fallback)
- [ ] Write unit test 11 (FAIL first): Claroty new_with_seed forwarded

- [ ] Read Cyberint `state.rs`, `clone.rs` before editing
- [ ] Add `generated_records: Vec<serde_json::Value>` to `CyberintState`
- [ ] Add `CyberintClone::new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> anyhow::Result<Self>` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg signature; fallible — mirrors existing `new() -> anyhow::Result<Self>`): forwards `archetype` parameter to Cyberint generator (Claroty/Cyberint use org_id BY REF `&OrgId`, opts BY REF `&GenOpts`, org_slug derived internally). DO NOT hardcode `Archetype::CompromisedEndpoint` here.
- [ ] Modify Cyberint route handlers: dual-path
- [ ] Write unit test 12 (FAIL first): Cyberint new_with_seed forwarded fallible

**Phase 4: build_clone_pairs coordination**

- [ ] Read `crates/prism-dtu-demo-server/Cargo.toml` fully before editing; add `uuid`, `prism-core` deps if absent; add `fixture-gen` feature fanning out to `prism-dtu-{crowdstrike,armis,claroty,cyberint}/fixture-gen` + `prism-dtu-common/fixture-gen` (without this feature the `new_with_seed` constructors are not visible to harness.rs at compile time)
- [ ] Read `crates/prism-dtu-demo-server/src/harness.rs` fully before editing
- [ ] Map `fixture_set` string → `Archetype` variant for each clone config using the INV-FIXTURE-SET-ARCHETYPE-MAP-001 8-entry table; unknown fixture_set returns E-DEMO-001 before any constructor is called
- [ ] Add E-DEMO-004 guard: if any clone's mapped archetype is non-`HealthyOtEnvironment` AND `org_id` is `None`, return `E-DEMO-004` before any constructor is called (per BC-2.06.018 §Error Codes)
- [ ] Add E-DEMO-005 guard: parse `org_id` string with `uuid::Uuid::parse_str()`; on error, return `E-DEMO-005`
- [ ] Forward `(seed, mapped_archetype, org_id)` to `new_with_seed` for each generator-backed clone — the canonical ADR-036 v2.2 3-arg form; NO hardcoded Archetype variants in harness.rs; Armis derives org_slug internally inside its new_with_seed
- [ ] For static-file clones (ThreatIntel, NVD): continue using existing `new()` / `new_with_access_token()` path (static-file clones are not generator-backed; enrichment injection is Story B scope)
- [ ] Write unit tests 5, 6, 7, 8, 9, 10 (FAIL first): disjoint IDs, backward compat, fixture_set mapping, E-DEMO-001, E-DEMO-004, E-DEMO-005
- [ ] Write differential route-output tests 15, 16, 17 (FAIL first) per ADR-036 v2.2 archetype-forwarding requirement and TD-VSDD-059 paper-test upgrade (F-P6-HIGH-001):
  - Test 15 (`test_BC_2_06_018_dormant_archetype_empty_served_response`): construct a clone (Armis or CrowdStrike) with `fixture_set = "dormant"`, invoke the served route (e.g., `/api/v1/devices` or `/devices/entities/devices/v2`), assert zero records in the response body (proves `Archetype::DormantTenant` reached `generate()` and produced empty output per BC-2.06.018 EC-018-003 / TV-018-006)
  - Test 16 (`test_BC_2_06_018_large_scale_archetype_record_count`): construct a clone with `fixture_set = "large_scale"`, invoke the served route, assert the record count matches the `Archetype::LargeScale` generator baseline — implementer MUST read `crates/prism-dtu-{clone}/src/generator.rs` `LargeScale` branch to obtain the exact expected count; do NOT assume 10 000 (BC-2.06.018 EC-018-005)
  - Test 17 (`test_BC_2_06_018_archetype_drives_served_output_differential`): construct two clones with identical `seed` + `org_id` but `fixture_set = "compromised"` vs `fixture_set = "dormant"`; assert their served route responses differ (proves archetype, not a hardcoded default, drives output)

**Phase 5: Cargo.toml and ci.yml**

- [ ] Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to `crates/prism-dtu-threatintel/Cargo.toml` (prism-dtu-demo-server feature gate will activate this)
- [ ] Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` to `crates/prism-dtu-nvd/Cargo.toml`
- [ ] Run compile-fail gate: `cargo test -p tests-external-perimeter-violation` — must pass with zero new violations
- [ ] Read `tests/external/non-exhaustive-violation/Cargo.toml` before editing; add `prism-dtu-common = { path = "../../../crates/prism-dtu-common", features = ["fixture-gen"] }` under `[dependencies]` (the crate currently imports no prism-dtu-* types; this dep is required before the ScenarioEntityCatalog import row can compile and the violation count can be measured)
- [ ] Run non-exhaustive gate; count new `#[non_exhaustive]` pub types (expected: at minimum `ScenarioEntityCatalog` = 1; implementer reads actual output); update `EXPECTED=N` in `ci.yml` by the exact count; add violation row(s) to `tests/external/non-exhaustive-violation/` for each new type
- [ ] Write compile-fail tests 13-14

**Phase 6: Final gate**

- [ ] Run `just check` — all 17 Red Gate tests pass; no clippy warnings; fmt clean
- [ ] Confirm backward compat regression (test 6) passes
- [ ] Confirm perimeter compile-fail gate (test 13) passes with zero new violations
- [ ] Run SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify any new `tracing::*!(event_type=...)` emissions have BC-2.16.002 Structured Event Catalog rows
- [ ] Confirm ci.yml EXPECTED bump matches actual non-exhaustive-violation count

---

## Previous Story Intelligence

This is Story A of the E-DEMO live-scenario split. No direct predecessor in the same epic.

**Critical substrate facts (ADR-036 v2.0 §1.3 — do NOT skip):**

- `CrowdstrikeClone::new()` creates a `CrowdstrikeState` with empty stores (containment_store, detection_status_store, session_registry). The generator in `generator.rs` is **never called** in the serving path. This is a stateful write-target clone. Story A adds a new serving path that uses generated records when `generated_devices` / `generated_detections` are non-empty.
- `ArmisClone::new()` loads `fixtures/devices.json`, `fixtures/device-activity.json`, and `fixtures/alerts.json` into immutable `Vec<DeviceRecord>` / `Vec<AlertRecord>`. Generator not in serving path. Story A adds `generated_records: Vec<serde_json::Value>` and a new constructor that calls the generator.
- `CloneConfig.seed` is declared in `config.rs` (default `42`) but is **never read** in `build_clone_pairs()`. This is the primary gap this story closes.
- `DemoConfig`/`CloneConfig` have no `org_id` field today. This story adds it.
- `gen_seeded_rng(seed: u64, org_id: &OrgId)` is the 2-arg XOR-formula function re-exported from `prism-dtu-common::rng` (Cyberint already imports it). Takes `&OrgId` — NOT `&str`. The bare 1-arg `seeded_rng` is a legacy helper; use `gen_seeded_rng` exclusively in this story.
- CrowdStrike generator's `org_slug(org_id: &OrgId) -> String` produces the same formula as `org_slug_from_org_id`. They must agree: `hex(org_id.as_bytes()[0..4])`.
- Armis generator takes `org_slug: &str` as an explicit argument. The catalog derives it and passes it.

**From sibling stories in Wave 5:**
- `#[non_exhaustive]` EXPECTED count: read ci.yml before editing to get the live value. Do not assume it is still 49 (it may have changed since ADR-036 was authored). Increment from the live value by the exact count of new types.
- `reqwest::Client` in any new integration test must use `.timeout(Duration::from_secs(30))` per CLAUDE.md conventions.
- SAP-1: grep `event_type\s*=` after implementation and add BC-2.16.002 catalog rows for any new `event_type` emissions before committing.
- `build_clone_pairs` returns `anyhow::Result<Vec<ClonePair>>`; use `anyhow::bail!` for E-DEMO-004/005 errors (consistent with E-DEMO-001 pattern).

**Implementer notes — per-clone generate() signature divergence (U-A-08):**
The four generator-backed clones have DIVERGING `generate()` signatures — verify each before calling.
NOTE: `new_with_seed` for ALL four clones now takes `(seed: u64, archetype: Archetype, org_id: OrgId)` per ADR-036 v2.2.
The divergence is in the INTERNAL call from new_with_seed to generate(), not in the constructor parameter list:
- **Armis:** `generate(org_id: OrgId, org_slug: &str, archetype: Archetype, opts: &GenOpts)` — org_id BY VALUE, org_slug explicit `&str` (derived internally in new_with_seed via org_slug_from_org_id before passing to generate), opts BY REF
- **CrowdStrike:** `generate(org_id: OrgId, archetype: Archetype, opts: GenOpts)` — org_id BY VALUE, opts BY VALUE (not ref), org_slug derived internally inside generate()
- **Claroty/Cyberint:** `generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts)` — org_id BY REF, opts BY REF, org_slug derived internally
All four return `FixtureSet { records: Vec<serde_json::Value>, .. }` so `generated_records: Vec<serde_json::Value>` works for all.
For CrowdStrike: verify whether `FixtureSet` records carry a `_record_type` discriminator before splitting `generated_devices` vs `generated_detections` — read `crates/prism-dtu-crowdstrike/src/generator.rs` and the FixtureSet struct before writing the split logic.

**Implementer notes — OrgId byte access (U-A-09):**
Mirror the exact form already used in existing generators. Read `crates/prism-dtu-crowdstrike/src/generator.rs` `org_slug()` method — it likely accesses `org_id.as_uuid().as_bytes()` or a method defined elsewhere in prism-core. Do not invent a byte-access API; copy the canonical form already in use.

**Implementer notes — Story A fixture-gen scope (U-A-10):**
Story A adds the `fixture-gen` feature to `prism-dtu-threatintel` and `prism-dtu-nvd` Cargo.toml, but NO Story-A code consumes `ScenarioEntityCatalog` from those crates (Story B does). Do NOT add a premature `use prism_dtu_common::scenario::ScenarioEntityCatalog` import in threatintel or nvd source files — an unused import is a clippy `-D warnings` error. The feature addition with no `#[cfg(feature = "fixture-gen")]` consumer in the source is inert and compiles cleanly.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `new()` constructors are unchanged — existing static-JSON path is byte-identical to pre-Story-A behavior | BC-2.06.018 PC-4, ADR-036 §2.5 | Regression test (RG-A-006) + adversary |
| `new_with_seed` calls `generate()` ONCE at construction; route handlers do NOT call `generate()` per-request | ADR-036 §3.2 | Adversary probe: no `generate()` call in route handler body |
| `generated_records: Vec<serde_json::Value>` in state — NOT `Arc<Mutex<...>>` wrapping (immutable after construction) | ADR-036 §2.3 | Adversary: no Mutex on generated_records field |
| `ScenarioEntityCatalog` MUST carry `#[non_exhaustive]` as a public type in `prism-dtu-common` | CLAUDE.md #[non_exhaustive] discipline | ci.yml EXPECTED bump + compile-fail gate |
| `org_slug_from_org_id` formula: `hex(org_id.as_bytes()[0..4])` — exactly 8 hex chars; MUST match CrowdStrike generator's internal `org_slug()` function | ADR-036 §2.2 | Test 2 + adversary comparison |
| `INV-PERIMETER-001`: new fixture-gen feature in prism-dtu-threatintel and prism-dtu-nvd must NOT introduce prism-spec-engine/prism-sensors/prism-query | ADR-036 §2.5, INV-PERIMETER-001 | Compile-fail gate `tests/external/perimeter-violation/` |
| E-DEMO-004/005 detected BEFORE any clone constructor is called in `build_clone_pairs` | BC-2.06.018 §Error Codes, INV-CONSTRUCTION-TIME-FAILURE-001 | Test 9, 10 |
| All tracing emission sites with `event_type =` must have BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| `await_holding_lock = "deny"` (ADR-002 §H1): no `.await` inside a Mutex lock guard | ADR-002 | clippy deny list |
| Forbidden pattern: `Arc::new(SomeThing::placeholder())` in production boot path | ADR-022 §C, CLAUDE.md | Adversary |
| **Demo-server slug authority:** For cross-DTU coherence the canonical slug is ADR-036 §2.2 `hex(org_id.as_bytes()[0..4])` (8-hex chars). The `org_slug = "acme-corp"` test vectors in BC-3.4.004 / BC-3.5.001 are standalone-generator / harness illustrations (Armis's injected-slug path) and do NOT govern demo-server seeding. INV-DISTINCT-DATA-001 assertions MUST use the 8-hex form derived from a real org UUID — not `"acme-corp"` or any hardcoded slug string. Tests using `"dev-acme-..."` IDs are incorrect for this story. | ADR-036 §2.2 | Adversary + Test 1 |

---

## Library & Framework Requirements

Versions pinned from `dependency-graph.md` and `rust-toolchain.toml`. Use these exact versions.
Do NOT invent version numbers from training data.

| Crate | Version | Usage |
|-------|---------|-------|
| `axum` | `0.7` | Route handlers in all prism-dtu-* crates |
| `tokio` | `1` (multi-threaded runtime) | Async runtime per ADR-002 / AD-013 |
| `chrono` | project-pinned | Already present in prism-dtu-armis and prism-dtu-crowdstrike; NOT added to prism-dtu-threatintel/nvd for this story (no chrono usage needed in Story A enrichment crates) |
| `serde` / `serde_json` | project-pinned | `CloneConfig` / `ScenarioConfig` TOML deserialization; `generated_records: Vec<serde_json::Value>` |
| `rand_chacha` (`ChaCha20Rng`) | project-pinned | `gen_seeded_rng(seed, &org_id)` secondary RNG stream in `build_scenario_entity_catalog` |
| `anyhow` | project-pinned | Error propagation in `build_clone_pairs` for E-DEMO-004 / E-DEMO-005 |
| `uuid` | project-pinned | `uuid::Uuid::parse_str()` for parsing `org_id` string to UUID bytes → `OrgId`; **prism-dtu-demo-server Cargo.toml must declare `uuid` as a dependency** (add under `[dependencies]` if not already present) |
| `prism-core` | workspace path | `prism_core::auth::OrgId` type; **prism-dtu-demo-server Cargo.toml must declare `prism-core` as a dependency** (add under `[dependencies]` if not already present) |
| `reqwest` | project-pinned | Integration test HTTP client; `.timeout(Duration::from_secs(30))` mandatory per CLAUDE.md |

**Forbidden versions / patterns:**
- Do NOT use `tokio::time::interval` or `tokio::spawn` for any scenario logic — pure construction model only
- Do NOT introduce `once_cell::sync::Lazy<Mutex<...>>` for generated_records — immutable after construction
- Do NOT call `generate()` in route handler bodies (generation is construction-time only)

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-common/src/scenario/mod.rs` | CREATE | `ScenarioEntityCatalog`, `org_slug_from_org_id`, `build_scenario_entity_catalog`; gated `#[cfg(feature = "fixture-gen")]` |
| `crates/prism-dtu-common/src/lib.rs` | MODIFY | Add `pub mod scenario;` under `#[cfg(feature = "fixture-gen")]` |
| `crates/prism-dtu-demo-server/Cargo.toml` | MODIFY | (1) Add `uuid = { workspace = true }` under `[dependencies]` if absent; (2) Add `prism-core = { path = "../../crates/prism-core" }` (for `OrgId`) if absent; (3) Add `fixture-gen = ["prism-dtu-crowdstrike/fixture-gen", "prism-dtu-armis/fixture-gen", "prism-dtu-claroty/fixture-gen", "prism-dtu-cyberint/fixture-gen", "prism-dtu-common/fixture-gen"]` feature so `build_clone_pairs` can activate `new_with_seed` on all 4 generator-backed clones |
| `crates/prism-dtu-demo-server/src/config.rs` | MODIFY | Add `org_id: Option<String>` + `scenario: Option<ScenarioConfig>` to `CloneConfig`; add `ScenarioConfig` struct |
| `crates/prism-dtu-demo-server/src/harness.rs` | MODIFY | `build_clone_pairs`: map `fixture_set→Archetype` via INV-FIXTURE-SET-ARCHETYPE-MAP-001 table; E-DEMO-004/005 guards; derive OrgId from org_id string; forward `(seed, mapped_archetype, org_id)` to `new_with_seed` for all 4 generator-backed clones — NO hardcoded Archetype variants in harness.rs (ADR-036 v2.2) |
| `crates/prism-dtu-crowdstrike/src/state.rs` | MODIFY | Add `generated_devices: Vec<serde_json::Value>`, `generated_detections: Vec<serde_json::Value>` |
| `crates/prism-dtu-crowdstrike/src/clone.rs` | MODIFY | Add `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> Self` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg, infallible) |
| `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` | MODIFY | Dual-path: serve from `generated_devices` when non-empty; fall back to `load_host_ids()` / `load_host_details()` (static embedded JSON — the READ path) when empty. Do NOT fall back to `containment_store` / `detection_status_store` (write-target overlays, not READ sources) |
| `crates/prism-dtu-crowdstrike/src/routes/detections.rs` | MODIFY | Dual-path: serve from `generated_detections` when non-empty |
| `crates/prism-dtu-armis/src/state.rs` | MODIFY | Add `generated_records: Vec<serde_json::Value>` |
| `crates/prism-dtu-armis/src/clone.rs` | MODIFY | Add `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> anyhow::Result<Self>` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg, fallible — mirrors clone.rs:58; org_slug derived INTERNALLY via org_slug_from_org_id(&org_id)) |
| `crates/prism-dtu-armis/src/routes/devices.rs` | MODIFY | `paginate_devices`: dual-path — generated_records when non-empty, `devices_ordered` fallback |
| `crates/prism-dtu-claroty/src/state.rs` | MODIFY | Add `generated_records: Vec<serde_json::Value>` |
| `crates/prism-dtu-claroty/src/clone.rs` | MODIFY | Add `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> Self` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg, infallible; org_slug derived internally) |
| `crates/prism-dtu-claroty/src/routes/` | MODIFY | Claroty route handler(s): dual-path (implementer reads current routes to identify which handler) |
| `crates/prism-dtu-cyberint/src/state.rs` | MODIFY | Add `generated_records: Vec<serde_json::Value>` |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Add `new_with_seed(seed: u64, archetype: Archetype, org_id: OrgId) -> anyhow::Result<Self>` under `#[cfg(feature = "fixture-gen")]` (ADR-036 v2.2 canonical 3-arg, fallible; org_slug derived internally) |
| `crates/prism-dtu-cyberint/src/routes/` | MODIFY | Cyberint route handler(s): dual-path |
| `crates/prism-dtu-threatintel/Cargo.toml` | MODIFY | Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature (enables `ScenarioEntityCatalog` usage in Story B) |
| `crates/prism-dtu-nvd/Cargo.toml` | MODIFY | Add `fixture-gen = ["prism-dtu-common/fixture-gen"]` feature (enables `ScenarioEntityCatalog` usage in Story B) |
| `.github/workflows/ci.yml` | MODIFY | Bump `EXPECTED=N` by count of new `#[non_exhaustive]` pub types (at minimum +1: `ScenarioEntityCatalog`; exact count verified by implementer) |
| `tests/external/non-exhaustive-violation/Cargo.toml` | MODIFY | Add `prism-dtu-common = { path = "../../../crates/prism-dtu-common", features = ["fixture-gen"] }` under `[dependencies]` (required before ScenarioEntityCatalog import row can resolve; the crate currently declares no prism-dtu-* deps) |
| `tests/external/non-exhaustive-violation/src/` | MODIFY | Add import/violation row(s) for new `#[non_exhaustive]` types in `prism-dtu-common::scenario` (at minimum `ScenarioEntityCatalog`) |

**Forbidden new files:**
- Do NOT create `crates/prism-dtu-scenario/` — scenario types belong in `prism-dtu-common/src/scenario/` per ADR-036 §3.4
- Do NOT create a separate `IncidentTimeline` or `StageMask` type in this story — those are Story B scope

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.06.018 EC-018-001 | `seed = 42` (default) + `fixture_set = "default"` for all clones | Backward-compatible: data identical to pre-seeding `new()` behavior; all existing integration tests pass (AC-006) |
| EC-002 | BC-2.06.018 EC-018-002 | Two clone configs with `seed_A = 100`, `seed_B = 200`, same `fixture_set = "default"` | INV-DISTINCT-DATA-001 holds: response ID sets pairwise-disjoint (AC-005) |
| EC-003 | BC-2.06.018 EC-018-003 | `fixture_set = "dormant"` for a generator-backed clone | `Archetype::DormantTenant` constructed; clone returns empty device/alert responses; no construction-time error |
| EC-004 | BC-2.06.018 EC-018-004 | `fixture_set = "xyzzy_unknown"` for any clone | Construction-time E-DEMO-001; `build_clone_pairs` returns `Err`; harness aborts (AC-008) |
| EC-005 | BC-2.06.018 EC-018-005 | `fixture_set = "large_scale"` for CrowdStrike clone | `Archetype::LargeScale` constructed; 10 000 device records generated at startup; no panic; startup time reasonable |
| EC-006 | BC-2.06.018 EC-018-008 | `seed = u64::MAX` for a generator-backed clone | Valid; `gen_seeded_rng(u64::MAX, &org_id)` (primary) and `gen_seeded_rng(0, &org_id)` (secondary after `u64::MAX.wrapping_add(1)`) both valid; no panic |
| EC-007 | BC-2.06.018 EC-018-009 | Process restart with same `demo.toml` | Byte-identical responses per BC-3.4.001 postcondition 6; determinism holds across restarts |
| EC-008 | ADR-036 §1.3 | `org_id` present as valid UUID but generator produces no records | `generated_records` is `vec![]`; route handler falls back to static-JSON path; no panic; error only if the generator itself returns Err |
| EC-009 | ADR-036 §2.3 | `new()` called (not `new_with_seed`) when `org_id = None` in config | Static-JSON path taken; backward-compatible; no error (E-DEMO-004 fires only when `new_with_seed` path is attempted) |
| EC-010 | INV-PERIMETER-001 | `prism-dtu-threatintel` or `prism-dtu-nvd` gains `fixture-gen` feature | Transitive `prism-dtu-common/fixture-gen` allowed; NO `prism-spec-engine`/`prism-sensors`/`prism-query` introduced; compile-fail gate passes (AC-013) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `ScenarioEntityCatalog` | `prism-dtu-common/src/scenario/` | Pure (data struct, no I/O; constructed from seed+org_id) | ADR-036 §2.2 |
| `org_slug_from_org_id` | `prism-dtu-common/src/scenario/` | Pure (deterministic formula: hex of first 4 bytes) | ADR-036 §2.2 |
| `build_scenario_entity_catalog` | `prism-dtu-common/src/scenario/` | Pure (deterministic from seed+org_id via secondary RNG) | ADR-036 §2.2 |
| `ScenarioConfig` | `prism-dtu-demo-server/src/config.rs` | Pure (TOML deserialization struct) | ADR-036 §2.4 |
| `build_clone_pairs` (seed-forwarding additions) | `prism-dtu-demo-server/src/harness.rs` | Effectful (constructs clones, reads config, calls generate) | BC-2.06.018 PC-1 |
| `CrowdstrikeClone::new_with_seed` | `prism-dtu-crowdstrike/src/clone.rs` | Effectful (calls generate, stores records in state) | ADR-036 §2.3 |
| `ArmisClone::new_with_seed` | `prism-dtu-armis/src/clone.rs` | Effectful (fallible — returns `anyhow::Result<Self>`; mirrors `ArmisClone::new()` at clone.rs:58; derives org_slug internally via org_slug_from_org_id; forwards archetype to generate; stores records in state) | ADR-036 v2.2 §2.3 |
| `ClarotyClone::new_with_seed` | `prism-dtu-claroty/src/clone.rs` | Effectful (calls generate, stores records in state) | ADR-036 §2.3 |
| `CyberintClone::new_with_seed` | `prism-dtu-cyberint/src/clone.rs` | Effectful (fallible; calls generate, stores records in state) | ADR-036 §2.3 |
| Route handler dual-path logic (4 clones) | `prism-dtu-{armis,crowdstrike,claroty,cyberint}/src/routes/` | Effectful (HTTP handler checks non-empty generated_records) | ADR-036 §2.3 |

---

## Forbidden Dependencies

These dependencies MUST NOT appear in the `[dependencies]` section of the following crates.
If they appear, the build MUST fail (enforced by `tests/external/perimeter-violation/`).

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
| `prism-dtu-claroty` | `prism-spec-engine` | INV-PERIMETER-001 |
| `prism-dtu-cyberint` | `prism-spec-engine` | INV-PERIMETER-001 |
| Any new crate | `prism-dtu-scenario` (does not exist) | ADR-036 §3.4 — no separate crate; scenario types live in `prism-dtu-common` |

The only permitted new cross-DTU dependency addition in this story is:
`prism-dtu-threatintel` → `prism-dtu-common` (feature = "fixture-gen") and
`prism-dtu-nvd` → `prism-dtu-common` (feature = "fixture-gen"), if those
`prism-dtu-common` deps do not already exist in those crates' Cargo.toml files.

---

## SAP-1 Compliance (Structured Event Catalog)

Per CLAUDE.md §SAP-1, any `tracing::*!(event_type = "...")` emission site added in this
story requires a corresponding row in BC-2.16.002 Structured Event Catalog with:
- event_type value, emitting module, field schema, audit role, recurrence policy

If the seed-forwarding logic in `build_clone_pairs` adds `event_type` emissions (e.g.,
`event_type = "demo.seeding_initialized"`), those catalog rows MUST be in the same commit.
Removal of an emission does NOT require a new catalog row per D-765 precedent.

If NO new `event_type` emissions are added, state so explicitly in the PR description.

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.6 | 2026-08-02 | Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 (D-2084): added §Authority section. |
| v1.5 | 2026-06-09 | OBS-P7-001 closure (LOW): §Token Budget ADR-036 full-read pin corrected v2.0→v2.2 (POL-23/POL-29 cross-document version-pin sync). The `(full)` row in the Token Budget table cited the superseded ADR-036 v2.0; canonical ADR-036 is now v2.2. §Narrative line (~84) and §Previous Story Intelligence line (~408) citations of "ADR-036 v2.0 §1.3" are intentionally preserved — §1.3 is self-titled "Substrate Reality (v2.0 Correction)" and those citations are correct provenance anchors, NOT stale pins. Only the Token Budget "(full)" row was the defect. Sibling sweep confirmed: all other `ADR-036 v2.0` occurrences are (a) provenance citations to the v2.0-named subsection §1.3 or (b) immutable historical changelog rows — none changed. |
| v1.4 | 2026-06-09 | F-P6-HIGH-001 closure + USER DECISION (full 8-archetype support): Reconciled story to ADR-036 v2.2 canonical 3-arg archetype-driven constructor. All four clone new_with_seed signatures updated to `(seed, archetype, org_id)` — CrowdStrike/Claroty return `Self`; Armis/Cyberint return `anyhow::Result<Self>`. AC-003/AC-011: archetype arg added, CompromisedEndpoint now forwarded-not-hardcoded. AC-004: removed explicit org_slug parameter (now derived internally via org_slug_from_org_id inside new_with_seed; test renamed to `test_BC_2_06_018_armis_new_with_seed_canonical_3arg`). AC-012: archetype arg added. AC-007: strengthened from is_ok()-only to archetype-drives-output differential assertions (TD-VSDD-059 paper-test upgrade). Three new differential Red Gate tests added: #15 dormant→empty served response (TV-018-006), #16 large_scale record count, #17 compromised-vs-dormant differential output. `red_gate_tests` 14→17. Tasks Phase 3: all CompromisedEndpoint hardcodes replaced with forwarded archetype param; Armis internal org_slug derivation documented. Tasks Phase 4: fixture_set→Archetype mapping step added before constructor dispatch. File Structure: all four clone.rs rows updated to canonical 3-arg; harness.rs row updated. BC pin bumped v1.4→v1.5. Token Budget test count 14→17. traces_to: F-P6-HIGH-001 added. risk_mitigations: archetype-forwarding and Armis-internal-slug invariants added. |
| v1.3 | 2026-06-09 | F-P5-HIGH-001: BC version pins synced v1.3→v1.4. §Behavioral Contracts table row pin and §Token Budget row pin both updated (BC-2.06.018 v1.3→v1.4). Phantom-anchor fix in BC-2.06.018: 7 story-ref sites corrected S-DEMO-DTU-DATA-SEEDING-001 → S-DEMO-DTU-LIVE-SCENARIO-001-A by product-owner. Sweep confirmed no other live-narrative BC-2.06.018 v1.x pins remain. Historical changelog rows v1.0, v1.1, and v1.2 are immutable audit trail per TD-VSDD-091 — not altered. POL-23/POL-29 sibling sweep complete. |
| v1.2 | 2026-06-09 | F-P4-MED-002: BC version pins synced v1.1→v1.3. §Behavioral Contracts table row pin and §Token Budget row pin both updated (BC-2.06.018 v1.1→v1.3). Sweep confirmed no other live-narrative v1.x pins remain. Historical changelog rows v1.0 and v1.1 are immutable audit trail per TD-VSDD-091 — not altered. POL-23/POL-29 sibling sweep complete. |
| v1.1 | 2026-06-09 | U-A-01: Replace `seeded_rng` with `gen_seeded_rng(seed.wrapping_add(1), &org_id)` (2-arg XOR-formula re-export) in AC-001, Tasks Phase 1, risk_mitigations, Library table. U-A-02: Add `crates/prism-dtu-demo-server/Cargo.toml` to File Structure (uuid+prism-core deps + fixture-gen feature) and Tasks Phase 4 (read-before-edit step). Add `prism-core` Library row. U-A-03: ArmisClone::new_with_seed returns `anyhow::Result<Self>` (mirrors clone.rs:58 fallibility) — updated AC-004, AC-006, File Structure (armis/clone.rs), Architecture Mapping, risk_mitigations. U-A-04: CrowdStrike READ fallback corrected from `containment_store` to `load_host_ids()`/`load_host_details()` in Tasks Phase 3 and File Structure. U-A-05: tests/external/non-exhaustive-violation/Cargo.toml dep addition (prism-dtu-common fixture-gen) added to AC-014, Tasks Phase 5, File Structure. U-A-06: Architecture Compliance rule added clarifying ADR-036 §2.2 8-hex slug authority vs BC-3.4.004/BC-3.5.001 standalone-generator test vectors. U-A-07: `GenOpts { seed, .. }` → `GenOpts { seed, ..GenOpts::default() }` in Tasks Phase 3 (both CrowdStrike and Armis). U-A-08/09/10: Per-clone generate() signature divergence, OrgId byte-access, and Story A fixture-gen scope (no premature unused import) added to Previous Story Intelligence. |
| v1.0 | 2026-06-09 | Initial authoring per ADR-036 v2.0 §8 story split (D-1077). Supersedes S-DEMO-DTU-LIVE-SCENARIO-001 for the BC-2.06.018 scope. |
