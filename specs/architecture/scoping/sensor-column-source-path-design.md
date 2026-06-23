---
title: "Sensor Column source_path — Nested/Array Extraction Design"
status: approved
decision_authority: architect
traces_to: ARCH-INDEX.md
supersedes: PIVOT-003 convention of brackets-in-name (iocs[].value)
related_bc: BC-2.06.019
related_adr: ADR-028, ADR-033
created: 2026-06-23
---

# Sensor Column `source_path` — Nested/Array Extraction Design

## Problem Statement

PIVOT-003 shipped sensor TOML specs (cyberint, crowdstrike) with column `name` fields
that encode JSON traversal paths: `iocs[].value`, `behaviors[].ioc_type`,
`alert_data.ip`, `ioc.value`. These are broken in two independent ways:

1. `build_column_array()` in `prism-bin/src/spec_driven_adapter.rs` calls
   `record.get(col_name)` with a flat string key. A key of `"iocs[].value"` never
   exists as a top-level field in any JSON object — it produces NULL for every row.

2. `ColumnMapper::map_record()` in `prism-spec-engine/src/column_mapping.rs` performs
   the same flat `raw.get(&col.name)` lookup — same NULL result.

3. The pipe field-path grammar rejects `[]` characters in column references, so
   `| enrich threat_intel(iocs[].value)` cannot even parse.

A working nested extractor already exists: `extract_at_path()` in
`prism-spec-engine/src/pipeline.rs`. It supports `$.field`, `$.a.b`,
`$.arr[*].field`, with depth and size caps (HIGH-007). This design reuses that
convention rather than inventing a new one.

---

## Design Decision 1 — The `source_path` Field

### Chosen field name: `source_path`

The field is added to `ColumnSpec` in `prism-spec-engine/src/spec_parser.rs`:

```rust
/// Optional JSONPath expression for extracting this column's value from the
/// raw JSON record returned by the pipeline executor.
///
/// ## Semantics
///
/// When `None` (default), the column value is extracted by looking up `col.name`
/// as a flat top-level key on the record — identical to the pre-ENRICH-1 behavior.
/// This default preserves full backward compatibility for all existing flat columns.
///
/// When `Some(path)`, the column value is extracted using `extract_at_path(record, path)`.
/// Paths MUST use the `$.` prefix convention of the existing `extract_at_path` function:
///   - `$.field`          — top-level key (redundant but valid)
///   - `$.a.b`            — nested object traversal
///   - `$.arr[*].field`   — wildcard: yields all `field` values from array `arr`
///
/// The `name` field is always the SQL column identifier — a clean identifier with
/// no `.`, `[`, or `]` characters. `source_path` is the extraction instruction only.
///
/// `#[serde(default)]` ensures backward compatibility: existing TOML files without
/// this field parse as `None`.
#[serde(default)]
pub source_path: Option<String>,
```

### Exact Rust struct delta

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSpec {
    pub name: String,
    pub column_type: ColumnType,
    pub ocsf_field: Option<String>,
    #[serde(default)]
    pub options: Vec<ColumnOptions>,
    #[serde(default)]
    pub timestamp_formats: Vec<String>,
    #[serde(default)]
    pub timestamp_fallback_chain: Vec<String>,
    // NEW — ENRICH-1
    #[serde(default)]
    pub source_path: Option<String>,
}
```

### `#[non_exhaustive]` implications

