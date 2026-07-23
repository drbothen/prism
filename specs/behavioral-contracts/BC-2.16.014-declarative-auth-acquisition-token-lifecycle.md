---
document_type: behavioral-contract
level: L3
version: "1.10"
status: draft
producer: product-owner
timestamp: 2026-07-22T00:00:00Z
phase: 1a
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: draft
introduced: "2026-07-22"
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md"
  - ".factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/domain-spec/invariants.md"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/src/error.rs"
input-hash: "88a7cdb"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.014: Declarative Auth Acquisition Token Lifecycle

## Description

`DeclarativeHttpAuthProvider` is a native Rust auth provider that performs standard HTTP-POST-for-token
flows declaratively from TOML sensor spec fields, without a WASM plugin. It handles two
`auth_type` variants: `token_exchange` (parameterized form-field → dotted-path response extraction)
and `oauth2_client_credentials` (RFC 6749 §4.4 client credentials, replicating the behavior of the
retired `crowdstrike-oauth2.prx` plugin). The provider implements the existing `AuthProvider` trait
defined in `crates/prism-spec-engine/src/auth_provider.rs` and is constructed per (org, sensor) at
boot step 9A (`step9a_populate_adapter_registry` in `crates/prism-bin/src/spec_driven_adapter.rs`).

This contract governs the full token lifecycle: lazy construction (zero network I/O at construction),
cache-aware `get_token()`, force-refresh `acquire_token()`, 401-retry integration with
`PipelineExecutor`, credential safety (AD-017), and TTL arithmetic for both `absolute_utc_string`
and `relative_seconds` expiry modes. All behavior is parameterized through the sensor spec's
`[auth_acquisition]` TOML block; no sensor-name-conditional engine logic is permitted (POL-36).
Sensor names appear only in TOML spec files and test fixtures.

This BC is the source authority for VP-159 (authored v1.0, D-1947; registered in VP-INDEX v1.83) and the behavioral
specification for the `DeclarativeHttpAuthProvider` implementation story in Wave-A.

## Preconditions

1. The sensor spec has `auth_type ∈ {oauth2_client_credentials, token_exchange}` — one of the
   two declarative auth types governed by this contract.

2. An `[auth_acquisition]` block is present in the sensor spec and has passed `E-SPEC-028`
   validation (BC-2.16.009 Rule 10) at spec-load time. Boot has succeeded past Rule 10.

3. For `auth_type = "token_exchange"`: `[auth_acquisition]` declares all five required fields:
   `token_path`, `credential_body_field`, `token_response_path`, `expiry_field`, and
   `expiry_mode` (value is one of `"absolute_utc_string"` or `"relative_seconds"`).

4. For `auth_type = "oauth2_client_credentials"`: `[auth_acquisition]` declares `token_path`.
   The `[[credential_refs]]` array declares entries named `client_id` and `client_secret`
   (validated by `E-SPEC-028(f)` at spec-load time).

5. `auth_plugin` is NOT present in the spec for `auth_type ∈ {oauth2_client_credentials,
   token_exchange}`. `E-SPEC-028(b)` unconditionally rejects any such combination at spec-load
   time — this arm of `step9a_populate_adapter_registry` is validation-unreachable from any
   valid spec (retained as defense-in-depth only, per ADR-054 §D7).

6. `base_url` in the parent `SensorSpec` resolves successfully via BC-2.16.009 Rule 6
   env-var interpolation at spec-load time. The per-org resolved `base_url` is available
   at `step9a_populate_adapter_registry` construction time.

7. `DeclarativeHttpAuthProvider` is constructed at boot step 9A with:
   `token_url = format!("{}{}", resolved_per_org_base_url, auth_acquisition.token_path)`.
   The token URL is computed once at construction time from the per-org overlay-resolved
   `base_url`; it is stored in `self.token_url` and is never recomputed.

8. The ADR-053 standalone Wave-A engine story has landed on `develop` before the ADR-054
   implementation story merges. The standalone engine story adds `SensorSpec::header_scheme`,
   switches `build_request()` to `header_scheme`-based dispatch, and registers E-SPEC-027
   as BC-2.16.009 Rule 9. ADR-054 implementation stories add Rule 10 / E-SPEC-028 and
   `AuthType::TokenExchange` — these land second (per ADR-054 §D7 merge-dependency).

9. A `CredentialResolver` (`Arc<dyn CredentialResolver>`) is injected at construction via
   `step9a_populate_adapter_registry`. Production uses `PrismCredentialResolver` (wraps
   `prism_credentials::resolve_credential` per ADR-034 §D1).

## Postconditions

### P1 — Zero Network I/O at Construction (Lazy Acquisition Invariant)

`DeclarativeHttpAuthProvider::new()` makes ZERO network calls. No credentials are resolved
and no HTTP POST is issued at construction time. The in-memory token cache
(`ArcSwap<Option<CachedAuthToken>>`) is initialized to `None`. The token URL is computed as
`format!("{}{}", resolved_per_org_base_url, auth_acquisition.token_path)` and stored in
`self.token_url` — pure string concatenation, no I/O. Credential values are never stored
as struct fields (AD-017: credentials are never resolved at construction time).

### P2 — First `get_token()` Issues Exactly One HTTP POST and Caches Result

On the first call to `get_token()` when the cache is cold (`ArcSwap::load()` yields `None`
or an entry with an empty token string), the provider calls `acquire_token()` internally.
`acquire_token()` issues exactly one HTTP POST to `self.token_url` with
`Content-Type: application/x-www-form-urlencoded`.

**Form body construction per `auth_type`:**

- `oauth2_client_credentials`: `client_id={}&client_secret={}&grant_type=client_credentials`
  (three fields in this order; values RFC-3986 §2.3 percent-encoded; field order matches
  `crowdstrike-oauth2/src/lib.rs::acquire_token()` for behavioral parity with the retired plugin)
