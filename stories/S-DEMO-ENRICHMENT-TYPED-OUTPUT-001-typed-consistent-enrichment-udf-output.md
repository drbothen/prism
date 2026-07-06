---
document_type: story
story_id: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
title: "Typed & Consistent Enrichment UDF Output — ADR-051 D1–D6 Implementation"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "1.9"
level: "L4"
producer: story-writer
timestamp: "2026-07-05T00:00:00Z"
created: "2026-07-05"
modified: "2026-07-06T14:00:00Z"
tdd_mode: strict
subsystems: [SS-09, SS-10, SS-19]
# Subsystem anchor justifications:
#   SS-09 (Query Engine) owns prism-query, which contains infusion_udf.rs — the primary
#   implementation site for output_arrow_type(), invoke_async_with_args(), coerce_to_typed().
#   SS-10 (Spec Engine) owns prism-spec-engine, which contains InfusionLoader::validate —
#   the site for E-INFUSE-013 sub-condition 7/8 enforcement.
#   SS-19 (Infusion / Enrichment) is the semantic subsystem covering all enrichment contracts
#   including BC-2.19.001, the infusion TOML specs, and the UDF registration path.
#   All three are required because the fix spans query-side UDF output typing (SS-09),
#   spec-load validation (SS-10), and the enrichment domain invariant (SS-19).
target_module: prism-query
crates_touched: [prism-query, prism-spec-engine, prism-core, prism-dtu-cyberint, prism-dtu-crowdstrike, prism-dtu-threatintel, prism-mcp]
behavioral_contracts: [BC-2.19.001, BC-2.16.002]
# BC array propagation:
# BC-2.19.001 v2.2 (amended 2026-07-05): primary contract governing INV-ENRICH-TYPED-001
# (typed UDF output), plugin-type source_column enforcement (E-INFUSE-013 sub-cond 8),
# E-INFUSE-013 sub-cond 7 (unknown output_type), and E-INFUSE-014 (TypeCoercionFailed).
# Every AC in this story traces back to a BC-2.19.001 postcondition or invariant.
#
# BC-2.16.002 v1.96: SAP-1 standing obligation — a Canonical Structured Event Catalog row
# for event_type = "infusion.coercion_failed" MUST be registered before the implementation
# PR merges (per ADR-051 D2 and CLAUDE.md §SAP-1). AC-012 anchors this obligation.
# Both BCs cited by ACs below; bidirectional trace requirement satisfied.
# BC status: both active (POL-14 lifecycle_status: active for both).
verification_properties: []
# VP note: VP-048 (Kani proof for InfusionRegistry::load_spec) governs the INV-INFUSE-001
# load-count invariant; this story's changes are additive (new postconditions, new errors)
# and do not invalidate the existing Kani proof. No new VP files created by this story.
depends_on:
  - S-DEMO-ENRICHMENT-PIVOT-003
  # Dependency anchor: PIVOT-003 delivered the threatintel.infusion.toml spec (written by
  # PIVOT-002) as the operational infusion driving the T13 demo pivot queries. This story
  # rewrites that spec (adding source_column, changing input_field to iocs_value_first).
  # The IOC-stamping infrastructure from PIVOT-003 is the direct substrate that this story
  # targets: PIVOT-003's nested array structures (iocs[].value for cyberint,
  # behaviors[].ioc_value for crowdstrike) are the structures that this story's JSONPath
  # source_path declarations in the sensor TOMLs resolve against. Without PIVOT-003's
  # IOC-stamping code on develop, the JSONPath source_path columns declared in AC-010/AC-011
  # would resolve to null at query time.
  #
  # S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 is MERGED on develop (PR #214, commit 11edbd36):
  # the parse_datetime_to_micros helper and Timestamp(Microsecond, Some("UTC")) DataType
  # mapping are already available in spec_driven_adapter.rs. This story reuses that helper
  # for datetime-output UDFs. No depends_on entry needed (already on develop).
blocks: []
# Blocks: T13 capstone demo Act 4 (numeric threat_score output) — demo objective, not story ID.
points: 13
# Points justification:
#   1. output_arrow_type() helper + return_type() delegation: 1 pt
#   2. invoke_async_with_args() typed array dispatch (5 array types + ENRICH-1 retention): 2 pt
#   3. coerce_to_typed() helper with 5 branches + JSON-list detection: 1.5 pt
#   4. InfusionError::TypeCoercionFailed variant + tracing::warn! emission: 0.5 pt
#   5. InfusionLoader::validate sub-cond 7 (unknown output_type): 0.5 pt
#   6. InfusionLoader::validate sub-cond 8 (plugin-type missing source_column): 0.5 pt
#   7. threatintel.infusion.toml rewrite (source_column + iocs_value_first): 0.5 pt
#   8. cyberint sensor TOML + crowdstrike sensor TOML + DTU fixture generators: 1.5 pt
#   9. BC-2.16.002 SAP-1 catalog row addition (infusion.coercion_failed): 0.5 pt
#  10. prism-mcp resources.rs example update + t13-preflight-audit.py update: 0.5 pt
#  11. TD-VSDD-060 sibling sweep across crates_touched: 0.5 pt
#  12. Red Gate test suite (23 tests across 4 crates): 3.5 pt
#   Total: 13 pts
estimated_days: 4
risk: HIGH
# Risk justification:
#   Changing return_type() from DataType::Utf8 to typed types is a breaking behavioral change
#   for all downstream callers of infusion UDFs in DataFusion. If invoke_async_with_args()
#   returns Int64Array but return_type() still declares Utf8, DataFusion will panic on the
#   array/type mismatch (Arrow schema enforcement). The two functions MUST change atomically.
#   SAP-1 requires BC-2.16.002 catalog row in the same commit as the tracing emission — if
#   the implementer commits the emission without the catalog row, the adversary will find a P1.
#   SAP-2 applies to all sensor TOML changes — adversary must read DTU types.rs/generator.rs
#   before validating TOML column declarations.
red_gate_tests: 23
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "return_type() and invoke_async_with_args() MUST change in the same atomic commit.
     If one changes without the other, DataFusion will panic with an Arrow schema violation
     (the array type returned by invoke_async_with_args does not match the declared return_type).
     Red Gate test test_return_type_matches_output_type_for_all_declared_types guards this
     invariant by asserting both return_type() and the array type produced by invoke_async."
  - "SAP-1 obligation (BC-2.16.002 catalog row): the tracing::warn!(event_type = 'infusion.coercion_failed')
     emission MUST be accompanied by a BC-2.16.002 Canonical Structured Event Catalog row in the
     same commit. Implementer MUST add the catalog row to BC-2.16.002 in the same commit that
     adds the tracing emission. Adversary SAP-1 probe: rg 'event_type.*infusion.coercion_failed'
     crates/ --type rust — every hit must have a corresponding BC-2.16.002 row."
  - "SAP-2 obligation (sensor TOML parity): before declaring iocs_value_first in
     cyberint.sensor.toml, read crates/prism-dtu-cyberint/src/ generator code to confirm
     the iocs_value_first field is emitted there. Same for behaviors_ioc_value_first in
     crowdstrike. Every TOML column MUST match an emitted field in the DTU generator after
     this story's generator changes. The SAP-2 probe is the adversarial review gate."
  - "parse_datetime_to_micros reuse (no duplication): the datetime→Timestamp coercion branch
     in coerce_to_typed() MUST call the same parse_datetime_to_micros function used by
     spec_driven_adapter.rs column_type_to_arrow. Do NOT implement a new date parser.
     If the helper is not pub in its current module, make it pub(crate) or move it to a
     shared location. Duplicating the implementation violates ADR-052 D2."
  - "threatintel.infusion.toml rewrite impacts live demo queries: the T13 canonical query
     uses iocs_value_first as the enrichment input (not iocs_value). The old iocs_value
     field does NOT produce a usable typed threat_score. After this story, the T13 queries
     must be verified against the rewritten spec using the demo server."
  - "ENRICH-1 double-encoding defect (ADV-P11-OBS-001 — adjudicated DEFECT 2026-07-06):
     threat_sources MUST use input_field = 'iocs_value_first' (scalar companion), NOT
     'iocs_value' (JSON list). Using iocs_value produces double-encoded output:
       (a) iocs_value = '[\"1.2.3.4\"]' → ENRICH-1 fires (is_json_output && starts_with('['))
       (b) for element '1.2.3.4': plugin returns {threat_sources: ['greynoise','abuseipdb']}
       (c) project_value extracts threat_sources Array via other.to_string() → String '[\"greynoise\",\"abuseipdb\"]'
       (d) list_results = vec!['[\"greynoise\",\"abuseipdb\"]'] (Vec<String> of one element)
       (e) serde_json::to_string(&list_results) → '[\"[\\\"greynoise\\\",\\\"abuseipdb\\\"]\"]'
       RESULT: outer array wrapping a JSON-encoded array string — Failure A class double-encoding.
     With iocs_value_first = '1.2.3.4' (scalar): ENRICH-1 does NOT fire; project_value
     extracts threat_sources Array via other.to_string() → String '[\"greynoise\",\"abuseipdb\"]'
     stored directly as Utf8 column value — correct single-encoding.
     AC-009 updated to require input_field = 'iocs_value_first' for threat_sources.
     Implementer MUST change input_field = 'iocs_value' → 'iocs_value_first' in
     specs/infusions/threatintel.infusion.toml. No code change to infusion_udf.rs needed.
     Test-writer MUST add test: test_threat_sources_json_output_no_double_encoding (see AC-009)."
traces_to: [DRIFT-PIVOT-UDF-OUTPUT-TYPE-001, ADR-051]
supersedes: []
---

# S-DEMO-ENRICHMENT-TYPED-OUTPUT-001: Typed & Consistent Enrichment UDF Output

