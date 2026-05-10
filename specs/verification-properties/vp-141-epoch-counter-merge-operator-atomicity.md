---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:30:00Z
phase: 4-W4-Phase2-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-018-differential-result-pack-format.md
  - .factory/STATE.md
input-hash: "88f5531"
traces_to: .factory/specs/architecture/decisions/ADR-018-differential-result-pack-format.md
source_bc: null
source_adr: ADR-018
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-2-adr
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-141: Epoch Counter Merge Operator Atomicity

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-018` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-018 transitions to Accepted and produces a concrete BC in Phase 4.B.

> **[STUB — full VP authoring deferred to Phase 4.B BC authoring]**

## Property Statement

ADR-018 §2 specifies that the epoch counter uses a RocksDB `merge_operator`
for atomic increment. This VP verifies that concurrent increments to the epoch
counter via the merge operator are never lost — the final counter value after N
concurrent increment operations must equal the initial value plus N, regardless
of interleaving. The merge operator's associativity and commutativity guarantees
are the basis for this atomicity claim.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-018 — Differential Result Pack Format, §2 Epoch Counter Merge Operator
- **Decision Reference:** epoch counter merge_operator atomicity; concurrent increments never lost
- **Postcondition/Invariant:** final_epoch == initial_epoch + N after N concurrent increment operations.
- **BC:** To be assigned when ADR-018 is Accepted and BC authoring completes in Wave 4 Phase 4.B.
- **Module:** prism-operations
- **Category:** Atomicity / Concurrency

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — bounded N concurrent increments | merge_operator atomicity under concurrent load |

**Feasibility:** The merge operator's semantics are deterministic and bounded;
proptest can generate randomized N-increment scenarios and verify the final
counter value against the expected sum.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::diff::epoch_counter (RocksDB merge_operator)
//
// proptest! {
//     #[test]
//     fn epoch_counter_concurrent_increments_not_lost(
//         n in 1usize..=64,
//         initial in 0u64..=1000,
//     ) {
//         let counter = EpochCounter::new(initial);
//         // Simulate N concurrent increments via merge_operator
//         let final_value = counter.apply_n_increments(n);
//         prop_assert_eq!(final_value, initial + n as u64);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | N increments bounded to small range; merge operator state is a single u64 |
| Proof complexity | Low | Single arithmetic assertion: final == initial + N |
| Tool support | Full | proptest 1.x; RocksDB merge_operator testable via MockStorageEngine |
| Estimated proof time | <30 seconds | Trivial state space; merge operator is pure arithmetic |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase2-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-018 §2. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 4.B. |
