---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:50:00Z
phase: 4-W4-Phase3-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-016-action-delivery-framework.md
  - .factory/STATE.md
input-hash: "7dd885d"
traces_to: .factory/specs/architecture/decisions/ADR-016-action-delivery-framework.md
source_bc: null
source_adr: ADR-016
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-3-adr
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

# VP-143: Action Delivery Non-Starvation Under Per-Subsystem Semaphore

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-016` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-016 transitions to Accepted and produces a concrete BC in Phase 2.

> **[STUB — full VP authoring deferred to Phase 2 of Wave 4]**

## Property Statement

The action delivery subsystem must make forward progress — dispatching at least
one queued action per registered client within the configured delivery window —
even when the per-subsystem semaphore for other operations subsystems is
concurrently saturated. This non-starvation property is enforced by the
per-subsystem semaphore design mandated in ADR-016 §11 (action delivery
semaphore, symmetric pair to the schedule executor semaphore in ADR-013 §3
D-209): the action delivery semaphore is independent from the schedule
executor semaphore, preventing action delivery from being blocked by a noisy
schedule execution burst. A proptest round-trip across randomized concurrent
delivery and scheduling workload configurations must demonstrate that at least
one delivery permit is acquirable by the action delivery subsystem within the
deadline window regardless of scheduler load.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-016 — Action Delivery Framework, §11 Semaphore Budget and Non-Starvation
- **Decision Reference:** D-209 (transitive) — per-subsystem semaphore; ADR-016 §11 applies symmetric design to action delivery
- **Postcondition/Invariant:** The action delivery semaphore is independently acquirable; no permits can be exhausted by concurrent schedule executor activity.
- **BC:** To be assigned when ADR-016 is Accepted and BC authoring completes in Wave 4 Phase 2.
- **Module:** prism-operations
- **Category:** Liveness / Concurrency

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized concurrent scheduler + delivery workload configurations | Per-subsystem semaphore acquire under cross-subsystem concurrent load |

**Feasibility:** The semaphore budget for the action delivery subsystem is bounded and small; proptest can generate randomized scenarios saturating the schedule executor semaphore and verify that delivery permits remain independently acquirable.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::action_delivery::ActionDeliverySubsystem
//
// proptest! {
//     #[test]
//     fn action_delivery_non_starvation(config in arb_concurrent_workload()) {
//         let delivery = ActionDeliverySubsystem::new(config.delivery_budget);
//         let scheduler = ScheduleExecutor::new(config.schedule_budget);
//         // Saturate scheduler semaphore
//         // Assert: delivery permit remains acquirable
//         prop_assert!(delivery.acquire_permit(client_id).is_ok());
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Semaphore budget per subsystem is small and finite; proptest can enumerate meaningful cross-subsystem occupancy configurations |
| Proof complexity | Low | Single acquire-under-cross-subsystem-load assertion |
| Tool support | Full | proptest 1.x handles concurrent config generation |
| Estimated proof time | <30 seconds | Small state space; symmetric to VP-137 |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase3-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-016 §11. source_bc null pending ADR acceptance + BC authoring. Symmetric pair to VP-137 (schedule executor liveness). Full harness deferred to Wave 4 Phase 2. |
