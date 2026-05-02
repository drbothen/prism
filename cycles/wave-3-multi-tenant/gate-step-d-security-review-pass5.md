---
document_type: security-review
level: ops
version: "1.0"
status: final
producer: security-reviewer
timestamp: 2026-05-02T21:00:00Z
phase: 3
wave: 3
step: d
pass: 5
previous_review: gate-step-d-security-review-pass4.md
develop_sha: ba3b10c7
reviewer: vsdd-factory:security-reviewer
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: "e4be29ae..ba3b10c7 (Wave 3.4 — 2 fix PRs: #124 W3-FIX-CODE-006, #125 W3-FIX-SEC-005)"
inputs:
  - .factory/STATE.md (v6.13)
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass4.md
  - .factory/cycles/wave-3-multi-tenant/adversarial-reviews/pass-51.md
  - .factory/specs/prd-supplements/error-taxonomy.md (v1.13)
  - crates/prism-dtu-cyberint/src/routes/dtu.rs
  - crates/prism-dtu-jira/src/routes/dtu.rs
  - crates/prism-dtu-nvd/src/routes/dtu.rs
  - crates/prism-dtu-pagerduty/src/routes/dtu.rs
  - crates/prism-dtu-threatintel/src/routes/dtu.rs
  - crates/prism-dtu-threatintel/src/routes/lookup.rs
  - crates/prism-dtu-cyberint/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-cyberint/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-jira/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-jira/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-nvd/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-nvd/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-pagerduty/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-pagerduty/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-threatintel/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-threatintel/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-armis/tests/cr023_activity_risk_org_id_guard.rs
  - crates/prism-dtu-harness/src/clone_server.rs (pre-existing sweep)
  - crates/prism-dtu-harness/src/builder.rs (pre-existing sweep)
  - crates/prism-dtu-harness/src/clones/* (pre-existing sweep, 7 files)
input-hash: "ba3b10c"
traces_to: "wave-3-integration-gate"
total_findings: 4
critical: 0
high: 0
medium: 0
low: 4
files_reviewed: 28
verdict: APPROVED
---

# Wave 3 Integration Gate — Gate Step D: Security Review (Pass 5)

**Scope:** e4be29ae..ba3b10c7 (Wave 3.4 fix PRs: #124 W3-FIX-CODE-006, #125 W3-FIX-SEC-005)
**Predecessor review:** gate-step-d-security-review-pass4.md (SHA e4be29ae, verdict: APPROVED)
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-05-02
**Develop SHA:** ba3b10c7
**Verdict:** APPROVED — 4 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW — 3 carry-forward deferred + 1 new pre-existing LOW from workspace CWE-208 sweep)

---

## Executive Summary

Wave 3.4 delivers the 5-DTU admin-token uniformity fix (W3-FIX-SEC-005, PR #125) and
Armis activity/risk org-id guard test coverage (W3-FIX-CODE-006, PR #124). Every
`POST /dtu/configure` and `POST /dtu/reset` handler across all 10 production DTU route
files now uses `subtle::ConstantTimeEq::ct_eq` for admin token comparison (CWE-208
MITIGATED). The ThreatIntel `lookup.rs` `configure` handler non-constant comparison
(R1-001, D-194) is also resolved in this batch. The mandatory workspace-wide CWE-208
sweep identified 11 pre-existing `!=` comparisons against admin tokens in
`prism-dtu-harness` — none introduced by this diff — which are correctly scoped to the
test-harness-only in-process execution model and are tracked under TD-W3-CT-EQ-COVERAGE-001.
No new CRITICAL, HIGH, or MEDIUM vulnerabilities are introduced.

---

## Pass-5 Scope: What Changed in Wave 3.4

Two PRs merged after the Pass 4 gate review:

| PR | Story | Purpose |
|----|-------|---------|
| #124 | W3-FIX-CODE-006 | CR-023: Armis `get_device_activity` / `get_device_risk` org-id guard regression tests (+6t) |
| #125 | W3-FIX-SEC-005 | CR-021/022, R1-001: 5-DTU admin-token uniformity — cyberint/jira/nvd/pagerduty/threatintel post_configure ct_eq + post_reset ct_eq gate (+21t); ThreatIntel lookup.rs configure ct_eq |

---

## Pass-4 Condition Closures — Verification

### CWE-863: 5-DTU post_reset admin-token gate gap (CR-021)

**Status: RESOLVED (PR #125)**

Pass-51 adversarial review (ADV-W3GATE-P51-L-001) and code reviewer CR-021 identified
that `POST /dtu/reset` in cyberint, jira, nvd, pagerduty, and threatintel had no
admin-token gate. All five DTU clones now gate `post_reset` (or `dtu_reset` for
ThreatIntel) with the standard `ct_eq` pattern.

**Verification methodology:** Read all five changed `dtu.rs` route files at ba3b10c7.
Confirmed `subtle::ConstantTimeEq` import and `ct_eq` pattern at both `post_configure`
and `post_reset` (or equivalent) in each DTU.

**Implementation uniformity check across all 10 DTU route files:**

| DTU | Handler File | post_configure ct_eq | post_reset ct_eq |
|-----|-------------|----------------------|------------------|
| Armis | routes/dtu.rs | YES (lines 48, 85) | YES (Pass 4, PR #122) |
| Claroty | routes/devices.rs | YES (line 337) | YES (line 405) |
| CrowdStrike | routes/mod.rs | YES (lines 47, 76) | YES |
| Slack | routes/dtu.rs | YES (lines 43, 80) | YES |
| Cyberint | routes/dtu.rs | YES (lines 46, 86) | YES — NEW (PR #125) |
| Jira | routes/dtu.rs | YES (lines 44, 83) | YES — NEW (PR #125) |
| NVD | routes/dtu.rs | YES (line 77) | YES (line 114) — NEW (PR #125) |
| PagerDuty | routes/dtu.rs | YES (lines 74, 114) | YES — NEW (PR #125) |
| ThreatIntel | routes/dtu.rs | N/A (no configure route) | YES (line 48) — NEW (PR #125) |
| ThreatIntel | routes/lookup.rs `configure` | YES (line 315) — R1-001 fix | N/A |

All 10 DTU admin-token check sites are now uniformly `ct_eq`. **CWE-863 FULLY MITIGATED.**

---

### CWE-208: 5-DTU post_configure non-constant-time comparison (CR-022)

**Status: RESOLVED (PR #125)**

Code reviewer CR-022 identified that the 5 newly-in-scope DTUs retained the prior
`provided != Some(state.admin_token.as_str())` pattern in `post_configure`. All five are
now upgraded to `ct_eq`. See the uniformity table above.

The `ct_eq` pattern is identical across all DTUs:
```rust
// SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
let provided = headers
    .get("x-admin-token")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
let provided_bytes = provided.as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { ... }
```

Length-mismatch early-exit assessment (same as pass-4): admin tokens are UUID v4
strings (36 bytes fixed length). Equal-length token submissions (the actual timing
attack vector) are processed in constant time. **CWE-208 FULLY MITIGATED at all
DTU route file sites.**

---

### R1-001: ThreatIntel lookup.rs configure non-constant-time comparison (D-194)

**Status: RESOLVED (PR #125)**

D-194 records that PR #125 includes the fc467937 remediation commit for ThreatIntel
`lookup.rs` `configure` handler. Before this fix, `lookup.rs:308` used:
```rust
let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
if provided != Some(state.admin_token.as_str()) {
```

After the fix, `lookup.rs:315` uses the same `ct_eq` pattern as all other DTUs.
`subtle::ConstantTimeEq` is imported at line 15. **R1-001 VERIFIED RESOLVED.**

Note on ThreatIntel `check_auth` (lines 31-53 of lookup.rs): This function validates
the presence of a non-empty API key query param or non-empty Bearer token for the
`ip_lookup`, `domain_lookup`, and `hash_lookup` endpoints. It performs a
presence/non-emptiness check only — it does not compare against a stored credential
value. There is no timing oracle concern here: the function branches on boolean
presence, not on a secret value comparison. This is correct behavior for the DTU's
simulation of the ThreatIntel API's "any non-empty key is accepted" auth model.

---

### CR-023: Armis activity/risk org-id guard test coverage gap

**Status: RESOLVED (PR #124)**

`get_device_activity` and `get_device_risk` received the dual-mode org-id guard in
W3-FIX-CODE-005 (PR #123) but `cr017_tag_alert_org_id_guard.rs` did not test those
endpoints. PR #124 adds `crates/prism-dtu-armis/tests/cr023_activity_risk_org_id_guard.rs`
with 6 test functions covering:
- AC-001/004: real-org, absent `X-Org-Id` → HTTP 401 (activity / risk)
- AC-002/005: real-org, correct `X-Org-Id` → HTTP 200 (activity / risk)
- AC-003/006: default-instance, absent `X-Org-Id` → HTTP 200 (backward compat, activity / risk)

The test correctly uses `OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000CC"))`
as a non-default, non-nil org id that triggers the `is_real_org` guard. No new source
code is changed; this PR adds tests only. No new security issues introduced.

---

## Workspace-Wide CWE-208 Sweep (Mandatory — Pass-4 Missed CR-021 Scope)

This pass performs an independent audit of ALL `!=` comparisons against admin token,
bearer token, secret, password, and credential values across the entire workspace.

### Sweep Methodology

Searched all `*.rs` files (excluding `//` comments and `#[` attributes) for patterns:
- `!= .*token`, `token !=`, `!= .*secret`, `!= .*password`, `!= .*bearer`, `api_key !=`

Also searched production crates (non-DTU, non-test) for any credential comparisons.

### DTU Route Files — All 10 Sites CLEAN (ct_eq)

All 10 DTU route files now use `ct_eq` as documented in the uniformity table above.
No `!=` admin-token comparisons remain in any DTU route implementation file.

### prism-dtu-harness — 11 Pre-Existing `!=` Comparisons

The workspace sweep identified 11 `!=` admin-token comparisons in `prism-dtu-harness`:

| File | Line | Pattern |
|------|------|---------|
| `src/clone_server.rs` | 331 | `if provided != Some(state.admin_token.as_str())` |
| `src/builder.rs` | 977 | `if token != admin_token` |
| `src/clones/armis.rs` | 744 | `if provided != Some(state.admin_token.as_str())` |
| `src/clones/claroty.rs` | 706 | `if provided != Some(state.admin_token.as_str())` |
| `src/clones/claroty.rs` | 941 | `if token != admin_token` |
| `src/clones/crowdstrike.rs` | 1010 | `if provided != Some(state.admin_token.as_str())` |
| `src/clones/crowdstrike.rs` | 1285 | `if provided_token != admin_token` |
| `src/clones/cyberint.rs` | 1056 | `if provided != Some(state.clone_state.admin_token.as_str())` |
| `src/clones/jira.rs` | 766 | `if provided != Some(ctx.clone_state.admin_token.as_str())` |
| `src/clones/pagerduty.rs` | 481 | `if provided != Some(ctx.clone_state.admin_token.as_str())` |
| `src/clones/slack.rs` | 260 | `if provided != Some(ctx.clone_state.admin_token.as_str())` |

**Confirmation that none are introduced in this diff:** `git diff e4be29ae..ba3b10c7 --
crates/prism-dtu-harness/` returns empty — `prism-dtu-harness` was not modified in
W3-FIX-SEC-005 or W3-FIX-CODE-006. All 11 patterns were present at `e4be29ae`.

**Exploitability assessment:**

`prism-dtu-harness` is an in-process test harness crate used exclusively during
`#[cfg(feature="dtu")]`-gated integration test execution. It does not expose network
endpoints in production deployments. The `prism-dtu-harness` clones run on ephemeral
loopback ports assigned by the OS, accessible only within the test process and its
localhost network context. The admin token is a `uuid::Uuid::new_v4()` generated fresh
per test run and never persisted or transmitted outside the test process.

In this context, timing oracle exploitation requires: (1) network reachability to the
ephemeral loopback port, (2) knowledge of the port number, and (3) the ability to send
many crafted requests within the test process lifetime. This is a test-environment-only
configuration. ADR-011 §2.3 explicitly scopes the harness to test execution.

**Disposition:** This is a coverage gap tracked under TD-W3-CT-EQ-COVERAGE-001 (D-194).
The risk is partially mitigated by scope (test-only) but the pattern is inconsistent
with the production DTU route files. Classified LOW — upgrade to `ct_eq` in harness
before Wave 4 recommended as technical debt.

### Production Crates (prism-core, prism-credentials, prism-audit, etc.)

No `!=` comparisons against credential, token, secret, password, or API-key values
found in any non-DTU, non-test production crate. The `prism-credentials` crate uses
`secrecy::SecretString` throughout credential storage and retrieval paths. No timing
oracle risk identified in production code paths.

---

## New Findings

### SEC-005: CWE-208 Pre-Existing Non-Constant-Time Admin Token Comparisons in prism-dtu-harness

- **Severity:** LOW
- **CWE:** CWE-208 (Observable Timing Discrepancy)
- **OWASP:** A02:2021 — Cryptographic Failures
- **Attack Vector:** An attacker with localhost access during test execution could measure
  response-time differences when submitting guesses for the admin token against
  `prism-dtu-harness` endpoints, potentially narrowing token value via timing oracle.
- **Impact:** Admin token recovery enabling unauthorized DTU configure/reset in the test
  harness context. No production impact — harness is test-scope only.
- **Evidence:** 11 sites in `crates/prism-dtu-harness/src/` use `if provided != Some(state.admin_token.as_str())` rather than `ct_eq`. Confirmed pre-existing at e4be29ae;
  not introduced by this diff.
- **Proposed Mitigation:** Apply the same `subtle::ConstantTimeEq::ct_eq` upgrade to all
  11 harness sites. `subtle = "2"` is already in the workspace dependency graph. TD filed
  as TD-W3-CT-EQ-COVERAGE-001 (D-194). Recommend completing before Wave 4 gate.
- **Note:** This finding was identified by the pass-4 missed-scope instruction (CR-021
  prompted mandatory harness sweep). It is pre-existing, not regressions from Wave 3.4.

---

## Carry-Forward LOW Findings (No Change)

### SEC-P3-004 (Carry-Forward) — OrgSlug 64-char Limit (ADR-006 OQ-1)

- **Severity:** LOW
- **CWE:** CWE-20 (Improper Input Validation)
- **File:** `crates/prism-core/src/tenant.rs` (`ORG_SLUG_PATTERN`)
- **Status:** Deferred — not addressed in Wave 3.4; unchanged.

Config-layer enforcement via E-CFG-019 remains in place. Core `ORG_SLUG_PATTERN =
r"^[a-zA-Z0-9_-]{1,64}$"` unchanged. Disposition: **PARTIALLY MITIGATED** (unchanged).

---

### SEC-P3-005 (Carry-Forward) — `org_slug` Cross-Check Operational Observability

- **Severity:** LOW
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity)
- **File:** `crates/prism-audit/src/audit_emitter.rs`
- **Status:** Deferred — not addressed in Wave 3.4; unchanged.

`validate_org_slug_cross_check` result remains `let _ = ...` with only `tracing::warn!`
for Mismatched/OrgNotInRegistry. No structured metrics counter. Acceptable for harness
context. Disposition: **PARTIALLY MITIGATED** (unchanged).

---

### SEC-P3-006 (Architectural Note, Carry-Forward) — `build_network()` Wildcard Dispatch

- **Severity:** LOW
- **CWE:** CWE-284 (Improper Access Control) — architectural quality gap
- **File:** `crates/prism-dtu-harness/src/builder.rs`
- **Status:** Pre-existing; not introduced in Wave 3.4; no change.

ADR-011 §2.3 scopes this to test-harness-only. Disposition: **ACCEPTED** (test-harness
scope, pre-existing design).

---

## CWE/OWASP Coverage Assessment

| CWE | Area | Pass-5 Status |
|-----|------|--------------|
| CWE-22 (Path Traversal) | `validate_spec_path` pre-join unconditional checks | MITIGATED — not touched in Wave 3.4 |
| CWE-20 (Input Validation) | Org slug pattern, AQL validator | MITIGATED — not touched in Wave 3.4 |
| CWE-208 (Timing Side-Channel) | Admin token ct_eq — all 10 DTU route sites | MITIGATED — Wave 3.4 closes 5-DTU + R1-001 gaps |
| CWE-208 (Timing Side-Channel) | Admin token in prism-dtu-harness (11 sites) | PARTIALLY MITIGATED — test-scope only; TD-W3-CT-EQ-COVERAGE-001 |
| CWE-209 (Info Exposure via Error Messages) | TOML redaction | MITIGATED — not touched in Wave 3.4 |
| CWE-306 (Missing Authentication) | `POST /dtu/reset` — all 10 DTU route files | MITIGATED — Wave 3.4 closes 5-DTU gap |
| CWE-284 (Improper Access Control) | Cross-org credential guard, X-Org-Id guards | MITIGATED — not touched in Wave 3.4 |
| CWE-345 (Insufficient Verification) | Org slug audit cross-check | PARTIALLY MITIGATED — observability gap tracked, SEC-P3-005 |
| CWE-863 (Incorrect Authorization) | `POST /dtu/configure` and `POST /dtu/reset` all DTUs | MITIGATED — Wave 3.4 closes 5-DTU gap |

---

## Dependency Advisory Check (Wave 3.4)

PR #125 adds `subtle = "2"` to five additional DTU crate `Cargo.toml` files
(cyberint, jira, nvd, pagerduty, threatintel). PR #124 adds no new dependencies.

All five resolve to `subtle 2.6.1` — confirmed via `Cargo.lock`:
```
name = "subtle"
version = "2.6.1"
checksum = "13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292"
```

This is the same version and checksum validated in pass-4 for the first four DTUs. The
crate is maintained by the RustCrypto organization. No known CVEs or RUSTSEC advisories.
No new supply-chain concerns.

---

## Error Taxonomy Verification

Error-taxonomy.md v1.13 is unchanged in Wave 3.4. E-CFG-018 (SpecPathTraversal, CWE-22)
and E-CFG-019 (InvalidOrgSlugPattern, CWE-20) remain present. No new error codes are
introduced by W3-FIX-SEC-005 or W3-FIX-CODE-006.

---

## Positive Findings (Defensive Measures Present)

- **5-DTU admin-token uniformity (CR-021/022):** All five previously missing DTUs
  (cyberint, jira, nvd, pagerduty, threatintel) now gate both `POST /dtu/configure` and
  `POST /dtu/reset` with `subtle::ConstantTimeEq::ct_eq`. Complete coverage: 10 of 10
  DTU route file handler sites now use constant-time comparison.
- **ThreatIntel lookup.rs ct_eq (R1-001):** The `configure` handler in `lookup.rs`
  upgraded from non-constant `!=` to `ct_eq`, closing the timing oracle found
  independently by pr-reviewer cycle 2 beyond the original story AC scope.
- **Test coverage uniformity:** All five new DTUs now have both
  `td_wv0_07_configure_requires_admin_token.rs` and
  `td_wv0_08_reset_requires_admin_token.rs` — 3 tests per file (missing → 401, wrong →
  401, correct → 200) covering the new gate behavior.
- **Existing test updates (Cyberint):** `ac_7_rate_limit.rs`, `ac_8_reset_semantics.rs`,
  `fidelity_validator.rs`, and `multi_tenant.rs` updated to supply `X-Admin-Token` header
  with correct token when calling `POST /dtu/reset`, maintaining test integrity after
  the gate was added.
- **Armis CR-023 test coverage:** `cr023_activity_risk_org_id_guard.rs` closes the
  coverage gap for `get_device_activity` and `get_device_risk` — 6 tests for the
  dual-mode org-id guard on the two Armis endpoints that received the guard in PR #123
  but were untested.
- **No new production attack surface:** Wave 3.4 adds no new public-facing routes, no
  new auth logic beyond extending the existing ct_eq pattern, and no new cross-tenant
  data paths.

---

## Risk Register Dispositions (Security-Category R-NNN Entries)

| Risk / ADR Reference | Pass-4 Status | Pass-5 Status | Change in Wave 3.4 |
|----------------------|--------------|--------------|-------------------|
| `POST /dtu/reset` unauthenticated (CWE-306) | Mitigated | **Mitigated** | 5-DTU gap closed: cyberint/jira/nvd/pagerduty/threatintel post_reset now gated. |
| `POST /dtu/configure` non-ct_eq (CWE-208 / CWE-863) | Mitigated (4 DTUs) | **Mitigated** | Remaining 5 DTUs upgraded to ct_eq. All 10 sites complete. |
| ThreatIntel lookup.rs configure non-ct_eq (R1-001, CWE-208) | Mitigated (pass-4 did not cover this) | **Mitigated** | fc467937 fix landed in PR #125. |
| prism-dtu-harness 11 non-ct_eq comparisons (TD-W3-CT-EQ-COVERAGE-001) | (Not previously scoped) | **Partially Mitigated** | Pre-existing, test-scope only. TD filed. SEC-005 LOW. |
| Armis X-Org-Id header-presence conditional (CWE-284) | Mitigated | **Mitigated** | CR-023 test coverage added. Source guard unchanged. |
| Pre-join path traversal bypass (CWE-22) | Mitigated | **Mitigated** | No change. |
| TOML credential redaction — inline-table + pipe-value (CWE-209) | Mitigated | **Mitigated** | No change. |
| `org_slug` audit cross-check (CWE-345) | Mitigated | **Mitigated** | No change. Observability gap (SEC-P3-005) remains LOW. |
| OrgSlug 64-char limit / ADR-006 OQ-1 (CWE-20) | Partially Mitigated | **Partially Mitigated** | No change. |
| Cross-tenant data leakage at adapter layer (ADR-006 §3.1) | Mitigated | **Mitigated** | No change. |
| Cross-tenant credential reachability (ADR-006 §3.2) | Mitigated | **Mitigated** | No change. |
| Path traversal in spec file loading (R-CUST-014/015) | Mitigated | **Mitigated** | No change. |

---

## Summary Table

| ID | Severity | CWE | Location | Origin | Pass-5 Status |
|----|----------|-----|----------|--------|--------------|
| SEC-P3-004 | **LOW** | CWE-20 | `prism-core/src/tenant.rs` | Pass-1 | Deferred (unchanged) |
| SEC-P3-005 | **LOW** | CWE-345 | `prism-audit/audit_emitter.rs` | Pass-2 | Partially Mitigated (unchanged) |
| SEC-P3-006 | **LOW** | CWE-284 | `prism-dtu-harness/src/builder.rs` | Pass-3 (architectural note) | Accepted (test-harness scope) |
| SEC-005 | **LOW** | CWE-208 | `prism-dtu-harness/src/` (11 sites) | Pass-5 sweep (pre-existing) | Partially Mitigated (test-scope); TD-W3-CT-EQ-COVERAGE-001 |

**No CRITICAL, HIGH, or MEDIUM open findings.**

---

## Recommendations Priority

### Immediate (before merge)

None. All CRITICAL and HIGH findings are closed. No blocking conditions exist.

### Before Release

1. **SEC-P3-004 (LOW, CWE-20):** Resolve ADR-006 OQ-1 — evaluate tightening
   `ORG_SLUG_PATTERN` max length. Config-layer E-CFG-019 check is in place.
2. **SEC-P3-005 (LOW, CWE-345):** Add structured metrics counter for
   `SlugCheckResult::Mismatched` and `OrgNotInRegistry` in
   `validate_org_slug_cross_check`.
3. **SEC-005 (LOW, CWE-208):** Apply `ct_eq` to all 11 admin token comparisons in
   `prism-dtu-harness/src/`. TD-W3-CT-EQ-COVERAGE-001. Recommend completing before
   Wave 4 gate.

### Post-Release

4. **SEC-P3-006 (LOW, CWE-284):** Refactor `build_network()` to use exhaustive match
   (no `_ =>` arm) consistent with `start_clone()`. Architectural quality improvement.

---

## Verdict

**APPROVED**

Wave 3.4 closes all security gaps identified in pass-51 and by the code reviewer's
CR-021/022 findings. All 10 DTU route file admin-token handler sites now use
`subtle::ConstantTimeEq::ct_eq` — uniform constant-time comparison pattern across
the entire DTU surface. The ThreatIntel R1-001 `configure` handler timing issue
(D-194) is also resolved. The Armis CR-023 test coverage gap closes test-layer
verification for the activity and risk endpoints.

The mandatory workspace-wide CWE-208 sweep identified 11 pre-existing `!=` comparisons
in `prism-dtu-harness` that were not changed by this diff. These are correctly scoped
to in-process test-harness execution (no production exposure), classified LOW, and
tracked under TD-W3-CT-EQ-COVERAGE-001. They do not block gate progression.

There are no open CRITICAL, HIGH, or MEDIUM findings. Four LOW findings carry forward:
OrgSlug 64-char ADR-006 OQ-1 (SEC-P3-004), audit cross-check observability gap
(SEC-P3-005), `build_network()` wildcard dispatch architectural note (SEC-P3-006),
and the harness non-ct_eq pre-existing gap (SEC-005). None block wave progression.

Wave 3 integration gate step D (Pass 5) is **unconditionally approved**.
This is pass 1 of 3 toward the convergence window at ba3b10c7.
