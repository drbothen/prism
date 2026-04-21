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
  - specs/behavioral-contracts/BC-2.18.001
input-hash: "8a1948c"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.18.001
module: prism-operations
priority: P0
proof_method: kani
verification_method: kani
feasibility: conditional
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

# VP-044: Action Retry State Machine — Bounded by 5 Attempts, Dead-Letter Terminal

## Property Statement

The action delivery retry state machine never permits an attempt counter exceeding 5.
After the 5th consecutive delivery failure, the state machine transitions exactly once
to `DeadLettered`, which is a terminal state: no further `Pending` transitions are
possible from `DeadLettered`. The attempt counter is strictly bounded and the
dead-letter transition fires exactly once.

## Source Contract

- **Anchor Story:** `S-4.08`
- **Source BC:** BC-2.18.001 — At-Least-Once Delivery with Retry
- **Module:** prism-operations
- **Category:** Safety-Critical / Delivery Guarantee

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — bounded by MAX_ATTEMPTS=5, symbolic DeliveryOutcome | All state transition paths within attempt bound |

**Feasibility caveat:** The proof targets a pure `advance_retry_state(current: RetryState, outcome: DeliveryOutcome) -> RetryState` function extracted from the retry loop. If the implementation does not provide this extraction (i.e., retry logic is tightly coupled to tokio task spawning), the Phase 3 story for S-4.08 must explicitly extract it to enable the Kani proof.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_operations::action::advance_retry_state
//
// State machine:
//   Pending(attempt: u32)  -- 0-indexed, range 0..=4
//   DeadLettered           -- terminal
//
// Sketch:
// #[kani::proof]
// fn verify_retry_state_bounded() {
//     let attempt: u32 = kani::any();
//     kani::assume(attempt <= 5);
//     let current = if attempt < 5 { RetryState::Pending(attempt) } else { RetryState::DeadLettered };
//     let outcome: DeliveryOutcome = kani::any();
//
//     let next = advance_retry_state(current, outcome);
//
//     match (current, outcome, next) {
//         (RetryState::Pending(n), DeliveryOutcome::Failure, RetryState::Pending(m)) => {
//             kani::assert(n < 4, "Pending(n) + Failure -> Pending only when n < 4");
//             kani::assert(m == n + 1, "attempt counter increments by 1");
//         }
//         (RetryState::Pending(4), DeliveryOutcome::Failure, RetryState::DeadLettered) => {
//             // Correct: 5th failure (0-indexed 4) triggers dead-letter
//         }
//         (RetryState::DeadLettered, _, next_state) => {
//             kani::assert(matches!(next_state, RetryState::DeadLettered),
//                 "DeadLettered is terminal; no further transitions");
//         }
//         (RetryState::Pending(_), DeliveryOutcome::Success, RetryState::Completed) => {
//             // Correct: success transitions to Completed (terminal)
//         }
//         _ => kani::assert(false, "unexpected state transition"),
//     }
//
//     // Invariant: attempt counter never exceeds 5
//     if let RetryState::Pending(n) = next { kani::assert(n <= 4); }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | attempt range 0..=5, two symbolic DeliveryOutcome variants |
| Tool support? | Full (conditional on pure function extraction) | Kani handles small bounded state machines in <5 minutes |
| Execution time budget | <5 minutes | Tiny state space; 3 states x 2 outcomes |
| Assumptions required | `advance_retry_state` is extracted as a pure function in Phase 3 S-4.08 story | Documented constraint for story author |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.18.001. P0 because retry bound is a safety-critical delivery guarantee. Persistence (RocksDB write) remains integration test. |
