---
document_type: behavioral-contract
level: L3
bc_id: BC-3.6.001
title: Per-Org Failure Injection
version: "0.8"
status: draft
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md]
input-hash: "efe00d6"
traces_to: ".factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md"
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-036
authors: [product-owner]
related_decisions: [D-044, D-045, D-1072, D-1096, P21-01]
related_adrs: [ADR-011]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: cycle-3
modified: ["2026-06-11"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.6.001: Per-Org Failure Injection

## Description

The `Harness` exposes runtime failure-injection knobs scoped to individual
`(OrgId, DtuType)` pairs via `Harness::inject_failure(org_slug, dtu_type, mode)`.
Failure modes include HTTP 5xx, timeout, malformed response, auth-reject (401/403),
slow-response (configurable delay), and rate-limit (429). A failure injected for
`(OrgA, DtuType::Claroty)` affects only that clone's responses; all other
`(OrgId, DtuType)` clones in the same harness continue to return normal responses.
This enables BC-3.6.x resilience scenarios where one customer's sensor is degraded
while another's remains healthy — a test class that requires per-org, per-sensor
failure injection granularity (ADR-011 §2.7, Rationale).

**Isolation-mode scope (Decision B, architect 2026-06-11, D-1072):** This BC
governs harness-local clones only (the clones managed by `prism-dtu-harness` and
spun up by `HarnessBuilder::build()`). Failure injection in Logical-mode and
Network-mode harnesses is functionally identical for the **per-org Security-Telemetry
clones** (CrowdStrike, Cyberint, Armis, Claroty) because each org's clone is a
distinct instance with its own `FailureLayerShared`. The **MSSP-Coordination clones**
(Jira, PagerDuty, Slack) are single-shared-instance clones using header-based
`X-Prism-Org-Id` isolation (BC-3.2.004). Network-mode per-org address isolation
is semantically undefined for them: the harness cannot assign a separate
`SocketAddr` per org for a single-shared-instance clone. Consequently:

- MSSP-Coordination clones operate in **Logical-mode only** within this BC.
- Network-mode for MSSP-Coordination clones belongs to the deferred TDE write-back
  track (D-1072) and is out of scope for this contract.
- A generic-router `404` when network-mode routes Jira/PagerDuty/Slack to
  per-org addresses is an **intentional loud failure** (Decision B), not a bug.

## Preconditions

1. A `Harness` has been built via `HarnessBuilder::build().await` — all clones are running.
2. The target `(org_slug, dtu_type)` pair is registered in the harness and present in
   `customer_endpoints`.
3. The target clone's `FailureLayerShared` (from `prism-dtu-common/src/layers/failure.rs`)
   is initialized and wired into the clone's axum middleware stack.
4. The `inject_failure` call uses `POST /dtu/configure` on the clone's admin endpoint,
   authenticated with that clone's `admin_token` (ADR-003 Amendment §5).
5. The `dtu` feature flag is enabled.

## Postconditions

1. After `inject_failure(org_slug, dtu_type, mode)` returns `Ok(())`, all subsequent HTTP
   requests to `(org_slug, dtu_type)` receive the injected failure response corresponding
   to `mode`, **subject to the per-clone supported-mode table in Invariant 5**:
   - `FailureMode::AuthReject` → HTTP 401 on every request **EXCEPT PagerDuty**, which
     returns HTTP 403 with body `{"status": "invalid key", "message": "Forbidden"}` matching
     the real PagerDuty Events API routing-key rejection semantics. See Invariant 5 for the
     per-clone AuthReject status-code table. The description prose ("auth-reject (401/403)")
     has always acknowledged this carve-out; this line now encodes it contractually so VP-129
     test vectors and Invariant 5 are internally consistent.
   - `FailureMode::InternalError { after_n }` → HTTP 500 after N requests
   - `FailureMode::RateLimit { after_n }` → HTTP 429 after N requests
   - `FailureMode::NetworkTimeout { after_ms }` → response delayed by `after_ms` ms
   - `FailureMode::MalformedResponse` → response body is not valid JSON
   - `FailureMode::Unprocessable { at_request_n }` → HTTP 422 at request N
2. All other `(OrgId, DtuType)` clones in the same harness return normal (non-injected)
   responses; their `FailureLayerShared` state is unchanged.
3. After `clear_failure(org_slug, dtu_type)` returns `Ok(())`, the target clone resumes
   returning normal responses; subsequent requests to that clone receive HTTP 200 with
   valid data.
4. Failure injection and clearing are idempotent: calling `inject_failure` with the same
   mode twice has the same observable effect as calling it once.
5. If `inject_failure` is called with a `FailureMode` that the target clone does not
   support (see Invariant 5 supported-mode table), `POST /dtu/configure` returns
   **HTTP 400** with body `{"error": "unsupported_failure_mode", "mode": "<mode-name>"}`.
   No state change occurs; a subsequent request to that clone continues to behave normally.
   The 400 is deterministic: it does not depend on request count or prior state.

## Invariants

1. Failure injection state is scoped strictly to the target `(OrgId, DtuType)` clone's
   `FailureLayerShared` instance; no shared mutable state exists between clone instances.
2. The `inject_failure` and `clear_failure` APIs are synchronous with respect to the
   clone's request-handling pipeline — a request arriving after `inject_failure` returns
   `Ok` will observe the injected mode; a request that arrived before `inject_failure`
   was called completes under the prior (non-injected) mode.
3. The `admin_token` used to authenticate `POST /dtu/configure` is per-clone and is not
   shared across clones; injecting a failure into one clone does not require knowledge of
   another clone's admin token.
4. `FailureMode::None` is equivalent to `clear_failure` — setting it explicitly clears
   any previously injected mode.
5. **Per-clone supported failure modes (harness-local clones).** This invariant
   binds **harness-local clones** managed by `prism-dtu-harness` — the clone
   instances spun up by `HarnessBuilder::build()`. The real production crates
   (`prism-dtu-jira`, `prism-dtu-pagerduty`, `prism-dtu-slack`) expose a `/dtu/configure`
   surface governed by their own story contracts, not by this invariant. See the
   Binding Scope note in the Description section and P20-04 ruling (PO adjudication
   burst, 2026-06-11).

   Not all clones support all `FailureMode` variants via the harness `inject_failure`
   API. The authoritative supported-mode table for harness-local clones is:

   | Clone | Harness Isolation Mode | Supported Modes | AuthReject HTTP Status | Rationale |
   |-------|----------------------|----------------|----------------------|-----------|
   | Claroty | Logical + Network | ALL (AuthReject, InternalError, RateLimit, NetworkTimeout, MalformedResponse, Unprocessable) | **401** | Cyber-sensor clone; `apply_failure_mode` Tower layer returns 401 for `FailureMode::AuthReject` (`failure.rs:159-162`) |
   | Armis | Logical + Network | ALL | **401** | Cyber-sensor clone; same Tower layer path as Claroty |
   | CrowdStrike | Logical + Network | ALL | **401** | Cyber-sensor clone; same Tower layer path; `auth_mode="reject"` sets `RuntimeConfig.auth_reject` which route handlers convert to 401 |
   | Cyberint | Logical + Network | ALL | **401** | Cyber-sensor clone; same Tower layer path as Claroty |
   | Jira | Logical only | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | **401** | MSSP-coordination clone; `auth_mode="reject"` routes through `FailureMode::AuthReject` Tower layer → 401; network-mode N/A (Decision B, D-1072) |
   | PagerDuty | Logical only | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | **403** with body `{"status":"invalid key","message":"Forbidden"}` | **PagerDuty exception.** `auth_mode="reject"` sets `PagerDutyState.auth_reject` boolean, checked at route level in `enqueue.rs` BEFORE the Tower layer. Route handler returns 403 matching real PagerDuty Events API routing-key rejection. Verified: `fidelity.rs:397-443` + `harness_tests.rs` (AC-8) + `bc_3_6_001_ops_clone_failure_modes.rs:438-466`. The Tower-layer `FailureMode::AuthReject` (which would return 401) is NOT the code path exercised by `auth_mode="reject"` for PagerDuty; network-mode N/A (Decision B, D-1072) |
   | Slack | Logical only | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | **401** | MSSP-coordination clone; `FailureMode::AuthReject` Tower layer returns 401; network-mode N/A (Decision B, D-1072) |

   **P21-01 AuthReject mechanism note (v0.8 amendment, 2026-06-11):** PagerDuty's 403 is NOT
   produced by `apply_failure_mode` in `prism-dtu-common/src/layers/failure.rs:159-162` (which
   universally returns 401 for `FailureMode::AuthReject`). It is produced by a route-level check
   in `crates/prism-dtu-pagerduty/src/routes/enqueue.rs:42-51` that reads `PagerDutyState.auth_reject`
   and returns 403 to match real PagerDuty Events API fidelity. Two distinct code paths; the spec
   must distinguish them. All other clones (Claroty, Armis, CrowdStrike, Cyberint, Jira, Slack)
   serve 401 through the Tower layer's `FailureMode::AuthReject` arm.

   A `POST /dtu/configure` call with a mode NOT listed as supported for that clone MUST
   return HTTP 400 with body `{"error": "unsupported_failure_mode", "mode": "<variant-name>"}`.
   Silent acceptance (200 ACK + no behavioral effect) is a SOUL.md §4 violation and is
   explicitly prohibited by Postcondition 5.

   **Note (2026-06-11):** The MSSP-coordination clones (Jira, PagerDuty, Slack) were
   initially implemented with route-level `match` that only honored RateLimit and
   InternalError while silently ACKing all other modes. This BC amendment (v0.5) mandates
   either full mode coverage (preferred) or honest 400 rejection for modes a clone does not
   honor. Implementer work-order: see D-1096 (PO adjudication burst, 2026-06-11).

   **Real-crate `apply_config` gap (P20-04 ruling, 2026-06-11):** The real production crates
   (Jira `state.rs:299-337`, PagerDuty `state.rs:186-231`, Slack `state.rs:107-151`) each
   lack an `"unprocessable"` match arm in their `apply_config()` function — the Tower
   `FailureLayer` CAN serve `Unprocessable` (it is wired in `failure.rs:191`), but
   `apply_config` cannot SET it. This is a code gap in the real crates, not a BC gap —
   these surfaces are governed by the real crates' own story contracts. A production-grade
   implementer work-order for the 3 missing `"unprocessable"` arms is included in this
   amendment (see below). The work-order is NOT deferred; it is in-scope under the
   production-grade default (CLAUDE.md §Canonical Principle Rule 4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `inject_failure` called with unknown `org_slug` | Returns `Err(HarnessError::UnknownOrg)`; no HTTP call made; no side effects |
| EC-002 | `inject_failure` called with unknown `dtu_type` for a known org | Returns `Err(HarnessError::UnknownDtuType)`; no side effects |
| EC-003 | Concurrent `inject_failure` and active request to the same clone | Request in-flight at injection time completes under prior mode; next request observes injected mode |
| EC-004 | `inject_failure` called on a clone that has already crashed | Returns `Err(HarnessError::CloneCrashed { ... })`; no attempt to communicate with dead clone |
| EC-005 | `AuthReject` injection on OrgA's Claroty; OrgB's Claroty queried simultaneously | OrgA's Claroty returns 401 (Tower layer); OrgB's Claroty returns HTTP 200 with valid data — no cross-contamination. For PagerDuty the same isolation applies but the auth-reject status is 403 (see Invariant 5). |
| EC-006 | `clear_failure` called when no failure is active | Returns `Ok(())`; no state change; idempotent |
| EC-007 | `Timeout` injection with `delay_ms = 0` | Treated as `FailureMode::None` (zero delay is a no-op); returns `Ok(())`; no latency injected |
| EC-008 | `inject_failure` called with a mode not in the clone's supported-mode list (Invariant 5) | `POST /dtu/configure` returns HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant-name>"}` — no state change, no silent ACK |
| EC-009 | Caller verifies a clone's behavior after sending an unsupported mode and receiving 400 | Clone continues to return normal responses as if no configure was called; 400 is stateless with respect to prior or future injections |

## Canonical Test Vectors

| Scenario | Setup | Action | Expected OrgA Result | Expected OrgB Result | Pass Condition |
|----------|-------|--------|---------------------|---------------------|----------------|
| TV-1: AuthReject scoped to OrgA (Claroty) | harness(OrgA:Claroty, OrgB:Claroty); inject AuthReject on OrgA | Query OrgA Claroty; query OrgB Claroty | HTTP 401 (Claroty Tower-layer `FailureMode::AuthReject`) | HTTP 200 with valid data | Both asserted in same harness instance |
| TV-2: RateLimit scoped to OrgA | harness(OrgA:Claroty, OrgB:Claroty); inject RateLimit(after_n=3) on OrgA | 4 requests to OrgA; 4 requests to OrgB | First 3 OK, 4th returns 429 | All 4 return 200 | Counts match exactly |
| TV-3: MalformedResponse scoped to OrgA | harness(OrgA:Armis, OrgB:Armis); inject MalformedResponse on OrgA | Query both orgs | Response body fails JSON parse | Valid JSON response | JSON parse error only on OrgA |
| TV-4: Clear restores normal behavior | harness(OrgA:CrowdStrike); inject AuthReject; clear failure | Query after inject; query after clear | HTTP 401 (post-inject) | HTTP 200 (post-clear) | State correctly restored |
| TV-5: Unknown org returns error | harness(OrgA:Claroty) | inject_failure("unknown-org", "claroty", AuthReject) | `HarnessError::UnknownOrg` | n/a | No panic; error returned |
| TV-6: Timeout does not block OrgB | harness(OrgA:Cyberint, OrgB:Cyberint); inject Timeout(delay_ms=2000) on OrgA | Concurrent queries to both orgs | OrgA responds after ~2s | OrgB responds in < 200ms | OrgB latency unaffected |
| TV-7: Jira rejects MalformedResponse mode | harness(OrgA:Jira); POST /dtu/configure with `{"malformed_response": true}` — before full ops-clone coverage implemented | POST /dtu/configure → HTTP 400 `{"error": "unsupported_failure_mode", "mode": "MalformedResponse"}` | HTTP 400 body matches error shape | n/a | No state change; subsequent issue-creation returns HTTP 200 normally |
| TV-8: Jira accepts RateLimit mode (currently supported) | harness(OrgA:Jira); POST /dtu/configure with `{"rate_limit_after": 2}` | POST /dtu/configure → HTTP 200 `{"status": "ok"}`; 3rd issue-creation request → HTTP 429 | HTTP 429 with Retry-After header | n/a | 429 only after N requests; count is zero-reset on configure |
| TV-9: Jira accepts AuthReject mode | harness(OrgA:Jira); POST /dtu/configure with `{"auth_mode": "reject"}` | POST /dtu/configure → HTTP 200; subsequent issue-creation → HTTP 401 | HTTP 401 on every request (Jira Tower-layer `FailureMode::AuthReject`) | n/a | See `bc_3_6_001_ops_clone_failure_modes.rs::test_BC_3_6_001_jira_auth_reject_honored` |
| TV-10: PagerDuty AuthReject returns 403 (not 401) | harness(OrgA:PagerDuty); POST /dtu/configure with `{"auth_mode": "reject"}` | POST /dtu/configure → HTTP 200; subsequent enqueue → HTTP 403 with `{"status":"invalid key"}` | HTTP 403 (route-level `PagerDutyState.auth_reject`, NOT Tower `FailureMode::AuthReject`) | n/a | Authoritative source: `fidelity.rs::test_ac8_auth_reject_mode_returns_403`; `bc_3_6_001_ops_clone_failure_modes.rs::test_BC_3_6_001_pagerduty_auth_reject_honored`. The Invariant 5 per-clone table governs — not the "HTTP 401" in the v0.7 contract-header summary. |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-128 | `inject_failure` on `(OrgA, X)` does not mutate `FailureLayerShared` of any `(OrgB, Y)` where `OrgA != OrgB` | proptest (over random org pairs) |
| VP-129 | All supported `FailureMode` variants for each clone produce the documented HTTP status code or behavior (per Invariant 5 supported-mode table) | integration test (one test per variant per clone category) |
| VP-130 | `clear_failure` followed by a request to the cleared clone always returns HTTP 200 (assuming no underlying clone error) | integration test |
| VP-157 | `POST /dtu/configure` with an unsupported mode returns HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant>"}` and leaves clone state unchanged | unit test (per ops clone — Jira, PagerDuty, Slack; once-per-unsupported-mode until full coverage is ported) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 — this BC describes per-`(OrgId, DtuType)` failure injection granularity, which is a core harness capability required to test multi-tenant resilience scenarios. No existing CAP-001 through CAP-035 covers this test infrastructure concern. |
| L2 Domain Invariants | n/a (harness is test infrastructure; no DI-NNN enforced) |
| Architecture Module | prism-dtu-harness (ADR-011 §2.7); prism-dtu-common/src/layers/failure.rs (FailureLayerShared) |
| Stories | S-3.3.03, S-3.3.05, S-3.4.04, S-3.6.01, S-3.6.02 |

## Related BCs

- BC-3.5.001 — logical-mode harness that hosts the clone instances targeted by failure injection
- BC-3.5.002 — network-mode harness; failure injection applies identically in both modes **for per-org Security-Telemetry clones** (CrowdStrike, Cyberint, Armis, Claroty); MSSP-Coordination clones (Jira, PagerDuty, Slack) are Logical-mode only in this BC (Decision B, architect 2026-06-11, D-1072; network-mode support deferred to TDE write-back track)
- BC-3.6.002 — crash detection; a clone that crashes after failure injection triggers `CloneCrashed`, not silent 5xx

## Architecture Anchors

- `architecture/decisions/ADR-011-harness-isolation-modes.md#27-failure-injection-api` — defines `inject_failure`, `clear_failure`, and `FailureMode` variants
- `architecture/decisions/ADR-011-harness-isolation-modes.md#rationale` — explains why per-`(OrgId, DtuType)` granularity is required (not per-sensor-type, which would affect all orgs equally)

## Story Anchor

S-3.3.03, S-3.3.05, S-3.4.04, S-3.6.01, S-3.6.02

## VP Anchors

- VP-128 — proptest: inject_failure on (OrgA, X) does not mutate FailureLayerShared of (OrgB, Y)
- VP-129 — integration_test: all supported FailureMode variants per clone produce documented HTTP status code (per Invariant 5 table)
- VP-130 — integration_test: clear_failure followed by request always returns HTTP 200
- VP-157 — unit_test: POST /dtu/configure with unsupported mode returns HTTP 400 with unsupported_failure_mode error body; no state change

## P20-04 Implementer Work-Order: Real-Crate `Unprocessable` Arm

**Authority:** Production-grade default (CLAUDE.md §Canonical Principle Rule 4). PO adjudication
2026-06-11 (this amendment). NOT deferred to tech-debt-register — three small match arms, in scope.

**Problem:** The Tower `FailureLayer` in `prism-dtu-common/src/layers/failure.rs:191` already
handles `FailureMode::Unprocessable { at_request_n }` correctly. But the `apply_config()` method
in each of the three MSSP-Coordination real crates does not map the string `"unprocessable"` to
`FailureMode::Unprocessable`, so callers of `POST /dtu/configure` cannot activate this mode via
the real crates' configure surface.

**Fix required in each of three crates:**

### 1. `crates/prism-dtu-jira/src/state.rs`

In `apply_config()`, after the `"network_timeout"` arm and before the `other =>` fallback (currently
at state.rs ~L322 in the branch), add:

```rust
"unprocessable" => {
    let at_n = payload.at_request_n.unwrap_or(1);
    FailureMode::Unprocessable { at_request_n: at_n }
}
```

**Verify `ConfigPayload` has `at_request_n: Option<u32>` field** — it already does (shares the
field with `internal_error`). No schema change required.

**Required unit test** (in `crates/prism-dtu-jira/src/state.rs` `#[cfg(test)] mod tests`):

```rust
#[test]
fn apply_config_unprocessable_sets_failure_mode() {
    let state = JiraState::new(/* test config */);
    let config = serde_json::json!({"failure_mode": "unprocessable", "at_request_n": 2});
    state.apply_config(&config).expect("apply_config should succeed");
    let guard = state.failure_mode.lock().unwrap();
    assert!(matches!(*guard, FailureMode::Unprocessable { at_request_n: 2 }));
}
```

### 2. `crates/prism-dtu-pagerduty/src/state.rs`

Same fix: in `apply_config()`, after the `"network_timeout"` arm (~L216), add:

```rust
"unprocessable" => {
    let at_n = payload.at_request_n.unwrap_or(1);
    FailureMode::Unprocessable { at_request_n: at_n }
}
```

**Required unit test** (in `crates/prism-dtu-pagerduty/src/state.rs` `#[cfg(test)] mod tests`):

```rust
#[test]
fn apply_config_unprocessable_sets_failure_mode() {
    let state = PagerDutyState::new(/* test config */);
    let config = serde_json::json!({"failure_mode": "unprocessable", "at_request_n": 3});
    state.apply_config(&config).expect("apply_config should succeed");
    let guard = state.failure_mode.lock().unwrap();
    assert!(matches!(*guard, FailureMode::Unprocessable { at_request_n: 3 }));
}
```

### 3. `crates/prism-dtu-slack/src/state.rs`

Slack's `apply_config` currently handles only `none`, `rate_limit`, and `internal_error`. The
`"unprocessable"` arm must be added. Note that Slack's Tower FailureLayer CAN serve all
`FailureMode` variants (the wiring is in `prism-dtu-common`); the gap is only in `apply_config`.

In `apply_config()`, after the `"internal_error"` arm (~L136), add:

```rust
"unprocessable" => {
    let at_n = payload.at_request_n.unwrap_or(1);
    FailureMode::Unprocessable { at_request_n: at_n }
}
```

**Verify `SlackConfigPayload`** has `at_request_n: Option<u32>` field. If not present, add it
(annotated `#[serde(default)]`).

**Required unit test** (in `crates/prism-dtu-slack/src/state.rs` `#[cfg(test)] mod tests`):

```rust
#[test]
fn apply_config_unprocessable_sets_failure_mode() {
    let state = SlackState::new(/* test config */);
    let config = serde_json::json!({"failure_mode": "unprocessable", "at_request_n": 1});
    state.apply_config(&config).expect("apply_config should succeed");
    let guard = state.failure_mode.lock().unwrap();
    assert!(matches!(*guard, FailureMode::Unprocessable { at_request_n: 1 }));
}
```

**Verification after all three fixes:**

```bash
just iter prism-dtu-jira apply_config_unprocessable
just iter prism-dtu-pagerduty apply_config_unprocessable
just iter prism-dtu-slack apply_config_unprocessable
just check  # pre-push gate
```

**POL-29 pin sites:** No POL-29 pin-site documentation is required — this work-order adds
match arms to existing `apply_config()` functions in private production crates. No public API
surface changes; no semver impact. `#[non_exhaustive]` does not apply to `FailureMode` (it
is `prism-dtu-common` internal, not a pub-API surface type governed by the perimeter gate).

## BC Changelog

| Version | Change |
|---------|--------|
| v0.8 | P21-01 (adversary pass-21, HIGH): Internal contradiction between Postcondition 1 ("HTTP 401 universally"), Description prose ("auth-reject (401/403)"), and PagerDuty's contractual 403 behavior. Fix: (1) Postcondition 1 AuthReject line amended with explicit PagerDuty carve-out: HTTP 403 with `{"status":"invalid key","message":"Forbidden"}` matching real Events API routing-key rejection; cross-ref to Invariant 5. (2) Invariant 5 table expanded with "AuthReject HTTP Status" column — per-clone verified status codes: Claroty 401, Armis 401, CrowdStrike 401, Cyberint 401, Jira 401, PagerDuty 403, Slack 401. P21-01 mechanism note added explaining the two distinct code paths (Tower-layer 401 vs PagerDuty route-level 403). (3) TV-9 updated; TV-10 added for PagerDuty-specific 403 test vector with per-clone table governance note. (4) EC-005 updated. Source-of-truth note: the `bc_3_6_001_ops_clone_failure_modes.rs` contract-header table line 12 ("HTTP 401 on every user-facing request") remains in the test file with the inline comment at lines 426-436 acknowledging the PagerDuty 403 carve-out — the Invariant 5 per-clone table is the authoritative governing surface; the test-file header summary is non-binding documentation. Implementer sweep required: see items-for-implementer-sweep section in this amendment's return. `related_decisions` updated with P21-01. |
| v0.7 | P20-03 + P20-04 adjudication (PO, 2026-06-11): (1) Description expanded with isolation-mode scope note — Decision B (architect 2026-06-11, D-1072): mode-parity claim binds per-org Security-Telemetry clones only (CrowdStrike, Cyberint, Armis, Claroty); MSSP-Coordination clones (Jira, PagerDuty, Slack) are Logical-mode only within this BC; network-mode for MSSP-Coordination is deferred to TDE write-back track (D-1072); generic-router 404 is intentional loud failure. (2) Related BCs section: "works identically in both modes" claim scoped to Security-Telemetry clones with explicit MSSP-Coordination carve-out. (3) Invariant 5 table expanded with "Harness Isolation Mode" column; binding-scope note added (harness-local clones only; real crates governed by own story contracts). (4) P20-04 ruling recorded: Option A — Invariant 5 binds harness-local clones only; real-crate `apply_config` `Unprocessable` gap is a code defect not a BC gap. (5) P20-04 implementer work-order section added with 3 match arms + 3 unit tests (Jira, PagerDuty, Slack). `related_decisions` updated with D-1072. |
| v0.6 | POL-32 ID-collision correction (2026-06-11): VP-131 was erroneously cited in v0.5 for the unsupported-mode-400 property. VP-131 is already registered to BC-3.6.002 (clone panic detection) — POL-1 append-only collision. State-manager allocated VP-157 (VP-INDEX v1.78; verification-architecture v1.43; coverage-matrix v1.44). All three v0.5 VP-131 references (Verification Properties table, VP Anchors, and the v0.5 changelog mention) corrected to VP-157. No substantive behavior change; ID correction only. |
| v0.5 | D-1096 (PO adjudication burst, 2026-06-11): Per-clone failure-mode scope ruling. Ruling: Option (A) REJECT-UNSUPPORTED. Postcondition 1 clarified with per-clone supported-mode caveat reference. Postcondition 5 added: `POST /dtu/configure` with unsupported mode MUST return HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant>"}`. Invariant 5 added: authoritative per-clone supported-mode table. Invariant 5 initially lists ALL modes as required for all clones (cyber-sensor and ops-coordination alike) because BC-3.6.001 Postcondition 1 binds all clones uniformly — the TDE-track deferral (D-1072) scopes the ops clones' write-back FEATURE scope, not their test-infrastructure failure-injection contract. EC-008/EC-009 added for unsupported-mode 400 behavior. TV-7/TV-8/TV-9 added. VP-157 added (originally cited as VP-131 in this row — corrected by v0.6). VP-129 scope note updated to cite Invariant 5 per-clone table. Implementer work-order captured in D-1096. |
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `8606916`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md`. |
| v0.2 | Initial authoring from ADR-011. |
