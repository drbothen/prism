---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.001: OCSF Schema Loading at Build Time via ocsf-proto-gen

## Preconditions
- `build.rs` is configured to invoke `ocsf-proto-gen` (with `default-features=false`) to generate `.proto` files from OCSF JSON schema
- `prost-build` compiles the generated `.proto` files into Rust types

## Postconditions
- All 83 OCSF v1.x event classes are available as protobuf message descriptors at runtime
- The `enum-value-map.json` file is generated and embedded for runtime integer-to-caption lookup
- No network access is required at runtime for OCSF schema resolution
- The OCSF version is fixed per Prism release (pinned in `build.rs` configuration)

## Invariants
- DI-005: OCSF schema validity -- generated types match the pinned OCSF version

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Build failure | `ocsf-proto-gen` fails to parse OCSF JSON schema | Compile-time error with the specific parsing failure; build aborts |
| Build failure | `prost-build` fails on generated `.proto` files | Compile-time error; indicates a bug in `ocsf-proto-gen` output |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-001 | OCSF schema version updated in `build.rs` but proto files cached | Build system detects schema version change and regenerates; `cargo clean` is not required |
| EC-02-002 | `ocsf-proto-gen` generates a `.proto` with reserved field numbers | `prost-build` rejects the proto; build failure with actionable message pointing to the conflicting field |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
