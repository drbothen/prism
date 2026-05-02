---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: a7f0d374..e4be29ae (W3-FIX-SEC-004 PR #122 + W3-FIX-CODE-005 PR #123)
reviewer: vsdd-factory:code-reviewer
develop_sha: e4be29ae
date: 2026-05-02
phase: 3
wave: 3
step: c
pass: 4
previous_review: gate-step-c-code-review-pass3.md
verdict: APPROVE_WITH_CONCERNS
total_findings: 3
high: 0
medium: 1
low: 2
---

# Wave 3 Integration Gate — Gate Step C: Code Review (Pass 4, post-W3.3)

**Scope:** `a7f0d374..e4be29ae` (W3-FIX-SEC-004 PR #122 + W3-FIX-CODE-005 PR #123)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-02
**Previous review:** `gate-step-c-code-review-pass3.md` (pass 3, SHA `a7f0d374`)
**Verdict:** APPROVE_WITH_CONCERNS — all three pass-3 MEDIUM findings are resolved.
One new MEDIUM finding (Cyberint `post_reset` admin token gap — a pass-3 verification
error now confirmed) and two LOW findings. No new HIGH or CRITICAL findings.

---

## Part A — Fix Verification (pass-3 findings)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| CR-016 | MEDIUM | RESOLVED | All three clone-specific `poll_test_hook` mirrors updated to 50ms: `clones/armis.rs:899`, `clones/claroty.rs:848`, `clones/crowdstrike.rs:1159`. Each carries the `// CR-016: 50ms cadence per CR-006 closure; TD-W3-POLL-NOTIFY-001 follow-up` comment, consistent with `clone_server.rs:838`. |
| CR-017 | MEDIUM | RESOLVED | Armis dual-mode `is_real_org` guard applied to all remaining org-keyed handlers: `tags.rs::post_device_tag` (line 65), `tags.rs::delete_device_tag` (line 105), `alerts.rs::get_alerts` (line 54). Guard also applied to `devices.rs::get_device_activity` (line 205) and `devices.rs::get_device_risk` (line 241) — two pre-existing endpoints not covered in pass-3. New test file `cr017_tag_alert_org_id_guard.rs` provides 8 tests covering tags (5) and alerts (3). Test coverage for activity/risk endpoints is partially absent — noted as CR-023. |
| CR-018 | MEDIUM | RESOLVED | CrowdStrike nil-instance guard applied to `detections.rs::list_detection_ids` (line 120) and `detections.rs::get_detection_summaries` (line 208). Uses the correct `OrgId::from_uuid(uuid::Uuid::nil())` sentinel (not `DTU_DEFAULT_INSTANCE_ORG_ID`). New test file `cr018_detections_org_id_guard.rs` provides 6 tests: reject/accept/backward-compat x 2 endpoints. Complete coverage. |
| CR-019 | LOW | RESOLVED | `find_snippet_pipe` now anchored: `validator.rs:456-477`. The implementation scans iteratively and returns only positions where the entire prefix before the pipe separator is ASCII digits and/or ASCII whitespace. The loop advances `i = abs + 1` past each non-matching occurrence; UTF-8 safety verified — space (0x20) is always a single-byte ASCII codepoint so `abs + 1` is always a valid slice boundary. |
| CR-020 | LOW | RESOLVED | Visibility deviation documented at `validator.rs:813-822` under `# Visibility deviation (CR-020)`. The comment explains the `pub` vs `pub(crate)` choice, cites the integration test requirement, and notes the deferred refactor. |

---

## Part B — Findings

### CR-021: Cyberint `post_reset` has no `X-Admin-Token` gate (SEC-NEW-001 incorrectly closed)

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-cyberint/src/routes/dtu.rs:61-72`
- **BC Reference:** BC-3.5.002 precondition 6; W3-FIX-SEC-002 AC-001 (SEC-NEW-001)
- **Description:** Pass-3 marked SEC-NEW-001 RESOLVED and cited "Cyberint (`dtu.rs:37`)" as
  evidence. Line 37 is inside `post_configure`, not `post_reset`. The `post_reset` function
  begins at line 61 and contains no admin token check — it accepts any caller without
  authentication. W3-FIX-SEC-002 (PR #119) applied the admin token gate to Armis, Claroty,
  CrowdStrike, and Slack, but not Cyberint. The commit stat for `f89e7044` confirms:
  `prism-dtu-cyberint` is absent from the changed-file list.

  No `dtu_reset_auth.rs` test file exists for Cyberint. The existing
  `ac_8_reset_semantics.rs` calls `POST /dtu/reset` without an `X-Admin-Token` header and
  expects HTTP 200, confirming the endpoint is and always has been unauthenticated.

  The other three DTUs (Armis, Claroty, CrowdStrike) and Slack all gate `POST /dtu/reset`
  behind `X-Admin-Token`. The Cyberint omission creates an inconsistent security posture:
  any caller who knows the Cyberint DTU URL can reset all accumulated multi-tenant state,
  disrupting test isolation invariants (BC-3.5.001 invariant 3) without presenting a token.

  Note: Cyberint's `post_reset` design differs from the other three — it accepts an optional
  `X-Prism-Org-Id` header for scoped resets. This is a separate feature; the admin token
  gate should be added as a prerequisite check before the org-scope branch, exactly as
  W3-FIX-SEC-002 did for Armis and Slack.
- **Evidence:**
  ```rust
  // crates/prism-dtu-cyberint/src/routes/dtu.rs:61-72
  pub async fn post_reset(
      State(state): State<Arc<CyberintState>>,
      headers: HeaderMap,
  ) -> impl IntoResponse {
      if headers.contains_key("x-prism-org-id") {
          let org_id = extract_org_id(&headers, state.instance_org_id);
          state.reset_for(org_id);
      } else {
          state.reset();
      }
      (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
  }
  // No admin token check. Contrast with Armis post_reset (dtu.rs:61-73):
  //   let provided = headers.get("x-admin-token")...
  //   let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
  //   if !valid { return (StatusCode::UNAUTHORIZED, ...) }
  ```
  W3-FIX-SEC-002 commit stat (`git show f89e7044 --stat`): no `prism-dtu-cyberint` files
  listed. No `tests/dtu_reset_auth.rs` exists in the crate.
- **Proposed Fix:** Add admin token gate to `post_reset` following the Armis pattern
  (same `subtle::ConstantTimeEq` comparison used for `post_configure`), then add
  `tests/dtu_reset_auth.rs` with AC-001/AC-002/AC-003 mirroring the other four crates.
  Update `ac_8_reset_semantics.rs` to supply the admin token. ~25 lines of production code
  and ~130 lines of test (matching the other crates' test files).

---

### CR-022: Cyberint `post_configure` still uses short-circuit `!=` for admin token (SEC-P3-003 gap)

- **Severity:** LOW
- **Category:** pattern-consistency
- **Location:** `crates/prism-dtu-cyberint/src/routes/dtu.rs:37-43`
- **BC Reference:** BC-3.5.002 precondition 6; W3-FIX-SEC-004 AC-003 (SEC-P3-003 / CWE-208)
- **Description:** W3-FIX-SEC-004 (PR #122) migrated the admin token comparison to
  `subtle::ConstantTimeEq::ct_eq` in Armis, Claroty, CrowdStrike, and Slack (all 8 handler
  sites, per commit message). Cyberint was not included in the scope. The story's AC-003
  table lists only those four crates; Cyberint is absent without explanation.

  Cyberint's `post_configure` retains the pre-fix pattern:
  `if provided != Some(state.admin_token.as_str())`. This is the exact comparison that
  SEC-P3-003 identified as potentially leaking timing information (CWE-208).

  While the theoretical timing attack risk is low (Cyberint tokens are UUID v4 strings and
  the attack surface requires network-level timing measurement), consistency with the rest
  of the codebase demands the same protection. The `#![deny(deprecated)]` pattern
  established in `prism-sensors/src/lib.rs` sets a precedent for uniform policy enforcement.
- **Evidence:**
  ```rust
  // crates/prism-dtu-cyberint/src/routes/dtu.rs:37-38 (unchanged)
  let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
  if provided != Some(state.admin_token.as_str()) {
  ```
  Contrast with Armis `dtu.rs:44-48` (after W3-FIX-SEC-004):
  ```rust
  let provided_bytes = provided.unwrap_or("").as_bytes();
  let expected_bytes = state.admin_token.as_bytes();
  let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
  if !valid {
  ```
  `grep -r "ct_eq" crates/prism-dtu-cyberint/` returns no results; `subtle` is absent from
  `Cargo.toml`.
- **Proposed Fix:** Replace the `!=` comparison in `post_configure` with the
  `subtle::ConstantTimeEq` pattern, add `subtle = "2"` to `[dependencies]` in `Cargo.toml`,
  and add the same pattern to `post_reset` once CR-021 is resolved (the two fixes are
  naturally bundled). ~10 lines of change, no behavioral difference.

---

### CR-023: `get_device_activity` and `get_device_risk` received org-id guard but lack test coverage

- **Severity:** LOW
- **Category:** maintainability
- **Location:**
  - `crates/prism-dtu-armis/src/routes/devices.rs:195-225` (`get_device_activity`)
  - `crates/prism-dtu-armis/src/routes/devices.rs:227-265` (`get_device_risk`)
- **BC Reference:** BC-3.5.002 precondition 3; W3-FIX-CODE-005 CR-017 closure
- **Description:** W3-FIX-CODE-005 correctly extended the `is_real_org` dual-mode guard to
  `get_device_activity` and `get_device_risk` as part of the CR-017 closure. Both handlers
  now have the guard (lines 205-209 and 241-245 respectively). However, the new test file
  `cr017_tag_alert_org_id_guard.rs` covers only `post_device_tag` (3 tests),
  `delete_device_tag` (2 tests), and `get_alerts` (3 tests). No tests exercise the guard on
  `GET /api/v1/devices/:device_id/activity` or `GET /api/v1/devices/:device_id/risk`.

  This is a test coverage gap, not a production correctness issue. The guard implementation
  in both handlers is identical to the well-tested `get_or_post_devices` pattern. The risk
  is that future changes to these handlers could silently remove or regress the guard without
  a failing test to catch it.

  The gap was not present in the original CR-017 scope (the pass-3 finding identified only
  tags.rs and alerts.rs), but the fix author extended the scope to include activity/risk
  without extending the test scope.
- **Evidence:**
  Guard at `devices.rs:205-209`:
  ```rust
  // CR-017 / M-50-001: dual-mode X-Org-Id policy (see module doc).
  let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
  if is_real_org || headers.get("x-org-id").is_some() {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
  ```
  `cr017_tag_alert_org_id_guard.rs` test inventory (8 tests):
  `test_post_device_tag_*` (3), `test_delete_device_tag_*` (2), `test_get_alerts_*` (3).
  No `test_get_device_activity_*` or `test_get_device_risk_*` functions exist.
- **Proposed Fix:** Add 3 tests per endpoint to `cr017_tag_alert_org_id_guard.rs` (or a
  new `cr023_activity_risk_org_id_guard.rs` file): real-org absent-header → 401,
  real-org correct-header → 200, default-instance absent-header → 200. This is ~50 lines
  of test code per endpoint mirroring the existing `test_get_alerts_*` structure.

---

## Positive Observations (Non-Finding)

**CR-016 fix is complete and idiomatic.** All three mirror functions are now 50ms with
consistent comments referencing the TD-W3-POLL-NOTIFY-001 follow-up. The fix is symmetric
with `clone_server.rs:838` and closes the 10→50ms migration gap cleanly.

**CR-017 fix is broader than originally required.** The fix correctly extends guard coverage
to `get_device_activity` and `get_device_risk` in addition to the specified tags and alerts
endpoints, eliminating the guard asymmetry that existed since W3-FIX-SEC-001. The dual-mode
semantics (real-org vs default-instance) are consistent across all five affected handlers.

**CR-018 fix uses the correct sentinel.** The `OrgId::from_uuid(uuid::Uuid::nil())` sentinel
is distinct from Armis's `DTU_DEFAULT_INSTANCE_ORG_ID`, which is architecturally correct —
the two DTU clones have different nil-instance conventions and must not share sentinels.
The in-code comment at `detections.rs:119` explicitly documents this boundary, preventing
future confusion.

**CR-019 fix is robust.** The iterative `find_snippet_pipe` implementation correctly handles
the case where a line contains multiple ` | ` occurrences (e.g., a credential value that also
contains the pipe sequence). The UTF-8 boundary safety of `abs + 1` holds because space
(0x20) is always a single-byte codepoint. The `content_has_credential_assignment` helper
correctly handles inline TOML tables by scanning all ` = ` positions, and the word-boundary
extraction via `rfind(|c| !alphanumeric && != '_')` is correct for TOML identifier syntax.

**CR-020 doc comment is comprehensive.** The `# Visibility deviation (CR-020)` section at
`validator.rs:813-822` explains the `pub` retention clearly, cites the constraint, and
mentions the deferred refactor path. This is the right level of documentation for a
deliberate deviation from a story acceptance criterion.

**W3-FIX-SEC-004 constant-time migration is complete for 4/5 DTU clones.** The `subtle`
crate is now correctly added to Armis, Claroty, CrowdStrike, and Slack. The `ct_eq` pattern
is consistently applied to both `dtu_configure` and `dtu_reset` in each crate (8 sites
total). The `sec_p3_003_constant_time_admin_token.rs` test suite in `prism-dtu-claroty`
provides behavioral regression coverage with a well-documented rationale for why a
deterministic wall-clock timing test is omitted.

---

## Summary of Open Items

| ID | Severity | Status | Description |
|----|----------|--------|-------------|
| CR-021 | MEDIUM | NEW | Cyberint `post_reset` has no admin token gate (SEC-NEW-001 was incorrectly closed in pass-3) |
| CR-022 | LOW | NEW | Cyberint `post_configure` uses non-constant-time `!=` comparison (SEC-P3-003 scope omission) |
| CR-023 | LOW | NEW | Armis `get_device_activity` and `get_device_risk` received org-id guard but lack tests |
| CR-007 | LOW | DEFERRED | archetype/scale declared but unread in build() — Wave 4 |
| CR-008 | LOW | DEFERRED | Placeholder CloneState sentinel strings — Wave 4 |
| CR-009 | LOW | DEFERRED | Wall-clock startup assertion — Wave 4 |

---

## Convergence Verdict

`findings remain -- iterate`

The three pass-3 MEDIUM findings (CR-016, CR-017, CR-018) are fully resolved and the two
pass-3 LOW findings (CR-019, CR-020) are also closed. The W3-FIX-SEC-004 and W3-FIX-CODE-005
PRs are net-positive: the guard coverage is now symmetric across Armis endpoints, and
the TOML redaction logic is hardened.

However, a new MEDIUM finding (CR-021) surfaced: Cyberint `post_reset` was never gated
behind `X-Admin-Token`, and pass-3's RESOLVED verdict for SEC-NEW-001 was based on an
incorrect line citation (`dtu.rs:37` is `post_configure`, not `post_reset`). This is a
genuine security gap — any caller can reset Cyberint multi-tenant state without a token —
that is inconsistent with the other four DTU clones. It should be resolved before declaring
the wave gate APPROVE.

CR-022 (Cyberint `post_configure` non-constant-time comparison) and CR-023 (missing
activity/risk guard tests) are LOW severity and may be bundled with the CR-021 fix or
deferred to a W3.4 hygiene story at the team's discretion.
