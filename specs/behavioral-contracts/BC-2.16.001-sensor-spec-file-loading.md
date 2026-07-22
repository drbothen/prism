---
document_type: behavioral-contract
level: L3
version: "1.9"
status: active
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "fc9d874"
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
- Each `.toml` file in the sensor specs directory is parsed into a `SensorSpec` struct containing: `sensor_id`, `name`, `auth_type` (oauth2_client_credentials | bearer_static | cookie_roundtrip | api_key | custom_via_plugin | token_exchange), `base_url`, `tables` (Vec<TableSpec>), `rate_limit_hints`, and `version`
- Each `TableSpec` within a `SensorSpec` is registered as a DataFusion table in the query engine's catalog, following the same pattern as external sensor tables (CAP-015)
- Table names follow the convention `{sensor_id}_{table_name}` (e.g., `sentinelone_alerts`, `sentinelone_agents`)
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
- The spec file declares the `auth_type` needed (e.g., `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`, `custom_via_plugin`, `token_exchange`)
- Actual credentials are resolved from the credential store (CAP-004) at query time using the namespace `(client_id, sensor_id, credential_name)` where `sensor_id` matches the spec's `sensor_id`
- If no client has credentials configured for the spec's `sensor_id`, the spec loads successfully but its tables are marked unavailable (DEC-036)

## Invariants
- No BC-specific invariants beyond those in the domain spec. See DI-008 (client scoping), DI-030 (partial-failure isolation for spec loading), DI-021 (REQUIRED column enforcement).

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | TOML parse errors | Structured error with file path, line number, and parse error message |
| `E-SPEC-001` | Schema validation errors | Structured error with file path, TOML path to the invalid field, and corrective guidance (BC-2.16.009) |
| `E-SPEC-009` | Duplicate `sensor_id` across spec files (two files declare the same sensor_id) | Second file is rejected, first wins |
| `E-SPEC-017` | Spec `sensor_id` does not case-sensitively match the filename stem (e.g., `crowdstrike.sensor.toml` with `sensor_id: "falcon"`) | File is rejected at load time; error includes filename and declared sensor_id. Enforces `{sensor_id}.sensor.toml` naming convention (INV-PARITY-002 in BC-2.16.013). **Enforcement contract (F-LP4-HIGH-003 + F-LP4-MED-002 closure):** `prism-core` exposes `SpecErrorCode::ESpec017` variant (added as part of PLUGIN-MIGRATION-001-D scope per D-737 Decision 3). `prism-spec-engine::spec_parser::SpecLoader::load_all()` (or `parse_spec_directory()`) emits E-SPEC-017 when the parsed `sensor_id` does not match the filename stem — these functions receive the file path and therefore have filename context. `SpecLoader::parse(toml_input: &str)` does NOT emit E-SPEC-017 because it accepts only the TOML content string with no filename context; callers using `parse()` directly cannot trigger this error. RG-09 and HS-018 must use `load_all()` or `parse_spec_directory()` as the test driver (not `parse()`) to exercise the filename-stem check. |
| `E-SPEC-004` | Duplicate table_name within a sensor | The spec file is rejected entirely |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-036 | No client has credentials for a spec's sensor_id | Spec loads; tables marked unavailable; `list_sensor_specs` shows `status: no_credentials` |
| DEC-030 | One invalid spec file among many | Invalid spec rejected with errors; all valid specs load normally |
| Empty directory | sensor_specs_dir exists but contains no *.sensor.toml files | Zero spec-driven tables registered; no error |
| Subdirectory in specs_dir | subdirectory present | Not recursively scanned; ignored |

## Known Gaps

