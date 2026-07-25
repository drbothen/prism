---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029, CAP-030"
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
input-hash: "76729b7"
traces_to:
  - "CAP-029"
  - "CAP-030"
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

**Note (MED-003):** The as-built MCP wire parameter names (verified in `crates/prism-mcp/src/server.rs` — `AddSensorSpecParams`) are `toml_content` (required) and `name` (required). The prior spec version used `spec_toml` and `file_name`, which are the names of the internal Rust struct `AddSensorSpecArgs` — NOT the wire parameter names. An LLM agent following the old spec would send `spec_toml` and receive a 400 error because the server expects `toml_content`. This spec amendment aligns §Tool Schema to the deployed wire API. The `spec_toml` name belongs to the unrelated `create_action` tool in server.rs. Both `name` and `toml_content` are required in the as-built wire API (per `validate_text_field` calls in the handler). The handler maps wire → internal as: `spec_toml: params.toml_content, file_name: Some(params.name)`.

```json
{
  "name": "add_sensor_spec",
  "description": "Upload a new sensor spec TOML file. The spec is validated, persisted to the sensor specs directory, and hot-loaded without restart. Use this to add support for a new sensor API at runtime.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "toml_content": {
        "type": "string",
        "description": "The full TOML content of the sensor spec file (≤256 KiB). The sensor_id is extracted from the parsed spec."
      },
      "name": {
        "type": "string",
        "pattern": "^[a-z][a-z0-9_-]*\\.sensor\\.toml$",
        "description": "File name to save as (≤256 bytes, must match pattern). Conventionally derived from sensor_id (e.g., 'newvendor.sensor.toml')."
      },
      "dry_run": {
        "type": "boolean",
        "description": "If true, validate only without persisting or loading. Default: false.",
        "default": false
      }
    },
    "required": ["toml_content", "name"]
  }
}
```

## Postconditions
- The `spec_toml` TOML content is validated via `parse_and_validate_spec_toml()` which calls `SpecLoader::parse()` as its first act — the same `SpecLoader::parse()` entry point used by startup loading (BC-2.16.009 §Entry points and function coverage). Rules 9 and 10 [PLANNED — Wave-A engine story] will be enforced because they are implemented inside `SpecLoader::parse()` and are therefore automatically reached by this path; `sensor_spec.write` clients cannot bypass Rule 9 by calling a path that omits `parse()`.
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

