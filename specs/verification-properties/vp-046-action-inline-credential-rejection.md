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
  - specs/behavioral-contracts/BC-2.18.007
input-hash: "1c4e1398c8650635ccbc893e784f53b4"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.18.007
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

# VP-046: Action Inline Credential Rejected at Load Time; Value Not in Error Message

## Property Statement

For any `ActionSpec` where a credential field contains an inline string value (non-reference
form), `validate_credential_fields(spec)` returns `Err(E-ACTION-001)`. The error message
contains the field name but does not contain the field value. For ActionSpec instances where
all credential fields use the reference form `{ source = "...", key = "..." }`, the function
returns `Ok(())`.

## Source Contract

- **Anchor Story:** `S-4.08`
- **Source BC:** BC-2.18.007 — Action Credential Opaque Reference
- **Module:** prism-operations
- **Category:** Security / Credential Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — generates ActionSpec with arbitrary inline vs. reference credential fields | All combinations of inline and reference credential field values |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::action::validate_credential_fields
//
// Sketch:
// proptest!(|(spec in arb_action_spec_with_credential_variant())| {
//     let result = validate_credential_fields(&spec);
//
//     for (field_name, field_value) in spec.credential_fields() {
//         match field_value {
//             CredentialField::Inline(value) => {
//                 // Inline value must produce E-ACTION-001
//                 prop_assert!(result.is_err(),
//                     "inline credential in field '{}' must produce error", field_name);
//                 let err_msg = format!("{}", result.as_ref().unwrap_err());
//                 // Error message must contain field name
//                 prop_assert!(err_msg.contains(field_name),
//                     "error message must contain field name '{}'", field_name);
//                 // Error message must NOT contain the inline value (prevent credential logging)
//                 prop_assert!(!err_msg.contains(value.as_str()),
//                     "error message must not contain credential value for field '{}'", field_name);
//             }
//             CredentialField::Reference { source, key } => {
//                 // Reference form is valid; function should continue (or return Ok)
//                 // (only error if another field is inline)
//             }
//         }
//     }
//
//     // If all fields are references, result must be Ok
//     if spec.credential_fields().all(|(_, v)| matches!(v, CredentialField::Reference { .. })) {
//         prop_assert!(result.is_ok(), "all-reference spec must return Ok");
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | proptest generates arbitrary ActionSpec with credential field variants |
| Tool support? | Full | proptest + structured input generation; security property (value absent from error) is directly assertable |
| Execution time budget | <60 seconds for 10k cases | Pure validation function; no I/O |
| Assumptions required | `validate_credential_fields` is a pure function taking `&ActionSpec` and returning `Result<(), ActionError>` | Must be extractable from any async loading context |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.18.007. P0 security property: inline credential values must never appear in error output. Credential-value-in-log half covered transitively by this VP plus BC-2.05.003 architecture policy. |
