---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Spec Engine"
capability: "CAP-029"
---

# BC-2.16.010: `list_sensor_specs` MCP Tool — List Loaded Sensor Specs with Table Schemas and Status

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

## Response Format
- Uses `structuredContent` for machine-parseable schema data
- `content[].text` summary includes sensor count, table count, and availability overview
- Follows the same response envelope pattern as other list tools (BC-2.09.008)

## Traces
- CAP-029 (Config-Driven Sensor Adapters)
- DEC-036 (No credentials configured)