`ColumnSpec` is already `#[non_exhaustive]`. Adding a new `#[serde(default)]` field
is a backward-compatible change: existing TOML files parse without it; existing
external code that uses `ColumnSpec::new()` or `..Default::default()` construction
continues to compile. The non-exhaustive violation test (`v05_column_spec()` in
`tests/external/non-exhaustive-violation/src/struct_violations.rs`) will fail to
compile with one MORE E0639 error after this field is added, not fewer. The
`EXPECTED=83` gate in `ci.yml` is a MINIMUM floor (`-lt` check), so adding one
more field to `ColumnSpec` still passes the gate — the count goes from 83 to 84.
Update the EXPECTED value to `84` and the type list commentary in ci.yml as part
of this story.
**CORRECTION (2026-06-23, ENRICH-1 post-implementation, D-1296):** EXPECTED stays at **83**. Adding a new `#[serde(default)]` optional field to an already-`#[non_exhaustive]` struct adds no new E0639 compile-fail site — the struct was already non-exhaustive, so no external exhaustive match arm fails. Empirically confirmed: `just check` passes with gate=83 after ENRICH-1 implementation. The "83→84" claim above was a design-time prediction that proved incorrect at implementation time. ci.yml EXPECTED remains 83; CLAUDE.md non-exhaustive count 82 (pre-ENRICH-1 baseline on develop@5504c152) is UNCHANGED by ENRICH-1.

### TOML syntax (how authors declare it)

```toml
[[tables.columns]]
name = "ioc_value"
column_type = "string"
source_path = "$.iocs[*].value"

[[tables.columns]]
name = "ioc_type"
column_type = "string"
source_path = "$.iocs[*].type"

[[tables.columns]]
name = "alert_ip"
column_type = "string"
source_path = "$.alert_data.ip"
```

### Validation gate (new, at parse time)

Add to `SpecLoader::parse()` after the existing `timestamp_formats` validation block:

- If `source_path` is `Some(p)` and `p` does not begin with `$.`, emit
  `E-SPEC-001` with message `"source_path '{p}' must start with '$.'"`
- If `source_path` is `Some(p)` and `p` == `"$."` with no segment after it, emit
  `E-SPEC-001` (same guard already in `extract_at_path`)
- Do NOT validate wildcard syntax at parse time — defer to runtime extraction errors,
  which are already surfaced via `SpecEngineError::JsonPathExtractionFailed`

---

## Design Decision 2 — Array Cardinality

### Decision: JSON-list string in the existing `string` column

When `source_path` contains a wildcard (`[*]`) and the extraction yields multiple
values (a `serde_json::Value::Array`), the result is serialized to a compact JSON
string: `["hash1","hash2"]`. The column type remains `string` (Utf8 Arrow array).

**Rationale:**

Row explosion (option c) is rejected. CrowdStrike detections carry 1–4 behaviors
per detection; Cyberint alerts carry 0–N IOCs. Exploding rows would change the
cardinality of the result set relative to all other columns, breaking the
"detection_id, severity, device_id, ioc_value" shape that analysts expect for
`SELECT *`. DataFusion supports array explosion via `unnest()`, but requiring it
for basic IOC column access is too heavyweight for the demo critical path.

First-value (option b) is rejected. It silently discards real IOC data — a detection
with two behaviors would lose the second IOC. This is an information-lossy default
that would be hard to reverse post-deployment.

JSON-list string (option a) preserves all values, does not change row count, works
with `SELECT *`, and makes the data directly readable in the MCP inspector output.
It also has a well-defined behavior for the zero-element case (empty string `""`
or `"[]"` — use `"[]"` for clarity).

### `pivot_enrich` UDF input contract

The existing `pivot_enrich` UDF signature is `(Utf8) -> Utf8`. The UDF is called
with a single-element `Utf8` column. When that column contains a JSON-list string,
the UDF receives `["hash1","hash2"]` as a string argument.

**The UDF MUST be updated by ENRICH-1 to handle both forms:**
- Scalar string (current behavior): `"10.0.0.1"` — enrich directly
- JSON-list string: `'["hash1","hash2"]'` — parse as JSON array, enrich each
  element, return a JSON-list of enriched results

The contract amendment: if the input value parses as a JSON array
(`serde_json::from_str::<Vec<String>>(&val).is_ok()`), treat it as multi-value
mode. Otherwise treat as scalar. This is backward-compatible with the existing
test at `bc_2_19_001_plugin_udf_registration_test.rs:368` which passes a scalar
`"10.0.0.1"` — that path is unchanged.

**For the demo critical path, the canonical query becomes:**

