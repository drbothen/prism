---
document_type: verification-property
level: L4
version: "0.21"
status: active
producer: architect
timestamp: 2026-05-15T00:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md
  - .factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md
input-hash: "[pending-recompute]"
traces_to: .factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md
source_bc: BC-2.16.012
source_adr: ADR-026
source_invariant: null
module: prism-query
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: "2026-05-18"
proof_file_hash: null
lifecycle_status: active
introduced: "2026-05-15"
modified: "2026-05-18"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-156: WriteToolInvalidationMap — Registration Uniqueness

## Property Statement

`register_write_tool` in `crates/prism-query/src/invalidation.rs` (ADR-026 D7 v1.23,
INV-INVALIDATION-EXT-001, TD-S-PLUGIN-PREREQ-A-003 closure) MUST enforce the uniqueness
invariant under all sequential registration patterns:

**Uniqueness invariant:** For any sequence of `register_write_tool(entry)` calls,
if two calls carry the same `tool_name`, the second call MUST return
`Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))`.
The first registration persists unchanged in the map; the second is rejected.
No silent last-writer-wins override is permitted.

This is a safety property — violating it produces a silent incorrectness that the
production-grade default forbids (Canonical Principle Rule 1). This VP is the primary
property resolved by F-LP1-MED-002.

**Visibility guarantee (structural, not proptest-verified):** The happens-before guarantee
that a successfully registered tool is always visible to subsequent read-side callers is
structurally provided by `std::sync::RwLock`'s documented `Release`/`Acquire` memory-model
contract combined with ADR-022 boot-step ordering (writes in step 7.5, reads only from step
8+). The single-threaded proptest harness cannot verify cross-thread `Release`/`Acquire`
semantics — and does not need to, because the guarantee derives from the container choice
(D7 rationale) and the boot-step ordering invariant, not from the call sequence covered by
proptest. Concurrent test cases would add CI variance without adding contract coverage.

## Acceptance Criteria

The proptest harness asserts:

- **Case 1 — unique registrations:** For any sequence of N entries with distinct `tool_name`
  values, all N calls to `register_write_tool` return `Ok(())`, and all N entries are
  observable in the map immediately after the final sequential registration
  (read-guard `len() == N`). This verifies uniqueness + sequential visibility within a
  single thread; cross-thread `Release`/`Acquire` visibility is guaranteed by `RwLock`
  contract (see §Property Statement).
- **Case 2 — duplicate name:** Given an initial successful registration of entry A
  (`tool_name = "tool_X"`), a subsequent call with any entry B where
  `B.tool_name == "tool_X"` returns `Err(SpecEngineError::DuplicateWriteToolRegistration("tool_X"))`.
  The map still contains exactly one entry for `"tool_X"` (the original A), not B.
- **Case 3 — mixed sequence:** A proptest-generated sequence of entries (some unique, some
  duplicate-named) produces `Ok(())` for every unique `tool_name` and `Err(DuplicateWriteToolRegistration)`
  for every duplicate. Final map length equals the count of distinct `tool_name` values.

## Source Contract

- **BC:** BC-2.16.012 — EC-016-012-004 (duplicate `register_write_tool` call behavior, resolved
  to error-on-duplicate by ADR-026 D7 v1.23). INV-INVALIDATION-EXT-001 (runtime extensibility
  postcondition). VP-156 provides proptest coverage for the uniqueness semantics that
  BC-2.16.012 §Verification Properties previously described as "(none in this story)".
  This VP closes that coverage gap per F-LP1-MED-003 resolution (option a: author VP-156).
- **ADR:** ADR-026 D7 v1.23 — specifies the error-on-duplicate API contract (`register_write_tool`
  returns `Result<(), SpecEngineError>`); `SpecEngineError::DuplicateWriteToolRegistration(String)`
  variant defined there. VP-156 is anchored in ADR-026 D7 as the proptest verification mechanism.
- **Invariant:** INV-INVALIDATION-EXT-001 — `WriteToolInvalidationMap` is runtime-extensible
  after startup; this VP verifies the registration path is correct and safe.
