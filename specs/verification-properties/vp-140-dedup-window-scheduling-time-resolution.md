---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:30:00Z
phase: 4-W4-Phase2-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-015-detection-rule-language.md
  - .factory/STATE.md
input-hash: "1360731"
traces_to: .factory/specs/architecture/decisions/ADR-015-detection-rule-language.md
source_bc: null
source_adr: ADR-015
module: prism-operations
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: wave-4-phase-2-adr
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

# VP-140: Dedup Window Scheduling-Time Resolution and Invalidation Correctness

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-015` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-015 transitions to Accepted and produces a concrete BC in Phase 4.B.

> **[STUB — full VP authoring deferred to Phase 4.B BC authoring]**

## Property Statement

ADR-015 §5 specifies that the alert deduplication window is resolved at
scheduling-time and baked into `RuleCondition`, with Schedule CRUD operations
invalidating the cached resolution. This VP verifies that: (1) the resolved
dedup window value stored in `RuleCondition` matches the OrgRegistry value
at the time of schedule creation, and (2) any Schedule CRUD operation that
modifies the schedule correctly triggers invalidation of the cached resolution,
ensuring subsequent evaluations use a freshly resolved value rather than a
stale one.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-015 — Detection Rule Language, §5 Dedup Window Scheduling-Time Resolution
- **Decision Reference:** dedup-window scheduling-time resolution + invalidation correctness
- **Postcondition/Invariant:** RuleCondition.dedup_window == OrgRegistry value at schedule-creation time; Schedule CRUD invalidates cached resolution.
- **BC:** To be assigned when ADR-015 is Accepted and BC authoring completes in Wave 4 Phase 4.B.
- **Module:** prism-operations
- **Category:** Correctness / Scheduling

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized schedule CRUD sequences | Scheduling-time resolution bake-in + CRUD invalidation |

**Feasibility:** Schedule CRUD sequences are bounded and finite; proptest can
generate randomized create/update/delete sequences and verify resolution
freshness at each evaluation point.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::scheduler::RuleCondition
//
// proptest! {
//     #[test]
//     fn dedup_window_baked_at_schedule_creation(config in arb_org_registry_config()) {
//         let rule = RuleCondition::new(&config);
//         prop_assert_eq!(rule.dedup_window, config.dedup_window());
//     }
//
//     #[test]
//     fn schedule_crud_invalidates_dedup_resolution(ops in arb_schedule_crud_sequence()) {
//         // After any mutating CRUD op, cached resolution must be invalidated
//         prop_assert!(resolution_is_fresh_after_crud(&ops));
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Schedule CRUD sequences are finite; OrgRegistry config space is small |
| Proof complexity | Low | Two independent assertions: bake-in correctness + invalidation trigger |
| Tool support | Full | proptest 1.x handles sequential operation generation |
| Estimated proof time | <30 seconds | Small state space; no symbolic execution required |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase2-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-015 §5. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 4.B. |
