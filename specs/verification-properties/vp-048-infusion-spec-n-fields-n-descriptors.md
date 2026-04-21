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
  - specs/behavioral-contracts/BC-2.19.001
input-hash: "3eb97f3"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.19.001
module: prism-spec-engine
priority: P1
proof_method: kani
verification_method: kani
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

# VP-048: Infusion Spec — N Fields Produces Exactly N UDF Descriptors; Duplicates Error

## Property Statement

`InfusionRegistry::load_spec(spec: &InfusionSpec)` with N valid, distinct field entries
produces exactly N `InfusionUdfDescriptor` objects in the output vector when the call
succeeds. If any two field entries share a UDF name, the function returns
`Err(E-INFUSE-002)` rather than silently merging duplicates. The output count invariant
holds for all N in the range covered by the proof bound.

## Source Contract

- **Anchor Story:** `S-1.14`
- **Source BC:** BC-2.19.001 — Infusion Spec Loading
- **Module:** prism-spec-engine
- **Category:** Correctness / INV-INFUSE-001

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — bounded N (e.g. N <= 16 for proof, generalizes) | All distinct-name cases + at-least-one-duplicate cases |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_spec_engine::infusion::InfusionRegistry::load_spec
//
// #[kani::proof]
// fn verify_n_fields_n_descriptors() {
//     // Symbolic field count N in 1..=16
//     let n: usize = kani::any();
//     kani::assume(n >= 1 && n <= 16);
//
//     // Generate N distinct field entries (symbolic names, all different)
//     let spec = build_symbolic_infusion_spec_distinct(n);
//
//     let result = InfusionRegistry::load_spec(&spec);
//
//     match result {
//         Ok(descriptors) => {
//             kani::assert(descriptors.len() == n,
//                 "N distinct fields must produce exactly N descriptors");
//         }
//         Err(_) => {
//             kani::assert(false, "distinct fields must not produce an error");
//         }
//     }
// }
//
// #[kani::proof]
// fn verify_duplicate_udf_name_errors() {
//     // Generate spec with at least one duplicate UDF name
//     let spec = build_symbolic_infusion_spec_with_duplicate();
//     let result = InfusionRegistry::load_spec(&spec);
//     kani::assert(result.is_err(), "duplicate UDF name must produce Err");
//     if let Err(e) = result {
//         kani::assert(matches!(e, InfusionError::DuplicateUdfName(_)),
//             "error must be E-INFUSE-002 / DuplicateUdfName");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | N <= 16 for Kani; structural induction argument generalizes |
| Tool support? | Full | Kani handles Vec length assertions with bounded inputs |
| Execution time budget | <15 minutes | Small symbolic N; load_spec is a pure loop |
| Assumptions required | `build_symbolic_infusion_spec_distinct(n)` generates a spec where all field names are provably distinct to Kani's symbolic engine | Custom symbolic builder required in harness |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.19.001. Proves INV-INFUSE-001 (N fields -> N descriptors) and duplicate detection. Two separate Kani proof harnesses. |
