---
document_type: behavioral-contract
level: L3
version: "2.3"
status: active
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
modified: 2026-08-13
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "4a1f396"
traces_to: ["CAP-031"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.19.001: Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF

## Description

When an `.infusion.toml` spec file is loaded by the `InfusionRegistry`, each
`[[infusion.fields]]` entry must result in exactly one `InfusionUdfDescriptor` being
exported. This descriptor is consumed by `prism-query` (S-3.02) to register a
DataFusion `ScalarUDF`. Duplicate UDF names across multiple infusion specs are detected
at load time and rejected. Missing required fields cause the entire spec to be rejected.
This is INV-INFUSE-001.

## Preconditions

- The `InfusionRegistry` loader is scanning `{config_dir}/infusions/*.infusion.toml`
- A spec file contains at least one `[[infusion.fields]]` entry with valid `name`,
  `input_field`, `input_type`, and `output_type` fields

## Postconditions

- For each `[[infusion.fields]]` entry in the spec:
  - Exactly one `InfusionUdfDescriptor` is produced with: `name`, `input_type`, `output_type`,
    and a reference to the `InfusionSource` lookup function
  - The descriptor is added to `InfusionRegistry::udf_descriptors()` output
- **API-backed source wiring (two phases) — applies to `InfusionType::Plugin` and `InfusionType::HttpLookup`:**
  - **PARSE PHASE** (`InfusionLoader::load_all`): parses `*.infusion.toml` files and returns
    `(Vec<InfusionSpec>, Vec<InfusionError>)`. It does NOT construct `PluginInfusionSource`
    or `HttpLookupSource`, and does NOT attach anything as `descriptor.source`. At the end of
    the PARSE PHASE each plugin-type or http-lookup-type descriptor carries `Arc<NullSource>`
    as a placeholder source.
  - **RUNTIME PHASE** (`InfusionRegistry::load_spec_with_runtime`, and future boot-time wiring
    chained from it): branches on `InfusionType`:
    - `Plugin`: builds `PluginInfusionSource` — carrying `plugin_id` and `config` populated
      from the `InfusionSpec` — and attaches it as `descriptor.source` (an `Arc<dyn InfusionSource>`).
      The `plugin_id` and `config` values from the spec are NOT fields on `InfusionUdfDescriptor`
      directly; they live on `PluginInfusionSource`, reachable via `descriptor.source`.
    - `HttpLookup` (added ADR-040 v2.0 §D8.6): builds `HttpLookupSource` — carrying `http_lookup_config`
      from the `InfusionSpec` (base URL, JSONPath, credential config) — and attaches it as
      `descriptor.source`. Construction also performs SSRF validation; if `base_url` resolves
      to a private/loopback address and `PRISM_DTU_MODE` is unset, returns `E-INFUSE-011`
      and rejects the spec.
  - A plugin-type or http-lookup-type spec that reaches query execution still carrying
    `Arc<NullSource>` as `descriptor.source` — because `load_spec_with_runtime` was not invoked
    or failed silently — is a loading defect equivalent to `E-INFUSE-003`: `NullSource` returns
    `None` for all enrichment lookups, making enrichment silently inoperative.
- `prism-query` (S-3.02) consumes `udf_descriptors()` and registers each as a DataFusion `ScalarUDF`
- **`enrich_descriptor()` API (AC-3):** `InfusionRegistry::enrich_descriptor(name: &str)` returns an
  `EnrichStageDescriptor` (defined in `prism-spec-engine::infusion::enrich_descriptor`) for any loaded
  infusion. The descriptor carries:
  - `infusion_name` — the registry lookup key passed to `enrich_descriptor()`, which EQUALS
    `infusion_id` (NOT the human-readable `spec.name` field). This is the key used to look up the
    infusion in `InfusionRegistryInner::entries`.
  - `input_field` — the `input_field` from the spec's first `[[infusion.fields]]` entry (the join key;
    all fields share the same input column)
  - `output_columns` — when `spec.pipe_stage` is `Some`, `pipe_stage.adds_columns` in declared order
    (validated by `InfusionLoader::validate_pipe_stage_columns` to be a subset of field names); when
    `spec.pipe_stage` is `None`, all `[[infusion.fields]]` names in declaration order. A pipe stage
    is permitted to surface a STRICT SUBSET of infusion fields — this is by design to allow selective
    projection in a `| enrich` pipeline step.
  - `infusion_id` — the `infusion_id` from the spec root
  This descriptor is consumed by `prism-query` (S-3.02) to execute the `| enrich` pipe stage
  transformation. Unknown name returns `E-INFUSE-001`.