Closes DRIFT-PIVOT-UDF-OUTPUT-TYPE-001. Implements ADR-051 (ACCEPTED v1.4, 2026-07-06)
decisions D1–D6: typed `output_arrow_type()` helper, typed array construction in
`invoke_async_with_args()`, `coerce_to_typed()` with NULL + E-INFUSE-014, spec-load
validation of plugin-type `source_column` (E-INFUSE-013 sub-condition 8) and unknown
`output_type` (sub-condition 7), scalar `_first` companion columns for cyberint/crowdstrike,
and `threatintel.infusion.toml` rewrite that closes the doubly-encoded JSON bug.

**Context (T13 OBS-1 defects now to be fixed):**
- Failure A: `enrich threat_score(iocs_value)` returned `["{\"threat_score\":95,...}"]` —
  doubly-encoded JSON because no `source_column` on any ThreatIntel field and `iocs_value`
  is a JSON-list string. Root cause: `project_value()` passthrough serializes the entire
  plugin response object when `source_column` is absent.
- Failure B: `cvss_base_score >= 8.0` evaluated lexicographically (`"10.0" >= "8.0"` was
  `false`) because `return_type()` hardcoded `DataType::Utf8` regardless of `output_type`.

After this story, `threat_score >= 75` evaluates as `Int64 >= Int64(75)` and
`cvss_base_score >= 8.0` evaluates as `Float64 >= Float64(8.0)`.

---

## Narrative

As a SOC analyst in the T13 capstone demo, I want enrichment columns to carry correct
Arrow DataTypes so that `| filter threat_score >= 75` and `| filter cvss_base_score >= 8.0`
evaluate as numeric comparisons, ThreatIntel output shows `95` (not `["{...}"]`), and any
type coercion failure produces a NULL row with a diagnostic log line — so that the demo
faithfully represents production enrichment behavior and numeric filters work correctly.

---

## Behavioral Contracts

| BC | Version | Title | Key Clauses Used |
|----|---------|-------|-----------------|
| BC-2.19.001 | v2.2 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | INV-ENRICH-TYPED-001; INV-INFUSE-001 (extended); Typed UDF output postcondition; Plugin-type field projection postcondition; E-INFUSE-013 sub-conditions 7 and 8; E-INFUSE-014; EC-19-008; EC-19-009; TV-19-001-typed-{integer,float,boolean,datetime}; TV-19-001-coerce-fail-{integer,datetime}; TV-19-001-json-list-typed-output; TV-19-001-plugin-no-source-col; TV-19-001-unknown-output-type |
| BC-2.16.002 | v1.96 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | SAP-1 Canonical Structured Event Catalog: new row for event_type = "infusion.coercion_failed" must be added in same commit as the tracing::warn! emission (per ADR-051 D2 E-INFUSE-014 section) |

---

## Acceptance Criteria

### AC-001 — output_arrow_type() helper returns correct DataType for all 6 output_type values
(traces to BC-2.19.001 v2.2 "Typed UDF output (INV-ENRICH-TYPED-001)" postcondition — INV-ENRICH-TYPED-001 clause 1; ADR-051 D1 canonical mapping table)

Given `InfusionAsyncUdf` with a descriptor carrying each of the six recognized `output_type` values,
when `output_arrow_type()` is called on the UDF,
then it returns:
- `"string"` → `DataType::Utf8`
- `"integer"` → `DataType::Int64`
- `"float"` → `DataType::Float64`
- `"boolean"` → `DataType::Boolean`
- `"json"` → `DataType::Utf8`
- `"datetime"` → `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`

An unrecognized value falls back to `DataType::Utf8` only at the UDF layer (spec-load
validation under AC-006 prevents unrecognized values from reaching this path in production).

Red Gate: `test_return_type_matches_output_type_for_all_declared_types` (ADR-051 §Enforcement)

### AC-002 — return_type() delegates to output_arrow_type(); no hardcoded DataType::Utf8 for typed fields
(traces to BC-2.19.001 v2.2 INV-INFUSE-001 extension — "the registered UDF's return_type() must return the Arrow DataType mapped from output_type per ADR-051 D1"; ADR-051 D1 implementation note)

Given `InfusionAsyncUdf::return_type()` implementation after this story,
when the TD-VSDD-060 grep is run:
```bash
rg 'return_type.*Utf8' crates/prism-query/src/infusion_udf.rs
```
then the result is ZERO matches (no hardcoded `DataType::Utf8` in `return_type()`).

NOTE: `output_type.*Utf8` is intentionally excluded from this check — `output_arrow_type()` contains a
sanctioned `_ => DataType::Utf8` fallback arm for unrecognized `output_type` values, and the implementation
contains legitimate `*output_type != DataType::Utf8` guard comparisons. These are correct behavior, not
violations of this invariant. The zero-match check applies only to the `return_type()` function body.

`return_type()` MUST delegate to `output_arrow_type()` rather than hardcoding a type.

Red Gate: covered by `test_return_type_matches_output_type_for_all_declared_types` (tests return_type()
via the DataFusion `ScalarUDF` trait path — DataFusion calls `return_type()` when planning; the test
constructs a UDF with each output_type and asserts the planned return type)

### AC-003 — invoke_async_with_args() builds correctly typed Arrow arrays per output_arrow_type()
(traces to BC-2.19.001 v2.2 "Typed UDF output (INV-ENRICH-TYPED-001)" postcondition — INV-ENRICH-TYPED-001 clause 2; ADR-051 D1 implementation note; TV-19-001-typed-integer/float/boolean/datetime)

Given an `InfusionAsyncUdf` with `output_type` set to a typed value, and a mock enrichment response returning the appropriate projected value,
when `invoke_async_with_args()` produces a `ColumnarValue::Array`,
then the Arrow array type matches `output_arrow_type()`:
- `"integer"` → `Int64Array` containing the parsed `i64` value (or NULL on failure)
- `"float"` → `Float64Array` containing the parsed `f64` value (or NULL on failure)
- `"boolean"` → `BooleanArray` with case-insensitive coercion (`"true"/"1"/"yes"` → true, `"false"/"0"/"no"` → false)
- `"datetime"` → `TimestampMicrosecondArray` with timezone `"UTC"` containing microseconds-since-epoch via `parse_datetime_to_micros` (ADR-052; reused from `spec_driven_adapter.rs` — do NOT duplicate the parser)
- `"string"` and `"json"` → `StringArray` (passthrough; no coercion)

For `output_type = "json"` fields, the ENRICH-1 list-dispatch path is RETAINED (do NOT remove it).

Red Gate: `test_invoke_async_with_args_returns_int64_array_for_integer_output_type` (fix-burst-2: also asserts `int_arr.value(0) == 42_i64` — catches null-row and wrong-type regressions)
Red Gate: `test_invoke_async_with_args_returns_float64_array_for_float_output_type` (fix-burst-2: also asserts `float_arr.value(0) ≈ 3.14` — numeric equality within 1e-10)
Red Gate: `test_invoke_async_with_args_returns_boolean_array_for_boolean_output_type` (fix-burst-2: also asserts `bool_arr.value(0) == true`)
Red Gate: `test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type` (fix-burst-2: also asserts `ts_arr.value(0) == expected_micros`)

### AC-004 — coerce_to_typed() coercion failure: NULL output + E-INFUSE-014 tracing::warn!
(traces to BC-2.19.001 v2.2 E-INFUSE-014 error condition — "output row is NULL, not a panic, not a passthrough string, not an empty string"; INV-ENRICH-TYPED-001 clause 5; TV-19-001-coerce-fail-integer/datetime)

Given a UDF with `output_type = "integer"` / `"float"` / `"boolean"` / `"datetime"` and a projected value that cannot be coerced to the declared type (e.g., `"not-a-number"` for integer, `"xyz"` for float, `"maybe"` for boolean, `"not-a-date"` for datetime),
when `invoke_async_with_args()` processes the failing row,
then:
1. The output array contains NULL at that row position (not a panic, not a passthrough string)
2. `tracing::warn!(event_type = "infusion.coercion_failed", field_name = ..., infusion_id = ..., declared_type = ..., truncated_value = ...)` is emitted (`truncated_value` = first 50 chars via `value.chars().take(50).collect::<String>()`, char-based, genuinely UTF-8-safe; AD-017 guard; `declared_type` = `output_type` spec-vocabulary string, e.g., `"integer"` — NOT Arrow debug format `Int64`)
3. The query does NOT fail — it returns successfully with NULL in the typed column
4. Recurrence is per-row, NOT aggregated per-batch

Coercion failure triggers (ADR-051 D2):
- `i64::from_str(s.trim())` failure for integer
- `f64::from_str(s.trim())` failure for float
- value not in `{"true","1","yes","false","0","no"}` (case-insensitive) for boolean
- `parse_datetime_to_micros` failure for datetime
- `Number.as_i64()` returns `None` for JSON Number projected into integer field

Red Gate: `test_coerce_to_typed_integer_failure_produces_null_e_infuse_014`
Red Gate: `test_coerce_to_typed_float_failure_produces_null_e_infuse_014`
Red Gate: `test_coerce_to_typed_boolean_unrecognized_value_produces_null_e_infuse_014`

### AC-005 — InfusionError::TypeCoercionFailed variant with correct E-INFUSE-014 message format
(traces to BC-2.19.001 v2.2 E-INFUSE-014 implementation obligation — "new InfusionError::TypeCoercionFailed { field_name, infusion_id, declared_type, truncated_value } variant"; ADR-051 D2)

Given `prism-core/src/error.rs` (or the module owning `InfusionError`) after this story,
when the `TypeCoercionFailed` variant is inspected,
then:
```rust
InfusionError::TypeCoercionFailed {
    field_name: String,
    infusion_id: String,
    declared_type: String,
    truncated_value: String,  // first 50 chars of projected value (AD-017)
}
```
with `#[error("E-INFUSE-014: enrichment field '{field_name}' (infusion '{infusion_id}'): declared output_type is '{declared_type}', but projected value '{truncated_value}' (first 50 chars) cannot be coerced; row produces NULL")]`.

