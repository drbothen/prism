---
document_type: verification-property
level: L4
version: "1.1"
status: active
producer: architect
timestamp: 2026-07-27T00:00:00Z
phase: wave-a
inputs:
  - specs/behavioral-contracts/BC-2.06.019-demo-server-scenario-progression.md
  - stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md
input-hash: "pending"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.06.019
source_invariant: null
module: prism-dtu-demo-server
priority: P1
proof_method: unit_test
verification_method: unit_test
feasibility: feasible
verification_lock: false
proof_completed_date: "2026-07-27"
proof_file_hash: null
lifecycle_status: active
introduced: "2026-06-12"
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-158: E-DEMO-006 — Scenario Org-ID Mismatch Guard Fires Before Clone Construction

**VP-INDEX alias: VP-019-I**

## Property Statement

For `prism-dtu-demo-server`, when `build_clone_pairs` encounters two or more
scenario-enabled clones within the same client config block that share the same `seed`
value but have **different `org_id` values**, `build_clone_pairs` MUST return
`Err(E-DEMO-006)` before constructing any clone.

Specifically:
1. **Early exit before construction:** No clone instance is created when E-DEMO-006 fires.
   The function returns `Err(...)` immediately upon detecting the `org_id` mismatch among
   scenario-enabled clones with the same seed.
2. **Error code identification:** The returned error contains the `E-DEMO-006` code
   (BC-2.06.019 §E-DEMO-006 error table).
3. **Error message content:** The error message names both clone identifiers and both
   mismatched `org_id` values, matching the format prescribed by BC-2.06.019 §E-DEMO-006
   message format: `"demo-server: E-DEMO-006: scenario clones '{clone_a}' (org_id={org_id_a})
   and '{clone_b}' (org_id={org_id_b}) have different org_ids; cross-DTU coherence requires
   all scenario-enabled clones to share the same org_id"`.

**Why this matters — INV-CROSS-DTU-ENTITY-COHERENCE-001:** Without this guard,
`build_clone_pairs` would silently proceed using the first scenario-enabled clone's
`(seed, org_id)` pair to derive the `ScenarioEntityCatalog`. A second clone with a
different `org_id` would then generate entity IDs keyed to its own slug, producing no
overlap with the catalog — cross-DTU joins in scenario progression would return empty
result sets, silently invalidating scenario playback without any observable error.
BC-2.06.019 PRE-6 classifies this as a SOUL.md §4 silent-failure class; the guard makes
the misconfiguration immediately visible.

**Relationship to E-DEMO-002:** BC-2.06.019 defines the guard order as
`E-DEMO-002 (seed mismatch) → E-DEMO-006 (org_id mismatch) → E-DEMO-003 (bad archetype)
→ E-DEMO-004 (missing org_id)`. VP-158 covers the E-DEMO-006 arm only; E-DEMO-002 is a
separate, independent guard that fires for a DIFFERENT input condition (two clones with
different seeds, regardless of org_id). The canonical test vector for VP-158 is
BC-2.06.019 TV-019-015: same seed (`100`), different org_ids (uuid-A and uuid-B), both
`scenario.enabled = true`.

## Source Contract

- **Anchor Story:** `S-DEMO-DTU-LIVE-SCENARIO-001-B`
  — Anchor justification (POL-5): `S-DEMO-DTU-LIVE-SCENARIO-001-B` is the delivery story
  for BC-2.06.019 PRE-6 (org_id equality guard) and VP-019-I. The guard (`E-DEMO-006`)
  was added to BC-2.06.019 at v1.2 specifically for this story's scope.
- **Source BC:** BC-2.06.019 v0.8 — demo-server scenario progression; PRE-6 (org_id
  equality guard), EC-019-013 (org_id mismatch edge case), TV-019-015 (canonical
  E-DEMO-006 test vector), §E-DEMO-006 (error code table with message format).
- **Alias:** VP-019-I (BC-2.06.019 §Verification Properties alias, named at BC v1.2)
- **Module:** prism-dtu-demo-server (specifically `build_clone_pairs` in `harness.rs`)
- **Category:** Error handling / Invariant enforcement / Demo server integrity

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| unit_test | std::test or tokio::test (synchronous config parsing) | Yes — canonical TV-019-015 scenario plus one no-mismatch control case | E-DEMO-006 fires for (same seed, different org_ids); no clone constructed; error code and both org_ids present in message; control (same org_ids) does not fire E-DEMO-006 |

**Why unit_test:** `build_clone_pairs` performs deterministic config-time validation
before any network or async operation. The E-DEMO-006 guard fires purely from config-field
inspection; no clones are started and no I/O occurs. A synchronous or minimal async test
can call `build_clone_pairs` directly with a crafted `ClientConfig` struct matching
TV-019-015, making this a pure unit test.

## Proof Evidence

**Status: PROVEN.** The E-DEMO-006 guard is implemented in `build_clone_pairs` in
`crates/prism-dtu-demo-server/src/harness.rs`. Tests are in
`crates/prism-dtu-demo-server/tests/bc_2_06_019_scenario_progression.rs`.

**Note on VP alias:** The test file cites this property as `VP-019-I` (the BC-2.06.019
alias registered at BC v1.2). Zero occurrences of the string `VP-158` appear in crates/;
this was the F-WASE-P66-LOW-003 finding. The alias and the VP-INDEX entry are the same
property; `VP-019-I` in tests corresponds to `VP-158` in the VP-INDEX.

