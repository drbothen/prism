---
document_type: story
story_id: S-ADR058-OCSF-COERCION-001
title: "ADR-058 Stage 1 — Column Coercion Gap Closure: EC-016-013-007/008/009 Fixes and column_coercion_failure Tracing Emission"
version: "1.46"
level: "L4"
status: draft
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
modified: "2026-08-20"
phase: 3
wave: claroty-live
epic_id: EPIC-OCSF-ROUTING
priority: P1
points: 5
tdd_mode: strict
target_module: prism-spec-engine
subsystems:
  - SS-01
  - SS-10
  - SS-16
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because `prism-spec-engine` is listed
#     under SS-01 per ARCH-INDEX (SS-01 row: "prism-sensors, prism-spec-engine, prism-dtu-*").
#     `prism-spec-engine::column_mapping` (`ColumnMapper::coerce_value`, `ColumnMapper::map_record`)
#     is the implementation site for EC-016-013-007/008/009 coercion fixes. NOTE: `prism-bin`
#     is NOT listed under SS-01 — see SS-10 below.
#   SS-10 (MCP Interface) owns this story's scope because `prism-bin` is listed under SS-10
#     per ARCH-INDEX (SS-10 row: "prism-mcp, prism-bin (planned — S-WAVE5-PREP-01)").
#     `prism-bin::spec_driven_adapter` (`build_column_array` ColumnType::String arm fix +
#     `column_coercion_failure` warn emission) is in prism-bin. SS-22 (Process Lifecycle)
#     is excluded: its scope is ADR-022 §B boot orchestration only, not sensor data processing.
#   SS-16 (Spec Engine) owns this story's scope because SS-16 is the canonical owner of
#     prism-spec-engine per ARCH-INDEX (SS-16 row: "prism-spec-engine"). BC-2.16.003 (the
#     governing behavioral contract) is also assigned to SS-16.
crates_touched:
  - prism-spec-engine
  - prism-bin
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.003
  - BC-2.02.011
  - BC-2.16.002
verification_properties:
  - VP-017
  - VP-016
holdout_scenarios:
  - "HS-COERCION-001-A-001"
  - "HS-COERCION-001-A-002"
  - "HS-COERCION-001-A-003"
  - "HS-COERCION-001-A-004"
depends_on: []
# depends_on is empty: Stage 1 coercion fixes operate within prism-spec-engine and prism-bin
# independently of the ADR-058 Stage 2 routing story. No upstream prerequisites exist.
blocks:
  - S-ADR058-OCSF-ROUTING-001
# blocks justification: S-ADR058-OCSF-ROUTING-001 (Stage 2) enables ocsf_column_naming=true
# for Claroty. When that flag is active, sensor data flows through build_column_array with
# coercion applied. If EC-016-013-007/008/009 gaps are unfixed when Stage 2 lands, Claroty's
# live API responses with Object/Array-valued fields on String columns will silently produce
# the wrong data. Stage 1 must land first to prevent silent data loss under Stage 2.
estimated_days: 2
risk: LOW
# Risk justification: three known gaps with clear behavioral specs; no algorithm design
# needed; implementation is confined to coerce_value and build_column_array string arm.
assumption_validations: []
risk_mitigations: []
cycle: "v1.0.0-brownfield"
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-spec-engine/src/column_mapping.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-spec-engine/tests/bc_2_16_003_test.rs"
input-hash: "006da3c"
traces_to:
  - "BC-2.16.003"
  - "BC-2.02.011"
tags:
  - ocsf-routing
  - coercion
  - adr-058
  - stage1
  - ec-016-013-007
  - ec-016-013-008
  - ec-016-013-009
  - ec-016-013-030
  - column_coercion_failure
  - claroty-live
---

# S-ADR058-OCSF-COERCION-001: ADR-058 Stage 1 — Column Coercion Gap Closure

## Authority

**BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec.** Version `1.19`, status: draft
(modified 2026-08-19). Primary behavioral authority. The §Type Coercion Algorithm, §Full
Coercion Matrix, EC-016-013-007/008/009 KNOWN GAP annotations, and §Coercion Warning
Observability DEFECT section are the acceptance-criteria source for this story. Note: BC-2.16.003
§Interpretation A (Arrow field naming) and §Claroty Contracted OCSF Mappings are Stage 2
territory and do not change Stage 1's scope.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**ADR-058 v2.26: v1 Column Naming — OCSF Field-Path Routing.** Version `2.26`, status:
accepted (2026-08-20). §H (Stage 1 Scope) enumerates the four deliverables this story
implements: EC-016-013-008 fix in `build_column_array`, EC-016-013-009 fix via
`ColumnMapper::coerce_value` integration, `column_coercion_failure` tracing emission, and
Integer+Object silent-null gap closure (EC-016-013-030, §H item 4, AC-008/RG-010/RG-011).
Note: ADR-058 §K (OCSF schema validation), §I5 (code obligations), §B2/§I2/§J2 amendments,
and process-gap obligation (`ocsf.unknown_class_name` WARN) affect Stage 2 scope only;
Stage 1's §H scope is unchanged.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable
Interpolation.** Version `2.30`, status: active (unchanged). Governs the Canonical
Structured Event Catalog obligation: AC-004 and AC-005 introduce the
`column_coercion_failure` tracing emission, which MUST be registered in BC-2.16.002
§Postconditions §Canonical Structured Event Catalog before the implementing PR merges
(SAP-1 / PG-LP11-001). See §BC-2.16.002 Catalog Row Obligation below.
Path: `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`.

**BC-2.02.011.** Governs the warning-emission obligation for each normalization issue.
The DEFECT in BC-2.16.003 §Coercion Warning Observability violates BC-2.02.011.

---

## Narrative

As a Prism operator, I want type-mismatched column values to be consistently diverted
to `raw_extensions` with a structured log warning, so that I can detect and diagnose
persistent type mismatches in sensor API responses and no record is silently corrupted
by passing structured JSON into a typed Arrow column.

---

## ADR-058 MUST Discharge: Mandate Anchor #2

**ADR-058 §H `ANCHOR-NEEDED`: DISCHARGED.** ADR-058 §H emission discharge anchor names this
story as the anchor for all three `column_coercion_failure` emission paths. No architect action required.

The mandate anchor record:

| MUST Statement | Path | Story | AC | Red Gate Test | Status |
|---|---|---|---|---|---|
| `column_coercion_failure` tracing emission MUST be emitted in `ColumnMapper::map_record` at demotion point (ADR-058 §H) | Path B | S-ADR058-OCSF-COERCION-001 | AC-004 | RG-005 | DISCHARGED |
| `column_coercion_failure` tracing emission MUST be emitted in `build_column_array` String+Object arm (ADR-058 §H) | Path A | S-ADR058-OCSF-COERCION-001 | AC-005 | RG-006 | DISCHARGED |
| `column_coercion_failure` tracing emission MUST be emitted in `build_column_array` Integer+String arm (ADR-058 §H) | Path A | S-ADR058-OCSF-COERCION-001 | AC-007 | RG-009 | DISCHARGED |
| Integer+Object null-substitution gap MUST be closed in `build_column_array` Integer arm (Path A: null+warn) and `coerce_value` Integer branch (Path B: Err(CoercionWarning)) (ADR-058 §H item 4) | Path A + B | S-ADR058-OCSF-COERCION-001 | AC-008 | RG-010 (Path A) / RG-011 (Path B) | DISCHARGED-pending-impl |

---

## Behavioral Contracts

| BC | Version | Status | Relevance |
|----|---------|--------|-----------|
| BC-2.16.003 | v1.19 | draft | Primary contract — §Type Coercion Algorithm, §Full Coercion Matrix, EC-016-013-007/008/009 KNOWN GAPs, §Coercion Warning Observability DEFECT |
| BC-2.02.011 | — | — | Warning-emission obligation for each normalization issue; BC-2.16.003 DEFECT violates this |
| BC-2.16.002 | v2.30 | active | Canonical Structured Event Catalog obligation — `column_coercion_failure` emit from AC-004/AC-005 must be registered in §Postconditions §Canonical Structured Event Catalog (SAP-1 / PG-LP11-001) |

---

## Red Gate Tests (SAC-1 — tdd_mode: strict)

All ten tests MUST be failing (RED) before any implementation code is written. The
test-writer is dispatched FIRST; implementer is dispatched only after all 10 are confirmed
failing with the correct compile-or-test-failure reason.