- `token_exchange`: `{credential_body_field}={resolved_credential_value}` (single form field;
  `credential_body_field` from `[auth_acquisition]`; value RFC-3986 §2.3 percent-encoded)

**Response parsing per `auth_type`:**

- `oauth2_client_credentials`: extract `$.access_token` (string, required); compute
  `expires_at = unix_now() + ($.expires_in as u64).saturating_sub(ttl_buffer_secs)` where
  `$.expires_in` defaults to `1799` when absent or zero, and `ttl_buffer_secs` defaults to `30`.
- `token_exchange` with `expiry_mode = "relative_seconds"`: extract token at
  `token_response_path` (dotted path, e.g., `"data.access_token"`); compute
  `expires_at = unix_now() + expiry_value.saturating_sub(ttl_buffer_secs)` where
  `expiry_value` is the u64 seconds at `expiry_field`, defaulting to `1799` when absent or zero.
- `token_exchange` with `expiry_mode = "absolute_utc_string"`: extract token at
  `token_response_path`; parse `expiry_field` value as RFC-3339 timestamp → Unix seconds;
  compute `expires_at = parsed_unix_secs.saturating_sub(ttl_buffer_secs)`.

On success, a `CachedAuthToken { token: String, expires_at: u64 }` is atomically stored via
`ArcSwap`. The token string is returned. No token value is logged at any level (AD-017,
INV-014-004).

> **PG-LP11-001 note:** If any `tracing::*!(event_type = "...")` emissions are added at
> acquisition time during implementation, they MUST be registered in BC-2.16.002
> §Postconditions (Canonical Structured Event Catalog) with full field schema, audit role,
> and recurrence policy before the PR merges.

### P3 — Cache Hit Returns Token Without Any HTTP Call

On subsequent `get_token()` calls where `unix_now() < cached.expires_at` AND
`!cached.token.is_empty()`: the provider returns the cached token string without issuing
any HTTP request. The `ArcSwap::load()` snapshot is a shared-reference read — no contention
with concurrent callers on this path.

### P4 — Stale Cache Triggers Exactly One Re-Acquisition POST

On `get_token()` when `unix_now() >= cached.expires_at` OR `cached.token.is_empty()` (stale
or poisoned cache entry): the provider calls `acquire_token()`, issues exactly one HTTP POST
to `self.token_url`, atomically updates the cache via `ArcSwap`, and returns the new token.
The re-acquisition execution path is identical to P2.

### P5 — `acquire_token()` Always Issues Exactly One HTTP POST (Cache Bypass)

`acquire_token()` is the force-refresh / cache-bypass path. It unconditionally issues exactly
one HTTP POST to `self.token_url` regardless of cache state, and always overwrites the cache
with the fresh result via `ArcSwap`. It is called:
- Internally by `get_token()` on cold or stale cache (P2, P4 paths).
- Directly by `PipelineExecutor` via `issue_request_with_retry` on HTTP 401 from a sensor
  endpoint (force-refresh, cache-bypass — the 401-refresh path MUST call `acquire_token()`
  directly, NOT `get_token()`, because `get_token()` would return the same stale or revoked
  token from the warm cache; see P6).
- By test code to drive acquisition without going through `get_token()`.

### P6 — 401 Retry: Single Force-Refresh; Double-401 Fails Non-Retryably

On HTTP 401 from a sensor API endpoint during pipeline execution, dispatched via
`issue_request_with_retry`:

1. `PipelineExecutor` calls `DeclarativeHttpAuthProvider::acquire_token()` directly (force-
   refresh, cache-bypass — NOT `get_token()`, which would return the same stale or revoked
   token from the warm cache). Exactly one HTTP POST to the token URL.
2. The sensor request is retried with the newly acquired token.
3. If the retried sensor request also returns HTTP 401 (second consecutive 401, with an
   intervening `acquire_token()` call between the two 401s): the pipeline fails with
   `SpecEngineError::AuthRefreshFailed` (E-AUTH-002). No further retry is issued. The failure
   is non-retryable at the pipeline level.

`PipelineExecutor` explicitly calls `acquire_token()` (not `get_token()`) on HTTP 401, so
two consecutive 401s from the sensor endpoint — with a single `acquire_token()` call between
them — constitute the E-AUTH-002 double-401 condition. A first 401 on the original request
triggers `acquire_token()` via `issue_request_with_retry`; a second 401 on the retried
request triggers E-AUTH-002.

This protocol prevents infinite retry loops on permanently invalid or expired credentials.
It preserves the existing double-401 abort semantics established for `PluginAuthProvider`-based
sensor pipelines.

### P7 — Credential Values Are Never Stored in `CachedAuthToken` (AD-017)

`CachedAuthToken` holds exactly two fields:
- `token: String` — the acquired access token (opaque string; never logged per AD-017;
  never appears in structured tracing events, error messages, or any output field)
- `expires_at: u64` — Unix timestamp (seconds since epoch) after which the token is stale

Credential values (`secret_key`, `client_id`, `client_secret`, or the value of any
`credential_body_field` ref) are resolved from the `CredentialResolver` at `acquire_token()`
call time only. They are used immediately to construct the HTTP POST body and are not retained
beyond the scope of that operation. They are not stored as struct fields, are not stored in
`CachedAuthToken`, and are never persisted to any store. The `Zeroizing<String>` wrapper
applies where the credential store returns `secrecy::SecretString` (which implements
`Zeroize` on drop).

### P8 — `base_url` Interpolated; `token_path` Literal (BC-2.16.009 Rule 6 / E-SPEC-024)

`base_url` in the parent `SensorSpec` undergoes env-var interpolation per BC-2.16.009 Rule 6.
An unresolved `${env.VARIABLE}` reference in `base_url` emits `E-SPEC-024` at spec-load time
and rejects the spec. By the time `step9a_populate_adapter_registry` constructs a
`DeclarativeHttpAuthProvider`, `base_url` is fully resolved with any per-org overlay applied.