```sql
SELECT detection_id, severity, behaviors_ioc_value
FROM crowdstrike_detections
WHERE pivot_enrich(behaviors_ioc_value) IS NOT NULL
```

or with explicit list handling:

```sql
SELECT alert_id, severity, ioc_value
FROM cyberint_alerts
```

The analyst workflow that needs to drive `pivot_enrich` passes the column directly.
If the column contains `["hash1","hash2"]`, the UDF enriches all elements and
returns a list-form result. If it contains a scalar value, it enriches that directly.

### Zero-element wildcard result

When `$.iocs[*].value` matches but `iocs` is an empty array, `extract_at_path`
returns `Value::Array([])`. Serialize to `"[]"`. The column value is `"[]"` (not
NULL). This is consistent and distinguishable from a record that truly had no `iocs`
field (NULL). Implementer note: the serialization should use `"[]"` not `""`.

---

## Design Decision 3 — Migration Spec

### Cyberint alerts table (MANDATORY — demo critical path)

All six IOC columns in the current spec must be migrated. The `name` field becomes
a clean SQL identifier; `source_path` holds the extraction path.

| Current broken `name` | New clean `name` | `source_path` | Notes |
|------------------------|-----------------|---------------|-------|
| `ioc.type` | `ioc_type` | `$.ioc.type` | Singleton IOC type. Wire key is `"type"` per `Ioc` serde rename. PENDING-LIVE-VALIDATION flag preserved in comment. |
| `ioc.value` | `ioc_value_singleton` | `$.ioc.value` | Singleton IOC value. Name `ioc_value_singleton` avoids collision with the array column below. |
| `iocs[].type` | `iocs_type` | `$.iocs[*].type` | Array IOC type. Wire key is `"type"`. `[*]` wildcard → JSON-list string. |
| `iocs[].value` | `iocs_value` | `$.iocs[*].value` | Array IOC value. THIS IS THE PRIMARY DEMO COLUMN used in `pivot_enrich`. Wire key is `"value"`. |
| `alert_data.ip` | `alert_data_ip` | `$.alert_data.ip` | Nested scalar — no wildcard. |
| `alert_data.domain` | `alert_data_domain` | `$.alert_data.domain` | Nested scalar. |
| `alert_data.url` | `alert_data_url` | `$.alert_data.url` | Nested scalar. |

Note on wire keys: the DTU `Ioc` struct serializes with `#[serde(rename = "type")]`
on `ioc_type` and no rename on `value`. Therefore the wire JSON from the DTU has
keys `"type"` and `"value"`. The `source_path` for the iocs columns uses `$.iocs[*].type`
and `$.iocs[*].value` to match the actual wire output. The serde alias `ioc_type` / `ioc_value`
is a deserialization convenience in the Rust struct — it does NOT affect the serialized
JSON keys that the TOML extractor sees.

### Cyberint incidents table

No IOC columns exist in the incidents table. No migration needed.

### CrowdStrike detections table (MANDATORY — demo critical path)

| Current broken `name` | New clean `name` | `source_path` | Notes |
|------------------------|-----------------|---------------|-------|
| `behaviors[].ioc_type` | `behaviors_ioc_type` | `$.behaviors[*].ioc_type` | Wire key is `ioc_type` (generator.rs line ~752 uses `"ioc_type"` key in untyped JSON). |
| `behaviors[].ioc_value` | `behaviors_ioc_value` | `$.behaviors[*].ioc_value` | Wire key is `ioc_value`. THIS IS THE PRIMARY DEMO COLUMN. |
| `behaviors[].ioc_source` | `behaviors_ioc_source` | `$.behaviors[*].ioc_source` | Wire key is `ioc_source`. |
| `behaviors[].ioc_description` | `behaviors_ioc_description` | `$.behaviors[*].ioc_description` | Wire key is `ioc_description`. |

CrowdStrike behaviors wire shape note: the generator (`make_detection_with_ioc` in
`crates/prism-dtu-crowdstrike/src/generator.rs`) emits untyped `serde_json::Value`
objects. The generator uses keys `"ioc_type"`, `"ioc_value"`, `"ioc_source"`,
`"ioc_description"` directly (NOT the `Ioc` struct serde convention). So the
CrowdStrike extraction paths use those exact key names — confirmed by reading
`generator.rs` lines 751–755.

