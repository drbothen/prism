---
document_type: story
story_id: S-ADR058-OCSF-COERCION-001
title: "ADR-058 Stage 1 — Column Coercion Gap Closure: EC-016-013-007/008/009 Fixes and column_coercion_failure Tracing Emission"
version: "1.6"
level: "L4"
status: draft
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
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
holdout_scenarios: []
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
input-hash: "0d79865"
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
  - column_coercion_failure
  - claroty-live
---

# S-ADR058-OCSF-COERCION-001: ADR-058 Stage 1 — Column Coercion Gap Closure

## Authority

**BC-2.16.003: Column-to-OCSF Mapping at Query Time.** Version `1.7`, status: draft
(modified 2026-08-16). Primary behavioral authority. The §Type Coercion Algorithm, §Full
Coercion Matrix, EC-016-013-007/008/009 KNOWN GAP annotations, and §Coercion Warning
Observability DEFECT section are the acceptance-criteria source for this story. Note: BC-2.16.003
§Interpretation A (Arrow field naming) and §Claroty Contracted OCSF Mappings are Stage 2
territory and do not change Stage 1's scope.
Path: `.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**ADR-058 v2.7: v1 Column Naming — OCSF Field-Path Routing.** Version `2.7`, status:
accepted (2026-08-16). §H (Stage 1 Scope) enumerates the three deliverables this story
implements: EC-016-013-008 fix in `build_column_array`, EC-016-013-009 fix via
`ColumnMapper::coerce_value` integration, and `column_coercion_failure` tracing emission.
Note: ADR-058 §K (OCSF schema validation), §I5 (code obligations), and process-gap
obligation (`ocsf.unknown_class_name` WARN) affect Stage 2 scope only; Stage 1's §H scope
is unchanged.
Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable
Interpolation.** Version `2.27`, status: active (unchanged). Governs the Canonical
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

**ADR-058 v2.0 §H carries an `ANCHOR-NEEDED` annotation (TD-VSDD-097 dim-3 obligation):**
> "MUST to add `column_coercion_failure` emission (Stage 1 story ID unconfirmed)"

**This story discharges that mandate.** The mandate anchor is:

| MUST Statement | Story | AC | Red Gate Test |
|---|---|---|---|
| `column_coercion_failure` tracing emission MUST be added to `build_column_array` at demotion point (ADR-058 §H item 3) | S-ADR058-OCSF-COERCION-001 | AC-004 | RG-005 |

**Architect routing obligation:** After this story reaches `status: ready`, the architect
MUST update ADR-058 v2.0 §H to replace the `ANCHOR-NEEDED` annotation with:
`(Anchored: S-ADR058-OCSF-COERCION-001 AC-004 RG-005)`.

---

## Behavioral Contracts

| BC | Version | Status | Relevance |
|----|---------|--------|-----------|
| BC-2.16.003 | v1.7 | draft | Primary contract — §Type Coercion Algorithm, §Full Coercion Matrix, EC-016-013-007/008/009 KNOWN GAPs, §Coercion Warning Observability DEFECT |
| BC-2.02.011 | — | — | Warning-emission obligation for each normalization issue; BC-2.16.003 DEFECT violates this |
| BC-2.16.002 | v2.27 | active | Canonical Structured Event Catalog obligation — `column_coercion_failure` emit from AC-004/AC-005 must be registered in §Postconditions §Canonical Structured Event Catalog (SAP-1 / PG-LP11-001) |

---

## Red Gate Tests (SAC-1 — tdd_mode: strict)

All seven tests MUST be failing (RED) before any implementation code is written. The
test-writer is dispatched FIRST; implementer is dispatched only after all 7 are confirmed
failing with the correct compile-or-test-failure reason.

- **RG-001:** `test_coerce_value_string_type_array_input_returns_err_coercion_warning` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for String column + Array input
  (currently returns `Ok(Array)` pass-through). Covers AC-001.

- **RG-002:** `test_coerce_value_string_type_object_input_returns_err_coercion_warning` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for String column + Object input
  (currently returns `Ok(Object)` pass-through). Covers AC-002.

- **RG-003:** `test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number` —
  fails until `coerce_value` parses `"42"` as `Value::Number(42)` for Integer column on
  non-numeric OCSF path (currently returns `Ok(String("42"))` pass-through). Covers AC-003.

- **RG-004:** `test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err` —
  fails until `coerce_value` returns `Err(CoercionWarning)` for Integer column + String
  `"not-a-number"` on non-numeric OCSF path (currently returns `Ok(String)` pass-through).
  Covers AC-003.

- **RG-005:** `test_map_record_string_object_input_demotes_to_raw_extensions_and_emits_warning` —
  fails until (a) `map_record` places Object-valued String-column field in `raw_extensions`
  and (b) a `tracing::warn!(event_type = "column_coercion_failure")` event is emitted.
  Requires a `tracing_test` subscriber in the test. Covers AC-004.

- **RG-006:** `test_build_column_array_string_type_object_input_returns_null_cell` —
  fails until `build_column_array` ColumnType::String arm returns `None` (null cell) for
  `Value::Object` input (currently returns `Some(Value::String("{...}"))` via wildcard
  `other => other.to_string()`). Covers AC-005.

- **RG-007:** `test_build_column_array_string_type_array_input_returns_null_cell` —
  fails until `build_column_array` ColumnType::String arm returns `None` (null cell) for
  `Value::Array` input. Covers AC-005.

### BC-5.38.001 Density Check

Red Gate test count: **7** (RG-001..RG-007).
Acceptance criteria directly driven by Red Gate tests: 5 (AC-001 through AC-005).
AC-006 is a non-regression criterion for already-passing tests — it does not require a
new failing Red Gate test.

Density: 7 RGTs / 6 ACs = **1.17 ≥ 0.5** — compliant with BC-5.38.001.

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
- `column = %warning.column_name` — the column identifier from `ColumnSpec`
- `column_type = %warning.expected_ocsf_type` — the declared TOML column type
- `actual_json_kind = %actual_kind` — the JSON kind that triggered demotion ("array",
  "object", or "string")

This closes the DEFECT documented in BC-2.16.003 §Coercion Warning Observability.
This event MUST be registered in BC-2.16.002 §Postconditions Canonical Structured Event
Catalog before the implementing PR merges (SAP-1 / PG-LP11-001 obligation — see §BC-2.16.002
Catalog Row Obligation below).

(traces to BC-2.16.003 §Coercion Warning Observability DEFECT: "The current implementation
does NOT emit a tracing::warn! at the point of demotion. This violates BC-2.02.011.")

### AC-005: build_column_array ColumnType::String arm returns null cell for Object/Array input

`build_column_array` in `prism-bin::spec_driven_adapter` ColumnType::String arm returns
`None` (null cell, not a stringified JSON dump) when the raw record value is
`Value::Array` or `Value::Object`. The current wildcard fallback
(`other => other.to_string()`) is replaced with an explicit arm that:
1. Returns `None` (null cell in the Arrow column), and
2. Emits `tracing::warn!(event_type = "column_coercion_failure", column = %col.name,
   column_type = "string", actual_json_kind = %kind)`.

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

(traces to BC-2.16.003 postcondition §Full Coercion Matrix EC-016-013-007/008 and
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
| `ColumnMapper::coerce_value` | `prism-spec-engine::column_mapping` | Pure | Modified: add `Err(CoercionWarning)` arms for Array, Object inputs on String column; extend Integer+String to non-numeric path |
| `ColumnMapper::map_record` | `prism-spec-engine::column_mapping` | Pure | Modified: add `tracing::warn!(event_type = "column_coercion_failure")` at demotion point |
| `build_column_array` (ColumnType::String arm) | `prism-bin::spec_driven_adapter` | Pure (data transformation) | Modified: replace wildcard `other => other.to_string()` with explicit null-cell arms for Array and Object; add tracing emission |
| Integration test file | `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` | Pure (tests) | New tests RG-001..RG-007 added |

Architecture section files: `architecture/module-decomposition.md` (SS-01, SS-16).

---

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| `ColumnMapper::coerce_value` | Pure | Takes `Value` + OCSF path + `ColumnType`, returns `Result<Value, CoercionWarning>`; no I/O, no mutation |
| `ColumnMapper::map_record` | Pure (data transformation) + side-effecting (tracing) | The tracing emission added by AC-004 is a side effect; the function's return value (`MappingResult`) is deterministic given the inputs |
| `build_column_array` (ColumnType::String arm) | Pure (data transformation) + side-effecting (tracing) | Returns `Option<Value>` deterministically; tracing emission added by AC-005 is a side effect |
| RG-001..RG-007 test functions | Pure (test assertions) | Test assertions over in-memory data structures; tracing subscriber in RG-005 is test infrastructure, not production I/O |

---

### Architecture Compliance Rules

From `architecture/module-decomposition.md` and ADR-023:

1. `prism-bin::spec_driven_adapter` MUST NOT take a hard dependency on
   `prism-spec-engine` types beyond what it already imports. The
   `build_column_array` fix uses only `ColumnType` (already imported) and
   `serde_json::Value` (already used) — no new type imports are needed.

2. The `tracing` crate is already a workspace dependency; no new crate additions
   are required.

3. `tracing_test` (or equivalent subscriber setup) MUST be declared as a
   `[dev-dependency]` in `prism-spec-engine/Cargo.toml` ONLY if it is not already
   present. Do NOT add `tracing_test` to `[dependencies]` (production). Verify
   before adding.

4. ADR-058 §H item 1 is specific: the `build_column_array` fix MUST use the
   null-cell approach (`None`) and NOT stringify the object/array. Stringifying
   preserves data in a wrong format; null-cell forces the operator to use
   `raw_extensions` (correct behavior).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `column_type = "string"` + `Value::Array([])` (empty array) | Returns `Err(CoercionWarning)` — empty array is still an Array; null-cell in Arrow |
| EC-002 | `column_type = "string"` + `Value::Object({})` (empty object) | Returns `Err(CoercionWarning)` — empty object is still an Object; null-cell in Arrow |
| EC-003 | `column_type = "integer"` + `Value::String("0")` + non-numeric path | Parses as `Value::Number(0)` — valid i64; returned as Ok; no demotion |
| EC-004 | `column_type = "integer"` + `Value::String("")` + non-numeric path | `"".parse::<i64>()` fails → `Err(CoercionWarning)` |
| EC-005 | `column_type = "string"` + `Value::Null` | Rule 1 pass-through unchanged (EC-016-013-006) — null is NOT Array or Object; not affected by this story |
| EC-006 | `column_type = "string"` + `Value::Number(132)` | Rule 1 unchanged (EC-016-013-004) — Number is already handled; not affected |
| EC-007 | `column_type = "string"` + `Value::Bool(true)` | Rule 1 unchanged — Bool is already handled; not affected |
| EC-008 | `column_type = "integer"` + `Value::String("42")` on NUMERIC-suffix path | EC-016-013-002 behavior unchanged — this is Rule 2 territory; not affected by this story |
| EC-009 | `build_column_array` ColumnType::Integer + Value::String on non-numeric path | Should behave consistently with `coerce_value` fix: attempt parse, return null cell on failure |

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
| `bc_2_16_003_test.rs` (existing tests to preserve) | ~2k |
| Tool outputs (just iter, cargo nextest) | ~1k |
| **Total** | **~21k** |

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
  with `tracing_test` subscriber (MUST FAIL before T-14)
- T-09: Write RG-006 — `test_build_column_array_string_type_object_input_returns_null_cell`
  (MUST FAIL before T-15)
- T-10: Write RG-007 — `test_build_column_array_string_type_array_input_returns_null_cell`
  (MUST FAIL before T-15)
- T-GATE: Run `just iter prism-spec-engine --no-fail-fast` — confirm RG-001..RG-007 fail
  with expected compile/test-failure reasons; confirm RG-001..RG-005 are in
  `bc_2_16_003_test.rs`; confirm RG-006..RG-007 are in an appropriate prism-bin test file.
  Confirm AC-006 tests still PASS. Report density: 7/6 = 1.17 ≥ 0.5. STOP and wait for
  implementer dispatch.

### Phase B: Implementation (implementer dispatched AFTER T-GATE)

- T-11: Fix `coerce_value` String branch — add explicit `Value::Array` arm and
  `Value::Object` arm returning `Err(CoercionWarning)` (AC-001, AC-002). Makes RG-001
  and RG-002 green.
- T-12: Run `cargo nextest run -p prism-spec-engine -E 'test(rg_001)' --no-fail-fast` —
  verify RG-001 and RG-002 pass.
- T-13: Fix `coerce_value` Integer branch — extend parse-attempt logic to non-numeric-suffix
  OCSF paths. When `column_type = Integer` and input is `Value::String`, attempt
  `s.parse::<i64>()` regardless of OCSF path suffix. (AC-003). Makes RG-003 and RG-004
  green.
- T-14: Add `tracing::warn!(event_type = "column_coercion_failure", column = ...,
  column_type = ..., actual_json_kind = ...)` in `ColumnMapper::map_record` at the
  demotion point (where CoercionWarning is converted to raw_extensions placement).
  (AC-004). Makes RG-005 green.
- T-15: Fix `build_column_array` String arm — replace wildcard `other => other.to_string()`
  with explicit arms for `Value::Array` and `Value::Object` returning `None` and emitting
  `tracing::warn!(event_type = "column_coercion_failure", ...)`. (AC-005). Makes RG-006
  and RG-007 green.
- T-16: Run `just iter prism-spec-engine` — all 7 RGTs must pass. AC-006 tests must pass.
  No regressions.
- T-17: Run `just iter prism-bin` — build_column_array changes must not break existing
  prism-bin tests.
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

1. `coerce_value` MUST remain a pure function (`fn coerce_value(&self, value: Value, ocsf_path: &str, column_type: &ColumnType) -> Result<Value, CoercionWarning>`). No I/O or mutation in the coerce_value body.

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
| `tracing-test` (or `tracing-subscriber` test harness) | Capture `tracing` events in RG-005 | Verify current `[dev-dependencies]` in `prism-spec-engine/Cargo.toml` before adding; if already present, use existing version |
| `serde_json` | `Value` type for coerce_value inputs | Workspace-pinned version |

Do NOT add `tracing-test` to production `[dependencies]`. Do NOT use `tracing-test` in
non-test code paths.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/column_mapping.rs` | Modify: `coerce_value` String branch, `coerce_value` Integer branch, `map_record` demotion point | Primary implementation file |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify: `build_column_array` ColumnType::String arm wildcard → explicit Array/Object arms | Secondary implementation file |
| `crates/prism-spec-engine/tests/bc_2_16_003_test.rs` | Modify or create: add RG-001..RG-005 test functions | Test file; create if not present |
| `crates/prism-bin/tests/` (actual file TBD at dispatch) | Modify: add RG-006..RG-007 test functions | Verify existing test file names via `find crates/prism-bin/tests -name "*.rs"` at dispatch |

