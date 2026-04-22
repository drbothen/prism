# Demo Evidence Report — S-1.11

**Story:** S-1.11 — prism-spec-engine: Spec Loading and Pipeline Execution
**Branch:** feature/S-1.11-spec-loading
**Base commit:** b146a97 (107/107 tests pass)
**Recorded:** 2026-04-22
**Policy:** POL-010

---

## Coverage Matrix

| AC | BC / VP | Title | Success Path | Error Path | Files |
|----|---------|-------|:---:|:---:|-------|
| AC-1 | BC-2.16.001 | Sensor Spec File Loading — Parse TOML → SensorTableDescriptors | PASS | PASS | [gif](AC-001-spec-parsing.gif) [webm](AC-001-spec-parsing.webm) [tape](AC-001-spec-parsing.tape) |
| AC-2 | BC-2.16.002 | Pipeline Variable Interpolation — `${step1.response.access_token}` in step 2 | PASS | PASS | [gif](AC-002-pipeline-interpolation.gif) [webm](AC-002-pipeline-interpolation.webm) [tape](AC-002-pipeline-interpolation.tape) |
| AC-3 | BC-2.16.003 | Column-to-OCSF Mapping — `created_timestamp` → OCSF `time` field | PASS | PASS | [gif](AC-003-column-mapping.gif) [webm](AC-003-column-mapping.webm) [tape](AC-003-column-mapping.tape) |
| AC-4 | BC-2.16.004 | CustomAdapter Override — registered adapter replaces TOML spec pipeline | PASS | PASS | [gif](AC-004-custom-adapter.gif) [webm](AC-004-custom-adapter.webm) [tape](AC-004-custom-adapter.tape) |
| AC-5 | BC-2.16.009 | Validation — dangling `${nonexistent.field}` → error with TOML path | PASS | PASS | [gif](AC-005-validation.gif) [webm](AC-005-validation.webm) [tape](AC-005-validation.tape) |
| AC-6 | VP-023 | Fuzz Harness — SpecParser no-panic on arbitrary TOML | Deferred (Phase 5) | — | [doc](AC-006-vp023-fuzz-harness.md) |
| AC-7 | VP-059 | Proptest — N injected errors → Err with exactly N items (no fail-fast) | PASS | — | [gif](AC-007-vp059-proptest.gif) [webm](AC-007-vp059-proptest.webm) [tape](AC-007-vp059-proptest.tape) |

**Coverage: 6/7 AC recorded via VHS (AC-6 deferred by design — 30-min fuzz run is Phase 5). AC-6 evidence = harness doc + unit test proxy.**

---

## Demo Binary

All VHS tapes invoke `crates/prism-spec-engine/examples/demo_spec_loading.rs` — a purpose-built
demo binary that exercises the public `prism-spec-engine` API with no test harness intermediary.
Subcommands: `ac1`, `ac1e`, `ac2`, `ac2e`, `ac3`, `ac3e`, `ac4`, `ac4e`, `ac5`, `ac5e`, `vp059`.

---

## AC-001 — BC-2.16.001: Sensor Spec File Loading

**Success path:** `SpecLoader::parse(CROWDSTRIKE_SENSOR_TOML)` produces a `SensorSpec` with:
- `sensor_id = "crowdstrike"`, `auth_type = Oauth2ClientCredentials`
- 2 tables: `detections` (2 cols, 1 step) and `hosts` (1 col, 1 step)

**Error path:** Malformed TOML (unterminated string) returns `Err(E-SPEC-001)` with line/column.

---

## AC-002 — BC-2.16.002: Pipeline Variable Interpolation

**Success path:** Template `/oauth2/revoke?token=${step1.response.access_token}` interpolates
to `/oauth2/revoke?token=tok-oauth-abc-123` (URL-context percent-encoding applied).

**Error path:** Template with `${nonexistent.token}` (no dot) returns `InterpolationError::UnknownStep`.

---

## AC-003 — BC-2.16.003: Column-to-OCSF Mapping

**Success path:** Raw record `{"created_timestamp": "2026-04-22T10:00:00Z", "severity_name": "High"}`
with spec mappings `created_timestamp → time` and `severity_name → severity` produces
`mapped_fields: {"time": "...", "severity": "High"}` with 0 raw_extensions.

**Error path:** Column with no `ocsf_field` mapping goes to `raw_extensions` (record not dropped —
BC-2.16.003 invariant: no records dropped during mapping).

---

## AC-004 — BC-2.16.004: Rust Escape Hatch (CustomAdapter)

**Success path:** `MockCrowdStrikeAdapter` registered in `CustomAdapterRegistry`, retrieved by
`sensor_id = "crowdstrike"`, `override_fetch` returns synthetic records — TOML spec pipeline
fully bypassed.

**Error path:** Second `register(Box::new(MockCrowdStrikeAdapter))` returns `Err(E-SPEC-009)`:
"duplicate adapter sensor_id 'crowdstrike' (EC-003: adapter name must be unique)".

---

## AC-005 — BC-2.16.009: Spec Validation

**Success path (dangling ref):** Spec with `step2.path_template = "/alerts?token=${nonexistent.field}"`
returns `Err([E-SPEC-001 at sensor.tables[0].steps[1].path_template])` — 1 error, TOML path included.

**Error path (multi-error):** Spec with 4 distinct schema errors (empty sensor_id, empty name,
invalid base_url, invalid semver version) returns `Err([4 errors])` in one pass — no fail-fast.
All 4 categories reported simultaneously.

---

## AC-006 — VP-023: Fuzz Harness (Deferred)

The fuzz target at `fuzz/fuzz_targets/spec_parser.rs` is implemented and registered.
The 30-minute continuous fuzz run is deferred to Phase 5 per project convention.

Evidence file: `AC-006-vp023-fuzz-harness.md`

Deterministic proxy: `test_BC_2_16_001_returns_parse_error_for_malformed_toml` exercises
malformed, missing-key, and invalid-field paths that cover the primary no-panic invariant.

---

## AC-007 — VP-059: All-errors-collected Proptest

**Property:** For a `SensorSpec` with N distinct injected validation errors (dangling refs),
`validate_sensor_spec` returns `Err` with exactly N items — never fewer (no fail-fast).

**Demonstrated N values:** 1, 3, 5, 10 — all return exactly N errors.

**Proptest harness:** `crates/prism-spec-engine/src/proofs/spec_validator.rs`
`test_BC_2_16_009_invariant_all_errors_collected` — passes in 0.02s.

---

## Placeholders / Deferred Items

| Item | Reason | Phase |
|------|--------|-------|
| AC-006 continuous fuzz run | 30-min fuzz is a Phase 5 activity; harness production-ready | Phase 5 |
