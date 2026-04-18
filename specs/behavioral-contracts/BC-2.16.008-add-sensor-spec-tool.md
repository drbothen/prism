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
capability: [CAP-029, CAP-030]
---

# BC-2.16.008: `add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime

## Preconditions
- Prism is running with a valid `ConfigSnapshot`
- The analyst (via AI agent) invokes the `add_sensor_spec` MCP tool
- The `sensor_specs_dir` is writable by the Prism process

## Tool Schema
```json
{
  "name": "add_sensor_spec",
  "description": "Upload a new sensor spec TOML file. The spec is validated, persisted to the sensor specs directory, and hot-loaded without restart. Use this to add support for a new sensor API at runtime.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "spec_toml": {
        "type": "string",
        "description": "The full TOML content of the sensor spec file. The sensor_id is extracted from the parsed spec."
      },
      "file_name": {
        "type": ["string", "null"],
        "pattern": "^[a-z][a-z0-9_-]*\\.sensor\\.toml$",
        "default": null,
        "description": "File name to save as. If null, derived from sensor_id in the spec (e.g., 'newvendor.sensor.toml')."
      },
      "dry_run": {
        "type": "boolean",
        "description": "If true, validate only without persisting or loading. Default: false.",
        "default": false
      }
    },
    "required": ["spec_toml"]
  }
}
```

## Postconditions
- The `spec_toml` is parsed as TOML and validated using the same validation pipeline as startup loading (BC-2.16.009)
- The `sensor_id` is extracted from the parsed spec's `[sensor]` section — no separate `sensor_id` parameter
- If a spec file for this `sensor_id` already exists in the specs directory, the tool returns a confirmation token (following the write gating pattern, BC-2.04.009) since this is an update to an existing sensor definition
- If this is a new sensor (no existing file):
  - The spec content is written atomically to `{sensor_specs_dir}/{sensor_id}.toml` (temp file + fsync + rename, matching the alias file write pattern)
  - A `reload_config` is triggered internally to pick up the new spec (BC-2.16.005)
  - The tool returns the list of registered tables and their schemas
- The `add_sensor_spec` tool is gated by the `sensor_spec.write` capability path and follows the hidden-tools pattern (BC-2.04.005)

## Dry Run Mode
- When `dry_run: true`, the spec is parsed and validated but not persisted or loaded
- Returns validation results and a preview of what tables and columns would be registered

## Error Handling
- `E-SPEC-001` through `E-SPEC-007`: validation errors (same as BC-2.16.001 and BC-2.16.009)
- `E-IO-001`: file write error (disk full, permission denied) — spec is not loaded
- Validation failure: the tool returns all errors in the multi-error format, no file is written, no tables are registered

## Audit
- Every `add_sensor_spec` invocation is audit-logged with: sensor_id, dry_run flag, validation result, file written (path), tables registered

## Traces
- CAP-029 (Config-Driven Sensor Adapters)
- CAP-030 (Hot Configuration Reload)
- BC-2.16.001 (Spec file loading)
- BC-2.16.005 (reload_config)
- BC-2.16.009 (Spec validation)
