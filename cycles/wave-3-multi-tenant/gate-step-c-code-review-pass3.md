---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: 6696e374^..a7f0d374 (Wave 3 + 3.1 + 3.2, ~49 commits; pass-3 focuses on W3.2 PRs #118-#121)
reviewer: vsdd-factory:code-reviewer
develop_sha: a7f0d374
date: 2026-05-01
phase: 3
wave: 3
step: c
pass: 3
previous_review: .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass2.md
verdict: APPROVE_WITH_CONCERNS
total_findings: 5
high: 0
medium: 3
low: 2
---

# Wave 3 Integration Gate — Gate Step C: Code Review (Pass 3, post-W3.2)

**Scope:** `6696e374^..a7f0d374` (Wave 3 + 3.1 + 3.2, ~49 commits; focus on W3.2 PRs #118–#121)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-01
**Previous review:** `gate-step-c-code-review-pass2.md` (pass 2, SHA `cda17ed4`)
**Verdict:** APPROVE_WITH_CONCERNS — all pass-2 findings either resolved or acknowledged.
Three new MEDIUM findings and two LOW findings introduced by the W3.2 mass merge.
No new HIGH or CRITICAL findings. W3.2 is a net positive: twelve items
from passes 1 and 2 are correctly closed.

---

## Part A — Fix Verification (pass-2 findings)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| CR-003 | MEDIUM | RESOLVED | `validate_structural` now calls `OrgSlug::new(&config.org_slug).is_err()` at line 519 of `validator.rs`. `ConfigError::InvalidOrgSlugPattern` is defined in `error.rs:137`. Four regression tests in `tests/cr003_slug_pattern.rs` cover space, unicode, dot, and length > 64. E-CFG-019 is correctly emitted. |
| CR-004 | MEDIUM | RESOLVED | `build_network()` in `builder.rs` uses an exhaustive `match dtu_type` at line 707 with explicit arms for `CrowdStrike`, `Cyberint`, and `Armis`, plus a `_` catchall for the generic stub. `Armis` correctly dispatches to `start_armis_clone_network` at line 727. Logical-mode `start_clone` in `clone_server.rs` also uses an exhaustive `match` pattern. Compile-time exhaustiveness enforced. |
| CR-005 | MEDIUM | RESOLVED | `validate_all` is `pub(crate)` at `validator.rs:122`. The doc comment explains the visibility rationale (CR-005 / W3-FIX-CODE-002). |
| CR-006 | MEDIUM | PARTIALLY_RESOLVED | `clone_server.rs:838` correctly changed from 10ms to 50ms with a comment noting the Notify follow-up. However, three clone-specific mirror functions were not updated: `poll_armis_test_hook` at `clones/armis.rs:901`, `poll_claroty_test_hook` at `clones/claroty.rs:850`, and `poll_test_hook_crowdstrike` at `clones/crowdstrike.rs:1161` all still sleep 10ms. These three mirrors combine for ~300 additional wake-ups/second in a 12-clone harness. Recorded as CR-016 below. |
| CR-010 | MEDIUM | RESOLVED | Module-level doc at `harness.rs:18-22` now reads the correct post-CR-002 description with no `handle.abort()` reference. |
| CR-011 | MEDIUM | RESOLVED | `with_failure` in `builder.rs:242` uses `initial_failure.remove(&dtu_type)` on `FailureMode::None`; the deferred drain at line 320 applies the same pattern. `is_empty()` now correctly returns `true` after clearing, preventing the spurious configure call. |
| CR-012 / SEC-P2-001 | MEDIUM | PARTIALLY_RESOLVED | The Armis `devices.rs` handlers now use a dual-mode `is_real_org \|\| header_present` guard, which satisfies the core security requirement: real-org clones reject absent `X-Org-Id`. However, (a) the fix was not applied to `tags.rs` (`post_device_tag`, `delete_device_tag`) and `alerts.rs` (`get_alerts`) as AC-003 required, and (b) tests only cover `GET /api/v1/devices`, not POST devices or tag endpoints. Recorded as CR-017 below. |
| CR-013 | MEDIUM | RESOLVED | `fanout.rs:366–370` has the exact `debug_assert_eq!` specified by the story, including both UUIDs in the message and the BC-3.2.001 precondition 4 reference. |
| CR-014 | LOW | DEVIATION ACCEPTED | `validate_spec_path` remains `pub` with `#[doc(hidden)]` rather than `pub(crate)`. The AC-005 story spec stated "no external callers" but `tests/path_traversal.rs` (an integration test binary, external to `src/`) calls it directly. Rust's `pub(crate)` excludes integration test binaries, so `pub` is the correct choice here. The `#[doc(hidden)]` attribute prevents accidental stable-API coupling. The deviation is architecturally sound and the comment at line 739 explains the rationale. |
| CR-015 | LOW | RESOLVED | `validate_org_id` removed from `crates/prism-dtu-cyberint/src/routes/alerts.rs`. The module-level doc comment at lines 11–21 now explains the session-routing architecture and the intentional deviation from the other three DTUs (W3-FIX-CODE-004 AC-006 Option A selected). |
| SEC-P2-002 | MEDIUM | RESOLVED | The pre-join `..` component scan (line 638–649 in `validate_dtu_block`) and absolute-path rejection (line 652–662) both fire before `resolved.exists()` at line 667. Ordering is correct: (1) reject `..`, (2) reject absolute, (3) existence check, (4) canonicalize + prefix. |
| SEC-P2-006 | LOW | RESOLVED | `prism-sensors/src/lib.rs:32` has `#![deny(deprecated)]` with the explanatory comment. |
| BC-3.5.001/002 timing | N/A | RESOLVED | Three tests in `network_isolation_test.rs` (lines 610, 647) and one in `logical_isolation_test.rs` (line 335) are `#[ignore]` with `TD-W3-TIMING-001` reference. Total: 3 tests gated as required. |
| BC-3.2.002 trait coverage | N/A | RESOLVED | `tests/bc_3_2_002_trait_impl.rs` has 7 test functions covering AC-001 through AC-006 (with AC-003 split into two tests for delete + double-delete). |
| SEC-006 | MEDIUM | RESOLVED | `sanitize_error_message` handles triple-quoted credential continuations. The `in_multiline_cred` state machine at lines 365–407 correctly enters on `field = """`, redacts all continuation lines, and exits on the closing `"""`. Tests in `tests/sec006_toml_multiline_redaction.rs` cover the multi-line case. |
| SEC-007 | MEDIUM | RESOLVED | `validate_org_slug_cross_check` is implemented in `prism-audit/src/org_slug_guard.rs` and wired into `AuditEmitterService::call()` at line 296 of `audit_emitter.rs`. The call correctly passes `Arc<OrgRegistry>` (live registry). Non-Matched results emit `tracing::warn!` and do not abort emission. Tests in `tests/sec007_org_slug_cross_check.rs` cover Matched, Mismatched, and OrgNotInRegistry cases. |
| SEC-NEW-001 | HIGH | RESOLVED | All four DTU `/dtu/reset` endpoints gate on `X-Admin-Token`: Armis (`dtu.rs:43`), Claroty (`devices.rs:332/396`), CrowdStrike (`mod.rs:42`), Cyberint (`dtu.rs:37`). Tests in `tests/dtu_reset_auth.rs` across all four crates verify the 401 response for absent/wrong tokens (3 test functions each). |

---

## Part B — New Findings

### CR-016: Three clone-specific `poll_test_hook` mirrors still spin at 10ms

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:**
  - `crates/prism-dtu-harness/src/clones/armis.rs:901`
  - `crates/prism-dtu-harness/src/clones/claroty.rs:850`
  - `crates/prism-dtu-harness/src/clones/crowdstrike.rs:1161`
- **BC Reference:** BC-3.5.001 (efficient startup budget); W3-FIX-CODE-002 AC-004
- **Description:** CR-006 was fixed by updating `clone_server.rs:poll_test_hook` to 50ms.
  However, each DTU variant that uses its own router also has a mirror function that
  was NOT updated:
  - `poll_armis_test_hook` (`armis.rs:901`): `sleep(10ms)` — no comment
  - `poll_claroty_test_hook` (`claroty.rs:850`): `sleep(10ms)` — no comment
  - `poll_test_hook_crowdstrike` (`crowdstrike.rs:1161`): `sleep(10ms)` — no comment

  In a 12-clone harness with two Armis, two Claroty, and two CrowdStrike clones, these
  six mirrors combined with the updated `clone_server.rs` function result in approximately
  300 wake-ups/second from the mirrors alone, versus the intended 60 wake-ups/second if
  all functions had been updated to 50ms. The W3-FIX-CODE-002 story AC-004 explicitly
  scoped the fix to `clone_server.rs`, so this is a partial-fix gap rather than a
  regression, but the gap is material for CI performance and should be closed.
- **Evidence:**
  ```rust
  // clones/armis.rs:901
  tokio::time::sleep(std::time::Duration::from_millis(10)).await;

  // clones/claroty.rs:850
  tokio::time::sleep(std::time::Duration::from_millis(10)).await;

  // clones/crowdstrike.rs:1161
  tokio::time::sleep(std::time::Duration::from_millis(10)).await;
  ```
  `clone_server.rs:838` (fixed):
  ```rust
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  ```
- **Proposed Fix:** Change all three mirror functions to 50ms and add the same
  `// 50ms polling cadence (CR-006 / W3-FIX-CODE-002 AC-004); replace with tokio::sync::Notify in a future pass.`
  comment, consistent with the `clone_server.rs` approach. Three-line change across
  three files.

---

### CR-017: Armis `validate_org_id` guard not applied to `tags.rs` or `alerts.rs` endpoints

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:**
  - `crates/prism-dtu-armis/src/routes/tags.rs:38` (`post_device_tag`)
  - `crates/prism-dtu-armis/src/routes/tags.rs:71` (`delete_device_tag`)
  - `crates/prism-dtu-armis/src/routes/alerts.rs:38` (`get_alerts`)
- **BC Reference:** BC-3.5.002 precondition 3; W3-FIX-CODE-004 AC-003 (CR-012/SEC-P2-001)
- **Description:** The AC-003 acceptance criterion for CR-012/SEC-P2-001 explicitly
  states: "Apply to `get_or_post_devices`, `post_devices`, and **all tag endpoints**."
  The current implementation applies the `is_real_org` dual-mode guard to
  `get_or_post_devices` (line 89) and `post_devices` (line 122), but not to:
  - `POST /api/v1/devices/{device_id}/tags/` (`post_device_tag`) — only `check_bearer_auth`
  - `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` (`delete_device_tag`) — only `check_bearer_auth`
  - `GET /api/v1/alerts` (`get_alerts`) — only `check_bearer_auth`

  A real-org Armis clone with a genuine customer `instance_org_id` will accept tag write
  requests and alert reads from any caller that provides a valid Bearer token, even if
  the caller omits or provides a different `X-Org-Id`. This is inconsistent with the
  device query endpoints that do enforce the guard, and inconsistent with Claroty's
  single data endpoint which also enforces it.

  The CR-012 regression test in `tests/cr012_validate_org_id_consistency.rs` only tests
  `GET /api/v1/devices` (5 tests); it does not test POST devices, tags, or alerts. The
  gap in test coverage allowed the partial fix to pass.
- **Evidence:**
  ```rust
  // tags.rs:38 — post_device_tag (no org-id guard)
  pub async fn post_device_tag(...) {
      if let Some(err) = check_bearer_auth(&headers) {
          return err;   // only bearer check; no validate_org_id
      }
      // ... tag write proceeds
  }

  // devices.rs:89 — get_or_post_devices (guard present)
  let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
  if is_real_org || headers.get("x-org-id").is_some() {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }
  ```
- **Proposed Fix:** Add the same `is_real_org` dual-mode guard to `post_device_tag`,
  `delete_device_tag`, and `get_alerts`. Add regression tests for at least one of the
  tag endpoints mirroring the existing CR-012 test structure. This is a ~10-line
  change per handler plus 3 additional test functions.

---

### CR-018: CrowdStrike `detections.rs` route handlers lack `validate_org_id` guard

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:**
  - `crates/prism-dtu-crowdstrike/src/routes/detections.rs:105` (`list_detection_ids`)
  - `crates/prism-dtu-crowdstrike/src/routes/detections.rs:183` (`get_detection_summaries`)
- **BC Reference:** BC-3.5.002 precondition 3; W3-FIX-SEC-001 AC-001/AC-002/AC-003
- **Description:** W3-FIX-SEC-001 applied the `validate_org_id` nil-instance guard to
  CrowdStrike's `hosts.rs` (`list_host_ids` at line 146, `get_host_details` at line 294)
  and `writes.rs` (`patch_detections` at lines 102, 248). However, the two handlers in
  `detections.rs` — `list_detection_ids` and `get_detection_summaries` — are registered
  routes in `mod.rs:181-186` but have no `validate_org_id` guard. Both handlers query
  the detection store, which is keyed per-org. A real-org CrowdStrike clone will accept
  detection queries from any Bearer-authenticated caller regardless of `X-Org-Id`.

  The asymmetry means that an adversary who can supply any `X-Org-Id` (or omit it) on
  the detection endpoints would bypass org isolation on the `list_detection_ids` and
  `get_detection_summaries` paths, while the same attack on `hosts` or `writes` would
  be blocked.

  This gap was present before W3.2 (it predates W3-FIX-SEC-001's partial fix) and was
  not caught by the W3.2 test suite because no detection-endpoint regression tests for
  the guard were added.
- **Evidence:**
  ```rust
  // detections.rs:105 — list_detection_ids (no org-id guard)
  pub async fn list_detection_ids(...) {
      // no validate_org_id call; no nil-instance check
      let session_id = extract_session_id(state, &headers);
      ...
  }

  // hosts.rs:146 — list_host_ids (guard present)
  if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }
  ```
  Router registration in `mod.rs:181-185`:
  ```rust
  .route("/detections/queries/detections/v1",
      get(detections::list_detection_ids))
  .route("/detections/entities/detections/v2",
      post(detections::get_detection_summaries))
  ```
- **Proposed Fix:** Add the nil-instance guard to both handlers in `detections.rs`,
  consistent with the existing pattern in `hosts.rs:146` and `writes.rs:102`:
  ```rust
  if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }
  ```
  Add regression tests mirroring the existing dtu_reset_auth.rs pattern.

---

### CR-019: `find_snippet_pipe` matches any ` | ` in line, not just TOML snippet prefix

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-customer-config/src/validator.rs:431-434`
- **BC Reference:** BC-3.3.001 (credential redaction in errors)
- **Description:** `find_snippet_pipe` is documented as finding the TOML 0.8 snippet
  separator `" | "` that divides the line-number prefix from the code content. The
  implementation is simply `line.find(" | ")`, which finds the FIRST occurrence of
  ` | ` in the entire line, not just in the line-number prefix position.

  A TOML value such as `display_name = "Ops | Engineering"` appearing in a parse
  error snippet would be classified as a snippet line at the position of ` | `, and
  the "content" extracted would be `Engineering"` — cutting off the actual field
  name. On such a line, `is_credential_pattern("display_name = \"Ops ")` evaluates to
  false (the trimmed field name has a trailing space and quote), so no data is
  wrongly redacted. However, a value like `api_token = "abc | xyz"` would have
  `find_snippet_pipe` match the ` | ` inside the value, making `eq_pos` find
  `api_token = "abc ` as the field, failing the `is_credential_pattern` check on
  the truncated fragment, and leaving `"abc | xyz"` visible in the error.

  The risk is low because TOML parse errors with pipe-containing credential values
  are uncommon in practice, and the broader `scan_for_credentials` step would catch
  the credential in the raw value. However, the `sanitize_error_message` function
  would not redact it from the snippet portion of the error message.
- **Evidence:**
  ```rust
  // validator.rs:431-434
  fn find_snippet_pipe(line: &str) -> Option<usize> {
      // Pattern: optional whitespace, digits or whitespace, then " | "
      // We look for " | " after stripping leading content.
      line.find(" | ")   // matches first occurrence, not just prefix format
  }
  ```
  A TOML 0.8 snippet line has the format `"  12 | api_token = \"abc | xyz\""`.
  `line.find(" | ")` returns the offset of the first ` | ` (between `12` and `api_token`),
  so this particular case works correctly. But `"   | api_token = \"abc | xyz\""` (the
  caret line after) would match the `|` after the leading whitespace and set content to
  `api_token = "abc | xyz"`, then find `eq_pos` at the first `=`, producing field
  `api_token` — which IS a credential pattern, so it would actually be correctly redacted.
  The failure case requires the pipe to appear BEFORE the actual `=` assignment in a
  non-caret-line context.
- **Proposed Fix:** Tighten `find_snippet_pipe` to verify the prefix is in the expected
  TOML snippet format (leading whitespace, optional digits, then ` | `):
  ```rust
  fn find_snippet_pipe(line: &str) -> Option<usize> {
      // TOML 0.8 snippet format: "  [digits] | content" or "   | ^^^^^"
      // Only classify as a snippet line if everything before " | " is
      // ASCII digits and/or whitespace.
      let pos = line.find(" | ")?;
      let prefix = &line[..pos];
      if prefix.chars().all(|c| c.is_ascii_digit() || c == ' ') {
          Some(pos)
      } else {
          None
      }
  }
  ```
  This makes the function robust against values containing ` | ` and matches the
  documented intent.

---

### CR-020: AC-005 story deviation not formally acknowledged — `validate_spec_path` stayed `pub`

- **Severity:** LOW
- **Category:** maintainability
- **Location:** `crates/prism-customer-config/src/validator.rs:742`
- **BC Reference:** W3-FIX-CODE-004 AC-005
- **Description:** The W3-FIX-CODE-004 AC-005 acceptance criterion specified changing
  `pub fn validate_spec_path` to `pub(crate) fn validate_spec_path`, asserting "no
  external crate uses this function directly." The function remained `pub` (with
  `#[doc(hidden)]`) because `tests/path_traversal.rs` is an integration test binary
  that lives outside `src/` and requires `pub` visibility.

  This is the correct implementation choice — `pub(crate)` would break the integration
  test, and `pub` with `#[doc(hidden)]` is the idiomatic Rust pattern for "stable
  enough for our own integration tests, but not intended as a public API." However, the
  AC-005 story says the change "does not break any test," which is only true because
  the visibility was NOT changed as specified.

  The deviation is undocumented in the commit message, the story status, or any
  decision record. Future maintainers reading AC-005 will see "change to pub(crate)"
  and find `pub` in the code, creating confusion about whether the story was fully
  implemented.

  No security risk: `#[doc(hidden)]` is a documentation annotation that does not
  restrict callers; any external crate with a Cargo dependency can still call this
  function, but the risk is the same as before the change.
- **Evidence:** `validator.rs:741-742`:
  ```rust
  #[doc(hidden)]
  pub fn validate_spec_path(
  ```
  AC-005 says: "changes from `pub fn validate_spec_path` to `pub(crate) fn validate_spec_path`."
- **Proposed Fix:** Add a comment at line 741 explaining the deviation explicitly:
  ```rust
  // AC-005 deviation: spec required pub(crate) but integration tests in tests/path_traversal.rs
  // (which are external binaries in Rust's visibility model) call this function directly.
  // pub(crate) would break those tests. pub + #[doc(hidden)] is the correct compromise.
  // See W3-FIX-CODE-004 decision record for rationale.
  #[doc(hidden)]
  pub fn validate_spec_path(
  ```
  No functional change required. Optionally update the W3-FIX-CODE-004 story status
  to note the deviation.

---

## Positive Observations (Non-Finding)

**CR-003 implementation is complete and well-tested.** The four test cases
(space, unicode, dot, length > 64) in `cr003_slug_pattern.rs` directly map to the
acceptance criteria. Using `OrgSlug::new().is_err()` correctly delegates to the
`^[a-zA-Z0-9_-]{1,64}$` regex compiled once via `OnceLock`.

**CR-011 fix is semantically correct.** The `remove`-on-None pattern in both the
immediate path (line 242) and the deferred drain (line 320) is idiomatic and
correct. `is_empty()` now reliably indicates whether failure injection is needed.

**SEC-006 multi-line redaction is well-structured.** The `in_multiline_cred` state
machine correctly handles both TOML snippet format lines (pipe-separated) and raw
source context lines appended for diagnostics. The non-snippet path at line 398 also
handles multi-line continuations. Tests in `sec006_toml_multiline_redaction.rs` cover
both the snippet format and the raw source format.

**SEC-007 wiring is architecturally correct.** `validate_org_slug_cross_check` is
called with a live `Arc<OrgRegistry>` that is cloned per-call (not captured at
service construction), so the check always reflects the current registry state. The
function never panics (the `match` on `Option` handles `None` cleanly). The
audit-must-not-fail semantics are preserved: the `let _ =` discard means slug
mismatch does not abort emission.

**Armis dual-mode guard for devices.rs is the correct compromise for the DTU model.**
The `is_real_org || header_present` condition at line 90 correctly enforces strict
auth for real-org clones (absent header → 401) while preserving backward
compatibility for the 50+ legacy tests that use the default instance and omit the
header. The module doc accurately explains both modes. The fix resolves the core
security concern of CR-012; the incomplete extension to tags/alerts (CR-017) is a
hygiene gap, not a regression.

**BC-3.2.002 trait coverage is complete and idiomatic.** All 7 test functions in
`bc_3_2_002_trait_impl.rs` use `MockStorageEngine` correctly gated behind
`cfg(any(test, feature = "test-utils"))`. AC naming convention (`test_BC_3_2_002_AC_NNN_*`)
is consistent with the rest of the test corpus.

**W3-FIX-SEC-002 admin token enforcement is complete across all four DTUs.** All
four `/dtu/reset` endpoints correctly compare `provided == Some(state.admin_token.as_str())`
and return HTTP 401 for absent or mismatched tokens. Evidence recordings
(`dtu_reset_auth.rs` across all four crates) provide 3-AC coverage each.

---

## Summary of Open Items

| ID | Severity | Status | Description |
|----|----------|--------|-------------|
| CR-016 | MEDIUM | NEW | 3 clone-specific poll_test_hook mirrors still 10ms (Armis, Claroty, CrowdStrike) |
| CR-017 | MEDIUM | NEW | Armis tags.rs and alerts.rs endpoints missing is_real_org guard |
| CR-018 | MEDIUM | NEW | CrowdStrike detections.rs handlers missing validate_org_id guard |
| CR-019 | LOW | NEW | find_snippet_pipe false-match risk on values containing " \| " |
| CR-020 | LOW | NEW | AC-005 pub(crate) deviation undocumented at call site |
| CR-007 | LOW | DEFERRED | archetype/scale declared but unread in build() — Wave 4 |
| CR-008 | LOW | DEFERRED | Placeholder CloneState sentinel strings — Wave 4 |
| CR-009 | LOW | DEFERRED | Wall-clock startup assertion — Wave 4 |

---

## Convergence Verdict

`findings remain -- iterate`

No new HIGH or CRITICAL findings were introduced by Wave 3.2. All pass-1 and
pass-2 HIGH findings (CR-001, CR-002, SEC-NEW-001) are resolved. Twelve items
from passes 1 and 2 are correctly closed in W3.2.

However, three new MEDIUM findings (CR-016, CR-017, CR-018) represent incomplete
application of fixes whose acceptance criteria explicitly specified the scope.
CR-017 and CR-018 are guard-bypass gaps on Armis tag/alert endpoints and
CrowdStrike detection endpoints respectively — both leave real-org clone paths
partially unprotected, inconsistent with the org isolation model. CR-016 is a
performance gap from an incomplete 10→50ms migration.

These three items should be addressed in a W3.3 hygiene story before the wave
gate proceeds to a final APPROVE verdict. CR-019 and CR-020 (LOW) may be
deferred to the same story or to Wave 4 at the team's discretion.