The variant MUST be `#[non_exhaustive]` per CLAUDE.md §Conventions non-exhaustive discipline
(all public types in prism-core require `#[non_exhaustive]`).

Red Gate: covered by AC-004 Red Gate tests (they assert the Display output matches the E-INFUSE-014 format)

### AC-006 — InfusionLoader::validate rejects plugin-type field without source_column (E-INFUSE-013 sub-condition 8)
(traces to BC-2.19.001 v2.2 Plugin-type field projection postcondition; E-INFUSE-013 sub-cond 8; EC-19-009; TV-19-001-plugin-no-source-col; ADR-051 D3)

Given `InfusionLoader::validate` is called during spec loading,
when a `[[infusion.fields]]` entry has `type = "plugin"` (or the parent infusion is `type = "plugin"`) and no `source_column` is declared on that field,
then the spec is rejected at parse time with `E-INFUSE-013` sub-condition 8:
```
"E-INFUSE-013: invalid field name 'source_column' in infusion spec '{spec_path}':
 plugin-type field '{name}' in infusion '{infusion_id}' must declare 'source_column'
 to project a specific field from the plugin response object; without source_column
 the full response object is serialized (DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 root cause)"
```
The spec is REJECTED (not loaded). Other infusion specs continue loading unaffected.

Verification: `rg 'E-INFUSE-013' crates/ --type rust` MUST hit the updated validation path.

Red Gate: `test_plugin_type_field_without_source_column_rejected_e_infuse_013` (ADR-051 §Enforcement)

### AC-007 — InfusionLoader::validate rejects unknown output_type (E-INFUSE-013 sub-condition 7)
(traces to BC-2.19.001 v2.2 E-INFUSE-013 sub-condition 7; TV-19-001-unknown-output-type; ADR-051 D3)

Given `InfusionLoader::validate` is called during spec loading,
when a `[[infusion.fields]]` entry has an `output_type` value not in the recognized set `{string, integer, float, boolean, json, datetime}` (e.g., `"bytes"`, `"numeric"`, `"timestamp"`),
then the spec is rejected at parse time with `E-INFUSE-013` sub-condition 7:
```
"E-INFUSE-013: invalid field name 'output_type' in infusion spec '{spec_path}':
 output_type '{value}' is not a recognized type; valid values: string, integer, float,
 boolean, json, datetime (datetime maps to Timestamp(µs,UTC) per ADR-051 v1.2 / ADR-052)"
```

Red Gate: `test_unknown_output_type_rejected_e_infuse_013_sub_condition_7`

### AC-008 — JSON-list input to typed-output UDF produces NULL + E-INFUSE-014; json-typed ENRICH-1 retained
(traces to BC-2.19.001 v2.2 EC-19-008 — "JSON-list string input (leading `[`) provided to typed-output UDF produces NULL + E-INFUSE-014"; INV-ENRICH-TYPED-001 clause 4; TV-19-001-json-list-typed-output; ADR-051 D4)

Given a UDF with `output_type = "integer"` (or float, boolean, datetime) and input value `"[\"hash1\",\"hash2\"]"` (leading `[` JSON-list string),
when `invoke_async_with_args()` processes the value,
then the output row is NULL and E-INFUSE-014 warning is emitted with `declared_type` set to the field's `output_type`.

Given a UDF with `output_type = "json"` and input value `"[\"hash1\",\"hash2\"]"`,
then the ENRICH-1 list-dispatch path is RETAINED and processes the list normally (this is intentional behavior for json-typed fields per ADR-051 D4).

Red Gate: `test_json_list_input_to_typed_output_udf_produces_null_e_infuse_014`

### AC-009 — threatintel.infusion.toml rewritten: source_column on all fields, iocs_value_first as input_field for typed fields
(traces to BC-2.19.001 v2.2 Plugin-type field projection postcondition — all plugin-type fields must declare source_column; INV-ENRICH-TYPED-001 clause 3; ADR-051 D3/D4 Required Spec Changes)

Given `specs/infusions/threatintel.infusion.toml` after this story,
when the file is inspected,
then:
- `threat_score` field: `source_column = "threat_score"`, `input_field = "iocs_value_first"`, `output_type = "integer"`
- `threat_is_known_malicious` field: `source_column = "threat_is_known_malicious"`, `input_field = "iocs_value_first"`, `output_type = "boolean"`
- `threat_sources` field: `source_column = "threat_sources"`, `input_field = "iocs_value_first"`, `output_type = "json"` (MUST use iocs_value_first — using iocs_value causes double-encoding; see ADV-P11-OBS-001 in risk_mitigations)

No field in the plugin-type infusion has an absent `source_column` (EC-19-009 would reject it at spec-load time).

The post-fix T13 canonical query uses `iocs_value_first` as the enrichment input column for every field including `threat_sources`:
```prismql
FROM cyberint_alerts
| where severity = "high"
| enrich threat_score(iocs_value_first)
| enrich threat_is_known_malicious(iocs_value_first)
| enrich threat_sources(iocs_value_first)
| where threat_is_known_malicious = true
| sort threat_score desc
```

Note: PrismQL `| enrich` uses per-field UDF names (registered as `threat_score`, `threat_is_known_malicious`,
`threat_sources`), NOT the infusion name (`threat_intel`). The `input_field` in the TOML spec documents
which column the analyst should pass; using `iocs_value_first` (scalar) avoids the ENRICH-1 list-dispatch
path and prevents the double-encoding defect (ADV-P11-OBS-001).

Red Gate: `test_threatintel_toml_has_source_column_and_iocs_value_first_input_field`

Red Gate: `test_threat_sources_json_output_no_double_encoding` (NEW — ADV-P11-OBS-001)
Source returns JSON object `{"threat_sources": ["greynoise","abuseipdb"], "threat_score": 95, ...}`.
Descriptor: `output_type = "json"`, `source_column = "threat_sources"`, `input_field = "iocs_value_first"` (scalar).
Input to UDF: scalar string `"1.2.3.4"`.
Expected output: Utf8 column value = `["greynoise","abuseipdb"]`.
Verify: `serde_json::from_str(output)` parses as JSON array of 2 plain string elements ("greynoise", "abuseipdb") — NOT double-encoded. Crate: prism-query (infusion_udf.rs tests or a dedicated test file).

### AC-010 — Scalar _first companion columns declared in sensor TOMLs (D4)
(traces to BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4 — "typed input columns require scalar input"; ADR-051 D4 canonical input pattern / Required Spec Changes table)

Given `specs/sensors/cyberint.sensor.toml` after this story,
when the `cyberint_alerts` `[[tables.columns]]` section is inspected,
then `iocs_value_first` column is present with `column_type = "string"` and description documenting it as the first IOC value from the `iocs_value` array.

Given `specs/sensors/crowdstrike.sensor.toml` after this story,
when the `crowdstrike_detections` `[[tables.columns]]` section is inspected,
then `behaviors_ioc_value_first` column is present with `column_type = "string"` and description documenting it as the first IOC value from the `behaviors_ioc_value` array.

Both additions are NON-BREAKING (additive columns; no existing column renamed or removed).

Red Gate: `test_cyberint_sensor_toml_has_iocs_value_first_column`
Red Gate: `test_crowdstrike_sensor_toml_has_behaviors_ioc_value_first_column`

### AC-011 — _first scalar columns populated via JSONPath extraction from nested arrays in DTU records
(traces to BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4 Blast Radius — cyberint/crowdstrike _first companion columns via spec-driven adapter JSONPath source_path extraction; SAP-2 TOML↔DTU parity)

Given the spec-driven adapter processing a Cyberint alert record after this story,
when the `cyberint_alerts` table is queried for the `iocs_value_first` column,
then `iocs_value_first` is populated by JSONPath `source_path = "$.iocs[0].value"` extraction
from the alert record's nested `iocs` array — NOT from a pre-computed top-level scalar field
on the alert surface record. The spec-driven adapter performs this JSONPath extraction; the DTU
fixture generator emits the nested `iocs` array structure from which the path is resolved.

Given the spec-driven adapter processing a CrowdStrike detection record after this story,
when the `crowdstrike_detections` table is queried for the `behaviors_ioc_value_first` column,
then `behaviors_ioc_value_first` is populated by JSONPath `source_path = "$.behaviors[0].ioc_value"`
extraction from the detection's nested `behaviors` array — NOT from a pre-computed top-level scalar
field on the detection surface record.

SAP-2 compliance: adversary MUST verify that the JSONPath `source_path` values declared in the
sensor TOML `[[tables.columns]]` entries resolve against fields actually present in the DTU fixture
generator's emitted nested record structures (`iocs[].value` for cyberint; `behaviors[].ioc_value`
for crowdstrike). The TOML columns declared in AC-010 MUST match fields reachable via JSONPath in
the structures emitted by the generators.

Red Gate: `test_ac011_cyberint_alerts_iocs_value_first_column_via_jsonpath` (prism-dtu-cyberint/src/generator.rs; reads source_path from cyberint.sensor.toml; uses `generate_with_scenario_iocs` to stamp `iocs[0].value`; asserts JSONPath `$.iocs[0].value` resolves to expected IOC)
Red Gate: `test_ac011_crowdstrike_detections_behaviors_ioc_value_first_column_via_jsonpath` (prism-dtu-crowdstrike/src/generator.rs; reads source_path from crowdstrike.sensor.toml; asserts `$.behaviors[0].ioc_value` JSONPath value; also asserts top-level `behaviors_ioc_value_first` scalar field is ABSENT from the generated record)

### AC-012 — BC-2.16.002 Canonical Structured Event Catalog gains row for infusion.coercion_failed (SAP-1)
(traces to BC-2.16.002 v1.96 SAP-1 Canonical Structured Event Catalog standing obligation; BC-2.19.001 v2.2 E-INFUSE-014 — "BC-2.16.002 catalog row required for event_type = 'infusion.coercion_failed' (SAP-1)")

