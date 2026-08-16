---
document_type: story
story_id: S-CLAROTY-AUDITLOG-TIMEBOX-001
title: "Claroty audit_logs time-filter push-down with bounded default (spec_driven_adapter + pipeline + TOML)"
level: "L4"
wave: claroty-live
epic_id: E-DTU-FIDELITY
priority: P1
status: draft
producer: story-writer
timestamp: "2026-08-15T00:00:00Z"
version: "2.1"
modified: "2026-08-15"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-dtu-claroty/src/types.rs"
input-hash: "05991ba"
traces_to: "BC-2.01.013"
points: 8
estimated_days: 2
tdd_mode: strict
subsystems: [SS-22, SS-16]
# Subsystem anchor justifications:
#   SS-22 (Process Lifecycle, prism-bin) owns `crates/prism-bin/src/spec_driven_adapter.rs §spec_driven_adapter`.
#     The Claroty audit_logs `filter_by` injection block lives here — the spec-driven adapter
#     translates ADR-033 Option T1 extracted `start_time`/`end_time` into a Claroty-native
#     `filter_by` JSON object and writes it into `query_filters["_claroty_audit_filter_by"]`.
#     Pattern mirrors CrowdStrike-FQL and Armis-AQL injection (ADR-033) per the same file.
#   SS-16 (Spec Engine, prism-spec-engine) owns `crates/prism-spec-engine/src/pipeline.rs §pipeline`.
#     The `step_vars` seeding extension auto-parses JSON-object/array filter strings (leading
#     `{`/`[`) into `serde_json::Value` for verbatim body insertion via
#     `${query.filter._claroty_audit_filter_by}`. Backward-compat: FQL/AQL strings stay `Value::String`.
target_module: prism-bin
crates_touched: [prism-bin, prism-spec-engine, prism-sensors]
# crates_touched:
#   prism-bin: spec_driven_adapter.rs — Claroty audit_logs filter_by injection block
#   prism-spec-engine: pipeline.rs — JSON-object auto-parse in step_vars seeding
#   prism-sensors: claroty.sensor.toml — audit_logs body_template updated to Layer-2 variable
behavioral_contracts:
  - BC-2.01.013
  # BC-2.01.013 (v1.21) §Postconditions Per-sensor push-down translation table —
  # `Claroty audit_logs` row: four filter cases (EC-01-030..EC-01-033); filter_by JSON-object
  # injection via spec_driven_adapter.rs; auto-parse in pipeline.rs; ISO-8601 value strings;
  # `operands` compound key; ApiQueryFilter DTU ground-truth.
  - BC-2.16.013
  # BC-2.16.013 (v1.39) §Postconditions §1 — Claroty audit_logs push-down block:
  # body_template = '{"filter_by": ${query.filter._claroty_audit_filter_by}}'
  # SPEC-GATE (S-7.01): both BCs are active canonical IDs — status may advance to ready.
verification_properties:
  - VP-148
  # VP-148 = VP-PLUGIN-003 DTU parity. The Claroty audit_logs pipeline path must still
  # produce spec-correct output after the body_template variable interpolation change.
depends_on: []
blocks: []
acceptance_criteria_count: 7
risk: MEDIUM
# Risk justification:
#   Three code sites must change in coordination (spec_driven_adapter.rs, pipeline.rs, TOML).
#   The pipeline.rs JSON-auto-parse is a new deserialization path affecting all sensors'
#   step_vars seeding — backward-compat must be verified against FQL/AQL strings (RG-004).
#   The filter_by JSON structure must match what xDome accepts per `ApiQueryFilter`
#   (`HashMap<String, serde_json::Value>`, `crates/prism-dtu-claroty/src/types.rs::ApiQueryFilter`)
#   DTU ground-truth (SAP-2). The 4xx error-surface path (RG-005) requires structured E-SENSOR-001
#   propagation, not a panic or silent Vec::new().
assumption_validations: [ASM-CLAROTY-AUDITLOG-001]
# ASM-CLAROTY-AUDITLOG-001 (re-scoped post remove-uncertainty): the filter_by field name
# on GetAuditLogParameters is confirmed `timestamp`; operations `greater_or_equal`/`less_or_equal`
# were validated by research. ASM is retained to gate on live field-name confirmation at demo
# prep via one-line live check before story ships.
risk_mitigations: []
holdout_scenarios:
  [HS-AUDITLOG-001-A-001, HS-AUDITLOG-001-A-002, HS-AUDITLOG-001-A-003, HS-AUDITLOG-001-A-004]
---

# S-CLAROTY-AUDITLOG-TIMEBOX-001: Claroty Audit Logs Time-Filter Push-Down with Bounded Default

## Authority

BC-2.01.013 v1.21 §Postconditions Per-sensor push-down translation table is the governing
behavioral contract for `spec_driven_adapter.rs` and `pipeline.rs` changes. Read the
`Claroty audit_logs` row specifically — it enumerates all four filter cases (EC-01-030 through
EC-01-033), the ISO-8601 `value` string requirement, the `operands` compound key (NOT
`conditions`), and the `ApiQueryFilter` DTU ground-truth.