- **Module:** prism-query (`crates/prism-query/src/invalidation.rs`)
- **Category:** Uniqueness / Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (with `prop::collection::vec`) | Bounded — proptest default cases (256) | Arbitrary sequences of WriteToolInvalidationMap entries; uniqueness violation always returns Err; sequential write visibility verified in single-threaded context |

**Why not Kani:** Kani excels at bounded model-checking over numeric state spaces. The
`register_write_tool` invariant is primarily a `String`-keyed uniqueness property; Kani's
symbolic execution over heap-allocated `String` values is expensive and does not add coverage
that proptest over realistic name strings cannot provide. Proptest with `any::<String>()` covers
the name-collision space adequately. Kani is overkill for a boot-phase Vec-push with a linear
uniqueness check.

**Feasibility:** The `WriteToolInvalidationMap` struct is a pure value type in prism-query.
The `register_write_tool` function is a synchronous `RwLock::write()` + Vec-push operation.
No async, no I/O, no WASM. Proptest can call it directly with generated inputs. The harness
does not require any test infrastructure beyond what is already in prism-query's test suite.

## Proof Harness Skeleton

```rust
// crates/prism-query/tests/vp156_write_tool_registration_uniqueness.rs
//
// VP-156: WriteToolInvalidationMap registration uniqueness
// Method: proptest
// Target: prism_query::invalidation::register_write_tool
// ADR: ADR-026 D7 v1.23; BC: BC-2.16.012 INV-INVALIDATION-EXT-001
//
// use proptest::prelude::*;
// use prism_query::invalidation::{WriteToolInvalidationMap, register_write_tool, invalidation_map};
//
// proptest! {
//     #[test]
//     fn unique_registrations_all_succeed(
//         names in prop::collection::vec(r"[a-zA-Z][a-zA-Z0-9_]{0,31}", 1..=16usize)
//     ) {
//         // Deduplicate the generated names to construct a purely unique sequence
//         let unique_names: Vec<String> = {
//             let mut seen = std::collections::HashSet::new();
//             names.into_iter().filter(|n| seen.insert(n.clone())).collect()
//         };
//         let n = unique_names.len();
//         // Reset process-global state for isolation (test-only hooks in invalidation.rs).
//         prism_query::invalidation::reset_query_phase_global();
//         prism_query::invalidation::reset_dynamic_registry_global();
//         for name in &unique_names {
//             let entry = WriteToolInvalidationMap { tool_name: name.clone(), ..Default::default() };
//             prop_assert!(register_write_tool(entry).is_ok(),
//                 "unique registration must succeed (VP-156)");
//         }
//         let count = dynamic_write_tool_count();
//         prop_assert_eq!(count, n, "all unique registrations visible (VP-156)");
//     }
//
//     #[test]
//     fn duplicate_name_returns_error(
//         first_name in r"[a-zA-Z][a-zA-Z0-9_]{0,31}",
//         second_name in r"[a-zA-Z][a-zA-Z0-9_]{0,31}",
//     ) {
//         prism_query::invalidation::reset_query_phase_global();
//         prism_query::invalidation::reset_dynamic_registry_global();
//         let a = WriteToolInvalidationMap { tool_name: first_name.clone(), ..Default::default() };
//         let b = WriteToolInvalidationMap { tool_name: first_name.clone(), ..Default::default() };
//         prop_assume!(register_write_tool(a).is_ok());
//         let result = register_write_tool(b);
//         prop_assert!(result.is_err(),
//             "duplicate tool_name must return Err(DuplicateWriteToolRegistration) (VP-156)");
//         let count = dynamic_write_tool_count();
//         prop_assert_eq!(count, 1usize, "exactly one entry for duplicate name (VP-156)");
//     }
// }
```

**Test-only reset hooks (as-built, proof-completed-date 2026-05-18):** The harness uses two `#[cfg(test)]`-gated reset helpers exported from `crates/prism-query/src/invalidation.rs`:
- `reset_dynamic_registry_global()` — clears the `DYNAMIC_WRITE_TOOLS` `RwLock<Vec<...>>` global (sets the vector to empty).
- `reset_query_phase_global()` — resets the `QUERY_PHASE_STARTED` `AtomicBool` to `false`.

