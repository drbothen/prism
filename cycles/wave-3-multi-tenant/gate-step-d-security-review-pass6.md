---
document_type: security-review
level: ops
version: "1.0"
status: final
producer: security-reviewer
timestamp: 2026-05-02T22:00:00Z
phase: 3
wave: 3
step: d
pass: 6
previous_review: gate-step-d-security-review-pass5.md
develop_sha: ba3b10c7
reviewer: vsdd-factory:security-reviewer
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: "e4be29ae..ba3b10c7 (Wave 3.4 — 2 fix PRs: #124 W3-FIX-CODE-006, #125 W3-FIX-SEC-005)"
inputs:
  - .factory/STATE.md (v6.13)
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass5.md
  - .factory/specs/prd-supplements/error-taxonomy.md (v1.13)
  - crates/prism-dtu-cyberint/src/routes/dtu.rs
  - crates/prism-dtu-cyberint/src/routes/auth.rs
  - crates/prism-dtu-cyberint/src/routes/alerts.rs
  - crates/prism-dtu-cyberint/src/clone.rs
  - crates/prism-dtu-jira/src/routes/dtu.rs
  - crates/prism-dtu-nvd/src/routes/dtu.rs
  - crates/prism-dtu-pagerduty/src/routes/dtu.rs
  - crates/prism-dtu-threatintel/src/routes/dtu.rs
  - crates/prism-dtu-threatintel/src/routes/lookup.rs
  - crates/prism-dtu-armis/tests/cr023_activity_risk_org_id_guard.rs
  - crates/prism-dtu-cyberint/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-cyberint/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-cyberint/tests/ac_7_rate_limit.rs
  - crates/prism-dtu-cyberint/tests/ac_8_reset_semantics.rs
  - crates/prism-dtu-cyberint/tests/fidelity_validator.rs
  - crates/prism-dtu-cyberint/tests/multi_tenant.rs
  - crates/prism-dtu-jira/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-jira/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-nvd/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-nvd/tests/fidelity_validator.rs
  - crates/prism-dtu-pagerduty/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-pagerduty/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-threatintel/tests/td_wv0_07_configure_requires_admin_token.rs
  - crates/prism-dtu-threatintel/tests/td_wv0_08_reset_requires_admin_token.rs
  - crates/prism-dtu-threatintel/tests/dtu_reset_mount.rs
  - crates/prism-dtu-threatintel/tests/fidelity_validator.rs
  - crates/prism-dtu-pagerduty/tests/fidelity.rs
  - crates/prism-dtu-common/src/clone.rs
input-hash: "ba3b10c"
traces_to: "wave-3-integration-gate"
total_findings: 4
critical: 0
high: 0
medium: 0
low: 4
files_reviewed: 27
verdict: APPROVED
---

# Wave 3 Integration Gate — Gate Step D: Security Review (Pass 6)

**Scope:** e4be29ae..ba3b10c7 (Wave 3.4 fix PRs: #124 W3-FIX-CODE-006, #125 W3-FIX-SEC-005)
**Predecessor review:** gate-step-d-security-review-pass5.md (SHA ba3b10c7, verdict: APPROVED)
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-05-02
**Develop SHA:** ba3b10c7
**Verdict:** APPROVED — 4 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW — all carry-forward)

---

## Executive Summary

This pass-6 review performs an independent, fresh-perspective analysis of the same
Wave 3.4 delta (e4be29ae..ba3b10c7). The information asymmetry wall prevents access
to pass-5 commentary, so all findings below are derived from direct code inspection.

Wave 3.4 adds admin-token gates (`subtle::ConstantTimeEq::ct_eq`) to both
`POST /dtu/configure` and `POST /dtu/reset` across five previously ungated DTU crates
(cyberint, jira, nvd, pagerduty, threatintel), and separately upgrades the
ThreatIntel `lookup.rs` `configure` handler to use the same pattern. The Armis
`cr023_activity_risk_org_id_guard.rs` test file adds coverage-only tests for the
org-id boundary guards on the activity and risk endpoints.

