---
document_type: behavioral-contract
level: L3
bc_id: BC-3.6.001
title: Per-Org Failure Injection
version: "0.5"
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
related_decisions: [D-044, D-045, D-1096]
related_adrs: [ADR-011]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: cycle-3
modified: []
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
   - `FailureMode::AuthReject` → HTTP 401 on every request
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
5. **Per-clone supported failure modes.** Not all clones support all `FailureMode`
   variants. The authoritative supported-mode table is:

   | Clone | Supported Modes | Rationale |
   |-------|----------------|-----------|
   | Claroty | ALL (AuthReject, InternalError, RateLimit, NetworkTimeout, MalformedResponse, Unprocessable) | Cyber-sensor clone; full `apply_failure_mode` coverage |
   | Armis | ALL | Cyber-sensor clone; full `apply_failure_mode` coverage |
   | CrowdStrike | ALL | Cyber-sensor clone; full `apply_failure_mode` coverage |
   | Cyberint | ALL | Cyber-sensor clone; full `apply_failure_mode` coverage |
   | Jira | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | MSSP-coordination clone; TDE-track (D-1072); full coverage required per this BC |
   | PagerDuty | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | MSSP-coordination clone; TDE-track (D-1072); full coverage required per this BC |
   | Slack | RateLimit, InternalError, AuthReject, NetworkTimeout, MalformedResponse, Unprocessable | MSSP-coordination clone; TDE-track (D-1072); full coverage required per this BC |

   A `POST /dtu/configure` call with a mode NOT listed as supported for that clone MUST
   return HTTP 400 with body `{"error": "unsupported_failure_mode", "mode": "<variant-name>"}`.
   Silent acceptance (200 ACK + no behavioral effect) is a SOUL.md §4 violation and is
   explicitly prohibited by Postcondition 5.

   **Note (2026-06-11):** The MSSP-coordination clones (Jira, PagerDuty, Slack) were
   initially implemented with route-level `match` that only honored RateLimit and
   InternalError while silently ACKing all other modes. This BC amendment (v0.5) mandates
   either full mode coverage (preferred) or honest 400 rejection for modes a clone does not
   honor. Implementer work-order: see D-1096 (PO adjudication burst, 2026-06-11).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `inject_failure` called with unknown `org_slug` | Returns `Err(HarnessError::UnknownOrg)`; no HTTP call made; no side effects |
| EC-002 | `inject_failure` called with unknown `dtu_type` for a known org | Returns `Err(HarnessError::UnknownDtuType)`; no side effects |
| EC-003 | Concurrent `inject_failure` and active request to the same clone | Request in-flight at injection time completes under prior mode; next request observes injected mode |
| EC-004 | `inject_failure` called on a clone that has already crashed | Returns `Err(HarnessError::CloneCrashed { ... })`; no attempt to communicate with dead clone |
| EC-005 | `AuthReject` injection on OrgA's Claroty; OrgB's Claroty queried simultaneously | OrgA's Claroty returns 401; OrgB's Claroty returns HTTP 200 with valid data — no cross-contamination |
| EC-006 | `clear_failure` called when no failure is active | Returns `Ok(())`; no state change; idempotent |
| EC-007 | `Timeout` injection with `delay_ms = 0` | Treated as `FailureMode::None` (zero delay is a no-op); returns `Ok(())`; no latency injected |
| EC-008 | `inject_failure` called with a mode not in the clone's supported-mode list (Invariant 5) | `POST /dtu/configure` returns HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant-name>"}` — no state change, no silent ACK |
| EC-009 | Caller verifies a clone's behavior after sending an unsupported mode and receiving 400 | Clone continues to return normal responses as if no configure was called; 400 is stateless with respect to prior or future injections |

## Canonical Test Vectors