Given `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` after this story,
when the Canonical Structured Event Catalog table is inspected,
then a row exists for `event_type = "infusion.coercion_failed"` with:
- Full field schema: `field_name`, `infusion_id`, `declared_type`, `truncated_value` (first 50 chars), severity (warn), and the E-INFUSE-014 message template
- Audit role: per-row coercion failure diagnostic (not a query error; NULL row is the primary signal)
- Recurrence policy: one log line per failing row per UDF call; NOT aggregated per-batch

This row MUST be added in the same commit that adds the `tracing::warn!(event_type = "infusion.coercion_failed", ...)` emission in `infusion_udf.rs` (SAP-1 standing rule from CLAUDE.md §SAP-1).

Red Gate: N/A (process/spec check — adversary SAP-1 probe verifies this post-merge)

### AC-013 — NVD cvss_base_score UDF returns Float64Array; numeric comparison semantics correct (D5)
(traces to BC-2.19.001 v2.2 TV-19-001-typed-float; ADR-051 D5 PrismQL comparison semantics — "cvss_base_score >= 8.0 evaluates as Float64 >= Float64(8.0) — numeric, correct")

Given `nvd.infusion.toml` (no spec changes required — NVD already has source_column and device_cves_first; AC-001 D1 typing fix covers this) after the infusion_udf.rs changes in this story,
when the `cvss_base_score` UDF is registered and a plan is made for:
```prismql
| filter cvss_base_score >= 8.0
```
then DataFusion resolves `cvss_base_score` as `DataType::Float64` (not `Utf8`), and the comparison is numeric. Specifically: `"10.0" >= "8.0"` evaluated lexicographically as `false` (pre-fix) becomes `10.0 >= 8.0` evaluated numerically as `true` (post-fix).

NOTE: `cvss_base_score` has `output_type = "float"` in `nvd.infusion.toml` — no TOML change needed. The fix is entirely in `return_type()` / `invoke_async_with_args()`.

Red Gate: covered by `test_invoke_async_with_args_returns_float64_array_for_float_output_type` (exercises the Float64 path with a string projected value "8.1" producing Float64Array; fix-burst-2: also asserts `float_arr.value(0) ≈ 3.14` — numeric row value, not just schema type)

### AC-014 — datetime-output UDF returns Timestamp(µs,UTC) using parse_datetime_to_micros; no new date parser
(traces to BC-2.19.001 v2.2 TV-19-001-typed-datetime — "produces TimestampMicrosecondArray consistent with sensor Datetime columns"; ADR-051 D1 datetime row; ADR-052 consistency rationale)

Given an `InfusionAsyncUdf` with `output_type = "datetime"` and a projected RFC-3339 string `"2026-07-05T00:00:00Z"`,
when `invoke_async_with_args()` processes the value,
then the output is a `TimestampMicrosecondArray` with timezone `"UTC"` containing the correct microseconds-since-epoch value for `2026-07-05T00:00:00Z`.

Implementation MUST reuse `parse_datetime_to_micros` (from the module used by `spec_driven_adapter.rs` `column_type_to_arrow` — ADR-052 D2). No new ISO-8601 / RFC-3339 parser may be introduced. The resulting column type is `DataType::Timestamp(Microsecond, Some("UTC"))` — consistent with sensor `Datetime` columns after ADR-052, enabling cross-column predicates like `| filter sensor_timestamp > enriched_event_time` without DataFusion type errors.

Red Gate: `test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type` (fix-burst-2: also asserts `ts_arr.value(0) == expected_micros` — verifies the parsed microseconds-since-epoch value, not just that a TimestampMicrosecondArray was produced)

---

## Red Gate Test Plan

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_return_type_matches_output_type_for_all_declared_types` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 1; ADR-051 §Enforcement | unit |
| 2 | `test_plugin_type_field_without_source_column_rejected_e_infuse_013` | prism-spec-engine | BC-2.19.001 v2.2 Plugin-type field projection postcondition; EC-19-009; ADR-051 §Enforcement | unit/integration |
| 3 | `test_invoke_async_with_args_returns_int64_array_for_integer_output_type` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 2; TV-19-001-typed-integer; fix-burst-2 RGT-002: also asserts `int_arr.value(0) == 42_i64` | unit |
| 4 | `test_invoke_async_with_args_returns_float64_array_for_float_output_type` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 2; TV-19-001-typed-float; fix-burst-2 RGT-003: also asserts `float_arr.value(0) ≈ 3.14` | unit |
| 5 | `test_invoke_async_with_args_returns_boolean_array_for_boolean_output_type` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 2; TV-19-001-typed-boolean; fix-burst-2 RGT-004: also asserts `bool_arr.value(0) == true` | unit |
| 6 | `test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 2; TV-19-001-typed-datetime; fix-burst-2 RGT-005: also asserts `ts_arr.value(0) == expected_micros` | unit |
| 7 | `test_coerce_to_typed_integer_failure_produces_null_e_infuse_014` | prism-query | BC-2.19.001 v2.2 E-INFUSE-014; INV-ENRICH-TYPED-001 clause 5; TV-19-001-coerce-fail-integer | unit |
| 8 | `test_coerce_to_typed_float_failure_produces_null_e_infuse_014` | prism-query | BC-2.19.001 v2.2 E-INFUSE-014; INV-ENRICH-TYPED-001 clause 5 | unit |
| 9 | `test_coerce_to_typed_boolean_unrecognized_value_produces_null_e_infuse_014` | prism-query | BC-2.19.001 v2.2 E-INFUSE-014; INV-ENRICH-TYPED-001 clause 5 | unit |
| 10 | `test_json_list_input_to_typed_output_udf_produces_null_e_infuse_014` | prism-query | BC-2.19.001 v2.2 EC-19-008; INV-ENRICH-TYPED-001 clause 4; TV-19-001-json-list-typed-output | unit |
| 11 | `test_unknown_output_type_rejected_e_infuse_013_sub_condition_7` | prism-spec-engine | BC-2.19.001 v2.2 E-INFUSE-013 sub-cond 7; TV-19-001-unknown-output-type | unit |
| 12 | `test_threatintel_toml_has_source_column_and_iocs_value_first_input_field` | prism-spec-engine | BC-2.19.001 v2.2 Plugin-type field projection postcondition; ADR-051 D3/D4 | unit/spec-load |
| 13 | `test_cyberint_sensor_toml_has_iocs_value_first_column` | prism-spec-engine or sensor spec tests | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4; SAP-2 | unit/parity |
| 14 | `test_crowdstrike_sensor_toml_has_behaviors_ioc_value_first_column` | prism-spec-engine or sensor spec tests | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4; SAP-2 | unit/parity |
| 15 | `test_ac011_cyberint_alerts_iocs_value_first_column_via_jsonpath` | prism-dtu-cyberint (generator.rs) | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4 Blast Radius; SAP-2; reads source_path from cyberint.sensor.toml; uses `generate_with_scenario_iocs` | unit |
| 16 | `test_ac011_crowdstrike_detections_behaviors_ioc_value_first_column_via_jsonpath` | prism-dtu-crowdstrike (generator.rs) | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001 clause 4; ADR-051 D4 Blast Radius; SAP-2; reads source_path from crowdstrike.sensor.toml; asserts `$.behaviors[0].ioc_value`; asserts top-level `behaviors_ioc_value_first` ABSENT | unit |
| 17 | `test_coerce_to_typed_integer_valid_returns_some_number` | prism-query | BC-2.19.001 v2.2 TV-19-001-typed-integer; MED-001+LOW-001: `coerce_to_typed("42", Int64)` returns `Some(Number(42))` — validates coerce_to_typed positive path | unit |
| 18 | `test_coerce_to_typed_float_valid_returns_some_number` | prism-query | BC-2.19.001 v2.2 TV-19-001-typed-float; MED-001+LOW-001: `coerce_to_typed("8.1", Float64)` returns `Some(Number(8.1))` within 1e-10 | unit |
| 19 | `test_coerce_to_typed_boolean_valid_variants_return_some_bool` | prism-query | BC-2.19.001 v2.2 TV-19-001-typed-boolean; MED-001+LOW-001: all true-variants (true/1/yes/TRUE/YES) and false-variants (false/0/no/FALSE/NO) return `Some(Bool(_))` | unit |
| 20 | `test_coerce_to_typed_datetime_valid_returns_some_micros` | prism-query | BC-2.19.001 v2.2 TV-19-001-typed-datetime; MED-001+LOW-001: `coerce_to_typed("2024-01-01T00:00:00Z", Timestamp(µs,UTC))` returns `Some(Number(micros))` | unit |
| 21 | `test_ec002_float_string_to_integer_yields_null` | prism-query | BC-2.19.001 v2.2 E-INFUSE-014; EC-002: `coerce_to_typed("95.7", Int64)` → None (JSON Number float-to-integer precision mismatch; fix-burst-3) | unit |
| 22 | `test_ec006_empty_input_yields_null` | prism-query | BC-2.19.001 v2.2 E-INFUSE-014; EC-006: `coerce_to_typed("", Int64)` → None (empty `iocs_value` array produces `""` first element; fix-burst-3) | unit |
| 23 | `test_threat_sources_json_output_no_double_encoding` | prism-query | BC-2.19.001 v2.2 INV-ENRICH-TYPED-001; ADV-P11-OBS-001: source_column + Vec<String> Array field + scalar input_field (`iocs_value_first`) → output `["greynoise","abuseipdb"]` (valid JSON array, elements are plain strings — NOT double-encoded `["[\"greynoise\",\"abuseipdb\"]"]`) | unit |

