---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.16.009: Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation

## Preconditions
- A sensor spec file has been parsed from TOML into the `SensorSpec` struct (BC-2.16.001)
- Validation runs at startup and on reload (BC-2.16.005) and on `add_sensor_spec` (BC-2.16.008)

## Validation Rules

### 1. Schema Validation
- `sensor_id` must match `^[a-z][a-z0-9_-]*$` — same character set as client_id (BC-2.06.010)
- `name` must be non-empty
- `auth_type` must be one of: `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`
- `base_url` must be a valid URL (parsed by `url::Url`)
- `version` must be a valid semver string
- Each table must have a non-empty `table_name` matching `[a-zA-Z0-9_]+`
- Each table must have at least one column
- Each table must have at least one step
- Column names must be unique within a table
- Column types must be one of: `string`, `integer`, `float`, `boolean`, `datetime`, `json`
- Column options must be one of: `REQUIRED`, `INDEX`, `ADDITIONAL`, `HIDDEN`, `OPTIMIZED` (or empty for no options)

### 2. Variable Reference Resolution (DEC-038)
- All `${step_name.field}` references in `path_template` and `body_template` are resolved against the step dependency graph
- A variable reference to a step that does not exist in the same table's steps array: **validation error** `E-SPEC-001` with message "Step '{step_name}' referenced in template but not defined. Available steps: [...]"
- A variable reference to a step that appears AFTER the referencing step (forward reference): **validation error** `E-SPEC-001` with message "Step '{referencing_step}' references '{referenced_step}' which has not executed yet. Steps execute in order."
- Self-references (`${this_step.field}`): **validation error** unless the step explicitly declares the variable in `variables_produced` from a prior execution context

### 3. OCSF Field Validation
- Each `ocsf_field` value is checked against the compiled OCSF protobuf schema
- Invalid OCSF field paths: **warning** (not error) — logged with the column name and invalid path
- Valid OCSF field paths with incompatible types (e.g., mapping a `string` column to an OCSF `int32` field): **warning** — coercion will be attempted at runtime (BC-2.16.003)

### 4. Pagination Configuration Validation
- If pagination type is `cursor_token`, `cursor_response_path` must be a valid JSONPath expression
- If pagination type is `offset_limit`, `page_size` must be > 0
- If pagination type is `none`, no pagination fields should be set (warning if they are)

### 5. Rate Limit Hint Validation
- `requests_per_second` must be > 0 if specified
- `burst_size` must be >= 1 if specified

## Multi-Error Reporting
- All validation errors and warnings are collected in a single pass (same pattern as BC-2.06.005)
- Errors are grouped by spec file, then by table, then by field
- Each error includes the exact TOML path (e.g., `sensor.tables[0].steps[1].path_template`) for actionable correction
- Warnings do not prevent the spec from loading; errors do

## Postconditions
- If any errors are found: the spec is rejected and the error list is returned
- If only warnings are found: the spec loads successfully and warnings are logged at startup and included in reload results
- If no issues: the spec loads cleanly

## Error Codes
- `E-SPEC-001`: Schema or variable reference validation error (with TOML path and corrective guidance)
- `E-SPEC-001`: TOML parse error (syntax error in the file)
- `E-SPEC-009`: Duplicate sensor_id across spec files
- `E-SPEC-004`: Duplicate table_name within a sensor

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec |
| Capabilities | CAP-029 |
| Priority | P0 |
