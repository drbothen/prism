# Demo Evidence Report — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 v2.5 — Correct per-sensor push-down wiring (ADR-033 T1 + Armis AQL full wiring + CrowdStrike DTU FQL honoring)
**Branch:** feature/S-DEMO-QUERY-PUSHDOWN-001
**Converged at commit:** 69aafcc7 (LOCAL 3-CLEAN, ADV-P08-MED-001 inclusive-boundary fix) + SEC-004 hardening @ f290a43d
**Wave:** wave-5-e-demo-fidelity
**Evidence date:** 2026-06-05
**Policy:** POLICY 10 — all evidence in `docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/` (story-scoped path)

---

## Coverage Summary

| AC ID | Description | Evidence file | Result | Notes |
|-------|-------------|---------------|--------|-------|
| AC-CWS-001 | CrowdStrike limit reaches DetectionListParams | AC-CWS-001-003-crowdstrike-limit-fql.gif/.webm | PASS | SAP-2 compliant; production crowdstrike.sensor.toml shape |
| AC-CWS-002 | FQL time-window both start+end via materialization pipeline | AC-CWS-002-e2e-fql-via-materialization.gif/.webm | PASS | Wire-level: DTU /dtu/filter-log receives combined FQL |
| AC-CWS-003 | No filter param when no time predicates | AC-CWS-001-003-crowdstrike-limit-fql.gif/.webm | PASS | Absence assertion load-bearing |
| AC-CWS-DTU-001 | CrowdStrike DTU honors filter= FQL — filtered_count < unfiltered_count | AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.gif/.webm | PASS | LOAD-BEARING assertion; 8/8 prism-dtu-crowdstrike tests PASS (includes SEC-004 hardening) |
| AC-ARMIS-001 | Armis AQL passthrough — no maxResults or timeFrame | AC-ARMIS-001-002-aql-passthrough.gif/.webm | PASS | Absence of maxResults/timeFrame fields asserted |
| AC-ARMIS-002 | No additional params beyond aql, offset, limit | AC-ARMIS-001-002-aql-passthrough.gif/.webm | PASS | Only SearchQueryParams fields aql/offset/limit present |
| AC-ARMIS-TW-001 | AQL augmentation — after:/before: clauses appended | AC-ARMIS-TW-001-003-aql-augmentation.gif/.webm | PASS | Bare unquoted timezone-naive form; no lastSeen:> form |
| AC-ARMIS-TW-002 | Armis DTU filters fixture by time window (LOAD-BEARING) | AC-ARMIS-TW-002-004-dtu-filter-equivalence.gif/.webm | PASS | filtered_count < unfiltered_count asserted |
| AC-ARMIS-TW-003 | Anti-double-filter guard — verbatim passthrough when AQL has time clause | AC-ARMIS-TW-001-003-aql-augmentation.gif/.webm | PASS | All canonical time keywords checked (after:/before:/timeFrame:) |
| AC-ARMIS-TW-004 | Result-equivalence — push-down vs DataFusion post-filter identical | AC-ARMIS-TW-002-004-dtu-filter-equivalence.gif/.webm | PASS | BC-2.11.007 invariant satisfied |
| AC-ARMIS-TW-005 | E2E aql-log contains augmented AQL with time clause | AC-ARMIS-TW-005-e2e-aql-log-ignored.gif/.webm | PASS | `#[ignore]`; run via `--run-ignored ignored-only`; 1/1 test PASS |
| AC-CYB-001 | Cyberint: no from_date/to_date/page_size in AlertListParams | AC-CYB-001-CLAR-001-correctness-removals.gif/.webm | PASS | GET cursor-only; no POST body injection |
| AC-CLAR-001 | Claroty: body_template remains empty {}; no time-window injection | AC-CYB-001-CLAR-001-correctness-removals.gif/.webm | PASS | OffsetLimit URL params only |
| AC-WIRE-001 | run_materialization_pipeline populates start_time from PrismQL AST | AC-WIRE-001-materialization-pipeline-time-extraction.gif/.webm | PASS | ADR-033 T1 heuristic; not direct FetchContext construction |
| AC-WIRE-001b | Safe default when resolved_spec_map is None | AC-WIRE-001-materialization-pipeline-time-extraction.gif/.webm | PASS | No panic; both start_time and end_time return None |
| AC-INDEX-001 | armis.sensor.toml last_seen + created_at have options=["INDEX"] | AC-INDEX-001-armis-toml-index-options.gif/.webm | PASS | Required for Option T1 AQL augmentation eligibility |
| AC-EQUIV-001 | Result-equivalence via real run_materialization_pipeline | AC-EQUIV-001-EC-009-e2e-result-equivalence.gif/.webm | PASS | Real materialization path; no direct FetchContext bypass |
| EC-009 | Inclusive boundary (>=/<=) — boundary record present in push-down result | AC-EQUIV-001-EC-009-e2e-result-equivalence.gif/.webm | PASS | `to_rfc3339_opts(SecondsFormat::Secs, true)` Z-suffix fix |

