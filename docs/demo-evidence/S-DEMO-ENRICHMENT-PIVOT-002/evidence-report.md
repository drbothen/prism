# Evidence Report — S-DEMO-ENRICHMENT-PIVOT-002

**Story:** ThreatIntel/NVD Infusion — Dual-Path: ThreatIntel WASM Plugin + NVD HttpLookup Built-in  
**Branch:** `feature/S-DEMO-ENRICHMENT-PIVOT-002`  
**HEAD at evidence capture:** `b436104d`  
**Date:** 2026-06-19  
**Status:** LOCAL 3-CLEAN converged; all 41 Red Gate tests pass; 2 DTU-EXT tests `#[ignore]`'d with T14 anchor  
**BC:** BC-2.19.001 v1.9

---

## Evidence Summary

| Total ACs | Live/test-execution evidence | Anchored-deferred to T14 | Coverage |
|-----------|------------------------------|--------------------------|----------|
| 21 (AC-001–021, AC-006 included) | 19 ACs | 2 ACs (AC-004 partial / AC-016 partial — DTU-EXT-NVD-001) | 100% of in-scope ACs have at least one evidence artifact |

---

## Recording Artifacts

All recordings are VHS-generated terminal captures. Each `.gif` is suitable for PR embedding; each `.webm` is the archival format. The `.tape` file is the reproducible script source.

| Recording | ACs Covered | Artifact |
|-----------|-------------|----------|
| TOML Loading | AC-001, AC-002 | `AC-001-002-toml-loading.{gif,webm,tape}` |
| ThreatIntel Val-lift Fix | AC-003, AC-019 | `AC-003-019-threatintel-val-lift.{gif,webm,tape}` |
| Pipe Stage Enrich | AC-005, AC-006 | `AC-005-006-pipe-stage-enrich.{gif,webm,tape}` |
| Security Gates | AC-007, AC-008, AC-009, AC-011, AC-012 | `AC-007-008-009-011-012-security-gates.{gif,webm,tape}` |
| HTTP Lookup Infrastructure | AC-013, AC-015, AC-017 | `AC-013-015-017-http-lookup-infra.{gif,webm,tape}` |
| HTTP Lookup Ops + Crate Removal + spawn_blocking | AC-016 (wiremock), AC-018, AC-020 | `AC-016-018-020-http-lookup-ops.{gif,webm,tape}` |
| Full Test Run | All 41 tests (AC-001–020 + extras) | `full-test-run-transcript.txt` |

---

## Per-AC Evidence

### AC-001 — threatintel.infusion.toml parses and loads as plugin-type infusion spec

**Evidence form:** Test-execution capture (VHS recording)  
**Test:** `test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs`  
**Result:** PASS (0.013s)  
**Behavior observed:** `InfusionLoader::load_all` with `threatintel.infusion.toml` (`type = "plugin"`,
`plugin_ref = "threatintel-lookup.prx"`) returns a registry with 3 UDF descriptors for
`threat_is_known_malicious` (Boolean), `threat_score` (Integer), `threat_sources` (Json).
`registry.is_api_backed("threat_score")` returns `true`.  
**Artifacts:**
- `AC-001-002-toml-loading.gif` — VHS recording showing test PASS
- `AC-001-002-toml-loading.webm` — archival recording
- `AC-001-002-toml-loading.tape` — VHS script source
- `full-test-run-transcript.txt` line 35: PASS

---

### AC-002 — nvd.infusion.toml parses and loads as http_lookup-type infusion spec

