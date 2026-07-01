# S-PERF-GATE-007 PR Evidence Bundle

**Story:** S-PERF-GATE-007 — nextest cap groups for uncapped WASMtime + HTTP binaries  
**Branch:** feature/S-PERF-GATE-007  
**Config change commit:** 2d11f540de9e3d555aec7e8258b8e56c2033de4b (`.config/nextest.toml`: wasm-cap + http-cap groups)  
**Current PR HEAD:** cf065761 (evidence commits on top of the config change)  
**Machine:** 16-core warm dev machine (macOS aarch64)  
**Baseline SHA:** develop@8bc0404e (post-S-PERF-GATE-005)

---

> **POL-10 note:** This is a manually-authored config-story evidence bundle (no VHS/Playwright demo-recorder output was produced for this story). POL-10's `evidence-report.md` naming convention applies to demo-recorder artifacts; this manual bundle retains `evidence.md`.

## Robust, Reproducible Outcomes (Lead)

These outcomes are the primary evidence for the caps. They are reproducible across
runs, not artefacts of specific machine-load conditions:

1. **TMT failures: 28 → 0.** Under high-contention conditions (many concurrent
   processes), uncapped WASMtime binaries caused Cranelift JIT initialization to
   starve and hit the 180s nextest hard timeout. With the caps applied, zero tests
   hit the timeout even under heavy contention.

2. **`just check` completes in ~2-7 min warm.** After the caps, `just check` completes
   reliably in the expected warm range. Measured post-cap `just check` (full workspace)
   runs: 127s on a clean pre-push run, 407s on a heavier-load run (Table 1, `just check`
   total row). Both are TMT-free.

3. **Modeled attributable scheduling savings from the caps: ~190-260s** (~1.5-1.8x
   on the nextest execution component). This figure is from the profiling model
   (`.factory/research/test-suite-perf-profile-2026-06-30.md` REC-1 + REC-4) and
   represents the expected cap-specific improvement from eliminating WASMtime/HTTP
   scheduling contention.

---

## Measured Results (AC-008 / AC-005 / AC-009)

### Table 1 — nextest wall-clock: TMT-free profiling baseline vs. post-cap

These two runs are both TMT-free (all tests green). The delta reflects cache/contention
differences between the 2026-06-30 profiling run and the post-cap verification run
IN ADDITION TO the cap-scheduling improvement. The excess above the modeled ~190-260s
prediction is not attributable to any specific mechanism (see Honest Framing below).

| Metric | Before (TMT-free baseline) | After (this change) | Measured delta |
|--------|---------------------------|---------------------|----------------|
| nextest wall-clock (`--profile prepush`) | 585.84s | 108.4s | -477s (5.4x) |
| `just check` total | ~798s | 407.3s | -391s (~1.96x) |
| Tests passed / skipped | 4976 / 60 | 4976 / 60 | unchanged |
| non-exhaustive gate EXPECTED | 87 | 87 | unchanged |

**Post-cap run commands:** nextest row (`--profile prepush`) — `cargo nextest run --workspace --all-features --profile prepush` (post-cap: 108.4s); `just check` row — `just check` (full workspace: fmt + clippy + nextest + doctests + crate-layout) (post-cap: 407.3s). A separate lighter-load post-cap `just check` run also measured 127s.

### Table 2 — TMT failure elimination (separate, heavier-contention run)

This run is DISTINCT from the 585.84s baseline in Table 1. Provenance: ~2026-06-30/07-01,
16 logical cores (macOS aarch64), command `just check` (full workspace), before the cap
fix was applied. Under high contention, 28 tests hit the 180s hard timeout; the run
completed only ~3600/5082 tests before stopping (wall-clock ~31 min at interruption).
Do NOT apply 28×180=5040s arithmetic against the 477s delta from Table 1 — these are
endpoints from different runs under different conditions.

| Metric | Before caps (contended run) | After caps |
|--------|-----------------------------|------------|
| TMT failures (180s hard timeout) | 28 | **0** |

---

## Honest Framing — Measurement Provenance

**Both the 585.84s baseline and the 108.4s post-cap run in Table 1 are TMT-free.**
The 585.84s profiling-report baseline (`.factory/research/test-suite-perf-profile-2026-06-30.md`,
develop@8bc0404e) was a GREEN run: 4976 passed, 60 skipped, zero timing-out tests. The
108.4s post-cap run (`cargo nextest run --workspace --all-features --profile prepush`,
16 logical cores, macOS aarch64) is also GREEN: 4976 passed, 60 skipped, zero TMT tests.

