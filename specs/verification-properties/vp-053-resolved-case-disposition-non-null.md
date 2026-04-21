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
  - specs/behavioral-contracts/BC-2.14.006
input-hash: "ac6b633"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.006
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

# VP-053: Resolved Case Always Has Non-Null Disposition

## Property Statement

For any `CaseRecord` produced by `advance_case_state(case, CaseStatus::Resolved)`,
`record.disposition.is_some()` holds. When `case.disposition.is_none()`, `advance_case_state`
returns `Err(E-CASE-006)` rather than producing a resolved case with null disposition. No
`CaseRecord` can simultaneously have `status = Resolved` and `disposition = None` — this
invariant is a postcondition of the state machine transition function.

## Source Contract

- **Anchor Story:** `S-4.06`
- **Source BC:** BC-2.14.006 — Disposition Assignment
- **Module:** prism-core
- **Category:** Data Integrity / State Machine Postcondition

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — symbolic CaseRecord with symbolic disposition | All paths through advance_case_state targeting Resolved |

**Feasibility:** The disposition field is `Option<CaseDisposition>` (an enum with a small
fixed set of variants). Kani can symbolically construct all `(CaseRecord, CaseStatus::Resolved)`
inputs and prove the postcondition. The state machine is small and deterministic.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::case::advance_case_state
//
// #[kani::proof]
// fn verify_resolved_case_disposition_non_null() {
//     let disposition: Option<CaseDisposition> = kani::any();
//     let current_status: CaseStatus = kani::any();
//
//     let case = CaseRecord {
//         status: current_status,
//         disposition,
//         ..kani::any()
//     };
//
//     let result = advance_case_state(&case, CaseStatus::Resolved);
//
//     match result {
//         Ok(updated) => {
//             // Any successful Resolved transition must have a disposition
//             kani::assert!(
//                 updated.disposition.is_some(),
//                 "Resolved CaseRecord must have non-null disposition"
//             );
//         }
//         Err(CaseError::MissingDisposition) => {
//             // Correct: transition rejected because disposition was None
//             kani::assert!(
//                 disposition.is_none(),
//                 "E-CASE-006 only when case.disposition is None"
//             );
//         }
//         Err(_) => {
//             // Other errors (invalid transition, self-transition) are acceptable
//         }
//     }
//
//     // Invariant: if result is Ok(Resolved), disposition must be Some
//     if let Ok(ref updated) = result {
//         if updated.status == CaseStatus::Resolved {
//             kani::assert!(updated.disposition.is_some(),
//                 "no CaseRecord with status=Resolved and disposition=None may exist");
//         }
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | CaseDisposition enum is small and finite; symbolic construction is tractable |
| Tool support? | Full | Kani handles Option<enum> symbolic values efficiently |
| Execution time budget | <60 seconds | Small state space; one function call with bounded inputs |
| Assumptions required | `advance_case_state` is a pure function | BC design requires this for Kani feasibility |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.14.006. Proves postcondition: no CaseRecord with Resolved status and None disposition. Method: Kani. P0. |