BC-2.16.013 v1.39 §Postconditions §1 is the governing contract for the `claroty.sensor.toml`
body_template change. Read the `audit_logs` bounded push-down block — it enumerates the four
filter cases and confirms the `operands` key for the compound filter.

ADR-033 §Decision is the governing architecture decision for push-down time-window extraction
(Option T1 pre-fan-out heuristic). The CrowdStrike-FQL and Armis-AQL push-down blocks already
in `crates/prism-bin/src/spec_driven_adapter.rs §spec_driven_adapter` are the canonical pattern
to mirror for the Claroty `filter_by` injection. Read those blocks before implementing.

`ApiQueryFilter` (`HashMap<String, serde_json::Value>`) in
`crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` is the DTU ground-truth for `filter_by`
field names and operation names (SAP-2 compliance per ADR-028 §D1). `ClarotyAuditLogFilter`
does NOT exist in the codebase — it is a phantom type reference (BC-2.01.013 v1.21
§Postconditions). ASM-CLAROTY-AUDITLOG-001 must be confirmed at demo prep.

---

## Narrative

As a SOC analyst issuing PrismQL queries against Claroty xDome audit logs,
I want time-range filters in my WHERE clause to be pushed down into the xDome `filter_by` POST body (with a 7-day bounded default when no filter is given),
so that unfiltered queries return a bounded recent window promptly (no E-QUERY-004 timeout) and explicit time filters for older data are honored exactly — not silently capped.

## Background

`claroty.sensor.toml` currently declares `body_template = '{}'` for the `fetch_audit_logs` step.
xDome's `POST /api/v1/audit_log/get` with an empty body returns the **entire audit history**,
reliably exceeding the 30-second E-QUERY-004 timeout on every query shape.

This story delivers the complete fix in one step:

1. **`spec_driven_adapter.rs`** — for Claroty `audit_logs` queries, extract `start_time`/`end_time`
   from the PrismQL AST via ADR-033 Option T1, construct a `filter_by` JSON object, and inject
   as `query_filters["_claroty_audit_filter_by"]`. Four filter cases (all `value` fields are
   ISO-8601 strings, NOT epoch-ms integers — BC-2.01.013 v1.21 EC-01-030..033):
   - (EC-01-030) No `start_time`, no `end_time` → `{"field": "timestamp", "operation": "greater_or_equal", "value": "<now−7d as ISO-8601>"}` (7d default; never unbounded)
   - (EC-01-031) `start_time` only, no `end_time` → `{"field": "timestamp", "operation": "greater_or_equal", "value": "<start_time as ISO-8601>"}`
   - (EC-01-032) `end_time` only, no `start_time` → `{"field": "timestamp", "operation": "less_or_equal", "value": "<end_time as ISO-8601>"}` (SINGLE filter; NO synthetic 7-day lower bound; never compound `and`)
   - (EC-01-033) Both bounds → `{"operation": "and", "operands": [{"field": "timestamp", "operation": "greater_or_equal", "value": "<start_time as ISO-8601>"}, {"field": "timestamp", "operation": "less_or_equal", "value": "<end_time as ISO-8601>"}]}` (compound key is `operands`, NOT `conditions`)

2. **`pipeline.rs`** — extend `step_vars` seeding to auto-parse query_filter values starting with
   `{` or `[` into `serde_json::Value::Object` / `Value::Array`; non-JSON strings remain
   `Value::String` (backward-compat for FQL/AQL).

3. **`claroty.sensor.toml`** — change `body_template` to:
   `'{"filter_by": ${query.filter._claroty_audit_filter_by}}'`

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | v1.21 | §Postconditions `Claroty audit_logs` row: four filter cases (EC-01-030..EC-01-033), ISO-8601 `value` strings, `operands` compound key (NOT `conditions`), `ApiQueryFilter = HashMap<String, serde_json::Value>` DTU ground-truth, error surface |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.39 | §Postconditions §1 push-down block specifies the final `body_template` value; SAP-2 parity gate for `ApiQueryFilter = HashMap<String, serde_json::Value>` DTU ground-truth |

## Acceptance Criteria

### AC-001: Default filter (no user time constraint) → `greater_or_equal (now−604800s)` in POST body (traces to BC-2.01.013 postcondition `Claroty audit_logs` row — EC-01-030 default fallback clause)

When a PrismQL query against `claroty_audit_logs` has no explicit time-range filter,
`spec_driven_adapter.rs §spec_driven_adapter` injects:
`query_filters["_claroty_audit_filter_by"]` = JSON string of
`{"field": "timestamp", "operation": "greater_or_equal", "value": "<now_minus_604800s as ISO-8601>"}`

The xDome POST body becomes `{"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "<now_minus_7d as ISO-8601>"}}`.
This bounds the response to the last 7 days on every unfiltered query shape, eliminating E-QUERY-004.

**Test:** `test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal`

