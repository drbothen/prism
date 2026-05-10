---
document_type: security-review
level: ops
version: "1.0"
status: final
producer: security-reviewer
timestamp: 2026-05-02T00:00:00Z
phase: 3
wave: 3
step: d
pass: 4
previous_review: gate-step-d-security-review-pass3.md
develop_sha: e4be29ae
reviewer: vsdd-factory:security-reviewer
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: "a7f0d374..e4be29ae (Wave 3.3 — 2 fix PRs: #122 W3-FIX-SEC-004, #123 W3-FIX-CODE-005)"
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass3.md
  - .factory/cycles/wave-3-multi-tenant/adversarial-reviews/pass-50.md
  - .factory/specs/prd-supplements/error-taxonomy.md (v1.13)
  - crates/prism-customer-config/src/validator.rs
  - crates/prism-dtu-armis/src/routes/dtu.rs
  - crates/prism-dtu-armis/src/routes/tags.rs
  - crates/prism-dtu-armis/src/routes/alerts.rs
  - crates/prism-dtu-armis/src/routes/devices.rs
  - crates/prism-dtu-claroty/src/routes/devices.rs
  - crates/prism-dtu-crowdstrike/src/routes/mod.rs
  - crates/prism-dtu-crowdstrike/src/routes/detections.rs
  - crates/prism-dtu-slack/src/routes/dtu.rs
input-hash: "11bc6a0"
traces_to: "wave-3-integration-gate"
total_findings: 3
critical: 0
high: 0
medium: 0
low: 3
files_reviewed: 12
verdict: APPROVED
---

# Wave 3 Integration Gate — Gate Step D: Security Review (Pass 4)

**Scope:** a7f0d374..e4be29ae (Wave 3.3 fix PRs: #122 W3-FIX-SEC-004, #123 W3-FIX-CODE-005)
**Predecessor review:** gate-step-d-security-review-pass3.md (SHA a7f0d374, verdict: APPROVED_WITH_CONDITIONS)
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-05-02
**Develop SHA:** e4be29ae
**Verdict:** APPROVED — 3 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW — all carry-forward from prior passes, no new findings)

---

## Executive Summary

Wave 3.3 delivers clean resolution of all two MEDIUM conditions from the Pass 3
APPROVED_WITH_CONDITIONS verdict. SEC-P3-001 (inline-table credential bypass in
`sanitize_error_message`) and SEC-P3-002 (`find_snippet_pipe` first-match bias) are both
fully remediated in PR #122 (W3-FIX-SEC-004). SEC-P3-003 (admin-token comparison
not constant-time, carry-forward LOW) is also resolved in the same PR by upgrading all
eight handler sites across four DTU clones to `subtle::ConstantTimeEq::ct_eq`. PR #123
(W3-FIX-CODE-005) extends the Armis dual-mode X-Org-Id guard to five previously
unguarded endpoints (CR-017) and extends the CrowdStrike nil-instance guard to two
detections endpoints (CR-018), closing the sibling-endpoint coverage gap identified in
pass-50 adversarial review as M-50-001.

No new CRITICAL, HIGH, or MEDIUM vulnerabilities are introduced by Wave 3.3. Three LOW
findings carry forward from prior passes (SEC-P3-004, SEC-P3-005, and the pre-existing
`build_network()` wildcard dispatch architectural note). All are deferred and tracked. Wave 3
integration gate step D is unconditionally approved. Phase 4 holdout evaluation may proceed.

---

## Pass-4 Scope: What Changed in Wave 3.3

Two PRs merged after the Pass 3 gate review:

| PR | Story | Purpose |
|----|-------|---------|
| #122 | W3-FIX-SEC-004 | SEC-P3-001/002/003: TOML inline-table redaction + pipe-finder anchor + constant-time admin token |
| #123 | W3-FIX-CODE-005 | CR-016/017/018: poll cadence + Armis sibling-endpoint org-id guards + CrowdStrike detections guards |

---

## Pass-3 Condition Closures — Verification

### Condition 1 — SEC-P3-001: Inline-Table Credential Bypass in `sanitize_error_message`

