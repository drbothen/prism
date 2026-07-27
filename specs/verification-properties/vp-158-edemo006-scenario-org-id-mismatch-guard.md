---
document_type: verification-property
level: L4
version: "1.0"
status: draft
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
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-06-12"
modified: null
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

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 3 story S-DEMO-DTU-LIVE-SCENARIO-001-B TDD delivery]
// Method: unit_test
//
// SYMBOL RESOLUTION — test-writer must verify grounding before authoring tests
//
// TARGET FUNCTION: `build_clone_pairs` in prism-dtu-demo-server::harness
//   Confirmed real: BC-2.06.019 §Traceability cites
//   `crates/prism-dtu-demo-server/src/harness.rs` as the site of this function.
//   Input type: `ClientConfig` (or equivalent config struct with `[[clones]]` entries
//   including `seed`, `org_id`, and `scenario.enabled` fields per BC-2.06.019 §Description).
//   Output type: `Result<Vec<ClonePair>, DemoServerError>` (or equivalent).
//
// HARNESS DEPENDENCIES:
//   - Config construction helper for `ClientConfig` with multiple clone entries
//   - `CloneConfig` struct must support `seed: u64`, `org_id: String`,
//     `scenario: Option<ScenarioConfig>` with `enabled: bool`
//
// TEST 1 — VP-158: E-DEMO-006 fires for same seed, different org_ids (BC-2.06.019 TV-019-015)
//
// #[test]  // or #[tokio::test] if build_clone_pairs is async
// fn edemo006_fires_for_same_seed_different_org_ids() {
//     // TV-019-015: seed=100 for both, org_id-A != org_id-B, both scenario.enabled=true
//     let config = ClientConfig::test_with_two_scenario_clones(
//         CloneEntry { name: "crowdstrike", seed: 100, org_id: "uuid-A", scenario_enabled: true },
//         CloneEntry { name: "armis",       seed: 100, org_id: "uuid-B", scenario_enabled: true },
//     );
//
//     let result = build_clone_pairs(&config);
//
//     // Assert: Err returned before any clone is constructed
//     assert!(result.is_err(),
//         "same seed + different org_ids must return Err (VP-158 / BC-2.06.019 PRE-6)");
//
//     // Assert: error identifies E-DEMO-006
//     let err_msg = result.unwrap_err().to_string();
//     assert!(err_msg.contains("E-DEMO-006"),
//         "error must cite E-DEMO-006 (VP-158 / BC-2.06.019 §E-DEMO-006)");
//
//     // Assert: both org_ids appear in the error message
//     assert!(err_msg.contains("uuid-A"),
//         "error message must name first org_id (VP-158)");
//     assert!(err_msg.contains("uuid-B"),
//         "error message must name second org_id (VP-158)");
// }
//
// TEST 2 — Control: same seed AND same org_id does NOT trigger E-DEMO-006
//
// #[test]
// fn edemo006_not_triggered_for_same_seed_same_org_id() {
//     let config = ClientConfig::test_with_two_scenario_clones(
//         CloneEntry { name: "crowdstrike", seed: 100, org_id: "uuid-A", scenario_enabled: true },
//         CloneEntry { name: "armis",       seed: 100, org_id: "uuid-A", scenario_enabled: true },
//     );
//
//     let result = build_clone_pairs(&config);
//
//     // May return Ok or Err for a different reason (E-DEMO-003 archetype check etc.)
//     // but must NOT return E-DEMO-006
//     if let Err(e) = result {
//         assert!(!e.to_string().contains("E-DEMO-006"),
//             "same seed + same org_id must not trigger E-DEMO-006 (VP-158 control)");
//     }
// }
//
// Kill conditions (mutation testing — these mutations MUST be caught):
//   - Remove the org_id equality check from build_clone_pairs
//     → test_1 fails: result is Ok, not Err(E-DEMO-006)
//   - Swap error code to a different E-DEMO-NNN
//     → test_1 fails: error message does not contain "E-DEMO-006"
//   - Suppress the first or second org_id from the error message
//     → test_1 fails on the org_id presence assertions
//   - Check seed mismatch instead of org_id mismatch (confuse E-DEMO-002 with E-DEMO-006)
//     → test_2 may incorrectly fail or test_1 may not trigger correctly
```

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
| 1.0 | FB68c | 2026-07-27 | architect | F-WASE-P65-OBS-001: Initial VP file authoring. VP-INDEX row and metadata existed since 2026-06-12 (BC-2.06.019 v1.2 PO micro-burst adding PRE-6 and VP-019-I alias). No metadata changes — module (prism-dtu-demo-server), method (unit_test), priority (P1), anchor story (S-DEMO-DTU-LIVE-SCENARIO-001-B), and source BC (BC-2.06.019) remain as originally registered. File gap closed. |