`token_path` in `[auth_acquisition]` is a LITERAL relative path string and does NOT undergo
env-var interpolation. A `${env.FOO}` placeholder in `token_path` is treated as literal URL
path characters; it is concatenated verbatim into the token URL, which will likely result in
an HTTP error at acquisition time. This is intentional: silent env-var lookup beyond the
documented Rule 6 surface is a SOUL.md §4 violation.

The full token URL is constructed once at provider construction time:
`format!("{}{}", resolved_per_org_base_url, auth_acquisition.token_path)`. The per-org
`base_url` overlay — e.g., DTU clone endpoints in tests, regional tenant URLs in production —
flows through automatically because `step9a_populate_adapter_registry` uses the overlay-resolved
`base_url` (the same per-org derivation used by the existing `Oauth2ClientCredentials` arm
before this migration).

### P9 — `get_token()` Production Callers: Eager Acquisition Before Step Execution [PLANNED — engine story]

`get_token()` is the cache-aware method added to the `AuthProvider` trait with a default
implementation that delegates to `self.acquire_token()` (ADR-054 §D4). All existing
`AuthProvider` implementors (`NullAuthProvider`, `MockAuthProvider`, `StaticCookieAuthProvider`,
`FailingAuthProvider`, `ChainAuthProvider`, `BearerStaticCredentialAuthProvider`,
`PluginAuthProvider`) inherit the default — each effectively becomes a force-refresh path
on every `get_token()` call, which is correct for non-caching providers. `DeclarativeHttpAuthProvider`
overrides `get_token()` with the ArcSwap cache-aware logic described in P2–P4.

**[PLANNED — engine story per ADR-054 D11]** The following `PipelineExecutor` call sites in
`crates/prism-spec-engine/src/pipeline.rs` are scheduled to call `get_token()` (cache-aware)
rather than `acquire_token()` (force-refresh) once the engine story lands:

