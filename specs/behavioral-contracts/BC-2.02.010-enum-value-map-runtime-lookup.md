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
input-hash: "1e29f9d"
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

# BC-2.02.010: OCSF Enum Value Map for Runtime Display Names

## Description

At build time, `ocsf-proto-gen` generates an `enum-value-map.json` file that maps OCSF enum type names and integer values to human-readable captions (e.g., `severity_id: 4` → `"Critical"`). This map is embedded in the binary and used at runtime to enrich MCP tool responses with both integer values and display captions, improving readability for AI agents. Enum values not present in the map (e.g., vendor-specific extensions) return `"Unknown ({value})"` rather than an error.

## Preconditions
- `ocsf-proto-gen` has generated the `enum-value-map.json` file at build time
- The map is embedded in the binary (via `include_str!` or similar)

## Postconditions
- All OCSF enum fields (e.g., `severity_id`, `activity_id`, `status_id`) can be resolved to human-readable captions at runtime
- The lookup function takes an enum type name and integer value, returning the display caption (e.g., `severity_id: 4` resolves to `"Critical"`)
- MCP tool responses include both the integer enum value and the display caption for AI agent consumption

## Invariants
- Enum value map is consistent with the pinned OCSF schema version

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Enum value not found in the map (e.g., vendor-specific extension value) | Returns `"Unknown ({value})"` as the caption; not an error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-018 | OCSF enum has value `99` (Other) mapped from an unrecognized vendor severity | Caption resolves to `"Other"`; the original vendor value is preserved in `raw_extensions` |
| EC-02-019 | `enum-value-map.json` is empty (zero enum definitions) | Valid but degenerate; all enum lookups return `"Unknown"` captions; warning logged at startup |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.010-001 | `severity_id: 4` lookup | Returns `"Critical"` |
| TV-BC-2.02.010-002 | `severity_id: 99` (Other) lookup | Returns `"Other"` |
| TV-BC-2.02.010-003 | Vendor-specific enum value not in map | Returns `"Unknown (42)"` (or equivalent); not an error |
| TV-BC-2.02.010-004 | Empty `enum-value-map.json` | All lookups return `"Unknown"`; startup warning logged |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

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