**Evidence form:** Test-execution capture (VHS recording)  
**Test:** `test_enrichment_pivot_002_nvd_toml_loads_as_http_lookup_and_registers_3_udfs`  
**Result:** PASS (0.035s)  
**Behavior observed:** `InfusionLoader::load_all` with `nvd.infusion.toml` (`type = "http_lookup"`,
`base_url = "https://services.nvd.nist.gov"`, response_path points into CVSS subtree) returns
registry with 3 UDF descriptors for `cvss_base_score` (Float), `cvss_severity` (String),
`cvss_vector` (String). `registry.is_api_backed("cvss_base_score")` returns `true`.
Source on each descriptor is `HttpLookupSource` (not `NullSource` or `PluginInfusionSource`).  
**Artifacts:**
- `AC-001-002-toml-loading.gif` — VHS recording showing test PASS
- `AC-001-002-toml-loading.webm` — archival recording
- `AC-001-002-toml-loading.tape` — VHS script source
- `full-test-run-transcript.txt` line 21: PASS

---

### AC-003 — ThreatIntel WASM plugin: Val-lift fix + wit_bindgen bindings + DTU dispatch

**Evidence form:** Test-execution capture (VHS recording) — in-process fixture, no live DTU  
**Tests:**
- `test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious` (PASS 0.011s)
- `test_enrichment_pivot_002_ac003_plugin_infusion_source_real_path` (PASS 0.044s)
- `test_enrichment_pivot_002_high1_crit2b_threat_intel_canned_fixture_end_to_end` (PASS 0.078s)

**Behavior observed:** `PluginInfusionSource::enrich_single` called with a scenario IOC value.
The Val-lift fix (D2) correctly lifts `Val::Option(Some(Box<Val::String(json_str)>))` to
`Ok(Some(value))`. Plugin dispatches on input-type to the correct DTU route.
`threat_is_known_malicious = true`, `threat_score >= 75` for the scenario IOC.

Note: The live integration test (`test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious`)
runs against an in-process canned fixture (ThreatIntelClone with pre-populated scenario registry).
No live WASM execution against a deployed DTU — but the test uses the production `PluginRuntime::enrich_single`
code path (F-003 rigor: no reimplemented logic).  
**Artifacts:**
- `AC-003-019-threatintel-val-lift.gif` — VHS recording showing test PASSes
- `AC-003-019-threatintel-val-lift.webm` — archival recording
- `AC-003-019-threatintel-val-lift.tape` — VHS script source
- `full-test-run-transcript.txt` lines 8, 26, 27, 33: PASS

---

### AC-004 — NVD HttpLookupSource calls DTU CVE endpoint and returns CVSS fields

**Evidence form:** Test-execution capture (wiremock mock server) — no live NVD API  
**Test:** `test_enrichment_pivot_002_nvd_http_lookup_resolves_scenario_cve_high_cvss` (PASS 0.016s)  
**Behavior observed:** `HttpLookupSource::enrich_single` called with a CVE ID from scenario fixture.
URL built via `Interpolator::interpolate`. Auth appended as `?apiKey=<value>`. Response parsed;
`extract_at_path` extracts CVSS subtree. Returns `cvss_base_score >= 7.0`, `cvss_severity = "HIGH"`.

The live integration tests against real NVD API (`test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template`
and `test_enrichment_pivot_002_http_lookup_source_extracts_response_path_fields`) are
`#[ignore = "DTU-EXT-NVD-001: requires live NVD API; unit coverage in http_lookup.rs::tests::test_enrich_single_extracts_response_path_via_wiremock"]`.
Unit-level coverage of the same `enrich_single` code path is provided by:
- `test_enrich_single_extracts_response_path_via_wiremock` (PASS 0.039s) — wiremock mock
- `test_enrich_single_returns_none_on_path_not_found_via_wiremock` (PASS 0.039s) — wiremock mock

**Deferred to T14 capstone recording:** live end-to-end NVD API call (requires deployed NVD DTU).
Anchor: DTU-EXT-NVD-001 (S-DEMO-ENRICHMENT-PIVOT-003 integration).  
**Artifacts:**
- `full-test-run-transcript.txt` line 2: PASS (nvd_http_lookup_resolves_scenario_cve_high_cvss)
- `full-test-run-transcript.txt` wiremock unit tests: both PASS

---

### AC-005 — | enrich threat_intel(ioc_value) returns Malicious for scenario IOCs