### AC-002: Explicit `start_time` → `greater_or_equal` at user-specified bound (NOT bounded to 7d) (traces to BC-2.01.013 postcondition `Claroty audit_logs` row — EC-01-031 explicit start_time clause)

When a PrismQL query has `WHERE timestamp > '<45d_ago>'`, ADR-033 Option T1 extracts
`start_time = Some("<45d_ago>")`. `spec_driven_adapter.rs §spec_driven_adapter` injects a
`greater_or_equal` filter at the 45-day-ago bound (ISO-8601 string value). The xDome POST body
carries the user's actual time filter. Rows from 45 days ago are returned — the 7-day bounded
default is NOT applied when an explicit filter is present.

**Test:** `test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated`
(assert POST body carries the 45-day-ago bound as an ISO-8601 string, NOT the 7-day fallback value)

### AC-003: Both `start_time` and `end_time` → compound `and` filter (traces to BC-2.01.013 postcondition `Claroty audit_logs` row — EC-01-033 compound `and` clause)

When a PrismQL query has `WHERE timestamp BETWEEN '<start>' AND '<end>'`, ADR-033 Option T1
extracts both bounds. `spec_driven_adapter.rs §spec_driven_adapter` injects:
`{"operation": "and", "operands": [{"field": "timestamp", "operation": "greater_or_equal", "value": "<start_time as ISO-8601>"}, {"field": "timestamp", "operation": "less_or_equal", "value": "<end_time as ISO-8601>"}]}`
(compound key is `operands`, NOT `conditions`; all `value` fields are ISO-8601 strings)

DataFusion applies an identical post-materialization filter as a correctness backstop
(BC-2.01.013 result-equivalence invariant, BC-2.11.007).

**Test:** `test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and`
(assert POST body contains `"operation": "and"`, `"operands"` key (NOT `"conditions"`), with two operand objects: `"greater_or_equal"` + `"less_or_equal"`, both with ISO-8601 string values)

### AC-004: `pipeline.rs` JSON-object filter strings parse to `Value::Object`; plain strings remain `Value::String` (traces to BC-2.16.013 postcondition §1 Layer-2 block — `pipeline.rs` auto-parse clause; BC-2.01.013 postcondition — backward-compat invariant)

`pipeline.rs §pipeline` `step_vars` seeding: a `query_filters` value starting with `{` or `[`
is parsed via `serde_json::from_str` into `Value::Object` or `Value::Array` respectively, so
it expands verbatim into `body_template` via `${query.filter._claroty_audit_filter_by}`.
A value NOT starting with `{`/`[` (CrowdStrike FQL string, Armis AQL string) remains
`Value::String` — backward-compat is preserved for all existing sensors. The else-branch
(non-JSON-string path) MUST be explicitly asserted as a positive `Value::String` outcome, not
merely the absence of a panic.

**Test:** `test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat`
(inject both a JSON-object string `'{"field": "timestamp", "operation": "greater_or_equal", "value": "2026-01-01T00:00:00Z"}'` and an FQL string `'created_timestamp:>2026-01-01'`; assert: (a) JSON-object string → `serde_json::Value::Object` in step_vars (parsed path); (b) FQL string → `serde_json::Value::String` in step_vars (else-branch backward-compat gate) — assertion MUST be a positive `assert_eq!(result, Value::String(...))`, NOT merely the absence of panic)

### AC-005: `claroty.sensor.toml` `fetch_audit_logs` body_template is the Layer-2 variable form (traces to BC-2.16.013 postcondition §1 Layer-2 block — TOML body_template change)

`crates/prism-sensors/specs/claroty.sensor.toml` `fetch_audit_logs` step declares:
```toml
body_template = '{"filter_by": ${query.filter._claroty_audit_filter_by}}'
```
The prior `body_template = '{}'` is replaced. `devices` and `alerts` tables are unaffected.
`SpecLoader::parse` on the modified file returns `Ok(SensorSpec)` without validation error.

**Test:** covered by RG-001 setup (SpecLoader::parse is invoked as part of test setup; parse failure causes test failure).

### AC-006: xDome 4xx rejection of the `filter_by` filter surfaces as E-SENSOR-001 (traces to BC-2.01.013 postcondition `Claroty audit_logs` row — error propagation clause; BC-2.16.013 v1.39 §1 — filter-rejection MUST anchor)

When xDome returns a 4xx response to a `POST /api/v1/audit_log/get` request carrying the
injected `filter_by` object (e.g., invalid operation name, unsupported field), the adapter
MUST surface the error as a structured `E-SENSOR-001` sensor error — NOT a panic, NOT a
`Vec::new()` silent empty return, NOT a `SensorError::Internal` catch-all. The error must
carry the HTTP status code and response body snippet so the caller can distinguish a filter
rejection from a connectivity failure.

This AC is the defense against ASM-CLAROTY-AUDITLOG-001 failure mode: if the operation name
assumption is wrong on the live xDome API, the error surfaces cleanly.

