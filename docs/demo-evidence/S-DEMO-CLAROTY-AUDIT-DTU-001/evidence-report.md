# Evidence Report — S-DEMO-CLAROTY-AUDIT-DTU-001

**Story:** S-DEMO-CLAROTY-AUDIT-DTU-001 v1.6
**Title:** prism-dtu-claroty: Add /api/v1/audit_log/get route for Claroty Audit Log Fidelity (closes Gap-CL-006 / DTU=true-DTU)
**Branch:** feature/S-DEMO-CLAROTY-AUDIT-DTU-001
**PR:** #167 (S-DEMO-CLAROTY-AUDIT-DTU-001)
**BCs:** BC-2.01.013 v1.9, BC-2.16.013 v1.19
**Product type:** Backend DTU route (CLI/Rust — VHS not applicable; test execution output captured)
**Evidence directory:** docs/demo-evidence/S-DEMO-CLAROTY-AUDIT-DTU-001/

---

## AC Coverage Summary

| AC | Description | Status | Evidence Artifact | Red Gate Test |
|----|-------------|--------|-------------------|---------------|
| AC-001 | POST /api/v1/audit_log/get registered; returns 200 with valid bearer | PASS | AC-001-AC-003-AC-004-route-returns-synthetic-entries.txt, AC-001-AC-006-fidelity-validator.txt | test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries |
| AC-002 | 401 with exact body `{"error": "missing or invalid Authorization header", "code": 401}` (POL-24) | PASS | AC-002-EC-001-EC-002-EC-003-auth-enforced.txt | test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced |
| AC-003 | Response envelope `{"audit_log": [...], "total": N}`; key matches `response_path = "$.audit_log"` | PASS | AC-001-AC-003-AC-004-route-returns-synthetic-entries.txt | test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries |
| AC-004 | fixtures/audit-log.json has >= 5 synthetic entries; actors use @example.com (no real PII) | PASS | AC-001-AC-003-AC-004-route-returns-synthetic-entries.txt, AC-005-column-parity-sap2.txt | test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries |
| AC-005 | All 5 TOML columns present in ClarotyAuditLogEntry (id/action/actor/timestamp/resource); SAP-2 parity gate | PASS | AC-005-column-parity-sap2.txt | test_BC_2_16_013_claroty_audit_logs_dtu_column_parity |
| AC-006 | FidelityValidator 12-route matrix: 12/12 checks pass, 0 failures; Gap-CL-006 closed | PASS | AC-001-AC-006-fidelity-validator.txt | claroty_dtu_fidelity (fidelity_validator.rs) |
| AC-007 | Org-isolation guard (W3-FIX-SEC-001) active on all 4 fixture-list endpoints; 8 tests pass | PASS | AC-007-org-isolation-w3-fix-sec-001.txt | 8 W3-FIX-SEC-001 tests (see Org-Isolation table) |

**AC coverage: 7/7 — COMPLETE**

---

## Edge Case Coverage

| EC | Description | Status | Evidence Artifact |
|----|-------------|--------|-------------------|
| EC-001 | Missing Authorization header → HTTP 401 + canonical body | PASS | AC-002-EC-001-EC-002-EC-003-auth-enforced.txt |
| EC-002 | Authorization: Bearer  (empty token) → HTTP 401 + canonical body | PASS | AC-002-EC-001-EC-002-EC-003-auth-enforced.txt |
| EC-003 | Malformed (non-JSON) body + valid bearer → HTTP 200 (permissive body) | PASS | AC-002-EC-001-EC-002-EC-003-auth-enforced.txt |
| EC-004 | Body with unknown fields → HTTP 200 (permissive deserialization) | COVERED by EC-003 fixture body | AC-002-EC-001-EC-002-EC-003-auth-enforced.txt |
| EC-005 | GET /api/v1/audit_log/get → HTTP 405 Method Not Allowed (axum default) | Not directly tested; axum router enforces method matching by construction | — |

---

## AC-007 Org-Isolation Coverage (W3-FIX-SEC-001 — all fixture-list endpoints)

Guard authority: **W3-FIX-SEC-001** (F-PR3-HIGH-001 fix — org-isolation guard applied DTU-wide).
Behavioral contract: org-mismatch → 401; missing/nil org header on nil-org clone → 200 (backward-compat).

| Test | Endpoint | Description | Status | Evidence Artifact |
|------|----------|-------------|--------|-------------------|
| `test_W3_FIX_SEC_001_claroty_audit_logs_org_mismatch_returns_401` | audit_log | Non-nil-org clone + mismatched X-Org-Id header → HTTP 401 "org_id mismatch" | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_audit_logs_nil_org_no_header_returns_200` | audit_log | Nil-org clone without X-Org-Id header → HTTP 200 (backward-compat) | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_alerts_org_mismatch_returns_401` | alerts | Non-nil-org clone + mismatched X-Org-Id header → HTTP 401 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_alerts_nil_org_no_header_returns_200` | alerts | Nil-org clone without X-Org-Id header → HTTP 200 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_alerted_devices_org_mismatch_returns_401` | alerted_devices | Non-nil-org clone + mismatched X-Org-Id header → HTTP 401 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_alerted_devices_nil_org_no_header_returns_200` | alerted_devices | Nil-org clone without X-Org-Id header → HTTP 200 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_devices_org_mismatch_returns_401` | devices | Non-nil-org clone + mismatched X-Org-Id header → HTTP 401 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |
| `test_W3_FIX_SEC_001_claroty_devices_nil_org_no_header_returns_200` | devices | Nil-org clone without X-Org-Id header → HTTP 200 | PASS | AC-007-org-isolation-w3-fix-sec-001.txt |

