# Evidence Report — S-DEMO-ARMIS-AQL-001

**Story:** S-DEMO-ARMIS-AQL-001 v1.8
**Title:** prism-dtu-armis + armis.sensor.toml: AQL Search Endpoint Fidelity — Add GET /api/v1/search Route; Update TOML path_template to /api/v1/search; Parity Test AQL Push-Down (ADR-031 §D8-a)
**Branch:** feature/S-DEMO-ARMIS-AQL-001
**BCs:** BC-2.16.013 v1.22 (ACTIVE), BC-2.16.002 (SAP-1 catalog invariant)
**Product type:** Backend DTU route (CLI/Rust — VHS not applicable; test execution output captured)
**Evidence directory:** docs/demo-evidence/S-DEMO-ARMIS-AQL-001/
**HEAD at evidence capture:** 9243a0d3

---

## AC Coverage Summary

| AC | Description | Status | Evidence Artifact | Red Gate Tests |
|----|-------------|--------|-------------------|----------------|
| AC-001 | GET /api/v1/search registered in build_router; in:devices AQL returns 200 with valid Bearer; missing Bearer returns 403 | PASS | AC-001-EC-004-search-route-registered-auth-enforced.txt | test_armis_aql_search_route_registered_returns_200_for_device_aql, test_armis_aql_search_returns_403_without_bearer |
| AC-002 | /api/v1/search with in:devices AQL returns DeviceRecord objects in data.results; AQL captured in GET /dtu/aql-log (R-DTU-002) | PASS | AC-002-devices-aql-returns-records-aql-captured.txt | test_armis_aql_search_devices_aql_returns_device_records, test_armis_aql_search_aql_captured_in_aql_log |
| AC-003 | /api/v1/search with in:alerts AQL returns AlertRecord objects in data.results | PASS | AC-003-EC-001-alerts-aql-returns-records-default-fallback.txt | test_armis_aql_search_alerts_aql_returns_alert_records |
| AC-004 | armis.sensor.toml devices + alerts steps: path_template=/api/v1/search, response_path=$.data.results; SAP-2 column parity gate | PASS | AC-004-toml-path-template-response-path-sap2-parity.txt | test_armis_aql_search_toml_path_template_updated, test_armis_aql_search_toml_response_path_updated, test_armis_aql_search_dtu_toml_column_parity |
| AC-005 | Pipeline round-trip parity: AQL string prism sends matches DTU-received AQL string (R-DTU-002 end-to-end) | PASS | AC-005-aql-roundtrip-pipeline-parity.txt | test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline, test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline |
| AC-006 | Direct endpoints /api/v1/devices + /api/v1/alerts remain accessible; no regressions (130/130 pass) | PASS | AC-006-back-compat-direct-endpoints.txt | Pre-existing suite (130/130 PASS; no regressions) |
| AC-007 | Zero uncatalogued tracing event_type emissions in prism-dtu-armis/src/ (SAP-1) | PASS | AC-007-no-uncatalogued-event-type-sap1.txt | SAP-1 grep: 0 event_type sites in production source |

**AC coverage: 7/7 — COMPLETE**

---

## Edge Case Coverage

| EC | Description | Status | Evidence Artifact |
|----|-------------|--------|-------------------|
| EC-001 | GET /api/v1/search with no aql param → returns devices (safe default) | PASS | AC-003-EC-001-alerts-aql-returns-records-default-fallback.txt |
| EC-004 | Missing Authorization header → HTTP 403 (Armis auth model; not 401) | PASS | AC-001-EC-004-search-route-registered-auth-enforced.txt |

---

## Tenant-Isolation Guard (F-P2-MED-001 / W3-FIX-SEC-001)

GET /api/v1/search enforces X-Org-Id org-isolation guard (same as all other org-scoped endpoints).
3-cell matrix tests covering /api/v1/search:

| Test | Cell | Description | Status | Evidence Artifact |
|------|------|-------------|--------|-------------------|
| `test_W3_FIX_SEC_001_search_real_org_mismatched_header_returns_401` | A | Non-nil-org clone + mismatched X-Org-Id → HTTP 401 | PASS | AC-001-X-Org-Id-search-tenant-isolation-guard.txt |
| `test_W3_FIX_SEC_001_search_real_org_absent_header_returns_401` | B | Non-nil-org clone + ABSENT X-Org-Id → HTTP 401 | PASS | AC-001-X-Org-Id-search-tenant-isolation-guard.txt |
| `test_W3_FIX_SEC_001_search_real_org_matching_header_returns_200` | C | Nil-org clone + no X-Org-Id header → HTTP 200 (backward-compat) | PASS | AC-001-X-Org-Id-search-tenant-isolation-guard.txt |

**3/3 search org-isolation tests pass — F-P2-MED-001 guard active on /api/v1/search.**

---

## Red Gate Test Table

