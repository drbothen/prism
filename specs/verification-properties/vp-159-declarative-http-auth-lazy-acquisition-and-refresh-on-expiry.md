---
document_type: verification-property
level: L4
version: "1.26"
status: draft
producer: architect
timestamp: 2026-07-22T00:00:00Z
phase: wave-a
inputs:
  - .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md
  - .factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md
input-hash: "f702703"
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
modified: "2026-07-24"
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
   issued when `unix_now() < cached.expires_at` AND `!cached.token.is_empty()` return the
   cached token and issue ZERO HTTP requests.

4. **P4 — stale or poisoned `get_token()` — exactly one HTTP POST:** A `get_token()` [PLANNED]
   call when `unix_now() >= cached.expires_at` (TTL elapsed) OR `cached.token.is_empty()`
   (poisoned cache entry — stored token is the empty string) issues exactly ONE HTTP POST and
   refreshes the cache atomically. Both triggers are equivalent re-acquisition paths
   (BC-2.16.014 P4).

5. **P5 — `acquire_token()` — always exactly one HTTP POST (cache bypass):** The `acquire_token()`
   method (satisfying the `AuthProvider` trait confirmed in
   `crates/prism-spec-engine/src/auth_provider.rs`) ALWAYS issues exactly ONE HTTP POST,
   regardless of cache state. It bypasses the TTL check and replaces the cached token on success.

**TTL arithmetic invariants:**

6. **P4-TTL-a — `absolute_utc_string` expiry mode:** When `ExpiryMode` [PLANNED] is
   `AbsoluteUtcString`, `expires_at = expiry_str.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64).saturating_sub(ttl_buffer_secs)` (lenient chrono relaxed `FromStr` per ADR-054 §D4 v0.51 — accepts both `T`-separator ISO-8601 and space-separated `"YYYY-MM-DD HH:MM:SS.ffffff+HH:MM"`; rejects only if even relaxed parse fails → E-AUTH-001). Kill condition for strict-parse impl: fixture `"2099-01-01 00:00:00.000000+00:00"` (space-separated) parses to the SAME epoch as `"2099-01-01T00:00:00Z"` under relaxed `FromStr`; a strict `parse_from_rfc3339`-only impl rejects it → `E-AUTH-001`, FAILING AC-6c's continuation assertion.

7. **P4-TTL-b — `relative_seconds` expiry mode:** When `ExpiryMode` [PLANNED] is
   `RelativeSeconds`, `expires_at = unix_now() + expires_in.saturating_sub(ttl_buffer_secs)`,
   where `expires_in` defaults to 1799 when absent or zero (matching the
   crowdstrike-oauth2 plugin's arithmetic; `.max(1)` is omitted because the absent/zero
   default is already 1799 — dead code per ADR-054 §D9 note).

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

> **Scope note — auth-type harness split (F-WASE-P55-OBS-001):** ACs 1–9b drive the
> `token_exchange` path for cache-lifecycle invariants (P1–P5, P7, P9); the `oauth2_client_credentials`
> path has identical cache-lifecycle behavior and is verified for form/response shaping via
> BC-2.16.014 TV-1/TV-5/TV-6/TV-11 + EC-016-014-016 test vectors. AC-7d is the sole harness in
> VP-159 that drives the `oauth2_client_credentials` path — it exclusively tests the RU-Q2 lenient
> parse of `$.expires_in` (ADR-054 §D2), which is the one behavioral difference between the two
> auth-type paths at the `DeclarativeHttpAuthProvider` level. The `token_exchange`
> `relative_seconds` path uses plain u64 (ADR-054 §D4 step 4 L308; no lenient-parse qualifier);
> extending lenient parsing to `token_exchange` would require a BC-2.16.014 §P2 amendment and new
> TV — no such amendment is in scope (F-WASE-P55-MED-001 adjudication, branch (a)).

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

The integration test harness uses `wiremock` (confirmed dev-dependency in
`crates/prism-spec-engine`) to intercept all outbound HTTP calls at the token endpoint by
setting `token_url = wiremock_server.uri() + token_path` in each test — no `MockHttpClient`
injection is needed in the production constructor. Tests that require deterministic TTL
expiry control use `DeclarativeHttpAuthProvider::new_for_test` [PLANNED — engine story;
`#[cfg(any(test, feature = "test-helpers"))]`], which accepts a
`now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` backed by an `Arc<AtomicU64>` that the test
advances directly. `MockCredentialResolver` (confirmed in
`crates/prism-spec-engine/src/auth_provider.rs`, gated
`#[cfg(any(test, feature = "test-helpers"))]`) is used for credential injection. The harness
asserts:

- **AC-1 (P1):** Constructing `DeclarativeHttpAuthProvider::new()` [PLANNED] with a valid
  `token_url` (pointing at a wiremock server), `AuthAcquisitionConfig` [PLANNED], and
  `MockCredentialResolver` results in zero requests received by the wiremock server.

- **AC-2 (P2):** After construction, calling `get_token()` [PLANNED] once on a cold cache
  results in exactly one POST request to the wiremock token endpoint. The returned token
  matches the mock response body.

- **AC-3 (P3):** Calling `get_token()` [PLANNED] again when `unix_now() < cached.expires_at`
  AND `!cached.token.is_empty()` results in zero additional requests to the wiremock token
  endpoint. The same token string is returned.

- **AC-4 (P4):** Advancing the mock clock (`Arc<AtomicU64>` passed via `now_fn` to
  `new_for_test` [PLANNED]) past `expires_at` and calling `get_token()` [PLANNED] results in
  exactly one additional POST request to the wiremock token endpoint. The refreshed token is
  returned.

- **AC-4b (P4 — empty-token trigger):** Construct via `DeclarativeHttpAuthProvider::new_for_test`
  [PLANNED — engine story] with `now_fn` backed by `Arc<AtomicU64>` pinned at
  `base_time = 1_700_000_000u64` (same pinned-clock pattern as AC-4, AC-6, AC-7). Seed
  the provider's `ArcSwap` cache with a poisoned entry (`CachedAuthToken::new("".to_string(), base_time + 86_400)`) [PLANNED] via `DeclarativeHttpAuthProvider::seed_cache_for_test`
  [PLANNED — engine story; `#[cfg(any(test, feature = "test-helpers"))]`] — a test seam that
  writes directly to the `ArcSwap<Option<CachedAuthToken>>` internal field. With the clock
  pinned at `base_time`, the TTL predicate `(self.now_fn)() >= expires_at` evaluates to
  `1_700_000_000 >= 1_700_086_400` = FALSE — the TTL check does NOT fire; only
  `cached.token.is_empty()` triggers re-acquisition, exercising that branch independently of
  TTL expiry. **Kill condition (mutation-thinking):** a TTL-only implementation that ignores
  `is_empty()` would return the cached empty string and FAIL `== "fresh_token_4b"` — the
  assertion is load-bearing precisely because the pinned clock keeps the TTL path inactive.
  The mock token server is configured to return a fresh non-empty token `"fresh_token_4b"`.
  After calling `get_token(&sensor_spec, &org_slug)` [PLANNED], assert:
  (1) exactly one POST request recorded by the wiremock server (`post_count == 1`), and
  (2) the returned token string equals `"fresh_token_4b"` — not the cached empty string
  (BC-2.16.014 P4 `is_empty()` branch).

- **AC-5 (P5):** Calling `acquire_token()` on a warm-cache provider results in exactly one
  POST request to the wiremock token endpoint (bypasses TTL check regardless of cache state).

- **AC-6 (P4-TTL-a):** For `ExpiryMode::AbsoluteUtcString` [PLANNED], the computed `expires_at`
  equals `expiry_str.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64).saturating_sub(ttl_buffer_secs)`
  (lenient chrono relaxed `FromStr` per ADR-054 §D4 v0.51). Fixture uses RFC-3339 `T`-form
  `"2099-01-01T00:00:00Z"` (see harness). See AC-6c for the space-separated form assertion.

- **AC-6c (P4-TTL-a — space-separated fixture):** For `ExpiryMode::AbsoluteUtcString` [PLANNED],
  the space-separated value `"2099-01-01 00:00:00.000000+00:00"` parses to the SAME Unix epoch as
  `"2099-01-01T00:00:00Z"` (4_070_908_800). **Kill condition:** a strict `parse_from_rfc3339`-only
  impl rejects the space-separated form and returns `E-AUTH-001` instead of proceeding; the
  wiremock mock returns a long-lived TTL and a second `get_token()` call should serve from cache —
  if the first call fails with `E-AUTH-001`, the cache is never populated and the test FAILS.
  The two-POST count assertion (cold → stale → re-acquire) still applies; the expiry arithmetic
  must match the `T`-form epoch (both forms represent the same instant).

- **AC-7 (P4-TTL-b):** For `ExpiryMode::RelativeSeconds` [PLANNED], the computed `expires_at`
  equals `unix_now() + expires_in.saturating_sub(ttl_buffer_secs)`, with `expires_in = 1799`
  when the response field is absent or zero.

- **AC-7d (P2 — `oauth2_client_credentials` string-typed `$.expires_in`, RU-Q2):** For
  `oauth2_client_credentials` [PLANNED], when the token endpoint response returns `$.expires_in` as
  a **JSON string** (e.g., `{"access_token": "tok-str-exp", "expires_in": "3599"}`) rather than a
  JSON number, the engine must leniently parse `"3599"` → `u64 3599` and compute
  `expires_at = unix_now() + 3599.saturating_sub(30) = unix_now() + 3569`. **Kill condition:** a
  u64-only (non-lenient) `as_u64()` parser returns `None` for a string value, triggering the
  default-1799 path; `expires_at` would then equal `unix_now() + 1769` instead of
  `unix_now() + 3569`. Advancing the clock by 1800s (past the 1769 default but NOT past the 3569
  correct value) and calling `get_token()` again would trigger an extra POST if the default-1799
  path was taken — the test asserts exactly zero additional POSTs at that clock position, exposing
  the lenient-parse defect. The wiremock server is configured to return `"expires_in": "3599"`
  (string-typed). Clock seam via `now_fn` / `new_for_test` [PLANNED]. Harness uses
  `build_test_sensor_spec_oauth2()` [PLANNED — engine story] and
  `AuthAcquisitionConfig::new_oauth2("/oauth2/token", ttl_buffer_secs)` [PLANNED — engine story]
  (no `ExpiryMode` parameter — `oauth2_client_credentials` TTL is always relative seconds via
  `$.expires_in`; no `expiry_mode` selector field per ADR-054 §D3). Scope: RU-Q2 lenient
  deserialization is scoped EXCLUSIVELY to `oauth2_client_credentials` `$.expires_in` (ADR-054 §D2;
  BC-2.16.014 §P2 oauth2 arm; EC-016-014-016). The `token_exchange` `relative_seconds` path uses
  plain u64 (ADR-054 §D4 step 4 L308) — no lenient parse applies there.