| SEC-004 (defense-in-depth) | CrowdStrike DTU: over-length FQL token returns None (input sanitization) | AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.gif/.webm | PASS | prism-dtu-crowdstrike state.rs; not an AC — hardening added in pass-1 @ f290a43d |
| SEC-004 (defense-in-depth) | Armis DTU: over-length AQL token returns None (input sanitization) | SEC-004-armis-dtu-security-hardening.gif/.webm | PASS | prism-dtu-armis search.rs; not an AC — hardening added in pass-1 @ f290a43d |

**Total: 18 ACs + EC-009 covered (16 story ACs + AC-WIRE-001b + EC-009) — all PASS**
**Security hardening: 2 additional SEC-004 defense-in-depth tests — NOT counted as ACs**

---

## Recording Index

### VHS Recordings

| File | ACs covered | Crate under test |
|------|-------------|-----------------|
| AC-WIRE-001-materialization-pipeline-time-extraction.gif / .webm | AC-WIRE-001, AC-WIRE-001b | prism-query |
| AC-WIRE-001-materialization-pipeline-time-extraction.tape | (VHS source) | — |
| AC-ARMIS-TW-001-003-aql-augmentation.gif / .webm | AC-ARMIS-TW-001, AC-ARMIS-TW-003 | prism-query |
| AC-ARMIS-TW-001-003-aql-augmentation.tape | (VHS source) | — |
| AC-CWS-001-003-crowdstrike-limit-fql.gif / .webm | AC-CWS-001, AC-CWS-003 (+ AC-CWS-002 wire-level boundary) | prism-spec-engine |
| AC-CWS-001-003-crowdstrike-limit-fql.tape | (VHS source) | — |
| AC-CWS-002-e2e-fql-via-materialization.gif / .webm | AC-CWS-002 (e2e path via prism-bin) | prism-bin |
| AC-CWS-002-e2e-fql-via-materialization.tape | (VHS source) | — |
| AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.gif / .webm | AC-CWS-DTU-001 | prism-dtu-crowdstrike |
| AC-CWS-DTU-001-crowdstrike-dtu-fql-filter.tape | (VHS source) | — |
| AC-ARMIS-001-002-aql-passthrough.gif / .webm | AC-ARMIS-001, AC-ARMIS-002 | prism-spec-engine |
| AC-ARMIS-001-002-aql-passthrough.tape | (VHS source) | — |
| AC-ARMIS-TW-002-004-dtu-filter-equivalence.gif / .webm | AC-ARMIS-TW-002, AC-ARMIS-TW-004 | prism-spec-engine (parity/armis.rs) |
| AC-ARMIS-TW-002-004-dtu-filter-equivalence.tape | (VHS source) | — |
| AC-ARMIS-TW-005-e2e-aql-log-ignored.gif / .webm | AC-ARMIS-TW-005 | prism-spec-engine (parity/armis.rs, `#[ignore]`) |
| AC-ARMIS-TW-005-e2e-aql-log-ignored.tape | (VHS source) | — |
| AC-CYB-001-CLAR-001-correctness-removals.gif / .webm | AC-CYB-001, AC-CLAR-001 | prism-spec-engine |
| AC-CYB-001-CLAR-001-correctness-removals.tape | (VHS source) | — |
| AC-INDEX-001-armis-toml-index-options.gif / .webm | AC-INDEX-001 | prism-spec-engine |
| AC-INDEX-001-armis-toml-index-options.tape | (VHS source) | — |
| AC-EQUIV-001-EC-009-e2e-result-equivalence.gif / .webm | AC-EQUIV-001, EC-009 | prism-bin |
| AC-EQUIV-001-EC-009-e2e-result-equivalence.tape | (VHS source) | — |
| SEC-004-armis-dtu-security-hardening.gif / .webm | SEC-004 defense-in-depth (Armis DTU, 5 tests) | prism-dtu-armis |
| SEC-004-armis-dtu-security-hardening.tape | (VHS source) | — |

