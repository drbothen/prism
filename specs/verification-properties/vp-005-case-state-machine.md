---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, architecture/module-decomposition.md]
input-hash: "1f68e1c"
traces_to: prd.md
source_bc: BC-2.14.002
module: prism-core
priority: P0
proof_method: kani
verification_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
removal_reason: null
removed: null
withdrawal_reason: null
---

# VP-005: Case State Machine — Exactly 12 Valid Transitions

## Property Statement

The function `CaseStatus::can_transition_to(target)` returns `true` for exactly 12 `(current, target)` pairs out of the 25 possible pairs (5 statuses x 5 statuses). Specifically:

**Forward linear (4):** New->Acknowledged, Acknowledged->Investigating, Investigating->Resolved, Resolved->Closed
**Skip-ahead (6):** New->Investigating, New->Resolved, New->Closed, Acknowledged->Resolved, Acknowledged->Closed, Investigating->Closed
**Reopen (2):** Resolved->Investigating, Closed->Investigating

All 13 other pairs (including all self-transitions) return `false`.

## Source Contract

- **BC:** BC-2.14.002 — Case State Transitions — 5-State Machine, 12 Valid Transitions
- **Invariant:** DI-025 — Case State Transition Validity

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | No — exhaustive (5x5 = 25 pairs) | 100% of state space |

## Proof Harness Skeleton

```rust
#[kani::proof]
fn verify_case_transitions_exhaustive() {
    let current: CaseStatus = kani::any();
    let target: CaseStatus = kani::any();
    let allowed = current.can_transition_to(target);

    // Count valid transitions
    let expected = matches!(
        (current, target),
        // Forward linear
        (CaseStatus::New, CaseStatus::Acknowledged) |
        (CaseStatus::Acknowledged, CaseStatus::Investigating) |
        (CaseStatus::Investigating, CaseStatus::Resolved) |
        (CaseStatus::Resolved, CaseStatus::Closed) |
        // Skip-ahead
        (CaseStatus::New, CaseStatus::Investigating) |
        (CaseStatus::New, CaseStatus::Resolved) |
        (CaseStatus::New, CaseStatus::Closed) |
        (CaseStatus::Acknowledged, CaseStatus::Resolved) |
        (CaseStatus::Acknowledged, CaseStatus::Closed) |
        (CaseStatus::Investigating, CaseStatus::Closed) |
        // Reopen
        (CaseStatus::Resolved, CaseStatus::Investigating) |
        (CaseStatus::Closed, CaseStatus::Investigating)
    );
    assert_eq!(allowed, expected);
}

#[kani::proof]
fn verify_no_self_transitions() {
    let status: CaseStatus = kani::any();
    assert!(!status.can_transition_to(status));
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Tiny (25 pairs) | Exhaustive enumeration trivial |
| Proof complexity | Low | Pure enum matching |
| Tool support | Full | Kani handles enum types natively |
| Estimated proof time | <10 seconds | |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-04-15 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