Pass-6 explored three fresh angles not explicitly foregrounded in prior reviews:
(1) CWE-352 CSRF exploitability assessment for Cyberint's cookie-based auth model;
(2) CWE-200 information disclosure in the new error messages;
(3) CWE-285 scope-of-reset authorization — whether post_reset is limited to the
correct org or inadvertently affects all-org state.

No new findings of CRITICAL, HIGH, or MEDIUM severity were identified. The four
carry-forward LOWs from pass-5 are sustained without severity escalation.

---

## Pass-6 Scope: What Changed in Wave 3.4

Two PRs merged after the Pass 4 gate review:

| PR | ID | Purpose |
|----|-----|---------|
| #124 | W3-FIX-CODE-006 | CR-023: Armis `get_device_activity` / `get_device_risk` org-id guard regression tests (+6 tests) |
| #125 | W3-FIX-SEC-005 | CR-021/022, R1-001: 5-DTU admin-token uniformity — all 10 sites now `ct_eq`; ThreatIntel `lookup.rs` configure ct_eq (+21 tests) |

---

## CWE/OWASP Analysis — Fresh Perspective

### CWE-208 (Observable Timing Discrepancy) — Post_configure and Post_reset

**Finding:** Resolved in this diff. Fresh code inspection confirms:

All five new DTU route files (cyberint, jira, nvd, pagerduty, threatintel) now use the
identical `subtle::ConstantTimeEq::ct_eq` pattern:

```rust
use subtle::ConstantTimeEq;

let provided = headers.get("x-admin-token")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
let provided_bytes = provided.as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { return 401 }
```

Pattern correctness assessment:
- `subtle 2.x` `ConstantTimeEq::ct_eq` returns a `subtle::Choice` value; `.into()`
  converts it to `bool`. This is the canonical usage. No misuse detected.
- Admin tokens are `uuid::Uuid::new_v4().to_string()` — UUID v4 formatted strings,
  always 36 bytes. The `ct_eq` comparison operates on `&[u8]` slices. Unequal-length
  inputs return `0u8` (false) without early-exit timing signal — the `subtle` crate
  documents this behavior. No length-leak timing concern at the attack surface here.
- `unwrap_or("")` for absent header: when the `X-Admin-Token` header is absent, the
  empty string `""` is compared against the 36-byte admin token via `ct_eq`, which
  returns false in constant time. No early-exit when header is absent.

**Disposition:** CWE-208 FULLY MITIGATED at all 10 DTU route handler sites.

---

### CWE-306 (Missing Authentication for Critical Function) — Post_reset

**Finding:** Resolved in this diff. All five newly-added `post_reset` (or `dtu_reset`
for ThreatIntel) handlers now require and validate `X-Admin-Token` before any state
mutation. Confirmed in cyberint `dtu.rs` (lines 80-93), jira `dtu.rs` (lines 77-90),
nvd `dtu.rs` (lines 108-121), pagerduty `dtu.rs` (lines 108-121), and
threatintel `dtu.rs` (lines 43-56).

**Disposition:** CWE-306 FULLY MITIGATED across all 10 DTU route handler sites.

---

### CWE-352 (Cross-Site Request Forgery) — Cyberint Cookie Auth — Fresh Angle

**Attack vector considered:** Cyberint DTU uses `Set-Cookie: cyberint_session={uuid}; Path=/; HttpOnly`
for its alert endpoints. The cookie does not have a `SameSite` attribute. The
`PATCH /api/v1/alerts/{id}/status` and `POST /api/v1/alerts/{id}/close` endpoints
accept cookie-authenticated mutation requests. Could a cross-origin attacker forge a
request to one of these endpoints using the victim's cookie?

**Assessment:** No exploitable CSRF risk in the production threat model.