---

## Test Execution Summary

All tests run against actual implementation on branch `feature/S-DEMO-QUERY-PUSHDOWN-001`. Story converged at `69aafcc7`; SEC-004 security hardening added at `f290a43d` (pass-1). Test counts reflect the feature HEAD `f290a43d`.

### prism-query (AC-WIRE-001, AC-WIRE-001b, AC-ARMIS-TW-001, AC-ARMIS-TW-003)

```
cargo nextest run -p prism-query -E 'test(ac_wire_001) or test(ac_armis_tw_001) or test(ac_armis_tw_003)' --no-fail-fast
Summary [0.048s] 6 tests run: 6 passed, 927 skipped

  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_wire_001_materialization_pipeline_populates_start_time_from_ast
  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_wire_001b_safe_default_when_spec_map_is_none
  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_armis_tw_001_time_window_augmented_into_aql
  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_armis_tw_001_bounded_range_after_and_before
  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_armis_tw_003_anti_double_filter_guard
  PASS prism-query pushdown::pushdown_red_gate_tests::test_ac_armis_tw_003_no_time_bounds_passes_through_verbatim
```

### prism-spec-engine (AC-CWS-001/002/003, AC-ARMIS-001/002, AC-CYB-001, AC-CLAR-001, AC-INDEX-001, AC-EQUIV-001 boundary, AC-ARMIS-TW-002, AC-ARMIS-TW-004)

```
cargo nextest run -p prism-spec-engine [selected ACs] --no-fail-fast
Summary [7.077s] 572 tests run: 572 passed, 12 skipped

  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_cws_002_wire_level_fql_both_bounds_via_pipeline_executor
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_cws_003_no_filter_param_when_no_time_predicates
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_armis_002_no_additional_params_beyond_aql_offset_limit
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_index_001_armis_toml_last_seen_created_at_have_index_option
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option
  PASS prism-spec-engine::bc_2_11_007_pushdown_test test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary
  PASS prism-spec-engine::parity_armis test_ac_armis_tw_002_dtu_filters_fixture_by_time_window
  PASS prism-spec-engine::parity_armis test_ac_armis_tw_004_result_equivalence_pushdown_vs_postfilter
```

### prism-spec-engine AC-ARMIS-TW-005 (ignored; run via --run-ignored)

```
cargo nextest run -p prism-spec-engine -E 'test(armis_tw_005)' --run-ignored ignored-only
Summary [0.040s] 1 test run: 1 passed, 583 skipped

  PASS prism-spec-engine::parity_armis test_ac_armis_tw_005_e2e_aql_log_contains_augmented_aql
```

### prism-dtu-crowdstrike (AC-CWS-DTU-001 + SEC-004 hardening)

```
cargo nextest run -p prism-dtu-crowdstrike --no-fail-fast
Summary [0.011s] 8 tests run: 8 passed, 0 skipped

  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_time_window
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_parse_fql_rfc3339_plus_offset_parses
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_parse_fql_naive_timestamp_parses
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_parse_fql_absent_filter_returns_none
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_parse_fql_malformed_filter_returns_none
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_crowdstrike_dtu_honors_fql_filter_after_only
  PASS prism-dtu-crowdstrike state::tests::test_ac_cws_dtu_001_parse_fql_before_only
  PASS prism-dtu-crowdstrike state::tests::test_sec_004_fql_over_length_token_returns_none  ← SEC-004 hardening @ f290a43d
```

### prism-dtu-armis (DTU parse unit tests + SEC-004 hardening)

```
cargo nextest run -p prism-dtu-armis --no-fail-fast
Summary [0.011s] 5 tests run: 5 passed, 0 skipped

  PASS prism-dtu-armis routes::search::pushdown_dtu_red_gate_tests::test_ac_armis_tw_002_dtu_parse_aql_after_clause_yields_bound
  PASS prism-dtu-armis routes::search::pushdown_dtu_red_gate_tests::test_ac_armis_tw_002_dtu_parse_aql_before_clause_yields_bound
  PASS prism-dtu-armis routes::search::pushdown_dtu_red_gate_tests::test_ac_armis_tw_002_dtu_parse_aql_bounded_range_yields_both_bounds
  PASS prism-dtu-armis routes::search::pushdown_dtu_red_gate_tests::test_ac_armis_tw_002_dtu_parse_aql_no_time_clause_returns_none
  PASS prism-dtu-armis routes::search::pushdown_dtu_red_gate_tests::test_sec_004_aql_over_length_token_returns_none  ← SEC-004 hardening @ f290a43d
```

