---
document_type: story
story_id: S-DEMO-FIDELITY-REMEDIATION-001
title: "Demo Fidelity Code Fixes — T13 Pre-Flight Audit Remediation (2026-06-26)"
wave: null
# Wave assignment: immediate — human directive "fix everything before T13". No wave scheduler needed;
# dispatch as soon as this story reaches status: ready after BC authorship is confirmed complete.
target_module: prism-mcp
# Primary crate is prism-mcp (resources.rs, prompts.rs, prism_describe.rs); prism-query and
# prism-core are secondary crates touched for: E-QUERY-039 net-new implementation (error.rs,
# engine.rs enrichment gate — AST visitor pass, no new InfusionRegistry API), and E-QUERY-037
# gate-ordering fix (table_registry.rs, engine.rs). NOTE: N1-B is NET-NEW (not a routing fix) —
# EnrichUdfNotFound variant + struct do not yet exist anywhere in the workspace (verified
# 2026-06-26 remove-uncertainty pass). E-QUERY-037 map_prism_error arm is CONFIRMED PRESENT
# (v1.2 C1 correction — NOT net-new as stated in v1.1).
# v2.15 fold-in (TLS-REMEDIATION): 9 DTU crates' [dev-dependencies] + prism-bin [dev-dependencies]
# + ocsf-proto-gen optional download-feature [dependencies] standardized to rustls-tls
# (default-features=false); 4 DTU integration tests un-quarantined; 7 stop() cleanups in
# prism-dtu-claroty tests. prism-mcp remains primary target_module; DTU crates are co-owners
# of the TLS-REMEDIATION scope.
subsystems: [SS-01, SS-10, SS-11, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns the TLS-REMEDIATION fold-in work per ARCH-INDEX Subsystem
#     Registry (SS-01 row lists prism-dtu-common, prism-dtu-claroty, prism-dtu-armis,
#     prism-dtu-crowdstrike, prism-dtu-cyberint, prism-dtu-slack, prism-dtu-pagerduty,
#     prism-dtu-jira, prism-dtu-nvd, prism-dtu-threatintel):
#     - TLS-REMEDIATION: [dev-dependencies] reqwest standardized to rustls-tls in all 9 DTU crates;
#       4 integration tests un-quarantined in prism-dtu-armis (3) and prism-dtu-crowdstrike (1);
#       7 stop() resource-cleanup calls added in prism-dtu-claroty sec_p3_003_constant_time_admin_token.rs
#   SS-10 (MCP Interface) owns the prism-mcp work per ARCH-INDEX Subsystem Registry:
#     - N1: build_reference_content in resources.rs (BC-2.11.022 v1.1) — per-field UDF names
#     - AUDIT-001: build_tables_for_client in prism_describe.rs (BC-2.10.012 v1.7) — sensor-prefixed name
#     - AUDIT-004: render_* functions in prompts.rs (BC-2.10.016 v1.2) — FROM-ready table names
#   SS-11 (Query Execution Engine) owns the prism-query + prism-core work per ARCH-INDEX:
#     - N1-B: E-QUERY-039 NET-NEW implementation: EnrichUdfNotFound variant+struct in prism-core/error.rs;
#             plan-time enrichment gate in prism-query/engine.rs (direct match, pipe EnrichStage +
#             SQL ScalarFunc::Unknown paths); map_prism_error -32602 net-new arm in error_mapping.rs.
#             NOTE: map_prism_error arm for E-QUERY-037 (TableNotAvailable) is CONFIRMED PRESENT —
#             only the E-QUERY-039 (EnrichUdfNotFound) arm is net-new. BC-2.11.019 v1.6 draft→active
#             at merge (POL-14).
#     - N2: E-QUERY-037 gate-ordering fix located in table_registry.rs (check_availability_gate /
#           is_registered) + engine.rs — NOT materialization.rs only (verified 2026-06-26).
#   SS-22 (Process Lifecycle) owns prism-bin per ARCH-INDEX Subsystem Registry:
#     - TLS-REMEDIATION: prism-bin [dev-dependencies] reqwest standardized to rustls-tls
#   NOTE: ocsf-proto-gen (optional download-feature dep) is a build-helper not registered in the
#     ARCH-INDEX Subsystem Registry; its change is a single Cargo.toml dep-feature flag and
#     does not alter any subsystem boundary.
#   SS-01 is added at v2.15 (TLS fold-in); SS-22 is added at v2.15 (TLS fold-in).
#   SS-10 is primary (largest scope); SS-11 and SS-01 are co-owners.
priority: P0
# P0: ALL findings targeted by this story are DEMO-BLOCKING under the human directive
# "fix everything before T13" (2026-06-26). The T13 recording cannot proceed with incorrect
# enrichment function names in the reference, silent empty results for dot-syntax, incorrect
# describe names, or prompt bodies that embed invalid FROM queries.
depends_on:
  - S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
  # S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (merged PR #203 develop@7e60df03) must be on develop
  # as a base. This story builds on the MCP server refactor (build_reference_content, prompts.rs
  # prompt body structure, TableRegistry plan-time gates) introduced by PR #203.
  # Dependency anchor: build-order requirement — the code surfaces (resources.rs build_reference_content,
  # materialization.rs E-QUERY-037 gate, prompts.rs render_* functions) introduced by PR #203
  # are the exact functions this story modifies.
blocks: []
estimated_days: 2.5
# Estimate: 5 targeted code fixes + gate-coverage + TLS-REMEDIATION fold-in.
# N1 (resources.rs dedup key change): 0.5d
# N1-B (E-QUERY-039 gate investigation + fix): 0.5d
# N2 (E-QUERY-037 plan-time dot-notation gate ordering): 0.5d
# AUDIT-001 (build_tables_for_client sensor-prefix): 0.25d
# AUDIT-004 (prompts.rs FROM-ready names): 0.25d
# TLS-REMEDIATION fold-in (v2.15): standardize 11 Cargo.toml reqwest deps to rustls-tls,
#   un-quarantine 4 DTU integration tests, add 7 stop() calls: 0.5d
points: 11
# Points breakdown (v2.15 — adds 1pt for TLS-REMEDIATION fold-in; prior total was 10):
#   BC-2.11.022 v1.1 — N1: fix dedup key in build_reference_content: 2 pts
#   BC-2.11.019 v1.6 — N1-B: NET-NEW E-QUERY-039 implementation:
#     create EnrichUdfNotFound variant + EnrichUdfNotFoundDetails #[non_exhaustive] struct
#     in prism-core/error.rs; plan-time enrichment gate in prism-query/engine.rs (AST visitor,
#     pipe PipeStage::Enrich + SQL ScalarFunc::Unknown paths; derive UDF names from udf_descriptors());
#     map_prism_error -32602 arm (E-QUERY-039 only — E-QUERY-037 arm confirmed-present):
#     4 pts (net-new > 2 pts originally)
#   BC-2.11.001 v1.15 — N2: fix gate ordering across table_registry.rs + engine.rs
#     (NOT materialization.rs only — gate is in check_availability_gate/is_registered): 2 pts
#   BC-2.10.012 v1.7 — AUDIT-001 + AC-CAT2: fix build_tables_for_client emit format + build_pql_hints Cat2: 1 pt
#   BC-2.10.016 v1.2 — AUDIT-004: fix render_* prompt FROM-ready table names: 1 pt
#   TLS-REMEDIATION (v2.15 fold-in): rustls-tls standardization across 11 Cargo.toml entries,
#     4 test un-quarantine + root-cause correction, 7 stop() cleanups: 1 pt
#   Total: 11 pts
level: "L4"
status: merged
# BC status: 7 active (BC-2.11.001 v1.15, BC-2.11.022 v1.1, BC-2.10.016 v1.2, BC-2.10.012 v1.7,
#   BC-2.11.016 v1.23, BC-2.11.007 v1.9) + BC-2.11.019 v1.6 draft→active at merge per POL-14. Canonical versions
# are authoritative in the body BC table (§Behavioral Contracts); this comment is a status note only.
# Per Spec-First Gate S-7.01 this story is valid for dispatch as behavioral_contracts is non-empty.
version: "2.42"
updated: "2026-07-09"
producer: story-writer
timestamp: "2026-06-26T00:00:00Z"
input-hash: "TBD"
inputs:
  - ".factory/research/demo-pre-flight-audit-2026-06-26.md"
  - ".factory/research/demo-finding-remediation-plan-2026-06-26.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.022-auto-generated-prismql-reference-content-contract-and-ci-parity-gate.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.016-mcp-prompts-fast-return-guarantee-no-hang.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.012-prism-describe-schema-discovery-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
# Epic E-5 (MCP Interface / Query Engine). Remediation story targeting T13 capstone demo fidelity.
phase: 2
acceptance_criteria_count: 17
# 17 ACs (v2.15 — adds AC-TLS for the TLS-REMEDIATION fold-in):
#   Discrete **AC-XXX** headers enumerated (16 pre-v2.15 + 1 new AC-TLS = 17):
#     AC-N1, AC-N1B, AC-N2, AC-AUDIT-001, AC-AUDIT-004, AC-CAT2,
#     AC-C1C2, AC-M1, AC-M2, AC-L1, AC-H1, AC-DISC,
#     AC-REG-1, AC-REG-2, AC-DEMO-001, AC-SAP-1, AC-TLS
#   Sub-behaviors folded into parent ACs (NOT counted as standalone headers):
#     CRIT-1 (embedded bold section within AC-AUDIT-001, not a separate **AC-CRIT1** header)
#     SqlPipe modes (covered in AC-N1B, AC-N2, AC-C1C2 — no standalone **AC-SQLPIPE** header)
#     did_you_mean (covered in AC-N1B — no standalone **AC-DID_YOU_MEAN** header)
red_gate_tests: 54
# 54 Red Gate tests total (42 base + fold-ins through v2.17; see arithmetic block at end):
# --- AC-N1 ---
#   test_bc_2_11_022_n1_per_field_udf_names (bc_2_11_022_n1_test.rs)
# --- AC-N1B core ---
#   test_bc_2_11_019_n1b_infusion_id_as_udf_name (bc_2_11_019_n1b_test.rs)
#   test_bc_2_11_019_n1b_sql_path_infusion_id_as_udf_name (bc_2_11_019_n1b_test.rs)
#   test_bc_2_11_019_n1b_mcp_maps_to_32602 (bc_2_11_019_n1b_mcp_test.rs)
# --- AC-N1B gate ordering ---
#   test_high001_gate_ordering_table_error_before_enrich_error (bc_2_11_019_n1b_test.rs)
#   test_high003_sql_select_unknown_scalar_triggers_enrich_error (bc_2_11_019_n1b_test.rs)
# --- AC-N1B availability sorted ---
#   test_med001_available_infusions_sorted_in_e_query_039_error (bc_2_11_019_n1b_test.rs)
# --- AC-N1B SqlPipe head (HIGH-1) ---
#   test_high1_sqlpipe_head_unknown_scalar_fires_e_query_039 (bc_2_11_019_n1b_test.rs)
# --- AC-N1B EC-11-059 wired-but-empty registry ---
#   test_ec_11_059_wired_empty_registry_fires_e_query_039_with_empty_available (bc_2_11_019_n1b_test.rs)
# --- AC-N1B DataFusion built-in passthrough (BC-2.11.019 v1.6 F-PJL1-HIGH-001) ---
#   test_bc_2_11_019_n1b_builtin_passthrough_lower (bc_2_11_019_n1b_test.rs — EC-11-064)
#   test_bc_2_11_019_n1b_builtin_passthrough_coalesce (bc_2_11_019_n1b_test.rs — EC-11-065)
#   test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039 (bc_2_11_019_n1b_test.rs — EC-11-066)
#   test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039 (bc_2_11_019_n1b_test.rs — EC-11-067)
# --- AC-N1B did_you_mean engine (OBS-2) ---
#   test_obs2_did_you_mean_some_from_strsim_levenshtein_within_threshold (bc_2_11_019_n1b_test.rs)
#   test_obs2b_did_you_mean_none_when_beyond_levenshtein_threshold (bc_2_11_019_n1b_test.rs)
# --- AC-N1B F-PJL mid-cascade additions (BC-2.11.019 v1.6 F-PJL1/F-PJL4) ---
#   test_f_pjl1_high001_non_builtin_unknown_still_triggers_e_query_039 (bc_2_11_019_n1b_test.rs — F-PJL1-HIGH-001)
#   test_f_pjl4_med001_scheduled_path_table_gate_fires_before_capability_gate (bc_2_11_019_n1b_test.rs — F-PJL4-MED-001)
# --- AC-C1C2 enrich gate JOIN/GROUP/ORDER (unit-level) ---
#   test_c1_collect_unknown_scalar_from_sql_query_group_by (bc_2_11_019_n1b_test.rs)
#   test_c1_collect_unknown_scalar_from_sql_query_order_by (bc_2_11_019_n1b_test.rs)
#   test_c2_collect_unknown_scalar_from_sql_query_join_on (bc_2_11_019_n1b_test.rs)
# --- AC-C1C2 enrich gate JOIN/GROUP/ORDER (engine-level) ---
#   test_c1_sql_group_by_unknown_scalar_triggers_e_query_039 (bc_2_11_019_n1b_test.rs)
#   test_c1_sql_order_by_unknown_scalar_triggers_e_query_039 (bc_2_11_019_n1b_test.rs)
#   test_c1_sqlpipe_group_by_unknown_scalar_triggers_e_query_039 (bc_2_11_019_n1b_test.rs)
# --- AC-N2 (dot-notation E-QUERY-037) ---
#   test_bc_2_11_001_n2_dot_notation_from_target_e_query_037 (bc_2_11_001_n2_test.rs)
#   test_bc_2_11_001_n2_filter_mode_underscore_no_regression (bc_2_11_001_n2_test.rs)
#   test_bc_2_11_001_n2_dot_notation_sqlpipe_e_query_037 (bc_2_11_001_n2_test.rs — HIGH-1 SqlPipe)
#   test_bc_2_11_001_n2_sqlpipe_underscore_no_regression (bc_2_11_001_n2_test.rs)
# --- AC-AUDIT-001 ---
#   test_bc_2_10_012_audit_001_sensor_prefixed_table_names (bc_2_10_012_audit_001_test.rs)
#   test_bc_2_10_012_audit_001_multi_tenant_sensor_prefixed_unique (bc_2_10_012_audit_001_test.rs)
#   test_crit1_no_datetime_column_produces_column_free_query (prism_describe.rs inline tests)
# --- AC-CAT2 (BC-2.10.012 v1.7 §pql_hints Category-2) ---
#   test_bc_2_10_012_cat2_enrichment_hint_with_udfs (bc_2_10_012_audit_001_test.rs)
#   test_bc_2_10_012_cat2_enrichment_absent_hint (bc_2_10_012_audit_001_test.rs)
#   test_bc_2_10_012_cat2_zero_table_no_category2 (bc_2_10_012_audit_001_test.rs)
# --- AC-AUDIT-004 ---
#   test_bc_2_10_016_audit_004_no_dot_notation_in_prompts (bc_2_10_016_audit_004_test.rs)
#   test_bc_2_10_016_audit_004_prompt_from_targets_include_registered_table (bc_2_10_016_audit_004_test.rs)
#   test_bc_2_10_016_audit_004_column_refs_resolve_to_real_columns (bc_2_10_016_audit_004_test.rs)
#   test_bc_2_10_016_audit_004_column_sets_loaded_for_all_sensor_tables (bc_2_10_016_audit_004_test.rs)
#   test_bc_2_10_016_med2_prompt_filter_values_match_dtu_vocabulary (bc_2_10_016_audit_004_test.rs — MED-2)
# --- F-PQL2-OBS-001 skeleton-placeholder guards (BC-2.10.016 v1.2; origin F-PQL2-OBS-001) ---
#   test_f_pql2_obs001_query_skeleton_no_bare_timestamp (f_pql2_obs001_skeleton_placeholder_guard_test.rs)
#   test_f_pql2_obs001_datetime_arithmetic_uses_placeholder (f_pql2_obs001_skeleton_placeholder_guard_test.rs)
# --- AC-M2 HAVING column gate (BC-2.11.016 v1.23 — Position 6; inline in engine.rs,
#     module f_pwl1_low001_having_column_gate_tests) ---
#   test_BC_2_11_016_having_column_gate_typo_fires_e_query_038 (engine.rs inline)
#   test_BC_2_11_016_having_column_gate_valid_col_no_e_query_038 (engine.rs inline)
# --- AC-M2 HAVING agg-fn predicate tests (ADR-048 — HAVING agg-fn grammar extension;
#     inline in engine.rs, module f_pxl3_med002_having_agg_predicate_col_gate_tests) ---
#   test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038 (engine.rs inline)
#   test_BC_2_11_016_having_agg_fn_predicate_valid_col_no_e_query_038 (engine.rs inline)
#   test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001 (engine.rs inline — WHERE divergence guard)
# --- AC-DISC (F-L2-CRIT-001 — armis entity-discriminator seeding; materialization.rs
#     inline, module armis_discriminator_tests) ---
#   test_f_l2_crit001_armis_alerts_no_aql_seeds_in_alerts_discriminator (materialization.rs inline)
#   test_f_l2_crit001_armis_devices_no_aql_seeds_in_devices_discriminator (materialization.rs inline)
#   test_f_l2_crit001_armis_alerts_existing_aql_not_overwritten (materialization.rs inline)
#   test_f_l2_crit001_non_armis_table_filters_unchanged (materialization.rs inline)
# --- AC-DISC wiring-seam tests (F-LENS4-MED-001 — load-bearing pipeline wiring seam;
#     materialization.rs inline, module armis_discriminator_wiring_seam_tests) ---
#   test_F_LENS4_MED001_armis_alerts_pipeline_seeds_in_alerts_aql_filter (materialization.rs inline)
#   test_F_LENS4_MED001_armis_devices_pipeline_seeds_in_devices_aql_filter (materialization.rs inline)
#   test_F_LENS4_MED001_armis_alerts_user_supplied_aql_passes_through_pipeline (materialization.rs inline)
# --- AC-REG-1 ---
#   scripts/check-non-exhaustive.sh EXPECTED=88 (compile-fail gate via shell script, not a Rust fn)
# --- AC-REG-2 ---
#   test_bc_2_11_022_registry_parity (crates/prism-mcp/tests/reference_content.rs, existing — per-field UDF parity guard)
#
# Arithmetic: 42 (v2.9) + 4 armis entity-discriminator tests (F-L2-CRIT-001, materialization.rs inline) = 46;
# 46 (v2.10) + 3 armis discriminator wiring-seam tests (F-LENS4-MED-001, materialization.rs inline) = 49;
# 49 (v2.13) + 3 Cat2 enrichment hint tests (AC-CAT2, bc_2_10_012_audit_001_test.rs) = 52;
# 52 (v2.16) + 2 built-in passthrough aggregate/window tests (BC-2.11.019 v1.6 F1 EC-11-066/067) = 54.
# Red Gate semantics: the TDD-driving Red Gate subset — tests that were written RED before the
# corresponding code landed, plus inline and guard tests that drive story-delivered code surfaces
# (mid-cascade regression guards included). This is a named subset; the COMPLETE delivered test
# set — including regression guards in shared files and inline modules not listed here — is
# enumerated in §File Structure Requirements. The two counts are mutually consistent under this
# definition: §File Structure enumerates the broader set; red_gate_tests counts the Red Gate
# driving subset. F-PJL, F-PQL2-OBS-001, and HAVING guard tests are all included because they
# guard story-delivered code surfaces and were written as part of TDD-closure for this story.
tdd_mode: strict
behavioral_contracts:
  [BC-2.11.001, BC-2.11.022, BC-2.11.019, BC-2.10.016, BC-2.10.012, BC-2.11.016, BC-2.11.007]
# BC array propagation (bc_array_changes_propagate_to_body_and_acs):
# BC-2.11.001 — query MCP tool (cited in AC-N2: dot-notation EC-11-067 plan-time gate)
# BC-2.11.022 — prismql://reference content contract (cited in AC-N1: per-field UDF dedup)
# BC-2.11.019 — E-QUERY-039 enrich-UDF-not-found gate (cited in AC-N1B)
# BC-2.10.016 — MCP prompts fast-return + FROM-ready names (cited in AC-AUDIT-004)
# BC-2.10.012 — prism_describe schema discovery tool (cited in AC-AUDIT-001 and AC-CAT2)
# BC-2.11.016 — E-QUERY-038 column-not-found gate (cited in AC-M1 and AC-M2)
# BC-2.11.007 — sensor filter push-down (cited in AC-DISC: armis AQL discriminator seeding §Mechanism B.1 / PC-DISC-001)
# All 7 BCs cited in at least one AC body trace.
verification_properties: [VP-021]
# VP-021 (PrismQL parser never panics on arbitrary input — fuzz) applies to changes in
# materialization.rs E-QUERY-037 gate ordering (N2) and any new plan-time checks.
assumption_validations: []
risk_mitigations: []
crates_touched:
  - prism-core
  # IMPLEMENTED: error.rs — PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>) variant
  # and #[non_exhaustive] EnrichUdfNotFoundDetails { infusion: String, available_infusions:
  # Vec<String>, did_you_mean: Option<String> } struct. Both carry #[non_exhaustive].
  # error.rs also gains variant_meta category "validation" for structured MCP output (MED-4).
  # New test file: crates/prism-core/src/tests/test_enrich_udf_not_found_display.rs
  # (5 tests: display_no_did_you_mean, display_with_did_you_mean, display_empty_available,
  # display_starts_with_error_code, f_pbl1_low002_display_self_sorts_available_infusions).
  # scripts/check-non-exhaustive.sh EXPECTED=88.
  - prism-mcp
  # IMPLEMENTED — resources.rs: build_reference_content dedup key changed from infusion_id
  # to descriptor.name; Some(empty)/None placeholders implemented.
  # New test file: crates/prism-mcp/tests/bc_2_11_022_n1_test.rs (test_bc_2_11_022_n1_per_field_udf_names)
  # IMPLEMENTED — tools/prism_describe.rs: build_tables_for_client in BOTH multi-tenant
  # (resolved_spec_map path) and single-tenant (config_manager path) emit
  # format!("{sensor_id}_{}", table.table_name) as the prefixed_name. New public function
  # build_example_query derives the datetime column from the spec (CRIT-1 fix: no longer
  # hardcodes 'timestamp'). New inline test module build_example_query_tests with
  # test_crit1_no_datetime_column_produces_column_free_query.
  # AC-CAT2: build_pql_hints gains 4th param
  # `infusion_registry: Option<&prism_spec_engine::InfusionRegistry>` (BC-2.10.012 v1.7
  # §pql_hints Category-2). pql_hints.len()==3 for non-empty tables (index 2 = enrichment
  # presence hint — sorted UDFs as `<name>(<input_field>)`); pql_hints.len()==1 for empty
  # tables (Category-2 suppressed). handle_prism_describe wired:
  # `let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());`
  # pass `infusion_registry.as_deref()` as 4th arg.
  # New test file: crates/prism-mcp/tests/bc_2_10_012_audit_001_test.rs
  # (test_bc_2_10_012_audit_001_sensor_prefixed_table_names, _multi_tenant_sensor_prefixed_unique,
  #  test_bc_2_10_012_cat2_enrichment_hint_with_udfs, _absent_hint, _zero_table_no_category2)
  # IMPLEMENTED — prompts.rs: 4 render_* functions updated with FROM-ready names; all
  # filter VALUES in embedded queries now match each DTU's exact emitted vocabulary (MED-1 fix).
  # New test file: crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs
  # (test_bc_2_10_016_audit_004_no_dot_notation_in_prompts,
  #  test_bc_2_10_016_audit_004_prompt_from_targets_include_registered_table,
  #  test_bc_2_10_016_audit_004_column_refs_resolve_to_real_columns,
  #  test_bc_2_10_016_audit_004_column_sets_loaded_for_all_sensor_tables,
  #  test_bc_2_10_016_med2_prompt_filter_values_match_dtu_vocabulary)
  # IMPLEMENTED — error_mapping.rs: explicit -32602 INVALID_PARAMS arm for
  # PrismError::EnrichUdfNotFound (E-QUERY-039); structured variant_meta category "validation";
  # available_infusions Vec<String> sorted+deduped before MCP output.
  # New test file: crates/prism-mcp/tests/bc_2_11_019_n1b_mcp_test.rs
  # (test_bc_2_11_019_n1b_mcp_maps_to_32602, test_med5_enrich_udf_not_found_suggestion_non_empty_no_brackets,
  #  test_med5_enrich_udf_not_found_suggestion_empty_infusions)
  # ALSO — tool_dispatch_tests.rs: new fail-closed guard tests (OBS-5 fix)
  # test_med4_enrich_udf_not_found_structured_category_is_validation (in tool_dispatch_tests.rs)
  # DELETED — crates/prism-mcp/tests/crit001_prompt_table_names.rs: superseded by
  # AUDIT-004 TOML-derived guard (bc_2_10_016_audit_004_test.rs) per OBS-4 finding.
  - prism-spec-engine
  # AC-CAT2: crates/prism-spec-engine/src/infusion/udf.rs —
  # `InfusionUdfDescriptor` gains `pub input_field: String`; `new()` gains this param
  # (BC-2.10.012 v1.7 §pql_hints Category-2). `udf_descriptors()` in
  # crates/prism-spec-engine/src/infusion/mod.rs propagates `field.input_field.clone()`.
  - prism-dtu-armis
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  # 3 tests un-quarantined (removed #[ignore]):
  #   test_BC_2_06_019_armis_primary_device_stage_visibility
  #   test_BPRL_P4_02_armis_alerts_stage_guard_primary_device
  #   test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility
  - prism-dtu-crowdstrike
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  # 1 test un-quarantined (removed #[ignore]):
  #   test_BPRL_P4_02_detections_stage_guard_primary_device
  - prism-dtu-claroty
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  # 7 stop() resource-cleanup calls added in
  #   crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs
  - prism-dtu-cyberint
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-dtu-slack
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-dtu-pagerduty
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-dtu-jira
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-dtu-nvd
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-dtu-threatintel
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  - prism-bin
  # TLS-REMEDIATION (v2.15): [dev-dependencies] reqwest → rustls-tls (default-features=false).
  #   Scope: dev-dependencies only; prism-bin production reqwest was ALREADY rustls — unchanged.
  - ocsf-proto-gen
  # TLS-REMEDIATION (v2.15): optional download-feature [dependencies] reqwest →
  #   rustls-tls (default-features=false). Production security posture unchanged
  #   (download feature is build-time only; not a runtime dep).
  - prism-query
  # IMPLEMENTED — materialization.rs: `pub(crate) fn seed_armis_entity_discriminator`
  # (F-L2-CRIT-001 fix) seeds the AQL search discriminator for armis tables when `aql`
  # is absent/empty: armis_alerts → aql="in:alerts"; armis_devices → aql="in:devices".
  # User-supplied non-empty `aql` predicate is preserved verbatim. Non-armis tables are
  # unaffected. Wired into the run_materialization_pipeline fan-out loop. New inline test
  # module armis_discriminator_tests with 4 tests (see AC-DISC below).
  # IMPLEMENTED — engine.rs: collect_unknown_scalars_from_sql_query (new fn) scans ALL
  # scalar-expr positions via helper fns collect_unknown_scalar_from_expr and
  # collect_unknown_scalar_from_predicate (SELECT projection, WHERE, JOIN ON, GROUP BY,
  # ORDER BY, HAVING) for BOTH Ast::Sql(Select) and Ast::SqlPipe head queries, plus
  # Pipe stages. gate check_enrich_udf_availability fires LAST (after E-QUERY-037 +
  # E-QUERY-038). Gate ordering in execute_inner AND execute_scheduled_inner aligned:
  # E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-011 (H1 fix:
  # capability gate moved AFTER enrich gate in execute_scheduled_inner to match execute_inner).
  # available_infusions Vec<String> sorted+deduped; strsim did_you_mean lexicographic tie-break.
  # New test file: crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs (22 tests — see
  # AC-N1B below for full inventory including C1/C2 unit-level + engine-level tests,
  # high001 gate ordering, high003 SQL projection, med001 sort, high1 SqlPipe, ec_11_059
  # wired-empty, obs2 did_you_mean Some/None tests, F-PJL mid-cascade F-PJL1/F-PJL4 tests).
  # ALSO — engine.rs: check_query_column_availability (E-QUERY-038) column gate now
  # validates GROUP BY/ORDER BY func-args and JOIN ON columns (M2 fix) using
  # extract_field_paths_with_bareness (positions 1/3/4/5) and extract_predicate_columns_with_bareness
  # (positions 2/6; per-reference (name, is_bare) pairs for HEAD-JOIN SUSPENSION gate);
  # accepts table_registry parameter for single-tenant mode (M1 wiring).
  # IMPLEMENTED — table_registry.rs: columns_by_table field + columns_for_table() method
  # added so check_query_column_availability can fire in single-tenant mode (M1 fix). New
  # collect_expr_sources_into_gate fn makes source walk cover HAVING, GROUP BY, ORDER BY,
  # JOIN ON subqueries (L1 fix). New test file:
  # crates/prism-query/src/tests/table_registry_tests.rs (new tests for columns_for_table,
  # availability gate coverage for all subquery positions, OBS-1 SqlPipe JOIN stage source
  # discovery, OBS-1 SELECT WHERE IN subquery source discovery)
  # New test file: crates/prism-query/src/tests/bc_2_11_001_n2_test.rs (4 tests: pipe-mode
  # dot-notation→E-QUERY-037, filter-mode regression, SqlPipe dot-notation→E-QUERY-037,
  # SqlPipe underscore regression)
---

# S-DEMO-FIDELITY-REMEDIATION-001: Demo Fidelity Code Fixes

## Narrative

As a Prism developer preparing the T13 capstone demo recording, I want the five
code defects identified in the 2026-06-26 pre-flight re-audit fixed, plus all
gate-coverage gaps found during LOCAL adversarial passes, AND the test-infrastructure
TLS reliability fix folded in from the un-parking rebase (commit cf66151f —
rustls-tls standardization removing macOS native-tls Keychain-init latency from DTU
integration tests), so that: the `prismql://reference`
resource lists the correct callable enrichment UDF names; calling an unregistered
enrichment name at any AST position (SELECT projection, JOIN ON, GROUP BY, ORDER BY,
HAVING, WHERE, or SqlPipe head) returns a self-correcting E-QUERY-039 error (not an
opaque internal error) with sorted `available_infusions` and optional `did_you_mean`;
`FROM cyberint.alerts` in ANY query mode (Pipe, SQL, SqlPipe) returns a pedagogical
E-QUERY-037 with `did_you_mean` (not a silent empty result); `prism_describe` table
`name` fields are FROM-ready sensor-prefixed names derived via `format!("{sensor_id}_{table_name}")`
on both tenant paths, and `example_query` uses the actual datetime column from the spec
rather than a hardcoded `timestamp` literal; all MCP prompt bodies embed valid FROM-ready
sensor-prefixed table names that execute without E-QUERY-037, with filter VALUES matching
each DTU's exact emitted vocabulary; the E-QUERY-038 column gate fires in single-tenant
mode (via `TableRegistry.columns_for_table`); capability-gate ordering is symmetric between
`execute_inner` and `execute_scheduled_inner`; and E-QUERY-037 availability gate covers
all subquery positions (HAVING, GROUP BY, ORDER BY, JOIN ON).

## Behavioral Contracts

| BC ID | Version | Title |
|-------|---------|-------|
| BC-2.11.001 | v1.15 | BC-2.11.001: `query` MCP Tool Accepts Scoping + PrismQL Query String |
| BC-2.11.022 | v1.1 | BC-2.11.022: Auto-Generated `prismql://reference` Content Contract and CI Parity Gate |
| BC-2.11.019 | v1.6 | BC-2.11.019: E-QUERY-039 Enrich-UDF-Not-Found Plan-Time Gate |
| BC-2.10.016 | v1.2 | BC-2.10.016: MCP Prompts Fast-Return Guarantee — No Indefinite Hang |
| BC-2.10.012 | v1.7 | BC-2.10.012: `prism_describe` Schema Discovery Tool (L2) |
| BC-2.11.016 | v1.23 | BC-2.11.016: E-QUERY-038 Column-Not-Found Plan-Time Gate (L4) |
| BC-2.11.007 | v1.9 | BC-2.11.007: Sensor Filter Push-Down |

---

## Acceptance Criteria

> NOTE: ACs are ordered by implementation dependency. AC-N2 (prism-query gate ordering)
> should be implemented before AC-AUDIT-004 (prompts), since AUDIT-004 fixes depend on
> knowing the correct FROM-ready table names — which are the same sensor-prefixed names
> that AUDIT-001 and the current TableRegistry already use. Each AC maps to one finding
> from the 2026-06-26 pre-flight audit and one BC postcondition.

---

### Area A — MCP Reference: Correct Enrichment Function Names (N1)

**AC-N1** (traces to BC-2.11.022 v1.1 postcondition — enrichment section per-field UDF names,
EC-11-022-006): `build_reference_content` in `crates/prism-mcp/src/resources.rs` iterates
`InfusionRegistry.udf_descriptors()` and deduplicates by `descriptor.name` (the per-field UDF
name), NOT by `descriptor.infusion_id`. For a live `InfusionRegistry` loaded from
`threatintel.infusion.toml` (infusion_id `threat_intel`, fields `threat_score`,
`threat_is_known_malicious`, `threat_sources`) and `nvd.infusion.toml` (infusion_id `nvd`,
fields `cvss_base_score`, `cvss_severity`, `cvss_vector`), the assembled reference enrichment
section MUST list exactly **six** callable entries: `enrich threat_score(col)`, `enrich
threat_is_known_malicious(col)`, `enrich threat_sources(col)`, `enrich cvss_base_score(col)`,
`enrich cvss_severity(col)`, `enrich cvss_vector(col)`. The strings `threat_intel` and `nvd`
(the infusion_ids, which are NOT callable UDF names) MUST NOT appear in the enrichment section.

**Red Gate test:** `test_bc_2_11_022_n1_per_field_udf_names` — build an `InfusionRegistry`
test fixture with `threat_intel` infusion (fields: `threat_score`, `threat_is_known_malicious`,
`threat_sources`) and `nvd` infusion (fields: `cvss_base_score`, `cvss_severity`,
`cvss_vector`); call `build_reference_content(Some(&registry))`; assert the enrichment section
contains all six per-field names; assert it does NOT contain `threat_intel(` or `nvd(` as
callable fn forms (the N1 regression guard).

---

### Area B — E-QUERY-039: Implement Net-New Enrichment UDF Not Found Gate (N1-B)

> **SCOPE NOTE (v1.2):** N1-B is NET-NEW implementation, NOT a routing investigation.
> `PrismError::EnrichUdfNotFound` and `EnrichUdfNotFoundDetails` do NOT exist anywhere in the
> workspace (zero matches as of 2026-06-26). E-QUERY-039 appears only as a doc table row in
> resources.rs. PR #203 did NOT implement this variant. This AC creates the variant, struct,
> gate, and MCP mapping from scratch per BC-2.11.019 v1.6. BC-2.11.019 promotes draft→active
> at merge (POL-14).
>
> **NO NEW PUBLIC API on `InfusionRegistry`** (I1 correction v1.2): Do NOT add a `udf_names()`
> method to `InfusionRegistry`. The public API already provides `udf_descriptors()` — derive
> the UDF name set from it inline: `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`.
> Use this expression everywhere `available_infusions` is populated and everywhere the
> strsim candidate set is built. This keeps the `InfusionRegistry` public API surface minimal
> and the new-#[non_exhaustive]-type count at exactly ONE (EnrichUdfNotFoundDetails), so
> ci.yml EXPECTED increments 87→88 (not 87→89).
>
> **E-QUERY-037 arm in `map_prism_error` is CONFIRMED PRESENT** (C1 correction v1.2): The
> `PrismError::TableNotAvailable(..)` arm already exists in `error_mapping.rs` (line ~166,
> doc block: "Reference: S-3.13 AC-2; BC-2.11.001; error-taxonomy.md E-QUERY-037"). No change
> needed to that arm. ONLY the `EnrichUdfNotFound` arm (E-QUERY-039) is net-new.

**AC-N1B** (traces to BC-2.11.019 v1.6 postconditions — EnrichUdfNotFound variant shape,
gate firing condition for pipe-mode `EnrichStage.infusion` NOT in
`InfusionRegistry.udf_to_infusion`, and SQL-mode `ScalarFunc::Unknown` gate (with DataFusion
built-in exclusion per F-PJL1-HIGH-001), and MCP -32602 mapping):

> **Gate-ordering note (BC-2.11.019 v1.6):** E-QUERY-039 fires LAST in the plan-time gate
> sequence. The full ordered sequence is: E-QUERY-001 (parse error) → E-QUERY-037 (table
> availability, `check_availability_gate`) → E-QUERY-038 (column gate) → E-QUERY-039 (enrichment
> UDF not found, this gate). A query with both a dot-notation FROM target AND an invalid
> enrichment name returns E-QUERY-037, NOT E-QUERY-039 — the table gate fires first.
>
> **WHERE-clause note (BC-2.11.019 v1.6 §Precondition 1(b)):** SQL-mode enrichment-validation
> gates `ScalarFunc::Unknown(name)` in SELECT PROJECTION expressions — this is the reachable,
> real-query path. The WHERE-predicate scan via `collect_unknown_scalar_from_predicate` is
> DEFENSIVE / forward-compatible coverage: it honors BC-2.11.019 v1.6 §Precondition 1(b)'s
> AST-contract ("a WHERE clause containing FuncCall::Scalar{...} must be gated at plan time"),
> but a real SQL query `WHERE udf(col) = v` is currently an **E-QUERY-001 parse error** —
> `build_predicate_parser` (the WHERE grammar, `comparison` atom) parses
> `field_path → compare_op → literal` only; there is no scalar-funcall atom, so the parser
> hits `(` where it expects a compare op and never produces a `ScalarFunc::Unknown` node from
> WHERE text. `ScalarFunc::Unknown` is produced ONLY by the SQL expression parser used for
> SELECT projections (`build_sql_expr_parser`). The WHERE scan is exercised by unit tests via
> programmatic AST construction (`engine::enrich_gate_where_clause_unit_tests`), not reachable
> from real parsed query text today. The pipe `enrich`-keyword form used in a WHERE position
> (e.g., `| WHERE enrich threat_score(col) > 0`) is also an E-QUERY-001 parse error (pipe
> filter grammar has no fn-call atom). Both projection (reachable) and WHERE (defensive) scans
> feed the same validation loop and the same `EnrichUdfNotFound` error type.

**Step 1 — Create the error type** (in `crates/prism-core/src/error.rs`):
- Add variant `EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` to `PrismError`.
- Add `#[non_exhaustive]` struct `EnrichUdfNotFoundDetails { pub infusion: String, pub available_infusions: Vec<String>, pub did_you_mean: Option<String> }`.
  - `available_infusions` is `Vec<String>` (canonical type per BC-2.11.019 v1.6; PO-ratified).
- Both type and variant MUST carry `#[non_exhaustive]`. Increment `ci.yml EXPECTED` 87→88. Update `CLAUDE.md` non-exhaustive sentence + attribution list in the same atomic commit.

**Step 2 — Add plan-time enrichment gate** (in `crates/prism-query/src/engine.rs`) (I2 anchor v1.3):
Add a new plan-time enrichment-validation pass in `crates/prism-query/src/engine.rs`, invoked
BEFORE `check_availability_gate`/fan-out. This pass uses a direct `match &ast { ... }` traversal
(not the `visit::Visitor` trait — avoids coupling with the full visitor infrastructure) to collect
enrichment function names from BOTH query paths and validates each against the registered
UDF name set (derived from `registry.udf_descriptors()`):
- **Pipe path** — match arm collects `EnrichStage.infusion` values from `PipeStage::Enrich` nodes.
- **SQL path** — match arm collects `ScalarFunc::Unknown(name)` values from SELECT projection expressions (reachable from real queries via `build_sql_expr_parser`) AND from WHERE clause predicates via `collect_unknown_scalar_from_predicate` (DEFENSIVE / forward-compat coverage per BC-2.11.019 v1.6 §Precondition 1(b) AST-contract; see WHERE-clause note above — a real `WHERE udf(col) = v` is an E-QUERY-001 parse error today; the WHERE scan is exercised by programmatic AST unit tests, not real parsed query text). **DataFusion built-in exclusion (v1.6 F-PJL1-HIGH-001):** for SQL-mode, the gate fires ONLY when `name` is NEITHER a DataFusion built-in (check `ctx.state().scalar_functions().get(name)`, `ctx.state().aggregate_functions().get(name)`, or `ctx.state().window_functions().get(name)`) NOR a registered infusion. Names like `lower`, `upper`, `coalesce` (scalar), `stddev`, `median`, `array_agg` (aggregate), or `row_number`, `rank` (window) that DataFusion can resolve in ANY registry must pass the gate without E-QUERY-039.

Both collection paths are DISTINCT match arms but feed the same validation loop and the same
`EnrichUdfNotFound` error type. For each collected name: if `name` is NOT a key in
`InfusionRegistry.udf_to_infusion`, return at plan time:
```rust
Err(PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails {
    infusion: name.to_owned(),
    available_infusions: registry.udf_descriptors().iter().map(|d| d.name.clone()).collect(),
    did_you_mean: strsim_closest(&name, &udf_names_vec),
})))
```
Gate MUST fire BEFORE any fan-out or sensor I/O. No new public methods on `InfusionRegistry`.
Gate ordering: this enrichment-validation pass runs AFTER the table availability gate
(`check_availability_gate` / E-QUERY-037) so that table-availability errors are reported first.

**Step 3 — Add MCP mapping** (in `crates/prism-mcp/src/error_mapping.rs`):
- Add an explicit arm for `PrismError::EnrichUdfNotFound(d)` in `map_prism_error` that returns
  `(codes::INVALID_PARAMS, ...)` with the canonical Display message format (BC-2.11.019 v1.6):
  ```
  E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}
  ```
  Where `{available_infusions}` is the comma-joined `Vec<String>` wrapped in brackets (e.g.,
  `[threat_score, threat_is_known_malicious, threat_sources]`), and `{did_you_mean}` is
  ` Did you mean: '{x}'?` when `did_you_mean` is `Some(x)`, or omitted (empty string) when `None`.
  Full example (no suggestion): `E-QUERY-039: enrichment infusion 'threat_intel' is not registered; available: [threat_score, threat_is_known_malicious, threat_sources, cvss_base_score, cvss_severity, cvss_vector]`
  Full example (with suggestion): `E-QUERY-039: enrichment infusion 'threat_scor' is not registered; available: [threat_score, ...] Did you mean: 'threat_score'?`
- This arm MUST NOT fall through to the `-32000` catch-all.
- The `PrismError::TableNotAvailable(..)` arm (E-QUERY-037) is CONFIRMED PRESENT — do NOT modify or duplicate it.

**Observable behavior**: A pipe-mode query `FROM cyberint_alerts | enrich threat_intel(iocs_value)` where `threat_intel` is an infusion_id (not a per-field UDF name) and therefore NOT a key in `InfusionRegistry.udf_to_infusion`, returns `PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` at plan time, surfaced as MCP `-32602 INVALID_PARAMS` with `code: "E-QUERY-039"`. It MUST NOT return `E-INT-001` "Internal error; see audit log". The `available_infusions: Vec<String>` field MUST list the registered per-field UDF names (e.g., `threat_score`, `threat_is_known_malicious`, `threat_sources`, ...). A `did_you_mean` suggestion is present IF any registered UDF name is within Levenshtein distance 3 of the queried name; `None` is a valid outcome when no registered name is within distance 3 (e.g., `"threat_intel"` vs per-field names like `"threat_score"` may exceed distance 3). The same gate applies to a SQL-mode `ScalarFunc::Unknown("nvd")` in a SELECT projection: it returns E-QUERY-039, NOT E-INT-001.

> **DataFusion built-in exclusion note (BC-2.11.019 v1.6 §F-PJL1-HIGH-001):** A SQL-mode query
> `SELECT lower(hostname) FROM crowdstrike_detections` with the infusion registry wired MUST NOT
> return E-QUERY-039. `lower` is a DataFusion built-in scalar function resolvable via
> `ctx.state().scalar_functions().get("lower")`; it satisfies the DataFusion built-in exclusion
> condition (b) of the three-part firing condition and passes the gate. The query proceeds to
> DataFusion execution where `lower(hostname)` resolves normally. The same exclusion applies to
> DataFusion built-in **aggregate** functions (e.g., `stddev`, `median`, `variance`, `array_agg` —
> resolvable via `ctx.state().aggregate_functions()`) and built-in **window** functions (e.g.,
> `row_number`, `rank` — resolvable via `ctx.state().window_functions()`). Any `ScalarFunc::Unknown`
> name that resolves in ANY of the three DataFusion function registries (scalar, aggregate, or window)
> passes the gate without E-QUERY-039. The AUDIT-005 reproducer
> (`SELECT cvss(device_cves_first) FROM armis_devices`) is unaffected: `cvss` is not a DataFusion
> built-in in any registry, so E-QUERY-039 still fires for unregistered non-builtin names.
> The pipe-mode `| enrich <name>(...)` path does NOT use this exclusion — pipe-mode fires E-QUERY-039
> for any `EnrichStage.infusion` name not in `InfusionRegistry.udf_to_infusion`, regardless of
> DataFusion registries. See EC-11-064, EC-11-065 (scalar examples), EC-11-066 (aggregate),
> EC-11-067 (window) in BC-2.11.019 v1.6.
> **Implementation requirement:** the exclusion check MUST exclude DataFusion built-in functions
> (scalar, aggregate, and window) by querying all three of DataFusion's runtime-derived function
> registries: `scalar_functions()` (or `SessionStateDefaults::default_scalar_functions()`),
> `aggregate_functions()` (or `SessionStateDefaults::default_aggregate_functions()`), and
> `window_functions()` (or `SessionStateDefaults::default_window_functions()`), NOT a hard-coded
> allowlist. New tests for EC-11-064/065: `test_bc_2_11_019_n1b_builtin_passthrough_lower` and
> `test_bc_2_11_019_n1b_builtin_passthrough_coalesce` in `bc_2_11_019_n1b_test.rs`. New tests
> for EC-11-066/067: `test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039` (aggregate) and
> `test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039` (window) in `bc_2_11_019_n1b_test.rs`.

**Red Gate tests:**

`test_bc_2_11_019_n1b_infusion_id_as_udf_name` — execute a plan-time validation with query
`FROM cyberint_alerts | enrich threat_intel(iocs_value)` where `threat_intel` is NOT registered
as a UDF name in `InfusionRegistry` (only per-field names are registered); assert the result
is `Err(PrismError::EnrichUdfNotFound(_))` with `infusion: "threat_intel"` and
`available_infusions` non-empty (listing the registered per-field UDF names); assert the
result is NOT `E-INT-001` (negative control). Do NOT assert `did_you_mean.is_some()` — assert
on `available_infusions` (always populated) and the error variant/code only, since registered
per-field UDF names are likely > Levenshtein-3 from `"threat_intel"` making `did_you_mean: None`
a valid outcome (S1 relaxation v1.2). Also assert SQL-mode `ScalarFunc::Unknown("nvd")` returns
`Err(PrismError::EnrichUdfNotFound(_))`.

`test_bc_2_11_019_n1b_mcp_maps_to_32602` — call `map_prism_error(PrismError::EnrichUdfNotFound(...))`;
assert the returned MCP error code is `-32602` (INVALID_PARAMS); assert it is NOT `-32000`
(the generic catch-all). This test lives in `crates/prism-mcp/tests/bc_2_11_019_n1b_mcp_test.rs`
(an integration test file, not an inline `#[cfg(test)]` module in error_mapping.rs).

`test_bc_2_11_019_n1b_builtin_passthrough_lower` — execute a plan-time validation with a
SQL-mode query `SELECT lower(hostname) FROM crowdstrike_detections` with the infusion registry
wired and `lower` NOT registered as an infusion; assert the result is `Ok(...)` (E-QUERY-039
does NOT fire); assert the result is NOT `Err(PrismError::EnrichUdfNotFound(_))`. This guards
the F-PJL1-HIGH-001 regression: DataFusion built-in `lower` must pass the gate (BC-2.11.019
v1.6 EC-11-064).

`test_bc_2_11_019_n1b_builtin_passthrough_coalesce` — same as above but for
`SELECT upper(device_name), coalesce(severity, 'unknown') FROM armis_devices`; assert neither
`upper` nor `coalesce` trigger E-QUERY-039 (BC-2.11.019 v1.6 EC-11-065).

`test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039` — execute a plan-time validation with a
SQL-mode query `SELECT stddev(latency) FROM crowdstrike_detections` with the infusion registry
wired and `stddev` NOT registered as an infusion; assert the result is `Ok(...)` (E-QUERY-039
does NOT fire); assert the result is NOT `Err(PrismError::EnrichUdfNotFound(_))`. This guards
the F1-HIGH DataFusion aggregate built-in exclusion: `stddev` is a DataFusion built-in aggregate
function resolvable via `ctx.state().aggregate_functions()` and must pass the gate (BC-2.11.019
v1.6 EC-11-066). The same check applies to `median`, `variance`, `array_agg`, and any other
DataFusion aggregate built-in. SQL-mode only — pipe-mode `| enrich stddev(col)` still fires
E-QUERY-039 if `stddev` is not a registered infusion.

`test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039` — execute a plan-time validation with a
SQL-mode query that includes `row_number()` as a `ScalarFunc::Unknown` node with the infusion
registry wired and `row_number` NOT registered as an infusion; assert the result is `Ok(...)`
(E-QUERY-039 does NOT fire); assert the result is NOT `Err(PrismError::EnrichUdfNotFound(_))`.
This guards the F1-HIGH DataFusion window built-in exclusion: `row_number` is a DataFusion
built-in window function resolvable via `ctx.state().window_functions()` and must pass the gate
(BC-2.11.019 v1.6 EC-11-067). The same check applies to `rank`, `dense_rank`, `lead`, `lag`,
and other DataFusion window built-ins. SQL-mode only — pipe-mode still fires E-QUERY-039.

---

### Area C — E-QUERY-037: Dot-Notation FROM Target Intercepted at Plan Time (N2)

> **SCOPE NOTE (v1.1, confirmed in v1.2, extended v2.0):** N2's gate-ordering fix is located
> in `crates/prism-query/src/table_registry.rs` (`check_availability_gate` / `is_registered`)
> and `crates/prism-query/src/engine.rs` — NOT in `materialization.rs` only. The N2 fix
> catches dot-notation strings BEFORE reaching fan-out or resolve_source_refs.
>
> **BC-2.11.001 v1.15 SqlPipe mode is NOT exempt (v2.0 HIGH-1 fix):** The original story
> (through v1.9) was silent on whether `Ast::SqlPipe` queries received the same E-QUERY-037
> treatment. BC-2.11.001 v1.15 is mode-agnostic for EC-11-067: dot-notation in FROM target
> position MUST return E-QUERY-037 regardless of whether the AST is `Ast::Sql`, `Ast::Pipe`,
> or `Ast::SqlPipe`. A previous is_sqlpipe exemption has been removed. The implementation
> and Red Gate tests confirm all three modes are covered.

**AC-N2** (traces to BC-2.11.001 v1.15 postcondition — table availability plan-time check,
EC-11-067: dot-notation in FROM target position): A query `FROM cyberint.alerts` (Pipe mode),
`SELECT * FROM crowdstrike.detections` (SQL mode), or a SqlPipe head `FROM cyberint.alerts |
SELECT *` (SqlPipe mode) where the dot-notation string is NOT a key in `TableRegistry` (only
underscore-qualified names like `cyberint_alerts`, `crowdstrike_detections` are registered),
returns `PrismError::TableNotAvailable` (`E-QUERY-037`) at plan time with `table: "cyberint.alerts"`,
`sensor: "cyberint"`, `did_you_mean: "cyberint_alerts"`. The `TableRegistry::is_registered`
check in `check_availability_gate` (table_registry.rs) MUST run BEFORE `sensor_id_from_table_name`
dot-notation extraction in the fan-out path. No SqlPipe-mode exemption. The result is
`isError: true`, NOT `isError: false, returned: 0` with `sensor_errors: ["E-SENSOR-030"]`.
The fix MUST NOT regress BC-2.11.023 / ADR-046 filter-mode dot-notation (filter mode uses
`<table_name> | <predicate>` syntax, not dot-syntax as a FROM target). The `map_prism_error`
arm for `PrismError::TableNotAvailable` (E-QUERY-037) is CONFIRMED PRESENT in
`error_mapping.rs` — no change to that arm.

**Red Gate tests:**
- `test_bc_2_11_001_n2_dot_notation_from_target_e_query_037` — pipe mode `FROM cyberint.alerts`; assert `Err(PrismError::TableNotAvailable(_))` with `did_you_mean: "cyberint_alerts"`.
- `test_bc_2_11_001_n2_filter_mode_underscore_no_regression` — filter mode `crowdstrike_detections | severity='HIGH'`; assert parse succeeds and passes availability gate (BC-2.11.023 regression guard).
- `test_bc_2_11_001_n2_dot_notation_sqlpipe_e_query_037` — SqlPipe head `FROM cyberint.alerts | SELECT *`; assert `Err(PrismError::TableNotAvailable(_))` (HIGH-1 fix: SqlPipe-not-exempt scope, BC-2.11.001 v1.15 mode-agnostic).
- `test_bc_2_11_001_n2_sqlpipe_underscore_no_regression` — SqlPipe head with valid underscore name; assert passes availability gate (regression guard).

---

### Area D — prism_describe: FROM-Ready Sensor-Prefixed Table Names (AUDIT-001)

**AC-AUDIT-001** (traces to BC-2.10.012 v1.7 postcondition — `name` postcondition fully-qualified
FROM-ready token, closes AUDIT-001 + AUDIT-008): `build_tables_for_client` in
`crates/prism-mcp/src/tools/prism_describe.rs` emits `name: format!("{sensor_id}_{}", table.table_name)`
for each table entry on BOTH code paths (multi-tenant: `resolved_spec_map` filtered by `OrgSlug`;
single-tenant fallback: `config_manager.sensor_specs` filtered by `sensor_id == client_id`).
For org-c with 4 sensors (crowdstrike, cyberint, claroty, armis), `prism_describe(org-c)`
returns table entries with distinct, fully-qualified `name` values: `crowdstrike_detections`,
`cyberint_alerts`, `claroty_devices`, `claroty_audit_logs`, `armis_devices`, etc. No two
`name` entries are identical (the disambiguation guarantee). The `example_query` field uses
the same sensor-prefixed name.

**CRIT-1 fix (v2.0) + F-L2-CRIT-001 fix:** `build_example_query` derives the example query from
the table's actual column spec using a priority ladder (highest priority wins and overrides lower):

| Priority | Condition | Query emitted |
|----------|-----------|---------------|
| 1 (highest) | `Integer` or `Float` column present | `SELECT <field>, COUNT(*) FROM <t> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10` |
| 2 | `severity` column present AND sensor prefix in `SENSOR_SEVERITY_VOCABULARY` (crowdstrike → Title-case `'High','Critical'`; armis → UPPER-case `'HIGH','CRITICAL'`; cyberint → lowercase `'high','critical'`; claroty → not registered, no `severity` column) | `SELECT * FROM <t> WHERE severity IN ('<high>', '<critical>') LIMIT 50` |
| 3 | `Datetime` column found (no Integer/Float or no-vocabulary severity) | `SELECT COUNT(*) FROM <t> WHERE <datetime_col> > NOW() - INTERVAL '1h'` |
| 4 (fallback) | No `Datetime` column and no Integer/Float (e.g., `claroty_devices`) | `SELECT * FROM <t> LIMIT 25` |

The datetime column at priority 3 is derived from the FIRST `ColumnType::Datetime` column in
the spec — NOT a hardcoded `'timestamp'` literal. Tables without any datetime column previously
produced a non-executable `example_query` (the old code hardcoded `WHERE timestamp > NOW() - INTERVAL '1h'`
regardless of the actual schema). For sensors with a severity column but no registered vocabulary
(unknown sensor prefix), the severity filter is suppressed and the query falls back to priority 3
or 4 rather than emitting literals that silently return 0 rows from DTU data. The `pql_hints`
array contains a generic usage hint (`"Use 'SELECT * FROM <table> LIMIT 25' to query any of the
N table(s) above."`) — no embedded table names.

**Red Gate tests:**
- `test_bc_2_10_012_audit_001_sensor_prefixed_table_names` — single-tenant 3-sensor call; assert no two `name` fields are identical; assert each `name` equals `format!("{sensor_id}_{table_name}")`; assert each `example_query` references the fully-qualified name.
- `test_bc_2_10_012_audit_001_multi_tenant_sensor_prefixed_unique` — multi-tenant resolved_spec_map path; assert sensor-prefixed uniqueness.
- `test_crit1_no_datetime_column_produces_column_free_query` (inline test in `prism_describe.rs`) — table with NO datetime column produces `SELECT * FROM claroty_devices LIMIT 25` (not `WHERE timestamp > ...`).

---

### Area D-B — prism_describe: Category-2 Enrichment UDF Discovery Hints (pql_hints)

**AC-CAT2** (traces to BC-2.10.012 v1.7 §pql_hints Category-2):

`build_pql_hints` in `crates/prism-mcp/src/tools/prism_describe.rs` gains a 4th parameter
`infusion_registry: Option<&prism_spec_engine::InfusionRegistry>`. The `pql_hints` array behavior
for non-empty-tables calls:

- When `infusion_registry` is `Some(reg)` AND `reg.udf_descriptors()` is non-empty:
  `pql_hints[2]` is the enrichment-presence hint. UDFs are sorted alphabetically by name
  (`str::cmp`); each entry formatted as `<name>(<input_field>)`; the first sorted entry is used as
  the example call. Byte-exact format (no trailing period or space variations):
  `"Enrichment available via pipe syntax: | enrich <first>. Available UDFs for this client: <comma-joined list>"`
- When `infusion_registry` is `None` OR `reg.udf_descriptors()` is empty:
  `pql_hints[2]` = `"No enrichment UDFs are registered for this client — enrichment is not available."`
- For any non-empty-tables call: `pql_hints.len() == 3` (always — regardless of registry state).
- When `tables` is empty: `pql_hints.len() == 1` (Category-2 suppressed — no enrichment hint
  when no tables are returned).

`InfusionUdfDescriptor` gains `pub input_field: String`; its `new()` constructor gains this
parameter; `udf_descriptors()` propagates `field.input_field.clone()` per descriptor; all
prism-query `new()` callers (including test fixtures) are updated to pass `""` for `input_field`
(TD-VSDD-060 sibling-site sweep — approximately 10 call sites).

Call-site wiring in `handle_prism_describe` (ADR-022 §C — adding proper plumbing, not redesign):
```rust
let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());
// ... (mirrors existing org_registry pattern)
build_pql_hints(&tables, org_registry.as_deref(), ..., infusion_registry.as_deref())
```

**Red Gate tests** (all in `crates/prism-mcp/tests/bc_2_10_012_audit_001_test.rs`):

- `test_bc_2_10_012_cat2_enrichment_hint_with_udfs` — construct an `InfusionRegistry` with 2 UDFs:
  `nvd_cvss` (input_field `device_cves_first`) and `threat_score` (input_field `ioc_value_singleton`);
  call `build_pql_hints` with non-empty tables and `Some(&registry)`; assert `pql_hints.len()==3`;
  assert `pql_hints[2]` equals exactly
  `"Enrichment available via pipe syntax: | enrich nvd_cvss(device_cves_first). Available UDFs for this client: nvd_cvss(device_cves_first), threat_score(ioc_value_singleton)"`.
  (Sorted alphabetically: `nvd_cvss` before `threat_score`.)

- `test_bc_2_10_012_cat2_enrichment_absent_hint` — call `build_pql_hints` with N≥1 tables and
  `infusion_registry: None`; assert `pql_hints.len()==3`; assert `pql_hints[2]` equals exactly
  `"No enrichment UDFs are registered for this client — enrichment is not available."`.

- `test_bc_2_10_012_cat2_zero_table_no_category2` — call `build_pql_hints` with N=0 tables and
  a non-empty `InfusionRegistry`; assert `pql_hints.len()==1` (Category-2 suppressed).

---

### Area E — MCP Prompts: FROM-Ready Table Names in All Prompt Bodies (AUDIT-004)

> **SCOPE NOTE (v2.0 OBS-1 clarification):** AUDIT-004 scope = scan 5 `render_*` functions,
> modify 4 (the 4 that contained dot-notation FROM refs). `render_query_tutorial` uses
> `<sensor_table>` placeholder syntax (no hardcoded table names) and was NOT modified —
> it was already compliant. Only `render_triage_alerts`, `render_client_overview`,
> `render_cross_client_status`, and `render_investigate_host` were changed.

**AC-AUDIT-004** (traces to BC-2.10.016 v1.2 postcondition — FROM-ready table names in prompt
bodies, EC-10-016-005 / EC-10-016-006): The 4 affected `render_*` functions in
`crates/prism-mcp/src/prompts.rs` (`render_triage_alerts`, `render_client_overview`,
`render_cross_client_status`, `render_investigate_host`) MUST NOT emit dot-notation table
references (`FROM crowdstrike.alerts`, `FROM claroty.alerts`, `FROM armis.devices`, etc.)
in any embedded PrismQL query in their message body. Every FROM clause in a rendered prompt
MUST use sensor-prefixed underscore-qualified names that resolve without error:
`crowdstrike_detections`, `armis_devices`, `claroty_devices`, `claroty_audit_logs`,
`cyberint_alerts`. A regex scan `FROM\s+\w+\.\w+` across all five rendered prompt bodies
MUST return zero matches. `render_query_tutorial` (uses `<sensor_table>` placeholder) is
unchanged and must not be inadvertently broken.

**MED-1 prompt-value fix (v2.0):** All filter VALUES in embedded queries across the 4 modified
prompts now match each DTU's exact emitted vocabulary. The `all-FROM-resolve` guard validates
that every FROM-target name in every rendered prompt is a currently-registered table name
(not a stale or fabricated one). This prevents the class of invisible failures where the
prompt text looked correct but the embedded query would return 0 rows because the filter
value used a casing or label not present in the DTU-emitted data.

**Red Gate tests:**
- `test_bc_2_10_016_audit_004_no_dot_notation_in_prompts` — call all 5 render_* functions; assert regex `FROM\s+\w+\.\w+` has zero matches; assert each body contains at least one valid FROM reference.
- `test_bc_2_10_016_audit_004_prompt_from_targets_include_registered_table` — assert all FROM targets in rendered prompts are registered in TableRegistry (all-FROM-resolve guard, MED-1).
- `test_bc_2_10_016_audit_004_column_refs_resolve_to_real_columns` — assert column references in embedded queries resolve to columns that exist in the spec.
- `test_bc_2_10_016_audit_004_column_sets_loaded_for_all_sensor_tables` — assert column sets are populated for all sensor tables used in prompts.
- `test_bc_2_10_016_med2_prompt_filter_values_match_dtu_vocabulary` — assert filter VALUES in embedded queries match each DTU's exact emitted vocabulary (MED-2 column-value-vocabulary validation).

---

### Regression and Workspace Gate

**AC-REG-1** (traces to BC-2.11.001 v1.15 invariant — DI-019 and DI-008, and
`#[non_exhaustive]` discipline in CLAUDE.md §Conventions): Full workspace `just check` exits
0 after all code fixes. No existing tests regress.

**REQUIRED (not optional):** The N1-B net-new work introduces `EnrichUdfNotFoundDetails` as
a new `#[non_exhaustive]` public struct in `prism-core/src/error.rs`. This MUST be reflected
in ALL of:
1. `EnrichUdfNotFoundDetails` carries `#[non_exhaustive]` attribute (required by CLAUDE.md discipline).
2. `scripts/check-non-exhaustive.sh EXPECTED` is incremented from `87` to `88` (the compile-fail gate count). The gate script is `scripts/check-non-exhaustive.sh`; CI invokes it via `.github/workflows/ci.yml`. Verify with: `grep EXPECTED= scripts/check-non-exhaustive.sh` → must show `EXPECTED=88`.
3. The CLAUDE.md `#[non_exhaustive]` sentence (currently "87 types currently enforced") is
   updated to `88` with `EnrichUdfNotFoundDetails` added to the attribution parenthetical.
4. The perimeter/non-exhaustive compile-fail gate (`tests/external/non-exhaustive-violation/`)
   continues to pass with `EXPECTED=88`.

All four changes MUST land in the same atomic commit as the `EnrichUdfNotFoundDetails` struct
definition. A story is NOT DONE if `EXPECTED` in `scripts/check-non-exhaustive.sh` still reads `87`
after this story merges.

**Red Gate verification:** `just check` exit code 0 (workspace gate). Additionally, the
implementer MUST verify `grep 'EXPECTED=' scripts/check-non-exhaustive.sh` shows `88` before
declaring done.

**AC-REG-2** (traces to BC-2.11.022 v1.1 invariant — CI 3-tier gate): The existing
`REFERENCE_EXAMPLES` CI round-trip gate tests continue to pass: (1) positive examples parse
as `Ok(_)`, (2) E-QUERY-040 negative examples return `Err(PrismError::RedundantRowLimit)`,
(3) registry-parity gate passes with the corrected per-field UDF deduplication. The AC-N1
fix specifically updates the registry-parity assertion to verify per-field names (not
infusion_id aggregate names). No gate test previously passing may regress.

**Red Gate test:** `test_bc_2_11_022_registry_parity` (existing in
`crates/prism-mcp/tests/reference_content.rs` — integration test file, not a src inline module)
— this is the load-bearing per-field UDF parity guard: it builds a known `InfusionRegistry`
with two infusion specs (`geoip` / `threatintel`) and asserts `build_reference_content` renders
the per-field callable names (`enrich geoip_country(col)`, `enrich threatintel_score(col)`), not
the infusion_ids. This test MUST continue to pass as the N1 regression guard.

Note: `test_bc_2_11_022_ci_3tier_gate` (also in `crates/prism-mcp/tests/reference_content.rs`) guards the 3-tier
`ExampleKind` shape (Positive / NegativeE040 / NegativeOther) — it is a separate concern from
per-field UDF parity. Both tests remain in the file; only `test_bc_2_11_022_registry_parity`
is the per-field-UDF-parity guard.

---

### Area F — Gate Coverage: Enrich Gate at All AST Positions (C1/C2)

**AC-C1C2** (traces to BC-2.11.019 v1.6 postcondition — gate covers all scalar-expr positions):
`collect_unknown_scalars_from_sql_query` walks ALL scalar-expression positions in a `SqlQuery`:
SELECT projections, WHERE predicate, JOIN ON conditions (typed as `Expr` in the AST), GROUP BY
expressions, ORDER BY expressions, and HAVING predicate. For both `Ast::Sql(Select)` and
`Ast::SqlPipe` head queries, `check_enrich_udf_availability` uses this SINGLE canonical walk.
A query like `SELECT crowdstrike_detections.severity FROM crowdstrike_detections GROUP BY
badudf(col)` returns `PrismError::EnrichUdfNotFound` — the GROUP BY position is NOT bypassed.
Similarly for ORDER BY and JOIN ON positions. Before this fix (C1/C2 finding), only the SELECT
projection and WHERE positions were scanned; GROUP BY, ORDER BY, and JOIN ON were silent bypass
paths.

**Red Gate tests (unit-level, no InfusionRegistry wiring needed):**
- `test_c1_collect_unknown_scalar_from_sql_query_group_by` — programmatic `SqlQuery` with unknown scalar in GROUP BY; assert `collect_unknown_scalars_from_sql_query` collects the name.
- `test_c1_collect_unknown_scalar_from_sql_query_order_by` — GROUP BY → ORDER BY variant.
- `test_c2_collect_unknown_scalar_from_sql_query_join_on` — programmatic `SqlQuery` with unknown scalar in JOIN ON condition; assert collected.

**Red Gate tests (engine-level, full InfusionRegistry wiring):**
- `test_c1_sql_group_by_unknown_scalar_triggers_e_query_039` — full engine execute with GROUP BY badudf; assert `Err(EnrichUdfNotFound)`.
- `test_c1_sql_order_by_unknown_scalar_triggers_e_query_039` — ORDER BY variant.
- `test_c1_sqlpipe_group_by_unknown_scalar_triggers_e_query_039` — SqlPipe mode GROUP BY variant.

---

### Area G — Gate Coverage: E-QUERY-038 Column Gate in Single-Tenant Mode (M1)

**AC-M1** (traces to BC-2.11.016 v1.23 postcondition — E-QUERY-038 fires for unknown columns): The
`check_query_column_availability` function (E-QUERY-038 column gate) MUST fire in single-tenant
mode where `resolved_spec_map` is `None`. Previously it returned `Ok(())` immediately in this
case, silently bypassing E-QUERY-038 for all single-tenant queries. The fix: `TableRegistry`
gains a `columns_by_table` field (populated by `register_sensor` from `[[tables]][*].columns`
spec entries) and a `columns_for_table(table_name: &str) -> Vec<String>` method. The
`check_query_column_availability` function accepts an additional `table_registry: Option<&TableRegistry>`
parameter; when `resolved_spec_map` is `None` but `table_registry` is `Some(r)`, it calls
`r.columns_for_table(table_name)` to validate columns. Fail-open: if `columns_for_table`
returns an empty `Vec` (table has no columns in spec or is unregistered), the column gate
skips that table rather than blocking — this preserves the existing behavior for tables without
column metadata.

> **EC for single-tenant column gate:** A query `SELECT unknown_field FROM cyberint_alerts`
> in single-tenant mode now returns `E-QUERY-038` (column gate) rather than executing and
> returning 0 rows or an opaque error.

---

### Area H — Gate Coverage: E-QUERY-038 Column Gate Validates GROUP BY / ORDER BY / JOIN ON (M2)

**AC-M2** (traces to BC-2.11.016 v1.23 postcondition — E-QUERY-038 fires for unknown columns at
all relevant positions): `check_query_column_availability` validates GROUP BY expressions,
ORDER BY expressions, JOIN ON column refs, and HAVING predicate column refs in addition to
SELECT projections and WHERE predicates. Previously GROUP BY, ORDER BY, JOIN ON, and HAVING
column references were not checked, creating bypass paths where an invalid column in
`GROUP BY invalid_col` or `HAVING count(typo_col)` would not fire E-QUERY-038 at plan time.

For the HAVING position specifically (BC-2.11.016 v1.23 §Implementation location gate-positions
table, Position 6):

Position 6 (HAVING): uses `extract_predicate_columns_with_bareness` (same helper as WHERE/Position 2), which
calls `collect_predicate_columns`. The `Predicate::Compare` arm in `collect_predicate_columns`
now handles both bare `Expr::Field` LHS (bare-column HAVING predicates) and `Expr::FuncCall`
LHS (aggregate-function HAVING predicates, ADR-048), in both cases recursing via
`extract_field_paths_with_bareness` to collect all `(name, is_bare)` column reference pairs. The WHERE
predicate grammar deliberately does NOT accept aggregate-function predicate LHS (ADR-048);
`WHERE count(col) > 5` remains an E-QUERY-001 parse error.

For positions 1/3/4/5, `extract_field_paths_with_bareness` is the extraction helper
used to recurse into FuncCall args and find `Expr::Field` references as `(name, is_bare)` pairs,
enabling the HEAD-JOIN SUSPENSION gate to distinguish bare from qualified column references.

> **BC-2.11.016 v1.5 HAVING addition (F-PWL1-LOW-001):** HAVING is the 6th column-gate
> position, added at v1.5 to close a coverage asymmetry: sibling gates E-QUERY-037 and
> E-QUERY-039 already walk HAVING; omitting it from E-QUERY-038 caused a `HAVING count(typo_col)`
> typo to bypass the clean column-not-found diagnostic and surface a less-actionable DataFusion
> error. HAVING uses the same `Option<Predicate>` type as WHERE and the same
> `extract_predicate_columns` extraction path — zero new machinery.
>
> **ADR-048 HAVING agg-fn grammar extension (F-PXL3-MED-002):** The HAVING predicate grammar was
> extended (via `build_having_predicate_parser`) to accept the `agg_fn(col) op literal` predicate
> form in addition to bare-column comparisons. This allows `HAVING count(typo_col) > 5` to parse
> successfully and reach the E-QUERY-038 column-gate (previously it was an E-QUERY-001 parse error).
> WHERE deliberately does NOT receive this grammar extension (ADR-048 §Constraint); `WHERE count(col)
> > 5` remains an E-QUERY-001 parse error. The `Predicate::Compare` arm in `collect_predicate_columns`
> now handles `Expr::FuncCall` LHS (aggregate-function HAVING predicates) by recursing via
> `extract_field_paths_from_expr` to find the column reference inside the function argument.
>
> Tests in `engine.rs` inline module `f_pwl1_low001_having_column_gate_tests` (bare-column gate):
> - `test_BC_2_11_016_having_column_gate_typo_fires_e_query_038` — bare-column HAVING typo fires E-QUERY-038
> - `test_BC_2_11_016_having_column_gate_valid_col_no_e_query_038` — valid bare-column HAVING passes gate
>
> Tests in `engine.rs` inline module `f_pxl3_med002_having_agg_predicate_col_gate_tests` (agg-fn gate, ADR-048):
> - `test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038` — agg-fn HAVING typo fires E-QUERY-038 (ADR-048)
> - `test_BC_2_11_016_having_agg_fn_predicate_valid_col_no_e_query_038` — agg-fn HAVING with valid col passes gate (ADR-048 acceptance)
> - `test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001` — WHERE does NOT accept agg-fn predicate form (ADR-048 divergence guard)

---

### Area I — Gate Coverage: E-QUERY-037 Source Walk Covers All Subquery Positions (L1)

**AC-L1** (traces to BC-2.11.001 v1.15 postcondition — availability gate is mode-agnostic):
The `extract_sources_from_ast_for_gate` function in `table_registry.rs` now covers subquery
sources at ALL expression positions: HAVING, GROUP BY, ORDER BY, and JOIN ON. The new
`collect_expr_sources_into_gate` helper walks `Expr::InSubquery`, `Expr::FuncCall`, `Expr::Compare`,
`Expr::Logical`, `Expr::Not`, and `Expr::TimestampArithmetic` to find any `SourceRef` embedded
in those positions. Before this fix, a FROM subquery embedded in a GROUP BY expression would
bypass the E-QUERY-037 gate and fail later with a less actionable error.

---

### Area I-B — Armis Entity-Discriminator Seeding (F-L2-CRIT-001)

**AC-DISC** (traces to BC-2.11.007 v1.9 §Mechanism B.1 / PC-DISC-001 — Planner-Side Entity-Discriminator Auto-Seeding):
`pub(crate) fn seed_armis_entity_discriminator` in `crates/prism-query/src/materialization.rs`
seeds the AQL search discriminator for armis tables in the `run_materialization_pipeline`
fan-out loop when the `aql` field is absent or empty:

| Table | Seeded `aql` value |
|-------|--------------------|
| `armis_alerts` | `in:alerts` |
| `armis_devices` | `in:devices` |
| (all other tables) | no change — filter unchanged |

A user-supplied non-empty `aql` predicate is preserved verbatim (the discriminator is
only seeded when `aql` is absent or empty). Non-armis tables are unaffected.

**Observable behavior:** Before this fix, a demo or prompt query against `armis_alerts`
with no explicit `aql` predicate would search with no discriminator, causing the Armis API
to return DEVICE records (the default entity type) — resulting in 0 ALERT rows. After this
fix, `armis_alerts` queries with absent/empty `aql` automatically receive `aql="in:alerts"`,
returning alert records as expected.

**Red Gate tests** (inline module `armis_discriminator_tests` in `materialization.rs`,
verified GREEN on HEAD d9bb75c2):
- `test_f_l2_crit001_armis_alerts_no_aql_seeds_in_alerts_discriminator` — armis_alerts table with absent `aql`; assert seeded value equals `"in:alerts"`.
- `test_f_l2_crit001_armis_devices_no_aql_seeds_in_devices_discriminator` — armis_devices table with absent `aql`; assert seeded value equals `"in:devices"`.
- `test_f_l2_crit001_armis_alerts_existing_aql_not_overwritten` — armis_alerts table with a non-empty user-supplied `aql`; assert value is preserved verbatim (NOT overwritten to `"in:alerts"`).
- `test_f_l2_crit001_non_armis_table_filters_unchanged` — non-armis table (e.g., `crowdstrike_detections`); assert filter is unchanged.

**Wiring-seam Red Gate tests** (F-LENS4-MED-001 — inline module `armis_discriminator_wiring_seam_tests`
in `materialization.rs`; these drive `run_materialization_pipeline` through a recording stub adapter,
asserting the seeded `aql` reaches `QueryParams` — guarding the call site against revert; Red→Green
confirmed on HEAD d9bb75c2):
- `test_F_LENS4_MED001_armis_alerts_pipeline_seeds_in_alerts_aql_filter` — end-to-end pipeline with armis_alerts and absent `aql`; assert `QueryParams.filters["aql"] == "in:alerts"` at the adapter call site.
- `test_F_LENS4_MED001_armis_devices_pipeline_seeds_in_devices_aql_filter` — companion for armis_devices; assert `QueryParams.filters["aql"] == "in:devices"`.
- `test_F_LENS4_MED001_armis_alerts_user_supplied_aql_passes_through_pipeline` — armis_alerts with user-supplied `WHERE aql = 'in:alerts status:Open'`; assert the user-supplied value passes through to `QueryParams` unchanged (not overwritten).

**Integration verification:** DTU armis integration tests (`s_demo_armis_aql_001_red_gate`,
parity_armis pipeline roundtrips, adv_p02 armis) confirmed GREEN on full `just check`
(feature HEAD d9bb75c2).

---

### Area J — Capability-Gate Ordering Symmetric Across execute_inner / execute_scheduled_inner (H1)

**AC-H1** (traces to BC-2.11.001 v1.15 invariant — gate ordering is consistent):
The capability gate (E-QUERY-011, `check_internal_table_capabilities`) in `execute_scheduled_inner`
has been moved AFTER E-QUERY-037, E-QUERY-038, and E-QUERY-039, mirroring the canonical order
in `execute_inner`. Previously in `execute_scheduled_inner`, E-QUERY-011 ran BEFORE E-QUERY-037,
E-QUERY-038, and E-QUERY-039, causing asymmetric first-error behavior: a query with both an
unknown table and a capability violation would report the capability error in scheduled mode
but the table error in interactive mode. Post-fix: the canonical gate sequence is identical for
both execution paths: E-QUERY-001 (parse) → E-QUERY-037 (table) → E-QUERY-038 (column) →
E-QUERY-039 (enrich) → E-QUERY-011 (capability, last pre-I/O gate).

---

### Area K — Case-Insensitive Querying (Split to S-PRISMQL-CASE-INSENSITIVE-001)

> **Out-of-scope deferred split (v2.0):** Case-insensitive querying (IEQ/IIN operators and
> adapter canonical-case normalization) is demo-critical but architecturally distinct from
> the gate-coverage and fidelity fixes in this story. Per human direction, it is split to a
> SEPARATE story **S-PRISMQL-CASE-INSENSITIVE-001**. The prompt VALUES implemented in this
> story (AUDIT-004 / AC-AUDIT-004) use each sensor's exact CURRENT casing as an interim
> (matching DTU-emitted values), which unblocks T13 recording. Case-insensitive querying
> will be delivered via S-PRISMQL-CASE-INSENSITIVE-001 in a subsequent cycle.

---

### Area L — TLS Standardization: native-tls → rustls-tls for DTU Test Infrastructure (TLS-REMEDIATION)

> **ROOT CAUSE CORRECTION (v2.15 fold-in — commit cf66151f):** During un-parking of this
> branch (rebased onto develop@aaa9bfe8), the REAL root cause of the 4 quarantined DTU
> stage-0 integration test failures was identified and fixed. The 4 tests were NOT failing
> due to "WASMtime plugin-init starvation" (the original misdiagnosis). The actual root
> cause was: macOS `native-tls` backend initializes the Security.framework Keychain
> per-test-process, adding ~65s of one-time overhead per test binary. Under full-suite
> parallel load (nextest running multiple test binaries concurrently), the stage-0 window
> (50s) was exhausted by TLS init BEFORE any HTTP request was issued — causing the 4
> stage-0 scenario tests to fail DETERMINISTICALLY (not flakily). The fix:
> standardize the `reqwest` dependency to `rustls-tls` (default-features=false) across
> all affected test configurations. `rustls-tls` init is ~0ms, giving ~800x margin over
> the 50s window.

**AC-TLS** (TLS-REMEDIATION compliance — DTU stage-0 test un-quarantine; partial BC-2.06.019 v1.15 context):

The 4 previously-quarantined DTU integration tests MUST pass when run under full-suite
parallel load (nextest default concurrency) on macOS:

| Test | Crate | Removal commit |
|------|-------|----------------|
| `test_BC_2_06_019_armis_primary_device_stage_visibility` | `prism-dtu-armis` | cf66151f |
| `test_BPRL_P4_02_armis_alerts_stage_guard_primary_device` | `prism-dtu-armis` | cf66151f |
| `test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility` | `prism-dtu-armis` | cf66151f |
| `test_BPRL_P4_02_detections_stage_guard_primary_device` | `prism-dtu-crowdstrike` | cf66151f |

**Additional observable behaviors (all part of AC-TLS):**

1. **`just check` passes workspace-wide** after the TLS standardization (5085 tests pass,
   Cargo.lock shrinks by −151 lines as native-tls and its macOS Security.framework transitive
   deps are removed from the lock file).

2. **Production reqwest deps are unchanged.** The `[dependencies]` (production) reqwest
   entries in `prism-bin`, `prism-spec-engine`, and `prism-sensors` were ALREADY configured
   with `rustls-tls` before this fix. This remediation touches ONLY `[dev-dependencies]`
   entries (test binaries) and the optional `download` feature `[dependencies]` in
   `ocsf-proto-gen`. The production TLS configuration and runtime security posture are
   unchanged — confirmed by the security review APPROVE on commit cf66151f.

3. **All 11 Cargo.toml entries use `default-features=false` with the `rustls-tls` feature**
   (not the mixed `native-tls`/`rustls-tls` feature set that caused the init overhead).

4. **`prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs`** — 7 tests in this
   file gain explicit `stop()` resource-cleanup calls to ensure DTU clone shutdown completes
   before the test process exits, preventing port-reuse races in subsequent test runs.

**Verification:** Run `cargo nextest run -p prism-dtu-armis -p prism-dtu-crowdstrike
--no-fail-fast` and confirm the 4 formerly-quarantined tests pass. Run `just check` and
confirm workspace exits 0.

> **No BC authorship required for TLS standardization.** The rustls-tls standardization is
> a test-infrastructure dependency configuration change, not a behavioral contract change.
> BC-2.06.019 (DTU stage-0 scenario visibility) is unchanged — the tests simply RUN now
> rather than being quarantined. The workspace-wide rustls-tls-only convention for reqwest
> dev-dependencies IS codified as **ADR-050** (workspace reqwest rustls-tls backend
> convention, ACCEPTED), established during this story.

---

### Deferred / Out-of-Perimeter Items (v2.0)

The following items were identified during the LOCAL adversarial cascade but are out of scope
for this story and deferred with explicit anchors:

| Item | Disposition | Target |
|------|-------------|--------|
| 4x-query-reparse perf (enrich gate re-parses the full query string instead of accepting the already-parsed AST) | Performance, not correctness; no user-visible impact at current demo scale | S-QUERY-GATE-REPARSE-CONSOLIDATION-001 |

---

### Demo Evidence and SAP Compliance

**AC-DEMO-001**: After all five code fixes land on the feature branch, a demo-recorder run
captures evidence for each finding:
- Evidence-N1: `prismql://reference` enrichment section listing six per-field UDF names (`threat_score`, etc.),
  NOT `threat_intel` / `nvd`.
- Evidence-N1B: calling `FROM cyberint_alerts | enrich threat_intel(iocs_value)` returns E-QUERY-039
  with `available_infusions` listing registered per-field names.
- Evidence-N2: `FROM cyberint.alerts` returns E-QUERY-037 with `did_you_mean: "cyberint_alerts"`
  (not a silent 0-row result).
- Evidence-AUDIT-001: `prism_describe(org-c)` returns table names `cyberint_alerts`,
  `claroty_devices`, `armis_devices`, etc. (no bare `alerts` / `devices` collisions).
- Evidence-AUDIT-004: `triage_alerts` prompt body contains `FROM crowdstrike_detections`
  (no `FROM crowdstrike.alerts`).

**AC-SAP-1** (SAP-1 standing-probe compliance — structured event catalog discipline; partial
BC-2.16.002 dependency): The implementer MUST run `rg 'event_type\s*=' crates/ --type rust`
after each fix and confirm every emission site has a catalog row in BC-2.16.002 §Postconditions.
SAP-1 discipline covers not only new bare `event_type` literal values but also extensions to
closed-set `method`/`label` enumerations within existing catalog rows.

For this delivery:

- **No new `event_type` literal value was introduced.** The enrich gate (`check_enrich_udf_availability`),
  column-free helper (`columns_for_table` success path), prompt renderer, describe helper
  (`build_example_query`, `build_tables_for_client`), and MCP mapper (`map_prism_error`) all
  use `?`-propagation and do not emit any new `tracing::*!(event_type=…)` call. The enrich
  gate fix (N1-B) and prompt fixes (AUDIT-004) route errors through the existing map_prism_error
  surface; no new emission sites.

- **However, two new closed-set `method` labels WERE introduced** on the existing
  `table_registry.rwlock_poisoned` catalog row, and the `column_not_found.rejected` row was
  extended with a second emission site:

  1. **M1 fix** (`columns_for_table` in `crates/prism-query/src/table_registry.rs`): emits
     `method = "columns_for_table"` when the `columns_by_table` RwLock is poisoned — returns
     empty `Vec` (fail-closed, column-gate single-tenant path degradation). This is the 7th
     method/label in the `table_registry.rwlock_poisoned` catalog row.

  2. **N2 fix** (`check_availability_gate` dot-notation rejection block in
     `crates/prism-query/src/table_registry.rs`): emits `method = "check_availability_gate.dot_notation"`
     when the `sensor_by_table` RwLock is poisoned during org-visible-tables snapshot construction
     for the dot-name error report — falls back to empty map; dot-notation `TableNotAvailable`
     error is still returned. This is the 8th label in the `table_registry.rwlock_poisoned`
     catalog row.

  3. **M1 fix** also added a second emission site to the `column_not_found.rejected` row:
     the `if resolved_spec_map.is_none()` branch in `check_column_availability`
     (`crates/prism-query/src/engine.rs`) now emits `column_not_found.rejected` WARN on the
     single-tenant path when `TableRegistry::columns_for_table` returns a non-empty column list
     that does not contain the referenced column. Both sites (single-tenant and multi-tenant)
     emit identical field schema (`column: %display`, `table: %display`, `client_id: %display`,
     `available_count: usize`).

  These additions required (and received) BC-2.16.002 v1.90→v1.91 catalog amendment
  (F-PHL3-MED-001, authored by product-owner 2026-06-28). The v1.91 amendment updated the
  `table_registry.rwlock_poisoned` row's description from "Six methods" to "Eight methods/labels"
  and extended the `column_not_found.rejected` row's trigger description to reflect both
  emission sites. The SAP-1 PG-LP11-001 obligation for this delivery is fulfilled by BC-2.16.002
  v1.91.

**§References:** BC-2.16.002 §Canonical Structured Event Catalog (this story's SAP-1 M1/N2 obligation was fulfilled by the v1.91 amendment; subsequent v1.92 amendment by S-PERF-GATE-008 is unrelated to this story). The enrich-last gate ordering fix (N1-B) requires no catalog amendment
because `check_enrich_udf_availability` uses `?`-propagation only.

ADR-048 governs the HAVING/WHERE predicate-grammar divergence: the `agg_fn(col) op literal`
predicate form is added to HAVING only (via `build_having_predicate_parser`); WHERE deliberately
does not receive this form. BC-2.11.016 v1.5's claim that `HAVING count(typo_col) > 5` triggers
E-QUERY-038 is now deliverable as a result of the ADR-048 grammar extension (F-PXL3-MED-002).
BC-2.11.016 stays at v1.5 — no version change required.

ADR-050 (workspace reqwest rustls-tls backend convention, ACCEPTED) governs the rustls-tls-only
standardization delivered by the TLS-REMEDIATION scope of this story. ADR-050 was authored by the
architect for this story (`anchor_stories: [S-DEMO-FIDELITY-REMEDIATION-001]`); it codifies the
`default-features=false, features=["rustls-tls"]` convention for all workspace reqwest entries.

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec (v2.23) | ~20,000 |
| BC files (7 BCs, BC-2.10.012 now v1.7) | ~14,000 |
| Source files touched (resources.rs, prompts.rs, prism_describe.rs, error.rs, table_registry.rs, engine.rs, error_mapping.rs + new test files) | ~32,000 |
| DTU Cargo.toml files (11 entries — 9 DTU crates + prism-bin + ocsf-proto-gen) | ~2,000 |
| DTU test files (sec_p3_003_constant_time_admin_token.rs + 2 un-quarantine test files) | ~3,000 |
| Research/audit docs (2) | ~6,000 |
| Test files (existing + new — 54 Red Gate tests across 8 new test files) | ~19,000 |
| Tool outputs (grep, rg scans, call-chain traces) | ~4,000 |
| **Total estimate** | **~100,000** |

Within the 20-30% context window budget for a Sonnet-class agent context (≈200k tokens).
The story has grown from the original 5-finding scope to cover gate-coverage expansion,
describe correctness, prompt-value fixes, and the TLS-REMEDIATION fold-in (v2.15). It
remains within a single dispatch budget — no splitting required. Implementations should be
delivered in sub-bursts to avoid context overflow (more than 8 artifacts → sub-burst A:
create files, sub-burst B: update indexes).

---

## Tasks

### N1 — build_reference_content dedup key fix
- [ ] 1. Read `crates/prism-mcp/src/resources.rs`; locate `build_reference_content`;
       identify deduplication logic iterating by `infusion_id`; change dedup key to
       `descriptor.name`; update emitted format to `enrich {name}(col)`.
- [ ] 2. Write Red Gate test `test_bc_2_11_022_n1_per_field_udf_names`; confirm RED.
- [ ] 3. Apply the fix; confirm test GREEN.

### N1-B — E-QUERY-039 net-new implementation
- [ ] 4. Read `crates/prism-core/src/error.rs`; verify `EnrichUdfNotFound` variant DOES NOT
       exist (zero-match prerequisite); add `EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)`
       variant to `PrismError`; add `#[non_exhaustive] pub struct EnrichUdfNotFoundDetails`
       with fields `infusion: String`, `available_infusions: Vec<String>`,
       `did_you_mean: Option<String>`.
- [ ] 5. Increment `EXPECTED` in `scripts/check-non-exhaustive.sh` from `87` to `88`;
       update `CLAUDE.md` non-exhaustive sentence (87→88) and add `EnrichUdfNotFoundDetails`
       to the attribution list. Also update `.github/workflows/ci.yml` if it pins EXPECTED.
- [ ] 6. Add plan-time enrichment-validation pass in `crates/prism-query/src/engine.rs`
       BEFORE `check_availability_gate`/fan-out; collect enrichment function names via
       direct `match &ast { ... }` (not the `visit::Visitor` trait — avoids coupling with
       the full visitor infrastructure) — (a) pipe path: `PipeStage::Enrich` nodes → `EnrichStage.infusion`;
       (b) SQL path: `ScalarFunc::Unknown(name)` in SELECT projection expressions (reachable
       from real queries) AND WHERE predicates via `collect_unknown_scalar_from_predicate`
       (DEFENSIVE / forward-compat per BC-2.11.019 v1.6 §Precondition 1(b) AST-contract;
       real `WHERE udf(col)=v` is E-QUERY-001 parse error today; WHERE scan is exercised by
       programmatic AST unit tests, not real parsed query text); these are DISTINCT match
       arms but feed the same validation loop. For each collected `name`: if NOT in
       `InfusionRegistry.udf_to_infusion`, build the UDF name vec inline via
       `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`
       (do NOT call `udf_names()` — that method does not exist), then return
       `Err(PrismError::EnrichUdfNotFound(Box::new(EnrichUdfNotFoundDetails { infusion: name.to_owned(),
       available_infusions: <vec from above>, did_you_mean: strsim_closest(&name, &udf_names_vec) })))`.
       Gate fires BEFORE any fan-out or sensor I/O. No new public methods added to `InfusionRegistry`.
- [ ] 7. Read `crates/prism-mcp/src/error_mapping.rs` `map_prism_error`; add explicit
       `-32602` arm for `PrismError::EnrichUdfNotFound` (E-QUERY-039 — net-new); confirm
       no fall-through to `-32000`. NOTE: the `PrismError::TableNotAvailable` (E-QUERY-037)
       arm is CONFIRMED PRESENT (doc: "S-3.13 AC-2; BC-2.11.001") — do NOT add a duplicate
       or modify it. Only the E-QUERY-039 arm is added here.
- [ ] 8. Write Red Gate test `test_bc_2_11_019_n1b_infusion_id_as_udf_name`; confirm RED.
- [ ] 9. Write Red Gate test `test_bc_2_11_019_n1b_mcp_maps_to_32602`; confirm RED.
- [ ] 10. Apply the plan-time gate + error_mapping.rs fixes; confirm both tests GREEN.

### N2 — E-QUERY-037 dot-notation FROM gate ordering
- [ ] 11. Read `crates/prism-query/src/table_registry.rs`; trace `check_availability_gate`
        and `is_registered`; understand where the availability check fires relative to
        `sensor_id_from_table_name` dot-extraction; determine the correct insertion point
        in engine.rs for the pre-check. Do NOT look at materialization.rs for this fix —
        the gate lives in the table_registry/engine layer.
- [ ] 12. Ensure `TableRegistry::is_registered(table_name_as_written)` check fires in
        `check_availability_gate` BEFORE any fan-out routing; dot-notation strings must
        return `PrismError::TableNotAvailable(...)` (E-QUERY-037) with `did_you_mean`.
- [ ] 13. Write Red Gate test `test_bc_2_11_001_n2_dot_notation_from_target_e_query_037`;
        confirm RED (must cover pipe mode, SQL mode, AND filter-mode regression guard).
- [ ] 14. Apply the gate-ordering fix in table_registry.rs + engine.rs; confirm test GREEN;
        confirm filter-mode `crowdstrike_detections | severity='HIGH'` still passes.

### AUDIT-001 — prism_describe sensor-prefixed table names
- [ ] 15. Read `crates/prism-mcp/src/tools/prism_describe.rs` `build_tables_for_client`;
        change `name: table.table_name.clone()` → `name: format!("{sensor_id}_{}", table.table_name)`;
        update `example_query` grounding to use the same sensor-prefixed name.
- [ ] 16. Write Red Gate test `test_bc_2_10_012_audit_001_sensor_prefixed_table_names`;
        confirm RED.
- [ ] 17. Apply the fix; confirm test GREEN.

### AUDIT-004 — prompts.rs FROM-ready table names
- [ ] 18. Read `crates/prism-mcp/src/prompts.rs`; identify all four affected `render_*`
        functions with dot-notation FROM clauses; determine correct sensor-prefixed names
        from the actual sensor TOML specs (function-name anchor: `render_triage_alerts`,
        `render_client_overview`, `render_cross_client_status`, `render_investigate_host`);
        replace all dot-notation FROM references.
- [ ] 19. Write Red Gate test `test_bc_2_10_016_audit_004_no_dot_notation_in_prompts`;
        confirm RED.
- [ ] 20. Apply the fix; confirm test GREEN.

### AC-CAT2 — Category-2 enrichment UDF discovery hints (BC-2.10.012 v1.7)
- [ ] 24. Read `crates/prism-spec-engine/src/infusion/udf.rs`; add `pub input_field: String`
         to `InfusionUdfDescriptor` struct; update `new()` constructor to accept this param.
         Read `crates/prism-spec-engine/src/infusion/mod.rs`; update `udf_descriptors()` to
         propagate `field.input_field.clone()` per descriptor.
- [ ] 25. TD-VSDD-060 sibling-site sweep: `grep -rn 'InfusionUdfDescriptor::new(' crates/`
         — update ALL ~10 prism-query test-fixture call sites to pass `""` for `input_field`.
         Do NOT miss any site; the compiler will confirm exhaustiveness once the struct changes.
- [ ] 26. Update `build_pql_hints` in `crates/prism-mcp/src/tools/prism_describe.rs` to accept
         a 4th parameter `infusion_registry: Option<&prism_spec_engine::InfusionRegistry>`.
         Implement Category-2 logic per BC-2.10.012 v1.7 §pql_hints:
         when `tables` non-empty, compute `pql_hints[2]` (sorted UDFs as `<name>(<input_field>)`,
         byte-exact format); when `tables` empty, suppress (return only `pql_hints[0]`).
- [ ] 27. Wire `infusion_registry` call-site in `handle_prism_describe` (ADR-022 §C):
         `let infusion_registry = query_engine.and_then(|qe| qe.infusion_registry());`
         Pass `infusion_registry.as_deref()` as 4th arg to `build_pql_hints`
         (mirrors existing `org_registry` pattern).
- [ ] 28. Write three Red Gate tests in `crates/prism-mcp/tests/bc_2_10_012_audit_001_test.rs`;
         confirm RED:
         `test_bc_2_10_012_cat2_enrichment_hint_with_udfs` (2 UDFs, assert exact hint string),
         `test_bc_2_10_012_cat2_enrichment_absent_hint` (None registry, assert absent hint string),
         `test_bc_2_10_012_cat2_zero_table_no_category2` (N=0 tables, assert pql_hints.len()==1).
- [ ] 29. Apply fixes; confirm all three Cat2 tests GREEN; confirm AUDIT-001 tests still pass.

### Final gates
- [ ] 21. Run `just check` (full workspace); confirm EXIT 0.
- [ ] 22. Verify `grep 'EXPECTED=' scripts/check-non-exhaustive.sh` shows `88` (not 87).
- [ ] 23. Run `rg 'event_type\s*=' crates/ --type rust`; verify every emission has a
        BC-2.16.002 catalog row (SAP-1 compliance).

### TLS-REMEDIATION — native-tls → rustls-tls (folded in via commit cf66151f)

> **STATUS: IMPLEMENTED** (commit cf66151f, `just check` green, 5085 tests pass, security
> review APPROVE). These tasks are documented for traceability; the implementer verifies
> they are already done on the branch before PR merge.

- [x] TLS-1. Read each DTU crate's `[dev-dependencies]` in `Cargo.toml`; confirm
        `reqwest` uses `default-features=false, features=["rustls-tls"]` (not `native-tls`).
        Affected crates: prism-dtu-armis, prism-dtu-crowdstrike, prism-dtu-claroty,
        prism-dtu-cyberint, prism-dtu-slack, prism-dtu-pagerduty, prism-dtu-jira,
        prism-dtu-nvd, prism-dtu-threatintel (9 DTU dev-dep entries).
- [x] TLS-2. Read `crates/prism-bin/Cargo.toml` `[dev-dependencies]`; confirm
        `reqwest` uses `rustls-tls` (dev-dep only; production dep already was rustls).
- [x] TLS-3. Read `crates/ocsf-proto-gen/Cargo.toml`; confirm the optional `download`
        feature's `[dependencies]` reqwest uses `rustls-tls` (default-features=false).
- [x] TLS-4. Confirm the 4 `#[ignore]` attributes are removed:
        `test_BC_2_06_019_armis_primary_device_stage_visibility`,
        `test_BPRL_P4_02_armis_alerts_stage_guard_primary_device`,
        `test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility` (prism-dtu-armis);
        `test_BPRL_P4_02_detections_stage_guard_primary_device` (prism-dtu-crowdstrike).
- [x] TLS-5. Confirm 7 `stop()` calls added in
        `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs`.
- [x] TLS-6. Run the 4 formerly-quarantined tests at least 3 times; confirm no flakiness
        at ~0.05s vs 50s window (≥800x margin):
        `cargo nextest run -p prism-dtu-armis -p prism-dtu-crowdstrike -E 'test(BPRL_P4_02)' --no-fail-fast`
- [x] TLS-7. Run `just check` (full workspace); confirm EXIT 0 with 5085 tests.

---

## Previous Story Intelligence

**Predecessor:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (merged PR #203, develop@7e60df03).

**Key lessons from the predecessor cascade (directly applicable here):**

1. **TD-VSDD-091 line-number pins are fragile.** This story uses function-name anchors
   throughout (e.g., `build_reference_content`, `sensor_id_from_table_name`,
   `build_tables_for_client`, `render_triage_alerts`) — never file:line references.
   Verify all code citations use function names, not line numbers, before declaring done.

2. **`build_reference_content` is in `resources.rs`, NOT `resources/schema.rs`.** The
   old static `PQL_REFERENCE_CONTENT` include_str! was in `resources/schema.rs`; the new
   dynamic `build_reference_content` introduced by PR #203 is in `resources.rs`. The
   implementer MUST read and confirm the actual location before writing tests.

3. **SAP-1 tracing catalog discipline.** Every `event_type =` tracing emission must have
   a BC-2.16.002 catalog row. The predecessor had two recurrences of this finding (passes
   1 and 2). This story's fixes are likely to use `?`-propagation without new emissions —
   but the SAP-1 check (`rg 'event_type\s*='`) MUST still be run post-fix.

4. **SAP-2 does NOT apply here.** SAP-2 (DTU↔TOML schema parity) applies to sensor TOML
   spec changes. This story does not modify sensor TOML files — it modifies MCP layer code
   that reads from existing specs. SAP-2 scan is NOT needed.

5. **E-QUERY-039 (N1-B) is NET-NEW, not an investigation.** A 2026-06-26 remove-uncertainty
   pass confirmed that `PrismError::EnrichUdfNotFound` and `EnrichUdfNotFoundDetails` have
   ZERO workspace matches — the variant, struct, plan-time gate, and MCP mapping all need to
   be created from scratch per BC-2.11.019 v1.6. The original remediation plan framed this as
   a "gate should fire / routing fix" but that was based on the incorrect assumption that PR
   #203 implemented E-QUERY-039. It did not. The implementer MUST create the error type first
   (error.rs), then the gate (prism-query/engine.rs), then the MCP mapping (error_mapping.rs),
   in that order. TDD discipline: write `test_bc_2_11_019_n1b_infusion_id_as_udf_name` (RED)
   before adding any gate code.

   **CRITICAL (v1.2):** `InfusionRegistry` has NO `udf_names()` method. The public API is:
   `new`, `load_spec`, `load_spec_with_runtime`, `udf_descriptors`, `enrich_descriptor`,
   `is_api_backed`, `hot_reload`. Do NOT add a new method. Derive the UDF name vector inline:
   `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect::<Vec<_>>()`.
   The E-QUERY-037 `map_prism_error` arm is CONFIRMED PRESENT (doc: "S-3.13 AC-2") — the
   v1.1 story text incorrectly marked it as net-new. Only the E-QUERY-039 arm is net-new.
   The enrichment gate insertion anchor is `engine.rs` (new pass before `check_availability_gate`),
   not an unspecified file to be determined by the implementer.

6. **Forbidden dependencies are unchanged from the predecessor.** `prism-query` MUST NOT
   depend on `prism-mcp`. The E-QUERY-039 gate lives in `prism-query`; its error type is
   in `prism-core`; the MCP mapping is in `prism-mcp/src/error_mapping.rs`. This layering
   is already established by the predecessor story.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and the predecessor story's lessons.
These rules are binding for this story.

1. **Dependency direction:** `prism-query` must NOT import from `prism-mcp`. Error types
   (`PrismError::EnrichUdfNotFound`, `PrismError::TableNotAvailable`) live in `prism-core`.
   MCP mapping (`map_prism_error`) lives in `prism-mcp/src/error_mapping.rs`. This direction
   MUST be preserved.

2. **`#[non_exhaustive]` discipline and InfusionRegistry API surface:** No new public struct
   or enum field may be added without `#[non_exhaustive]` and a corresponding `ci.yml EXPECTED`
   increment. This story DOES introduce exactly ONE new public type: `EnrichUdfNotFoundDetails`
   (for AC-N1B). It MUST carry `#[non_exhaustive]`. `scripts/check-non-exhaustive.sh EXPECTED`
   increments 87→88. The CLAUDE.md non-exhaustive sentence is updated 87→88 with
   `EnrichUdfNotFoundDetails` in the attribution list. These three changes are REQUIRED —
   not conditional on "if a new type is needed."

   **No new public methods on `InfusionRegistry`** (I1 v1.2): The UDF name set needed for
   `available_infusions` and strsim candidates MUST be derived from the EXISTING public method
   `udf_descriptors()`: `registry.udf_descriptors().iter().map(|d| d.name.clone()).collect()`.
   Do NOT introduce a `udf_names()` accessor or any other new public method — this keeps the
   new-#[non_exhaustive]-type count at exactly ONE (EnrichUdfNotFoundDetails → EXPECTED 87→88,
   not 87→89).

3. **`InfusionRegistry` reload-awareness:** The `build_reference_content` function receives
   `InfusionRegistry` via `Arc<ArcSwap<InfusionRegistry>>` at request time (per BC-2.11.022
   invariant and ADR-042). The N1 fix (change dedup key) must NOT change the reload pattern.
   No caching of the assembled string.

4. **`TableRegistry` is source of truth for table availability.** `TableRegistry` stores
   only underscore-qualified keys (e.g., `"cyberint_alerts"`). The E-QUERY-037 gate consults
   `TableRegistry::is_registered(table_name_as_written)` first. For N2, the gate ordering
   fix ensures this check precedes `sensor_id_from_table_name` dot-extraction — NOT the
   other way around. ADR-046 filter-mode dot-notation is NOT handled via `TableRegistry`
   (filter-mode uses `<table_name> | <predicate>` syntax; the `<table_name>` in filter
   mode uses underscore-qualified names like `crowdstrike_detections`, not dot-syntax).

5. **`sensor_id_from_table_name` dot-notation extraction must NOT be removed.** This function
   was intentionally extended by PR #203 for BC-2.11.023 filter-mode source refs. The N2
   fix is gate ORDERING only: `TableRegistry::is_registered` first, then fan-out. The
   dot-notation extraction stays in the codebase for its legitimate filter-mode use case.

6. **Forbidden dependencies (module perimeter):**
   - `prism-mcp` must NOT depend on `prism-query` internals (parser types); it accesses
     only the `PrismError` type from `prism-core`.
   - `prism-query` must NOT depend on `prism-mcp`.
   - `crates/prism-core/src/error.rs` is the authoritative `PrismError` location.

---

## Library & Framework Requirements

These version pins are from the `Cargo.lock` at develop HEAD `7e60df03` (authoritative).
The implementer MUST use these exact versions — no drift.

| Crate | Version | Use in this story |
|-------|---------|------------------|
| `strsim` | per Cargo.lock (same version used by E-QUERY-037/038) | `did_you_mean` Levenshtein computation in E-QUERY-039; already in use — no new dep |
| `tokio` | per Cargo.lock | async test harness for timing assertions (AC-REG-2 if needed) |
| `datafusion` | per Cargo.lock | DataFusion plan-time API used in materialization.rs gate; no version change |
| `serde_json` | per Cargo.lock | JSON shape assertions in prism_describe tests |

No new dependencies are introduced by this story. All fixes modify existing code paths using
already-present library calls.

---

## File Structure Requirements

All files modified in the implemented scope (v2.0):

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-core/src/error.rs` | MODIFIED | N1-B: added `PrismError::EnrichUdfNotFound(Box<EnrichUdfNotFoundDetails>)` variant; added `#[non_exhaustive] pub struct EnrichUdfNotFoundDetails { pub infusion, pub available_infusions, pub did_you_mean }` with variant_meta category "validation" |
| `crates/prism-core/src/tests/test_enrich_udf_not_found_display.rs` | CREATED | N1-B: 5 Display format tests for EnrichUdfNotFoundDetails (display_no_did_you_mean, display_with_did_you_mean, display_empty_available, display_starts_with_error_code, f_pbl1_low002_display_self_sorts_available_infusions) |
| `crates/prism-core/src/tests/mod.rs` | MODIFIED | N1-B: register new test module |
| `crates/prism-mcp/src/resources.rs` | MODIFIED | N1: `build_reference_content` dedup key changed from `infusion_id` → `descriptor.name`; Some(empty)/None placeholders |
| `crates/prism-mcp/src/error_mapping.rs` | MODIFIED | N1-B: explicit `-32602` INVALID_PARAMS arm for `PrismError::EnrichUdfNotFound` (E-QUERY-039); sorted+deduped available_infusions; structured variant_meta category "validation" |
| `crates/prism-mcp/src/tools/prism_describe.rs` | MODIFIED | AUDIT-001 + CRIT-1 + AC-CAT2: `build_tables_for_client` emits `format!("{sensor_id}_{}", table.table_name)` on BOTH multi-tenant and single-tenant paths; new `pub fn build_example_query(table_name, columns)` derives datetime column from spec (no hardcoded 'timestamp'); `build_pql_hints` gains 4th param `infusion_registry: Option<&prism_spec_engine::InfusionRegistry>` (Cat2 hint); `handle_prism_describe` wired with `query_engine.and_then(|qe| qe.infusion_registry()).as_deref()`; inline test module |
| `crates/prism-mcp/src/prompts.rs` | MODIFIED | AUDIT-004 + MED-1: 4 `render_*` functions updated with FROM-ready names; filter VALUES aligned to DTU vocabulary; all-FROM-resolve guard |
| `crates/prism-query/src/engine.rs` | MODIFIED | N1-B + N2 + H1 + M2: new `collect_unknown_scalars_from_sql_query`, `collect_unknown_scalar_from_expr`, `collect_unknown_scalar_from_predicate`, `check_enrich_udf_availability` functions; E-QUERY-039 gate wired in execute_inner AND execute_scheduled_inner (fires LAST); capability-gate moved AFTER enrich gate in execute_scheduled_inner (H1); `check_query_column_availability` signature updated to accept `table_registry` param + validates GROUP BY/ORDER BY/JOIN ON columns (M2) |
| `crates/prism-query/src/table_registry.rs` | MODIFIED | N2 + M1 + L1: `columns_by_table` field + `columns_for_table()` method (M1 single-tenant column gate); `collect_expr_sources_into_gate` fn for subquery source walk at HAVING/GROUP BY/ORDER BY/JOIN ON positions (L1) |
| `crates/prism-mcp/tests/bc_2_11_022_n1_test.rs` | CREATED | N1: `test_bc_2_11_022_n1_per_field_udf_names` |
| `crates/prism-mcp/tests/bc_2_11_019_n1b_mcp_test.rs` | CREATED | N1-B MCP: `test_bc_2_11_019_n1b_mcp_maps_to_32602`, `test_med5_enrich_udf_not_found_suggestion_non_empty_no_brackets`, `test_med5_enrich_udf_not_found_suggestion_empty_infusions` |
| `crates/prism-spec-engine/src/infusion/udf.rs` | MODIFIED | AC-CAT2: `InfusionUdfDescriptor` gains `pub input_field: String`; `new()` gains this param (BC-2.10.012 v1.7 §pql_hints Category-2) |
| `crates/prism-spec-engine/src/infusion/mod.rs` | MODIFIED | AC-CAT2: `udf_descriptors()` propagates `field.input_field.clone()` per descriptor |
| `crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs` (+ ~10 other `InfusionUdfDescriptor::new()` call sites) | MODIFIED | AC-CAT2: TD-VSDD-060 sibling-site sweep — all `new()` callers in prism-query updated to pass `""` for `input_field` |
| `crates/prism-mcp/tests/bc_2_10_012_audit_001_test.rs` | CREATED | AUDIT-001 + AC-CAT2: `test_bc_2_10_012_audit_001_sensor_prefixed_table_names`, `test_bc_2_10_012_audit_001_multi_tenant_sensor_prefixed_unique`, `test_bc_2_10_012_cat2_enrichment_hint_with_udfs`, `test_bc_2_10_012_cat2_enrichment_absent_hint`, `test_bc_2_10_012_cat2_zero_table_no_category2` |
| `crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs` | CREATED | AUDIT-004 + MED-2: 5 tests (no_dot_notation, from_targets_include_registered, column_refs_resolve, column_sets_loaded, med2_prompt_filter_values_match_dtu_vocabulary) |
| `crates/prism-mcp/tests/f_pql2_obs001_skeleton_placeholder_guard_test.rs` | CREATED | F-PQL2-OBS-001 process-gap closure: 2 skeleton-placeholder guard tests (`test_f_pql2_obs001_query_skeleton_no_bare_timestamp` — guards server.rs SCHEMA-AGNOSTIC SKELETONS for bare `timestamp` in query tool description; `test_f_pql2_obs001_datetime_arithmetic_uses_placeholder` — guards `build_reference_content` Datetime Arithmetic section for `<datetime_col>` placeholder vs bare `WHERE timestamp >`). Traces to BC-2.10.016 v1.2. |
| `crates/prism-mcp/tests/reference_content.rs` | MODIFIED | N1: added `test_bc_2_11_022_crit001_positive_examples_runtime_valid` (OBS-4 migration from deleted file), `test_bc_2_11_022_some_empty_registry_placeholder` |
| `crates/prism-mcp/tests/tool_dispatch_tests.rs` | MODIFIED | OBS-5: new fail-closed guard tests; `test_med4_enrich_udf_not_found_structured_category_is_validation` |
| `crates/prism-query/src/tests/bc_2_11_001_n2_test.rs` | CREATED | N2 + HIGH-1: 4 tests (`dot_notation_from_target_e_query_037`, `filter_mode_underscore_no_regression`, `dot_notation_sqlpipe_e_query_037`, `sqlpipe_underscore_no_regression`) |
| `crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs` | CREATED | N1-B + C1/C2 + gate coverage: 22 tests (infusion_id_as_udf_name, sql_path, high001_gate_ordering, high003_sql_select_projection, med001_sort, high1_sqlpipe_head, ec_11_059, c1 unit-level × 3, c1/c2 engine-level × 3, obs2 did_you_mean Some/None, builtin_passthrough_lower, builtin_passthrough_coalesce — v2.3 for EC-11-064/065; ec_11_066_builtin_aggregate_stddev_not_e_query_039, ec_11_067_builtin_window_row_number_not_e_query_039 — v2.17 for EC-11-066/067; f_pnl1_pipe_mode_builtin_aggregate_still_fires_e_query_039 — v2.17 pipe-mode guard; f_pjl1_high001_non_builtin_unknown_still_triggers_e_query_039, f_pjl4_med001_scheduled_path_table_gate_fires_before_capability_gate — added v2.7 for F-PJL1/F-PJL4) |
| `crates/prism-query/src/tests/table_registry_tests.rs` | MODIFIED | M1 + L1 + OBS-1: new tests for `columns_for_table`, availability gate subquery position coverage, OBS-1 SqlPipe JOIN stage, OBS-1 SELECT WHERE IN subquery |
| `crates/prism-query/src/tests/mod.rs` | MODIFIED | Register new test modules |
| `crates/prism-query/src/materialization.rs` | MODIFIED | AC-DISC (F-L2-CRIT-001): `pub(crate) fn seed_armis_entity_discriminator` added; wired into run_materialization_pipeline fan-out loop; inline test module `armis_discriminator_tests` with 4 tests |
| `scripts/check-non-exhaustive.sh` | MODIFIED | AC-REG-1: `EXPECTED=87` → `EXPECTED=88` (EnrichUdfNotFoundDetails) |
| `.github/workflows/ci.yml` | MODIFIED | Minor CI wiring for non-exhaustive gate |
| `CLAUDE.md` | MODIFIED | AC-REG-1: non-exhaustive sentence count 87→88; attribution list updated |

Files DELETED:
| File | Reason |
|------|--------|
| `crates/prism-mcp/tests/crit001_prompt_table_names.rs` | Superseded by AUDIT-004 TOML-derived guard in `bc_2_10_016_audit_004_test.rs` (OBS-4 finding) |

Files NOT modified:
- BC files (PO owns them — this story reads BCs, never modifies them)
- `.factory/STATE.md` (state-manager owns it)
- `crates/prism-query/src/materialization.rs` — see MODIFIED entry above (F-L2-CRIT-001 armis discriminator fix). NOTE: the `resolve_source_refs` path in this file (which returns E-QUERY-036 UnknownSourceTable) is a DIFFERENT code path and remains untouched by this story's gate-ordering work.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `build_reference_content` | `crates/prism-mcp/src/resources.rs` | Pure (takes `Option<&InfusionRegistry>`, returns `String`) |
| `build_tables_for_client` (multi-tenant + single-tenant paths) | `crates/prism-mcp/src/tools/prism_describe.rs` | Pure (takes spec data, returns `Vec<TableDescriptor>`) |
| `build_example_query` (NEW — CRIT-1: derives datetime col from spec) | `crates/prism-mcp/src/tools/prism_describe.rs` | Pure |
| `render_triage_alerts` / `render_client_overview` / `render_cross_client_status` / `render_investigate_host` (4 modified; `render_query_tutorial` unchanged) | `crates/prism-mcp/src/prompts.rs` | Pure (synchronous per BC-2.10.016 invariant) |
| `PrismError::EnrichUdfNotFound` variant + `EnrichUdfNotFoundDetails` struct | `crates/prism-core/src/error.rs` | Pure (data type, no I/O) |
| `collect_unknown_scalars_from_sql_query` (NEW — canonical walk for Sql+SqlPipe) | `crates/prism-query/src/engine.rs` | Pure |
| `collect_unknown_scalar_from_expr` (NEW — recursive Expr walker) | `crates/prism-query/src/engine.rs` | Pure |
| `collect_unknown_scalar_from_predicate` (NEW — Predicate walker, DEFENSIVE) | `crates/prism-query/src/engine.rs` | Pure |
| `check_enrich_udf_availability` (NEW — E-QUERY-039 gate, fires LAST) | `crates/prism-query/src/engine.rs` | Pure (takes `&str` query + `Option<&InfusionRegistry>`, returns `Result<_, PrismError>`) |
| E-QUERY-037 plan-time availability gate — `check_availability_gate` / `is_registered` | `crates/prism-query/src/table_registry.rs` | Pure (gate check, no I/O) |
| `columns_by_table` field + `columns_for_table()` method (NEW — M1 single-tenant column gate) | `crates/prism-query/src/table_registry.rs` | Pure |
| `collect_expr_sources_into_gate` (NEW — L1 subquery source walk for HAVING/GROUP BY/ORDER BY/JOIN ON) | `crates/prism-query/src/table_registry.rs` | Pure |
| Gate ordering (execute_inner + execute_scheduled_inner) | `crates/prism-query/src/engine.rs` | Pure (plan-time orchestration); canonical: E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-011 |
| `map_prism_error` (E-QUERY-039 arm net-new; E-QUERY-037 arm confirmed-present — no change) | `crates/prism-mcp/src/error_mapping.rs` | Pure (mapping function) |
| `seed_armis_entity_discriminator` (NEW — AC-DISC, F-L2-CRIT-001; wired into run_materialization_pipeline fan-out) | `crates/prism-query/src/materialization.rs` | Pure (takes table_name + filters, returns seeded filters) |
| reqwest `[dev-dependencies]` — rustls-tls standardization (AC-TLS) | `crates/prism-dtu-{armis,crowdstrike,claroty,cyberint,slack,pagerduty,jira,nvd,threatintel}/Cargo.toml`, `crates/prism-bin/Cargo.toml`, `crates/ocsf-proto-gen/Cargo.toml` | Effectful (HTTP test client dep config change only; no production code path affected) |
| DTU integration tests — un-quarantined (AC-TLS) | `crates/prism-dtu-armis/tests/`, `crates/prism-dtu-crowdstrike/tests/` | Effectful (integration tests start DTU HTTP server; root cause was native-tls Keychain init latency, fixed by rustls-tls) |
| `stop()` resource-cleanup (AC-TLS) | `crates/prism-dtu-claroty/tests/sec_p3_003_constant_time_admin_token.rs` | Effectful (test teardown — ensures DTU clone shuts down before next test run) |

---

## UX References

N/A — this is a server-side correctness story with no UI component. The demo-recorder will
capture evidence of the fixed MCP tool output and prompt rendering as per AC-DEMO-001.

---

## Dependencies

| Type | Story | Reason |
|------|-------|--------|
| `depends_on` | S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | build_reference_content, materialization.rs gate structure, and prompts.rs prompt dispatch — all introduced by PR #203 — are the code surfaces being modified. Dependency anchor: these code surfaces must exist (merged) before this fix story can be written and tested against them. |
| `blocks` | (none) | No subsequent story depends on these fixes; they unblock the T13 demo recording (human-scheduled milestone, not a story dependency). |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | N1: `InfusionRegistry` has zero infusions registered at reference request time | `build_reference_content(Some(&registry))` returns enrichment section with "No enrichment functions are currently registered for your deployment." (not six UDF entries). Test: `test_bc_2_11_022_some_empty_registry_placeholder` covers the Some(empty) path. |
| EC-002 | N1-B: Calling a name that IS registered as a per-field UDF name | E-QUERY-039 does NOT fire — gate only fires when name is absent from `udf_to_infusion`. Negative control for the fix. |
| EC-003 | N2: Filter-mode query `crowdstrike_detections | severity='HIGH'` continues to work after gate-ordering fix | Gate fix must NOT alter filter-mode behavior. `Ast::Filter` path uses `<table_name> | <predicate>` syntax, NOT dot-syntax as FROM target. |
| EC-004 | N2: `SELECT * FROM crowdstrike.detections` (SQL mode, dot-notation) returns E-QUERY-037 | All three modes (SQL, Pipe, SqlPipe) covered per EC-11-067 in BC-2.11.001 v1.15. |
| EC-005 | AUDIT-001: `prism_describe(org-c)` for a sensor with a table name containing an underscore (e.g., `audit_logs`) | `format!("{sensor_id}_{}", table.table_name)` → `"claroty_audit_logs"` — correct. No double-underscore since `sensor_id` is a simple identifier. |
| EC-006 | AUDIT-004: `render_query_tutorial` must not be broken | Already compliant (uses `<sensor_table>` placeholder); MUST NOT be modified. |
| EC-007 | N2: SqlPipe head with dot-notation `FROM cyberint.alerts | SELECT *` returns E-QUERY-037 | BC-2.11.001 v1.15 is mode-agnostic; SqlPipe-not-exempt scope. Test: `test_bc_2_11_001_n2_dot_notation_sqlpipe_e_query_037`. |
| EC-008 | N1-B: Wired InfusionRegistry with zero UDF descriptors (EC-11-059) | `check_enrich_udf_availability` fires E-QUERY-039 with `available_infusions: []` (empty Vec) when the registry is wired but has no descriptors. Gate does NOT skip — an empty registry with an enrich call is still an error. Test: `test_ec_11_059_wired_empty_registry_fires_e_query_039_with_empty_available`. |
| EC-009 | N1-B: did_you_mean Some vs None based on Levenshtein distance | When queried name is within distance ≤ 3 of a registered UDF name, `did_you_mean: Some(closest)`. When all registered names are > distance 3 from queried name, `did_you_mean: None`. Tie-break: lexicographic on distance-equal candidates. Tests: `test_obs2_did_you_mean_some_from_strsim_levenshtein_within_threshold`, `test_obs2b_did_you_mean_none_when_beyond_levenshtein_threshold`. |
| EC-010 | N1-B: available_infusions is sorted in E-QUERY-039 error | Registered UDF names appear sorted (ascending) in the `available_infusions` Vec of the error and in the MCP Display string. Test: `test_med001_available_infusions_sorted_in_e_query_039_error`. |
| EC-011 | C1/C2: Unknown scalar in GROUP BY or ORDER BY returns E-QUERY-039 | `check_enrich_udf_availability` catches GROUP BY `badudf(col)` and ORDER BY `badudf(col)` positions via `collect_unknown_scalars_from_sql_query`. Tests: `test_c1_sql_group_by_unknown_scalar_triggers_e_query_039`, `test_c1_sql_order_by_unknown_scalar_triggers_e_query_039`. |
| EC-012 | C2: Unknown scalar in JOIN ON condition returns E-QUERY-039 | `collect_unknown_scalar_from_expr` handles JOIN ON (typed as `Expr` not `Predicate`). Test: `test_c2_collect_unknown_scalar_from_sql_query_join_on`. |
| EC-013 | H1: Query in execute_scheduled_inner with both unknown table and capability violation — first error is E-QUERY-037 (table), not E-QUERY-011 (capability) | Gate ordering symmetric with execute_inner: table gate fires first. This is the canonical first-error ordering. |
| EC-014 | M1: Single-tenant column gate for table with NO columns in spec | `columns_for_table` returns empty `Vec` → column gate skips that table (fail-open). No false E-QUERY-038 for tables without column metadata in the spec. |
| EC-015 | AUDIT-001: No datetime column in table spec → column-free example_query (when also no Integer/Float column) | `build_example_query` produces `SELECT * FROM <t> LIMIT 25` (not `WHERE timestamp > ...`). This is the lowest-priority fallback in the variant ladder: aggregate (Integer/Float) → severity-filter (severity + known vocabulary) → count-recent (Datetime) → column-free (fallback). Test: `test_crit1_no_datetime_column_produces_column_free_query`. |
| EC-016 | N1-B (BC-2.11.019 v1.6 EC-11-064): `SELECT lower(hostname) FROM crowdstrike_detections` with infusion registry wired but `lower` not registered as an infusion | E-QUERY-039 does NOT fire. `lower` is a DataFusion built-in scalar resolved via `ctx.state().scalar_functions()`; it satisfies built-in exclusion condition (b). Query proceeds to DataFusion execution. Test: `test_bc_2_11_019_n1b_builtin_passthrough_lower`. |
| EC-017 | N1-B (BC-2.11.019 v1.6 EC-11-065): `SELECT upper(device_name), coalesce(severity, 'unknown') FROM armis_devices` with infusion registry wired | E-QUERY-039 does NOT fire for `upper` or `coalesce` — both are DataFusion built-in scalars excluded from the gate. Query proceeds normally. Test: `test_bc_2_11_019_n1b_builtin_passthrough_coalesce`. |
| EC-018 | TLS-REMEDIATION: macOS native-tls/Security.framework Keychain init overhead (~65s) under nextest full-suite parallel load | Resolved by rustls-tls standardization (commit cf66151f). With rustls-tls, init is ~0ms → ~800x margin over the 50s stage-0 window. DTU stage-0 tests no longer time out deterministically. This was NOT flakiness — it was a deterministic failure masked as a random one because parallel test scheduling varied which binary crossed the threshold. |
| EC-019 | TLS-REMEDIATION: production reqwest deps in prism-bin/prism-spec-engine/prism-sensors were ALREADY rustls-tls before this fix | Production code path is unaffected. Only `[dev-dependencies]` (test binaries) and the optional build-tool dep in ocsf-proto-gen changed. Security posture unchanged — confirmed by security review APPROVE on commit cf66151f. |
| EC-020 | N1-B (BC-2.11.019 v1.6 EC-11-066): `SELECT stddev(latency) FROM crowdstrike_detections` with infusion registry wired but `stddev` not registered as an infusion | E-QUERY-039 does NOT fire. `stddev` is a DataFusion built-in **aggregate** function resolved via `ctx.state().aggregate_functions()`; it satisfies the built-in exclusion condition (b). The same applies to `median`, `variance`, `array_agg`, and other DataFusion aggregate built-ins. Query proceeds to DataFusion execution. SQL-mode exclusion only — pipe-mode `| enrich stddev(col)` still fires E-QUERY-039 if `stddev` is not a registered infusion. Test: `test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039`. |
| EC-021 | N1-B (BC-2.11.019 v1.6 EC-11-067): SQL-mode query with `row_number()` appearing as `ScalarFunc::Unknown` with infusion registry wired but `row_number` not registered as an infusion | E-QUERY-039 does NOT fire. `row_number` is a DataFusion built-in **window** function resolved via `ctx.state().window_functions()`; it satisfies the built-in exclusion condition (b). The same applies to `rank`, `dense_rank`, `lead`, `lag`, and other DataFusion window built-ins. SQL-mode exclusion only — pipe-mode still fires E-QUERY-039 for unregistered names. Test: `test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039`. |

---

## Estimated Complexity

**11 story points** (v2.15 — +1 pt for TLS-REMEDIATION fold-in; prior total was 10).

Root causes are all confirmed, code paths are known, and BCs are in place. The implementation
spans the original 5 findings, 6 gate-coverage fixes found during LOCAL adversarial passes,
and the TLS-REMEDIATION fold-in (commit cf66151f):
- N1: one-line dedup key change in `build_reference_content` + test
- N1-B: net-new error type (error.rs) + `collect_unknown_scalars_from_sql_query` + `check_enrich_udf_availability` in engine.rs covering ALL AST positions (SELECT, WHERE, JOIN ON, GROUP BY, ORDER BY, HAVING) for both Sql and SqlPipe; map_prism_error -32602 arm; sorted+deduped available_infusions; strsim did_you_mean; DataFusion built-in exclusion (BC-2.11.019 v1.6 F-PJL1-HIGH-001: scalar+aggregate+window via scalar_functions(), aggregate_functions(), window_functions()); + 21 new tests (15 original + 2 builtin_passthrough scalar EC-11-064/065 + 2 F-PJL mid-cascade F-PJL1/F-PJL4 + 2 builtin_passthrough aggregate/window EC-11-066/067)
- N2: gate ordering fix in `check_availability_gate` (table_registry.rs) with SqlPipe-not-exempt scope; 4 new N2 tests
- AUDIT-001: sensor-prefixed names on both tenant code paths; `build_example_query` datetime column derivation from spec (CRIT-1 fix); 3 new tests
- AUDIT-004: FROM-ready names in 4 `render_*` functions (not 5 — `render_query_tutorial` was clean); prompt VALUES aligned to DTU vocabulary (MED-1); `all-FROM-resolve` guard; 5 new tests
- Gate-coverage fixes: C1/C2 (enrich gate JOIN/GROUP/ORDER), M1 (single-tenant column gate via `columns_for_table`), M2 (E-QUERY-038 validates GROUP BY/ORDER BY/JOIN ON columns), L1 (E-QUERY-037 source walk covers all subquery positions), H1 (capability-gate ordering symmetric in execute_scheduled_inner)
- Deleted: `crates/prism-mcp/tests/crit001_prompt_table_names.rs` (superseded by AUDIT-004 TOML-derived guard, OBS-4)
- TLS-REMEDIATION (v2.15 fold-in): standardize 11 reqwest Cargo.toml entries to rustls-tls; remove 4 `#[ignore]` attributes from DTU stage-0 integration tests; add 7 stop() resource-cleanup calls in prism-dtu-claroty. Root cause was macOS native-tls/Security.framework Keychain init (~65s per test process) exceeding the 50s stage-0 window under parallel nextest load — corrected root cause replaces prior misdiagnosis of "WASMtime plugin-init starvation".

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.42 | ADV-PR-P3-LOW-001-semantic-currency-2026-07-09 | 2026-07-09 | story-writer | **semantic-currency completion for BC-2.11.016 v1.23 _with_bareness extractor rename (ADV-PR-P3-LOW-001). Two live present-tense mechanism description sites updated — (1) frontmatter implementation comment (~line 344): "extract_field_paths_from_expr helper (the SINGLE extraction point for all 5 positions)" → "extract_field_paths_with_bareness (positions 1/3/4/5) and extract_predicate_columns_with_bareness (positions 2/6; per-reference (name, is_bare) pairs for HEAD-JOIN SUSPENSION gate)"; (2) AC-M2 §HAVING body prose (~lines 866–876): "Position 6 (HAVING): uses `extract_predicate_columns`...recursing via `extract_field_paths_from_expr`..." → "uses `extract_predicate_columns_with_bareness`...recursing via `extract_field_paths_with_bareness` to collect all `(name, is_bare)` column reference pairs"; "For the other positions (1–5), `extract_field_paths_from_expr` is the single extraction helper..." → "For positions 1/3/4/5, `extract_field_paths_with_bareness` is the extraction helper...enabling the HEAD-JOIN SUSPENSION gate to distinguish bare from qualified column references". Two historical blockquotes preserved per POL-29: "BC-2.11.016 v1.5 HAVING addition (F-PWL1-LOW-001)" and "ADR-048 HAVING agg-fn grammar extension (F-PXL3-MED-002)" — origin notes describing mechanism at introduction time, not live currency prose. Post-edit grep confirms zero remaining present-tense extract_field_paths_from_expr/extract_predicate_columns claims about positions 1–6 outside historical blockquotes. ADR-048 WHERE-vs-HAVING grammar point UNCHANGED (WHERE count(col) > 5 remains E-QUERY-001 parse error). BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.41→2.42; updated 2026-07-09 (POL-23).** |
| 2.41 | ADV-PR-P3-LOW-001-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.22→v1.23 (_with_bareness extractor rename, ADV-PR-P3-LOW-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 PR-LEVEL pass-3 story pin round). BC-2.11.016 v1.22→v1.23: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column — v1.23 renames internal extractors for positions 1–6 to _with_bareness variants and positions 10–14 to extract_column_name_from_field_path; HAVING position-6 behavior unchanged; AC-M2 §HAVING prose mentions extract_predicate_columns and extract_field_paths_from_expr as implementation context — these are in AC body prose, not a Key Clauses or task cell; L22 scope excludes AC prose per task instructions; prose preserved as-is. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.40→2.41; updated 2026-07-09 (POL-23).** |
| 2.40 | ADV-PR-P1-MED-001-OBS-001-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.21→v1.22 (Injection-safety of `column` MCP-facing payload, CWE-116, ADV-PR-P1-MED-001/OBS-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 PR-LEVEL pass-1 story pin round). BC-2.11.016 v1.21→v1.22: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column — v1.22 adds injection-safety of `column` (MCP-facing E-QUERY-038 payload CWE-116 sanitization); HAVING position-6 behavior unchanged; AC-M1/AC-M2 trace to gate-firing postcondition (not payload content postcondition); no semantic-cell extension needed. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.39→2.40; updated 2026-07-09 (POL-23).** |
| 2.39 | ADV-FIX-P16-MED-001-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.20→v1.21 (PER-REFERENCE SCOPING, ADV-FIX-P16-MED-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-16 story pin round). BC-2.11.016 v1.20→v1.21: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column — PER-REFERENCE SCOPING is a HEAD-JOIN per-reference precision clarification (suspension applies per individual column reference, not per column name; qualified refs retain full E-QUERY-038 checking; EC-11-076 added); HAVING position-6 behavior in joinless queries UNCHANGED; AC-M2 tests do not use JOIN-containing queries, so test semantics UNCHANGED. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.38→2.39; updated 2026-07-09 (POL-23).** |
| 2.38 | ADV-FIX-P15-MED-001-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.19→v1.20 (HEAD-JOIN SUSPENSION RULE, ADV-FIX-P15-MED-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-15 story pin round). BC-2.11.016 v1.19→v1.20: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column (only BC ID / Version / Title) — HEAD-JOIN SUSPENSION RULE is a new SQL-head positions-1–6 rule (when head query JOIN list non-empty AND bare unqualified column ref absent → E-QUERY-038 MUST NOT fire); HAVING position-6 behavior in joinless queries unchanged; AC-M2 tests do not use JOIN-containing queries, so test semantics UNCHANGED. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.37→2.38; updated 2026-07-09 (POL-23).** |
| 2.37 | FIX-IEQ-ERRPATH-001-pass-14-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.19 / BC-2.11.017 v1.7 / BC-2.11.020 v1.12 / BC-2.11.004 v1.24 / error-taxonomy v2.32 (STAGE-JOIN SUSPENSION RULE, ADV-FIX-P14-OBS-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-14 story pin round). BC-2.11.016 v1.18→v1.19: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column (only BC ID / Version / Title) — STAGE-JOIN SUSPENSION RULE is a new stage-walk binding clause (PipeStage::Join → suspended:=true for remainder of walk); HAVING position-6 behavior unchanged; no semantic-cell extension needed. Historical changelog rows left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.36→2.37; updated 2026-07-09 (POL-23).** |
| 2.36 | FIX-IEQ-ERRPATH-001-pass-12-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.18 / BC-2.11.017 v1.6 / BC-2.11.020 v1.11 / BC-2.11.004 v1.23 / error-taxonomy v2.31 (STAR-WITH-JOIN SUSPENSION RULE, ADV-FIX-P12-OBS-002) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-12 story pin round). BC-2.11.016 v1.17→v1.18: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column (only BC ID / Version / Title) — STAR-WITH-JOIN SUSPENSION RULE is a new SqlPipe head-projection binding clause; HAVING position-6 behavior unchanged; no semantic-cell extension needed. Historical changelog rows left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.35→2.36; updated 2026-07-09 (POL-23).** |
| 2.35 | FIX-IEQ-ERRPATH-001-pass-10-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.17 / BC-2.11.017 v1.5 / BC-2.11.020 v1.10 / BC-2.11.004 v1.22 / error-taxonomy v2.30 (LAST-SEGMENT rule, ADV-FIX-P10-OBS-001) + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-10 story pin round). BC-2.11.016 v1.16→v1.17: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: FIDELITY BC table has no Key Clauses column (only BC ID / Version / Title) — LAST-SEGMENT OUTPUT-NAME RULE does not affect HAVING position-6 behavior; no semantic-cell extension needed. Historical changelog rows left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.34→2.35; updated 2026-07-09 (POL-23).** |
| 2.34 | FIX-IEQ-ERRPATH-001-pass-8-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync BC-2.11.016 v1.16 (narrative-only E-QUERY-039 note fix, ADV-FIX-P8-OBS-002). BC-2.11.016 v1.15→v1.16: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: AC-M1 and AC-M2 HAVING postcondition references verified CURRENT — v1.16 is narrative-only (Enrich bullet E-QUERY-039 note corrected to actual gate ordering); no rule/EC/behavior change affects HAVING position-6 semantics. "unreachable-by-design" grep across story: zero hits — no semantic-cell edits required. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.33→2.34; updated 2026-07-09 (POL-23).** |
| 2.33 | FIX-IEQ-ERRPATH-001-pass-7-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync to BC-2.11.016 v1.15 / BC-2.11.017 v1.4 / BC-2.11.020 v1.9 / BC-2.11.004 v1.21 / error-taxonomy v2.29 + semantic-cell currency (FIX-IEQ-ERRPATH-001 pass-7 fix round). BC-2.11.016 v1.14→v1.15: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Semantic-cell currency: AC-M1 and AC-M2 HAVING postcondition references verified CURRENT (v1.15 adds SIBLING-GATE CONSISTENCY / FROM-ALIAS RESOLUTION / FIELDS TRANSITION — none affect HAVING position-6 behavior). Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.017, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.32→2.33; updated 2026-07-09 (POL-23).** |
| 2.32 | ADV-FIX-P6-MED-001-OBS-001-pin-sync-2026-07-09 | 2026-07-09 | story-writer | **pin-sync to BC-2.11.016 v1.14 / BC-2.11.004 v1.20 / BC-2.11.020 v1.8 / error-taxonomy v2.28 + Key Clauses semantic refresh (closes ADV-FIX-P6-MED-001/OBS-001). BC-2.11.016 v1.13→v1.14: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. TASK 2 OBS-001 sweep: AC-M1 and AC-M2 semantic content verified CURRENT (HAVING position-6 behavior unchanged in v1.14; MIXED-STAR branch is a new SqlPipe branch, not a HAVING clause change). Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.31→2.32; updated 2026-07-09 (POL-23).** |
| 2.31 | pin-sync-FIX-IEQ-ERRPATH-001-D-1615-2026-07-09 | 2026-07-09 | story-writer | **pin-sync to BC-2.11.016 v1.13 / BC-2.11.004 v1.19 / BC-2.11.020 v1.7 / error-taxonomy v2.27 (FIX-IEQ-ERRPATH-001 D-1615 frozen pin round). BC-2.11.016 v1.12→v1.13: six live sites updated — (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + prose `BC-2.11.016 v1.5's claim` left unchanged per POL-29. BC-2.11.004, BC-2.11.020, and error-taxonomy not cited in this story. AC semantics UNCHANGED. Frontmatter version 2.30→2.31; updated 2026-07-09 (POL-23).** |
| 2.30 | BC-2.11.016-v1.12-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **Reconciling pin round (pass-4 closures): BC-2.11.016 v1.11→v1.12. Six live version-pin cites updated: (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + lines 1123/1125 left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.29→2.30; updated 2026-07-08 (POL-23).** |
| 2.29 | BC-2.11.016-v1.11-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **Reconciling pin round (pass-3 closures): BC-2.11.016 v1.10→v1.11 (pass-3 CRIT closure burst). Six live version-pin cites updated: (1) frontmatter `# BC status:` comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Historical changelog rows + narrative blockquote `BC-2.11.016 v1.5 HAVING addition` + lines 1123/1125 left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.28→2.29; updated 2026-07-08 (POL-23).** |
| 2.28 | BC-2.11.016-v1.10-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.9→v1.10 version-pin propagation (suspend-clause clarification micro-amendment, POL-29/POL-23).** PO bumped BC-2.11.016 v1.9→v1.10 (suspend-clause clarification). Six live version-pin cites updated: (1) frontmatter `# BC status:` comment v1.9→v1.10; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label) v1.9→v1.10; (3) §Behavioral Contracts body table BC-2.11.016 version cell v1.9→v1.10; (4) AC-M1 trace v1.9→v1.10; (5) AC-M2 trace v1.9→v1.10; (6) AC-M2 §HAVING body prose v1.9→v1.10. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.27→2.28; updated 2026-07-08 (POL-23). |
| 2.27 | BC-2.11.016-v1.9-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.8→v1.9 version-pin propagation (sort-grammar fix round, POL-29/POL-23).** PO bumped BC-2.11.016 v1.8→v1.9 (sort-grammar micro-fix). Six live version-pin cites updated: (1) frontmatter BC status comment; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label); (3) §Behavioral Contracts body table BC-2.11.016 version cell; (4) AC-M1 trace; (5) AC-M2 trace; (6) AC-M2 §HAVING body prose. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. Frontmatter version 2.26→2.27; updated 2026-07-08 (POL-23). |
| 2.26 | BC-2.11.016-v1.8-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.7→v1.8 version-pin propagation (pass-2 CRIT closure burst, POL-29/POL-23).** PO bumped BC-2.11.016 v1.7→v1.8 (14-position gate + derived-column binding rule). Six live version-pin cites updated: (1) frontmatter `# BC status:` comment v1.7→v1.8; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label) v1.7→v1.8; (3) §Behavioral Contracts body table BC-2.11.016 version cell v1.7→v1.8; (4) AC-M1 trace v1.7→v1.8; (5) AC-M2 trace v1.7→v1.8; (6) AC-M2 §HAVING body prose v1.7→v1.8. Historical-narrative sites left unchanged per POL-29: blockquote `BC-2.11.016 v1.5 HAVING addition`; lines 1123/1125 `BC-2.11.016 v1.5's claim`; §Changelog rows 2.25 and earlier. AC semantics UNCHANGED. Frontmatter version 2.25→2.26; updated 2026-07-08 (POL-23). |
| 2.25 | BC-2.11.016-v1.7-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.6→v1.7 version-pin propagation (POL-29/POL-23).** Product-owner keyword sweep bumped BC-2.11.016 v1.6→v1.7 (`\| project`→`\| fields` keyword). Six live version-pin cites updated: (1) frontmatter `# BC status:` comment v1.6→v1.7; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label) v1.6→v1.7; (3) §Behavioral Contracts body table BC-2.11.016 version cell v1.6→v1.7; (4) AC-M1 trace v1.6→v1.7; (5) AC-M2 trace v1.6→v1.7; (6) AC-M2 §HAVING body prose v1.6→v1.7. Historical-narrative sites left unchanged per POL-29: blockquote `BC-2.11.016 v1.5 HAVING addition`; lines 1123/1125 `BC-2.11.016 v1.5's claim`; §Changelog rows 2.24 and earlier. AC semantics UNCHANGED. Frontmatter version 2.24→2.25; updated 2026-07-08 (POL-23). |
| 2.24 | ADV-FIX-P1-HIGH-001-BC-2.11.016-v1.6-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.5→v1.6 version-pin propagation (ADV-FIX-P1-HIGH-001, POL-29/POL-23).** Product-owner amended BC-2.11.016 v1.5→v1.6 expanding E-QUERY-038 gate from six SQL positions to twelve positions (Filter/Pipe/SqlPipe predicate + sort/stats/project positions). Six live version-pin cites updated: (1) frontmatter `# BC status:` comment v1.5→v1.6; (2) frontmatter red-gate test inventory comment (AC-M2 HAVING label) v1.5→v1.6; (3) §Behavioral Contracts body table BC-2.11.016 version cell v1.5→v1.6; (4) AC-M1 trace `traces to BC-2.11.016 v1.5 postcondition` → v1.6; (5) AC-M2 trace `traces to BC-2.11.016 v1.5 postcondition` → v1.6; (6) AC-M2 §HAVING body prose `BC-2.11.016 v1.5 §Implementation location` → v1.6. Historical-narrative sites left unchanged per POL-29 rule: (a) blockquote `BC-2.11.016 v1.5 HAVING addition (F-PWL1-LOW-001): HAVING is the 6th column-gate position, added at v1.5...` — this documents the historical v1.5 amendment and changing it to v1.6 would be factually wrong; (b) lines 1123/1125 `BC-2.11.016 v1.5's claim that... BC-2.11.016 stays at v1.5` — historical narrative accurate at time of authorship; (c) §Changelog rows 2.22/2.9/2.8 — immutable changelog rows. AC semantics UNCHANGED. Frontmatter version 2.23→2.24; updated 2026-07-08 (POL-23). |
| 2.23 | exhaustive-count-annotation-reconciliation-2026-07-03 | 2026-07-03 | story-writer | **Exhaustive count/annotation reconciliation sweep — zeroes stale-count defect class.** Ran complete grep sweep across all count claims (red_gate_tests, acceptance_criteria_count, points, estimated_days, per-test-file counts, test-file totals, Token Budget rows). Built a 17-row reconciliation table; found 3 stale live-current count claims. Fixes applied: (1) **R-02 (PRIMARY DEFECT — STALE LEAD-IN)** `red_gate_tests:` comment lead-in at line ~125: `# 49 Red Gate tests (v2.12 fold-in — adds three armis discriminator wiring-seam tests / # from F-LENS4-MED-001 fix in materialization.rs; see arithmetic below):` → `# 54 Red Gate tests total (42 base + fold-ins through v2.17; see arithmetic block at end):`. The two-line v2.12-framed comment implied 49 as current; the field value is 54. The arithmetic block at end of the comment (42→46→49→52→54) is CORRECT and UNCHANGED. (2) **R-15 (Token Budget self-ref)** `This story spec (v2.22)` → `(v2.23)` (version bump). (3) **R-17 (Token Budget test-file count)** `54 Red Gate tests across 9 new test files` → `8 new test files` — source-verified: File Structure CREATED rows = 8 (test_enrich_udf_not_found_display.rs, bc_2_11_022_n1_test.rs, bc_2_11_019_n1b_mcp_test.rs, bc_2_10_012_audit_001_test.rs, bc_2_10_016_audit_004_test.rs, f_pql2_obs001_skeleton_placeholder_guard_test.rs, bc_2_11_001_n2_test.rs, bc_2_11_019_n1b_test.rs); worktree-verified by directory listing. All other 14 count rows verified CORRECT: acceptance_criteria_count 17 ✓, points 11 ✓, estimated_days 2.5 ✓, 5-test display file ✓, 22-test N1B file ✓, 4-test N2 file ✓, 4-test armis discriminator module ✓, 7 BCs ✓, arithmetic 42→46→49→52→54 ✓, all dated-historical annotations correctly dated. ADR-050 citation verified present. No code, BC, or STORY-INDEX change. |
| 2.22 | version-pin-reconciliation-2026-07-03 | 2026-07-03 | story-writer | **Exhaustive version-pin reconciliation sweep.** Ran full BC/ADR version audit across all 1580 lines of the story. Found exactly **one** stale live-prose pin: **line 32 (frontmatter subsystem anchor comment, SS-10 entry)** cited `BC-2.10.012 v1.5` — canonical is `v1.7`. Fixed to `v1.7`. All other version citations verified current against BC frontmatter: BC-2.11.001 v1.15 ✓, BC-2.11.022 v1.1 ✓, BC-2.11.019 v1.6 ✓, BC-2.10.016 v1.2 ✓, BC-2.10.012 v1.7 (all other sites) ✓, BC-2.11.016 v1.5 ✓, BC-2.11.007 v1.9 ✓, BC-2.06.019 v1.15 ✓. BC-2.16.002 v1.91 citations at lines 1112 and 1119 are historical narrative accurately describing when this story's SAP-1 obligation was fulfilled; v1.92 is a subsequent S-PERF-GATE-008 amendment correctly noted as unrelated — left unchanged. Token Budget story-spec row updated v2.20→v2.22. |
| 2.21 | f-p208-adr050-anchor-2026-07-03 | 2026-07-03 | story-writer | **F-P208-ADR050-ANCHOR (MED) — ADR-050 anchor added to AC-TLS + §References; stale conditional blockquote replaced.** (1) **Stale blockquote replaced (AC-TLS, ~line 1038):** The six-line blockquote ending "if the architect or PO determines that a standing ADR is warranted... that authorship is routed to architect/product-owner — NOT authored here" was FALSE — ADR-050 (ACCEPTED) was authored specifically for this story. Replaced with accurate text: "The workspace-wide rustls-tls-only convention for reqwest dev-dependencies IS codified as ADR-050 (workspace reqwest rustls-tls backend convention, ACCEPTED), established during this story." (2) **§References extended:** Added ADR-050 citation paragraph after ADR-048 paragraph: "ADR-050 (workspace reqwest rustls-tls backend convention, ACCEPTED) governs the rustls-tls-only standardization delivered by the TLS-REMEDIATION scope of this story. ADR-050 was authored by the architect for this story (`anchor_stories: [S-DEMO-FIDELITY-REMEDIATION-001]`); it codifies the `default-features=false, features=["rustls-tls"]` convention for all workspace reqwest entries." (3) **Sibling sweep:** grep for "standing ADR\|route to architect\|not authored here\|pending architect\|ADR.*warranted\|warranted for\|authorship is routed" — two instances found. Instance 1: the stale blockquote at line 1038 (FIXED). Instance 2: changelog row 2.15 historical record ("ADR/BC recommendation for rustls-tls convention flagged to orchestrator for routing to architect") — this is an immutable historical record of what was believed at that time; it accurately records past state and is NOT stale prose in active AC scope. Left unchanged. No code, BC, or STORY-INDEX change. |
| 2.20 | consistency-audit-5-findings-2026-07-03 | 2026-07-03 | story-writer | **Consistency-audit 5-finding cleanup.** Source-verified `bc_2_11_019_n1b_test.rs` contains 22 test functions (not 19). Five fixes applied: (1) **F1 (MED)** File Structure table row for `bc_2_11_019_n1b_test.rs`: count 19→22; added 3 missing tests to description: `ec_11_066_builtin_aggregate_stddev_not_e_query_039`, `ec_11_067_builtin_window_row_number_not_e_query_039` (v2.17 EC-11-066/067), `f_pnl1_pipe_mode_builtin_aggregate_still_fires_e_query_039` (v2.17 pipe-mode guard). (2) **F2 (LOW)** `crates_touched` frontmatter comment: `(19 tests — see` → `(22 tests — see`. (3) **F3 (LOW)** Token Budget row: `52 Red Gate tests` → `54 Red Gate tests` (aligns with `red_gate_tests: 54` frontmatter). (4) **F4 (LOW)** Token Budget story-spec row: `(v2.15)` → `(v2.20)`. (5) **F5 (LOW)** AC-SAP-1 §References: volatile `BC-2.16.002 v1.91` pin replaced with TD-VSDD-091 preferred §-anchor form `BC-2.16.002 §Canonical Structured Event Catalog (this story's SAP-1 M1/N2 obligation was fulfilled by the v1.91 amendment; subsequent v1.92 amendment by S-PERF-GATE-008 is unrelated to this story)`. Immutable historical narrative (line ~1112, v1.90→v1.91) left unchanged. No additional stale self-refs found in sweep. No code, BC, or STORY-INDEX change. |
| 2.19 | f-p208-tls-anchor-001-2026-07-03 | 2026-07-03 | story-writer | **F-P208-TLS-ANCHOR-001 (LOW) — AC-TLS unfilled version placeholder + S-7.01 compliance-note reword.** (1) `v1.?` placeholder resolved to `v1.15` (BC-2.06.019 confirmed at v1.15 from frontmatter). (2) AC-TLS header reworded from `(traces to BC-2.06.019 v1.? precondition — DTU stage-0 scenarios execute within the stage-0 window under parallel test load)` to `(TLS-REMEDIATION compliance — DTU stage-0 test un-quarantine; partial BC-2.06.019 v1.15 context)`, mirroring AC-SAP-1's compliance-note pattern (SAP-1 references BC-2.16.002 as "partial dependency" without frontmatter inclusion; same treatment applied here). BC-2.06.019 is NOT added to frontmatter array — the existing blockquote already states "BC-2.06.019 behavior is unchanged — the tests simply RUN now rather than being quarantined"; this is compliance/context reference, not a primary behavioral anchor. (3) Sibling sweep (`rg -n 'v[0-9]+\.\?\|v\?\.' .factory/stories/S-DEMO-FIDELITY-REMEDIATION-001-*`): only one instance found and fixed; no other unfilled version placeholders present. No BC, frontmatter array, body BC table, or STORY-INDEX change. |
| 2.18 | f-p208-n1b-testname-drift-2026-07-03 | 2026-07-03 | story-writer | **F-P208-N1B-TESTNAME-DRIFT (MED) — test-name citation correction.** The two EC-11-066/067 Red Gate test names cited in v2.17 did not match the actual implemented test function names. Story cited `test_bc_2_11_019_n1b_builtin_passthrough_stddev` and `test_bc_2_11_019_n1b_builtin_passthrough_row_number`; source-verified actual names in `bc_2_11_019_n1b_test.rs` (lines 1294, 1338) are `test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039` and `test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039`. Behavior IS covered (real tests exist and pass); this is a citation/traceability defect only. Six citation sites corrected: (1) frontmatter `red_gate_tests` inventory comments (lines 145–146); (2) DataFusion built-in exclusion blockquote EC-11-066/067 note; (3) Red Gate test paragraph for EC-11-066 (stddev); (4) Red Gate test paragraph for EC-11-067 (row_number); (5) EC-020 edge-case table Test cell; (6) EC-021 edge-case table Test cell; (7) v2.17 changelog row retrospective cite. Adjudication per CLAUDE.md SoT rule 7: test-name spelling is an implementation detail; story is updated to match the verified-passing code. No code, BC, or STORY-INDEX change. |
| 2.17 | bc-2.11.019-v1.6-propagation-2026-07-02 | 2026-07-02 | story-writer | **BC-2.11.019 v1.5→v1.6 cite propagation + F1 aggregate/window DataFusion built-in exclusion expansion.** PO amended BC-2.11.019 v1.5→v1.6 (F1 HIGH fix: E-QUERY-039 enrich gate's DataFusion built-in exclusion corrected from SCALAR-ONLY to SCALAR + AGGREGATE + WINDOW — DataFusion built-in aggregate/window functions that parse as `ScalarFunc::Unknown` in SQL-mode must pass the gate via `aggregate_functions()` and `window_functions()` registries in addition to `scalar_functions()`; pipe-mode `\| enrich <name>(...)` still fires E-QUERY-039 for unregistered names regardless of DataFusion registries). **Version cite sweep (POLICY 23):** all live (non-changelog) `BC-2.11.019 v1.5` cites updated to `v1.6`. **Functional story changes:** (1) Step 2 SQL path bullet: exclusion check expanded to all three registries with aggregate/window examples (`stddev`, `median`, `array_agg`, `row_number`, `rank`); (2) DataFusion built-in exclusion note blockquote broadened from scalar-only to scalar+aggregate+window, pipe-mode-still-fires distinction made explicit, EC-11-066/067 references added; (3) Implementation requirement expanded to cover all three registry APIs including `default_aggregate_functions()` and `default_window_functions()`; (4) New edge cases EC-020 (EC-11-066: aggregate built-in `stddev`/`median` passes gate) and EC-021 (EC-11-067: window built-in `row_number`/`rank` passes gate); (5) Two new Red Gate test paragraphs: `test_bc_2_11_019_ec_11_066_builtin_aggregate_stddev_not_e_query_039` and `test_bc_2_11_019_ec_11_067_builtin_window_row_number_not_e_query_039`; (6) `red_gate_tests` 52→54; (7) Frontmatter inventory: 2 new tests added under built-in passthrough section. |
| 2.16 | obs-2-mechanism-description-reconcile-2026-07-02 | 2026-07-02 | story-writer | **OBS-2 mechanism-description reconciliation.** Story over-specified the E-QUERY-039 enrichment gate as using the AST `visit::Visitor` trait. The implementation (engine.rs:1625-1676) deliberately uses a direct `match &ast { ... }` traversal instead, with documented rationale: "avoids coupling with the full visitor infrastructure." Observable behavior is identical and BC-2.11.019 is unaffected — only the story's prose was wrong. Three sites updated: (1) **Frontmatter subsystem anchor comment (SS-11 N1-B line):** `(AST visitor, pipe EnrichStage +` → `(direct match, pipe EnrichStage +`. (2) **Step 2 body (~lines 486-492):** opening sentence `"This pass uses the AST \`visit::Visitor\` to collect"` → `"This pass uses a direct \`match &ast { ... }\` traversal (not the \`visit::Visitor\` trait — avoids coupling with the full visitor infrastructure) to collect"`; pipe/SQL path bullets `"visitor arm"` → `"match arm"`; `"DISTINCT visitor arms but feed"` → `"DISTINCT match arms but feed"`. (3) **Tasks step 6 (~lines 1141-1148):** `"use AST \`visit::Visitor\` to collect"` → `"collect via direct \`match &ast { ... }\` (not the \`visit::Visitor\` trait — avoids coupling with the full visitor infrastructure)"`; `"DISTINCT visitor arms"` → `"DISTINCT match arms"`. BC-2.11.019 not touched — behavioral contract is spec-compliant and unaffected. |
| 2.15 | tls-remediation-fold-2026-07-02 | 2026-07-02 | story-writer | **TLS-REMEDIATION fold-in (commit cf66151f) + root cause correction.** (1) **AC-TLS added (Area L):** documents the native-tls → rustls-tls standardization across 11 Cargo.toml entries (9 DTU `[dev-dependencies]`, prism-bin `[dev-dependencies]`, ocsf-proto-gen optional download-feature `[dependencies]`); 4 DTU integration tests un-quarantined (removed `#[ignore]`): `test_BC_2_06_019_armis_primary_device_stage_visibility`, `test_BPRL_P4_02_armis_alerts_stage_guard_primary_device`, `test_F_PIVOT003_R8C_001_search_primary_device_stage_visibility` (prism-dtu-armis), `test_BPRL_P4_02_detections_stage_guard_primary_device` (prism-dtu-crowdstrike); 7 stop() resource-cleanup calls added in prism-dtu-claroty `sec_p3_003_constant_time_admin_token.rs`. (2) **Root cause corrected:** prior misdiagnosis "WASMtime plugin-init starvation" replaced with the REAL root cause: macOS native-tls/Security.framework Keychain init (~65s/process) exceeding the 50s stage-0 window under nextest parallel load — a DETERMINISTIC failure, not flakiness. (3) **Frontmatter updates:** `version` 2.14→2.15; `updated` 2026-06-29→2026-07-02; `subsystems` [SS-10, SS-11]→[SS-01, SS-10, SS-11, SS-22] (SS-01 for 9 DTU crates; SS-22 for prism-bin); `points` 10→11 (+1pt TLS-REMEDIATION); `estimated_days` 2→2.5 (+0.5d); `acceptance_criteria_count` 16→17 (+AC-TLS); `crates_touched` adds 9 DTU crates + prism-bin + ocsf-proto-gen (11 entries). (4) **No BC authorship:** BC-2.06.019 behavior is unchanged — the tests simply run now. ADR/BC recommendation for rustls-tls convention flagged to orchestrator for routing to architect. |
| 2.14 | cat2-ac-adv-p208-p02-fold-2026-06-29 | 2026-06-29 | story-writer | **AC-CAT2 add + ADV-P208-P02-001 close + ADV-P208-P02-002 close.** (1) **AC-CAT2 (BC-2.10.012 v1.7 §pql_hints Category-2):** `build_pql_hints` gains 4th param `infusion_registry: Option<&prism_spec_engine::InfusionRegistry>`; `pql_hints[2]` = enrichment-presence hint when tables non-empty (sorted UDFs as `<name>(<input_field>)`, byte-exact format); absent hint when `None`/empty registry; Category-2 suppressed when tables empty. `InfusionUdfDescriptor` gains `pub input_field: String`; `new()` gains this param; `udf_descriptors()` propagates `field.input_field.clone()`; ~10 prism-query `new()` callers updated (TD-VSDD-060). `handle_prism_describe` wired via `query_engine.and_then(|qe| qe.infusion_registry()).as_deref()` (ADR-022 §C). 3 new Red Gate tests in `bc_2_10_012_audit_001_test.rs`. `red_gate_tests` 49→52. `prism-spec-engine` added to `crates_touched`. (2) **ADV-P208-P02-001 close (MED, Category-1):** Deferred-items table row 1 ("BC-2.10.012 §pql_hints Category-1 hint-text divergence — PO adjudication required") removed. Resolved spec-only by PO via BC-2.10.012 v1.6→v1.7. Category-2 implemented in-scope; row is no longer deferred. Row 2 (S-QUERY-GATE-REPARSE-CONSOLIDATION-001) unchanged. (3) **ADV-P208-P02-002 close (LOW, AC count drift):** `acceptance_criteria_count` 17→16 (honest body count: 15 pre-v2.14 discrete `**AC-XXX**` headers + 1 new AC-CAT2 = 16). CRIT-1 is a folded sub-behavior within AC-AUDIT-001, not a standalone `**AC-CRIT1**` header; SqlPipe/did_you_mean are folded into AC-N1B/AC-N2/AC-C1C2. Frontmatter count comment rewritten with explicit enumeration. (4) **BC-2.10.012 v1.5→v1.7** in body BC table, AC-AUDIT-001 trace, frontmatter BC status comment, and points breakdown comment. |
| 2.13 | adv-p208-p01-001-deferral-anchor-2026-06-29 | 2026-06-29 | story-writer | **ADV-P208-P01-001 deferral-anchor fix: "4x-query-reparse perf" Target cell updated from "follow-up story" to concrete story ID S-QUERY-GATE-REPARSE-CONSOLIDATION-001.** Per CLAUDE.md Canonical Principle Rule 3, a deferral target must be a concrete real story ID, not an open-ended phrase. Searched STORY-INDEX (v2.526, 219 stories) — no existing query-engine performance/gate-consolidation story covers this surface. Created NEW draft stub story `S-QUERY-GATE-REPARSE-CONSOLIDATION-001-query-gate-reparse-consolidation.md` (P3; SS-11 Query Execution; prism-query; 5 pts; depends_on S-DEMO-FIDELITY-REMEDIATION-001; `behavioral_contracts: []` — pending PO authorship per Spec-First Gate S-7.01; post-demo-backlog wave). Deferred items table Target cell: "follow-up story" → "S-QUERY-GATE-REPARSE-CONSOLIDATION-001". No code change. No AC, BC, or Red Gate test change. State-manager to register new story in STORY-INDEX. |
| 2.12 | 4lens-regate-ac-disc-reanchor-wiring-seam-tests-2026-06-29 | 2026-06-29 | story-writer | **4-lens re-gate reconciliation: AC-DISC re-anchored to BC-2.11.007 v1.9 §Mechanism B.1/PC-DISC-001 (F-L3-MED-001); BC-2.10.016 co-trace dropped from AC-DISC (F-L3-MED-002); 3 wiring-seam tests added (F-LENS4-MED-001); "7 BCs" comment fix (F-L3-LOW-001); feature HEAD 33817a82→d9bb75c2.** (1) **F-L3-MED-001 — AC-DISC re-anchored to BC-2.11.007 v1.9 §Mechanism B.1/PC-DISC-001.** PO amended BC-2.11.007 v1.8→v1.9 adding §Mechanism B.1 "Planner-Side Entity-Discriminator Auto-Seeding" with postconditions PC-DISC-001/002/003. AC-DISC now single-anchors to `BC-2.11.007 v1.9 §Mechanism B.1 / PC-DISC-001` — the precise contractual anchor for the absent-aql auto-seeding behavior. BC-2.11.007 version updated v1.8→v1.9 in: frontmatter BC status comment, body BC table row, BC-array-propagation comment line. (2) **F-L3-MED-002 — BC-2.10.016 co-trace dropped from AC-DISC.** BC-2.10.016 governs `render_*` prompt FROM-ready names (AC-AUDIT-004's domain), not the planner seeding behavior. The co-trace was a semantic mismatch. BC-2.10.016 REMAINS in the `behavioral_contracts:` array and body BC table — it is still validly cited by AC-AUDIT-004. Only removed from AC-DISC's single anchor line. BC-array-propagation comment for BC-2.10.016 updated: "cited in AC-AUDIT-004 and AC-DISC" → "cited in AC-AUDIT-004". (3) **F-LENS4-MED-001 — 3 wiring-seam tests added to red_gate inventory.** Source-verified against worktree (`.worktrees/S-DEMO-FIDELITY-REMEDIATION-001/crates/prism-query/src/materialization.rs`, module `armis_discriminator_wiring_seam_tests`, lines 3317+): `test_F_LENS4_MED001_armis_alerts_pipeline_seeds_in_alerts_aql_filter`, `test_F_LENS4_MED001_armis_devices_pipeline_seeds_in_devices_aql_filter`, `test_F_LENS4_MED001_armis_alerts_user_supplied_aql_passes_through_pipeline`. These drive `run_materialization_pipeline` through a recording stub adapter asserting the seeded `aql` reaches `QueryParams`. `red_gate_tests` 46→49. Arithmetic: 46 (v2.10) + 3 wiring-seam tests = 49. AC-DISC body section updated with wiring-seam test descriptions. Integration verification HEAD updated 33817a82→d9bb75c2. (4) **F-L3-LOW-001 — "6 BCs" comment corrected to "7 BCs".** Frontmatter BC-array-propagation comment "All 6 BCs cited in at least one AC body trace" was stale (array has 7 entries). Fixed to "All 7 BCs cited in at least one AC body trace." Array verified: BC-2.11.001, BC-2.11.022, BC-2.11.019, BC-2.10.016, BC-2.10.012, BC-2.11.016, BC-2.11.007 = 7 entries. |
| 2.11 | pol-8-bc-2.11.007-propagation-2026-06-28 | 2026-06-28 | story-writer | **POL-8 reconciliation: BC-2.11.007 v1.8 added to `behavioral_contracts:` array and body BC table for AC-DISC anchor; BC count 6→7.** AC-DISC traces to BC-2.11.007 v1.8 §Mechanism B (armis AQL discriminator seeding) — the precise contractual anchor for AQL discriminator injection behavior. BC-2.11.007 was absent from the frontmatter array (POL-8 violation: every AC-traced BC must be in the array + propagated to the body BC table). Fixes: (1) BC-2.11.007 added to `behavioral_contracts:` array (now 7 entries). (2) Body BC table row added: `| BC-2.11.007 | v1.8 | BC-2.11.007: Sensor Filter Push-Down |` (H1 title verbatim per POL-7 — read from BC file). (3) BC-array-propagation comment updated: BC-2.16.002 → BC-2.10.016 line now notes AC-DISC co-trace; new BC-2.11.007 line added. (4) Token Budget "6 BCs" → "7 BCs" (~12k → ~14k). (5) AC-DISC trace pinned to `BC-2.11.007 v1.8 §Mechanism B`. (6) BC-2.11.007 added to inputs list. BC-2.11.007 is `status: active`, `lifecycle_status: active` (v1.8 since 2026-06-05) — POL-14 draft→active at merge is a no-op. Version bump 2.10→2.11. |
| 2.10 | f-l2-crit001-armis-discriminator-4lens-regate-2026-06-28 | 2026-06-28 | story-writer | **Armis entity-discriminator AC added (F-L2-CRIT-001), WHERE-divergence guard test made load-bearing, doc-comment + LOW-2 path + OBS-1 module-attribution corrections; red_gate_tests 42→46.** (1) **F-L2-CRIT-001 (CRIT) — AC-DISC added.** `pub(crate) fn seed_armis_entity_discriminator` in `materialization.rs` seeds the Armis AQL discriminator when absent/empty: `armis_alerts → aql="in:alerts"`, `armis_devices → aql="in:devices"`; user-supplied non-empty `aql` preserved verbatim; non-armis tables unaffected. Four Red Gate tests in inline module `armis_discriminator_tests`: `test_f_l2_crit001_armis_alerts_no_aql_seeds_in_alerts_discriminator`, `_devices_no_aql_seeds_in_devices_discriminator`, `_armis_alerts_existing_aql_not_overwritten`, `_non_armis_table_filters_unchanged`. Full `just check` GREEN (5074/5074, HEAD 33817a82), DTU armis integration tests pass. AC-DISC added to story body (new §Area I-B), frontmatter inventory, crates_touched prism-query comment, File Structure MODIFIED row, Architecture Mapping row. `acceptance_criteria_count` 16→17. `red_gate_tests` 42→46 (arithmetic: 42 v2.9 + 4 AC-DISC tests). materialization.rs removed from "Files NOT modified" note (now MODIFIED). (2) **WHERE-divergence guard test (note only).** `test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001` is a load-bearing guard ensuring WHERE does NOT accept the agg-fn predicate form per ADR-048 §Constraint. The test lives in `f_pxl3_med002_having_agg_predicate_col_gate_tests` module — code was already delivered by the implementer; this entry records the module attribution verification. (3) **LOW-2 — AC-REG-2 path disambiguation.** `test_bc_2_11_022_registry_parity` is cited with full path `crates/prism-mcp/tests/reference_content.rs` (integration test file, not a src inline module). Companion note for `test_bc_2_11_022_ci_3tier_gate` also updated with full path. (4) **OBS-1 — module attribution correction.** Frontmatter Red Gate inventory comment and AC-M2 blockquote both previously attributed the three ADR-048 agg-fn tests (`test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038`, `_valid_col_no_e_query_038`, `test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001`) to `f_pwl1_low001_having_column_gate_tests`. Source-verified against engine.rs: these tests are in `f_pxl3_med002_having_agg_predicate_col_gate_tests` (line 7102); `f_pwl1_low001_having_column_gate_tests` (line 6838) contains only the bare-column tests. Both frontmatter comment and AC-M2 blockquote corrected. Anchored to BC-2.11.007 §Mechanism B and BC-2.10.016 v1.2 postcondition. |
| 2.9 | f-pxl3-med-001-f-pxl4-low-001-adr-048-tests-2026-06-28 | 2026-06-28 | story-writer | **F-PXL3-MED-001 AC-M2 corrected, F-PXL4-LOW-001 cite corrected, 3 HAVING agg-fn Red Gate tests added (ADR-048), red_gate_tests 39→42.** (1) **F-PXL3-MED-001 (MED) — AC-M2 HAVING-extraction prose corrected.** Prior text claimed HAVING (Position 6) uses `extract_field_paths_from_expr` (recurses into FuncCall args). The ACTUAL code path is `extract_predicate_columns` → `collect_predicate_columns`, whose `Predicate::Compare` arm NOW handles both bare `Expr::Field` LHS (bare-column HAVING predicates) and `Expr::FuncCall` LHS (aggregate-function HAVING predicates, ADR-048), in both cases recursing via `extract_field_paths_from_expr` to collect all `Expr::Field` column references. The WHERE predicate grammar deliberately does NOT accept aggregate-function predicate LHS (ADR-048); `WHERE count(col) > 5` remains an E-QUERY-001 parse error. AC-M2 prose replaced with the architect-verified description. BC-2.11.016 v1.5 HAVING blockquote expanded to document the ADR-048 grammar extension and all 5 test names in the inline module. §References extended: ADR-048 governs HAVING/WHERE predicate-grammar divergence; BC-2.11.016 stays v1.5. (2) **F-PXL4-LOW-001 (LOW) — AC-REG-2 test cite corrected.** `test_bc_2_11_022_ci_3tier_gate` was cited as the per-field-UDF-parity guard. The actual per-field parity guard is `test_bc_2_11_022_registry_parity` (in `reference_content.rs`, line 304) — it builds a known `InfusionRegistry` and asserts `build_reference_content` renders per-field callable names, not infusion_ids. `test_bc_2_11_022_ci_3tier_gate` guards the 3-tier `ExampleKind` shape (a separate concern). AC-REG-2 Red Gate test paragraph replaced with accurate prose naming `test_bc_2_11_022_registry_parity` as the parity guard; note added clarifying the distinct role of `test_bc_2_11_022_ci_3tier_gate`. Frontmatter inventory `--- AC-REG-2 ---` comment updated from `test_bc_2_11_022_ci_3tier_gate` to `test_bc_2_11_022_registry_parity`. (3) **3 HAVING agg-fn Red Gate tests added (ADR-048).** Three new tests from the ADR-048 grammar extension (all in `engine.rs` inline module `f_pwl1_low001_having_column_gate_tests`) added to the frontmatter red_gate inventory under a new `--- AC-M2 HAVING agg-fn predicate tests ---` section: `test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038`, `test_BC_2_11_016_having_agg_fn_predicate_valid_col_no_e_query_038`, `test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001`. `red_gate_tests` 39→42. Arithmetic updated: 39 (v2.8) + 3 HAVING agg-fn tests = 42. |
| 2.8 | bc-2.11.016-v1.5-cite-propagation-having-tests-2026-06-28 | 2026-06-28 | story-writer | **BC-2.11.016 v1.4→v1.5 cite propagation + F-PWL3-MED-001 red_gate_tests semantics fix + HAVING tests inventory (ITEM 1/2/3).** (1) **BC-2.11.016 v1.4→v1.5 cite propagation (ITEM 1):** PO bumped BC-2.11.016 v1.4→v1.5 (F-PWL1-LOW-001 HAVING coverage mandate — HAVING added as 6th column-gate position; same `Option<Predicate>` extraction path as WHERE). Live cite sweep (POL-29: `rg 'BC-2.11.016 v1\.4' .factory/`): 2 live cites in this story updated — frontmatter BC status comment (line 70) and body BC table version cell. AC-M1 trace updated from bare `BC-2.11.016` to `BC-2.11.016 v1.5`. AC-M2 header trace updated to `BC-2.11.016 v1.5`; body prose expanded to name HAVING as the 6th gate position per BC-2.11.016 v1.5 §Implementation location table; blockquote added describing the F-PWL1-LOW-001 HAVING mandate, extraction mechanism, and new tests. POL-7 body BC table Title cell verified verbatim: `BC-2.11.016: E-QUERY-038 Column-Not-Found Plan-Time Gate (L4)` — no change needed. POL-29 sweep result: `S-DEMO-PRISMQL-ONBOARDING-001-B` also has live `BC-2.11.016 v1.4` cites (status: draft, not merged) — OUTSIDE this dispatch scope; reported to orchestrator for separate update (ONBOARDING-001-B is the BC anchor story per PO). (2) **F-PWL3-MED-001 red_gate_tests semantics fix (ITEM 2):** The `red_gate_tests` inventory comment previously claimed "counts ALL story-delivered tests" — a FALSE universal. The correct semantics: `red_gate_tests` is the TDD-driving Red Gate SUBSET (tests written RED before code landed, plus inline and guard tests that drive story-delivered code surfaces). The complete delivered test set is enumerated in §File Structure Requirements. Chosen option: (a) — reword to precise subset definition, point to §File Structure for the full set. Removed the false "ALL" claim. Frontmatter comment, Token Budget (`§File Structure Requirements`), and semantics description are now mutually consistent under the new precise definition. (3) **HAVING tests added (ITEM 3):** Two inline tests in `engine.rs` module `f_pwl1_low001_having_column_gate_tests` — `test_BC_2_11_016_having_column_gate_typo_fires_e_query_038` and `test_BC_2_11_016_having_column_gate_valid_col_no_e_query_038` — added to the frontmatter Red Gate inventory under a new `--- AC-M2 HAVING column gate ---` section. `red_gate_tests` 37→39. Arithmetic updated: 37 (v2.7) + 2 HAVING = 39. |
| 2.7 | f-pul3-med-001-test-inventory-reconcile-2026-06-28 + f-pql2-obs001-inventory-gap-2026-06-28 | 2026-06-28 | story-writer | **F-PUL3-MED-001 test-inventory reconcile + F-PQL2-OBS-001 inventory gap closure (folded).** (1) F-PUL3-MED-001: `red_gate_tests` corrected 33→35 (+2 F-PJL tests added mid-cascade: `test_f_pjl1_high001_non_builtin_unknown_still_triggers_e_query_039` and `test_f_pjl4_med001_scheduled_path_table_gate_fires_before_capability_gate`, both in `bc_2_11_019_n1b_test.rs`). File Structure table: `bc_2_11_019_n1b_test.rs` cell 17→19; `test_enrich_udf_not_found_display.rs` cell 4→5 (5th test: `test_f_pbl1_low002_display_self_sorts_available_infusions`, verified against worktree). `crates_touched` prism-core comment updated 4→5 tests with 5th test name. (2) F-PQL2-OBS-001 inventory gap: `crates/prism-mcp/tests/f_pql2_obs001_skeleton_placeholder_guard_test.rs` (2 tests: `test_f_pql2_obs001_query_skeleton_no_bare_timestamp`, `test_f_pql2_obs001_datetime_arithmetic_uses_placeholder`; BC-2.10.016 v1.2) was unlisted in the File Structure table. Added CREATED row. Red Gate semantics decision: ALL story-delivered tests including mid-cascade regression guards are counted; F-PJL (2) and F-PQL2-OBS-001 (2) guards are both mid-cascade regression guards — counting both or neither is the only consistent choice; both are counted. **Final arithmetic: 33 prior + 2 F-PJL + 2 F-PQL2-OBS-001 = 37.** `red_gate_tests` 35→37. Token Budget table "8 new test files"→"9 new test files". Frontmatter inventory comment: new `F-PQL2-OBS-001 skeleton-placeholder guards` section added with both test names; arithmetic comment added. Internal consistency: 9 named story-owned test files (19+4+2+5+1+3+5+2+2 = 43) + 1 inline test + 1 existing test + 1 compile-fail gate = 46 countable items; `red_gate_tests: 37` counts the 34 Rust `fn test_` items (excluding shared-file pre-existing tests) + 1 inline test + 1 existing test + 1 compile-fail gate. No ACs/BCs modified. |
| 2.6 | f-prl3-prose-reconcile-2026-06-28 | 2026-06-28 | story-writer | **F-PRL3-MED-001 + F-PRL3-LOW-001 + comprehensive prose reconcile.** (1) **F-PRL3-MED-001 (MED) — AC-SAP-1 rewritten.** Prior text falsely stated "this delivery added no new `event_type` emission" and "SAP-1 scan confirmed zero new `event_type` values were introduced. No BC-2.16.002 catalog row addition is required." This was FALSE: M1 + N2 fixes introduced two new closed-set `method` labels on the `table_registry.rwlock_poisoned` catalog row (`columns_for_table`, `check_availability_gate.dot_notation`) and a second emission site on the `column_not_found.rejected` row (M1 single-tenant path), which required and received BC-2.16.002 v1.90→v1.91 amendment (F-PHL3-MED-001). New AC-SAP-1 accurately states: no new `event_type` literal value introduced; two closed-set method labels and one additional emission site extended existing catalog rows; PG-LP11-001 obligation fulfilled by BC-2.16.002 v1.91; §References cite BC-2.16.002 v1.91. (2) **F-PRL3-LOW-001 (LOW) — AC-AUDIT-001 priority-2 severity-vocabulary ladder extended.** Prior prose listed only "(crowdstrike → Title-case, armis → UPPER-case)". BC-2.10.012 v1.5 §Auto-generated example queries and the SENSOR_SEVERITY_VOCABULARY in code also carry: cyberint → lowercase `'high','critical'`; claroty → not registered (no `severity` column). Both entries added to the priority-2 table cell. (3) **Comprehensive prose reconcile (no further stale items found).** All other AC behavioral descriptions (DataFusion built-in exclusion SQL-mode-only, pipe-mode fires E-QUERY-039, CWE-407 cap, dot-notation is_registered suggestion gating, 4-tier example ladder) verified accurate against delivered code+BCs. §Changelog verified complete for substantive cascade fixes (cyberint vocab F-PHL2 already captured in BC-2.10.012 v1.5 cite; BC-2.16.002 v1.91 dependency F-PHL3 now covered by this row; built-in exclusion F-PJL1 captured in v2.3; pipe/SQL split F-PNL1 captured in v1.8/v1.9). No genuine code/BC defects identified beyond the already-deferred pql_hints Category-1 hint-text divergence (BC-2.10.012 §pql_hints vs. code, tracked in deferred items table, requiring PO adjudication). |
| 2.5 | f-ppl3-low-001-ac-n1b-impl-req-prose-alignment-2026-06-28 | 2026-06-28 | story-writer | **F-PPL3-LOW-001 closure — AC-N1B implementation-requirement prose made mechanism-agnostic.** The prior wording `"the exclusion check MUST use ctx.state().scalar_functions().get(name) (live SessionContext registry), NOT a hard-coded allowlist"` was stricter than the ratified BC: BC-2.11.019 v1.5 §Postconditions implementation note (F-PJL1-HIGH-001) uses `"e.g., ctx.state().scalar_functions().get(name)"` (permissive) and `"or equivalent"`. The shipped code uses `SessionStateDefaults::default_scalar_functions()` (a `LazyLock`), which the BC already ratifies as the equivalent mechanism. New prose: `"the exclusion check MUST exclude DataFusion built-in scalars by querying DataFusion's runtime-derived default scalar-function set (SessionStateDefaults::default_scalar_functions(), or the equivalent ctx.state().scalar_functions()), NOT a hard-coded allowlist."` Observable behavior and BC trace are unchanged. BC-2.11.019 v1.5 is already permissive — no BC amendment required. |
| 2.4 | bc-2.10.012-v1.5-propagation-2026-06-28 | 2026-06-28 | story-writer | **BC-2.10.012 v1.4→v1.5 cite propagation (F-PLL2-MED-001).** PO bumped BC-2.10.012 v1.4→v1.5 (§"Auto-generated example queries" rewritten to document the accurate shipped 4-tier per-sensor priority ladder; no behavioral/AC change — code already implements this, BC now matches it). Version cite sweep: all live (non-changelog) `BC-2.10.012 v1.4` cites updated to `v1.5` — 4 sites: frontmatter subsystem anchor comment (line 20), frontmatter points breakdown comment (line 64), frontmatter BC status comment (line 69), body BC table version cell, AC-AUDIT-001 header trace. POL-7 body BC table Title cell verified verbatim against BC H1: `BC-2.10.012: \`prism_describe\` Schema Discovery Tool (L2)` — no change needed. AC-AUDIT-001 alignment confirmed: AC specifies the CRIT-1 priority ladder for `build_example_query`; v1.5 BC description now matches what AC-AUDIT-001 and the code already do — no new AC needed, no contradiction introduced. |
| 2.3 | bc-2.11.019-v1.5-propagation-2026-06-28 | 2026-06-28 | story-writer | **BC-2.11.019 v1.4→v1.5 propagation + F-PJL1-HIGH-001 built-in-exclusion.** PO bumped BC-2.11.019 v1.4→v1.5 (F-PJL1-HIGH-001: DataFusion built-in scalar functions are excluded from E-QUERY-039 SQL-mode firing condition; gate now requires (a) not a PQL built-in ScalarFunc variant AND (b) not in DataFusion `ctx.state().scalar_functions()` AND (c) not in `InfusionRegistry.udf_to_infusion`). **Version cite sweep:** all live (non-changelog) `BC-2.11.019 v1.4` cites updated to `v1.5` — 11 body sites (frontmatter comments ×3, body BC table version cell, AC-N1B header trace, Gate-ordering note, WHERE-clause note, §Precondition 1(b) body-cite, `available_infusions Vec<String>` note, AC-C1C2 trace, Tasks step 6(b), Previous Story Intelligence §5). **AC-N1B decision:** added built-in-exclusion note block-quote to AC-N1B (not a new standalone AC — the exclusion is a refinement of the existing gate firing condition, covered by BC-2.11.019 v1.5 §Postconditions; new tests added per EC-11-064/065). **New tests (EC-11-064/065):** `test_bc_2_11_019_n1b_builtin_passthrough_lower` + `test_bc_2_11_019_n1b_builtin_passthrough_coalesce` added to Red Gate test inventory and File Structure table; `bc_2_11_019_n1b_test.rs` 15→17 tests; `red_gate_tests` 31→33. **New edge cases:** EC-016 (EC-11-064: `lower` passes gate) + EC-017 (EC-11-065: `upper`/`coalesce` pass gate). Token Budget story spec ~16k→~17k, test files ~18k→~19k, total ~88k→~90k. |
| 2.2 | pol-7-title-normalization-2026-06-27 | 2026-06-27 | story-writer | POL-7 title normalization (D-571 amendment): normalized all 6 Title cells in the Behavioral Contracts body table to match each BC's H1 VERBATIM — added the `BC-N.NN.NNN:` prefix to the 5 rows that had stripped it (BC-2.11.001, BC-2.11.022, BC-2.11.019, BC-2.10.016, BC-2.10.012). BC-2.11.016 was already verbatim. All 6 version cites confirmed current against BC frontmatter (v1.15, v1.1, v1.4, v1.2, v1.4, v1.4 — no drift). No §References section present; no other citation surfaces. |
| 2.1 | f-l3-high-001-f-l3-obs-001-remediation-2026-06-27 | 2026-06-27 | story-writer | F-L3-HIGH-001 (POL-8 frontmatter↔body coherence): Added BC-2.11.016 (E-QUERY-038 Column-Not-Found Plan-Time Gate, v1.4) to `behavioral_contracts:` frontmatter array and body BC table — AC-M1 and AC-M2 genuinely trace to this BC (single-tenant column gate `columns_for_table`/`columns_by_table` + GROUP BY/ORDER BY/JOIN ON column validation). BC-2.11.016 input file added to `inputs:`. All BC-2.11.019 version cites in body updated v1.3→v1.4 (PO bump). Token Budget updated: "5 BCs"→"6 BCs", ~10k→~12k BC tokens, total ~86k→~88k. Frontmatter `behavioral_contracts` comment updated to reference all 6 BCs. AC-SAP-1 rewording: removed "(traces to ... BC-2.16.002)" — per PO verdict, SAP-1 is a standing-probe/discipline compliance reference only; this delivery added no new `event_type` emission, so there is no behavioral trace to BC-2.16.002. AC-SAP-1 now describes the compliance check outcome explicitly (all code fixes use `?`-propagation, SAP-1 scan confirmed zero new `event_type` values). F-L3-OBS-001 (prose accuracy): Updated AC-AUDIT-001 CRIT-1 prose to describe the full `build_example_query` variant priority ladder as implemented in code: (1) aggregate — Integer/Float column present; (2) severity-filter — severity column + registered sensor vocabulary; (3) count-recent — Datetime column found; (4) column-free fallback — no Datetime. Previous prose only described the datetime→column-free axis. EC-015 updated to reference the full priority ladder. Version bump 2.0→2.1. |
| 2.0 | full-scope-expansion-prose-accuracy-sweep-2026-06-27 | 2026-06-27 | story-writer | Major revision: story updated to comprehensively document ALL implemented work beyond the original 5 findings. Gate-coverage expansion: (1) C1/C2 — enrich gate `collect_unknown_scalars_from_sql_query` scans SELECT, WHERE, JOIN ON, GROUP BY, ORDER BY, HAVING positions via canonical single-walk fn; (2) HIGH-1 — SqlPipe-not-exempt scope for N2 (BC-2.11.001 v1.15 mode-agnostic) + SqlPipe enrich gate (test_high1_sqlpipe_head_unknown_scalar_fires_e_query_039); (3) H1 — capability-gate ordering symmetric in execute_scheduled_inner (E-QUERY-011 moved AFTER 037/038/039); (4) M1 — single-tenant column gate via TableRegistry.columns_by_table + columns_for_table; (5) M2 — E-QUERY-038 validates GROUP BY/ORDER BY/JOIN ON columns; (6) L1 — E-QUERY-037 source walk covers HAVING/GROUP BY/ORDER BY/JOIN ON subqueries via collect_expr_sources_into_gate. Describe+prompt correctness: CRIT-1 (build_example_query derives datetime column from spec, not hardcoded 'timestamp'); MED-1 (prompt VALUES aligned to DTU vocabulary); MED-2 (test_bc_2_10_016_med2_prompt_filter_values_match_dtu_vocabulary); OBS-1 (AUDIT-004 scope = 4 render_* modified, not 5 — render_query_tutorial unchanged); OBS-2 (did_you_mean=Some engine test + None test); OBS-4 (deleted crit001_prompt_table_names.rs, superseded); OBS-5 (fail-closed guards). Prose fixes: CRIT-1 (AC-N2 SqlPipe-not-exempt scope added); HIGH-1 (Red Gate inventory expanded 9→31 tests); HIGH-2 (test_bc_2_11_019_n1b_mcp_maps_to_32602 correct file: crates/prism-mcp/tests/bc_2_11_019_n1b_mcp_test.rs, not error_mapping.rs #[cfg(test)]); HIGH-3 (all ci.yml refs → scripts/check-non-exhaustive.sh, grep command corrected); OBS-2 (Estimated Complexity 8→10 pts; N2 anchor → check_availability_gate in table_registry.rs); OBS-3 (Token Budget label v1.8→v2.0); full crates_touched + File Structure updated to list all 20 files touched/created/deleted; 15 new Edge Cases (EC-007–EC-015); new ACs for gate-coverage (AC-C1C2, AC-M1, AC-M2, AC-L1, AC-H1); acceptance_criteria_count 10→16; red_gate_tests 9→31; case-insensitive querying split noted (→ S-PRISMQL-CASE-INSENSITIVE-001); deferred items noted (BC-2.10.012 §pql_hints divergence, 4x-query-reparse perf). Version bump 1.9→2.0. |
| 1.9 | med-1-re-correction-where-clause-code-verified-2026-06-27 | 2026-06-27 | story-writer | MED-1 re-correction: AC-N1B WHERE note aligned to code-verified reality (over-corrected in v1.8). `build_predicate_parser` has no scalar-funcall atom → real `WHERE udf(col)=v` is E-QUERY-001 parse error; `collect_unknown_scalar_from_predicate` WHERE scan is DEFENSIVE/forward-compat (programmatic AST), honoring BC-2.11.019 §Precondition 1(b) AST-contract; SQL projection is the reachable gated path. `ScalarFunc::Unknown` is produced ONLY by `build_sql_expr_parser` (SELECT projections), not by the WHERE predicate grammar. Five locations corrected: (1) AC-N1B WHERE-clause note block quote; (2) AC-N1B Step 2 SQL path bullet; (3) Tasks step 6(b); (4) File Structure table engine.rs enrichment gate row; (5) Architecture Mapping E-QUERY-039 gate row. Matches the implementing test docstring in `crates/prism-query/src/tests/bc_2_11_019_n1b_test.rs` (~lines 355-366). Version bump 1.8→1.9. |
| 1.8 | med-1-where-clause-note-correction-2026-06-27 | 2026-06-27 | story-writer | MED-1: corrected AC-N1B WHERE-clause note — SQL-mode `ScalarFunc::Unknown` gating covers projection AND WHERE per BC-2.11.019 v1.3 §Precondition 1(b) (the WHERE scan is required+implemented via `collect_unknown_scalar_from_predicate`, not "defensive/unneeded"); only the pipe `enrich`-keyword WHERE form is an E-QUERY-001 parse error. Five locations fixed: (1) AC-N1B WHERE-clause note block quote (lines ~249-261); (2) AC-N1B Step 2 SQL path bullet; (3) Tasks step 6(b); (4) File Structure table engine.rs enrichment gate row; (5) Architecture Mapping E-QUERY-039 gate row. Incorrect assertions "no WHERE-clause scan is needed" and "projection-arm scan is COMPLETE coverage" removed. Version bump 1.7→1.8. |
| 1.7 | low-1-ec001-exhaustive-claim-audit-2026-06-27 | 2026-06-27 | story-writer | LOW-1 + EXHAUSTIVE whole-story claim audit: corrected EC-001 phantom string `"No enrichment infusions are currently registered."` → actual code string `"No enrichment functions are currently registered for your deployment."` (resources.rs ~line 1519, Some(empty) path); updated EC-001 test cite from `test_bc_2_11_022_none_registry_placeholder` (covers None path) → `test_bc_2_11_022_some_empty_registry_placeholder` (covers the Some(empty) path). Second inaccuracy found and fixed: frontmatter Red Gate test comment listed phantom test `test_non_exhaustive_count_87_to_88` — this test does not exist; the non-exhaustive gate is a compile-fail crate run via `scripts/check-non-exhaustive.sh EXPECTED=88`, not a named Rust `fn test_*`; corrected to `scripts/check-non-exhaustive.sh EXPECTED=88 (compile-fail gate via shell script, not a named Rust test)`. All other claims across every section verified accurate against feature worktree code: AC-N1 (resources.rs dedup key, per-field UDF names, test name), AC-N1B (EnrichUdfNotFoundDetails struct fields, map_prism_error -32602 arm, Display template, gate ordering, test names), AC-N2 (check_availability_gate / is_registered function names, TableNotAvailable variant, udf_to_infusion field), AC-AUDIT-001 (build_tables_for_client format string, pql_hints generic hint text), AC-AUDIT-004 (render_* function names, FROM-ready table names, regex), AC-REG-1/REG-2/DEMO-001/SAP-1, §Edge Cases (EC-002 through EC-006), §Red Gate Tests table (all other names confirmed present), §File Structure, §Library Requirements, §Architecture Mapping, §Dev Notes, §Previous Story Intelligence. No further inaccuracies found. Token Budget label 1.6→1.7. Version bump 1.6→1.7. |
| 1.6 | obs-1-ac-prose-accuracy-audit-2026-06-27 | 2026-06-27 | story-writer | OBS-1 + AC-prose accuracy audit: corrected AC-AUDIT-001 phantom `pql_hints[0]` "This client has N tables:" string claim — that string does not exist in `build_pql_hints`; the actual non-empty `pql_hints[0]` is `"Use 'SELECT * FROM <table> LIMIT 25' to query any of the N table(s) above."` (a generic usage hint with `<table>` placeholder, no embedded table names). The disambiguation guarantee is in `TableDescriptor.name` + `example_query`, not in `pql_hints`. Full AC-prose-vs-code accuracy sweep (AC-N1, AC-N1B, AC-N2, AC-AUDIT-004, AC-REG-1, AC-REG-2, AC-DEMO-001, AC-SAP-1): all other AC prose matches code — no further inaccuracies found. Token Budget label 1.5→1.6. Version bump 1.5→1.6. |
| 1.5 | med-1-exhaustive-all-forms-bc-cite-audit-2026-06-27 | 2026-06-27 | story-writer | Exhaustive all-forms BC version-cite audit (Pass MED-1): residual compact-form `(v1.14/v1.1/v1.3/v1.4/v1.2)` at frontmatter line 69 contained stale `v1.14` for BC-2.11.001 (canonical v1.15) — missed by prior prefixed-grep sweeps that searched `BC-2.11.001 v1.14` but not the parenthesized slash-joined form. Redundant compact version enumeration removed from frontmatter comment (drift risk; body BC table is canonical); replaced with accurate status note: 4 active BCs + BC-2.11.019 draft→active at merge per POL-14. Descriptor accuracy fix: prior comment said "all 5 BCs are active" — BC-2.11.019 is `status: draft` (confirmed against BC frontmatter). All prefixed-form cites verified correct (zero stale). Re-grep confirms ZERO live stale `v1.14` or stale compact-form cites remain (changelog rows excepted, TD-VSDD-091 exempt). Token Budget version label updated 1.4→1.5. Version bump 1.4→1.5. |
| 1.4 | pass-5-bc-version-cite-sweep-2026-06-27 | 2026-06-27 | story-writer | Comprehensive BC version-cite sweep (Pass-5 MED-1/MED-2): BC-2.11.001 v1.14→v1.15 (5 sites: frontmatter comment line 62, BC table line 175, AC-N2 header trace line 338, AC-REG-1 trace line 415, EC-004 table line 748); BC-2.11.019 v1.2→v1.3 (2 residual sites the v1.3 "throughout" claim missed: AC-N1B scope note line 222, Previous Story Intelligence line 596). Token Budget version label updated 1.3→1.4. Frontmatter version bumped 1.3→1.4. BC-2.11.022 v1.1, BC-2.10.016 v1.2, BC-2.10.012 v1.4 verified current — no changes needed. POL-29 version-cite recurrence break. |
| 1.3 | po-e-query-039-reconciliation-2026-06-27 | 2026-06-27 | story-writer | HIGH-005 changelog reorder (POL-32 monotonic_descending violation — rows were 1.0→1.2→1.1; reordered to strict descending 1.3→1.2→1.1→1.0). Sync to PO E-QUERY-039 reconciliation: (1) canonical Display message template added to AC-N1B Step 3 — EXACTLY `E-QUERY-039: enrichment infusion '{infusion}' is not registered; available: [{available_infusions}]{did_you_mean}` (bracket-wrapped comma-joined Vec<String>; did_you_mean = ` Did you mean: '{x}'?` when Some, omitted when None); (2) available_infusions confirmed as `Vec<String>` (PO-ratified canonical type — already matched struct definition; now explicit in AC text and observable behavior); (3) enrich-LAST gate ordering added as explicit callout in AC-N1B: E-QUERY-039 fires LAST (E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039); (4) WHERE-clause note added: SQL-mode ScalarFunc::Unknown gate covers projections only — WHERE-clause enrichment calls are E-QUERY-001 parse errors at the grammar level; the projection-arm scan is complete coverage; defensive visitor arm noted for programmatic AST; (5) BC-2.11.019 version references updated v1.2→v1.3 throughout (BC table, AC-N1B header trace, frontmatter I2 anchor comment, crates_touched comment, BC status comment). LOCAL Pass-1 fix-burst closures noted for record: CRIT-001 (prompt table names), HIGH-001 (gate ordering), HIGH-002 (Display template), HIGH-003 (WHERE scan), OBS-1 (tie-break), OBS-2 (doc-comment). Version bump 1.2→1.3. |
| 1.2 | pre-tdd-api-mismatch-corrections-2026-06-26 | 2026-06-26 | story-writer | Four internal-API mismatch corrections found before TDD delivery. C1 (HIGH): E-QUERY-037 `map_prism_error` arm CONFIRMED PRESENT in error_mapping.rs (~line 166, doc "S-3.13 AC-2; BC-2.11.001") — v1.1 wrongly marked it net-new; corrected throughout (AC-N1B, AC-N2, File Structure table, Tasks, Architecture Mapping). I1 (HIGH): `InfusionRegistry.udf_names()` does NOT exist — only `udf_descriptors()` exists; corrected gate code to derive UDF names inline via `udf_descriptors().iter().map(|d| d.name.clone()).collect()` throughout; no new public method added (keeps EXPECTED increment at exactly 87→88). I2 (MED-HIGH): enrichment-gate insertion point pinned to `engine.rs` (new AST-visitor pass before `check_availability_gate`; pipe arm: `PipeStage::Enrich` → `EnrichStage.infusion`; SQL arm: `ScalarFunc::Unknown` in projections; both distinct visitor arms, same validation loop); "implementer determines exact file" language removed. S1 (LOW): Red Gate test for N1-B relaxed — `did_you_mean.is_some()` assertion removed; test must assert `available_infusions` non-empty and error variant/code only (registered per-field UDF names likely > Levenshtein-3 from "threat_intel" so `did_you_mean: None` is valid). Points/Red Gate test count/token budget UNCHANGED from v1.1 (10 pts / 9 tests / ~53k tokens). |
| 1.1 | remove-uncertainty-scope-correction-2026-06-26 | 2026-06-26 | story-writer | Scope corrections from post-materialization remove-uncertainty pass. THREE HIGH findings resolved: (1) N1-B re-scoped net-new — EnrichUdfNotFound variant + EnrichUdfNotFoundDetails struct do NOT exist in workspace (zero matches); AC-N1B now requires creating error.rs variant + plan-time gate (prism-query) + map_prism_error -32602 arm (error_mapping.rs) from scratch; story-investigation framing removed; BC-2.11.019 promotes draft→active at merge (POL-14). (2) N2 gate-ordering fix relocated from materialization.rs to table_registry.rs (check_availability_gate / is_registered) + engine.rs; materialization.rs resolve_source_refs is a DIFFERENT code path (E-QUERY-036, not E-QUERY-037); AC-N2 scope note + scope-note re-written accordingly. (3) AC-REG-1 amended: previously incorrectly stated no new #[non_exhaustive] types; now REQUIRES EnrichUdfNotFoundDetails with #[non_exhaustive]; ci.yml EXPECTED 87→88; CLAUDE.md sentence updated. map_prism_error E-QUERY-037 arm also changed from conditional to net-new. crates_touched expanded: + prism-core/src/error.rs, + prism-query/src/table_registry.rs, + prism-query/src/engine.rs. Points: 8→10. Red Gate tests: 7→9. Token budget: ~43k→~53k. |
| 1.0 | demo-fidelity-remediation-2026-06-26 | 2026-06-26 | story-writer | Initial story. Materializes the 5 code-fix ACs from the 2026-06-26 pre-flight audit remediation plan. Traces to BC-2.11.001 v1.14 (EC-11-067 N2), BC-2.11.022 v1.1 (EC-11-022-006 N1), BC-2.11.019 v1.2 (N1-B), BC-2.10.012 v1.4 (AUDIT-001), BC-2.10.016 v1.2 (AUDIT-004). 10 ACs; 7 Red Gate tests; 8 pts; P0; depends_on S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001. |
