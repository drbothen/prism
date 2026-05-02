---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: 6696e374^..cda17ed4 (Wave 3 + 3.1, ~45 commits; pass-2 focuses on 5 fix-wave PRs)
reviewer: vsdd-factory:code-reviewer
develop_sha: cda17ed4
date: 2026-05-01
phase: 3
wave: 3
step: c
pass: 2
previous_review: .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review.md
verdict: APPROVE_WITH_CONCERNS
total_findings: 6
high: 0
medium: 4
low: 2
---

# Wave 3 Integration Gate — Gate Step C: Code Review (Pass 2)

**Scope:** `6696e374^..cda17ed4` (Wave 3 + Wave 3.1 fix wave — ~45 commits)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-01
**Previous review:** `gate-step-c-code-review.md` (pass 1, SHA `a3bd5a0f`)
**Verdict:** APPROVE_WITH_CONCERNS — 2 HIGH fixed, 4 new MEDIUM, 2 new LOW. All
critical/high are resolved. MEDIUM findings do not block merge; they require
follow-on work or acceptance in the W3-FIX-CODE-002 batch.

---

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| CR-001 | HIGH | RESOLVED | `CustomerSpec.initial_failure` changed to `HashMap<DtuType, FailureMode>`. Both the immediate-resolution path (`existing.initial_failure.insert(dtu_type, mode)`) and the deferred path (`pending` drain in `build()`) now correctly scope failures to the specific `(slug, DtuType)` pair. Phase 4 and Phase 5 injection loops iterate the map, injecting only the specified types. |
| CR-002 | HIGH | RESOLVED | `Drop` no longer calls `handle.abort()`. The fix removes the abort loop and relies on axum's `with_graceful_shutdown` drain contract. The handles are dropped, which in Tokio does not cancel the underlying task (tasks are only cancelled by `abort()`), preserving the graceful-exit window. The `fn drop` doc comment is now accurate at the method level. One doc residue remains (see CR-010 below). |
| CR-003 | MEDIUM | UNRESOLVED (deferred) | `validate_structural` in `validator.rs` still does not validate `org_slug` against the `^[a-zA-Z0-9_-]{1,64}$` pattern. W3-FIX-CODE-002 was not merged. Recorded as tech-debt below. |
| CR-004 | MEDIUM | UNRESOLVED (deferred) | `start_clone` still uses sequential `if dtu_type ==` chains. No change. Recorded as tech-debt below. |
| CR-005 | MEDIUM | UNRESOLVED (deferred) | `validate_all` remains `pub`. No change. Recorded as tech-debt below. |
| CR-006 | MEDIUM | UNRESOLVED (deferred) | `poll_test_hook` still spins at 10ms with no cancellation. No change. Recorded as tech-debt below. |
| CR-007 | LOW | UNRESOLVED (deferred) | `archetype`/`scale` fields still declared but unread. No change. |
| CR-008 | LOW | UNRESOLVED (deferred) | Placeholder `CloneState` sentinel strings unchanged. No change. |
| CR-009 | LOW | UNRESOLVED (deferred) | Wall-clock startup assertion unchanged. No change. |

---

## Part B — New Findings

### CR-010: Module-level doc in `harness.rs` still describes the old abort-on-drop behavior

- **Severity:** MEDIUM
- **Category:** maintainability
- **Location:** `crates/prism-dtu-harness/src/harness.rs:18-20` (SHA `702d10b5`)
- **BC Reference:** BC-3.5.001 EC-004
- **Description:** The `fn drop` implementation doc and inline comments were
  correctly updated by W3-FIX-CODE-001 to describe the preferred resolution
  (no abort, rely on axum `with_graceful_shutdown`). However, the module-level
  doc comment at the top of the file still reads:

  > "waits up to 5s for graceful exit, then calls `handle.abort()` on any that
  > have not exited within the grace period (BC-3.5.001 EC-004)."

  This is now false — the Drop implementation no longer calls `handle.abort()`.
  The module doc is the first thing a developer reads when opening the file; a
  stale description of abort behavior will create confusion about whether the
  harness performs hard teardown.
- **Evidence:** `harness.rs:18-20` (HEAD `cda17ed4`):
  ```
  //! `impl Drop for Harness` sends shutdown signals to all non-crashed clones,
  //! waits up to 5s for graceful exit, then calls `handle.abort()` on any that
  //! have not exited within the grace period (BC-3.5.001 EC-004).
  ```
  Actual `drop` implementation at line 364-387: sends shutdown signal, then
  `drop(self.task_handles.drain().collect::<Vec<_>>())` — no `abort()` call.