- **Duplicate `infusion_id` on `load_spec` / `load_spec_with_runtime`:** If a spec is loaded with an
  `infusion_id` that is already registered, the new spec REPLACES the prior entry (last-writer-wins
  semantics). The implementation MUST purge stale `udf_to_infusion` reverse-index entries for ALL
  UDF names that belonged to the OLD spec's fields before inserting the new spec's UDF names. Failure
  to purge creates dangling reverse-index entries pointing to the replaced `infusion_id`, which can
  cause `is_api_backed()` to return incorrect results. Note: `hot_reload` already handles this
  correctly by removing the old spec's UDF mappings before validation. `load_spec` and
  `load_spec_with_runtime` must apply the same purge logic.
- **Typed UDF output (INV-ENRICH-TYPED-001 — ADR-051 D1/D2/D6):** Each `InfusionUdfDescriptor.output_type`
  value maps to a specific Arrow `DataType` per the ADR-051 D1 table (updated by ADR-051 v1.2:
  `datetime` maps to `DataType::Timestamp(Microsecond, Some("UTC"))` per ADR-052 — NOT `DataType::Utf8`):
  `string` → `Utf8`; `integer` → `Int64`; `float` → `Float64`; `boolean` → `Boolean`;
  `json` → `Utf8` (JSON stored as string); `datetime` → `Timestamp(Microsecond, Some("UTC"))`.
  `InfusionAsyncUdf::return_type()` MUST return this mapped type via an `output_arrow_type()` helper
  (not always `DataType::Utf8`). `invoke_async_with_args()` MUST produce a typed output
  `ColumnarValue::Array` whose Arrow array type matches `return_type()` — dispatching on
  `output_arrow_type()` to build the correct array type (e.g., `Int64Array`, `Float64Array`,
  `BooleanArray`, `TimestampMicrosecondArray`). On coercion failure (string → typed value, or JSON
  Number precision mismatch), the output row is NULL and E-INFUSE-014 is emitted via
  `tracing::warn!(event_type = "infusion.coercion_failed", ...)`. Hardcoding `DataType::Utf8` as
  the return type for all infusion UDFs is a violation of this postcondition. For `output_type =
  "json"` fields the ENRICH-1 list-dispatch path in `invoke_async_with_args` is RETAINED. For all
  other typed output types (`integer`, `float`, `boolean`, `datetime`), a JSON-list string input
  (detected by leading `[`) produces NULL + E-INFUSE-014 at runtime.
- **Plugin-type field projection (D3 / E-INFUSE-013 sub-condition 8 — ADR-051 D3):** Every
  `[[infusion.fields]]` entry in a `type = "plugin"` infusion MUST declare `source_column`.
  `InfusionLoader::validate` (spec-load time) rejects any plugin-type field that lacks
  `source_column` with E-INFUSE-013 sub-condition 8. A plugin infusion source
  (`PluginInfusionSource` / DTU clone path) returns a JSON object containing multiple fields;
  without `source_column`, `project_value()` serializes the entire response object — the root
  cause of DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 (Failure A: doubly-encoded JSON). This validation
  MUST fire at spec-load time, before any UDF is registered. For `type = "http_lookup"` fields
  `source_column` is already practically required (existing NVD spec is compliant). For
  `type = "local_lookup"` fields `source_column` is OPTIONAL.
- **Scope boundary — pipe-mode `| enrich` runtime execution (S-3.01 anchor, NOT this BC):**
  This BC's contract surface is fully satisfied when:
  (a) each `[[infusion.fields]]` entry registers exactly one DataFusion scalar UDF (SQL-mode
      `SELECT udf(field)` path, wired by `prism-query` S-3.02), AND
  (b) `InfusionRegistry::enrich_descriptor(name)` returns a well-formed `EnrichStageDescriptor`.
  The pipe-stage `| enrich` RUNTIME dispatch — i.e., resolving `Ast::Pipe { stage: Enrich, ... }`
  through materialization into actual RecordBatch column hydration — is universally unimplemented
  for ALL pipe stages and is owned by **S-3.01** (the PrismQL pipe-execution story). This is not
  a S-1.14-REDO gap; it is a system-wide materialization gap. Fresh-context adversaries MUST NOT
  flag the `| enrich` pipe-mode no-op as a S-1.14-REDO or BC-2.19.001 defect. Architect ruling:
  S-1.14-REDO Q1 (2026-06-18). The `enrich_descriptor()` API is the correct handoff boundary.
