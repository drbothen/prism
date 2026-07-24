---
document_type: story
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
title: "prism-dtu-cyberint + prism-spec-engine: Cyberint Auth Fidelity — Remove POST /login DTU Route; Implement StaticCookieAuthProvider; Inject Cookie: access_token={api_key}; No Session UUID"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P0
status: merged
# BC status: BC-2.01.017 authored by PO at commit b8cf19e1 (2026-05-29). All 4 BCs are
# either active (BC-2.01.013, BC-2.01.016, BC-2.16.013) or draft-pending-POL-14 (BC-2.01.017).
# status=ready: administrative flip 2026-05-29 D-850 per user direction to satisfy deliver-story
# skill strict prereq. No content/maturity change. BC-2.01.017 will auto-promote draft→active
# at this story's merge per POL-14. Prism precedent is draft-at-dispatch + POL-14-promote-at-merge;
# this flip is purely to satisfy the vsdd-factory deliver-story skill's strict prereq gate.
version: "1.10"
updated: "2026-07-24"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
modified: "2026-05-31 D-904 post-merge status→merged; PR #164 squash-merged develop@e798e67c"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns prism-dtu-cyberint and all prism-dtu-* crates per ARCH-INDEX
#     Subsystem Registry; the DTU route removal, extract_access_token rewrite, CyberintState
#     allowlist refactor, and StaticCookieAuthProvider (SS-01 auth territory) are all SS-01 work.
#   SS-16 (Spec Engine) owns prism-spec-engine including PipelineExecutor and the build_request
#     dispatch table; this story amends build_request to inject Cookie: access_token per ADR-031 §D3-b.
#   SS-17 (WASM Plugin Runtime) is NOT anchored here — SS-17's canonical name per ARCH-INDEX is
#     "WASM Plugin Runtime" (prism-spec-engine WASM tier, AD-019); it does not own DTU clones.
#     DTU clone work (prism-dtu-cyberint) is owned by SS-01 per the ARCH-INDEX crate column.
crates_touched: [prism-dtu-cyberint, prism-spec-engine, prism-bin]
target_module: prism-dtu-cyberint
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait Eliminates Per-Sensor Code Duplication — StaticCookieAuthProvider
                 # is the spec-driven auth implementation for cookie_roundtrip sensors.
  - BC-2.01.016  # SensorAuth Open Trait — auth_type_name() must return "cookie_roundtrip" for
                 # Cyberint; this story implements the correct behavior for that value.
  - BC-2.01.017  # StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection.
                 # ADR-031 §D1-b no-HTTP-call invariant; PO authored 2026-05-29 b8cf19e1.
                 # AC-005 traces to §Postconditions (no-HTTP acquire_token);
                 # AC-006 traces to §Invariants (zero HTTP calls during acquire_token);
                 # AC-010 traces to §Error Cases (E-AUTH-005 NotFound, E-AUTH-006 empty/invalid
                 #   api_key, E-AUTH-007 BackendUnavailable) + §Edge Cases EC-017-010.
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — this story
                 # restores DTU parity: DTU enforces access_token (matching real Cyberint API).
                 # cookie shape assertion now in scope per BC-2.01.017 no-login invariant.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — parity tests exercise the pipeline auth path that
            # this story fixes; after fix, a passing parity test proves real-API compatibility.
depends_on:
  - PLUGIN-MIGRATION-001-A  # Must merge first: cleans up legacy hardcoded auth modules;
                             # AuthProvider trait surface is stable before StaticCookieAuthProvider
                             # is added. S-DTU-CYBERINT-AUTH-FIDELITY-001 does NOT depend on
                             # S-DEMO-001 — it unblocks S-DEMO-001 (see blocks:).
blocks:
  - S-DEMO-001  # S-DEMO-001 depends_on this story: implements StaticCookieAuthProvider in its
                # boot step 9A. The Cyberint auth path in S-DEMO-001 (AC-003/AC-009) requires
                # the corrected DTU to be running. Per ADR-031 §D3-c.
# Dependency anchor justifications:
#   depends_on PLUGIN-MIGRATION-001-A: PLUGIN-MIGRATION-001-A deletes legacy named-auth modules
#   and establishes the AuthProvider trait surface. StaticCookieAuthProvider must be added to
#   a stable AuthProvider API. PLUGIN-MIGRATION-001-A is already merged (PR #156); this
#   dependency is satisfied.
#   blocks S-DEMO-001: S-DEMO-001 v1.3 §depends_on explicitly cites this story as a required
#   predecessor. S-DEMO-001 AC-003/AC-009 assume the corrected DTU is running.
points: 8
# Points justification:
#   DTU-side changes (prism-dtu-cyberint):
#   - Remove POST /login route from build_router + routes/auth.rs: ~0.5 pts
#   - Rewrite extract_session_token → extract_access_token in routes/alerts.rs: ~0.5 pts
#   - Refactor CyberintState session store → access_token allowlist + check_auth update: ~1 pt
#   - Update DTU tests (remove POST /login steps; use access_token cookie): ~1 pt
#   Prism-side changes (prism-spec-engine + prism-bin):
#   - StaticCookieAuthProvider in prism-spec-engine/src/auth_provider.rs: ~1.5 pts
#   - build_request CookieRoundtrip dispatch → Cookie: access_token=: ~0.5 pts
#   - Boot wiring: match CookieRoundtrip → construct StaticCookieAuthProvider: ~0.5 pts
#   - Red Gate tests (6 per AC structure below): ~2 pts
#   - BC-2.16.002 catalog row for new auth event: ~0.5 pts
#   Total: 8 points (~2-3 days of focused TDD work)
estimated_days: 3
risk: HIGH
# Risk justification:
#   The CyberintState refactor from session-store to static-token-allowlist touches
#   state management that is exercised by existing DTU tests. Any test using the
#   POST /login → cyberint_session pattern must be rewritten. The prism-side
#   build_request change is invasive (affects ALL auth-type paths' code structure)
#   even though the change is localized to CookieRoundtrip. Both sides must be tested
#   end-to-end (DTU running, pipeline connecting) to verify the cookie name is correct.
acceptance_criteria_count: 11
red_gate_tests: 7
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "DTU-side: After removing POST /login from build_router, any test that sends POST /login
    must be updated (or the route returns 404 per the 'removed' strategy in ADR-031 §D3-a).
    Prefer complete removal over a 200 no-op stub — a no-op stub hides future misuse."
  - "extract_access_token must parse the cookie value from the raw 'cookie' header (lowercase)
    per RFC 6265. The cookie name is literally 'access_token' — not 'cyberint_session',
    not 'access-token'. The adversary will probe for the exact cookie name per SAP-2."
  - "CyberintState allowlist model: at DTU startup, register demo access_token from config
    (or environment). Auth validation checks the access_token cookie value against this
    allowlist. This is still validation — just static rather than session-UUID-based."
  - "StaticCookieAuthProvider must NOT hold the credential at construction time (AD-017).
    The api_key is resolved from the credential store at acquire_token() time only."
  - "reqwest::Client timeout: any new HTTP client construction must use
    .timeout(Duration::from_secs(30)) per CLAUDE.md conventions. StaticCookieAuthProvider
    makes NO HTTP calls — this constraint applies to any adjacent HTTP client work."
  - "BC-2.16.002 Structured Event Catalog: any new tracing event_type = ... emission site
    requires a catalog row per SAP-1 standing probe. Auth provider construction event if
    emitted must be catalogued."
