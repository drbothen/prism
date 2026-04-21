---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "ac6b633"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.010: `list_sensor_specs` MCP Tool — List Loaded Sensor Specs with Table Schemas and Status

## Description

The `list_sensor_specs` MCP tool provides a read-only introspection view of all
config-driven sensor specs currently loaded in the `ConfigSnapshot`. For each spec
it reports metadata, table definitions, column schemas with OCSF mappings, pagination
type, and availability status. When a `client_id` is provided, per-client credential
status is included so analysts can quickly determine which sensors are ready to query
for a specific client.

The tool is always visible (no capability gating) and uses `structuredContent` for
machine-parseable output, enabling AI agents to reason about available data sources.

## Preconditions
- Prism is running with a valid `ConfigSnapshot` that may contain zero or more loaded sensor specs

## Tool Schema
```json
{
  "name": "list_sensor_specs",
  "description": "List all loaded config-driven sensor specs with their table schemas, column definitions, OCSF mappings, and availability status per client.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "client_id": {
        "type": "string",
        "description": "Optional. If provided, show availability status for this specific client (whether credentials are configured). If null, show specs without client-specific status.",
        "nullable": true
      },
      "sensor_id": {
        "type": "string",
        "description": "Optional. If provided, show only the spec for this sensor. If null, show all specs.",
        "nullable": true
      }
    }
  }
}
```

## Postconditions
- Returns a structured list of all loaded `SensorSpec` entries from the current `ConfigSnapshot`
- For each sensor spec:
  - `sensor_id`, `name`, `version`, `auth_type`, `base_url`
  - `tables`: list of table definitions, each with:
    - `table_name` (fully qualified as `{sensor_id}.{table_name}`)
    - `columns`: list of column definitions with name, type, options, ocsf_field mapping
    - `steps_count`: number of fetch pipeline steps
    - `pagination_type`: cursor/offset/none
  - `status`: one of:
    - `"available"` — credentials are configured for at least one client
    - `"no_credentials"` — spec loaded but no client has credentials for this sensor (DEC-036)
    - `"validation_warnings"` — spec loaded with warnings (includes warning list)
  - If `client_id` is provided:
    - `client_status`: `"configured"` (client has credentials) or `"not_configured"` (client lacks credentials for this sensor)
- If `sensor_id` is provided and not found, returns an empty list (not an error)
- The `list_sensor_specs` tool is always visible (read-only, no capability gating)
- Response uses `structuredContent` for machine-parseable schema data
- `content[].text` summary includes sensor count, table count, and availability overview
- Follows the same response envelope pattern as other list tools (BC-2.09.008)

## Invariants
- Read-only: the tool does not modify any state
- Always visible: no capability gating required
- Returns empty list (not an error) when no specs are loaded or when `sensor_id` is not found

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Tool cannot fail under normal operation | Unknown `sensor_id` → empty list; unknown `client_id` → `client_status: not_configured` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-036 | Spec loaded but no client has credentials | `status: no_credentials` |
| No specs loaded | sensor_specs_dir is empty | Empty list returned; no error |
| sensor_id not found | `sensor_id: "nonexistent"` | Empty list (not an error) |
| With client_id | `client_id: "acme"` | Per-spec `client_status: configured | not_configured` |
| Spec with validation warnings | warnings at load time | `status: validation_warnings` with warning list |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — specs loaded | no filters | All specs with tables, columns, status; machine-parseable structuredContent |
| With client_id | `client_id="acme"` | Each spec includes `client_status: configured|not_configured` |
| Filter by sensor_id | `sensor_id="crowdstrike"` | Only crowdstrike spec returned |
| No credentials | spec loaded; no client credentials | `status: no_credentials` |
| No specs | empty directory | Empty list; no error |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Always-visible is a boolean field in tool registration (unit test); structuredContent format correctness is a serialization integration test; no pure-function formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | -- |
| Related BCs | BC-2.16.001 (spec loading), DEC-036 (no credentials) |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline notes); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