- `execute_impl` — eager acquisition before the `'steps:` loop (before any HTTP request is
  issued to the sensor). Calling `get_token()` here means warm pipeline executions (within
  the token's TTL window) return the cached token with zero token-POST overhead, restoring
  the caching behavior of the retired `crowdstrike-oauth2.prx` plugin.
- `execute_step` — eager acquisition before single-request dispatch. Same rationale as
  `execute_impl`; symmetric eager-acquisition semantics across both executor entry points.

Until the engine story lands, the as-built `pipeline.rs` continues to call `acquire_token()`
at these two sites (consistent with all existing `AuthProvider` trait callers). After the
engine story merges: `execute_impl` and `execute_step` call `get_token()` (cache-aware),
while `issue_request_with_retry`'s 401 arm continues to call `acquire_token()` directly
(force-refresh, per P5/P6 — calling `get_token()` on the 401 path is incorrect because it
would return the same stale or revoked token from the warm cache).

## Invariants

- **INV-014-001 (Generalization Constraint — POL-36):** The `DeclarativeHttpAuthProvider`
  engine implementation MUST NOT contain sensor-name-conditional logic (e.g.,
  `if sensor_id == "armis" { ... } else if sensor_id == "crowdstrike" { ... }`). All runtime
  behavior is parameterized exclusively by `auth_type` value, `[auth_acquisition]` fields,
  and `[[credential_refs]]` declarations in the sensor spec. Sensor names MAY appear only
  in TOML spec files, integration test fixtures, and TOML config examples.

- **INV-014-002 (Token URL Derivation — Per-Org Correctness):** `token_url` is ALWAYS derived
  as `format!("{}{}", resolved_per_org_base_url, auth_acquisition.token_path)` at step 9A
  construction time. `token_url` is NOT a field in `[auth_acquisition]`; a global token URL
  is never hardcoded. Per-org `base_url` overlays (multi-tenant, multi-region, DTU clone
  endpoints) propagate to the token POST target automatically. This preserves the per-org
  derivation pattern of the existing `Oauth2ClientCredentials` arm.

- **INV-014-003 (Credential Lazy Resolution — AD-017):** Credential values are resolved at
  `acquire_token()` call time only. They are never stored as fields on `DeclarativeHttpAuthProvider`
  or in `CachedAuthToken`. `CredentialResolver::resolve()` is the only site where credential
  values are accessed during the token lifecycle.

- **INV-014-004 (Token Never Logged — AD-017):** `CachedAuthToken::token` is never emitted in
  log output, structured tracing events, error messages, or any output field at any log level.
  This is the same contract as `AuthToken(Zeroizing<String>)` in `auth_provider.rs`.

- **INV-014-005 (ArcSwap Atomicity):** The `ArcSwap<Option<CachedAuthToken>>` is updated
  atomically via `ArcSwap::store()`. Concurrent `get_token()` calls from the tokio multi-thread
  runtime (fan-out) see a consistent cache state — either the old or the new `CachedAuthToken`,
  never a partial write. On concurrent cold-cache calls, multiple HTTP POSTs may be issued
  (one per concurrent caller that races to the cold-cache path before any store completes).
  This is non-fatal: all yield valid tokens; last `ArcSwap::store()` wins. Subsequent callers
  hit the warm cache.

- **INV-014-006 (DI-012 Compliance — `token_exchange` as 6th Variant):** Each sensor spec
  declares exactly one `auth_type`. ADR-054 amends DI-012 (amends_dis: DI-012) by adding
  `token_exchange` as the 6th valid variant to the `AuthType` closed enum. The three DI-012
  runtime composition guards (single auth_type per spec, single credential_ref per method,
  structural credential ⇆ auth_type match) continue to apply; E-SPEC-028 is the Rule 10
  enforcement gate for declarative auth coherence.

- **INV-014-007 (reqwest ADR-050 Compliance):** The HTTP POST to the token endpoint uses a
  `reqwest::Client` configured per ADR-050: `default-features = false, features = ["rustls-tls"]`;
  `native-tls` and its aliases (`default-tls`, `native-tls-alpn`, `native-tls-vendored`) are
  forbidden. The client is constructed with `.timeout(Duration::from_secs(30))` (30s timeout)
  per the workspace convention established by `build_http_client_with_timeout()`.

- **INV-014-008 (E-SPEC-028 Spec-Load Gate):** `DeclarativeHttpAuthProvider` is never
  constructed from a spec that would fail `E-SPEC-028` validation. By the time
  `step9a_populate_adapter_registry` runs at boot step 9A, all `[auth_acquisition]` coherence
  errors (E-SPEC-028(a)–(h)) have been caught at spec-load time (BC-2.16.009 Rule 10) and
  the spec has been rejected with exit code 2.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-014-001 | `$.expires_in` absent or zero in `oauth2_client_credentials` response | TTL defaults to `1799 saturating_sub(ttl_buffer_secs)`. Token is cached normally. |
| EC-016-014-002 | `expiry_mode = "relative_seconds"` with the expiry value absent or zero in response | TTL defaults to `1799 saturating_sub(ttl_buffer_secs)`. Same behavior as EC-016-014-001. |
| EC-016-014-003 | `expiry_mode = "absolute_utc_string"` with a malformed (non-RFC-3339-parseable) expiry value | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` (E-AUTH-001). No token cached. |
| EC-016-014-004 | `expiry_mode = "absolute_utc_string"` and the parsed expiry timestamp is already in the past at cache-store time | Token stored with `expires_at` in the past. Next `get_token()` call immediately falls through to re-acquisition (P4 path). No error on initial acquisition. |
| EC-016-014-005 | Token POST returns HTTP 4xx or 5xx | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` (E-AUTH-001). No token cached. |
| EC-016-014-006 | Token POST response body is not valid JSON or not a JSON object | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` (E-AUTH-001). |
| EC-016-014-007 | `token_exchange`: `token_response_path` field absent or null in the response JSON | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` (E-AUTH-001). |
| EC-016-014-008 | `oauth2_client_credentials`: `$.access_token` absent or null in the response JSON | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` (E-AUTH-001). |
| EC-016-014-009 | `CredentialResolver::resolve()` returns `NotFound` for any required credential ref | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` with `detail = "E-AUTH-005: credential not found — no credential configured for ({client_id}, {sensor_id}) ref '{ref_name}'"`, where `{ref_name}` is the specific credential ref being resolved (e.g., `secret_key` for `token_exchange`; `client_id` or `client_secret` for `oauth2_client_credentials`). E-AUTH-005 is the standalone wire code per error-taxonomy.md §E-AUTH-005 and the `CredentialResolver` trait: "Callers should map this to `E-AUTH-005`." Zero HTTP POSTs are issued. |
| EC-016-014-010 | HTTP 401 from sensor endpoint (single occurrence) | `PipelineExecutor` calls `acquire_token()` (force-refresh). Request retried with new token. If retry succeeds, pipeline continues normally. |
| EC-016-014-011 | HTTP 401 from sensor endpoint after force-refresh (second consecutive 401) | Pipeline fails with `SpecEngineError::AuthRefreshFailed` (E-AUTH-002). No further retry. Non-retryable. |
| EC-016-014-012 | `ttl_buffer_secs` >= effective TTL value (e.g., buffer 60, `expires_in` 30) | `saturating_sub` produces `expires_at = unix_now()` or less. Token is stored but immediately stale. Every `get_token()` call triggers P4 re-acquisition. Non-fatal but causes one HTTP POST per request. |
| EC-016-014-013 | Concurrent `get_token()` calls on cold cache (multiple tokio tasks racing) | At most N HTTP POSTs for N concurrent callers that all observe a cold cache before any ArcSwap store completes. All calls return a valid token. Last `ArcSwap::store()` wins. Subsequent callers hit warm cache. No corruption. |
| EC-016-014-014 | `token_path` contains env-var-style placeholders (e.g., `"/api/${env.VER}/token"`) | Treated as literal URL path characters (no env-var interpolation for `token_path`). Token URL constructed verbatim. Likely results in HTTP 404 or connection error; propagates as `AuthAcquisitionFailed` (E-AUTH-001). |
| EC-016-014-015 | `[auth_acquisition]` block present for non-declarative `auth_type` (e.g., `bearer_static`, `cookie_roundtrip`, `api_key`, `custom_via_plugin`) | Rejected at spec-load time with `E-SPEC-028(g)`. Boot fails exit code 2. `DeclarativeHttpAuthProvider` never constructed. |

## Error Conditions

| Error Code | Condition | Behavior |
|------------|-----------|----------|
| `E-SPEC-028(a)` | `auth_type ∈ {oauth2_client_credentials, token_exchange}` but `[auth_acquisition]` block absent or `token_path` absent | Spec rejected at load time (BC-2.16.009 Rule 10). Boot fails exit code 2. |
| `E-SPEC-028(b)` | `auth_type ∈ {oauth2_client_credentials, token_exchange}` with `auth_plugin` present | Spec rejected at load time unconditionally. Boot fails exit code 2. |
| `E-SPEC-028(c)` | `token_exchange` with unrecognized `expiry_mode` value | Spec rejected at load time. Boot fails exit code 2. |
| `E-SPEC-028(d)` | `token_exchange` missing any of: `credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode` | Spec rejected at load time (one error emitted per missing field). Boot fails exit code 2. |
| `E-SPEC-028(e)` | `[auth_acquisition].credential_body_field` names a ref not declared in `[[credential_refs]]` | Spec rejected at load time. Boot fails exit code 2. |
| `E-SPEC-028(f)` | `auth_type = "oauth2_client_credentials"` with missing `client_id` or `client_secret` credential refs | Spec rejected at load time. Boot fails exit code 2. |
| `E-SPEC-028(g)` | `[auth_acquisition]` block present for `auth_type ∈ {bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` | Spec rejected (prevents silent ignore; SOUL.md §4). Boot fails exit code 2. |
| `E-SPEC-028(h)` | `token_exchange`-only fields present in an `oauth2_client_credentials` `[auth_acquisition]` block | Spec rejected (prevents silent ignore of misconfigured fields; SOUL.md §4). Boot fails exit code 2. |
| `E-AUTH-001` (`AuthAcquisitionFailed`) | Any acquisition-level failure at `acquire_token()` time: token POST non-200, malformed JSON response, missing token field at declared path, malformed expiry string, credential resolver error | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed`. No token cached. Pipeline aborts for this `(org, sensor)` request. |
| `E-AUTH-002` (`AuthRefreshFailed`) | Double-401: sensor endpoint returns 401 both before and after force-refresh (`acquire_token()`) | `PipelineExecutor` propagates `SpecEngineError::AuthRefreshFailed`. Non-retryable. |
| `E-AUTH-005` (standalone — `CredentialResolutionError::NotFound`) | `CredentialResolver::resolve()` returns `CredentialResolutionError::NotFound` for any required credential ref during acquisition | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` with `detail = "E-AUTH-005: credential not found — no credential configured for ({client_id}, {sensor_id}) ref '{ref_name}'"` (where `{ref_name}` is the specific credential ref being resolved — e.g., `secret_key` for `token_exchange`; `client_id` or `client_secret` for `oauth2_client_credentials`). E-AUTH-005 is the canonical standalone wire code (error-taxonomy.md §E-AUTH-005: "Credentials not found for ({client_id}, {sensor_id})"; `CredentialResolver` trait: "Callers should map this to `E-AUTH-005`"). Zero HTTP POSTs issued. Detail template generalizes the `StaticCookieAuthProvider` E-AUTH-005-in-detail pattern (BC-2.01.017 EC-017-003) with ref-name-agnostic placeholders. |
| `E-SPEC-024` | `base_url` in the sensor spec contains an unresolved env-var interpolation placeholder | Spec rejected at load time per BC-2.16.009 Rule 6. Boot fails exit code 2. `DeclarativeHttpAuthProvider` never constructed. |

## Canonical Test Vectors

| Scenario | auth_type | Input | Expected Outcome |
|----------|-----------|-------|-----------------|
| **TV-1** oauth2_client_credentials happy path | `oauth2_client_credentials` | Mock token server: `POST /oauth2/token` with `client_id=test-id&client_secret=test-secret&grant_type=client_credentials` → `{"access_token":"tok-abc","expires_in":3600}` | `get_token()` returns `"tok-abc"`; exactly one HTTP POST issued; `expires_at ≈ unix_now() + 3600 - 30`; second `get_token()` call (before TTL) returns `"tok-abc"` with zero additional HTTP POSTs |
| **TV-2** token_exchange absolute_utc_string expiry | `token_exchange` with `expiry_mode = "absolute_utc_string"` | Mock server: `POST /api/v1/access_token/` with `secret_key=sk-val` → `{"success":true,"data":{"access_token":"arm-tok","expiration_utc":"2099-01-01T00:00:00Z"}}`; spec has `token_response_path = "data.access_token"`, `expiry_field = "data.expiration_utc"` | `get_token()` returns `"arm-tok"`; one HTTP POST issued; `expires_at = parse_rfc3339("2099-01-01T00:00:00Z") - 30`; subsequent `get_token()` (within TTL): zero HTTP POSTs |
| **TV-3** token_exchange relative_seconds expiry | `token_exchange` with `expiry_mode = "relative_seconds"` | Mock server returns `{"token":"rel-tok","expires_in":7200}`; spec has `token_response_path = "token"`, `expiry_field = "expires_in"` | `get_token()` returns `"rel-tok"`; `expires_at ≈ unix_now() + 7200 - 30` |
| **TV-4** Cache hit — zero additional HTTP POSTs | Either | After TV-1 or TV-2 (warm cache, TTL not elapsed): call `get_token()` a second time | Returns cached token immediately; HTTP POST count from mock server unchanged (no new request) |
| **TV-5** expires_in absent — default 1799 | `oauth2_client_credentials` | Mock returns `{"access_token":"tok-noexp"}` (no `expires_in` field) | `get_token()` returns `"tok-noexp"`; `expires_at ≈ unix_now() + 1799 - 30 = unix_now() + 1769`; no error |
| **TV-6** Stale cache triggers re-acquisition | `oauth2_client_credentials` | First acquisition: `{"access_token":"tok-v1","expires_in":0}` (zero → default 1799 → `expires_at = unix_now() + 1769`); advance simulated clock past `expires_at`; second `get_token()` call | Second call issues one new HTTP POST; returns fresh token; total HTTP POST count = 2 |
| **TV-7** Direct `acquire_token()` — cache bypass | Either | Warm cache with valid token; call `acquire_token()` directly | Issues exactly one HTTP POST regardless of warm cache; updates cache with new token |
| **TV-8** Double-401 auth refresh failure | Either | Sensor endpoint always returns HTTP 401; `PipelineExecutor` calls `acquire_token()` after first 401; retried request also returns 401 | `PipelineExecutor` propagates `SpecEngineError::AuthRefreshFailed` (E-AUTH-002); no additional acquisition attempt; HTTP POST count = 1 (force-refresh) |
| **TV-9** Credential not found | Either | `MockCredentialResolver` returns `CredentialResolutionError::NotFound` | `acquire_token()` returns `SpecEngineError::AuthAcquisitionFailed` with `detail = "E-AUTH-005: credential not found — no credential configured for ({client_id}, {sensor_id}) ref '{ref_name}'"` (E-AUTH-005 standalone wire code per error-taxonomy.md §E-AUTH-005); zero HTTP POSTs issued |
| **TV-10** Token POST returns HTTP 500 | Either | Mock token server returns HTTP 500 for all requests | `acquire_token()` returns `AuthAcquisitionFailed` (E-AUTH-001); no token cached; no retry of the token POST |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-159 | Lazy acquisition and refresh-on-expiry invariants for `DeclarativeHttpAuthProvider`. Integration tests via wiremock (`MockServer`) for network isolation — `token_url` is set to the `MockServer` URI at `new_for_test` construction time (ADR-054 §D4 OPTION (b): internally-constructed ADR-050 reqwest client; no `MockHttpClient`; no HTTP injection seam in the production constructor). Clock seam: `now_fn` parameter on `new_for_test` (`Arc<dyn Fn() -> u64 + Send + Sync>`, typically wrapping `Arc<AtomicU64>`) advances time deterministically for TTL expiry assertions. Behavioral state-transition sequences. Module: `prism-spec-engine`. Tool: `integration_test`. BC: BC-2.16.014 (this contract). Properties verified: (a) `::new()` makes zero network calls; (b) cold `get_token()` → exactly one HTTP POST; (c) warm `get_token()` (within TTL) → zero HTTP POSTs; (d) stale `get_token()` (past TTL) → exactly one HTTP POST re-acquisition; (e) empty-token `get_token()` → exactly one HTTP POST (same as cold); (f) direct `acquire_token()` → exactly one HTTP POST (cache bypass); (g) TTL arithmetic correctness for both `absolute_utc_string` and `relative_seconds` expiry modes; (h) `CachedAuthToken` never stores credential values (AD-017 assertion). Authored v1.0 (vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md); registered in VP-INDEX v1.83 per D-1947. |

## Related BCs

- BC-2.16.009: Sensor Spec File Validation — Rule 10 is the E-SPEC-028 `[auth_acquisition]`
  coherence gate; all preconditions in this BC depend on Rule 10 having passed (depends on)
- BC-2.16.001: Sensor Spec File Loading — discovery and loading of sensor specs containing
  `[auth_acquisition]` blocks (depends on)
- BC-2.16.002: Multi-Step Fetch Pipeline Execution — `PipelineExecutor` owns the P6 401-retry
  dispatch; any `event_type` tracing emissions added during acquisition require BC-2.16.002
  Canonical Structured Event Catalog rows (PG-LP11-001) (composes with)
- BC-2.16.013: Bundled Sensor Spec DTU Parity — TOML specs for the 4 initial sensors that
  gain `[auth_acquisition]` blocks are governed by both this BC and BC-2.16.013 (composes with)
- BC-2.01.016: SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) — the plugin-based path this BC supersedes
  for `oauth2_client_credentials`; `custom_via_plugin` + `PluginAuthProvider` path is preserved
  (supersedes within declarative-native oauth2_client_credentials scope)