The validation error surface covers all errors that `parse_and_validate_spec_toml()` → `SpecLoader::parse()` → `resolve_env_var_tokens()` → `validate_step_methods()` can emit. See BC-2.16.009 §Validation Rules 1–10 and §Error Conditions for the definitive per-rule error catalog.

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | TOML parse / schema validation error (field types, regex, enumeration) or TOML syntax error | All validation errors returned in multi-error format; no file written |
| `E-SPEC-003` | Undefined variable reference (forward reference or dangling step reference) | Spec rejected; error names both referencing and referenced steps |
| `E-SPEC-009` | Duplicate `sensor_id` across spec files | Second spec rejected; first wins (write gate for new upload) |
| `E-SPEC-024` | Unresolvable `${env.VAR_NAME}` token in a string field (Rule 6) | Spec rejected; message names the variable (value never echoed — AD-017) |
| `E-SPEC-025` | `step.method` value not in HTTP whitelist after env-var resolution (Rule 7) | Spec rejected; message includes step_name, sensor_id, table_name, method_value |
| `E-SPEC-026` | `probe_table` references a non-existent table name (Rule 8) | Spec rejected; message includes sensor_id, probe_table value, declared table list |
| `E-SPEC-027` | `header_scheme` syntactically invalid, incoherent with auth_type, or absent for cookie_roundtrip (Rule 9) | **Rule 9 validation** [PLANNED — S-WAVE-A-ENGINE-001]: Spec rejected; no file written. **Current MCP contract (as-built, pre-S-WAVE-A-MCP-001):** validation failures on this path return multi-error prose strings via the `ValidationFailed` display mechanism — no `structuredContent.error.code`, no BC-2.10.007 structured envelope. This is a documented gap (ADR-053 §D6 Option B), not intended design: the prose format is LLM-agent-opaque on an agent-facing surface. **Authorized MCP contract (S-WAVE-A-MCP-001):** when S-WAVE-A-MCP-001 is delivered, `add_sensor_spec` MUST emit `isError: true` + `structuredContent.error.code` + `structuredContent.error.errors` (per-error detail array) in the BC-2.10.007 structured envelope. Rule 9 landing in S-WAVE-A-ENGINE-001 leaves this BC satisfied under the current-contract clause — the envelope obligation is independent of Rule 9 and takes effect only when S-WAVE-A-MCP-001 ships. This row is the blocking prerequisite for S-WAVE-A-MCP-001 per ADR-053 §D6 Option B — implementation of that story may proceed once this BC is committed. |
| `E-SPEC-028` | `[auth_acquisition]` block coherence violation — any of 8 sub-conditions (Rule 10) | [PLANNED — Wave-A engine story] Spec rejected; no file written |
| `E-SPEC-002` | Filesystem write error (disk full, permissions, path not found) | Spec not loaded; structured error with path and OS error |
| (confirmation token) | `sensor_id` already exists in spec directory | Confirmation token returned; no write until confirmed (BC-2.04.009 write-gating pattern) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| New sensor | spec for new sensor_id | Validated; written; hot-loaded; tables registered |
| Update existing | spec for existing sensor_id | Confirmation token required first |
| Dry run | `dry_run: true` | Validation only; preview returned; no write |
| Validation failure | invalid TOML | All errors returned; no file written; no reload |
| Disk full | write fails mid-operation | `E-SPEC-002`; no partial spec; reload not triggered |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — new spec | valid TOML for new sensor_id | Spec written; reload triggered; tables returned |
| Dry run | `dry_run: true`, valid TOML | Validation passes; table preview returned; no file written |
| Existing sensor_id | spec for already-loaded sensor | Confirmation token returned |
| Invalid TOML | malformed spec | All validation errors; no write |
| Disk full | valid spec; disk full on write | `E-SPEC-002`; no reload triggered |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Temp+rename atomicity is an OS-level file system guarantee, not a Prism code property; sensor_spec.write gate covered transitively by VP-002 (deny-by-default capability); no pure-function formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029, CAP-030 |
| L2 Invariants | DI-004 |
| Related BCs | BC-2.16.001 (spec loading), BC-2.16.005 (reload_config), BC-2.16.009 (spec validation — this tool MUST reach `SpecLoader::parse()` per BC-2.16.009 §Validation Rules 9 §Security requirement), BC-2.04.005 (hidden-tools), BC-2.04.009 (write gating) |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | FB46 | 2026-07-25 | product-owner | F-WASE-P62-HIGH-005: §Error Conditions expanded from closed range `E-SPEC-001..007` to the real validation-error surface: E-SPEC-001, E-SPEC-003, E-SPEC-009, E-SPEC-024, E-SPEC-025, E-SPEC-026, E-SPEC-027 [PLANNED Wave-A], E-SPEC-028 [PLANNED Wave-A], E-SPEC-002 (write error). Introductory sentence added naming `parse_and_validate_spec_toml()` → `SpecLoader::parse()` as the validation entry point and cross-referencing BC-2.16.009 §Error Conditions. `E-IO-001` (was: incorrect error code) replaced by `E-SPEC-002` (correct code for filesystem write error per error-taxonomy.md). E-SPEC-027 row CRIT-001 closure (same burst): `[PLANNED — Wave-A engine story]` tag disambiguated — Rule 9 validation is S-WAVE-A-ENGINE-001; MCP envelope is S-WAVE-A-MCP-001; MUST trigger rebound from "when Rule 9 is implemented" to "when S-WAVE-A-MCP-001 is delivered" (the engine story satisfies this BC under the current-contract clause on its own merge — no violation); current contract stated: multi-error prose strings via `ValidationFailed` (no BC-2.10.007 structured envelope); documented as known gap per ADR-053 §D6 Option B; authorized MCP contract for S-WAVE-A-MCP-001 specified (`isError: true` + `structuredContent.error.code` + `structuredContent.error.errors`); this amendment is the blocking prerequisite for that story. TD-VSDD-060 sibling sweep: `E-IO-001` corrected to `E-SPEC-002` in §Edge Cases (Disk full row) and §Canonical Test Vectors (Disk full row) — two sibling sites that were not swept when §Error Conditions was updated. F-WASE-P62-HIGH-005 also: §Postconditions updated to name `SpecLoader::parse()` as the shared entry point and assert the security property that `sensor_spec.write` clients cannot bypass Rule 9 via this path. §Related BCs back-reference to BC-2.16.009 §Validation Rules 9 §Security requirement added. F-WASE-P62-MED-003: §Tool Schema reconciled to as-built wire parameter names (MED-003). Prior spec used internal Rust struct names `spec_toml`/`file_name` (AddSensorSpecArgs) rather than wire names. Wire names verified in server.rs AddSensorSpecParams: `toml_content` (required) and `name` (required). The name `spec_toml` belongs to the unrelated `create_action` tool. LLM agents using the old spec would send `spec_toml` and receive a 400 error. §Tool Schema updated to `toml_content` (required) + `name` (required); explanatory note added. BC-2.16.009 §Security requirement already referenced `toml_content` correctly; no change needed there. |
| 1.5 | pass-94-fix | 2026-04-21 | product-owner | F94-002: body Traceability L2 Capability row updated CAP-029 → CAP-029, CAP-030 to match frontmatter (frontmatter was corrected in pass-92 F92-001 but body Traceability was missed). |
| 1.4 | pass-92-fix | 2026-04-21 | product-owner | F92-001: corrected capability frontmatter from "CAP-029" to "CAP-029, CAP-030" and expanded traces_to to dual-anchor ["CAP-029","CAP-030"] to match BC-INDEX line 209 and PRD line 862 declarations. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; fixed capability frontmatter (was array [CAP-029,CAP-030] → CAP-029 primary); added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