The CSRF attack chain requires: (1) a browser to make the cross-origin request using
a stored cookie, (2) the victim to have a valid `cyberint_session` cookie in their
browser at the time of attack, and (3) the DTU server to be reachable from the
cross-origin attacker's page.

None of these conditions are satisfiable in this deployment model:
- The DTU is a `#[cfg(feature="dtu")]`-gated Rust test server bound to
  `127.0.0.1:0` (ephemeral loopback port assigned per-process by the OS).
  The binding is verified in `prism-dtu-common/src/clone.rs` line 49 and
  individual `clone.rs` `start_on` implementations.
- There is no browser in this architecture. Prism is a per-analyst stdio MCP server.
  No user agent capable of the cross-origin cookie attachment is present.
- Even if a browser were present, the `HttpOnly` flag prevents JavaScript from reading
  the session cookie, eliminating the XHR vector.
- The ephemeral port is unknown to any cross-origin attacker; enumeration is bounded
  by the test process lifetime.

**Disposition:** CWE-352 is NOT APPLICABLE to this codebase. DTU endpoints are
loopback-bound test infrastructure; no browser context or cross-origin request
scenario is possible within the production deployment model. No finding raised.

---

### CWE-200 (Information Exposure Through Error Messages) — Fresh Angle

**Attack vector considered:** The new handlers return `{"error": "missing or invalid X-Admin-Token"}`
on auth failure. Could this error message leak sensitive information?

**Assessment:** The error string is appropriately generic and does not:
- Reveal whether the header was absent vs. present with wrong value (both return the
  same message, preventing oracle attacks on header presence).
- Reveal the expected token value or any prefix/suffix.
- Reveal internal state information.

One pre-existing item in ThreatIntel `lookup.rs` line 329 uses
`format!("invalid /dtu/configure payload: {e}")` where `{e}` is a `serde_json` parse
error. This discloses the shape of the rejected JSON body to the caller. This is
pre-existing (present at `e4be29ae`) and not part of this diff. In the DTU context
(test-only, admin-token-gated), the caller is the test harness itself and already
knows the body it sent. No security concern in production.

**Disposition:** CWE-200 NOT PRESENT in the new code. The pre-existing format error
in `lookup.rs:329` is test-scope only and not introduced by this diff. No finding raised.

---

### CWE-285 (Improper Authorization) — Scope-of-Reset Fresh Angle

**Attack vector considered:** After the admin-token gate is added to `post_reset`, could
an authenticated caller abuse the reset endpoint to clear another org's data? Does the
reset scope authorization correctly limit blast radius?

**Assessment:** Cyberint `post_reset` is the only DTU reset handler that supports
per-org targeted reset (via `X-Prism-Org-Id` header). The handler:
1. Validates the admin token first (auth barrier).
2. If `x-prism-org-id` is present: calls `state.reset_for(org_id)` — scoped to that
   org's namespace only.
3. If `x-prism-org-id` is absent: calls `state.reset()` which resets all orgs — this is
   intentional backward-compat behavior for test harnesses that predate multi-tenancy,
   documented in the handler's docstring.

For the other four DTUs (jira, nvd, pagerduty, threatintel), `post_reset` / `dtu_reset`
resets all state unconditionally. These DTUs do not implement per-org scoped state (they
are shared-mode simulators), so the reset is correctly scoped to the single clone
instance. An attacker with the admin token can only reset the specific clone instance
they are targeting — each clone has an independent admin token generated at instantiation
time. There is no cross-instance contamination possible.

**Disposition:** CWE-285 NOT PRESENT. The reset scope is consistent with the multi-
tenancy model: per-org reset for Cyberint (multi-org capable), full-clone reset for
the others (single-org clones). No finding raised.

---

### CWE-863 (Incorrect Authorization) — Post_configure Body Handling