**Note on test crate placement:**
- Tests 1–10 and 17–23: live in `crates/prism-query/src/infusion_udf.rs` `#[cfg(test)] mod tests` block
- Tests 11–12: live in `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs` (extend the existing test file per ADR-051 §Blast Radius)
- Tests 13–14: live in sensor spec tests or `enrichment_pivot_002_tests.rs`
- Tests 15–16: live in `crates/prism-dtu-cyberint/src/generator.rs` and `crates/prism-dtu-crowdstrike/src/generator.rs` respectively; exercise JSONPath `source_path` extraction — test 15 uses `generate_with_scenario_iocs`; test 16 additionally asserts the dead top-level `behaviors_ioc_value_first` scalar has been removed from `make_detection_with_ioc`

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| Story spec (this document) | 18k |
| `crates/prism-query/src/infusion_udf.rs` (primary impl) | 15k |
| `crates/prism-spec-engine/src/infusion/loader.rs` | 10k |
| `crates/prism-core/src/error.rs` (InfusionError enum) | 6k |
| BC-2.19.001 v2.2 (spec reference during impl) | 15k |
| ADR-051 v1.4 (decisions D1–D6) | 12k |
| ADR-052 (parse_datetime_to_micros reference) | 8k |
| `specs/infusions/threatintel.infusion.toml` | 2k |
| `specs/sensors/cyberint.sensor.toml` + `crowdstrike.sensor.toml` | 4k |
| `crates/prism-dtu-cyberint/src/` (fixture generator) | 8k |
| `crates/prism-dtu-crowdstrike/src/` (fixture generator) | 8k |
| BC-2.16.002 (SAP-1 catalog addition) | 10k |
| `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs` | 8k |
| `crates/prism-mcp/src/resources.rs` | 6k |
| `scripts/t13-preflight-audit.py` | 4k |
| Total | ~134k |

134k / 200k = 67% of a 200k context window. Within the 20-30% overshoot rule for complex multi-crate stories.

---

## Tasks

Implementation checklist for the TDD implementer:

**Phase A — Red Gate stubs (infusion_udf.rs changes)**
- [ ] Add `output_arrow_type(&self) -> DataType` method to `InfusionAsyncUdf` (private helper) — bodies are `todo!()`
- [ ] Modify `return_type()` to call `output_arrow_type()` instead of returning hardcoded `DataType::Utf8` — bodies are `todo!()`
- [ ] Add `coerce_to_typed()` helper function — body is `todo!()`
- [ ] Modify `invoke_async_with_args()` to dispatch on `output_arrow_type()` — bodies are `todo!()`
- [ ] Run `just iter prism-query` — all 17 Red Gate tests (those in prism-query: tests 1–10 and 17–23) must FAIL

**Phase B — Red Gate stubs (loader.rs changes)**
- [ ] Add sub-condition 7 check (unknown `output_type`) to `InfusionLoader::validate` — body is `todo!()` / returns Ok for now
- [ ] Add sub-condition 8 check (plugin-type missing `source_column`) to `InfusionLoader::validate` — body is `todo!()`
- [ ] Run `just iter prism-spec-engine` — tests 11–14 must FAIL

**Phase C — Red Gate stubs (DTU crate changes)**
- [ ] Add `generate_with_scenario_iocs` stub to `crates/prism-dtu-cyberint/src/generator.rs` — emits `iocs[]` array with `.value` entries in the nested structure; no top-level `iocs_value_first` scalar field (JSONPath `source_path = "$.iocs[0].value"` at adapter layer provides the column)
- [ ] Extend `make_detection_with_ioc` stub in `crates/prism-dtu-crowdstrike/src/generator.rs` to emit `behaviors[].ioc_value` nested structure; no top-level `behaviors_ioc_value_first` scalar field added (RGT-016 asserts its absence)
- [ ] Run `just iter prism-dtu-cyberint` and `just iter prism-dtu-crowdstrike` — tests 15–16 must FAIL

**Phase D — Red Gate density check**
Confirm all 23 Red Gate tests are failing before starting implementation.

**Phase E — Implement InfusionError::TypeCoercionFailed**
- [ ] Add `TypeCoercionFailed` variant to `InfusionError` in `prism-core/src/error.rs` (or equivalent)
- [ ] Mark variant `#[non_exhaustive]` per CLAUDE.md conventions

**Phase F — Implement output_arrow_type() and return_type()**
- [ ] Implement the full D1 mapping in `output_arrow_type()`
- [ ] `return_type()` delegates to `output_arrow_type()`
- [ ] Run test 1: `test_return_type_matches_output_type_for_all_declared_types` — must now PASS

**Phase G — Implement coerce_to_typed()**
- [ ] Implement string→Int64 coercion via `i64::from_str(s.trim())`
- [ ] Implement string→Float64 coercion via `f64::from_str(s.trim())`
- [ ] Implement string→Boolean case-insensitive coercion
- [ ] Implement string→Timestamp via `parse_datetime_to_micros` (REUSE — do not duplicate)
- [ ] Implement JSON-list detection (leading `[`) → NULL + E-INFUSE-014 for typed fields
- [ ] Emit `tracing::warn!(event_type = "infusion.coercion_failed", ...)` on all failure branches
- [ ] Run tests 7–10: coerce_to_typed failures and json-list must PASS

**Phase H — Implement invoke_async_with_args() typed dispatch**
- [ ] Dispatch on `output_arrow_type()` to build Int64Array / Float64Array / BooleanArray / TimestampMicrosecondArray / StringArray
- [ ] Retain ENRICH-1 list-dispatch for `output_type = "json"` fields only
- [ ] Run tests 3–6: typed array construction must PASS

**Phase I — SAP-1 catalog row (SAME COMMIT as tracing emission)**
- [ ] Add `event_type = "infusion.coercion_failed"` row to BC-2.16.002 Canonical Structured Event Catalog
- [ ] This MUST be in the same commit as the `tracing::warn!(event_type = "infusion.coercion_failed", ...)` emission

**Phase J — Implement InfusionLoader::validate sub-conditions 7 and 8**
- [ ] Sub-condition 7: reject unknown `output_type` with E-INFUSE-013 message
- [ ] Sub-condition 8: reject plugin-type field without `source_column` with E-INFUSE-013 message
- [ ] Run tests 11–12: loader validation must PASS

**Phase K — Rewrite threatintel.infusion.toml**
- [ ] Add `source_column = "threat_score"` and `input_field = "iocs_value_first"` to threat_score field
- [ ] Add `source_column = "threat_is_known_malicious"` and `input_field = "iocs_value_first"` to threat_is_known_malicious field
- [ ] Add `source_column = "threat_sources"` and `input_field = "iocs_value_first"` to threat_sources field (DEFECT ADV-P11-OBS-001: iocs_value causes double-encoding; iocs_value_first is REQUIRED)
- [ ] Run test 12: `test_threatintel_toml_has_source_column_and_iocs_value_first_input_field` must PASS

**Phase L — Sensor TOML additions + DTU fixture generators**
- [ ] Add `iocs_value_first` column to `specs/sensors/cyberint.sensor.toml`
- [ ] Add `behaviors_ioc_value_first` column to `specs/sensors/crowdstrike.sensor.toml`
- [ ] Confirm `crates/prism-dtu-cyberint/src/generator.rs` `generate_with_scenario_iocs` emits nested `iocs[].value` array entries; do NOT add a top-level `iocs_value_first` scalar field — spec-driven adapter's `source_path = "$.iocs[0].value"` extracts the column at query time (AC-011)
- [ ] Confirm `crates/prism-dtu-crowdstrike/src/generator.rs` `make_detection_with_ioc` emits nested `behaviors[].ioc_value` entries; do NOT add a top-level `behaviors_ioc_value_first` scalar field (RGT-016 asserts its absence) — spec-driven adapter's `source_path = "$.behaviors[0].ioc_value"` extracts the column at query time (AC-011)
- [ ] SAP-2 check: read actual generator code before committing TOML column declarations
- [ ] Run tests 13–16: all TOML parity and fixture emission tests must PASS

**Phase M — Blast radius sweep (TD-VSDD-060)**
- [ ] Update `prism_describe` / pql_hints output to include the new `_first` columns (BC-2.10.012)
- [ ] Update `crates/prism-mcp/src/resources.rs` enrichment UDF examples (use `iocs_value_first`; show bare `95` not JSON)
- [ ] Update `scripts/t13-preflight-audit.py` (E6 check numeric comparison; E1/E5 use `_first` columns)
- [ ] Update `.factory/objectives/T13-capstone-demo-runbook.md` Act 4 expected output
- [ ] Run TD-VSDD-060 sibling sweep grep commands from ADR-051 §Enforcement:
  ```bash
  rg 'return_type.*Utf8' crates/prism-query/src/infusion_udf.rs
  # Must return ZERO results (output_type.*Utf8 excluded — sanctioned fallback in output_arrow_type() is correct)
  rg 'E-INFUSE-013' crates/ --type rust
  # Must hit the updated validation path covering sub-conditions 7 and 8
  ```

**Phase N — Final gate**
- [ ] `just check` full workspace — all tests pass
- [ ] `just iter prism-dtu-threatintel` — update expected column values in DTU integration tests to typed output (integer/boolean, not JSON strings)
- [ ] Verify `non_exhaustive` gate (CLAUDE.md EXPECTED count unchanged — TypeCoercionFailed is on `InfusionError` which was already `#[non_exhaustive]`; verify gate count is still EXPECTED=89)

---

## Previous Story Intelligence

