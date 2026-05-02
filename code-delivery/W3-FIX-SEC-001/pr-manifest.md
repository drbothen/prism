# PR Manifest — W3-FIX-SEC-001

**Story:** DTU clones: bind OrgId to clone instance — reject mismatched X-Org-Id header
**Epic:** E-3.5 — Wave 3 Multi-Tenant Security Hardening
**Generated:** 2026-05-01T22:40:00-05:00

---

## Merge Record

| Field | Value |
|-------|-------|
| PR Number | #113 |
| PR URL | https://github.com/drbothen/prism/pull/113 |
| Branch | feature/W3-FIX-SEC-001 |
| HEAD SHA at merge | b412f547 |
| Merge commit SHA on develop | 59803de362ce2f3e5c3ddf6be6fff3079f8aa6f6 |
| Merge strategy | squash |
| Branch deleted | yes |
| Base branch | develop |
| Merge timestamp | 2026-05-01T22:39:56-05:00 |

---

## CI Run IDs (Final Passing Runs)

| Run ID | Name | Status | Conclusion |
|--------|------|--------|------------|
| 25241363347 | Crate Layout | completed | success |
| 25241363346 | CI | completed | success |
| 25241362576 | Crate Layout | completed | success |
| 25241362586 | CI | completed | success |
| 25237487643 | CI | completed | success |

**CI Gate:** 26/26 checks pass, 0 fail, 0 pending at merge time.

---

## Reviewer Audit Trail

### Security Review (Step 4)

**Reviewer:** vsdd-factory:security-review
**Verdict:** APPROVED after HIGH-001 fix
**Date:** 2026-05-01

| Finding ID | Severity | Description | Status |
|------------|----------|-------------|--------|
| HIGH-001 | HIGH | CrowdStrike write/read endpoints bypass validate_org_id (hosts.rs:294, writes.rs:99, writes.rs:237) | RESOLVED — commit e8ca86ae |

### PR Review Convergence (Step 5)

**Reviewer:** vsdd-factory:pr-review-triage
**Converged:** Cycle 2
**Date:** 2026-05-01

| Cycle | Findings | Blocking | Non-Blocking | Fixed | Remaining |
|-------|----------|----------|--------------|-------|-----------|
| 1 | 2 | 1 | 1 | 0 | 1 blocking |
| 2 | 0 | 0 | 0 | 2 | 0 — APPROVE |

**Verdict:** APPROVE (Cycle 2)

| Finding ID | Severity | Description | Status |
|------------|----------|-------------|--------|
| REVIEW-001 | BLOCKING | Claroty tags.rs — add_tag and remove_tag use extract_org_id without validate_org_id guard (tags.rs:71, tags.rs:101) | RESOLVED — commit 17a881c4 |
| REVIEW-002 | NON-BLOCKING | Test stale assertion messages reference "not yet wired" phrasing | ACCEPTED — cosmetic only, no re-review required |

---

## Gate Findings Closed

| Finding ID | Severity | Source | Class | Status |
|------------|----------|--------|-------|--------|
| SEC-001 | HIGH | gate-step-d wave-3-multi-tenant | CWE-287/CWE-639, OWASP A01 — DTU clones accept arbitrary X-Org-Id from wire | CLOSED — validate_org_id wired to all 4 DTU clones |
| HIGH-001 | HIGH | security-reviewer step 4 | CWE-639 — CrowdStrike hosts.rs:294, writes.rs:99, writes.rs:237 bypass validate_org_id | CLOSED — commit e8ca86ae |
| REVIEW-001 | BLOCKING | pr-review-triage cycle 1 | Claroty tags.rs add_tag/remove_tag no validate_org_id guard | CLOSED — commit 17a881c4 |

---

## Follow-Up Items (Not Blocking Merge)

| ID | Description | Tracking |
|----|-------------|---------|
| TD-W3-TIMING-001 | Armis validate-on-presence model allows ~150ms timing window before clone is bound (instance_org_id nil at startup) — no exploit possible in current harness but may warrant hardening | BC-3.5.001 budget relaxation discussion deferred to wave gate |
| REVIEW-002 | Test assertion messages in claroty and crowdstrike x_org_id_auth.rs still reference "not yet wired" phrasing | Low-priority cosmetic — acceptable to leave |

---

## AC Coverage at Merge

| AC | Status | Evidence |
|----|--------|---------|
| AC-001 — X-Org-Id validated against bearer token | PASS | test_AC_001_x_org_id_validated_against_bearer_token in all 4 crates |
| AC-002 — Cross-org credential returns 401 | PASS | test_AC_002_cross_org_credential_returns_401 + body test in all 4 crates |
| AC-003 — Auth model divergence documented | PASS (per-clone model) | Model A: 401; Model B (Cyberint): 200 default; validate-on-presence (Armis): 200 |
| AC-004 — 30 new tests added | PASS | 30 x_org_id_auth tests across 4 crates |
| AC-005 — Cross-org header rejected | PASS | test_cross_org_header_rejected in all 4 crates |
| AC-006 — Zero regressions in multi-tenant suite | PASS | pre-existing multi_tenant suite: 0 regressions |

---

## 9-Step Gate Log

| Step | Name | Status | Evidence |
|------|------|--------|---------|
| 1 | populate-pr-description | ok | pr-description.md written, all sections populated |
| 2 | verify-demo-evidence | ok | Mermaid trace diagrams embedded; N/A for security-fix story |
| 3 | create-pr | ok | PR #113 created |
| 4 | security-review | ok (after fix) | HIGH-001 resolved in commit e8ca86ae |
| 5 | review-convergence | ok | APPROVE in cycle 2; REVIEW-001 resolved in commit 17a881c4 |
| 6 | wait-for-ci | ok | 26/26 checks pass |
| 7 | dependency-check | ok | depends_on=[] confirmed |
| 8 | execute-merge | ok | SHA=59803de362ce2f3e5c3ddf6be6fff3079f8aa6f6 |
| 9 | post-merge | ok | manifest written; develop HEAD confirmed |

---

_PR Manager: vsdd-factory:pr-manager | Merge authorized by orchestrator (AUTHORIZE_MERGE=yes)_