- **Proposed Fix:** Update lines 18-20 to read:
  ```
  //! `impl Drop for Harness` sends shutdown signals to all non-crashed clones
  //! and drops their `JoinHandle`s without abort, honoring the spec-promised
  //! graceful-exit window via axum's `with_graceful_shutdown` drain
  //! (BC-3.5.001 EC-004; W3-FIX-CODE-001 AC-002/AC-004).
  ```

---

### CR-011: `with_failure(FailureMode::None)` inserts sentinel into HashMap instead of removing the entry

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-harness/src/builder.rs:233-246` (SHA `702d10b5`)
- **BC Reference:** BC-3.6.001 Invariant 4
- **Description:** The `with_failure` doc comment states: "Passing `FailureMode::None`
  clears any previously set `initial_failure` on the matching spec (EC-002; BC-3.6.001
  Invariant 4)." The implementation does:
  ```rust
  existing.initial_failure.insert(dtu_type, mode); // mode = FailureMode::None
  ```
  This stores `FailureMode::None` as a map entry rather than removing it. The
  subsequent Phase 4/5 injection loop then calls `inject_failure(slug, dtu_type,
  FailureMode::None)`, which — because `inject_failure` maps `FailureMode::None` to
  `{"clear": true}` — does effectively clear the failure. The behavior is therefore
  functionally equivalent to the documented intent at runtime.

  However, the representation is semantically wrong in two ways:

  1. `initial_failure.is_empty()` is the condition used to skip the injection loop
     (Network mode `customers_for_injection` filter). After calling
     `with_failure("alpha", DtuType::Claroty, FailureMode::None)`, the map contains
     one entry and `is_empty()` returns `false`, causing an unnecessary HTTP round-trip
     to `/dtu/configure` with `{"clear": true}` during build. For the common idiom of
     "set then clear before build", this is a spurious inject call.

  2. A caller who inspects `spec.initial_failure` directly (e.g., in a test helper or
     a future serialisation path) will see a `FailureMode::None` entry and may
     misinterpret it as "a failure IS configured for this type", defeating the purpose
     of clearing.

  No test covers the `with_failure(FailureMode::None)` path post-fix.
- **Evidence:**
  ```rust
  // builder.rs:234-246
  pub fn with_failure(mut self, slug: &str, dtu_type: DtuType, mode: FailureMode) -> Self {
      if let Some(existing) = self.customers.iter_mut().find(|s| ...) {
          existing.initial_failure.insert(dtu_type, mode); // FailureMode::None stored as entry
          return self;
      }
      self.pending_failures.push((slug.to_owned(), dtu_type, mode));
      self
  }
  ```
- **Proposed Fix:** Use `HashMap::remove` when `mode` is `FailureMode::None`; otherwise
  `insert`. This makes the clear semantics structurally correct and avoids the spurious
  configure call:
  ```rust
  if let Some(existing) = self.customers.iter_mut().find(|s| ...) {
      if matches!(mode, FailureMode::None) {
          existing.initial_failure.remove(&dtu_type);
      } else {
          existing.initial_failure.insert(dtu_type, mode);
      }
      return self;
  }
  ```
  Apply the same pattern in the deferred `pending` drain in `build()`.

---

### CR-012: Armis `validate_org_id` gate is header-presence-based rather than instance-identity-based, inconsistent with three other DTUs

- **Severity:** MEDIUM
- **Category:** pattern-consistency
- **Location:** `crates/prism-dtu-armis/src/routes/devices.rs:64-68, 96-100`
  (SHA `59803de3`)
- **BC Reference:** BC-3.5.002 precondition 3; W3-FIX-SEC-001 AC-001/AC-002/AC-003
- **Description:** All four DTU clones received `validate_org_id` guards via
  W3-FIX-SEC-001. However, Armis uses a different activation model from the other
  three:

  - **Claroty, CrowdStrike, CrowdStrike writes:** guard is `if state.instance_org_id !=
    nil_org` — validation is active whenever the clone was created with a real OrgId.
  - **Armis:** guard is `if headers.get("x-org-id").is_some()` — validation only fires
    when the header is present. When the header is absent, the request is let through
    unconditionally, even if the clone has a real `instance_org_id`.

  The Armis DTU uses `DTU_DEFAULT_INSTANCE_ORG_ID = 0...AA` (not nil) as its default.
  This means: a real multi-tenant Armis clone (constructed via `with_admin_token_and_org`
  with a customer's OrgId) will accept requests that omit the `X-Org-Id` header
  entirely, even though the clone has a binding `instance_org_id` and should reject
  unauthenticated callers. Claroty with the same scenario would return 401.

  Additionally, `ArmisState` has no nil-OrgId path at all (it uses `0...AA` as
  default), so the Armis guard semantics (header-presence check) differ structurally
  from the nil-instance check used by the others. This inconsistency makes the
  four-DTU auth model harder to reason about as a uniform set.
- **Evidence:**
  ```rust
  // devices.rs:64-68 (Armis)
  // When header is absent, fall through (backward compat for callers without org header).
  if headers.get("x-org-id").is_some() {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }

  // vs. hosts.rs:142-148 (CrowdStrike)
  if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }
  ```
- **Proposed Fix:** Replace the header-presence guard in Armis with the same
  instance-identity guard used by Claroty/CrowdStrike:
  ```rust
  // Armis uses DTU_DEFAULT_INSTANCE_ORG_ID (not nil) as its legacy default.
  // Guard on whether this is a "real" org clone by checking a distinct sentinel.
  if state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID {
      if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
          return (status, body).into_response();
      }
  }
  ```
  Apply to `get_or_post_devices`, `post_devices`, and all tag endpoints. This makes
  the Armis multi-tenant path consistent: absent `X-Org-Id` header returns 401 for
  real-org clones, just as in Claroty/CrowdStrike.

---

### CR-013: `fan_out` does not assert `target.spec.org_id == target.org_id` before `adapter.fetch()`

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-sensors/src/fanout.rs:362-380` (SHA `cda17ed4`)
- **BC Reference:** BC-3.2.001 precondition 4; S-3.1.06-ImplPhase AC-004
- **Description:** `fan_out()` correctly looks up the adapter via
  `registry.get(target.org_id, target.sensor_type)`, so the adapter returned is always
  registered for `target.org_id`. However, `adapter.fetch(&target.spec, ...)` then
  fires the adapter's OrgId mismatch guard using `spec.org_id`, which is a separate
  field on the `SensorSpec` struct. `SensorSpec.org_id` defaults to `Uuid::nil()`
  via `#[serde(default)]`, so a caller who constructs a `FanOutTarget` without
  explicitly setting `target.spec.org_id = target.org_id` will silently produce a
  mismatch — the `OrgIdMismatch` error will fire from inside the adapter's `fetch()`
  after the registry lookup succeeds. This is confusing because the registry dispatch
  was correct but the spec's identity is wrong.

  The dual-identity design (one `org_id` on `FanOutTarget` for registry dispatch,
  another on `SensorSpec` for adapter guard) creates a usability trap: callers can
  construct `FanOutTarget { org_id: org_a, spec: SensorSpec { org_id: org_b, ... } }`
  without any compile-time or build-time indication that these must be equal.

  There is no test in `org_id_binding.rs` that constructs a `FanOutTarget` with
  inconsistent `target.org_id` vs `target.spec.org_id` to verify the error message
  is actionable.
