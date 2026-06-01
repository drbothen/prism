# Evidence Report: S-DEMO-001

**Story:** S-DEMO-001 — prism-bin: SpecDrivenSensorAdapter + Boot Step 9A (closes GAP-002-A)
**Version:** 1.10
**LOCAL Adversary Cascade:** Converged (pre-PR, per-story-delivery step)
**PR:** pending (branch feature/S-DEMO-001, HEAD c50e72f7)
**Date:** 2026-06-01

---

## Coverage Summary

| AC | Description | Evidence File | Evidence Type | Status |
|----|-------------|---------------|---------------|--------|
| AC-001 | SpecDrivenSensorAdapter delegates to PipelineExecutor for CrowdStrike (plugin auth) | AC-001-crowdstrike-delegates-to-pipeline-executor.txt | Red Gate test | PASS |
| AC-002 | SpecDrivenSensorAdapter delegates for bearer_static sensors (Armis/Claroty) | AC-002-bearer-static-extracts-token-from-sensor-auth.txt | Red Gate test | PASS |
| AC-003 | SpecDrivenSensorAdapter delegates for cookie_roundtrip sensor (Cyberint) | AC-003-cyberint-cookie-auth-injects-access-token.txt | Red Gate test | PASS |
| AC-004 | Boot step 9A registers exactly N adapters (N = eligible after skips) | AC-004-boot-step9a-registers-correct-adapter-count.txt | Red Gate test | PASS |
| AC-005 | Adapter registration is per-org with correct overlay applied | AC-005-boot-step9a-uses-resolved-spec-overlay-url.txt | Red Gate test | PASS |
| AC-006 | Empty spec_catalog → AdapterRegistry empty; no error | AC-006-empty-spec-catalog-zero-adapters.txt | Unit test | PASS |
| AC-007 | AdapterRegistry::get(org_id, sensor_id) returns adapter for registered pairs | AC-007-adapter-registry-get-returns-adapter.txt | Integration test | PASS |
| AC-008 | BearerStaticAuthProvider correctly converts SensorAuth to Authorization Bearer | AC-008-bearer-static-auth-provider-authorization-header.txt | Unit test | PASS |
| AC-009 | StaticCookieAuthProvider injects Cookie: access_token, not Authorization Bearer | AC-009-static-cookie-auth-provider-access-token-injection.txt | 3 unit tests (2 crates) | PASS |
| AC-010 | Adapter fetch returns OCSF-conformant Arrow RecordBatches (BC-2.01.013 v1.9 items 1-3) | AC-010-ocsf-conformance-arrow-record-batches.txt | 3 conformance tests | PASS |
| AC-011 | No todo!() or unimplemented!() in adapter, boot step 9A, StaticCookieAuthProvider (POL-12) | AC-011-no-todo-pol-12.txt | POL-12 scan test | PASS |
| AC-012 | Adapter handles plugin-auth-failure (double-401 → AuthRefreshFailed error) | AC-012-double-401-auth-refresh-failed.txt | Unit test | PASS |

---

## Red Gate Tests (5 required, 5 pass)

| Test Name | AC | Crate | Result |
|-----------|----|-------|--------|
| test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor | AC-001 | prism-bin | PASS |
| test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth | AC-002 | prism-bin | PASS |
| test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie | AC-003 | prism-bin | PASS |
| test_BC_2_22_001_boot_step9a_registers_correct_adapter_count | AC-004 | prism-bin | PASS |
| test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url | AC-005 | prism-bin | PASS |

---

## Supporting Tests (non-Red-Gate, per-AC)

