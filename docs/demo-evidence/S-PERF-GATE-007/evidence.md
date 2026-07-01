# S-PERF-GATE-007 PR Evidence Bundle

**Story:** S-PERF-GATE-007 — nextest cap groups for uncapped WASMtime + HTTP binaries  
**Branch:** feature/S-PERF-GATE-007  
**Implementation commit:** 2d11f540de9e3d555aec7e8258b8e56c2033de4b  
**Machine:** 16-core warm dev machine (macOS aarch64)  
**Baseline SHA:** develop@8bc0404e (post-S-PERF-GATE-005)  

---

## Measured Results (AC-008 / AC-005 / AC-009)

### Before / After — nextest wall-clock and `just check`

| Metric | Before (baseline) | After (this change) | Delta |
|--------|-------------------|---------------------|-------|
| nextest wall-clock (`--profile prepush`) | 585.84s | 108.4s | **-477s (5.4x faster)** |
| `just check` total | ~798s | 407.3s | **-391s (~1.96x faster)** |
| TMT failures (180s hard timeout) | 28 | **0** | -28 |
| Tests passed / skipped | 4976 / 60 | 4976 / 60 | unchanged |
| non-exhaustive gate EXPECTED | 87 | 87 | **unchanged** |

### Honest Framing — Measurement Provenance (story spec v1.6 §PR Evidence Framing Note)

**Both the 585.84s baseline and the 108.4s post-cap run are TMT-free.** The 585.84s
profiling-report baseline (`.factory/research/test-suite-perf-profile-2026-06-30.md`,
develop@8bc0404e) was a GREEN run: 4976 passed, 60 skipped, zero timing-out tests. The
108.4s post-cap run is also GREEN: 4976 passed, 60 skipped, zero TMT tests.

**The 28 TMT failures come from a SEPARATE, heavier-contention run — NOT the 585.84s
baseline.** During S-PERF-GATE-007 delivery (the PR #208 resume sequence), a full-workspace
`just check` run taken BEFORE the cap fix was applied observed 28 tests hitting the 180s
nextest hard timeout. Under that heavier-contention condition (more concurrent processes
competing for the same 16 CPUs), uncapped WASMtime binaries caused Cranelift JIT
initialization to starve. That run's wall-clock is a SEPARATE measurement from the 585.84s
profiling baseline — it does NOT enter the 585.84→108.4 delta arithmetic. Do NOT perform
28×180=5040s arithmetic against the (585.84−108.4)=477s delta; these are endpoints from
two different runs under two different conditions. The 28-TMT heavier-contention run
provides CONTEXT showing the cap groups prevent TMT recurrence under high-contention
conditions; it is not the before-endpoint of the 5.4x headline.

**The ~477s delta substantially exceeds the scheduling model prediction.** The profiling
report's REC-1 + REC-4 predicted ~190-260s of scheduling savings from the WASMtime/HTTP
cap groups. The measured delta (~477s) is ~220-290s larger. This excess is NOT attributable
to TMT-elimination (the 585.84s baseline had zero TMT tests — there is nothing to eliminate
from the baseline endpoint). Candidate causes: cache and contention differences between the
2026-06-30 profiling run and the post-cap verification run (different machine load state,
warm-cache state, other concurrent processes at measurement time). This excess is presently
unexplained and should not be attributed to any specific mechanism. For future tuning
decisions, the caps' standalone scheduling value is the ~190-260s figure from the model;
the full 5.4x / ~477s improvement is the measured outcome under the specific conditions of
these two runs.

---

## AC-009 Binary-Resolution Proof

Command: `cargo nextest show-config test-groups --profile prepush`

All 11 capped binaries resolve to their assigned groups with non-empty test lists.

### spec-engine-wasm-cap (7 binaries, max threads = 4)

Filter: `binary(plugin_integration_tests) | binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(spec_driven_mapper_fixtures) | binary(plugin_boot_tests) | binary(infusion_boot_integration)`

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
          [41 tests — non-empty, resolves correctly]
      prism-spec-engine::plugin_integration_tests:
          [34 tests — non-empty, resolves correctly]
      prism-spec-engine::plugin_tests:
          [25 tests — non-empty, resolves correctly]
```

All 7 spec-engine-wasm-cap binaries resolve with non-empty test lists. No zero-match filters.

### spec-engine-http-cap (4 binaries, max threads = 4)

Filter: `binary(pipeline_http_integration) | binary(pipeline_oauth_retry) | binary(bc_2_11_007_pushdown_test) | binary(bc_2_16_002_crowdstrike_two_step)`

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
          [27 tests — non-empty, resolves correctly]
      prism-spec-engine::pipeline_oauth_retry:
          test_BC_2_01_017_dtu_401_surfaces_e_auth_004_no_retry
          test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately
          test_BC_2_16_002_execute_aborts_on_double_401
          test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request
          test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401
          test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution
```

All 4 spec-engine-http-cap binaries resolve with non-empty test lists.
`bc_2_11_007_pushdown_test` resolves with 11 tests — DTU-cap gap confirmed closed.

**Binary resolution summary: 11/11 capped binaries resolve. Zero zero-match filters.**

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