- BC-2.01.017: StaticCookieAuthProvider Contract — sibling auth provider implementing the same
  `AuthProvider` trait (`crates/prism-spec-engine/src/auth_provider.rs`) for `cookie_roundtrip`;
  reference pattern for trait-object-safe async trait, `CredentialResolver` DI, and AD-017
  credential safety (sibling pattern reference)
- BC-2.06.003: Four-Tier Per-Client Credential Resolution — the credential resolution chain
  invoked by `CredentialResolver::resolve()` at `acquire_token()` call time (depends on)

## Architecture Anchors

- ADR-054 §D1 — `token_exchange` variant added to `AuthType` closed enum
  (`crates/prism-spec-engine/src/spec_parser.rs`); `VALID_AUTH_TYPES` const extended;
  `AuthType::as_str()` exhaustive match gains `TokenExchange => "token_exchange"` arm
- ADR-054 §D2 — `oauth2_client_credentials` native migration: `step9a_populate_adapter_registry`
  `Oauth2ClientCredentials` arm rewrites from per-org `PluginAuthProvider` to
  `DeclarativeHttpAuthProvider`; amends ADR-023 §Rule 4 (standard HTTP flows do not require
  a WASM plugin)
- ADR-054 §D3 — `[auth_acquisition]` TOML block schema (`AuthAcquisitionConfig` struct);
  `token_path` is a literal string; `ttl_buffer_secs` defaults to 30; `token_exchange`-only
  fields: `credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode`
