---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-07-22T00:00:00Z
phase: wave-a
inputs:
  - .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md
  - .factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md
input-hash: "0fcf6c0"
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
network-isolation and cache-lifecycle invariants (ADR-054 §D9; BC-2.16.014 P1–P5, P7; TTL sub-properties P4-TTL-a/b):

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

> **Scope note — P6 and P8:** P6 (double-401 → `AuthRefreshFailed`, E-AUTH-002) is inherent in the
> `acquire_token()` contract per the confirmed `AuthProvider` trait and is verified via AC-5 +
> error-path assertions in the engine implementation story (not directly asserted by VP-159's
> mock-HTTP harness). P8 (`base_url` env-var interpolation obeys BC-2.16.009 Rule 6 / E-SPEC-024;
> `token_path` is a literal relative path and does not undergo env-var interpolation) is a spec-load
> validation property, not a runtime lifecycle invariant of `DeclarativeHttpAuthProvider`; deferred
> to the spec-engine validation story's error-path assertions.

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

## Source Contract

- **BC:** BC-2.16.014 (`DeclarativeHttpAuthProvider` Token Lifecycle) — postconditions P1–P8
  are the primary authoring source for this VP. INV-014-003 (BC-local invariant:
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
// BC: BC-2.16.014 (P1–P5, P7; P4-TTL-a/b sub-properties; P6/P8 deferred — see §Property Statement scope note); ADR: ADR-054 §D9; source_invariant: DI-012
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
//         let creds = MockCredentialResolver::default();
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
//         let creds = MockCredentialResolver::with_secret("client_secret_xyz");
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
//         let creds = MockCredentialResolver::default();
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
//         let creds = MockCredentialResolver::default();
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
//         let creds = MockCredentialResolver::default();
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
| 1.1 | D-1947/D-1948 Wave-A fix-burst 1 | 2026-07-22 | architect | F-WASE-P1-MED-001: burst attribution corrected D-1946→D-1947 in Lifecycle table and v1.0 Burst cell (VP-159/VP-INDEX authoring is burst 2, D-1947; BC-2.16.014 authoring is burst 1, D-1946). F-WASE-P1-LOW-002: §Property Statement preamble narrowed from P1–P8 to P1–P5, P7 (P4-TTL-a/b sub-properties); scope note added after P7 for P6 (inherent in acquire_token() contract per AuthProvider trait, verified via AC-5 + error-path assertions in engine implementation story) and P8 (spec-load validation property per BC-2.16.009 Rule 6 / E-SPEC-024, deferred to spec-engine validation story — not a runtime lifecycle invariant of DeclarativeHttpAuthProvider). F-WASE-P1-OBS-002 closure: new_unchecked_audit.rs allowlist-entry note added to AC-5 harness skeleton for OrgSlug::new_unchecked per CLAUDE.md credential-safety convention. |
| 1.0 | D-1947 Wave-A spec-evolution burst 2 | 2026-07-22 | architect | Initial authoring. Authoring source: ADR-054 §D9. BC-2.16.014 P1–P8 all covered (P6 — double-401 → AuthRefreshFailed (E-AUTH-002) — is inherent in acquire_token() contract per AuthProvider trait; verified via AC-5 + error-path assertions in the implementation story). DRIFT-D849-002 folded: StaticCookieAuthProvider zero-HTTP is structural (no reqwest::Client field, confirmed in codebase), covered by BC-2.01.017 §P1 (INV-COOKIE-001); VP-159 covers the equivalent invariant for DeclarativeHttpAuthProvider [PLANNED]. All DeclarativeHttpAuthProvider / CachedAuthToken / AuthAcquisitionConfig / ExpiryMode / MockHttpClient symbols marked [PLANNED — engine story] per POL-31 (crates/prism-spec-engine/src/auth/ directory does not exist at authoring time). Existing verified symbols: AuthProvider trait, CredentialResolver trait, MockCredentialResolver, SpecEngineError::AuthAcquisitionFailed (E-AUTH-001), SpecEngineError::AuthRefreshFailed (E-AUTH-002). source_invariant: DI-012 (workspace canonical, domain-spec/invariants.md); INV-014-003 (BC-local credential-opacity invariant) cited in §Source Contract body prose only per VP-INDEX source_invariant schema convention. |
