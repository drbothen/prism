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

# BC-2.16.001: Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables

## Description

At startup and on reload, Prism scans the configured `sensor_specs_dir` for files
matching `*.sensor.toml` and parses each into a `SensorSpec` struct. Valid specs have
their tables registered with the DataFusion query catalog as `SpecDrivenTableProvider`
instances that implement `TableProvider`. Spec-driven tables are immediately queryable
via PrismQL alongside built-in sensor tables.

OCSF field mappings from `ColumnSpec.ocsf_field` are registered with the OCSF
normalizer so spec-driven columns participate in cross-sensor correlation. Files that
fail validation are skipped with actionable errors but do not block valid specs from
loading (DI-030). If no client has credentials for a spec's `sensor_id`, the spec
loads but its tables are marked unavailable (DEC-036).

## Preconditions
- Prism is starting up or `reload_config` has been invoked (BC-2.16.005)
- A `sensor_specs_dir` path is configured in `prism.toml` (default: `./sensor-specs/`)
- One or more `.toml` sensor spec files exist in the configured directory

## Postconditions
- Each `.toml` file in the sensor specs directory is parsed into a `SensorSpec` struct containing: `sensor_id`, `name`, `auth_type` (oauth2/bearer/cookie/api_key), `base_url`, `tables` (Vec<TableSpec>), `rate_limit_hints`, and `version`
- Each `TableSpec` within a `SensorSpec` is registered as a DataFusion table in the query engine's catalog, following the same pattern as external sensor tables (CAP-015)
- Table names follow the convention `{sensor_id}.{table_name}` (e.g., `sentinelone.alerts`, `sentinelone.agents`)
- Column definitions from `ColumnSpec` entries are translated to Arrow schema fields with appropriate Arrow types: `string` -> Utf8, `integer` -> Int64, `float` -> Float64, `boolean` -> Boolean, `datetime` -> TimestampMicrosecond, `json` -> Utf8 (JSON string)
- OCSF field mappings from `ColumnSpec.ocsf_field` are registered with the OCSF normalizer (CAP-003) so spec-driven columns participate in cross-sensor correlation
- Column options (REQUIRED, INDEX, ADDITIONAL, HIDDEN) are respected: REQUIRED columns enforce WHERE clause constraints (DI-021), INDEX columns enable push-down hints, ADDITIONAL columns trigger enrichment steps, HIDDEN columns are excluded from schema introspection
- The `explain_query` tool (BC-2.11.010) includes spec-driven tables in its available sources listing
- Spec files that fail validation are rejected with actionable errors (BC-2.16.009) but do not prevent other valid specs from loading (DI-030)
- Successfully loaded specs are included in the `ConfigSnapshot` (entity) with their individual file hashes

## Spec File Discovery
- The loader scans `sensor_specs_dir` for files matching `*.sensor.toml`
- Subdirectories are NOT recursively scanned (flat directory model)
- Files with non-`.toml` extensions are ignored with a debug-level log
- An empty specs directory is valid (zero config-driven sensors)

## Table Registration with DataFusion
- Each `TableSpec` is wrapped in a `SpecDrivenTableProvider` that implements DataFusion's `TableProvider` trait
- The `scan()` method on `SpecDrivenTableProvider` executes the table's fetch pipeline (BC-2.16.002) and returns an Arrow RecordBatch
- Virtual fields `sensor = "{sensor_id}"` and `source = "{table_name}"` are injected into results
- Spec-driven tables are queryable via the same `query` MCP tool (BC-2.11.001) and the same PrismQL syntax

## Auth Type Resolution
- The spec file declares the `auth_type` needed (e.g., `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`)
- Actual credentials are resolved from the credential store (CAP-004) at query time using the namespace `(client_id, sensor_id, credential_name)` where `sensor_id` matches the spec's `sensor_id`
- If no client has credentials configured for the spec's `sensor_id`, the spec loads successfully but its tables are marked unavailable (DEC-036)

## Invariants
- No BC-specific invariants beyond those in the domain spec. See DI-008 (client scoping), DI-030 (partial-failure isolation for spec loading), DI-021 (REQUIRED column enforcement).

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | TOML parse errors | Structured error with file path, line number, and parse error message |
| `E-SPEC-001` | Schema validation errors | Structured error with file path, TOML path to the invalid field, and corrective guidance (BC-2.16.009) |
| `E-SPEC-009` | Duplicate sensor_id across spec files | Second file is rejected, first wins |
| `E-SPEC-004` | Duplicate table_name within a sensor | The spec file is rejected entirely |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-036 | No client has credentials for a spec's sensor_id | Spec loads; tables marked unavailable; `list_sensor_specs` shows `status: no_credentials` |
| DEC-030 | One invalid spec file among many | Invalid spec rejected with errors; all valid specs load normally |
| Empty directory | sensor_specs_dir exists but contains no *.sensor.toml files | Zero spec-driven tables registered; no error |
| Subdirectory in specs_dir | subdirectory present | Not recursively scanned; ignored |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — valid spec | well-formed `crowdstrike.sensor.toml` | Tables registered; queryable via PrismQL |
| TOML parse error | malformed TOML | `E-SPEC-001` with line number; other specs unaffected |
| Duplicate sensor_id | two files with sensor_id="crowdstrike" | First loaded; second rejected with E-SPEC-009 |
| No credentials | spec loaded; no client credentials | Spec registered; tables show `status: no_credentials` |
| Empty directory | sensor_specs_dir is empty | Zero tables; no error |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify partial-failure isolation (one bad spec does not block others) |
| (placeholder) | VP to be assigned — verify OCSF field mappings registered with normalizer |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-008, DI-030 |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec, ConfigSnapshot |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Invariants section; added ## Error Conditions (from inline error handling); added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