- ADR-054 §D4 — `DeclarativeHttpAuthProvider` implementation contract (planned location:
  `crates/prism-spec-engine/src/auth/declarative.rs`); internal state (all 6 fields per §D4):
  `config: AuthAcquisitionConfig`, `credential_resolver: Arc<dyn CredentialResolver>`,
  `cached_token: ArcSwap<Option<CachedAuthToken>>`, `token_url: String` (per-org derived,
  `format!("{}{}", resolved_base_url, config.token_path)`),
  `http_client: reqwest::Client` (ADR-050-compliant, internally constructed via
  `build_http_client_with_timeout()`; NOT injectable — no HTTP injection seam in production
  constructor), `now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` (clock seam; sole test seam);
  `acquire_token()` and `get_token()` step sequences; `AuthProvider` trait from
  `crates/prism-spec-engine/src/auth_provider.rs`
- ADR-054 §D7 — auth strategy dispatch table for `step9a_populate_adapter_registry`
  (`crates/prism-bin/src/spec_driven_adapter.rs`): new `TokenExchange` arm constructs
  `DeclarativeHttpAuthProvider(TokenExchange)`; `Oauth2ClientCredentials AND auth_acquisition.is_some()`
  arm constructs `DeclarativeHttpAuthProvider(Oauth2ClientCredentials)`
