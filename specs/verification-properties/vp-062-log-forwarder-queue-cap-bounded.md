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
  - specs/behavioral-contracts/BC-2.20.003-log-forwarder-queue-cap.md
input-hash: "1e29f9d"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.20.003
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

# VP-062: Log Forwarder Queue Cap — Bounded Queue Never Exceeds 10 × batch_size

## Property Statement

Given a pure function `enqueue_with_cap(queue: &mut BoundedQueue, entry: LogEntry, batch_size: usize) -> EnqueueResult`,
where `BoundedQueue` has `cap = 10 * batch_size` and implements drop-oldest on overflow:

1. After any sequence of `enqueue_with_cap` calls, `queue.len() <= 10 * batch_size`.
2. If `queue.len() == cap` before an enqueue call, `queue.len() == cap` after (one drop for one enqueue).
3. If `queue.len() == cap` before an enqueue call, `drop_count` increases by exactly 1.
4. The function is total and terminates for all inputs.

The cap invariant holds regardless of `batch_size` value (1, 50, 100, 1000, or any positive usize).

## Source Contract

- **Anchor Story:** `S-5.09`
- **Source BC:** BC-2.20.003 — Log Forwarder Queue Cap
- **Module:** prism-mcp
- **Category:** Correctness / Memory Safety / Log Forwarding

## Scope

**IN SCOPE:** Pure bounded-queue operations: `enqueue_with_cap(queue, entry, batch_size)`.
The function operates on a `BoundedQueue` struct with `Vec<LogEntry>` storage and
`drop_count: u64` counter. No I/O, no networking, no async.

**OUT OF SCOPE:** The WARN emission to the local sink on overflow (effectful I/O — tested
by integration test TV-20-003-cap). The delivery flush path (separate effectful operation).
Concurrent access (each destination has an independent queue; concurrency is a Tokio
task isolation concern per BC-2.20.005).

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — sequence of N enqueue operations with varying batch_size and N | Queue length bounded invariant, drop-count accuracy, exactly-one-drop-per-overflow |

**Feasibility:** `BoundedQueue` is a pure data structure (Vec-backed with drop_oldest policy).
The cap invariant `len <= 10 * batch_size` is a bounded numeric property over a finite
struct. This is structurally similar to VP-010 (token cap: store rejects at 100 active —
pure count boundary) and VP-030 (schedule/rule count caps — pure numeric invariant).
Proptest can generate sequences of enqueue calls across varied batch_size values and
assert the invariant after each call. Extraction of `enqueue_with_cap` as a pure function
is required by BC-2.20.003 Invariants: "Queue size never exceeds 10 × batch_size" and
"Drop-oldest is the ONLY overflow strategy."

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_mcp::log_forward::queue::BoundedQueue
//
// use proptest::prelude::*;
// use prism_mcp::log_forward::queue::{BoundedQueue, EnqueueResult};
//
// proptest! {
//     /// Invariant 1: After N enqueue calls, queue.len() <= 10 * batch_size
//     #[test]
//     fn verify_cap_invariant_holds_after_any_sequence(
//         batch_size in 1usize..=200,
//         n_entries in 0usize..=3000,
//     ) {
//         let mut queue = BoundedQueue::new(batch_size);
//         let cap = 10 * batch_size;
//         for i in 0..n_entries {
//             let entry = make_log_entry(i as u64);
//             queue.enqueue(entry);
//             prop_assert!(queue.len() <= cap,
//                 "queue.len()={} exceeded cap={} after {} enqueues (batch_size={})",
//                 queue.len(), cap, i + 1, batch_size);
//         }
//     }
//
//     /// Invariant 2+3: At-cap enqueue → len unchanged, drop_count +1
//     #[test]
//     fn verify_drop_oldest_at_cap(
//         batch_size in 1usize..=200,
//     ) {
//         let cap = 10 * batch_size;
//         let mut queue = BoundedQueue::new(batch_size);
//         // Fill to cap
//         for i in 0..cap {
//             queue.enqueue(make_log_entry(i as u64));
//         }
//         prop_assert_eq!(queue.len(), cap);
//         let drop_count_before = queue.drop_count();
//
//         // One more enqueue at cap
//         queue.enqueue(make_log_entry(999_999));
//
//         prop_assert_eq!(queue.len(), cap, "len must equal cap after overflow enqueue");
//         prop_assert_eq!(queue.drop_count(), drop_count_before + 1,
//             "drop_count must increment by exactly 1 per overflow");
//     }
//
//     /// Invariant 4: Under-cap enqueue — no drop, len increases by 1
//     #[test]
//     fn verify_under_cap_enqueue_no_drop(
//         batch_size in 1usize..=200,
//         pre_fill in 0usize..=9,
//     ) {
//         let mut queue = BoundedQueue::new(batch_size);
//         // Fill to less than cap
//         let fill_count = pre_fill.min(10 * batch_size - 1);
//         for i in 0..fill_count {
//             queue.enqueue(make_log_entry(i as u64));
//         }
//         prop_assert!(queue.len() < 10 * batch_size);
//         let len_before = queue.len();
//         let drops_before = queue.drop_count();
//
//         queue.enqueue(make_log_entry(999));
//
//         prop_assert_eq!(queue.len(), len_before + 1);
//         prop_assert_eq!(queue.drop_count(), drops_before, "no drop when under cap");
//     }
// }
//
// fn make_log_entry(id: u64) -> LogEntry {
//     LogEntry { id, level: LogLevel::Info, message: format!("entry-{}", id) }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | `batch_size` ∈ [1, 200]; sequence length ∈ [0, 3000]; cap = 10 * batch_size; finite state space |
| Tool support? | Full | Pure Vec-backed struct with no async or I/O; proptest generates sequences |
| Execution time budget | <30 seconds | Three families × 10,000 cases; linear in N per case |
| Assumptions required | `BoundedQueue::new(batch_size)` and `BoundedQueue::enqueue()` extracted as pure operations | Required by BC-2.20.003 invariant structure |
| Edge cases | `batch_size=1` (cap=10), `batch_size=200` (cap=2000), fill exactly to cap-1 then overflow | Covered by proptest ranges |

## Success Criteria

- `queue.len() <= 10 * batch_size` holds after every enqueue in any sequence
- `drop_count` increments by exactly 1 on each overflow enqueue
- Under-cap enqueue increases `len` by 1 with `drop_count` unchanged
- No panic / unwrap / divergence (function is total)

## Dependencies

- Depends on `BoundedQueue` being implemented as a pure data structure in `crates/prism-mcp/src/log_forward/queue.rs`
- Depends on `BoundedQueue::new(batch_size)`, `BoundedQueue::enqueue()`, `BoundedQueue::len()`, `BoundedQueue::drop_count()` being pure (no async, no I/O)
- WARN emission to local sink is NOT part of this pure function (effectful; verified by integration test TV-20-003-cap)

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
| 1.0 | pass-81-remediation | 2026-04-21 | architect | Initial draft. Closes BC-2.20.003 VP-TBD-20-003 per F81-009. Bounded queue cap invariant is proptest-amenable (pure Vec struct, numeric boundary). WARN emission remains integration-test only (effectful I/O). |