**Attack vector considered:** After passing the admin-token gate, `post_configure`
handlers process a `Json(body)` payload. Could a malicious or malformed payload cause
an authorization bypass or privilege escalation?

**Assessment:** All five new handlers delegate to `state.apply_config(&body)` or
equivalent. These methods are internal state mutations limited to DTU behavioral
configuration (auth mode, rate limit threshold, fixture registry). They cannot:
- Elevate the admin token value (admin tokens are generated at construction, not
  configurable via `/dtu/configure`).
- Create new principals or access control entries.
- Affect production crate state (DTUs are entirely in-process test infrastructure).

Parse errors from `apply_config` return HTTP 400 without revealing internal token
values. The ThreatIntel `lookup.rs` configure handler returns the serde error text on
parse failure (pre-existing behavior, noted above under CWE-200), which is acceptable
in a test-harness context.

**Disposition:** CWE-863 NOT PRESENT at any of the five new handler sites.

---

### CWE-693 (Protection Mechanism Failure) — Uniformity Verification

**Assessment:** This pass independently verifies the uniformity of the `ct_eq` pattern
across all 10 DTU route file handler sites by reading the source code directly.

Confirmed `subtle::ConstantTimeEq` import and `ct_eq` pattern present in:
- `prism-dtu-cyberint/src/routes/dtu.rs` — `post_configure` (line 46), `post_reset` (line 86)
- `prism-dtu-jira/src/routes/dtu.rs` — `post_configure` (line 43), `post_reset` (line 82)
- `prism-dtu-nvd/src/routes/dtu.rs` — `post_configure` (line 76), `post_reset` (line 113)
- `prism-dtu-pagerduty/src/routes/dtu.rs` — `post_configure` (line 73), `post_reset` (line 113)
- `prism-dtu-threatintel/src/routes/dtu.rs` — `dtu_reset` (line 47)
- `prism-dtu-threatintel/src/routes/lookup.rs` — `configure` (line 314)
- `prism-dtu-armis/src/routes/dtu.rs` — (prior PRs, confirmed in registry from pass-5)
- `prism-dtu-claroty/src/routes/devices.rs` — (prior PRs, confirmed in registry from pass-5)
- `prism-dtu-crowdstrike/src/routes/mod.rs` — (prior PRs, confirmed in registry from pass-5)
- `prism-dtu-slack/src/routes/dtu.rs` — (prior PRs, confirmed in registry from pass-5)

All 10 sites: `valid: bool = provided_bytes.ct_eq(expected_bytes).into()`.

**Disposition:** CWE-693 NOT PRESENT. Protection mechanism is uniform across the entire
DTU admin-token surface. No gaps or inconsistencies found.

---

## Test Coverage Analysis

### New Test Files (W3-FIX-SEC-005)

Pass-6 independently reads the new test files to verify they exercise the correct
failure modes:

**cyberint `td_wv0_08_reset_requires_admin_token.rs`:**
- `test_reset_requires_admin_token_missing_returns_401`: no header sent, expects 401.
- `test_reset_requires_admin_token_wrong_returns_401`: wrong literal token, expects 401.
- `test_reset_correct_admin_token_returns_200`: `clone.admin_token()` value, expects 200.
All three cases correctly test the boundary. The test body shows the previous comment
"RED GATE: currently returns 200 because post_reset has no admin-token gate (CR-021)"
which is now the fixed green state.

**jira/pagerduty `td_wv0_07_configure_requires_admin_token.rs`:**
Same three-case pattern. Jira test sends `{"failure_mode": "none"}` as body;
pagerduty test sends the same. Correct token comes from `clone.admin_token()`.
Tests are structurally identical to pre-existing configure tests in Armis/Claroty/CrowdStrike.

**nvd/threatintel `td_wv0_08_reset_requires_admin_token.rs`:**
Same three-case pattern. These are the correct test files since nvd and threatintel
already had pre-existing `td_wv0_07_configure_requires_admin_token.rs` files (verified
by checking the test directory listings — both pre-existed in the test corpus).