- **Evidence:**
  ```rust
  // fanout.rs:338-365
  let adapter = match registry.get(target.org_id, target.sensor_type) { // uses target.org_id
      Some(a) => a,
      None => { ... }
  };
  match adapter.fetch(&target.spec, &target.params, auth.as_ref()).await { // uses target.spec.org_id
      ...
  }
  ```
  `SensorSpec` in `adapter.rs:39`:
  ```rust
  #[serde(default)]
  pub org_id: OrgId,  // defaults to nil UUID
  ```
- **Proposed Fix:** Two options. Option A (preferred): add a debug assertion in
  `fan_out` before the `fetch()` call:
  ```rust
  debug_assert_eq!(
      target.org_id, target.spec.org_id,
      "fan_out precondition violation: target.org_id ({}) != target.spec.org_id ({}) — \
       callers must set spec.org_id = target.org_id (BC-3.2.001 precondition 4)",
      target.org_id, target.spec.org_id
  );
  ```
  Option B: remove `SensorSpec.org_id` and derive it from `FanOutTarget.org_id`
  at the call site — `adapter.fetch(&SensorSpec { org_id: target.org_id, ..target.spec }, ...)`
  — eliminating the dual-identity design. Option B requires more refactor but
  removes the inconsistency structurally.

---

### CR-014: `validate_spec_path` is `pub` (not `pub(crate)`) — exposes internal CWE-22 guard as public API