### Other sensors (Armis, Claroty)

All Armis and Claroty column names are clean SQL identifiers with no `.` or `[]`
characters. No migration needed. Scope: OUT.

### Customer overlay TOMLs

The customer overlay TOMLs at
`crates/prism-sensors/specs/customers/acme/armis.sensor.toml` and
`crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` use Armis only.
No migration needed.

---

## Design Decision 4 — Contract Amendments

### BC-2.06.019 IOC-surface-matrix wording change

The PO must update BC-2.06.019 §Per-Sensor IOC-Surface Matrix to replace all
references to bracket-in-name column identifiers (`iocs[].value`, `behaviors[].ioc_type`,
etc.) with the clean identifier names from the migration table above.

Specific changes the PO should make:
- Cyberint row: `iocs[].value` → `iocs_value` (with note: `source_path = "$.iocs[*].value"`)
- CrowdStrike row: `behaviors[].ioc_value` → `behaviors_ioc_value` (with note: `source_path = "$.behaviors[*].ioc_value"`)
- The canonical ThreatIntel pivot query in the BC (if present) should reference
  `iocs_value` not `iocs[].value`

### PIVOT-003 convention superseded

This design supersedes PIVOT-003's implicit convention of embedding traversal syntax
in the column `name` field. Record this as a superseding decision in STATE.md with
this design document as the reference artifact. PIVOT-003 merged content (the IOC
column additions to the sensor TOMLs) is not reverted — it is migrated by ENRICH-1.

---

## Design Decision 5 — Files-to-Change List for Implementer (ENRICH-1)

### 1. `crates/prism-spec-engine/src/spec_parser.rs`

**Change:** Add `source_path: Option<String>` field to `ColumnSpec` with
`#[serde(default)]`. Update `ColumnSpec::Default::default()` to set
`source_path: None`. Update `ColumnSpec::new()` to accept the field (or add
a separate `new_with_source_path` constructor). Add parse-time validation gate
in `SpecLoader::parse()` checking `$.` prefix when `source_path` is `Some`.

### 2. `crates/prism-spec-engine/src/column_mapping.rs`

**Change:** `ColumnMapper::map_record()` currently does `raw.get(&col.name)`.
When `col.source_path.is_some()`, call `extract_at_path(raw, path)` instead.
The return from `extract_at_path` is `Result<serde_json::Value, String>` — on
`Err`, treat as absent (same as column not present) and emit a `tracing::warn!`
with `event_type = "column_source_path_extraction_failed"`.

For wildcard paths that return a `Value::Array`, serialize to a compact JSON string
before inserting into `raw_extensions` or mapping to OCSF.

### 3. `crates/prism-bin/src/spec_driven_adapter.rs`

**Change:** `build_column_array()` currently calls `r.get(col_name)` with a flat key.
The function signature needs access to `col.source_path` to call `extract_at_path`
when needed. Options:

Option A (preferred): Change `build_column_array` to accept `col: &ColumnSpec` instead
of just `col_name: &str` and `col_type`. Inside, dispatch on `col.source_path`:
- `None` → existing `r.get(&col.name)` flat lookup
- `Some(path)` → `extract_at_path(r, path)`, then coerce to the target type

For wildcard paths returning `Value::Array` with `col_type == ColumnType::String`:
serialize the array to a compact JSON string for the Arrow `StringArray`.

For wildcard paths returning `Value::Array` with integer/float/bool types: use
first-element extraction (wildcard on non-string types is unusual; warn and
first-value is the safest fallback for numeric types).

The callers at lines 849 and 852 currently pass `&col_spec.name` and
`&col_spec.column_type` — update them to pass `col_spec` directly (or add a
`source_path` parameter).

### 4. `crates/prism-sensors/specs/cyberint.sensor.toml`

