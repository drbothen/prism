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
  - specs/behavioral-contracts/BC-2.18.004
input-hash: "3ff257e"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.18.004
module: prism-operations
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

# VP-045: Schedule Semaphore — try_acquire Used (Non-Blocking), Never acquire

## Property Statement

When all 16 concurrency semaphore permits in `ActionEngine` are held by other tasks,
`ActionEngine::fire_schedule()` returns immediately (within 10ms) without acquiring a
permit. The function never blocks or awaits on a blocking semaphore `acquire()`. This
prevents deadlock in the cron tick loop: a saturated executor must skip the tick, not
stall indefinitely.

## Source Contract

- **Anchor Story:** `S-4.08`
- **Source BC:** BC-2.18.004 — Schedule Semaphore try_acquire
- **Module:** prism-operations
- **Category:** Safety-Critical / Deadlock Prevention

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — hold all permits, invoke fire_schedule, measure return time | Saturation scenario: all 16 permits held |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::action::ActionEngine::fire_schedule
//
// Sketch:
// proptest!(|(schedule_id in arb_schedule_id())| {
//     let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
//     rt.block_on(async {
//         let semaphore = Arc::new(Semaphore::new(16));
//         // Hold all 16 permits
//         let _permits: Vec<_> = (0..16)
//             .map(|_| semaphore.try_acquire().unwrap())
//             .collect();
//
//         let engine = ActionEngine::new_with_semaphore(semaphore.clone());
//         let start = std::time::Instant::now();
//         let result = engine.fire_schedule(schedule_id).await;
//         let elapsed = start.elapsed();
//
//         // Must return immediately — not block waiting for a permit
//         prop_assert!(elapsed.as_millis() < 10,
//             "fire_schedule must return in <10ms when all permits held, got {}ms",
//             elapsed.as_millis());
//         prop_assert!(matches!(result, Err(ActionError::ConcurrencyLimitReached)),
//             "saturated semaphore must return ConcurrencyLimitReached, not block");
//     });
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | schedule_id is arbitrary; key variable is semaphore saturation which is controlled |
| Tool support? | Full | proptest + tokio test runtime; time assertion is straightforward |
| Execution time budget | <60 seconds for 1000 cases | Each iteration is fast: try_acquire returns immediately |
| Assumptions required | ActionEngine accepts injected Semaphore for testability | Standard dependency-injection pattern |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.18.004. P0 because blocking acquire() in cron tick loop causes deadlock (safety-critical failure mode FM-018). |