inputs:
  - "crates/prism-dtu-cyberint/src/clone.rs"
  - "crates/prism-dtu-cyberint/src/routes/auth.rs"
  - "crates/prism-dtu-cyberint/src/routes/alerts.rs"
  - "crates/prism-dtu-cyberint/src/state.rs"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-bin/src/boot.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.017-static-cookie-auth-provider-no-login-roundtrip.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/proposals/POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md"
  - ".factory/semport/poller-express/poller-express-broad-sweep.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DTU-CYBERINT-AUTH-FIDELITY-001 v1.9 — Cyberint DTU Auth Fidelity

**Story ID:** S-DTU-CYBERINT-AUTH-FIDELITY-001
**Status:** ready
**Version:** v1.9
**Wave:** 5
**Priority:** P0 (pre-demo BLOCKING)
**Points:** 8

---

## Origin

Established by ADR-031 DTU=true-DTU Fidelity Principle (2026-05-29), authored per user
directive 2026-05-29: "the cyberint fix needs to happen pre-demo."

ADR-031 supersedes ADR-028 §D12, which had accepted the divergence between the real
Cyberint API (`Cookie: access_token={api_key}`, no login step) and the DTU clone
(`POST /login → Set-Cookie: cyberint_session={uuid}`). That acceptance was wrong under
the DTU=true-DTU rule: a DTU that accepts a cookie name the real API does not use proves
prism can talk to its own DTU, not that prism can talk to Cyberint.

This story is the corrected canonical implementation of Cyberint auth. It is NOT a
"live auth follow-up" (the old P2-post-demo framing) — it is the P0 pre-demo-BLOCKING
auth story that makes the live demo have evidentiary value.

**Real Cyberint API behavior (canonical reference: poller-express `cookieTransport`):**
```go
func (t *cookieTransport) RoundTrip(req *http.Request) (*http.Response, error) {
    req.AddCookie(&http.Cookie{Name: "access_token", Value: t.apiKey})
    return http.DefaultTransport.RoundTrip(req)
}
```
- Cookie name: `access_token`. Not `cyberint_session`. Not `session`. `access_token`.
- No login step. The API key IS the credential; it is injected as a cookie on every request.
- Source: `.factory/semport/poller-express/poller-express-broad-sweep.md §2.1`.

---

## Narrative

As the Prism platform team, I want `prism-dtu-cyberint` corrected to accept
`Cookie: access_token={api_key}` (matching real Cyberint behavior) and
`StaticCookieAuthProvider` implemented in `prism-spec-engine` to inject that cookie, so
that the live demo against the DTU clone proves the same pipeline will work against a real
Cyberint instance — not just against prism's own (wrong) DTU.

---

## Story-Level Goal

After this story merges:

1. `prism-dtu-cyberint`'s `build_router` no longer registers `POST /login`.
2. `extract_session_token` in `routes/alerts.rs` is renamed/rewritten to
   `extract_access_token`, extracting the `access_token` cookie value.
3. `CyberintState`'s session store model changes from per-UUID session tokens to a static
   `access_token` allowlist; `check_auth` validates `access_token` cookie against the
   allowlist.
4. All DTU tests that previously sent `POST /login → cyberint_session` are rewritten to
   use `Cookie: access_token={value}` directly.
5. `StaticCookieAuthProvider` exists in `prism-spec-engine/src/auth_provider.rs` (or
   a new `static_cookie_auth_provider.rs` module) as a `pub` production type (NOT
   feature-gated); it reads the API key from the credential store at `acquire_token()` time
   and returns it as an `AuthToken`; it makes NO HTTP call during `acquire_token`.
6. `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`) injects `Cookie: access_token={token}` when
   `auth_type == AuthType::CookieRoundtrip` (replacing the former `Cookie: cyberint_session`
   injection, which was the ADR-028 §D12 value — now superseded by ADR-031 §D3-b).
7. Boot wiring in `prism-bin/src/boot.rs` (or in S-DEMO-001's boot step 9A — these are
   ordered, but the pattern is established here): `CookieRoundtrip` auth_type with no
   `auth_plugin` field → construct `StaticCookieAuthProvider`.
8. A parity test with `Cookie: access_token=<value>` succeeds; a parity test with
   `Cookie: cyberint_session=<value>` returns 401.

---

## Behavioral Contracts

| BC ID | Title | Role in This Story |
|-------|-------|-------------------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | StaticCookieAuthProvider is the spec-driven auth implementation for cookie_roundtrip sensors; AC-005/AC-006 cover it. |
| BC-2.01.016 | SensorAuth Open Trait — Plugin-Implementable Auth Contract | auth_type_name() for Cyberint must return "cookie_roundtrip"; the BEHAVIOR of that type changes in this story. AC-006 covers the trait contract. |
| BC-2.01.017 | StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection | Canonical contract for StaticCookieAuthProvider. §Postconditions: acquire_token returns api_key without HTTP call. §Invariants: zero HTTP calls during acquire_token (ADR-031 §D1-b). §Error Cases: E-AUTH-005 (NotFound), E-AUTH-006 (empty/whitespace/illegal-char/oversized api_key), E-AUTH-007 (BackendUnavailable — credential backend unavailable). §Edge Cases EC-017-010: BackendUnavailable → E-AUTH-007. AC-005/AC-006/AC-010 implement it. PO authored 2026-05-29 b8cf19e1. |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification | DTU parity: after this story, DTU access_token cookie enforcement matches real Cyberint API. AC-001/AC-004 cover DTU parity. Cookie shape assertion now in scope per BC-2.01.017 no-login invariant. |

---

## Acceptance Criteria

### DTU-Side ACs

These ACs cover changes to `crates/prism-dtu-cyberint`. They must be GREEN before
the prism-side ACs are tested against the running DTU.

#### AC-001: POST /login route removed from DTU build_router
`crates/prism-dtu-cyberint/src/clone.rs` `build_router` does NOT register a route for
`POST /login`. The `post_login` handler in `routes/auth.rs` is either deleted or not
referenced. A test request to `POST /login` on a running DTU clone returns 404 (route
not found).
(traces to BC-2.16.013 postcondition — DTU parity requires only endpoints the real API
exposes; real Cyberint has no login endpoint per poller-express-broad-sweep §2.1)
Red Gate test: `test_BC_2_16_013_dtu_post_login_route_removed_returns_404`