### Proven Tests

| Test Function | Scenario | Input Config | Asserts |
|---|---|---|---|
| `test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones` | TV-019-015: seed=100 for both clones; CrowdStrike org_id=DEMO_ORG_UUID_DEADBEEF, Armis org_id=DEMO_ORG_UUID_CAFEBABE | `make_cs_armis_same_seed_different_org(100, DEADBEEF, CAFEBABE)` | `build_clone_pairs` returns `Err`; error string contains `"E-DEMO-006"`; error names both org_ids; guard fires before any clone is constructed (BC-2.06.019 PRE-6 / EC-019-013) |
| `test_BC_2_06_019_e_demo_006_case_variant_org_ids_succeed` | UUID byte-identity control: same UUID in different case (lower/upper); byte-identical after parse | `make_cs_armis_same_seed_different_org(100, DEADBEEF_LOWER, DEADBEEF_UPPER)` | `build_clone_pairs` does NOT return `Err("E-DEMO-006")`; byte-based comparison avoids false positive |

### Test Infrastructure (verified real symbols)

- **`build_clone_pairs`** — the target function in `crates/prism-dtu-demo-server/src/harness.rs`.
  Takes a config struct, validates scenario-clone invariants (E-DEMO-002 → E-DEMO-006 →
  E-DEMO-003 → E-DEMO-004 guard order), and returns `Err` containing the E-DEMO-NNN code
  on first violation.
- **`make_cs_armis_same_seed_different_org`** — test helper in `bc_2_06_019_scenario_progression.rs`
  that constructs a two-clone config (CrowdStrike + Armis) with the specified seed and
  per-clone org_ids.
- **`DEMO_ORG_UUID_DEADBEEF` / `DEMO_ORG_UUID_CAFEBABE`** — UUID string constants used
  as distinct org_id values for TV-019-015.
- **`DEMO_ORG_UUID_DEADBEEF_UPPER`** — uppercase variant of `DEADBEEF` UUID for the
  byte-identity control case.

### Kill Conditions (mutation targets, BC-2.06.019 PRE-6)

- Remove org_id equality check from `build_clone_pairs` → `test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones` fails: `is_err()` returns false
- Change error code from `E-DEMO-006` to another code → error string assertion fails
- Suppress first or second org_id from error message → org_id presence assertions fail
- Use raw string comparison instead of byte-based UUID comparison → `test_BC_2_06_019_e_demo_006_case_variant_org_ids_succeed` fails: `E-DEMO-006` false positive for case-variant UUIDs

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Canonical TV-019-015 is a single (seed=100, org_id-A, org_id-B) tuple; control case is one additional config |
| Tool support? | Full | Standard Rust test; `build_clone_pairs` takes a config struct; no network, no async required |
| Execution time budget | < 100ms | Pure config validation — no clone startup, no I/O |
| Assumptions required | One | `ClientConfig` and `CloneEntry` structs must support test construction (confirmed via BC-2.06.019 §Traceability `harness.rs` + `config.rs`) |
| Guard order dependency | Noted | E-DEMO-002 (seed mismatch) fires before E-DEMO-006 per BC-2.06.019 PRE-6. VP-158 test inputs have matching seeds, so E-DEMO-002 guard is bypassed. Test-writer must ensure the config uses `seed: 100` for both clones (same seed, different org_id) to isolate the E-DEMO-006 arm. |
| Platform constraint | None | Pure Rust deterministic test; all platforms |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| BC-2.06.019 PRE-6 and VP-019-I established | 2026-06-12 | product-owner (BC-2.06.019 v1.2 PO micro-burst) |
| registered in VP-INDEX as VP-158 | 2026-06-12 | state-manager |
| file authored | 2026-07-27 | architect (F-WASE-P65-OBS-001 — VP-INDEX row existed since 2026-06-12 but no VP file was ever created) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | FB70 | 2026-07-27 | architect | F-WASE-P66-LOW-003 (VP file leg): Promoted `draft` → `active`. E-DEMO-006 guard confirmed implemented in `build_clone_pairs` (`harness.rs`). Phantom proof harness skeleton removed (symbols `ClientConfig::test_with_two_scenario_clones` and `CloneEntry` do not exist in crates/). Replaced with real proof evidence citing `test_BC_2_06_019_e_demo_006_org_id_mismatch_across_scenario_clones` and `test_BC_2_06_019_e_demo_006_case_variant_org_ids_succeed` from `bc_2_06_019_scenario_progression.rs`; real helpers `make_cs_armis_same_seed_different_org` and `DEMO_ORG_UUID_*` constants documented. VP-INDEX citation note added: crates/ cites this property as alias `VP-019-I`, not `VP-158` — zero occurrences of `VP-158` in crates/ is the finding; the alias is the same property. `proof_completed_date` set to 2026-07-27. |
| 1.0 | FB68c | 2026-07-27 | architect | F-WASE-P65-OBS-001: Initial VP file authoring. VP-INDEX row and metadata existed since 2026-06-12 (BC-2.06.019 v1.2 PO micro-burst adding PRE-6 and VP-019-I alias). No metadata changes — module (prism-dtu-demo-server), method (unit_test), priority (P1), anchor story (S-DEMO-DTU-LIVE-SCENARIO-001-B), and source BC (BC-2.06.019) remain as originally registered. File gap closed. |
