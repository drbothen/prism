---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T03:00:00Z
phase: 4-W4-Phase4A-Pass1-fix
inputs:
  - .factory/specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md
  - .factory/STATE.md
input-hash: "746d597"
traces_to: .factory/specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md
source_bc: null
source_adr: ADR-017
source_invariant: INV-CASE-006
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-4-a-pass-1-fix
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

# VP-145: Case reopen_count Monotonic Increment (INV-CASE-006)

> **NOTE:** This VP traces to ADR-017 §3.3 and the invariant INV-CASE-006, which was added in
> ADR-017 v0.2 as part of the Wave 4 Phase 4.A Pass 1 remediation. `source_bc: null` and
> `source_adr: ADR-017` are used because the primary constraint originates in the architecture
> decision record; a BC anchor will be added when ADR-017 transitions to Accepted and produces
> a concrete BC.

> **[STUB — full VP authoring deferred to Phase 5 formal-verify]**

## Property Statement

The `reopen_count` field on a Case MUST increment by exactly 1 on every
`Resolved/Closed → Investigating` state transition (case reopening event) and MUST NEVER
be reset to zero or decremented at any point during the case lifecycle, including on case
reassignment, status updates that do not constitute a reopen, or full case archival.
This monotonicity invariant is INV-CASE-006 as defined in ADR-017 v0.2 §3.3.

A proptest round-trip generates arbitrary sequences of case lifecycle transitions including
multiple open/resolve/reopen cycles and verifies that: (a) `reopen_count` equals exactly the
number of `Resolved/Closed → Investigating` transitions in the sequence, (b) no non-reopen
transition changes `reopen_count`, and (c) `reopen_count` is never decremented.

## Source Contract

- **ADR:** ADR-017 — Case Lifecycle Invariants, §3.3 reopen_count Monotonicity
- **Invariant:** INV-CASE-006 — reopen_count MUST increment on every Resolved/Closed →
  Investigating transition; never reset.
- **BC:** To be assigned when ADR-017 is Accepted and BC authoring completes.
- **Module:** prism-operations
- **Category:** State Machine Invariant / Monotonicity

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized lifecycle transition sequences | reopen_count monotonicity across all valid transition sequences |

**Feasibility:** The case lifecycle is a finite state machine with 5 states and 12 valid
transitions. proptest can generate all meaningful transition sequences and verify the
reopen_count invariant holds across all of them. The state space is small and bounded.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::case::CaseManager
//
// proptest! {
//     #[test]
//     fn reopen_count_monotonic(transitions in arb_case_transition_sequence()) {
//         let mut case = Case::new();
//         let mut expected_reopens = 0u32;
//         for t in transitions {
//             if is_reopen_transition(&case.status, &t) {
//                 expected_reopens += 1;
//             }
//             case.apply_transition(t).ok(); // may reject invalid transitions
//             prop_assert!(case.reopen_count >= expected_reopens,
//                 "reopen_count must be monotonically non-decreasing");
//         }
//         prop_assert_eq!(case.reopen_count, expected_reopens);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | 5-state FSM, 12 valid transitions; proptest can enumerate all meaningful sequences |
| Proof complexity | Low | Single monotonicity assertion over a counter field |
| Tool support | Full | proptest 1.x handles arbitrary transition sequence generation |
| Estimated proof time | <15 seconds | Very small state space; counter-increment property is trivial to check |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | state-manager (W4-Phase4A-Pass1-fix burst) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase4A-Pass1-fix | 2026-05-02 | state-manager | Initial stub. Traces to ADR-017 v0.2 §3.3 INV-CASE-006. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Phase 5 formal-verify. Added as gap closure from adversary Pass 1 finding H-010. |