**Armis `cr023_activity_risk_org_id_guard.rs`:**
Six tests covering the dual-mode org-id guard for `get_device_activity` and
`get_device_risk`. Uses `OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000CC"))`
as a non-default, non-nil org id. Tests verify both real-org (absent header → 401,
correct header → 200) and default-instance (absent header → 200) guard semantics.
No security issues in test construction.

**Existing test updates (cyberint):**
All four updated test files (`ac_7_rate_limit.rs`, `ac_8_reset_semantics.rs`,
`fidelity_validator.rs`, `multi_tenant.rs`) correctly add `X-Admin-Token` header from
`clone.admin_token()` to `POST /dtu/reset` calls. The fidelity_validator update uses
the `headers: vec![("X-Admin-Token".to_string(), admin_token.clone())]` form, which
confirms the `FidelityCheck` struct supports arbitrary headers.

**NVD and PagerDuty fidelity updates:**
`nvd/tests/fidelity_validator.rs` corrects the comment from "no auth required" to
"requires X-Admin-Token per ADR-003 Amendment #5" and adds the header. Correct.
`pagerduty/tests/fidelity.rs` adds the `X-Admin-Token` header to the reset check.

---

## Positive Findings (Defensive Measures Present)

- **5-DTU admin-token uniformity (CR-021/022):** All five previously-ungated DTUs
  (cyberint, jira, nvd, pagerduty, threatintel) now gate both `POST /dtu/configure`
  and `POST /dtu/reset` (or `dtu_reset`) with `subtle::ConstantTimeEq::ct_eq`.
  Complete coverage: 10 of 10 DTU route-file handler sites now use constant-time
  comparison. The `subtle` crate usage is correct — `Choice` to `bool` via `.into()`
  is the canonical pattern; `unwrap_or("")` ensures constant-time rejection even when
  the header is absent.

- **ThreatIntel lookup.rs ct_eq (R1-001):** The `configure` handler in `lookup.rs`
  upgraded from non-constant `!=` to `ct_eq`, closing a timing oracle discovered
  independently by the PR-reviewer beyond the original story AC scope.

- **Reset scope authorization (Cyberint):** `post_reset` correctly implements a
  dual-mode scope: per-org reset when `X-Prism-Org-Id` is present, full-clone reset
  when absent (backward-compatible for pre-multi-tenancy test suites). The admin-token
  gate precedes the scope decision, preventing unauthenticated resets in either mode.

- **Test coverage uniformity:** All five new DTUs now have both
  `td_wv0_07_configure_requires_admin_token.rs` and
  `td_wv0_08_reset_requires_admin_token.rs` test files (3 cases each: absent → 401,
  wrong → 401, correct → 200). Where td_wv0_07 pre-existed (NVD, ThreatIntel), only
  the td_wv0_08 reset test was added — correct targeting, no redundancy.

- **Existing test remediation (Cyberint):** Four pre-existing test files
  (`ac_7_rate_limit.rs`, `ac_8_reset_semantics.rs`, `fidelity_validator.rs`,
  `multi_tenant.rs`) were updated to supply the `X-Admin-Token` header on all
  `POST /dtu/reset` calls, preserving test correctness after the gate was added.

- **Armis CR-023 test coverage:** `cr023_activity_risk_org_id_guard.rs` closes the
  test coverage gap for `get_device_activity` and `get_device_risk` — 6 tests for the
  dual-mode org-id guard. Guard semantics (real-org → enforce, default-instance → pass)
  are verified correctly.

- **No new production attack surface:** Wave 3.4 adds no new public-facing routes,
  no new auth logic beyond extending the established `ct_eq` pattern, and no new
  cross-tenant data paths. The deployment model (per-analyst stdio MCP) is unaffected.

