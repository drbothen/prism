---
document_type: verification-property
level: L4
version: "1.6"
status: draft
producer: architect
timestamp: 2026-07-22T00:00:00Z
phase: wave-a
inputs:
  - .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md
  - .factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md
input-hash: "d0f0001"
traces_to: .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md
source_bc: BC-2.16.014
source_adr: ADR-054
source_invariant: DI-012
module: prism-spec-engine
priority: P1
proof_method: integration_test
verification_method: integration_test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-07-22"
modified: "2026-07-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-159: DeclarativeHttpAuthProvider — Lazy Acquisition and Refresh-on-Expiry

## Property Statement

`DeclarativeHttpAuthProvider` [PLANNED — engine story:
`crates/prism-spec-engine/src/auth/declarative.rs`], implementing the `AuthProvider` trait
(confirmed in `crates/prism-spec-engine/src/auth_provider.rs`), MUST satisfy the following
network-isolation and cache-lifecycle invariants (ADR-054 §D9; BC-2.16.014 P1–P5, P7, P9; TTL sub-properties P4-TTL-a/b):

**Network-isolation invariants:**

1. **P1 — zero network calls at construction:** `DeclarativeHttpAuthProvider::new()` [PLANNED]
   makes ZERO network calls. Token acquisition is deferred until the first call to `get_token()`
   [PLANNED] or `acquire_token()`. No HTTP client is invoked, no credential is resolved, and no
   TCP connection is opened during construction.

2. **P2 — cold `get_token()` — exactly one HTTP POST:** On the first call to `get_token()`
   [PLANNED] with an empty or absent `CachedAuthToken` [PLANNED], exactly ONE HTTP POST is
   issued to the derived token URL (`base_url + token_path`), the response token is cached in
   `ArcSwap<Option<CachedAuthToken>>` [PLANNED], and the token is returned.

3. **P3 — warm `get_token()` — zero HTTP requests:** Subsequent `get_token()` [PLANNED] calls
   issued before `unix_now() >= expires_at` return the cached token and issue ZERO HTTP requests.

4. **P4 — stale `get_token()` — exactly one HTTP POST:** A `get_token()` [PLANNED] call when
   the cache entry's `expires_at` timestamp is in the past (i.e., `unix_now() >= expires_at`)
   issues exactly ONE HTTP POST and refreshes the cache atomically.

5. **P5 — `acquire_token()` — always exactly one HTTP POST (cache bypass):** The `acquire_token()`
   method (satisfying the `AuthProvider` trait confirmed in
   `crates/prism-spec-engine/src/auth_provider.rs`) ALWAYS issues exactly ONE HTTP POST,
   regardless of cache state. It bypasses the TTL check and replaces the cached token on success.

**TTL arithmetic invariants:**

6. **P4-TTL-a — `absolute_utc_string` expiry mode:** When `ExpiryMode` [PLANNED] is
   `AbsoluteUtcString`, `expires_at = parse_rfc3339(expiry_str).as_unix_secs().saturating_sub(ttl_buffer_secs)`.

7. **P4-TTL-b — `relative_seconds` expiry mode:** When `ExpiryMode` [PLANNED] is
   `RelativeSeconds`, `expires_at = unix_now() + expires_in.saturating_sub(ttl_buffer_secs)`,
   where `expires_in` defaults to 1799 when absent or zero (matching the retired
   crowdstrike-oauth2 plugin's arithmetic; `.max(1)` is omitted because the absent/zero
   default is already 1799 — dead code per ADR-054 §D4 note).

**Credential-opacity invariant:**

8. **P7 — `CachedAuthToken` never stores credential values:** The `CachedAuthToken` struct
   [PLANNED] stores ONLY the opaque token string and the expiry timestamp. Credential values
   (API secrets, client secrets, passphrases) resolved via the `CredentialResolver` trait
   (confirmed in `crates/prism-spec-engine/src/auth_provider.rs`) are NEVER stored in
   `CachedAuthToken` fields. Violating this invariant leaks credentials to process memory and
   log drains (AD-017).

> **Scope note — P6 and P8 (deferred); P9 (verified via AC-9 + AC-9b):** P9 (`get_token()`
> production callers per ADR-054 v0.38 §D4/D11; `[PLANNED — engine story]`) is split across two
> production-reachable paths: the `PipelineExecutor::execute` → `execute_impl` path is verified by
> AC-9 (SAP-3 executor reachability test for the execute path); the `PipelineExecutor::execute_step`
> direct-call path (plugin-runtime entry point per ADR-054 v0.38 §D11) is verified by AC-9b (SAP-3
> execute_step reachability test). Together AC-9 and AC-9b fully cover P9. P6 (double-401 →
> `AuthRefreshFailed`, E-AUTH-002) is inherent in the `acquire_token()` contract per the confirmed
> `AuthProvider` trait and is verified via AC-5 + error-path assertions in the engine
> implementation story (not directly asserted by VP-159's mock-HTTP harness). P8 (`base_url`
> env-var interpolation obeys BC-2.16.009 Rule 6 / E-SPEC-024; `token_path` is a literal relative
> path and does not undergo env-var interpolation) is a spec-load validation property, not a
> runtime lifecycle invariant of `DeclarativeHttpAuthProvider`; deferred to the spec-engine
> validation story's error-path assertions.

---

