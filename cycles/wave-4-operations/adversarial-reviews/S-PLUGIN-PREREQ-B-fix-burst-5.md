---
document_type: adversarial-review
level: LOCAL
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-05-11T23:45:00
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
pass: 5
previous_review: S-PLUGIN-PREREQ-B-fix-burst-4.md
target_artifact: S-PLUGIN-PREREQ-B
fix_burst_for_pass: 5
target_sha: 2fe7068c
base_sha: e19372f4
verdict: CLOSED
finding_summary_closed: { critical: 0, high: 0, medium: 0, low: 1, design_decision: 1, bc_amendment: 1 }
prior_passes: fix-burst-4 already closed 3 actionable + filed 5 TDs; F-LP5-LOW-003 design surfaced to orchestrator; user authorized Option A (eager-token + BC v1.5)
---

# Adversarial Review: S-PLUGIN-PREREQ-B fix-burst-5 Closure (Pass 5)

## Finding ID Convention

Finding IDs use the format: `F-LP<PASS>-<SEV>-<SEQ>` for LOCAL pass findings.

- `F`: Fixed prefix
- `LP<PASS>`: LOCAL pass number (e.g., `LP5`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass >= 2 only)

This is a fix-burst closure record, not an adversary pass. The finding closed here was surfaced in LOCAL pass-5 and held for orchestrator decision before fix-burst-5 was authorized.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP5-LOW-003 | LOW (design-level) | RESOLVED | Eager-token implementation (worktree 2fe7068c) + BC-2.16.002 v1.5 amendment (factory 82fd868c). Option A authorized by user. |
| TD-S-PLUGIN-PREREQ-B-010 | P2 ORCHESTRATOR-DECISION-PENDING | CLOSED | Status updated in tech-debt-register v2.10. Resolution: eager-token implemented; lazy-on-401 anti-pattern eliminated. |

## Part B — New Findings (or all findings for pass 1)

No new findings in this fix-burst closure record. Fix-burst-5 closed F-LP5-LOW-003 via dual commit:

**Layer 1 — Code (worktree 2fe7068c):**

pipeline.rs eager acquire_token at pipeline start. AuthType has no Null variant, so acquisition is unconditional for all real specs. Two new audit-log events added: `auth_initial_acquired` (info) and `auth_initial_failed` (error). Two new Red Gate tests added. Two existing tests adjusted for new acquire_token call count semantics (lazy 1-call → eager 2-call pattern for refresh scenarios).

**Layer 2 — Spec (factory-artifacts 82fd868c):**

BC-2.16.002 v1.4 → v1.5 amendment. Precondition lifecycle changed lazy→eager. New postconditions for request_count semantics (HTTP requests only, excludes acquire_token transport) and auth_initial_* audit family. Existing auth_refresh_* family amended with full Ok/Err/double-401 events.

**Architectural notes:**

- AuthType variants: Oauth2ClientCredentials, BearerStatic, CookieRoundtrip, ApiKey (4 variants, no Null/None). Eager acquisition is unconditional for all valid specs.
- NullAuthProvider is test-only — returns empty token without I/O. No test behavior changes.
- request_count semantics: counts only HTTP pipeline requests, NOT acquire_token transport. Single-step single-page → request_count == 1 (was 2 with lazy).

**Audit-log event family (final v1.5 semantic):**

| Event | Level | Trigger |
|---|---|---|
| `auth_initial_acquired` | info | Eager success at pipeline start |
| `auth_initial_failed` | error | Eager failure at pipeline start; aborts pipeline immediately |
| `auth_refresh_triggered` | warn | Mid-pipeline 401 detected (rare, legitimate) |
| `auth_refresh_succeeded` | info | Refresh on 401 succeeded |
| `auth_refresh_failed` | error | Refresh on 401 failed |
| `auth_refresh_double_401` | error | Refresh succeeded but retry returned 401; aborts |

Token value is NEVER included in any event.

**Tests:**

| Test | Status | Notes |
|---|---|---|
| `test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request` | NEW | Verifies eager acquire_token before first HTTP request |
| `test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution` | NEW | Verifies auth_initial_acquired fires; auth_refresh_triggered does NOT fire on clean execution |
| `test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401` | ADJUSTED | calls() count semantics: 1→2 (eager+refresh pattern) |
| `test_BC_2_16_002_execute_aborts_on_double_401` | ADJUSTED | calls() count semantics: 1→2 (eager+refresh pattern) |

Red Gate count: 37 → 39 (+2 new). Test pass: 273/273 (was 271). Workspace build: clean.

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

_None._

### LOW

_None._

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

**Overall Assessment:** pass (fix-burst closure — all findings from pass-5 now resolved)
**Convergence:** findings remain — pass-6 next to verify eager-token closure + audit-log family + novel dimensions
**Readiness:** pass-6 dispatchable at HEAD 2fe7068c

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 (fix-burst closure) |
| **New findings** | 0 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | n/a (fix-burst record, not adversary pass) |
| **Median severity** | n/a |
| **Trajectory** | 20→10→4→7→10 (pass-5 was non-monotonic; fix-burst-5 closes F-LP5-LOW-003; pass-6 next) |
| **Verdict** | FINDINGS_REMAIN — pass-6 required to verify closure + find novel defects |
