---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.14.002
input-hash: "1e29f9d"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.002
module: prism-core
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
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

# VP-051: Case State Machine — Exhaustive 5×5 Transition Table

## Property Statement

For every (from_state, to_state) pair in the 5×5 case state matrix (25 combinations),
`advance_case_state(from, to)` returns `Ok` for exactly the 12 valid transitions and
`Err(E-CASE-004)` or `Err(E-CASE-005)` for all 13 invalid pairs. Self-transitions always
return `Err(E-CASE-005)`. Backward transitions (to `New` or `Acknowledged` from any state)
always return `Err(E-CASE-004)`. Reopen targets only the `Investigating` state from `Closed`.

This VP complements VP-005 (which proves the count of valid transitions = 12) and VP-006
(which proves no self-transitions) by proving the per-pair exhaustive accept/reject outcomes
and correct structured error codes for each invalid pair.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** BC-2.14.002 — Case State Transitions
- **Module:** prism-core
- **Category:** Safety-Critical / State Machine Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — all 25 (state × state) combinations | All transition paths, all error code branches |

**Feasibility:** The case state machine has 5 states (New, Acknowledged, Investigating, Resolved,
Closed) and 25 transition pairs. Kani can enumerate all symbolic `(CaseStatus, CaseStatus)` pairs
and prove exact Ok/Err outcomes. The state space is tiny; proof should complete in under 60 seconds.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::case::advance_case_state
//
// State machine:
//   Valid transitions (Ok):
//     New → Acknowledged
//     New → Investigating
//     Acknowledged → Investigating
//     Acknowledged → Resolved
//     Investigating → Resolved
//     Investigating → Acknowledged
//     Resolved → Closed
//     Resolved → Investigating  (reopen)
//     Closed → Investigating    (reopen)
//     New → Resolved            (direct resolve — if spec allows)
//     Acknowledged → Closed     (if spec allows)
//     ... (exactly 12 as enumerated in BC-2.14.002)
//
// Invalid transitions (Err):
//   - Self-transitions: New→New, Ack→Ack, etc. → E-CASE-005
//   - Backward to New or Acknowledged from Resolved/Closed → E-CASE-004
//   - All other prohibited pairs → E-CASE-004
//
// #[kani::proof]
// fn verify_exhaustive_transition_table() {
//     let from: CaseStatus = kani::any();
//     let to: CaseStatus = kani::any();
//     let result = advance_case_state(from, to);
//
//     // Enumerate valid transitions
//     let is_valid = matches!(
//         (from, to),
//         (CaseStatus::New, CaseStatus::Acknowledged)
//         | (CaseStatus::New, CaseStatus::Investigating)
//         | ... // all 12 valid pairs
//     );
//
//     if is_valid {
//         kani::assert(result.is_ok(), "valid transition must return Ok");
//     } else if from == to {
//         kani::assert!(matches!(result, Err(CaseError::SelfTransition)),
//             "self-transition must return E-CASE-005");
//     } else {
//         kani::assert!(result.is_err(), "invalid transition must return Err");
//         // Verify error code is E-CASE-004 or E-CASE-005 as appropriate
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 5×5 = 25 pairs, fully enumerable by Kani |
| Tool support? | Full | Trivially bounded state space; Kani handles enum symbolic values |
| Execution time budget | <60 seconds | 25 pairs × simple match — very fast |
| Assumptions required | `advance_case_state` is a pure function extractable for proof | BC-2.14.002 design requires this extraction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.14.002. Complements VP-005/006 with per-pair exhaustive table proof. Method: Kani. P0. |
