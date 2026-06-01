---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: "2026-05-31"  # v1.7 S-DEMO-CROWDSTRIKE-MULTIREGION-001 BC attachment burst
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
- `auth_type` must be one of: `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`, `custom_via_plugin` — per `VALID_AUTH_TYPES` in `spec_parser.rs::validate_cross_composition` (5-value canonical set; `custom_via_plugin` permits external plugin-registered auth strategies per S-PLUGIN-PREREQ-E / ADR-026)
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

### 6. Env Var Token Resolution (AC-6)
Post-TOML-parse, before URL-format validation, the resolver scans all string fields for `${env.VAR_NAME}` tokens and resolves them against `std::env::var`.

**Sibling-sweep (TD-VSDD-060):** All four canonical sensor specs (`crates/prism-sensors/specs/`) were audited. As of S-SPEC-ENV-VAR-001, the `${env.VAR}` pattern was used in `base_url` of `armis.sensor.toml`, `claroty.sensor.toml`, `cyberint.sensor.toml`. As of S-DEMO-CROWDSTRIKE-MULTIREGION-001, `crowdstrike.sensor.toml` also uses `${env.CROWDSTRIKE_BASE_URL}` for `base_url` — all four canonical sensor specs now use the env-var pattern for `base_url`. No other string fields (`path_template`, `body_template`, `response_path`, `ocsf_class`, etc.) in the four canonical sensor specs currently use env tokens. The resolver MUST scan `base_url` at minimum; the implementation SHOULD scan all `String` fields in `SensorSpec` to remain correct for future specs. Per-org overlay `base_url` is also in scope (overlays are merged before validation runs).

**Partial interpolation** is supported within `base_url`: the pattern `"https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` replaces only the `${env.VAR_NAME}` token, preserving the surrounding literal string. After resolution, the full URL is subject to the `starts_with("http://")` / `starts_with("https://")` URL-format validation rule (Validation Rule 1).

**Token format:** `${env.VAR_NAME}` where `VAR_NAME` matches `[A-Z0-9_]+` (uppercase letters, digits, underscores — standard POSIX env var names). Tokens with a different namespace (e.g., `${step.field}`) are NOT resolved by this pass; those belong to the runtime interpolation engine (BC-2.16.002).

**Error path (E-SPEC-024):**
- If `VAR_NAME` is absent from the environment (`std::env::var` returns `VarError::NotPresent`) → validation error `E-SPEC-024`
- If `VAR_NAME` is present but the value is empty string (`""`), → validation error `E-SPEC-024` (empty value is treated as missing)
- The error message MUST include the variable NAME (`VAR_NAME`) and the TOML path (`toml_path`) of the failing field
- The error message MUST NOT include the variable VALUE — per AD-017 / AI-opaque-credentials discipline; credential or instance endpoint values must not appear in logs or MCP error responses
- Multiple unresolvable tokens produce multiple E-SPEC-024 errors, one per token, collected in the same multi-error pass (no fail-fast)
- Fail-closed: a spec with any unresolved env tokens is REJECTED ENTIRELY; it does not load in a degraded state