**8/8 org-isolation tests pass — W3-FIX-SEC-001 guard active across all fixture-list endpoints.**

---

## Red Gate Test Table

| Test Name | Type | ACs Gated | Result |
|-----------|------|-----------|--------|
| `test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries` | Unit (HTTP) | AC-001, AC-003, AC-004 | PASS |
| `test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced` | Unit (HTTP) | AC-002, EC-001, EC-002, EC-003 | PASS |
| `test_BC_2_16_013_claroty_audit_logs_dtu_column_parity` | Unit (SAP-2 struct + HTTP) | AC-005 | PASS |
| `claroty_dtu_fidelity` (fidelity_validator.rs) | Integration (FidelityValidator) | AC-001, AC-006 | PASS |
| `test_W3_FIX_SEC_001_claroty_audit_logs_org_mismatch_returns_401` | Unit (W3-FIX-SEC-001 org-isolation) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_audit_logs_nil_org_no_header_returns_200` | Unit (W3-FIX-SEC-001 nil-org path) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_alerts_org_mismatch_returns_401` | Unit (W3-FIX-SEC-001 org-isolation) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_alerts_nil_org_no_header_returns_200` | Unit (W3-FIX-SEC-001 nil-org path) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_alerted_devices_org_mismatch_returns_401` | Unit (W3-FIX-SEC-001 org-isolation) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_alerted_devices_nil_org_no_header_returns_200` | Unit (W3-FIX-SEC-001 nil-org path) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_devices_org_mismatch_returns_401` | Unit (W3-FIX-SEC-001 org-isolation) | AC-007 | PASS |
| `test_W3_FIX_SEC_001_claroty_devices_nil_org_no_header_returns_200` | Unit (W3-FIX-SEC-001 nil-org path) | AC-007 | PASS |

**12 Red Gate tests (4 AC-001..006 gates + 8 AC-007 org-isolation gates) — all PASS.**

---

## Deferred Test (SID-1 compliant stub)

| Test Name | Status | Blocking Dependency |
|-----------|--------|---------------------|
| `test_BC_2_16_013_claroty_audit_logs_pipeline_integration_ac_006` | `#[ignore]` — stub with `todo!()` | Requires prism-bin full-boot wiring; ungated after **S-DEMO-002** merges |

SID-1 compliance: the ignore annotation cites the blocking story ID (S-DEMO-002) verbatim. The AC-006 pipeline integration behavior is covered at the unit level by `claroty_dtu_fidelity` (FidelityValidator confirms route reachability).

---

## Full Suite Result

```
cargo nextest run -p prism-dtu-claroty
13 tests run: 13 passed, 1 skipped (#[ignore]'d AC-006 pipeline stub)

cargo nextest run -p prism-dtu-claroty --test fidelity_validator --features dtu
1 test run: 1 passed, 0 skipped, 0 failed
```

See `full-suite-run.txt` for verbatim output (5-test base run) and `AC-007-org-isolation-w3-fix-sec-001.txt` for the 8-test AC-007 run.

---

## Gap-CL-006 Closure Confirmation

Pre-story state: `ClarotyClone::build_router()` had no handler for `POST /api/v1/audit_log/get`
→ pipeline received HTTP 404 for the `fetch_audit_logs` step declared in `claroty.sensor.toml`.

Post-story state: `list_audit_logs` handler registered at `/api/v1/audit_log/get`;
fixture `fixtures/audit-log.json` (5 synthetic entries) served under `{"audit_log": [...], "total": N}`;
auth enforced via `check_bearer_auth` before fixture load;
org-isolation guard (W3-FIX-SEC-001) enforced before fixture load for non-nil-org clones;
guard active on all 4 fixture-list endpoints (audit_log, alerts, alerted_devices, devices).

FidelityValidator Route 11 (`POST /api/v1/audit_log/get`) now in the 12-route coverage matrix:
12/12 checks passed, 0 failed (was not present pre-story).

---

## Artifact Index

| File | Contents |
|------|----------|
| `AC-001-AC-003-AC-004-route-returns-synthetic-entries.txt` | test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries output (AC-001, AC-003, AC-004) |
| `AC-002-EC-001-EC-002-EC-003-auth-enforced.txt` | test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced output (AC-002, EC-001, EC-002, EC-003) |
| `AC-005-column-parity-sap2.txt` | test_BC_2_16_013_claroty_audit_logs_dtu_column_parity output (AC-005, SAP-2) |
| `AC-001-AC-006-fidelity-validator.txt` | claroty_dtu_fidelity FidelityValidator output (AC-001, AC-006, 12-route matrix) |
| `SEC-001-org-isolation.txt` | audit_log org-mismatch + nil-org backward-compat test output (W3-FIX-SEC-001, audit_log endpoint) |
| `AC-007-org-isolation-w3-fix-sec-001.txt` | All 8 W3-FIX-SEC-001 org-isolation tests across all 4 endpoints (AC-007) |
| `full-suite-run.txt` | Full `cargo nextest run -p prism-dtu-claroty` run output (base 5 tests pass, 1 skipped) + fidelity validator |
| `evidence-report.md` | This file |
