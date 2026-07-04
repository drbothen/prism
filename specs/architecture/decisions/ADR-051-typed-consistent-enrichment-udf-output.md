---
document_type: adr
adr_id: "ADR-051"
title: "Typed & Consistent Enrichment UDF Output — output_type→Arrow DataType Mapping, Mandatory source_column, Scalar-Input Rule, and INV-ENRICH-TYPED-001"
status: PROPOSED
date: "2026-07-03"
modified: "2026-07-03"
version: "1.1"
producer: architect
subsystems_affected: [SS-09, SS-10, SS-19]
supersedes: []
superseded_by: null
amends: null
anchor_stories: []
related_adrs: [ADR-024, ADR-040, ADR-044]
related_bcs: [BC-2.19.001]
locked_decisions: []
wiring_deferred_to: null
closes_defect: "DRIFT-PIVOT-UDF-OUTPUT-TYPE-001"
---

# ADR-051: Typed & Consistent Enrichment UDF Output

## Status

PROPOSED v1.0 (2026-07-03). Awaiting human ratification before story decomposition.
Closes escaped defect DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 (former AI-default deferral in
`InfusionAsyncUdf::return_type`, Canonical Principle Rule 3 violation).

---

## Context

### The Defect (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001)

`InfusionAsyncUdf::return_type` in `crates/prism-query/src/infusion_udf.rs` hardcodes
`Ok(DataType::Utf8)` and ignores `descriptor.output_type`, which already exists on
`InfusionUdfDescriptor` (carried from the `[[infusion.fields]]` `output_type` string
in every `.infusion.toml` spec). The comment in the code explicitly names this as a
deferred item ("Full typed mapping of `descriptor.output_type` → Arrow DataType is
deferred to S-1.14-REDO"). That deferral violated Canonical Principle Rule 3 — it was
an AI default deferral, not a human-directed one.

### Observed Failures (T13 Comprehensive Audit OBS-1, 2026-07-03)

Two distinct failure modes were documented in `demo-comprehensive-preflight-audit-2026-07-03.md`
§4 OBS-1:

**Failure A — ThreatIntel doubly-encoded JSON (cyberint and crowdstrike paths):**

```
| enrich threat_score(iocs_value) | limit 1
→ threat_score = ["{\"threat_score\":95,\"threat_is_known_malicious\":true,...}"]
```

Two layers of encoding:
- Layer 1: `iocs_value` is a JSON-list string (e.g., `["hash1","hash2"]`). The ENRICH-1
  list-dispatch path enriches each element individually and JSON-encodes the results into
  a list. Each element is the full plugin response JSON object serialized as a string,
  because `threatintel.infusion.toml` declares NO `source_column` on any field.
- Layer 2: the whole response object gets wrapped in a list: `["{...}"]`.

Root cause: `project_value()` passthrough when `source_column` is absent serializes the
entire `serde_json::Value::Object` response. With no `source_column`, every ThreatIntel
field returns the whole object.

**Failure B — NVD cvss_base_score is a String, not a Float (silent wrong comparison):**

```
| enrich cvss_base_score(device_cves_first) | filter cvss_base_score >= 8.0
→ cvss_base_score = "8.1"   (DataType::Utf8, not Float64)
```

NVD correctly uses `source_column = "baseScore"` and returns the extracted value.
But because `return_type()` always returns `Utf8`, DataFusion treats the column as a
string. The `>= 8.0` comparison is lexicographic: `"8.1" >= "8.0"` evaluates `true`
by accident here, but `"10.0" >= "8.0"` evaluates `false` (lexicographic "1" < "8").
Any CVSS score >= 10.0 would silently fail a `>= 8.0` filter.

### Prior Decision Record

ADR-040 v2.0 (Dual-Path Infusion Architecture) established the architectural split
between HttpLookup (declarative, NVD pattern) and WASM Plugin (ThreatIntel pattern).
It did not address typed output — that gap is closed by this ADR.

ADR-024 established `prism_core::column::ColumnType` (String/Integer/Float/Boolean/
Datetime/Json) as the canonical sensor-schema type vocabulary. This ADR aligns the
infusion output-type vocabulary to the same semantic space.

---

## Decisions

### D1 — Canonical output_type → Arrow DataType Mapping

The following table is the authoritative mapping. `InfusionAsyncUdf::return_type()`
MUST return the Arrow `DataType` corresponding to `descriptor.output_type`. The
mapping also governs which typed Arrow array `invoke_async_with_args()` must build.

| `output_type` in TOML | Arrow `DataType` | Notes |
|---|---|---|
| `"string"` | `DataType::Utf8` | Passthrough; no coercion needed |
| `"integer"` | `DataType::Int64` | JSON Number parsed via `as_i64()` or string parsed via `i64::from_str(s.trim())` |
| `"float"` | `DataType::Float64` | JSON Number parsed via `as_f64()` or string parsed via `f64::from_str(s.trim())` |
| `"boolean"` | `DataType::Boolean` | JSON Bool passthrough; string coercion: `"true"`/`"1"`/`"yes"` → true; `"false"`/`"0"`/`"no"` → false; case-insensitive |
| `"json"` | `DataType::Utf8` | JSON stored as Utf8 string; DataFusion JSON path functions operate on Utf8 |
| `"datetime"` | `DataType::Utf8` | ISO 8601 string. **This is the correct consistency choice — not a deferral.** See "Datetime = Utf8 rationale" below. |
| unknown / missing | `DataType::Utf8` | Fallback only; spec-load validation (E-INFUSE-013 sub-condition 7, D3 below) MUST reject unknown type names before any UDF is registered |

**Alignment with ADR-024:** The semantic taxonomy (`string`, `integer`, `float`,
`boolean`, `json`, `datetime`) maps 1-to-1 with `prism_core::column::ColumnType`
variants (String, Integer, Float, Boolean, Json, Datetime). Infusion output types
use lowercase-kebab vocabulary; ColumnType uses PascalCase variants; both represent
the same six-type domain. No new type vocabulary is introduced.

**Datetime = Utf8 rationale (authoritative cross-reference, related_adrs: ADR-044):**

The query language's Datetime representation is a settled architectural fact. The
authoritative mapping in `crates/prism-bin/src/spec_driven_adapter.rs` line 886 is:

```rust
ColumnType::Datetime => DataType::Utf8,
```

Every sensor Datetime column registered in DataFusion today is `DataType::Utf8` (ISO-8601
string). This is confirmed by `crates/prism-query/src/pipe_sql_emitter.rs` lines 817-818:
"Datetime fields is DataType::Utf8 (ISO-8601 string) — see spec_driven_adapter
`column_type_to_arrow`: `ColumnType::Datetime => DataType::Utf8`", and by the
load-bearing test commentary in
`crates/prism-query/src/tests/high002_plan_pinning_tests.rs` lines 169/191/313, which
explicitly states: "for OCSF Datetime: `column_type_to_arrow` returns `DataType::Utf8`"
and "DataFusion fails to compare `Utf8` against `Timestamp(Microsecond, None)`".

ADR-044 D4 / BC-2.11.021: `NOW() - INTERVAL 'Nh'` is lowered to a `Literal::Timestamp`
that DataFusion receives as an ISO-8601 string `'2026-06-24T00:00:00Z'` — a `Utf8`→`Utf8`
comparison against sensor datetime columns. ISO-8601 is lexicographically sortable, so
this is correct in practice.

If enrichment `datetime` output used `DataType::Timestamp(Microsecond, None)`, any
predicate crossing a sensor datetime column and an enrichment datetime column — e.g.,
`| filter sensor_timestamp > enriched_created_at` — would produce a DataFusion type
error (`Utf8 > Timestamp(Microsecond, None)` is unsupported without explicit CAST).
This would create a two-representation split that breaks PrismQL's consistency guarantee.
Changing enrichment datetime to Timestamp while sensor datetimes remain Utf8 would WORSEN
consistency, not improve it.

**Active usage:** No current infusion spec declares `output_type = "datetime"`. This
mapping is future-proofing. When such a spec is authored it receives Utf8 output consistent
with the rest of the query language.

**Note — stale doc-comment in `column.rs`:** `crates/prism-core/src/column.rs` line 28
currently reads `/// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.`
This does NOT match the implementation (`DataType::Utf8`). The comment must be corrected
in the implementation story to: `/// ISO-8601 UTC datetime string. Arrow: Utf8.` This is
a code-comment fix, not a behavioral change (the code is correct; the comment is wrong).

**What "typing datetime properly" requires:** Migrating PrismQL Datetime to Arrow
`Timestamp` requires changing `spec_driven_adapter.rs`, the pushdown logic, the
`inject_now_predicate` path (BC-2.11.021), ALL sensor schemas, the pipe-SQL emitter, and
the corresponding query tests. That is a cross-cutting language-level story that is
correctly out of scope for this defect fix. This ADR does not defer it — it documents that
the datetime-as-Utf8 design is intentional in the current system and that enrichment must
match it for consistency.

**Implementation:** `InfusionAsyncUdf` must add a private `output_arrow_type()`
helper that pattern-matches `self.descriptor.output_type.as_str()` and returns the
`DataType`. `return_type()` delegates to this helper. `invoke_async_with_args()`
dispatches on `output_arrow_type()` to build the correct output array type (e.g.,
`Int64Array`, `Float64Array`, `BooleanArray`, or `StringArray`).

### D2 — Coercion Semantics and Failure Mode

**Coercion path (string → typed value):**

After `project_value()` extracts a `String` from the JSON response (either via
`source_column` projection or direct passthrough for plain-string sources), a typed
coercion step produces the declared output type:

| Target type | Coercion from String |
|---|---|
| `Int64` | `i64::from_str(s.trim())` → `Ok(i64)` or `Err` → NULL + E-INFUSE-014 |
| `Float64` | `f64::from_str(s.trim())` → `Ok(f64)` or `Err` → NULL + E-INFUSE-014 |
| `Boolean` | case-insensitive match against `{"true","1","yes"}` / `{"false","0","no"}` → `true`/`false`; anything else → NULL + E-INFUSE-014 |
| `Utf8` (`string`, `json`) | passthrough; no coercion; no error |

**Coercion path (JSON Number → typed value):**

When `project_value()` returns a non-string JSON value (e.g., `serde_json::Value::Number(95)`
projected from `source_column = "threat_score"`), use Arrow-native conversion:
`Number.as_i64()` for Int64, `Number.as_f64()` for Float64. If `as_i64()` returns
`None` (e.g., the number is a float but the spec declares `integer`), produce NULL
+ E-INFUSE-014.

**Failure mode: NULL + E-INFUSE-014 (no panic, no empty string, no passthrough):**

When any coercion fails, the output row is NULL. This is consistent with DataFusion's
behavior for `CAST()` failures under `TRY_CAST`. Silently swallowing the failure with
a passthrough string is forbidden (Canonical Principle Standing Rule 3 §2 — "no silent
Vec::new() return where partial-failure data should propagate"). Panicking is also
forbidden. NULL is the correct partial-failure signal for a single enrichment row.

**New error code: E-INFUSE-014 (TypeCoercionFailed):**

```
"E-INFUSE-014: enrichment field '{field_name}' (infusion '{infusion_id}'): \
 declared output_type is '{declared_type}', but projected value \
 '{truncated_value}' (first 50 chars) cannot be coerced; row produces NULL"
```

- Severity: runtime warning (logged at `tracing::warn!`; not surfaced as a query error)
- Recurrence: one log line per failing row per UDF call; NOT aggregated per-batch
  (individual rows may fail coercion for valid reasons, e.g., enrichment returned a
  human-readable error string in place of a numeric score)
- MCP surface: NOT propagated as E-QUERY-034; the query succeeds with NULLs in the
  typed column. Only systematic coercion failure (e.g., every row returns NULL)
  warrants operator investigation.
- `{truncated_value}`: first 50 characters of the projected string value. Credential
  values MUST NOT appear here (AD-017); enrichment response values are considered
  untrusted external data, not credentials.
- A new BC-2.16.002 catalog row MUST be registered for the `event_type = "infusion.coercion_failed"` tracing emission (SAP-1).

### D3 — Mandatory source_column for Plugin-Type Fields + Spec-Load Enforcement

**Rule:** Every `[[infusion.fields]]` entry with its parent infusion declared as
`type = "plugin"` MUST declare `source_column`. The plugin enrichment source
(`PluginInfusionSource` / `prism-dtu-threatintel`) returns a JSON object containing
multiple fields. Without `source_column`, `project_value()` falls into the
passthrough branch and serializes the entire object — the root cause of Failure A.

**Enforcement:** `InfusionLoader::validate` (spec-load time) must check: for any
`[[infusion.fields]]` entry where the parent infusion `type = "plugin"`, if
`source_column` is absent, reject the entire spec with E-INFUSE-013 sub-condition 8:

> `"plugin-type field '{name}' in infusion '{infusion_id}' must declare 'source_column' \
>  to project a specific field from the plugin response object; without source_column \
>  the full response object is serialized (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 root cause)"`

This is sub-condition 8 of E-INFUSE-013. The existing E-INFUSE-013 description covers
"an `[[infusion.fields]]` entry contains ... or another field-level validation constraint
is violated". The product-owner must add sub-condition 8 to BC-2.19.001's E-INFUSE-013
row and to the error-taxonomy.

For `type = "http_lookup"` fields, `source_column` is already practically required
(the JSONPath subtree `response_path` extracts an object; `source_column` picks the
field from that subtree). The existing NVD spec is already compliant. No additional
enforcement needed for http_lookup beyond the existing field-level validation.

For `type = "local_lookup"` fields (csv, json_lookup, maxmind_mmdb), `source_column`
is OPTIONAL. Local lookup sources return a pre-projected scalar per field (the column
projection happens in the source implementation). If `source_column` is declared on
a local-lookup field, it is applied via `project_value()` as before.

**Extended E-INFUSE-013 sub-condition 7 (unknown output_type):**

Add sub-condition 7 to E-INFUSE-013: `output_type` string value is not one of the
recognized values (`string`, `integer`, `float`, `boolean`, `json`, `datetime`). This
fires at spec-load time and rejects the spec.

### D4 — Scalar-Input Consistency Rule

**Rule:** Typed (`integer`, `float`, `boolean`) enrichment output requires a scalar
input column. A JSON-list string (detected by leading `[`) as input to a typed-output
UDF produces NULL + E-INFUSE-014 at runtime, because a list cannot coerce to a single
scalar of these types.

**Canonical input pattern (consistent with NVD precedent):**
Sensors whose raw columns contain JSON-list values (e.g., cyberint `iocs_value`,
crowdstrike `behaviors_ioc_value`) MUST expose scalar `_first` companion columns,
extracted at the fixture generator / DTU response layer. Enrichment specs targeting
typed output MUST reference the `_first` scalar column.

| Sensor | Current JSON-list column | Required scalar companion | Used by |
|---|---|---|---|
| cyberint_alerts | `iocs_value` (e.g., `["hash1","hash2"]`) | `iocs_value_first` (e.g., `"hash1"`) | threatintel enrichment |
| crowdstrike_detections | `behaviors_ioc_value` (e.g., `["hash1"]`) | `behaviors_ioc_value_first` (e.g., `"hash1"`) | threatintel enrichment |

**ENRICH-1 list-dispatch path retention:** The ENRICH-1 list-dispatch path in
`invoke_async_with_args` is RETAINED for `output_type = "json"` fields only. This
is correct behavior: `threat_sources` declares `output_type = "json"` and legitimately
returns a JSON array of source strings. For any field with `output_type != "json"`,
a JSON-list input MUST produce NULL + E-INFUSE-014 (not attempt list-enrichment and
return a JSON-encoded list of typed values).

**Rationale for scalar-first over list-semantics for typed fields:** Defining "list
enrichment → typed output" semantics creates ambiguity (which element? aggregation?
first-wins?). Scalar-first is unambiguous, testable, and consistent with NVD's
`device_cves_first` precedent (established by S-DEMO-ENRICHMENT-PIVOT-003).

### D5 — PrismQL Comparison Semantics on Typed Enrichment Columns

After this fix, enrichment columns have correct DataFusion-native types. DataFusion
auto-casts literal constants to match the column type in comparison predicates:

| Query pattern | Before fix | After fix |
|---|---|---|
| `\| filter threat_score >= 75` | `Utf8 >= Utf8("75")` — lexicographic, wrong | `Int64 >= Int64(75)` — numeric, correct |
| `\| filter cvss_base_score >= 8.0` | `Utf8 >= Utf8("8.0")` — lexicographic, wrong | `Float64 >= Float64(8.0)` — numeric, correct |
| `\| filter threat_is_known_malicious = true` | `Utf8 = Utf8("true")` — string comparison | `Boolean = Boolean(true)` — native boolean |

No CAST wrappers are required in the query. DataFusion's type coercion rules handle
literal constant promotion automatically when the column type is non-Utf8.

**Important:** DataFusion's implicit cast of integer literal `75` to `Int64` is correct.
Queries using `threat_score > 75` (integer literal, no decimal) are valid because
DataFusion promotes integer literals to `Int64` when the column is `Int64`.

### D6 — Cross-UDF Consistency Invariant: INV-ENRICH-TYPED-001

**Statement:** All enrichment UDFs registered from `[[infusion.fields]]` entries
satisfy:

1. The UDF's `return_type()` returns the exact Arrow `DataType` declared by `output_type`
   per the D1 mapping table.
2. The UDF's `invoke_async_with_args()` builds an output `ColumnarValue::Array` whose
   Arrow array type matches `return_type()`.
3. For `type = "plugin"` sources: a `source_column` is declared and applied, preventing
   whole-response-object serialization.
4. For typed output (`integer`, `float`, `boolean`): the input column is a scalar
   (not a JSON-list); JSON-list inputs produce NULL + E-INFUSE-014.
5. On coercion failure: the output row is NULL, not a panic, not a passthrough string,
   not an empty string.

This invariant is violated by any infusion spec or UDF implementation that silently
produces wrong-typed output. Violations are a P1 finding in adversarial review.

---

## New Error Code: E-INFUSE-014

| Code | Category | Format | Notes |
|---|---|---|---|
| E-INFUSE-014 | runtime / enrichment | `"E-INFUSE-014: enrichment field '{field_name}' (infusion '{infusion_id}'): declared output_type is '{declared_type}', but projected value '{truncated_value}' (first 50 chars) cannot be coerced; row produces NULL"` | Runtime per-row warning. Not propagated as query error. NULL output in the typed column. New `InfusionError::TypeCoercionFailed { field_name, infusion_id, declared_type, truncated_value }` variant. Tracing emission: `tracing::warn!(event_type = "infusion.coercion_failed", field_name, infusion_id, declared_type, ...)`. BC-2.16.002 catalog row required (SAP-1). |

---

## Required Spec Changes

### threatintel.infusion.toml — Full Rewrite of All Fields

Current state: no `source_column` on any field; all three fields use
`input_field = "iocs_value"` (JSON-list column).

Required changes:
1. Add `source_column` to each `[[infusion.fields]]` entry, projecting the specific
   field from the `ThreatLookupResponse` JSON object:
   - `threat_is_known_malicious`: `source_column = "threat_is_known_malicious"`
   - `threat_score`: `source_column = "threat_score"`
   - `threat_sources`: `source_column = "threat_sources"` (output_type = "json",
     retains list; no scalar-input requirement for json-typed fields)
2. Change `input_field` for `threat_is_known_malicious` and `threat_score` to
   `"iocs_value_first"` (scalar companion column, D4).
   `threat_sources` may retain `input_field = "iocs_value"` IF the ENRICH-1 list-
   dispatch path is used for the json-typed output; OR switch to `"iocs_value_first"`
   for consistency. Recommendation: use `"iocs_value_first"` for all three fields
   and produce a single-element JSON array from a scalar input for `threat_sources`.
   This eliminates the ENRICH-1 list-dispatch path from the production code paths
   entirely, simplifying the implementation.

### nvd.infusion.toml — No Changes Required

NVD already has `source_column` on all fields and uses `device_cves_first` (scalar).
After the D1 typing fix, `cvss_base_score` will return `Float64` and `cvss_severity`/
`cvss_vector` will return `Utf8`. No spec change needed.

### Sensor TOMLs — New Scalar Companion Columns

```toml
# specs/sensors/cyberint.sensor.toml — add to [[tables]] for cyberint_alerts
[[tables.columns]]
name = "iocs_value_first"
column_type = "String"
description = "First IOC value from iocs_value array (scalar projection for typed enrichment)"

# specs/sensors/crowdstrike.sensor.toml — add to [[tables]] for crowdstrike_detections
[[tables.columns]]
name = "behaviors_ioc_value_first"
column_type = "String"
description = "First IOC value from behaviors_ioc_value array (scalar projection for typed enrichment)"
```

---

## Blast Radius (TD-VSDD-060 Sibling Sweep)

The following surfaces must be updated in the implementation story:

| Surface | Required Change |
|---|---|
| `crates/prism-query/src/infusion_udf.rs` | `return_type()`: implement D1 mapping; `invoke_async_with_args()`: dispatch on output type to build typed array; add `coerce_to_typed()` helper; ENRICH-1 list-dispatch: restrict to `output_type = "json"` only; add E-INFUSE-014 emission on coercion failure |
| `crates/prism-spec-engine/src/infusion/loader.rs` (or equivalent validation path) | Add E-INFUSE-013 sub-condition 7 (unknown output_type) and sub-condition 8 (plugin-type field missing source_column) to spec-load validation |
| `crates/prism-core/src/error.rs` (or `infusion.rs`) | Add `InfusionError::TypeCoercionFailed { field_name: String, infusion_id: String, declared_type: String, truncated_value: String }` variant with `#[error("E-INFUSE-014: ...")]`; add BC-2.16.002 catalog row for `event_type = "infusion.coercion_failed"` |
| `specs/infusions/threatintel.infusion.toml` | Add `source_column` to all three fields; change `input_field` to `iocs_value_first`; verify `output_type` values |
| `specs/sensors/cyberint.sensor.toml` | Add `iocs_value_first: String` column to `cyberint_alerts` table |
| `specs/sensors/crowdstrike.sensor.toml` | Add `behaviors_ioc_value_first: String` column to `crowdstrike_detections` table |
| Cyberint fixture generator (`crates/prism-dtu-demo-server/src/` or `crates/prism-dtu-cyberint/src/`) | Emit `iocs_value_first` field: first element of `iocs_value` array, or empty string if array is empty |
| CrowdStrike fixture generator (`crates/prism-dtu-demo-server/src/` or `crates/prism-dtu-crowdstrike/src/`) | Emit `behaviors_ioc_value_first` field: first element of `behaviors_ioc_value` array, or empty string if array is empty |
| `prism_describe` response and pql_hints (BC-2.10.012) | Include `iocs_value_first` and `behaviors_ioc_value_first` in table schema output; update Category-2 enrichment-discovery hints that reference these columns |
| `crates/prism-mcp/src/resources.rs` (PrismQL reference resource, ADR-045) | Update enrichment UDF examples to use `iocs_value_first` and `behaviors_ioc_value_first`; update example output values from JSON-encoded strings to bare typed values (95 not `["{...}"]`) |
| `scripts/t13-preflight-audit.py` | Update E6 check: `threat_score >= 75` now evaluates as a numeric comparison; update column name in E1/E5 queries to `iocs_value_first` / `behaviors_ioc_value_first` |
| `.factory/objectives/T13-capstone-demo-runbook.md` | Steps 3.2 and 6.2: update expected output to show `threat_score = 95` (bare integer), not JSON; update query examples to use `_first` columns |
| `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs` | Add test vectors for typed output (integer, float, boolean); add test for plugin-type-missing-source_column rejection (E-INFUSE-013 sub-condition 8) |
| `crates/prism-query/src/infusion_udf.rs` unit tests | Add typed-output tests: `threat_score` UDF returns `Int64Array`; `cvss_base_score` UDF returns `Float64Array`; coercion failure produces NULL + E-INFUSE-014 |
| `crates/prism-dtu-threatintel/tests/` | Update expected column values in enrichment tests to typed (integer/boolean) output |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Add E-INFUSE-014 row; add sub-conditions 7 and 8 to E-INFUSE-013 row |
| `BC-2.19.001` | See Recommended BC Amendments section below |

**TD-VSDD-060 sweep command:** After implementation, run:
```bash
rg 'output_type.*Utf8\|return_type.*Utf8' crates/prism-query/src/infusion_udf.rs
```
Must return zero results (no hardcoded Utf8 fallback in `return_type()`).

```bash
rg 'E-INFUSE-013' crates/ --type rust
```
Must hit the updated validation path covering sub-conditions 7 and 8.

---

## Recommended BC-2.19.001 Amendments

The product-owner must amend BC-2.19.001 in the same story delivery burst as the
implementation. The following changes are required:

**1. New postcondition: INV-ENRICH-TYPED-001 (typed output)**

Add after the existing `enrich_descriptor()` API postcondition:

> "**Typed UDF output (INV-ENRICH-TYPED-001):** Each `InfusionUdfDescriptor.output_type`
> value maps to a specific Arrow `DataType` per the ADR-051 D1 table.
> `InfusionAsyncUdf::return_type()` MUST return this mapped type (not always
> `DataType::Utf8`). `invoke_async_with_args()` MUST produce a typed output
> `ColumnarValue::Array` whose Arrow array type matches `return_type()`. On
> coercion failure, the output row is NULL and E-INFUSE-014 is emitted.
> Hardcoding `DataType::Utf8` as the return type for all infusion UDFs is a
> violation of this postcondition."

**2. New postcondition: mandatory source_column for plugin-type**

Add:

> "**Plugin-type field projection (D3 / E-INFUSE-013 sub-condition 8):** Every
> `[[infusion.fields]]` entry in a `type = \"plugin\"` infusion MUST declare
> `source_column`. A plugin-type field without `source_column` is rejected at
> spec-load time with E-INFUSE-013 (sub-condition 8). The root cause of
> DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 was the absence of this validation."

**3. Update INV-INFUSE-001**

Extend the existing invariant statement to include: "...and the registered UDF's
`return_type()` must return the Arrow DataType mapped from `output_type` per
ADR-051 D1."

**4. New invariant: INV-ENRICH-TYPED-001**

Add to the Invariants section:

> "INV-ENRICH-TYPED-001: All enrichment UDFs registered from `[[infusion.fields]]`
> entries produce typed output per the ADR-051 D1 mapping. No enrichment UDF may
> return `DataType::Utf8` for a field whose `output_type` is `integer`, `float`, or
> `boolean`. Typed input columns (`integer`, `float`, `boolean`) require scalar input;
> JSON-list input to a typed-output UDF produces NULL + E-INFUSE-014."

**5. New E-INFUSE-013 sub-conditions**

Add to the E-INFUSE-013 error row's sub-condition list:
- Sub-condition 7: `output_type` value is not in `{string, integer, float, boolean, json, datetime}`
- Sub-condition 8: `type = "plugin"` field lacks `source_column`

**6. New error row E-INFUSE-014**

Add to Error Conditions table the E-INFUSE-014 row per the error code defined in
this ADR's "New Error Code" section above.

**7. Version bump**

BC-2.19.001 version must advance from 2.1 → 2.2 with this burst.

**New BC needed?** No. These changes are additive postconditions on the existing
spec-loading and UDF-registration contract. No new behavioral contract boundary is
created. All changes fit within BC-2.19.001's scope (infusion spec loading, UDF
descriptor registration, and typed output as a property of the registered UDF).