**Change:** Apply the migration table from Decision 3. Replace the six broken IOC
column declarations with clean names + `source_path` attributes. Preserve all
existing comments explaining PENDING-LIVE-VALIDATION status and wire-key
clarifications (update to reference new names).

### 5. `crates/prism-sensors/specs/crowdstrike.sensor.toml`

**Change:** Apply the migration table from Decision 3. Replace the four `behaviors[].*`
column declarations with clean names + `source_path` attributes.

### 6. `crates/prism-spec-engine/src/infusion/udf.rs`

**Change:** Update `pivot_enrich` UDF's `invoke_async_with_args` to handle the
JSON-list string input contract defined in Decision 2. If the input value starts
with `[`, attempt `serde_json::from_str::<Vec<String>>`. On success, call
`enrich_single` for each element and serialize the results back to a JSON-list
string. On failure (malformed JSON), fall through to scalar path.

### 7. `tests/external/non-exhaustive-violation/src/struct_violations.rs`

**Change:** Update `v05_column_spec()` to still omit `source_path` (E0639 still
fires because `ColumnSpec` is `#[non_exhaustive]`). No functional change needed —
the violation function already doesn't include all fields; the E0639 is about
the struct being non-exhaustive, not about which fields are included.

### 8. `.github/workflows/ci.yml`

**Change:** Update `EXPECTED=83` to `EXPECTED=84` and update the commentary to
include `source_path` in `spec_parser::ColumnSpec` (field count is now 7 with
the new field). Also update the inline human-readable type list.

**CORRECTION (2026-06-23, D-1296):** NO CI GATE CHANGE NEEDED. EXPECTED stays at **83**. See correction note in §`#[non_exhaustive]` implications above. This design-doc item (DD-5 item 8) was written under the incorrect assumption that adding a field to a `#[non_exhaustive]` struct would produce a new E0639 compile-fail site. It does not — the struct was already non-exhaustive, so existing external match arms already require a wildcard. ci.yml is NOT modified as part of ENRICH-1.

### 9. Tests to add

| Test location | What to test |
|--------------|--------------|
| `crates/prism-spec-engine/tests/` | `source_path` round-trips through TOML parse; validation rejects non-`$.` paths |
| `crates/prism-spec-engine/tests/` | `ColumnMapper::map_record` with `source_path = "$.a.b"` extracts nested field |
| `crates/prism-spec-engine/tests/` | `ColumnMapper::map_record` with `source_path = "$.iocs[*].value"` returns JSON-list string |
| `crates/prism-spec-engine/tests/` | `source_path = None` behavior unchanged (backward compat) |
| `crates/prism-bin/tests/` | `build_column_array` with `source_path` on a wildcard path returns correct Arrow `StringArray` |
| `crates/prism-spec-engine/tests/` | UDF handles JSON-list string input `'["hash1","hash2"]'` (scalar backward compat) |

### 10. No changes needed

- `prism-spec-engine/src/pipeline.rs` — `extract_at_path` is already correct and
  will be called by the column mapper and adapter without modification
- `prism-spec-engine/src/validation.rs` — no method-whitelist impact
- `prism-dtu-cyberint/src/types.rs` — DTU wire format is unchanged
- `prism-dtu-crowdstrike/src/generator.rs` — DTU wire format is unchanged

---

## Summary

| Aspect | Decision |
|--------|----------|
| Field name | `source_path: Option<String>` on `ColumnSpec` |
| Default | `None` (backward compatible; flat key lookup unchanged) |
| Path convention | `$.` prefix, reusing `extract_at_path` grammar |
| Wildcard cardinality | JSON-list string in `string` column |
| `pivot_enrich` contract | Accepts scalar OR JSON-list string; enriches each element |
| CI gate | EXPECTED stays **83** (CORRECTED D-1296 — adding field to already-`#[non_exhaustive]` struct adds no new E0639 site) |
| Cyberint migration | 7 columns renamed + `source_path` added |
| CrowdStrike migration | 4 columns renamed + `source_path` added |
| Other sensors | No changes needed |