**The 28 TMT failures in Table 2 come from a SEPARATE, heavier-contention run.**
During S-PERF-GATE-007 delivery (the PR #208 resume sequence), a full-workspace
`just check` run taken BEFORE the cap fix was applied observed 28 tests hitting the
180s nextest hard timeout. Under that heavier-contention condition (more concurrent
processes competing for the same 16 CPUs), uncapped WASMtime binaries caused Cranelift
JIT initialization to starve. That run's wall-clock is NOT the 585.84s baseline.

**The ~477s measured delta substantially exceeds the scheduling model prediction.**
The profiling report's REC-1 + REC-4 predicted ~190-260s of scheduling savings from
the WASMtime/HTTP cap groups. The measured delta (~477s) is ~220-290s larger. This
excess is NOT attributable to the caps alone, and NOT attributable to TMT-elimination
(the 585.84s baseline had zero TMT tests). Candidate causes: cache and contention
differences between the 2026-06-30 profiling run and the post-cap verification run
(different machine load state, warm-cache state, other concurrent processes at
measurement time). This excess is presently unexplained and should not be attributed
to any specific mechanism. For future tuning decisions, the caps' standalone scheduling
value is the modeled ~190-260s figure; the full 5.4x / ~477s is the measured outcome
under the specific conditions of these two runs.

---

## AC-009 Binary-Resolution Proof — Both Profiles

Commands:
- `cargo nextest show-config test-groups --profile prepush`
- `cargo nextest show-config test-groups --profile ci`

All 11 capped binaries resolve to their assigned groups with non-empty test lists
on BOTH profiles. Raw output below.

### Profile: prepush

#### spec-engine-http-cap (4 binaries, max threads = 4)

```
group: spec-engine-http-cap (max threads = 4)
  * override for prepush profile with filter 'binary(pipeline_http_integration) | binary(pipeline_oauth_retry) | binary(bc_2_11_007_pushdown_test) | binary(bc_2_16_002_crowdstrike_two_step)':
      prism-spec-engine::bc_2_11_007_pushdown_test:
          test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe
          test_ac_armis_002_no_additional_params_beyond_aql_offset_limit
          test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields
          test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params
          test_ac_cws_002_wire_level_fql_both_bounds_via_pipeline_executor
          test_ac_cws_003_no_filter_param_when_no_time_predicates
          test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu
          test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params
          test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary
          test_ac_index_001_armis_toml_last_seen_created_at_have_index_option
          test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option
      prism-spec-engine::bc_2_16_002_crowdstrike_two_step:
          test_BC_2_16_002_crowdstrike_batch_boundary_100_ids_one_batch
          test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec
          test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven
      prism-spec-engine::pipeline_http_integration:
          test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state
          test_BC_2_16_002_cursor_preview_handles_multi_byte_utf8_without_panic
          test_BC_2_16_002_cursor_unsupported_type_emits_structured_event
          test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap
          test_BC_2_16_002_execute_aborts_at_max_pages_per_step
          test_BC_2_16_002_execute_aborts_on_non_advancing_cursor
          test_BC_2_16_002_execute_coerces_numeric_cursor_to_string
          test_BC_2_16_002_execute_decodes_gzipped_response
          test_BC_2_16_002_execute_derives_application_json_for_array_body
          test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500
          test_BC_2_16_002_execute_fan_out_invokes_step_per_batch
          test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls
          test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls
          test_BC_2_16_002_execute_interpolates_body_template_and_derives_content_type
          test_BC_2_16_002_execute_interpolates_query_filter_in_path_template
          test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url
          test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records
          test_BC_2_16_002_execute_iterates_cursor_pagination_until_null
          test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page
          test_BC_2_16_002_execute_only_final_step_records_in_pipeline_result
          test_BC_2_16_002_execute_percent_encodes_opaque_cursor
          test_BC_2_16_002_execute_truncates_at_10k_with_truncated_flag_set
          test_BC_2_16_002_fanout_ambiguous_multi_array_emits_structured_event
          test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object
          test_BC_2_16_002_pipeline_cumulative_request_cap_exercised_via_wiremock
          test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected
          test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock
      prism-spec-engine::pipeline_oauth_retry:
          test_BC_2_01_017_dtu_401_surfaces_e_auth_004_no_retry
          test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately
          test_BC_2_16_002_execute_aborts_on_double_401
          test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request
          test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401
          test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution
```

#### spec-engine-wasm-cap (7 binaries, max threads = 4)

```
group: spec-engine-wasm-cap (max threads = 4)
  * override for prepush profile with filter 'binary(plugin_integration_tests) | binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(spec_driven_mapper_fixtures) | binary(plugin_boot_tests) | binary(infusion_boot_integration)':
      prism-bin::infusion_boot_integration:
          test_boot_infusion_load_step_empty_dir_returns_empty_registry
          test_boot_plugin_infusion_spec_wired_with_real_plugin_source_not_null_source
          test_boot_with_csv_infusion_udf_query_resolves
          test_boot_with_csv_infusion_udf_resolves
          test_infusion_tier3_production_read_without_source
      prism-bin::plugin_boot_tests:
          test_AC_4_VP_PLUGIN_004_unsigned_plugin_durable_audit_entry
          test_BC_2_16_012_plugin_runtime_registers_write_tools_pre_query_phase
          test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin
          test_BC_2_16_012_write_tool_reg_failure_rolls_back_plugin
          test_BC_2_22_001_boot_step_plugin_load_placement
          test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config
          test_BC_2_22_001_plugin_load_disabled_env
          test_BC_2_22_001_plugin_load_failure_exits_code_4
          test_BC_2_22_001_plugin_load_step_is_registered_between_step7_and_step8
          test_BC_2_22_001_prism_config_plugin_dir_default_and_explicit
          test_F_PASS2_CRIT_001_prism_command_start_routes_through_run_boot_sequence
          test_F_PASS3_CRIT_001_plugin_load_runs_before_step7
          test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit
          test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded
          test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist
          test_boot_error_unknown_auth_plugin_exits_code_2
          test_plugin_auth_provider_construction_production_api
          test_validate_and_construct_auth_providers_empty_returns_empty_map
          test_validate_and_construct_auth_providers_happy_path
          test_validate_and_construct_auth_providers_mixed_sensors_one_with_auth_plugin
          test_validate_and_construct_auth_providers_typo_returns_error
          test_validate_auth_plugin_fields_passes_when_no_auth_plugin
          test_validate_auth_plugin_fields_rejects_unregistered_plugin
      prism-ocsf::spec_driven_mapper_fixtures:
          test_BC_2_02_002_spec_driven_identity_passthrough
          test_BC_2_02_002_spec_driven_int_to_string_cast
          test_BC_2_02_002_spec_driven_nullable_propagation
          test_BC_2_02_002_spec_driven_rfc3339_timestamp
          test_BC_2_02_002_spec_driven_string_to_string
          test_BC_2_02_007_spec_driven_extensions_preserved
          test_F_LP2_HIGH_001_json_null_value_placed_in_extensions_not_corrupted_to_string_null
          test_F_LP2_HIGH_002_dynamic_message_field_value_written_to_real_descriptor
          test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern
          test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed
          test_PLUGIN_MIGRATION_001_C_005_no_hardcoded_mapper_symbols_in_production_src
          test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases
          test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper
      prism-spec-engine::crowdstrike_oauth2_plugin_tests:
          test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates
          test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials
          test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint
          test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request
          test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition
          test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry
          test_PLUGIN_MIGRATION_001_E_007_crowdstrike_toml_declares_auth_plugin
          test_PLUGIN_MIGRATION_001_E_007b_unknown_auth_plugin_emits_e_spec_012
          test_PLUGIN_MIGRATION_001_E_007c_registered_auth_plugin_passes_validation
          test_PLUGIN_MIGRATION_001_E_007d_no_auth_plugin_field_passes_validation
          test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment
          test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn
          test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output
          test_PLUGIN_MIGRATION_001_E_crit_001_kv_store_arc_shared_across_dispatches
          test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime
          test_PLUGIN_MIGRATION_001_E_task1_sensor_spec_without_auth_plugin_parses_to_none
          test_PLUGIN_MIGRATION_001_F_crowdstrike_oauth2_plugin_dispatch_via_toml
          test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log
          test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed
      prism-spec-engine::enrichment_pivot_002_tests:
          test_enrichment_pivot_002_ac003_plugin_infusion_source_real_path
          test_enrichment_pivot_002_bc2_19_001_duplicate_udf_name_rejected
          test_enrichment_pivot_002_bc2_19_001_zero_fields_spec_rejected
          test_enrichment_pivot_002_credential_resolution_failed_excludes_env_var_name
          test_enrichment_pivot_002_e_infuse_013_sc3_base_url_empty_returns_invalid_field_spec
          test_enrichment_pivot_002_e_infuse_013_sc5_response_path_empty_returns_invalid_field_spec
          test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves
          test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs
          test_enrichment_pivot_002_high1_crit2b_plugin_infusion_source_canonical_identity_resolves
          test_enrichment_pivot_002_high1_crit2b_threat_intel_canned_fixture_end_to_end
          test_enrichment_pivot_002_http_lookup_failed_error_format_excludes_credentials
          test_enrichment_pivot_002_http_lookup_infusion_type_parses_nvd_spec
          test_enrichment_pivot_002_http_lookup_parse_rejects_invalid_method
          test_enrichment_pivot_002_http_lookup_parse_rejects_missing_input_placeholder
          test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx
          test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found
          test_enrichment_pivot_002_nvd_http_lookup_resolves_scenario_cve_high_cvss
          test_enrichment_pivot_002_nvd_plugin_crate_removed
          test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs
          test_enrichment_pivot_002_plugin_enrich_call_failed_maps_to_infusion_error
          test_enrichment_pivot_002_sap2_nvd_toml_columns_match_dtu_fields
          test_enrichment_pivot_002_sap2_threatintel_toml_columns_match_dtu_fields
          test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers
          test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit
          test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars
          test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking
          test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path
          test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub
          test_enrichment_pivot_002_sec003_load_all_rejects_traversal_plugin_ref_production_path
          test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref
          test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted
          test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log
          test_enrichment_pivot_002_sec003_symlink_escape_rejected_by_canonicalize_guard
          test_enrichment_pivot_002_ssrf_accepts_private_base_url_with_dtu_mode
          test_enrichment_pivot_002_ssrf_rejected_error_excludes_resolved_ip
          test_enrichment_pivot_002_ssrf_rejects_private_base_url_without_dtu_mode
          test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious
          test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs
          test_enrichment_pivot_002_val_lift_fix_option_none_returns_ok_none
          test_enrichment_pivot_002_val_lift_fix_option_some_returns_json_value
          test_enrichment_pivot_002_val_lift_fix_unexpected_val_returns_enrich_call_failed
      prism-spec-engine::plugin_integration_tests:
          test_BC_2_16_002_pipeline_max_requests_exceeded
          test_BC_2_17_001_plugin_panic_isolation
          test_BC_2_17_002_allowlist_enforcement_allows_listed_url
          test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url
          test_BC_2_17_002_linker_imports_match_host_functions
          test_BC_2_17_002_wasi_not_linked_trap_on_fs_call
          test_BC_2_17_003_memory_limit_enforced_default_64mb
          test_BC_2_17_004_cpu_timeout_enforced_infinite_loop
          test_BC_2_17_006_duplicate_plugin_id_first_wins
          test_BC_2_17_006_wit_validation_rejects_missing_export
          test_BC_2_17_007_absent_format_version_is_rejected_e019
          test_BC_2_17_007_empty_allowed_url_entry_is_rejected
          test_BC_2_17_007_malformed_toml_manifest_returns_parse_error_e017
          test_BC_2_17_007_manifest_format_version_exceeded_rejected
          test_BC_2_17_007_manifest_missing_allowed_urls_rejected
          test_BC_2_17_007_manifest_name_empty_rejected
          test_BC_2_17_007_manifest_version_malformed_rejected
          test_BC_2_17_007_plugin_without_manifest_returns_not_found_e018
          test_BC_2_17_007_strict_semver_rejects_partial_versions
          test_F_PASS2_CRIT_002_http_request_callback_delegates_to_allowlist_gate
          test_F_PASS2_CRIT_002_log_callback_delegates_to_host_log
          test_F_PASS2_HIGH_003_kv_set_err_propagated_not_swallowed
          test_F_PASS2_HIGH_003_kv_set_within_limit_returns_ok
          test_F_PASS3_CRIT_002_http_response_status_is_val_u16_not_val_u32
          test_F_PASS3_CRIT_002_log_level_is_val_enum_not_val_u8
          test_F_PASS3_CRIT_003_component_model_dispatch_allowlist_gate
          test_F_PASS3_MED_002_HIGH_001_log_callback_schema_violation_and_unrecognized_enum
          test_F_PASS3_MED_002_schema_violation_wrong_val_type_traps_not_silently_defaults
          test_F_PASS4_HIGH_001_component_model_dispatch_invokes_host_http_request_through_registered_callback
          test_F_PASS5_HIGH_001_production_linker_dispatch_via_build_linker_route_a
          test_TD_S_PLUGIN_PREREQ_B_002_authtoken_uses_zeroizing_wrapper
          test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once
          test_hot_reload_atomic_swap_success
          test_hot_reload_failed_recompile_retains_old
      prism-spec-engine::plugin_tests:
          test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped
          test_BC_2_17_001_ec17_001_trap_on_first_call_plugin_stays_registered
          test_BC_2_17_001_ec17_003_batch_trap_returns_no_partial_results
          test_BC_2_17_001_ec17_004_concurrent_traps_independent
          test_BC_2_17_002_ac4_wasi_filesystem_not_accessible
          test_BC_2_17_002_ac5_http_request_proxied_via_host
          test_BC_2_17_002_ac8_kv_store_scoped_per_plugin
          test_BC_2_17_002_ec17_006_http_request_allowlisted_url_succeeds
          test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked
          test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403
          test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err
          test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds
          test_BC_2_17_003_ec17_011_per_plugin_memory_override
          test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout
          test_BC_2_17_004_ec17_015_per_plugin_timeout_override
          test_BC_2_17_005_ac6_hot_reload_atomic_swap
          test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin
          test_BC_2_17_005_ec17_delete_plugin_new_calls_return_not_loaded
          test_BC_2_17_006_ac1_load_valid_infusion_plugin
          test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001
          test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit
          test_BC_2_17_006_ec17_026_bulk_discovery_partial_failure
          test_BC_2_17_006_ec17_027_empty_plugin_id_rejected
          test_F6_body_read_failure_maps_to_synthetic_error_status
```

**prepush: 11/11 capped binaries resolve. Zero zero-match filters.**

---

### Profile: ci

#### spec-engine-http-cap (4 binaries, max threads = 4)

```
group: spec-engine-http-cap (max threads = 4)
  * override for ci profile with filter 'binary(pipeline_http_integration) | binary(pipeline_oauth_retry) | binary(bc_2_11_007_pushdown_test) | binary(bc_2_16_002_crowdstrike_two_step)':
      prism-spec-engine::bc_2_11_007_pushdown_test:
          test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe
          test_ac_armis_002_no_additional_params_beyond_aql_offset_limit
          test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields
          test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params
          test_ac_cws_002_wire_level_fql_both_bounds_via_pipeline_executor
          test_ac_cws_003_no_filter_param_when_no_time_predicates
          test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu
          test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params
          test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary
          test_ac_index_001_armis_toml_last_seen_created_at_have_index_option
          test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option
      prism-spec-engine::bc_2_16_002_crowdstrike_two_step:
          test_BC_2_16_002_crowdstrike_batch_boundary_100_ids_one_batch
          test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec
          test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven
      prism-spec-engine::pipeline_http_integration:
          test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state
          test_BC_2_16_002_cursor_preview_handles_multi_byte_utf8_without_panic
          test_BC_2_16_002_cursor_unsupported_type_emits_structured_event
          test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap
          test_BC_2_16_002_execute_aborts_at_max_pages_per_step
          test_BC_2_16_002_execute_aborts_on_non_advancing_cursor
          test_BC_2_16_002_execute_coerces_numeric_cursor_to_string
          test_BC_2_16_002_execute_decodes_gzipped_response
          test_BC_2_16_002_execute_derives_application_json_for_array_body
          test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500
          test_BC_2_16_002_execute_fan_out_invokes_step_per_batch
          test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls
          test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls
          test_BC_2_16_002_execute_interpolates_body_template_and_derives_content_type
          test_BC_2_16_002_execute_interpolates_query_filter_in_path_template
          test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url
          test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records
          test_BC_2_16_002_execute_iterates_cursor_pagination_until_null
          test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page
          test_BC_2_16_002_execute_only_final_step_records_in_pipeline_result
          test_BC_2_16_002_execute_percent_encodes_opaque_cursor
          test_BC_2_16_002_execute_truncates_at_10k_with_truncated_flag_set
          test_BC_2_16_002_fanout_ambiguous_multi_array_emits_structured_event
          test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object
          test_BC_2_16_002_pipeline_cumulative_request_cap_exercised_via_wiremock
          test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected
          test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock
      prism-spec-engine::pipeline_oauth_retry:
          test_BC_2_01_017_dtu_401_surfaces_e_auth_004_no_retry
          test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately
          test_BC_2_16_002_execute_aborts_on_double_401
          test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request
          test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401
          test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution
```

#### spec-engine-wasm-cap (7 binaries, max threads = 4)

```
group: spec-engine-wasm-cap (max threads = 4)
  * override for ci profile with filter 'binary(plugin_integration_tests) | binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(spec_driven_mapper_fixtures) | binary(plugin_boot_tests) | binary(infusion_boot_integration)':
      prism-bin::infusion_boot_integration:
          test_boot_infusion_load_step_empty_dir_returns_empty_registry
          test_boot_plugin_infusion_spec_wired_with_real_plugin_source_not_null_source
          test_boot_with_csv_infusion_udf_query_resolves
          test_boot_with_csv_infusion_udf_resolves
          test_infusion_tier3_production_read_without_source
      prism-bin::plugin_boot_tests:
          test_AC_4_VP_PLUGIN_004_unsigned_plugin_durable_audit_entry
          test_BC_2_16_012_plugin_runtime_registers_write_tools_pre_query_phase
          test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin
          test_BC_2_16_012_write_tool_reg_failure_rolls_back_plugin
          test_BC_2_22_001_boot_step_plugin_load_placement
          test_BC_2_22_001_disable_env_takes_precedence_over_plugin_dir_config
          test_BC_2_22_001_plugin_load_disabled_env
          test_BC_2_22_001_plugin_load_failure_exits_code_4
          test_BC_2_22_001_plugin_load_step_is_registered_between_step7_and_step8
          test_BC_2_22_001_prism_config_plugin_dir_default_and_explicit
          test_F_PASS2_CRIT_001_prism_command_start_routes_through_run_boot_sequence
          test_F_PASS3_CRIT_001_plugin_load_runs_before_step7
          test_VP_PLUGIN_004_unsigned_plugin_boot_warn_audit
          test_VP_PLUGIN_007_plugin_load_rejected_format_version_exceeded
          test_VP_PLUGIN_007_plugin_load_rejected_no_allowlist
          test_boot_error_unknown_auth_plugin_exits_code_2
          test_plugin_auth_provider_construction_production_api
          test_validate_and_construct_auth_providers_empty_returns_empty_map
          test_validate_and_construct_auth_providers_happy_path
          test_validate_and_construct_auth_providers_mixed_sensors_one_with_auth_plugin
          test_validate_and_construct_auth_providers_typo_returns_error
          test_validate_auth_plugin_fields_passes_when_no_auth_plugin
          test_validate_auth_plugin_fields_rejects_unregistered_plugin
      prism-ocsf::spec_driven_mapper_fixtures:
          test_BC_2_02_002_spec_driven_identity_passthrough
          test_BC_2_02_002_spec_driven_int_to_string_cast
          test_BC_2_02_002_spec_driven_nullable_propagation
          test_BC_2_02_002_spec_driven_rfc3339_timestamp
          test_BC_2_02_002_spec_driven_string_to_string
          test_BC_2_02_007_spec_driven_extensions_preserved
          test_F_LP2_HIGH_001_json_null_value_placed_in_extensions_not_corrupted_to_string_null
          test_F_LP2_HIGH_002_dynamic_message_field_value_written_to_real_descriptor
          test_PLUGIN_MIGRATION_001_C_002_wasm_dispatch_called_for_complex_pattern
          test_PLUGIN_MIGRATION_001_C_003_missing_plugin_returns_normalization_failed
          test_PLUGIN_MIGRATION_001_C_005_no_hardcoded_mapper_symbols_in_production_src
          test_PLUGIN_MIGRATION_001_C_006_vp_plugin_006_fixture_catalog_six_cases
          test_PLUGIN_MIGRATION_001_C_007_normalizer_wired_with_spec_driven_mapper
      prism-spec-engine::crowdstrike_oauth2_plugin_tests:
          test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates
          test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials
          test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint
          test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request
          test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition
          test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry
          test_PLUGIN_MIGRATION_001_E_007_crowdstrike_toml_declares_auth_plugin
          test_PLUGIN_MIGRATION_001_E_007b_unknown_auth_plugin_emits_e_spec_012
          test_PLUGIN_MIGRATION_001_E_007c_registered_auth_plugin_passes_validation
          test_PLUGIN_MIGRATION_001_E_007d_no_auth_plugin_field_passes_validation
          test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment
          test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn
          test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output
          test_PLUGIN_MIGRATION_001_E_crit_001_kv_store_arc_shared_across_dispatches
          test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime
          test_PLUGIN_MIGRATION_001_E_task1_sensor_spec_without_auth_plugin_parses_to_none
          test_PLUGIN_MIGRATION_001_F_crowdstrike_oauth2_plugin_dispatch_via_toml
          test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log
          test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed
      prism-spec-engine::enrichment_pivot_002_tests:
          test_enrichment_pivot_002_ac003_plugin_infusion_source_real_path
          test_enrichment_pivot_002_bc2_19_001_duplicate_udf_name_rejected
          test_enrichment_pivot_002_bc2_19_001_zero_fields_spec_rejected
          test_enrichment_pivot_002_credential_resolution_failed_excludes_env_var_name
          test_enrichment_pivot_002_e_infuse_013_sc3_base_url_empty_returns_invalid_field_spec
          test_enrichment_pivot_002_e_infuse_013_sc5_response_path_empty_returns_invalid_field_spec
          test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves
          test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs
          test_enrichment_pivot_002_high1_crit2b_plugin_infusion_source_canonical_identity_resolves
          test_enrichment_pivot_002_high1_crit2b_threat_intel_canned_fixture_end_to_end
          test_enrichment_pivot_002_http_lookup_failed_error_format_excludes_credentials
          test_enrichment_pivot_002_http_lookup_infusion_type_parses_nvd_spec
          test_enrichment_pivot_002_http_lookup_parse_rejects_invalid_method
          test_enrichment_pivot_002_http_lookup_parse_rejects_missing_input_placeholder
          test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx
          test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found
          test_enrichment_pivot_002_nvd_http_lookup_resolves_scenario_cve_high_cvss
          test_enrichment_pivot_002_nvd_plugin_crate_removed
          test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs
          test_enrichment_pivot_002_plugin_enrich_call_failed_maps_to_infusion_error
          test_enrichment_pivot_002_sap2_nvd_toml_columns_match_dtu_fields
          test_enrichment_pivot_002_sap2_threatintel_toml_columns_match_dtu_fields
          test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers
          test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit
          test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars
          test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking
          test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path
          test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub
          test_enrichment_pivot_002_sec003_load_all_rejects_traversal_plugin_ref_production_path
          test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref
          test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted
          test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log
          test_enrichment_pivot_002_sec003_symlink_escape_rejected_by_canonicalize_guard
          test_enrichment_pivot_002_ssrf_accepts_private_base_url_with_dtu_mode
          test_enrichment_pivot_002_ssrf_rejected_error_excludes_resolved_ip
          test_enrichment_pivot_002_ssrf_rejects_private_base_url_without_dtu_mode
          test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious
          test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs
          test_enrichment_pivot_002_val_lift_fix_option_none_returns_ok_none
          test_enrichment_pivot_002_val_lift_fix_option_some_returns_json_value
          test_enrichment_pivot_002_val_lift_fix_unexpected_val_returns_enrich_call_failed
      prism-spec-engine::plugin_integration_tests:
          test_BC_2_16_002_pipeline_max_requests_exceeded
          test_BC_2_17_001_plugin_panic_isolation
          test_BC_2_17_002_allowlist_enforcement_allows_listed_url
          test_BC_2_17_002_allowlist_enforcement_blocks_non_allowlisted_url
          test_BC_2_17_002_linker_imports_match_host_functions
          test_BC_2_17_002_wasi_not_linked_trap_on_fs_call
          test_BC_2_17_003_memory_limit_enforced_default_64mb
          test_BC_2_17_004_cpu_timeout_enforced_infinite_loop
          test_BC_2_17_006_duplicate_plugin_id_first_wins
          test_BC_2_17_006_wit_validation_rejects_missing_export
          test_BC_2_17_007_absent_format_version_is_rejected_e019
          test_BC_2_17_007_empty_allowed_url_entry_is_rejected
          test_BC_2_17_007_malformed_toml_manifest_returns_parse_error_e017
          test_BC_2_17_007_manifest_format_version_exceeded_rejected
          test_BC_2_17_007_manifest_missing_allowed_urls_rejected
          test_BC_2_17_007_manifest_name_empty_rejected
          test_BC_2_17_007_manifest_version_malformed_rejected
          test_BC_2_17_007_plugin_without_manifest_returns_not_found_e018
          test_BC_2_17_007_strict_semver_rejects_partial_versions
          test_F_PASS2_CRIT_002_http_request_callback_delegates_to_allowlist_gate
          test_F_PASS2_CRIT_002_log_callback_delegates_to_host_log
          test_F_PASS2_HIGH_003_kv_set_err_propagated_not_swallowed
          test_F_PASS2_HIGH_003_kv_set_within_limit_returns_ok
          test_F_PASS3_CRIT_002_http_response_status_is_val_u16_not_val_u32
          test_F_PASS3_CRIT_002_log_level_is_val_enum_not_val_u8
          test_F_PASS3_CRIT_003_component_model_dispatch_allowlist_gate
          test_F_PASS3_MED_002_HIGH_001_log_callback_schema_violation_and_unrecognized_enum
          test_F_PASS3_MED_002_schema_violation_wrong_val_type_traps_not_silently_defaults
          test_F_PASS4_HIGH_001_component_model_dispatch_invokes_host_http_request_through_registered_callback
          test_F_PASS5_HIGH_001_production_linker_dispatch_via_build_linker_route_a
          test_TD_S_PLUGIN_PREREQ_B_002_authtoken_uses_zeroizing_wrapper
          test_TD_S_PLUGIN_PREREQ_B_011_execute_step_eager_token_calls_auth_once
          test_hot_reload_atomic_swap_success
          test_hot_reload_failed_recompile_retains_old
      prism-spec-engine::plugin_tests:
          test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped
          test_BC_2_17_001_ec17_001_trap_on_first_call_plugin_stays_registered
          test_BC_2_17_001_ec17_003_batch_trap_returns_no_partial_results
          test_BC_2_17_001_ec17_004_concurrent_traps_independent
          test_BC_2_17_002_ac4_wasi_filesystem_not_accessible
          test_BC_2_17_002_ac5_http_request_proxied_via_host
          test_BC_2_17_002_ac8_kv_store_scoped_per_plugin
          test_BC_2_17_002_ec17_006_http_request_allowlisted_url_succeeds
          test_BC_2_17_002_ec17_007_http_request_empty_allowlist_blocked
          test_BC_2_17_002_ec17_url_not_in_allowlist_returns_403
          test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err
          test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds
          test_BC_2_17_003_ec17_011_per_plugin_memory_override
          test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout
          test_BC_2_17_004_ec17_015_per_plugin_timeout_override
          test_BC_2_17_005_ac6_hot_reload_atomic_swap
          test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin
          test_BC_2_17_005_ec17_delete_plugin_new_calls_return_not_loaded
          test_BC_2_17_006_ac1_load_valid_infusion_plugin
          test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001
          test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit
          test_BC_2_17_006_ec17_026_bulk_discovery_partial_failure
          test_BC_2_17_006_ec17_027_empty_plugin_id_rejected
          test_F6_body_read_failure_maps_to_synthetic_error_status
```

**ci: 11/11 capped binaries resolve. Zero zero-match filters.**

**Binary resolution summary: 11/11 capped binaries resolve on BOTH prepush and ci profiles.**

---

## AC-001 through AC-007 Grep Verification

All grep counts verified against `.config/nextest.toml` on implementation run:

| AC | Command | Expected | Actual |
|----|---------|----------|--------|
| AC-001 | `grep -c 'spec-engine-wasm-cap = { max-threads = 4 }'` | 1 | 1 |
| AC-002 | `grep -c 'spec-engine-http-cap = { max-threads = 4 }'` | 1 | 1 |
| AC-003 | `grep -c "test-group = 'spec-engine-wasm-cap'"` | 2 | 2 |
| AC-004 | `grep -c "test-group = 'spec-engine-http-cap'"` | 2 | 2 |
| AC-005 | `grep -c 'bc_2_11_007_pushdown_test'` | 2 | 2 |
| AC-006 | `grep -c 'plugin_integration_tests'` | 2 | 2 |
| AC-007 | `grep -c 'serial-subprocess = { max-threads = 1 }'` | 1 | 1 |
| AC-007 | `grep -c 'adv-p02-serial = { max-threads = 1 }'` | 1 | 1 |
| AC-007 | `grep -c 'bc-2-01-013-serial = { max-threads = 1 }'` | 1 | 1 |
| AC-007 | `grep -c 'dtu-cap = { max-threads = 4 }'` | 1 | 1 |

All ACs pass.

---

## Files Modified

| File | Change |
|------|--------|
| `.config/nextest.toml` | +35 lines: 2 new `[test-groups]` entries + 2 `[[profile.prepush.overrides]]` + 2 `[[profile.ci.overrides]]` stanzas |

No production Rust code changes. No Justfile changes. No `.factory/` changes (state-manager handles STORY-INDEX registration post-merge per POL-14).

---

## LOCAL 3-CLEAN Adversarial Status

Story passed LOCAL 3-CLEAN before this evidence bundle was authored. The cap groups are
config-only changes; all 4976 tests pass under the new scheduling policy.
