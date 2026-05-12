# Demo Evidence Index — S-PLUGIN-PREREQ-B

**Story:** S-PLUGIN-PREREQ-B — Real PipelineExecutor — HTTP Client, JSONPath, Fan-out, Paginate, 401-Retry  
**Story version:** v1.22  
**Evidence captured:** 2026-05-12  
**HEAD SHA at capture:** `b75f317e` (LOCAL CONVERGED — pass-16 final, 297/297 tests pass)  
**Crate:** `prism-spec-engine`  
**BC primary:** BC-2.16.002 v1.8  
**BC secondary:** BC-2.01.013 v1.6

---

## AC-Satisfaction Table

| AC | Title | Evidence File | Status |
|----|-------|---------------|--------|
| AC-1 | HTTP execution — real request per FetchStep, non-empty records | [AC-1-evidence.md](AC-1-evidence.md) | SATISFIED |
| AC-2 | Variable interpolation survives HTTP boundary (two-step pipeline) | [AC-2-evidence.md](AC-2-evidence.md) | SATISFIED |
| AC-3 | Cursor pagination — iterates pages until null cursor | [AC-3-evidence.md](AC-3-evidence.md) | SATISFIED |
| AC-4 | Offset pagination — iterates until short page | [AC-4-evidence.md](AC-4-evidence.md) | SATISFIED |
| AC-5 | 401 retry via AuthProvider — one retry, double-401 aborts | [AC-5-evidence.md](AC-5-evidence.md) | SATISFIED |
| AC-6 | Fan-out reuse — existing `fan_out_batches()` called, not duplicated | [AC-6-evidence.md](AC-6-evidence.md) | SATISFIED |
| AC-7 | Rate-limit hints — `tokio::time::sleep` inserted between calls | [AC-7-evidence.md](AC-7-evidence.md) | SATISFIED |
| AC-8 | DI-019 limit — truncates at 10K, sets `truncated: true` flag | [AC-8-evidence.md](AC-8-evidence.md) | SATISFIED |
| AC-9 | VP-PLUGIN-002 integration test passes against wiremock mock | [AC-9-evidence.md](AC-9-evidence.md) | SATISFIED |

All 9 ACs: **SATISFIED**.

---

## Red Gate Test Summary

All 64 Red Gate tests were defined as failing stubs before implementation. At HEAD `b75f317e`, 297 tests pass across `prism-spec-engine`. The 56 BC-2.16.002-anchored tests below are the load-bearing evidence set.

Full suite: `cargo nextest run -p prism-spec-engine --no-fail-fast` → `297 tests run: 297 passed, 1 skipped` (1 skipped = `#[ignore]` proptest variant; 1 leaky = known tokio thread-pool test, non-blocking).

### BC-2.16.002 + VP-PLUGIN-002 Tests (56 of 297)