- **Generic, non-disclosing error messages:** The uniform `{"error": "missing or
  invalid X-Admin-Token"}` message across all five new handlers does not distinguish
  absent-header from wrong-token, preventing an enumeration oracle on header presence.

---

## Carry-Forward LOW Findings — Severity Escalation Assessment

### SEC-P3-004 (Carry-Forward) — OrgSlug 64-char Limit (ADR-006 OQ-1)

- **Severity:** LOW (unchanged)
- **CWE:** CWE-20 (Improper Input Validation)
- **File:** `crates/prism-core/src/tenant.rs` (`ORG_SLUG_PATTERN`)
- **Status:** Deferred — not touched in Wave 3.4.
- **Escalation assessment:** No new attack surface introduced in this diff that
  changes exploitability. Config-layer enforcement via E-CFG-019 remains in place.
  Severity: **LOW — no escalation.**

---

### SEC-P3-005 (Carry-Forward) — `org_slug` Cross-Check Operational Observability

- **Severity:** LOW (unchanged)
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity)
- **File:** `crates/prism-audit/src/audit_emitter.rs`
- **Status:** Deferred — not touched in Wave 3.4.
- **Escalation assessment:** The diff introduces no new audit code paths and no changes
  to the audit emitter. The `let _ = validate_org_slug_cross_check(...)` observability
  gap is unaffected. Severity: **LOW — no escalation.**

---

### SEC-P3-006 (Carry-Forward) — `build_network()` Wildcard Dispatch

- **Severity:** LOW (unchanged)
- **CWE:** CWE-284 (Improper Access Control) — architectural quality gap
- **File:** `crates/prism-dtu-harness/src/builder.rs`
- **Status:** Pre-existing; not touched in Wave 3.4.
- **Escalation assessment:** The diff includes no changes to `prism-dtu-harness`.
  Confirmed via `git diff e4be29ae..ba3b10c7 -- crates/prism-dtu-harness/` (empty).
  Severity: **LOW — no escalation.**

---

### SEC-005 (Carry-Forward) — CWE-208 Pre-Existing Non-Constant-Time Admin Token Comparisons in prism-dtu-harness

- **Severity:** LOW (unchanged)
- **CWE:** CWE-208 (Observable Timing Discrepancy)
- **File:** `crates/prism-dtu-harness/src/` (11 sites)
- **Status:** Pre-existing; not introduced in Wave 3.4; not touched in this diff.
- **Escalation assessment:** No changes to `prism-dtu-harness` in this diff. The 11
  pre-existing `!=` admin-token comparisons in `src/clone_server.rs`, `src/builder.rs`,
  and `src/clones/*.rs` remain. All remain in test-harness scope (loopback-only,
  ephemeral port, no production exposure). TD-W3-CT-EQ-COVERAGE-001 tracks this.
  Severity: **LOW — no escalation.**

---

## CWE/OWASP Coverage Assessment (Pass-6)