- ADR-054 §D10 — E-SPEC-028 validation error suite (8 message templates a–h); registered
  as BC-2.16.009 Rule 10 (after ADR-053 D2's Rule 9 for `header_scheme`)
- ADR-053 §D2 — Armis `token_exchange` wiring (canonical TOML config example illustrating
  how the general `[auth_acquisition]` mechanism expresses the Armis v1 token-exchange flow)
- ADR-050 §D3 — `reqwest` rustls-tls requirement applies to the token acquisition POST client
- AD-017 — AI-opaque credential model; token strings cached but never logged; credentials
  never stored in provider state
- ADR-034 §D1 — `PrismCredentialResolver` (wraps `prism_credentials::resolve_credential`);
  OrgSlug→OrgId resolution in `prism-spec-engine`, not in `prism-credentials`
- ADR-023 §Rule 4 (amended by ADR-054) — standard RFC 6749 / standard HTTP form-POST-for-token
  flows do NOT require a WASM plugin; `custom_via_plugin` is preserved for genuinely non-standard
  auth only

## Story Anchor

`[PENDING — Wave-A story decomposition per ADR-054 §D7 merge-dependency sequencing: ADR-054
implementation stories land AFTER the standalone Wave-A engine story (ADR-053). Implementation
story IDs to be assigned during Wave-A story decomposition.]`

## VP Anchors

- VP-159: lazy acquisition and refresh-on-expiry integration tests; authored v1.0
  (vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md); registered in
  VP-INDEX v1.83 per D-1947

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the runtime token lifecycle of `DeclarativeHttpAuthProvider`, which is the auth acquisition component for sensors using config-driven declarative auth types (`oauth2_client_credentials` and `token_exchange`) expressed through `[auth_acquisition]` TOML blocks. Without this contract, the config-driven sensor adapter layer (CAP-029's primary concern) cannot authenticate against sensor APIs that require token acquisition — making this BC a core behavioral obligation of CAP-029. |
| L2 Invariants | DI-012 ("Spec-Driven Auth With Runtime Composition Guards") per invariants.md — ADR-054 amends DI-012 (`amends_dis: ["DI-012"]`) by adding `token_exchange` as the 6th valid `auth_type` variant; this BC enforces that `DeclarativeHttpAuthProvider` handles both declarative variants without sensor-name-conditional logic (INV-014-001, POL-36), directly implementing DI-012's "config-driven, not code-driven" invariant for the new auth type |
| L2 Entities | SensorSpec, AuthAcquisitionConfig, DeclarativeHttpAuthProvider, CachedAuthToken |
| Priority | P0 |
| ADR anchors | ADR-054 §D1/D2/D3/D4/D7/D10 (source authority for postconditions P1–P8); ADR-054 §D4/§D11 (source authority for P9 — get_token() PipelineExecutor production call sites, PLANNED per engine story); ADR-053 §D2 (Armis wiring as TOML config example); ADR-050 §D3 (reqwest rustls-tls); ADR-034 §D1 (credential resolver construction); AD-017 (credential safety); ADR-023 §Rule 4 (amended — standard flows do not require WASM) |
| Subsystem | SS-16 (Spec Engine) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.10 | wave-a-spec-evolution-fix-burst-15 | 2026-07-22 | product-owner | F-WASE-P15-LOW-001: §Architecture Anchors ADR-054 §D4 bullet extended from 3 internal-state fields to the full 6 ratified by ADR-054 v0.41 §D4: added `token_url: String` (per-org derived, `format!("{}{}", resolved_base_url, config.token_path)`), `http_client: reqwest::Client` (ADR-050-compliant, internally constructed via `build_http_client_with_timeout()`; NOT injectable), `now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` (clock seam; sole test seam). input-hash updated at commit time. |
| 1.9 | wave-a-spec-evolution-fix-burst-14 | 2026-07-22 | product-owner | F-WASE-P14-HIGH-001: §Verification Properties VP-159 row rewritten — replaced abandoned `MockHttpClient` network-isolation description with the ratified ADR-054 v0.40 §D4 OPTION (b) model: wiremock (`MockServer`) for HTTP interception (`token_url` routed to `MockServer` URI at `new_for_test` construction time; no `MockHttpClient`; no HTTP injection seam in production constructor) plus `now_fn` clock seam (`Arc<dyn Fn() -> u64 + Send + Sync>`, typically `Arc<AtomicU64>`) for deterministic TTL expiry assertions. Sweep: one live-body `MockHttpClient` hit found (§Verification Properties row, line 365) and fixed; no changelog rows contained `MockHttpClient` (exempt). input-hash updated at commit time. |
| 1.8 | wave-a-spec-evolution-fix-burst-12 | 2026-07-22 | product-owner | F-WASE-P12-LOW-001: §Related BCs label for BC-2.01.016 corrected from "Plugin Auth Provider Construction" to canonical H1 title "SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker)". The old label was a stale paraphrase; H1 is the authoritative title source per bc_h1_is_title_source_of_truth policy. Relationship description and supersedes rationale preserved unchanged. input-hash updated at commit time. |
| 1.7 | wave-a-spec-evolution-fix-burst-9 | 2026-07-22 | product-owner | F-WASE-P9-OBS-002: §Error Conditions E-AUTH-005 row trailing claim "aligned with `StaticCookieAuthProvider` pattern (BC-2.01.017 EC-017-003) generalized for variable ref names" reworded to "generalizes the `StaticCookieAuthProvider` E-AUTH-005-in-detail pattern (BC-2.01.017 EC-017-003) with ref-name-agnostic placeholders" — the as-built sibling detail differs materially (hardcoded `api_key` wording plus trailing `configure_credential_source` guidance); the new wording is precise about the relationship without overstating alignment fidelity. input-hash updated at commit time. |
| 1.6 | wave-a-spec-evolution-fix-burst-8 | 2026-07-22 | product-owner | F-WASE-P8-LOW-001: reordered §Postconditions so P8 precedes P9 (P9 was inserted between P7 and P8 at v1.5; P8 section block moved before P9; IDs not renumbered per append-only-numbering policy; ADR-054 D4/D11 and VP-159 AC-9 references to P9 identity remain valid). F-WASE-P8-LOW-002: §Traceability ADR anchors cell updated from "source authority for all postconditions P1–P8" to P1–P9, noting P9's source authority as ADR-054 §D4/§D11 (P1–P8 source authority remains §D1/D2/D3/D4/D7/D10). Live-body sweep for "P1–P8": one instance found and fixed (ADR anchors cell); changelog rows exempt. input-hash updated at commit time. |
| 1.5 | wave-a-spec-evolution-fix-burst-7 | 2026-07-22 | product-owner | F-WASE-P7-HIGH-001: P5 caller list reconciled with ADR-054 v0.38 §D4 — bullet 1 changed "By" to "Internally by"; bullet 2 expanded to name `issue_request_with_retry` as the dispatch site and adds explicit note that the 401-refresh arm MUST call `acquire_token()` NOT `get_token()`. P6 updated to name `issue_request_with_retry` in heading; step 1 adds NOT-`get_token()` rationale; step 3 adds "with an intervening `acquire_token()` call between the two 401s" to specify the exact E-AUTH-002 trigger condition; appended paragraph making acquire_token()-not-get_token() explicit and tracing the two-401 sequence. P9 added: get_token() production callers (PipelineExecutor::execute_impl and execute_step, both marked [PLANNED — engine story] per ADR-054 D11). F-WASE-P7-LOW-001: E-AUTH-005 detail template specified at all three sites (EC-016-014-009, §Error Conditions E-AUTH-005 row, TV-9): `"E-AUTH-005: credential not found — no credential configured for ({client_id}, {sensor_id}) ref '{ref_name}'"` aligned with canonical error-taxonomy §E-AUTH-005 and StaticCookieAuthProvider pattern (BC-2.01.017 EC-017-003) generalized for variable ref names. |
| 1.4 | wave-a-spec-evolution-fix-burst-6 | 2026-07-22 | product-owner | F-WASE-P6-LOW-001: clarify E-AUTH-005 credential-not-found contract at all three sites. (1) §Error Conditions E-AUTH-005 row: removed "(detail within `E-AUTH-001`)" label — E-AUTH-005 is a standalone wire code per error-taxonomy.md §E-AUTH-005 and `CredentialResolver` trait ("Callers should map this to E-AUTH-005"); updated Behavior column to name the implementation vehicle (`AuthAcquisitionFailed` with E-AUTH-005 in `detail`) and cite BC-2.01.017 EC-017-003 as sibling pattern. (2) EC-016-014-009: replaced "E-AUTH-005 detail" with explicit statement that E-AUTH-005 is the standalone wire code carried in `AuthAcquisitionFailed.detail`. (3) TV-9: same phrasing fix — E-AUTH-005 is the standalone wire code, implementation vehicle is `AuthAcquisitionFailed` with E-AUTH-005 in `detail`. No as-built SpecEngineError variant gap exists: `AuthAcquisitionFailed{detail: "E-AUTH-005: ..."}` is the ratified mechanism (matches StaticCookieAuthProvider in BC-2.01.017 EC-017-003; confirmed in auth_provider.rs lines 516–526). |
| 1.3 | wave-a-spec-evolution-fix-burst-5 | 2026-07-22 | product-owner | F-WASE-P5-MED-002: trail reconciliation — input-hash trail: bc9f412 (post-v1.1/D-1948) → recomputed to current frontmatter value at commit time (D-1951 upstream input edits: error-taxonomy v2.60, BC-2.16.009 v1.15; v1.2 changelog row did not document the D-1951 recompute; see frontmatter for settled value). No BC content changed. |
| 1.2 | wave-a-fix-burst-1 | 2026-07-22 | product-owner | F-WASE-P1-LOW-003: §Preconditions item 3 count corrected "four" → "five" (the item lists five fields: `token_path`, `credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode`). F-WASE-P1-MED-001: §Changelog v1.0 burst corrected from "D-1943 Wave-A spec-evolution BC authoring" → "D-1946 Wave-A spec-evolution BC authoring" (D-1943 was the ADR acceptance decision, not the BC authoring decision); §Changelog v1.1 Change description "D-1947 authorship event" corrected to "D-1948 authorship event" (D-1947 was VP-159 registration, not the v1.1 burst decision; VP-159 citations in §Description/§Verification Properties/§VP Anchors citing "per D-1947" are CORRECT and left unchanged). |
| 1.1 | wave-a-spec-evolution-burst-3-correction | 2026-07-22 | product-owner | ADR-054 v0.33 D10(c) adjudication applied: §Description VP-159 reference updated from "(planned, see ADR-054 §D9)" to "(authored v1.0, D-1947; registered in VP-INDEX v1.83)"; §Verification Properties VP-159 row updated to present tense — removed "(PLANNED — see ADR-054 §D9)" label and "VP-159 will be registered in VP-INDEX by the architect after implementation begins per ADR-054 §D9" future-tense sentence, replaced with "Authored v1.0 (vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md); registered in VP-INDEX v1.83 per D-1947"; §VP Anchors updated to present tense referencing the authored VP file and VP-INDEX registration. D-1948 authorship event. input-hash 827ac61→bc9f412. |
| 1.0 | D-1946 Wave-A spec-evolution BC authoring | 2026-07-22 | product-owner | Initial draft — BC anchor for `DeclarativeHttpAuthProvider` token lifecycle. Postconditions P1–P8 authored from ADR-054 §D8 (authoritative source). Preconditions from ADR-054 §D3/D4/D7/D8. Invariants INV-014-001..008 (POL-36 generalization constraint, per-org token URL derivation, AD-017 credential safety, ArcSwap atomicity, DI-012 compliance, ADR-050 reqwest compliance, E-SPEC-028 spec-load gate). Edge cases EC-016-014-001..015. Error conditions referencing E-SPEC-028(a)–(h), E-AUTH-001/002/005, E-SPEC-024. Canonical test vectors TV-1..10. VP-159 planned per ADR-054 §D9 (not authored here — architect scope). ADR-054 D11 amendment manifest items NOT executed here — separate burst. BC-INDEX registration NOT performed here — separate burst. |
