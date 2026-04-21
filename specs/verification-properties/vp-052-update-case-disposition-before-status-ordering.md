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
  - specs/behavioral-contracts/BC-2.14.003
input-hash: "8e43eb2"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.003
module: prism-core
priority: P0
proof_method: proptest
verification_method: proptest
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

# VP-052: update_case — Disposition Applied Before Status Transition

## Property Statement

For any `CaseUpdateSpec` containing both `disposition: Some(d)` and `status: Some(Resolved)`,
`apply_update_fields(case, spec)` applies the disposition update before the status transition.
A single call with `disposition=FalsePositive` and `status=Resolved` succeeds when the case
has no prior disposition. The same call with status applied first would fail with `E-CASE-006`
(status transition requires disposition already set). The timeline records `DispositionSet`
before `StatusChanged` in all combined-update scenarios.

## Source Contract

- **Anchor Story:** `S-4.06`
- **Source BC:** BC-2.14.003 — `update_case` MCP Tool
- **Module:** prism-core
- **Category:** Business Rule / Update Ordering

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — bounded CaseUpdateSpec generation | All disposition+status combination paths |

**Feasibility:** `apply_update_fields` is a pure function: it takes a `CaseRecord` and a
`CaseUpdateSpec` and produces a `CaseRecord` update delta. A proptest can generate specs with
both fields set, verify ordering outcomes, and assert that the combined call succeeds while a
status-first call fails. No I/O or side effects in the pure function.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_core::case::apply_update_fields
//
// Strategy:
//   - Generate CaseRecord with status=Investigating, disposition=None
//   - Generate CaseUpdateSpec with disposition=Some(FalsePositive), status=Some(Resolved)
//
// proptest! {
//     #[test]
//     fn verify_disposition_before_status_ordering(
//         disposition in arb_case_disposition(),
//         initial_status in arb_case_status_open(),
//     ) {
//         let case = CaseRecord {
//             status: initial_status,
//             disposition: None,
//             ..Default::default()
//         };
//         let spec = CaseUpdateSpec {
//             disposition: Some(disposition),
//             status: Some(CaseStatus::Resolved),
//             ..Default::default()
//         };
//
//         // Combined update (disposition-first) must succeed
//         let result = apply_update_fields(&case, &spec);
//         prop_assert!(result.is_ok(), "combined disposition+resolve must succeed");
//
//         // Verify timeline order: DispositionSet before StatusChanged
//         let delta = result.unwrap();
//         let disp_idx = delta.timeline.iter().position(|e| matches!(e, TimelineEvent::DispositionSet(_)));
//         let stat_idx = delta.timeline.iter().position(|e| matches!(e, TimelineEvent::StatusChanged(_)));
//         prop_assert!(disp_idx < stat_idx, "DispositionSet must precede StatusChanged in timeline");
//
//         // Status-first application must fail
//         let status_first_spec = CaseUpdateSpec {
//             disposition: None,
//             status: Some(CaseStatus::Resolved),
//             ..Default::default()
//         };
//         let status_first_result = apply_update_fields(&case, &status_first_spec);
//         prop_assert!(status_first_result.is_err(), "resolve without disposition must fail E-CASE-006");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | CaseRecord and CaseUpdateSpec are finite structs; proptest generates bounded variants |
| Tool support? | Full | Pure function — no RocksDB or async required for the ordering proof |
| Execution time budget | <2 minutes | Small struct space; fast proptest exploration |
| Assumptions required | `apply_update_fields` is extractable as a pure function | BC-2.14.003 design requires this extraction for testability |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.14.003. Proves disposition-before-status ordering as a pure-function property. Method: Proptest. P0. |