**Evidence form:** Test-execution capture (VHS recording) — in-process mock source  
**Test:** `test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs`  
**Result:** PASS (0.056s)  
**Behavior observed:** `| enrich threat_intel(ioc_value)` pipe stage executes against an in-process
mock source. Result set includes `threat_is_known_malicious`, `threat_score`, `threat_sources`
columns. Scenario IOCs show `threat_is_known_malicious = true`. Output column is `threat_sources`
(Json array — confirmed NOT `threat_source` singular string).  
**Artifacts:**
- `AC-005-006-pipe-stage-enrich.gif` — VHS recording showing test PASS
- `AC-005-006-pipe-stage-enrich.webm` — archival recording
- `AC-005-006-pipe-stage-enrich.tape` — VHS script source
- `full-test-run-transcript.txt` line 12: PASS

---

### AC-006 — | enrich nvd(device_cves_first) returns HIGH CVSS for scenario CVEs

**Evidence form:** Test-execution capture (VHS recording) — in-process mock source  
**Test:** `test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves`  
**Result:** PASS (0.045s)  
**Behavior observed:** `| enrich nvd(device_cves_first)` pipe stage executes against in-process
mock source with pre-populated CVE registry. Result set includes `cvss_base_score`, `cvss_severity`,
`cvss_vector` columns. Scenario CVEs show `cvss_base_score >= 7.0` and `cvss_severity = "HIGH"`.
Uses `device_cves_first` scalar field (NOT `device_cves[0]` — bracket-index not supported, per Ruling 1b).  
**Artifacts:**
- `AC-005-006-pipe-stage-enrich.gif` — VHS recording showing test PASS
- `AC-005-006-pipe-stage-enrich.webm` — archival recording
- `AC-005-006-pipe-stage-enrich.tape` — VHS script source
- `full-test-run-transcript.txt` line 9: PASS

---

### AC-007 — UDF name validation rejects non-identifier characters at parse time (CWE-20)

**Evidence form:** Test-execution capture (VHS recording) — unit tests, no external deps  
**Tests:**
- `test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars` (PASS 0.017s)
- `test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit` (PASS 0.018s)
- `test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers` (PASS 0.015s)

**Behavior observed:** `InfusionLoader::parse` rejects names with SQL injection chars
(`"threat; DROP TABLE"`, `" leading_space"`, `"has-hyphen"`), empty strings, and names starting
with a digit (`"1starts_with_digit"`). Returns `Err(InfusionError::InvalidFieldSpec)`.
Valid names (`threat_is_known_malicious`, `cvss_base_score`, `field1`, `THREAT_SCORE`) accepted.  
**Artifacts:**
- `AC-007-008-009-011-012-security-gates.gif` — VHS recording showing all PASSes
- `AC-007-008-009-011-012-security-gates.webm` — archival recording
- `AC-007-008-009-011-012-security-gates.tape` — VHS script source
- `full-test-run-transcript.txt` lines 14-16: PASS

---

### AC-008 — PluginInfusionSource.config is not publicly readable (CWE-200)

**Evidence form:** Test-execution capture (VHS recording) — unit test  
**Test:** `test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub`  
**Result:** PASS (0.021s)  
**Behavior observed:** `PluginInfusionSource.config` field is `pub(crate)` — external crates
cannot access resolved credential values via struct field access. Test verifies visibility
restriction is in place before credentials are wired in PIVOT-002.  
**Artifacts:**
- `AC-007-008-009-011-012-security-gates.gif` — VHS recording showing PASS
- `AC-007-008-009-011-012-security-gates.webm` — archival recording
- `full-test-run-transcript.txt` line 28: PASS

---

### AC-009 — SandboxViolation URL is not logged at WARN in analyst-visible output (CWE-209)

