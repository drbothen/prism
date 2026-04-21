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
  - specs/behavioral-contracts/BC-2.18.009
input-hash: "7766569"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.18.009
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

# VP-047: UUID v7 Validation — Non-v7 Always Rejected, v7 Always Accepted, Order Preserved

## Property Statement

`validate_uuid_v7(s: &str) -> bool` returns `true` only for strings that parse as
valid version-7 UUIDs. It returns `false` for all UUID v1, v4, v6 strings, all
non-UUID strings (including SQL injection payloads), and empty strings. For any input
sequence of valid UUID v7 strings, the output sequence preserves input order.

## Source Contract

- **Anchor Story:** `S-4.08`
- **Source BC:** BC-2.18.009 — UUID v7 Validation for alert_ids_quoted
- **Module:** prism-operations
- **Category:** Correctness / Input Validation

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — generates valid v7 UUIDs, other UUID versions, non-UUID strings, injection payloads, empty strings | All UUID version variants and adversarial non-UUID inputs |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::validation::validate_uuid_v7
//
// Three sub-properties:
//
// 1. Valid v7 always accepted
// proptest!(|(uuid_v7 in arb_uuid_v7())| {
//     prop_assert!(validate_uuid_v7(&uuid_v7.to_string()),
//         "valid UUID v7 must be accepted");
// });
//
// 2. Non-v7 always rejected (covers v1, v4, v6, non-UUID, injection, empty)
// proptest!(|(non_v7 in arb_non_v7_uuid_string())| {
//     prop_assert!(!validate_uuid_v7(&non_v7),
//         "non-v7 input '{}' must be rejected", non_v7);
// });
//
// 3. Order preservation on valid sequence
// proptest!(|(ids in prop::collection::vec(arb_uuid_v7(), 1..100))| {
//     let input_order: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
//     let validated: Vec<String> = input_order.iter()
//         .filter(|s| validate_uuid_v7(s))
//         .cloned()
//         .collect();
//     prop_assert_eq!(&validated, &input_order,
//         "output order must match input order for all-valid sequences");
// });
//
// arb_non_v7_uuid_string() generates: Uuid::new_v4(), Uuid::now_v6(), Uuid::new_v1(),
// arbitrary &str including SQL injection payloads ("' OR '1'='1", "DROP TABLE"),
// empty string "".
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | proptest explores UUID string space including adversarial inputs |
| Tool support? | Full | proptest + uuid crate generators; pure predicate function is ideal proptest target |
| Execution time budget | <30 seconds for 10k cases | String parsing is cheap |
| Assumptions required | arb_uuid_v7() produces all valid v7 UUID formats; arb_non_v7_uuid_string() covers all other UUID versions plus adversarial strings | Standard uuid crate test helpers |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.18.009. Covers three sub-properties: v7 always accepted, non-v7 always rejected (all UUID versions + injection payloads + empty), order preserved. |