- **Duplicate UDF name detection:** If two specs declare the same `[[infusion.fields]]` name
  (e.g., both declare `name = "geoip_country"`), the second spec is rejected with:
  `E-INFUSE-002: "Duplicate UDF name 'geoip_country' in '{path2}' — already registered from '{path1}'."`
  The first-registered spec is retained.
- **Missing required field:** Spec is rejected with actionable error per missing field

## Invariants

- INV-INFUSE-001: Each `[[infusion.fields]]` entry must register exactly one DataFusion scalar UDF,
  and the registered UDF's `return_type()` must return the Arrow DataType mapped from `output_type`
  per ADR-051 D1 (e.g., `integer` → `Int64`, `float` → `Float64`, `datetime` → `Timestamp(µs,UTC)`)
- INV-ENRICH-TYPED-001: All enrichment UDFs registered from `[[infusion.fields]]` entries produce
  typed output per the ADR-051 D1 mapping. No enrichment UDF may return `DataType::Utf8` for a
  field whose `output_type` is `integer`, `float`, `boolean`, or `datetime`. Typed input columns
  (`integer`, `float`, `boolean`, `datetime`) require scalar input; JSON-list input to a
  typed-output UDF produces NULL + E-INFUSE-014. Violations of this invariant are a P1 finding
  in adversarial review (machine-checkable: grep `DataType::Utf8` in `return_type()` implementations)
- UDF names are global within a DataFusion `SessionContext`; duplicates are a load-time error
- `prism-spec-engine` does NOT depend on DataFusion — it exports `InfusionUdfDescriptor`
  structs; `prism-query` handles actual DataFusion registration