- **AC-8 (P7):** After a successful `get_token()` [PLANNED] call with distinct mock credential
  (`"mock_client_secret_P7"`) and mock token (`"opaque_bearer_P7"`) values, the returned token
  equals `"opaque_bearer_P7"` (not the credential): `assert_eq!(returned, "opaque_bearer_P7")` AND
  `assert_ne!(returned, "mock_client_secret_P7")`. Structural enforcement: `CachedAuthToken` [PLANNED]
  is `#[non_exhaustive]` and its sole constructor `pub fn new(token: String, expires_at: u64) -> Self`
  accepts no credential parameter — adding a credential field requires a constructor change, making
  the omission architecturally documented. The runtime inequality assertion is the load-bearing check
  that `get_token()` returns the acquired token, never the resolved credential (BC-2.16.014 P7,
  AD-017, INV-014-003).

- **AC-9 (P9-execute_impl path; SAP-3 execute reachability):** An end-to-end test drives
  `PipelineExecutor::execute` (confirmed in `pipeline.rs`; its `get_token()` cache-aware wiring is
  [PLANNED — engine story] per ADR-054 §D4) with a `DeclarativeHttpAuthProvider`
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
  `PipelineExecutor::execute_step` (confirmed in `pipeline.rs`; its `get_token()` cache-aware
  wiring is [PLANNED — engine story] per ADR-054 §D11) directly — the plugin-runtime entry
  point per ADR-054 v0.38 §D11. A `FetchStep` is constructed via
  `prism_spec_engine::spec_parser::FetchStep::new(name, method, path_template, body_template,
  response_path, pagination_cursor_path, variables_produced, fan_out_batch_size, pagination)`
  (confirmed `pub fn new` signature at `crates/prism-spec-engine/src/spec_parser.rs`; struct-literal
  construction is E0639-impossible from `tests/` because `FetchStep` is `#[non_exhaustive]`). Two
  consecutive `PipelineExecutor::execute_step` calls are
  issued with the same `DeclarativeHttpAuthProvider` [PLANNED] instance and
  `prior_vars: HashMap::new()` (no cross-step variable
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
  client (the sensor API reqwest::Client passed into execute_step, distinct from the internally-
  constructed ADR-050 client held inside `DeclarativeHttpAuthProvider`); both the token endpoint
  (POST `/oauth/token`) and the sensor API mock endpoint (GET `/items` → `{"items": [{"id": 1}]}`)
  are served by a single wiremock (confirmed dev-dependency in `prism-spec-engine`) server — no
  separate `MockHttpClient` needed. `FetchContext::new` is the confirmed non-exhaustive constructor (`OrgSlug`,
  `HashMap<String, String>`).

## Source Contract

- **BC:** BC-2.16.014 (`DeclarativeHttpAuthProvider` Token Lifecycle) v1.18 — postconditions P1–P9
  (BC-2.16.014 v1.18) are the primary **authoring source** for this VP; the verified set is
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
| integration_test | `wiremock` (confirmed dev-dep in `crates/prism-spec-engine`) for HTTP interception — token endpoint set to `wiremock_server.uri() + token_path`; `MockCredentialResolver` (confirmed in `crates/prism-spec-engine/src/auth_provider.rs`) for credential injection; `Arc<AtomicU64>` clock via `now_fn` seam in `DeclarativeHttpAuthProvider::new_for_test` [PLANNED — engine story; `#[cfg(any(test, feature = "test-helpers"))]`] for deterministic TTL expiry — no `MockHttpClient` needed | Deterministic — fixed scenario sequences covering each cache state; not combinatorial | Cold → warm → stale → refresh state machine; both ExpiryMode variants; cache-bypass via acquire_token; credential-opacity structural assertion |

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
// Method: integration_test (wiremock for HTTP interception; now_fn clock seam for TTL control)
// Target module: prism-spec-engine
// Target path: crates/prism-spec-engine/src/auth/declarative.rs [PLANNED — engine story]
// BC: BC-2.16.014 v1.18 (P1–P5, P7, P9; P4-TTL-a/b sub-properties; P6/P8 deferred, P9-via-AC-9+AC-9b verified — see §Property Statement scope note); ADR: ADR-054 §D9; source_invariant: DI-012
//
// ALL DeclarativeHttpAuthProvider / CachedAuthToken / AuthAcquisitionConfig / ExpiryMode /
// DeclarativeHttpAuthProvider::new_for_test (cfg(any(test, feature = "test-helpers")))
// symbols below are [PLANNED — engine story].
// MockHttpClient is NOT used; wiremock (confirmed dev-dep in prism-spec-engine) handles
// HTTP interception by pointing token_url at the wiremock server. Clock seam (now_fn:
// Arc<dyn Fn() -> u64 + Send + Sync>) handles deterministic TTL expiry testing.
//
// Confirmed existing symbols used:
//   AuthProvider trait:        crates/prism-spec-engine/src/auth_provider.rs
//   MockCredentialResolver:    crates/prism-spec-engine/src/auth_provider.rs
//                              (cfg(any(test, feature = "test-helpers")))
//   SpecEngineError::AuthAcquisitionFailed: crates/prism-spec-engine/src/error.rs (E-AUTH-001)
//   SpecEngineError::AuthRefreshFailed:     crates/prism-spec-engine/src/error.rs (E-AUTH-002)
//   wiremock:                  dev-dep in crates/prism-spec-engine/Cargo.toml
//   ArcSwap:                   arc_swap crate (external)

// #[cfg(test)]
// mod vp159_tests {
//     use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
//     use wiremock::{MockServer, Mock as WmMock, ResponseTemplate,
//                    matchers::{method as wm_method, path as wm_path}};
//     use prism_spec_engine::auth::declarative::{
//         DeclarativeHttpAuthProvider,  // [PLANNED]
//         AuthAcquisitionConfig,        // [PLANNED]
//         ExpiryMode,                   // [PLANNED]
//         CachedAuthToken,              // [PLANNED]
//     };
//     use prism_spec_engine::auth_provider::{AuthProvider, MockCredentialResolver};
//
//     fn base_config(token_path: &str, expiry_mode: ExpiryMode, ttl_buffer_secs: u64) -> AuthAcquisitionConfig { // [PLANNED]
//         // AuthAcquisitionConfig::new [PLANNED] — replaces struct-literal + ..Default::default()
//         // (E0639-impossible from external tests/ crate under #[non_exhaustive]; F-WASE-P38-MED-001).
//         AuthAcquisitionConfig::new(token_path, expiry_mode, ttl_buffer_secs)  // [PLANNED]
//     }
//
//     // AC-1 (P1): zero network calls at construction
//     // wiremock records all received requests — zero at construction proves P1.
//     #[tokio::test]
//     async fn test_vp159_ac1_zero_network_at_construction() {
//         let mock_server = MockServer::start().await;  // confirmed dev-dep
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "tok", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/token", mock_server.uri());  // token endpoint → wiremock
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let _provider = DeclarativeHttpAuthProvider::new(  // [PLANNED] 3-arg: token_url + config + creds
//             token_url, config, Arc::new(creds),
//         );
//         let received = mock_server.received_requests().await.unwrap();
//         assert_eq!(received.len(), 0,
//             "VP-159 AC-1: construction must make zero network calls (BC-2.16.014 P1)");
//     }
//
//     // AC-2 (P2): cold get_token → exactly one HTTP POST
//     #[tokio::test]
//     async fn test_vp159_ac2_cold_cache_one_post() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_token_abc", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("client_secret_xyz");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _token = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED]
//             .expect("VP-159 AC-2: cold get_token must succeed");
//         let post_count = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(post_count, 1,
//             "VP-159 AC-2: cold get_token must issue exactly one HTTP POST (BC-2.16.014 P2)");
//     }
//
//     // AC-3 (P3): warm get_token → zero additional HTTP calls
//     #[tokio::test]
//     async fn test_vp159_ac3_warm_cache_zero_post() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_token_abc", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("first call");  // [PLANNED] warms cache
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("second call");  // [PLANNED]
//         let posts_after_second = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_second, posts_after_warm,
//             "VP-159 AC-3: warm get_token must make zero additional HTTP POSTs (BC-2.16.014 P3)");
//     }
//
//     // AC-4 (P4): stale get_token → exactly one additional HTTP POST
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     #[tokio::test]
//     async fn test_vp159_ac4_stale_cache_one_post() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_token_abc", "expires_in": 60})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 0); // [PLANNED]
//         // Mock clock: base_time = 1_700_000_000; expires_at = base_time + 60.saturating_sub(0) = base_time + 60
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("cold call");  // [PLANNED] warms cache
//         // Advance clock past expires_at: base_time + 120 > base_time + 60
//         now_secs.fetch_add(120, Ordering::SeqCst);
//         let posts_before_stale_refresh = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("stale call");  // [PLANNED]
//         let posts_after_stale_refresh = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_stale_refresh, posts_before_stale_refresh + 1,
//             "VP-159 AC-4: stale get_token must issue exactly one HTTP POST (BC-2.16.014 P4)");
//     }
//
//     // AC-4b (P4 — empty-token trigger): poisoned cache (empty-string token, valid TTL) →
//     // exactly one HTTP POST + returned token is the fresh one (not the empty cached string).
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     //   pinned at base_time = 1_700_000_000u64 (same pattern as AC-4, AC-6, AC-7). Cache seam:
//     //   DeclarativeHttpAuthProvider::seed_cache_for_test [PLANNED — engine story;
//     //   cfg(any(test, feature = "test-helpers"))] writes directly to the
//     //   ArcSwap<Option<CachedAuthToken>> internal field. The production constructor never
//     //   produces an empty-token entry; direct injection is the only test path for the
//     //   is_empty() branch.
//     //
//     // Kill condition (mutation-thinking): with pinned clock now_fn() = base_time =
//     //   1_700_000_000 and expires_at = base_time + 86_400 = 1_700_086_400, the TTL
//     //   predicate (self.now_fn)() >= expires_at evaluates to 1_700_000_000 >= 1_700_086_400
//     //   = FALSE. Only cached.token.is_empty() can trigger re-acquisition. A TTL-only
//     //   implementation (ignoring is_empty()) returns the cached empty string and
//     //   FAILS == "fresh_token_4b" — the assertion is load-bearing because the pinned
//     //   clock keeps the TTL path inactive (BC-2.16.014 P4).
//     #[tokio::test]
//     async fn test_vp159_ac4b_empty_token_reacquisition() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "fresh_token_4b", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         // Pinned clock: base_time = 1_700_000_000; now_fn always returns base_time.
//         // expires_at = base_time + 86_400 = 1_700_086_400 (genuinely far future relative to
//         // pinned clock; TTL predicate 1_700_000_000 >= 1_700_086_400 is always FALSE here).
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         // Seed a poisoned-cache entry: empty token string, expires_at = base_time + 86_400.
//         // TTL predicate (1_700_000_000 >= 1_700_086_400) = FALSE → only is_empty() triggers
//         // re-acquisition; a TTL-only implementation would return "" and fail the assertion.
//         let expires_at: u64 = base_time + 86_400;
//         // CachedAuthToken::new [PLANNED] — replaces struct-literal
//         // (E0639-impossible from external tests/ crate under #[non_exhaustive]; F-WASE-P38-MED-001).
//         let poisoned_entry = CachedAuthToken::new("".to_string(), expires_at);  // [PLANNED] empty-string token — poisoned-cache case (BC-2.16.014 P4)
//         // seed_cache_for_test [PLANNED — engine story; cfg(any(test, feature = "test-helpers"))]
//         // writes poisoned_entry into the provider's ArcSwap<Option<CachedAuthToken>> directly.
//         provider.seed_cache_for_test(Some(poisoned_entry));  // [PLANNED]
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let token = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED]
//             .expect("VP-159 AC-4b: get_token with poisoned cache (empty token, valid TTL) must succeed");
//         let post_count = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(post_count, 1,
//             "VP-159 AC-4b: empty-token cache must trigger exactly one re-acquisition POST \
//              (BC-2.16.014 P4 is_empty() branch; independent of TTL; pinned clock ensures \
//              TTL path is inactive)");
//         assert_eq!(token, "fresh_token_4b",
//             "VP-159 AC-4b: get_token must return the freshly-acquired token, not the cached empty string \
//              (BC-2.16.014 P4)");
//     }
//
//     // AC-5 (P5): acquire_token bypasses cache → exactly one HTTP POST
//     #[tokio::test]
//     async fn test_vp159_ac5_acquire_token_cache_bypass() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "acquired_token", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("warm cache");  // [PLANNED]
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         // acquire_token() is the AuthProvider::acquire_token() method (confirmed symbol)
//         let _token = provider.acquire_token(&sensor_spec, &org_slug).await
//             .expect("VP-159 AC-5: acquire_token must succeed even on warm cache");
//         let posts_after_force_refresh = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_force_refresh, posts_after_warm + 1,
//             "VP-159 AC-5: acquire_token must issue exactly one HTTP POST regardless of cache (BC-2.16.014 P5)");
//     }
//
//     // AC-6 (P4-TTL-a): absolute_utc_string expiry arithmetic (T-form fixture)
//     // Formula: expires_at = expiry_str.parse::<DateTime<FixedOffset>>()..timestamp()..saturating_sub(ttl_buffer_secs)
//     //   (lenient chrono relaxed FromStr — ADR-054 §D4 v0.51 RU-Q1)
//     // "2099-01-01T00:00:00Z" ≈ Unix 4_070_908_800; ttl_buffer=30 → expires_at ≈ 4_070_908_770
//     // For space-separated form see AC-6c below.
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     #[tokio::test]
//     async fn test_vp159_ac6_absolute_utc_expiry_ttl_arithmetic() {
//         let expiry_utc = "2099-01-01T00:00:00Z";
//         let expiry_unix: u64 = 4_070_908_800;   // "2099-01-01T00:00:00Z".parse::<DateTime<FixedOffset>>()..timestamp()
//         let ttl_buffer_secs: u64 = 30;
//         let expires_at = expiry_unix - ttl_buffer_secs;  // = 4_070_908_770
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/api/v1/access_token/"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({
//                         "success": true,
//                         "data": {
//                             "access_token": "arm-tok",
//                             "expiration_utc": expiry_utc
//                         }
//                     })),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/api/v1/access_token/", mock_server.uri());
//         let creds = MockCredentialResolver::new("long_lived_secret");
//         // AuthAcquisitionConfig::new_token_exchange [PLANNED] — replaces struct-literal
//         // (E0639-impossible from external tests/ crate under #[non_exhaustive]; F-WASE-P38-MED-001).
//         let config = AuthAcquisitionConfig::new_token_exchange(  // [PLANNED]
//             "/api/v1/access_token/",
//             "secret_key",
//             "data.access_token",
//             "data.expiration_utc",
//             ExpiryMode::AbsoluteUtcString,  // [PLANNED]
//             ttl_buffer_secs,
//         );
//         // Mock clock starts at 0 (well before expires_at = 4_070_908_770)
//         let now_secs = Arc::new(AtomicU64::new(0u64));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         // Phase 1: warms cache (now = 0, expires_at = 4_070_908_770 → warm)
//         let _ = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED]
//             .expect("VP-159 AC-6: absolute_utc_string get_token must succeed on well-formed expiry");
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         // Phase 2: advance clock to just before expires_at (expires_at - 10) → still warm
//         now_secs.store(expires_at - 10, Ordering::SeqCst);
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm pre-expiry");  // [PLANNED]
//         let posts_still_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_still_warm, posts_after_warm,
//             "VP-159 AC-6: clock before expires_at → cache valid, zero additional HTTP POSTs \
//              (BC-2.16.014 P3/P4-TTL-a)");
//         // Phase 3: advance clock past expires_at (expires_at + 10) → stale → one more POST
//         now_secs.store(expires_at + 10, Ordering::SeqCst);
//         let posts_before_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("stale — re-acquisition");  // [PLANNED]
//         let posts_after_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_stale, posts_before_stale + 1,
//             "VP-159 AC-6: clock past expires_at → one HTTP POST re-acquisition \
//              (BC-2.16.014 P4 / P4-TTL-a: expires_at = expiry_str.parse::<DateTime<FixedOffset>>()..timestamp()..saturating_sub(ttl_buffer_secs))");
//     }
//
//     // AC-6c (P4-TTL-a — space-separated fixture): "2099-01-01 00:00:00.000000+00:00" parses to the
//     // SAME epoch as "2099-01-01T00:00:00Z" (4_070_908_800) under lenient chrono relaxed FromStr.
//     // Kill condition: strict parse_from_rfc3339-only impl rejects the space-separated form →
//     // E-AUTH-001 on first get_token(); the cache is never populated; a second get_token() call
//     // makes a SECOND POST (cache miss) instead of returning from cache — post_count == 2 not 1.
//     // (ADR-054 §D4 v0.51 RU-Q1 adjudication)
//     #[tokio::test]
//     async fn test_vp159_ac6c_space_separated_expiry_parses_to_same_epoch() {
//         let expiry_space = "2099-01-01 00:00:00.000000+00:00";  // space-separated UTC — lenient form
//         let expiry_unix: u64 = 4_070_908_800;                   // same epoch as "2099-01-01T00:00:00Z"
//         let ttl_buffer_secs: u64 = 30;
//         let expires_at = expiry_unix - ttl_buffer_secs;          // = 4_070_908_770
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/api/v1/access_token/"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({
//                         "success": true,
//                         "data": {
//                             "access_token": "arm-tok-space",
//                             "expiration_utc": expiry_space  // space-separated UTC string
//                         }
//                     })),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/api/v1/access_token/", mock_server.uri());
//         let creds = MockCredentialResolver::new("long_lived_secret");
//         let config = AuthAcquisitionConfig::new_token_exchange(  // [PLANNED]
//             "/api/v1/access_token/",
//             "secret_key",
//             "data.access_token",
//             "data.expiration_utc",
//             ExpiryMode::AbsoluteUtcString,  // [PLANNED]
//             ttl_buffer_secs,
//         );
//         let now_secs = Arc::new(AtomicU64::new(0u64));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         // Phase 1: cold call — lenient parse must succeed, not E-AUTH-001
//         let tok = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED]
//             .expect("VP-159 AC-6c: space-separated expiry must succeed with lenient parse \
//                      (strict parse_from_rfc3339 would reject this form → E-AUTH-001)");
//         assert_eq!(tok, "arm-tok-space",
//             "VP-159 AC-6c: returned token must match mock response");
//         let posts_after_cold = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_cold, 1, "VP-159 AC-6c: cold call must issue exactly 1 POST");
//         // Phase 2: second call at clock=0 (well before expires_at = 4_070_908_770) → warm cache
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm");  // [PLANNED]
//         let posts_after_second = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_second, posts_after_cold,
//             "VP-159 AC-6c: warm cache must issue 0 additional POSTs — confirms space-separated \
//              expiry was parsed correctly to expires_at={} (not defaulted or errored)", expires_at);
//     }
//
//     // AC-6b (EC-016-014-003): malformed expiry string → AuthAcquisitionFailed (E-AUTH-001)
//     // No token cached when parse fails — acquire_token returns Err immediately (BC-2.16.014 P4-TTL-a).
//     #[tokio::test]
//     async fn test_vp159_ac6b_malformed_rfc3339_expiry_returns_auth_acquisition_failed() {
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/api/v1/access_token/"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({
//                         "success": true,
//                         "data": {
//                             "access_token": "arm-tok",
//                             "expiration_utc": "not-a-date"  // malformed RFC-3339 value — parse must fail
//                         }
//                     })),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/api/v1/access_token/", mock_server.uri());
//         let creds = MockCredentialResolver::new("long_lived_secret");
//         // AuthAcquisitionConfig::new_token_exchange [PLANNED] — replaces struct-literal
//         // (E0639-impossible from external tests/ crate under #[non_exhaustive]; F-WASE-P38-MED-001).
//         let config = AuthAcquisitionConfig::new_token_exchange(  // [PLANNED]
//             "/api/v1/access_token/",
//             "secret_key",
//             "data.access_token",
//             "data.expiration_utc",
//             ExpiryMode::AbsoluteUtcString,  // [PLANNED]
//             30,
//         );
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let result = provider.get_token(&sensor_spec, &org_slug).await;  // [PLANNED]
//         assert!(
//             matches!(result, Err(SpecEngineError::AuthAcquisitionFailed { .. })),
//             "VP-159 AC-6b: malformed RFC-3339 expiry must return AuthAcquisitionFailed \
//              (E-AUTH-001; EC-016-014-003; BC-2.16.014 P4-TTL-a)"
//         );
//     }
//
//     // AC-7 (P4-TTL-b): relative_seconds expiry arithmetic — verify formula + absent/zero → default 1799
//     // Formula: expires_at = (self.now_fn)() + expires_in.saturating_sub(ttl_buffer_secs)
//     //          where expires_in defaults to 1799 when absent or zero (EC-016-014-001 / EC-016-014-002)
//     // Sub-case 7a: normal expires_in (3600s) — verify clock-based re-acquisition timing
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     #[tokio::test]
//     async fn test_vp159_ac7_relative_seconds_expiry_ttl_arithmetic() {
//         let expires_in: u64 = 3600;
//         let ttl_buffer_secs: u64 = 30;
//         // expires_at = base_time + 3600.saturating_sub(30) = base_time + 3570
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_tok_rel", "expires_in": expires_in})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, ttl_buffer_secs);  // [PLANNED]
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("warms cache");  // [PLANNED]
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         now_secs.fetch_add(3500, Ordering::SeqCst);  // 3500s < 3570s (expires_in 3600 − ttl_buffer 30)
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm at 3500s");  // [PLANNED]
//         let posts_still_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_still_warm, posts_after_warm,
//             "VP-159 AC-7a: 3500s < 3570s expires_at → cache valid (BC-2.16.014 P4-TTL-b)");
//         now_secs.fetch_add(100, Ordering::SeqCst);  // total +3600s > +3570s → stale
//         let posts_before_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("stale at 3600s");  // [PLANNED]
//         let posts_after_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_stale, posts_before_stale + 1,
//             "VP-159 AC-7a: 3600s > 3570s expires_at → re-acquisition \
//              (BC-2.16.014 P4-TTL-b: expires_at = (self.now_fn)() + expires_in.saturating_sub(ttl_buffer_secs))");
//     }
//
//     // AC-7b (EC-016-014-001): absent expires_in → default 1799 before saturating_sub
//     // expires_at = base_time + 1799.saturating_sub(30) = base_time + 1769
//     // wiremock responds with a body that omits the "expires_in" key entirely
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     #[tokio::test]
//     async fn test_vp159_ac7b_absent_expires_in_defaults_to_1799() {
//         let ttl_buffer_secs: u64 = 30;
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     // Response omits "expires_in" key entirely — absent case (EC-016-014-001)
//                     .set_body_json(serde_json::json!({"access_token": "tok-noexp"})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, ttl_buffer_secs);  // [PLANNED]
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED] cold — absent expires_in defaults to 1799
//             .expect("VP-159 AC-7b: absent expires_in must succeed with default 1799 TTL");
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         now_secs.fetch_add(1700, Ordering::SeqCst);  // 1700s < 1769s (1799 − 30) → still warm
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm at 1700s");  // [PLANNED]
//         let posts_still_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_still_warm, posts_after_warm,
//             "VP-159 AC-7b: absent expires_in → 1799 default; 1700s < 1769s expires_at → cache valid \
//              (EC-016-014-001; BC-2.16.014 P4-TTL-b)");
//         now_secs.fetch_add(100, Ordering::SeqCst);  // total +1800s > +1769s → stale
//         let posts_before_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("stale at 1800s");  // [PLANNED]
//         let posts_after_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_stale, posts_before_stale + 1,
//             "VP-159 AC-7b: absent expires_in → 1799 default; 1800s > 1769s expires_at → re-acquisition \
//              (EC-016-014-001; BC-2.16.014 P4-TTL-b)");
//     }
//
//     // AC-7c (EC-016-014-002): zero expires_in → same default 1799 as absent
//     // wiremock responds with expires_in: 0 in response JSON
//     // Clock seam: Arc<AtomicU64> via now_fn in DeclarativeHttpAuthProvider::new_for_test [PLANNED]
//     #[tokio::test]
//     async fn test_vp159_ac7c_zero_expires_in_defaults_to_1799() {
//         let ttl_buffer_secs: u64 = 30;
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "tok-zeroexp", "expires_in": 0u64})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, ttl_buffer_secs);  // [PLANNED]
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange(); // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let _ = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED] cold — zero expires_in defaults to 1799
//             .expect("VP-159 AC-7c: zero expires_in must succeed with default 1799 TTL");
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         now_secs.fetch_add(1700, Ordering::SeqCst);  // still warm
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm at 1700s");  // [PLANNED]
//         let posts_still_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_still_warm, posts_after_warm,
//             "VP-159 AC-7c: zero expires_in → 1799 default; 1700s < 1769s expires_at → cache valid \
//              (EC-016-014-002; BC-2.16.014 P4-TTL-b)");
//         now_secs.fetch_add(100, Ordering::SeqCst);  // total +1800s → stale
//         let posts_before_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("stale at 1800s");  // [PLANNED]
//         let posts_after_stale = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_stale, posts_before_stale + 1,
//             "VP-159 AC-7c: zero expires_in → 1799 default; 1800s > 1769s expires_at → re-acquisition \
//              (EC-016-014-002; BC-2.16.014 P4-TTL-b)");
//     }
//
//     // AC-7d (P2 — oauth2_client_credentials string-typed $.expires_in, RU-Q2):
//     // mock returns "expires_in": "3599" as JSON string instead of number →
//     // engine must leniently parse "3599" → u64 3599 (ADR-054 §D2 RU-Q2; RFC 6749 does not fix
//     // the JSON type; confirmed: Microsoft Entra ID emits string-typed expires_in).
//     // Scope: RU-Q2 applies ONLY to oauth2_client_credentials $.expires_in (ADR-054 §D2);
//     //   NOT to token_exchange relative_seconds (plain u64 per ADR-054 §D4 step 4).
//     // Harness uses build_test_sensor_spec_oauth2() [PLANNED] +
//     //   AuthAcquisitionConfig::new_oauth2("/oauth2/token", ttl_buffer_secs) [PLANNED] — NO ExpiryMode.
//     // Kill condition: as_u64() returns None for string values → default-1799 path →
//     // expires_at = base_time + 1769; advancing clock by 1800s (> 1769 but < 3569) causes an
//     // extra POST under the incorrect default path, which the zero-additional-POST assertion catches.
//     #[tokio::test]
//     async fn test_vp159_ac7d_string_typed_expires_in_lenient_parse() {
//         let ttl_buffer_secs: u64 = 30;
//         // mock returns "3599" as a JSON string — lenient parse must treat as u64 3599
//         // expires_at = base_time + 3599.saturating_sub(30) = base_time + 3569
//         // Scope: oauth2_client_credentials $.expires_in lenient parse (RU-Q2; ADR-054 §D2).
//         //   token_exchange relative_seconds uses plain u64 (ADR-054 §D4 step 4) — not tested here.
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth2/token"))  // oauth2_client_credentials token path (ADR-054 §D2 CrowdStrike example)
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({
//                         "access_token": "tok-str-exp",
//                         "expires_in": "3599"  // JSON STRING not number — lenient parse required (RU-Q2)
//                     })),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth2/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("test-credential");
//         // AuthAcquisitionConfig::new_oauth2 [PLANNED — engine story]: oauth2_client_credentials
//         // config — no ExpiryMode parameter (oauth2 TTL is always relative seconds via $.expires_in;
//         // no expiry_mode selector field per ADR-054 §D3 common-fields table).
//         let config = AuthAcquisitionConfig::new_oauth2("/oauth2/token", ttl_buffer_secs);  // [PLANNED — engine story]
//         let base_time = 1_700_000_000u64;
//         let now_secs = Arc::new(AtomicU64::new(base_time));
//         let mock_time_fn = {
//             let t = Arc::clone(&now_secs);
//             Arc::new(move || t.load(Ordering::SeqCst)) as Arc<dyn Fn() -> u64 + Send + Sync>
//         };
//         let provider = DeclarativeHttpAuthProvider::new_for_test(  // [PLANNED — engine story; cfg(test)]
//             token_url, config, Arc::new(creds), mock_time_fn,
//         );
//         // build_test_sensor_spec_oauth2 [PLANNED — engine story]: creates an oauth2_client_credentials
//         // sensor spec (auth_type = "oauth2_client_credentials", base_url = mock_server.uri()).
//         // Distinct from build_test_sensor_spec_token_exchange() used by ACs 1-9b.
//         let sensor_spec = build_test_sensor_spec_oauth2(); // [PLANNED — engine story]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         // Cold call: lenient parse "3599" → 3599 → expires_at = base_time + 3569
//         let _ = provider.get_token(&sensor_spec, &org_slug).await
//             .expect("VP-159 AC-7d: oauth2_client_credentials string-typed $.expires_in must succeed with lenient parse (RU-Q2)");
//         let posts_after_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         // Advance clock 1800s > 1769 (what default-1799 path would compute) but < 3569 (correct)
//         // If lenient parse worked: clock 1_700_001_800 < expires_at 1_700_003_569 → still warm → 0 POSTs
//         // If default-1799 path (bug): clock 1_700_001_800 > expires_at 1_700_001_769 → stale → 1 POST → FAIL
//         now_secs.store(base_time + 1800, Ordering::SeqCst);
//         let _ = provider.get_token(&sensor_spec, &org_slug).await.expect("still warm at 1800s");
//         let posts_still_warm = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_still_warm, posts_after_warm,
//             "VP-159 AC-7d: oauth2_client_credentials string-typed $.expires_in '3599' → 3599 (not default 1799); \
//              at 1800s < 3569 expires_at, cache must still be valid, 0 additional POSTs \
//              (RU-Q2; BC-2.16.014 P2 oauth2_client_credentials arm; EC-016-014-016)");
//     }
//
//     // AC-8 (P7): CachedAuthToken never stores credential values — runtime inequality assertion.
//     // Distinct mock credential ("mock_client_secret_P7") and mock token ("opaque_bearer_P7") values.
//     // After get_token() [PLANNED]: returned_token == "opaque_bearer_P7" AND returned_token != credential.
//     // Structural enforcement: CachedAuthToken::new(token, expires_at) [PLANNED] accepts no credential
//     // parameter; #[non_exhaustive] prevents external struct literals — adding a credential field
//     // forces a constructor change. Runtime assertion is the load-bearing check (BC-2.16.014 P7, AD-017).
//     #[tokio::test]
//     async fn test_vp159_ac8_cached_token_no_credential_value_stored() {
//         let credential_value = "mock_client_secret_P7";  // distinct from mock token value
//         let mock_token_value = "opaque_bearer_P7";        // distinct from credential
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": mock_token_value, "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new(credential_value);
//         let config = AuthAcquisitionConfig::new("/oauth/token", ExpiryMode::RelativeSeconds, 30);  // [PLANNED]
//         let provider = DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         );
//         let sensor_spec = build_test_sensor_spec_token_exchange();  // [PLANNED]
//         let org_slug = prism_core::OrgSlug::new("test-org");
//         let returned_token = provider.get_token(&sensor_spec, &org_slug).await  // [PLANNED]
//             .expect("VP-159 AC-8: get_token must succeed (BC-2.16.014 P7 setup)");
//         // Load-bearing runtime assertions: the returned token is the mock HTTP response value,
//         // not the resolved credential value. Proves CachedAuthToken never stores credentials.
//         assert_eq!(returned_token, mock_token_value,
//             "VP-159 AC-8: get_token must return the acquired token value, not a credential value \
//              (BC-2.16.014 P7; AD-017; INV-014-003)");
//         assert_ne!(returned_token, credential_value,
//             "VP-159 AC-8: returned token must NOT equal the resolved credential \
//              (CachedAuthToken stores only token + expires_at — no credential field; \
//               BC-2.16.014 P7; AD-017)");
//         // Structural note: CachedAuthToken::new(token, expires_at) [PLANNED] accepts no credential
//         // parameter. #[non_exhaustive] prevents external struct literals. These two properties
//         // together enforce the credential-opacity invariant architecturally (BC-2.16.014 P7).
//     }
//
//     // AC-9 (P9-execute_impl path; SAP-3 execute reachability):
//     // PipelineExecutor::execute called twice, same warm DeclarativeHttpAuthProvider [PLANNED],
//     // long TTL → exactly 1 token-endpoint POST total across both calls.
//     // Single wiremock server hosts both POST /oauth/token (token endpoint) and
//     // GET /items (sensor API). POST count via received_requests() filter.
//     // [PLANNED — engine story]: DeclarativeHttpAuthProvider, build_test_sensor_spec_token_exchange,
//     //   build_test_table_spec
//     #[tokio::test]
//     async fn test_vp159_ac9_execute_impl_path_cache_sharing() {
//         // Single wiremock server hosts both endpoints:
//         //   POST /oauth/token → token exchange response (token endpoint)
//         //   GET  /items       → sensor API response (sensor API mock)
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_token_ac9", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         WmMock::given(wm_method("GET"))
//             .and(wm_path("/items"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"items": [{"id": 1}]})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("client_secret_ac9");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let provider = Arc::new(DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         ));
//         let spec = build_test_sensor_spec_token_exchange(); // [PLANNED — engine story helper]
//         // build_test_sensor_spec_token_exchange() sets base_url = mock_server.uri()
//         let table = build_test_table_spec(); // [PLANNED — engine story helper]
//         let context = FetchContext::new(
//             prism_core::OrgSlug::new("test-org"),
//             std::collections::HashMap::new(),
//         );
//         let http_client = reqwest::Client::builder()
//             .timeout(std::time::Duration::from_secs(30))
//             .build()
//             .expect("test client"); // 30s timeout per CLAUDE.md ADR-050; direct construction
//         // (build_http_client_with_timeout is pub(crate) and inaccessible from tests/)
//         // First execute call: cold cache → 1 POST to token endpoint
//         let _ = PipelineExecutor::execute(&spec, &table, &context, &http_client, provider.as_ref())
//             .await.expect("VP-159 AC-9: first execute call must succeed");
//         let posts_after_first = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_first, 1,
//             "VP-159 AC-9: first execute call must issue exactly 1 token-endpoint POST");
//         // Second execute call: warm cache (TTL 3600s >> 0 elapsed) → 0 additional POSTs
//         let _ = PipelineExecutor::execute(&spec, &table, &context, &http_client, provider.as_ref())
//             .await.expect("VP-159 AC-9: second execute call must succeed");
//         let posts_after_second = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_second, 1,
//             "VP-159 AC-9: second execute call must use cached token — zero additional \
//              token-endpoint POSTs (BC-2.16.014 P9 execute_impl path; ADR-054 §D4)");
//     }
//
//     // AC-9b (P9-execute_step path; SAP-3 execute_step reachability):
//     // PipelineExecutor::execute_step called twice directly (plugin-runtime entry point per
//     // ADR-054 §D11), same warm DeclarativeHttpAuthProvider [PLANNED], long TTL → exactly 1
//     // token-endpoint POST total across both calls.
//     // Single wiremock server hosts both POST /oauth/token (token endpoint) and
//     // GET /items (sensor API). No MockHttpClient needed.
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
//     // [PLANNED — engine story]: DeclarativeHttpAuthProvider, build_test_sensor_spec_token_exchange
//     #[tokio::test]
//     async fn test_vp159_ac9b_execute_step_path_cache_sharing() {
//         // Single wiremock server hosts both endpoints:
//         //   POST /oauth/token → token exchange response (token endpoint)
//         //   GET  /items       → sensor API response (sensor API mock)
//         // The http_client parameter to execute_step is the sensor API client; it uses
//         // the same wiremock server URI as base_url for the sensor API GET /items call.
//         // The token_url inside DeclarativeHttpAuthProvider also points at this server's
//         // POST /oauth/token — no separate MockHttpClient needed.
//         let mock_server = MockServer::start().await;
//         WmMock::given(wm_method("POST"))
//             .and(wm_path("/oauth/token"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"access_token": "bearer_token_ac9b", "expires_in": 3600})),
//             )
//             .mount(&mock_server)
//             .await;
//         WmMock::given(wm_method("GET"))
//             .and(wm_path("/items"))
//             .respond_with(
//                 ResponseTemplate::new(200)
//                     .set_body_json(serde_json::json!({"items": [{"id": 1}]})),
//             )
//             .mount(&mock_server)
//             .await;
//         let token_url = format!("{}/oauth/token", mock_server.uri());
//         let creds = MockCredentialResolver::new("client_secret_ac9b");
//         let config = base_config("/oauth/token", ExpiryMode::RelativeSeconds, 30); // [PLANNED]
//         let provider = Arc::new(DeclarativeHttpAuthProvider::new(  // [PLANNED]
//             token_url, config, Arc::new(creds),
//         ));
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
//             prism_core::OrgSlug::new("test-org"),
//             std::collections::HashMap::new(),
//         );
//         let http_client = reqwest::Client::builder()
//             .timeout(std::time::Duration::from_secs(30))
//             .build()
//             .expect("test client"); // 30s timeout per CLAUDE.md ADR-050; direct construction
//         // (build_http_client_with_timeout is pub(crate) and inaccessible from tests/)
//         // First execute_step call: cold cache → 1 POST to token endpoint
//         let _ = PipelineExecutor::execute_step(
//             &step, &spec, &prior_vars, &context, &http_client, provider.as_ref(),
//         ).await.expect("VP-159 AC-9b: first execute_step call must succeed");
//         let posts_after_first = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_first, 1,
//             "VP-159 AC-9b: first execute_step call must issue exactly 1 token-endpoint POST");
//         // Second execute_step call: warm cache (TTL 3600s >> 0 elapsed) → 0 additional POSTs
//         let _ = PipelineExecutor::execute_step(
//             &step, &spec, &prior_vars, &context, &http_client, provider.as_ref(),
//         ).await.expect("VP-159 AC-9b: second execute_step call must succeed");
//         let posts_after_second = mock_server.received_requests().await.unwrap()
//             .iter().filter(|r| r.method == wiremock::http::Method::POST).count();
//         assert_eq!(posts_after_second, 1,
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
| Proof complexity | Medium | Requires wiremock server for HTTP interception and `Arc<AtomicU64>` clock seam for TTL testing; both are straightforward patterns in the prism-spec-engine test suite |
| Tool support | Full | `MockCredentialResolver` is confirmed at `crates/prism-spec-engine/src/auth_provider.rs` (test-helpers gate); `wiremock` is a confirmed dev-dep in `crates/prism-spec-engine/Cargo.toml`; `DeclarativeHttpAuthProvider::new_for_test` and `Arc<AtomicU64>` clock seam are co-located with `DeclarativeHttpAuthProvider` [PLANNED] implementation in the same story — no `MockHttpClient` needed |
| Harness dependencies | Medium (planned) | `DeclarativeHttpAuthProvider`, `AuthAcquisitionConfig`, `ExpiryMode`, `CachedAuthToken` are all [PLANNED — engine story]; `DeclarativeHttpAuthProvider::seed_cache_for_test` [PLANNED — engine story; `cfg(any(test, feature = "test-helpers"))`] required by AC-4b to inject a poisoned-cache entry (empty-string token with valid TTL) directly into the `ArcSwap` — co-located with the implementation in the same story; `wiremock` and `Arc<AtomicU64>` (from std) are confirmed; harness is authored in the same Wave-A story as the implementation |
| Estimated proof time | < 1 second | Deterministic async scenarios with mock I/O; no real network, no real clock dependency |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-07-22 | architect (D-1947 Wave-A spec-evolution burst 2) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.26 | wave-a-spec-evolution-fix-burst-43 | 2026-07-24 | architect | F-WASE-P56-MED-001. **ADR-054 §D11 alignment — `new_oauth2` constructor manifested.** VP-159 v1.25 (FB42) introduced `AuthAcquisitionConfig::new_oauth2("/oauth2/token", ttl_buffer_secs)` [PLANNED — engine story] at AC-7d (three harness sites: prose ~L215, harness header ~L957, harness body ~L985), but FB42's changelog claim "No ADR-054 edits needed" was incorrect — `new_oauth2` was absent from ADR-054 §D11, leaving a cross-artifact contradiction: an engine-story implementer building from §D11 would construct only `new` + `new_token_exchange`, and the AC-7d harness (referencing `new_oauth2`) would not compile. Adjudication: Option (a) — ratify `new_oauth2`. Rationale: §D3 lists `expiry_mode` as token_exchange-only; §D10(h) rejects `expiry_mode` on an `oauth2_client_credentials` block at spec-load time; there is no valid `ExpiryMode` value for an oauth2 config; a two-parameter `new_oauth2(token_path, ttl_buffer_secs)` precisely models §D3 (oauth2 expiry is always relative seconds via `$.expires_in`, engine-internal per RFC 6749). ADR-054 v0.52 (companion burst) amends §D11: "two named constructors" → "three named constructors"; `new(...)` re-scoped to token_exchange minimal harness; `new_oauth2` added as constructor (3) with §D3/§D10(h) rationale and F-WASE-P56-MED-001 provenance. VP-159 content unchanged — no AC or harness body edit required; `new_oauth2` [PLANNED — engine story] references at ~L215/~L957/~L985 are correct as-is and now consistent with the amended §D11. POL-29 sweep: zero additional live-body `new_oauth2` sites in .factory/ beyond VP-159 (STATE.md + VP-INDEX.md changelog rows are historical). |
| 1.25 | wave-a-spec-evolution-fix-burst-42 | 2026-07-24 | architect | F-WASE-P55-MED-001 + F-WASE-P55-OBS-001. **MED-001 — AC-7d harness mis-modeled as token_exchange; adjudication branch (a).** RU-Q2 lenient `$.expires_in` parsing is scoped EXCLUSIVELY to `oauth2_client_credentials` (ADR-054 §D2 L164-165; BC-2.16.014 §P2 oauth2 arm; TV-11; EC-016-014-016). `token_exchange` `relative_seconds` uses plain u64 (ADR-054 §D4 step 4 L308) — no lenient-parse qualifier. No api-spec evidence of any token_exchange endpoint returning string-typed relative-seconds values. Branch (b) rejected. AC-7d heading updated: `"P4-TTL-b — string-typed expires_in"` → `"P2 — oauth2_client_credentials string-typed $.expires_in, RU-Q2"`. AC-7d body rewritten to state oauth2 scope + ADR-054 §D2 citation + scope boundary note. Harness comment header rewritten: scope note added, "NOT to token_exchange relative_seconds" documented. Harness body: (1) wm_path `/oauth/token` → `/oauth2/token` (aligns with CrowdStrike migration example ADR-054 §D2); (2) token_url updated to `/oauth2/token`; (3) `base_config("/oauth/token", ExpiryMode::RelativeSeconds, ttl_buffer_secs)` → `AuthAcquisitionConfig::new_oauth2("/oauth2/token", ttl_buffer_secs)` [PLANNED — engine story] (no ExpiryMode parameter; oauth2 has no expiry_mode selector per ADR-054 §D3); (4) `build_test_sensor_spec_token_exchange()` → `build_test_sensor_spec_oauth2()` [PLANNED — engine story]; (5) assertion message citation updated from `BC-2.16.014 P4-TTL-b` → `BC-2.16.014 P2 oauth2_client_credentials arm; EC-016-014-016`. **OBS-001 — oauth2 harness coverage asymmetry.** Added §Property Statement scope note "Scope note — auth-type harness split (F-WASE-P55-OBS-001)" documenting: ACs 1–9b drive token_exchange path for cache-lifecycle (correct); AC-7d is the sole oauth2_client_credentials harness in VP-159 (RU-Q2 lenient parse); oauth2 form/response shaping delegated to BC-2.16.014 TV-1/TV-5/TV-6/TV-11 + EC-016-014-016; token_exchange relative_seconds plain-u64 boundary documented. POL-29 sweep: `rg 'token_exchange' .factory/specs/verification-properties/vp-159-` confirms no remaining token_exchange references inside AC-7d text or harness. All other ACs correctly retain build_test_sensor_spec_token_exchange() for cache-lifecycle coverage. AC-6c ExpiryMode::AbsoluteUtcString correctly scoped to token_exchange (RU-Q1 adjudication unchanged). No ADR-054 edits needed — §D2 and §D4 are already correct and consistent. No BC edits needed — product-owner owns BC content; no BC amendment required for branch (a). |
| 1.24 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | state-manager | F-WASE-P49-HIGH-001: POL-23 standing pin sweep. BC-2.16.014 promoted to v1.18 (FB37 bump). Three live-body citation pins advanced v1.17→v1.18: §Source Contract authoring-source bullet, §Source Contract inline restatement, §Proof Harness Skeleton header comment. No behavioral content changed. Precedent: BC-INDEX v8.55 pin sweep. |
| 1.23 | wave-a-ru-amendment-D1944 | 2026-07-23 | architect | RU-Q1/Q2 + ADR-054 §D4 v0.51 alignment. §Property Statement P4-TTL-a formula updated: `parse_rfc3339` → lenient chrono relaxed `FromStr` (`expiry_str.parse::<DateTime<FixedOffset>>()`), accepts both `T`-separator ISO-8601 and space-separated UTC strings per ADR-054 §D4 v0.51 RU-Q1 adjudication; kill condition documented (space-separated fixture FAILS strict-parse-only impl). AC-6 prose updated to reference lenient parse formula. AC-6c added: new AC (prose + harness skeleton) asserting `"2099-01-01 00:00:00.000000+00:00"` parses to the same epoch as `"2099-01-01T00:00:00Z"` (kill condition: strict `parse_from_rfc3339`-only impl rejects space-separated form → test FAILS). AC-7d added: new AC (prose + harness skeleton) for string-typed `expires_in` `"3599"` (JSON string) → lenient parse to u64 3599; kill condition: `as_u64()`-only path defaults to 1799, clock advance 1800s > 1769 (default) but < 3569 (correct) triggers spurious re-acquire POST (RU-Q2; BC-2.16.014 P4-TTL-b). modified: synced. |
| 1.22 | wave-a-fix-burst-33 | 2026-07-23 | architect | F-WASE-P38-MED-001 + F-WASE-P38-LOW-001. **MED-001 — external struct-literal construction of `#[non_exhaustive]`-destined types (E0639).** 5 harness sites replaced with constructor-based construction: (1) `base_config` helper — `AuthAcquisitionConfig { token_path, expiry_mode, ttl_buffer_secs, ..Default::default() }` → `AuthAcquisitionConfig::new(token_path, expiry_mode, ttl_buffer_secs)` [PLANNED]; (2) AC-4b — `CachedAuthToken { token: "".to_string(), expires_at }` → `CachedAuthToken::new("".to_string(), expires_at)` [PLANNED]; (3) AC-6 — full token_exchange `AuthAcquisitionConfig` struct literal → `AuthAcquisitionConfig::new_token_exchange("/api/v1/access_token/", "secret_key", "data.access_token", "data.expiration_utc", ExpiryMode::AbsoluteUtcString, ttl_buffer_secs)` [PLANNED]; (4) AC-6b — same pattern with `ttl_buffer_secs = 30` literal; (5) AC-4b prose — `CachedAuthToken { token: "".to_string(), expires_at: base_time + 86_400 }` example → `CachedAuthToken::new("".to_string(), base_time + 86_400)` constructor call. Constructor signatures decided: `AuthAcquisitionConfig::new(token_path: impl Into<String>, expiry_mode: ExpiryMode, ttl_buffer_secs: u64) -> Self` (common fields; token-exchange-specific fields default to empty string); `AuthAcquisitionConfig::new_token_exchange(token_path, credential_body_field, token_response_path, expiry_field, expiry_mode, ttl_buffer_secs) -> Self` (full form); `CachedAuthToken::new(token: String, expires_at: u64) -> Self`. ADR-054 §D11 gains two new rows (v0.48). POL-29 sweep: `rg 'AuthAcquisitionConfig \{' .factory/` + `rg 'CachedAuthToken \{' .factory/` → zero external struct-literal constructions remain in VP-159, VP-153 (never constructs these types), BC-2.16.014 (spec prose, no Rust code). **LOW-001 — AC-8 prose ↔ skeleton drift.** Implemented stronger runtime assertion: AC-8 rewritten from synchronous `#[test] fn test_vp159_ac8_cached_token_no_credential_field` to async `#[tokio::test] async fn test_vp159_ac8_cached_token_no_credential_value_stored` — drives `get_token()` [PLANNED] with distinct mock credential `"mock_client_secret_P7"` and mock token `"opaque_bearer_P7"` values; asserts `returned_token == "opaque_bearer_P7"` AND `returned_token != "mock_client_secret_P7"` (the promised negative assertion; both load-bearing per SID-2). Old exhaustive-struct-literal rationale removed — it was vacuous under `#[non_exhaustive]` (external literals always impossible, so "this literal would fail to compile if a field were added" proved nothing about field absence). Structural note preserved: `CachedAuthToken::new(token, expires_at)` accepts no credential parameter — adding a credential field requires a constructor signature change, making omission architecturally documented. AC-8 prose updated to match skeleton. input-hash unchanged (inputs not modified in this burst). |
| 1.21 | wave-a-fix-burst-30 | 2026-07-23 | architect | F-WASE-P34-LOW-001: §Property Statement P3 was missing the `!cached.token.is_empty()` conjunct — stated "issued before `unix_now() >= expires_at`" (TTL-only), while P4 fires on `(unix_now() >= cached.expires_at) OR (cached.token.is_empty())`. This created a gap state (now < expires_at AND token empty) where P3 asserted ZERO requests but P4 asserted ONE POST — an intra-document contradiction. Fix: P3 rewritten to "issued when `unix_now() < cached.expires_at` AND `!cached.token.is_empty()`", restoring P3/P4 as exact De Morgan complements. Complement totality: P4 = `(unix_now() >= cached.expires_at) OR (cached.token.is_empty())`; NOT(P4) = `(unix_now() < cached.expires_at) AND (!cached.token.is_empty())` = P3 ✓ (mutually exclusive and collectively exhaustive over cache-entry-present states). POL-29 sweep: AC-3 prose stated "within TTL" (TTL-only, missing `!is_empty()`) — updated to explicit dual-conjunct form `unix_now() < cached.expires_at AND !cached.token.is_empty()`. AC-6 harness assertion "clock before expires_at → cache valid" cites BC-2.16.014 P3 in a TTL-a scenario where the token is structurally non-empty (just acquired) — not a definitional restatement; left as-is. Harness header comment and AC-3 test assertion message use "warm" as shorthand (structurally non-empty by test flow) — not definitional restatements; left as-is. POL-23 pin sweep (BC-2.16.014 v1.15→v1.16 per PO bump in this burst for TV-9/F-WASE-P34-LOW-002): 3 live-body pins updated — §Source Contract first occurrence `Token Lifecycle) v1.15` → v1.16, §Source Contract inline restatement `(BC-2.16.014 v1.15)` → v1.16, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.15` → v1.16. Grep of all architect-owned artifacts (VPs, ADRs, verification-architecture.md, verification-coverage-matrix.md) confirms no other live v1.15 pins beyond these 3. Historical changelog rows untouched. |
| 1.20 | wave-a-fix-burst-29 | 2026-07-23 | architect | F-WASE-P32-HIGH-001: AC-4b clock-domain defect — harness was constructing via production `new(...)` (real wall-clock) and seeding `expires_at = 1_700_000_000 + 86_400` (2023-11-15, a past date in 2026+). At any real run time (2026+), `unix_now() >= expires_at` was TRUE, so the TTL-stale branch fired regardless of `is_empty()` — AC-4b added zero marginal coverage. Fix: (1) §AC-4b prose rewritten — construction changed to `new_for_test` with `now_fn` pinned at `base_time = 1_700_000_000u64`; `expires_at = base_time + 86_400` is genuinely far future relative to pinned clock; kill-condition stated explicitly. (2) Harness skeleton `test_vp159_ac4b_empty_token_reacquisition` rewritten — `new(...)` → `new_for_test(token_url, config, Arc::new(creds), mock_time_fn)` with `Arc<AtomicU64>` at `base_time`; `far_future_expires_at` binding removed; `expires_at = base_time + 86_400`; kill-condition comment added. (3) POL-29 sweep: no other AC-4b-context occurrences of `1_700_000_000`-based far-future constants or "no-clock-seam ⇒ now effectively 0" premise found in VP-159 (the constant appears only in the fixed test and legitimately in AC-4/AC-7 as a `base_time` clock anchor, not as a "far future" assertion — those are correct). No BC-2.16.014 §VP row (e) edit needed (property (e) describes the `is_empty()` branch behavior, not harness mechanics). input-hash unchanged (inputs not modified in this burst). |
| 1.19 | wave-a-fix-burst-25 | 2026-07-23 | architect | STANDING PIN SWEEP (FIX-BURST 25): BC-2.16.014 v1.14→v1.15 bump (DI-012 counting-unit clarification in INV-014-006). 3 live-body pins updated — §Source Contract first occurrence `Token Lifecycle) v1.14`, §Source Contract inline restatement `(BC-2.16.014 v1.14)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.14` — all now v1.15. input-hash updated 87576a4→f702703 (BC-2.16.014 input drifted since last hash). No behavioral content changed. input-hash: at-commit-time hash per POL-32 (modified: sync). |
| 1.18 | wave-a-spec-evolution-fix-burst-24 | 2026-07-23 | architect | F-WASE-P26-LOW-001: (1) §Property Statement P4 heading and body extended from TTL-only trigger to include `cached.token.is_empty()` poisoned-cache trigger — "stale `get_token()`" → "stale or poisoned `get_token()`"; condition updated from `unix_now() >= expires_at` alone to `unix_now() >= cached.expires_at` OR `cached.token.is_empty()` (BC-2.16.014 P4 parity). (2) AC-4b prose bullet added between AC-4 and AC-5: seeds provider cache via `DeclarativeHttpAuthProvider::seed_cache_for_test` [PLANNED — engine story] with `CachedAuthToken { token: "".to_string(), expires_at: far_future }`, calls `get_token()`, asserts wiremock POST count == 1 and returned token == fresh non-empty value. (3) Harness skeleton `test_vp159_ac4b_empty_token_reacquisition` added after `test_vp159_ac4_stale_cache_one_post`; `seed_cache_for_test` seam marked [PLANNED — engine story; cfg(any(test, feature = "test-helpers"))]. (4) §Feasibility Assessment Harness dependencies row updated to list `DeclarativeHttpAuthProvider::seed_cache_for_test` [PLANNED] as additional test seam. No BC-2.16.014 changes — its (e) property claim at VP-159 row is already correct; AC-4b existence makes it verified. input-hash: at-commit-time hash per POL-32 (modified: sync). |
| 1.17 | wave-a-spec-evolution-fix-burst-22 | 2026-07-23 | architect | STANDING PIN SWEEP (FIX-BURST 22): 3 live-body BC-2.16.014 pins advanced v1.13→v1.14 (PO bumped BC-2.16.014 v1.13→v1.14 in parallel) — §Source Contract first occurrence `Token Lifecycle) v1.13`, §Source Contract inline restatement `(BC-2.16.014 v1.13)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.13` — all now v1.14. input-hash updated dc3b3bd→9491150 (BC-2.16.014 input drifted since last hash). At-commit-time hash per POL-32. |
| 1.16 | wave-a-fix-burst-21 | 2026-07-23 | architect | F-WASE-P21-HIGH-001(a): §Property Statement P4-TTL-b — removed present-tense "retired" from "matching the retired crowdstrike-oauth2 plugin's arithmetic" → "matching the crowdstrike-oauth2 plugin's arithmetic"; forward framing matches ADR-054 §D9 source ("matches the plugin's", no "retired"). F-WASE-P21-LOW-001: §Property Statement P4-TTL-b dead-code note — "per ADR-054 §D4 note" → "per ADR-054 §D9 note" (the note lives in §D9, not §D4). Standing pin sweep: BC-2.16.014 v1.12→v1.13 at all 3 live-body pins — §Source Contract first occurrence `Token Lifecycle) v1.12`, §Source Contract inline restatement `(BC-2.16.014 v1.12)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.12` — all now v1.13. input-hash: at-commit-time hash per POL-32. |
| 1.15 | wave-a-fix-burst-19 | 2026-07-23 | architect | Pin sweep only (POL-32). BC-2.16.014 v1.11→v1.12 bump: 3 live-body pins updated — §Source Contract first occurrence `Token Lifecycle) v1.11`, §Source Contract inline restatement `(BC-2.16.014 v1.11)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.11` — all now v1.12. No behavioral content changed. input-hash: at-commit-time hash per POL-32. |
| 1.14 | wave-a-fix-burst-18 | 2026-07-23 | architect | F-WASE-P18-LOW-001: §Proof Harness Skeleton — all 11 `prism_core::OrgSlug::new_unchecked("test-org")` call sites replaced with `prism_core::OrgSlug::new("test-org")`. `OrgSlug::new` is infallible (`pub fn new(s: impl AsRef<str>) -> Self`, confirmed `crates/prism-core/src/tenant.rs`); returns `Self` directly — no `.unwrap()` required; "test-org" satisfies `^[a-zA-Z0-9_-]{1,64}$`; sibling idiom in `crates/prism-spec-engine/src/pipeline.rs` tests is `OrgSlug::new("test-org")` (10 occurrences confirmed). Sites fixed: AC-2 (1), AC-3 (1), AC-4 (1), AC-5 (1), AC-6 (1), AC-6b (1), AC-7a (1), AC-7b (1), AC-7c (1), AC-9 (1), AC-9b (1) — 11 total (finding cited 6; sweep found 5 additional in AC-7a/7b/7c/AC-9/AC-9b). All ALLOWLIST-REQUIRED notes deleted: 4-line block in AC-2, 8 one-line "see AC-2 note above" notes in AC-3/4/5/6/6b/7a/7b/7c, 2-line note after FetchContext::new in AC-9, 2-line note after FetchContext::new in AC-9b. Sweep result: zero `new_unchecked` or `ALLOWLIST REQUIRED` in code or prose; changelog rows (lines 957, 965) exempt and unchanged. input-hash: at-commit-time hash per POL-32. |
| 1.13 | wave-a-fix-burst-17 | 2026-07-23 | architect | Pin sweep only (POL-7 Related-BCs label sweep; POL-32). Parallel PO BC-2.16.014 v1.10→v1.11 bump: 3 live-body pins updated — §Source Contract authoring-source first occurrence `Token Lifecycle) v1.10`, §Source Contract inline restatement `(BC-2.16.014 v1.10)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.10` — all now v1.11. input-hash updated 48b9704→9b909f8 (input drift resolved). No behavioral content changed. input-hash: at-commit-time hash per POL-32. |
| 1.12 | wave-a-spec-evolution-fix-burst-15 | 2026-07-22 | architect | Pin sweep only (F-WASE-P15 pin obligation; POL-32). Parallel PO BC-2.16.014 v1.9→v1.10 bump: 3 live-body pins updated — §Source Contract authoring-source first occurrence `Token Lifecycle) v1.9`, §Source Contract inline restatement `(BC-2.16.014 v1.9)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.9` — all now v1.10. No behavioral content changed. input-hash: at-commit-time hash per POL-32. |
| 1.11 | wave-a-spec-evolution-fix-burst-14 | 2026-07-22 | architect | F-WASE-P14-MED-001 (fix-burst 14): `ExpiryMode::RelativeSeconds` struct-variant form corrected to unit-variant form throughout harness skeleton. Per ADR-054 §D3: `ttl_buffer_secs` is a common `AuthAcquisitionConfig` field (default 30), independent of `expiry_mode`; `ExpiryMode` variants are unit variants (no fields). `base_config` helper signature updated: `base_config(token_path: &str, expiry_mode: ExpiryMode)` → `base_config(token_path: &str, expiry_mode: ExpiryMode, ttl_buffer_secs: u64)`; `ttl_buffer_secs` field added to the returned `AuthAcquisitionConfig` struct body. All 10 call sites fixed — `ExpiryMode::RelativeSeconds { ttl_buffer_secs: N }` → `ExpiryMode::RelativeSeconds, N`: AC-1 (30), AC-2 (30), AC-3 (30), AC-4 (0), AC-5 (30), AC-7a (ttl_buffer_secs variable), AC-7b (ttl_buffer_secs variable), AC-7c (ttl_buffer_secs variable), AC-9 (30), AC-9b (30). AC-6 and AC-6b confirmed correct (explicit `AuthAcquisitionConfig` structs with `expiry_mode: ExpiryMode::AbsoluteUtcString` unit variant + `ttl_buffer_secs` as separate field — no changes). `rg 'RelativeSeconds \{'` sweep: zero struct-variant forms remain. Standing pin sweep (parallel PO BC-2.16.014 v1.8→v1.9 bump): 3 live-body sites updated — §Source Contract first pin `v1.8`, §Source Contract inline restatement `v1.8`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.8` — all now v1.9. input-hash: at-commit-time hash per POL-32. |
| 1.10 | wave-a-spec-evolution-fix-burst-13 | 2026-07-22 | architect | F-WASE-P13-MED-001 + OBS-P13-001 (fix-burst 13): Ratified OPTION (b) — internal reqwest client, no HTTP injection seam. §Acceptance Criteria header: `MockHttpClient` [PLANNED] → wiremock (confirmed dev-dep) for all HTTP interception; `DeclarativeHttpAuthProvider::new_for_test` [PLANNED] clock seam noted. AC-1 prose: "zero calls recorded by `MockHttpClient`" → "zero requests received by the wiremock server". AC-2 prose: "one `MockHttpClient` POST call" → "one POST request to the wiremock token endpoint". AC-3 prose: "zero additional `MockHttpClient` calls" → "zero additional requests to the wiremock token endpoint". AC-4 prose: "Advancing the mock clock" → "Advancing the mock clock (`Arc<AtomicU64>` passed via `now_fn`)"; "one additional `MockHttpClient` POST call" → "one additional POST request to the wiremock token endpoint". AC-5 prose: "one `MockHttpClient` POST call" → "one POST request to the wiremock token endpoint". §Proof Method table: `MockHttpClient` [PLANNED] for HTTP interception → `wiremock` (confirmed dev-dep) for HTTP interception; `mock clock` → `Arc<AtomicU64>` via `now_fn` clock seam (`new_for_test` [PLANNED]). §Feasibility Assessment Tool support: `MockHttpClient` and mock clock → `wiremock` confirmed dev-dep + `new_for_test`/`Arc<AtomicU64>` clock seam [PLANNED]. §Proof Harness Skeleton: full rewrite — `MockHttpClient` eliminated; all ACs (AC-1 through AC-9b) use wiremock `MockServer` for HTTP interception (`token_url = mock_server.uri() + path`); post-counts via `mock_server.received_requests().await.unwrap().iter().filter(POST).count()`; constructor changed from 3-arg `new(config, Arc::new(mock_http), Arc::new(creds))` to 3-arg `new(token_url, config, Arc::new(creds))`; clock-sensitive tests (AC-4, AC-6, AC-7a/b/c) use `new_for_test(token_url, config, creds, mock_time_fn)` with `Arc<AtomicU64>` clock advanced via `fetch_add()`; AC-6 `advance_clock_to_before/past_expires_at` replaced with `now_secs.store(expires_at ∓ 10)`; AC-9/9b use single wiremock server hosting both `POST /oauth/token` and `GET /items` endpoints — no `MockHttpClient` needed for executor reachability tests. `[PLANNED]` marker audit: `MockHttpClient` references removed; `DeclarativeHttpAuthProvider::new_for_test` marked `[PLANNED — engine story; cfg(test)]`; `wiremock` confirmed dev-dep (no [PLANNED]); `Arc<AtomicU64>` from std (no [PLANNED]). input-hash recomputed 8a305d3 → 232a706 (ADR-054 changed: §D4 Internal state + Constructor added in same burst). BC-2.16.014 INV-014-007 already consistent with OPTION (b) — no BC-2.16.014 edits required. |
| 1.9 | wave-a-spec-evolution-fix-burst-12 | 2026-07-22 | architect | F-WASE-P12-MED-001: §Proof Harness Skeleton — all 19 single-arg `get_token("test-org")` call sites across 9 test functions (AC-2, AC-3, AC-4, AC-5, AC-6, AC-6b, AC-7a, AC-7b, AC-7c) updated to the 2-arg ADR-054 §D4 trait form `get_token(&sensor_spec, &org_slug)`. Per-function: `build_test_sensor_spec_token_exchange()` [PLANNED] and `prism_core::OrgSlug::new_unchecked("test-org")` declarations added before the first `get_token` call in each function; ALLOWLIST note added per CLAUDE.md credential-safety convention. AC-5: hoisted `sensor_spec` + ALLOWLIST + `org_slug` declarations before the warm-cache `get_token` call (previously placed only before `acquire_token`, leaving the `get_token` call above them with wrong single-arg form). Prose verification: all P-statement and AC prose uses `get_token()` without args as behavior description — no wrong single-arg signature stated; no prose change required. Adversary-missed sites swept: AC-5 (~380), AC-7a third site (~496), AC-7b third site (~524), AC-7c third site (~551) — adversary cited 9 grouped locations; sweep found 19 individual code sites; all fixed. input-hash drift resolved: stored f9726cc → computed 8a305d3 (ADR-054 or BC-2.16.014 changed since v1.8 burst; updated via `compute-input-hash --update`). BC-2.16.014 live-body pin sweep (parallel PO bump v1.7→v1.8 in same burst): 3 sites updated — §Source Contract authoring-source first occurrence `Token Lifecycle) v1.7`, §Source Contract inline restatement `(BC-2.16.014 v1.7)`, §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.7` — all now read v1.8. input-hash for this burst: at-commit-time hash per POL-32. |
| 1.8 | wave-a-spec-evolution-fix-burst-11 | 2026-07-22 | architect | F-WASE-P11-MED-001: AC-9 and AC-9b `[PLANNED — engine story]` qualifiers moved off the executor-method symbols onto their `get_token()` cache-aware wiring. `PipelineExecutor::execute` confirmed in `crates/prism-spec-engine/src/pipeline.rs` (~line 138); `PipelineExecutor::execute_step` confirmed (~line 605). What is [PLANNED] is the ADR-054 §D4/§D11 wiring inside them, not the methods themselves. AC-9 prose updated: "drives `PipelineExecutor::execute` (confirmed in `pipeline.rs`; its `get_token()` cache-aware wiring is [PLANNED — engine story] per ADR-054 §D4)". AC-9b prose updated: "drives `PipelineExecutor::execute_step` (confirmed in `pipeline.rs`; its `get_token()` cache-aware wiring is [PLANNED — engine story] per ADR-054 §D11) directly". Per-marker audit: all other [PLANNED] markers verified on genuinely-absent symbols (DeclarativeHttpAuthProvider, get_token(), CachedAuthToken, ExpiryMode, AuthAcquisitionConfig, MockHttpClient, auth/declarative.rs path, mock clock methods, test helper fns) — all correct. Pin sweep: VP-INDEX.md status cell `draft — v1.7` → `draft — v1.8` (state-manager scope; no live body prose pins to VP-159 v1.7 found). input-hash unchanged (ADR-054 and BC-2.16.014 not modified in this burst; frontmatter value f9726cc remains valid per POL-32 at-commit-time wording). |
| 1.7 | wave-a-spec-evolution-fix-burst-10 | 2026-07-22 | architect | F-WASE-P10-MED-001: BC-2.16.014 pin updated v1.6→v1.7 at all three live-body sites: §Source Contract authoring-source bullet first occurrence (`BC-2.16.014 Token Lifecycle) v1.6`), §Source Contract inline restatement `(BC-2.16.014 v1.6)`, and §Proof Harness Skeleton header comment `// BC: BC-2.16.014 v1.6`. Pin strategy: all three pins retain exact version (no unversioned substitution) — rationale: all three serve authoring-context purposes (§Source Contract: "authored against v1.7"; harness comment: implementer-visible as-of-authoring marker); POL-23 sweep cost for a single-file VP with 3 co-located pins is minimal; consistency within the §Source Contract bullet (line 209 and 210 both in the same sentence) requires both to carry the same version. input-hash recomputed d0f0001→f9726cc (BC-2.16.014 content changed as part of the v1.7 bump; hash updated via validator-reported value per POL-32). |
| 1.6 | wave-a-spec-evolution-fix-burst-9 | 2026-07-22 | architect | F-WASE-P9-MED-002: AC-9b description updated — "struct literal" → `prism_spec_engine::spec_parser::FetchStep::new(name, method, path_template, body_template, response_path, pagination_cursor_path, variables_produced, fan_out_batch_size, pagination)` with note that struct-literal is E0639-impossible from `tests/` (`FetchStep` is `#[non_exhaustive]`; confirmed at `spec_parser.rs`). Harness skeleton: comment "FetchStep struct-literal fields confirmed" → "FetchStep::new(...) — struct-literal is E0639-impossible: FetchStep is #[non_exhaustive]" with full `pub fn new` parameter list confirmed from `spec_parser.rs`; skeleton construction changed from `crate::spec_parser::FetchStep { ... }` struct-literal to `prism_spec_engine::spec_parser::FetchStep::new("main", "GET", "/items", None, "$.items", None, vec![], None, None)` with correct external-crate path. F-WASE-P9-MED-003: AC-9 + AC-9b `http_client` construction updated — `crate::pipeline::build_http_client_with_timeout().expect(...)` replaced with `reqwest::Client::builder().timeout(Duration::from_secs(30)).build().expect("test client")` (direct construction per CLAUDE.md ADR-050; `build_http_client_with_timeout` is `pub(crate)` and inaccessible from `tests/`; confirmed return type is `reqwest::Client` not `Result` — `.expect()` would not compile on the original). "confirmed helper (closed TD-S-PLUGIN-PREREQ-B-005)" mislabel removed. input-hash recomputed to d0f0001 (ADR-054 content changed since v1.5; updated via `compute-input-hash --update`). |
| 1.5 | wave-a-spec-evolution-fix-burst-8 | 2026-07-22 | architect | F-WASE-P8-MED-001: AC-9b added (SAP-3 execute_step reachability for P9). Scope note updated: P9 now explicitly split across AC-9 (execute → execute_impl path) and AC-9b (execute_step direct-call path; confirmed sig: `PipelineExecutor::execute_step`). AC-9 heading narrowed to `(P9-execute_impl path; SAP-3 execute reachability)`; AC-9 first-paragraph closing sentence narrowed to cite execute_impl path only (not execute_step); AC-9 SAP-3 note updated to reference AC-9b as the execute_step complement. AC-9b added: drives `PipelineExecutor::execute_step` twice with the same `DeclarativeHttpAuthProvider` [PLANNED] instance and long-lived TTL (3600s), asserts exactly 1 token-endpoint POST total; a mis-wiring leaving `acquire_token` in `execute_step` produces 2 POSTs and fails this test. `FetchStep` struct-literal fields confirmed from `spec_parser::FetchStep`; `FetchContext::new` confirmed non-exhaustive constructor. Sensor API mock uses wiremock (confirmed dev-dep) alongside `MockHttpClient` [PLANNED] for the token endpoint. Harness skeleton: AC-9 skeleton added (`test_vp159_ac9_execute_impl_path_cache_sharing`); AC-9b skeleton added (`test_vp159_ac9b_execute_step_path_cache_sharing`). §Proof Harness header updated: P9-via-AC-9 → P9-via-AC-9+AC-9b. input-hash recomputed to current frontmatter value at commit time (at-commit-time hash wording per POL-32). |
| 1.4 | wave-a-spec-evolution-fix-burst-7 | 2026-07-22 | architect | F-WASE-P7-MED-001: AC-9 added (SAP-3 executor reachability). AC-2/3/4 verify `get_token()` by direct invocation (isolation; defense-in-depth). AC-9 requires an end-to-end test driving `PipelineExecutor::execute` twice against the same `DeclarativeHttpAuthProvider` instance; the second call must record zero additional token-endpoint POSTs, confirming the cache is reached through the production executor call-path. SAP-3 note added in AC-9 text: both AC-2/3/4 (isolation) and AC-9 (reachability) are required. Production caller: `PipelineExecutor::execute_impl` calls `get_token()` per ADR-054 v0.38 §D4 PipelineExecutor call-site dispatch table and §D11 engine-story wiring rows. P9 reconciliation (BC-2.16.014 v1.5): §Source Contract authoring-source updated P1–P8 → P1–P9 (BC-2.16.014 v1.5); §Property Statement preamble updated to P1–P5, P7, P9; scope note heading updated "P6 and P8 (deferred); P9 (verified via AC-9)" with P9 caller text added; AC-9 heading updated to `(P9; SAP-3 executor reachability)` and body cites BC-2.16.014 P9 explicitly; §Proof Harness Skeleton header comment updated to `BC-2.16.014 v1.5 (P1–P5, P7, P9; ...)`. Verified set: P1–P5, P7, P9 (plus P4-TTL-a/b sub-properties); P6/P8 remain deferred. input-hash recomputed to 043b10a (BC-2.16.014 v1.5 content change). |
| 1.3 | Wave-A fix-burst 5 | 2026-07-22 | architect | F-WASE-P5-MED-001: input-hash trail reconciliation — v1.2 changelog row recorded `3af7dc1` as the post-v1.2 input-hash; frontmatter `f761188` is the authoritative current value (recomputed at D-1953 burst immediately after v1.2 was authored, when ADR-054 v0.37 was edited in that same burst; the 3af7dc1→f761188 transition was not captured in the v1.2 row — v1.2 row left immutable per changelog policy). F-WASE-P5-LOW-001: §Proof Harness Skeleton constructor fixes — `MockCredentialResolver::default()` (7 confirmed sites: AC-1, AC-3, AC-4, AC-5, AC-7a, AC-7b, AC-7c; finding cited 8 — 1 discrepancy, all located sites fixed) rewritten to `MockCredentialResolver::new("test-credential")`; `MockCredentialResolver::with_secret("client_secret_xyz")` (AC-2) and `MockCredentialResolver::with_secret("long_lived_secret")` (AC-6, AC-6b) rewritten to `MockCredentialResolver::new("...")` with identical argument value. All 10 sites resolved using the existing `pub fn new(value: impl Into<String>) -> Self` constructor — no new `MockCredentialResolver` extension required. input-hash trail: f761188 (D-1953 committed) → recomputed to current frontmatter value at commit time (hook-detected drift — ADR-054 or BC-2.16.014 changed since D-1953; updated via `compute-input-hash --update`; see frontmatter for settled value). |
| 1.2 | Wave-A fix-burst 4 | 2026-07-22 | architect | F-WASE-P4-OBS-001: §Proof Harness Skeleton — added skeleton test functions for AC-6 (P4-TTL-a `absolute_utc_string` expiry arithmetic, including AC-6b malformed-RFC-3339 → `AuthAcquisitionFailed` per EC-016-014-003) and AC-7 (P4-TTL-b `relative_seconds` expiry arithmetic, including AC-7b absent `expires_in` → default 1799 per EC-016-014-001 and AC-7c zero `expires_in` → default 1799 per EC-016-014-002). All new symbols marked `[PLANNED — engine story]` per POL-31. F-WASE-P4-OBS-003: §Source Contract P1–P8 authoring-source sentence disambiguated — "P1–P8 are the primary **authoring source**" now explicitly followed by "the verified set is P1–P5, P7 (plus P4-TTL-a/b sub-properties) — see §Property Statement scope note for P6/P8 coverage"; eliminates the false-verified-set reading. input-hash updated to `3af7dc1` (inputs unchanged; hash recomputed after prior edit). |
| 1.1 | D-1947/D-1948 Wave-A fix-burst 1 | 2026-07-22 | architect | F-WASE-P1-MED-001: burst attribution corrected D-1946→D-1947 in Lifecycle table and v1.0 Burst cell (VP-159/VP-INDEX authoring is burst 2, D-1947; BC-2.16.014 authoring is burst 1, D-1946). F-WASE-P1-LOW-002: §Property Statement preamble narrowed from P1–P8 to P1–P5, P7 (P4-TTL-a/b sub-properties); scope note added after P7 for P6 (inherent in acquire_token() contract per AuthProvider trait, verified via AC-5 + error-path assertions in engine implementation story) and P8 (spec-load validation property per BC-2.16.009 Rule 6 / E-SPEC-024, deferred to spec-engine validation story — not a runtime lifecycle invariant of DeclarativeHttpAuthProvider). F-WASE-P1-OBS-002 closure: new_unchecked_audit.rs allowlist-entry note added to AC-5 harness skeleton for OrgSlug::new_unchecked per CLAUDE.md credential-safety convention. |
| 1.0 | D-1947 Wave-A spec-evolution burst 2 | 2026-07-22 | architect | Initial authoring. Authoring source: ADR-054 §D9. BC-2.16.014 P1–P8 all covered (P6 — double-401 → AuthRefreshFailed (E-AUTH-002) — is inherent in acquire_token() contract per AuthProvider trait; verified via AC-5 + error-path assertions in the implementation story). DRIFT-D849-002 folded: StaticCookieAuthProvider zero-HTTP is structural (no reqwest::Client field, confirmed in codebase), covered by BC-2.01.017 §P1 (INV-COOKIE-001); VP-159 covers the equivalent invariant for DeclarativeHttpAuthProvider [PLANNED]. All DeclarativeHttpAuthProvider / CachedAuthToken / AuthAcquisitionConfig / ExpiryMode / MockHttpClient symbols marked [PLANNED — engine story] per POL-31 (crates/prism-spec-engine/src/auth/ directory does not exist at authoring time). Existing verified symbols: AuthProvider trait, CredentialResolver trait, MockCredentialResolver, SpecEngineError::AuthAcquisitionFailed (E-AUTH-001), SpecEngineError::AuthRefreshFailed (E-AUTH-002). source_invariant: DI-012 (workspace canonical, domain-spec/invariants.md); INV-014-003 (BC-local credential-opacity invariant) cited in §Source Contract body prose only per VP-INDEX source_invariant schema convention. |
