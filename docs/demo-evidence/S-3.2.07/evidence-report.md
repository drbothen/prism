# Demo Evidence Report — S-3.2.07

**Story:** S-3.2.07 — prism-dtu-jira: Shared-mode OrgId ingress tagging
**Behavioral Contracts:** BC-3.2.004, BC-3.2.005
**Implementation commit:** ce742ea5
**Recorded:** 2026-04-29
**Tool:** VHS 0.10.0

---

## Coverage Map

| Recording | Acceptance Criterion | BC Anchor | Result |
|-----------|---------------------|-----------|--------|
| AC-001-all-8-tests-green | All 8 org_tagging tests GREEN — covers AC-001 through AC-006 | BC-3.2.004, BC-3.2.005 | PASS |
| AC-002-concurrent-orgid | AC-003 — Concurrent issues from different orgs are distinguished | BC-3.2.004 postcondition 4 | PASS |

---

## AC-001 — All 8 org_tagging tests GREEN

**Command:** `cargo test -p prism-dtu-jira --features dtu --test org_tagging`

**Demonstrates:**
- AC-001: OrgId in Jira issue record (BC-3.2.004 postcondition 1) — `test_BC_3_2_004_ac001_org_id_in_issue_record`
- AC-002: OrgId absent from issue key and HTTP routing (BC-3.2.004 postcondition 2) — `test_BC_3_2_004_ac002_org_id_absent_from_routing`, `test_BC_3_2_004_ac002_issue_key_not_org_scoped`
- AC-003: Concurrent issues distinguished (BC-3.2.004 postcondition 4) — `test_BC_3_2_004_ac003_concurrent_issues_distinguished`
- AC-004: Mode metadata absent from query results (BC-3.2.004 postcondition 5) — `test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results`
- AC-005: OrgId UUID is opaque (BC-3.2.004 invariant 1) — `test_BC_3_2_005_ac005_jira_dtu_mode_is_shared`
- AC-006: DtuMode::Shared immutable (BC-3.2.005 postconditions 1 and 4) — `test_BC_3_2_005_ac005_mode_immutable_after_startup`, `test_BC_3_2_005_ac006_invalid_mode_string_rejected`

**Test result:** `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.05s`

**Artifacts:**
- `AC-001-all-8-tests-green.gif`
- `AC-001-all-8-tests-green.webm`
- `AC-001-all-8-tests-green.tape`

---

## AC-002 — Concurrent OrgId tagging

**Command:** `cargo test -p prism-dtu-jira --features dtu --test org_tagging test_BC_3_2_004_ac003_concurrent_issues_distinguished -- --nocapture`

**Demonstrates:**
- AC-003: org_A and org_B create issues concurrently; each IssueRecord contains its sender's OrgId UUID independently (BC-3.2.004 postcondition 4).
- Issue key is MSSP-scoped and does not encode OrgId — concurrent access does not conflate org attribution.

**Test result:** `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 5.07s`

**Artifacts:**
- `AC-002-concurrent-orgid.gif`
- `AC-002-concurrent-orgid.webm`
- `AC-002-concurrent-orgid.tape`

---

## Error Path Coverage

Both demos use `cargo test` which exercises the full error-path suite embedded in the test module:

| Test name | Error path covered |
|-----------|--------------------|
| `test_BC_3_2_004_ac002_org_id_absent_from_routing` | Issue key must NOT contain OrgId UUID |
| `test_BC_3_2_004_ac002_issue_key_not_org_scoped` | Issue key must NOT contain org slug |
| `test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results` | Query results must NOT expose mode fields |
| `test_BC_3_2_005_ac006_invalid_mode_string_rejected` | Invalid DtuMode string at startup is rejected |

---

## Summary

All 6 acceptance criteria (AC-001 through AC-006) are demonstrated. All 8 tests green. Both success and error paths are covered via the test suite visible in the terminal recordings.
