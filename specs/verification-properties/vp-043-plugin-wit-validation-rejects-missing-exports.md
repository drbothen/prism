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
  - specs/behavioral-contracts/BC-2.17.006
input-hash: "[pending-recompute]"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.17.006
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

# VP-043: WIT Validation Rejects Plugin Missing Required Exports

## Property Statement

For any WASM Component with a strict subset of the required Prism WIT exports,
`validate_wit_interface(component, required_exports)` returns
`Err(PluginError::InvalidInterface)` with an error message that names the missing export.
For a component that provides all required exports, the function returns `Ok(plugin_type)`.
The function is deterministic: the same component + required export set always produces
the same result.

## Source Contract

- **Anchor Story:** `S-1.15`
- **Source BC:** BC-2.17.006 — WIT Interface Validation
- **Module:** prism-spec-engine
- **Category:** Correctness / Contract Enforcement

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — generates all permutations of present/absent exports | Full cross-product of required export presence/absence |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::plugin::validate_wit_interface
//
// Sketch: Use a fixed required export list (e.g. ["prism:plugin/query", "prism:plugin/schema"]).
// Generate components with arbitrary subsets of that list as their actual exports.
//
// proptest!(|(present_exports in arb_export_subset(REQUIRED_EXPORTS))| {
//     let component = build_synthetic_component(&present_exports);
//     let result = validate_wit_interface(&component, REQUIRED_EXPORTS);
//
//     if present_exports.iter().all(|e| REQUIRED_EXPORTS.contains(e))
//         && present_exports.len() == REQUIRED_EXPORTS.len()
//     {
//         // All required exports present
//         prop_assert!(result.is_ok(), "complete export set must return Ok");
//     } else {
//         // At least one required export missing
//         prop_assert!(result.is_err(), "missing export must return Err");
//         let err = result.unwrap_err();
//         prop_assert!(matches!(err, PluginError::InvalidInterface { .. }),
//             "error type must be InvalidInterface");
//         let missing: Vec<_> = REQUIRED_EXPORTS.iter()
//             .filter(|e| !present_exports.contains(e))
//             .collect();
//         let err_msg = format!("{:?}", err);
//         for m in &missing {
//             prop_assert!(err_msg.contains(m),
//                 "error message must name missing export {}", m);
//         }
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No — but export set is finite | Required export list is small (<=10 interfaces); 2^10 = 1024 possible subsets |
| Tool support? | Full | proptest + synthetic component builder (mock, no wasmtime execution needed) |
| Execution time budget | <30 seconds for 10k cases | Subset enumeration is cheap without real WASM compilation |
| Assumptions required | Synthetic component builder produces a mock Component that reports its exports via the same API as a real wasmtime Component | Abstracted behind a trait in test code |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.17.006. Tests all required-export subset permutations including empty set and single-missing cases. |
