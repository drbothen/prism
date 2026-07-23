---
document_type: behavioral-contract
level: L3
version: "1.9"
status: active
producer: product-owner
timestamp: 2026-05-29T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
introduced: "2026-05-29"
modified: "2026-07-23"
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/stories/S-DTU-CYBERINT-AUTH-FIDELITY-001-cyberint-dtu-static-cookie-auth.md"
input-hash: "356e637"
traces_to:
  - "CAP-001"
  - "ADR-023"
  - "ADR-031"
  - "ADR-028"
  - "BC-2.01.016"
  - "S-DTU-CYBERINT-AUTH-FIDELITY-001"
extracted_from: null
error_codes:
  - "E-AUTH-004"
  - "E-AUTH-005"
  - "E-AUTH-006"
  - "E-AUTH-007"
---

# BC-2.01.017: StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection

## Description

`StaticCookieAuthProvider` implements the `AuthProvider` trait (the TOML-driven replacement for
compile-time SensorAuth dispatch per ADR-023) for sensors that authenticate via a static API key
injected as a named HTTP `Cookie` header on every request, with NO prior login step.
`StaticCookieAuthProvider` is defined in `crates/prism-spec-engine/src/auth_provider.rs` and
selected by `PipelineExecutor` when `spec.auth_type == AuthType::CookieRoundtrip`. Its
`acquire_token()` method reads the API key from the credential resolver and returns it directly
as the token value — it makes zero HTTP calls. `PipelineExecutor::build_request` then injects
`Cookie: {cookie_name}={token}` on every data-fetch request, where the cookie name is derived
from the `header_scheme = "cookie:<name>"` TOML field (per ADR-053 D2). This is the correct auth
implementation for Cyberint (where the real API uses `Cookie: access_token={api_key}` per
poller-express), and it supersedes the incorrect `CookieLoginAuthProvider` (which performed a
`POST /login` round-trip to obtain a `cyberint_session` cookie, violating the ADR-031 §D1-b
DTU=True-DTU fidelity principle).

## Preconditions

- `auth_type = "cookie_roundtrip"` is declared in the sensor's TOML spec. The `auth_plugin`
  field is NOT declared (the static-cookie path applies when the built-in provider handles
  `CookieRoundtrip`; plugin authors may override via `auth_plugin`).
- `header_scheme = "cookie:<name>"` (e.g., `"cookie:access_token"`) is declared in the sensor's
  TOML spec (per ADR-053 D2). The cookie name is the substring after `cookie:`.
- A `credential_ref` is declared in the TOML spec naming a credential reference for the API
  key. The credential resolver can resolve this reference to an API key string value via the
  OS keyring or file backend per AD-017 (reference-only model; credential value never transits
  AI context).
- The cookie name for this sensor is `access_token` (Cyberint canonical per poller-express
  §2.1), specified via `header_scheme = "cookie:access_token"`. Future sensors with different
  cookie names declare a different `header_scheme = "cookie:<name>"` value in their TOML spec.
- `StaticCookieAuthProvider` implements the `AuthProvider` trait (defined in
  `crates/prism-spec-engine/src/auth_provider.rs`, per ADR-023 §PREREQ-B). `PipelineExecutor`
  selects this provider at runtime when `spec.auth_type == AuthType::CookieRoundtrip` — no
  `auth_type_name()` method call is required; dispatch is driven by the TOML `auth_type` field
  directly. The `AuthProvider` trait surface is `acquire_token(&self, spec, client_id)` only.
- The 6-value canonical auth_type set (BC-2.01.016 §Postconditions) includes
  `"cookie_roundtrip"` as a valid value. No spec-load rejection occurs for this sensor.

## Postconditions

### P1 — Token Acquisition (acquire_token)

- `StaticCookieAuthProvider::acquire_token()` calls the `CredentialResolver` with the
  `credential_ref` declared in the sensor TOML spec. The resolver returns the API key string.
- `acquire_token()` returns `Ok(token)` where `token` wraps the raw API key string value.
- **`acquire_token()` makes zero HTTP calls.** No request is dispatched to any endpoint
  (including `/login`, `/auth`, `/session`, `/token`, or any other path) during token
  acquisition. The API key IS the token; there is no authentication exchange.
- The API key value is never written to logs, error messages, or MCP responses per AD-017.

### P2 — Request Header Injection (build_request)

