---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: 2026-08-11
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

# BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec

## Description

After a spec-driven table's multi-step fetch pipeline returns raw records, columns with
`ocsf_field` mappings are translated to the corresponding OCSF protobuf fields using
the standard four-tier resolution from BC-2.02.008. Columns without mappings are
preserved in the `raw_extensions` JSON blob per BC-2.02.007. Type coercion is applied
for mismatched types with non-fatal fallback to `raw_extensions` on failure.

The coercion rule follows a **String-type-first** precedence: when the TOML spec
declares `column_type = "string"`, any scalar JSON value (Number, Bool) from the API
is normalized to a JSON string before the OCSF-path numeric-suffix heuristic fires.
This prevents integer API IDs from landing in string columns as un-coerced numbers, and
prevents string usernames mapped to `*.uid`-style OCSF paths from being incorrectly
demoted to `raw_extensions` by the numeric heuristic (LIVE-DRIFT-003, human-authorized
gap closure 2026-08-11 per CLAUDE.md §Source-of-Truth Precedence item 7; this was a
genuine absence of specification, not a code-vs-spec conflict).

The resulting `OcsfEvent` is uniform across all sensors: downstream consumers
(detection rules, cross-sensor correlation, decorators) cannot distinguish spec-driven
data from built-in adapter data. Invalid OCSF field paths produce a warning at spec
load time (not a hard error) because OCSF schema extensions may introduce fields not
in the compiled schema.

## Preconditions
- A spec-driven table has been fetched via the multi-step pipeline (BC-2.16.002) and raw records are available
- The table's `ColumnSpec` entries include `ocsf_field` mappings (some columns may have `ocsf_field: None`, meaning no OCSF mapping)
- The OCSF normalizer (CAP-003) is available
- `ColumnSpec.column_type` is one of `String | Integer | Float | Boolean | Datetime | Json` (prism_core::column::ColumnType variants per ADR-024)

## Postconditions

### Column Routing

For each record fetched from the spec-driven sensor:
- Columns with an `ocsf_field` value are mapped to the corresponding OCSF field in the DynamicMessage protobuf representation
- The mapping follows the standard four-tier field resolution (BC-2.02.008): Prism metadata fields, proto descriptor fields, unmapped JSON blob, None
- Columns without an `ocsf_field` mapping are preserved in the `raw_extensions` JSON blob (consistent with BC-2.02.007)
- The `ocsf_class` declared at the table level determines which OCSF event class the DynamicMessage uses (e.g., `security_finding`, `device_inventory`, `network_activity`)

### Type Coercion Algorithm

`ColumnMapper::coerce_value` applies the following precedence:

**Rule 1 — String-type-first (LIVE-DRIFT-003):**
When `column_type = "string"`, any scalar JSON value is normalized to a JSON string value
before the OCSF path heuristic is consulted:

| Input JSON type | Wire output | Notes |
|-----------------|-------------|-------|
| String          | String (unchanged) | No transformation |
| Number          | String(n.to_string()) | e.g. `132` → `"132"` |
| Bool            | String(b.to_string()) | `true` → `"true"`, `false` → `"false"` |
| Null            | Null (pass-through) | See EC-016-013-006 |
| Array           | Array (pass-through) | KNOWN GAP — see EC-016-013-007 |
| Object          | Object (pass-through) | KNOWN GAP — see EC-016-013-008 |

Rule 1 applies regardless of the OCSF field path's numeric suffix. This correctly
overrides the `is_numeric_ocsf_field` heuristic for string-declared columns, because
the spec author's declared `column_type` is authoritative over a path-name pattern.

**Rule 2 — OCSF numeric-path heuristic:**
For non-`String` columns only, when the OCSF field path's last segment is one of
`event_code`, `class_uid`, `activity_id`, `type_uid`, `severity_id`, `status_id`,
`action_id`, `count`, `duration`, `port`, `pid`, `uid`, `code`:
- If the incoming value is `Value::String`, attempt `i64` parse
- On parse success: return `Value::Number(n)` — string-encoded integer coerced
- On parse failure: return `Err(CoercionWarning)` — demotion to `raw_extensions`

NOTE: the `uid` suffix correctly coerces `class_uid`/`type_uid` (OCSF integer enum
codes). It would incorrectly trigger for `actor.user.uid` (a string identifier) unless
the spec declares `column_type = "string"` — in which case Rule 1 preempts it.
The canonical `actor.user.uid` usage pattern requires `column_type = "string"` in the
TOML spec.