- A spec with 3 `[[infusion.fields]]` entries produces exactly 3 `InfusionUdfDescriptor` objects

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-INFUSE-001` | `enrich_descriptor(name)` called with unknown infusion name | Returns `InfusionError::UnknownInfusion { name }` |
| `E-INFUSE-002` | Duplicate UDF name across specs | Second spec rejected; first retained; `ERROR` log |
| `E-INFUSE-003` | Missing required field in spec (`infusion_id`, `[[infusion.fields]]`) | Spec rejected with per-field error list; other specs continue |
| `E-INFUSE-004` | Source type not recognized (`type = "unknown"`) | Spec rejected; `E-INFUSE-004: "Unknown source type 'unknown'. Valid types: maxmind_mmdb, csv, json_lookup, plugin, http_lookup."` |
| `E-INFUSE-012` | Infusion source file (CSV, JSON-lookup, or MMDB) exceeds `MAX_SOURCE_FILE_BYTES` (100 MiB = 104,857,600 bytes) at load or hot-reload time | Spec rejected before any bytes are read into memory; `E-INFUSE-012: "infusion source file '{path}' exceeds maximum size ({size} bytes > {limit} bytes); reduce the file or raise MAX_SOURCE_FILE_BYTES"`. Other infusion specs continue loading. CWE-400 guard. |
| `E-INFUSE-013` | An `[[infusion.fields]]` entry or spec-level attribute fails parse-time validation in `InfusionLoader::validate` | Spec rejected with per-field message: `"E-INFUSE-013: invalid field name '{field}' in infusion spec '{spec_path}': {message}"`. Sub-conditions: (1) UDF name fails `^[a-zA-Z][a-zA-Z0-9_]*$` pattern (CWE-20; prevents DataFusion SQL injection); (2) `url_template` missing `${input}` placeholder; (3) `base_url` is empty; (4) `method` is not `GET` or `POST`; (5) `response_path` is empty; (6) `plugin_ref` contains path-traversal characters (CWE-22); **(7) `output_type` value not in `{string, integer, float, boolean, json, datetime}` — unknown output type rejected at spec-load time; `datetime` maps to `Timestamp(µs,UTC)` per ADR-051 v1.2 / ADR-052**; **(8) `type = "plugin"` field lacks `source_column` — without source_column the entire plugin response object is serialized, causing doubly-encoded JSON; DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 root cause closure)**. Other infusion specs continue loading. |
| `E-INFUSE-014` | A UDF's coercion from the projected value to the declared `output_type` fails at runtime in `invoke_async_with_args()` | Output row is NULL (not a panic, not passthrough string, not empty string). Runtime `tracing::warn!` emitted with `event_type = "infusion.coercion_failed"`. Message: `"E-INFUSE-014: enrichment field '{field_name}' (infusion '{infusion_id}'): declared output_type is '{declared_type}', but projected value '{truncated_value}' (first 50 chars) cannot be coerced; row produces NULL"`. NOT propagated as a query error (query succeeds with NULL in the typed column). Triggers: JSON-list input to typed-output UDF (leading `[`); `i64::from_str()` failure for integer; `f64::from_str()` failure for float; unrecognized value for boolean; `parse_datetime_to_micros` failure for datetime. `{truncated_value}` is the first 50 characters of the projected string (AD-017 credential exposure guard). BC-2.16.002 catalog row required for `event_type = "infusion.coercion_failed"` (SAP-1). |
| `E-INFUSE-015` | `build_http_client_with_timeout` fails during the RUNTIME PHASE of an `HttpLookup`-type infusion spec in `load_spec_with_runtime` or `hot_reload` — TLS init failure at `reqwest` client construction | Spec rejected; `InfusionError::HttpClientBuildFailed { detail }` returned; other infusion specs continue loading. Display: `"E-INFUSE-015: infusion HTTP client build failed (TLS init): {detail}"`. Effectively unreachable in production under the workspace-wide `rustls-tls` mandate (ADR-050 D3); reachable only if the TLS backend is misconfigured or `native-tls` is erroneously activated. Not retryable — the client build error is structural, not transient. When triggered during `hot_reload`, the previous registry is retained (BC-2.19.004 atomicity contract). DEFECT-ADAPTER-TLS-XDOME-LIVE-001 F-2; verified by RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015`). |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-001 | Spec with 0 `[[infusion.fields]]` entries | Rejected: at least one field required per INV-INFUSE-001 |
| EC-19-002 | Spec with 10 fields, all valid | 10 `InfusionUdfDescriptor` objects exported |
| EC-19-003 | Hot reload adds a new spec with 3 fields | 3 new descriptors exported; `prism-query` notified to register new UDFs; old UDFs from other specs unchanged |
| EC-19-004 | Spec loaded but source file (MMDB, CSV) missing | Spec is registered but `InfusionSource::enrich_single` returns `None` for all lookups; spec is not rejected (source file may be mounted later) |
| EC-19-005 | Source file (CSV, JSON-lookup, or MMDB) is exactly `MAX_SOURCE_FILE_BYTES` bytes (boundary value) | Spec is accepted; the guard fires only for sizes **strictly greater than** the limit |
| EC-19-006 | Source file is `MAX_SOURCE_FILE_BYTES + 1` bytes | Spec is rejected with `E-INFUSE-012`; `{size}` = `MAX_SOURCE_FILE_BYTES + 1`, `{limit}` = `MAX_SOURCE_FILE_BYTES`; no bytes are read into memory |
| EC-19-007 | Hot-reload triggered while an oversized source file is on disk (file grew beyond limit after initial load) | Hot-reload path rejects the spec with `E-INFUSE-012`; the previously loaded (acceptable-size) version of the spec remains active (atomic swap semantics per BC-2.19.004) |
| EC-19-008 | JSON-list string input (leading `[`) provided to a typed-output (`integer`, `float`, `boolean`, or `datetime`) UDF at runtime | Output row is NULL; `E-INFUSE-014` warning emitted with `declared_type` set to the field's `output_type`. For `output_type = "json"` fields the ENRICH-1 list-dispatch path is RETAINED — this edge case applies to non-json typed output only. |
| EC-19-009 | Plugin-type `[[infusion.fields]]` entry with no `source_column` declared (e.g., `threatintel.infusion.toml` pre-ADR-051 state) | Spec rejected at parse time with `E-INFUSE-013` sub-condition 8; message includes `"plugin-type field '{name}' in infusion '{infusion_id}' must declare 'source_column' to project a specific field from the plugin response object; without source_column the full response object is serialized"` |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-19-001-happy | `geoip.infusion.toml` with 1 valid field | 1 `InfusionUdfDescriptor` exported; `geoip_country` UDF registered | AC-1 |
| TV-19-001-10fields | Spec with 10 valid fields | 10 descriptors exported exactly | EC-19-002 |
| TV-19-001-dup | Two specs both declare `geoip_country` | Second spec rejected with `E-INFUSE-002`; first retained | Error row 1 |
| TV-19-001-empty | Spec with 0 `[[infusion.fields]]` | Rejected: zero fields | EC-19-001 |
| TV-19-001-enrich-desc | `geoip.infusion.toml` with 4 fields (`geoip_country`, `geoip_city`, `geoip_asn`, `geoip_is_tor`), NO `pipe_stage`; call `enrich_descriptor("geoip")` | Returns `EnrichStageDescriptor { infusion_name: "geoip", input_field: "device_ip", output_columns: ["geoip_country","geoip_city","geoip_asn","geoip_is_tor"], infusion_id: "geoip" }`. `infusion_name` == lookup key == `infusion_id`; NOT `spec.name` (human name). `output_columns` = all 4 field names in declaration order (no pipe_stage). | AC-3 |
| TV-19-001-enrich-desc-pipe | Same spec with `[infusion.pipe_stage]\nadds_columns = ["geoip_country","geoip_city"]`; call `enrich_descriptor("geoip")` | Returns `EnrichStageDescriptor { infusion_name: "geoip", input_field: "device_ip", output_columns: ["geoip_country","geoip_city"], infusion_id: "geoip" }`. `output_columns` = `adds_columns` (2-element subset); `geoip_asn` and `geoip_is_tor` are excluded. | A1 pipe-stage subset |
| TV-19-001-enrich-desc-unknown | Call `enrich_descriptor("nonexistent_infusion")` on empty registry | Returns `Err(InfusionError::UnknownInfusion { name: "nonexistent_infusion" })` | E-INFUSE-001 |
| TV-19-001-overwrite-purge | Load spec A (infusion_id="geoip", fields=[geoip_country, geoip_asn]); then load spec B (infusion_id="geoip", fields=[geoip_city]) | After B loads: `udf_to_infusion` contains only `geoip_city → "geoip"`; `geoip_country` and `geoip_asn` keys are ABSENT from `udf_to_infusion` (stale entries purged). `is_api_backed("geoip_country")` returns `false` (unknown → false). | A3 overwrite-purge |
| TV-19-001-oversized-csv | A CSV infusion spec referencing a source file whose `fs::metadata().len()` = `MAX_SOURCE_FILE_BYTES + 1` bytes (104,857,601 bytes) | Returns `Err(InfusionError::SourceFileTooLarge { path: "<path>", size: 104857601, limit: 104857600 })`; display string starts with `"E-INFUSE-012: infusion source file '"`; zero bytes of the file are read into memory; other infusion specs in the same registry load continue unaffected | EC-19-006; SEC-001; CWE-400 |
| TV-19-001-at-limit-csv | A CSV source file whose `fs::metadata().len()` = `MAX_SOURCE_FILE_BYTES` bytes exactly (104,857,600 bytes) | Spec accepted; `Ok(descriptor)` returned; the boundary value is NOT rejected | EC-19-005 (strictly-greater-than semantics) |
| TV-19-001-oversized-mmdb | An MMDB infusion spec referencing a MaxMind MMDB file whose `fs::metadata().len()` = `MAX_SOURCE_FILE_BYTES + 1` bytes | Returns `Err(InfusionError::SourceFileTooLarge { .. })`; MMDB reader is never opened; display starts with `"E-INFUSE-012:"` | EC-19-006; MMDB variant |
| TV-19-001-oversized-json | A JSON-lookup infusion spec referencing a JSON file whose `fs::metadata().len()` = `MAX_SOURCE_FILE_BYTES + 1` bytes | Returns `Err(InfusionError::SourceFileTooLarge { .. })`; JSON bytes never read; display starts with `"E-INFUSE-012:"` | EC-19-006; JSON-lookup variant |
| TV-19-001-typed-integer | Infusion field `threat_score` with `output_type = "integer"`, `source_column = "threat_score"`, projected JSON Number value `95` from plugin response | UDF `return_type()` returns `DataType::Int64`; `invoke_async_with_args()` produces `Int64Array` with value `95`; `filter threat_score >= 75` evaluates as numeric comparison (correct) | INV-ENRICH-TYPED-001; ADR-051 D1 integer; ADR-051 D5 |
| TV-19-001-typed-float | Infusion field `cvss_base_score` with `output_type = "float"`, `source_column = "baseScore"`, projected string value `"8.1"` from HTTP response | UDF `return_type()` returns `DataType::Float64`; produces `Float64Array` with value `8.1`; `filter cvss_base_score >= 8.0` evaluates as `Float64 >= Float64(8.0)` (numeric, correct) | INV-ENRICH-TYPED-001; ADR-051 D1 float; ADR-051 D5 |
| TV-19-001-typed-boolean | Infusion field `threat_is_known_malicious` with `output_type = "boolean"`, projected string value `"true"` | UDF `return_type()` returns `DataType::Boolean`; produces `BooleanArray` with value `true`; case-insensitive coercion: `"true"/"1"/"yes"` → `true`, `"false"/"0"/"no"` → `false` | INV-ENRICH-TYPED-001; ADR-051 D1 boolean; ADR-051 D2 |
| TV-19-001-typed-datetime | Infusion field `enriched_event_time` with `output_type = "datetime"`, projected RFC-3339 string `"2026-07-03T12:00:00Z"` | UDF `return_type()` returns `DataType::Timestamp(Microsecond, Some("UTC"))`; produces `TimestampMicrosecondArray` with microseconds-since-epoch value for `2026-07-03T12:00:00Z`; consistent with sensor Datetime columns (ADR-052) enabling cross-column predicates | INV-ENRICH-TYPED-001; ADR-051 D1 datetime; ADR-051 v1.2; ADR-052 |
| TV-19-001-coerce-fail-integer | Infusion field `threat_score` with `output_type = "integer"`, projected string value `"not-a-number"` | Output row is NULL; `E-INFUSE-014` warning emitted with `declared_type = "integer"`, `truncated_value = "not-a-number"` (first 50 chars); query succeeds with NULL in `threat_score` column | ADR-051 D2 coercion failure; EC-19-008 |
| TV-19-001-coerce-fail-datetime | Infusion field `enriched_event_time` with `output_type = "datetime"`, projected string `"not-a-date"` | Output row is NULL; `E-INFUSE-014` emitted with `declared_type = "datetime"`; parse failure via `parse_datetime_to_micros` | ADR-051 D2; datetime coercion failure |
| TV-19-001-json-list-typed-output | JSON-list string `"[\"hash1\",\"hash2\"]"` fed as input to a field with `output_type = "integer"` (leading `[` detected) | Output row is NULL; `E-INFUSE-014` warning emitted; list cannot coerce to scalar integer | EC-19-008; ADR-051 D4 |
| TV-19-001-plugin-no-source-col | Plugin-type infusion spec with a `[[infusion.fields]]` entry that has no `source_column` declared | Spec rejected at parse time with `E-INFUSE-013` sub-condition 8; message contains `"must declare 'source_column'"` | EC-19-009; ADR-051 D3 |
| TV-19-001-unknown-output-type | Infusion spec with `output_type = "bytes"` (not in recognized set) | Spec rejected with `E-INFUSE-013` sub-condition 7; `"bytes"` not in `{string, integer, float, boolean, json, datetime}` | ADR-051 D3 sub-condition 7 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-048 | `InfusionRegistry::load_spec()` with N valid, distinct field entries produces exactly N `InfusionUdfDescriptor` objects in the output; duplicate UDF names produce `Err(E-INFUSE-002)` rather than silently merging | Kani |

