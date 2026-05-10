---
document_type: verification-property
level: L4
version: "0.1"
status: draft
producer: architect
timestamp: 2026-05-02T20:00:00Z
phase: 4-W4-Phase1-ADR
inputs:
  - .factory/specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md
  - .factory/STATE.md
input-hash: "ac49635"
traces_to: .factory/specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md
source_bc: null
source_adr: ADR-017
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

# VP-138: Case Cross-Org Access Always Denied

> **NOTE:** This VP traces to an ADR, not a BC. `source_bc: null` and
> `source_adr: ADR-017` are used because the primary constraint originates
> in the architecture decision record. A BC anchor will be added when
> ADR-017 transitions to Accepted and produces a concrete BC in Phase 2.

> **[STUB — full VP authoring deferred to Phase 2 of Wave 4]**

## Property Statement

For any case access request where the requesting `OrgId` differs from the
`OrgId` embedded in the `CaseId`, the access must be denied — returning an
authorization error — regardless of the caller's role or permission set within
their own organization. This cross-org isolation invariant is specified in
ADR-017 §6 INV-CASE-003: a `CaseRecord` is owned by exactly one `OrgId`, and
all read, write, and state-transition operations that receive a mismatched
`OrgId` must fail with `Err(E-CASE-AUTH-001)` before any state is read or
mutated. A proptest round-trip across randomized `(requester_org, case_owner_org)`
pairs where `requester_org != case_owner_org` must confirm that no access path
succeeds.

## Source Contract

> **ADR-sourced stub — BC not yet assigned.**

- **ADR:** ADR-017 — Case Lifecycle Invariants, §6
- **Invariant Reference:** INV-CASE-003 — cross-org case access denied
- **Postcondition/Invariant:** Any operation on a `CaseRecord` with `requester_org != case.org_id` returns `Err(E-CASE-AUTH-001)`; no state is read or mutated.
- **BC:** To be assigned when ADR-017 is Accepted and BC authoring completes in Wave 4 Phase 2.
- **Module:** prism-operations
- **Category:** Authorization / Isolation Invariant

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest 1.x | Yes — randomized (requester_org, case_owner_org) pairs | All cross-org access paths (read, write, state-transition) |

**Feasibility:** `OrgId` is a UUID v7; proptest can generate arbitrary
`(requester_org, case_owner_org)` pairs and filter to the `!=` subset.
The access-check function is a pure guard — no external I/O — making
round-trip property testing tractable.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::case::access_check (or equivalent guard function)
//
// proptest! {
//     #[test]
//     fn cross_org_access_denied(
//         requester_org in arb_org_id(),
//         case_owner_org in arb_org_id(),
//     ) {
//         prop_assume!(requester_org != case_owner_org);
//         let result = access_check(&requester_org, &case_owner_org);
//         prop_assert!(matches!(result, Err(CaseError::Unauthorized)));
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | OrgId is UUID v7; proptest generates arbitrary pairs filtered to cross-org subset |
| Proof complexity | Low | Single guard function call; pure — no I/O |
| Tool support | Full | proptest 1.x; no symbolic execution required |
| Estimated proof time | <15 seconds | Trivial assertion; fast filtering via prop_assume! |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-02 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | W4-Phase1-ADR-commit | 2026-05-02 | architect | Initial stub. Traces to ADR-017 §6 INV-CASE-003. source_bc null pending ADR acceptance + BC authoring. Full harness deferred to Wave 4 Phase 2. |
