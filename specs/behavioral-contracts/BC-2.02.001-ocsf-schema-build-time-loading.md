---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[pending-recompute]"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
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

## Description

At build time, `build.rs` invokes `ocsf-proto-gen` (with `default-features=false`) to generate `.proto` files from the pinned OCSF JSON schema, which `prost-build` then compiles into Rust types. This approach ensures all 83 OCSF v1.x event class descriptors and the `enum-value-map.json` are embedded in the binary, eliminating any runtime network dependency on OCSF schema resolution and guaranteeing schema consistency per release.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.001-001 | `build.rs` with valid OCSF v1.7.0 pin | All 83 event class descriptors compiled; `enum-value-map.json` embedded; build succeeds |
| TV-BC-2.02.001-002 | OCSF version updated in `build.rs` with cached protos | Cache invalidated; protos regenerated; build succeeds |
| TV-BC-2.02.001-003 | `ocsf-proto-gen` produces proto with reserved field numbers | `prost-build` fails; actionable compile error with conflicting field identified |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-022 | OCSF normalizer: never panics on arbitrary input (fuzz) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors with VP-016/VP-022 cross-reference; added ## Verification Properties; added ## Changelog. |
