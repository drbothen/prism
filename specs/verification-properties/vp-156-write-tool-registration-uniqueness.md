---
document_type: verification-property
level: L4
version: "0.4"
status: draft
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
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-05-15"
modified: "2026-05-15"
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

`register_write_tool` in `crates/prism-query/src/invalidation.rs` (ADR-026 D7,
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
  to error-on-duplicate by ADR-026 D7 v1.2). INV-INVALIDATION-EXT-001 (runtime extensibility
  postcondition). VP-156 provides proptest coverage for the uniqueness semantics that
  BC-2.16.012 §Verification Properties previously described as "(none in this story)".
  This VP closes that coverage gap per F-LP1-MED-003 resolution (option a: author VP-156).
- **ADR:** ADR-026 D7 — specifies the error-on-duplicate API contract (`register_write_tool`
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
// ADR: ADR-026 D7; BC: BC-2.16.012 INV-INVALIDATION-EXT-001
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
//         // Reset state (test-only reset hook required in invalidation.rs)
//         prism_query::invalidation::reset_for_test();
//         for name in &unique_names {
//             let entry = WriteToolInvalidationMap { tool_name: name.clone(), ..Default::default() };
//             prop_assert!(register_write_tool(entry).is_ok(),
//                 "unique registration must succeed (VP-156)");
//         }
//         let guard = invalidation_map().read().unwrap();
//         prop_assert_eq!(guard.len(), n, "all unique registrations visible (VP-156)");
//     }
//
//     #[test]
//     fn duplicate_name_returns_error(
//         first_name in r"[a-zA-Z][a-zA-Z0-9_]{0,31}",
//         second_name in r"[a-zA-Z][a-zA-Z0-9_]{0,31}",
//     ) {
//         prism_query::invalidation::reset_for_test();
//         let a = WriteToolInvalidationMap { tool_name: first_name.clone(), ..Default::default() };
//         let b = WriteToolInvalidationMap { tool_name: first_name.clone(), ..Default::default() };
//         prop_assume!(register_write_tool(a).is_ok());
//         let result = register_write_tool(b);
//         prop_assert!(result.is_err(),
//             "duplicate tool_name must return Err(DuplicateWriteToolRegistration) (VP-156)");
//         let guard = invalidation_map().read().unwrap();
//         let count = guard.iter().filter(|e| e.tool_name == first_name).count();
//         prop_assert_eq!(count, 1usize, "exactly one entry for duplicate name (VP-156)");
//     }
// }
```

**Test-only reset hook:** The harness requires `prism_query::invalidation::reset_for_test()`
(gated behind `#[cfg(test)]`) to clear the global `RwLock<Vec<...>>` between proptest cases.
The implementer adds this function in `crates/prism-query/src/invalidation.rs` under
`#[cfg(test)]`.

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | proptest default 256 cases; name strings bounded to 32 chars |
| Proof complexity | Low | Synchronous RwLock + Vec — no async, no I/O, no WASM dependency |
| Tool support | Full | proptest is already in prism-query's dev-dependencies |
| Harness dependencies | Low | Requires `reset_for_test()` hook and `invalidation_map()` accessor in prism-query; no external services |
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