### prism-bin (AC-CWS-002 e2e, AC-EQUIV-001, EC-009)

```
cargo nextest run -p prism-bin -E 'test(ac_cws_002) or test(ac_equiv_001) or test(adv_p08) or test(adv_p02)' --no-fail-fast
Summary [5.209s] 8 tests run: 8 passed, 125 skipped

  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p02_e2e_crowdstrike_fql_from_where_predicate
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate
  PASS prism-bin::adv_p02_e2e_pushdown_pipeline_test test_adv_p02_sid1_armis_fetch_start_time_augments_aql
```

---

## Key Behavioral Claims Verified

### CrowdStrike FQL wire-level push-down

- `run_materialization_pipeline` (ADR-033 T1) extracts `WHERE created_timestamp > 'T'` and `< 'T'` predicates from the PrismQL AST before fan-out.
- CrowdStrike adapter injects `created_timestamp:>'T'+created_timestamp:<'T'` as a single `filter` query param on Step 1 (`query_detection_ids`).
- Step 2 (`fetch_detections`) receives `FetchContext::default()` — no push-down.
- CrowdStrike DTU (`prism-dtu-crowdstrike`) parses the FQL `filter` param via `parse_fql_time_bounds` and filters its fixture dataset: `filtered_count < unfiltered_count` (LOAD-BEARING non-vacuous assertion).
- DTU `/dtu/filter-log` capture route records the applied filter expression.

### Armis AQL augmentation

- `augment_armis_aql_with_time_window` appends `after:YYYY-MM-DDTHH:MM:SS` (lower bound) and `before:YYYY-MM-DDTHH:MM:SS` (upper bound) to the base AQL string.
- Form is bare, unquoted, timezone-naive per `armis-aql-time-window-syntax-2026-06.md` §2.2. `lastSeen:>"T"` form is NOT used (unattested).
- Anti-double-filter guard: if base AQL already contains `after:`, `before:`, or `timeFrame:`, the AQL is forwarded verbatim with no augmentation.
- Armis DTU parses `after:`/`before:` clauses and filters device fixture by `last_seen` (with `first_seen` fallback): `filtered_count < unfiltered_count` (LOAD-BEARING).
- No `maxResults` or `timeFrame` params injected (REMOVAL verified by absence assertion).

### Cyberint / Claroty correctness removals

- Cyberint `from_date`/`to_date` POST-body injection: REMOVED. Endpoint is GET with cursor-only; `AlertListParams` has no `from_date`, `to_date`, or `page_size`.
- Claroty time-window body injection: REMOVED. `body_template: '{}'` remains empty. Pagination is OffsetLimit URL params only.

### Result-equivalence invariant (BC-2.11.007)

- AC-EQUIV-001 (CrowdStrike): rows returned via push-down path are a subset of rows returned without push-down for the same time range and LIMIT. No row fabrication or silent drop beyond what the predicate specifies. Test exercises real `run_materialization_pipeline` path.
- AC-ARMIS-TW-004 (Armis): result sets from push-down path and DataFusion post-filter path are identical (order-independent). BC-2.11.007 invariant confirmed.

### EC-009 inclusive boundary

- DTU time-window filtering is inclusive at the boundary: records with `ts == bound` are KEPT.
- `to_rfc3339_opts(SecondsFormat::Secs, true)` (`Z` suffix) used instead of `to_rfc3339()` (`+00:00`): `+00:00 < Z` lexicographically, which caused exact-boundary records to be silently dropped at DataFusion string-comparison until this fix.
- Red Gate tests `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` and `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` both PASS.

---

## Policy Compliance

- **POLICY 10 (story-scoped path):** All evidence in `docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/`. No flat `docs/demo-evidence/*.md` files.
- **No AI attribution in recordings:** VHS tapes contain only actual `cargo nextest` invocations against the live codebase.
- **Evidence accuracy:** All recordings show actual test runner output from the branch. AC recordings: converged commit `69aafcc7`. CrowdStrike DTU and Armis DTU recordings re-captured at feature HEAD `f290a43d` (post SEC-004 hardening) showing 8/5 test counts. No staged or aspirational output.
- **VHS toolchain:** `vhs v0.10.0`, font `FiraCode Nerd Font Mono`.