- **RG-001:** `test_coerce_value_string_type_array_input_returns_err_coercion_warning` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for String column + Array input
  (currently returns `Ok(Array)` pass-through). Covers AC-001.
  **SAP-3 reachability note (defense-in-depth):** `coerce_value` is on Path B
  (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. Path A intentionally serializes `Value::Array` to a JSON-list
  string per ENRICH-1 Design Decision 2 (EC-016-013-026) — it does NOT demote. There
  is no Path-A equivalent of RG-001's String+Array→Err demotion behavior; live Array-arm
  coverage on Path A is provided by the existing passing test
  `test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string`
  (see §RG-007 retirement note).

- **RG-002:** `test_coerce_value_string_type_object_input_returns_err_coercion_warning` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for String column + Object input
  (currently returns `Ok(Object)` pass-through). Covers AC-002.
  **SAP-3 reachability note (defense-in-depth):** `coerce_value` is on Path B
  (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. The equivalent LIVE coercion behavior on Path A for `Value::Object`
  input is covered by RG-006 (`build_column_array` String+Object→null+warn). RG-008
  and RG-009 cover Integer+String (Path A) and are not Path-A equivalents for the
  Object case.

- **RG-003:** `test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number` —
  fails until `coerce_value` parses `"42"` as `Value::Number(42)` for Integer column on
  non-numeric OCSF path (currently returns `Ok(String("42"))` pass-through). Covers AC-003.
  **SAP-3 reachability note (defense-in-depth):** `coerce_value` is on Path B
  (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. The equivalent LIVE coercion behavior on Path A is covered by
  RG-006/RG-008/RG-009 (`build_column_array`).

- **RG-004:** `test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for Integer column + String
  `"not-a-number"` on non-numeric OCSF path (currently returns `Ok(String)` pass-through).
  Covers AC-003.
  **SAP-3 reachability note (defense-in-depth):** `coerce_value` is on Path B
  (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. The equivalent LIVE coercion behavior on Path A is covered by
  RG-006/RG-008/RG-009 (`build_column_array`).

- **RG-005:** `test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning` —
  fails until (a) `map_record` places Object-valued String-column field in `raw_extensions`
  and (b) a `tracing::warn!(event_type = "column_coercion_failure")` event is emitted.
  Requires a `tracing_test` subscriber in the test. Covers AC-004.
  **Placement: in-crate unit test in `crates/prism-spec-engine/src/column_mapping.rs`
  `#[cfg(test)] mod tests`** (NOT in `tests/bc_2_16_003_test.rs`). Rationale:
  `tracing-test` (0.2.x, default features) sets env-filter `<test_crate>=trace`. An
  integration test in `tests/` compiles as its own crate (`bc_2_16_003_test`), so the
  filter becomes `bc_2_16_003_test=trace`, which excludes events whose target is
  `prism_spec_engine::column_mapping`. `logs_contain("column_coercion_failure")` would
  return false even with a correct implementation. An in-crate test uses filter
  `prism_spec_engine=trace`, which matches the emission target. Mirrors the proven
  workspace precedent in `prism-spec-engine/src/pipeline.rs`. No `no-env-filter`
  feature flag is needed — default features are correct for in-crate tests.
  **SAP-3 reachability note (defense-in-depth):** `map_record` is on Path B
  (`ColumnMapper::map_record` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. The equivalent LIVE coercion behavior on Path A is covered by
  RG-006/RG-008/RG-009 (`build_column_array`).

- **RG-006:** `test_build_column_array_string_type_object_input_returns_null_and_emits_warning` —
  fails until `build_column_array` ColumnType::String arm returns `None` (null cell) AND emits
  `tracing::warn!(event_type = "column_coercion_failure", column_type = "string",
  actual_json_kind = "object")` for `Value::Object` input. Installs a `tracing_test` subscriber
  (same pattern as RG-009/RG-005). Currently returns `Some(Value::String("{...}"))` via wildcard
  `other => other.to_string()` and emits no warn. Covers AC-005.

_(RG-007 retired: `test_build_column_array_string_type_array_input_returns_null_cell` asserted
WRONG behavior — the `Value::Array(arr)` arm correctly serializes arrays to JSON-list strings
per ENRICH-1 Design Decision 2 (BC-2.16.003 EC-016-013-026) and MUST NOT be removed.
Coverage for the ENRICH-1 Array arm is provided by existing passing tests
`test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string` and
`test_build_column_array_claroty_vlan_list_integer_elements_stringify_to_json_list_string`.)_

- **RG-008:** `test_build_column_array_integer_type_string_parseable_returns_integer` —
  fails until `build_column_array` ColumnType::Integer arm parses `Value::String("42")` as
  `Some(42)` in the Arrow Int64Array (currently `other.as_i64()` returns `None` for String,
  dropping valid integer strings). Build a ColumnSpec with ColumnType::Integer, pass a record
  with `"42"` as JSON string, assert Arrow Int64Array == `[Some(42)]` not `[None]`.
  Both tests go in the same prism-bin test file as RG-006. Covers AC-007.

- **RG-009:** `test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning` —
  fails until the Integer arm returns `None` (null cell) AND emits
  `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
  column_type = "integer", actual_json_kind = "string")` for `Value::String("not-a-number")`.
  Requires a `tracing_test` subscriber (same pattern as RG-005). Both tests go in the same
  prism-bin test file as RG-006. Covers AC-007.

- **RG-010:** `test_build_column_array_integer_type_object_input_returns_null_and_emits_warning` —
  fails until the `ColumnType::Integer` arm in `build_column_array` returns `None` (null cell)
  AND emits `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
  column_type = "integer", actual_json_kind = "object")` for `Value::Object` input. Installs
  a `tracing_test` subscriber (same pattern as RG-006 and RG-009). Placed in
  `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` (same file as
  RG-006, RG-008, RG-009). Currently `Value::Object` falls through to the `other =>
  other.as_i64()` wildcard which returns `None` silently without warning. Covers AC-008 Path A (LIVE).

- **RG-011:** `test_coerce_value_integer_type_object_input_returns_err_coercion_warning` —
  fails until `coerce_value`'s Integer branch returns `Err(CoercionWarning)` for
  `Value::Object` input (currently returns `Ok(value.clone())` pass-through). Placed in
  `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` (same in-crate
  placement as RG-005; do NOT put in the integration test file `bc_2_16_003_test.rs` — the
  `prism_spec_engine=trace` in-crate filter is required for any tracing capture, and consistent
  in-crate placement with RG-005 reduces maintenance surface). This is a pure return-value
  assertion (no tracing capture required for RG-011 itself, since Path B emits the warn via
  `map_record` at demotion time, not in `coerce_value`). Covers AC-008 Path B.
  **SAP-3 reachability note (defense-in-depth):** `coerce_value` is on Path B
  (`ColumnMapper::coerce_value` in `column_mapping.rs`), which has zero live production
  callers per ADR-058 §K5; this test is intentionally defense-in-depth / forward-compat
  per SAP-3 rule 2/3. The equivalent LIVE coverage on Path A is RG-010 (`build_column_array`
  Integer+Object→null+warn).

### BC-5.38.001 Density Check

Red Gate test count: **10** (RG-001..RG-006, RG-008, RG-009, RG-010, RG-011; RG-007 retired).
Acceptance criteria directly driven by Red Gate tests: 7 (AC-001 through AC-005, AC-007, AC-008).
AC-006 is a non-regression criterion for already-passing tests — it does not require a
new failing Red Gate test.

Density: 10 RGTs / 8 ACs = **1.25 ≥ 0.5** — compliant with BC-5.38.001.

---

## Acceptance Criteria

### AC-001: coerce_value returns Err(CoercionWarning) for String column + Array input

`ColumnMapper::coerce_value` returns `Err(CoercionWarning)` when called with
`column_type = ColumnType::String` and input `Value::Array(...)`, regardless of OCSF
field path. The array value is NOT converted to a string via `to_string()`.

(traces to BC-2.16.003 postcondition §Type Coercion Algorithm Rule 1 EC-016-013-007:
"MUST divert to raw_extensions with CoercionWarning")

### AC-002: coerce_value returns Err(CoercionWarning) for String column + Object input

`ColumnMapper::coerce_value` returns `Err(CoercionWarning)` when called with
`column_type = ColumnType::String` and input `Value::Object(...)`, regardless of OCSF
field path. The object value is NOT converted to a string via `to_string()`.

(traces to BC-2.16.003 postcondition §Type Coercion Algorithm Rule 1 EC-016-013-008:
"same defect class as EC-016-013-007")

### AC-003: coerce_value handles Integer + String on non-numeric OCSF path correctly

`ColumnMapper::coerce_value` for `column_type = ColumnType::Integer` + `Value::String`
on a non-numeric-suffix OCSF path:
- If `s.parse::<i64>()` succeeds: returns `Ok(Value::Number(n))` — string coerced to integer
- If `s.parse::<i64>()` fails: returns `Err(CoercionWarning)` — string diverted to `raw_extensions`

This extends the existing numeric-suffix Rule 2 behavior to ALL Integer+String combinations,
consistent with the spec's stated intent that `column_type` is authoritative.

(traces to BC-2.16.003 postcondition §Type Coercion Algorithm Rule 2 and EC-016-013-009:
"MUST parse and divert on failure")

### AC-004: ColumnMapper::map_record emits column_coercion_failure at demotion

`ColumnMapper::map_record` emits a structured `tracing::warn!` with
`event_type = "column_coercion_failure"` at the demotion point whenever
`coerce_value` returns `Err(CoercionWarning)` and the column value is diverted to
`raw_extensions`. The emission fields are:
- `column = %col.name` — the column identifier from `ColumnSpec`
- `column_type = %column_type_toml_name(&col.column_type)` — the declared TOML column type name (e.g. "string", "integer")
- `actual_json_kind = %actual_kind` — the JSON kind that triggered demotion ("array",
  "object", or "string")

This binding matches ADR-058 §H item 3, BC-2.16.002 catalog row 95, and the shipped
`column_type_toml_name` helper — all three now consistent (source-of-truth for the emission field schema).

This closes the DEFECT documented in BC-2.16.003 §Coercion Warning Observability.
This event MUST be registered in BC-2.16.002 §Postconditions Canonical Structured Event
Catalog before the implementing PR merges (SAP-1 / PG-LP11-001 obligation — see §BC-2.16.002
Catalog Row Obligation below).

(traces to BC-2.16.003 §Coercion Warning Observability DEFECT: "The current implementation
does NOT emit a tracing::warn! at the point of demotion. This violates BC-2.02.011.")

(traces to BC-2.16.002 §Canonical Structured Event Catalog `column_coercion_failure` (SAP-1/PG-LP11-001 obligation — catalog row MUST be registered before PR merges))

(traces to BC-2.02.011 §Graceful Normalization Error Handling (No Silent Data Loss) — warning-emission obligation for each normalization issue)

### AC-005: build_column_array ColumnType::String arm returns null cell for Object input

Returns `null` cell (`None`) for `Value::Object` input (`column_type = "string"`, Path A
`build_column_array`). An explicit `serde_json::Value::Object(_) => None` arm (plus a
`tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
column_type = "string", actual_json_kind = "object")` emission) is **added BEFORE** the
`other => Some(other.to_string())` wildcard arm. The wildcard is **retained** to stringify
the remaining scalar variants `Value::Number` and `Value::Bool` — this is correct
LIVE-DRIFT-003 behavior (BC-2.16.003 §Full Coercion Matrix Path-A Number/Bool rows) and
MUST NOT be removed. The existing `serde_json::Value::Array(arr)` arm — which correctly
serializes arrays to JSON-list strings per ENRICH-1 Design Decision 2 (BC-2.16.003
EC-016-013-026) — MUST NOT be modified.

Resulting match arm order (exhaustive): `Null` | `String(s)` | `Array(arr)` |
`Object(_)` (new) | `other =>` wildcard (Number, Bool).

**Wire-level null serialization:** The `None` null cell MUST serialize as
`"<col_name>": null` (key present, JSON null value — not an absent key) when the
RecordBatch is serialized via the production MCP path. The single production
RecordBatch→JSON path in `prism-mcp::server` §RecordBatch-to-JSON already uses
`WriterBuilder::with_explicit_nulls(true)` (BC-2.11.001 EC-11-079). The existing
`bc_2_11_001_null_row_shape_test.rs` regression suite in `prism-mcp/tests/` locks this
invariant for the MCP surface. This story introduces NO second RecordBatch→JSON emit
path (Architecture Compliance Rule 7); the Rust-level `None` return from
`build_column_array` is correct, and the downstream wire-level guarantee is held by the
existing chokepoint.

(traces to BC-2.16.003 postcondition §Full Coercion Matrix EC-016-013-008 and
BC-2.16.003 invariant "Coercion failures are non-fatal: the field value is preserved
in raw_extensions; record is NEVER dropped due to type mismatch." — At the Arrow
RecordBatch materialization layer, the null cell represents the demotion that in the
OcsfEvent path would go to raw_extensions.)

### AC-006: Existing EC-016-013-004 and EC-016-013-005 tests remain passing

The two tests added under `fix/claroty-live-api-fidelity` commit `3e9825288` remain
passing after this story's changes:
- `test_coerce_value_string_type_normalizes_integer_to_string` (EC-016-013-004)
- `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` (EC-016-013-005)

No regression on the existing String-type-first Rule 1 behavior for Number and Bool
inputs, or for the uid-suffix path with String column type.

(traces to BC-2.16.003 EC-016-013-004 and EC-016-013-005: existing test evidence for
the LIVE-DRIFT-003 fix must remain valid)

### AC-007: build_column_array ColumnType::Integer arm handles Value::String inputs with parse-attempt

`build_column_array` in `prism-bin::spec_driven_adapter` ColumnType::Integer arm, when the
extracted value is `Value::String(s)`:

- `s.parse::<i64>()` succeeds → returns `Some(n)` (string-encoded integer materialized as
  correct integer in Arrow Int64 column; no data loss for valid numeric strings)
- `s.parse::<i64>()` fails → returns `None` (null cell) AND emits
  `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
  column_type = "integer", actual_json_kind = "string")`

The new `Value::String(s)` arm is placed BEFORE the `other => other.as_i64()` wildcard in
the ColumnType::Integer block. This fixes the live Path A silent-null data loss for valid
string-encoded integers (e.g. Claroty column returning `"42"` where spec declares
`column_type = "integer"`), consistent with AC-003 on Path B (`coerce_value`).

The `column_coercion_failure` event for the parse-failure case is already covered by the
BC-2.16.002 catalog row §BC-2.16.002 Catalog Row Obligation condition (2); no catalog
amendment is needed.

(traces to BC-2.16.003 EC-016-013-025; ADR-058 §H item 2 'or dispatching through it')

### AC-008: build_column_array / coerce_value handle Integer column + Value::Object input

**Path A (`build_column_array` ColumnType::Integer arm):** An explicit
`serde_json::Value::Object(_) => None` arm is added BEFORE the `other => other.as_i64()`
wildcard in the ColumnType::Integer match block, returning `None` (null cell) AND emitting
`tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
column_type = "integer", actual_json_kind = "object")`.

**Path B (`coerce_value` Integer branch):** Add `Value::Object(_) => Err(CoercionWarning)`
symmetric with the String-branch `Value::Object` handling added in AC-002, so the demotion
path emits the warn via `map_record` (which calls `coerce_value` and handles `CoercionWarning`
by emitting the structured log and placing the value in `raw_extensions`).

Both paths use the existing `column_coercion_failure` event type and field schema
(`column`, `column_type`, `actual_json_kind`) already registered in BC-2.16.002
§Canonical Structured Event Catalog (catalog row 95) — no new `event_type` is introduced,
and no catalog amendment is required. `ColumnType::Integer` + `Value::Array` input is
already warned via the existing DD-5 downcast/skip path; this AC covers `ColumnType::Integer`
+ `Value::Object` only.

Without these arms, `Value::Object` on an Integer column falls through `other.as_i64()`
(Path A, returns `None` silently without warning — a silent-null substitution that violates
BC-2.16.003's no-silent-data-loss invariant) or returns `Ok(value.clone())` pass-through
(Path B, emits no warning).

(traces to BC-2.16.003 EC-016-013-030: Integer column + Object input — no-silent-data-loss
invariant violated by current silent-null pass-through on Path A and silent pass-through
on Path B; ADR-058 §H item 4: symmetric gap closure for Integer+Object; BC-2.16.002
§Canonical Structured Event Catalog catalog row 95 trigger (3) — `actual_json_kind =
"object"` on Integer column reuses existing catalog entry)

---

## BC-2.16.002 Catalog Row Obligation

AC-004 and AC-005 introduce a new `tracing::warn!(event_type = "column_coercion_failure")`
site. Per SAP-1 / PG-LP11-001, this MUST be registered in BC-2.16.002 §Postconditions
Canonical Structured Event Catalog in the SAME COMMIT as the implementation.

BC-2.16.002 is product-owner owned. The exact row to add is:

```
| `column_coercion_failure` | warn | `ColumnMapper::map_record`
  (`prism-spec-engine::column_mapping`) and `build_column_array`
  (`prism-bin::spec_driven_adapter`) | `column` (string — column name from `col.name`),
  `column_type` (string — declared TOML column_type value, e.g. "string", "integer"),
  `actual_json_kind` (string — one of "array", "object", "string": the JSON kind that
  triggered demotion) | `coerce_value` returned `Err(CoercionWarning)` due to type
  mismatch: (1) `column_type = "string"` with `Value::Array` or `Value::Object` input;
  (2) `column_type = "integer"` with non-parseable `Value::String` on non-numeric or
  any OCSF path. Column value diverted to `raw_extensions`; record preserved. Audit
  role: data quality observability — enables operators to detect persistent type mismatches
  between sensor API responses and TOML column declarations. Recurrence: once per demoted
  column per record. Retention: per organization audit policy. |
```

**Routing:** This catalog row addition is a product-owner amendment to BC-2.16.002. The
implementer cannot merge Story 1's PR without it. Orchestrator must dispatch product-owner
to add this row to BC-2.16.002 §Postconditions §Canonical Structured Event Catalog before
the PR is opened (or as a co-commit in the same PR). The catalog row number and version
increment are determined at delivery time by the product-owner.

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Scope |
|-----------|--------|---------------|-------|
| `ColumnMapper::coerce_value` | `prism-spec-engine::column_mapping` | Pure | Modified: add `Err(CoercionWarning)` arms for Array, Object inputs on String column; extend Integer+String to non-numeric path; add `Value::Object(_) => Err(CoercionWarning)` to Integer branch (AC-008 Path B) |
| `ColumnMapper::map_record` | `prism-spec-engine::column_mapping` | Pure | Modified: add `tracing::warn!(event_type = "column_coercion_failure")` at demotion point |
| `build_column_array` (ColumnType::String arm) | `prism-bin::spec_driven_adapter` | Pure (data transformation) | Modified: add explicit `Value::Object(_) => None` null-cell arm (+ tracing emission) BEFORE the `other => Some(other.to_string())` wildcard; wildcard retained for Number/Bool (LIVE-DRIFT-003). Array arm (ENRICH-1 EC-016-013-026) preserved. |
| `build_column_array` (ColumnType::Integer arm) | `prism-bin::spec_driven_adapter` | Pure (data transformation) | Modified (AC-008 Path A): add explicit `Value::Object(_) => None` arm (+ `tracing::warn!(event_type = "column_coercion_failure", column_type = "integer", actual_json_kind = "object")`) BEFORE the `other => other.as_i64()` wildcard; closes EC-016-013-030 silent-null gap. |
| Integration test file (Path B — coerce_value) | `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` | Pure (tests) | New tests RG-001..RG-004 added (return-value assertions only — no tracing capture; integration test filter `bc_2_16_003_test=trace` excludes library crate events) |
| In-crate unit test block (Path B — map_record + coerce_value Integer+Object) | `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` | Pure (tests) | RG-005 and RG-011 placed here (NOT in integration test) — in-crate filter `prism_spec_engine=trace` captures `column_coercion_failure` warn (RG-005); RG-011 is a pure return-value assertion for Integer+Object → Err(CoercionWarning) |
| Unit test block (Path A — build_column_array) | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` | Pure (tests) | New tests RG-006, RG-008, RG-009, RG-010 added (RG-007 retired — ENRICH-1 Array arm correct per EC-016-013-026); RG-006, RG-009, and RG-010 use `tracing_test` subscriber |

Architecture section files: `architecture/module-decomposition.md` (SS-01, SS-16).

---

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| `ColumnMapper::coerce_value` | Pure | Takes `&Value` + `&ColumnSpec` (type read via `column.column_type`) + OCSF field path, returns `Result<Value, CoercionWarning>`; called as `Self::coerce_value(&raw_value, col, ocsf_path)` in `map_record`; no I/O, no mutation |
| `ColumnMapper::map_record` | Pure (data transformation) + side-effecting (tracing) | The tracing emission added by AC-004 is a side effect; the function's return value (`MappingResult`) is deterministic given the inputs |
| `build_column_array` (ColumnType::String arm) | Pure (data transformation) + side-effecting (tracing) | Returns `Option<Value>` deterministically; tracing emission added by AC-005 is a side effect |
| RG-001..RG-006, RG-008, RG-009, RG-010, RG-011 test functions | Pure (test assertions) | Test assertions over in-memory data structures; tracing subscribers in RG-005, RG-006, RG-009, and RG-010 are test infrastructure, not production I/O |

---

### Architecture Mapping Constraints

From `architecture/module-decomposition.md` and ADR-023:

1. `prism-bin::spec_driven_adapter` MUST NOT take a hard dependency on
   `prism-spec-engine` types beyond what it already imports. The
   `build_column_array` fix uses only `ColumnType` (already imported) and
   `serde_json::Value` (already used) — no new type imports are needed.

2. The `tracing` crate is already a workspace dependency; no new crate additions
   are required.

3. `tracing_test` (or equivalent subscriber setup) MUST be declared as a
   `[dev-dependency]` in BOTH `prism-spec-engine/Cargo.toml` (for RG-005) and
   `prism-bin/Cargo.toml` (for RG-006 and RG-009). Do NOT add `tracing_test` to `[dependencies]`
   (production). At dispatch time: `prism-spec-engine/Cargo.toml` already carries
   `tracing-test = "0.2"` under `[dev-dependencies]` — no change needed there;
   `prism-bin/Cargo.toml` does NOT carry `tracing-test` — the implementer MUST add
   `tracing-test = "0.2"` to `prism-bin/Cargo.toml` `[dev-dependencies]`.

4. ADR-058 §H item 1 is specific: the `build_column_array` fix MUST use the
   null-cell approach (`None`) and NOT stringify the object/array. Stringifying
   preserves data in a wrong format; null-cell forces the operator to use
   `raw_extensions` (correct behavior).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `column_type = "string"` + `Value::Object({})` (empty object) → Path A `build_column_array` | Explicit Object arm returns `None` (null cell) + `column_coercion_failure` warn. Empty arrays are correctly handled by the ENRICH-1 arm: `Value::Array([])` → `Some("[]")` (empty JSON-list string, NOT null). |
| EC-002 | `column_type = "string"` + `Value::Object({})` (empty object) | Returns `Err(CoercionWarning)` — empty object is still an Object; null-cell in Arrow |
| EC-003 | `column_type = "integer"` + `Value::String("0")` + non-numeric path | Parses as `Value::Number(0)` — valid i64; returned as Ok; no demotion |
| EC-004 | `column_type = "integer"` + `Value::String("")` + non-numeric path | `"".parse::<i64>()` fails → `Err(CoercionWarning)` |
| EC-005 | `column_type = "string"` + `Value::Null` | Rule 1 pass-through unchanged (EC-016-013-006) — null is NOT Array or Object; not affected by this story |
| EC-006 | `column_type = "string"` + `Value::Number(132)` | Rule 1 unchanged (EC-016-013-004) — Number is already handled; not affected |
| EC-007 | `column_type = "string"` + `Value::Bool(true)` | Rule 1 unchanged — Bool is already handled; not affected |
| EC-008 | `column_type = "integer"` + `Value::String("42")` on NUMERIC-suffix path | EC-016-013-002 behavior unchanged — this is Rule 2 territory; not affected by this story |
| EC-009 | `build_column_array` ColumnType::Integer + Value::String on non-numeric path | Should behave consistently with `coerce_value` fix: attempt parse, return null cell on failure |
| EC-010 | `column_type = "integer"` + `Value::Object({...})` (any object, including empty) → Path A `build_column_array` and Path B `coerce_value` | Path A: explicit `Value::Object(_) => None` arm returns null cell + `column_coercion_failure` warn (`column_type = "integer"`, `actual_json_kind = "object"`). Path B: `Err(CoercionWarning)` returned; demotion and warn emitted via `map_record`. Ties to EC-016-013-030 (ADR-058 §H item 4). |

---

## Token Budget Estimate

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~4.5k |
| `column_mapping.rs` (coerce_value + map_record) | ~4k |
| `spec_driven_adapter.rs` (build_column_array String arm) | ~3k |
| BC-2.16.003 (coercion matrix, edge cases) | ~3.5k |
| ADR-058 §H | ~1k |
| BC-2.16.002 (catalog header, context for new row) | ~2k |
| BC-2.02.011 (warning-emission obligation) | ~1k |
| `bc_2_16_003_test.rs` (existing tests to preserve) | ~2k |
| Tool outputs (just iter, cargo nextest) | ~1k |
| **Total** | **~22k** |

21k tokens is well within a 200k agent context window (~10.5%). This story does NOT need
splitting.

---

## Tasks

### Phase A: Red Gate (test-writer dispatched FIRST — before implementer)

- T-01: Read `column_mapping.rs` `coerce_value` function and identify the Array/Object
  pass-through arms in the String branch (Rule 1 gap)
- T-02: Read `spec_driven_adapter.rs` `build_column_array` String arm wildcard fallback
- T-03: Read `bc_2_16_003_test.rs` to understand existing test structure and naming
- T-04: Write RG-001 — `test_coerce_value_string_type_array_input_returns_err_coercion_warning`
  (MUST FAIL before T-11)
- T-05: Write RG-002 — `test_coerce_value_string_type_object_input_returns_err_coercion_warning`
  (MUST FAIL before T-12)
- T-06: Write RG-003 — `test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number`
  (MUST FAIL before T-13)
- T-07: Write RG-004 — `test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err`
  (MUST FAIL before T-13)
- T-08: Write RG-005 — `test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning`
  with `tracing_test` subscriber in `crates/prism-spec-engine/src/column_mapping.rs`
  `#[cfg(test)] mod tests` block (NOT in `bc_2_16_003_test.rs` — in-crate placement
  required so `tracing-test` default filter `prism_spec_engine=trace` captures the
  `column_coercion_failure` warn; integration test filter would exclude it).
  (MUST FAIL before T-14)
- T-09: Write RG-006 — `test_build_column_array_string_type_object_input_returns_null_and_emits_warning`
  — install `tracing_test` subscriber (same pattern as RG-009) and assert BOTH null cell AND
  `tracing::warn!(event_type = "column_coercion_failure", column_type = "string",
  actual_json_kind = "object")`. (MUST FAIL before T-15)
- T-10a: Write RG-008 — `test_build_column_array_integer_type_string_parseable_returns_integer`
  in the same prism-bin test file as RG-006. Build a ColumnSpec with ColumnType::Integer,
  pass a record with `"42"` as JSON string, assert Arrow Int64Array == `[Some(42)]`.
  (MUST FAIL before T-15b)
- T-10b: Write RG-009 — `test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning`
  in the same prism-bin test file as RG-006. Use `tracing_test` subscriber to capture
  warn event; assert `None` null cell AND `event_type = "column_coercion_failure"` with
  `column_type = "integer"` and `actual_json_kind = "string"`.
  (MUST FAIL before T-15b)
- T-10c: Write RG-010 — `test_build_column_array_integer_type_object_input_returns_null_and_emits_warning`
  in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` (same file as
  RG-006, RG-008, RG-009). Install `tracing_test` subscriber. Assert `None` null cell for
  ColumnType::Integer + Value::Object AND `column_coercion_failure` warn with
  `column_type = "integer"` and `actual_json_kind = "object"`.
  (MUST FAIL before T-15c)
- T-10d: Write RG-011 — `test_coerce_value_integer_type_object_input_returns_err_coercion_warning`
  in `crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` (same in-crate
  block as RG-005; do NOT put in `bc_2_16_003_test.rs`). Assert `coerce_value` returns
  `Err(CoercionWarning)` for ColumnType::Integer + Value::Object. No tracing subscriber
  required (pure return-value assertion).
  (MUST FAIL before T-15c)
- T-GATE: Run `just iter prism-spec-engine --no-fail-fast` — confirm RG-001..RG-004 fail
  with expected compile/test-failure reasons in `crates/prism-spec-engine/tests/bc_2_16_003_test.rs`;
  confirm RG-005 and RG-011 fail in `crates/prism-spec-engine/src/column_mapping.rs`
  `#[cfg(test)] mod tests` block (in-crate placement). Confirm AC-006 tests still PASS
  (prism-spec-engine run).
  Then run `just iter prism-bin --no-fail-fast` — confirm RG-006, RG-008, RG-009, RG-010
  fail with expected compile/test-failure reasons; confirm all four are in
  `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` block (direct
  private-fn calls per Architecture Compliance Rule 2).
  Report density: 10/8 = 1.25 ≥ 0.5.
  STOP and wait for implementer dispatch.

### Phase B: Implementation (implementer dispatched AFTER T-GATE)

- T-11: Fix `coerce_value` String branch — add explicit `Value::Array` arm and
  `Value::Object` arm returning `Err(CoercionWarning)` (AC-001, AC-002). Makes RG-001
  and RG-002 green.
- T-12: Run `just iter prism-spec-engine` — verify RG-001 and RG-002 pass (along with
  all other prism-spec-engine tests; no regression on AC-006 tests).
- T-13: Fix `coerce_value` Integer branch — extend parse-attempt logic to non-numeric-suffix
  OCSF paths. When `column_type = Integer` and input is `Value::String`, attempt
  `s.parse::<i64>()` regardless of OCSF path suffix. (AC-003). Makes RG-003 and RG-004
  green.
- T-14: Add `tracing::warn!(event_type = "column_coercion_failure", column = ...,
  column_type = ..., actual_json_kind = ...)` in `ColumnMapper::map_record` at the
  demotion point (where CoercionWarning is converted to raw_extensions placement).
  (AC-004). Makes RG-005 green.
- T-15: Fix `build_column_array` String arm — add an explicit `serde_json::Value::Object(_) => None`
  arm (plus `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
  column_type = "string", actual_json_kind = "object")` emission) BEFORE the existing
  `other => Some(other.to_string())` wildcard. Do NOT remove or replace the wildcard —
  it correctly stringifies `Value::Number` and `Value::Bool` (LIVE-DRIFT-003 behavior,
  BC-2.16.003 §Full Coercion Matrix Path-A). Do NOT touch the `serde_json::Value::Array(arr)`
  arm above it — that arm is correct ENRICH-1 behavior (EC-016-013-026). Resulting exhaustive
  arm order: `Null` | `String(s)` | `Array(arr)` | `Object(_)` (new) | `other =>` wildcard.
  (AC-005). Makes RG-006 green.
- T-15b: Fix `build_column_array` ColumnType::Integer arm — add explicit `Value::String(s)`
  match arm BEFORE `other => other.as_i64()` wildcard in the ColumnType::Integer block.
  Body: attempt `s.parse::<i64>()`; `Ok(n)` → `Some(n)`; `Err(_)` → emit
  `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
  column_type = "integer", actual_json_kind = "string", ...)` and return `None`.
  (AC-007). Makes RG-008 and RG-009 green. The `column_coercion_failure` event is already
  covered by the BC-2.16.002 catalog row §BC-2.16.002 Catalog Row Obligation condition (2);
  no catalog amendment needed.
- T-15c: Fix Integer+Object gap (AC-008 — both paths):
  - **Path A:** In `build_column_array` ColumnType::Integer match block, add explicit
    `serde_json::Value::Object(_) => None` arm BEFORE the `other => other.as_i64()`
    wildcard (after T-15b's `Value::String(s)` arm). Emit
    `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
    column_type = "integer", actual_json_kind = "object")` before returning `None`.
    Makes RG-010 green.
  - **Path B:** In `coerce_value` Integer branch (`prism-spec-engine::column_mapping`), add
    `Value::Object(_) => Err(CoercionWarning)` arm symmetric with the String-branch Object
    arm added in T-11/T-12. Makes RG-011 green.
- T-16: Run `just iter prism-spec-engine` — RG-001..RG-005 and RG-011 must pass; AC-006
  tests must pass; no regressions in prism-spec-engine tests.
- T-17: Run `just iter prism-bin` — confirm RG-006, RG-008, RG-009, RG-010 pass
  (build_column_array changes from T-15, T-15b, and T-15c); no existing prism-bin tests
  regress.
- T-18: Run `just check` — full workspace gate. All tests pass.

---

## Previous Story Intelligence

N/A — this is the first story in EPIC-OCSF-ROUTING. No predecessor stories have been
implemented.

Key branch context: `fix/claroty-live-api-fidelity` contains the String-type-first
coercion fix (LIVE-DRIFT-003) in `ColumnMapper::coerce_value`. This story builds on
that fix by closing the remaining three gaps (EC-016-013-007/008/009) in the same
function and adding the missing tracing emission. The implementer MUST read the existing
`coerce_value` implementation on that branch before making changes.

The two tests from that branch that document existing behavior:
- `test_coerce_value_string_type_normalizes_integer_to_string` — EC-016-013-004
- `test_coerce_value_string_type_preserves_string_username_against_uid_heuristic` — EC-016-013-005

Both MUST remain passing after this story's changes (AC-006).

---

## Architecture Compliance Rules

1. `coerce_value` MUST remain a pure function (`pub fn coerce_value(value: &Value, column: &ColumnSpec, ocsf_field_path: &str) -> Result<Value, CoercionWarning>`). No `&self`; called as `Self::coerce_value(&raw_value, col, ocsf_path)` in `map_record`. No I/O or mutation in the coerce_value body.

2. The `tracing::warn!` emission in `map_record` MUST use `event_type = "column_coercion_failure"` as a structured field (not a format string). The emission site is `map_record`, not `coerce_value` — `coerce_value` remains pure; `map_record` handles the side effect of the warning log.

3. The `build_column_array` null-cell return MUST be `None` (not `Some(Value::Null)`). Arrow null and Arrow null-cell are different representations; `None` is correct for "column absent/demoted."

4. The `build_column_array` emission MUST use `event_type = "column_coercion_failure"` with identical field names to the `map_record` emission, per BC-2.16.002 catalog row (one catalog entry covers both emission sites per the catalog row specification in this story).

5. `prism-sensors` MUST NOT gain any new dependency on `prism-spec-engine` as a side effect of this story. Verify with `cargo tree -p prism-sensors` after implementing.

6. No `unwrap()` or `expect()` in the changed code paths. Error handling via `?` propagation or `Result` arms.

7. This story introduces NO second `RecordBatch`→JSON serialization path. The single
   production path in `prism-mcp::server` §RecordBatch-to-JSON already uses
   `WriterBuilder::with_explicit_nulls(true)` (BC-2.11.001 EC-11-079), ensuring the
   null cell from `build_column_array` surfaces as `"col": null` to LLM agents. Any new
   RecordBatch→JSON path introduced by this story MUST reuse the same
   `with_explicit_nulls(true)` configuration — failure to do so causes the null-key
   absent defect class that triggered the [C3]/[H20] live-audit escape.

---

## Library & Framework Requirements

| Library | Role | Constraint |
|---------|------|-----------|
| `tracing` | Structured log emission in `map_record` and `build_column_array` | Workspace-pinned version — do NOT specify version; use workspace inheritance |
| `tracing-test` | Capture `tracing` events in RG-005 (`crates/prism-spec-engine/src/column_mapping.rs` `#[cfg(test)] mod tests` — in-crate placement) | `tracing-test = "0.2"` in `prism-spec-engine/Cargo.toml` `[dev-dependencies]` — already present; no change needed. Do NOT add `no-env-filter` feature — default features are correct for in-crate tests. |
| `tracing-test` | Capture `tracing` events in RG-006 and RG-009 (`crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`) | `tracing-test = "0.2"` in `prism-bin/Cargo.toml` `[dev-dependencies]` — NOT yet present; implementer MUST add |
| `serde_json` | `Value` type for coerce_value inputs | Workspace-pinned version |

Do NOT add `tracing-test` to production `[dependencies]`. Do NOT use `tracing-test` in
non-test code paths.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/column_mapping.rs` | Modify: `coerce_value` String branch, `coerce_value` Integer branch (String + Object arms), `map_record` demotion point; add RG-005 and RG-011 to `#[cfg(test)] mod tests` block | Primary implementation file; RG-005 (tracing capture) and RG-011 (Integer+Object return-value assertion) placed here (in-crate) |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: `build_column_array` ColumnType::String arm (explicit Object arm) and ColumnType::Integer arm (String + Object arms) | Secondary implementation file |
| `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` | Modify or create: add RG-001..RG-004 test functions | Test file; create if not present. RG-005 and RG-011 are NOT placed here — see `column_mapping.rs` row above. |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: add RG-006, RG-008, RG-009, RG-010 to `#[cfg(test)] mod tests` block (RG-007 retired) | Direct calls to private `build_column_array` — Architecture Compliance Rule 2: no public API surface expansion just for a test |
| `crates/prism-bin/Cargo.toml` | Modify: add `tracing-test = "0.2"` to `[dev-dependencies]` | Required for RG-006, RG-009, RG-010 `tracing_test` subscribers — NOT yet present in prism-bin |

Do NOT modify: any TOML sensor spec file; any BC or ADR file (product-owner / architect
scope); `prism-spec-engine/Cargo.toml` (already carries `tracing-test = "0.2"` — no change
needed).

---

## Forbidden Dependencies

Build-time enforcement rules:

- `prism-sensors` MUST NOT gain a dependency on `prism-spec-engine` — if `cargo tree -p prism-sensors` shows `prism-spec-engine` after this story, the story has introduced a forbidden import.
- `prism-bin` MUST NOT gain any new non-test dependency on `prism-spec-engine` types beyond the existing `prism_spec_engine::ColumnType` and friends already imported in `spec_driven_adapter.rs`. The fix to `build_column_array` uses only the existing imports.

---

## TD-VSDD-097 / POL-29 Three-Dimension Sweep Verdict

**Dimension 1 — Sibling pair:** BC-2.16.003 (the governing contract for this story) was
authored in isolation, not as part of a split. Its related BCs (BC-2.02.007, BC-2.02.008,
BC-2.02.011) are dependency relationships, not sibling pairs created at the same time.
No named twin artifact exists that requires simultaneous update. VERDICT: CLEAR.
Action: none required.

**Dimension 2 — Downstream copy target:** The `column_coercion_failure` tracing emission
spec (AC-004 field schema) is the source from which the BC-2.16.002 catalog row is
derived. The product-owner will transcribe the row contents from this story's
§BC-2.16.002 Catalog Row Obligation section into BC-2.16.002. The product-owner
transcription is WITHIN the same implementation burst (same PR or co-committed)
per SAP-1 obligation. If the product-owner transcribes correctly, downstream copy
target is swept in the same atomic commit. VERDICT: MITIGATED (requires product-owner
to transcribe the catalog row in the same PR — not auto-satisfied; orchestrator must
dispatch product-owner as part of this story's delivery).

**Dimension 3 — Mandate anchor:** ADR-058 §H carries the `column_coercion_failure` emission
and gap-closure MUSTs for four items. This story's §ADR-058 MUST Discharge section anchors
all four: AC-004/RG-005 (Path-B map_record emission), AC-005/RG-006 (Path-A String+Object
null+warn), AC-007/RG-009 (Path-A Integer+String warn), and AC-008/RG-010+RG-011
(§H item 4 Integer+Object null+warn Path-A and Err(CoercionWarning) Path-B). ADR-058 §H
anchor line now cites AC-008/RG-010/RG-011 explicitly (placeholder filled this burst).
VERDICT: DISCHARGED — bidirectional: §H item 4 → AC-008 → RG-010/RG-011 (story §ADR-058
MUST Discharge table row 4); AC-008 → ADR-058 §H item 4 (traces-to clause in AC-008 body);
no architect action pending.

---

### v1.46 Amendment Sweep (AC-008 + RG-010/RG-011 Integer+Object gap — EC-016-013-030 + ADR-058 §H item 4 allocation)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): AC-008 / RG-010 / RG-011 are COERCION-001
Stage 1 scope only — the Integer+Object coercion fix is in `build_column_array`
(ColumnType::Integer arm, prism-bin) and `coerce_value` (Integer branch, prism-spec-engine).
ROUTING-001 has no Integer+Object AC, no Integer+Object RG, and no EC-016-013-030 scope.
ADR-058 §Authority pin updated v2.25→v2.26 in this burst; ROUTING-001's ADR-058 pin refresh
remains deferred to ROUTING-001's own delivery per established scope rule (consistent with
v1.45 COERCION-001-specific pin bump pattern). VERDICT: ROUTING-001 UNAFFECTED by content
additions; ADR-058 pin refresh DEFERRED (intentional, consistent with prior bursts).

**Dimension 2 — Downstream copy target:**

(a) EC-016-013-030 and ADR-058 §H item 4 are the sources that drove this amendment — they
were already added to BC-2.16.003 and ADR-058 by the architect/PO. No further downstream
copy artifact to sweep. VERDICT: CLEAR.
(b) BC-2.16.002 §Canonical Structured Event Catalog catalog row 95 covers
`column_coercion_failure` with `actual_json_kind` as a field; the trigger condition now
includes `column_type = "integer"` + `Value::Object` input (trigger 3). The catalog row
prose already covers this case via its generic `actual_json_kind` field definition — no
catalog amendment is required (no new `event_type`, no new fields). VERDICT: CLEAR.
(c) ADR-058 §H item 4 anchor line: placeholder "(pending story-writer/PO allocation this
burst)" replaced with "AC-008 (RG-010 Path-A build_column_array Integer+Object null+warn;
RG-011 Path-B coerce_value Integer+Object Err(CoercionWarning))". ADR-058 `input-hash`
updated 18b74fe→ae3047b (hook-computed). Story `input-hash` updated 638c633→006da3c
(hook-computed). VERDICT: COMPLETE.

**Dimension 3 — Mandate anchor:**

ADR-058 §H item 4 anchor line now names AC-008/RG-010/RG-011 (placeholder filled this
burst — see ADR-058 §H item 4, post-edit). Story §ADR-058 MUST Discharge table row 4
anchors: "Integer+Object null-substitution gap MUST be closed" → AC-008 → RG-010 (Path-A)
/ RG-011 (Path-B), status DISCHARGED-pending-impl. Bidirectional trace confirmed:
(forward) ADR-058 §H item 4 → "AC-008 (RG-010 ... RG-011 ...)" in anchor line;
(backward) AC-008 body `(traces to ... ADR-058 §H item 4 ...)` in §Acceptance Criteria.
VERDICT: MANDATE ANCHOR BIDIRECTIONAL; NO NEW UNANCHORED MUSTs.

---

### v1.42 Amendment Sweep (RG-005 relocation to in-crate unit test — tracing-test filter fix)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ROUTING-001 has its own unrelated RG-005
(`test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names` in
`crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`). That test asserts
on Arrow field naming, not on tracing capture, and its placement was already correct for
the private-fn direct-call pattern. No mirror of COERCION-001's `bc_2_16_003_test.rs`
RG-005 placement text exists in ROUTING-001: search confirmed zero occurrences of
`bc_2_16_003_test` in ROUTING-001, and no `map_record` tracing-subscriber pattern
anywhere in that file. Absence-of-string verified semantically: ROUTING-001's tracing
tests (RG-018 etc.) are in `prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests`
and cover `pipeline_result_to_record_batch`, not `map_record`. VERDICT: NO MIRROR IN
ROUTING-001; ROUTING-001 UNAFFECTED.

**Dimension 2 — Downstream copy target:**

The RG-005 placement text — §Red Gate Tests placement note, §Architecture Mapping in-crate
row, §Tasks T-08 file attribution, §Tasks T-GATE split confirmation, §Library & Framework
tracing-test location cell, and §File Structure Requirements rows — are dispatch instructions
to the test-writer. No downstream BC, ADR, or index artifact copies these placement
instructions verbatim. The BC-2.16.002 catalog row obligation (§BC-2.16.002 Catalog Row
Obligation) cites `S-ADR058-OCSF-COERCION-001 AC-004 RG-005` as the anchor, naming only
the test ID and AC, not the file location; that anchor is unchanged by relocation.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

RG-005 anchors AC-004 and the ADR-058 §H `column_coercion_failure` emission MUST for Path B
(`ColumnMapper::map_record`). The §ADR-058 MUST Discharge table row reads:
"Path B | S-ADR058-OCSF-COERCION-001 | AC-004 | RG-005 | DISCHARGED". Relocation of
RG-005 from `tests/bc_2_16_003_test.rs` to `src/column_mapping.rs #[cfg(test)] mod tests`
does not change the test name, AC reference, or mandate anchor binding — it only changes
which file the test lives in. The anchor remains valid and discharged after relocation.
VERDICT: MANDATE ANCHOR HOLDS; NO NEW UNANCHORED MUSTs.

---

### v1.40 Amendment Sweep (Leg 2 pin bump — BC-2.16.003 v1.18→v1.19 + BC-2.16.002 v2.28→v2.29; sibling coordination with ROUTING-001 v1.43→v1.44)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): amended in same burst (v1.43→v1.44) — §Authority BC-2.16.003 pin v1.18→v1.19; §Authority BC-2.16.002 pin v2.28→v2.29; §Behavioral Contracts table pins updated; 13 `§Interpretation A v1.18` inline stamps stripped to version-free form per Bucket B terminal normalization. VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

Changed surfaces in COERCION-001: (1) §Authority BC-2.16.003 `Version \`1.18\`` → `Version \`1.19\``; (2) §Authority BC-2.16.002 `Version \`2.28\`` → `Version \`2.29\``; (3) §Behavioral Contracts table BC-2.16.003 row v1.18→v1.19; (4) §Behavioral Contracts table BC-2.16.002 row v2.28→v2.29. None of these loci are verbatim-copied into any downstream artifact. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward with behavioral content unchanged. VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.39 Amendment Sweep (FB-58/60 records micro-burst: ADR-058 §Authority status-date "(2026-08-18)"→"(2026-08-19)"; sibling provenance-label normalization in ROUTING-001)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ROUTING-001 amended in same burst (v1.41→v1.42) — §Authority §B2/§D1/§G/§I1/§I2/§J2 provenance labels normalized to version-free form and ADR-058 status-date corrected "(2026-08-18)"→"(2026-08-19)". COERCION-001 §Authority already uses clean version-free form for all provenance references; only the ADR-058 date fix applies here.
VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surface is: §Authority ADR-058 status-date parenthetical "(2026-08-18)"→"(2026-08-19)". No downstream artifact copies this date parenthetical verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward unchanged.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.38 Amendment Sweep (FB-55/56/57 LEG 2 — §Authority BC-2.16.003 pin v1.17→v1.18 + modified-date "(modified 2026-08-19)"; completes pin sweep the v1.37 row claimed but did not apply)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ROUTING-001 amended in same burst (v1.40→v1.41) — §Authority BC-2.16.003 modified-date corrected "(modified 2026-08-18)"→"(modified 2026-08-19)" and §I1 origin-provenance label corrected v2.23→v2.21. Both sibling §Authority BC-2.16.003 modified-date corrections applied in the same burst.
VERDICT: SIBLING AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The changed surfaces are: (1) §Authority BC-2.16.003 version pin v1.17→v1.18 (completing the v1.37 changelog claim — §Behavioral Contracts table was already at v1.18; §Authority was stale at v1.17); (2) §Authority BC-2.16.003 modified-date parenthetical "(modified 2026-08-18)"→"(modified 2026-08-19)". No downstream artifact copies these version-pin or date sites verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced. All existing MUST anchors carried forward unchanged.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.37 Amendment Sweep (FB-52/53/54 LEG 3 sibling pin maintenance — ADR-058 v2.22→v2.23 + BC-2.16.003 v1.17→v1.18 at §Authority + §Behavioral Contracts table; content unaffected)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): pin bumps ADR-058 v2.22→v2.23 and BC-2.16.003
v1.17→v1.18 are coordinated. ROUTING-001 bumped to v1.40 in the same burst.
VERDICT: SIBLING PIN BUMP EXECUTED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

Only version pins in §Authority and §Behavioral Contracts table changed. No prose, AC, RG, or
mechanism content changed. No downstream artifact copies these pin values verbatim.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs added. All existing MUST anchors carried forward unchanged.
VERDICT: NO NEW UNANCHORED MUSTs.

---

### v1.36 Amendment Sweep (BC-2.16.003 pin v1.15→v1.17 — FB-49/51 Leg 2 sibling pin maintenance; F-P49-MED-003/F-P51-LOW-001 stale pin at §Authority + §Behavioral Contracts table)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P51-MED-001/F-P49-MED-001/F-P49/51-MED-002
are ROUTING-001 scope only — RG-026, T-15, AC-007a, and AC-013 are specific to the
`pipeline_result_to_record_batch` raw_extensions aggregation path and the §J2 synthesized-name
guard. This story's content is UNAFFECTED — COERCION-001 has no raw_extensions aggregation
path, no §J2 guard logic, and no T-15/AC-013 scope. BC-2.16.003 pin v1.16→v1.17 requires
sibling coordination; ROUTING-001 amended in same burst (v1.38→v1.39). VERDICT: CONTENT
UNAFFECTED; BC PIN BUMP ONLY; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Authority BC-2.16.003 pin and §Behavioral Contracts body table BC-2.16.003 row are the two
live BC version-reference sites in COERCION-001. Both updated v1.15→v1.17 (§Authority was
stale at v1.15; §Behavioral Contracts table was stale at v1.16; both now reflect v1.17).
No downstream artifact copies these version-pin sites verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced; no new mandates require anchoring. VERDICT: N/A — no new mandates.

---

### v1.35 Amendment Sweep (ADR-058 pin v2.21→v2.22; BC-2.16.003 pin v1.15→v1.16 — OCSF-correctness Claroty fix-burst leg 3 sibling coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): pin bumps v2.21→v2.22 and v1.15→v1.16
swept to both stories in same burst. ROUTING-001 amended in same burst (v1.37→v1.38
story-writer leg) with F-P46-MED-001/F-P48-MED-001/F-P48-MED-002 content changes plus
pin sweeps. This story's content is UNAFFECTED by those findings — EC-016-013-028,
§J2 reserved-name guard, and EC-016-013-011 corrected text are Stage 2 scope; COERCION-001
has no ip_list routing, no §J2 guard logic, and no `prism_describe` Tier-1/Tier-2 model.
VERDICT: CONTENT UNAFFECTED; ADR/BC PIN BUMP ONLY.

**Dimension 2 — Downstream copy target:**

§Authority ADR-058 and §Behavioral Contracts BC-2.16.003 version pins are terminal
references in this artifact — no downstream artifact copies them verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUST blocks introduced; no new mandates require anchoring. VERDICT: CLEAR.

---

### v1.30 Amendment Sweep (ADR-058 pin v2.16→v2.17 + SAP-3 RG-001..005 annotations — OCSF-correctness Claroty SPEC pass-32 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ADR-058 pin v2.16→v2.17 swept to both stories in same burst. ROUTING-001 amended in same burst (v1.30→v1.31 — F-P32-MED-001 raw_extensions locus re-attribution). VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §I1 (v2.17) clarification (individual-field naming applies to `ocsf_field == Some` only) is Stage 2 ROUTING-001 scope. COERCION-001's scope is Stage 1 type coercion; no §I1/§I2 column-routing prose exists in this story. SAP-3 annotations are rationale-only, no normative prose. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

SAP-3 annotations on RG-001..005 are reachability rationale only — no new behavioral MUSTs introduced. ADR-058 pin is records-tier. VERDICT: N/A — no new mandates.

---

### v1.29 Amendment Sweep (BC-2.16.003 pin v1.12→v1.13 — OCSF-correctness Claroty SPEC pass-30 sibling coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): BC-2.16.003 pin v1.12→v1.13 swept to both stories in same burst. ROUTING-001 amended in same burst (v1.28→v1.29). VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §OCSF Field Validation (v1.13) adds Path-A/Path-B qualifier (vendor-extended paths → first-class Arrow columns on Path A, NOT raw_extensions). Downstream contradiction check: this story's scope is Stage 1 type coercion (§Type Coercion Algorithm, EC-016-013-007/008/009 gap closure) — no §OCSF Field Validation prose exists in COERCION-001. VERDICT: NO DOWNSTREAM CONTRADICTION; no correction needed.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. BC-2.16.003 pin is a version-tracking update (PO bump — §OCSF Field Validation Path-A/Path-B qualifier). VERDICT: N/A — no new mandates.

---

### v1.28 Amendment Sweep (BC-2.16.002 pin v2.27→v2.28 — OCSF-correctness Claroty SPEC pass-28 sibling coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): BC-2.16.002 pin v2.27→v2.28 swept to both stories in same burst. ROUTING-001 amended in same burst (v1.27→v1.28 — BC-2.16.002 pin v2.27→v2.28 sibling coordination). VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.002 §Authority entry and §Behavioral Contracts body table are the sole live BC-2.16.002 pin sites in this story. Both updated v2.27→v2.28. No downstream artifact copies these verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. BC-2.16.002 pin update is a records-tier version-tracking update (PO bump — §Canonical Structured Event Catalog ocsf.unknown_class_name row gains pending-wiring annotation in v2.28). VERDICT: N/A — no new mandates.

---

### v1.27 Amendment Sweep (F-P27-MED-001 §Mandate Anchor #2 stale §H verbatim quote + stale AC/RG trace — OCSF-correctness Claroty SPEC pass-27 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P27-MED-001 is COERCION-001 scope only — §Mandate Anchor #2 is a COERCION-001-specific discharge record; ROUTING-001 §Mandate Anchor sections reference §D2/§J2 (not §H three-way anchor), confirmed unchanged. VERDICT: ROUTING-001 UNAFFECTED.

**Dimension 2 — Downstream copy target:**

§Mandate Anchor #2 prose and table are the only changed loci. The verbatim ADR-058 §H anchor-string copy has been removed and replaced with a section-anchor-only reference to ADR-058 §H emission discharge anchor. §Mandate Anchor #2 now contains NO verbatim copy of the ADR §H anchor text — future ADR §H edits will not drift this locus. The main §TD-VSDD-097 Dimension 3 prose was also carrying the stale verbatim quote; corrected in the same burst. VERDICT: COMPLETE — §Mandate Anchor #2 and §TD-VSDD-097 Dimension 3 are both anchor-only; no verbatim §H anchor-string copy remains in this story.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P27-MED-001 re-anchors three existing emission MUSTs to their correct AC/RG pairs (AC-004/RG-005 Path-B map_record; AC-005/RG-006 Path-A String+Object; AC-007/RG-009 Path-A Integer+String); no new behavioral obligation is added. VERDICT: N/A — no new mandates.

---

### v1.26 Amendment Sweep (F-P26-MED-001 RG-006 extended null+warn; ADR-058 pin v2.16 — OCSF-correctness Claroty SPEC pass-26 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P26-MED-001 is COERCION-001 scope only — RG-006 extension (null+warn) has no counterpart in ROUTING-001. ADR-058 pin v2.15→v2.16 swept to both stories in same burst. VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST (pin sweep only).

**Dimension 2 — Downstream copy target:**

§Red Gate Tests RG-006 description, T-09, §Architecture Mapping unit-test row, §Purity Classification, §Architecture Mapping Constraints item 3, and §Library & Framework Requirements tracing-test prism-bin row are the dispatch instructions carrying RG-006 test specification. All six loci updated in this burst. ADR-058 §Authority pin is the sole live ADR pin site. No downstream artifact copies these surfaces verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P26-MED-001 extends an existing Red Gate test; it does not add a new behavioral obligation. The warn emission was already specified in AC-005 and T-15; RG-006 now enforces it at the Red Gate layer. VERDICT: N/A — no new mandates.

---

### v1.25 Amendment Sweep (F-P25-MED-001 AC-005/T-15 add-Object-retain-wildcard; ADR-058 pin v2.15; BC-2.16.003 pin v1.12 — OCSF-correctness Claroty SPEC pass-25 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P25-MED-001 is COERCION-001 scope only — AC-005/T-15 add-Object-retain-wildcard has no counterpart in ROUTING-001. ADR-058 pin v2.14→v2.15 and BC-2.16.003 pin v1.11→v1.12 swept to both stories in same burst. VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST (pin sweeps only).

**Dimension 2 — Downstream copy target:**

AC-005 and T-15 are the authoritative dispatch instructions for the test-writer and implementer. §Architecture Mapping `build_column_array` scope column updated to match add-Object-retain-wildcard semantics. No downstream artifact copies these surfaces verbatim. ADR-058 §Authority entry and BC-2.16.003 §Authority entry and §Behavioral Contracts body table are the sole live pin sites; no downstream artifact copies them. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P25-MED-001 is a behavioral-precision fix: wildcard-retention obligation now explicit in AC-005 and T-15. Existing mandate anchors (AC-004/RG-005 for `column_coercion_failure` emission; AC-007/RG-008/RG-009 for Integer arm) unchanged. VERDICT: N/A — no new mandates.

---

### v1.24 Amendment Sweep (F-P24-HIGH-001 Path-A Array-arm preserved/Object-only null-demote + RG-007 retired + F-P24-MED-001 coerce_value signature + BC-2.16.003 v1.11 pin — OCSF-correctness Claroty SPEC pass-24 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P24-HIGH-001 and F-P24-MED-001 are COERCION-001 scope only — ROUTING-001 has no `build_column_array` String arm or `coerce_value` signature. BC-2.16.003 pin v1.10→v1.11 swept to both stories in same burst. VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST (BC-2.16.003 pin only).

**Dimension 2 — Downstream copy target:**

RG-007 retirement swept across all RG-enumeration surfaces in this story:
- §Red Gate Tests header: "nine"→"eight", "9"→"8".
- RG-007 entry: removed; ENRICH-1 coverage note referencing `test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string` and `test_build_column_array_claroty_vlan_list_integer_elements_stringify_to_json_list_string` added.
- §BC-5.38.001 Density Check: count 9→8 (RG-001..RG-006, RG-008, RG-009), density 9/7=1.29→8/7=1.14.
- §Architecture Mapping `build_column_array` scope column: "explicit null-cell arms for Array and Object"→"explicit `Value::Object(_)` null-cell arm; Array arm (ENRICH-1 EC-016-013-026) preserved".
- §Architecture Mapping Path A unit-test row: "RG-006..RG-009"→"RG-006, RG-008, RG-009 (RG-007 retired)".
- §Purity Classification RG range: "RG-001..RG-009"→"RG-001..RG-006, RG-008, RG-009".
- §File Structure Requirements prism-bin unit-test row: "RG-006..RG-009"→"RG-006, RG-008, RG-009 (RG-007 retired)".
- T-10 (RG-007 authoring task): removed; T-10a/T-10b "RG-006/RG-007" references updated to "RG-006".
- §T-GATE: "RG-006..RG-009"→"RG-006, RG-008, RG-009", "all four"→"all three", density 9/7=1.29→8/7=1.14.
- T-15 green-driver: "Makes RG-006 and RG-007 green"→"Makes RG-006 green"; body rewritten to Object-only.
- T-17 verify targets: "RG-006..RG-009"→"RG-006, RG-008, RG-009".

Post-sweep grep for `RG-007` in normative body: zero live references remain. §v1.24 Amendment Sweep and §Changelog v1.24 row carry the retirement record; additional records-tier mentions appear in historical amendment-sweep (§v1.23, §v1.17, §v1.7) and changelog rows — all grandfathered per TD-VSDD-091 ratchet scoping. VERDICT: COMPLETE; all RG-enumeration surfaces verified.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P24-HIGH-001 retires a wrong test (no new obligation). F-P24-MED-001 is a signature accuracy fix. BC-2.16.003 pin is a version-tracking update. VERDICT: N/A — no new mandates.

---

### v1.23 Amendment Sweep (F-P23-MED-001 §Library & Framework RG-009 location text sync — OCSF-correctness Claroty SPEC pass-23 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P23-MED-001 is a same-defect-class fix applied symmetrically to both stories — ROUTING-001 §Library & Framework Requirements tracing-test row (RG-018) and §File Structure Requirements Cargo.toml Notes cell both carried `prism-bin/tests/` text; the stale origin is identical (provisioned together in pass-19/pass-20 before the pass-21 relocation propagated only to §Architecture Mapping, §T-GATE, and §File Structure prism-bin row but not to §Library & Framework). ROUTING-001 amended in same burst (v1.23→v1.24). VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

All test-location-bearing surfaces in this story checked for `prism-bin/tests/` references to private-fn RGs:

- §Library & Framework Requirements tracing-test row (RG-009 location): corrected from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — this is the sole locus changed in this burst.
- §Architecture Mapping Path A row: references `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — confirmed CURRENT (updated v1.21 per F-P21-MED-001).
- §Architecture Mapping Constraints item 3: references `prism-bin/Cargo.toml` and `prism-spec-engine/Cargo.toml` only (Cargo.toml provisioning, not a test-file location) — CURRENT.
- §File Structure Requirements prism-bin unit-test row: references `src/spec_driven_adapter.rs #[cfg(test)] mod tests` block — confirmed CURRENT (updated v1.21 per F-P21-MED-001).
- §File Structure Requirements Cargo.toml row Notes: `Required for RG-009 tracing_test subscriber — NOT yet present in prism-bin` — contains no `prism-bin/tests/` reference; CURRENT.
- §T-GATE: references `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` block explicitly — confirmed CURRENT (updated v1.21 per F-P21-MED-001).
- §Tasks T-10a/T-10b: "in the same prism-bin test file as RG-006/RG-007" — resolves to spec_driven_adapter.rs `#[cfg(test)] mod tests` per §File Structure ground truth; contains no `prism-bin/tests/` reference; CURRENT.
- §Purity Classification: no `prism-bin/tests/` reference — CURRENT.
- §Red Gate Tests RG-008/RG-009 text: references `tracing_test` subscriber without naming a file path — CURRENT.

Post-edit grep: ZERO `prism-bin/tests/` references for private-fn RGs remain in this story. VERDICT: COMPLETE; all location-bearing surfaces verified.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P23-MED-001 is a records-tier text-sync correction. VERDICT: N/A — no new mandates.

---

### v1.22 Amendment Sweep (ADR-058 pin sweep v2.13→v2.14 — sibling coordination with ROUTING-001 pass-22)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P22-MED-001/002, F-P22-OBS-2/3 are ROUTING-001 scope only — prism-ocsf private-fn routing, TOML-driven wire-shape RG verify commands, and T-22(c) doc-table count are not present in COERCION-001. ADR-058 §Authority pin v2.13→v2.14 swept to both stories in same burst. VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority entry is the sole live ADR pin site in this story. Updated v2.13→v2.14. No downstream artifact copies this entry verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. This is a records-tier ADR pin update. VERDICT: N/A — no new mandates.

---

### v1.21 Amendment Sweep (F-P21-MED-001 prism-bin RG relocation to src mod tests — OCSF-correctness Claroty SPEC pass-21 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P21-MED-001 applies to BOTH stories —
ROUTING-001 §File Structure Requirements row for prism-bin RG-003..006/008..010/014..022
(`pipeline_result_to_record_batch` / `ocsf_field_to_arrow_name`) relocated to
`src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`; T-12/T-14 attribution corrected
(F-P21-LOW-001, ROUTING-001 scope only). ROUTING-001 amended in same burst (v1.21→v1.22).
VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Architecture Mapping (Path A row), §T-GATE (prism-bin confirmation text), and §File
Structure Requirements (prism-bin row) are the authoritative dispatch instructions.
No downstream artifact copies these rows verbatim. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P21-MED-001 is a test-location correctness fix. VERDICT: N/A
— no new mandates.

---

### v1.20 Amendment Sweep (F-P20-LOW-001b false-sibling-sentence fix + F-P20-LOW-002 Token Budget BC-2.02.011 row — OCSF-correctness Claroty SPEC pass-20 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P20-MED-002, F-P20-LOW-001a, F-P20-OBS-001,
and F-P20-OBS-002 are ROUTING-001 scope — tracing-test dependency-aware provisioning, sweep
reordering, RG-013 SAP-3 rationale, and RG-006/T-12/T-14 green-driver attribution. ROUTING-001
amended in same burst (v1.20→v1.21). F-P20-LOW-001b and F-P20-LOW-002 are COERCION-001 scope:
§v1.18 false sibling sentence corrected; Token Budget BC-2.02.011 row added. VERDICT: SWEPT;
ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

F-P20-LOW-001b: §v1.18 Amendment Sweep §Dimension 1 is a records-tier correction. No downstream
artifact copies this sentence verbatim. VERDICT: CLEAR.

F-P20-LOW-002: Token Budget table is a POL-8 count-parity fix (BC-2.02.011 was in frontmatter
and §Authority but absent from Token Budget). No downstream artifact copies the Token Budget
table. Total updated ~21k → ~22k. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. F-P20-LOW-001b and F-P20-LOW-002 are accuracy and count-parity
corrections. VERDICT: N/A — no new mandates.

---

### v1.19 Amendment Sweep (F-P19-MED-001 prism-bin tracing-test provisioning + F-P19-LOW-001 T-12 nextest filter false-green fix — OCSF-correctness Claroty SPEC pass-19 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P19-MED-001 applies to BOTH stories —
ROUTING-001 also gains a `prism-bin/Cargo.toml` `tracing-test = "0.2"` `[dev-dependencies]`
row in §Library & Framework Requirements and §File Structure Requirements (RG-018 in
`prism-bin/tests/` requires the same `tracing_test` subscriber). F-P19-LOW-001 is
COERCION-001 scope only — ROUTING-001 has no equivalent T-12 nextest filter false-green.
ROUTING-001 amended in same burst (v1.19→v1.20). VERDICT: SWEPT; ROUTING-001 AMENDED
IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Architecture Mapping Constraints item 3, §Library & Framework Requirements (tracing-test
rows), and §File Structure Requirements (prism-bin/Cargo.toml row) are the authoritative
provisioning instructions for the implementer. No downstream artifact copies these tables
verbatim. T-12 task text is the authoritative gate-command specification for the implementer.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The prism-bin tracing-test provisioning is a
dev-infrastructure correctness fix; the T-12 filter fix is a TDD-gate-accuracy correction.
VERDICT: N/A — no new mandates.

---

### v1.18 Amendment Sweep (F-P18-MED-001 T-GATE/T-16 prism-bin split for RG-006..009 + F-P18-OBS-001 sweep subsection descending reorder — OCSF-correctness Claroty SPEC pass-18 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P18-MED-001 is COERCION-001 scope only —
the prism-bin red-gate/green-gate split applies to RG-006..RG-009 in `build_column_array`,
which has no counterpart in ROUTING-001. F-P18-OBS-001 applies to COERCION-001 amendment
sweep ordering only; ROUTING-001 amendment sweep ordering was corrected in ROUTING-001 v1.21
(pass-20, F-P20-LOW-001a). ROUTING-001 receives separate fixes this burst (F-P18-MED-002/003/004).
VERDICT: SWEPT; ROUTING-001 UNAFFECTED BY COERCION-001-SPECIFIC FINDINGS.

**Dimension 2 — Downstream copy target:**

F-P18-MED-001: §T-GATE and T-16/T-17 task-plan edits. These task-plan sections are the
authoritative dispatch instructions for the test-writer and implementer. No downstream
artifact copies the T-GATE or T-16/T-17 task text. F-P18-OBS-001: The amendment sweep
subsections are record-tier prose; no downstream artifact copies sweep ordering.
VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. F-P18-MED-001 is a task-plan coherence fix;
F-P18-OBS-001 is a records-tier ordering correction. VERDICT: N/A — no new mandates.

---

### v1.17 Amendment Sweep (F-P17-MED-001 RG-008/009 propagation to 3 tables + RG-006/007 crate correction + F-P17-LOW-001 false sweep sentence + F-P17-LOW-002 heading de-duplication — OCSF-correctness Claroty SPEC pass-17 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F-P17-MED-001 is COERCION-001 scope only —
`build_column_array` Integer arm (RG-006..RG-009, §Architecture Mapping, §Purity
Classification, §File Structure Requirements) has no counterpart in ROUTING-001. F-P17-LOW-001
corrects a sentence in COERCION-001 §v1.16 Amendment Sweep §Dimension 1; ROUTING-001's
amendment sweeps do not contain an equivalent false-deferral sentence. F-P17-LOW-002 resolves
a duplicate heading in COERCION-001 only; ROUTING-001 §Architecture Mapping has no subordinate
`### Architecture Compliance Rules` heading (confirmed by sweep of ROUTING-001 v1.18).
VERDICT: SWEPT; ROUTING-001 UNAFFECTED.

**Dimension 2 — Downstream copy target:**

F-P17-MED-001: Three live enumeration-sites in this story file were updated — §Architecture
Mapping (split row), §Purity Classification (RG range row), §File Structure Requirements
(prism-bin/tests row). The §Red Gate Tests section (individual RG-008/009 definitions) and
§T-GATE task already carried correct prism-bin placement from v1.16 — these are the source of
truth per dispatch. No artifact outside this story file derives its RG-range enumerations from
these three table rows. VERDICT: CLEAR.

F-P17-LOW-002: The subordinate `### Architecture Mapping Constraints` heading (renamed from
`### Architecture Compliance Rules`, rules 1–4) is not referenced by name in any AC or task
text. AC-005's "Architecture Compliance Rule 7" cite resolves to the top-level
`## Architecture Compliance Rules` (rules 1–7), which is unchanged. No downstream artifact
copies the subordinate heading text. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. F-P17-MED-001 is a table propagation fix;
F-P17-LOW-001 is a records-tier correction; F-P17-LOW-002 is a structural de-duplication.
VERDICT: N/A — no new mandates.

---

### v1.16 Amendment Sweep (F-P16-MED-003 AC-007/RG-008/009/T-15b + F-P16-OBS-001 title expansion + BC-2.16.003 v1.9→v1.10 pin — OCSF-correctness Claroty adversary SPEC pass-16 fix-burst)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ROUTING-001 §Authority also carried abbreviated BC-2.16.003 and BC-2.16.002 titles, and a wrong BC-2.01.013 title ("DataSource Trait Adapter Pattern" → "DataSource Trait Eliminates Per-Sensor Code Duplication"). Both corrected in the same burst (ROUTING-001 v1.17→v1.18). ROUTING-001 carries no F-P16-MED-003 equivalent (build_column_array Integer arm is COERCION-001 scope only). ROUTING-001 §Authority BC-2.16.003 pin was propagated v1.9→v1.10 in the same D-2220 burst (no deferral occurred). VERDICT: SWEPT; ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §Authority pin and body BC table pin are the two live BC version-reference sites in COERCION-001. Both updated v1.9→v1.10 in this amendment. The §BC-2.16.002 Catalog Row Obligation condition (2) already covers the `column_coercion_failure` emission for Integer + non-parseable String — no catalog amendment needed for F-P16-MED-003. AC-007's trace citation references BC-2.16.003 EC-016-013-025 (new clause added by PO in same burst); the §Behavioral Contracts body table already lists BC-2.16.003 as the primary contract. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

AC-007 contains the obligation: `build_column_array` ColumnType::Integer arm MUST parse `Value::String(s)` and return `Some(n)` or `None` + warn. This MUST is anchored to BC-2.16.003 EC-016-013-025 + RG-008 (`test_build_column_array_integer_type_string_parseable_returns_integer`) + RG-009 (`test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning`). Story S-ADR058-OCSF-COERCION-001 AC-007. VERDICT: DISCHARGED — anchor present in AC-007 trace parenthetical.

---

### v1.15 Amendment Sweep (ADR-058 re-pin v2.12→v2.13 + sibling ROUTING-001 F2 task-wording coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F2 §Tasks T-11G/H/L/M/N/O authoring-wording fixed
— all six TOML-driven wire-shape RG authoring tasks now specify "load the corrected
`claroty.sensor.toml` [table] table spec (post-T-17, KF-xx: ...)" instead of "build a SensorSpec
with corrections applied"; ADR-058 §Authority pin v2.12→v2.13; §v1.16 Amendment Sweep added in
same burst. COERCION-001 tasks audit: CLEAN — no TOML-load/inline-spec inconsistency exists (all
COERCION tasks are code-level). VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority pin is the only changed site in COERCION-001 this burst. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced. VERDICT: N/A — no new mandates.

---

### v1.14 Amendment Sweep (LOW-2 AC-004 trace additions + ADR-058 re-pin v2.11→v2.12)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ADR-058 §Authority pin updated v2.11→v2.12 in
same burst. ROUTING-001 has no LOW-2 equivalent — its BC-2.16.002 and BC-2.02.011 traces were
already present from prior authoring. VERDICT: ROUTING-001 AMENDED IN SAME BURST (ADR pin only).

**Dimension 2 — Downstream copy target:**

AC-004 is the source from which the implementer reads the emission obligation. The LOW-2 additions
add two formal trace parentheticals that make BC-2.16.002 and BC-2.02.011 coverage explicit without
changing the behavioral obligation (which was already stated in AC-004 prose). Downstream: the
§BC-2.16.002 Catalog Row Obligation section references AC-004 and is unchanged. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The trace additions and ADR re-pin are notational
consistency fixes. VERDICT: N/A — no new mandates.

---

### v1.13 Amendment Sweep (ADR-058 re-pin v2.10→v2.11 + sibling sweep ROUTING-001 F3 coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F3 §Mandate Anchor #1 provenance fix applied — both
§D2 and §J2 table rows and inline prose stripped of "(v2.1)" / "since v2.1" version qualifiers;
version-free "DISCHARGED — ADR-058 §D2/§J2 carries the inline (Anchored: …) mark" form adopted.
ADR-058 §Authority pin v2.10→v2.11; §v1.14 Amendment Sweep added in same burst.
VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority pin is the sole live ADR pin site in this story. Updated v2.10→v2.11.
COERCION-001 §Mandate Anchor #2 was already version-free since pass-7 (no version qualifiers to
strip here). Sibling sweep of COERCION-001 normative prose confirmed zero additional POL-39
violations. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. VERDICT: N/A — no new mandates.

---

### v1.12 Amendment Sweep (ADR-058 re-pin v2.9→v2.10 + sibling sweep ROUTING-001 F2 coordination)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): F2 AC-011 normative prose pin removed (v2.27 +
stale temporal aside stripped; section-anchor retained); ADR-058 §Authority pin v2.9→v2.10;
§v1.13 Amendment Sweep added in same burst. VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority pin is the sole live ADR pin site in this story. Updated v2.9→v2.10.
No normative prose POL-39 violations exist in this story outside §Authority and exempt zones
(comprehensive sibling sweep confirmed zero). VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. VERDICT: N/A — no new mandates.

---

### v1.11 Amendment Sweep (ADR-058 re-pin v2.8→v2.9 + comprehensive hygiene sweep)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): Comprehensive hygiene sweep applied in same
burst (v1.11→v1.12) — F1 §Mandate Anchor #1 rewritten (§D2 and §J2 DISCHARGED), F4 §Authority
§J4 count corrected (31 pre-correction / 26 post-correction), F3 v1.9 changelog row line-cite
removed, ADR-058 §Authority pin v2.8→v2.9. VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

ADR-058 §Authority pin is the sole live ADR pin site in this story. Updated v2.8→v2.9. Comprehensive
hygiene sweep of narrative prose confirmed zero ADR-058 version pins outside §Authority and historical
amendment-sweep sections (grandfathered by TD-VSDD-091 ratchet scoping). VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The ADR-058 re-pin and hygiene sweep are accuracy
fixes, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.10 Amendment Sweep (F3 ADR-058 §H discharge mark + v2.0 volatile-pin removal)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): task-plan audit completed in same burst
(v1.10→v1.11) — F1 gate ordering fixed, F2 green-driver attributions corrected. COERCION-001's
F3 (ADR-058 §H discharge) has no equivalent in ROUTING-001 (ROUTING-001's §Mandate Anchor
sections use ROUTING-001 specific anchors already discharged in earlier passes). VERDICT:
ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The §ADR-058 MUST Discharge §Mandate Anchor #2 section is the authoritative prose. The
§v1.3 Amendment Sweep Dimension 3 contained a downstream copy of the same `v2.0` volatile
pin; both locations updated in this amendment. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. Marking an anchor as discharged removes a
pending obligation; it does not create a new one. VERDICT: N/A — no new mandates.

---

### v1.9 Amendment Sweep (F3 date cites 2026-08-16→2026-08-17 + modified: frontmatter)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): §Authority ADR-058 + BC-2.16.003 date cites
updated 2026-08-16 → 2026-08-17; `modified:` frontmatter added; F1 T-11H and F2 T-11P
fixed. All changes in the same pass-6 burst. VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The §Authority date cites in this story are authoring-accuracy values. No downstream artifact
copies them. The `modified:` frontmatter field is new in this amendment; it has no downstream
copy. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. VERDICT: N/A — no new mandates.

---

### v1.8 Amendment Sweep (ADR-058 v2.7→v2.8 + BC-2.16.003 v1.8→v1.9 pin sweep)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): §Authority ADR-058 pin updated v2.7→v2.8;
§Authority BC-2.16.003 pin updated v1.8→v1.9; body BC table pin updated v1.8→v1.9; F2 stale
count fix applied (Red-then-green gate `all 20` → `all 23`). All changes in the same pass-5
burst. VERDICT: ROUTING-001 AMENDED IN SAME BURST.

**Dimension 2 — Downstream copy target:**

§Authority BC-2.16.003 pin and body BC table pin are the two copies of the BC version reference
in this story. Both updated v1.8→v1.9. §Authority ADR-058 pin updated v2.7→v2.8. No other
downstream artifact in COERCION-001 copies these pins. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. VERDICT: N/A — no new mandates.

---

### v1.7 Amendment Sweep (BC-2.16.003 re-pin v1.7→v1.8 + AC↔RG coverage cross-check)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (Stage 2 sibling): ROUTING-001 is the primary amendment target
for pass-4 (RG-021/022/023 added, density 20→23, BC-2.16.003 re-pin v1.7→v1.8). COERCION-001
re-pin is a mechanical downstream-copy propagation of the same BC version bump. No COERCION-001
AC or RG needs amendment in response to the ROUTING-001 KF coverage additions (KF-05/06/07 are
Stage 2 field-routing obligations, not Stage 1 coercion-algorithm obligations). VERDICT: SWEPT;
ROUTING-001 AMENDED IN SAME BURST; COERCION-001 BC-PIN-ONLY UPDATE.

**Dimension 2 — Downstream copy target:**

BC-2.16.003 §Authority pin (from `1.7` to `1.8`) and body BC table pin both updated in this
amendment. These are the only two copies of the BC-2.16.003 version reference in COERCION-001.
No other downstream artifact in COERCION-001 copies the pin value. AC↔RG coverage cross-check:
RG-001..RG-007 cover all 6 ACs; no new RGs or ACs needed for Stage 1 scope. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The BC re-pin is a mechanical version-tracking
update, not a new behavioral obligation. VERDICT: N/A — no new mandates.

---

### v1.6 Amendment Sweep (F1 subsystem correction + F3 date correction)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (sibling story, same epic): subsystem cross-checked against
ARCH-INDEX ground truth per coordinator instruction. ROUTING-001 sets SS-01/SS-02/SS-10/SS-16
with prism-bin attributed to SS-10 (ARCH-INDEX SS-10 row: "prism-mcp, prism-bin (planned
— S-WAVE5-PREP-01)"). Confirmed correct per ARCH-INDEX. No changes required to ROUTING-001
subsystem section. VERDICT: SWEPT; CORRECT.

F3 date corrections apply only to COERCION-001 (dates I introduced in the v1.5 amendment
sweep). ROUTING-001 §Authority already cites "2026-08-16" correctly. VERDICT: ROUTING-001
UNAFFECTED.

**Dimension 2 — Downstream copy target:**

The COERCION-001 subsystem justification is authoring-only prose in frontmatter comments.
No downstream artifact copies these comments. The corrected SS-10 attribution for prism-bin
confirms the ARCH-INDEX assignment; no propagation to BC or ADR files is required (those
are read-only per coordinator constraint). VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

No new MUSTs introduced by this amendment. The subsystem correction and date correction
are authoring-accuracy fixes, not new behavioral obligations. VERDICT: N/A — no new mandates.

---

### v1.5 Amendment Sweep (F2 pin sweep + F5 BC-2.16.002 addition + F6 catalog prose fix)

**Dimension 1 — Sibling pair:**

*S-ADR058-OCSF-ROUTING-001* (sibling story, same epic): swept in full for the same
F2/F5/F6 findings. ROUTING-001 is amended in the same fix-burst (v1.5→v1.6): ADR-058
pin v2.6→v2.7, BC-2.16.003 pin v1.6→v1.7, narrative version labels stripped per POL-39,
RG-019/RG-020 wire-shape coverage added, SS-01 attribution corrected. COERCION-001's
own BC-2.16.002 addition (F5) does not apply to ROUTING-001 because ROUTING-001 already
carried BC-2.16.002 in its frontmatter from v1.5. VERDICT: SWEPT; ROUTING-001 AMENDED
IN SAME BURST.

**Dimension 2 — Downstream copy target:**

The `column_coercion_failure` catalog row content in §BC-2.16.002 Catalog Row Obligation
is the source from which the product-owner will transcribe the BC-2.16.002 row entry.
The F6 fix removes the volatile version-count phrase (`currently v1.62 with 90 events
becomes v1.63 with 91 events`) and replaces it with a section-anchor cite. The transcribed
BC-2.16.002 row itself does not contain that version-count phrase — only the routing
instruction in this story did. The downstream copy target (BC-2.16.002 §Postconditions
§Canonical Structured Event Catalog) is unchanged; the product-owner transcribes the
catalog row definition, not the routing instruction. VERDICT: CLEAR.

**Dimension 3 — Mandate anchor:**

BC-2.16.002 §Postconditions §Canonical Structured Event Catalog — `column_coercion_failure`
row addition obligation: anchored to `S-ADR058-OCSF-COERCION-001 AC-004 RG-005` and
routed to product-owner per §BC-2.16.002 Catalog Row Obligation. No unanchored MUSTs
introduced. VERDICT: DISCHARGED IN THIS AMENDMENT.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.46 | 2026-08-20 | story-writer | Human-approved in-scope expansion: Integer+Object coercion gap (EC-016-013-030, ADR-058 §H item 4). Added AC-008 (build_column_array / coerce_value handle Integer column + Value::Object: Path A null+warn; Path B Err(CoercionWarning)). Added RG-010 (`test_build_column_array_integer_type_object_input_returns_null_and_emits_warning` — prism-bin src/spec_driven_adapter.rs #[cfg(test)] mod tests, tracing_test subscriber, covers AC-008 Path A LIVE). Added RG-011 (`test_coerce_value_integer_type_object_input_returns_err_coercion_warning` — prism-spec-engine src/column_mapping.rs #[cfg(test)] mod tests, pure return-value assertion, covers AC-008 Path B). §BC-5.38.001 Density Check: RGT count 8→10, AC count 7→8, density 10/8=1.25. §ADR-058 MUST Discharge table: row 4 added (ADR-058 §H item 4 → AC-008 → RG-010/RG-011, DISCHARGED-pending-impl). §Edge Cases: EC-010 added (Integer+Object via EC-016-013-030). §Architecture Mapping: coerce_value row scope extended (Integer+Object arm); new Integer arm row; in-crate test row extended to RG-005+RG-011; Path-A test row extended to RG-006/008/009/010. §Tasks: T-10c/T-10d added (test-writer Phase A); T-15c added (implementer Phase B); T-GATE density updated 10/8=1.25; T-16/T-17 verify commands updated. §Authority: ADR-058 pin v2.25→v2.26; §H deliverables count three→four. ADR-058 §H item 4 placeholder filled (architect-pre-authorized): "AC-008 (RG-010 Path-A build_column_array Integer+Object null+warn; RG-011 Path-B coerce_value Integer+Object Err(CoercionWarning))". ADR-058 input-hash 18b74fe→ae3047b; story input-hash 638c633→006da3c (both hook-computed at edit time). Tags: ec-016-013-030 added. §v1.46 Amendment Sweep: Dim 1 (sibling) — ROUTING-001 UNAFFECTED; ADR pin refresh deferred per established pattern. Dim 2 (downstream copy) — BC-2.16.003/ADR-058 already correct (source of fix); catalog row 95 covers new trigger via existing field schema; ADR §H anchor filled; both input-hashes resolved. Dim 3 (mandate anchor) — ADR-058 §H item 4 → AC-008 → RG-010/RG-011 bidirectional; DISCHARGED-pending-impl. |
| 1.45 | 2026-08-20 | story-writer | Fix (a) §RG-001/002 SAP-3 tightening: §RG-001 note corrected — Path A serializes `Value::Array` to JSON-list string per ENRICH-1 DD-2 (EC-016-013-026), NOT demotion; stale "covered by RG-006/RG-008/RG-009" replaced with pointer to `test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string` (see §RG-007 retirement note). §RG-002 note corrected — Path A equivalent for `Value::Object` input is RG-006 only (`build_column_array` String+Object→null+warn); RG-008/RG-009 cover Integer+String and are not Object-case equivalents. Fix (b) §AC-004 note: added shipped `column_type_toml_name` helper as third consistent source alongside ADR-058 §H item 3 and BC-2.16.002 catalog row 95; explicit "all three now consistent" qualifier added. Fix (c) ADR-058 §Authority pin sweep: title and version v2.23→v2.25 (§H item 3 field-expression `%col.column_type` → `%column_type_toml_name(&col.column_type)` now matches BC-2.16.002 catalog row 95 and shipped code); status-date (2026-08-19)→(2026-08-20). Input-hash: fb7a031→638c633 (ADR-058 input updated v2.23→v2.25; recomputed by compute-input-hash hook). TD-VSDD-097 three-dimension sweep — Dim 1 (sibling pair): S-ADR058-OCSF-ROUTING-001 ADR-058 pin refresh DEFERRED to ROUTING-001's own delivery per explicit task scope; §RG-001/002 and §AC-004 content changes are COERCION-001-specific (Path B prism-spec-engine `coerce_value`/`map_record`; no ROUTING-001 mirror); VERDICT: DEFERRED (intentional). Dim 2 (downstream copy): ADR-058 §H item 3 and BC-2.16.002 catalog row 95 are already correct (they drove this fix); no further downstream copy target; VERDICT: CLEAR. Dim 3 (mandate anchor): §AC-004→ADR-058 §H item 3→RG-005 anchor in §ADR-058 MUST Discharge unchanged; no new MUST blocks; VERDICT: NO NEW UNANCHORED MUSTs. |
| 1.44 | 2026-08-20 | story-writer | F-P1-OBS-001 records-tier reconciliation: §AC-004 field-binding expressions corrected to match ADR-058 §H item 3 + BC-2.16.002 catalog row 95 + shipped `ColumnMapper::map_record` implementation. Stale `%warning.column_name` → `%col.name`; stale `%warning.expected_ocsf_type` → `%column_type_toml_name(&col.column_type)`. One-line note added citing ADR-058 §H item 3 + catalog row 95 as source-of-truth for the emission field schema. Zero code/mechanism change — no AC identity change, no RG change, no BC/pin change, no behavioral change; purely reconciling stale field-binding prose to the already-governing ADR/catalog. §v1.44 Amendment Sweep: Dim 1 (sibling pair) — ROUTING-001 has no mirror of AC-004 field-binding text; AC-004 is COERCION-specific (Path B `ColumnMapper::map_record` in `prism-spec-engine::column_mapping`; ROUTING-001 is Stage 2 field-path routing scope in prism-bin); VERDICT: NO SIBLING MIRROR. Dim 2 (downstream copy) — ADR-058 §H item 3 and catalog row 95 are already correct; this fix aligns story TO them (not vice-versa); no further downstream copy artifact to sweep; VERDICT: CLEAR. Dim 3 (mandate anchor) — AC-004 anchor to ADR-058 §H → RG-005 unchanged per §Mandate Anchor #2 table; no new MUST blocks introduced; VERDICT: UNCHANGED. |
| 1.43 | 2026-08-20 | state-manager | D-2254 SAP-1/PG-LP11-001 discharge burst (state-manager leg): BC-2.16.002 §Authority pin v2.29→v2.30 + §Behavioral Contracts table pin v2.29→v2.30 (product-owner registered catalog row 95 `column_coercion_failure` WARN in BC-2.16.002 §Postconditions §Canonical Structured Event Catalog in same burst). Input-hash updated 67f13c7→fb7a031 (BC-2.16.002 changed v2.29→v2.30; hash computed by validate-input-hash hook against develop-branch crate files + updated .factory/ specs). Sibling ROUTING-001 bumped to v1.45 (BC-2.16.002 pin v2.29→v2.30) in same burst. NOT merged — develop still @69d821be; workspace_test_count stays 5743. §v1.43 Amendment Sweep: Dimension 1 (sibling pair) — ROUTING-001 amended same burst (v1.44→v1.45 state-manager leg); CLEAR. Dimension 2 (downstream copy) — §Authority pin is terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.42 | 2026-08-19 | story-writer | RG-005 relocation to in-crate unit test (pre-TDD remove-uncertainty fix). §Red Gate Tests RG-005 bullet: added placement note — `crates/prism-spec-engine/src/column_mapping.rs #[cfg(test)] mod tests` (NOT `tests/bc_2_16_003_test.rs`); rationale: `tracing-test` default env-filter is `bc_2_16_003_test=trace` in integration test, excluding `prism_spec_engine::column_mapping` events; in-crate filter is `prism_spec_engine=trace`, which captures them. §Architecture Mapping: integration-test row split into two rows — RG-001..RG-004 in `bc_2_16_003_test.rs`; new in-crate row for RG-005 in `column_mapping.rs #[cfg(test)] mod tests`. §Tasks T-08: target file updated to `src/column_mapping.rs` in-crate block. §Tasks T-GATE: split confirmation — RG-001..004 in integration test, RG-005 in in-crate block. §Library & Framework tracing-test RG-005 row: location updated from `prism-spec-engine/tests/` to `src/column_mapping.rs #[cfg(test)] mod tests`; `no-env-filter` explicitly excluded. §File Structure Requirements: `column_mapping.rs` row notes RG-005 added to `#[cfg(test)] mod tests`; `bc_2_16_003_test.rs` row corrected to RG-001..RG-004 only. Version 1.41→1.42. §v1.42 Amendment Sweep added. No AC, RG identity, BC contract, pin, or mandate anchor change — purely test-file placement correction. |
| 1.40 | 2026-08-19 | story-writer | Leg 2 pin bump — BC-2.16.003 v1.18→v1.19 (BC-2.16.003 updated to v1.19 in Leg 1 of this burst); BC-2.16.002 v2.28→v2.29 (BC-2.16.002 updated to v2.29 in Leg 1); §Authority and §Behavioral Contracts table pins updated to current. No prose/AC/RG/mechanism change — COERCION-001 content is unaffected by BC-2.16.003 v1.19 and BC-2.16.002 v2.29 changes (the §Interpretation A stamp normalization and Leg 1 amendments are ROUTING-001 scope only). Input-hash updated 0912bc2→cabe74f (BC-2.16.003 and BC-2.16.002 updated in Leg 1). Sibling ROUTING-001 bumped to v1.44 in same burst. §v1.40 Amendment Sweep added. |
| 1.39 | 2026-08-19 | story-writer | FB-58/60 records micro-burst — sibling coordination. F-P58-LOW-001: ADR-058 §Authority status-date parenthetical corrected "(2026-08-18)"→"(2026-08-19)" (ADR-058 frontmatter `modified:` is 2026-08-19). No prose/AC/RG/mechanism change — COERCION-001 §Authority already uses clean version-free provenance form; the §B2/§I2/§J2/§D1 version-stamp normalization (F-P58-LOW-002) applies to ROUTING-001 §Authority only. Sibling ROUTING-001 bumped to v1.42 in same burst. §v1.39 Amendment Sweep added. |
| 1.38 | 2026-08-19 | story-writer | FB-55/56/57 LEG 2 — records-tier §Authority BC-2.16.003 pin completion. F-P55/56/57-MED-001: §Authority block pin was stale at Version `1.17` (should be `1.18`; §Behavioral Contracts table already at v1.18; BC frontmatter is v1.18) — corrected to Version `1.18`. §Authority modified-date parenthetical corrected "(modified 2026-08-18)"→"(modified 2026-08-19)" (BC-2.16.003 `modified:` is now 2026-08-19). This completes the §Authority half of the pin sweep that the v1.37 changelog row described but did not apply (v1.37 updated §Behavioral Contracts table only; §Authority pin remained stale at v1.17). Sibling ROUTING-001 bumped to v1.41 in same burst. §v1.38 Amendment Sweep added. |
| 1.37 | 2026-08-19 | story-writer | FB-52/53/54 LEG 3 sibling pin maintenance: ADR-058 §Authority pin v2.22→v2.23; BC-2.16.003 §Behavioral Contracts table pin v1.17→v1.18. No prose/AC/RG/mechanism change — COERCION-001 content is unaffected by those ADR-058 §Authority and BC-2.16.003 §Behavioral Contracts pin changes (§J2 unconditional/conditional raw_extensions reword and §Interpretation A v1.18 corrections are ROUTING-001 scope only). input-hash updated baeb9ab→3cbb64b (ADR-058 + BC-2.16.003 updated in Legs 1-2). §v1.37 Amendment Sweep added. |
| 1.36 | 2026-08-18 | story-writer | FB-49/51 Leg 2 sibling pin maintenance (F-P49-MED-003/F-P51-LOW-001): BC-2.16.003 §Authority pin was stale at v1.15 (should be v1.17); §Behavioral Contracts table pin was stale at v1.16 (should be v1.17). Both updated to v1.17. No prose/AC/RG/mechanism change — COERCION-001 content is unaffected by BC-2.16.003 v1.17 additions (EC-016-013-029 synthesized-name guard and EC-016-013-028 reworded source_path attribution are ROUTING-001 scope only). input-hash updated 51956ac→baeb9ab (BC-2.16.003 updated in FB-49/51 Leg 1). §v1.36 Amendment Sweep added. |
| 1.35 | 2026-08-18 | story-writer | OCSF-correctness Claroty fix-burst leg 3 sibling pin bump: ADR-058 pin v2.21→v2.22 + BC-2.16.003 pin v1.15→v1.16; no prose/AC/RG/mechanism change. input-hash updated 4cdc61e→51956ac (source documents ADR-058 + BC-2.16.003 changed by legs 1-2 of burst: EC-016-013-028, §J2 reserved-name guard, EC-016-013-011 corrected text — all Stage 2 scope, COERCION-001 content unaffected). §v1.35 Amendment Sweep added. |
| 1.34 | 2026-08-18 | state-manager | D-2243 P43/44/45 fix-burst (state-manager leg): sibling ADR-058 pin v2.20→v2.21 + BC-2.16.003 pin v1.14→v1.15; no prose/AC/RG/mechanism change. input-hash updated 759227b→4cdc61e (source documents ADR-058 + BC-2.16.003 changed in same burst). §v1.34 Amendment Sweep: Dimension 1 (sibling pair) — ROUTING-001 amended same burst (v1.36→v1.37 story-writer leg); CLEAR. Dimension 2 (downstream copy) — §Authority pins are terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.33 | 2026-08-18 | state-manager | D-2242 P40/41/42 fix-burst (state-manager leg): sibling ADR-058 pin v2.19→v2.20 + BC-2.16.003 pin v1.13→v1.14; no content change (P40/41/42 fix-burst). §v1.33 Amendment Sweep: Dimension 1 (sibling pair) — ROUTING-001 amended same burst (v1.35→v1.36 story-writer leg); CLEAR. Dimension 2 (downstream copy) — §Authority pins are terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.32 | 2026-08-18 | state-manager | D-2239 F-P34 fix-burst (state-manager leg): sibling ADR-058 pin v2.18→v2.19; no content change. §v1.32 Amendment Sweep: Dimension 1 (sibling pair) — ROUTING-001 amended same burst (v1.32→v1.33 story-writer leg); CLEAR. Dimension 2 (downstream copy) — §Authority pin is terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.31 | 2026-08-18 | state-manager | D-2238 F-P33-MED-001 fix-burst (state-manager leg): sibling ADR-058 pin v2.17→v2.18; no content change. §v1.31 Amendment Sweep: Dimension 1 (sibling pair) — ROUTING-001 amended same burst (v1.31→v1.32 story-writer leg); CLEAR. Dimension 2 (downstream copy) — §Authority pin is terminal; no independent copy artifact; CLEAR. Dimension 3 (mandate anchor) — no new MUST blocks; CLEAR. |
| 1.30 | 2026-08-18 | story-writer | OCSF-correctness Claroty SPEC pass-32 fix-burst: F-P32-MED-002 [MED, SAP-3]: SAP-3 defense-in-depth annotations added to RG-001..005 — each now carries reachability rationale stating `coerce_value`/`map_record` (Path B, `ColumnMapper::*` in `column_mapping.rs`) has zero live production callers per ADR-058 §K5; tests are intentionally defense-in-depth / forward-compat per SAP-3 rule 2/3; live Path A coercion covered by RG-006/RG-008/RG-009 (`build_column_array`). ADR-058 pin v2.16→v2.17 at §Authority (architect bump — §I1 clarified: individual-field naming for `ocsf_field == Some` only; §I2 raw_extensions routing is ROUTING-001 scope). Sibling coordination: ROUTING-001 amended same burst (v1.30→v1.31 — F-P32-MED-001 raw_extensions locus re-attribution). §v1.30 Amendment Sweep added. |
| 1.29 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-30 sibling coordination: BC-2.16.003 pin v1.12→v1.13 at §Authority and §Behavioral Contracts table (PO bump — §OCSF Field Validation Path-A/Path-B qualifier). Downstream contradiction check: Stage 1 coercion scope; no §OCSF Field Validation prose in COERCION-001; no correction needed. Sibling coordination: ROUTING-001 amended same burst (v1.28→v1.29). §v1.29 Amendment Sweep added. |
| 1.28 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-28 sibling coordination: BC-2.16.002 pin v2.27→v2.28 at §Authority entry and §Behavioral Contracts table (PO bumped BC-2.16.002 v2.28 with pending-wiring annotation on §Canonical Structured Event Catalog ocsf.unknown_class_name row). Sibling coordination: ROUTING-001 amended same burst (v1.27→v1.28 — BC-2.16.002 pin v2.27→v2.28). §v1.28 Amendment Sweep added. |
| 1.27 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-27 fix-burst: F-P27-MED-001 [MED]: §Mandate Anchor #2 rewritten — stale verbatim ADR-058 §H anchor-string copy removed (replaced with section-anchor cite: ADR-058 §H emission discharge anchor); emission MUST table expanded from 1 row to 3 rows tracing each path to its correct AC/RG: Path-B map_record → AC-004/RG-005; Path-A String+Object → AC-005/RG-006; Path-A Integer+String → AC-007/RG-009. §TD-VSDD-097 main Dimension 3 updated to cite all three AC/RG pairs and remove stale verbatim §H quote. §v1.27 Amendment Sweep added. |
| 1.26 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-26 fix-burst: F-P26-MED-001 [MED]: RG-006 extended — renamed `test_build_column_array_string_type_object_input_returns_null_and_emits_warning`; installs `tracing_test` subscriber; asserts null cell AND `column_coercion_failure` warn (`column_type = "string"`, `actual_json_kind = "object"`) — mirrors RG-009/RG-005. T-09 test name updated. §Purity Classification: RG-006 added to tracing-subscriber list (alongside RG-005/RG-009). §Library & Framework: prism-bin tracing-test row updated RG-009 → RG-006 and RG-009. §Architecture Mapping Constraints item 3: (for RG-009) → (for RG-006 and RG-009). §Architecture Mapping unit-test row: RG-006/RG-009 tracing-test note added. ADR-058 pin v2.15→v2.16 at §Authority (architect bump — §H now cites AC-005/RG-006). Sibling: ROUTING-001 amended same burst (v1.26→v1.27 — ADR-058 pin v2.16 only). §v1.26 Amendment Sweep added. |
| 1.25 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-25 fix-burst: F-P25-MED-001 [MED]: AC-005 rewritten — add explicit `Value::Object(_) => None` arm BEFORE wildcard; retain `other => Some(other.to_string())` wildcard for Number/Bool (LIVE-DRIFT-003 behavior, BC-2.16.003 §Full Coercion Matrix Path-A). T-15 rewritten to match. §Architecture Mapping `build_column_array` scope updated. Exhaustive arm order documented: Null/String/Array/Object/wildcard. ADR-058 pin v2.14→v2.15 at §Authority. BC-2.16.003 pin v1.11→v1.12 at §Authority and §Behavioral Contracts table. Sibling: ROUTING-001 amended same burst (v1.25→v1.26 — pin sweeps only). §v1.25 Amendment Sweep added. |
| 1.24 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-24 fix-burst: F-P24-HIGH-001 [HIGH]: AC-005 rewritten — Object-only null-demote; Array arm (ENRICH-1 Design Decision 2, EC-016-013-026) preserved. RG-007 (`test_build_column_array_string_type_array_input_returns_null_cell`) retired (asserts wrong behavior); ENRICH-1 array-arm coverage referenced from existing passing tests. T-15 rewritten to Object-only. EC-001 rewritten. §Architecture Mapping `build_column_array` scope updated. Density 9/7=1.29→8/7=1.14. Swept: §Red Gate header, §T-GATE, §Architecture Mapping, §Purity Classification, §File Structure Requirements, T-10a/T-10b, T-17. F-P24-MED-001 [MED]: `coerce_value` signature corrected at §Architecture Compliance Rule 1 and §Purity Classification (fabricated `&self`/`Value`/`&ColumnType` → real `pub fn coerce_value(value: &Value, column: &ColumnSpec, ocsf_field_path: &str)`). BC-2.16.003 pin v1.10→v1.11 (PO bump + EC-016-013-026 addition) at §Authority and §Behavioral Contracts body table. Sibling: ROUTING-001 amended same burst (v1.24→v1.25 — BC-2.16.003 pin sweep only). §v1.24 Amendment Sweep added. |
| 1.23 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-23 fix-burst: F-P23-MED-001 [MED, text-sync]: §Library & Framework Requirements tracing-test row for RG-009 location corrected from `prism-bin/tests/` to `crates/prism-bin/src/spec_driven_adapter.rs #[cfg(test)] mod tests` — stale text introduced in pass-19 (provisioned together with ROUTING-001 RG-018) before pass-21 relocation propagated to §Architecture Mapping, §T-GATE, and §File Structure but not to §Library & Framework. Complete-sweep grep verified: zero `prism-bin/tests/` references for private-fn RGs remain. Sibling sweep: ROUTING-001 amended in same burst (v1.23→v1.24 — F-P23-MED-001 two loci: §Library & Framework + §File Structure Cargo.toml Notes). §v1.23 Amendment Sweep added. |
| 1.22 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-22 fix-burst: ADR-058 §Authority pin v2.13→v2.14 (architect bump; `anchor_stories` gain S-ADR058-DTU-PARITY-MIGRATION-001 + §H enumeration fix). Sibling coordination with ROUTING-001 pass-22 fix-burst (v1.22→v1.23 — F-P22-MED-001/002/OBS-2/3; COERCION-001 has no equivalent ROUTING findings). §v1.22 Amendment Sweep added. |
| 1.21 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-21 fix-burst: F-P21-MED-001 [MED, compile-correctness]: §Architecture Mapping Path A row relabeled from "Integration test file / `crates/prism-bin/tests/` (file TBD)" to "Unit test block / `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`"; §T-GATE "appropriate prism-bin test file" → explicit src mod tests block; §File Structure Requirements `crates/prism-bin/tests/` row → `src/spec_driven_adapter.rs` row (direct calls to private `build_column_array` — Architecture Compliance Rule 2). Sibling sweep: ROUTING-001 amended in same burst (v1.21→v1.22 — F-P21-MED-001 `pipeline_result_to_record_batch` RGs relocated + F-P21-LOW-001 T-12/T-14 attribution). §v1.21 Amendment Sweep added. |
| 1.20 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-20 fix-burst: (1) F-P20-LOW-001b [LOW, records-tier]: §v1.18 Amendment Sweep §Dimension 1 false sentence corrected — prior text stated "ROUTING-001 amendment sweeps are already in consistent descending order within their own file"; corrected to "ROUTING-001 amendment sweep ordering was corrected in ROUTING-001 v1.21 (pass-20, F-P20-LOW-001a)". (2) F-P20-LOW-002 [LOW, POL-8 count parity]: BC-2.02.011 row added to §Token Budget Estimate (~1k); Total updated ~21k → ~22k. BC-2.02.011 was present in frontmatter `behavioral_contracts`, §Authority, and §Behavioral Contracts body table but absent from Token Budget — POL-8 count-parity violation. Sibling sweep: ROUTING-001 amended in same burst (v1.20→v1.21). §v1.20 Amendment Sweep added. |
| 1.19 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-19 fix-burst: (1) F-P19-MED-001 [MED, TDD gate coherence]: prism-bin provisioning added — §Architecture Mapping Constraints item 3 expanded to name BOTH `prism-spec-engine/Cargo.toml` (for RG-005, already present as `tracing-test = "0.2"`) and `prism-bin/Cargo.toml` (for RG-009, must be added) as permitted `tracing-test = "0.2"` `[dev-dependencies]` sites; §Library & Framework Requirements split tracing-test row into two rows (one per crate, with explicit presence/absence status); §File Structure Requirements added `crates/prism-bin/Cargo.toml` row (Modify: add `tracing-test = "0.2"` to `[dev-dependencies]`); "Do NOT modify" note updated to confirm `prism-spec-engine/Cargo.toml` needs no change. (2) F-P19-LOW-001 [LOW, TDD gate accuracy]: T-12 nextest filter `'test(rg_001)'` replaced with `just iter prism-spec-engine` — the old filter was a substring match that matched zero test names (RG-001/RG-002 test names contain `coerce_value`, not `rg_001`) and exited 0 vacuously, constituting a false-green gate. Sibling sweep: ROUTING-001 amended in same burst (v1.19→v1.20). §v1.19 Amendment Sweep added. |
| 1.18 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-18 fix-burst: (1) F-P18-MED-001 [MED, POL-8 / TDD gate coherence]: §T-GATE split to run both `just iter prism-spec-engine --no-fail-fast` (observe RG-001..RG-005 fail) and `just iter prism-bin --no-fail-fast` (observe RG-006..RG-009 fail) — prior single-crate command could not reach prism-bin RGs; T-16 split to `just iter prism-spec-engine` (RG-001..005 + AC-006) and T-17 updated to name RG-006..RG-009 as explicit pass targets for `just iter prism-bin`. (2) F-P18-OBS-001 [records-tier]: amendment-sweep subsection ordering corrected to consistent descending (newest first): v1.17→v1.16→v1.15→v1.14→v1.13→v1.12→v1.11→v1.10→v1.9→v1.8→v1.7→v1.6→v1.5. Sibling sweep: ROUTING-001 fixes applied in same burst (v1.18→v1.19). §v1.18 Amendment Sweep added. |
| 1.17 | 2026-08-17 | story-writer | OCSF-correctness Claroty SPEC pass-17 fix-burst: (1) F-P17-MED-001 [MED, POL-8 / TD-VSDD-097 dim-2 / TD-VSDD-060]: §Architecture Mapping split into separate prism-spec-engine row (RG-001..RG-005, §bc_2_16_003_test.rs) and prism-bin row (RG-006..RG-009, §prism-bin/tests); RG-006/007 crate misplacement corrected (was prism-spec-engine, now prism-bin per §T-GATE and §File Structure Requirements as source of truth); §Purity Classification extended RG-001..RG-007 → RG-001..RG-009, tracing subscriber note updated to name RG-005 and RG-009; §File Structure Requirements prism-bin/tests row updated RG-006..RG-007 → RG-006..RG-009. (2) F-P17-LOW-001 [LOW, records-tier]: §v1.16 Amendment Sweep §Dimension 1 false sentence corrected — prior text stated ROUTING-001 BC-2.16.003 pin was deferred; correct fact is ROUTING-001 §Authority BC-2.16.003 pin was propagated v1.9→v1.10 in the same D-2220 burst (no deferral). (3) F-P17-LOW-002 [LOW, structural]: duplicate "Architecture Compliance Rules" heading resolved — subordinate `###` heading (rules 1–4, under §Architecture Mapping) renamed to "Architecture Mapping Constraints"; top-level `## Architecture Compliance Rules` (rules 1–7) is now the sole instance; AC-005 "Architecture Compliance Rule 7" cite resolves correctly to the unchanged top-level section. §v1.17 Amendment Sweep added. |
| 1.16 | 2026-08-17 | story-writer | OCSF-correctness Claroty adversary SPEC pass-16 fix-burst: (1) F-P16-MED-003 [Option A, per PO adjudication]: AC-007 added (build_column_array ColumnType::Integer arm handles Value::String inputs with parse-attempt — parseable → Some(n), non-parseable → None + column_coercion_failure warn); RG-008 (test_build_column_array_integer_type_string_parseable_returns_integer) and RG-009 (test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning) added; T-10a/T-10b Red Gate authoring tasks added; T-15b implementation task added; T-16 count 7→9 RGTs; density updated 7/6=1.17→9/7=1.29. (2) F-P16-OBS-001 [records-tier, POL-7]: BC-2.16.003 §Authority title expanded to full H1 verbatim. (3) BC-2.16.003 pin propagated v1.9→v1.10 at §Authority + §Behavioral Contracts body table. (4) input-hash updated (BC-2.16.003 input bumped to v1.10 by PO in same burst). Sibling sweep: ROUTING-001 amended in same burst (v1.17→v1.18). §v1.16 Amendment Sweep added. |
| 1.15 | 2026-08-17 | story-writer | Adversary pass-12 fix-burst: (1) ADR-058 §Authority pin v2.12→v2.13 (concurrent architect bump). (2) Sibling coordination: ROUTING-001 F2 §Tasks T-11G/H/L/M/N/O authoring-wording fixed in same burst; COERCION-001 tasks audit confirmed CLEAN (all tasks code-level, no TOML-load/inline-spec inconsistency). (3) Sibling sweep: zero normative prose version pins found in either story. (4) §v1.15 Amendment Sweep added. |
| 1.14 | 2026-08-17 | story-writer | Adversary pass-11 fix-burst: (1) LOW-2 [LOW, POL-8] AC-004 trace parentheticals added — `(traces to BC-2.16.002 §Canonical Structured Event Catalog ...)` and `(traces to BC-2.02.011 §Graceful Normalization Error Handling (No Silent Data Loss) ...)` added after existing BC-2.16.003 trace; all three frontmatter BCs (BC-2.16.003, BC-2.02.011, BC-2.16.002) now have at least one AC `(traces to …)` parenthetical (POL-8 no-orphan satisfied). (2) ADR-058 §Authority pin v2.11→v2.12 (concurrent architect bump). (3) Sibling sweep: zero ADR-058/BC normative prose version pins found in either story outside §Authority (exempt) and historical amendment-sweep/changelog rows (grandfathered). (4) §v1.14 Amendment Sweep added. |
| 1.13 | 2026-08-17 | story-writer | Adversary pass-10 fix-burst: (1) ADR-058 §Authority pin v2.10→v2.11 (concurrent architect bump). (2) Sibling sweep coordination: ROUTING-001 F3 §Mandate Anchor #1 provenance fix applied in same burst — both §D2 and §J2 version qualifiers "(v2.1)" / "since v2.1" stripped; COERCION-001 §Mandate Anchor #2 was already version-free since pass-7. Zero additional POL-39 violations found in COERCION-001 normative prose. (3) §v1.13 Amendment Sweep added. |
| 1.12 | 2026-08-17 | story-writer | Adversary pass-9 fix-burst: (1) ADR-058 §Authority pin v2.9→v2.10 (concurrent architect bump). (2) Sibling sweep coordination: ROUTING-001 F2 AC-011 prose pin cleaned in same burst. Zero additional POL-39 violations found in COERCION-001 normative prose. (3) §v1.12 Amendment Sweep added. |
| 1.11 | 2026-08-17 | story-writer | Adversary pass-8 fix-burst: (1) ADR-058 §Authority pin v2.8→v2.9 (concurrent architect bump). (2) Comprehensive hygiene sweep: zero POL-39 narrative prose violations and zero line-cites found outside §Authority and historical amendment-sweep sections (all grandfathered per TD-VSDD-091 ratchet scoping). (3) §v1.11 Amendment Sweep added. |
| 1.10 | 2026-08-17 | story-writer | Adversary pass-7 fix-burst: (1) F3 ADR-058 §H MUST Discharge section updated: ANCHOR-NEEDED marked DISCHARGED (ADR-058 §H already reads "(Anchored: S-ADR058-OCSF-COERCION-001 AC-004, RG-005)" since v2.1); volatile `v2.0` pin removed from both the §Mandate Anchor #2 prose and the §v1.3 Amendment Sweep Dimension 3; `ANCHOR-NEEDED` language replaced with `DISCHARGED`; architect routing obligation removed. (2) §v1.10 Amendment Sweep added. |
| 1.9 | 2026-08-17 | story-writer | Adversary pass-6 fix-burst: (1) F3 §Authority date cites ADR-058 + BC-2.16.003 updated 2026-08-16 → 2026-08-17; `modified:` frontmatter field added as 2026-08-17. (2) §v1.9 Amendment Sweep added. |
| 1.8 | 2026-08-17 | story-writer | Adversary pass-5 fix-burst: (1) ADR-058 re-pin v2.7→v2.8; BC-2.16.003 re-pin v1.8→v1.9 (concurrent architect/PO bumps); §Authority pins and body BC table updated. (2) §v1.8 Amendment Sweep added. |
| 1.7 | 2026-08-17 | story-writer | Adversary pass-4 fix-burst: (1) BC-2.16.003 re-pin v1.7→v1.8 (PO concurrent bump); §Authority pin and body BC table updated. (2) Quick AC↔RG coverage cross-check: RG-001..RG-007 cover all 6 ACs (AC-001..AC-006); CLEAN — no gaps found. Stage 1 coercion-engine scope does not include KF-05/06/07 field-routing corrections (those are Stage 2 ROUTING-001 territory). (3) §v1.7 Amendment Sweep added. |
| 1.6 | 2026-08-17 | story-writer | Adversary pass-3 fix-burst: (1) F1 subsystem mis-anchoring corrected: `prism-bin` removed from SS-01 justification (fabricated per POL-5 — ARCH-INDEX SS-01 lists `prism-sensors, prism-spec-engine, prism-dtu-*`; NOT prism-bin); SS-10 added to frontmatter and justified as owner of prism-bin (ARCH-INDEX SS-10 row: "prism-mcp, prism-bin (planned — S-WAVE5-PREP-01)"); SS-22 excluded (boot orchestration only, not data-processing scope). SS-01 justification now cites only prism-spec-engine with ARCH-INDEX SS-01 row verbatim excerpt. (2) F3 §Authority date corrections: BC-2.16.003 `modified 2026-08-17` → `2026-08-16`; ADR-058 `accepted (2026-08-17)` → `(2026-08-16)` (cite on-disk frontmatter dates per POL-37). (3) §v1.6 Amendment Sweep added. |
| 1.5 | 2026-08-17 | story-writer | Adversary pass-2 fix-burst: (1) F2 BC-2.16.003 pin v1.5→v1.7; ADR-058 pin v2.6→v2.7; narrative version labels stripped per POL-39 (section-anchor-only cites in §Authority BC-2.16.003 note and ADR-058 process-gap phrase). (2) F5 BC-2.16.002 added to `behavioral_contracts:` frontmatter; BC-2.16.002 v2.27 body BC table row added; BC-2.16.002 v2.27 §Authority entry added (POL-8 full propagation). Token Budget already carried BC-2.16.002 ~2k row from v1.3; no token budget change needed. (3) F6 §Catalog Row Obligation stale version prose replaced with section-anchor cite per POL-39 (removed `currently v1.62 with 90 events becomes v1.63 with 91 events`). (4) §v1.5 Amendment Sweep added. |
| 1.4 | 2026-08-16 | story-writer | Adversary pass-1 fix-burst: (1) Subsystems [SS-07, SS-16] → [SS-01, SS-16]; removed fabricated SS-07 citation ("Spec Engine" — SS-07 is Adapter Pagination & Response Cache per ARCH-INDEX); correct citations per ARCH-INDEX: SS-01 (Sensor Adapters, owns prism-bin/spec_driven_adapter.rs + prism-spec-engine), SS-16 (Spec Engine, owns prism-spec-engine/column_mapping.rs). (2) §Authority ADR-058 pin v2.5→v2.6 with note that §I5 v2.6 process-gap obligation and §K (class_selector.rs obligations) are Stage 2 scope only; Stage 1 §H scope unchanged. |
| 1.3 | 2026-08-16 | story-writer | ADR-058 §K pin sweep: §Authority pin v2.4→v2.5; narrative prose v2.4 version label removed per POL-39 (section-anchor-only cites). |
| 1.2 | 2026-08-16 | story-writer | BC-2.16.003 v1.4→v1.5 version pin propagation (TD-VSDD-097 dim-2 downstream copy target). Authority section: BC-2.16.003 version updated to v1.5 (modified 2026-08-16). Behavioral Contracts table: BC-2.16.003 v1.4→v1.5. ADR-058 version pin in Authority section updated to v2.4 (modified 2026-08-16). No substantive scope change — Stage 1 coercion algorithm (Rules 1/2/3, EC-016-013-007/008/009 gap closures, column_coercion_failure emission) is unchanged in BC-2.16.003 v1.5; new v1.5 content (§Interpretation A, §Claroty Contracted OCSF Mappings) is Stage 2 territory. |
| 1.1 | 2026-08-12 | story-writer | Remove-uncertainty pass: AC-005 strengthened with wire-level null serialization note citing existing `WriterBuilder::with_explicit_nulls(true)` chokepoint in `prism-mcp::server` §RecordBatch-to-JSON and `bc_2_11_001_null_row_shape_test.rs` regression. Architecture Compliance Rule 7 added: no second RecordBatch→JSON emit path. Q4 CORRECTED (stale concern — `explicit_nulls` already set correctly in production). |
| 1.0 | 2026-08-12 | story-writer | Initial authorship — ADR-058 Stage 1 story. Fixes EC-016-013-007/008/009 (coerce_value String + Array/Object, coerce_value Integer + String non-numeric path), adds column_coercion_failure tracing emission in map_record and build_column_array. Discharges ADR-058 v2.0 §H ANCHOR-NEEDED mandate for the emission MUST. BC-2.16.003 v1.4, BC-2.02.011, ADR-058 v2.0 at authoring time. |
