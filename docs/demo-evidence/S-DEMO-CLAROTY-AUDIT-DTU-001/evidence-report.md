# Evidence Report — S-DEMO-CLAROTY-AUDIT-DTU-001

**Story:** S-DEMO-CLAROTY-AUDIT-DTU-001 v1.5
**Title:** prism-dtu-claroty: Add /api/v1/audit_log/get route for Claroty Audit Log Fidelity (closes Gap-CL-006 / DTU=true-DTU)
**Branch:** feature/S-DEMO-CLAROTY-AUDIT-DTU-001
**HEAD at recording:** 7f3584b6
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

**AC coverage: 6/6 — COMPLETE**

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

## Red Gate Test Table

| Test Name | Type | ACs Gated | Result |
|-----------|------|-----------|--------|
| `test_BC_2_16_013_claroty_audit_logs_dtu_route_returns_synthetic_entries` | Unit (HTTP) | AC-001, AC-003, AC-004 | PASS |
| `test_BC_2_16_013_claroty_audit_logs_dtu_auth_enforced` | Unit (HTTP) | AC-002, EC-001, EC-002, EC-003 | PASS |
| `test_BC_2_16_013_claroty_audit_logs_dtu_column_parity` | Unit (SAP-2 struct + HTTP) | AC-005 | PASS |
| `claroty_dtu_fidelity` (fidelity_validator.rs) | Integration (FidelityValidator) | AC-001, AC-006 | PASS |

---

## Full Suite Result

```
cargo nextest run -p prism-dtu-claroty
4 tests run: 4 passed, 0 skipped, 0 failed

cargo nextest run -p prism-dtu-claroty --test fidelity_validator --features dtu
1 test run: 1 passed, 0 skipped, 0 failed
```

See `full-suite-run.txt` for verbatim output.

---

## Gap-CL-006 Closure Confirmation

Pre-story state: `ClarotyClone::build_router()` had no handler for `POST /api/v1/audit_log/get`
→ pipeline received HTTP 404 for the `fetch_audit_logs` step declared in `claroty.sensor.toml`.

Post-story state: `list_audit_logs` handler registered at `/api/v1/audit_log/get`;
fixture `fixtures/audit-log.json` (5 synthetic entries) served under `{"audit_log": [...], "total": N}`;
auth enforced via `check_bearer_auth` before fixture load.

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
| `full-suite-run.txt` | Full `cargo nextest run -p prism-dtu-claroty` run output (all 4 tests) |
| `evidence-report.md` | This file |
