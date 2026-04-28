---
document_type: behavioral-contract
level: L3
bc_id: BC-3.6.001
title: Per-Org Failure Injection
version: "0.4"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md]
input-hash: "c1610fc"
traces_to: ".factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md"
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-036
authors: [product-owner]
related_decisions: [D-044, D-045]
related_adrs: [ADR-011]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: wave-3
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
   to `mode`:
   - `FailureMode::AuthReject` → HTTP 401 on every request
   - `FailureMode::InternalError { after_n }` → HTTP 500 after N requests
   - `FailureMode::RateLimit { after_n }` → HTTP 429 after N requests
   - `FailureMode::Timeout { after_n, delay_ms }` → response delayed by `delay_ms` after N requests
   - `FailureMode::MalformedResponse` → response body is not valid JSON
2. All other `(OrgId, DtuType)` clones in the same harness return normal (non-injected)
   responses; their `FailureLayerShared` state is unchanged.
3. After `clear_failure(org_slug, dtu_type)` returns `Ok(())`, the target clone resumes
   returning normal responses; subsequent requests to that clone receive HTTP 200 with
   valid data.
4. Failure injection and clearing are idempotent: calling `inject_failure` with the same
   mode twice has the same observable effect as calling it once.

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

## Canonical Test Vectors

| Scenario | Setup | Action | Expected OrgA Result | Expected OrgB Result | Pass Condition |
|----------|-------|--------|---------------------|---------------------|----------------|
| TV-1: AuthReject scoped to OrgA | harness(OrgA:Claroty, OrgB:Claroty); inject AuthReject on OrgA | Query OrgA Claroty; query OrgB Claroty | HTTP 401 | HTTP 200 with valid data | Both asserted in same harness instance |
| TV-2: RateLimit scoped to OrgA | harness(OrgA:Claroty, OrgB:Claroty); inject RateLimit(after_n=3) on OrgA | 4 requests to OrgA; 4 requests to OrgB | First 3 OK, 4th returns 429 | All 4 return 200 | Counts match exactly |
| TV-3: MalformedResponse scoped to OrgA | harness(OrgA:Armis, OrgB:Armis); inject MalformedResponse on OrgA | Query both orgs | Response body fails JSON parse | Valid JSON response | JSON parse error only on OrgA |
| TV-4: Clear restores normal behavior | harness(OrgA:CrowdStrike); inject AuthReject; clear failure | Query after inject; query after clear | HTTP 401 (post-inject) | HTTP 200 (post-clear) | State correctly restored |
| TV-5: Unknown org returns error | harness(OrgA:Claroty) | inject_failure("unknown-org", "claroty", AuthReject) | `HarnessError::UnknownOrg` | n/a | No panic; error returned |
| TV-6: Timeout does not block OrgB | harness(OrgA:Cyberint, OrgB:Cyberint); inject Timeout(delay_ms=2000) on OrgA | Concurrent queries to both orgs | OrgA responds after ~2s | OrgB responds in < 200ms | OrgB latency unaffected |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-128 | `inject_failure` on `(OrgA, X)` does not mutate `FailureLayerShared` of any `(OrgB, Y)` where `OrgA != OrgB` | proptest (over random org pairs) |
| VP-129 | All `FailureMode` variants produce the documented HTTP status code or behavior | integration test (one test per variant) |
| VP-130 | `clear_failure` followed by a request to the cleared clone always returns HTTP 200 (assuming no underlying clone error) | integration test |

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
- VP-129 — integration_test: all FailureMode variants produce documented HTTP status code or behavior
- VP-130 — integration_test: clear_failure followed by request always returns HTTP 200

## BC Changelog

| Version | Change |
|---------|--------|
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `8606916`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md`. |
| v0.2 | Initial authoring from ADR-011. |