#### AC-002: extract_access_token extracts the access_token cookie value
A new function `extract_access_token(headers: &HeaderMap) -> Option<String>` exists in
`crates/prism-dtu-cyberint/src/routes/alerts.rs` (replacing `extract_session_token`).
It parses the raw `cookie` header value (case-insensitive header lookup, lowercase `cookie`
per HTTP/1.1 normalization) and returns the value of the cookie named `access_token`.
It returns `None` if the cookie header is absent, empty, or no `access_token` cookie is
present. It returns `None` (NOT the `cyberint_session` value) if only a `cyberint_session`
cookie is present.
(traces to BC-2.01.013 precondition — auth validation must use the correct credential form
per the adapter pattern contract)
Red Gate test: `test_BC_2_01_017_dtu_extract_access_token_parses_cookie_header`

#### AC-003: DTU check_auth validates access_token cookie, not session UUID
`check_auth` in `routes/alerts.rs` calls `extract_access_token` (not `extract_session_token`).
A request with `Cookie: access_token={registered_token}` is admitted (returns 200 from the
alerts endpoint). A request with NO cookie header returns 401. A request with
`Cookie: cyberint_session={any_value}` returns 401 (the `cyberint_session` name is the
pre-fix wrong name and must be rejected by the corrected DTU).
(traces to BC-2.16.013 postcondition — DTU enforces access_token, not cyberint_session;
ADR-031 §D1-a: cookie names must match the real API)
Red Gate test: `test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session`

#### AC-004: CyberintState static-token allowlist replaces session store
`CyberintState` (in `src/state.rs`) registers an `access_token` allowlist at construction
time (or via a `configure()` call that provides the demo token value). Auth validation
checks the extracted `access_token` cookie value against this allowlist.
The `session_store` HashMap (previously keyed by `(OrgId, String)` for UUID session tokens)
is either removed or repurposed for the `access_token` allowlist. Session UUID issuance
code is deleted; no `Uuid::new_v4()` call exists in auth logic.
(traces to BC-2.16.013 invariant — D2-d of ADR-031 permits persistence-reset on restart;
the static-token model is correct per D1-b — real API issues no session UUIDs)
Red Gate test: `test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid`

### Prism-Side ACs

These ACs cover changes to `crates/prism-spec-engine` and `crates/prism-bin`. They depend
on the DTU-side ACs being complete because integration tests run against the corrected DTU.

#### AC-005: StaticCookieAuthProvider implements AuthProvider with no HTTP calls
`StaticCookieAuthProvider` exists as a `pub` struct in `prism-spec-engine/src/auth_provider.rs`
(or a new `static_cookie_auth_provider.rs` module in the same crate).
Constructor: `StaticCookieAuthProvider::new(sensor_id: SensorId, credential_resolver: Arc<dyn CredentialResolver>) -> Self`.
`AuthProvider::acquire_token()` implementation:
- Resolves the API key credential via `credential_resolver.resolve(sensor_id, "api_key")`.
- Returns `Ok(AuthToken::new(api_key_value))`.
- Makes NO HTTP call during `acquire_token`. The token returned is the raw API key.
- On credential resolution failure: returns `Err(SpecEngineError::AuthAcquisitionFailed { ... })`.
(traces to BC-2.01.013 postcondition 4 — spec-driven adapter auth provider; ADR-031 §D3-b rule 2;
 traces to BC-2.01.017 §Postconditions — acquire_token returns api_key value without any HTTP call)
Red Gate test: `test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call`

#### AC-006: StaticCookieAuthProvider::acquire_token never issues an HTTP request
When `StaticCookieAuthProvider::acquire_token()` is called in any test environment (including
tests with a mock HTTP server), the mock server receives ZERO HTTP requests during
`acquire_token`. The `acquire_token` method is a pure credential-store read with no network I/O.
This is a mandatory invariant per ADR-031 §D1-b: if the real API requires no login step,
the auth provider must not introduce one.
(traces to BC-2.01.016 postcondition — auth contract: acquire_token for cookie_roundtrip
sensor is a credential read, not an HTTP exchange;
 traces to BC-2.01.017 §Invariants — zero HTTP calls during acquire_token is a hard invariant
 of the StaticCookieAuthProvider contract; any HTTP call is a BC-2.01.017 violation)
Red Gate test: `test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call`

#### AC-007: build_request injects Cookie: access_token header for CookieRoundtrip
`build_request` dispatches the auth header based on `auth_type`:
- `AuthType::CookieRoundtrip`: sets the HTTP request header `Cookie: access_token={token}`.
  NOT `Cookie: cyberint_session={token}` (that was the ADR-028 §D12 value, superseded).
  NOT `Authorization: Bearer {token}`.
