---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.18.004
input-hash: "248b3b0"
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
modified: ["2026-05-04"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

> **Naming history note:** This VP's filename slug (`vp-045-schedule-semaphore-try-acquire-nonblocking`) preserves the original Wave 1 nomenclature when the property was anchored to a single shared semaphore. Per D-209 LOCKED (2026-05-02), the semaphore design split into two independent pools; this VP now anchors specifically to the `action_delivery_semaphore` (8-permit, prism-operations::action_delivery per ADR-016 §2.11). Filename slug preserved per POL-1 (append-only-numbering: filename slugs immutable).

# VP-045: Schedule Semaphore — try_acquire Used (Non-Blocking), Never acquire

## Property Statement

When all 8 permits in the `action_delivery_semaphore` (module-private to
`prism-operations::action_delivery` per ADR-016 §2.11 / D-209 LOCKED) are held by
other concurrent action deliveries, `ActionDeliveryEngine::fire()` returns immediately
(within 10ms) without acquiring a permit. The function never blocks or awaits on a
blocking semaphore `acquire()`. This prevents deadlock in the cron tick loop: a
saturated executor must skip the tick, not stall indefinitely.

## Source Contract

- **Anchor Story:** `S-4.08`
- **Source BC:** BC-2.18.004 — Action Delivery Semaphore — 8-Permit Independent Pool, try_acquire() Skip-If-Unavailable
- **Module:** prism-operations
- **Category:** Safety-Critical / Deadlock Prevention

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — hold all permits, invoke fire, measure return time | Saturation scenario: all 8 permits held |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::action_delivery::ActionDeliveryEngine::fire
//
// Sketch:
// proptest!(|(schedule_id in arb_schedule_id())| {
//     let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
//     rt.block_on(async {
//         let semaphore = Arc::new(Semaphore::new(8));
//         // Hold all 8 permits
//         let _permits: Vec<_> = (0..8)
//             .map(|_| semaphore.try_acquire().unwrap())
//             .collect();
//
//         let engine = ActionDeliveryEngine::new_with_semaphore(semaphore.clone());
//         let start = std::time::Instant::now();
//         let result = engine.fire(schedule_id).await;
//         let elapsed = start.elapsed();
//
//         // Must return immediately — not block waiting for a permit
//         prop_assert!(elapsed.as_millis() < 10,
//             "fire must return in <10ms when all permits held, got {}ms",
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
| Assumptions required | ActionDeliveryEngine accepts injected Semaphore for testability | Standard dependency-injection pattern |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | F-PreP27-H-001 | 2026-05-04 | product-owner | Pre-Pass-27 sibling-spec sweep: 3 orphan tokens `action_dispatcher` → `action_delivery` (line 37 banner note + line 44 Property Statement + line 68 Proof Harness target). Same drift class as F-P25-H-001 (PRD) + F-P26-H-001 (ADR-016) — orchestrator-prompt-introduced orphans across multiple bursts. |
| 1.2 | F-PreP22-H-004 | 2026-05-03 | product-owner | Pre-Pass-22 sweep: body content rewritten to reflect VP-045 current scope (action_delivery_semaphore 8-permit, ADR-016 §2.11 / D-209 LOCKED). Was completely outdated post-Pass-20 F-P20-H-001 cascade. Filename slug preserved per POL-1. Banner note added explaining slug-vs-content drift. |
| 1.1 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Schedule Semaphore try_acquire" → "Scheduled Report Queries — try_acquire() on 16-Permit Semaphore, Skip If Unavailable" (matches BC-2.18.004 H1). |
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.18.004. P0 because blocking acquire() in cron tick loop causes deadlock (safety-critical failure mode FM-018). |