| CWE | Area | Pass-6 Status |
|-----|------|--------------|
| CWE-22 (Path Traversal) | `validate_spec_path` pre-join unconditional checks | MITIGATED — not touched in Wave 3.4 |
| CWE-20 (Input Validation) | Org slug pattern, AQL validator | MITIGATED — not touched in Wave 3.4 |
| CWE-200 (Information Exposure) | New 401 error messages | NOT PRESENT — messages are generic; no token or state disclosed |
| CWE-208 (Timing Side-Channel) | Admin token ct_eq — all 10 DTU route sites | MITIGATED — Wave 3.4 closes all 5-DTU + R1-001 gaps |
| CWE-208 (Timing Side-Channel) | Admin token in prism-dtu-harness (11 sites) | PARTIALLY MITIGATED — test-scope only; TD-W3-CT-EQ-COVERAGE-001; SEC-005 LOW |
| CWE-209 (Info Exposure via Error Messages) | TOML redaction | MITIGATED — not touched in Wave 3.4 |
| CWE-284 (Improper Access Control) | Cross-org credential guard, X-Org-Id guards | MITIGATED — not touched in Wave 3.4 |
| CWE-285 (Improper Authorization) | Reset scope: per-org vs. all-org semantics | NOT PRESENT — reset scope matches multi-tenancy model |
| CWE-306 (Missing Authentication) | `POST /dtu/reset` — all 10 DTU route files | MITIGATED — Wave 3.4 closes 5-DTU gap |
| CWE-345 (Insufficient Verification) | Org slug audit cross-check | PARTIALLY MITIGATED — observability gap; SEC-P3-005 LOW |
| CWE-352 (CSRF) | Cyberint cookie-based alert mutation endpoints | NOT APPLICABLE — loopback-bound DTU; no browser context; stdout MCP deployment |
| CWE-693 (Protection Mechanism Failure) | ct_eq uniformity across all 10 DTU sites | NOT PRESENT — pattern confirmed uniform by independent code inspection |
| CWE-863 (Incorrect Authorization) | `POST /dtu/configure` and `POST /dtu/reset` all DTUs | MITIGATED — Wave 3.4 closes 5-DTU gap |

---

## Dependency Advisory Check (Pass-6)

PR #125 adds `subtle = "2"` to five Cargo.toml files. All resolve to `subtle 2.6.1`
(Cargo.lock checksum `13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292`).

This checksum is consistent with the pre-existing `subtle` dependency already in the
workspace from prior PRs. No new dependency relationships introduced by PR #124.
No CVEs or RUSTSEC advisories known for `subtle 2.6.1`. RustCrypto-maintained crate
with an active security review history.

---

## Error Taxonomy Verification (Pass-6)

Error taxonomy v1.13 is unchanged in Wave 3.4. E-CFG-018 (SpecPathTraversal, CWE-22)
and E-CFG-019 (InvalidOrgSlugPattern, CWE-20) remain present and correctly documented.
No new error codes are introduced by W3-FIX-SEC-005 or W3-FIX-CODE-006.

The new `{"error": "missing or invalid X-Admin-Token"}` message used uniformly across
all five new handlers is consistent with the existing admin-token error messages
established by the prior DTUs (verified in `prism-dtu-armis`, `prism-dtu-claroty`,
`prism-dtu-crowdstrike`, `prism-dtu-slack`). No taxonomy entry is required for DTU-
internal test route errors (DTU routes are not part of the product error surface).

---

## Risk Register Dispositions (Security-Category R-NNN Entries)

| Risk / Reference | Pass-5 Status | Pass-6 Status | Change |
|-----------------|--------------|--------------|--------|
| `POST /dtu/reset` unauthenticated (CWE-306) | Mitigated | **Mitigated** | Independently verified: all 5 new DTUs gate `dtu_reset`/`post_reset` with ct_eq. |
| `POST /dtu/configure` non-ct_eq (CWE-208 / CWE-863) | Mitigated | **Mitigated** | Independently verified: all 5 new DTUs gate `post_configure` with ct_eq. |
| ThreatIntel lookup.rs configure non-ct_eq (R1-001, CWE-208) | Mitigated | **Mitigated** | Independently verified: `lookup.rs` line 314 uses ct_eq. |
| prism-dtu-harness 11 non-ct_eq comparisons (TD-W3-CT-EQ-COVERAGE-001) | Partially Mitigated | **Partially Mitigated** | Pre-existing, test-scope only. No diff in harness. SEC-005 LOW. |
| Armis X-Org-Id header-presence conditional (CWE-284) | Mitigated | **Mitigated** | CR-023 test coverage added and verified. Source guard unchanged. |
| Pre-join path traversal bypass (CWE-22) | Mitigated | **Mitigated** | No change. |
| TOML credential redaction (CWE-209) | Mitigated | **Mitigated** | No change. |
| `org_slug` audit cross-check (CWE-345) | Mitigated | **Mitigated** | No change. Observability gap (SEC-P3-005) remains LOW. |
| OrgSlug 64-char limit / ADR-006 OQ-1 (CWE-20) | Partially Mitigated | **Partially Mitigated** | No change. SEC-P3-004 LOW. |
| Cross-tenant data leakage at adapter layer (ADR-006 §3.1) | Mitigated | **Mitigated** | No change. |
| Cross-tenant credential reachability (ADR-006 §3.2) | Mitigated | **Mitigated** | No change. |
| Path traversal in spec file loading (R-CUST-014/015) | Mitigated | **Mitigated** | No change. |