| # | Test Name | File | Line | Status |
|---|-----------|------|------|--------|
| 1 | `test_BC_2_16_002_fan_out_250_ids_produces_3_batches` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 2 | `test_BC_2_16_002_fan_out_scalar_value_produces_single_batch` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 3 | `test_BC_2_16_002_extracts_all_variable_references_from_template` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 4 | `test_BC_2_16_002_auth_provider_trait_object_is_object_safe` | `src/auth_provider.rs` | 343 | PASS |
| 5 | `test_BC_2_16_002_json_escapes_values_in_body_context` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 6 | `test_BC_2_16_002_execute_auth_initial_failed_emits_event_with_detail` | `src/pipeline.rs` (unit) | — | PASS |
| 7 | `test_BC_2_16_002_execute_step_emits_auth_initial_failed_with_step_name_field` | `src/pipeline.rs` (unit) | — | PASS |
| 8 | `test_BC_2_16_002_fan_out_empty_array_produces_zero_batches` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 9 | `test_BC_2_16_002_auth_refresh_failed_emits_event_with_detail` | `src/pipeline.rs` (unit) | — | PASS |
| 10 | `test_BC_2_16_002_execute_step_emits_auth_initial_acquired_empty_with_step_name_field` | `src/pipeline.rs` (unit) | — | PASS |
| 11 | `test_BC_2_16_002_interpolates_step_variable_in_path_template` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 12 | `test_BC_2_16_002_auth_refresh_triggered_emits_event_with_step_name` | `src/pipeline.rs` (unit) | — | PASS |
| 13 | `test_BC_2_16_002_execute_step_emits_auth_initial_acquired_with_step_name_field` | `src/pipeline.rs` (unit) | — | PASS |
| 14 | `test_BC_2_16_002_auth_refresh_succeeded_emits_event_with_step_name` | `src/pipeline.rs` (unit) | — | PASS |
| 15 | `test_BC_2_16_002_auth_refresh_double_401_emits_event` | `src/pipeline.rs` (unit) | — | PASS |
| 16 | `test_BC_2_16_002_fan_out_exactly_batch_size_produces_one_batch` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 17 | `test_BC_2_16_002_percent_encodes_values_in_url_context` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 18 | `test_BC_2_16_002_template_without_variables_returns_unchanged` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 19 | `test_BC_2_16_002_validation_accepts_fan_out_batch_size_none` | `tests/bc_2_16_009_test.rs` | — | PASS |
| 20 | `test_BC_2_16_002_returns_e_spec_010_on_interpolation_failure` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 21 | `test_BC_2_16_002_validation_accepts_valid_dollar_dot_response_path` | `tests/bc_2_16_009_test.rs` | — | PASS |
| 22 | `test_BC_2_16_002_validation_accepts_fan_out_batch_size_one` | `tests/bc_2_16_009_test.rs` | — | PASS |
| 23 | `test_BC_2_16_002_validation_rejects_malformed_dollar_dot_response_path` | `tests/bc_2_16_009_test.rs` | — | PASS |
| 24 | `test_BC_2_16_002_validation_rejects_fan_out_batch_size_zero` | `tests/bc_2_16_009_test.rs` | — | PASS |
| 25 | `test_BC_2_16_002_cursor_unsupported_type_emits_structured_event` | `tests/pipeline_http_integration.rs` | 2175 | PASS |
| 26 | `test_BC_2_16_002_two_step_pipeline_step2_uses_step1_token` | `tests/bc_2_16_002_test.rs` | — | PASS |
| 27 | `test_BC_2_16_002_cursor_preview_handles_multi_byte_utf8_without_panic` | `tests/pipeline_http_integration.rs` | 2418 | PASS |
| 28 | `test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state` | `tests/pipeline_http_integration.rs` | 1876 | PASS |
| 29 | `test_BC_2_16_002_execute_coerces_numeric_cursor_to_string` | `tests/pipeline_http_integration.rs` | 1460 | PASS |
| 30 | `test_BC_2_16_002_execute_decodes_gzipped_response` | `tests/pipeline_http_integration.rs` | 1666 | PASS |
| 31 | `test_BC_2_16_002_execute_aborts_on_non_advancing_cursor` | `tests/pipeline_http_integration.rs` | 1312 | PASS |
| 32 | `test_BC_2_16_002_execute_derives_application_json_for_array_body` | `tests/pipeline_http_integration.rs` | 1384 | PASS |
| 33 | `test_BC_2_16_002_execute_fan_out_invokes_step_per_batch` | `tests/pipeline_http_integration.rs` | 775 | PASS |
| 34 | `test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500` | `tests/pipeline_http_integration.rs` | 2068 | PASS |
| 35 | `test_BC_2_16_002_execute_interpolates_body_template_and_derives_content_type` | `tests/pipeline_http_integration.rs` | 487 | PASS |
| 36 | `test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls` | `tests/pipeline_http_integration.rs` | 1151 | PASS |
| 37 | `test_BC_2_16_002_execute_interpolates_query_filter_in_path_template` | `tests/pipeline_http_integration.rs` | 974 | PASS |
| 38 | `test_BC_2_16_002_execute_iterates_cursor_pagination_until_null` | `tests/pipeline_http_integration.rs` | 298 | PASS |
| 39 | `test_BC_2_16_002_execute_only_final_step_records_in_pipeline_result` | `tests/pipeline_http_integration.rs` | 679 | PASS |
| 40 | `test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url` | `tests/pipeline_http_integration.rs` | 196 | PASS |
| 41 | `test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records` | `tests/pipeline_http_integration.rs` | 137 | PASS |
| 42 | `test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page` | `tests/pipeline_http_integration.rs` | 396 | PASS |
| 43 | `test_BC_2_16_002_execute_percent_encodes_opaque_cursor` | `tests/pipeline_http_integration.rs` | 584 | PASS |
| 44 | `test_BC_2_16_002_fanout_ambiguous_multi_array_emits_structured_event` | `tests/pipeline_http_integration.rs` | 2535 | PASS |
| 45 | `test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object` | `tests/pipeline_http_integration.rs` | 2704 | PASS |
| 46 | `test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected` | `tests/pipeline_http_integration.rs` | 2303 | PASS |
| 47 | `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` | `tests/pipeline_http_integration.rs` | 89 | PASS |
| 48 | `test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately` | `tests/pipeline_oauth_retry.rs` | 284 | PASS |
| 49 | `test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap` | `tests/pipeline_http_integration.rs` | 1729 | PASS |
| 50 | `test_BC_2_16_002_execute_aborts_on_double_401` | `tests/pipeline_oauth_retry.rs` | 158 | PASS |
| 51 | `test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request` | `tests/pipeline_oauth_retry.rs` | 214 | PASS |
| 52 | `test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution` | `tests/pipeline_oauth_retry.rs` | 337 | PASS |
| 53 | `test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401` | `tests/pipeline_oauth_retry.rs` | 75 | PASS |
| 54 | `test_BC_2_16_002_execute_truncates_at_10k_with_truncated_flag_set` | `tests/pipeline_http_integration.rs` | 1048 | PASS |
| 55 | `test_BC_2_16_002_execute_aborts_at_max_pages_per_step` | `tests/pipeline_http_integration.rs` | 1558 | PASS |
| 56 | `test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls` | `tests/pipeline_http_integration.rs` | 877 | PASS |