- **Severity:** LOW
- **Category:** architecture-alignment
- **Location:** `crates/prism-customer-config/src/validator.rs:625` (SHA `a68d1748`)
- **BC Reference:** S-3.3.01 (validator crate boundary)
- **Description:** `validate_spec_path` is declared `pub`, making it part of the
  public API of `prism-customer-config`. The function is an internal validation helper
  used only by `validate_dtu_block` within the same file. No external caller in the
  workspace uses it directly. Exporting it:
  - Implies it is a stable, supported interface that downstream crates may call.
  - Expands the trust surface of the CWE-22 guard: if a caller invokes it directly
    with a `config_path` that doesn't reflect the real customers directory anchor,
    the boundary check (`canonical_candidate.starts_with(&canonical_parent)`) is
    trivially defeated by passing an adversarial `config_path`.
  - Contradicts the pattern in the same file where `validate_dtu_block`,
    `validate_structural`, and the error-helper functions are all crate-private.
- **Evidence:** `validator.rs:625`:
  ```rust
  pub fn validate_spec_path(
      config_path: &Path,
      spec_path: &str,
  ) -> Result<std::path::PathBuf, ConfigError>
  ```
  No usages outside `validator.rs` itself found via `git grep`.
- **Proposed Fix:** Change `pub fn validate_spec_path` to `pub(crate) fn validate_spec_path`.
  This is a non-breaking change unless external crates are intentionally testing it —
  inspection shows only internal callers.

---

### CR-015: Cyberint `validate_org_id` is defined but marked `#[allow(dead_code)]` — not wired into any route handler

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-cyberint/src/routes/alerts.rs:84` (SHA `59803de3`)
- **BC Reference:** W3-FIX-SEC-001 AC-001/AC-002/AC-003; BC-3.5.002 precondition 3
- **Description:** W3-FIX-SEC-001 was intended to wire `X-Org-Id` validation into all
  four DTU clones. For Cyberint, `validate_org_id` was defined and documented but
  intentionally not called from any route handler. The design note in the doc comment
  explains that Cyberint supports multi-org session routing and that
  `instance_org_id`-based gating is incompatible with that design. This architectural
  reasoning is valid. However:

  1. The function is `#[allow(dead_code)]` — this attribute suppresses the compiler
     warning but does not communicate intent to a future developer. If the function
     is intentionally not used, it should either be removed (if truly not needed) or
     have a `TODO(S-X.Y.Z)` comment explaining when and how it will be wired.

  2. The header name diverges: Cyberint uses `X-Prism-Org-Id` while Armis/Claroty/
     CrowdStrike use `X-Org-Id`. If the intent is a uniform four-DTU auth model, the
     header name inconsistency should be documented as a tracked deviation, not left
     as a silent convention difference.

  3. The AC-003 requirement (missing header → 401) is NOT met for Cyberint: absent
     `X-Prism-Org-Id` falls through to `extract_org_id` which falls back to
     `instance_org_id`, proceeding normally. This is by design given the session
     routing architecture, but the gap vs. the three other DTUs is undocumented.
- **Evidence:** `alerts.rs:84`:
  ```rust
  #[allow(dead_code)]
  pub(crate) fn validate_org_id(...)
  ```
  Route handlers use `extract_org_id(&headers, state.instance_org_id)` without
  calling `validate_org_id`.
- **Proposed Fix:** Two options. Option A: remove `validate_org_id` from Cyberint
  entirely; add a module-level comment in `alerts.rs` explaining why Cyberint uses
  session-routing rather than instance-identity enforcement, and that this is the
  intentional deviation from the other three DTUs. Option B: retain the function
  but remove `#[allow(dead_code)]`, add a `// TODO(S-X.Y.Z): wire into a future
  single-tenant mode path` comment, and open a tech-debt item. Option A is cleaner.

---

## Positive Observations (Non-Finding)

**CR-001 fix is idiomatic and complete.** The `HashMap<DtuType, FailureMode>` type
correctly replaces `Option<FailureMode>`. Both the logical-mode path (Phase 4 in
`build_logical`) and the network-mode path (Phase 5 in `build_network`) were updated
consistently. The deferred-resolution path via `pending_failures` drain correctly
calls `spec.initial_failure.insert(dtu_type, mode)` for each resolved slug. The
`customers_for_injection` capture in network mode uses `!s.initial_failure.is_empty()`
as the filter, which correctly skips orgs with no failure configured. AC-001 and
AC-002 are both satisfied.