Both must be called before each proptest run to guarantee process-global isolation. The `dynamic_write_tool_count()` helper (also `#[cfg(test)]`-gated) provides the count observable from tests without requiring direct access to the RwLock guard.

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | proptest default 256 cases; name strings bounded to 32 chars |
| Proof complexity | Low | Synchronous RwLock + Vec — no async, no I/O, no WASM dependency |
| Tool support | Full | proptest is already in prism-query's dev-dependencies |
| Harness dependencies | Low | Requires `reset_query_phase_global()` + `reset_dynamic_registry_global()` (two-function reset pattern) and `dynamic_write_tool_count()` accessor in prism-query; no external services |
| Estimated proof time | < 5 seconds | Tight inner loop; 256 cases × synchronous Vec-push |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-15 | architect (prereq-e-fix-burst-1) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | prereq-e-fix-burst-1 | 2026-05-15 | architect | F-LP1-MED-003 resolution (option a). VP-156 authored to provide proptest coverage for `register_write_tool` uniqueness semantics (error-on-duplicate, ADR-026 D7) and happens-before correctness of the `RwLock<Vec<...>>` container (INV-INVALIDATION-EXT-001). Closes BC-2.16.012 §VP Anchors "(none in this story)" gap. Proptest chosen over Kani — String-keyed uniqueness is proptest territory, not Kani bounded model-checking. Harness skeleton provided; full authoring in S-PLUGIN-PREREQ-E scope. |
| 0.2 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `prereq-e-fix-burst-1` was informal slug; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 0.3 | prereq-e-fix-burst-2 | 2026-05-15 | architect | F-LP2-MED-002 (option b): happens-before claim removed from §Property Statement title and body; §Property Statement rewritten to cover uniqueness only; §Acceptance Criteria Case 1 updated to clarify sequential-only scope; §Proof Method coverage cell updated; proof harness skeleton comment updated. Visibility guarantee now documented as structural (RwLock contract + ADR-022 boot ordering) not proptest-verified. F-LP2-MED-003: source_invariant changed from INV-INVALIDATION-EXT-001 to null; invariant trace preserved in §Source Contract body via existing BC-2.16.012 INV-INVALIDATION-EXT-001 cite. F-LP2-HIGH-002: TD-A-003 alias canonicalized to TD-S-PLUGIN-PREREQ-A-003 in §Property Statement. |
| 0.4 | fix-burst-5 renumber-repair-redo | 2026-05-15 | state-manager | F-LP5-HIGH-003 renumber-repair-redo. FB4 assigned both the changelog-repair row and the modified-field-sync row to v0.3, producing two rows at the same version and violating monotonic strict order. Repair row renumbered 0.3→0.4. Absorbs FB4 modified-field-sync content: `modified:` field confirmed synced to ISO date "2026-05-15" per F-LP4-LOW-002 / POL-27 (most recent change: fix-burst-1 authoring + fix-burst-2 uniqueness-only reframe). Content summary retained: prior changelog had duplicate 0.1 entries (architect prereq-e-fix-burst-1 + state-manager catch both labeled 0.1); corrected to monotonic 0.1 → 0.2 (state-manager catch) → 0.3 (fix-burst-2 architect) → 0.4 (this row). Each distinct content change now holds a unique version. Frontmatter version updated to 0.4. Monotonic sequence verified: 0.1 → 0.2 → 0.3 → 0.4. |
| 0.5 | prereq-e-fix-burst-6 | 2026-05-16 | architect | F-LP6-MED-001 + F-LP6-LOW-002 — All live-narrative ADR-026 D7 version pins updated from stale v1.2 to current v1.7: §Source Contract BC row ("D7 v1.2" → "D7 v1.7"), §ADR row ("D7" → "D7 v1.7"), §Property Statement ("ADR-026 D7" → "ADR-026 D7 v1.7"), proof harness skeleton comment ("ADR-026 D7" → "ADR-026 D7 v1.7"). Consistent with BC-2.16.012 §Verification Properties VP-156 row pin of v1.7. TD-VSDD-091 exception confirmed: changelog rows citing historical D7 versions (v1.2, v1.5, v1.6, v1.7) are immutable records and are unchanged. |
| 0.6 | prereq-e-fix-burst-7 | 2026-05-16 | architect | F-LP7-HIGH-001 — within-FB6 sibling-sweep asymmetry catch: all 4 live-narrative ADR-026 D7 pins advanced v1.7 → v1.8 (FB6 architect bumped ADR-026 v1.7→v1.8 in same burst as the v1.2→v1.7 sweep; the sweep targeted the in-progress-version snapshot, leaving VP-156 behind by one version). POL-23 within-burst version-pin-order-gap defect class. |
| 0.7 | prereq-e-fix-burst-8 | 2026-05-16 | architect | F-LP8-HIGH-001 — within-FB7 sibling-sweep asymmetry final close: all 4 live-narrative ADR-026 D7 pins advanced v1.8 → v1.9 (FB7 D-586 bumped ADR-026 v1.8→v1.9 in same burst as the v1.7→v1.8 sweep; sweep targeted intermediate snapshot). POL-23 within-burst-version-pin-order-gap RECURRING-class defect; single-bump-per-source-artifact discipline applied this burst (ADR-026 stays at v1.9). |
| 0.8 | prereq-e-fix-burst-13 | 2026-05-16 | architect | F-LP14-HIGH-001 — within-FB12 sibling-sweep asymmetry close (5th RECURRENCE of POL-23 class): all 4 live-narrative ADR-026 D7 pins advanced v1.9 → v1.10 (FB12 architect D-603 bumped ADR-026 v1.9→v1.10 for Option A adjudication but did not sibling-sweep). Single-bump-per-source-artifact discipline applied this burst (ADR-026 stays at v1.10; only downstream pin sweep). POL-29 codification candidate strongly reinforced. |
| 0.9 | FB44 | 2026-05-16 | architect | F-LP56-HIGH-001 POL-23 sibling-sweep: all 4 live-narrative ADR-026 D7 pins advanced v1.10 → v1.15 (FB44 architect bumped ADR-026 v1.14→v1.15 for boot.rs call-site designation; VP-156 swept in same burst). |
| 0.10 | FB45 | 2026-05-16 | architect | FB45 sibling-sweep: all 4 live-narrative ADR-026 D7 pins advanced v1.15 → v1.16 (FB45 architect bumped ADR-026 v1.15→v1.16 for SS-22 + runtime_deliverables boot.rs entry; VP-156 swept in same burst). |
| 0.11 | FB50 | 2026-05-17 | architect | POL-23 sibling-sweep OBS-LP62-002 interpretation #2: ADR-026 D7 v1.16 live-narrative pins bumped to v1.17 (current ADR-026 version per FB47 §Related ADRs row edit; D7 content unchanged since v1.16). |
| 0.12 | FB51 | 2026-05-17 | state-manager | F-LP63-HIGH-003 closure: §Changelog v0.10/v0.11 row positions swapped to ascending order (v0.10 FB45 2026-05-16 BEFORE v0.11 FB50 2026-05-17); 7th POL-26 recurrence within FB50 sibling-sweep itself; POL-26 corollary bookkeeping repair. |
| 0.13 | FB55 | 2026-05-17 | product-owner | F-LP67-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.17→v1.18 propagation at VP-156 lines 42, 86, 90, 124 (4 live-narrative sites). POL-29 v1.16 step 3a (b) recurrence #18 within-burst closure. |
| 0.14 | FB56b | 2026-05-17 | product-owner | F-LP68-HIGH-001 closure cascade (FB56b PO scope): ADR-026 D7 pin v1.18→v1.19 propagation at VP-156 lines 42, 86, 90, 124 (4 sites). POL-29 v1.17 step 8a FIRST APPLICATION cascade closure. |
| 0.15 | FB62 | 2026-05-17 | product-owner | F-LP74-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.19→v1.21 at lines 42, 86, 90, 124 (4 sites). |
| 0.16 | FB64 | 2026-05-17 | product-owner | F-LP76-HIGH-001 closure (PO scope): burst-label cell corrected FB74→FB62 in §Changelog row for v0.15. Original FB62 closure of F-LP74-HIGH-001 was labeled "FB74" derived from finding ID; canonical FB sequential counter was FB62 per state-manager records. POL-26 schema integrity + POL-29 cross-domain sibling consistency restored. |
| 0.17 | FB69 | 2026-05-17 | product-owner | F-LP81-HIGH-002 closure (PO scope): ADR-026 D7 pin v1.21→v1.22 propagation at lines 42, 86, 90, 124 (4 sites; ascending changelog). Recurrence #22+ of POL-29 step 3a class (b). Sibling files story v1.44 + BC-2.16.012 v1.25 + BC-2.16.002 v1.29 + error-taxonomy v1.37 + HS-003 v1.14 swept in same burst. |
| 0.18 | FB73 | 2026-05-17 | product-owner | F-LP85-HIGH-001 closure (PO scope): ADR-026 D7 pin v1.22→v1.23 propagation at VP-156 lines 42, 86, 90, 124 (4 sites; ascending changelog). Sibling files story v1.46 + BC-2.16.011 v1.10 + BC-2.16.012 v1.26 + BC-2.16.002 v1.31 (POL-30 Fork B preserved) + HS-003 v1.15 + error-taxonomy v1.38 swept in same burst. |
| 0.19 | FB-IMPL-6 | 2026-05-18 | test-writer | Proptest landed. Authors 5 proptests in two integration test binaries: (1) `vp156_write_tool_registration_uniqueness.rs` — 4 proptests covering VP-156 AC Cases 1/2/3 and full-key idempotency; (2) `vp156_write_tool_post_boot_proptest.rs` — 1 proptest covering EC-016-012-005 post-boot rejection (separate binary per mark_query_phase_started global-state isolation pattern). Uniqueness invariant: `tool_name` alone (VP-156 §Property Statement; ADR-026 D7 v1.23). All 5 proptests pass (PROPTEST_CASES=32; 908 total prism-query tests pass; cargo check --workspace --tests exit 0). lifecycle_status: draft → active; proof_completed_date: 2026-05-18. Sibling-sweep of F-LP-IMPL-P8-IMP-001 (VP-153 landing) — same VP-artifact-existence blind-spot class. Anchored: BC-2.16.012 EC-016-012-004/005 + TD-S-PLUGIN-PREREQ-A-003 closure semantic. |
| 0.21 | pass-11-spec-hygiene | 2026-05-18 | product-owner | §Feasibility Assessment row 184 symbol corrections — sibling-sweep completion of F-LP-IMPL-P10-OBS-002 (closes F-LP-IMPL-P11-MED-001). `reset_for_test()` → two-function pattern `reset_query_phase_global()` + `reset_dynamic_registry_global()`; `invalidation_map()` → `dynamic_write_tool_count()`. As-built API names per §Test-only reset hooks paragraph (lines 171-175). |
| 0.20 | pass-10-spec-hygiene | 2026-05-18 | product-owner | F-LP-IMPL-P10-OBS-002 closure: §Proof Harness Skeleton stale symbol corrections. (1) `reset_for_test()` → two-function pattern: `reset_query_phase_global()` + `reset_dynamic_registry_global()` (2 skeleton sites, lines 141+156). (2) `invalidation_map().read().unwrap()` + `.len()` → `dynamic_write_tool_count()` (first site, line 147). (3) `invalidation_map().read().unwrap()` + `.iter().filter(...).count()` → `dynamic_write_tool_count()` (second site, line 163). (4) Test-only reset hook paragraph updated to describe the two-function as-built API and `dynamic_write_tool_count()` helper. (5) POL-26 monotonic-order repair: rows v0.19 (FB-IMPL-6 2026-05-18) and v0.18 (FB73 2026-05-17) were out of ascending order — swapped to restore v0.18 → v0.19 → v0.20. Spec brought into alignment with as-built code per CLAUDE.md Source-of-Truth Precedence Rule 7. |