**DRIFT-D849-002 fold note:** This VP resolves DRIFT-D849-002 ("VP for no-network calls during
spec-load phase for auth providers"). The drift item sought formal verification that auth
providers make no network calls during spec load. `StaticCookieAuthProvider`'s zero-HTTP
property is a structural guarantee — no `reqwest::Client` field exists in that type (confirmed
in `crates/prism-spec-engine/src/auth_provider.rs`), making zero HTTP calls architecturally
impossible — and is already covered by BC-2.01.017 §P1 (INV-COOKIE-001); no separate VP is
needed for that provider. VP-159 extends the verified network-isolation invariant to
`DeclarativeHttpAuthProvider` [PLANNED], which is the new provider that does make HTTP calls
(at `acquire_token()` / `get_token()` time) but MUST NOT at construction or spec-load time.
DRIFT-D849-002 status: **FOLDED into VP-159**.

## Acceptance Criteria

The integration test harness uses `MockHttpClient` [PLANNED — engine story] to intercept all
outbound HTTP calls without requiring a real network. `MockCredentialResolver` (confirmed in
`crates/prism-spec-engine/src/auth_provider.rs`, gated
`#[cfg(any(test, feature = "test-helpers"))]`) is used for credential injection. The harness
asserts:

- **AC-1 (P1):** Calling `DeclarativeHttpAuthProvider::new()` [PLANNED] with a valid
  `AuthAcquisitionConfig` [PLANNED] results in zero calls recorded by `MockHttpClient`
  [PLANNED].

- **AC-2 (P2):** After construction, calling `get_token()` [PLANNED] once on a cold cache
  results in exactly one `MockHttpClient` POST call. The returned token matches the mock
  response body.

- **AC-3 (P3):** Calling `get_token()` [PLANNED] again within TTL results in zero additional
  `MockHttpClient` calls. The same token string is returned.

- **AC-4 (P4):** Advancing the mock clock past `expires_at` and calling `get_token()` [PLANNED]
  results in exactly one additional `MockHttpClient` POST call. The refreshed token is returned.

- **AC-5 (P5):** Calling `acquire_token()` on a warm-cache provider results in exactly one
  `MockHttpClient` POST call (bypasses TTL check regardless of cache state).

- **AC-6 (P4-TTL-a):** For `ExpiryMode::AbsoluteUtcString` [PLANNED], the computed `expires_at`
  equals `parse_rfc3339(response_expiry_field).as_unix_secs().saturating_sub(ttl_buffer_secs)`.

- **AC-7 (P4-TTL-b):** For `ExpiryMode::RelativeSeconds` [PLANNED], the computed `expires_at`
  equals `unix_now() + expires_in.saturating_sub(ttl_buffer_secs)`, with `expires_in = 1799`
  when the response field is absent or zero.

- **AC-8 (P7):** Inspecting the `CachedAuthToken` [PLANNED] fields after a successful
  `get_token()` call reveals only `token: String` (opaque bearer/session string) and
  `expires_at: u64` (Unix timestamp) — no credential field. The test fixture controls the mock
  credential and mock token values to be distinct strings, enabling a negative assertion that the
  cached token does not equal the resolved credential.

- **AC-9 (P9-execute_impl path; SAP-3 execute reachability):** An end-to-end test drives
  `PipelineExecutor::execute` [PLANNED — engine story] with a `DeclarativeHttpAuthProvider`
  [PLANNED] instance. Two consecutive `PipelineExecutor::execute` calls are issued against the
  same provider (same org, same sensor), with the mock token server configured to return a
  long-lived TTL (e.g., `expires_in = 3600`). The mock token server records exactly ONE POST to
  the token endpoint across both `execute` calls — confirming that the second execution returns
  the cached token via `get_token()` (cache-aware path) rather than force-refreshing via
  `acquire_token()`. This test is the SAP-3 end-to-end reachability requirement for BC-2.16.014
  P9 `execute_impl` path (`PipelineExecutor::execute` → `execute_impl` per ADR-054 v0.38 §D4):
  it proves `get_token()` is reached through the production `execute_impl` caller path, not only
  by direct mock invocation. The `execute_step` path is separately verified by AC-9b.

  **SAP-3 note (CLAUDE.md Standing Adversary Probes §SAP-3):** AC-2/3/4 verify `get_token()` by
  direct invocation on `DeclarativeHttpAuthProvider` — they provide precise state-machine coverage
  of the cache lifecycle but are synthetic-invocation tests. Per SAP-3, each production-reachable
  path must have at least one end-to-end test from the public surface. AC-9 covers the
  `PipelineExecutor::execute` → `execute_impl` path; AC-9b (below) covers the
  `PipelineExecutor::execute_step` direct-call path. Both AC-2/3/4 (isolation) and AC-9 + AC-9b
  (reachability) are required. Without AC-9, a mis-wiring in `execute_impl` would pass AC-2/3/4;
  without AC-9b, a mis-wiring in `execute_step` would pass AC-9 and AC-2/3/4 undetected.

- **AC-9b (P9-execute_step path; SAP-3 execute_step reachability):** An end-to-end test drives
  `PipelineExecutor::execute_step` [PLANNED — engine story] directly — the plugin-runtime entry
  point per ADR-054 v0.38 §D11. A `FetchStep` is constructed via
  `prism_spec_engine::spec_parser::FetchStep::new(name, method, path_template, body_template,
  response_path, pagination_cursor_path, variables_produced, fan_out_batch_size, pagination)`
  (confirmed `pub fn new` signature at `crates/prism-spec-engine/src/spec_parser.rs`; struct-literal
  construction is E0639-impossible from `tests/` because `FetchStep` is `#[non_exhaustive]`). Two
  consecutive `PipelineExecutor::execute_step` calls are
  issued with the same `DeclarativeHttpAuthProvider` [PLANNED] instance, the same
  `MockHttpClient` [PLANNED], and `prior_vars: HashMap::new()` (no cross-step variable
  dependencies required for a single-step reachability test). The mock token server is configured
  with long-lived TTL (`expires_in = 3600`). The mock token server records exactly ONE POST to the
  token endpoint across both `execute_step` calls — confirming that the second call returns the
  cached token via `get_token()` (cache-aware path) rather than force-refreshing via
  `acquire_token()`. A mis-wiring that leaves `acquire_token()` in `execute_step` (the current
  pre-engine-story state, confirmed in `crates/prism-spec-engine/src/pipeline.rs`) produces 2
  POSTs and FAILS this test, exposing the defect that AC-9 cannot detect (AC-9 drives only
  `execute` → `execute_impl` and never calls `execute_step`).

  **Signature note:** `PipelineExecutor::execute_step` takes `(step: &FetchStep, spec: &SensorSpec,
  prior_vars: &std::collections::HashMap<String, serde_json::Value>, context: &FetchContext,
  http_client: &reqwest::Client, auth_provider: &dyn AuthProvider)` returning
  `Result<serde_json::Value, SpecEngineError>` — confirmed from `PipelineExecutor::execute_step`
  in `crates/prism-spec-engine/src/pipeline.rs`. The `http_client` parameter is the sensor API
  client (separate from `MockHttpClient` [PLANNED] held inside `DeclarativeHttpAuthProvider`); the
  sensor API mock endpoint (GET `/items` → `{"items": [{"id": 1}]}`) uses wiremock (confirmed
  dev-dependency in `prism-spec-engine`) alongside `MockHttpClient` [PLANNED] for the token
  endpoint. `FetchContext::new` is the confirmed non-exhaustive constructor (`OrgSlug`,
  `HashMap<String, String>`).

## Source Contract

- **BC:** BC-2.16.014 (`DeclarativeHttpAuthProvider` Token Lifecycle) v1.6 — postconditions P1–P9
  (BC-2.16.014 v1.6) are the primary **authoring source** for this VP; the verified set is
  P1–P5, P7, P9 (plus P4-TTL-a/b sub-properties) — see §Property Statement scope note for
  P6/P8 (deferred) and P9-via-AC-9 (verified) coverage.
  INV-014-003 (BC-local invariant:
  "Credential Lazy Resolution — AD-017") specifically governs the credential-opacity property
  verified by AC-8. Note: INV-014-003 is a BC-scoped invariant identifier; it is cited in
  body prose only and does not populate `source_invariant:` per VP-INDEX source_invariant schema
  convention.

- **ADR:** ADR-054 §D9 — "VP-159 Authoring Source: Lazy Acquisition and Refresh-on-Expiry
  Invariants". The property statement, tool selection (integration_test), and TTL arithmetic
  formulas are sourced directly from ADR-054 §D9 and §D4.

- **Invariant:** DI-012 ("Spec-Driven Auth With Runtime Composition Guards" from
  `domain-spec/invariants.md`) — the workspace-canonical invariant governing auth_type dispatch
  and provider construction. ADR-054 amends DI-012 to add `token_exchange` as the 6th variant.
  VP-159 verifies the runtime lifecycle invariants for the `token_exchange` and
  `oauth2_client_credentials` (declarative) providers that DI-012 governs.

- **Module:** prism-spec-engine (`crates/prism-spec-engine/src/auth/declarative.rs` [PLANNED —
  engine story; directory does not exist at authoring time])

- **Existing symbols verified in codebase at authoring time (2026-07-22):**
  - `AuthProvider` trait with `acquire_token()` — `crates/prism-spec-engine/src/auth_provider.rs`
  - `CredentialResolver` trait with `resolve()` — `crates/prism-spec-engine/src/auth_provider.rs`
  - `MockCredentialResolver` — `crates/prism-spec-engine/src/auth_provider.rs`
    (gated `#[cfg(any(test, feature = "test-helpers"))]`)
  - `SpecEngineError::AuthAcquisitionFailed` — `crates/prism-spec-engine/src/error.rs`
    (E-AUTH-001: acquisition-level network/credential failure)
  - `SpecEngineError::AuthRefreshFailed` — `crates/prism-spec-engine/src/error.rs`
    (E-AUTH-002: double-401 scenario only — first 401 triggers force-refresh; second
    consecutive 401 → AuthRefreshFailed per BC-2.16.014 P6)

- **DRIFT-D849-002 fold:** See §Property Statement "DRIFT-D849-002 fold note" above.

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| integration_test | `MockHttpClient` [PLANNED — engine story] for HTTP interception with call-count tracking; `MockCredentialResolver` (confirmed in `crates/prism-spec-engine/src/auth_provider.rs`) for credential injection; mock clock for deterministic TTL expiry | Deterministic — fixed scenario sequences covering each cache state; not combinatorial | Cold → warm → stale → refresh state machine; both ExpiryMode variants; cache-bypass via acquire_token; credential-opacity structural assertion |

**Why integration_test over Kani:** The VP covers a behavioral state machine with I/O interaction
(HTTP) and time-dependent cache state. Kani model-checks bounded numeric state spaces but cannot
model async HTTP client interactions or real-time TTL arithmetic with mock clocks. The behavioral
sequences (cold → warm → stale → refresh) are deterministic fixed scenarios, not combinatorial
input spaces — this is the same pattern as VP-033 and VP-036 (integration_test for DTU parity)
and VP-146..VP-155 (integration_test for plugin lifecycle invariants per ADR-054 §D9).

**Why not proptest:** The invariants are state-machine lifecycle properties (construction → cold
→ warm → stale → refresh), not an input-space exploration over a mathematical domain. Proptest's
combinatorial generation adds no coverage over a well-chosen set of deterministic mock scenarios.

## Proof Harness Skeleton

```rust
// crates/prism-spec-engine/tests/vp159_declarative_auth_lazy_acquisition.rs
//
// VP-159: DeclarativeHttpAuthProvider lazy acquisition and refresh-on-expiry
// Method: integration_test (MockHttpClient for network isolation)
// Target module: prism-spec-engine
// Target path: crates/prism-spec-engine/src/auth/declarative.rs [PLANNED — engine story]
// BC: BC-2.16.014 v1.6 (P1–P5, P7, P9; P4-TTL-a/b sub-properties; P6/P8 deferred, P9-via-AC-9+AC-9b verified — see §Property Statement scope note); ADR: ADR-054 §D9; source_invariant: DI-012
//
// ALL DeclarativeHttpAuthProvider / CachedAuthToken / AuthAcquisitionConfig / ExpiryMode /
// MockHttpClient symbols below are [PLANNED — engine story].
//
// Confirmed existing symbols used:
//   AuthProvider trait:        crates/prism-spec-engine/src/auth_provider.rs
//   MockCredentialResolver:    crates/prism-spec-engine/src/auth_provider.rs
//                              (cfg(any(test, feature = "test-helpers")))
//   SpecEngineError::AuthAcquisitionFailed: crates/prism-spec-engine/src/error.rs (E-AUTH-001)
//   SpecEngineError::AuthRefreshFailed:     crates/prism-spec-engine/src/error.rs (E-AUTH-002)
//   ArcSwap:                   arc_swap crate (external)

// #[cfg(test)]
// mod vp159_tests {
//     use std::sync::Arc;
//     use prism_spec_engine::auth::declarative::{
//         DeclarativeHttpAuthProvider,  // [PLANNED]
//         AuthAcquisitionConfig,        // [PLANNED]
//         ExpiryMode,                   // [PLANNED]
//         CachedAuthToken,              // [PLANNED]
//     };
//     use prism_spec_engine::auth_provider::{AuthProvider, MockCredentialResolver};
//     // MockHttpClient tracks POST call count and returns configurable responses [PLANNED]:
//     use prism_spec_engine::auth::test_helpers::MockHttpClient;  // [PLANNED]
//
//     fn base_config(token_path: &str, expiry_mode: ExpiryMode) -> AuthAcquisitionConfig { // [PLANNED]
//         AuthAcquisitionConfig {
//             token_path: token_path.to_string(),
//             expiry_mode,
//             ..Default::default()
//         }
//     }
//
//     // AC-1 (P1): zero network calls at construction
//     #[tokio::test]
//     async fn test_vp159_ac1_zero_network_at_construction() {
//         let mock_http = MockHttpClient::new();   // [PLANNED]
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let _provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config,
//             Arc::new(mock_http.clone()),
//             Arc::new(creds),
//         );
//         assert_eq!(mock_http.post_call_count(), 0,
//             "VP-159 AC-1: construction must make zero network calls (BC-2.16.014 P1)");
//     }
//
//     // AC-2 (P2): cold get_token → exactly one HTTP POST
//     #[tokio::test]
//     async fn test_vp159_ac2_cold_cache_one_post() {
//         let mock_http = MockHttpClient::with_response("bearer_token_abc", 3600); // [PLANNED]
//         let creds = MockCredentialResolver::new("client_secret_xyz");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _token = provider.get_token("test-org").await  // [PLANNED]
//             .expect("VP-159 AC-2: cold get_token must succeed");
//         assert_eq!(mock_http.post_call_count(), 1,
//             "VP-159 AC-2: cold get_token must issue exactly one HTTP POST (BC-2.16.014 P2)");
//     }
//
//     // AC-3 (P3): warm get_token → zero additional HTTP calls
//     #[tokio::test]
//     async fn test_vp159_ac3_warm_cache_zero_post() {
//         let mock_http = MockHttpClient::with_response("bearer_token_abc", 3600); // [PLANNED]
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await.expect("first call");  // [PLANNED] warms cache
//         let calls_after_warm = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("second call");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_after_warm,
//             "VP-159 AC-3: warm get_token must make zero HTTP calls (BC-2.16.014 P3)");
//     }
//
//     // AC-4 (P4): stale get_token → exactly one additional HTTP POST
//     #[tokio::test]
//     async fn test_vp159_ac4_stale_cache_one_post() {
//         // [PLANNED] MockHttpClient supports mock_clock::advance() to simulate TTL expiry
//         let mock_http = MockHttpClient::with_response("bearer_token_abc", 60); // [PLANNED] 60s TTL
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 0 }); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await.expect("cold call");  // [PLANNED] warms cache
//         mock_http.advance_clock_secs(120); // [PLANNED] advance past expires_at
//         let calls_before_stale_refresh = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("stale call");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_before_stale_refresh + 1,
//             "VP-159 AC-4: stale get_token must issue exactly one HTTP POST (BC-2.16.014 P4)");
//     }
//
//     // AC-5 (P5): acquire_token bypasses cache → exactly one HTTP POST
//     #[tokio::test]
//     async fn test_vp159_ac5_acquire_token_cache_bypass() {
//         let mock_http = MockHttpClient::with_response("acquired_token", 3600); // [PLANNED]
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await.expect("warm cache");  // [PLANNED]
//         let calls_after_warm = mock_http.post_call_count();
//         // acquire_token() is the AuthProvider::acquire_token() method (confirmed symbol)
//         // Called with test SensorSpec and OrgSlug [PLANNED test helpers]
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         // ALLOWLIST REQUIRED: OrgSlug::new_unchecked is used in this proof harness.
//         // The engine story implementing DeclarativeHttpAuthProvider MUST add this call site
//         // to crates/prism-core/tests/new_unchecked_audit.rs per CLAUDE.md
//         // credential-safety convention before the PR can merge.
//         let org_slug = prism_core::OrgSlug::new_unchecked("test-org");
//         let _token = provider.acquire_token(&sensor_spec, &org_slug).await
//             .expect("VP-159 AC-5: acquire_token must succeed even on warm cache");
//         assert_eq!(mock_http.post_call_count(), calls_after_warm + 1,
//             "VP-159 AC-5: acquire_token must issue exactly one HTTP POST regardless of cache (BC-2.16.014 P5)");
//     }
//
//     // AC-6 (P4-TTL-a): absolute_utc_string expiry arithmetic
//     // Formula: expires_at = parse_rfc3339(expiry_value).as_unix_secs().saturating_sub(ttl_buffer_secs)
//     // Verifies formula correctness by advancing mock clock to just before and then past expires_at.
//     // MockHttpClient::with_token_exchange_response [PLANNED — engine story] returns token + RFC-3339 expiry field.
//     // See BC-2.16.014 TV-2 for the canonical test vector (Armis-style token_exchange with absolute_utc expiry).
//     #[tokio::test]
//     async fn test_vp159_ac6_absolute_utc_expiry_ttl_arithmetic() {
//         // "2099-01-01T00:00:00Z" parse → Unix secs ≈ 4_070_908_800; ttl_buffer_secs=30 → expires_at ≈ 4_070_908_770
//         let expiry_utc = "2099-01-01T00:00:00Z";
//         let ttl_buffer_secs: u64 = 30;
//         let mock_http = MockHttpClient::with_token_exchange_response(  // [PLANNED — engine story]
//             "arm-tok",
//             expiry_utc,
//             // Response shape: {"success":true,"data":{"access_token":"arm-tok","expiration_utc":"<expiry_utc>"}}
//         );
//         let creds = MockCredentialResolver::new("long_lived_secret");
//         let config = AuthAcquisitionConfig {  // [PLANNED]
//             token_path: "/api/v1/access_token/".to_string(),
//             credential_body_field: "secret_key".to_string(),
//             token_response_path: "data.access_token".to_string(),
//             expiry_field: "data.expiration_utc".to_string(),
//             expiry_mode: ExpiryMode::AbsoluteUtcString,  // [PLANNED]
//             ttl_buffer_secs,
//             ..Default::default()
//         };
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await  // [PLANNED] — warms cache
//             .expect("VP-159 AC-6: absolute_utc_string get_token must succeed on well-formed expiry");
//         let calls_after_warm = mock_http.post_call_count();
//         // Advance clock to just before expires_at — no re-acquisition expected
//         // [PLANNED] advance_clock_to_before_expires_at sets mock time to (parse_rfc3339(expiry_utc) - ttl_buffer_secs - 10)
//         mock_http.advance_clock_to_before_expires_at(expiry_utc, ttl_buffer_secs);  // [PLANNED — engine story]
//         let _ = provider.get_token("test-org").await.expect("still warm pre-expiry");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_after_warm,
//             "VP-159 AC-6: clock before expires_at → cache valid, zero additional HTTP POSTs \
//              (BC-2.16.014 P3/P4-TTL-a)");
//         // Advance clock past expires_at — re-acquisition must fire
//         // [PLANNED] advance_clock_past_expires_at sets mock time to (parse_rfc3339(expiry_utc) - ttl_buffer_secs + 10)
//         mock_http.advance_clock_past_expires_at(expiry_utc, ttl_buffer_secs);  // [PLANNED — engine story]
//         let calls_before_stale = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("stale — re-acquisition");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_before_stale + 1,
//             "VP-159 AC-6: clock past expires_at → one HTTP POST re-acquisition \
//              (BC-2.16.014 P4 / P4-TTL-a: expires_at = parse_rfc3339(expiry_str).as_unix_secs().saturating_sub(ttl_buffer_secs))");
//     }
//
//     // AC-6b (EC-016-014-003): malformed RFC-3339 expiry string → AuthAcquisitionFailed (E-AUTH-001)
//     // No token cached when parse fails — acquire_token returns Err immediately (BC-2.16.014 P4-TTL-a).
//     #[tokio::test]
//     async fn test_vp159_ac6b_malformed_rfc3339_expiry_returns_auth_acquisition_failed() {
//         let mock_http = MockHttpClient::with_token_exchange_response(  // [PLANNED — engine story]
//             "arm-tok",
//             "not-a-date",  // malformed RFC-3339 value — parse must fail
//         );
//         let creds = MockCredentialResolver::new("long_lived_secret");
//         let config = AuthAcquisitionConfig {  // [PLANNED]
//             token_path: "/api/v1/access_token/".to_string(),
//             credential_body_field: "secret_key".to_string(),
//             token_response_path: "data.access_token".to_string(),
//             expiry_field: "data.expiration_utc".to_string(),
//             expiry_mode: ExpiryMode::AbsoluteUtcString,  // [PLANNED]
//             ttl_buffer_secs: 30,
//             ..Default::default()
//         };
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http), Arc::new(creds),
//         );
//         let result = provider.get_token("test-org").await;  // [PLANNED]
//         assert!(
//             matches!(result, Err(SpecEngineError::AuthAcquisitionFailed { .. })),
//             "VP-159 AC-6b: malformed RFC-3339 expiry must return AuthAcquisitionFailed \
//              (E-AUTH-001; EC-016-014-003; BC-2.16.014 P4-TTL-a)"
//         );
//     }
//
//     // AC-7 (P4-TTL-b): relative_seconds expiry arithmetic — verify formula + absent/zero → default 1799
//     // Formula: expires_at = unix_now() + expires_in.saturating_sub(ttl_buffer_secs)
//     //          where expires_in defaults to 1799 when absent or zero (EC-016-014-001 / EC-016-014-002)
//     // Sub-case 7a: normal expires_in (3600s) — verify clock-based re-acquisition timing
//     #[tokio::test]
//     async fn test_vp159_ac7_relative_seconds_expiry_ttl_arithmetic() {
//         let expires_in: u64 = 3600;
//         let ttl_buffer_secs: u64 = 30;
//         // expires_at ≈ unix_now() + 3600.saturating_sub(30) = unix_now() + 3570
//         let mock_http = MockHttpClient::with_response("bearer_tok_rel", expires_in);  // [PLANNED]
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs });  // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await.expect("warms cache");  // [PLANNED]
//         let calls_after_warm = mock_http.post_call_count();
//         mock_http.advance_clock_secs(3500);  // [PLANNED] 3500s < 3570s (expires_in 3600 − ttl_buffer 30)
//         let _ = provider.get_token("test-org").await.expect("still warm at 3500s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_after_warm,
//             "VP-159 AC-7a: 3500s < 3570s expires_at → cache valid (BC-2.16.014 P4-TTL-b)");
//         mock_http.advance_clock_secs(100);  // [PLANNED] total 3600s > 3570s → stale
//         let calls_before_stale = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("stale at 3600s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_before_stale + 1,
//             "VP-159 AC-7a: 3600s > 3570s expires_at → re-acquisition \
//              (BC-2.16.014 P4-TTL-b: expires_at = unix_now() + expires_in.saturating_sub(ttl_buffer_secs))");
//     }
//
//     // AC-7b (EC-016-014-001): absent expires_in → default 1799 before saturating_sub
//     // expires_at = unix_now() + 1799.saturating_sub(30) = unix_now() + 1769
//     // [PLANNED] MockHttpClient::with_absent_expires_in: response omits the expires_in key entirely
//     #[tokio::test]
//     async fn test_vp159_ac7b_absent_expires_in_defaults_to_1799() {
//         let ttl_buffer_secs: u64 = 30;
//         let mock_http = MockHttpClient::with_absent_expires_in("tok-noexp");  // [PLANNED — engine story]
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs });  // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await  // [PLANNED] cold — absent expires_in defaults to 1799
//             .expect("VP-159 AC-7b: absent expires_in must succeed with default 1799 TTL");
//         let calls_after_warm = mock_http.post_call_count();
//         mock_http.advance_clock_secs(1700);  // [PLANNED] 1700s < 1769s (1799 − 30) → still warm
//         let _ = provider.get_token("test-org").await.expect("still warm at 1700s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_after_warm,
//             "VP-159 AC-7b: absent expires_in → 1799 default; 1700s < 1769s expires_at → cache valid \
//              (EC-016-014-001; BC-2.16.014 P4-TTL-b)");
//         mock_http.advance_clock_secs(100);  // [PLANNED] total 1800s > 1769s → stale
//         let calls_before_stale = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("stale at 1800s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_before_stale + 1,
//             "VP-159 AC-7b: absent expires_in → 1799 default; 1800s > 1769s expires_at → re-acquisition \
//              (EC-016-014-001; BC-2.16.014 P4-TTL-b)");
//     }
//
//     // AC-7c (EC-016-014-002): zero expires_in → same default 1799 as absent
//     // MockHttpClient::with_response("tok-zeroexp", 0u64) supplies expires_in: 0 in response JSON
//     #[tokio::test]
//     async fn test_vp159_ac7c_zero_expires_in_defaults_to_1799() {
//         let ttl_buffer_secs: u64 = 30;
//         let mock_http = MockHttpClient::with_response("tok-zeroexp", 0u64);  // [PLANNED] expires_in: 0
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs });  // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         );
//         let _ = provider.get_token("test-org").await  // [PLANNED] cold — zero expires_in defaults to 1799
//             .expect("VP-159 AC-7c: zero expires_in must succeed with default 1799 TTL");
//         let calls_after_warm = mock_http.post_call_count();
//         mock_http.advance_clock_secs(1700);  // [PLANNED] still warm
//         let _ = provider.get_token("test-org").await.expect("still warm at 1700s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_after_warm,
//             "VP-159 AC-7c: zero expires_in → 1799 default; 1700s < 1769s expires_at → cache valid \
//              (EC-016-014-002; BC-2.16.014 P4-TTL-b)");
//         mock_http.advance_clock_secs(100);  // [PLANNED] total 1800s → stale
//         let calls_before_stale = mock_http.post_call_count();
//         let _ = provider.get_token("test-org").await.expect("stale at 1800s");  // [PLANNED]
//         assert_eq!(mock_http.post_call_count(), calls_before_stale + 1,
//             "VP-159 AC-7c: zero expires_in → 1799 default; 1800s > 1769s expires_at → re-acquisition \
//              (EC-016-014-002; BC-2.16.014 P4-TTL-b)");
//     }
//
//     // AC-8 (P7): CachedAuthToken never stores credential values — structural assertion
//     #[test]
//     fn test_vp159_ac8_cached_token_no_credential_field() {
//         // Exhaustive struct literal: if a 'credential' or 'secret' field were added
//         // to CachedAuthToken [PLANNED], this literal would fail to compile — enforcing
//         // the credential-opacity invariant at build time (BC-2.16.014 P7, AD-017).
//         let cached = CachedAuthToken {  // [PLANNED]
//             token: "opaque_bearer_token".to_string(),
//             expires_at: 9_999_999_999u64,
//         };
//         assert!(!cached.token.is_empty(),
//             "VP-159 AC-8: CachedAuthToken.token is the opaque token string, not empty");
//         assert!(cached.expires_at > 0,
//             "VP-159 AC-8: CachedAuthToken.expires_at is a valid Unix timestamp");
//         // The exhaustive struct literal above is the load-bearing assertion:
//         // CachedAuthToken has exactly {token, expires_at} — no credential field.
//     }
//
//     // AC-9 (P9-execute_impl path; SAP-3 execute reachability):
//     // PipelineExecutor::execute called twice, same warm DeclarativeHttpAuthProvider [PLANNED],
//     // long TTL → exactly 1 token-endpoint POST total across both calls.
//     // [PLANNED — engine story]: DeclarativeHttpAuthProvider, MockHttpClient,
//     //   build_test_sensor_spec_token_exchange, build_test_table_spec
//     #[tokio::test]
//     async fn test_vp159_ac9_execute_impl_path_cache_sharing() {
//         use wiremock::{MockServer, Mock as WmMock, ResponseTemplate,
//                        matchers::{method as wm_method, path as wm_path}};
//         // Token endpoint mock via MockHttpClient [PLANNED — engine story]:
//         // records POST call count and returns {"access_token": "...", "expires_in": 3600}
//         let mock_http = MockHttpClient::with_response("bearer_token_ac9", 3600); // [PLANNED]
//         let creds = MockCredentialResolver::new("client_secret_ac9");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let provider = Arc::new(DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         ));
//         // Sensor API mock via wiremock (confirmed dev-dep in prism-spec-engine):
//         // serves GET /items → {"items": [{"id": 1}]}
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("GET"))
//             .and(wm_path("/items"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"items": [{"id": 1}]})),
//             )
//             .mount(&mock_server)
//             .await;
//         let spec = build_test_sensor_spec_token_exchange(); // [PLANNED — engine story helper]
//         // build_test_sensor_spec_token_exchange() sets base_url = mock_server.uri()
//         let table = build_test_table_spec(); // [PLANNED — engine story helper]
//         let context = FetchContext::new(
//             prism_core::OrgSlug::new_unchecked("test-org"),
//             std::collections::HashMap::new(),
//         );
//         // ALLOWLIST REQUIRED: OrgSlug::new_unchecked in this harness must be entered in
//         // crates/prism-core/tests/new_unchecked_audit.rs per CLAUDE.md credential-safety convention.
//         let http_client = reqwest::Client::builder()
//             .timeout(std::time::Duration::from_secs(30))
//             .build()
//             .expect("test client"); // 30s timeout per CLAUDE.md ADR-050; direct construction
//         // (build_http_client_with_timeout is pub(crate) and inaccessible from tests/)
//         // First execute call: cold cache → 1 POST to token endpoint
//         let _ = PipelineExecutor::execute(&spec, &table, &context, &http_client, provider.as_ref())
//             .await.expect("VP-159 AC-9: first execute call must succeed");
//         assert_eq!(mock_http.post_call_count(), 1,
//             "VP-159 AC-9: first execute call must issue exactly 1 token-endpoint POST");
//         // Second execute call: warm cache (TTL 3600s >> 0 elapsed) → 0 additional POSTs
//         let _ = PipelineExecutor::execute(&spec, &table, &context, &http_client, provider.as_ref())
//             .await.expect("VP-159 AC-9: second execute call must succeed");
//         assert_eq!(mock_http.post_call_count(), 1,
//             "VP-159 AC-9: second execute call must use cached token — zero additional \
//              token-endpoint POSTs (BC-2.16.014 P9 execute_impl path; ADR-054 §D4)");
//     }
//
//     // AC-9b (P9-execute_step path; SAP-3 execute_step reachability):
//     // PipelineExecutor::execute_step called twice directly (plugin-runtime entry point per
//     // ADR-054 §D11), same warm DeclarativeHttpAuthProvider [PLANNED], long TTL → exactly 1
//     // token-endpoint POST total across both calls.
//     //
//     // execute_step signature (confirmed: PipelineExecutor::execute_step in prism-spec-engine):
//     //   (step: &FetchStep, spec: &SensorSpec,
//     //    prior_vars: &std::collections::HashMap<String, serde_json::Value>,
//     //    context: &FetchContext, http_client: &reqwest::Client,
//     //    auth_provider: &dyn AuthProvider) -> Result<serde_json::Value, SpecEngineError>
//     //
//     // FetchStep::new(...) — struct-literal is E0639-impossible: FetchStep is #[non_exhaustive]
//     // Confirmed pub fn new signature (spec_parser.rs): name: impl Into<String>,
//     //   method: impl Into<String>, path_template: impl Into<String>,
//     //   body_template: Option<String>, response_path: impl Into<String>,
//     //   pagination_cursor_path: Option<String>, variables_produced: Vec<String>,
//     //   fan_out_batch_size: Option<u32>, pagination: Option<PaginationConfig> -> Self
//     //
//     // [PLANNED — engine story]: DeclarativeHttpAuthProvider, MockHttpClient,
//     //   build_test_sensor_spec_token_exchange
//     #[tokio::test]
//     async fn test_vp159_ac9b_execute_step_path_cache_sharing() {
//         use wiremock::{MockServer, Mock as WmMock, ResponseTemplate,
//                        matchers::{method as wm_method, path as wm_path}};
//         // Token endpoint mock via MockHttpClient [PLANNED — engine story]
//         let mock_http = MockHttpClient::with_response("bearer_token_ac9b", 3600); // [PLANNED]
//         let creds = MockCredentialResolver::new("client_secret_ac9b");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds { ttl_buffer_secs: 30 }); // [PLANNED]
//         let provider = Arc::new(DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             config, Arc::new(mock_http.clone()), Arc::new(creds),
//         ));
//         // Sensor API mock via wiremock — execute_step uses the injected reqwest::Client
//         // (separate from MockHttpClient which is held inside DeclarativeHttpAuthProvider)
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("GET"))
//             .and(wm_path("/items"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"items": [{"id": 1}]})),
//             )
//             .mount(&mock_server)
//             .await;
//         // FetchStep::new — struct-literal is E0639-impossible: FetchStep is #[non_exhaustive]
//         // pub fn new(name, method, path_template, body_template, response_path,
//         //   pagination_cursor_path, variables_produced, fan_out_batch_size, pagination)
//         //   confirmed at crates/prism-spec-engine/src/spec_parser.rs
//         let step = prism_spec_engine::spec_parser::FetchStep::new(
//             "main",
//             "GET",
//             "/items",
//             None,
//             "$.items",
//             None,
//             vec![],
//             None,
//             None,
//         );
//         let spec = build_test_sensor_spec_token_exchange(); // [PLANNED — engine story helper]
//         // build_test_sensor_spec_token_exchange() sets base_url = mock_server.uri()
//         let prior_vars: std::collections::HashMap<String, serde_json::Value> =
//             std::collections::HashMap::new(); // no cross-step variable dependencies
//         let context = FetchContext::new(
//             prism_core::OrgSlug::new_unchecked("test-org"),
//             std::collections::HashMap::new(),
//         );
//         // ALLOWLIST REQUIRED: OrgSlug::new_unchecked in this harness must be entered in
//         // crates/prism-core/tests/new_unchecked_audit.rs per CLAUDE.md credential-safety convention.
//         let http_client = reqwest::Client::builder()
//             .timeout(std::time::Duration::from_secs(30))
//             .build()
//             .expect("test client"); // 30s timeout per CLAUDE.md ADR-050; direct construction
//         // (build_http_client_with_timeout is pub(crate) and inaccessible from tests/)
//         // First execute_step call: cold cache → 1 POST to token endpoint
//         let _ = PipelineExecutor::execute_step(
//             &step, &spec, &prior_vars, &context, &http_client, provider.as_ref(),
//         ).await.expect("VP-159 AC-9b: first execute_step call must succeed");
//         assert_eq!(mock_http.post_call_count(), 1,
//             "VP-159 AC-9b: first execute_step call must issue exactly 1 token-endpoint POST");
//         // Second execute_step call: warm cache (TTL 3600s >> 0 elapsed) → 0 additional POSTs
//         let _ = PipelineExecutor::execute_step(
//             &step, &spec, &prior_vars, &context, &http_client, provider.as_ref(),
//         ).await.expect("VP-159 AC-9b: second execute_step call must succeed");
//         assert_eq!(mock_http.post_call_count(), 1,
//             "VP-159 AC-9b: second execute_step call must use cached token — zero additional \
//              token-endpoint POSTs (BC-2.16.014 P9 execute_step path; ADR-054 §D11; \
//              a mis-wiring leaving acquire_token in execute_step produces 2 POSTs and FAILS here)");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Deterministic | Fixed mock scenarios for each cache state (cold, warm, stale, bypass); no combinatorial generation |
| Proof complexity | Medium | Requires `MockHttpClient` with call-count tracking and mock clock injection; both are standard test-helper patterns in the prism-spec-engine test suite |
| Tool support | Full | `MockCredentialResolver` is confirmed at `crates/prism-spec-engine/src/auth_provider.rs` (test-helpers gate); `MockHttpClient` and mock clock are co-located with `DeclarativeHttpAuthProvider` [PLANNED] implementation in the same story |
| Harness dependencies | Medium (planned) | `DeclarativeHttpAuthProvider`, `AuthAcquisitionConfig`, `ExpiryMode`, `CachedAuthToken`, `MockHttpClient` are all [PLANNED — engine story]; harness is authored in the same Wave-A story as the implementation |
| Estimated proof time | < 1 second | Deterministic async scenarios with mock I/O; no real network, no real clock dependency |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-07-22 | architect (D-1947 Wave-A spec-evolution burst 2) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.6 | wave-a-spec-evolution-fix-burst-9 | 2026-07-22 | architect | F-WASE-P9-MED-002: AC-9b description updated — "struct literal" → `prism_spec_engine::spec_parser::FetchStep::new(name, method, path_template, body_template, response_path, pagination_cursor_path, variables_produced, fan_out_batch_size, pagination)` with note that struct-literal is E0639-impossible from `tests/` (`FetchStep` is `#[non_exhaustive]`; confirmed at `spec_parser.rs`). Harness skeleton: comment "FetchStep struct-literal fields confirmed" → "FetchStep::new(...) — struct-literal is E0639-impossible: FetchStep is #[non_exhaustive]" with full `pub fn new` parameter list confirmed from `spec_parser.rs`; skeleton construction changed from `crate::spec_parser::FetchStep { ... }` struct-literal to `prism_spec_engine::spec_parser::FetchStep::new("main", "GET", "/items", None, "$.items", None, vec![], None, None)` with correct external-crate path. F-WASE-P9-MED-003: AC-9 + AC-9b `http_client` construction updated — `crate::pipeline::build_http_client_with_timeout().expect(...)` replaced with `reqwest::Client::builder().timeout(Duration::from_secs(30)).build().expect("test client")` (direct construction per CLAUDE.md ADR-050; `build_http_client_with_timeout` is `pub(crate)` and inaccessible from `tests/`; confirmed return type is `reqwest::Client` not `Result` — `.expect()` would not compile on the original). "confirmed helper (closed TD-S-PLUGIN-PREREQ-B-005)" mislabel removed. input-hash recomputed to d0f0001 (ADR-054 content changed since v1.5; updated via `compute-input-hash --update`). |
| 1.5 | wave-a-spec-evolution-fix-burst-8 | 2026-07-22 | architect | F-WASE-P8-MED-001: AC-9b added (SAP-3 execute_step reachability for P9). Scope note updated: P9 now explicitly split across AC-9 (execute → execute_impl path) and AC-9b (execute_step direct-call path; confirmed sig: `PipelineExecutor::execute_step`). AC-9 heading narrowed to `(P9-execute_impl path; SAP-3 execute reachability)`; AC-9 first-paragraph closing sentence narrowed to cite execute_impl path only (not execute_step); AC-9 SAP-3 note updated to reference AC-9b as the execute_step complement. AC-9b added: drives `PipelineExecutor::execute_step` twice with the same `DeclarativeHttpAuthProvider` [PLANNED] instance and long-lived TTL (3600s), asserts exactly 1 token-endpoint POST total; a mis-wiring leaving `acquire_token` in `execute_step` produces 2 POSTs and fails this test. `FetchStep` struct-literal fields confirmed from `spec_parser::FetchStep`; `FetchContext::new` confirmed non-exhaustive constructor. Sensor API mock uses wiremock (confirmed dev-dep) alongside `MockHttpClient` [PLANNED] for the token endpoint. Harness skeleton: AC-9 skeleton added (`test_vp159_ac9_execute_impl_path_cache_sharing`); AC-9b skeleton added (`test_vp159_ac9b_execute_step_path_cache_sharing`). §Proof Harness header updated: P9-via-AC-9 → P9-via-AC-9+AC-9b. input-hash recomputed to current frontmatter value at commit time (at-commit-time hash wording per POL-32). |
| 1.4 | wave-a-spec-evolution-fix-burst-7 | 2026-07-22 | architect | F-WASE-P7-MED-001: AC-9 added (SAP-3 executor reachability). AC-2/3/4 verify `get_token()` by direct invocation (isolation; defense-in-depth). AC-9 requires an end-to-end test driving `PipelineExecutor::execute` twice against the same `DeclarativeHttpAuthProvider` instance; the second call must record zero additional token-endpoint POSTs, confirming the cache is reached through the production executor call-path. SAP-3 note added in AC-9 text: both AC-2/3/4 (isolation) and AC-9 (reachability) are required. Production caller: `PipelineExecutor::execute_impl` calls `get_token()` per ADR-054 v0.38 §D4 PipelineExecutor call-site dispatch table and §D11 engine-story wiring rows. P9 reconciliation (BC-2.16.014 v1.5): §Source Contract authoring-source updated P1–P8 → P1–P9 (BC-2.16.014 v1.5); §Property Statement preamble updated to P1–P5, P7, P9; scope note heading updated "P6 and P8 (deferred); P9 (verified via AC-9)" with P9 caller text added; AC-9 heading updated to `(P9; SAP-3 executor reachability)` and body cites BC-2.16.014 P9 explicitly; §Proof Harness Skeleton header comment updated to `BC-2.16.014 v1.5 (P1–P5, P7, P9; ...)`. Verified set: P1–P5, P7, P9 (plus P4-TTL-a/b sub-properties); P6/P8 remain deferred. input-hash recomputed to 043b10a (BC-2.16.014 v1.5 content change). |
| 1.3 | Wave-A fix-burst 5 | 2026-07-22 | architect | F-WASE-P5-MED-001: input-hash trail reconciliation — v1.2 changelog row recorded `3af7dc1` as the post-v1.2 input-hash; frontmatter `f761188` is the authoritative current value (recomputed at D-1953 burst immediately after v1.2 was authored, when ADR-054 v0.37 was edited in that same burst; the 3af7dc1→f761188 transition was not captured in the v1.2 row — v1.2 row left immutable per changelog policy). F-WASE-P5-LOW-001: §Proof Harness Skeleton constructor fixes — `MockCredentialResolver::default()` (7 confirmed sites: AC-1, AC-3, AC-4, AC-5, AC-7a, AC-7b, AC-7c; finding cited 8 — 1 discrepancy, all located sites fixed) rewritten to `MockCredentialResolver::new("test-credential")`; `MockCredentialResolver::with_secret("client_secret_xyz")` (AC-2) and `MockCredentialResolver::with_secret("long_lived_secret")` (AC-6, AC-6b) rewritten to `MockCredentialResolver::new("...")` with identical argument value. All 10 sites resolved using the existing `pub fn new(value: impl Into<String>) -> Self` constructor — no new `MockCredentialResolver` extension required. input-hash trail: f761188 (D-1953 committed) → recomputed to current frontmatter value at commit time (hook-detected drift — ADR-054 or BC-2.16.014 changed since D-1953; updated via `compute-input-hash --update`; see frontmatter for settled value). |
| 1.2 | Wave-A fix-burst 4 | 2026-07-22 | architect | F-WASE-P4-OBS-001: §Proof Harness Skeleton — added skeleton test functions for AC-6 (P4-TTL-a `absolute_utc_string` expiry arithmetic, including AC-6b malformed-RFC-3339 → `AuthAcquisitionFailed` per EC-016-014-003) and AC-7 (P4-TTL-b `relative_seconds` expiry arithmetic, including AC-7b absent `expires_in` → default 1799 per EC-016-014-001 and AC-7c zero `expires_in` → default 1799 per EC-016-014-002). All new symbols marked `[PLANNED — engine story]` per POL-31. F-WASE-P4-OBS-003: §Source Contract P1–P8 authoring-source sentence disambiguated — "P1–P8 are the primary **authoring source**" now explicitly followed by "the verified set is P1–P5, P7 (plus P4-TTL-a/b sub-properties) — see §Property Statement scope note for P6/P8 coverage"; eliminates the false-verified-set reading. input-hash updated to `3af7dc1` (inputs unchanged; hash recomputed after prior edit). |
| 1.1 | D-1947/D-1948 Wave-A fix-burst 1 | 2026-07-22 | architect | F-WASE-P1-MED-001: burst attribution corrected D-1946→D-1947 in Lifecycle table and v1.0 Burst cell (VP-159/VP-INDEX authoring is burst 2, D-1947; BC-2.16.014 authoring is burst 1, D-1946). F-WASE-P1-LOW-002: §Property Statement preamble narrowed from P1–P8 to P1–P5, P7 (P4-TTL-a/b sub-properties); scope note added after P7 for P6 (inherent in acquire_token() contract per AuthProvider trait, verified via AC-5 + error-path assertions in engine implementation story) and P8 (spec-load validation property per BC-2.16.009 Rule 6 / E-SPEC-024, deferred to spec-engine validation story — not a runtime lifecycle invariant of DeclarativeHttpAuthProvider). F-WASE-P1-OBS-002 closure: new_unchecked_audit.rs allowlist-entry note added to AC-5 harness skeleton for OrgSlug::new_unchecked per CLAUDE.md credential-safety convention. |
| 1.0 | D-1947 Wave-A spec-evolution burst 2 | 2026-07-22 | architect | Initial authoring. Authoring source: ADR-054 §D9. BC-2.16.014 P1–P8 all covered (P6 — double-401 → AuthRefreshFailed (E-AUTH-002) — is inherent in acquire_token() contract per AuthProvider trait; verified via AC-5 + error-path assertions in the implementation story). DRIFT-D849-002 folded: StaticCookieAuthProvider zero-HTTP is structural (no reqwest::Client field, confirmed in codebase), covered by BC-2.01.017 §P1 (INV-COOKIE-001); VP-159 covers the equivalent invariant for DeclarativeHttpAuthProvider [PLANNED]. All DeclarativeHttpAuthProvider / CachedAuthToken / AuthAcquisitionConfig / ExpiryMode / MockHttpClient symbols marked [PLANNED — engine story] per POL-31 (crates/prism-spec-engine/src/auth/ directory does not exist at authoring time). Existing verified symbols: AuthProvider trait, CredentialResolver trait, MockCredentialResolver, SpecEngineError::AuthAcquisitionFailed (E-AUTH-001), SpecEngineError::AuthRefreshFailed (E-AUTH-002). source_invariant: DI-012 (workspace canonical, domain-spec/invariants.md); INV-014-003 (BC-local credential-opacity invariant) cited in §Source Contract body prose only per VP-INDEX source_invariant schema convention. |