**Status: RESOLVED (PR #122)**

**Verification methodology:** Read full diff for `validator.rs` in PR #122 and read current
`crates/prism-customer-config/src/validator.rs:351-516`. Read test file
`crates/prism-customer-config/tests/sec_p3_001_inline_table_redaction.rs`.

**Implementation:**

The fix introduces the `content_has_credential_assignment(content: &str) -> bool` helper
(lines 496-516) that scans all ` = ` positions in a content string rather than only the
leading one. For each ` = ` position it extracts the word token immediately to the left
by walking backwards through `before.trim_end_matches([' ', '\t'])` and `rfind` on
non-alphanumeric-non-underscore characters, then calls `is_credential_pattern` on the
extracted token.

Both the snippet-line path (when `find_snippet_pipe` succeeds) and the raw-source-line
path (the `else` branch at line 405) now call `content_has_credential_assignment` instead
of the prior `content.find(" = ")` + check-only-leading-field approach.

**Bypass analysis — `rfind` + 1 byte boundary:**

`before[..word_end].rfind(closure)` returns a char-boundary-aligned index.
The `+ 1` applied to the `rfind` result would only misalign if the separator character
just before the field name is a multi-byte codepoint. TOML field names are `[a-zA-Z0-9_-]`
(ASCII), and the only separator that can appear before a field name in a TOML snippet or
inline table is `{`, `,`, space, or tab — all single-byte ASCII characters. No panic or
incorrect slice is possible in practice.

**Bypass analysis — credential value containing ` = `:**

If a credential value string itself contains ` = ` (e.g., `bearer_token = "val=x=y"`),
`content_has_credential_assignment` will also scan the ` = ` positions within the value
half. The word token to the left of ` = y"` is `"val=x` stripped to... actually since
`rfind` on non-alnum/non-underscore will stop at the `=` in `val=x`, it will extract
`x` as the field candidate. `is_credential_pattern("x")` is false. The extra ` = `
occurrences within values produce only false candidates that fail `is_credential_pattern`.
They do not cause false positives (redacting non-credential fields) and do not prevent
detection of the real credential field name. This edge case is benign.

**Test coverage:** `sec_p3_001_inline_table_redaction.rs` contains 6 test functions:
- `test_AC_001_inline_table_credentials_redacted` — single credential field in inline table
- `test_AC_001_inline_table_multiple_credential_fields_both_redacted` — two credential fields
- `test_AC_002_nested_credentials_in_array_table_redacted` — credential in `[[dtu]]` block
- `test_AC_003_non_credential_inline_field_visible` — non-credential field passes through
- `test_AC_003_single_level_non_credential_not_redacted` — regression: non-cred field not redacted
- `test_AC_001_single_line_credential_regression` — regression: original single-line case unaffected

Coverage is complete for the stated bypass vector and its negation. **SEC-P3-001 VERIFIED CLOSED.**

---

### Condition 2 — SEC-P3-002: `find_snippet_pipe` First-Match Bias

**Status: RESOLVED (PR #122)**

**Verification methodology:** Read diff of `find_snippet_pipe` in PR #122 and current
implementation at `crates/prism-customer-config/src/validator.rs:456-477`. Read test
file `crates/prism-customer-config/tests/sec_p3_002_pipe_anchor.rs`.

**Implementation:**

The new `find_snippet_pipe` iterates all ` | ` occurrences from left to right. For each
candidate position it checks that `&line[..abs]` consists entirely of ASCII digits and
ASCII whitespace (`c.is_ascii_digit() || c.is_ascii_whitespace()`). If the check passes
it returns that position. If not, it advances `i = abs + 1` and continues the search.
Only a ` | ` occurrence preceded exclusively by digits and spaces is accepted.

**Correctness analysis:**

TOML 0.8 error snippets are formatted as `"  N | content"` where N is the source line
number. Caret-marker lines use `"   | ^^^^^"` (spaces only before `|`). Both formats
satisfy the "digits and spaces only" anchor. A raw source line whose value contains
` | ` will always have non-digit, non-space characters before the ` | ` (at minimum
a field name), so the prefix check correctly rejects them.

**Edge case — caret lines:**

A caret line like `"   | ^^^^^^"` has only spaces before ` | `. The anchor
`c.is_ascii_digit() || c.is_ascii_whitespace()` accepts spaces, so caret lines
correctly match the snippet-pipe discriminator. This is the intended behavior —
caret lines are part of the TOML snippet and should be treated as snippet content
(their `content` after ` | ` will be `"^^^^^^"`, which contains no ` = `, so
`content_has_credential_assignment` returns false and the caret line passes through
unmodified). **No bypass and no false positive on caret lines.**

**Empty-line case:**

If `line` is empty or contains no ` | `, `find(" | ")` immediately returns `None`
and the `while` loop exits returning `None`. Correct.

**Test coverage:** `sec_p3_002_pipe_anchor.rs` contains 5 test functions:
- `test_AC_001_credential_value_with_pipe_does_not_break_extraction` — snippet line with ` | ` in value is correctly classified and redacted
- `test_AC_001_raw_source_line_api_key_with_pipe_redacted` — raw source line (no digit prefix) with ` | ` in credential value is redacted via `content_has_credential_assignment`
- `test_AC_002_only_digit_prefix_matches_pipe` — confirms only digit-prefixed ` | ` lines are treated as snippets
- `test_AC_002_caret_lines_not_suppressed_by_anchor` — confirms caret lines are passed through correctly
- `test_AC_001_single_line_credential_through_anchored_pipe_finder` — regression: original single-line credential case works with new anchor

Coverage is complete for all discriminator cases. **SEC-P3-002 VERIFIED CLOSED.**

---

### Condition 3 — SEC-P3-003: Admin Token Non-Constant-Time Comparison (Carry-Forward LOW → RESOLVED)

**Status: RESOLVED (PR #122)**

**Verification methodology:** Read diff of all four DTU route files and grep output
confirming `subtle::ConstantTimeEq` import and `ct_eq` usage at all 8 handler sites
(2 handlers × 4 clones).

**Implementation:**

All four DTU clones now import `subtle::ConstantTimeEq` and use:

```rust
let provided_bytes = provided.unwrap_or("").as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { ... }
```

This pattern replaces the prior `provided != Some(state.admin_token.as_str())` short-circuit
comparison. The `subtle = "2"` dependency (resolved to 2.6.1) is added to all four
`Cargo.toml` files. The pattern is applied to both `dtu_configure`/`post_configure` and
`dtu_reset`/`post_reset` handlers in every clone.

**Length-mismatch constant-time assessment:**

`subtle::ConstantTimeEq` for `[u8]` returns `Choice::from(0)` immediately on length
mismatch — this is not constant-time across different lengths. This is the documented
behavior of `subtle` and is acceptable here: the admin token is a UUID v4 string of
fixed, known length (36 bytes). An attacker cannot learn anything from the early-exit
on length mismatch beyond that their submitted token has a different length than 36 bytes,
which is public information. For equal-length submissions (the actual attack vector),
`ct_eq` operates in constant time. **No exploitable timing oracle remains.**

**Handler coverage confirmation:** All 8 sites confirmed via grep:
- `prism-dtu-armis/src/routes/dtu.rs` lines 48, 85 (post_configure, post_reset)
- `prism-dtu-claroty/src/routes/devices.rs` lines 337, 405 (dtu_configure, dtu_reset)
- `prism-dtu-crowdstrike/src/routes/mod.rs` lines 47, 76 (dtu_configure, dtu_reset)
- `prism-dtu-slack/src/routes/dtu.rs` lines 43, 80 (post_configure, post_reset)

**Test coverage:** `sec_p3_003_constant_time_admin_token.rs` (in `prism-dtu-claroty/tests/`)
contains 7 tests confirming behavioral contract is unchanged (401 on absent/wrong token,
200 on correct token) with the new ct_eq path. Claroty's test suite is representative; the
pattern is structurally identical across all four clones. **SEC-P3-003 VERIFIED CLOSED.**

---

## Pass-3 Adversarial Finding Closures (M-50-001 / CR-017/018)

### CR-017 / M-50-001: Armis Sibling Endpoint X-Org-Id Coverage Gap

**Status: RESOLVED (PR #123)**

Pass-50 adversarial review identified that the Armis dual-mode X-Org-Id guard applied
in PR #118 covered only `get_or_post_devices` and `post_devices` in `devices.rs`, but
not the five other Armis org-keyed endpoints: `tags.rs::post_device_tag`,
`tags.rs::delete_device_tag`, `alerts.rs::get_alerts`, `devices.rs::get_device_activity`,
and `devices.rs::get_device_risk`.

**Verification:** Read diff of `crates/prism-dtu-armis/src/routes/tags.rs`,
`alerts.rs`, and `devices.rs`. Confirmed that all five endpoints now apply the same
dual-mode pattern:

```rust
let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
if is_real_org || headers.get("x-org-id").is_some() {
    if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body_err).into_response();
    }
}
```

The implementation reuses `validate_org_id` from `devices.rs` via
`use crate::routes::devices::validate_org_id`, consistent with the existing pattern.

Module-level doc comments in `tags.rs` and `alerts.rs` document the dual-mode policy,
citing CR-017, M-50-001, and BC-3.5.002 precondition 3. **No new attack surface introduced.**

**Test coverage:** `cr017_tag_alert_org_id_guard.rs` contains 10 test functions covering
absent-header/real-org (401), correct-header (200), mismatched-header (401), and
default-instance-absent-header (200/201) for both tags and alerts endpoints.

---

### CR-018: CrowdStrike Detections Endpoint Org-Id Coverage Gap

**Status: RESOLVED (PR #123)**

Pass-50 adversarial review identified that `detections.rs::list_detection_ids` and
`get_detection_summaries` lacked the nil-instance guard applied to hosts endpoints.

**Verification:** Read diff of `crates/prism-dtu-crowdstrike/src/routes/detections.rs`.
Both endpoints now apply:

```rust
if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
    if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
        return (status, body).into_response();
    }
}
```

The CrowdStrike sentinel (`OrgId::from_uuid(Uuid::nil())`) is consistent with
`CrowdStrikeClone::new()` state construction. The guard reuses `validate_org_id` from
`crates/prism-dtu-crowdstrike/src/routes/hosts.rs`, consistent with the existing pattern.

**Test coverage:** `cr018_detections_org_id_guard.rs` contains 9 test functions covering
the three guard states (absent-with-real-org → 401, correct → 200, nil-instance → skip).

---

## Carry-Forward LOW Findings (No Change)

### SEC-P3-004 (Carry-Forward) — OrgSlug 64-char Limit (ADR-006 OQ-1)

- **Severity:** LOW
- **CWE:** CWE-20 (Improper Input Validation)
- **File:** `crates/prism-core/src/tenant.rs` (`ORG_SLUG_PATTERN`)
- **Status:** Deferred — not addressed in Wave 3.3; unchanged from Pass 3.

ADR-006 §8 OQ-1 remains unresolved. Config-layer enforcement via E-CFG-019 (PR #120) is
in place. Core `ORG_SLUG_PATTERN = r"^[a-zA-Z0-9_-]{1,64}$"` unchanged.
Disposition: **PARTIALLY MITIGATED** (unchanged from Pass 3).

---

### SEC-P3-005 (Carry-Forward) — `org_slug` Cross-Check Operational Observability

- **Severity:** LOW
- **CWE:** CWE-345 (Insufficient Verification of Data Authenticity)
- **File:** `crates/prism-audit/src/audit_emitter.rs`
- **Status:** Deferred — not addressed in Wave 3.3; unchanged from Pass 3.

`validate_org_slug_cross_check` result is `let _ = ...`. The `tracing::warn!` fires for
Mismatched/OrgNotInRegistry but there is no structured metrics counter. Acceptable for
harness context. Disposition: **PARTIALLY MITIGATED** (unchanged from Pass 3).

---

### SEC-P3-006 (Architectural Note, Carry-Forward) — `build_network()` Wildcard Dispatch

- **Severity:** LOW
- **CWE:** CWE-284 (Improper Access Control) — architectural quality gap
- **File:** `crates/prism-dtu-harness/src/builder.rs`
- **Status:** Pre-existing; not introduced in Wave 3.3; no change.

`build_network()` uses `_ => start_clone_network()` for non-Armis/CrowdStrike/Cyberint
types. `start_clone_network()` permits unauthenticated reads (by design for test harness).
ADR-011 §2.3 scopes this to test-harness-only. Not exploitable beyond loopback in the
current deployment model. Disposition: **ACCEPTED** (test-harness scope, pre-existing design).

---

## CWE/OWASP Coverage Assessment

| CWE | Area | Pass-4 Status |
|-----|------|--------------|
| CWE-22 (Path Traversal) | `validate_spec_path` pre-join unconditional checks | MITIGATED (closed in Wave 3.2 PR #118) |
| CWE-20 (Input Validation) | Org slug pattern, AQL validator | MITIGATED (E-CFG-018/019; AQL HIGH-002 closed Wave 2) |
| CWE-208 (Timing Side-Channel) | Admin token `ct_eq` | MITIGATED (closed in Wave 3.3 PR #122) |
| CWE-209 (Info Exposure via Error Messages) | TOML redaction — inline-table + pipe-anchor | MITIGATED (closed in Wave 3.3 PR #122) |
| CWE-306 (Missing Authentication) | `POST /dtu/reset` all four clones | MITIGATED (closed in Wave 3.2 PR #119) |
| CWE-284 (Improper Access Control) | Cross-org credential guard, X-Org-Id guards | MITIGATED (all endpoints covered Wave 3.2-3.3) |
| CWE-345 (Insufficient Verification) | Org slug audit cross-check | PARTIALLY MITIGATED (observability gap tracked, SEC-P3-005) |
| CWE-863 (Incorrect Authorization) | `AdapterRegistry` OrgIdMismatch guard | MITIGATED (closed Wave 3 baseline) |

---

## Tenant Isolation End-to-End Assessment

**Question:** With all Wave 3.3 fixes applied, can a malicious caller bypass any isolation boundary?

**Assessment:** No exploitable cross-tenant path identified. All isolation boundaries at
`e4be29ae`:

| Boundary | Defense | Status |
|----------|---------|--------|
| HTTP layer — X-Org-Id spoofing | `validate_org_id` on all Armis org-keyed endpoints (devices, tags, alerts, activity, risk) + Claroty + CrowdStrike (hosts, detections) | MITIGATED (Wave 3.3 completes coverage) |
| HTTP layer — unauthenticated reset | `X-Admin-Token` with `ct_eq` on `POST /dtu/reset` all 4 clones | MITIGATED |
| HTTP layer — unauthenticated configure | `X-Admin-Token` with `ct_eq` on `POST /dtu/configure` all 4 clones | MITIGATED |
| Query dispatch — cross-tenant adapter | `(OrgId, SensorType)` composite key; `OrgIdMismatch` guard before I/O | MITIGATED |
| Credential isolation — keyring | `namespace_key_by_org_id("{uuid}/{sensor}/{name}")` — physical separation | MITIGATED |
| Config layer — spec path traversal | Pre-join `..` + absolute-path checks unconditional; post-join canonical prefix check | MITIGATED |
| Config layer — credential redaction | Single-line, multi-line `"""`, inline-table, and ` \| `-in-value cases all covered | MITIGATED |
| Audit layer — org_slug integrity | `validate_org_slug_cross_check` wired into `AuditEmitterService::call()` | MITIGATED (observability gap LOW — SEC-P3-005) |

---

## Error Taxonomy Verification

Error-taxonomy.md v1.13 confirmed to include:
- `E-CFG-018`: SpecPathTraversal (HIGH, CWE-22) — introduced by W3-FIX-SEC-003 PR #114
- `E-CFG-019`: InvalidOrgSlugPattern (MEDIUM, CWE-20) — introduced by W3-FIX-CODE-002 PR #120

Both error codes are present at lines 119-120. The BLOCKING consistency finding
ADV-W3GATE-P50-MED-001 (E-CFG-018/019 absent from taxonomy) is **resolved**.

---

## New Dependency Advisory Check (Wave 3.3)

PR #122 adds `subtle = "2"` (resolved to 2.6.1) to four DTU crate `Cargo.toml` files.

- `subtle 2.6.1`: No known CVEs. RUSTSEC advisory database: clean. The `subtle` crate
  is the standard Rust constant-time comparison library, maintained by the
  RustCrypto organization. No supply-chain concerns.

PR #123 adds no new dependencies.

---

## Positive Findings (Defensive Measures Present)

- **TOML inline-table credential redaction (SEC-P3-001):** `content_has_credential_assignment` correctly scans all ` = ` positions per line, catching credential field names embedded inside inline TOML tables. Both snippet-line and raw-source-line paths are covered. 6 regression tests in `sec_p3_001_inline_table_redaction.rs`.
- **Anchored snippet-pipe discriminator (SEC-P3-002):** `find_snippet_pipe` now requires a digits-and-spaces-only prefix before ` | `, eliminating the false-match on user-controlled field values containing ` | `. 5 regression tests in `sec_p3_002_pipe_anchor.rs`.
- **Constant-time admin token comparison (SEC-P3-003):** All 8 handler sites across 4 DTU clones (`dtu_configure` and `dtu_reset` on Armis, Claroty, CrowdStrike, Slack) upgraded to `subtle::ConstantTimeEq::ct_eq`. `subtle = "2"` (v2.6.1, RustCrypto-maintained) added to each crate.
- **Armis sibling-endpoint X-Org-Id coverage (CR-017):** Dual-mode guard extended to `post_device_tag`, `delete_device_tag`, `get_alerts`, `get_device_activity`, and `get_device_risk` — the five Armis org-keyed endpoints that lacked the guard after PR #118. Reuses the existing `validate_org_id` function with no code duplication.
- **CrowdStrike detections org-id coverage (CR-018):** Nil-instance guard extended to `list_detection_ids` and `get_detection_summaries`, matching the pattern already applied to hosts endpoints.
- **Error taxonomy completeness:** E-CFG-018 (SpecPathTraversal, CWE-22) and E-CFG-019 (InvalidOrgSlugPattern, CWE-20) confirmed present in error-taxonomy.md v1.13, closing the BLOCKING consistency finding ADV-W3GATE-P50-MED-001.
- **Complete tenant isolation boundary coverage:** As of `e4be29ae`, every org-keyed HTTP endpoint across all four DTU clones carries an appropriate X-Org-Id guard. No unguarded org-keyed endpoint remains.

---

## Recommendations Priority

### Immediate (before merge)

None. All CRITICAL and HIGH findings are closed. No blocking conditions exist.

### Before Release

1. **SEC-P3-004 (LOW, CWE-20):** Resolve ADR-006 OQ-1 — evaluate tightening `ORG_SLUG_PATTERN` max length from 64 to 32 characters if no existing fixture exceeds 32 chars. The E-CFG-019 config-layer check is in place; this is a core-type hygiene item.
2. **SEC-P3-005 (LOW, CWE-345):** Add a structured metrics counter or `tracing::event!` span for `SlugCheckResult::Mismatched` and `OrgNotInRegistry` in `validate_org_slug_cross_check` to enable operational monitoring before production deployment.

### Post-Release

3. **SEC-P3-006 (LOW, CWE-284):** Refactor `build_network()` to use an exhaustive match (no `_ =>` arm) consistent with `start_clone()`. This is an architectural quality improvement; the current behavior is correct for test-harness scope per ADR-011 §2.3.

---

## Summary Table

| ID | Severity | CWE | Location | Origin | Pass-4 Status |
|----|----------|-----|----------|--------|--------------|
| SEC-P3-001 | MEDIUM → **CLOSED** | CWE-209 | `prism-customer-config/validator.rs` | Pass-3 | RESOLVED (PR #122) |
| SEC-P3-002 | MEDIUM → **CLOSED** | CWE-209 | `prism-customer-config/validator.rs` | Pass-3 | RESOLVED (PR #122) |
| SEC-P3-003 | LOW → **CLOSED** | CWE-208 | 4 DTU dtu_reset + dtu_configure handlers | Pass-3 | RESOLVED (PR #122) |
| SEC-P3-004 | **LOW** | CWE-20 | `prism-core/src/tenant.rs` | Pass-1 | Deferred (unchanged) |
| SEC-P3-005 | **LOW** | CWE-345 | `prism-audit/audit_emitter.rs` | Pass-2 | Partially Mitigated (unchanged) |
| SEC-P3-006 | **LOW** | CWE-284 | `prism-dtu-harness/src/builder.rs` | Pass-3 (architectural note) | Accepted (test-harness scope) |

**No CRITICAL, HIGH, or MEDIUM open findings.**

---

## Risk Register Dispositions (Security-Category R-NNN Entries)

| Risk / ADR Reference | Pass-3 Status | Pass-4 Status | Change in Wave 3.3 |
|----------------------|--------------|--------------|-------------------|
| `POST /dtu/reset` unauthenticated (CWE-306) | Mitigated | **Mitigated** | No change (ct_eq hygiene improvement applied to same handlers). |
| Armis X-Org-Id header-presence conditional (CWE-284) | Mitigated | **Mitigated** | CR-017: guard extended to 5 additional Armis endpoints (tags, alerts, activity, risk). Stronger than Pass 3. |
| Pre-join path traversal bypass (CWE-22) | Mitigated | **Mitigated** | No change. |
| TOML credential redaction — multi-line + inline-table + pipe-value (CWE-209) | Partially Mitigated | **Mitigated** | Inline-table bypass (SEC-P3-001) and pipe-anchor bias (SEC-P3-002) both resolved. |
| `org_slug` audit cross-check (CWE-345) | Mitigated | **Mitigated** | No change. Observability gap (SEC-P3-005) remains LOW. |
| `init_registry` deprecation enforcement (CWE-284) | Mitigated | **Mitigated** | No change. |
| OrgSlug 64-char limit / ADR-006 OQ-1 (CWE-20) | Partially Mitigated | **Partially Mitigated** | No change. Core pattern unchanged. |
| Cross-tenant data leakage at adapter layer (ADR-006 §3.1) | Mitigated | **Mitigated** | CR-017/018: additional endpoints covered; no regression. |
| Cross-tenant credential reachability (ADR-006 §3.2) | Mitigated | **Mitigated** | No change. |
| Path traversal in spec file loading (R-CUST-014/015) | Mitigated | **Mitigated** | No change. |

---

## Verdict

**APPROVED**

Wave 3.3 delivers clean, unconditional closure of all MEDIUM conditions from the Pass 3
APPROVED_WITH_CONDITIONS verdict. The two TOML redaction bypasses (SEC-P3-001:
inline-table credentials, SEC-P3-002: pipe-finder first-match bias) are fully remediated
with correct implementations, comprehensive test coverage (6+5 tests respectively), and
no residual bypass vectors identified. The admin token timing oracle concern (SEC-P3-003)
is eliminated via `subtle::ConstantTimeEq` across all eight handler sites on all four DTU
clones. The Armis and CrowdStrike sibling-endpoint org-id coverage gaps (CR-017/018)
identified in pass-50 adversarial review are fully closed.

There are no open CRITICAL, HIGH, or MEDIUM findings. Three LOW findings remain, all
deferred: OrgSlug 64-char ADR-006 OQ-1 (SEC-P3-004), audit cross-check observability
gap (SEC-P3-005), and the `build_network()` wildcard dispatch architectural note
(SEC-P3-006). None block wave progression or Phase 4 evaluation.

Wave 3 integration gate step D (Pass 4) is **unconditionally approved**. This is
clean pass 1 of 3 toward the convergence window.
