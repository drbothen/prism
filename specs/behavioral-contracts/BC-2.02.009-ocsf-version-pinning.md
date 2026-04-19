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

# BC-2.02.009: OCSF Version Pinning Per Release

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