Do NOT modify: any TOML sensor spec file; any BC or ADR file (product-owner / architect
scope); `prism-spec-engine/Cargo.toml` unless `tracing-test` must be added as
`[dev-dependency]`.

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

**Dimension 3 — Mandate anchor:** ADR-058 v2.0 §H carries `ANCHOR-NEEDED` for the
`column_coercion_failure` emission MUST. This story's §ADR-058 MUST Discharge section
anchors it to AC-004 / RG-005. The architect must update ADR-058 to replace
`ANCHOR-NEEDED` with the story/AC/RG reference. VERDICT: DISCHARGED IN THIS STORY
(ADR update required by architect — reported above).

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

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.6 | 2026-08-17 | story-writer | Adversary pass-3 fix-burst: (1) F1 subsystem mis-anchoring corrected: `prism-bin` removed from SS-01 justification (fabricated per POL-5 — ARCH-INDEX SS-01 lists `prism-sensors, prism-spec-engine, prism-dtu-*`; NOT prism-bin); SS-10 added to frontmatter and justified as owner of prism-bin (ARCH-INDEX SS-10 row: "prism-mcp, prism-bin (planned — S-WAVE5-PREP-01)"); SS-22 excluded (boot orchestration only, not data-processing scope). SS-01 justification now cites only prism-spec-engine with ARCH-INDEX SS-01 row verbatim excerpt. (2) F3 §Authority date corrections: BC-2.16.003 `modified 2026-08-17` → `2026-08-16`; ADR-058 `accepted (2026-08-17)` → `(2026-08-16)` (cite on-disk frontmatter dates per POL-37). (3) §v1.6 Amendment Sweep added. |
| 1.5 | 2026-08-17 | story-writer | Adversary pass-2 fix-burst: (1) F2 BC-2.16.003 pin v1.5→v1.7; ADR-058 pin v2.6→v2.7; narrative version labels stripped per POL-39 (section-anchor-only cites in §Authority BC-2.16.003 note and ADR-058 process-gap phrase). (2) F5 BC-2.16.002 added to `behavioral_contracts:` frontmatter; BC-2.16.002 v2.27 body BC table row added; BC-2.16.002 v2.27 §Authority entry added (POL-8 full propagation). Token Budget already carried BC-2.16.002 ~2k row from v1.3; no token budget change needed. (3) F6 §Catalog Row Obligation stale version prose replaced with section-anchor cite per POL-39 (removed `currently v1.62 with 90 events becomes v1.63 with 91 events`). (4) §v1.5 Amendment Sweep added. |
| 1.4 | 2026-08-16 | story-writer | Adversary pass-1 fix-burst: (1) Subsystems [SS-07, SS-16] → [SS-01, SS-16]; removed fabricated SS-07 citation ("Spec Engine" — SS-07 is Adapter Pagination & Response Cache per ARCH-INDEX); correct citations per ARCH-INDEX: SS-01 (Sensor Adapters, owns prism-bin/spec_driven_adapter.rs + prism-spec-engine), SS-16 (Spec Engine, owns prism-spec-engine/column_mapping.rs). (2) §Authority ADR-058 pin v2.5→v2.6 with note that §I5 v2.6 process-gap obligation and §K (class_selector.rs obligations) are Stage 2 scope only; Stage 1 §H scope unchanged. |
| 1.3 | 2026-08-16 | story-writer | ADR-058 §K pin sweep: §Authority pin v2.4→v2.5; narrative prose v2.4 version label removed per POL-39 (section-anchor-only cites). |
| 1.2 | 2026-08-16 | story-writer | BC-2.16.003 v1.4→v1.5 version pin propagation (TD-VSDD-097 dim-2 downstream copy target). Authority section: BC-2.16.003 version updated to v1.5 (modified 2026-08-16). Behavioral Contracts table: BC-2.16.003 v1.4→v1.5. ADR-058 version pin in Authority section updated to v2.4 (modified 2026-08-16). No substantive scope change — Stage 1 coercion algorithm (Rules 1/2/3, EC-016-013-007/008/009 gap closures, column_coercion_failure emission) is unchanged in BC-2.16.003 v1.5; new v1.5 content (§Interpretation A, §Claroty Contracted OCSF Mappings) is Stage 2 territory. |
| 1.1 | 2026-08-12 | story-writer | Remove-uncertainty pass: AC-005 strengthened with wire-level null serialization note citing existing `WriterBuilder::with_explicit_nulls(true)` chokepoint in `prism-mcp::server` §RecordBatch-to-JSON and `bc_2_11_001_null_row_shape_test.rs` regression. Architecture Compliance Rule 7 added: no second RecordBatch→JSON emit path. Q4 CORRECTED (stale concern — `explicit_nulls` already set correctly in production). |
| 1.0 | 2026-08-12 | story-writer | Initial authorship — ADR-058 Stage 1 story. Fixes EC-016-013-007/008/009 (coerce_value String + Array/Object, coerce_value Integer + String non-numeric path), adds column_coercion_failure tracing emission in map_record and build_column_array. Discharges ADR-058 v2.0 §H ANCHOR-NEEDED mandate for the emission MUST. BC-2.16.003 v1.4, BC-2.02.011, ADR-058 v2.0 at authoring time. |