**CR-002 fix is correct for the chosen resolution path.** Dropping `JoinHandle`
without calling `abort()` is the correct Tokio-idiomatic approach: Tokio tasks are
not cancelled on `JoinHandle` drop, only on `abort()`. Since `axum::Server::with_graceful_shutdown`
is already wired in both `clone_server::run_server` and `builder::run_network_server`,
the shutdown broadcast causes axum to drain in-flight requests and exit cleanly
within cooperative scheduling. The removal of the abort loop is sufficient and correct.

**W3-FIX-SEC-003 path traversal hardening is well-structured.** The `validate_spec_path`
function separates pre-join checks (absolute path rejection, `..` component scan)
from post-join checks (canonicalize + prefix comparison). The ordering is correct:
pre-join checks are zero-cost and catch the common cases before filesystem I/O.
The `canonicalize()` call correctly resolves symlinks (AC-004). The integration
with `validate_dtu_block` correctly gates the E-CFG-015 existence check behind the
traversal check (no double-reporting). Error variant `SpecPathTraversal` is
well-documented.

**S-3.1.06-ImplPhase OrgId binding is structurally sound.** `AdapterRegistry` with
`HashMap<(OrgId, SensorType), Arc<dyn SensorAdapter>>` is the correct idiomatic
Rust composite-key pattern. The four adapters all store `pub(crate) org_id: OrgId`
and check it at the start of `fetch()` before any network I/O (CR-013 above notes
a path where this guard can be bypassed via inconsistent `FanOutTarget` construction,
but the guard itself is correct). `init_registry_for_org` is a clean migration path
from `init_registry` with `#[deprecated]` annotations threading through correctly.

**W3-FIX-CODE-003 KeyringBackend regression tests are appropriately gated.** The
`keyring_org_id.rs` integration test suite is correctly `#[ignore]`-gated for
headless CI environments, with explicit instructions for running on machines with a
live keyring service. The AC coverage table and the test naming convention
(`test_AC_NNN_*`) are consistent with the rest of the test corpus.

---

## Residual Tech Debt — Deferred to W3-FIX-CODE-002

The following findings from pass 1 were explicitly deferred and NOT resolved in the
Wave 3.1 fix wave. They remain open and must be addressed in a follow-on story
(W3-FIX-CODE-002 or a Wave 4 cleanup story):

| ID | Severity | Title | Deferred To |
|----|----------|-------|-------------|
| CR-003 | MEDIUM | `OrgSlug` regex not validated in config validator | W3-FIX-CODE-002 |
| CR-004 | MEDIUM | `start_clone` sequential if-chains for DtuType dispatch | W3-FIX-CODE-002 |
| CR-005 | MEDIUM | `validate_all` is `pub` despite being a usability trap | W3-FIX-CODE-002 |
| CR-006 | MEDIUM | `poll_test_hook` 10ms spin with no cancellation | W3-FIX-CODE-002 |
| CR-007 | LOW | `archetype`/`scale` declared but never read in `build()` | Wave 4 |
| CR-008 | LOW | Placeholder `CloneState` sentinel strings | Wave 4 |
| CR-009 | LOW | Wall-clock startup assertion susceptible to CI jitter | Wave 4 |

---

## Convergence Verdict

`CONVERGENCE_REACHED`

Both HIGH findings from pass 1 (CR-001, CR-002) are correctly resolved. No new HIGH
findings were introduced by the fix wave. The four MEDIUM findings in this pass
(CR-010 through CR-013) are all improvement-level items:

- CR-010 (stale module doc) is a one-line fix.
- CR-011 (FailureMode::None HashMap insert) is functionally correct at runtime but
  semantically inconsistent; it does not cause incorrect behavior today.
- CR-012 (Armis header-presence guard vs. nil-instance guard) is an inconsistency
  in the auth model; it does not create a regression vs. the pre-fix state.
- CR-013 (fan_out spec.org_id consistency) is a usability gap in the dual-identity
  design, not a behavioral bug given the adapter's own mismatch guard.

The two LOW findings (CR-014, CR-015) are clean-up items.

None of CR-010 through CR-015 represent regressions introduced by the fix wave.
The wave may proceed to the gate decision with these findings tracked as follow-on
work items. W3-FIX-CODE-002 should absorb CR-010, CR-011, CR-012, and CR-013 in
addition to the deferred pass-1 items listed above.