| Test Name | AC | Crate | Result |
|-----------|----|-------|--------|
| test_BC_2_22_001_boot_step9a_empty_spec_catalog_registers_zero_adapters | AC-006 | prism-bin | PASS |
| test_BC_2_22_001_boot_step9a_unsupported_auth_type_skips_adapter_not_error | AC-006/EC-007 | prism-bin | PASS |
| test_BC_2_11_001_step8_adapter_registry_not_empty | AC-007 | prism-bin | PASS |
| test_BC_2_06_014_boot_step9a_translates_org_slug_to_org_id | AC-005/OQ-2 | prism-bin | PASS |
| test_BC_2_01_013_bearer_static_auth_provider_returns_bearer_token | AC-008 | prism-bin | PASS |
| test_BC_2_01_013_static_cookie_auth_strategy_injects_access_token_not_bearer | AC-009(a) | prism-bin | PASS |
| test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip | AC-009(b) | prism-spec-engine | PASS |
| test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call | AC-009(c) | prism-spec-engine | PASS |
| test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call | AC-009(c) | prism-spec-engine | PASS |
| test_BC_2_01_013_ocsf_conformance_spec_columns_survive_into_arrow_schema | AC-010(a) | prism-bin | PASS |
| test_BC_2_01_013_ocsf_conformance_envelope_derived_not_raw_copied | AC-010(b) | prism-bin | PASS |
| test_BC_2_01_013_ocsf_conformance_sensor_virtual_column_is_canonical_sensor_id | AC-010(c) | prism-bin | PASS |
| test_pol_12_no_untraced_todo_in_prism_bin_production_code | AC-011 | prism-bin | PASS |
| test_BC_2_01_013_spec_driven_adapter_double_401_returns_auth_refresh_failed | AC-012 | prism-bin | PASS |
| test_BC_2_01_013_auth_refresh_failed_display_carries_e_auth_002_taxonomy_code | AC-012 | prism-bin | PASS |
| test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_plugin | AC-001/AC-010 | prism-bin | PASS |
| test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_bearer_static | AC-002/AC-010 | prism-bin | PASS |
| test_BC_2_01_013_fetch_returns_non_empty_ocsf_batches_static_cookie | AC-003/AC-010 | prism-bin | PASS |
| test_BC_2_01_013_spec_driven_adapter_sensor_type_returns_sensor_id_from_spec | AC-001 | prism-bin | PASS |
| test_BC_2_01_013_build_http_client_with_timeout_succeeds | reqwest 30s timeout | prism-bin | PASS |

---

## Suite Totals

| Crate | Tests Run | Passed | Failed | Skipped |
|-------|-----------|--------|--------|---------|
| prism-bin | 107 | 107 | 0 | 0 |
| prism-spec-engine | 513 | 513 | 0 | 10 |

---

## POL-10 Compliance

All evidence files are in `docs/demo-evidence/S-DEMO-001/`
(story-scoped subdirectory). No flat `docs/demo-evidence/*.md` files were created.

---

## Architecture Compliance Verified

| Rule | Verification |
|------|-------------|
| SpecDrivenSensorAdapter lives in prism-bin (NOT prism-sensors) | File is crates/prism-bin/src/spec_driven_adapter.rs — ADR-023 §D3 compliant |
| StaticCookieAuthProvider lives in prism-spec-engine/src/auth_provider.rs | pub struct in prism-spec-engine; not feature-gated |
| BearerStaticAuthProvider lives in prism-bin/src/spec_driven_adapter.rs | Bridges SensorAuth (prism-sensors) + AuthProvider (prism-spec-engine); ADR-023 §Permitted Patterns |
| Cookie name for CookieRoundtrip is access_token (not cyberint_session) | AC-003/AC-009 Red Gate tests assert exact cookie name; ADR-031 D1-a |
| StaticCookieAuthProvider::acquire_token makes ZERO HTTP calls | AC-009(c): struct has no reqwest::Client field; test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call |
| reqwest::Client constructed with .timeout(Duration::from_secs(30)) | test_BC_2_01_013_build_http_client_with_timeout_succeeds: PASS |
| boot.step9a.adapter_registry_populated event has BC-2.16.002 catalog row | SAP-1 probe confirmed in LOCAL adversary cascade; catalog row added in implementation commits |
| Boot step 9A appears between steps 7.5b and 9 in ADR-022 §B table | ADR-022 amended in implementation; no boot sequence reordering |
| No todo!() or unimplemented!() in production code | AC-011 POL-12 scan: PASS |
| OrgSlug → OrgId translation via OrgRegistry::resolve() | test_BC_2_06_014_boot_step9a_translates_org_slug_to_org_id: PASS; id_for_slug NOT used (D-922) |