| Test Name | AC | File | Result |
|-----------|----|----|--------|
| `test_armis_aql_search_route_registered_returns_200_for_device_aql` | AC-001 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_returns_403_without_bearer` | AC-001 / EC-004 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_devices_aql_returns_device_records` | AC-002 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_aql_captured_in_aql_log` | AC-002 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_alerts_aql_returns_alert_records` | AC-003 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_no_aql_defaults_to_devices` | AC-001 / EC-001 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_toml_path_template_updated` | AC-004 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_toml_response_path_updated` | AC-004 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_armis_aql_search_dtu_toml_column_parity` | AC-004 / SAP-2 | s_demo_armis_aql_001_red_gate.rs | PASS |
| `test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline` | AC-005 | parity/armis.rs | PASS |
| `test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline` | AC-005 | parity/armis.rs | PASS |

**11/11 Red Gate tests PASS (9 in s_demo_armis_aql_001_red_gate.rs + 2 AC-005 parity tests in parity/armis.rs)**

---

## Deferred Tests (SID-1 compliant stubs)

| Test Name | Status | Blocking Dependency |
|-----------|--------|---------------------|
| `test_BC_2_16_013_dtu_parity_armis` | `#[ignore]` | Requires ${env.ARMIS_INSTANCE_URL} env-var resolution; ungated after **S-SPEC-ENV-VAR-001** merges (story frontmatter D-914 gate note) |
| `test_BC_2_16_013_dtu_parity_armis_timestamp_fallback_pass_by_convention` | `#[ignore]` | Same S-SPEC-ENV-VAR-001 gate |

SID-1 compliance: both ignore annotations cite the blocking story ID (S-SPEC-ENV-VAR-001) verbatim.
AC-005 pipeline behavior is fully covered by the two un-ignored roundtrip tests.

---

## Full Suite Result

```
cargo nextest run -p prism-dtu-armis --features dtu --no-fail-fast
130 tests run: 130 passed, 0 skipped

cargo nextest run -p prism-spec-engine --test parity_armis --no-fail-fast
5 tests run: 5 passed (1 leaky), 2 skipped (#[ignore] — S-SPEC-ENV-VAR-001 gate)
```

Total non-ignored: 135 tests PASS, 0 failed. See `full-suite-run.txt` for verbatim output.

---

## Gap-AR-001 / DTU-EXT-003/004 Closure Confirmation

Pre-story state: `prism-dtu-armis::build_router()` had no handler for `GET /api/v1/search`.
Pipeline used non-production direct endpoints (`/api/v1/devices`, `/api/v1/alerts`) while
the real Armis Centrix production poller uses `GET /api/v1/search?aql=<query>` exclusively.

Post-story state: `get_search` handler registered at `GET /api/v1/search`;
AQL discrimination (`in:devices` → device fixture, `in:alerts` → alert fixture);
AQL captured verbatim via `state.capture_aql()` (R-DTU-002);
Response envelope `{"data": {"results": [...], "total": N}}` per real Armis API shape;
`armis.sensor.toml` devices + alerts steps updated: `path_template = "/api/v1/search?aql=..."`,
`response_path = "$.data.results"`; DTU-EXT-003 and DTU-EXT-004 comments marked CLOSED.
Direct endpoints (`/api/v1/devices`, `/api/v1/alerts`) retained for backward compatibility.
X-Org-Id tenant-isolation guard applied to `/api/v1/search` (F-P2-MED-001 closed).

---

## Artifact Index

| File | Contents |
|------|----------|
| `AC-001-EC-004-search-route-registered-auth-enforced.txt` | test_armis_aql_search_route_registered_returns_200_for_device_aql + test_armis_aql_search_returns_403_without_bearer (AC-001, EC-004) |
| `AC-002-devices-aql-returns-records-aql-captured.txt` | test_armis_aql_search_devices_aql_returns_device_records + test_armis_aql_search_aql_captured_in_aql_log (AC-002, R-DTU-002) |
| `AC-003-EC-001-alerts-aql-returns-records-default-fallback.txt` | test_armis_aql_search_alerts_aql_returns_alert_records + test_armis_aql_search_no_aql_defaults_to_devices (AC-003, EC-001) |
| `AC-004-toml-path-template-response-path-sap2-parity.txt` | 3 TOML/SAP-2 tests: toml_path_template_updated, toml_response_path_updated, dtu_toml_column_parity (AC-004, SAP-2) |
| `AC-005-aql-roundtrip-pipeline-parity.txt` | test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline + _alerts_pipeline (AC-005, R-DTU-002) |
| `AC-006-back-compat-direct-endpoints.txt` | Full 130-test suite run showing all pre-existing tests pass (AC-006, no regressions) |
| `AC-007-no-uncatalogued-event-type-sap1.txt` | SAP-1 grep result: zero event_type emissions in prism-dtu-armis/src/ (AC-007) |
| `AC-001-X-Org-Id-search-tenant-isolation-guard.txt` | 3-cell W3-FIX-SEC-001 org-isolation tests on /api/v1/search (F-P2-MED-001) |
| `full-suite-run.txt` | Verbatim output: 130-test prism-dtu-armis run + 5-test parity_armis run (135 pass, 0 fail, 2 skipped/ignored) |
| `evidence-report.md` | This file |