**Evidence form:** Test-execution capture (VHS recording) — unit test with tracing span capture  
**Test:** `test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log`  
**Result:** PASS (0.016s)  
**Behavior observed:** When `PluginRuntime::enrich_single` returns `Err(PluginError::SandboxViolation { url, .. })`,
the URL is emitted at DEBUG level only. The WARN-level output does NOT contain the URL string.
`plugin_id` may appear in WARN (identifies plugin config, not a network address). Verified via
span capture: formatted WARN output does not contain the sandbox URL.  
**Artifacts:**
- `AC-007-008-009-011-012-security-gates.gif` — VHS recording showing PASS
- `AC-007-008-009-011-012-security-gates.webm` — archival recording
- `full-test-run-transcript.txt` line 30: PASS

---

### AC-010 — spawn_blocking gate (SUPERSEDED by AC-020)

AC-010 is superseded by AC-020 (v1.3 F-004 rigor tightening). Evidence is captured under AC-020.

---

### AC-011 — plugin_ref path is canonicalized and restricted to plugin directory (CWE-22)

**Evidence form:** Test-execution capture (VHS recording) — unit tests  
**Tests:**
- `test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref` (PASS 0.015s)
- `test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted` (PASS 0.016s)
- `test_enrichment_pivot_002_sec003_load_all_rejects_traversal_plugin_ref_production_path` (PASS 0.017s)
- `test_enrichment_pivot_002_sec003_symlink_escape_rejected_by_canonicalize_guard` (PASS 0.018s)

**Behavior observed:** `plugin_ref = "../../etc/passwd.prx"` is rejected via `canonicalize` + `starts_with(plugin_dir)` check before any file I/O. Returns `InfusionError::InvalidFieldSpec`. Relative paths within plugin directory (e.g., `subdir/plugin.prx`) are accepted. Symlink escape via `canonicalize` guard is also rejected.  
**Artifacts:**
- `AC-007-008-009-011-012-security-gates.gif` — VHS recording showing PASSes
- `AC-007-008-009-011-012-security-gates.webm` — archival recording
- `AC-007-008-009-011-012-security-gates.tape` — VHS script source
- `full-test-run-transcript.txt` lines 22-23, 29, 32: PASS

---

### AC-012 — load_all errors do not disclose absolute filesystem paths in MCP responses (CWE-209)

**Evidence form:** Test-execution capture (VHS recording) — unit test  
**Test:** `test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path`  
**Result:** PASS (0.022s)  
**Behavior observed:** `InfusionLoader::load_all` processing a deliberately malformed TOML at
an absolute path. The resulting error string that would surface in an MCP response does NOT
contain the absolute filesystem prefix. Only filename or relative path is exposed.  
**Artifacts:**
- `AC-007-008-009-011-012-security-gates.gif` — VHS recording showing PASS
- `AC-007-008-009-011-012-security-gates.webm` — archival recording
- `full-test-run-transcript.txt` line 31: PASS

---

### AC-013 — InfusionType::HttpLookup + new config types added to infusion/mod.rs

**Evidence form:** Test-execution capture (VHS recording) — unit tests  
**Tests:**
- `test_enrichment_pivot_002_http_lookup_infusion_type_parses_nvd_spec` (PASS 0.056s)
- `test_enrichment_pivot_002_http_lookup_parse_rejects_missing_input_placeholder` (PASS 0.037s)
- `test_enrichment_pivot_002_http_lookup_parse_rejects_invalid_method` (PASS 0.019s)

**Behavior observed:** `InfusionType::HttpLookup` variant parses NVD spec correctly.
`spec.infusion_type == InfusionType::HttpLookup` and `spec.http_lookup_config.is_some()`.
Validation enforced at parse time: `url_template` without `${input}` → `E-INFUSE-013` error.
Method other than GET/POST → `E-INFUSE-013` error. `#[non_exhaustive]` applied to all 4 new types.
`ci.yml` EXPECTED count updated for new types.  
**Artifacts:**
- `AC-013-015-017-http-lookup-infra.gif` — VHS recording showing test PASSes
- `AC-013-015-017-http-lookup-infra.webm` — archival recording
- `AC-013-015-017-http-lookup-infra.tape` — VHS script source
- `full-test-run-transcript.txt` lines 1, 4, 11: PASS

