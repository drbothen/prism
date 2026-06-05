# Red Gate Log — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Query-Param Push-Down into PipelineExecutor
**Phase:** 3 (TDD Implementation) — Red Gate Step
**Wave:** wave-5-e-demo-fidelity
**Date:** 2026-06-04
**Author:** test-writer
**Commit:** 0ac34b85 (feature/S-DEMO-QUERY-PUSHDOWN-001)

---

## Red Gate Status: RED

3 of 5 new tests FAIL. Workspace COMPILES. Red Gate discipline satisfied per BC-5.38.001.

---

## Stub Changes

| File | Change |
|------|--------|
| `crates/prism-spec-engine/src/pipeline.rs` | `FetchContext` gains 4 optional push-down fields (`cursor`, `limit`, `start_time`, `end_time`) + `Default` impl + `with_push_down()` constructor. `apply_push_down_to_request()` stub (returns `req` unchanged). `build_request()` + `issue_request_with_retry()` signatures updated to thread `&FetchContext`. |
| `crates/prism-bin/src/spec_driven_adapter.rs` | `fetch()` extracts push-down fields from `QueryParams` and populates `FetchContext` via `with_push_down()`. |

**`cargo check -p prism-spec-engine -p prism-bin` result: PASS (exit 0)**

---

## Test Results

### cargo nextest run -p prism-spec-engine -E 'test(push_down_red_gate)'

| Test | AC | Result | Reason |
|------|----|--------|--------|
| `test_BC_2_01_013_fetch_context_push_down_fields_default_to_none` | AC-001 | PASS (GREEN-BY-DESIGN) | Structural test — fields exist; Default/with_push_down correct per BC-5.38.002 |
| `test_BC_2_11_007_unsupported_push_down_cursor_silently_ignored` | AC-006 | PASS (GREEN-BY-DESIGN) | Stub does nothing with cursor → cursor absent from request → AC-006 satisfied vacuously |
| `test_BC_2_11_007_crowdstrike_limit_and_time_window_pushed_to_fql` | AC-002 | **FAIL** | wiremock `query_param("limit", "50")` matcher not satisfied — stub does not add `limit=50` to request URL |
| `test_BC_2_11_007_push_down_result_equivalence_invariant` | AC-005 | **FAIL** | wiremock `query_param("limit", "1")` matcher not satisfied — stub does not add `limit=1` to request URL |
| `test_BC_2_11_007_cyberint_time_window_pushed_to_post_body` | AC-003 | **FAIL** | wiremock `body_partial_json({from_date, to_date})` matcher not satisfied — stub does not inject time fields into POST body |

**Summary: 2 PASS (GREEN-BY-DESIGN), 3 FAIL (load-bearing behavioral assertions)**

### Regression check: cargo nextest run -p prism-spec-engine -p prism-bin

674 tests total: 671 PASS, 3 FAIL (the 3 Red Gate tests above). 24 SKIPPED.
No regressions introduced by stub changes.

---

## Failure Analysis

Each FAIL is for the correct reason (unimplemented push-down translation):

1. **`test_BC_2_11_007_crowdstrike_limit_and_time_window_pushed_to_fql`**
   - Received request: `GET /detections/queries/detections/v1` (no query params)
   - Expected: `GET /detections/queries/detections/v1?limit=50` (with optional FQL `filter` param)
   - Root cause: `apply_push_down_to_request` returns `req` unchanged; no `limit` param added

2. **`test_BC_2_11_007_push_down_result_equivalence_invariant`**
   - Received request: `GET /detections/queries/detections/v1` (no query params)
   - Expected: `GET /detections/queries/detections/v1?limit=1`
   - Root cause: same as above

