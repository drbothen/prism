---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "OCSF Normalization"
capability: "CAP-003"
---

# BC-2.02.010: OCSF Enum Value Map for Runtime Display Names

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