**56/56 PASS.** Full suite: `297 tests run: 297 passed, 1 skipped`.

---

## Adversarial Convergence Summary

| Metric | Value |
|--------|-------|
| Total LOCAL adversary passes | 16 |
| Total fix-burst closure reports | 13 |
| Convergence streak | 3/3 (passes 14, 15, 16 each 0 genuine findings) |
| Convergence date | 2026-05-12 |
| Convergence status | LOCAL CONVERGED |

**Finding trajectory (genuine findings per pass):**

Pass 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12 → 13 → 14 → 15 → 16

`20 → 10 → 4 → 7 → 10 → 9 → 8 → 4 → 4 → 2 → 3 → 3 → 2 → 0 → 0 → 0`

Key convergence events:
- Pass 1 (F-LP1): 20 genuine findings — initial Red Gate stubs; executor skeleton had rate-limit flag scoped inside steps loop (HIGH) and fan-out detection returned only value not key (HIGH)
- Pass 5 (F-LP5): 10 genuine findings — eager token acquisition design confirmed; lazy-token pattern replaced
- Pass 10 (F-LP10): 2 findings — MockAuthProvider.token/call_count field visibility; ChainAuthProvider OOB behavior
- Pass 13 (F-LP13): 2 findings — BC catalog coverage gap (14/14 rows); cursor coercion edge case
- Passes 14–16: 0 genuine findings each — convergence streak 3/3 achieved

---

## Tech-Debt Carry-Forward

| ID | Description | Severity | Target |
|----|-------------|----------|--------|
| TD-006 | `AuthToken` does not implement `zeroize::Zeroize` on Drop | P3 | PREREQ-D (credential store integration) |
| TD-016 | `MAX_REQUESTS_PER_PIPELINE` global bound deferred (only per-step bound implemented) | P3 | Wave 1 |
| TD-093 | Production `PipelineExecutor` callers must construct `reqwest::Client` with explicit 30s timeout | P2 | PREREQ-D / chassis boot |

---

## BC Versions Cited

| BC | Version | Role |
|----|---------|------|
| BC-2.16.002 — Multi-Step Fetch Pipeline Execution | v1.8 (14-row Structured Event Catalog) | Primary authority for all 9 ACs |
| BC-2.01.013 — DataSource Trait: Spec-Driven Adapter Pattern | v1.6 | Secondary authority for AC-5 (AuthProvider trait) |

---

## Process-Gap Codifications

**PG-LP11-001** — "truncate_at_char_boundary helper must be unit-tested independent of the pipeline integration tests" — codified at `.factory/code-delivery/S-PLUGIN-PREREQ-B/adversarial-review/cycles/wave-4-operations/lessons.md`.

This gap was identified during pass-11 when the helper was only tested via the integration path (pipeline truncation test) rather than directly. Fix-burst F-LP11-L001 added 6 direct unit tests in `src/pipeline.rs #[cfg(test)]` block.
