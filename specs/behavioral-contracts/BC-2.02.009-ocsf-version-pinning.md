---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "3eb97f3"
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

# BC-2.02.009: OCSF Version Pinning Per Release

## Description

Each Prism release pins a specific OCSF schema version in `build.rs` (e.g., `v1.7.0`). All protobuf descriptors in the compiled binary correspond exclusively to this pinned version, and the OCSF version is exposed in MCP resource metadata. Field mappings are validated against the pinned schema at compile time, ensuring that upgrading OCSF is a deliberate, release-gated change rather than an accidental dependency drift.

## Preconditions
- `build.rs` specifies an exact OCSF schema version (e.g., `v1.7.0`)
- `ocsf-proto-gen` is invoked with this pinned version

## Postconditions
- All OCSF protobuf descriptors in the compiled binary correspond to the pinned version
- The OCSF version is exposed in MCP resource metadata (`ocsf_version` field)
- Field mappings are validated against the pinned schema version at compile time
- Upgrading OCSF version requires a new Prism release (deliberate, not accidental)

## Invariants
- DI-005: OCSF schema validity -- binary contains exactly one OCSF schema version

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Build failure | Pinned OCSF version not found by `ocsf-proto-gen` | Compile-time error with suggestion to verify OCSF version string |
| Build failure | Field mapping references an OCSF field removed in the pinned version | Compile-time error identifying the removed field and the sensor mapper that references it |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-016 | OCSF v1.8.0 adds new event classes not present in pinned v1.7.0 | New classes unavailable until Prism upgrades its pin; sensor records that would map to new classes use the closest existing class or Base Event |
| EC-02-017 | OCSF v1.8.0 deprecates a field used in Prism's mappers | Deprecation detected at build time when pin is updated; mapper must be updated before build succeeds |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.009-001 | Build with pinned v1.7.0; all mappers reference valid fields | Build succeeds; binary contains v1.7.0 descriptors; `ocsf_version: "v1.7.0"` in MCP metadata |
| TV-BC-2.02.009-002 | Pinned version string typo (e.g., "v1.7.x") | `ocsf-proto-gen` reports version not found; build fails with actionable error |
| TV-BC-2.02.009-003 | Pin updated to v1.8.0 which removed a field used in mapper | Compile-time error identifies removed field and mapper location |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