**Rule 3 — Pass-through default:**
All other cases: return the value unchanged. No coercion error.

### Full Coercion Matrix

| declared `column_type` | Input JSON type | OCSF path | Rule | Wire output | Outcome |
|------------------------|-----------------|-----------|------|-------------|---------|
| String | String | any | Rule 1 | String (unchanged) | OCSF field |
| String | Number | any | Rule 1 | String(n.to_string()) | OCSF field |
| String | Bool | any | Rule 1 | String(b.to_string()) | OCSF field |
| String | Null | any | Rule 1 | Null | OCSF field |
| String | Array | any | Rule 1 gap | Array (WRONG) | OCSF field — KNOWN GAP |
| String | Object | any | Rule 1 gap | Object (WRONG) | OCSF field — KNOWN GAP |
| Integer | Number | any | Rule 3 | Number (unchanged) | OCSF field |
| Integer | String | numeric suffix | Rule 2 | Number(parse) or Err | OCSF field or raw_extensions |
| Integer | String | non-numeric suffix | Rule 3 gap | String (WRONG) | OCSF field — KNOWN GAP |
| Integer | Bool/Null/Array/Object | any | Rule 3 | Unchanged | OCSF field (no coercion) |
| Float | (any) | any | Rule 3 | Unchanged | OCSF field (no float coercion in v1) |
| Boolean | (any) | any | Rule 3 | Unchanged | OCSF field (no bool coercion in v1) |
| Datetime | (any) | any | Rule 3 | Unchanged | OCSF field; datetime parsing is downstream |
| Json | (any) | any | Rule 3 | Unchanged | OCSF field (any JSON is valid) |

**KNOWN GAPs (require a fix story — see §Traceability):**
- EC-016-013-007: `column_type = "string"` + Array input: currently passes array to OCSF field; MUST divert to raw_extensions with CoercionWarning
- EC-016-013-008: `column_type = "string"` + Object input: same defect class as EC-016-013-007
- EC-016-013-009: `column_type = "integer"` + String input on non-numeric OCSF path: currently passes string to OCSF field; MUST parse and divert on failure

### Coercion Warning Observability

`ColumnMapper::coerce_value` returns `Err(CoercionWarning)` on failure; the caller
(`ColumnMapper::map_record`) places the value in `raw_extensions` and records the
warning in `MappingResult.coercion_warnings`.

**DEFECT — missing `tracing::warn!`:** The current implementation does NOT emit a
`tracing::warn!` at the point of demotion. This violates BC-2.02.011 §Postconditions
("A warning-level log entry is emitted for each normalization issue") and the
§Error Conditions table below. Per BC-5.39.001, this defect routes to the implementer
for fix in the next cascade. Until fixed, `CoercionWarning` is only observable via the
returned `MappingResult.coercion_warnings` vec — it is NOT surfaced to operators or
the audit trail. See §Traceability for story anchor status.

The required emission at demotion time is:
```
tracing::warn!(
    column = %warning.column_name,
    expected_ocsf_type = %warning.expected_ocsf_type,
    actual_value = %warning.actual_value,
    event_type = "column_coercion_failure",
    "coerce_value: type mismatch; field diverted to raw_extensions"
);
```
This `event_type` value MUST be registered in BC-2.16.002 §Postconditions Canonical
Structured Event Catalog per PG-LP11-001.

## OCSF Field Validation
- At spec load time (BC-2.16.009), each `ocsf_field` value is validated against the compiled OCSF protobuf schema
- Invalid OCSF field paths produce a warning at load time but do not reject the spec (the mapping is skipped at runtime, and the column goes to `raw_extensions`)
- This is a warning, not an error, because OCSF schema extensions may introduce fields not in the compiled schema