## Related BCs

- BC-2.19.002 — Per-Query Dedup Cache (governs how UDF calls are deduplicated)
- BC-2.19.003 — API-Backed UDF Rejection in Detection Rules (INV-INFUSE-003)
- BC-2.19.004 — Hot Reload Atomicity (CI-002 pattern applies to infusion registry)
- BC-2.13.009 — Rule-to-SQL Compilation (detection rules that reference infusion UDFs)

## Architecture Anchors

- AD-020: Infusions — enrichment framework
- ADR-051: Typed & Consistent Enrichment UDF Output — output_type → Arrow DataType mapping (ACCEPTED v1.3)
- ADR-052: PrismQL Native Temporal Typing — Datetime → Timestamp(µs,UTC) migration (merged PR #214)
- `specs/architecture/infusions.md` — `InfusionUdfDescriptor`, spec structure, UDF registration
- S-1.14 Task 4: `infusion/udf.rs` — UDF descriptor export

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-001, AC-1)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Load `geoip.infusion.toml` → verify `geoip_country` UDF registered."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-031 |
| Story Invariant | INV-INFUSE-001 |
| ADR | AD-020 |
| Story | S-1.14 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.3 | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 | 2026-08-13 | product-owner | **E-INFUSE-015 (HttpClientBuildFailed) added to §Error Conditions (F-2 obligation).** New error row documents `build_http_client_with_timeout` failure during `HttpLookup`-type RUNTIME PHASE (`load_spec_with_runtime` / `hot_reload`): spec rejected; other specs continue; effectively unreachable under ADR-050 `rustls-tls` mandate; when triggered during `hot_reload` the previous registry is retained per BC-2.19.004 atomicity contract. Verified by RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015`); code commit a1864d3eb. No existing semantics changed. |
| 2.2 | ADR-051-typed-enrichment-udf-output | 2026-07-05 | product-owner | **ADR-051 (ACCEPTED v1.3) typed UDF output amendments.** (1) **New postcondition — Typed UDF output (INV-ENRICH-TYPED-001 / ADR-051 D1/D2/D6):** `InfusionAsyncUdf::return_type()` MUST return the Arrow DataType mapped from `output_type` per ADR-051 D1: `string` → `Utf8`, `integer` → `Int64`, `float` → `Float64`, `boolean` → `Boolean`, `json` → `Utf8`, `datetime` → `Timestamp(Microsecond, Some("UTC"))` (per ADR-051 v1.2/ADR-052 — NOT `Utf8`); `invoke_async_with_args()` builds matching typed array; coercion failure → NULL + E-INFUSE-014; ENRICH-1 list-dispatch retained for json-typed fields only. (2) **New postcondition — Plugin-type field projection (ADR-051 D3 / E-INFUSE-013 sub-condition 8):** plugin-type `[[infusion.fields]]` without `source_column` rejected at spec-load time. (3) **INV-INFUSE-001 extended:** now includes `return_type()` → ADR-051 D1 mapping requirement. (4) **New invariant INV-ENRICH-TYPED-001** added to Invariants section. (5) **E-INFUSE-013 added to Error Conditions** with all 8 sub-conditions including new (7) unknown `output_type` and (8) plugin-type field missing `source_column`. (6) **New E-INFUSE-014 (TypeCoercionFailed)** added: per-row runtime warning on coercion failure; NULL output; `event_type = "infusion.coercion_failed"` tracing emission; BC-2.16.002 catalog row required (SAP-1). (7) **Edge cases EC-19-008** (JSON-list to typed-output UDF → NULL + E-INFUSE-014) and **EC-19-009** (plugin-type without source_column → E-INFUSE-013 sub-condition 8) added. (8) **Test vectors TV-19-001-typed-{integer,float,boolean,datetime}**, **TV-19-001-coerce-fail-{integer,datetime}**, **TV-19-001-json-list-typed-output**, **TV-19-001-plugin-no-source-col**, **TV-19-001-unknown-output-type** added. (9) ADR-051/ADR-052 added to Architecture Anchors. v2.1→v2.2. |
| 2.1 | SEC-001-CWE-400-source-file-size-bound | 2026-06-18 | product-owner | **E-INFUSE-012 source-file size guard added (SEC-001, CWE-400, human-approved in-scope fix).** (1) Error Conditions table: new row `E-INFUSE-012` — infusion source file (CSV, JSON-lookup, MMDB) exceeds `MAX_SOURCE_FILE_BYTES` (100 MiB = 104,857,600 bytes) at load or hot-reload time; spec rejected before any bytes are read; other specs continue. (2) Edge Cases: EC-19-005 (at-limit boundary, accepted), EC-19-006 (over-limit, rejected), EC-19-007 (hot-reload with file that grew beyond limit). (3) Canonical Test Vectors: TV-19-001-oversized-csv, TV-19-001-at-limit-csv, TV-19-001-oversized-mmdb, TV-19-001-oversized-json — covering all three source types and the boundary-value semantics. No existing BC semantics weakened. |
| 2.0 | round-2-adversary-cluster-A1-A2-A3 | 2026-06-18 | product-owner | **Round-2 adversary cluster adjudication (A1/A2/A3).** **(A1 — `output_columns` source-of-truth):** Ruled Option (b): `output_columns = pipe_stage.adds_columns` (validated subset of field names, in declared order) when `pipe_stage` is present, else all field names in declaration order. A pipe stage may legitimately project a strict subset. `validate_pipe_stage_columns` subset validation is CORRECT as-is (no tightening needed); no code change to `enrich_descriptor()` required. Added TV-19-001-enrich-desc-pipe test vector. **(A2 — `infusion_name` definition):** Corrected prose: `infusion_name` = registry lookup key == `infusion_id`, NOT the human `spec.name` field. Updated TV-19-001-enrich-desc notes to state "NOT `spec.name`." **(A3 — duplicate `infusion_id` on `load_spec`):** Ruled last-writer-wins replacement; implementer MUST purge stale `udf_to_infusion` entries for the old spec's fields on overwrite (same pattern as `hot_reload`). Added postcondition clause and TV-19-001-overwrite-purge test vector. |
| 1.9 | S-1.14-REDO-Q1-scope-clarification | 2026-06-18 | product-owner | **Scope clarification per architect ruling S-1.14-REDO Q1 (2026-06-18).** (1) Added `enrich_descriptor()` API postcondition (AC-3): `InfusionRegistry::enrich_descriptor(name)` returns `EnrichStageDescriptor` carrying `infusion_name`, `input_field`, `output_columns`, and `infusion_id`; unknown name returns `E-INFUSE-001`. (2) Added explicit "Scope boundary — pipe-mode `\| enrich` runtime execution" postcondition clarifying that this BC's contract is satisfied by UDF descriptor registration (SQL-mode) + `enrich_descriptor()` returning a well-formed `EnrichStageDescriptor`; `\| enrich` pipe RUNTIME dispatch (Ast::Pipe arm, RecordBatch hydration) is universally unimplemented for ALL pipe stages and is owned by **S-3.01** — not a S-1.14-REDO gap; fresh-context adversaries must not flag this as BC-2.19.001 defect. (3) Added `E-INFUSE-001` to Error Conditions table (was tested but absent). (4) Added AC-3 canonical test vectors `TV-19-001-enrich-desc` and `TV-19-001-enrich-desc-unknown`. |
| 1.8 | PIVOT-002-bc-amendment-http-lookup | 2026-06-17 | product-owner | **Added `http_lookup` as valid `InfusionType` source per ADR-040 v2.0 §D8.3 and error-taxonomy.md v1.88.** (1) E-INFUSE-004 valid-types list: `maxmind_mmdb, csv, json_lookup, plugin` → `maxmind_mmdb, csv, json_lookup, plugin, http_lookup`. (2) Two-phase source wiring postcondition expanded: heading renamed from "Plugin-type source wiring" to "API-backed source wiring" to cover both `Plugin` and `HttpLookup` types; RUNTIME PHASE now explicitly branches on `InfusionType` — `Plugin` path unchanged, `HttpLookup` path (ADR-040 §D8.6) documents `HttpLookupSource` construction with SSRF validation and `E-INFUSE-011` rejection. `NullSource` defect note extended to cover both `Plugin` and `HttpLookup`. Scope confirmed: `HttpLookup` flows through the same `InfusionLoader::parse` (PARSE PHASE) + `InfusionRegistry::load_spec_with_runtime` (RUNTIME PHASE) two-phase path already specified by this BC — no sibling BC needed. |
| 1.7 | PIVOT-001-LOW-2-regression-fix | 2026-06-15 | product-owner | Regression fix (PIVOT-001 LOW-2): v1.6 reword incorrectly re-introduced `load_all` as constructor of `PluginInfusionSource`. Corrected to accurate two-phase model: PARSE PHASE (`load_all`) returns `(Vec<InfusionSpec>, Vec<InfusionError>)` and does NOT construct `PluginInfusionSource`; RUNTIME PHASE (`load_spec_with_runtime`) builds `PluginInfusionSource` (carrying `plugin_id`/`config` from the spec) and attaches it as `descriptor.source`. Reverses the v1.6 regression; restores and extends the v1.5 accuracy. |
| 1.6 | OBS-plugin-id-type-correction | 2026-06-15 | product-owner | Prose precision fix (OBS finding): `plugin_id`/`config` are NOT fields on `InfusionUdfDescriptor` — they live on `PluginInfusionSource`, reachable via `descriptor.source`. Reworded plugin-type source wiring postcondition to name `PluginInfusionSource` as the carrier struct and `descriptor.source` as the access path. Contract semantics unchanged; implementation was already correct. |
| 1.5 | PIVOT-001-LOCAL-HIGH-2 | 2026-06-14 | product-owner | Corrected plugin-type source wiring postcondition (PIVOT-001 LOCAL HIGH-2). Prior wording named `InfusionLoader::load_all` as producer of real `Arc<PluginInfusionSource>` — incorrect: `load_all` returns specs+errors, not runtime-wired descriptors. Reworded to name `InfusionRegistry::load_spec_with_runtime` (and future boot-time runtime wiring) as the step that attaches the real `PluginInfusionSource`; `load_all` role limited to parsing and populating `plugin_id`/`config` fields. Anti-NullSource defect definition retained (a plugin-type spec reaching query execution with `NullSource` is E-INFUSE-003 equivalent). No line-number pins (TD-VSDD-091). |
| 1.4 | S-DEMO-ENRICHMENT-PIVOT-001-po-sign-off | 2026-06-14 | product-owner | Closed NullSource gap: added plugin-type source wiring postcondition — plugin-type descriptors MUST carry Arc<PluginInfusionSource> (not NullSource) or loading is a defect equivalent to E-INFUSE-003. Needed for PIVOT-001 AC-003 Phase 3 / NullSource-replacement task. |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-048); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
