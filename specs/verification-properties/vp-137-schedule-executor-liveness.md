---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:00:00Z
phase: 4-W4-Phase1-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
  - .factory/STATE.md
input-hash: "d498d85"
traces_to: .factory/specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
source_bc: null
source_adr: ADR-013
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-1-adr
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

# VP-137: Schedule Executor Liveness Under Per-Subsystem Semaphore

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-013` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-013 transitions to Accepted and produces a concrete BC in Phase 2.

> **[STUB — full VP authoring deferred to Phase 2 of Wave 4]**

## Property Statement

The schedule executor must make forward progress — executing at least one
scheduled task per subsystem within the configured execution window — even when
all available executor slots for other subsystems are concurrently occupied.
This liveness property is enforced by the per-subsystem semaphore design
mandated in ADR-013 §3 (Decision D-209): each subsystem receives an independent
counting semaphore sized to its concurrency budget, preventing starvation of
any single subsystem by noisy neighbors occupying the global thread pool. A
proptest round-trip across randomized multi-subsystem schedule configurations
must demonstrate that at least one permit per subsystem is always acquirable
within the deadline window.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-013 — Schedule Execution Semantics, §3 Per-Subsystem Semaphore
- **Decision Reference:** D-209 — per-subsystem semaphore (independent counting semaphore per subsystem)
- **Postcondition/Invariant:** Each subsystem's semaphore is independently acquirable; no subsystem's permits can be exhausted by activity in another subsystem.
- **BC:** To be assigned when ADR-013 is Accepted and BC authoring completes in Wave 4 Phase 2.
- **Module:** prism-operations
- **Category:** Liveness / Concurrency

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized multi-subsystem configurations | Per-subsystem semaphore acquire under concurrent load |

**Feasibility:** The semaphore budget per subsystem is bounded and small; proptest
can generate randomized occupancy scenarios and verify that each subsystem's
semaphore independently satisfies the liveness condition.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::scheduler::ScheduleExecutor
//
// proptest! {
//     #[test]
//     fn schedule_executor_liveness(config in arb_subsystem_config()) {
//         let executor = ScheduleExecutor::new(config);
//         // Saturate all but one slot per subsystem
//         // Assert: remaining slot is acquirable within deadline
//         prop_assert!(executor.acquire_permit(subsystem).is_ok());
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Semaphore budget per subsystem is small and finite; proptest can enumerate all meaningful occupancy configurations |
| Proof complexity | Low | Single acquire-under-load assertion per subsystem |
| Tool support | Full | proptest 1.x handles concurrent config generation |
| Estimated proof time | <30 seconds | Small state space; no symbolic execution required |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase1-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-013 §3 D-209. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 2. |
