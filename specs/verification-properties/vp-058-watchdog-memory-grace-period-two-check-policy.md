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
  - specs/behavioral-contracts/BC-2.15.007
input-hash: "47125c0"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.15.007
module: prism-persistence
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

# VP-058: Watchdog Memory Grace Period — Two-Check Policy

## Property Statement

`should_terminate_for_memory(state: WatchdogCheckState) -> bool` returns `true` if and
only if `state.consecutive_over_limit >= 2`. A single check with memory above the limit
does not terminate (returns `false`). Two consecutive checks above the limit do terminate
(returns `true`). The threshold is exactly 2 checks — not 1 or 3. The function implements
DI-027 (Watchdog) and enforces the grace period to tolerate single transient memory spikes.

## Source Contract

- **Anchor Story:** `S-2.02`
- **Source BC:** BC-2.15.007 — Watchdog Query Termination
- **Module:** prism-persistence
- **Category:** Safety-Critical / Resource Protection

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — consecutive_over_limit values 0..=u8::MAX | All counter values including boundary conditions |

**Feasibility:** `should_terminate_for_memory` is a pure predicate on `WatchdogCheckState`.
Structurally identical to VP-057 (crash counter) but for the memory violation counter.
A proptest verifies the exact boundary (0 → false, 1 → false, 2 → true, all >= 2 → true).
The proptest approach is chosen over Kani because the function's simplicity makes proptest
the lower-overhead proof vehicle while still providing exhaustive coverage.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_persistence::watchdog::should_terminate_for_memory
//
// proptest! {
//     #[test]
//     fn verify_two_check_policy(consecutive_over_limit in 0u8..=u8::MAX) {
//         let state = WatchdogCheckState { consecutive_over_limit };
//         let result = should_terminate_for_memory(state);
//
//         if consecutive_over_limit >= 2 {
//             prop_assert!(result, "must terminate when consecutive_over_limit >= 2");
//         } else {
//             prop_assert!(!result, "must NOT terminate when consecutive_over_limit < 2");
//         }
//     }
//
//     #[test]
//     fn verify_exact_boundary(_dummy in 0u8..=0) {
//         // Boundary: single check (count=1) does not terminate
//         let single_check = WatchdogCheckState { consecutive_over_limit: 1 };
//         assert!(!should_terminate_for_memory(single_check),
//             "single check above limit must not terminate (grace period)");
//
//         // Boundary: two checks (count=2) terminates
//         let two_checks = WatchdogCheckState { consecutive_over_limit: 2 };
//         assert!(should_terminate_for_memory(two_checks),
//             "two consecutive checks above limit must terminate");
//
//         // Below boundary: zero checks does not terminate
//         let zero_checks = WatchdogCheckState { consecutive_over_limit: 0 };
//         assert!(!should_terminate_for_memory(zero_checks),
//             "zero checks above limit must not terminate");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | consecutive_over_limit is u8 (0..=255); fully enumerable by proptest |
| Tool support? | Full | Pure predicate function; no async or I/O |
| Execution time budget | <30 seconds | Trivial predicate over u8 range; instant proptest run |
| Assumptions required | `should_terminate_for_memory` is a pure function extractable from the watchdog loop | BC-2.15.007 design requires this extraction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.15.007. Proves two-check grace period policy: single spike does not terminate, two consecutive checks do. Method: Proptest. P0. |