- `PipelineExecutor::build_request` derives the injection mode from the sensor's `header_scheme`
  TOML field (ADR-053 D2). The full dispatch table per `header_scheme` is:

  | `header_scheme` | Header injected | Notes |
  |-----------------|----------------|-------|
  | `"cookie:<name>"` | `Cookie: <name>={token}` | `cookie_roundtrip` only (e.g., Cyberint `cookie:access_token`) |
  | `"bearer"` | `Authorization: Bearer {token}` | `bearer_static`, `oauth2_client_credentials`, `custom_via_plugin` |
  | `"raw"` | `Authorization: {token}` (no "Bearer" prefix) | `token_exchange` (e.g., Armis Centrix — Bearer prefix causes HTTP 401) |

- For `cookie_roundtrip` sensors: the `Authorization` header is NOT set. The `Cookie` header
  is set to `Cookie: <name>={token_value}` where `<name>` is the string after `cookie:` in
  `header_scheme` (e.g., `header_scheme = "cookie:access_token"` → `Cookie: access_token={token}`).
- No `cyberint_session` cookie is ever injected. The header name `cyberint_session` is
  permanently superseded by `access_token` per ADR-031 §D3 and §D4.

### P3 — Auth Type Dispatch

- `PipelineExecutor` selects `StaticCookieAuthProvider` when and only when
  `spec.auth_type == AuthType::CookieRoundtrip`. There is no `auth_type_name()` method on
  `AuthProvider` — the dispatch check compares the TOML-parsed `AuthType` enum variant directly
  (per `PipelineExecutor::execute` at the `CookieRoundtrip` branch). This preserves the
  `AuthType::CookieRoundtrip` enum variant and the 6-value canonical auth_type set per
  BC-2.01.016 §Postconditions INV-AUTH-OPEN-002/003.

### P4 — Zero Login-Shaped Requests

- The set of HTTP requests made during a complete sensor fetch for a `cookie_roundtrip` sensor
  contains ZERO requests to any login-shaped endpoint. Login-shaped endpoints are defined as
  endpoints whose path contains one or more of: `/login`, `/auth`, `/session`, `/token` (as
  path components, not query parameters or data field names).
- The DTU clone (`prism-dtu-cyberint`) MUST also satisfy this property after ADR-031 §D3-a
  correction: its `POST /login` route is removed or repurposed as a no-op, and auth is
  validated via static `access_token` allowlist on every request.

## Invariants

- **INV-COOKIE-001 (No-HTTP-Call Invariant):** `acquire_token()` never makes an HTTP request.
  This is a hard invariant provable by static analysis (the method body does not hold a
  reference to any `reqwest::Client` or HTTP call site). A Kani proof or mock-client assertion
  (zero-call-count) can verify this property at the unit test level.
- **INV-COOKIE-002 (Cookie-Name Immutability):** For Cyberint sensors, the cookie name is
  canonically `access_token`, specified via `header_scheme = "cookie:access_token"`. This name
  is derived from poller-express (`cookieTransport` struct, `Name: "access_token"` per
  `.factory/semport/poller-express/`) and is immutable for the Cyberint sensor spec. Future
  sensors with different cookie names MUST explicitly declare their cookie name via
  `header_scheme = "cookie:<name>"` in their TOML spec.
- **INV-COOKIE-003 (Token = API Key Identity):** The token returned by `acquire_token()` is
  exactly the API key string from the credential resolver, with no transformation. There is no
  encoding, hashing, or session-token wrapping.
- **INV-COOKIE-004 (No Authorization Header):** For `cookie_roundtrip` sensors, the HTTP
  `Authorization` header is never set. Prism's `build_request` dispatch uses
  `header_scheme = "cookie:<name>"` to inject only the `Cookie: <name>={token}` header; the
  `Authorization` header is never set for any sensor with a `cookie:` prefix in `header_scheme`.
