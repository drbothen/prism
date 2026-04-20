---
document_type: behavioral-contract
level: L3
version: "1.1"
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
input-hash: "[pending-recompute]"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.008: `add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime

## Description

The `add_sensor_spec` MCP tool allows analysts to add support for a new sensor API
at runtime by uploading a TOML spec file. The spec is validated using the same pipeline
as startup loading, written atomically to the `sensor_specs_dir`, and then hot-loaded
via an internal `reload_config` call — no restart required.

If a spec for the same `sensor_id` already exists, a confirmation token is required
(write-gating pattern, BC-2.04.009) since this is an update to an existing definition.
The tool supports dry-run mode to validate and preview without persisting. It is gated
by the `sensor_spec.write` capability and follows the hidden-tools pattern.

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

## Invariants
- Spec is validated before any file write — an invalid spec never reaches disk
- File write is atomic (temp file + fsync + rename) — no partial spec files
- Every invocation is audit-logged (DI-004)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` to `E-SPEC-007` | Validation errors | All errors returned in multi-error format; no file written |
| `E-IO-001` | File write error (disk full, permission denied) | Spec not loaded; structured error with path and OS error |
| (confirmation token) | `sensor_id` already exists | Confirmation token returned; no write until confirmed |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| New sensor | spec for new sensor_id | Validated; written; hot-loaded; tables registered |
| Update existing | spec for existing sensor_id | Confirmation token required first |
| Dry run | `dry_run: true` | Validation only; preview returned; no write |
| Validation failure | invalid TOML | All errors returned; no file written; no reload |
| Disk full | write fails mid-operation | `E-IO-001`; no partial spec; reload not triggered |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — new spec | valid TOML for new sensor_id | Spec written; reload triggered; tables returned |
| Dry run | `dry_run: true`, valid TOML | Validation passes; table preview returned; no file written |
| Existing sensor_id | spec for already-loaded sensor | Confirmation token returned |
| Invalid TOML | malformed spec | All validation errors; no write |
| Disk full | valid spec; disk full on write | `E-IO-001`; no reload triggered |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify atomic write (temp + rename pattern) |
| (placeholder) | VP to be assigned — verify sensor_spec.write capability gate |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.16.001 (spec loading), BC-2.16.005 (reload_config), BC-2.16.009 (spec validation), BC-2.04.005 (hidden-tools), BC-2.04.009 (write gating) |
| Priority | P1 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; fixed capability frontmatter (was array [CAP-029,CAP-030] → CAP-029 primary); added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
