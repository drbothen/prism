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
  - specs/behavioral-contracts/BC-2.16.009
input-hash: "1e29f9d"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.16.009
module: prism-spec-engine
priority: P1
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

# VP-059: Spec Validator — All Errors Collected, No Fail-Fast

## Property Statement

For any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec(spec)`
returns `Err(errors)` where `errors.len() == N`. The validator never returns early on the first
error; all validation rules are applied and all errors are collected before returning. For a
spec with only warnings and no errors, `validate_sensor_spec(spec)` returns `Ok(warnings)`
(the spec is accepted with warnings attached). The function is deterministic: same input
always produces the same output.

## Source Contract

- **Anchor Story:** `S-1.11`
- **Source BC:** BC-2.16.009 — Spec File Validation
- **Module:** prism-spec-engine
- **Category:** Developer Experience / Validation Completeness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — N validation errors 1..=10; warning-only specs | All error-count values and warning-only path |

**Feasibility:** `validate_sensor_spec` is a pure function: `SensorSpec -> ValidationResult`.
A proptest can generate specs with exactly N injected validation errors (e.g., invalid sensor_id
+ forward variable reference + duplicate column names) and verify all N errors appear in the
output. The warning-only path is a separate proptest variant. No I/O or side effects in the
validation function.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::validation::validate_sensor_spec
//
// proptest! {
//     #[test]
//     fn verify_all_errors_collected_no_fail_fast(
//         n_errors in 1usize..=10,
//     ) {
//         // Generate a SensorSpec with exactly n_errors distinct validation errors
//         // Each error comes from a different validation rule:
//         //   - invalid sensor_id (rule 1)
//         //   - forward variable reference (rule 2)
//         //   - duplicate column name (rule 3)
//         //   - missing required field (rule 4)
//         //   etc.
//         let spec = build_spec_with_n_errors(n_errors);
//
//         let result = validate_sensor_spec(&spec);
//         prop_assert!(result.is_err(), "spec with errors must return Err");
//
//         let errors = result.unwrap_err();
//         prop_assert_eq!(errors.len(), n_errors,
//             "all {} errors must be collected; got {}", n_errors, errors.len());
//     }
//
//     #[test]
//     fn verify_warning_only_spec_returns_ok(
//         n_warnings in 1usize..=5,
//     ) {
//         let spec = build_spec_with_n_warnings_no_errors(n_warnings);
//
//         let result = validate_sensor_spec(&spec);
//         prop_assert!(result.is_ok(), "warning-only spec must return Ok (spec accepted)");
//
//         let warnings = result.unwrap();
//         prop_assert_eq!(warnings.len(), n_warnings,
//             "all {} warnings must be returned in Ok variant", n_warnings);
//     }
//
//     #[test]
//     fn verify_deterministic(spec in arb_sensor_spec()) {
//         let result1 = validate_sensor_spec(&spec);
//         let result2 = validate_sensor_spec(&spec);
//         prop_assert_eq!(result1, result2, "validate_sensor_spec must be deterministic");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | N errors bounded 1..=10; SensorSpec is a bounded struct |
| Tool support? | Full | Pure function with no I/O or async |
| Execution time budget | <2 minutes | Validation rules are fast; proptest with small N runs quickly |
| Assumptions required | Ability to construct SensorSpec values with exactly N known validation errors | Requires `build_spec_with_n_errors` test helper in Phase 3 story for S-1.11 |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.16.009. Proves all-errors-collected (no fail-fast) and warning-only-spec returns Ok. Method: Proptest. P1. |