| Scenario | Setup | Action | Expected OrgA Result | Expected OrgB Result | Pass Condition |
|----------|-------|--------|---------------------|---------------------|----------------|
| TV-1: AuthReject scoped to OrgA | harness(OrgA:Claroty, OrgB:Claroty); inject AuthReject on OrgA | Query OrgA Claroty; query OrgB Claroty | HTTP 401 | HTTP 200 with valid data | Both asserted in same harness instance |
| TV-2: RateLimit scoped to OrgA | harness(OrgA:Claroty, OrgB:Claroty); inject RateLimit(after_n=3) on OrgA | 4 requests to OrgA; 4 requests to OrgB | First 3 OK, 4th returns 429 | All 4 return 200 | Counts match exactly |
| TV-3: MalformedResponse scoped to OrgA | harness(OrgA:Armis, OrgB:Armis); inject MalformedResponse on OrgA | Query both orgs | Response body fails JSON parse | Valid JSON response | JSON parse error only on OrgA |
| TV-4: Clear restores normal behavior | harness(OrgA:CrowdStrike); inject AuthReject; clear failure | Query after inject; query after clear | HTTP 401 (post-inject) | HTTP 200 (post-clear) | State correctly restored |
| TV-5: Unknown org returns error | harness(OrgA:Claroty) | inject_failure("unknown-org", "claroty", AuthReject) | `HarnessError::UnknownOrg` | n/a | No panic; error returned |
| TV-6: Timeout does not block OrgB | harness(OrgA:Cyberint, OrgB:Cyberint); inject Timeout(delay_ms=2000) on OrgA | Concurrent queries to both orgs | OrgA responds after ~2s | OrgB responds in < 200ms | OrgB latency unaffected |
| TV-7: Jira rejects MalformedResponse mode | harness(OrgA:Jira); POST /dtu/configure with `{"malformed_response": true}` — before full ops-clone coverage implemented | POST /dtu/configure → HTTP 400 `{"error": "unsupported_failure_mode", "mode": "MalformedResponse"}` | HTTP 400 body matches error shape | n/a | No state change; subsequent issue-creation returns HTTP 200 normally |
| TV-8: Jira accepts RateLimit mode (currently supported) | harness(OrgA:Jira); POST /dtu/configure with `{"rate_limit_after": 2}` | POST /dtu/configure → HTTP 200 `{"status": "ok"}`; 3rd issue-creation request → HTTP 429 | HTTP 429 with Retry-After header | n/a | 429 only after N requests; count is zero-reset on configure |
| TV-9: Jira accepts AuthReject mode (after ops-clone full coverage) | harness(OrgA:Jira); POST /dtu/configure with `{"auth_mode": "reject"}` | POST /dtu/configure → HTTP 200; subsequent issue-creation → HTTP 401 | HTTP 401 on every request | n/a | Requires ops-clone full apply_failure_mode implementation |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-128 | `inject_failure` on `(OrgA, X)` does not mutate `FailureLayerShared` of any `(OrgB, Y)` where `OrgA != OrgB` | proptest (over random org pairs) |
| VP-129 | All supported `FailureMode` variants for each clone produce the documented HTTP status code or behavior (per Invariant 5 supported-mode table) | integration test (one test per variant per clone category) |
| VP-130 | `clear_failure` followed by a request to the cleared clone always returns HTTP 200 (assuming no underlying clone error) | integration test |
| VP-131 | `POST /dtu/configure` with an unsupported mode returns HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant>"}` and leaves clone state unchanged | unit test (per ops clone — Jira, PagerDuty, Slack; once-per-unsupported-mode until full coverage is ported) |

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
- BC-3.5.002 — network-mode harness; failure injection works identically in both modes
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
- VP-131 — unit_test: POST /dtu/configure with unsupported mode returns HTTP 400 with unsupported_failure_mode error body; no state change

## BC Changelog

| Version | Change |
|---------|--------|
| v0.5 | D-1096 (PO adjudication burst, 2026-06-11): Per-clone failure-mode scope ruling. Ruling: Option (A) REJECT-UNSUPPORTED. Postcondition 1 clarified with per-clone supported-mode caveat reference. Postcondition 5 added: `POST /dtu/configure` with unsupported mode MUST return HTTP 400 with `{"error": "unsupported_failure_mode", "mode": "<variant>"}`. Invariant 5 added: authoritative per-clone supported-mode table. Invariant 5 initially lists ALL modes as required for all clones (cyber-sensor and ops-coordination alike) because BC-3.6.001 Postcondition 1 binds all clones uniformly — the TDE-track deferral (D-1072) scopes the ops clones' write-back FEATURE scope, not their test-infrastructure failure-injection contract. EC-008/EC-009 added for unsupported-mode 400 behavior. TV-7/TV-8/TV-9 added. VP-131 added. VP-129 scope note updated to cite Invariant 5 per-clone table. Implementer work-order captured in D-1096. |
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `8606916`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md`. |
| v0.2 | Initial authoring from ADR-011. |