---

### AC-014 — PluginError::EnrichCallFailed + InfusionError::PluginCallFailed variants added

**Evidence form:** Test-execution capture (transcript)  
**Test:** `test_enrichment_pivot_002_plugin_enrich_call_failed_maps_to_infusion_error`  
**Result:** PASS (0.025s)  
**Behavior observed:** `PluginError::EnrichCallFailed { plugin_id, reason }` exists in `prism-core/error.rs`.
`InfusionError::PluginCallFailed { plugin_id, infusion_id, reason }` (E-INFUSE-008) exists.
`map_plugin_error_to_infusion_error` correctly maps `EnrichCallFailed` → `PluginCallFailed`.
Error message format: `"E-INFUSE-008: plugin infusion call failed for '{infusion_id}' via plugin '{plugin_id}': {reason}"`.  
**Artifacts:**
- `full-test-run-transcript.txt` line 17: PASS

---

### AC-015 — InfusionError HttpLookup variants added (E-INFUSE-009/010/011)

**Evidence form:** Test-execution capture (VHS recording) — unit tests  
**Tests:**
- `test_enrichment_pivot_002_http_lookup_failed_error_format_excludes_credentials` (PASS 0.037s)
- `test_enrichment_pivot_002_credential_resolution_failed_excludes_env_var_name` (PASS 0.056s)
- `test_enrichment_pivot_002_ssrf_rejected_error_excludes_resolved_ip` (PASS 0.012s)

**Behavior observed:**
- E-INFUSE-009 (`HttpLookupFailed`): error message does NOT contain credential values (AD-017).
- E-INFUSE-010 (`CredentialResolutionFailed`): error message does NOT contain env var name.
- E-INFUSE-011 (`SsrfRejected`): error message does NOT contain resolved IP address (CWE-209).
All three Display formats verified. Error taxonomy rows at v1.88 confirmed (pre-existing).  
**Artifacts:**
- `AC-013-015-017-http-lookup-infra.gif` — VHS recording showing test PASSes
- `AC-013-015-017-http-lookup-infra.webm` — archival recording
- `full-test-run-transcript.txt` lines 5, 13, 25: PASS

---

### AC-016 — HttpLookupSource implements InfusionSource with credential AD-017 discipline

**Evidence form:** Test-execution capture (VHS recording) — wiremock mock server tests  
**Tests (wiremock in-process, no real NVD calls):**
- `test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found` (PASS 18.519s)
- `test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx` (PASS 20.177s)
- `test_enrich_single_extracts_response_path_via_wiremock` (PASS 0.039s — unit test in http_lookup.rs)
- `test_enrich_single_returns_none_on_path_not_found_via_wiremock` (PASS 0.039s — unit test)

**DTU-EXT-gated tests:**
- `test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template`  
  `[ignored: DTU-EXT-NVD-001: requires live NVD API; unit coverage in http_lookup.rs::tests::test_enrich_single_extracts_response_path_via_wiremock]`
- `test_enrichment_pivot_002_http_lookup_source_extracts_response_path_fields`  
  `[ignored: DTU-EXT-NVD-001: requires live NVD API; unit coverage in http_lookup.rs::tests::test_enrich_single_extracts_response_path_via_wiremock]`

**Behavior observed:** `HttpLookupSource::enrich_single` built via `build_http_client_with_timeout(30)`.
Returns `Ok(None)` when JSONPath not found. Returns `Err(E-INFUSE-009)` on non-2xx HTTP response.
Credential value not stored in struct fields after construction.
`event_type = "http_lookup_enrich_failed"` emitted on E-INFUSE-009.

**Deferred to T14 capstone recording:** live URL-template interpolation and response-path extraction against real NVD API. Anchor: DTU-EXT-NVD-001.  
**Artifacts:**
- `AC-016-018-020-http-lookup-ops.gif` — VHS recording showing PASSes (wiremock tests)
- `AC-016-018-020-http-lookup-ops.webm` — archival recording
- `full-test-run-transcript.txt` lines 40-41: PASS (wiremock tests)