| ID | Gap | Owner Story | Justification |
|----|-----|-------------|---------------|
| KG-006-001 | DEC-036 DataFusion-level unavailability marking untested at integration layer. The AC-006 parse-time assertion (`credential_refs.is_empty()`) verifies the spec loads without error, but the runtime behavior "tables marked unavailable at DataFusion catalog registration" is not exercised. Per AD-015, `prism-spec-engine` MUST NOT import DataFusion; catalog registration is `prism-query`'s responsibility. | S-3.02 (prism-query DataFusion catalog wiring) | Architectural boundary: `prism-spec-engine` exports descriptors only; the descriptor's `availability_status` field is consumed by `prism-query` at registration time, which is outside this story's crate boundary. The parse-time portion of DEC-036 is fully verified. |

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
| (none) | Partial-failure isolation is a behavioral loop property (integration test); OCSF field mapping registration is a side-effectful normalizer call; VP-023 covers the critical panic-safety property for spec parsing; no additional formal VP. |

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
| 1.9 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-054 D1 amendment: §Postconditions `SensorSpec` `auth_type` parenthetical updated from `(oauth2/bearer/cookie/api_key)` to full 6-value canonical set `(oauth2_client_credentials | bearer_static | cookie_roundtrip | api_key | custom_via_plugin | token_exchange)`; §Auth Type Resolution example list extended to include `custom_via_plugin` and `token_exchange`. input-hash updated "76729b7"→"fc9d874" (prd.md + capabilities.md current hash). modified date 2026-07-22. |
| 1.8 | S-3.13-LOCAL-adversary-OBS-1 | 2026-06-16 | product-owner | Prose drift fix: §Postconditions table-name convention corrected from DOT separator to UNDERSCORE (`{sensor_id}_{table_name}`, e.g., `sentinelone_alerts`, `sentinelone_agents`). Aligns with BC-2.11.001 authoritative convention (EC-11-033..036: `crowdstrike_alerts`, `armis_alerts`), `table_registry.rs::register_sensor` `format!("{}_{}", ...)` implementation, and E-QUERY-037 message format. DOT form `sentinelone.alerts` would require DataFusion identifier quoting; underscore is unquoted and ratified. No semantic or behavioral change — convention was always underscore; prose was wrong. |
| 1.7 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 auto-promotion at merge: PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status draft→active (lifecycle_status was already active). |
| 1.6 | FB-IMPL-1-PO | 2026-05-21 | product-owner | F-LP1-HIGH-005 closure (Option a — narrow AC-006): Added §Known Gaps section with KG-006-001 — DEC-036 DataFusion-level unavailability marking is not exercisable in prism-spec-engine test harness due to AD-015 (prism-spec-engine MUST NOT import DataFusion; catalog registration is prism-query S-3.02 scope). The parse-time portion of DEC-036 (credential_refs.is_empty() on load) remains the in-scope PASS criterion for AC-006. Gap will close in S-3.02. |
| 1.5 | FB-IMPL-P4-PO fix-burst-4 | 2026-05-20 | product-owner | F-LP4-HIGH-003 + F-LP4-MED-002 closure: Expanded E-SPEC-017 row with explicit enforcement contract — (1) `prism-core` exposes `SpecErrorCode::ESpec017` variant (D-737 Decision 3 scope expansion); (2) `SpecLoader::load_all()` / `parse_spec_directory()` emits E-SPEC-017 (has filename context); (3) `SpecLoader::parse(toml_input: &str)` does NOT emit E-SPEC-017 (no filename context); (4) RG-09 / HS-018 must use `load_all()` / `parse_spec_directory()` as test driver, not `parse()`. This closes F-LP4-MED-002 (RG-09 driver ambiguity) and F-LP4-HIGH-003 (enforcement scope gap). |
| 1.4 | FB-IMPL-P2-PO fix-burst-2 | 2026-05-20 | product-owner | F-002 closure (pass-2 adversarial, PO scope): Added E-SPEC-017 row to §Error Conditions — filename-stem-vs-sensor_id mismatch now has its own error code (registered in error-taxonomy.md v1.41). Clarified E-SPEC-009 row to make clear it covers ONLY duplicate-sensor_id, not filename-stem mismatch. BC-2.16.013 §Error Conditions v1.2 and HS-018 cite E-SPEC-017 consistently. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Invariants section; added ## Error Conditions (from inline error handling); added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
