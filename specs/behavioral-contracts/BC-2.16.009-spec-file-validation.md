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
input-hash: "3ff257e"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.009: Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation

## Description

Spec file validation runs at startup, on reload, and on `add_sensor_spec`. It performs
five categories of checks in a single pass: schema validation (field types, regex
patterns, enumerations), variable reference resolution (ensuring `${step.field}`
references point to steps that exist and have already executed), OCSF field validation
(against the compiled protobuf schema), pagination configuration consistency, and rate
limit hint validity.

All errors and warnings are collected in a single pass and reported together in a
multi-error format grouped by file, table, and field, including exact TOML paths for
actionable correction. Warnings do not prevent loading; errors do.

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

## Postconditions
- If any errors are found: the spec is rejected and the error list is returned
- If only warnings are found: the spec loads successfully and warnings are logged at startup and included in reload results
- If no issues: the spec loads cleanly

## Multi-Error Reporting
- All validation errors and warnings are collected in a single pass
- Errors are grouped by spec file, then by table, then by field
- Each error includes the exact TOML path (e.g., `sensor.tables[0].steps[1].path_template`) for actionable correction
- Warnings do not prevent the spec from loading; errors do

## Invariants
- Validation is always a single-pass, all-errors-collected operation (no fail-fast on first error)
- A spec with any errors is never loaded or written to disk
- Warnings are reported but never block loading

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | Schema or variable reference validation error (with TOML path and corrective guidance) | Spec rejected; all errors reported together |
| `E-SPEC-001` | TOML parse error (syntax error in the file) | Spec rejected; parse error with line number |
| `E-SPEC-009` | Duplicate sensor_id across spec files | Second file rejected; first wins |
| `E-SPEC-004` | Duplicate table_name within a sensor | Spec file rejected entirely |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-038 | Forward variable reference | `E-SPEC-001` with message identifying referencing and referenced steps |
| Warning-only | invalid ocsf_field paths (not in compiled schema) | Spec loads; warnings logged; runtime falls back to raw_extensions |
| Multiple errors | 3 schema errors + 2 variable errors in one file | All 5 reported in single response; spec rejected |
| Empty table | table with no columns | `E-SPEC-001`: "Table must have at least one column" |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — valid spec | well-formed TOML; all fields valid | Spec loads; no errors; no warnings |
| Schema error | `sensor_id: "123-invalid"` (starts with digit) | `E-SPEC-001` with TOML path `sensor.sensor_id` |
| Forward variable reference | step 2 references step 3 | `E-SPEC-001` with message identifying forward reference |
| Invalid OCSF field | `ocsf_field: "nonexistent.field"` | Warning logged; spec loads; field goes to raw_extensions at runtime |
| Multiple errors | invalid sensor_id + forward reference | Both errors reported together; spec rejected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok — for any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec()` returns `Err(errors)` where `errors.len() == N`; for a spec with only warnings and no errors, returns `Ok(warnings)` (spec accepted); the function never returns early on the first error. Method: Proptest. Priority: P1. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (normalized from inline Error Codes section); added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