- All other `AuthType` variants: behavior unchanged (Authorization: Bearer).
(traces to BC-2.01.013 postcondition — pipeline injects correct auth form per sensor's auth_type)
Red Gate test: `test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip`

#### AC-008: End-to-end parity: access_token cookie → DTU alerts endpoint returns data
Integration test against a running `CyberintClone` (corrected DTU): a pipeline call for
sensor `cyberint` with a `StaticCookieAuthProvider` holding demo token `demo-access-key`
results in an HTTP request with `Cookie: access_token=demo-access-key` reaching the DTU.
The DTU returns 200 with alert data. The pipeline returns non-empty `Vec<RecordBatch>`.
(traces to BC-2.16.013 postcondition — parity test proves DTU + prism use the same cookie
name; ADR-031 §D5 validation discipline: parity tests must assert cookie NAMES match real API)

#### AC-009: Negative parity: cyberint_session cookie returns 401 from corrected DTU
Integration test against a running `CyberintClone` (corrected DTU): a pipeline call with
a synthetic `StaticCookieAuthProvider` that injects `Cookie: cyberint_session=anything`
(or a test that directly sends the wrong cookie name) results in the DTU returning 401.
This proves the DTU fidelity correction is in force: the old pre-fix cookie name is
explicitly rejected, not silently ignored.
(traces to BC-2.16.013 invariant — ADR-031 §D1-a and §D5: parity tests must assert
cookie names match and reject wrong names)

#### AC-010: E-AUTH-005/E-AUTH-006/E-AUTH-007 surfaced on auth failure (error taxonomy compliance)
`StaticCookieAuthProvider::acquire_token` called when the credential resolver returns
`CredentialResolutionError::NotFound` for `(client_id, sensor_id)` → error propagates as
`SpecEngineError::AuthAcquisitionFailed` with detail matching the **E-AUTH-005** taxonomy
entry (`"Credentials not found for ({client_id}, {sensor_id})"`). No panic; no generic
error; the error code is **E-AUTH-005** as defined in
`.factory/specs/prd-supplements/error-taxonomy.md`.
(traces to BC-2.01.017 §Error Cases row 1 — credential resolver returns NotFound →
AuthAcquisitionFailed carrying E-AUTH-005; TV-BC-2.01.017-004 is the canonical test vector)
When the api_key credential is present but is empty, all-whitespace, contains illegal
characters, or exceeds the maximum allowed length, the error code is `E-AUTH-006` per
the E-AUTH-006 entry in error-taxonomy.md (introduced with BC-2.01.017).
(traces to BC-2.01.013 error case — adapter error taxonomy compliance;
 traces to BC-2.01.017 §Edge Cases — E-AUTH-006 on empty/whitespace/illegal-char/oversized
 api_key credential per the E-AUTH-006 entry in error-taxonomy.md)
When the credential resolver returns `CredentialResolutionError::BackendUnavailable` (the
credential backend — e.g., keyring, vault, env store — is temporarily unavailable), the
error propagates as `SpecEngineError::AuthAcquisitionFailed` with error code **E-AUTH-007**.
No HTTP fetch is attempted. No retry is issued.
(traces to BC-2.01.017 §Error Cases row 3 — credential resolver returns BackendUnavailable →
AuthAcquisitionFailed carrying E-AUTH-007; BC-2.01.017 EC-017-010 is the canonical edge case;
TV-BC-2.01.017-009 is the canonical test vector)

#### AC-011: No event_type emission without BC-2.16.002 catalog row (SAP-1)
If any new `tracing::*!(event_type = ...)` site is introduced in this story's implementation
(e.g., a `auth_provider.constructed` event at boot wiring), it must have a corresponding
row in BC-2.16.002 Structured Event Catalog with full field schema, audit role, and
recurrence policy. Zero uncatalogued `event_type` emissions are permitted.
(traces to BC-2.16.013 postcondition — SAP-1 standing adversary probe enforced at every pass)

---

## Red Gate Tests

| Test Name | AC | Crate | Description |
|-----------|----|-------|-------------|
| `test_BC_2_16_013_dtu_post_login_route_removed_returns_404` | AC-001 | prism-dtu-cyberint | POST /login to running DTU returns 404 |
| `test_BC_2_01_017_dtu_extract_access_token_parses_cookie_header` | AC-002 | prism-dtu-cyberint | extract_access_token extracts access_token; returns None for cyberint_session |
| `test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session` | AC-003 | prism-dtu-cyberint | check_auth: access_token cookie → 200; cyberint_session cookie → 401 |
| `test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid` | AC-004 | prism-dtu-cyberint | CyberintState: no UUID issuance; allowlist-based validation |
| `test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call` | AC-005 | prism-spec-engine | StaticCookieAuthProvider::acquire_token returns api_key; no HTTP call (BC-2.01.017 §Postconditions) |
| `test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call` | AC-006 | prism-spec-engine | Zero HTTP calls during acquire_token (BC-2.01.017 §Invariants — no-HTTP-call invariant) |
| `test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip` | AC-007 | prism-spec-engine | build_request dispatches Cookie: access_token for CookieRoundtrip |

---

## Tasks

### DTU-Side Tasks (complete first; prism-side tests run against corrected DTU)

1. **Read** `crates/prism-dtu-cyberint/src/routes/auth.rs` — understand current `post_login`
   handler; note session store interaction (`register_session`, `CyberintState`).
2. **Read** `crates/prism-dtu-cyberint/src/routes/alerts.rs` — understand `extract_session_token`,
   `check_auth`; note how `is_valid_session` is called.
3. **Read** `crates/prism-dtu-cyberint/src/state.rs` — understand `CyberintState` struct;
   find `session_store`, `register_session`, `is_valid_session` methods.
4. **Read** `crates/prism-dtu-cyberint/src/clone.rs` `build_router` — find `post_login` route
   registration.
5. **Write stub** (empty implementations) for:
   - New `extract_access_token(headers: &HeaderMap) -> Option<String>` function in alerts.rs
   - `check_auth` updated to call `extract_access_token` (stubs can have `todo!()` bodies)
6. **Write Red Gate tests** (must ALL FAIL before implementation):
   - `test_BC_2_16_013_dtu_post_login_route_removed_returns_404`
   - `test_BC_2_01_017_dtu_extract_access_token_parses_cookie_header`
   - `test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session`
   - `test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid`
   Verify they FAIL (RED gate confirmed) before proceeding to step 7.
7. **Remove** `POST /login` route from `build_router` in `clone.rs`.
   Delete or remove the `use` of `routes::auth::post_login` from `clone.rs`.
   If `routes/auth.rs` becomes empty, either delete the file or leave only the module declaration.
8. **Implement** `extract_access_token` in `routes/alerts.rs`:
   ```rust
   pub fn extract_access_token(headers: &HeaderMap) -> Option<String> {
       let cookie_header = headers.get("cookie")?.to_str().ok()?;
       for pair in cookie_header.split(';') {
           let pair = pair.trim();
           if let Some(val) = pair.strip_prefix("access_token=") {
               return Some(val.to_owned());
           }
       }
       None
   }
   ```
   This is the exact function from ADR-031 §D3-a. The cookie name is `access_token`.
   Remove `extract_session_token` (or mark it `#[deprecated]` with a cfg note if it has
   external test callers that haven't been updated yet — but prefer full removal).
9. **Update `CyberintState`** in `src/state.rs`:
   - Replace `session_store: Mutex<HashMap<(OrgId, String), SessionRecord>>` with
     `access_token_allowlist: Mutex<HashSet<String>>` (or equivalent structure).
   - Replace `register_session(org_id, token)` with `register_access_token(token: String)`.
   - Replace `is_valid_session(org_id, token)` with `is_valid_access_token(token: &str) -> bool`.
   - Remove UUID session issuance (`Uuid::new_v4()`) from auth logic entirely.
   - Add `with_demo_token(demo_token: String) -> Self` constructor variant (or register via
     `configure()`) so test harnesses can specify the allowed access_token at startup.
10. **Update `check_auth`** in `routes/alerts.rs`:
    - Call `extract_access_token(headers)` instead of `extract_session_token`.
    - Validate result with `state.is_valid_access_token(&token)` instead of `is_valid_session`.
    - Remove the `org_id` extraction from auth logic (access_token is org-agnostic per real API).
    - Keep rate limit check unchanged.
11. **Update existing DTU tests**: find all tests that call `POST /login` or use
    `cyberint_session` cookie in request headers. Rewrite them to:
    - Register demo token at DTU startup via `configure()` or `with_demo_token()`.
    - Send `Cookie: access_token={demo_token}` in the request header.
    - Assert the correct response (200 for valid token; 401 for missing/wrong token).
12. **Run DTU tests**: `cargo nextest run -p prism-dtu-cyberint` — all must pass GREEN.
13. **Verify cookie name in test assertions**: search `prism-dtu-cyberint` test files for any
    remaining `cyberint_session` string. There must be ZERO occurrences except in a negative
    test asserting that `cyberint_session` is rejected (AC-003 Red Gate test).

### Prism-Side Tasks (complete after DTU-side tasks pass)

14. **Read** `crates/prism-spec-engine/src/auth_provider.rs` — understand `AuthProvider` trait
    signature (`acquire_token` returns `Pin<Box<dyn Future<...>>>`); understand `AuthToken`.
15. **Read** `crates/prism-spec-engine/src/pipeline.rs` — find `build_request`; understand
    current `Authorization: Bearer {token}` header injection; find `AuthType` import.
16. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — confirm `AuthType::CookieRoundtrip`
    variant exists; note its exact variant name.
17. **Read** `crates/prism-bin/src/boot.rs` — understand boot step 7.5b (PluginAuthProvider
    construction) and the pattern for constructing auth providers at boot time.
18. **Write stubs** for:
    - `StaticCookieAuthProvider` struct + `AuthProvider` impl in `auth_provider.rs` (stub bodies)
    - Updated `build_request` dispatch arm for `CookieRoundtrip` (stub)
19. **Write Red Gate tests** (must ALL FAIL before implementation):
    - `test_BC_2_01_017_static_cookie_auth_provider_returns_api_key_without_http_call`
    - `test_BC_2_01_017_static_cookie_auth_provider_acquire_token_no_http_call`
    - `test_BC_2_01_017_build_request_injects_access_token_cookie_for_cookie_roundtrip`
    Verify they FAIL (RED gate confirmed) before proceeding to step 20.
20. **Implement `StaticCookieAuthProvider`** in `prism-spec-engine/src/auth_provider.rs`:
    ```rust
    /// Production auth provider for sensors using `auth_type = "cookie_roundtrip"`.
    ///
    /// Reads the API key from the credential store at acquire_token() time.
    /// Makes NO HTTP call. Returns the raw API key as the AuthToken.
    ///
    /// Per ADR-031 §D3-b rule 2 and AD-017: credentials are never held at construction time.
    pub struct StaticCookieAuthProvider {
        sensor_id: SensorId,
        credential_resolver: Arc<dyn CredentialResolver>,
    }
    ```
    Constructor: `pub fn new(sensor_id: SensorId, credential_resolver: Arc<dyn CredentialResolver>) -> Self`
    `acquire_token` impl: call `credential_resolver.resolve(&sensor_id, "api_key")`; wrap
    result in `AuthToken::new(...)`. On `CredentialResolutionError::NotFound`: return
    `SpecEngineError::AuthAcquisitionFailed` with **E-AUTH-005** message template
    (`"Credentials not found for ({client_id}, {sensor_id})"`). No HTTP fetch attempted.
    This is a `pub` type, NOT feature-gated (it is a production type per ADR-031 §D3-b).
21. **Amend `build_request`** in `pipeline.rs`:
    - Add `auth_type: &AuthType` parameter (or read from the `SensorSpec` already in scope).
    - Add dispatch arm:
      ```rust
      AuthType::CookieRoundtrip => {
          request_builder.header("cookie", format!("access_token={}", token.as_str()))
      }
      ```
      Cookie name MUST be `access_token` (all-lowercase). NOT `cyberint_session`. NOT
      `Access-Token`. The adversary probes for this specifically per SAP-2.
    - All other `AuthType` variants: existing `Authorization: Bearer {token}` behavior
      unchanged.
    - Update all callers of `build_request` to pass the `auth_type` argument.
22. **Verify no HTTP call during acquire_token**: if using a mock HTTP server in tests,
    add an assertion that the server received 0 requests from the `acquire_token` call path.
    This satisfies AC-006 (ADR-031 §D1-b invariant).
23. **If any new tracing event is emitted** in this story's implementation:
    - Add a BC-2.16.002 Structured Event Catalog row with full field schema, audit role,
      and recurrence policy (SAP-1 standing probe).
    - If NO new `event_type = ...` sites are added, this task is complete (zero catalog rows
      needed for zero new emissions).
24. **Run prism-spec-engine tests**: `cargo nextest run -p prism-spec-engine` — all GREEN.
25. **Run integration test** (end-to-end, AC-008): start `CyberintClone` with `demo-access-key`
    as the registered token; call the pipeline with `StaticCookieAuthProvider`; assert
    non-empty `Vec<RecordBatch>` returned.
26. **Run negative integration test** (AC-009): send `Cookie: cyberint_session=anything`
    to the corrected DTU; assert 401.
27. **Run** `just check` — final pre-push gate (fmt + clippy + nextest + doctests + layout).

---

## File List

### DTU-Side Files

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Remove `use routes::auth::post_login`; remove `.route("/login", post(post_login))` from `build_router` |
| `crates/prism-dtu-cyberint/src/routes/auth.rs` | MODIFY or DELETE | Remove `post_login` handler; if file becomes empty, delete it (or keep as empty module for future use) |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | MODIFY | Replace `extract_session_token` with `extract_access_token`; update `check_auth` to use `extract_access_token` and `is_valid_access_token` |
| `crates/prism-dtu-cyberint/src/routes/mod.rs` | MODIFY | Remove `pub mod auth;` if auth.rs is deleted |
| `crates/prism-dtu-cyberint/src/state.rs` | MODIFY | Replace session_store with access_token_allowlist; replace `register_session`/`is_valid_session` with `register_access_token`/`is_valid_access_token`; remove UUID issuance |
| `crates/prism-dtu-cyberint/tests/` (existing test files) | MODIFY | Update all tests that use `POST /login` or `cyberint_session` cookie to use `access_token` cookie pattern |

### Prism-Side Files

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/auth_provider.rs` | MODIFY | Add `StaticCookieAuthProvider` struct + `AuthProvider` impl (production type, pub, NOT feature-gated) |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Amend `build_request` to dispatch `Cookie: access_token={token}` for `AuthType::CookieRoundtrip` |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY (if applicable) | Add catalog row for any new `event_type = ...` emission (SAP-1); if no new emission, no change needed |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| DTU cookie name MUST be `access_token` — not `cyberint_session` | ADR-031 §D1-a; poller-express `cookieTransport` | Adversary probes for exact cookie name per SAP-2 on every pass |
| DTU MUST NOT have a `POST /login` route | ADR-031 §D1-b; real Cyberint API has no login step | AC-001 Red Gate test; `build_router` must not register it |
| `StaticCookieAuthProvider::acquire_token` MUST NOT make HTTP calls | ADR-031 §D1-b; D3-b rule 2 | AC-006 Red Gate test; any HTTP call during acquire_token is an ADR-031 violation |
| `StaticCookieAuthProvider` lives in `prism-spec-engine`, NOT `prism-bin` | Crate cohesion — it is a pure credential-read provider; no prism-bin types needed | ADR-023 §Permitted Patterns alignment |
| `StaticCookieAuthProvider` is NOT feature-gated (`#[cfg(test)]` or `test-helpers`) | It is a production type used at boot step 9A in S-DEMO-001 | Production code path; must be `pub` without cfg gate |
| AD-017: credentials never held at construction time | AD-017 credential safety | `acquire_token` resolves at call time; no credential field in struct |
| No `println!` in production code | CLAUDE.md Conventions | Use `tracing::*!` with structured fields only |
| New `event_type = ...` emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary greps `event_type =` on every pass |

### Forbidden Dependencies

`prism-spec-engine` (where `StaticCookieAuthProvider` lives) MUST NOT gain a new dependency
on `prism-dtu-cyberint`. The DTU is a test fixture; production spec-engine code must not
import it. If the build gains this dependency, it MUST fail.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | AuthProvider trait, SpecEngineError, AuthToken newtype |
| `prism-dtu-cyberint` (workspace) | current workspace path | DTU clone being corrected |
| `axum` | workspace version | DTU route handlers |
| `uuid` | workspace version | REMOVED from auth logic (no UUID session tokens) |
| `zeroize` | workspace version | AuthToken zeroize (existing; do not add new dep) |
| `tokio` | workspace version | async acquire_token implementation |

Version source: `Cargo.toml` workspace `[dependencies]` table. Do not pin versions independently.
Note: `uuid` may be removed from `prism-dtu-cyberint/Cargo.toml` if `post_login` was its
only consumer. Verify before removing — uuid may still be used for other DTU state.

---

## Previous Story Intelligence

N/A — first story in E-DTU-FIDELITY epic. Key lessons from adjacent stories:

- **PLUGIN-MIGRATION-001-D** (merged PR #153): Delivered the 4 TOML sensor specs including
  `cyberint.sensor.toml` with `auth_type = "cookie_roundtrip"` (D-747 LOCKED). Do NOT
  change the `auth_type` value — the BEHAVIOR changes, not the label.

- **S-PLUGIN-PREREQ-B** (merged): Delivered `PipelineExecutor::execute()` and `build_request`.
  Read `pipeline.rs` before modifying `build_request` — understand the full request-building
  pipeline and all callers of `build_request` to ensure they all pass the new `auth_type`
  parameter (sibling-site sweep per TD-VSDD-060).

- **PLUGIN-MIGRATION-001-A** (merged PR #156): Cleaned up legacy hardcoded auth modules.
  The `AuthProvider` trait surface is stable. `StaticCookieAuthProvider` will be added
  alongside `NullAuthProvider`, `MockAuthProvider`, etc. in `auth_provider.rs`.

- **S-DEMO-001 v1.3** (draft): Depends on this story. S-DEMO-001 implements
  `StaticCookieAuthProvider` usage at boot step 9A. This story provides the type; S-DEMO-001
  wires it into the boot sequence. Do NOT add boot-step-9A code here — that is S-DEMO-001 scope.
  This story delivers: (a) DTU correction, (b) `StaticCookieAuthProvider` type, (c)
  `build_request` dispatch amendment. S-DEMO-001 delivers: (d) boot step 9A wiring.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU starts with no registered access_token (empty allowlist) | Any request → 401; configure() must be called to register a token before use |
| EC-002 | access_token cookie value is empty string | `extract_access_token` returns `Some("")`; `is_valid_access_token("")` returns false → 401 |
| EC-003 | Cookie header contains multiple cookies: `access_token=val1; session=val2` | `extract_access_token` returns `Some("val1")` — only the `access_token` cookie value |
| EC-004 | Cookie header uses mixed case: `Cookie: Access-Token=val` | Real RFC 6265 cookies are case-sensitive in name; `Access-Token` is NOT `access_token`; returns None → 401 (real Cyberint likely sends lowercase) |
| EC-005 | `StaticCookieAuthProvider::acquire_token` called when credential resolver returns `CredentialResolutionError::NotFound` for `(client_id, sensor_id)` | Returns `Err(SpecEngineError::AuthAcquisitionFailed)` carrying **E-AUTH-005** message (`"Credentials not found for ({client_id}, {sensor_id})"`); no HTTP fetch attempted. (BC-2.01.017 §Error Cases row 1; TV-BC-2.01.017-004) |
| EC-006 | Cyberint DTU returns 401 during data fetch after a valid access_token was injected (e.g., DTU reset mid-session or stale key) | Pipeline surfaces **E-AUTH-004** in `sensor_errors` per BC-2.01.010 partial-failure contract. No retry. No second `acquire_token` call. Call count == 1. `StaticCookieAuthProvider` has no refreshable token — no retry is correct per BC-2.01.017 §Edge Cases EC-017-002. Operator action: update the static API key in the keyring. (BC-2.01.017 §Edge Cases EC-017-002; TV-BC-2.01.017-006) |
| EC-007 | test sends both `cyberint_session` AND `access_token` cookies in same header | `extract_access_token` finds `access_token` first (if ordered before `cyberint_session`) or whichever appears first; test should be deterministic — only include one or the other |
| EC-008 | `uuid` crate removed from prism-dtu-cyberint Cargo.toml | Verify no other DTU code uses uuid (beyond auth.rs); if it does, retain the dep; if auth.rs was the sole consumer, remove cleanly |
| EC-009 | `StaticCookieAuthProvider::acquire_token` called when credential backend is unavailable (`CredentialResolutionError::BackendUnavailable`) | Returns `Err(SpecEngineError::AuthAcquisitionFailed)` carrying **E-AUTH-007** (`"Credential backend unavailable for ({client_id}, {sensor_id})"`); no HTTP fetch attempted; no retry issued. (BC-2.01.017 §Error Cases row 3; BC-2.01.017 EC-017-010; TV-BC-2.01.017-009) |

---

## Notes for Implementer

**Test naming convention — F-LP6-LOW-001 v1.3 amendment:** Test function names use the
prefix of the BC they DIRECTLY PROVE (primary-BC prefix convention, project-wide). Each
test in the Red Gate table has a `test_BC_X_XX_XXX_` prefix that matches the BC whose
postcondition/invariant the test verifiably exercises. Tests for `StaticCookieAuthProvider`
behavior (AC-002, AC-005, AC-006, AC-007) use `test_BC_2_01_017_*` because BC-2.01.017
is the StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection; tests
proving DTU route/state behavior (AC-001, AC-003, AC-004) use `test_BC_2_16_013_*`
because BC-2.16.013 is the DTU parity contract. BC-2.01.013 and BC-2.01.016 are in
`behavioral_contracts:` (their invariants are covered by the full test suite), but no
Red Gate test uses their prefix because no individual test independently proves their
specific postconditions/invariants more directly than it proves BC-2.01.017 or
BC-2.16.013. Implementer must use the exact test names listed in §Red Gate Tests.

**BC-2.01.017 — canonical contract for StaticCookieAuthProvider:** BC-2.01.017 (authored
by PO at commit b8cf19e1, 2026-05-29) is the primary behavioral contract for this story's
key deliverable. Read it before implementing `StaticCookieAuthProvider`. Its §Postconditions
assert AC-005 (acquire_token returns api_key without HTTP call); its §Invariants assert
AC-006 (zero HTTP calls during acquire_token per ADR-031 §D1-b); its §Edge Cases assert
AC-010 (E-AUTH-006 on empty/whitespace/illegal-char/oversized api_key). The 8 canonical
test vectors in BC-2.01.017 are the ground truth for Red Gate test design.

**Cookie name invariant (CRITICAL):** The cookie name on the outbound HTTP request MUST be
`access_token`. Not `cyberint_session`. Not `Access_Token`. Not `session`. The adversary
will assert the exact cookie name per SAP-2 (DTU↔TOML schema parity probe, extended here
to DTU↔real-API parity). Any implementation that uses the wrong cookie name is a bug, not
a style issue.

**AD-017 credential discipline:** The `api_key` value must never appear in log output,
tracing events, or error messages. `AuthToken` uses `Zeroizing<String>` and a redacted
`Debug` impl — use it correctly. The `as_str()` method is for HTTP header injection only.

**boot.rs scope boundary:** Do NOT add boot step 9A code to this story. Boot step 9A
(iterating the ResolvedSensorSpec map and constructing `StaticCookieAuthProvider` for each
org's Cyberint sensor) is the scope of S-DEMO-001. This story's scope ends at:
(a) DTU correction, (b) `StaticCookieAuthProvider` type in `auth_provider.rs`,
(c) `build_request` `CookieRoundtrip` dispatch amendment.

**D-747 LOCKED value preserved:** `auth_type = "cookie_roundtrip"` in `cyberint.sensor.toml`
is NOT changed by this story. The enum variant `AuthType::CookieRoundtrip` is NOT renamed.
Only the BEHAVIOR of the CookieRoundtrip path changes.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| DTU state refactor breaks existing multi-org tests | Read all existing prism-dtu-cyberint tests before modifying state.rs; update each test before moving to the next to maintain a green test suite |
| `build_request` callers not updated after signature change | Sibling-site sweep (TD-VSDD-060): `rg 'build_request' crates/prism-spec-engine/src/'` before committing; all callers must pass auth_type |
| New `event_type` emission uncatalogued | SAP-1 sweep after implementation: `rg 'event_type\s*=' crates/ --type rust` on full workspace; zero new emissions without catalog rows |
| `StaticCookieAuthProvider` accidentally feature-gated | Verify no `#[cfg(test)]` or `#[cfg(feature = "test-helpers")]` annotation on the struct or its impl block — it is a production type |
| uuid crate dependency dangling after auth.rs removal | After removing post_login, run `cargo check -p prism-dtu-cyberint`; if uuid is still in Cargo.toml but unused, cargo will warn (udeps will catch it); remove cleanly |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| BC files (4 BCs) | ~6,000 |
| crates/prism-dtu-cyberint/src/routes/auth.rs | ~500 |
| crates/prism-dtu-cyberint/src/routes/alerts.rs | ~3,000 |
| crates/prism-dtu-cyberint/src/state.rs | ~2,000 |
| crates/prism-dtu-cyberint/src/clone.rs | ~2,500 |
| crates/prism-spec-engine/src/auth_provider.rs | ~4,000 |
| crates/prism-spec-engine/src/pipeline.rs | ~6,000 |
| crates/prism-spec-engine/src/spec_parser.rs (AuthType enum only) | ~1,500 |
| crates/prism-bin/src/boot.rs (context only; no code changes in this story) | ~3,000 |
| ADR-031 (full) | ~3,500 |
| POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §4 (Cyberint section) | ~2,000 |
| error-taxonomy.md (E-AUTH-NNN section) | ~500 |
| Test files (prism-dtu-cyberint/tests/) | ~3,000 |
| Tool outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~44,000 tokens (~17% of 256K context)** |

Well within the 20-30% budget.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.10 | 2026-07-24 | story-writer | F-WASE-P52-LOW-001 POL-29 class sweep, burst wave-a-spec-evolution-fix-burst-41: 3 occurrences of stale `PipelineExecutor::build_request` qualifier corrected to free-function citation — first mention (§Acceptance Criteria item 6) uses `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`), subsequent mentions plain `build_request`. No ACs, BCs, or behavioral semantics changed. |
| 1.9 | 2026-05-30 | story-writer FB-PR6 | F-P11-HIGH-001 closure (SS-17 mis-anchor drop): `subsystems:` corrected from `[SS-01, SS-16, SS-17]` to `[SS-01, SS-16]`. SS-17's canonical name per ARCH-INDEX Subsystem Registry is "WASM Plugin Runtime" (prism-spec-engine WASM tier, AD-019) — it does NOT own DTU clones. DTU clone work (prism-dtu-cyberint) is owned by SS-01 (Sensor Adapters), which was already the story's first anchor. Removed incorrect "SS-17 (DTU Clones) owns crates/prism-dtu-cyberint" justification comment; replaced with clarifying note citing ARCH-INDEX. SS-01 justification updated to explicitly name the DTU route/state/extract_access_token work as SS-01 scope. SS-16 anchor and justification preserved (PipelineExecutor + build_request dispatch are prism-spec-engine / SS-16 scope — correct). F-PR12-MED-001 closure (E-AUTH-007 propagation): AC-010 heading updated from `E-AUTH-005/E-AUTH-006` to `E-AUTH-005/E-AUTH-006/E-AUTH-007`; AC-010 body extended with `CredentialResolutionError::BackendUnavailable → E-AUTH-007` paragraph (traces to BC-2.01.017 §Error Cases row 3, EC-017-010, TV-BC-2.01.017-009); Edge Cases table: EC-009 added (BackendUnavailable → E-AUTH-007, no HTTP fetch, no retry); BC table BC-2.01.017 row updated to enumerate E-AUTH-007 alongside E-AUTH-005/006; frontmatter BC-2.01.017 comment updated to cite EC-017-010 and all three error codes. Sibling-sweep (TD-VSDD-060): no remaining "Static Cookie AuthProvider" split-symbol; no stale SensorAuth references. E-AUTH-004/005/006 semantics from prior fix-bursts preserved AS-IS. STORY-INDEX not touched (state-manager updates last). |
| 1.8 | 2026-05-30 | story-writer FB-PR5 | F-PR8-LOW-001 closure (POL-7 BC-H1-verbatim, BC-2.01.017 title-symbol whitespace correction): replaced 3 occurrences of erroneous split-symbol form "Static Cookie AuthProvider" with canonical "StaticCookieAuthProvider" (no space — one code symbol). Locations: (1) frontmatter comment line ~37 (BC-2.01.017 annotation); (2) body BC table row line ~216 (BC title column); (3) changelog row 1.1 historical reference line ~689. Sibling-sweep (TD-VSDD-060): confirmed canonical (no-space) form at line ~604 and all other occurrences unchanged. No other content altered — ACs, BCs, error codes (E-AUTH-004/005/006), and behavioral semantics from prior fix-bursts preserved AS-IS. |
| 1.7 | 2026-05-30 | story-writer FB-PR3 | OBS-PR3-002 closure (TD-VSDD-091 anti-volatile-pin): removed volatile `v1.53` version qualifiers from AC-010 body. (1) Line 337: `error-taxonomy.md v1.53 (NEW in v1.53 — introduced with BC-2.01.017)` → `the E-AUTH-006 entry in error-taxonomy.md (introduced with BC-2.01.017)`. (2) Line 340: `error-taxonomy.md v1.53 §E-AUTH-006` → `the E-AUTH-006 entry in error-taxonomy.md`. Sibling-sweep (TD-VSDD-060): changelog v1.1 row cites `error-taxonomy.md v1.53` as a historical record of what the v1.1 author referenced — exempt from TD-VSDD-091 (historical narrative, not live spec content). No error codes, behavioral claims, or BC traces changed. |
| 1.6 | 2026-05-30 | story-writer FB-PR2 | F-PR2-MED-001 closure: PO adjudicated spec-vs-spec conflict — BC-2.01.017 v1.5 is authoritative; story EC-005/EC-006/AC-010 corrected to align. EC-005: credential-not-found error changed from E-AUTH-004 → E-AUTH-005 per BC-2.01.017 §Error Cases row 1 and TV-BC-2.01.017-004; message template updated to `"Credentials not found for ({client_id}, {sensor_id})"`. EC-006: retry/AuthRefreshFailed semantics replaced with no-retry E-AUTH-004 in sensor_errors per BC-2.01.010 partial failure, call count==1, per BC-2.01.017 §Edge Cases EC-017-002 and TV-BC-2.01.017-006. AC-010 first paragraph: credential-not-found error changed from E-AUTH-004 → E-AUTH-005; heading updated from E-AUTH-004/E-AUTH-006 → E-AUTH-005/E-AUTH-006; AC-010 second paragraph (E-AUTH-006 coverage) kept AS-IS — correct. Task 20 acquire_token error clause updated to E-AUTH-005 with correct NotFound path. Sibling-sweep (TD-VSDD-060): all remaining E-AUTH-004 references verified — only correct usages remain (HTTP-401-during-fetch path in EC-006 and the one pre-existing Task 20 context). |
| 1.5 | 2026-05-30 | story-writer D-874 | F-LP12-MED-001 closure: body-sync H1 + §Version field to match frontmatter (was H1 v1.1 + §Version v1.3 vs frontmatter v1.4 — three distinct version values). POL-29 step 8b body-sync was missed at v1.1 → v1.2 (D-850) and v1.2 → v1.3 (D-863) and v1.3 → v1.4 (D-868); this burst retroactively synchronizes the body. No content change. STORY-INDEX v2.215 → v2.216. |
| 1.4 | 2026-05-30 | story-writer D-868 | F-LP9-MED-001 closure: changelog row reorder to monotonic descending (was 1.0, 1.1, 1.3, 1.2 non-monotonic). Sibling-sweep on F-LP8-MED-001 (BC-2.01.017 v1.4 monotonic descending convention established by PO D-866 at 399ef378). Story changelog now matches BC convention: 1.4 → 1.3 → 1.2 → 1.1 → 1.0. No content change to story body or ACs. STORY-INDEX v2.213→v2.214. |
| 1.3 | 2026-05-30 | story-writer D-863 | F-LP6-LOW-001 Option A (rename): corrected 4 Red Gate test names from wrong BC prefix to primary-BC prefix per project-wide prefix-by-primary-BC convention (S-CONFIG/S-PLUGIN-PREREQ-E evidence). (1) `test_BC_2_01_013_dtu_extract_access_token_parses_cookie_header` → `test_BC_2_01_017_*`: proves extract_access_token behavior which is BC-2.01.017 §Postconditions, not BC-2.01.013. (2) `test_BC_2_01_013_static_cookie_auth_provider_returns_api_key_without_http_call` → `test_BC_2_01_017_*`: proves StaticCookieAuthProvider::acquire_token which is BC-2.01.017 §Postconditions. (3) `test_BC_2_01_016_static_cookie_auth_provider_acquire_token_no_http_call` → `test_BC_2_01_017_*`: proves no-HTTP-call invariant which is BC-2.01.017 §Invariants, not BC-2.01.016. (4) `test_BC_2_01_013_build_request_injects_access_token_cookie_for_cookie_roundtrip` → `test_BC_2_01_017_*`: proves build_request Cookie: access_token dispatch which is BC-2.01.017 behavior. Red Gate table, AC body inline citations, and Tasks step-6/step-19 all updated. Test naming convention note added to §Notes for Implementer. Implementer follow-on: rename 4 test functions in `crates/prism-spec-engine/` and `crates/prism-dtu-cyberint/` test files to match. STORY-INDEX v2.212→v2.213. |
| 1.2 | 2026-05-29 | state-manager D-850 | Administrative status flip `draft` → `ready` per user direction 2026-05-29 to satisfy vsdd-factory deliver-story skill strict prereq. No content or maturity change. Prism precedent is draft-at-dispatch with POL-14 auto-promotion at merge (PLUGIN-MIGRATION-001-A/B/C/D/E + S-CONFIG all entered cascade as draft); this flip is purely administrative to satisfy the skill gate. BCs BC-2.01.017 (new draft per PO b8cf19e1) and BC-2.16.013 will auto-promote draft→active at this story's merge per POL-14. Also: created `.factory/stories/sprint-state.yaml` (first creation on prism; derived view for deliver-story skill compatibility; STORY-INDEX is canonical source of truth). STORY-INDEX v2.211→v2.212. STATE v7.536→v7.537. 250th consecutive single-commit per TD-VSDD-053. |
| 1.1 | 2026-05-29 | story-writer | D-849-prep: BC-2.01.017 (StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection; PO authored commit b8cf19e1) propagated into story per bc_array_changes_propagate_to_body_and_acs policy. Changes: (1) behavioral_contracts: BC-2.01.017 added (4 BCs total); (2) Body BC table: BC-2.01.017 row added; (3) AC-005 citation: BC-2.01.017 §Postconditions added; (4) AC-006 citation: BC-2.01.017 §Invariants added; (5) AC-010 expanded to cover E-AUTH-006 per error-taxonomy.md v1.53 + BC-2.01.017 §Edge Cases; (6) Red Gate tests table: test_BC_2_01_016_static_cookie_auth_provider_acquire_token_no_http_call added (was in AC-006 body but absent from summary table); red_gate_tests 6→7; (7) Token Budget: BC files 3→4, ~4,500→~6,000 tokens; (8) Notes for Implementer: BC-2.01.017 canonical-contract note added; (9) inputs: BC-2.01.017 file added; (10) BC status comment updated to reflect PO authorship complete. |
| 1.0 | 2026-05-29 | story-writer | Initial materialization from [planned] stub per ADR-031 §D3-c and user directive 2026-05-29. Structured as DTU-side ACs + prism-side ACs per user direction. 11 ACs, 6 Red Gate tests, 8 pts, wave 5, P0-pre-demo-BLOCKING. |