- **INV-COOKIE-005 (AD-017 Credential Safety):** The API key value never appears in:
  (a) `tracing::*!` log output (at any log level), (b) error messages returned to the MCP
  caller, (c) structured event catalog fields. Error messages cite sensor name and client_id
  only; the credential value is fully redacted.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-AUTH-005` | Credential resolver finds no credential entry for `(client_id, sensor_id)` — backend is healthy but no API key is configured (`CredentialResolutionError::NotFound`). Distinct from E-AUTH-007 where the backend itself has failed. | `acquire_token()` returns `Err(E-AUTH-005)`. Message: `"Credentials not found for ({client_id}, {sensor_id})"`. Pipeline propagates as a per-sensor partial failure (BC-2.01.010). No HTTP fetch attempted for this sensor. |
| `E-AUTH-006` | Credential resolver returns an empty or invalid value for the API key: empty string (including env var set to `""`), all-whitespace, exceeds 4096 bytes, or contains RFC 6265-illegal characters (e.g., semicolons, control characters) | `acquire_token()` returns `Err(E-AUTH-006)`. Message: `"Empty or invalid API key for cookie_roundtrip sensor '{sensor}' on client '{client_id}'"`. This fires when the resolver SUCCEEDS (`Ok(Some(SecretString))`) but the value fails validation. NOTE: `prism_credentials::resolve_secret` (see `crates/prism-credentials/src/resolve_secret.rs` lines 78-81) performs NO empty-string filtering on the direct-env path — `std::env::var` returns `Ok(value)` for any set env var including `""`, which is then wrapped in `SecretString` and returned as `Ok(Some(...))`. Therefore `CYBERINT_API_KEY=""` produces `Ok(Some(SecretString("")))`, NOT `Ok(None)`, and flows through `acquire_token`'s `is_empty()` check returning E-AUTH-006. |
| `E-AUTH-004` | DTU (or real Cyberint API) returns HTTP 401 for a request carrying `Cookie: access_token={token}` | The HTTP 401 is propagated as a sensor fetch error. The pipeline does NOT retry with a refreshed token (there is no refresh mechanism — the API key is static). The error is surfaced in `sensor_errors` (BC-2.01.010). Root cause: API key is invalid, expired, or revoked. Operator must update the credential in the keyring. |
| Cookie format characters invalid | API key contains characters that are illegal in an HTTP cookie value (e.g., control characters, spaces unescaped, or semicolons) | `acquire_token()` validates the API key against RFC 6265 cookie-value syntax at construction time (newtype validation per AD-017 credential discipline). Returns `E-AUTH-006` with message `"API key for cookie_roundtrip sensor '{sensor}' contains invalid cookie characters"`. |
| `E-AUTH-007` | Credential backend infrastructure failed (`CredentialResolutionError::BackendUnavailable`) — file read error (the _FILE env var path exists but is unreadable due to permissions or I/O failure), keyring daemon unavailable, or other backend-level failure. Distinct from E-AUTH-005: here the backend is DOWN, not merely missing the entry. | `acquire_token()` returns `Err(E-AUTH-007)`. Message: `"Credential resolver backend unavailable for ({client_id}, {sensor_id}): {detail}"`. Retryable — the backend condition (keyring daemon restart, permission fix) may recover. Pipeline propagates as a per-sensor partial failure (BC-2.01.010). No HTTP fetch attempted for this sensor. |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-017-001 | Valid API key present; sensor DTU returns 200 on first request | Happy path. `acquire_token()` returns `Ok(token)`; `build_request` injects `Cookie: access_token={token}` per `header_scheme = "cookie:access_token"`; HTTP 200 received; data pipeline continues normally. |
| EC-017-002 | DTU returns 401 on request carrying `Cookie: access_token={token}` | Pipeline surfaces a `E-AUTH-004` error in `sensor_errors`. No retry, no token refresh. Operator must update credential. |
| EC-017-003 | Operator provides credential but stores it with a typo (wrong key name) | `E-AUTH-005` — resolver finds no entry for the canonical `(client_id, sensor_id)` pair. |
| EC-017-004 | API key string contains a semicolon (`;`) | Newtype validation rejects at token acquisition time with `E-AUTH-006` (invalid cookie characters). Semicolon is the cookie-attribute separator in RFC 6265 and must not appear unescaped in cookie values. |
| EC-017-005 | Empty or whitespace-only API key: env var set to `""`, env var set to all-whitespace (e.g., `"   "`), or credential backend resolves to an empty/whitespace string | `E-AUTH-006`. `prism_credentials::resolve_secret` (lines 78-81) performs NO empty-string normalization on the direct-env path — `std::env::var("FOO")` returns `Ok("")` for `FOO=""`, which is wrapped as `Ok(Some(SecretString("")))`. This reaches `acquire_token`'s `is_empty()` check (or `chars().all(char::is_whitespace)` for whitespace-only) and returns `E-AUTH-006`. The resolver does NOT return `Ok(None)` for this path. Contrast with EC-017-003 (env var not set at all → `std::env::var` returns `Err` → resolver returns `Ok(None)` → `E-AUTH-005`). |
| EC-017-006 | Concurrent fan-out to two Cyberint instances (multi-org via BC-2.06.014) | Each fan-out target independently calls `acquire_token()` with its own `(org_id, sensor_id)` credential lookup. No shared mutable state. Two independent tokens, two independent requests. |
| EC-017-007 | Config hot-reload changes the credential_ref for `cookie_roundtrip` sensor | Next `acquire_token()` call resolves the new reference from the updated config snapshot (ArcSwap per AD-007 per BC-2.16.006). In-flight requests use their own snapshot; no race condition. |
| EC-017-008 | `auth_type = "cookie_roundtrip"` sensor but `credential_ref` absent from TOML spec (count=0 where expected count=1 for `cookie_roundtrip`) | This is a spec-load-time error (E-SPEC-013 per BC-2.01.016 §Error Cases — `cookie_roundtrip` requires exactly one `[[credential_refs]]` entry per DI-012 Rule 2 v1.11; count=0 does not equal expected count=1 for this auth_type). Rejected at boot, not at query time. |
| EC-017-009 | API key length exceeds 4096 bytes | `E-AUTH-006` with message `"API key for cookie_roundtrip sensor exceeds maximum cookie value length"`. Cookie values above 4KB violate RFC 6265 §4.1 and common HTTP server limits. |
| EC-017-010 | Credential backend unavailable: `CredentialResolutionError::BackendUnavailable` returned (e.g., _FILE env var set but file unreadable, keyring daemon stopped) | `E-AUTH-007` — NOT E-AUTH-005. The error code label must reflect the backend-failure semantic, not "credential not found." Distinct from EC-017-003 (NotFound: backend works, no entry → E-AUTH-005). Message includes the `detail` string from `BackendUnavailable.detail`. Implementer must pattern-match on `CredentialResolutionError` variants in `StaticCookieAuthProvider::acquire_token` and emit separate error codes. |

## Canonical Test Vectors

| Test Vector ID | Description | Setup | Expected |
|----------------|-------------|-------|----------|
| TV-BC-2.01.017-001 | Happy path — acquire_token with valid api_key | `CredentialResolver` returns `"test-api-key-abc123"` for the sensor's `credential_ref`; no HTTP mock configured | `acquire_token()` returns `Ok(token)` where token value is `"test-api-key-abc123"`; HTTP call count on any mock client is 0 |
| TV-BC-2.01.017-002 | build_request injects Cookie header, no Authorization | `acquire_token()` called with valid key; `build_request` called | Outgoing request has header `Cookie: access_token=test-api-key-abc123`; `Authorization` header absent |
| TV-BC-2.01.017-003 | build_request uses `access_token` cookie name, not `cyberint_session` | Same setup as TV-002 | Header value contains `access_token=`; must NOT contain `cyberint_session=` |
| TV-BC-2.01.017-004 | Missing credential returns E-AUTH-005 | `CredentialResolver` returns `Err(CredNotFound)` for `(client_id, sensor_id)` | `acquire_token()` returns `Err` containing `E-AUTH-005`; no HTTP request made |
| TV-BC-2.01.017-005 | Empty credential value returns E-AUTH-006 | `MockCredentialResolver` configured to return `Ok(SecretString(""))` for `(client_id, sensor_id)` — simulating `CYBERINT_API_KEY=""` as returned by `prism_credentials::resolve_secret` (lines 78-81: no empty-string filter on direct-env path) | `acquire_token()` returns `Err` containing `E-AUTH-006`; message includes sensor name and client_id; no HTTP request made. Separate test vector for the not-set case: env var absent entirely causes `std::env::var` `Err` → resolver returns `Ok(None)` → `E-AUTH-005` (TV-BC-2.01.017-004 covers the not-found path). |
| TV-BC-2.01.017-006 | DTU 401 response surfaces E-AUTH-004 | Valid token acquired; mock HTTP server returns 401 on data fetch | Fetch error surfaced in `sensor_errors` with `E-AUTH-004`; no retry attempt; call count == 1 |
| TV-BC-2.01.017-007 | Cookie value with semicolon rejected | `CredentialResolver` returns `Ok("key;with;semicolons")` | `acquire_token()` returns `Err(E-AUTH-006)` with invalid-cookie-characters message; no HTTP request made |
| TV-BC-2.01.017-008 | TOML-driven dispatch selects StaticCookieAuthProvider for CookieRoundtrip | Construct a `SensorSpec` with `auth_type = "cookie_roundtrip"` and `header_scheme = "cookie:access_token"`; invoke `PipelineExecutor` auth dispatch path | `PipelineExecutor` reaches the `AuthType::CookieRoundtrip` branch and invokes `StaticCookieAuthProvider::acquire_token()` (not any other provider). `build_request` uses `header_scheme = "cookie:access_token"` to inject `Cookie: access_token={token}`. `StaticCookieAuthProvider` has no `auth_type_name()` method — dispatch is TOML-driven via `spec.auth_type` enum comparison per ADR-023. |
| TV-BC-2.01.017-009 | Backend unavailable returns E-AUTH-007 (not E-AUTH-005) | Inject a `BackendUnavailableCredentialResolver` (returns `CredentialResolutionError::BackendUnavailable{detail: "keyring daemon stopped"}` — or inject as `String` via `CredentialResolver` trait returning `Err("E-AUTH-007: backend unavailable: keyring daemon stopped")`) | `acquire_token()` returns `Err` containing `E-AUTH-007`; error string must NOT contain `E-AUTH-005`; no HTTP request made |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-148 (VP-PLUGIN-003) | DTU parity for sensor spec driven against DTU clone (verified GREEN by PLUGIN-MIGRATION-001-D for the wrapper layer). This BC adds the cookie-shape verification dimension: parity test MUST assert `Cookie: access_token=...` header (not `cyberint_session`) in the outgoing request captured by the DTU mock server. |
| VP-TBD (No-HTTP-Call during acquire_token) | Formal property: `acquire_token()` body contains no calls to `reqwest::Client::*` or any async HTTP-initiating function. Provable by Kani proof on the provider's acquire_token method body (static call graph analysis) or by unit test with a zero-call-count mock HTTP client. Surface to architect for VP catalog assignment. Likely VP-NNN after VP-148 in the verification-properties catalog. |

## Related BCs

- BC-2.01.016 (SensorAuth Open Trait — Plugin-Implementable Auth Contract): parent contract establishing the open `SensorAuth` trait surface (ADR-026) and the 6-value canonical auth_type set (including `"cookie_roundtrip"`). Note: `StaticCookieAuthProvider` is NOT a `SensorAuth` impl — it implements the `AuthProvider` trait (ADR-023 §PREREQ-B, `crates/prism-spec-engine/src/auth_provider.rs`). BC-2.01.016 governs the `auth_type` enumeration and plugin-callable auth surface; this BC governs the as-built TOML-driven `AuthProvider` implementation for `cookie_roundtrip` sensors.
- BC-2.01.005 (CrowdStrike OAuth2 Authentication): sibling auth BC; contrasts by having a live HTTP token-acquisition step (OAuth2 `acquire_token` makes an HTTP request to the token endpoint). Illustrates the architectural contrast: CrowdStrike auth is stateful (acquired token), Cyberint static-cookie auth is stateless (API key IS the token).
- BC-2.01.006 (Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing): predecessor BC covering the old `CookieLoginAuthProvider` behavior (login step + `cyberint_session`). Superseded-in-behavior by this BC per ADR-031 §D3/D4. BC-2.01.006 remains active as it covers timestamp parsing and other Cyberint Assets behaviors; only the auth flow description within it is superseded. Story-writer must update BC-2.01.006's Related BCs section to cross-reference this BC.
- BC-2.16.013 (Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors): covers DTU parity validation for all four initial sensors including Cyberint. The DTU-parity test family for Cyberint MUST now assert the `Cookie: access_token=...` shape per this BC's TV-BC-2.01.017-002/003. This cross-reference enables test-writer to identify and update the Cyberint parity tests.

## Architecture Anchors

- ADR-031 §D1-b — "Auth flow: if the real API requires static cookie injection (no login step) → DTU MUST also accept static cookie injection."
- ADR-031 §D3-b — Prism-side changes required: `StaticCookieAuthProvider` description; `build_request` dispatch table; `Cookie: access_token={token}` injection.
- ADR-031 §D3-b — "does NOT perform any HTTP request during `acquire_token`"
- ADR-053 D2 — `header_scheme` TOML field: governs HTTP header injection mode (`"bearer"` / `"raw"` / `"cookie:<name>"`), decoupled from auth_type acquisition.
- ADR-028 §D-747 LOCKED — `auth_type_label = "cookie_roundtrip"` is preserved (label not changed); behavior changes.
- ADR-023 §PREREQ-B — `AuthProvider` trait definition (TOML-driven replacement for compile-time SensorAuth dispatch). `StaticCookieAuthProvider` implements `AuthProvider` with a single required method: `acquire_token(&self, spec: &SensorSpec, client_id: &OrgSlug)`. There is no `auth_type_name()` on `AuthProvider`; dispatch is TOML-driven via `spec.auth_type` enum comparison.
- `crates/prism-spec-engine/src/auth_provider.rs` — `AuthProvider` trait definition and `StaticCookieAuthProvider` implementation site (constructors: `StaticCookieAuthProvider::new(sensor_id)` for production, `StaticCookieAuthProvider::new_with_resolver(sensor_id, Arc<dyn CredentialResolver>)` for test injection).
- `crates/prism-spec-engine/src/pipeline.rs` — `PipelineExecutor::build_request` header injection site; uses `spec.header_scheme` to determine which header to set (ADR-053 D2).
- `.factory/semport/poller-express/poller-express-broad-sweep.md §2.1` — canonical reference for `access_token` cookie name from real Cyberint API.

## Story Anchor

S-DTU-CYBERINT-AUTH-FIDELITY-001

## VP Anchors

- VP-148 (VP-PLUGIN-003 DTU parity — Cyberint cookie-shape verification dimension added by this BC)
- VP-TBD (No-HTTP-Call during acquire_token — to be assigned by architect)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Capability Anchor Justification | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001. This BC specifies the authentication mechanism by which `StaticCookieAuthProvider` acquires credentials and injects them into sensor API requests — exactly the "auth (OAuth2, cookie, bearer)" behavior that CAP-001 defines for the sensor adapter layer. The no-login-roundtrip static-cookie injection is a direct variant of the cookie auth mechanism described in CAP-001's per-sensor auth handling. |
| L2 Invariants | DI-012 (auth composition prevention — `cookie_roundtrip` is a single auth_type; the static-cookie path cannot be combined with another auth type in a single spec; runtime Rule 2 enforcement per ADR-023 applies) |
| Related BCs | BC-2.01.016 (parent SensorAuth trait contract), BC-2.01.006 (Cyberint Assets auth predecessor), BC-2.16.013 (DTU parity verification family) |
| Priority | P0 |
| ADR | ADR-031 (DTU = True DTU — Fidelity Principle), ADR-028 (TOML Spec Grounding vs DTU Routes), ADR-053 (header_scheme field) |
| Story | S-DTU-CYBERINT-AUTH-FIDELITY-001 |

## Notes for Implementers — Cite-pin convention

Code doc-comments and `assert!` messages in `auth_provider.rs` cite `BC-2.01.017 v<N>` where
`<N>` is the BC version that introduced or re-established the specific EC/postcondition being
anchored (pinned-at-write-time convention). This is intentional:

- `v1.2` pins (EC-017-003, EC-017-005): anchor to the D-854 re-adjudication that restored
  correct semantics after the bad v1.1 amendment. The pin asserts the implementer verified
  behavior against the re-adjudicated spec, not the superseded v1.1 text.
- `v1.3` pins (EC-017-010, E-AUTH-007): anchor to D-857, which first introduced
  EC-017-010 and the `CredentialResolutionError::BackendUnavailable→E-AUTH-007` mapping.
- `v1.8` pins (INV-COOKIE-004, §P2 dispatch, TV-008): anchor to Wave-A ADR-053 D2 amendment
  that replaced auth_type-keyed dispatch with `header_scheme`-keyed dispatch.

The current BC version is tracked in BC-INDEX — code citations need not be updated when a
version bump contains no semantic content change (e.g., the v1.3→v1.4 changelog hygiene bump,
D-866). A hygiene-only bump does NOT invalidate existing cite-pins. See also: F-LP12-LOW-001
adjudication in `cycles/wave-0-plugin-prereqs/S-DTU-CYBERINT-AUTH-FIDELITY-001/po-adjudications/`.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.9 | wave-a-spec-evolution-fix-burst-28 | 2026-07-23 | product-owner | F-WASE-P31-MED-001 closure: EC-017-008 Expected Behavior cell updated for counting-unit accuracy per DI-012 Rule 2 v1.11. Old phrasing "each auth method must declare exactly one `credential_ref`" implied a universal one-ref rule; new phrasing scopes the requirement to `cookie_roundtrip` explicitly: "count=0 does not equal expected count=1 for this auth_type". Description column extended to clarify the EC precondition (count=0 vs expected=1). No change to error code (E-SPEC-013) or rejection behavior (spec-load time, boot failure). Companion: error-taxonomy.md v2.64, BC-2.01.016 v1.14. |
| 1.8 | wave-a-spec-evolution-burst-3 | 2026-07-22 | product-owner | ADR-053 D2 + ADR-054 D1 amendment: §Preconditions updated — cookie name now derived from `header_scheme = "cookie:<name>"` TOML field (not hardcoded); §P2 dispatch table replaced from 4-row auth_type-keyed table to 3-row header_scheme-keyed table (`"cookie:<name>"` / `"bearer"` / `"raw"`); `"raw"` row added for `token_exchange` (Armis); INV-COOKIE-002 updated — cookie name specified via `header_scheme = "cookie:access_token"`, not a separate field; INV-COOKIE-004 re-grounded from auth_type dispatch to `header_scheme = "cookie:<name>"` dispatch; §P3 updated 5-value → 6-value canonical auth_type set per ADR-054 D1; §Related BCs BC-2.01.016 reference updated 5-value → 6-value; §Related BCs BC-2.01.006 title updated to "Cyberint Assets..." per ADR-053 D3 split; TV-BC-2.01.017-008 updated to include `header_scheme = "cookie:access_token"` in SensorSpec construction and assert header_scheme-driven injection; §Architecture Anchors: added ADR-053 D2 anchor; pipeline.rs annotation updated to `header_scheme`; §Traceability ADR field: added ADR-053; §Notes for Implementers: added v1.8 pins guidance. modified date 2026-07-22. |
| 1.7 | D-904 POL-14 auto-promotion | 2026-05-31 | state-manager | POL-14 auto-promotion at merge: PR #164 (S-DTU-CYBERINT-AUTH-FIDELITY-001) squash-merged to develop@e798e67c; status draft→active; lifecycle_status was already active (idempotent). BC-INDEX v5.64→v5.65 (active_contracts 236→237, draft_contracts 3→2). |
| 1.6 | FB-PR6 F-P10-MED-001 | 2026-05-30 | product-owner | F-P10-MED-001 closure: SensorAuth→AuthProvider prose correction throughout (mis-anchor finding). §Description: "implements `SensorAuth` trait" corrected to "implements `AuthProvider` trait (ADR-023)"; implementation site corrected from `crates/prism-sensors/src/auth/mod.rs` to `crates/prism-spec-engine/src/auth_provider.rs`. §Preconditions: stale `SensorAuth` + `auth_type_name()` bullet replaced with correct TOML-driven dispatch description (`spec.auth_type == AuthType::CookieRoundtrip`, `acquire_token()` as the sole AuthProvider method). §Postconditions P3: renamed from "Auth Type Name" to "Auth Type Dispatch"; corrected from `auth_type_name()` method (which does NOT exist on AuthProvider) to TOML-driven enum dispatch. TV-BC-2.01.017-008: rewritten from "auth_type_name returns canonical string" to TOML-driven dispatch test. §Related BCs: BC-2.01.016 description clarified — `StaticCookieAuthProvider` implements `AuthProvider`, NOT `SensorAuth`. §Architecture Anchors: ADR-026 §D1 `SensorAuth` anchor replaced by ADR-023 §PREREQ-B `AuthProvider` anchor; crate path corrected to `crates/prism-spec-engine/src/auth_provider.rs`. Contract semantics unchanged (error codes E-AUTH-004/005/006/007, no-retry, zero-HTTP, cookie injection all unmodified). BC-INDEX title column unchanged (not a title change). |
| 1.5 | D-875 F-LP12-LOW-001 | 2026-05-30 | product-owner | F-LP12-LOW-001 adjudication: all 21 cite-pins confirmed Category A (behavioral anchors); no code change required. Added §Notes for Implementers — Cite-pin convention section to document pinned-at-write-time convention. POL-29 step 8f amendment recommended (hygiene-only version bumps exempt from cite-pin sweep obligation). BC-INDEX v5.60→v5.61. |
| 1.4 | D-866 F-LP8-MED-001 | 2026-05-30 | product-owner | F-LP8-MED-001 closure: changelog hygiene — deleted byte-identical duplicate of v1.2 row (was at line 237 alongside canonical v1.2 row at line 235); reordered changelog rows to monotonic descending by version (1.4 → 1.3 → 1.2 → 1.1 → 1.0). No semantic content change to BC. BC-INDEX v5.59→v5.60. |
| 1.3 | D-857 F-LP3-HIGH-001 | 2026-05-30 | product-owner | F-LP3-HIGH-001 resolution: allocate E-AUTH-007 for `CredentialResolutionError::BackendUnavailable`. Add EC-017-010 (BackendUnavailable → E-AUTH-007, distinct from EC-017-003 NotFound → E-AUTH-005). Add TV-BC-2.01.017-009 (BackendUnavailable test vector). Update E-AUTH-005 Error Cases row to explicitly scope it to `CredentialResolutionError::NotFound`. Add E-AUTH-007 Error Cases row. `error_codes` frontmatter: add E-AUTH-007. error-taxonomy.md v1.53→v1.54. BC-INDEX v5.58→v5.59. Implementer follow-on: match-arm on CredentialResolutionError variants in `StaticCookieAuthProvider::acquire_token` — NotFound→E-AUTH-005, BackendUnavailable→E-AUTH-007. Add `BackendUnavailableCredentialResolver` test helper and unit test asserting E-AUTH-007. |
| 1.2 | D-854 | 2026-05-30 | product-owner | F-LP1-MED-002 RE-ADJUDICATION — revert v1.1 EC-017-005 amendment per F-LP2-CRIT-001. v1.1 amendment was based on a fabricated BC-2.03.006 normalization claim that does not exist in BC-2.03.006 text or `crates/prism-credentials/src/resolve_secret.rs`. Orchestrator independently verified against source (lines 78-81): `std::env::var(direct_env)` returns `Ok(value)` for ANY set env var including empty string; there is NO `is_empty()` filter, no whitespace check, no normalization. `CYBERINT_API_KEY=""` resolves as `Ok(Some(SecretString("")))`, propagates through `acquire_token`'s `is_empty()` check, and returns E-AUTH-006 — exactly what BC v1.0 EC-017-005 originally specified. BC v1.0 original author was correct. Restored EC-017-005 to E-AUTH-006 semantics with precise resolver behavior cited verbatim from source (lines 78-81). Updated E-AUTH-006 Error Cases row to accurately describe "resolver returns empty value" as the trigger (not just non-empty-invalid). Restored TV-BC-2.01.017-005 to assert E-AUTH-006 for MockCredentialResolver returning `Ok(SecretString(""))`. BC-INDEX v5.57→v5.58. |
| 1.1 | D-852 | 2026-05-30 | product-owner | F-LP1-MED-002 adjudication (Option A — impl wins) — SUPERSEDED BY v1.2 (fabricated evidence). Corrected EC-017-005 and TV-BC-2.01.017-005: empty env-var path returns E-AUTH-005 (resolver-not-found), NOT E-AUTH-006 (value-validation). THIS AMENDMENT WAS INCORRECT: it relied on a claim that BC-2.03.006 normalizes empty strings as not-found, but BC-2.03.006 makes no such claim and resolve_secret.rs lines 78-81 confirm no such normalization exists. Retained for audit trail per append_only_numbering policy. |
| 1.0 | D-849 | 2026-05-29 | product-owner | Initial draft. Authored to close BC gap surfaced by story-writer during S-DTU-CYBERINT-AUTH-FIDELITY-001 materialization. Specifies `StaticCookieAuthProvider` no-login-roundtrip contract per ADR-031 §D1-b/D3-b. Error codes: E-AUTH-004 (DTU 401), E-AUTH-005 (missing credential), E-AUTH-006 (new — empty/invalid API key value). VP-TBD (No-HTTP-Call) surfaced for architect VP catalog assignment. |