---

### AC-017 — HttpLookupSource SSRF validation at construction time (CWE-918)

**Evidence form:** Test-execution capture (VHS recording) — unit tests  
**Tests:**
- `test_enrichment_pivot_002_ssrf_rejects_private_base_url_without_dtu_mode` (PASS 0.017s)
- `test_enrichment_pivot_002_ssrf_accepts_private_base_url_with_dtu_mode` (PASS 0.013s)

**Behavior observed:** `HttpLookupSource::new` with RFC-1918 `base_url` and `PRISM_DTU_MODE` unset
returns `Err(InfusionError::SsrfRejected { spec_path })`. The resolved IP address does NOT appear
in the error message. With `PRISM_DTU_MODE=true`, the same `base_url` is accepted (DTU override).
`event_type = "http_lookup_ssrf_rejected"` emitted on rejection.  
**Artifacts:**
- `AC-013-015-017-http-lookup-infra.gif` — VHS recording showing test PASSes
- `AC-013-015-017-http-lookup-infra.webm` — archival recording
- `full-test-run-transcript.txt` lines 24, 34: PASS

---

### AC-018 — prism-nvd-infusion crate and build recipe removed (ADR-040 D9)

**Evidence form:** Test-execution capture (VHS recording) — filesystem assertion test  
**Test:** `test_enrichment_pivot_002_nvd_plugin_crate_removed`  
**Result:** PASS (0.020s)  
**Behavior observed:** `assert!(!Path::new("crates/plugins/prism-nvd-infusion").exists())` passes.
Root `Cargo.toml` `exclude` list does not contain `prism-nvd-infusion`. `Justfile` does not contain
`build-plugin-nvd-infusion` recipe. No CI step builds `nvd-lookup.prx`.  
**Artifacts:**
- `AC-016-018-020-http-lookup-ops.gif` — VHS recording showing PASS
- `AC-016-018-020-http-lookup-ops.webm` — archival recording
- `full-test-run-transcript.txt` line 10: PASS

---

### AC-019 — Val-lift fix covered by unit tests exercising the real production path (F-001 CRIT)

**Evidence form:** Test-execution capture (VHS recording) — unit tests driving production code path  
**Tests:**
- `test_enrichment_pivot_002_val_lift_fix_option_some_returns_json_value` (PASS 0.018s)
- `test_enrichment_pivot_002_val_lift_fix_option_none_returns_ok_none` (PASS 0.020s)
- `test_enrichment_pivot_002_val_lift_fix_unexpected_val_returns_enrich_call_failed` (PASS 0.018s)

**Behavior observed (F-003 rigor: real `PluginRuntime::enrich_single`, no reimplementation):**
- `Val::Option(Some(Box<Val::String("{}"))))` → `Ok(Some(serde_json::Value::Object(_)))` (not `Ok(None)`)
- `Val::Option(None)` → `Ok(None)` (no enrichment data — not an error)
- `Val::String("not-json")` in unexpected position → `Err(PluginError::EnrichCallFailed { .. })`
  with `event_type = "plugin_enrich_json_parse_error"` emitted  
**Artifacts:**
- `AC-003-019-threatintel-val-lift.gif` — VHS recording showing all 3 sub-cases PASS
- `AC-003-019-threatintel-val-lift.webm` — archival recording
- `AC-003-019-threatintel-val-lift.tape` — VHS script source
- `full-test-run-transcript.txt` lines 36-38: PASS

---

### AC-020 — spawn_blocking wraps the real InfusionAsyncUdf invoke path (F-004 rigor)

