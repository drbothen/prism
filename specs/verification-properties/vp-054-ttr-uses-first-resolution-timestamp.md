---
document_type: verification-property
level: L4
version: "1.2"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.14.008
input-hash: "572c2a9"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.008
module: prism-operations
priority: P1
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

# VP-054: TTR Uses First Resolution Timestamp Across Reopen Cycles

## Property Statement

For any `CaseRecord` with N reopen cycles (N >= 1), `compute_ttr(case)` equals
`case.resolved_at_first - case.created_at` using the FIRST `resolved_at` timestamp;
`resolved_at_first` is set on the initial transition to `Resolved` and is never overwritten
on subsequent reopen+resolve cycles. Additionally, `compute_mttd_avg([])`,
`compute_mtti_avg([])`, and `compute_mttr_avg([])` all return `None` (not zero) when given
an empty input slice. All computed metrics are non-negative (floored at `Duration::ZERO`).

## Source Contract

- **Anchor Story:** `S-4.06`
- **Source BC:** BC-2.14.008 — TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions
- **Module:** prism-operations
- **Category:** Business Rule / Metric Computation Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — bounded reopen cycle count (0..=10), timestamp ranges | All reopen-cycle permutations |

**Feasibility:** `compute_ttr` is a pure function over `CaseRecord`. A proptest can generate
`CaseRecord` values with multiple reopen cycles (alternating Resolved → Investigating → Resolved)
and verify the first-resolution semantics. The function operates only on timestamps and enum
values — no I/O dependency.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::metrics::{compute_ttr, compute_mttd_avg, compute_mtti_avg, compute_mttr_avg}
//
// proptest! {
//     #[test]
//     fn verify_ttr_uses_first_resolution(
//         reopen_cycles in 1usize..=10,
//         created_at in any::<SystemTime>(),
//         first_resolved_offset in 1u64..=3600,
//     ) {
//         let first_resolved_at = created_at + Duration::from_secs(first_resolved_offset);
//
//         // Construct case with N reopen cycles; each subsequent resolution happens later
//         let case = build_case_with_reopen_cycles(
//             created_at,
//             first_resolved_at,
//             reopen_cycles,
//         );
//
//         // TTR must always use the first resolution timestamp
//         let ttr = compute_ttr(&case);
//         prop_assert!(ttr.is_some(), "TTR must be Some for a resolved case");
//         let expected = first_resolved_at.duration_since(created_at).unwrap();
//         prop_assert_eq!(ttr.unwrap(), expected,
//             "TTR must equal first-resolved-at minus created-at");
//     }
//
//     #[test]
//     fn verify_aggregate_metrics_none_on_empty(_dummy in 0u8..=0) {
//         let empty: Vec<CaseRecord> = vec![];
//         prop_assert!(compute_mttd_avg(&empty).is_none(), "empty input → None MTTD");
//         prop_assert!(compute_mtti_avg(&empty).is_none(), "empty input → None MTTI");
//         prop_assert!(compute_mttr_avg(&empty).is_none(), "empty input → None MTTR");
//     }
//
//     #[test]
//     fn verify_metrics_non_negative(cases in vec(arb_case_record(), 1..=50)) {
//         if let Some(mttd) = compute_mttd_avg(&cases) {
//             prop_assert!(mttd >= Duration::ZERO, "MTTD must be non-negative");
//         }
//         if let Some(mttr) = compute_mttr_avg(&cases) {
//             prop_assert!(mttr >= Duration::ZERO, "MTTR must be non-negative");
//         }
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Reopen cycles bounded 1..=10; timestamps are numeric offsets |
| Tool support? | Full | Pure computation functions with no side effects |
| Execution time budget | <2 minutes | Fast arithmetic operations over bounded case records |
| Assumptions required | `compute_ttr` exposes `resolved_at_first` field separately from subsequent resolutions | BC-2.14.008 requires this field distinction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.2 | pass-90-F90-004 | 2026-04-21 | architect | F90-004: module canonicalized prism-core → prism-operations; proof skeleton target updated to prism_operations::metrics::{compute_ttr, compute_mttd_avg, compute_mtti_avg, compute_mttr_avg}. |
| 1.1 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "TTD/TTI/TTR Computation" → "TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions" (matches BC-2.14.008 H1). |
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.14.008. Proves TTR uses first resolution timestamp across reopen cycles and null-propagation on empty inputs. Method: Proptest. P1. |