## Invariants
- Coercion failures are non-fatal: the field value is preserved in `raw_extensions` (record is NEVER dropped due to type mismatch)
- The `ocsf_class` at table level determines the OCSF event class for all records in that table
- Spec-driven OcsfEvents are indistinguishable from built-in adapter OcsfEvents to downstream consumers
- The declared `column_type` in the TOML spec is the authoritative wire shape for the column; the OCSF path name heuristic is a secondary fallback only for non-String columns
- NULL vs absent: a column absent from the raw record is SKIPPED (not placed in either `mapped_fields` or `raw_extensions`); a column present with `Value::Null` is placed in its destination (either OCSF field or `raw_extensions`) as a JSON null value

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | Coercion failure for a column value | Field diverted to `raw_extensions`; `CoercionWarning` created; MUST emit `tracing::warn!(event_type = "column_coercion_failure")` per BC-2.02.011 — DEFECT: not yet emitted |
| Warning (non-fatal) | Invalid `ocsf_class` in table spec | All records use generic `base_event` class (OCSF class 0) with startup warning |
| KNOWN GAP | `column_type = "string"` + Array/Object input | Currently passes structured value to OCSF field instead of diverting to raw_extensions (EC-016-013-007, EC-016-013-008) |
| KNOWN GAP | `column_type = "integer"` + String input on non-numeric OCSF path | Currently passes string to OCSF field instead of coercing/diverting (EC-016-013-009) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-013-001 | Column with `ocsf_field: None` | Column value placed in `raw_extensions`; no coercion attempted |
| EC-016-013-002 | `column_type = "integer"`, string `"42"` on numeric-suffix OCSF path | Parsed as integer 42; wire output is `Value::Number(42)`; placed in OCSF field |
| EC-016-013-003 | `column_type = "integer"`, string `"not-a-number"` on numeric-suffix OCSF path | Parse fails; `Err(CoercionWarning)` returned; field diverted to `raw_extensions`; record included |
| EC-016-013-004 | `column_type = "string"`, API returns `Value::Number(132)`, OCSF path `"finding.uid"` (uid suffix) | Rule 1 fires before heuristic; wire output is `Value::String("132")`; placed in OCSF field. Concrete exemplar: Claroty `alerts.id` (LIVE-DRIFT-003) |
| EC-016-013-005 | `column_type = "string"`, string username `"analyst"`, OCSF path `"actor.user.uid"` (uid suffix) | Rule 1 fires; `is_numeric_ocsf_field` NOT consulted; string preserved; placed in OCSF field without CoercionWarning. Concrete exemplar: Claroty audit_log `username` column (LIVE-DRIFT-003) |
| EC-016-013-006 | `column_type = "string"`, `Value::Null` input | Rule 1 pass-through; `Value::Null` placed in OCSF field; absent key is DISTINCT from null (see wire-shape invariant in §Invariants) |
| EC-016-013-007 | `column_type = "string"`, `Value::Array([...])` input | KNOWN GAP: Rule 1 pass-through; array placed in OCSF string field (incorrect — MUST divert to raw_extensions). Awaiting fix story (no story ID exists; see §Traceability) |
| EC-016-013-008 | `column_type = "string"`, `Value::Object({...})` input | KNOWN GAP: same defect class as EC-016-013-007 |
| EC-016-013-009 | `column_type = "integer"`, `Value::String("42")`, OCSF path `"device.bytes"` (non-numeric suffix) | KNOWN GAP: Rule 3 pass-through; string placed in integer-typed OCSF field (incorrect — MUST parse and divert on failure). Awaiting fix story (no story ID exists; see §Traceability) |
| EC-016-013-010 | `column_type = "boolean"`, `Value::Bool(true)` | Rule 3 pass-through; bool placed in OCSF field unchanged |
| EC-016-013-011 | Invalid `ocsf_class` (e.g., `"made_up_class"`) | Records use base_event (class 0); startup warning at spec load time |
| EC-016-013-012 | Two sensors both map `device_ip` → `ocsf_field = "device.ip"` | Both queryable as `device.ip`; cross-sensor JOIN works transparently |

## Canonical Test Vectors

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — full mapping | all columns have `ocsf_field`; all types match | OcsfEvent with all fields mapped; `raw_extensions` empty |
| Mixed mapping | some columns have `ocsf_field`, some don't | Mapped columns in OCSF proto; unmapped in `raw_extensions` |
| Coercion failure — non-parseable string | `"not-a-number"` for integer field on numeric-suffix path | Field in `raw_extensions`; `CoercionWarning` emitted; record included |
| Integer JSON on String column | `Value::Number(132)` on `finding.uid`, `column_type = "string"` | Wire output `Value::String("132")`; test: `test_coerce_value_string_type_normalizes_integer_to_string` |
| String username on uid path | `Value::String("analyst")` on `actor.user.uid`, `column_type = "string"` | String preserved in OCSF field; no CoercionWarning; test: `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` |
| Invalid ocsf_class | table has unknown `ocsf_class` | base_event class used; warning at load |