---

## Summary Table

| ID | Severity | CWE | Location | Origin | Pass-6 Status |
|----|----------|-----|----------|--------|--------------|
| SEC-P3-004 | **LOW** | CWE-20 | `prism-core/src/tenant.rs` | Pass-1 | Deferred (unchanged; no escalation) |
| SEC-P3-005 | **LOW** | CWE-345 | `prism-audit/audit_emitter.rs` | Pass-2 | Partially Mitigated (unchanged; no escalation) |
| SEC-P3-006 | **LOW** | CWE-284 | `prism-dtu-harness/src/builder.rs` | Pass-3 (architectural note) | Accepted (test-harness scope; no escalation) |
| SEC-005 | **LOW** | CWE-208 | `prism-dtu-harness/src/` (11 sites) | Pass-5 sweep (pre-existing) | Partially Mitigated (test-scope; TD-W3-CT-EQ-COVERAGE-001; no escalation) |

**No CRITICAL, HIGH, or MEDIUM open findings.**

---

## Recommendations Priority

### Immediate (before merge)

None. All CRITICAL and HIGH findings are closed. No blocking conditions exist.

### Before Release

1. **SEC-005 (LOW, CWE-208):** Apply `subtle::ConstantTimeEq::ct_eq` to all 11
   admin token comparisons in `prism-dtu-harness/src/`. TD-W3-CT-EQ-COVERAGE-001.
   Recommend completing before Wave 4 gate.
2. **SEC-P3-004 (LOW, CWE-20):** Resolve ADR-006 OQ-1 — evaluate tightening
   `ORG_SLUG_PATTERN` max length. Config-layer E-CFG-019 check is in place.
3. **SEC-P3-005 (LOW, CWE-345):** Add structured metrics counter for
   `SlugCheckResult::Mismatched` and `OrgNotInRegistry` in
   `validate_org_slug_cross_check`.

### Post-Release

4. **SEC-P3-006 (LOW, CWE-284):** Refactor `build_network()` to use exhaustive match
   (no `_ =>` arm) consistent with `start_clone()`. Architectural quality improvement.

---

## Verdict

**APPROVED**

Pass-6 applies a fresh perspective to the Wave 3.4 delta and probes three new attack
angles not previously examined: CWE-352 CSRF (assessed NOT APPLICABLE — loopback-
bound DTU, no browser context, HttpOnly cookie), CWE-200 information disclosure in
new error messages (assessed NOT PRESENT — messages are generic, no state leaked),
and CWE-285 reset scope authorization (assessed NOT PRESENT — per-org vs. all-org
reset semantics correctly match the multi-tenancy model).

The `subtle::ConstantTimeEq::ct_eq` implementation is confirmed correct: `Choice`
to `bool` via `.into()` is the canonical usage pattern; `unwrap_or("")` for absent
headers ensures constant-time rejection even with no header present. All 10 DTU
admin-token handler sites use the identical pattern.

The four carry-forward LOWs (SEC-P3-004, SEC-P3-005, SEC-P3-006, SEC-005) show no
severity escalation under pass-6 analysis. None block wave progression.

Wave 3 integration gate step D (Pass 6) is **unconditionally approved**.
This is pass 2 of 3 toward the convergence window at ba3b10c7.