**Success path:**
- Every `${env.VAR_NAME}` token in every string field is replaced with the resolved value
- The resulting string (with tokens replaced) is passed to subsequent validation rules (URL-format check, etc.)
- If a field contained only the token (e.g., `base_url = "${env.ARMIS_INSTANCE_URL}"`), the resolved value must itself pass URL-format validation

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
| `E-SPEC-002` | Invalid column type for a column in a sensor table (type not in: string, integer, float, boolean, datetime, json) | Spec rejected; error message includes sensor_id, table_name, column_name, and the invalid type value |
| `E-SPEC-003` | Undefined variable reference `${step.field}` — step name does not exist, or forward reference to a step that has not yet executed | Spec rejected; error message names both the referencing step and the undefined step; forward-reference message distinguishes "not defined" from "not yet executed" |
| `E-SPEC-009` | Duplicate `sensor_id` across spec files | Second file rejected; first wins |
| `E-SPEC-004` | Duplicate table_name within a sensor | Spec file rejected entirely |
| `E-SPEC-024` | `${env.VAR_NAME}` token in a string field (e.g., `base_url`) references an env var that is absent or empty at spec-load time | Spec rejected; error message includes var NAME and TOML path; var VALUE never included (AD-017); multiple tokens → multiple errors in same multi-error pass |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-038 | Forward variable reference | `E-SPEC-001` with message identifying referencing and referenced steps |
| Warning-only | invalid ocsf_field paths (not in compiled schema) | Spec loads; warnings logged; runtime falls back to raw_extensions |
| Multiple errors | 3 schema errors + 2 variable errors in one file | All 5 reported in single response; spec rejected |
| Empty table | table with no columns | `E-SPEC-001`: "Table must have at least one column" |
| EC-009-001 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL` not set | `E-SPEC-024` with message citing `ARMIS_INSTANCE_URL` (name only, no value); TOML path `sensor.base_url`; spec rejected |
| EC-009-002 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL=""` (empty string) | `E-SPEC-024` (empty value treated as missing); spec rejected |
| EC-009-003 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with `CYBERINT_ENVIRONMENT=us1` | Resolves to `"https://us1.cyberint.io"`; URL-format validation passes; spec loads |
| EC-009-004 | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"` with `CYBERINT_ENVIRONMENT` not set | `E-SPEC-024` with message citing `CYBERINT_ENVIRONMENT`; partial URL not constructed; spec rejected |
| EC-009-005 | `base_url = "${env.ARMIS_INSTANCE_URL}"` with `ARMIS_INSTANCE_URL="not-a-url"` (no http/https prefix) | Env var resolves successfully; then URL-format validation (`E-SPEC-001`: "base_url must start with http:// or https://") fires on the resolved value |
| EC-009-006 | Spec has two fields with unresolvable env tokens | Two `E-SPEC-024` errors emitted (one per token); both included in the multi-error response; spec rejected |
| EC-009-007 | Per-org overlay has `base_url = "${env.ARMIS_INSTANCE_URL}"` and var is not set | `E-SPEC-024` emitted with TOML path `base_url` and file path identifying the overlay file; overlay rejected; TYPE spec's `base_url` is NOT substituted as fallback — fail-closed |
| EC-009-008 | `crowdstrike.sensor.toml` `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with `CROWDSTRIKE_BASE_URL` set to eu-1 URL (`https://api.eu-1.crowdstrike.com`) | Env var resolves; `base_url` = `"https://api.eu-1.crowdstrike.com"`; URL-format validation passes; spec loads. Demonstrates sensor-agnosticism: same resolver handles any CrowdStrike region URL (us-1, us-2, eu-1, gov) — S-DEMO-CROWDSTRIKE-MULTIREGION-001 |
| EC-009-009 | `crowdstrike.sensor.toml` `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with `CROWDSTRIKE_BASE_URL` not set | `E-SPEC-024` with message citing `CROWDSTRIKE_BASE_URL` (name only, no value); TOML path `sensor.base_url`; spec rejected; fail-closed — S-DEMO-CROWDSTRIKE-MULTIREGION-001 |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — valid spec | well-formed TOML; all fields valid | Spec loads; no errors; no warnings |
| Schema error | `sensor_id: "123-invalid"` (starts with digit) | `E-SPEC-001` with TOML path `sensor.sensor_id` |
| Forward variable reference | step 2 references step 3 | `E-SPEC-001` with message identifying forward reference |
| Invalid OCSF field | `ocsf_field: "nonexistent.field"` | Warning logged; spec loads; field goes to raw_extensions at runtime |
| Multiple errors | invalid sensor_id + forward reference | Both errors reported together; spec rejected |
| Env var — missing var | `base_url = "${env.ARMIS_INSTANCE_URL}"`, `ARMIS_INSTANCE_URL` unset | `E-SPEC-024` message includes `ARMIS_INSTANCE_URL` (name); does NOT include any resolved value; TOML path is `sensor.base_url` |
| Env var — empty var | `base_url = "${env.ARMIS_INSTANCE_URL}"`, `ARMIS_INSTANCE_URL=""` | `E-SPEC-024` (empty treated as missing) |
| Env var — partial interpolation success | `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"`, `CYBERINT_ENVIRONMENT=us1` | Resolved: `"https://us1.cyberint.io"`; spec loads (URL-format valid) |
| Env var — value set but invalid URL | `base_url = "${env.MY_URL}"`, `MY_URL="ftp://example.com"` | Env var resolves; then `E-SPEC-001` (base_url not http/https); value `ftp://example.com` may appear in E-SPEC-001 message (not a credential; plain URL); spec rejected |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok — for any `SensorSpec` with N distinct validation errors (N >= 1), `validate_sensor_spec()` returns `Err(errors)` where `errors.len() == N`; for a spec with only warnings and no errors, returns `Ok(warnings)` (spec accepted); the function never returns early on the first error. Method: Proptest. Priority: P1. |

