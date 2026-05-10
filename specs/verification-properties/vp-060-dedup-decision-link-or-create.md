---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.14.013-auto-case-creation.md
input-hash: "76729b7"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.14.013
module: prism-operations
priority: P0
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1/pass-74-defer-close
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

# VP-060: Dedup Decision — Link or Create

## Property Statement

Given a pure function `decide_dedup_action(existing_case: Option<CaseSummary>, new_alert: Alert, window_secs: u64, now: TimestampSecs) -> DedupDecision`, where `DedupDecision = Link(CaseId) | Create`, the function returns `Link(c.id)` if and only if `existing_case` is `Some(c)` AND `now - c.created_at < window_secs`; returns `Create` otherwise. The function is total, deterministic, and side-effect-free.

## Source Contract

- **Anchor Story:** `S-4.06`
- **Source BC:** BC-2.14.013 — Auto-Case-Creation from High-Severity Detection Rules
- **Module:** prism-operations
- **Category:** Correctness / Case Management

## Scope

**IN SCOPE:** Pure decision function `decide_dedup_action()` extracted from `CaseDedupRegistry::check_and_create()`. Operates on input parameters only; no I/O, no clock access, no random.

**OUT OF SCOPE:** The effectful wrapper `CaseDedupRegistry::check_and_create()` itself (which queries RocksDB and executes WriteBatch). Wrapper is verified by integration test in S-4.06 AC-12 (link existing case under dedup window) and AC-12b (create new case outside dedup window).

**OUT OF SCOPE:** Concurrent dedup atomicity (RocksDB WriteBatch transaction guarantees handle this; integration concern).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — three input families with full boundary coverage | Link/Create decision correctness for all input combinations |

**Feasibility:** `decide_dedup_action` is a pure function over `Option<CaseSummary>`, `u64` window, and `u64` timestamp. No async, no I/O. The three proptest families fully partition the input space (Link case, Create case, boundary case). This is structurally similar to VP-027 (alert dedup key correctness) and VP-047 (UUID v7 validation) — both proven feasible via proptest. Extraction of `decide_dedup_action` as a standalone pure function is mandated by S-4.06 task 9 update.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::case::dedup::decide_dedup_action
//
// use proptest::prelude::*;
// use prism_operations::case::dedup::{decide_dedup_action, DedupDecision};
// use prism_core::case::{CaseSummary, CaseId};
//
// proptest! {
//     /// Family 1: Link case
//     /// existing_case = Some(c), now - c.created_at < window_secs
//     #[test]
//     fn verify_link_case(
//         case_id in any::<u128>().prop_map(CaseId::from_u128),
//         created_at in 0u64..u64::MAX / 2,
//         window_secs in 1u64..u64::MAX / 2,
//         // delta chosen so that now - created_at < window_secs
//         delta in 0u64..1,
//         alert in any_alert(),
//     ) {
//         let now = created_at + delta.min(window_secs.saturating_sub(1));
//         let existing_case = Some(CaseSummary { id: case_id, created_at, ..Default::default() });
//         let result = decide_dedup_action(existing_case, alert, window_secs, now);
//         prop_assert_eq!(result, DedupDecision::Link(case_id));
//     }
//
//     /// Family 2a: Create case — no existing case
//     #[test]
//     fn verify_create_when_no_existing_case(
//         window_secs in 0u64..=u64::MAX,
//         now in 0u64..=u64::MAX,
//         alert in any_alert(),
//     ) {
//         let result = decide_dedup_action(None, alert, window_secs, now);
//         prop_assert_eq!(result, DedupDecision::Create);
//     }
//
//     /// Family 2b: Create case — existing case outside window
//     #[test]
//     fn verify_create_when_case_outside_window(
//         case_id in any::<u128>().prop_map(CaseId::from_u128),
//         created_at in 0u64..u64::MAX / 2,
//         window_secs in 0u64..u64::MAX / 2,
//         alert in any_alert(),
//     ) {
//         // now - created_at >= window_secs
//         let now = created_at.saturating_add(window_secs);
//         let existing_case = Some(CaseSummary { id: case_id, created_at, ..Default::default() });
//         let result = decide_dedup_action(existing_case, alert, window_secs, now);
//         prop_assert_eq!(result, DedupDecision::Create);
//     }
//
//     /// Family 3: Boundary — now - created_at == window_secs exactly => Create (exclusive upper bound)
//     #[test]
//     fn verify_boundary_exclusive_upper(
//         case_id in any::<u128>().prop_map(CaseId::from_u128),
//         created_at in 0u64..u64::MAX / 2,
//         window_secs in 1u64..u64::MAX / 2,
//         alert in any_alert(),
//     ) {
//         let now = created_at + window_secs; // delta == window_secs, so NOT < window_secs
//         let existing_case = Some(CaseSummary { id: case_id, created_at, ..Default::default() });
//         let result = decide_dedup_action(existing_case, alert, window_secs, now);
//         prop_assert_eq!(result, DedupDecision::Create,
//             "boundary (delta == window_secs) must return Create (window is exclusive at upper bound)");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | `Option<CaseSummary>`, `u64` window, `u64` now; three partitioned families exhaust the decision space |
| Tool support? | Full | Pure function with no async or I/O; proptest handles all families |
| Execution time budget | <60 seconds | 10,000 cases across 4 harnesses; comparable to VP-027 and VP-047 |
| Assumptions required | `decide_dedup_action` is extracted as a pure function in `crates/prism-operations/src/case/dedup.rs` | Mandated by S-4.06 task 9 update |
| Edge cases | window_secs=0 (always Create), window_secs=u64::MAX (always Link if Some) | Covered by Family 2b and Family 1 respectively |

## Success Criteria

- 10,000 proptest cases pass without counterexample across all four harnesses
- Boundary cases (window_secs == 0, window_secs == u64::MAX) verified
- Decision is total: every input combination has a defined output
- No panic / unwrap / divergence in the function body (covered by `#![deny(clippy::panic_in_result_fn)]` in prism-operations)

## Dependencies

- Depends on `decide_dedup_action()` being extracted as a pure function in `crates/prism-operations/src/case/dedup.rs` (mandated by S-4.06 task 9 update)
- Depends on `CaseSummary`, `Alert`, `DedupDecision`, `TimestampSecs` types (defined in prism-core / prism-operations)

## Lifecycle

| Field | Value |
|-------|-------|
| Status | draft |
| Introduced | cycle-1 / pass-74-defer-close |
| Modified | null |
| Withdrawal Reason | null |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Auto Case Deduplication Atomicity" → "Auto-Case-Creation from High-Severity Detection Rules" (matches BC-2.14.013 H1). |
| 1.0 | pass-74-defer-close | 2026-04-20 | architect | Initial draft. Closes BC-2.14.013 DEFER from pass-74 decision matrix v1.1. Verifies the pure decision function `decide_dedup_action()` extracted from `CaseDedupRegistry::check_and_create()`. Effectful wrapper remains integration-tested via S-4.06 AC-12 and AC-12b. |
