---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-21T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.20.002-log-forwarder-min-level-filter.md
input-hash: "ac6b633"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.20.002
module: prism-mcp
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1/pass-81-remediation
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

# VP-061: Log Forwarder Min-Level Filter — Per-Destination Enqueue/Discard Determinism

## Property Statement

Given a pure function `should_enqueue(entry_level: LogLevel, min_level: LogLevel) -> bool`,
where `LogLevel` is an ordered enum `Trace < Debug < Info < Warn < Error`, the function
returns `true` if and only if `level_rank(entry_level) >= level_rank(min_level)`.
The function is total, deterministic, and side-effect-free. For every pair in the
5×5 input space (25 combinations), the decision matches the level-rank ordering invariant.
A destination that omits `min_level` defaults to `Info`.

## Source Contract

- **Anchor Story:** `S-5.09`
- **Source BC:** BC-2.20.002 — Log Forwarder Min-Level Filter
- **Module:** prism-mcp
- **Category:** Correctness / Log Forwarding

## Scope

**IN SCOPE:** Pure function `should_enqueue(entry_level, min_level)` implementing
level-rank comparison. Operates on two `LogLevel` enum variants; no I/O, no queue access,
no state mutation.

**OUT OF SCOPE:** The effectful queue write operation itself (integration concern). The
multi-destination dispatch loop (BC-2.20.005 scope). Min-level config validation at
load time (config-parsing path; not a runtime pure function).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — exhaustive 5×5 matrix (25 pairs) + default min_level=Info case | Enqueue/discard decision correctness for all level combinations |

**Feasibility:** `should_enqueue` is a pure function over two `LogLevel` enum variants
(5 variants each). The input space is fully bounded at 25 pairs. This is structurally
identical to VP-027 (alert dedup key correctness — pure enum comparison) and VP-047
(UUID v7 validation — pure predicate). Proptest can exhaustively cover all 25 combinations
within milliseconds. Extraction of `should_enqueue` as a standalone pure function is
required by BC-2.20.002 Invariants: "min_level filtering is a pure function of
(entry.level, destination.min_level) — no state accumulates for filtered entries."

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_mcp::log_forward::filter::should_enqueue
//
// use proptest::prelude::*;
// use prism_mcp::log_forward::filter::{should_enqueue, LogLevel};
//
// proptest! {
//     /// Exhaustive 5×5 level matrix
//     /// All 25 (entry_level, min_level) pairs must match level_rank ordering
//     #[test]
//     fn verify_level_filter_exhaustive(
//         entry_level in prop_oneof![
//             Just(LogLevel::Trace),
//             Just(LogLevel::Debug),
//             Just(LogLevel::Info),
//             Just(LogLevel::Warn),
//             Just(LogLevel::Error),
//         ],
//         min_level in prop_oneof![
//             Just(LogLevel::Trace),
//             Just(LogLevel::Debug),
//             Just(LogLevel::Info),
//             Just(LogLevel::Warn),
//             Just(LogLevel::Error),
//         ],
//     ) {
//         let result = should_enqueue(entry_level, min_level);
//         let expected = (entry_level as u8) >= (min_level as u8);
//         prop_assert_eq!(result, expected,
//             "should_enqueue({:?}, {:?}) should be {}", entry_level, min_level, expected);
//     }
//
//     /// Default min_level == Info when omitted
//     /// Entry at Debug should be discarded; entry at Info should be enqueued
//     #[test]
//     fn verify_default_min_level_is_info(
//         entry_level in prop_oneof![
//             Just(LogLevel::Trace),
//             Just(LogLevel::Debug),
//             Just(LogLevel::Info),
//             Just(LogLevel::Warn),
//             Just(LogLevel::Error),
//         ],
//     ) {
//         let default_min = LogLevel::default(); // must be Info
//         prop_assert_eq!(default_min, LogLevel::Info, "Default must be Info");
//         let result = should_enqueue(entry_level, default_min);
//         let expected = (entry_level as u8) >= (LogLevel::Info as u8);
//         prop_assert_eq!(result, expected);
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 5×5 discrete enum space = 25 pairs; fully exhaustible |
| Tool support? | Full | Pure enum comparison with no async or I/O; proptest handles all pairs |
| Execution time budget | <5 seconds | 25 combinations × 10,000 each; trivial runtime |
| Assumptions required | `should_enqueue` extracted as a pure function in `crates/prism-mcp/src/log_forward/filter.rs` | Required by BC-2.20.002 invariant: "filtering is a pure function" |
| Edge cases | `min_level = Trace` (all pass), `min_level = Error` (only Error passes), boundary at each level | Covered by exhaustive matrix |

## Success Criteria

- All 25 (entry_level, min_level) combinations return the expected boolean
- `LogLevel::default()` returns `Info`
- No panic / unwrap / divergence in the function body
- Boundary cases: `entry_level == min_level` always returns `true` (at-threshold enqueued)

## Dependencies

- Depends on `should_enqueue()` being extracted as a pure function in `crates/prism-mcp/src/log_forward/filter.rs`
- Depends on `LogLevel` enum implementing `PartialOrd` with rank `Trace < Debug < Info < Warn < Error`
- Depends on `LogLevel::default() == Info`

## Lifecycle

| Field | Value |
|-------|-------|
| Status | draft |
| Introduced | cycle-1 / pass-81-remediation |
| Modified | null |
| Withdrawal Reason | null |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | pass-81-remediation | 2026-04-21 | architect | Initial draft. Closes BC-2.20.002 VP-TBD-20-002 per F81-009. Pure level-rank filter function is proptest-amenable (5×5 bounded input space). |