See `.factory/specs/prd-supplements/test-vectors.md` for extended canonical vector tables.

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-017 | OCSF normalization: unmapped fields preserved in raw_extensions (proptest) — coercion failures fall into the same preservation guarantee |
| VP-016 | OCSF normalization: output is valid protobuf — coercion failures do not produce malformed protobufs (record still encodes; field merely moves to raw_extensions) |

## Related BCs

- BC-2.02.007 (composes with): governs the raw_extensions blob that coercion failures land in
- BC-2.02.008 (depends on): four-tier field resolution used for OCSF field placement
- BC-2.02.011 (depends on): defines the warning-emission obligation for each coercion failure
- BC-2.16.002 (depends on): multi-step fetch pipeline whose output records are consumed here

## Architecture Anchors

- `crates/prism-spec-engine/src/column_mapping.rs` — `ColumnMapper::coerce_value`, `ColumnMapper::map_record`, `is_numeric_ocsf_field`
- `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` — integration tests for column routing and coercion
- `crates/prism-spec-engine/src/column_mapping.rs` `CoercionWarning` struct — returned data structure (not yet observed at wire level; SAP-1 violation until `event_type = "column_coercion_failure"` is added to BC-2.16.002 catalog)

## Story Anchor

No story currently exists that implements the coercion semantics contracted here. The
two tests cited (`test_coerce_value_string_type_normalizes_integer_to_string` and
`test_coerce_value_string_type_preserves_string_username_against_uid_heuristic`) were
added under branch `fix/claroty-live-api-fidelity` (commit `3e9825288`) without a
governing story, as part of an emergency live-API fidelity push. Story-writer must
create a story to:

1. Add `tracing::warn!(event_type = "column_coercion_failure")` in `ColumnMapper::map_record` at demotion time
2. Register `column_coercion_failure` in BC-2.16.002 §Postconditions Canonical Structured Event Catalog
3. Fix EC-016-013-007 and EC-016-013-008 (structured-type demotion)
4. Fix EC-016-013-009 (Integer column, non-numeric OCSF path, string input)

Until that story is created and delivered, the three KNOWN GAPs and the missing-log
defect remain open obligations under this contract. Do NOT invent a story ID here —
per CLAUDE.md §Canonical Principle rule 3, the orchestrator must direct the deferral
with a real future story anchor.

## VP Anchors

No VPs directly verify the coercion matrix at the property level. VP-017 (proptest,
raw_extensions preservation) covers the coercion-failure demotion path indirectly.
A dedicated VP for the coercion matrix (exhaustive column_type × JSON-type combinatorics
via proptest) is recommended as part of the fix story above.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — the column-to-OCSF mapping is explicitly named as a spec-engine capability: "tables with columns (typed, with ColumnOptions and OCSF mappings)" and "Column OCSF mappings are validated against the compiled protobuf schema (warnings for invalid paths, not errors)" |
| L2 Invariants | DI-005 (no vendor data silently dropped) |
| Related BCs | BC-2.02.007 (raw_extensions preservation), BC-2.02.008 (four-tier field resolution), BC-2.02.011 (normalization error handling) |
| Priority | P0 |
| Known-Gap Story Needed | YES — story-writer must create a story for: (1) CoercionWarning tracing emission, (2) EC-016-013-007/008 structured-type demotion fix, (3) EC-016-013-009 integer-column string-input coercion fix. No ID exists yet; do not fabricate one. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | coercion-gap-closure | 2026-08-11 | product-owner | Human-authorized gap closure (CLAUDE.md §Source-of-Truth item 7): expanded coercion matrix section (String-type-first rule LIVE-DRIFT-003), full column_type × JSON-type matrix, EC-016-013-001..012 edge case catalog with IDs, CoercionWarning observability defect flag, KNOWN GAP annotations for structured-type and integer-column gaps, capability anchor justification, Related BCs, Architecture Anchors, Story Anchor, VP Anchors sections added. Two implementing tests (`test_coerce_value_string_type_normalizes_integer_to_string`, `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic`) cited as evidence for EC-016-013-004 and EC-016-013-005. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