3. **`test_BC_2_11_007_cyberint_time_window_pushed_to_post_body`**
   - Received body: `{"customer_id": "test"}` (no time fields)
   - Expected body: `{"customer_id": "test", "from_date": "2026-01-01T00:00:00+00:00", "to_date": "2026-01-31T23:59:59+00:00"}`
   - Root cause: `apply_push_down_to_request` returns `req` unchanged; no `from_date`/`to_date` injection

---

## AC Coverage Mapping

| AC | Test Name | Coverage | Notes |
|----|-----------|----------|-------|
| AC-001 | `test_BC_2_01_013_fetch_context_push_down_fields_default_to_none` | Full | Structural: field existence + Default + EC-001 normalisation |
| AC-002 | `test_BC_2_11_007_crowdstrike_limit_and_time_window_pushed_to_fql` | Partial | Tests `limit`; FQL `filter` param for `start_time` tested implicitly via wiremock (implementer adds full assertion) |
| AC-003 | `test_BC_2_11_007_cyberint_time_window_pushed_to_post_body` | Full | Tests `from_date`/`to_date` injection in POST body via `body_partial_json` matcher |
| AC-004 (Claroty/Armis) | Not in Red Gate table | Not covered | Story spec Red Gate table has 4 named tests; AC-004 is covered by implementer's unit tests |
| AC-005 | `test_BC_2_11_007_push_down_result_equivalence_invariant` | Full | Tests that `limit=1` reaches the wire (the invariant precondition) |
| AC-006 | `test_BC_2_11_007_unsupported_push_down_cursor_silently_ignored` | Full | Asserts cursor NOT in request URL for unsupported sensor |

---

## BC Clause → Test Mapping

| BC Clause | Test |
|-----------|------|
| BC-2.01.013 Pagination/Push-Down Scope Clause — FetchContext field additions (precondition) | `test_BC_2_01_013_fetch_context_push_down_fields_default_to_none` |
| BC-2.01.013 Pagination/Push-Down Scope Clause — graceful degradation (postcondition) | `test_BC_2_11_007_unsupported_push_down_cursor_silently_ignored` |
| BC-2.11.007 push-down translation postcondition — CrowdStrike limit | `test_BC_2_11_007_crowdstrike_limit_and_time_window_pushed_to_fql` |
| BC-2.11.007 push-down translation postcondition — Cyberint time-window | `test_BC_2_11_007_cyberint_time_window_pushed_to_post_body` |
| BC-2.11.007 invariant — "push-down is an optimization only; result must be identical" | `test_BC_2_11_007_push_down_result_equivalence_invariant` |

---

## Implementer Handoff Instructions

1. Fill in `apply_push_down_to_request()` in `pipeline.rs` with per-sensor translation logic:
   - CrowdStrike (`auth_type = Oauth2ClientCredentials`): add `limit=N` query param; add `filter=created_timestamp:>'<ISO8601>'` FQL param when `start_time` is `Some`.
   - Cyberint (`auth_type = CookieRoundtrip`): inject `from_date`/`to_date` into POST body JSON.
   - Claroty (`auth_type = BearerStatic`, sensor_id = "claroty"): inject `limit` into POST body JSON.
   - Armis (`auth_type = BearerStatic`, sensor_id = "armis"): inject `maxResults` + `timeFrame` params.
   - Unsupported params: silently skip (AC-006).
2. NOTE: `auth_type` alone is ambiguous for `BearerStatic` (Armis and Claroty both use it). The implementation must use `sensor_id` (available via `spec.sensor_id`) to discriminate between Armis and Claroty. `apply_push_down_to_request` needs the sensor_id — update the function signature to accept `sensor_id: &str`.
3. After implementing: `cargo nextest run -p prism-spec-engine -E 'test(push_down_red_gate)'` must show 5 PASS, 0 FAIL.
4. Add unit tests for AC-004 (Claroty limit; Armis maxResults + timeFrame).
5. Add edge-case tests for EC-001 (limit=0), EC-002 (open-ended range), EC-005 (start > end).
6. Run `just check` for final pre-push gate.