**From S-DEMO-ENRICHMENT-PIVOT-003 (predecessor in enrichment family, merged PR #196):**

- The canonical ThreatIntel pivot query uses `iocs[].value` (list form). After this story, the query MUST be updated to use `iocs_value_first` (scalar form) for `threat_score` and `threat_is_known_malicious`. The `iocs[].value` form no longer works with typed-output enrichment fields.
- The Cyberint Alert struct uses serde dual-alias for ioc fields (`#[serde(rename = "type", alias = "ioc_type")]`). This story does not change the struct — it only adds `iocs_value_first` emission to the fixture generator.
- CrowdStrike detection records are untyped `serde_json::Value` built by `generator.rs`. The SAP-2 check reads `src/generator.rs` `make_detection()` — NOT `types.rs` (no typed struct). Apply the same approach for `behaviors_ioc_value_first`.
- The PIVOT-003 adversary cascade ran 12 LOCAL rounds and 2 PR-LEVEL rounds. Expect similar depth for this story given multi-crate impact. Budget 3-5 LOCAL rounds minimum.

**From S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 (merged PR #214):**

- `parse_datetime_to_micros` is now available on develop. The function is used in `spec_driven_adapter.rs` `column_type_to_arrow` to convert `ColumnType::Datetime` to `Timestamp(Microsecond, Some("UTC"))`. Implementer: find the function by name (not line number), confirm it is accessible from `infusion_udf.rs` (same crate or pub(crate)), and reuse it.
- The seven-arm temporal dispatch in `check_temporal_literals` handles Timestamp columns correctly. After this story, enrichment `datetime` columns will also be `Timestamp(Microsecond, Some("UTC"))`, so they are handled by the same dispatch automatically.
- The `non_exhaustive` gate on develop is at EXPECTED=89. This story adds `InfusionError::TypeCoercionFailed` — `InfusionError` was already `#[non_exhaustive]` (variant addition to an existing enum does NOT increase the gate count). Confirm EXPECTED=89 is unchanged.

---

## Architecture Compliance Rules

Extracted from ADR-051 v1.4, ADR-052, ADR-040, ADR-024, and CLAUDE.md §Conventions:

1. **INV-ENRICH-TYPED-001 invariant (ADR-051 D6):** All enrichment UDFs must produce typed output per the D1 mapping. No enrichment UDF may return `DataType::Utf8` for a field whose `output_type` is `integer`, `float`, `boolean`, or `datetime`. This is machine-checkable: adversary greps for `DataType::Utf8` in `return_type()` implementations.

2. **Datetime = Timestamp(µs,UTC) (ADR-051 v1.2 / ADR-052):** Enrichment `datetime` output MUST map to `DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))`. NOT `DataType::Utf8`. NOT `DataType::Timestamp(TimeUnit::Microsecond, None)` (missing timezone). The timezone string is `"UTC"`, not `"Etc/UTC"` or anything else.

3. **parse_datetime_to_micros reuse (ADR-052 D2):** The datetime parser MUST be the same function used by `spec_driven_adapter.rs`. Do not implement a new parser.

4. **InfusionError crate boundary (ADR-040 / CLAUDE.md §Conventions):** `InfusionError` lives in `prism-core` (or a shared types crate). The `TypeCoercionFailed` variant must be added to the same enum without breaking the `prism-spec-engine does NOT depend on DataFusion` invariant (BC-2.19.001 §Invariants).

5. **Non-exhaustive enforcement (CLAUDE.md §Conventions):** `InfusionError::TypeCoercionFailed` struct fields use named fields (not tuple). The enum was already `#[non_exhaustive]`. Verify the variant addition does not change the EXPECTED=89 count in `ci.yml`.

6. **No `println!` in production code (CLAUDE.md §Conventions):** All diagnostic output MUST use `tracing::warn!` with structured fields. The E-INFUSE-014 emission is `tracing::warn!(event_type = "infusion.coercion_failed", field_name = %field_name, infusion_id = %infusion_id, declared_type = %declared_type, truncated_value = %truncated_value, ...)`.

7. **TOML spec column declarations require DTU parity (SAP-2):** Every column added to a sensor TOML must have a matching field in the DTU fixture generator for that sensor. The adversary MUST read generator code to verify parity.

8. **Single-workspace MSRV (CLAUDE.md §Conventions):** All code changes build under the single pinned toolchain channel. No per-crate MSRV divergence.

---

## Library & Framework Requirements

| Library | Version | Purpose | Source |
|---------|---------|---------|--------|
| `datafusion` | workspace pin | `DataType`, `Int64Array`, `Float64Array`, `BooleanArray`, `TimestampMicrosecondArray`, `StringArray`, `ColumnarValue`, `ScalarUDF` | Cargo.toml workspace |
| `arrow` | workspace pin (same as DataFusion) | Arrow DataType definitions and array builders | Cargo.toml workspace |
| `tracing` | workspace pin | Structured event logging for E-INFUSE-014 emission | Cargo.toml workspace |
| `serde_json` | workspace pin | JSON Number projection via `as_i64()`, `as_f64()` in coerce_to_typed() | Cargo.toml workspace |
| `chrono` | workspace pin | Used internally by `parse_datetime_to_micros` — do NOT add a new chrono dependency; reuse the function | Existing dependency |

NOTE: Do not pin specific version numbers in this story — always defer to the workspace `Cargo.toml` and `Cargo.lock`. The workspace uses DataFusion 53 and Arrow 53 (verified by ADR-052 implementation). If the workspace Cargo.lock is your source of truth, use it; these numbers may have advanced since the story was written.

---

## File Structure Requirements

**MODIFIED files (existing files requiring changes):**

| File | Change | Story Anchor |
|------|--------|--------------|
| `crates/prism-query/src/infusion_udf.rs` | Add `output_arrow_type()`, modify `return_type()`, add `coerce_to_typed()`, modify `invoke_async_with_args()`, add unit tests 1–10 | AC-001, AC-002, AC-003, AC-004, AC-008, AC-013, AC-014 |
| `crates/prism-spec-engine/src/infusion/loader.rs` (or equivalent validation path) | Add E-INFUSE-013 sub-cond 7 and sub-cond 8 checks to `InfusionLoader::validate` | AC-006, AC-007 |
| `crates/prism-core/src/error.rs` (or `infusion.rs`) | Add `InfusionError::TypeCoercionFailed` variant | AC-005 |
| `specs/infusions/threatintel.infusion.toml` | Add `source_column` to all three fields; change `input_field` to `iocs_value_first` for threat_score and threat_is_known_malicious | AC-009 |
| `specs/sensors/cyberint.sensor.toml` | Add `iocs_value_first: String` column to `cyberint_alerts` table | AC-010 |
| `specs/sensors/crowdstrike.sensor.toml` | Add `behaviors_ioc_value_first: String` column to `crowdstrike_detections` table | AC-010 |
| `crates/prism-dtu-cyberint/src/generator.rs` | Add `generate_with_scenario_iocs` helper to stamp `iocs[0].value` in the nested `iocs[]` array structure; no top-level `iocs_value_first` scalar field emitted — `source_path = "$.iocs[0].value"` at spec-driven adapter layer populates the column (AC-011) | AC-011 |
| `crates/prism-dtu-crowdstrike/src/generator.rs` | Extend `make_detection_with_ioc` to emit nested `behaviors[].ioc_value` structure; no top-level `behaviors_ioc_value_first` scalar field emitted (RGT-016 asserts its absence) — `source_path = "$.behaviors[0].ioc_value"` at spec-driven adapter layer populates the column (AC-011) | AC-011 |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | Add SAP-1 catalog row for `event_type = "infusion.coercion_failed"` | AC-012 |
| `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs` | Add test vectors for typed output (tests 11–14): unknown output_type rejection, threatintel TOML source_column, sensor TOML _first columns | AC-006, AC-007, AC-009, AC-010 |
| `crates/prism-dtu-threatintel/tests/` | Update expected column values from JSON-encoded strings to typed (integer/boolean) output | AC-003 (integration side effect) |
| `crates/prism-mcp/src/resources.rs` | Update enrichment UDF examples in PrismQL reference resource — uses GENERIC `sensor_table` / `src_ip` placeholders per genericization decision (F-PQL2/CRIT-001); no sensor-specific `iocs_value_first` column change applied | Phase M blast radius |
| `scripts/t13-preflight-audit.py` | Update E6 check for numeric comparison; update E1/E5 to use `_first` columns | Phase M blast radius |
| `.factory/objectives/T13-capstone-demo-runbook.md` | Steps 3.2 and 6.2: update expected output to show `threat_score = 95` (bare integer); update queries to use `_first` columns | Phase M blast radius |
| `.github/workflows/ci.yml` | Non-exhaustive gate count annotation — EXPECTED=89 comment updated to confirm that `InfusionError::TypeCoercionFailed` (a variant added to an already-`#[non_exhaustive]` enum) does not increase the gate count | Phase N |
| `crates/prism-spec-engine/src/datetime.rs` | Extract `parse_datetime_to_micros` into a dedicated module for crate-wide reuse per ADR-052 D2 | AC-014 |
| `crates/prism-spec-engine/src/lib.rs` | `pub use datetime::parse_datetime_to_micros` re-export so that `prism-bin` and other consumers can call the helper | AC-014 |
| `crates/prism-bin/src/spec_driven_adapter.rs` | JSONPath `source_path` extraction logic: resolves `$.iocs[0].value` and `$.behaviors[0].ioc_value` from nested DTU array structures to populate the `iocs_value_first` / `behaviors_ioc_value_first` sensor columns | AC-011 |
| `crates/prism-spec-engine/src/pipeline.rs` | `extract_at_path` helper: array-index–based JSON path extraction enabling `$.iocs[0].value` / `$.behaviors[0].ioc_value` resolution inside the spec-engine pipeline | AC-011 |

**NO NEW FILES** are expected from this story. All changes are modifications to existing files.

---

## Edge Cases

| ID | Description | Expected Behavior | BC Anchor |
|----|-------------|-------------------|-----------|
| EC-001 | `output_type = "datetime"` — no current infusion spec uses this; future-proofing only | `return_type()` returns `Timestamp(µs,UTC)`; `invoke_async_with_args()` calls `parse_datetime_to_micros`; behavior correct when such a spec is authored | AC-014; BC-2.19.001 TV-19-001-typed-datetime |
| EC-002 | `Number.as_i64()` returns `None` for a float projected into an integer field (e.g., source returns `95.7` and `output_type = "integer"`) | NULL + E-INFUSE-014 emitted (ADR-051 D2 JSON Number precision mismatch case) | AC-004; Red Gate: `test_ec002_float_string_to_integer_yields_null` (RGT-021) |
| EC-003 | Boolean coercion with mixed-case input (`"True"`, `"YES"`, `"FALSE"`) | Case-insensitive match: `"True"` → true, `"YES"` → true, `"FALSE"` → false | AC-003 |
| EC-004 | Empty string projected into typed field | `i64::from_str("".trim())` fails → NULL + E-INFUSE-014; `f64::from_str("".trim())` fails → NULL | AC-004 |
| EC-005 | `truncated_value` in E-INFUSE-014 is at most 50 chars (no credential in log) | `truncated_value = value.chars().take(50).collect::<String>()` (char-based, genuinely UTF-8-safe; AD-017 guard) | AC-004 |
| EC-006 | `iocs_value` array is empty (e.g., cyberint alert with no IOCs) | `iocs_value_first = ""` (empty string); enrichment UDF receives `""` as input; `i64::from_str("".trim())` fails → NULL + E-INFUSE-014 (benign: no IOC to enrich) | AC-011; Red Gate: `test_ec006_empty_input_yields_null` (RGT-022) |
| EC-007 | json-typed `threat_sources` field with scalar `iocs_value_first` input (NOT list input) | ENRICH-1 does NOT fire (scalar input, not `[`-prefixed); `project_value` extracts `threat_sources` Array via `other.to_string()` → JSON string `["greynoise","abuseipdb"]`; stored directly as Utf8 in `StringArray`. Double-encoding is prevented by using scalar input (ADV-P11-OBS-001 DEFECT closure; see RGT-023). | AC-008, AC-009 |
| EC-008 | Pre-existing `crates/prism-dtu-threatintel/tests/` integration tests assert JSON-encoded output | Tests must be updated in this story to assert bare typed output (integer 95, boolean true) | Phase N tasks |

---

## Implementation Notes

### Reuse parse_datetime_to_micros (no new parser)

`parse_datetime_to_micros` is the shared datetime parser introduced by ADR-052 (PR #214).
It is used in `spec_driven_adapter.rs` `column_type_to_arrow`
for the `ColumnType::Datetime` → `Timestamp(Microsecond, Some("UTC"))` mapping.

The implementer MUST find this function by name (not file path — TD-VSDD-091), confirm it
is accessible from the `infusion_udf.rs` module (it may be `pub(crate)` or need to be
made accessible), and call it from `coerce_to_typed()` for the `datetime` branch. Do NOT
implement a new ISO-8601 / RFC-3339 parser.

### BC-2.19.001 amendment already on develop (POL-14 notes)

BC-2.19.001 v2.2 was amended by the product-owner as part of the ADR-051 ratification burst
(2026-07-05). It is already on develop (`factory-artifacts` branch). The story does NOT need
to amend BC-2.19.001 further — it traces TO the v2.2 postconditions. The implementer writes
code to satisfy those postconditions; the product-owner wrote the postconditions.

Per POL-14: when this story's PR merges, BC-2.19.001 and BC-2.16.002 are already `active`
(BC-2.19.001 lifecycle_status was `active` before this story; BC-2.16.002 is `active`). No
lifecycle transition needed at merge time.

### SAP-1 catalog row: same-commit obligation

The `tracing::warn!(event_type = "infusion.coercion_failed", ...)` emission in
`infusion_udf.rs` and the BC-2.16.002 Canonical Structured Event Catalog row for
`event_type = "infusion.coercion_failed"` MUST be in the same atomic commit. The adversary
SAP-1 probe greps for `event_type.*infusion.coercion_failed` across `crates/` — every hit
must have a BC-2.16.002 catalog row. If the emission lands in one commit and the catalog
row in a subsequent commit, the adversary will find a P1.

### `_first` companion columns are String, not typed

`iocs_value_first` and `behaviors_ioc_value_first` are declared as `column_type = "String"`
in the sensor TOMLs. They carry the raw string extracted from the first array element. The
typed coercion happens inside the enrichment UDF when it processes the scalar input — not
at the sensor schema level. This is consistent with `device_cves_first: String` precedent
from S-DEMO-ENRICHMENT-PIVOT-003 (NVD pattern).

### ENRICH-1 list-dispatch path decision (CLOSED — DEFECT ADV-P11-OBS-001)

ADR-051 §Required Spec Changes recommends switching `threat_sources` to `input_field = "iocs_value_first"`.
This decision is now REQUIRED, not optional. ADV-P11-OBS-001 (adjudicated 2026-07-06) proved that
retaining `input_field = "iocs_value"` causes double-encoding: the ENRICH-1 list-dispatch path fires,
`project_value` returns a JSON-serialized array string, and `serde_json::to_string` wraps it into
`["[\"greynoise\",\"abuseipdb\"]"]` — a JSON string containing an escaped JSON array, not a proper
JSON array of strings. The implementer MUST use `input_field = "iocs_value_first"` for `threat_sources`.
"Either approach is valid" and "adversary will accept ENRICH-1 retention" are INCORRECT. See
risk_mitigations for the full runtime chain. RGT-023 enforces this at the test level.

---

## Forbidden Dependencies

- `crates/prism-spec-engine` MUST NOT depend on `crates/prism-query` (existing architectural constraint per BC-2.19.001 §Invariants — "prism-spec-engine does NOT depend on DataFusion"). The dependency is one-way: `prism-spec-engine` exports `InfusionUdfDescriptor`; `prism-query` imports and registers it. If the implementer introduces a reverse dependency to share `coerce_to_typed()`, refactor: move the coercion helper to `prism-core` (shared types crate) instead.
- `crates/prism-query/src/infusion_udf.rs` MUST NOT import `chrono` directly. Use `parse_datetime_to_micros` which encapsulates the chrono dependency.
- No `unwrap()` or `expect()` on `Result` in the coerce_to_typed() production path — use `?` or explicit NULL production.

---

## References

| Document | Relevance |
|----------|-----------|
| ADR-051 v1.4 | Closes DRIFT-PIVOT-UDF-OUTPUT-TYPE-001; D1 type mapping; D2 coercion semantics + E-INFUSE-014; D3 mandatory source_column + E-INFUSE-013 sub-conditions 7/8; D4 scalar-input rule; D5 PrismQL comparison semantics; D6 INV-ENRICH-TYPED-001 |
| ADR-052 (merged PR #214) | parse_datetime_to_micros; Timestamp(µs,UTC) mapping for ColumnType::Datetime; consistency argument for enrichment datetime |
| ADR-040 v2.0 | Dual-path infusion architecture (HttpLookup NVD + WASM ThreatIntel); no changes to the architecture in this story |
| ADR-024 | prism_core::column::ColumnType six-type vocabulary; alignment with infusion output_type vocabulary |
| BC-2.19.001 v2.2 | Primary behavioral contract: INV-ENRICH-TYPED-001, Plugin-type field projection, E-INFUSE-013 sub-conds 7/8, E-INFUSE-014 |
| BC-2.16.002 v1.96 | SAP-1 catalog row obligation for infusion.coercion_failed |
| error-taxonomy v2.16 | E-INFUSE-013 sub-conditions 7/8 added; E-INFUSE-014 TypeCoercionFailed allocated |
| DRIFT-PIVOT-UDF-OUTPUT-TYPE-001 | Root defect this story closes: return_type() hardcoded Utf8; missing source_column on ThreatIntel |
| T13 audit OBS-1 | Original defect documentation: doubly-encoded JSON + lexicographic CVSS comparison bugs |
| S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 (merged) | parse_datetime_to_micros availability on develop; Timestamp(µs,UTC) as canonical datetime type |
| CLAUDE.md §SAP-1 | SAP-1 standing probe: tracing emission catalog completeness |
| CLAUDE.md §SAP-2 | SAP-2 standing probe: sensor TOML↔DTU schema parity |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.9 | 2026-07-06 | story-writer | ADV-P22-LOW-001 closure + BC-2.16.002 v1.95→v1.96 pin propagation (POL-23) + final count/pin sweep. **(LOW-001)** Points justification item 12 corrected: "16 tests" → "23 tests" (matches `red_gate_tests: 23` frontmatter and 23-row Red Gate Test Plan table; "across 4 crates" verified accurate: prism-query, prism-spec-engine, prism-dtu-cyberint, prism-dtu-crowdstrike). **(POL-23)** BC-2.16.002 version pin propagated v1.95→v1.96 at all four non-historical sites: frontmatter comment, Behavioral Contracts table, AC-012 trace citation, References table. Changelog v1.2 `v1.93→v1.95` historical entry left intact. **(Sweep — SHA volatile pins)** Five SHA volatile pins removed from non-exempted narrative prose per TD-VSDD-091: `develop@f6739764` (Previous Story Intelligence §PIVOT-003), `develop@11edbd36` (§TEMPORAL-TYPING-001 heading), `develop@11edbd36` (Library table datafusion cell), `develop@11edbd36` (Library NOTE paragraph), `develop@11edbd36` (Implementation Notes §Reuse parse_datetime_to_micros). Frontmatter comment SHA (`commit 11edbd36`) left intact — frontmatter comments are not narrative prose. **(Sweep — counts)** All other counts verified: `red_gate_tests: 23` ✓, Phase A "17 Red Gate tests" ✓, Phase D "23 Red Gate tests" ✓, test crate placement Note "Tests 1–10 and 17–23" ✓. No ACs added or removed; no BC-semantics, code, or test changes. |
| 1.8 | 2026-07-06 | story-writer | ADV-P19-MED-002 + ADV-P18-OBS-001 closure (prose-accuracy only; no code/AC/BC changes). **(MED-002) Stale abandoned-approach prose removed**: File Structure Requirements generator rows, Phase C tasks, and Phase L tasks previously described the abandoned top-level-scalar approach (DTU generators emitting pre-computed `iocs_value_first` / `behaviors_ioc_value_first` scalar fields). All three sections rewritten to describe the actual implemented approach: JSONPath `source_path` resolution at the spec-driven adapter layer (`$.iocs[0].value`, `$.behaviors[0].ioc_value`) against nested array structures already emitted by the DTU generators — consistent with AC-011 v1.1, RGT-015, and RGT-016 (which explicitly asserts the top-level scalar is ABSENT). **(OBS-001) File Structure completeness**: five files present in the `d098be6f..d51c508a` diff added to MODIFIED table: `.github/workflows/ci.yml` (non-exhaustive gate note), `crates/prism-spec-engine/src/datetime.rs` (parse_datetime_to_micros extraction per ADR-052 D2), `crates/prism-spec-engine/src/lib.rs` (pub use re-export), `crates/prism-bin/src/spec_driven_adapter.rs` (JSONPath extraction for AC-011), `crates/prism-spec-engine/src/pipeline.rs` (extract_at_path index support for AC-011). **(Belt-and-suspenders)** `depends_on` comment stale "adds the _first scalar columns to the DTU fixture generators" and "scalar _first projections" language replaced with accurate description: PIVOT-003's nested array structures are the JSONPath resolution targets, not a scalar-emission substrate. |
| 1.7 | 2026-07-06 | story-writer | ADV-P15-LOW-001 closure + comprehensive prose-accuracy audit. (1) **Forbidden Dependencies**: opening clause corrected from "`crates/prism-query` MUST NOT depend on `crates/prism-spec-engine`" to "`crates/prism-spec-engine` MUST NOT depend on `crates/prism-query`" — the original was a self-contradicting inversion of the true invariant; confirmed against Cargo.toml: `prism-query/Cargo.toml` declares `prism-spec-engine` as a dependency (correct direction); `prism-spec-engine/Cargo.toml` carries an explicit "MUST NOT depend on datafusion" comment and no `prism-query` dep. (2) **Phase A Red Gate count**: "all 16 Red Gate tests (those in prism-query)" corrected to "all 17 Red Gate tests (those in prism-query: tests 1–10 and 17–23)" — count 16 was stale from v1.0 origin; actual prism-query subset is tests 1–10, 17–20, 21–22, 23 = 17 tests. (3) **Phase D Red Gate count**: "all 16 Red Gate tests" corrected to "all 23 Red Gate tests" — matches `red_gate_tests: 23` frontmatter. (4) **Test crate placement Note**: "Tests 1–10 and 17–20" corrected to "Tests 1–10 and 17–23" — tests 21–22 (added v1.3) and test 23 (added v1.6) were omitted from the Note when they were added. (5) **AC-006 E-INFUSE-013 sub-condition 8 error message**: `{field_name}` template slot corrected to the literal `source_column` per error-taxonomy v2.16 sub-condition 8 canonical form — `{field}` = the attribute name whose absence triggered the error (`"source_column"`), consistent with sub-condition 7 precedent where `{field}` = the literal string `"output_type"` (v2.16 taxonomy text, v1.6 story MED-002 closure). No ACs added or removed; no BC changes; no code changes. |
| 1.6 | 2026-07-06 | product-owner | ADV-P11-OBS-001 adjudication: DEFECT verdict. Runtime trace proves double-encoding when `threat_sources` uses `input_field = "iocs_value"` (JSON-list): ENRICH-1 fires, `project_value` returns JSON-serialized array string, `serde_json::to_string` double-wraps to `["[\"greynoise\",\"abuseipdb\"]"]`. Fix: `input_field = "iocs_value_first"` (scalar) so ENRICH-1 does NOT fire and output is `["greynoise","abuseipdb"]`. risk_mitigations "Either approach is valid" entry replaced with DEFECT notice + exact runtime chain. AC-009 `threat_sources` row updated to require `input_field = "iocs_value_first"`. AC-009 T13 canonical query corrected to per-field UDF syntax. RGT-023 `test_threat_sources_json_output_no_double_encoding` added. red_gate_tests 22→23. |
| 1.5 | 2026-07-06 | story-writer | Comprehensive prose-accuracy audit against code HEAD a3083468 (ADV-P08 closure). (EC-005) `truncated_value` truncation corrected: byte-slice description `&value[..50.min(value.len())]` replaced with char-based implementation `value.chars().take(50).collect::<String>()` (genuinely UTF-8-safe; "exactly 50 chars" → "at most 50 chars"). (AC-004) Point 2 updated: added explicit note that `declared_type` uses the `output_type` spec-vocabulary string (e.g., `"integer"`) — NOT Arrow debug format (`Int64`); truncation implementation cited for completeness. (ADR-051 cite-pin) Body intro and §References updated v1.3→v1.4 (canonical current version; v1.4 = post-pass-1 column_type example PascalCase→lowercase fix; no D1–D6 decision-content change). red_gate_tests UNCHANGED 22. No ACs added or removed; no BC changes; no code changes. |
| 1.4 | 2026-07-06 | story-writer | Spec-only fix: ADV-P05-LOW-001 closure (POL-25 sibling-cite propagation). §References error-taxonomy pin v2.15→v2.16 (canonical version; E-INFUSE-013 sub-cond 7/8 + E-INFUSE-014 present in v2.16). v1.0 changelog entry error-taxonomy pin updated to match (sweep complete — 0 remaining v2.15 citations). red_gate_tests UNCHANGED 22. No ACs added or removed; no BC changes; no code changes. |
| 1.3 | 2026-07-06 | story-writer | Spec reconciliation to code HEAD ce93229a (LOCAL adversary pass-3 OBS-003 closure + test-plan drift). (OBS-003) File Structure resources.rs row corrected: blast-radius task uses GENERIC `sensor_table`/`src_ip` placeholders per genericization decision (F-PQL2/CRIT-001) — no sensor-specific `iocs_value_first` column change; annotation updated accordingly. (fix-burst-3 test additions) RGT-021 added: `test_ec002_float_string_to_integer_yields_null` (`coerce_to_typed("95.7", Int64)` → None, EC-002); RGT-022 added: `test_ec006_empty_input_yields_null` (`coerce_to_typed("", Int64)` → None, EC-006); both in prism-query infusion_udf.rs; red_gate_tests 20→22. (fix-burst-3 test removal) `test_cyberint_dtu_fixture_emits_iocs_value_first_field` removed from code (asserted now-removed speculative top-level scalar field); cyberint AC-011 coverage retained via `test_ac011_cyberint_alerts_iocs_value_first_column_via_jsonpath` (already RGT-015 since v1.1 — no table removal needed). EC-002 and EC-006 edge-case rows updated to reference their Red Gate tests (RGT-021, RGT-022). No ACs added or removed; no BC changes; no code changes. |
| 1.2 | 2026-07-06 | story-writer | Spec reconciliation to code HEAD 4699551e (LOCAL adversary pass-2 LOW-002 closure). (LOW-002) All BC-2.16.002 version pins updated v1.93→v1.95 (canonical version on factory-artifacts). (fix-burst-2 test-name reconciliation) RGT-003..006 (invoke_async typed-array tests) now note `.value(0)` value assertions (42, 3.14, true, micros) added in fix-burst-2. (fix-burst-2 test-name reconciliation) RGT-015 (cyberint AC-011 test) updated: lives in prism-dtu-cyberint/src/generator.rs, uses `generate_with_scenario_iocs`. RGT-016 (crowdstrike AC-011 test) updated: lives in prism-dtu-crowdstrike/src/generator.rs, asserts `$.behaviors[0].ioc_value` + asserts top-level `behaviors_ioc_value_first` ABSENT. (fix-burst-2 new tests) 4 new positive-value coerce_to_typed tests added as RGT-017..020: test_coerce_to_typed_integer_valid_returns_some_number, test_coerce_to_typed_float_valid_returns_some_number, test_coerce_to_typed_boolean_valid_variants_return_some_bool, test_coerce_to_typed_datetime_valid_returns_some_micros; red_gate_tests 16→20. No ACs added or removed; no BC changes; no code changes. |
| 1.1 | 2026-07-06 | story-writer | Spec reconciliation to code HEAD 89a09782 (LOCAL adversary pass-1 closures). (HIGH-001) AC-011 clarified: iocs_value_first/behaviors_ioc_value_first populated by spec-driven adapter via JSONPath source_path extraction ($.iocs[0].value, $.behaviors[0].ioc_value) from nested DTU array structures — NOT pre-computed top-level scalar fields; RGT-015/016 test names reconciled to test_ac011_cyberint_alerts_iocs_value_first_column_via_jsonpath / test_ac011_crowdstrike_detections_behaviors_ioc_value_first_column_via_jsonpath. (LOW-003) AC-002 TD-VSDD-060 grep narrowed from 'output_type.*Utf8\|return_type.*Utf8' to 'return_type.*Utf8' (output_type.*Utf8 excluded — sanctioned _ => DataType::Utf8 fallback in output_arrow_type() and legitimate *output_type != DataType::Utf8 guard comparisons are correct behavior). (process-gap) AC-010 column_type examples corrected PascalCase "String" → lowercase "string" (prism-core/src/column.rs #[serde(rename_all = "snake_case")] canonical form). No ACs added or removed; no BC changes; no code changes. |
| 1.0 | 2026-07-05 | story-writer | Initial decomposition. ADR-051 ACCEPTED v1.3 (2026-07-05); BC-2.19.001 v2.2 (amended 2026-07-05); error-taxonomy v2.16. 14 ACs; 16 Red Gate tests; 13 pts; Wave 5; E-DEMO; depends_on S-DEMO-ENRICHMENT-PIVOT-003. |