**Test:** `test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001`
(mock HTTP returns 400; assert adapter yields `E-SENSOR-001` or equivalent `SensorError::HttpError { status: 400 }`,
NOT panic/empty Vec)

### AC-007: End-only filter (`end_time` only, no `start_time`) → single `less_or_equal` at end, no synthetic lower bound (traces to BC-2.01.013 postcondition `Claroty audit_logs` row — EC-01-032 end-only clause)

When a PrismQL query has `WHERE timestamp < '<past_date>'` and ADR-033 Option T1 extracts
`end_time = Some("<past_date>")` with `start_time = None`, `spec_driven_adapter.rs §spec_driven_adapter`
injects a SINGLE filter object:
`{"field": "timestamp", "operation": "less_or_equal", "value": "<end_time as ISO-8601>"}`

NO synthetic `greater_or_equal` lower bound is injected — no compound `and` filter is constructed.
A query requesting records BEFORE a date more than 7 days in the past MUST NOT have a
`greater_or_equal (now−7d)` floor added. Adding such a floor silently produces an inverted/empty
window when `end_time < now−7d` (SOUL.md §4 silent-wrong-result; BC-2.01.013 EC-01-032).

**Test:** `test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal`
(set `FetchContext { end_time: Some(past_date_older_than_7d), start_time: None }`; assert POST body
contains single `{"field": "timestamp", "operation": "less_or_equal", "value": "<ISO-8601>"}`;
assert POST body does NOT contain `"greater_or_equal"`; assert POST body does NOT contain compound
`"operation": "and"`)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal` | Unit (mock HTTP — intercepts POST body) | AC-001: no time filter → `greater_or_equal (now−604800s)` in POST body; TOML parses OK |
| RG-002 | `test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated` | Unit (mock HTTP) | AC-002: explicit `start_time` > 7 days ago → correct bound in POST body; 7d cap NOT applied |
| RG-003 | `test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and` | Unit (mock HTTP) | AC-003: both `start_time` and `end_time` → compound `and` filter in POST body |
| RG-004 | `test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat` | Unit (pipeline.rs in-module test) | AC-004: (a) JSON-object string → `Value::Object` in step_vars (parsed path); (b) FQL/AQL string → `Value::String` in step_vars (else-branch backward-compat gate — positive `assert_eq!` required, NOT merely absence of panic) |
| RG-005 | `test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001` | Unit (mock HTTP returns 400) | AC-006: xDome 4xx rejection → E-SENSOR-001 / `SensorError::HttpError { status: 400 }`; NOT panic/empty Vec |
| RG-006 | `test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal` | Unit (mock HTTP) | AC-007: end-only filter (`end_time` present, no `start_time`) → single `less_or_equal` at end with ISO-8601 value; POST body MUST NOT contain `"greater_or_equal"` or compound `"operation": "and"` |

**BC-5.38.001 density check:** 6 Red Gate tests / 7 acceptance criteria = 0.86 ≥ 0.5 threshold. PASS.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Claroty audit_logs `filter_by` injection block | `crates/prism-bin/src/spec_driven_adapter.rs §spec_driven_adapter` | Pure (constructs JSON from `Option<DateTime<Utc>>`; no I/O) |
| `step_vars` JSON-object auto-parse | `crates/prism-spec-engine/src/pipeline.rs §pipeline` | Pure (string → `serde_json::Value`; no I/O, deterministic) |
| `fetch_audit_logs` step `body_template` | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| `PipelineExecutor::execute` (body injection) | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome / DTU; expands `body_template` with step_vars) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-22 Process Lifecycle (prism-bin; `spec_driven_adapter.rs` is the adapter bridge)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; `pipeline.rs` is the execution engine)
- ADR-033 §Decision (push-down time-window extraction — Option T1 pre-fan-out heuristic; this story extends the Claroty path)

## Purity Classification

- **Pure functions** (no I/O, deterministic): Claroty audit_logs `filter_by` JSON construction in `spec_driven_adapter.rs §spec_driven_adapter` (takes `Option<DateTime<Utc>>` inputs, returns a serialized JSON string; no I/O, deterministic); `step_vars` JSON-object auto-parse extension in `pipeline.rs §pipeline` (string → `serde_json::Value`; deterministic); `body_template` string value in TOML (static data); `SpecLoader::parse` for RG-001 test setup (pure TOML parse).
- **Effectful functions** (I/O, network, tracing): `PipelineExecutor::execute` (HTTP POST to xDome using the expanded body_template; runtime I/O); RG-001/RG-002/RG-003/RG-005 mock-HTTP test interceptors (spawn HTTP server to capture request bodies / return synthetic 4xx responses).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `start_time` is `None` (no time filter in query) | Default `greater_or_equal (now−604800s)` injected; response bounded to last 7 days; never unbounded |
| EC-002 | `end_time` is `Some` but `start_time` is `None` (end-only case; BC-2.01.013 EC-01-032) | Single `{"field": "timestamp", "operation": "less_or_equal", "value": "<end_time as ISO-8601>"}` — NO synthetic `greater_or_equal` lower bound. A `WHERE timestamp < X` with no `start_time` MUST NOT add a 7-day default floor; doing so silently produces an inverted/empty window when `end_time < now−7d` (SOUL.md §4). |
| EC-003 | CrowdStrike FQL string `'created_timestamp:>...'` passes through `step_vars` seeding | Does NOT start with `{`/`[` → remains `Value::String`; no regression in CrowdStrike push-down |
| EC-004 | Armis AQL string `'in:devices after:2026-01-01T00:00:00'` passes through `step_vars` seeding | Does NOT start with `{`/`[` → remains `Value::String`; no regression in Armis AQL push-down |
| EC-005 | `_claroty_audit_filter_by` JSON string fails `serde_json::from_str` (malformed JSON) | Log WARN; fall back to `Value::String` passthrough — degrade gracefully, no panic |
| EC-006 | xDome returns 4xx (invalid `operation`, unsupported `field`, bad filter structure) | E-SENSOR-001 / `SensorError::HttpError { status: N }` — NOT panic, NOT silent `Vec::new()` return (Standing Rule 3 §2) |
| EC-007 | xDome returns 5xx (server error during filter processing) | Same structured error path as EC-006 — `SensorError::HttpError { status: 5xx }` |
| EC-008 | User issues `FROM claroty_audit_logs WHERE timestamp > '2025-01-01T00:00:00Z'` (> 7 months ago) | `greater_or_equal` at the 2025-01-01 bound is sent to xDome; xDome returns rows from that date onward (subject to its retention policy); DataFusion post-filter confirms result correctness (BC-2.01.013 result-equivalence invariant) |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~6,500 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (full — read CrowdStrike FQL + Armis AQL pattern) | ~12,000 |
| `crates/prism-spec-engine/src/pipeline.rs` (step_vars seeding section) | ~6,000 |
| BC-2.01.013 v1.21 (full — push-down translation table) | ~8,000 |
| BC-2.16.013 v1.39 (audit_logs push-down block) | ~4,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` | ~5,000 |
| `crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` (SAP-2 ground-truth) | ~2,000 |
| ADR-033 §Decision (push-down mechanism reference) | ~3,000 |
| Test file (6 Red Gate tests) | ~7,000 |
| **Total estimate** | **~53,500 tokens** |

Borderline — within 20-30% of a 200K window (~40-60K target per story). If context is tight,
load `spec_driven_adapter.rs` in sections: read the CrowdStrike FQL injection block first
as the canonical pattern, then Armis AQL, then write the Claroty block.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write `test_BC_2_16_013_pipeline_json_filter_string_parsed_to_value_object_backward_compat` inside `crates/prism-spec-engine/src/pipeline.rs §pipeline` `#[cfg(test)] mod tests`. Test injects a `query_filters` map with one JSON-object string `'{"field": "timestamp", "operation": "greater_or_equal", "value": 1234567890}'` and one FQL string `'created_timestamp:>2026-01-01'`. Asserts the JSON string becomes `serde_json::Value::Object` in step_vars; FQL string remains `serde_json::Value::String`. MUST fail before Task 3.

- [ ] **Task 2 (Red Gate — test first):** Write the five mock-HTTP Red Gate tests (RG-001, RG-002, RG-003, RG-005, RG-006) in a new test file (e.g., `crates/prism-bin/tests/bc_2_01_013_claroty_audit_logs_layer2.rs`). Each test: (a) sets up a mock HTTP server capturing the POST body; (b) constructs `FetchContext` with appropriate `start_time`/`end_time`; (c) invokes the spec-driven pipeline; (d) asserts POST body shape OR error type. RG-005 uses a mock returning HTTP 400 and asserts `SensorError::HttpError { status: 400 }`. RG-006 sets `FetchContext { end_time: Some(past_date_older_than_7d), start_time: None }` and asserts POST body has a single `less_or_equal` object with no `greater_or_equal` and no compound `and`. All five MUST fail before Task 4.

- [ ] **Task 3 (Implementation — pipeline.rs):** Extend `step_vars` seeding in `crates/prism-spec-engine/src/pipeline.rs §pipeline`. For each `(k, v)` in `FetchContext::query_filters`, if `v.starts_with('{') || v.starts_with('[')`, attempt `serde_json::from_str::<serde_json::Value>(&v)`; on `Ok(val)` insert `val`; on `Err` log WARN and insert `Value::String(v)`. Otherwise insert `Value::String(v)`. Run `just iter prism-spec-engine` — RG-004 must pass (Green).

- [ ] **Task 4 (Implementation — spec_driven_adapter.rs):** Add the Claroty audit_logs `filter_by` injection block in `crates/prism-bin/src/spec_driven_adapter.rs §spec_driven_adapter`, mirroring the CrowdStrike FQL injection pattern:
  - Read the existing CrowdStrike block FIRST to understand the `sensor_id`/`table_name` guard, `query_filters.insert`, and serialization idiom.
  - Claroty guard: `sensor_id == "claroty" && table_name == "audit_logs"`
  - (EC-01-030) Default (no `start_time`, no `end_time`): `filter_by = json!({"field": "timestamp", "operation": "greater_or_equal", "value": (Utc::now() - Duration::seconds(604800)).to_rfc3339()})`
  - (EC-01-031) `start_time` only, no `end_time`: `filter_by = json!({"field": "timestamp", "operation": "greater_or_equal", "value": start.to_rfc3339()})`
  - (EC-01-032) `end_time` only, no `start_time`: `filter_by = json!({"field": "timestamp", "operation": "less_or_equal", "value": end.to_rfc3339()})` — SINGLE `less_or_equal`; NO compound `and`; NO synthetic lower bound
  - (EC-01-033) Both bounds: `filter_by = json!({"operation": "and", "operands": [{"field": "timestamp", "operation": "greater_or_equal", "value": start.to_rfc3339()}, {"field": "timestamp", "operation": "less_or_equal", "value": end.to_rfc3339()}]})` (key is `operands`, NOT `conditions`)
  - Insert: `context.query_filters.insert("_claroty_audit_filter_by".to_string(), filter_by.to_string())`
  - Run `just iter prism-bin` — RG-001/RG-002/RG-003/RG-005/RG-006 must pass (Green).

- [ ] **Task 5 (Implementation — claroty.sensor.toml):** Change `body_template` in `crates/prism-sensors/specs/claroty.sensor.toml` `fetch_audit_logs` step to:
  `body_template = '{"filter_by": ${query.filter._claroty_audit_filter_by}}'`
  Run `just check` to confirm the TOML parses without error and `SpecLoader::parse` returns `Ok`.

- [ ] **Task 6 (SAP-2 self-check):** Read `crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` (NOT `§ClarotyAuditLogFilter` — that type does not exist in the codebase; it is a phantom reference per BC-2.01.013 v1.21). `ApiQueryFilter = HashMap<String, serde_json::Value>` is the DTU ground-truth for the `filter_by` field. Read the audit_log route handler `crates/prism-dtu-claroty/src/routes/audit_log.rs` emission site to confirm the request body `filter_by` key is deserialized from the incoming POST body into `ApiQueryFilter` (SAP-2 Rule 6). Verify that `field`, `operation`, `value`, and `operands` are valid JSON keys (HashMap keys, not Rust struct fields). If an operation name mismatch is found (`greater_or_equal`/`less_or_equal`/`and`), fix `spec_driven_adapter.rs` to match the DTU ground-truth before committing.

- [ ] **Task 7 (SAP-1 self-check):** If any new `tracing::*!(event_type = ...)` is added to `spec_driven_adapter.rs` or `pipeline.rs`, add a corresponding row to BC-2.16.002 §Postconditions Structured Event Catalog. The existing CrowdStrike/Armis push-down pattern emits no new event_type values for the injection logic — follow the same convention where possible.

- [ ] **Task 8 (Final gate):** Run `just check` (full workspace). Confirm all 6 Red Gate tests pass (RG-001 through RG-006). Confirm CrowdStrike and Armis push-down tests pass (backward-compat). Confirm no new `unwrap()`/`expect()` on `Result` in production code paths.

## Previous Story Intelligence

1. **CrowdStrike FQL injection (S-DEMO-QUERY-PUSHDOWN-001, merged):** The canonical pattern for
   push-down injection lives in `spec_driven_adapter.rs §spec_driven_adapter`. The Claroty block
   mirrors this structure but produces a JSON object string. Read the CrowdStrike block FIRST.

2. **Armis AQL augmentation (S-DEMO-ARMIS-AQL-001, in-progress):** The Armis path produces a
   plain string (augmented AQL expression). The JSON auto-parse in Task 3 must NOT accidentally
   parse an Armis AQL value (AQL does not use JSON syntax — no practical risk, but worth noting).

3. **S-DEMO-CLAROTY-PAGINATION-001 (merged):** Established OffsetLimit POST-body pagination for
   audit_logs. OffsetLimit merges `{"offset": N, "limit": 1000}` into the body AFTER body_template
   expansion. The final POST body is:
   `{"filter_by": {...}, "offset": 0, "limit": 1000}` — `filter_by` from injection + `offset`/`limit` from OffsetLimit.
   Verify this coexistence produces valid JSON (no key collision: `filter_by` ≠ `offset`/`limit`).

4. **S-DEMO-CLAROTY-AUDIT-DTU-001 (merged PR #167):** Added `POST /api/v1/audit_log/get` to the DTU
   and created `ClarotyAuditLogEntry`. The DTU uses `filter_by: Option<ApiQueryFilter>` where
   `ApiQueryFilter = HashMap<String, serde_json::Value>` — NOT a dedicated `ClarotyAuditLogFilter`
   struct (that type is a phantom reference; see BC-2.01.013 v1.21 §Postconditions). Read
   `crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` for the DTU ground-truth.

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-22 Process Lifecycle:
- `spec_driven_adapter.rs §spec_driven_adapter` translates PrismQL query parameters into sensor-native API shapes. Push-down injection (FQL, AQL, JSON filter_by) is its canonical responsibility.
- Push-down applies on the first/query-plan step only. `audit_logs` has a single step — no hydration step. `FetchContext::default()` is irrelevant here.
- ADR-033 §Decision: push-down is an optimization; result correctness is DataFusion post-materialization backstop (BC-2.01.013 result-equivalence invariant, BC-2.11.007).

From `architecture/module-decomposition.md` §SS-16 Spec Engine:
- `pipeline.rs §pipeline` step_vars seeding is the only authorized location for JSON-object auto-parse. Do NOT add parsing logic to `spec_driven_adapter.rs` or `spec_parser.rs`.
- Backward-compat is mandatory: existing specs using `${query.filter.KEY}` with plain string values must not be affected. FQL and AQL strings do not start with `{`/`[` — safe.
- `INV-PARITY-003`: TOML spec files are source of truth. `sensor_id: "claroty"` is immutable (INV-PARITY-002).

From ADR-033 §Decision:
- Option T1 pre-fan-out heuristic: `start_time`/`end_time` arrive in `FetchContext` after extraction in `run_materialization_pipeline §run_materialization_pipeline`. The injection block in `spec_driven_adapter.rs` reads these fields and writes to `query_filters["_claroty_audit_filter_by"]`.
- The `devices` and `alerts` Claroty tables retain `body_template: '{}'` — the `filter_by` injection is `audit_logs`-specific.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `serde_json` | per workspace Cargo.toml | JSON object construction in spec_driven_adapter.rs; `json!` macro; auto-parse in pipeline.rs |
| `chrono` | per workspace Cargo.toml | `(Utc::now() - Duration::seconds(604800)).to_rfc3339()` for default ISO-8601 value; `DateTime::to_rfc3339()` for all `value` fields |
| `prism-spec-engine` | workspace path | `PipelineExecutor`, `FetchContext`, step_vars seeding |
| `prism-dtu-claroty` | workspace path | `ApiQueryFilter` (`HashMap<String, serde_json::Value>`) is the SAP-2 ground-truth type (read `types.rs §ApiQueryFilter`; `ClarotyAuditLogFilter` does not exist; do not add as build dep to prism-bin) |
| `wiremock` or equivalent mock HTTP | per dev-dep in prism-bin/Cargo.toml | Mock HTTP server for RG-001/RG-002/RG-003/RG-005 request-body capture |
| `tokio` | per workspace Cargo.toml | Async test runtime |

Do NOT add new Cargo.toml production dependencies. `serde_json` and `chrono` are already present in `prism-bin` and `prism-spec-engine`. `prism-bin` MUST NOT depend on `prism-dtu-claroty` in production code (dev-dep only for SAP-2 type-check tests if needed).

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-bin/src/spec_driven_adapter.rs` | Add Claroty audit_logs `filter_by` injection block (Task 4) |
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | Extend step_vars seeding with JSON-object auto-parse (Task 3); add RG-004 in-module unit test |
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Change `fetch_audit_logs` `body_template` to Layer-2 variable (Task 5) |
| CREATE | `crates/prism-bin/tests/bc_2_01_013_claroty_audit_logs_layer2.rs` | RG-001, RG-002, RG-003, RG-005, RG-006 mock-HTTP Red Gate tests |

Files MUST NOT be modified:
- `crates/prism-query/` (no query engine changes required)
- `crates/prism-sensors/specs/crowdstrike.sensor.toml`, `armis.sensor.toml`, `cyberint.sensor.toml`
- `crates/prism-dtu-claroty/src/types.rs` — read only (SAP-2 ground-truth); escalate if DTU mismatch found

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain a new dependency on `prism-bin` (direction is prism-bin → prism-spec-engine, not the reverse; build MUST fail if this dep appears).

`prism-bin` MUST NOT gain a new production dependency on `prism-dtu-claroty`. SAP-2 field verification is a dev-time check only (read the source file; do not import the types in production code).

The JSON auto-parse extension in `pipeline.rs` uses `serde_json` which is already a `prism-spec-engine` dependency — no new crate dependency.

## Notes for Implementer

1. **`value` field format: ISO-8601 strings.** BC-2.01.013 v1.21 explicitly documents that all
   `value` fields in the Claroty `filter_by` JSON are ISO-8601 strings (NOT epoch-ms integers).
   Use `.to_rfc3339()` on `DateTime<Utc>` values in Task 4. The DTU ground-truth is
   `filter_by: Option<ApiQueryFilter>` where `ApiQueryFilter = HashMap<String, serde_json::Value>`,
   defined at `crates/prism-dtu-claroty/src/types.rs::ApiQueryFilter`. `ClarotyAuditLogFilter`
   does NOT exist as a type in the codebase (phantom type — BC-2.01.013 v1.21 §Postconditions).
   Read `crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` (NOT `§ClarotyAuditLogFilter`)
   for the DTU ground-truth field structure.

2. **E-SENSOR-001 path for RG-005.** The existing `map_spec_engine_error_to_sensor_error` in
   `spec_driven_adapter.rs §spec_driven_adapter` maps `SpecEngineError::HttpRequestFailed { status, ... }`
   to `SensorError::HttpError { status }`. Verify a 400 response from xDome reaches this path
   (not a different error variant). If `HttpRequestFailed` is only emitted on 5xx and not 4xx,
   trace where 4xx responses are currently mapped and confirm the E-SENSOR-001 path is exercised.

3. **OffsetLimit coexistence.** S-DEMO-CLAROTY-PAGINATION-001 (merged) set up OffsetLimit POST-body
   pagination. OffsetLimit merges `{"offset": N, "limit": 1000}` AFTER body_template expansion.
   The final merged body is `{"filter_by": {...}, "offset": 0, "limit": 1000}` — valid JSON, no
   collision. Verify this with an integration test against the DTU if time permits.

4. **No BC-2.16.002 catalog update** unless new `tracing::*!(event_type = ...)` emissions are added.
   The existing push-down patterns emit no new event_type values for injection logic — follow suit.

---

## References

- BC-2.01.013 v1.21 (ACTIVE) — §Postconditions `Claroty audit_logs` row: four filter cases EC-01-030..033, ISO-8601 `value` strings, `operands` compound key, `ApiQueryFilter` DTU ground-truth, error surface
- BC-2.16.013 v1.39 (ACTIVE) — §Postconditions §1 Claroty `audit_logs` push-down block; LIVE-API ASSUMPTION ASM-CLAROTY-AUDITLOG-001
- ADR-033 §Decision — push-down time-window extraction Option T1 pre-fan-out heuristic
- ADR-028 §D1 — TOML body_template grounding (DTU types are ground-truth for field/operation names)
- `crates/prism-dtu-claroty/src/types.rs §ApiQueryFilter` — DTU ground-truth (SAP-2); `ApiQueryFilter = HashMap<String, serde_json::Value>`; `ClarotyAuditLogFilter` does NOT exist (phantom type reference)
- `crates/prism-bin/src/spec_driven_adapter.rs §spec_driven_adapter` — CrowdStrike FQL injection pattern to mirror
- `crates/prism-spec-engine/src/pipeline.rs §pipeline` — step_vars seeding to extend
- `crates/prism-sensors/specs/claroty.sensor.toml §fetch_audit_logs` — step being modified
- S-DEMO-CLAROTY-AUDIT-DTU-001 (merged PR #167) — established `ClarotyAuditLogEntry` + `ApiQueryFilter` DTU type; `ClarotyAuditLogFilter` is a phantom reference that does not exist in the codebase

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 2.1 | 2026-08-15 | story-writer | LOCAL adversary pass-1 fix-burst. BC-array propagation from BC-2.01.013 v1.20→v1.21 and BC-2.16.013 v1.38→v1.39 (product-owner amendments). Changes: (1) Frontmatter BC version pins bumped to v1.21 and v1.39; (2) phantom type `ClarotyAuditLogFilter` replaced with `ApiQueryFilter = HashMap<String, serde_json::Value>` across §Authority, §Behavioral Contracts, §Tasks, §Library, §Notes-for-Implementer, §Previous-Story-Intelligence, §References (F-P1-MED-002 closure); (3) EC-002 corrected — end-only case now prescribes single `less_or_equal` at end with NO synthetic 7-day floor, NOT compound `and` with floor (F-P1-MED-004 closure); (4) Added AC-007 (end-only → single `less_or_equal`, no synthetic floor) + RG-006 (`test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal`) per BC-2.01.013 v1.21 EC-01-032 mandate anchor; (5) datetime `value` fields corrected to ISO-8601 strings (NOT epoch-ms integers) across §Background, §ACs, §Tasks (F-P1-MED-002); (6) compound filter key `operands` made explicit (NOT `conditions`) in AC-003 and §Background (F-P1-HIGH-001 closure); (7) AC-004 and RG-004 description strengthened — FQL/AQL → `Value::String` must be a positive assertion, not merely absence of panic (F-P1-MED-003 closure); (8) Density check updated: 6 RGTs / 7 ACs = 0.86. BC-5.38.001 PASS. acceptance_criteria_count: 6→7. |
| 2.0 | 2026-08-15 | story-writer | Design-change collapse: two-story Layer-1 + Layer-2 design collapsed into single story per human-decided coordinator directive. Story now delivers complete Layer-2 push-down fix in one step. Removed AC-TRUNC-001 and EC-016-013-010 (PO retired silent-truncation behavior — explicit old filters now honored). Added AC-006 E-SENSOR-001 filter-rejection path + RG-005. Added explicit-start-time-honored assertion (RG-002). Updated holdout_scenarios to all four HS-AUDITLOG-001-A-001..004. Updated depends_on [] blocks []. 6 ACs, 5 RGTs, density 0.83. BC-5.38.001 PASS. |
| 1.0 | 2026-08-15 | story-writer | Initial authoring (Layer-1 TOML-only design, since superseded by v2.0). |