---

## Considered Alternatives

### Alt-A: Keep Utf8 output; add CAST() in PrismQL queries

Rejected. Shifting the coercion burden to every consumer query violates INV-ENRICH-TYPED-001
and the Canonical Principle ("for now, add CAST() later" is a deferral pattern). Every
downstream story, test, and demo query would need a `CAST(cvss_base_score AS FLOAT)` wrapper.
PrismQL is explicitly designed to be consumed by LLM agents (ADR-041); LLMs will not
reliably add CAST() wrappers without explicit instruction. The correct fix is typed output
at the source.

### Alt-B: List-enrichment semantics for iocs_value (enrich all, return typed list)

Rejected. Returning `List(Int64)` from `threat_score(iocs_value)` would require DataFusion
list-arithmetic support for `>=` comparisons, which is not standard DataFusion behavior.
It also creates ambiguity for analysts: "threat_score >= 75" with a list column has unclear
semantics (any-element >= 75? all-elements? max-element?). The scalar-first pattern
(established by NVD's `device_cves_first`) is unambiguous and already proven.

### Alt-C: Separate scalar-extraction UDF from typed-output UDF

Rejected. A two-step `first_ioc(iocs_value) | enrich threat_score(...)` approach adds
cognitive overhead for analysts and requires a new built-in UDF. The `_first` column
is already extractable at the fixture/adapter layer with zero query complexity cost.

### Alt-D: Validation-only fix (E-INFUSE-013 sub-condition 8) without typing fix

Rejected. Validating that `source_column` is present prevents the double-encoding bug,
but `cvss_base_score >= 8.0` would still compare lexicographically. Both bugs (missing
source_column AND wrong return type) have the same root cause (ignored `output_type`)
and must be fixed together.

---

## Consequences

### Positive

- `threat_score >= 75` and `cvss_base_score >= 8.0` evaluate as numeric comparisons.
  Silent lexicographic comparison bugs are structurally eliminated for all enrichment
  columns.
- ThreatIntel enrichment columns return bare typed values (`95`, `true`, `["virustotal"]`),
  not doubly-encoded JSON objects. T13 demo Act 4 can show clean numeric output.
- INV-ENRICH-TYPED-001 is a machine-checkable invariant: adversarial review can grep for
  `DataType::Utf8` in `return_type()` implementations to catch regressions.
- NVD enrichment was accidentally correct (scalar + source_column); it now has a
  principled basis (D1 mapping enforces Float64 for `cvss_base_score`).
- Spec-load validation (E-INFUSE-013 sub-conditions 7/8) prevents silent misconfiguration
  from new infusion authors who omit `source_column` on plugin-type fields.

### Tradeoffs

- **`_first` column addition expands sensor TOML schema.** Two new columns
  (`iocs_value_first`, `behaviors_ioc_value_first`) appear in `prism_describe` output
  for cyberint_alerts and crowdstrike_detections. These are additive and non-breaking.
- **threatintel.infusion.toml changes are spec-breaking for any consumer using the
  old JSON-encoded output format.** The old output was documented as a presenter-awareness
  issue (OBS-1), not a contract. No downstream consumer should be depending on the
  doubly-encoded format.
- **ENRICH-1 list-dispatch path is restricted to json-typed fields.** Any future infusion
  author who wants list-enrichment behavior for non-json fields must use a json-typed
  intermediate field and parse in the query layer. This is intentional.

---

## Enforcement

A new `#[test]` in `crates/prism-query/src/infusion_udf.rs` named
`test_return_type_matches_output_type_for_all_declared_types` must verify that
`InfusionAsyncUdf::return_type()` returns the correct `DataType` for each of the
six `output_type` strings. This test is the primary regression guard for INV-ENRICH-TYPED-001.

A spec-load integration test in `crates/prism-spec-engine/tests/` named
`test_plugin_type_field_without_source_column_rejected_e_infuse_013` must verify
that loading a plugin-type infusion spec without `source_column` on any field
produces E-INFUSE-013.

---

## Changelog

| Version | Date | Author | Change |
|---|---|---|---|
| 1.1 | 2026-07-03 | architect | Post-ratification datetime reconciliation. D1 `"datetime"` → `DataType::Utf8` row: replaced stale "deferred" language with the correct consistency rationale. Cross-references added: `spec_driven_adapter.rs:886` (authoritative mapping), `pipe_sql_emitter.rs:817-818` (confirming comment), `high002_plan_pinning_tests.rs:169/191/313` (load-bearing test documentation), ADR-044 D4 / BC-2.11.021 (NOW() string-comparison pattern). Documents that Datetime=Utf8 is intentional pipeline-wide design (not deferral); explains the two-representation-split risk of using Arrow Timestamp for enrichment while sensors use Utf8; notes the stale `column.rs` doc-comment as a blast-radius fix item. `related_adrs` extended: [ADR-024, ADR-040, ADR-044]. |
| 1.0 | 2026-07-03 | architect | Initial PROPOSED. Closes DRIFT-PIVOT-UDF-OUTPUT-TYPE-001. D1 type-mapping table; D2 coercion semantics + E-INFUSE-014; D3 mandatory source_column for plugin-type; D4 scalar-input consistency rule; D5 PrismQL comparison semantics; D6 INV-ENRICH-TYPED-001. Blast-radius list. Recommended BC-2.19.001 amendments. |