## Traceability
| Field | Value |
|-------|-------|
| Stories | S-1.11, S-1.13, PLUGIN-MIGRATION-001-F, S-SPEC-ENV-VAR-001, S-DEMO-CROWDSTRIKE-MULTIREGION-001 |
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029. This BC specifies spec-file validation — exactly what CAP-029 mandates: "Every spec file is validated at load time and reload time (DI-030). Variable references in step templates are resolved against the step dependency graph — forward references and undefined variables are validation errors (DEC-038)." Env-var token resolution (AC-6) is a prerequisite of that load-time validation: a spec whose `base_url` contains an unresolved `${env.VAR}` token cannot pass URL-format validation, so resolution must occur in the same spec-load pass. |
| L2 Invariants | DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | S-DEMO-CROWDSTRIKE-MULTIREGION-001 BC attachment burst | 2026-05-31 | product-owner | Updated §Validation Rules 6 sibling-sweep note: `crowdstrike.sensor.toml` now also uses `${env.CROWDSTRIKE_BASE_URL}` for `base_url` (S-DEMO-CROWDSTRIKE-MULTIREGION-001), making all four canonical sensor specs env-var-based for `base_url`. Added EC-009-008 (CrowdStrike eu-1 URL resolution — happy path) and EC-009-009 (CrowdStrike missing CROWDSTRIKE_BASE_URL → E-SPEC-024) for explicit CrowdStrike multi-region coverage. Added S-DEMO-CROWDSTRIKE-MULTIREGION-001 to Stories traceability. No semantic behavior change — the resolver was already sensor-agnostic; this adds CrowdStrike-specific test vectors to the existing contract. |
| 1.6 | S-SPEC-ENV-VAR-001 spec burst | 2026-05-31 | product-owner | Added §Validation Rules 6: Env Var Token Resolution (AC-6). Covers `${env.VAR_NAME}` token resolution in sensor spec string fields at spec-load time (post-TOML-parse, pre-URL-format-validation). Sibling-sweep (TD-VSDD-060): `${env.VAR}` pattern confirmed in `base_url` of `armis.sensor.toml`, `claroty.sensor.toml`, `cyberint.sensor.toml`; no other string fields in the four canonical sensor specs currently use env tokens. Added E-SPEC-024 to §Error Conditions. Added 7 edge cases EC-009-001..EC-009-007 covering: missing var, empty var, partial interpolation (cyberint pattern), failed partial interpolation, invalid URL after resolution, multiple failing tokens, overlay file failing. Added 4 canonical test vectors for env-var scenarios. AD-017 no-value-leak constraint documented explicitly in the rule (var NAME acceptable, var VALUE forbidden). error-taxonomy.md bumped to v1.56 in same burst. |
| 1.5 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 auto-promotion at merge: PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status draft→active (lifecycle_status was already active). |
| 1.4 | FB-IMPL-P2-PO fix-burst-2 | 2026-05-20 | product-owner | F-007 closure (pass-2 adversarial): Added E-SPEC-002 (invalid column type) and E-SPEC-003 (undefined variable reference) to §Error Conditions — both codes were present in error-taxonomy.md and exercised by HS-017 sub-scenarios but absent from this BC's error table (AI-built defect fixed in-scope per CLAUDE.md Canonical Principle Rule 4). F-008 closure: §Validation Rules 1 `auth_type` enumeration expanded from 4-value to 5-value set — added `custom_via_plugin` per `VALID_AUTH_TYPES` constant in `spec_parser.rs::validate_cross_composition` (CODE-GROUNDED: 5 values confirmed in source). |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (normalized from inline Error Codes section); added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