**Evidence form:** Test-execution capture (VHS recording) — tokio runtime test on real code path  
**Test:** `test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking` (prism-query)  
**Result:** PASS (0.363s) — also PASS in prism-spec-engine (0.077s)  
**Behavior observed (F-004 rigor: real `InfusionAsyncUdf::invoke_with_args` path, not a helper assertion):**
The test calls `InfusionAsyncUdf::invoke_with_args` from within a `tokio::test` runtime and
demonstrates the async path does NOT block the tokio runtime. The synchronous WASM call is wrapped
in `tokio::task::spawn_blocking`. Test would deadlock or time out if the call were blocking directly.  
**Artifacts:**
- `AC-016-018-020-http-lookup-ops.gif` — VHS recording showing PASS
- `AC-016-018-020-http-lookup-ops.webm` — archival recording
- `full-test-run-transcript.txt` line 39: PASS (prism-spec-engine)
- `full-test-run-transcript.txt` prism-query section: PASS

---

### AC-021 — SAP-1: BC-2.16.002 catalog rows for 4 new event_types

**Evidence form:** Adversary verification (per story spec — not a Rust unit test)  
**Observed:** SAP-1 probe (`rg 'event_type\s*=' crates/ --type rust`) confirms all 4 event_types
have corresponding BC-2.16.002 catalog rows:
- `plugin_enrich_json_parse_error` — fields: `plugin_id`, `error`
- `plugin_enrich_unexpected_val` — fields: `plugin_id`
- `http_lookup_enrich_failed` — fields: `infusion_id`, `spec_path`, `status_code`
- `http_lookup_ssrf_rejected` — fields: `infusion_id`, `spec_path`

Added in the same commit as emission sites (PG-LP11-001 precedent). Verified during LOCAL
adversarial cascade (3 CLEAN passes, branch HEAD b436104d).  
**Artifacts:**
- `full-test-run-transcript.txt` — all 41 tests pass including those that emit these event_types
- SAP-1 probe result captured in LOCAL adversary cascade reports (`.factory/` branch)

---

## Deferred to T14 Capstone Recording

The following two test scenarios require a deployed NVD DTU (live external service) and are
`#[ignore]`'d under DTU-EXT-NVD-001. Unit-level coverage of the same production code path
is provided by wiremock-backed tests.

| Ignored Test | Gating Reason | Unit Coverage |
|--------------|---------------|---------------|
| `test_enrichment_pivot_002_http_lookup_source_enrich_single_calls_url_template` | DTU-EXT-NVD-001: requires live NVD API (services.nvd.nist.gov — rate-limited) | `http_lookup.rs::tests::test_enrich_single_extracts_response_path_via_wiremock` |
| `test_enrichment_pivot_002_http_lookup_source_extracts_response_path_fields` | DTU-EXT-NVD-001: requires live NVD API | `http_lookup.rs::tests::test_enrich_single_extracts_response_path_via_wiremock` |

**T14 anchor:** Live end-to-end NVD HttpLookup recording is scoped to the T14 capstone demo
(S-DEMO-ENRICHMENT-PIVOT-003 integration, which deploys the NVD DTU clone and exercises
`| enrich nvd(device_cves_first)` against a real HTTP endpoint).

---

## POL-10 Compliance

All evidence files are under `docs/demo-evidence/S-DEMO-ENRICHMENT-PIVOT-002/` (story-scoped subfolder).
No files were placed directly at `docs/demo-evidence/*.md`. Verified: `ls docs/demo-evidence/` shows
no `.md` files at the flat level from this story.

---

## Full Test Run Summary

```
cargo nextest run -p prism-spec-engine -E 'test(enrichment_pivot_002)' --no-fail-fast
Summary [20.178s] 41 tests run: 41 passed, 704 skipped

cargo nextest run -p prism-query -E 'test(enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking)'
Summary [0.364s] 1 test run: 1 passed, 1015 skipped

cargo nextest run -p prism-spec-engine -E 'test(test_enrich_single)'
Summary [0.040s] 2 tests run: 2 passed, 743 skipped (wiremock unit tests covering DTU-EXT-gated tests)
```

Total tests run for this story: 44 (41 + 1 prism-query + 2 wiremock unit) — all PASS.
Ignored tests: 2 (DTU-EXT-NVD-001 gated, with T14 anchor and unit coverage).
