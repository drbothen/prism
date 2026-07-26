---
document_type: adr
adr_id: "ADR-054"
title: "Native Declarative HTTP Auth Acquisition — TokenExchange and OAuth2ClientCredentials via DeclarativeHttpAuthProvider; Retire crowdstrike-oauth2.prx"
status: accepted
date: "2026-07-20"
modified: "2026-07-25"
version: "0.54"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes: null
superseded_by: null
amends:
  - "ADR-023 (partial — §Rule 4 walk-back: standard HTTP token-acquisition flows do not require WASM plugins; custom_via_plugin escape hatch preserved for genuinely non-standard auth)"
  - "ADR-026 (partial — §D3: AuthType closed enum gains token_exchange variant; affects E-SPEC-012 enum validation and step9a_populate_adapter_registry dispatch)"
  - "ADR-028 (partial — §D13 oauth2_client_credentials: PluginAuthProvider (WASM) path spec-load-rejected per D10(b) E-SPEC-028(b) — fires for auth_type ∈ {oauth2_client_credentials, token_exchange} + auth_plugin present regardless of [auth_acquisition] (Definition 1, adjudicated F-WASE-P2-HIGH-001); DeclarativeHttpAuthProvider (native) is the sole live path; §D2 + §D13 Armis blockquotes updated from custom_via_plugin to token_exchange; crowdstrike-oauth2.prx plugin to be retired per D5)"
related_adrs: [ADR-023, ADR-026, ADR-028, ADR-031, ADR-032, ADR-050, ADR-053]
related_bcs: [BC-2.01.016, BC-2.01.017, BC-2.06.003, BC-2.16.001, BC-2.16.009, BC-2.16.014]
amends_dis: ["DI-012"]
human_authorization: "D-1895 (2026-07-20) — 'Armis auth must NOT require a plugin. Complete the TOML engine to express standard HTTP auth acquisition DECLARATIVELY; retire crowdstrike-oauth2.prx; custom_via_plugin stays only as escape hatch for genuinely arbitrary auth'"
wave_scope: "Wave-A — applies to Armis token-exchange (new sensor) and CrowdStrike oauth2 migration (remove plugin dependency)"
---

# ADR-054: Native Declarative HTTP Auth Acquisition — TokenExchange and OAuth2ClientCredentials via DeclarativeHttpAuthProvider; Retire crowdstrike-oauth2.prx

> **Human Authorization:** D-1895 (2026-07-20). The human (senior architect) ruled that
> standard HTTP token-acquisition flows MUST NOT require a WASM plugin. This ADR responds to
> that ruling by completing the TOML sensor spec engine to express both the Armis
> token-exchange flow and the CrowdStrike OAuth2 client-credentials flow natively.

<!-- BROWNFIELD: You MUST cite implementation evidence (function names + behavioral anchors from
     crates/ or legacy-design-docs/) before this ADR can be accepted. Omitting evidence is a
     template-compliance failure. -->

## Status

Accepted 2026-07-22 (D-1943, human Wave-A approval gate). Amendments to ADR-023/026/028 and amends_dis DI-012 are now EFFECTIVE. Implementation of ADR-054 stories may proceed after the ADR-053 standalone Wave-A engine story lands first, per §D7 merge-dependency.

Current contract highlights: D1 adds `token_exchange` as the 6th AuthType variant; E-SPEC-028(f) validates `client_id`/`client_secret` credential refs for `oauth2_client_credentials`; E-SPEC-028(b) rejects `auth_plugin` for `auth_type ∈ {oauth2_client_credentials, token_exchange}` regardless of `[auth_acquisition]` presence (Definition 1, D10(b)) — no "when `[auth_acquisition]` present" conditional. D2 makes `oauth2_client_credentials` native via `DeclarativeHttpAuthProvider`. D5 retires `crowdstrike-oauth2.prx`. D11 amendment manifest includes 5 downstream "5→6-value" BC count corrections (BC-2.01.016 §Related BCs, BC-2.01.017 §Preconditions/§P3/§Related BCs, BC-2.16.009 §Validation Rules). ADR-054 implementation stories land AFTER ADR-053's standalone Wave-A engine story (Rule 9/E-SPEC-027 must be registered before Rule 10/E-SPEC-028 — see §D7). §D4 step 4 (absolute_utc_string): lenient chrono relaxed `FromStr` (space-separated + T-form) replaces strict `parse_from_rfc3339` per RU-Q1 (v0.51). §D2 `$.expires_in`: lenient deserialization (JSON number OR numeric string) per RU-Q2 (v0.51). Non-exhaustive gate EXPECTED bump 92→95 (AuthAcquisitionConfig/CachedAuthToken/ExpiryMode) documented in D11 per RU-Scanner-1 (v0.51). See §Changelog for full revision history.

---

## Context

### The crowdstrike-oauth2.prx Plugin Does Standard RFC 6749

The CrowdStrike sensor's auth is implemented as a WASM plugin (`crowdstrike-oauth2.prx` at
`crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`). Reviewing the plugin source
reveals it performs exactly the RFC 6749 OAuth2 client credentials flow:

1. POST `{token_endpoint}` with form body `client_id={}&client_secret={}&grant_type=client_credentials`
2. Parse `$.access_token` (string) and `$.expires_in` (u64, default 1799 when absent or zero)
3. Cache token + expiry as `now + expires_in - ttl_buffer_secs` (buffer = 30s)
4. Serve cached token on subsequent calls; re-acquire when `now >= expires_at`

This is a standard RFC 6749 §4.4 client credentials exchange. There is no CrowdStrike-specific
logic in the plugin; the OAuth2 flow is vendor-agnostic. The plugin exists because ADR-023
(Plugin-Only Sensor Architecture) mandated WASM plugins for all auth acquisition that requires
an HTTP POST, regardless of whether the flow is standard.

### Armis Requires a Token Exchange Flow

ADR-053 D2 established that Armis v1 auth uses a token-exchange flow: `POST /api/v1/access_token/`
with form body `secret_key={long-lived-credential}`, response `{"success":true,"data":{"access_token":"...","expiration_utc":"..."}}`.
The acquired token is injected as `Authorization: {raw_token}` (no Bearer prefix, handled by
`header_scheme = "raw"` per ADR-053). This is structurally similar to the OAuth2 plugin pattern —
HTTP POST → token extraction → TTL-based caching — but differs in: form body field name,
JSON response path, and expiry encoding (absolute UTC string vs relative seconds).

Under ADR-023 §Rule 4 (complex sensor-specific HTTP logic belongs in WASM), a new `armis-token-exchange.prx`
WASM plugin would be required. ADR-053 v0.6 D2 proposed this plugin approach as `auth_type = "custom_via_plugin"`.

### The Plugin Approach Has Unnecessary Overhead for Standard Flows

A WASM plugin for standard HTTP auth acquisition adds complexity without behavioral benefit:
- Each plugin is a separate compilation target (`wasm32-wasip1`)
- Plugin loading adds boot-time overhead (WASM instantiation, `validate_wit_interface` check)
- Plugin KV store (for caching) is an indirection when the host could cache natively
- Plugin source code, WIT interface, cargo manifest, and wit-bindgen setup must be maintained
- Two structurally identical plugins (crowdstrike-oauth2.prx + armis-token-exchange.prx) with
  only URL, form-field, and response-path differences is redundant duplication
- ADR-023's §Rule 4 motivation was genuinely-arbitrary, sensor-specific auth logic; standard
  HTTP form-POST-for-token is not "genuinely arbitrary"

### The AuthType Enum Can Be Extended

`AuthType` in `crates/prism-spec-engine/src/spec_parser.rs` is a closed enum with variants
`Oauth2ClientCredentials`, `BearerStatic`, `CookieRoundtrip`, `ApiKey`, `CustomViaPlugin`.
ADR-026 §D3 defines the closed-enum validation rule — unknown `auth_type` strings are rejected
by E-SPEC-012 at spec-load time. Adding a `token_exchange` variant is a compile-safe,
backward-compatible extension (the enum is non-exhaustive per CLAUDE.md conventions and new
variants only add cases, not remove them).

### Auth Strategy Dispatch Is in step9a_populate_adapter_registry, Not validate_and_construct_auth_providers

There are two distinct boot-time auth construction sites:

1. **`validate_and_construct_auth_providers` (boot step 7.5b, `crates/prism-bin/src/boot.rs`)** —
   plugin-provider construction only. Iterates sensors where `auth_plugin.is_some()` and
   builds a `PluginAuthProvider` for each. Sensors without `auth_plugin` produce no entry.
   This function has NO auth_type switch; it is driven exclusively by `auth_plugin` presence.

2. **`step9a_populate_adapter_registry` (`crates/prism-bin/src/spec_driven_adapter.rs`)** —
   the real auth_type-keyed dispatch site. Contains:
   `match resolved_spec.spec.auth_type { CustomViaPlugin | Oauth2ClientCredentials | BearerStatic | CookieRoundtrip | other => ... }`.
   Each arm constructs the appropriate auth strategy per resolved (org, sensor) spec. The
   `Oauth2ClientCredentials` arm currently: fetches the global `PluginAuthProvider` from
   step 7.5b, then builds a PER-ORG `PluginAuthProvider` using
   `format!("{}/oauth2/token", resolved_spec.spec.base_url)` — explicitly using the
   per-org overlay-resolved `base_url` to avoid posting to the wrong region/tenant endpoint.

For declarative auth types (`oauth2_client_credentials` with `[auth_acquisition]` block,
and the new `token_exchange`), the migration rewrites step 9A's `Oauth2ClientCredentials`
arm to construct a `DeclarativeHttpAuthProvider` directly from `[auth_acquisition]`, and
adds a new `TokenExchange` arm. The per-org derivation of the token URL is preserved — the
provider receives `base_url + token_path` (per-org resolved) at step 9A construction time.
`validate_and_construct_auth_providers` is NOT the target of this change.

### Human Decision (D-1895)

The human architect ruled: Armis auth MUST NOT require a plugin. The TOML spec engine MUST be
completed to express standard HTTP auth acquisition declaratively. `crowdstrike-oauth2.prx` MUST
be retired. `custom_via_plugin` MUST be preserved as an escape hatch for genuinely non-standard
auth (e.g., multi-step OAuth2 with out-of-band device codes, vendor-specific challenge-response
flows). This rules out the ADR-053 v0.6 D2 `custom_via_plugin` + armis-token-exchange.prx approach.

---

## Decision

### D1 — Add `token_exchange` to the AuthType Closed Enum (amends ADR-026 §D3 partial)

A new `AuthType::TokenExchange` variant is added to the closed enum in
`crates/prism-spec-engine/src/spec_parser.rs`. The `VALID_AUTH_TYPES` const gains the
`"token_exchange"` string. E-SPEC-012 closed-enum validation continues to reject unknown strings;
`"token_exchange"` is now a valid value.

`token_exchange` semantics: the sensor spec engine performs a native HTTP POST to the per-org
derived token URL (`base_url + [auth_acquisition].token_path`) using a single form field
supplied by a credential reference.
The response is a JSON object; the token is extracted at a declared dotted path; expiry is
an absolute UTC timestamp at a declared dotted path. The acquired token is cached in the
`DeclarativeHttpAuthProvider`'s in-memory token store (no plugin KV).

The `token_exchange` auth_type REQUIRES an `[auth_acquisition]` block in the TOML spec.
Absent block → E-SPEC-028(a) (see D10).

### D2 — `oauth2_client_credentials` Becomes Native (amends ADR-023 §Rule 4 for standard flows)

`auth_type = "oauth2_client_credentials"` without an `auth_plugin` field now routes to
`DeclarativeHttpAuthProvider(Oauth2ClientCredentials)` at boot time (see D4). The `auth_plugin`
field MUST NOT be present for `auth_type ∈ {oauth2_client_credentials, token_exchange}` —
the combination is a D10 E-SPEC-028(b) validation error **regardless of whether `[auth_acquisition]`
is declared**. A spec with either of these auth_types and `auth_plugin` present is rejected at
spec-load time. (See D10(b) for the error message; D7 for the unreachable dispatch implication.)

`oauth2_client_credentials` semantics in declarative mode: the engine performs an RFC 6749 §4.4
client credentials POST. Form body: `client_id={}&client_secret={}&grant_type=client_credentials`
(values URL-form-encoded per RFC 3986 §2.3; field order matches the plugin implementation in
`crowdstrike-oauth2/src/lib.rs` `acquire_token()`). **Wire-encoding note (RU-Q3):** reqwest
`.form()` (internally `serde_urlencoded`) encodes `+`, `=`, and `/` identically to non-alphanumeric
percent-encoding for these payload values; parity with the reference plugin implementation is
preserved. Response: `$.access_token` (string, required);
`$.expires_in` — **lenient deserialization (RU-Q2):** accept JSON number (`as_u64()`) OR numeric
string (attempt `str.parse::<u64>()`); non-numeric string or any other wrong type treated as absent;
default 1799 when absent, zero, or unparseable (RFC 6749 §5.1 does not fix the JSON type; confirmed
providers such as Microsoft Entra ID emit `"3599"` as a string); `ttl_buffer_secs` (default 30)
subtracted from the computed `expires_at`. Credential refs MUST include one named `client_id`
and one named `client_secret` — validated by E-SPEC-028(f) at spec-load time.

**CrowdStrike migration:** `crowdstrike.sensor.toml` drops `auth_plugin = "crowdstrike-oauth2"`
and gains:

```toml
auth_type = "oauth2_client_credentials"
# auth_plugin removed — declarative native provider per ADR-054 D2

[auth_acquisition]
token_path = "/oauth2/token"
# Token URL derived per-org at step9a_populate_adapter_registry:
#   format!("{}{}", resolved_spec.spec.base_url, "/oauth2/token")
# Per-org base_url overlays (e.g., DTU clone at "http://127.0.0.1:<port>") flow through
# automatically — no global env var required.
# Body: client_id={}&client_secret={}&grant_type=client_credentials
# Token: $.access_token; TTL: $.expires_in.saturating_sub(30s) (default 1799s on absent/zero)

[[credential_refs]]
name = "client_id"
description = "CrowdStrike OAuth2 client ID"

[[credential_refs]]
name = "client_secret"
description = "CrowdStrike OAuth2 client secret"
```

The credential_refs names `client_id` and `client_secret` are unchanged from the current spec.
This migration preserves the per-org token endpoint derivation that the existing
`Oauth2ClientCredentials` arm in `step9a_populate_adapter_registry` already performs
(`format!("{}/oauth2/token", resolved_spec.spec.base_url)` with per-org resolved `base_url`).
The `DeclarativeHttpAuthProvider` replicates the RFC 6749 logic from `crowdstrike-oauth2.prx`
(`acquire_token` + `get_token`, TTL semantics, URL-form-encoded body), constructed per
(org, sensor) resolved spec at step 9A.

### D3 — `[auth_acquisition]` TOML Block Schema

The `[auth_acquisition]` sub-table is added to `SensorSpec`. In Rust:
`auth_acquisition: Option<AuthAcquisitionConfig>`. `token_path` is a literal relative path
string — no env-var interpolation. Env-var interpolation (Rule 6) applies to `base_url` in
the parent `SensorSpec`, which is used to derive the full token URL at step 9A.

**Fields common to both auth types:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `token_path` | string | YES | — | Path component of the token endpoint, joined with the per-org resolved `base_url` at step 9A: `format!("{}{}", resolved_spec.spec.base_url, token_path)`. Ensures per-org `base_url` overlays (DTU clone endpoints, multi-region) flow through to the token POST target. |
| `ttl_buffer_secs` | u64 | no | 30 | Seconds to subtract from expiry (via `saturating_sub`) for early-renewal buffer |

> **Per-org derivation invariant:** `token_url` is NOT a field in `[auth_acquisition]`. The
> full token URL is always derived at adapter-construction time in `step9a_populate_adapter_registry`
> as `format!("{}{}", resolved_spec.spec.base_url, auth_acquisition.token_path)`. This mirrors the
> existing `Oauth2ClientCredentials` arm's per-org derivation
> (`format!("{}/oauth2/token", resolved_spec.spec.base_url)`) and preserves multi-tenant and
> multi-region correctness — per-org `base_url` overlays (E-SPEC-023 allowed keys: `extends`,
> `instance_id`, `base_url`, `timeout_secs`, `rate_limit_hints`) flow through automatically.

**Additional fields for `token_exchange` only:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `credential_body_field` | string | YES | — | Form field name AND the credential_ref name supplying its value |
| `token_response_path` | string | YES | — | Dotted JSON path to the access token in the response (e.g., `data.access_token`) |
| `expiry_field` | string | YES | — | Dotted JSON path to the expiry value in the response (e.g., `data.expiration_utc`) |
| `expiry_mode` | string enum | YES | — | `"absolute_utc_string"` or `"relative_seconds"` |

For `oauth2_client_credentials`: no additional `[auth_acquisition]` fields are needed.
The form body, response parsing, and TTL computation are fixed by RFC 6749 (engine-internal).

**Armis wiring (`token_exchange`):**

```toml
sensor_id = "armis"
auth_type = "token_exchange"       # native declarative provider (ADR-054 D1)
header_scheme = "raw"              # Authorization: {token} — no Bearer prefix (ADR-053 D2)
# base_url = "${env.ARMIS_INSTANCE_URL}" (per-org resolved; overlays flow to token URL)

[auth_acquisition]
token_path = "/api/v1/access_token/"
# Token URL derived per-org: base_url + "/api/v1/access_token/"
credential_body_field = "secret_key"        # form body: secret_key={resolved_value}
token_response_path = "data.access_token"   # $.data.access_token
expiry_field = "data.expiration_utc"        # $.data.expiration_utc (absolute UTC string)
expiry_mode = "absolute_utc_string"
# ttl_buffer_secs = 30 (default)

[[credential_refs]]
name = "secret_key"
description = "Armis long-lived secret key for token exchange"
# Resolved via BC-2.06.003 four-tier per-client chain (PRISM_CLIENTS_{ID}_SENSORS_ARMIS_SECRET_KEY)
```

### D4 — `DeclarativeHttpAuthProvider` Implementation Contract

`DeclarativeHttpAuthProvider` is a new Rust struct in `crates/prism-spec-engine/src/auth/`.
It implements the `AuthProvider` trait (`crates/prism-spec-engine/src/auth_provider.rs`;
defines `acquire_token` — the trait `PluginAuthProvider`, `BearerStaticCredentialAuthProvider`,
and `StaticCookieAuthProvider` also implement). The construction site for
`DeclarativeHttpAuthProvider` instances is `step9a_populate_adapter_registry` in
`crates/prism-bin/src/spec_driven_adapter.rs` — not `validate_and_construct_auth_providers`
(boot step 7.5b), which handles plugin-auth providers and is not modified by this ADR.

**Internal state:**
- `config: AuthAcquisitionConfig` — from the sensor spec's `[auth_acquisition]` block
- `credential_resolver: Arc<dyn CredentialResolver>` — for resolving credential_ref values
  at token-acquisition time (lazy, per AD-017: never at construction)
- `cached_token: ArcSwap<Option<CachedAuthToken>>` — in-memory token cache (no plugin KV store)
- `token_url: String` — per-org derived token URL, computed at step 9A as
  `format!("{}{}", resolved_base_url, config.token_path)` and passed to the constructor;
  immutable after construction (OBS-P13-001 — was referenced in algorithm prose and
  BC-2.16.014 P1/INV-014-002 but absent from this field enumeration prior to v0.40)
- `http_client: reqwest::Client` — ADR-050-compliant client (rustls-tls, 30s timeout),
  constructed internally inside `new()` using `build_http_client_with_timeout()` (confirmed
  `pub(crate)` helper in `crates/prism-spec-engine/src/pipeline.rs`, closed TD-S-PLUGIN-PREREQ-B-005);
  NOT exposed in the production constructor API — no test-only HTTP injection seam
  (F-WASE-P13-MED-001; see Constructor note below)
- `now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` — returns current Unix timestamp (seconds
  since UNIX_EPOCH); production default: `SystemTime::now().duration_since(UNIX_EPOCH)
  .unwrap_or_default().as_secs()`; overridable in test builds exclusively via the
  `new_for_test` constructor gated `#[cfg(any(test, feature = "test-helpers"))]` — this
  is the SOLE test seam; the HTTP client (`self.http_client`) is NOT injectable
  (F-WASE-P13-MED-001)

**CachedAuthToken** holds:
- `token: String` — the acquired access token (opaque bytes; never logged per AD-017)
- `expires_at: u64` — Unix timestamp after which the token must be re-acquired

**`acquire_token()` (force-refresh, bypasses cache):**
1. Resolve credential(s) from the credential store (lazy per AD-017 — no credential access at boot)
2. Build form body (RFC-3986 percent-encoded):
   - `oauth2_client_credentials`: `client_id={}&client_secret={}&grant_type=client_credentials`
   - `token_exchange`: `{credential_body_field}={resolved_value}`
3. POST to `self.token_url` (per-org derived URL, stored immutably at construction)
   with `Content-Type: application/x-www-form-urlencoded`,
   using `self.http_client` (internally constructed per ADR-050: rustls-tls, 30s timeout;
   not exposed in the production constructor — see Constructor note below)
4. Parse response:
   - `oauth2_client_credentials`: extract `$.access_token`; compute TTL from `$.expires_in` (default 1799 if absent/zero) minus `ttl_buffer_secs`
   - `token_exchange`: extract at `token_response_path`; parse expiry at `expiry_field` per `expiry_mode`:
     - `"relative_seconds"`: u64 seconds, default 1799, minus `ttl_buffer_secs`
     - `"absolute_utc_string"`: lenient `s.parse::<DateTime<FixedOffset>>()` (chrono relaxed
       `FromStr` — accepts both `T`-separator ISO-8601 `"2099-01-01T00:00:00Z"` AND
       space-separated `"YYYY-MM-DD HH:MM:SS.ffffff+HH:MM"`; rejects only if even relaxed parse
       fails → `E-AUTH-001`) → `.timestamp() as u64` → Unix timestamp, minus `ttl_buffer_secs`.
       **Adjudication rationale (RU-Q1/Option B):** production Armis backend most likely emits
       space-separated UTC strings (Python backend convention; XSOAR uses format-agnostic
       dateparser); strict `parse_from_rfc3339` rejects space-separated values per chrono
       Context7 docs; relaxed chrono `FromStr` accepts both forms. Option B (proactive TTL
       via lenient parse + reactive P6 401-backstop) is chosen over Option A (reactive re-auth
       only) because P2/P4/AC-6 are designed around proactive TTL; the 401 backstop (P6) is
       preserved as defense-in-depth regardless.
5. Store `CachedAuthToken` in `cached_token` ArcSwap
6. Return `Ok(token_string)`

**`get_token()` (cache-aware):**
1. Load `cached_token` snapshot
2. If `Some(cached)` and `(self.now_fn)() < cached.expires_at` and `!cached.token.is_empty()` → return cached token (zero network calls)
3. Otherwise → call `acquire_token()` (refreshes cache atomically via ArcSwap)

**Constructor (F-WASE-P13-MED-001):**

Production constructor:
```rust
pub fn new(
    token_url: String,                          // step 9A: format!("{}{}", resolved_base_url, config.token_path)
    config: AuthAcquisitionConfig,
    credential_resolver: Arc<dyn CredentialResolver>,
) -> Self
```
Constructs `self.http_client` internally via `build_http_client_with_timeout()` (ADR-050;
no HTTP client injection seam — VP-159 uses wiremock to intercept the token endpoint by
setting `token_url = wiremock_server_uri + token_path` at test construction time).
Sets `self.now_fn` to the real-time function. The `token_url` parameter is computed
externally at `step9a_populate_adapter_registry` and passed in.

Test constructor (gated — NOT in production builds):
```rust
#[cfg(any(test, feature = "test-helpers"))]
pub fn new_for_test(
    token_url: String,
    config: AuthAcquisitionConfig,
    credential_resolver: Arc<dyn CredentialResolver>,
    now_fn: Arc<dyn Fn() -> u64 + Send + Sync>,
) -> Self
```
Accepts a custom clock function (typically wrapping an `Arc<AtomicU64>`) for deterministic
TTL testing. Tests advance the clock via `Arc<AtomicU64>::fetch_add()` between calls.
This is the ONLY non-production seam; `self.http_client` is ALWAYS internally constructed.

**`AuthProvider` trait extension — `get_token()` addition (F-WASE-P7-HIGH-001):**

`get_token()` is added to the `AuthProvider` trait in `crates/prism-spec-engine/src/auth_provider.rs`
with a **default implementation** that delegates to `self.acquire_token(spec, client_id)`. This
preserves the existing behavior of all 7 current `AuthProvider` implementors (`NullAuthProvider`,
`MockAuthProvider`, `StaticCookieAuthProvider`, `FailingAuthProvider`, `ChainAuthProvider`,
`BearerStaticCredentialAuthProvider`, `PluginAuthProvider`) without any code changes — each
inherits the default, which calls its respective `acquire_token()` unchanged.
`DeclarativeHttpAuthProvider` [PLANNED — engine story] OVERRIDES `get_token()` with the ArcSwap
cache-aware logic described above.

The method signature is identical to `acquire_token()` for object-safety compliance
(same `Pin<Box<dyn Future<...> + Send + 'a>>` return type, same lifetime bounds):

```rust
fn get_token<'a>(
    &'a self,
    spec: &'a SensorSpec,
    client_id: &'a OrgSlug,
) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>> {
    self.acquire_token(spec, client_id) // default: delegates to force-refresh
}
```

**`PipelineExecutor` call-site dispatch — lifecycle algorithm:**

| Call site | Method called | Rationale |
|-----------|---------------|-----------|
| `execute_impl` — eager acquisition before the `'steps:` loop (`crates/prism-spec-engine/src/pipeline.rs`) | `get_token()` | Cache-aware: warm pipeline executions return the cached token with zero token-POST overhead; preserves the eager-before-steps design that closed TD-S-PLUGIN-PREREQ-B-010 |
| `execute_step` — eager acquisition before single-request dispatch (`crates/prism-spec-engine/src/pipeline.rs`) | `get_token()` | Same rationale as `execute_impl`; symmetric eager-acquisition semantics across both executor entry points |
| `issue_request_with_retry` — 401-refresh path (inner loop, called from `execute_impl`) | `acquire_token()` | Force-refresh required: the sensor endpoint rejected the cached token; calling `get_token()` would return the same rejected (stale or revoked) token from the warm cache; BC-2.16.014 P5/P6 explicitly require `acquire_token()` here |

The `auth_refresh_triggered` structured event fires only on HTTP 401 mid-pipeline (legitimate token
expiry or revocation), not on every pipeline start. Warm pipeline executions (within the token's TTL
window) issue zero token-POST requests — restoring the caching behavior of the
`crowdstrike-oauth2.prx` plugin (to be retired by the §D2/§D5 migration story) and fulfilling the performance commitment of §D2 and BC-2.16.014
P2/P3. Behavioral anchors: `let mut bearer_token = match auth_provider.` block before the `'steps:`
loop in `execute_impl`; `let bearer_token = match auth_provider.` block at the top of
`execute_step`; `let fresh_token = match auth_provider.acquire_token` block in the
`UNAUTHORIZED` arm of `issue_request_with_retry` (TD-VSDD-091: no line pins).

**Error handling:** Acquisition-level failures propagate as `SpecEngineError::AuthAcquisitionFailed`
(E-AUTH-001 — the existing error variant for token acquisition failures; no new error variant
needed). Double-401 from the sensor endpoint — PipelineExecutor retries once after a 401; a
second consecutive 401 — propagates as `SpecEngineError::AuthRefreshFailed` (E-AUTH-002) per P6.
E-SPEC-028 covers spec-load validation failures only.

**Security:** Credential values are resolved at `acquire_token()` time, not at construction.
They are NEVER stored beyond the scope of the HTTP POST call. Token strings are cached but
never logged. AD-017 opaque-credential model applies.

### D5 — Retire crowdstrike-oauth2.prx

`crates/prism-spec-engine/plugins/crowdstrike-oauth2/` is retired. The retirement story:
1. Removes `auth_plugin = "crowdstrike-oauth2"` from `crates/prism-sensors/specs/crowdstrike.sensor.toml`
2. Adds the `[auth_acquisition]` block to `crowdstrike.sensor.toml` (D2 above)
3. Deletes `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` (source, Cargo.toml, WIT)
4. Removes the crowdstrike-oauth2 workspace member from `Cargo.toml`
5. Implements `DeclarativeHttpAuthProvider` (D4)
6. Rewrites the `Oauth2ClientCredentials` arm of `step9a_populate_adapter_registry` in
   `crates/prism-bin/src/spec_driven_adapter.rs` to construct a per-org
   `DeclarativeHttpAuthProvider` from `[auth_acquisition]` — replacing the existing per-org
   `PluginAuthProvider` construction (which looks up the global provider from step 7.5b and
   builds a per-org `PluginAuthProvider` with token endpoint `base_url + "/oauth2/token"`).
   Adds a new `TokenExchange` arm for the same native provider.
   `validate_and_construct_auth_providers` (boot step 7.5b) is NOT modified — CrowdStrike will
   no longer have an `auth_plugin` field, so it produces no entry there (correct behavior).

The WASM plugin infrastructure (PluginRuntime, WIT interfaces, KV store, manifest loader) is
NOT retired — it remains for `custom_via_plugin` sensors. Only the `crowdstrike-oauth2.prx`
plugin binary and its source crate are removed.

### D6 — `custom_via_plugin` Preserved as Escape Hatch

`auth_type = "custom_via_plugin"` + `auth_plugin = "<plugin-id>"` remains the mechanism for
sensors with genuinely non-standard auth that cannot be expressed declaratively (e.g., multi-step
OAuth2 with device flow, challenge-response auth, vendor-specific signed-request auth). The
`PluginAuthProvider` construction path in `boot.rs` is unchanged. No existing `custom_via_plugin`
sensor is broken. The Armis remediation story no longer needs to author a new plugin.

### D7 — Updated Auth Strategy Dispatch in step9a_populate_adapter_registry

The auth_type-keyed dispatch lives in `step9a_populate_adapter_registry` in
`crates/prism-bin/src/spec_driven_adapter.rs` (NOT in `boot.rs::validate_and_construct_auth_providers`,
which is plugin-provider construction only — see Context §Auth Strategy Dispatch).

> **`validate_and_construct_auth_providers` note:** This function (boot step 7.5b, `boot.rs`)
> iterates sensors with `auth_plugin.is_some()` and builds `PluginAuthProvider` instances. It
> has no `auth_type` switch and is NOT modified by this ADR. After CrowdStrike migration (D5),
> `crowdstrike.sensor.toml` has no `auth_plugin` field, so CrowdStrike produces no step 7.5b
> entry — correct behavior; it no longer needs a plugin provider.

**step9a_populate_adapter_registry auth dispatch table (after ADR-054 migration):**

| `auth_type` match arm | Auth strategy constructed at step 9A |
|-----------------------|--------------------------------------|
| `Oauth2ClientCredentials` AND `auth_acquisition.is_some()` (post-migration) | Construct `DeclarativeHttpAuthProvider(Oauth2ClientCredentials)` with `token_url = base_url + token_path` — **new behavior; replaces per-org `PluginAuthProvider` construction** |
| `Oauth2ClientCredentials` AND `auth_plugin.is_some()` | **[VALIDATION-UNREACHABLE — defense-in-depth only]** D10(b) rejects any spec with `auth_type = oauth2_client_credentials` AND `auth_plugin` present before step 9A executes; no valid spec reaches this arm. Retained as defense-in-depth for test paths that bypass spec validation. |
| `TokenExchange` | Construct `DeclarativeHttpAuthProvider(TokenExchange)` with `token_url = base_url + token_path` — **new arm** |
| `CustomViaPlugin` | Look up pre-built `PluginAuthProvider` from step-7.5b map (unchanged) |
| `BearerStatic` | Construct `BearerStaticCredentialAuthProvider` (resolves bearer token from credential store at acquire_token time; no HTTP call — unchanged) |
| `CookieRoundtrip` | Construct `StaticCookieAuthProvider` (unchanged) |
| `ApiKey` / `other` | No dedicated arm — falls into `other =>` branch → logs E-SPEC-012 and skips adapter registration (EC-007; api_key unimplemented at current scope) |

The `auth_type × auth_plugin × auth_acquisition` coherence check runs at spec-load time (BC-2.16.009
Rule 10, see D10 E-SPEC-028) before step 9A. By the time step 9A runs, specs are guaranteed valid.

> **Rule numbering:** ADR-053 D2 reserves Rule 9 for `header_scheme` value validation. This ADR's
> `[auth_acquisition]` coherence check is therefore Rule 10 — the next available rule number in
> BC-2.16.009's sequential rule set.

> **Story sequencing dependency:** ADR-054's CrowdStrike-retirement / Armis-token-exchange
> implementation stories MUST merge AFTER the ADR-053 standalone Wave-A engine story that
> delivers `SensorSpec::header_scheme` and authors BC-2.16.009 Rule 9 / E-SPEC-027. Rule 10
> (this ADR's `[auth_acquisition]` coherence check, E-SPEC-028) runs in the same
> `BC-2.16.009` validation pass and depends on the `spec_parser.rs` extension authored in
> that engine story. The story-writer MUST encode this as an explicit story-level merge
> dependency in the Wave-A dependency graph. **Coherence matrix scope boundary:** the
> standalone engine story authors E-SPEC-027 and the `auth_type × header_scheme` coherence
> matrix rows for the **5 existing variants** (bearer_static, oauth2_client_credentials,
> cookie_roundtrip, custom_via_plugin, api_key) only — `AuthType::TokenExchange`, its
> coherence-matrix row, and its E-SPEC-027 `{allowed_set}` entry ship atomically with the
> ADR-054 story (lands second), ensuring no forward-reference where a coherence row predates
> its enum variant.

### D8 — BC-2.16.014: Declarative Auth Acquisition Token Lifecycle

> **[AUTHORED — D-1946 2026-07-22]** `BC-2.16.014` was authored by the product-owner during
> Wave-A spec evolution burst 1 (decision D-1946). The postconditions P1–P8 below served as the
> **authoring source**; `BC-2.16.014` is now the authoritative behavioral contract.
> File: `.factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md`

A new BC `BC-2.16.014` will be authored covering the behavioral contract for
`DeclarativeHttpAuthProvider`:

**Preconditions:**
- Sensor spec has `auth_type ∈ {oauth2_client_credentials (declarative), token_exchange}`
- `[auth_acquisition]` block is present and validated by E-SPEC-028 at spec-load time
- `DeclarativeHttpAuthProvider` is constructed per (org, sensor) during boot step 9A
  (`step9a_populate_adapter_registry` in `spec_driven_adapter.rs`)

**Postconditions (summary — BC-2.16.014 is authoritative; this section is the historical authoring source):**
- P1: `DeclarativeHttpAuthProvider::new()` makes ZERO network calls (lazy acquisition invariant)
- P2: First `get_token()` call issues exactly ONE HTTP POST to the derived token URL (`base_url + token_path`, stored in provider at construction) and caches the result
- P3: Subsequent `get_token()` calls within TTL return the cached token without issuing an HTTP request
- P4: `get_token()` call when `unix_now() >= expires_at` issues exactly ONE HTTP POST (re-acquisition)
- P5: `acquire_token()` always issues exactly ONE HTTP POST (cache bypass, force-refresh)
- P6: On HTTP 401 from a sensor endpoint, `PipelineExecutor` calls `acquire_token()` (force-refresh);
  on second consecutive 401, the request fails with `AuthRefreshFailed` (no infinite retry)
- P7: Credential values are NEVER stored in the `CachedAuthToken` — only the opaque token string
  and expiry timestamp are cached (AD-017)
- P8: `base_url` env-var interpolation obeys BC-2.16.009 Rule 6 (unresolved var → E-SPEC-024 at spec-load); `token_path` is a literal relative path and does not undergo env-var interpolation

### D9 — VP-159: Lazy Acquisition and Refresh-on-Expiry Invariants

> **[AUTHORED — D-1947 2026-07-22]** `VP-159` was registered in `VP-INDEX.md` by the architect
> during Wave-A spec evolution burst 2 (decision D-1947); authored after BC-2.16.014 (burst 1, D-1946).
> The properties listed below served as the **authoring source**; VP-159 is now registered as DRAFT.
> File: `.factory/specs/verification-properties/vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md`

Verification property `VP-159` covers the network-call invariants of `DeclarativeHttpAuthProvider`:

- **Module:** `prism-spec-engine`
- **Tool:** `integration_test` (wiremock network-level interception — token_url routed to MockServer; now_fn clock seam via new_for_test for TTL determinism)
- **Phase:** P1
- **BC:** BC-2.16.014 (primary; forward reference — see D8)
- **Properties:**
  - `acquire_token()` makes exactly one HTTP POST (no cached-token bypass)
  - `get_token()` on cold cache → exactly one HTTP POST; on warm cache → zero HTTP POSTs
  - `get_token()` on stale cache (expired TTL) → exactly one HTTP POST (re-acquisition)
  - `get_token()` on cache-hit with empty token string → exactly one HTTP POST (same as cold cache)
  - TTL arithmetic for `absolute_utc_string` expiry mode: `expires_at = expiry_str.parse::<DateTime<FixedOffset>>().map(|dt| dt.timestamp() as u64).saturating_sub(ttl_buffer_secs)` (lenient chrono relaxed `FromStr` — see §D4 step 4 v0.51)
  - TTL arithmetic for `relative_seconds` expiry mode: `expires_at = now + expires_in.saturating_sub(ttl_buffer_secs)` where `expires_in` is defaulted to 1799 when absent or zero (matches the plugin's `saturating_sub(30)` arithmetic; `.max(1)` is omitted as dead code when the absent/zero default is already 1799)
  - Credential values are not stored in `CachedAuthToken` (AD-017 assertion)

VP-159 is registered as DRAFT; it will be promoted to ACTIVE when the implementation story (D5 retirement story)
ships and the integration tests are green.

### D10 — E-SPEC-028: Declarative Auth Acquisition Validation Errors

`E-SPEC-028` (next free after E-SPEC-027 reserved by ADR-053) is registered in `error-taxonomy.md`
covering validation errors for `[auth_acquisition]` blocks. Validation runs in the same multi-error
pass as other spec-file validation rules (BC-2.16.009 **Rule 10** — after ADR-053 D2's Rule 9 for
`header_scheme` validation). Spec rejected on any E-SPEC-028; boot fails exit code 2.

**Rule 10 execution site and ADR-055 reconciliation (authoritative):**

Rule 10 executes inside `SpecLoader::parse()` — not inside `validate_sensor_spec()`. The
`S-WAVE-A-ENGINE-001` story implementation adds Rules 9 and 10 as the final gates inside
`SpecLoader::parse()`, after the existing timestamp-format and Rule 8 `probe_table` gates,
immediately before `Ok(spec)` is returned. This is the same execution site as Rule 8
(`probe_table` reference, E-SPEC-026) and Rule 9 (`header_scheme` validation, E-SPEC-027).

**Rationale — why `SpecLoader::parse()`, not `validate_sensor_spec()`:**

1. **BC-2.16.009 §Integration function** is explicit: "The S-WAVE-A-ENGINE-001 implementation
   adds Rules 9 and 10 inside `SpecLoader::parse()` — not inside `validate_sensor_spec()` —
   ensuring they execute on every path that calls `parse()`."

2. **Rule 10 is interpolation-independent.** ADR-054 §D3 states that `token_path` is "a
   literal relative path string — no env-var interpolation." All eight sub-conditions of Rule
   10 check only literal TOML fields and structural block presence — none reads `base_url` or
   any env-var-interpolated value. Per-sub-condition verification:
   - **(a)** `auth_type` (literal) + `[auth_acquisition]` block presence (structural) +
     `token_path` (literal, interpolation-free per §D3): interpolation-independent.
   - **(b)** `auth_type` (literal) + `auth_plugin` presence (structural):
     interpolation-independent.
   - **(c)** `expiry_mode` string value (literal TOML field): interpolation-independent.
   - **(d)** token_exchange required-field presence (`credential_body_field`,
     `token_response_path`, `expiry_field`, `expiry_mode` — all literal TOML fields):
     interpolation-independent.
   - **(e)** `credential_body_field` value (literal) vs `[[credential_refs]]` entry names
     (literal TOML array entries): interpolation-independent.
   - **(f)** `[[credential_refs]]` entry names `client_id`/`client_secret` (literal TOML
     array entries): interpolation-independent.
   - **(g)** `[auth_acquisition]` block presence (structural) vs `auth_type` (literal):
     interpolation-independent.
   - **(h)** token_exchange-only field names present (structural/literal) vs `auth_type`
     (literal): interpolation-independent.

3. **ADR-055 §D3 scoping.** ADR-055 §D3 (status: accepted) argues against folding
   `validate_sensor_spec()` into `SpecLoader::parse()` because `parse()` runs before
   `resolve_env_var_tokens()`, so Rules 1–5 would receive unresolved `${env.VAR}` tokens
   in `base_url` and incorrectly reject the canonical sensor suite (Rule 1 checks `base_url`
   scheme; `base_url` IS env-var-interpolated in all four canonical sensor specs). That
   env-var ordering constraint is factually inapplicable to Rule 10, which never inspects
   `base_url` or any interpolated value. ADR-055 §D3 carries an explicit scoping note
   confirming that its env-var ordering argument is scoped to Rules 1–5 and does not extend
   to Rules 8, 9, or 10. This factual characterization of Rule 10 holds independent of
   ADR-055's ratification status.

4. **Bypass risk if placed in `validate_sensor_spec()`.** `validate_sensor_spec()` has zero
   production callers on all three production spec-loading surfaces confirmed in ADR-055
   §Context: `SpecLoader::load_all()`, `process_spec_changes()`, and `add_sensor_spec()` all
   invoke `SpecLoader::parse()` directly; none invokes `validate_sensor_spec()`. An
   implementation that places Rule 10 inside `validate_sensor_spec()` makes E-SPEC-028
   unreachable from the `add_sensor_spec` MCP write surface and from hot-reload — the two
   write surfaces where a malformed or adversarial `[auth_acquisition]` block enters the
   system. BC-2.16.009 §Security requirement explicitly identifies this as the bypass class
   that Rules 8/9/10 were placed inside `parse()` to prevent.

**Message templates:**

> **Canonical source for message templates (F-WASE-P2-MED-001):** `error-taxonomy.md` E-SPEC-028 is the canonical source of truth for message template text. The text in §D10 sub-conditions below represents condition-authoring intent (trigger logic, parameter substitution, sub-condition labeling). On any wording-only conflict between §D10 and the taxonomy, the taxonomy wins. Exception: sub-condition (b) is explicitly adjudicated below (F-WASE-P2-HIGH-001); §D10(b) is the authoritative semantic source and the taxonomy must be updated to match it.

**(a) Required block absent:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' requires an [auth_acquisition] block with token_path. Add an [auth_acquisition] block."`
Fires when: `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND (`[auth_acquisition]` absent OR `token_path` absent).

**(b) Conflicting auth_plugin:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."`

**Fires when:** `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` is present — regardless of whether `[auth_acquisition]` is declared.

> **F-WASE-P2-HIGH-001 adjudication (2026-07-22) — §D10(b) is the authoritative source; taxonomy must follow:**
>
> Two conflicting trigger definitions existed across the package:
>
> - **Definition 1** (this §D10(b), §D2, §D7, BC-2.16.014 Precondition 5): fires when `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` present — **regardless of `[auth_acquisition]`**.
> - **Definition 2** (error-taxonomy.md E-SPEC-028(b) prior to this adjudication, marked "UNCONDITIONAL"): fires when `auth_plugin` AND `[auth_acquisition]` BOTH declared — regardless of `auth_type`.
>
> **Definition 1 is canonical.** Four independent reasons:
>
> 1. **§D2 design intent is explicit and unambiguous:** "The `auth_plugin` field MUST NOT be present for `auth_type ∈ {oauth2_client_credentials, token_exchange}` ... regardless of whether `[auth_acquisition]` is declared." Definition 2 contradicts this.
>
> 2. **§D7 validation-unreachable claim depends on Definition 1:** The §D7 dispatch table annotates the `Oauth2ClientCredentials AND auth_plugin.is_some()` arm as "VALIDATION-UNREACHABLE — defense-in-depth only" because D10(b) rejects that spec before step 9A runs. Under Definition 2, a spec with `auth_type = oauth2_client_credentials` + `auth_plugin` + **no** `[auth_acquisition]` would NOT be caught by (b) — only by (a). That leaves the §D7 unreachable annotation incomplete. Under Definition 1, D10(b) fires regardless, maintaining the §D7 guarantee.
>
> 3. **Definition 2 is redundant given (g):** `auth_plugin` + `[auth_acquisition]` on non-declarative auth_types (`bearer_static`, `cookie_roundtrip`, `api_key`, `custom_via_plugin`) is already fully caught by sub-condition (g) (`[auth_acquisition]` present for non-declarative auth_type). Definition 2 would make (b) a partial duplicate of (g) for those cases, while still missing the critical declarative-type + auth_plugin case where `[auth_acquisition]` is absent.
>
> 4. **The message template is auth_type-centric:** the message text "auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin" signals the auth_type as the discriminating criterion, not the presence of `[auth_acquisition]`. Consistent only with Definition 1.
>
> **Required error-taxonomy.md update (PO sweep — immediately after this ADR fix-burst, F-WASE-P2-HIGH-001):**
> - Replace E-SPEC-028(b) `message_template` with: `"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."`
> - Replace E-SPEC-028(b) Description sub-condition text with: fires when `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` is present — regardless of whether `[auth_acquisition]` is declared.
> - Remove the "UNCONDITIONAL (fires whenever both coexist, regardless of auth_type)" language — that described Definition 2 and is superseded.
>
> §D2, §D7, and §D10(b) are all consistent under Definition 1.

**(c) Unknown expiry_mode:**
`"sensor '{sensor_id}': [auth_acquisition].expiry_mode = '{value}' is not valid. Accepted values: absolute_utc_string, relative_seconds."`
Fires for `token_exchange` when `expiry_mode` is present but not in the two-value closed set.

> **expiry_mode ratified value set (Wave-A burst 3 adjudication — source of truth):** The two valid
> values are `absolute_utc_string` (UTC string, parsed at acquire_token time via lenient chrono relaxed
> `FromStr` to Unix timestamp per §D4 v0.51) and `relative_seconds` (u64 seconds TTL). All three
> design-authority sites are self-consistent: D3 field table (`"absolute_utc_string"` or
> `"relative_seconds"`), D4 algorithm (`absolute_utc_string`: lenient chrono relaxed `FromStr` per
> v0.51; `relative_seconds`: u64 default 1799), and the Armis wiring example
> (`expiry_mode = "absolute_utc_string"`). BC-2.01.008's use of `expiry_mode = "absolute_utc_string"`
> is CORRECT per D3. Note: error-taxonomy.md v2.57 E-SPEC-028(c) and BC-2.16.009 Rule 10(c)/EC-009-038
> erroneously listed `absolute_epoch_secs, ttl_secs` — these were PO authoring errors from burst 3,
> corrected in D-1948 (error-taxonomy.md v2.58, BC-2.16.009 v1.13); see D11 [COMPLETED — D-1948] rows.

> **F-WASE-P40-MED-001 adjudication (2026-07-23) — token_exchange-gating is canonical; §D10(c) is the authoritative trigger-logic source:**
>
> Three artifacts state divergent trigger scopes for sub-condition (c):
>
> - **§D10(c) (this section):** fires for `token_exchange` when `expiry_mode` is present but not in the two-value closed set — **token_exchange-gated**.
> - **BC-2.16.009 Rule 10(c):** "If `[auth_acquisition]` is present and `expiry_mode` is set…" — **any auth_type** (broad).
> - **error-taxonomy.md E-SPEC-028(c) description:** "`[auth_acquisition].expiry_mode` value not in allowed set" — **any auth_type** (broad).
>
> **Token_exchange-gating (§D10(c)) is canonical.** Four independent reasons:
>
> 1. **`expiry_mode` is a token_exchange-only field, already policed by (h).** Sub-condition (h) fires when token_exchange-only fields (`credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode`) appear in an `[auth_acquisition]` block whose `auth_type` is not `token_exchange`. Emitting an additional (c) value-validity error for a field that (h) has already flagged as wrong-position creates contradictory repair guidance for LLM-agent-consumed errors (project agent-harness design, AD-013): (h) says "remove this field"; (c) says "fix this field's value." Contradictory guidance degrades the agent's ability to infer the correct repair path.
>
> 2. **Semantic cleanness: value validity presupposes positional validity.** Validating the value of `expiry_mode` on an `oauth2_client_credentials` block is a category error — the field should not exist there, so its value is irrelevant. The correct single signal is (h). This is the exact structural parallel of §D10(b): (b) validates the presence/absence of `auth_plugin` for declarative types; it does not validate what `auth_plugin` references when present on the wrong type. Similarly, (c) validates the value of `expiry_mode` only when it is in a valid position; position-validity is (h)'s responsibility.
>
> 3. **Co-fire list consistency.** The taxonomy's illustrative co-fire list pairs (c) only with (d) — both `token_exchange`-context errors. It does NOT list `(c)∩(h)` or `(c)∩(g)`, which would be mandatory additions if the broad scope were canonical (since (g) and (h) fire for non-`token_exchange` types with `[auth_acquisition]` present). The co-fire list was authored as if narrow scope was intended; the broad-scope trigger description in BC/taxonomy is the inconsistency, not the co-fire list.
>
> 4. **Clean partition between (c) and (h).** Under the narrow scope: (c) = value-validity for valid-position use (`token_exchange`); (h) = position-validity for wrong-position use (non-`token_exchange`). These are disjoint, complementary checks. Under the broad scope, (c) and (h) overlap and emit contradictory signals for the same field at the same spec. The narrow scope eliminates the overlap.
>
> **§D10(c) overrides BC-2.16.009 Rule 10(c) and error-taxonomy E-SPEC-028(c) on this trigger-logic question** (the §D10 meta-note at the top of this section states §D10 is the authoritative source for trigger logic; taxonomy wins on wording only). BC and taxonomy are PO-owned; required PO updates:
>
> **Required BC-2.16.009 Rule 10(c) replacement text (PO sweep — finding F-WASE-P40-MED-001):**
> Replace: "If `[auth_acquisition]` is present and `expiry_mode` is set, its value must be one of: `"absolute_utc_string"` or `"relative_seconds"`. Any other value → `E-SPEC-028` citing the invalid `expiry_mode` value."
> With: "If `auth_type = "token_exchange"` AND `[auth_acquisition]` is present AND `expiry_mode` is set, its value must be one of: `"absolute_utc_string"` or `"relative_seconds"`. Any other value → `E-SPEC-028` citing the invalid `expiry_mode` value. Note: when `expiry_mode` appears in an `[auth_acquisition]` block whose `auth_type` is not `token_exchange`, sub-condition (h) handles the position-validity check; sub-condition (c) does NOT additionally fire for value-validity of a wrong-position field."
>
> **Required error-taxonomy.md E-SPEC-028(c) description sub-condition replacement text (PO sweep — finding F-WASE-P40-MED-001):**
> Replace: "(c) `[auth_acquisition].expiry_mode` value not in allowed set (`absolute_utc_string`, `relative_seconds`)"
> With: "(c) `auth_type = "token_exchange"` AND `[auth_acquisition].expiry_mode` value not in allowed set (`absolute_utc_string`, `relative_seconds`); does NOT fire when `expiry_mode` appears on a non-`token_exchange` block — sub-condition (h) handles position-validity in that case"
>
> **Required BC-2.16.009 EC-009-038 clarification (PO sweep — finding F-WASE-P40-MED-001):**
> The EC-009-038 description does not specify `auth_type`. Under the narrow scope, (c) only fires for `token_exchange`. PO should make the context explicit: replace the input description "`expiry_mode = "absolute_utc_string"` in `[auth_acquisition]`" with "`auth_type = "token_exchange"`, `expiry_mode = "absolute_utc_string"` in `[auth_acquisition]`". The expected-behavior cell ("Rule 10(c) passes; spec loads") is correct for the `token_exchange` case and requires no change.
>
> **Taxonomy co-fire list requires NO changes.** The current illustrative list `(c)∩(d)` is valid under narrow scope (`token_exchange` + expiry_mode invalid + required field absent). `(c)∩(h)` and `(c)∩(g)` are impossible under narrow scope — (c) requires `auth_type = "token_exchange"` while (h) and (g) require `auth_type ≠ "token_exchange"`. The existing list is correct as written.

**(d) token_exchange missing required fields:**
`"sensor '{sensor_id}': auth_type = 'token_exchange' requires [auth_acquisition].{field_name}. Add the missing field."`
Fires for each of `credential_body_field`, `token_response_path`, `expiry_field`, `expiry_mode` when absent.

**(e) credential_body_field undeclared:**
`"sensor '{sensor_id}': [auth_acquisition].credential_body_field = '{field}' does not match any declared [[credential_refs]] name. Declared refs: [{refs}]. Add a [[credential_refs]] block with name = '{field}'."`
Fires when `credential_body_field` names a ref not found in the sensor's `[[credential_refs]]` array.

**(f) oauth2_client_credentials missing required credential_refs:**
`"sensor '{sensor_id}': auth_type = 'oauth2_client_credentials' requires [[credential_refs]] entries named 'client_id' and 'client_secret'. Missing: {missing_refs}."`
Fires when `auth_type = "oauth2_client_credentials"` and one or both of `client_id`, `client_secret` are absent from `[[credential_refs]]`.

**(g) auth_acquisition declared for non-declarative auth_type:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' does not use [auth_acquisition]. Remove the [auth_acquisition] block or change auth_type to oauth2_client_credentials or token_exchange."`
Fires when `[auth_acquisition]` is present for `auth_type ∈ {bearer_static, cookie_roundtrip, api_key, custom_via_plugin}`.

**(h) token_exchange-only fields on non-token_exchange block:**
`"sensor '{sensor_id}' [auth_acquisition] contains token_exchange-only fields ({field_list}) but auth_type = '{auth_type}'"`
Fires as a **single aggregated emission** when one or more of `credential_body_field`, `token_response_path`, `expiry_field`, or `expiry_mode` is present in an `[auth_acquisition]` block whose `auth_type` is not `token_exchange`; `{field_list}` is a comma-separated list of all offending field names and `{auth_type}` is the declared auth_type. Prevents token_exchange-only fields from being silently ignored when misconfigured on an `oauth2_client_credentials` block (SOUL.md #4 violation class). Note (F-WASE-P2-MED-001): prior §D10(h) used per-field `{field_name}` emission (one error per offending field); aligned to taxonomy E-SPEC-028(h) single-aggregated-emission cardinality.

All templates echo only config values (sensor_id, auth_type, field names), never credential values
(AD-017). Emitted via `SpecErrorCode::ESpec028` (additive variant — no semver break per `#[non_exhaustive]`).

### D11 — Spec Amendment Manifest

| Artifact | Amendment | Triggered by |
|----------|-----------|--------------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Drop `auth_plugin = "crowdstrike-oauth2"`; add `[auth_acquisition]` block (D2) | D5 |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` (entire crate) | Delete crate directory and workspace member | D5 |
| `Cargo.toml` workspace `members` array | Remove `crates/prism-spec-engine/plugins/crowdstrike-oauth2` | D5 |
| `crates/prism-spec-engine/src/spec_parser.rs` `AuthType` enum | Add `TokenExchange` variant; add `"token_exchange"` to `VALID_AUTH_TYPES`; `#[non_exhaustive]` already present | D1 |
| `crates/prism-spec-engine/src/spec_parser.rs` `SensorSpec` struct | Add `auth_acquisition: Option<AuthAcquisitionConfig>` field | D3 |
| `crates/prism-spec-engine/src/spec_parser.rs` `AuthType::as_str()` exhaustive match (FIX-BURST 28) | Add `TokenExchange => "token_exchange"` arm to the exhaustive no-wildcard match in `impl AuthType { pub fn as_str(&self) -> &'static str }`: current match has 5 arms covering `Oauth2ClientCredentials`, `BearerStatic`, `CookieRoundtrip`, `ApiKey`, `CustomViaPlugin`. Adding `TokenExchange` variant (ADR-054 D1) makes this arm required by E0004 — the site is **compile-enforced** (omission is a hard compile error, not a silent runtime gap), so this D11 row exists for **story-checklist completeness** only; rustc enforcement is stronger than the manifest. Behavioral anchor: the exhaustive match body in `impl AuthType::as_str` (TD-VSDD-091). Update in same spec-evolution story as `AuthType` enum D1 extension. | D1 |
| `crates/prism-spec-engine/src/spec_parser.rs` `validate_cross_composition` fn doc-comment (FIX-BURST 27) | Add `token_exchange` to the Rule A brace list in the `validate_cross_composition` fn doc-comment: current text `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key,` / `  custom_via_plugin}` — update to include `token_exchange` as 6th variant. Behavioral anchor: the Rule A `{...}` brace expression in the `validate_cross_composition` fn doc-comment (grep-recoverable by brace-list text; TD-VSDD-091). Distinct from the already-manifested `AuthType` enum and `VALID_AUTH_TYPES` const rows — this is documentation prose, not an enum definition or validation constant. Update in same spec-evolution story as the `AuthType` enum D1 extension. | D1 (doc-hygiene) |
| `crates/prism-spec-engine/src/spec_parser.rs` `validate_cross_composition` inline comment (FIX-BURST 27) | Add `token_exchange` to the inline comment immediately above `VALID_AUTH_TYPES` const definition: current text `// {oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` — update to include `token_exchange` as 6th variant. Behavioral anchor: the `// {oauth2_client_credentials...}` inline comment in the `validate_cross_composition` function body (grep-recoverable by comment text; TD-VSDD-091). Update in same spec-evolution story as the `VALID_AUTH_TYPES` const extension. | D1 (doc-hygiene) |
| New `crates/prism-spec-engine/src/auth/declarative.rs` | Implement `DeclarativeHttpAuthProvider` + `AuthAcquisitionConfig` + `ExpiryMode` | D4 |
| `crates/prism-spec-engine/src/auth/declarative.rs` `AuthAcquisitionConfig` constructors (F-WASE-P38-MED-001) | Add `#[non_exhaustive]` attribute to `AuthAcquisitionConfig` (per CLAUDE.md convention for public TOML-deserialized types in prism-spec-engine). Implement three named constructors: (1) `pub fn new(token_path: impl Into<String>, expiry_mode: ExpiryMode, ttl_buffer_secs: u64) -> Self` — populates common fields; `credential_body_field`, `token_response_path`, and `expiry_field` default to empty string; used for `token_exchange` minimal harness configs (ExpiryMode selector required; NOT for `oauth2_client_credentials` — see constructor (3) below); (2) `pub fn new_token_exchange(token_path: impl Into<String>, credential_body_field: impl Into<String>, token_response_path: impl Into<String>, expiry_field: impl Into<String>, expiry_mode: ExpiryMode, ttl_buffer_secs: u64) -> Self` — populates all fields for `token_exchange` auth_type; (3) `pub fn new_oauth2(token_path: impl Into<String>, ttl_buffer_secs: u64) -> Self` — populates common fields for `oauth2_client_credentials` auth_type; no `ExpiryMode` parameter (`expiry_mode` is token_exchange-only per §D3; E-SPEC-028(h) rejects it if present on an `oauth2_client_credentials` block at spec-load time; oauth2 expiry is always relative seconds via `$.expires_in`, engine-internal per RFC 6749); `credential_body_field`, `token_response_path`, and `expiry_field` are inapplicable for oauth2 (form body + response parsing are fixed by RFC 6749, engine-internal); used for `oauth2_client_credentials` configs and VP-159 AC-7d harness (F-WASE-P56-MED-001). VP-159 harness uses these constructors throughout (not struct literals, which are E0639 from the external `tests/` crate under `#[non_exhaustive]`). Behavioral anchor: `AuthAcquisitionConfig` struct definition and `impl AuthAcquisitionConfig` block in `auth/declarative.rs` (grep `pub struct AuthAcquisitionConfig`; TD-VSDD-091). | D3, F-WASE-P38-MED-001 |
| `crates/prism-spec-engine/src/auth/declarative.rs` `CachedAuthToken` constructor (F-WASE-P38-MED-001) | Add `#[non_exhaustive]` attribute to `CachedAuthToken` (per CLAUDE.md convention for new pub types in prism-spec-engine). Implement `pub fn new(token: String, expires_at: u64) -> Self` constructor. The signature enforces credential-opacity architecturally: it accepts only the token string and expiry timestamp — no credential parameter — making it impossible to store a credential value in `CachedAuthToken` without a deliberate constructor signature change (BC-2.16.014 P7, AD-017, INV-014-003). VP-159 harness uses `CachedAuthToken::new(...)` at AC-4b (poisoned-cache seeding) and the AC-8 structural note. Behavioral anchor: `CachedAuthToken` struct definition and `impl CachedAuthToken` block in `auth/declarative.rs` (grep `pub struct CachedAuthToken`; TD-VSDD-091). | D4, F-WASE-P38-MED-001 |
| `scripts/check-non-exhaustive.sh` + `scripts/check-non-exhaustive-per-symbol.py` + `CLAUDE.md` — non-exhaustive gate EXPECTED bump (RU-Scanner-1) | Three new `#[non_exhaustive]` pub types added by the ADR-054 engine story: `AuthAcquisitionConfig`, `CachedAuthToken`, `ExpiryMode`. All **THREE** update sites are mandatory and must land atomically in the same commit: **(1)** `scripts/check-non-exhaustive.sh` EXPECTED value: `92 → 95`; **(2)** `scripts/check-non-exhaustive-per-symbol.py` `EXPECTED_COUNT = 92 → 95` AND append `AuthAcquisitionConfig`, `CachedAuthToken`, `ExpiryMode` to `EXPECTED_SYMBOLS`; **(3)** `CLAUDE.md` "92 types currently enforced" sentence — update count AND append new symbols to the inline list. Omitting any one of the three sites causes CI failure (shell script) or startup-assert failure (Python Layer-2 check). Behavioral anchors: `EXPECTED=92` in `check-non-exhaustive.sh`; `EXPECTED_COUNT = 92` + `EXPECTED_SYMBOLS` list in `check-non-exhaustive-per-symbol.py`; "92 types currently enforced" in CLAUDE.md. | D3, D4, RU-Scanner-1 |
| `crates/prism-bin/src/spec_driven_adapter.rs` `step9a_populate_adapter_registry` `Oauth2ClientCredentials` arm | Rewrite from per-org `PluginAuthProvider` construction to `DeclarativeHttpAuthProvider` construction with `token_url = base_url + token_path` | D4, D7 |
| `crates/prism-bin/src/spec_driven_adapter.rs` `step9a_populate_adapter_registry` | Add new `TokenExchange` arm: construct `DeclarativeHttpAuthProvider(TokenExchange)` with `token_url = base_url + token_path` | D1, D7 |
| `BC-2.16.009` Rule set | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Add `[auth_acquisition]` coherence validation as **Rule 10** (after ADR-053 D2's Rule 9 for `header_scheme`); E-SPEC-028 error suite per D10 | D10 |
| `error-taxonomy.md` | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Register E-SPEC-028 with all message templates (D10) | D10 |
| New `BC-2.16.014` **[AUTHORED — D-1946]** | Declarative Auth Acquisition Token Lifecycle contract authored during Wave-A spec evolution burst 1; postconditions P1–P8 specified in §D8 above were the authoring source | D8 |
| `VP-INDEX.md` **[REGISTERED — D-1947]** | VP-159 registered (D9) during Wave-A spec evolution burst 2 (D-1947); BC-2.16.014 was authored in burst 1 (D-1946) | D9 |
| ADR-053 D2 | **[COMPLETED — ADR-053 v0.7]** Armis TOML block rewritten from `custom_via_plugin` + `auth_plugin` to `token_exchange` + `[auth_acquisition]` block; coherence matrix updated with `token_exchange → bearer, raw` row; rationale section rewritten. | D1, D3 |
| ADR-053 §Why native `token_exchange` for Armis (not `custom_via_plugin` + plugin)? | **[COMPLETED — ADR-053 v0.7/v0.12]** Rationale updated to reflect native declarative provider decision; heading renamed from `§Why custom_via_plugin` (retired per POL-21) to current heading `§Why native token_exchange for Armis (not custom_via_plugin + plugin)?`. | D2 |
| ADR-053 D5 manifest | **[EXECUTED — ADR-053 v0.7, 2026-07-20]** BC-2.01.008 amendment description updated: reflects `token_exchange` (native `DeclarativeHttpAuthProvider` per ADR-054) + `header_scheme = "raw"` (ADR-053 D5 manifest BC-2.01.008 row; ADR-053 v0.7 §Changelog). | D1 |
| ADR-026 §D3 (partial) | **[EXECUTED — ADR-026 v1.35, 2026-07-20]** Frontmatter `amended_by` back-ref added for ADR-054 (token_exchange + oauth2_client_credentials reclassification); §D3 amendment note added with Rule A at-point annotation "[ADR-054 D1 adds `token_exchange` as 6th variant on acceptance]" (ADR-026 v1.35 + v1.38 §Changelog). | D1 |
| ADR-023 Rule 2 (Rule A) (census note — annotation-only, no amendment row) | **Covered-by-annotation-only by design — no D11 amendment row required.** ADR-023 Rule 2 (Rule A) restates the auth_type set but is not the defining site; the defining-site amendment is the ADR-026 §D3 row above. ADR-023 Rule 2 (Rule A) already carries an at-point annotation noting the 6th variant; a formal D11 amendment row would duplicate the change. This asymmetry with ADR-026 §D3 is intentional. | D1 (annotation only) |
| `domain-spec/invariants.md` DI-012 "Single auth_type per spec" enumeration | **[EXECUTED — invariants.md v1.8, 2026-07-22]** `token_exchange` added as 6th variant to DI-012 Rule 1 pipe-delimited enumeration; amended text now reads: `SensorSpec.auth_type` accepts exactly one value (`oauth2_client_credentials` \| `bearer_static` \| `cookie_roundtrip` \| `api_key` \| `custom_via_plugin` \| `token_exchange`). Bidirectional ADR-054 back-ref also present in DI-012 amendment parenthetical: "; ADR-054 D1 (Wave-A token_exchange addition)" **[EXECUTED — invariants.md v1.8]**. DI-012 is the canonical domain root that VP-153 traces to (VP-153 `source_invariant: DI-012`) and BC-2.01.016 operationalizes. Behavioral anchor: DI-012 rule 1 "Single auth_type per spec" pipe-delimited value list. | D1 |
| `BC-2.16.001` §Postconditions + §Auth Type Resolution (doc-hygiene) | **[EXECUTED — BC-2.16.001 v1.9, Wave-A spec evolution burst 3, 2026-07-22]** (1) §Postconditions `auth_type` parenthetical updated from `(oauth2/bearer/cookie/api_key)` slash shorthand to full 6-value canonical set `(oauth2_client_credentials | bearer_static | cookie_roundtrip | api_key | custom_via_plugin | token_exchange)`; (2) §Auth Type Resolution example list extended to include `custom_via_plugin` and `token_exchange` (BC-2.16.001 v1.9 §Changelog). | D1 (doc-hygiene) |
| BC-2.06.003 §Per-Sensor `[[credential_refs]]` Declarations (Canonical) table | **[EXECUTED — BC-2.06.003 v1.12, Wave-A spec evolution burst 3, 2026-07-22]** CrowdStrike auth provider column updated from `crowdstrike-oauth2 WASM plugin` to `DeclarativeHttpAuthProvider(Oauth2ClientCredentials)` per ADR-054 D1 (BC-2.06.003 v1.12 §Changelog). Credential ref names (`client_id`, `client_secret`) and `auth_type` (`oauth2_client_credentials`) are UNCHANGED. | D5 |
| BC-2.01.016 §Related BCs | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Update "one entry in the 5-value canonical auth_type set" → 6-value canonical auth_type set (`token_exchange` is the 6th variant per D1) | D1 |
| BC-2.01.017 §Preconditions | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Update "The 5-value canonical auth_type set (BC-2.01.016 §Postconditions)" → 6-value canonical auth_type set | D1 |
| BC-2.01.017 §P3 (Auth Type Dispatch) | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Update "the 5-value canonical auth_type set per BC-2.01.016 §Postconditions" → 6-value canonical auth_type set | D1 |
| BC-2.01.017 §Related BCs | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Update "the 5-value canonical auth_type set (including `"cookie_roundtrip"`)" → 6-value canonical auth_type set | D1 |
| BC-2.16.009 §Validation Rules (Schema Validation, `auth_type` rule) | **[EXECUTED — Wave-A spec evolution burst 3, 2026-07-22]** Add `token_exchange` to the enumerated allowed-values list; update "(5-value canonical set)" → "(6-value canonical set)" in the parenthetical | D1 |
| `VP-153` §Property Statement Rule A enumerated set + E-SPEC-012 expected message string | **[EXECUTED — VP-153 v0.21, Wave-A spec evolution burst 3, 2026-07-22]** `token_exchange` added as 6th variant to the Rule A enumerated set `{…, custom_via_plugin, token_exchange}`; E-SPEC-012 expected message "Valid values: …" updated to include `token_exchange` as the 6th variant. Both sites in `vp-153-sensorauth-runtime-cross-composition-prevention.md` §Property Statement Rule A section. Executed in same commit as error-taxonomy.md v2.57 E-SPEC-012 amendment (POL-24 atomicity satisfied). | D1 |
| `VP-153` §Feasibility Assessment | **[EXECUTED — VP-153 v0.21, Wave-A spec evolution burst 3, 2026-07-22]** "5 auth_type variants × 5 credential structural shapes = 25 pairs" → "6 auth_type variants × 5 credential structural shapes = 30 pairs"; all harness amendments for VALID_AUTH_TYPES, arb_valid_auth_type(), arb_matching_auth_type(), and arb_mismatched_auth_type_pair() range bounds executed in VP-153 v0.21. `token_exchange` occupies a DISTINCT 6th credential structural shape under VP-153's identifier-based harness model (`reported_shape` IS the canonical `auth_type` identifier string per `vp153_rule_c_shaped_probe.rs`; `token_exchange` as `reported_shape` is therefore a distinct 6th shape, not an alias for `api_key`). The record-schema similarity (token_exchange's credential is a single-string secret, structurally similar to `api_key`'s) is a RECORD-schema observation that does NOT alter the identifier-based SHAPE model. Total credential structural shapes in the harness model: 6. Mismatched shapes per variant: 5 (6 total − 1 matching). 6 variants × 5 mismatched shapes = 30 ordered pairs (VP-153 §Feasibility v0.26: "of the 6 total credential structural shapes, excluding the matching one"). Rule B `allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 }` auto-routes `token_exchange` to `allowed_count = 1` via `else` branch — no separate change. | D1 |
| `VP-153` §Proof Method member-count sentence | **[EXECUTED — VP-153 v0.21, Wave-A spec evolution burst 3, 2026-07-22]** "The valid `auth_type` enumerated set has 5 members" → "6 members". Executed in same burst as §Feasibility Assessment "25 pairs"→"30 pairs" correction (POL-24 consistency satisfied). | D1 |
| `VP-153` §Proof Harness Skeleton `arb_valid_auth_type()` + `VALID_AUTH_TYPES` constants | **[EXECUTED — VP-153 v0.21, Wave-A spec evolution burst 3, 2026-07-22]** All five harness amendments executed: (1) `"token_exchange"` added as 6th entry in `VALID_AUTH_TYPES` (both test files); (2) `Just("token_exchange")` added to `arb_valid_auth_type()` `prop_oneof!` (FILE 1); (3) `Just("token_exchange")` added to `arb_matching_auth_type()` `prop_oneof!` (FILE 2); (4) `arb_mismatched_auth_type_pair()` range bounds `(0usize..5, 0usize..4)` → `(0usize..6, 0usize..5)` (FILE 2); (5) `arb_invalid_auth_type()` filter auto-expanded via VALID_AUTH_TYPES reference — no separate edit. `Just("token_exchange")` arms carry `[PLANNED — ADR-054 D1 engine story]` markers until the engine story activates them; spec scaffolding is in place. Behavioral anchors: `VALID_AUTH_TYPES` constant in both test files; `arb_valid_auth_type()` `prop_oneof!` arm list; `arb_matching_auth_type()` `prop_oneof!` arm list; `arb_mismatched_auth_type_pair()` range tuple. | D1 |
| `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` module-level `//!` doc (FIX-BURST 27) | Add `token_exchange` to the Rule A brace list in the module-level `//!` doc: current text `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` — update to include `token_exchange` as 6th variant. Behavioral anchor: the Rule A enumeration brace list in the module-level `//!` doc (grep-recoverable by brace-list text; TD-VSDD-091). Distinct from the already-manifested VP-153 `§Property Statement Rule A enumerated set` and `§Proof Harness Skeleton` spec-document rows — this is the test file's module-level documentation prose. Update in same spec-evolution story as the VP-153 spec prop-statement and harness rows. | D1 (doc-hygiene) |
| `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` test-inventory `//!` doc table row 4 (F-WASE-P29-OBS-001) | Correct row-4 function name in the test-inventory docstring table from `prop_rule_b_single_or_zero_credential_refs_accepted` to `prop_rule_b_single_credential_ref_accepted` to match the defined `fn` (confirmed at the `proptest!` fn declaration). One-word divergence: `single_or_zero` vs `single` — the function accepts a single valid credential_ref count (not "single or zero"), matching the postcondition that `allowed_count` is the correct count for the given auth_type. Corrected row-4 table text: `\| 4 \| prop_rule_b_single_credential_ref_accepted \| B \| postcondition \|`. Behavioral anchor: the `prop_rule_b_single_or_zero_credential_refs_accepted` string in the test-inventory `//!` doc table row-4 Name column (grep-recoverable by `single_or_zero`; TD-VSDD-091). No behavioral change — doc-string-only correction. Route to implementer at engine-story time (same story that activates the `token_exchange` proptest arms per the existing harness-amendment D11 rows). | F-WASE-P29-OBS-001 (doc-hygiene) |
| `VP-153` proof re-run — engine story gate (F-WASE-P4-OBS-002) | After the ADR-054 engine story activates the `token_exchange` proptest arms (dropping `[PLANNED — ADR-054 D1 engine story]` markers from `Just("token_exchange")` in `arb_valid_auth_type()` in `vp153_sensorauth_cross_composition.rs` and from `Just("token_exchange")` in `arb_matching_auth_type()` plus the updated range bounds in `arb_mismatched_auth_type_pair()` in `vp153_rule_c_shaped_probe.rs` per the §Proof Harness Skeleton and §Feasibility Assessment D11 rows above), the engine story MUST re-run all 8 VP-153 proptests with these arms **active** as an **explicit story gate before the PR can merge**. The engine story checklist must include: (1) drop `[PLANNED]` markers from the six harness sites listed in the §Proof Harness Skeleton and §Feasibility Assessment D11 rows; (2) run `cargo nextest run -p prism-spec-engine -E 'test(vp153)' && cargo nextest run -p prism-bin -E 'test(vp153)'`; (3) confirm all 8 proptests PASS; (4) update VP-153 `proof_completed_date` and advance `status` to `active` (already active; confirm green). Until the engine story lands, the current green proof (proof-completed-date 2026-05-18) covers the 5-value as-built set; the `token_exchange` arms in the spec are scaffolding that has not yet executed. Behavioral anchors: `Just("token_exchange")` arm in `arb_valid_auth_type()` `prop_oneof!` (FILE 1); `Just("token_exchange")` arm in `arb_matching_auth_type()` `prop_oneof!` (FILE 2); `(0usize..6, 0usize..5)` range bounds in `arb_mismatched_auth_type_pair()` (FILE 2). See VP-153 §Re-verification Gate section. | D1 (proof gate) |
| **--- AuthProvider trait + PipelineExecutor call-site wiring (engine story, F-WASE-P7-HIGH-001) ---** | | |
| `crates/prism-spec-engine/src/auth_provider.rs` `AuthProvider` trait | Add `get_token<'a>(&'a self, spec: &'a SensorSpec, client_id: &'a OrgSlug) -> Pin<Box<dyn Future<Output = Result<AuthToken, SpecEngineError>> + Send + 'a>>` with a default implementation that delegates to `self.acquire_token(spec, client_id)`. All 7 existing implementors (`NullAuthProvider`, `MockAuthProvider`, `StaticCookieAuthProvider`, `FailingAuthProvider`, `ChainAuthProvider`, `BearerStaticCredentialAuthProvider`, `PluginAuthProvider`) inherit the default without code changes. `DeclarativeHttpAuthProvider` overrides `get_token()` with the ArcSwap cache-aware logic (§D4). Behavioral anchor: `AuthProvider` trait body in `auth_provider.rs`, immediately after the existing `acquire_token` method declaration (TD-VSDD-091). | F-WASE-P7-HIGH-001 |
| `crates/prism-spec-engine/src/pipeline.rs` `PipelineExecutor::execute_impl` — normal eager path | Change the call site from `auth_provider.acquire_token(spec, &context.client_id)` to `auth_provider.get_token(spec, &context.client_id)` in the `let mut bearer_token = match auth_provider.` block before the `'steps: for` loop. **The 401-refresh call in `issue_request_with_retry` remains `acquire_token()` — see next row.** Behavioral anchor: `let mut bearer_token = match auth_provider.` block in `execute_impl`, before the `'steps:` loop (TD-VSDD-091: no line pins). | F-WASE-P7-HIGH-001 |
| `crates/prism-spec-engine/src/pipeline.rs` `PipelineExecutor::execute_step` — normal eager path | Change the call site from `auth_provider.acquire_token(spec, &context.client_id)` to `auth_provider.get_token(spec, &context.client_id)` in the `let bearer_token = match auth_provider.` block at the top of `execute_step`. Behavioral anchor: `let bearer_token = match auth_provider.` block at the top of `execute_step` (TD-VSDD-091: no line pins). | F-WASE-P7-HIGH-001 |
| `crates/prism-spec-engine/src/pipeline.rs` `issue_request_with_retry` — 401-refresh path | **No change.** `auth_provider.acquire_token(spec, client_id)` on the 401 path REMAINS `acquire_token()` (force-refresh, cache-bypass). This is intentional: the sensor endpoint rejected the cached token; calling `get_token()` would return the same rejected token on a warm-cache hit. BC-2.16.014 P5/P6 explicitly require `acquire_token()` here. Behavioral anchor: `let fresh_token = match auth_provider.acquire_token` block in the `reqwest::StatusCode::UNAUTHORIZED` arm of `issue_request_with_retry` (TD-VSDD-091: no line pins). | F-WASE-P7-HIGH-001 (intentional no-change) |
| `error-taxonomy.md` E-SPEC-012 message template (Rule A — auth_type closed-enum validation) + `error.rs` `AuthTypeCrossComposition` Display rewrite | **Step 1 — taxonomy amendment [EXECUTED — error-taxonomy.md v2.57, Wave-A spec evolution burst 3, 2026-07-22]:** `token_exchange` added as 6th value to the E-SPEC-012 `message_template` "Valid values: …" enumeration; amended text now reads "…oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange". **Step 2 — code Display rewrite (F-WASE-P16-OBS-003) [PENDING — engine story]:** The as-built `#[error(…)]` attribute on `AuthTypeCrossComposition` in `error.rs` diverges from the taxonomy template in BASE WORDING — not only in the missing `token_exchange` value. The engine story MUST rewrite the Display to match the taxonomy template VERBATIM under POL-24. Current code wording (behavioral anchor: `#[error("E-SPEC-012: sensor '{sensor_id}' auth_type value '{provided_value}' is invalid — must be a scalar from the closed enumeration per ADR-026 §D3")]` attribute on `AuthTypeCrossComposition` in `error.rs`): `"E-SPEC-012: sensor '{sensor_id}' auth_type value '{provided_value}' is invalid — must be a scalar from the closed enumeration per ADR-026 §D3"`. Taxonomy template (behavioral anchor: E-SPEC-012 `message_template` column in `error-taxonomy.md`): `"auth_type for sensor '{sensor_id}' must be a single value; got: {value}. Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange"`. The field substitution variable name also diverges: code uses `{provided_value}`, taxonomy uses `{value}` — the engine story must align the `AuthTypeCrossComposition` struct field name or the `#[error(…)]` substitution token so the emitted string matches the taxonomy template VERBATIM. **POL-24 atomicity — two of three prose sites already executed:** `error-taxonomy.md` E-SPEC-012 (source of truth) executed in burst 3 (v2.57); VP-153 §Property Statement Rule A prose executed in VP-153 v0.21 burst 3. Only `error.rs` `AuthTypeCrossComposition` `#[error(…)]` attribute (code copy) remains pending — the engine story updates this single remaining site at code-alignment time. Note: VP-153 §Proof Harness Skeleton `arb_valid_auth_type()` and `VALID_AUTH_TYPES` are executed via their own D11 row (VP-153 v0.21); no "Valid values:" literal appears in the test code, so no additional POL-24 copy site exists in the harness. | D1 |
| `crates/prism-spec-engine/src/error.rs` `MultipleCredentialRefs` Display — E-SPEC-013 code-alignment (F-WASE-P31-MED-001, FIX-BURST 28) | The as-built `MultipleCredentialRefs` `#[error(…)]` attribute in `error.rs` emits `"E-SPEC-013: sensor '{sensor_id}' declares {credential_count} credential_refs — exactly 1 is required per BC-2.01.016 §Error Cases"`. This diverges from the error-taxonomy.md v2.65 canonical template `"auth method for sensor '{sensor_id}' declares {count} credential_refs; exactly {expected} required for auth_type '{auth_type}'"` in three ways: (1) hardcoded `"1"` vs parameterized `{expected}`; (2) missing `{auth_type}` parameter; (3) nonstandard prefix (`"E-SPEC-013: sensor..."` instead of `"auth method for sensor..."`) and nonstandard suffix (`"...per BC-2.01.016 §Error Cases"` absent from template). The implementer rewrites the `MultipleCredentialRefs` `#[error(…)]` attribute (and struct field names as needed: `credential_count` → `count`; add `expected: usize` and `auth_type: &'static str` fields or `String`) to match the taxonomy template VERBATIM (POL-24) at engine-story time. Behavioral anchor: `MultipleCredentialRefs` variant in `error.rs` (grep `MultipleCredentialRefs`; TD-VSDD-091). Provenance: F-WASE-P31-MED-001; error-taxonomy.md v2.65 is the POL-24 source of truth for the new template. | D1 (code-alignment) |
| `error-taxonomy.md` E-SPEC-005 notes column (doc-hygiene) | **[EXECUTED — error-taxonomy.md v2.57, Wave-A spec evolution burst 3, 2026-07-22]** E-SPEC-005 notes column modernized: "Auth type must be one of: oauth2, bearer, cookie, api_key." → "Auth type must be one of: oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange." (error-taxonomy.md v2.57 §Changelog; canonical 6-value set now present in error-taxonomy.md E-SPEC-005 notes). | D1 (doc-hygiene) |
| ADR-028 §D13 consistency table | **[EXECUTED — ADR-028 v1.18/v1.20, 2026-07-21]** `oauth2_client_credentials` row updated: `PluginAuthProvider` (WASM) path marked **spec-load-rejected** per D10(b) E-SPEC-028(b); `DeclarativeHttpAuthProvider` (native) is the sole live path; "when `[auth_acquisition]` present" conditional framing removed. §D2 + §D13 Armis blockquotes updated from `custom_via_plugin` + `armis-token-exchange.prx` → `token_exchange` + native `DeclarativeHttpAuthProvider`. Frontmatter `amended_by` framing reflects D10(b) unconditional rejection (ADR-028 v1.18 + v1.20 §Changelog). | D2, D5, D10 |
| **--- CrowdStrike plugin retirement blast-radius (atomic with D5 migration story) ---** | | |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Drop `auth_plugin = "crowdstrike-oauth2"`; add `[auth_acquisition]` block with `token_path = "/oauth2/token"` and `credential_refs` (D2) | D5 |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` (entire crate dir) | Delete source, `Cargo.toml`, WIT, committed `.prx` binary | D5 |
| `Cargo.toml` workspace `members` array | Remove `"crates/prism-spec-engine/plugins/crowdstrike-oauth2"` entry | D5 |
| `crates/prism-spec-engine/Cargo.toml` | Remove `[[test]]` block with `name = "crowdstrike_oauth2_plugin_tests"` (build target anchored by test target name `crowdstrike_oauth2_plugin_tests`); remove `async-trait` dev-dep and its comment `# async-trait: used in crowdstrike_oauth2_plugin_tests.rs for NullTestOrgIdStore stub` — both stale post-retirement; leaving the build target causes a cargo build error (missing `.rs` source file) | D5 |
| `Justfile` `build-plugin-crowdstrike-oauth2` recipe | Delete entire `build-plugin-crowdstrike-oauth2` recipe (multi-step wasm32 build + wasm-tools validate + wasm-pack chain; recipe name is the behavioral anchor) | D5 |
| `.github/workflows/ci.yml` `wasm32-compile-check` job | Remove all four crowdstrike plugin steps (anchored by step names): "Check crowdstrike-oauth2-plugin compiles", "Validate committed crowdstrike-oauth2.prx structural integrity (F-MCPRS-PRL14-LOW-001)", "Build crowdstrike-oauth2.prx", "Upload crowdstrike-oauth2.prx artifact" | D5 |
| `.github/workflows/ci.yml` first CI self-guard in `validate-workflow-structure` job | Remove `grep -qE 'just build-plugin-crowdstrike-oauth2'` + `exit 1` reachability assertion (grep pattern is the behavioral anchor) — **this guard fails CI BY DESIGN once the build step is removed; must be removed in the same change** | D5 |
| `.github/workflows/ci.yml` second CI self-guard in `validate-workflow-structure` job | Remove `grep -q 'F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike'` + `exit 1` reachability assertion (grep string `F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike` is the behavioral anchor) — **this guard fails CI BY DESIGN once the committed-.prx wasm-tools step is removed; must be removed in the same change** | D5 |
| `.config/nextest.toml` binary filter entries and profile-documentation comment | Remove `binary(crowdstrike_oauth2_plugin_tests)` from all filter group expressions (grep `binary(crowdstrike_oauth2_plugin_tests)` to locate all affected filter entries — currently four occurrences in two profile blocks); also remove or update the profile-documentation comment that enumerates `crowdstrike_oauth2_plugin_tests` (behavioral anchor: the profile-documentation comment enumerating `crowdstrike_oauth2_plugin_tests`) | D5 |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | Port behavioral coverage (EC-001 through EC-006c, cache-hit, cache-miss, URL encoding, form body) to `DeclarativeHttpAuthProvider` unit tests before retiring; delete the WASM plugin test file | D5 |
| `crates/prism-spec-engine/tests/bc_2_16_013_crowdstrike_multiregion.rs` | Retarget the `D-747 LOCKED: auth_plugin must remain 'crowdstrike-oauth2'` assertion in the `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly` and `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works` tests (postcondition 3, `assert_eq!(spec.auth_plugin.as_deref(), Some("crowdstrike-oauth2"), …)`): remove the `auth_plugin` check and replace with an assertion confirming `spec.auth_acquisition` is present with `token_path = "/oauth2/token"` (native declarative config). D-1895/ADR-054 walks back the D-747 lock. (Behavioral anchor: the `AC-005 / D-747 LOCKED` assert_eq! message string.) | D5 |
| `crates/prism-bin/tests/plugin_boot_tests.rs` | Audit and remove CrowdStrike plugin staging paths: all call sites referencing `crowdstrike-oauth2.prx` staging and `auth_plugin = "crowdstrike-oauth2"` boot assertions (grep `crowdstrike-oauth2` in the file to locate) | D5 |
| `crates/prism-bin/tests/helpers/mod.rs` | Remove `stage_crowdstrike_plugin` function (behavioral anchor: function name `stage_crowdstrike_plugin`) and all call sites that invoke it (grep `stage_crowdstrike_plugin` to locate; currently includes `write_org_config` and integration test setup functions) | D5 |
| `crates/prism-bin/fixtures/multi-org-prism.toml.template` | Remove comment referencing `crowdstrike-oauth2.prx` staging (behavioral anchor: comment text `crowdstrike-oauth2.prx staged for SEC-003`) | D5 |
| `scripts/demo-setup.sh` | Remove Step 5 (behavioral anchor: section header `# Step 5: Copy crowdstrike-oauth2.prx plugin`): delete `PLUGIN_ARTIFACT` variable definition, the `crowdstrike-oauth2.prx` copy command, the `crowdstrike-oauth2.manifest.toml` heredoc write, and the EC-003 hard-`exit 1` guard (`if [[ ! -f "${PLUGIN_ARTIFACT}" ]]`) — this guard exits 1 at demo time once the crate directory is deleted | D5 |
| `scripts/demo-run.sh` | Remove or update the `CROWDSTRIKE_BASE_URL must be "http://127.0.0.1"` comment block that references the `crowdstrike-oauth2 plugin manifest allowed_urls` SEC-003 constraint (behavioral anchor: comment text `crowdstrike-oauth2 plugin manifest allowed_urls`); the SEC-003 host-check restriction goes away with the plugin | D5 |
| `scripts/t13-preflight-audit.py` | Retarget or remove the `[A20] HANG-FIX: plugin_status returns promptly` probe: it issues `plugin_status` for `plugin_id = "crowdstrike-oauth2"` (behavioral anchor: the `plugin_id: "crowdstrike-oauth2"` argument in the `[A20]` params block); post-retirement this plugin_id no longer exists, so the probe must be retargeted to the remaining plugin (e.g., `threatintel-lookup`) or removed from the coverage matrix | D5 |
| `docs/DEMO-RUNBOOK.md` | Update the setup step list: remove item "Copies `crowdstrike-oauth2.prx` plugin artifact" and "Writes `crowdstrike-oauth2.manifest.toml`" (behavioral anchors: the step description strings); remove the `crowdstrike-oauth2` plugin manifest SEC-003 `allowed_urls` explanation; update the `CROWDSTRIKE_BASE_URL` constraint note to remove the plugin-manifest host-check rationale | D5 |
| ARCH-INDEX `adr_registry` AD-001 | Update "26 member workspace" / "crowdstrike-oauth2 plugin member" narrative → 25-crate workspace after this ADR removes `crowdstrike-oauth2`; `root Cargo.toml members` is source of truth. **Interaction with ADR-037:** ADR-037 also decrements by 1 (prism-customer-config retirement); the workspace count becomes 25 after ADR-054 alone and 24 once both ADRs have landed. Whichever ADR lands second MUST update to the combined value (24); do not hard-code 25 if ADR-037 has already merged. | D5 (architect-owned) |
| `CLAUDE.md` "26-crate workspace" count | Update `26-crate workspace (25 once ADR-037 retires...)` count — **HUMAN-FOLLOW-UP**: CLAUDE.md is human-maintained per project git rules; do NOT auto-edit; flag to human at PR time | D5 (human-follow-up) |
| `tests/fixtures/README.md` | Remove the H2 section `## \`crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx\`` and its full body (behavioral anchor: H2 section heading containing `crowdstrike-oauth2.prx`); update `wasi_snapshot_preview1.wasm` "How to update" step-3 reference to `just build-plugin-crowdstrike-oauth2` — stale post-retirement; `wasi_snapshot_preview1.wasm` itself is preserved for remaining `custom_via_plugin` plugins | D5 |
| `crates/prism-spec-engine/src/plugin/mod.rs` `test_F_LP7_MED_001_host_dispatch_acquire_token_component_model_path_emits_audit_event` | Remove or retarget this `#[ignore]` / `todo!()` integration test (behavioral anchor: function name `test_F_LP7_MED_001_host_dispatch_acquire_token_component_model_path_emits_audit_event`); test body requires a pre-built `crowdstrike-oauth2.prx` — post-retirement the plugin binary no longer exists; behavioral coverage migrates to `DeclarativeHttpAuthProvider` unit tests per the EC-001 through EC-006c port obligation above | D5 |
| Doc-hygiene sweep — preserved plugin-infrastructure files with stale `crowdstrike-oauth2` doc examples | Update or generalize `crowdstrike-oauth2` references in five preserved-infrastructure files: `crates/prism-spec-engine/src/spec_parser.rs` (`SensorSpec::auth_plugin` field doc-comment example `auth_plugin = "crowdstrike-oauth2"`); `crates/prism-spec-engine/src/plugin_auth_provider.rs` (module-level preamble doc + `PluginAuthProvider` struct `plugin_id` field doc — both cite `"crowdstrike-oauth2"` as canonical example); `crates/prism-spec-engine/src/error.rs` (`UnknownAuthPlugin` variant doc + `PluginAuthDispatchError::plugin_id` field doc — both cite `"crowdstrike-oauth2"`); `crates/prism-spec-engine/src/plugin/discovery.rs` (module-doc "crowdstrike-oauth2 plugin exports" + `find_host_interface_name` function doc + `validate_wit_interface` function doc — all cite `"prism:crowdstrike-oauth2/..."` WIT namespaces); `crates/prism-spec-engine/src/plugin/host_functions.rs` (`register_host_functions` function doc — two sites citing `"prism:crowdstrike-oauth2/host@0.1.0"`). Update examples to reference the remaining `custom_via_plugin` escape-hatch context or a hypothetical non-crowdstrike plugin | D5 (doc-hygiene) |
| `crates/` code doc-comment domain — census passes 3+4+5 (FIX-BURST 26 + 27 + 28 extension, per-SITE + compile-enforced class) | **Census note — per-SITE (NOT E0004) domain + compile-enforced class (updated FIX-BURST 28).** **Per-site census domain scope:** docs, comments, string literals, spec prose — the NOT-E0004 domain. Compile-enforced match exhaustiveness is a separate class handled below; those sites are listed for completeness but are NOT in the brace-list grep sweep domain. **FB-26 exclusion error (corrected):** excluded "spec_parser.rs and vp153_*.rs as whole files" when the correct exclusion granularity is per-CONSTRUCT — the enum/const/harness constructs already manifested by existing D11 rows were excluded, but doc-comments and inline comments in those same files were NOT excluded and were incorrectly swept past. Fixed in FIX-BURST 27 by adding three new rows. Tracers: `'custom_via_plugin}'` brace-closing pattern AND `', custom_via_plugin'` adjacency AND `'oauth2_client_credentials'` multi-value contexts (via grep `crates/`). **Final per-site dispositions for all 7 brace-list carrier sites (NOT-E0004 domain):** (1) `crates/prism-spec-engine/src/spec_parser.rs` `validate_cross_composition` fn doc-comment — **new-D11-row** (FIX-BURST 27, see row above); (2) `crates/prism-spec-engine/src/spec_parser.rs` inline comment above `VALID_AUTH_TYPES` — **new-D11-row** (FIX-BURST 27, see row above); (3) `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` module-level `//!` doc — **new-D11-row** (FIX-BURST 27, see row above); (4) `crates/prism-sensors/src/auth/mod.rs` `SensorAuth::auth_type_name()` doc-comment — **manifested** (FIX-BURST 26); (5) `crates/prism-spec-engine/src/error.rs` `AuthTypeCrossComposition` doc-comment — **manifested** (FIX-BURST 26); (6) `crates/prism-spec-engine/tests/bc_2_01_016_test.rs` test comment (non-normative) — **already-manifested** (BC-2.01.016 story scope); (7) `vp153_sensorauth_cross_composition.rs` multi-value attack strings simulating cross-composition input (`"[oauth2_client_credentials,bearer_static]"` / `"oauth2_client_credentials+bearer_static"`) — **single-value-non-carrier** (Rule A rejection test inputs, not a valid-set enumeration). **Compile-enforced no-wildcard match class (E0004):** sites where adding `TokenExchange` to the `AuthType` enum causes a hard compile error (E0004) if the match arm is absent — these cannot silently go stale. (1) `AuthType::as_str()` exhaustive no-wildcard match in `impl AuthType` (`crates/prism-spec-engine/src/spec_parser.rs`) — **sole E0004 site**; D11 row added FIX-BURST 28 for story-checklist completeness; rustc enforcement is stronger than the manifest. No other no-wildcard `match` on `AuthType` exists in `crates/` (grep confirms: `step9a_populate_adapter_registry` and `build_request` both carry wildcards — see wildcard class below). **Wildcard-bearing behavior-reviewed sites (NOT E0004, already dispositioned):** match expressions over `AuthType` that carry an `other =>` or `_ =>` wildcard arm — `token_exchange` falls into the existing catch-all; no D11 mutation row needed beyond the D7 dispatch-table row. (1) `step9a_populate_adapter_registry` with `other => {}` wildcard in `crates/prism-bin/src/spec_driven_adapter.rs` — D7 table row; (2) `build_request` with `_ =>` wildcard in `crates/prism-spec-engine/src/pipeline.rs` — D7 table row. **Census completeness claim:** every `AuthType` enumeration site in `crates/` is in exactly one of: (i) manifested D11 row (7 NOT-E0004 carrier sites), (ii) compile-enforced E0004 (1 site: `as_str()` in `impl AuthType`), or (iii) dispositioned wildcard (2 sites: `step9a_populate_adapter_registry`, `build_request`). Zero unmanifested normative enumeration carriers remain in `crates/`. | D1 (census note) |
| `crates/prism-sensors/src/auth/mod.rs` `SensorAuth::auth_type_name()` fn doc-comment (FIX-BURST 26) | **Direction (b) — SensorAuth-scope clarification (not token_exchange addition).** Reword the doc-comment to exclude `token_exchange` from `SensorAuth` scope. Updated doc (behavioral anchor: the "Must return one of the closed enumeration values defined in ADR-026 §D3" sentence, per TD-VSDD-091): "Must return one of the auth-type discriminator values from ADR-026 §D3 that are applicable to `SensorAuth` credentials: `\"oauth2_client_credentials\"`, `\"bearer_static\"`, `\"cookie_roundtrip\"`, `\"api_key\"`, `\"custom_via_plugin\"`. Note: `\"token_exchange\"` (ADR-054 D1) is NOT in `SensorAuth` scope — `token_exchange` auth acquisition is handled natively by `DeclarativeHttpAuthProvider` (prism-spec-engine, ADR-054 D2/D10b) and never produces a `SensorAuth` instance." Direction (b) rationale: a `SensorAuth` impl returning `"token_exchange"` would have no caller — the `DeclarativeHttpAuthProvider` path bypasses the credential store → `SensorAuth` resolution chain entirely. Adding `token_exchange` to the doc (direction a) would mislead external `SensorAuth` implementors into providing a meaningless `"token_exchange"` value. Update in same spec-evolution story as `spec_parser.rs` D1 extension. | D1 (doc-hygiene) |
| `crates/prism-spec-engine/src/error.rs` `AuthTypeCrossComposition` variant doc-comment (FIX-BURST 26) | Add `token_exchange` to the closed-enum brace expression in the `AuthTypeCrossComposition` variant doc-comment: current text `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` → update to `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange}`. Behavioral anchor: `AuthTypeCrossComposition` variant doc-comment brace expression in `error.rs` (grep-recoverable by brace-list text; TD-VSDD-091). This is distinct from the existing D5 doc-hygiene row for `error.rs` — that row covers `UnknownAuthPlugin` and `PluginAuthDispatchError::plugin_id` doc-comments (which cite `"crowdstrike-oauth2"` as a plugin example). This row covers the E-SPEC-012 validation set enumeration in `AuthTypeCrossComposition`. Update in same spec-evolution story as `spec_parser.rs` D1 extension and `error-taxonomy.md` E-SPEC-012 update (POL-24 atomicity: keep the runtime error code doc consistent with the `error-taxonomy.md` message template). | D1 (doc-hygiene) |
| `error-taxonomy.md` E-SPEC-028(c) message template | **[COMPLETED — D-1948]** The `valid values:` clause read `absolute_epoch_secs, ttl_secs`; corrected to `absolute_utc_string, relative_seconds` in error-taxonomy.md v2.58 to match the D3 field table and D10(c) message template. This was a PO authoring error in burst 3. BC-2.01.008's use of `expiry_mode = "absolute_utc_string"` is CORRECT per D3 and must NOT be changed. Behavioral anchor: E-SPEC-028(c) `valid values:` clause in the message_template column (grep-recoverable by `E-SPEC-028` id in error-taxonomy.md; TD-VSDD-091). | D3/D10(c) |
| `BC-2.16.009` Rule 10(c) description and EC-009-038 expected-error row | **[COMPLETED — D-1948]** Rule 10(c) and its companion EC-009-038 row referenced `{absolute_epoch_secs, ttl_secs}` as the valid-value set; corrected to `{absolute_utc_string, relative_seconds}` in BC-2.16.009 v1.13 to match the D3 field table and D10(c) ratified value set. Behavioral anchor: Rule 10(c) body text and EC-009-038 expected-error string in BC-2.16.009 (grep-recoverable by `EC-009-038` in BC-2.16.009; TD-VSDD-091). | D3/D10(c) |
| `crates/prism-spec-engine/src/spec_parser.rs` `FetchStep` struct doc-comment + `Default for FetchStep` impl doc-comment (F-WASE-P9-OBS-003) | Replace struct-literal + `..Default::default()` construction guidance with `FetchStep::new(...)` guidance at both sites. (1) `FetchStep` struct doc-comment — reword "use the `Default` impl or builder pattern for external construction" to state that external callers MUST use `FetchStep::new(...)`; struct-literal construction is E0639-impossible from external crates because `FetchStep` is `#[non_exhaustive]`. (2) `Default for FetchStep` impl doc-comment — remove the `FetchStep { name: "fetch".to_string(), ..Default::default() }` struct-literal example and replace with a note that `FetchStep::new(...)` is the correct external construction path. Root cause: the `#[non_exhaustive]` attribute (confirmed present on `FetchStep` in `spec_parser.rs`) makes struct-literal + `..Default::default()` E0639-impossible from external crates; the current doc-comments contradict this attribute and misled VP-159 AC-9b's harness skeleton into using E0639-invalid struct-literal syntax (caught as F-WASE-P9-MED-002 in wave-a-spec-evolution-fix-burst-9). Behavioral anchors: "use the `Default` impl or builder pattern for external construction" sentence in `FetchStep` struct doc-comment; `FetchStep { name: "fetch".to_string(), ..Default::default() }` example in `Default for FetchStep` impl doc-comment (grep-recoverable by `Default::default()` in `FetchStep` context in `spec_parser.rs`; TD-VSDD-091). Update in the engine story that authors `DeclarativeHttpAuthProvider` — VP-159 AC-9b harness uses `prism_spec_engine::spec_parser::FetchStep::new(...)` as the confirmed correct external construction path. | F-WASE-P9-OBS-003 (doc-hygiene) |

---

## Rationale

### Why Native Over Plugin for Standard Flows

ADR-023's Rule 4 mandate was correct for the problem it was solving: eliminating the legacy
Rust `CustomAdapter` trait that allowed unbounded sensor-specific logic in core code. The
WASM sandbox was the right isolation mechanism for that transition. However, the mandate
over-generalized from "arbitrary sensor logic should be sandboxed" to "ALL auth acquisition
requires WASM." For standard RFC 6749 flows, the plugin adds compile overhead, boot overhead,
and maintenance surface with no isolation benefit — there is nothing unsafe about a standard
form POST to an OAuth2 token endpoint that requires WASM sandboxing.

The `crowdstrike-oauth2.prx` plugin source is evidence: its entire business logic is
an HTTP POST, a JSON parse, and a KV-store write. The `HostInterface` trait, MockHost test
double, `WasmHost` wiring, and `wit_bindgen::generate!` overhead is pure scaffolding that wraps
those three operations. This scaffolding is justified for plugins that use host functions for
capabilities they cannot otherwise access; it is not justified when the host can express the
same logic natively.

The `DeclarativeHttpAuthProvider` replaces the plugin-side business logic with host-native Rust:
same reqwest client (already workspace-standard per ADR-050), same TTL semantics, same credential
resolution model (BC-2.06.003 tier chain), zero WASM ABI overhead.

### Why `token_exchange` as a Distinct AuthType

The coordinator's instruction (D-1895) was to use `token_exchange` for Armis (not
`custom_via_plugin`). The behavioral distinction is accurate: `token_exchange` is a defined
authentication pattern — POST a long-lived credential to receive a short-lived token, then use
that token. It differs from `oauth2_client_credentials` in form body structure, response path,
and expiry encoding. Labeling Armis as `oauth2_client_credentials` would be semantically wrong
(RFC 6749 defines a specific form body format; Armis does not follow it). `custom_via_plugin`
is no longer accurate now that a native provider handles the flow. `token_exchange` is the
correct semantic label.

From a provider construction standpoint, adding `token_exchange` to the `AuthType` enum adds
one new boot dispatch arm (D7 table) — a minimal change that the ADR-026 §D3 amendment
accommodates.

### Why the `[auth_acquisition]` Schema Is Structured This Way

`oauth2_client_credentials` is a standard (RFC 6749 §4.4) that defines the form body
(`grant_type=client_credentials`), the response format (`$.access_token`, `$.expires_in`),
and the error semantics. The engine can hard-code these. The only variables are `token_path`
(relative path to the token endpoint) and optionally `ttl_buffer_secs`. Requiring sensors to re-declare what the RFC specifies
would be redundant.

`token_exchange` is not a standard — it is a generic "POST a credential to get a token"
pattern used by sensors that don't implement OAuth2. The response format, expiry encoding,
and form field name vary by vendor. The `[auth_acquisition]` block for `token_exchange` must
therefore declare all variable parameters explicitly. The field set is minimal: only what is
genuinely variable between vendors. Future sensors using token exchange will extend the same
block without changing the provider implementation.

### Why Retire `crowdstrike-oauth2.prx` Immediately

Keeping `crowdstrike-oauth2.prx` alongside `DeclarativeHttpAuthProvider` for OAuth2 client
credentials creates two code paths for the same behavior. That duplication becomes a maintenance
liability (two sets of tests, two upgrade surfaces, two places to update if RFC 6749 behavior
changes). The CrowdStrike spec migration story has the minimum scope needed to retire the plugin
cleanly. Deferring the retirement while the native provider ships would introduce a split-brain
period where `oauth2_client_credentials` is sometimes plugin-backed and sometimes natively-backed
depending on whether `auth_plugin` is present.

---

## Consequences

### Positive

- Armis token-exchange auth ships without a new WASM plugin being authored, tested, and compiled
- CrowdStrike auth is now a pure Rust path — no WASM instantiation overhead at boot, no plugin
  KV indirection, no WIT binding maintenance
- `DeclarativeHttpAuthProvider` is a single unit-testable struct that covers both OAuth2 client
  credentials and generic token-exchange patterns; future sensors with similar patterns get native
  support at zero additional plugin cost
- Plugin infrastructure is preserved for `custom_via_plugin` sensors; the escape hatch is real
  and unimpeded
- BC-2.16.014 formalizes the token-lifecycle contract that was previously only implicit in the
  WASM plugin source, making it testable and verifiable via VP-159 (BC-2.16.014 authored D-1946, VP-159 authored D-1947; see D8/D9)

### Negative / Trade-offs

- CrowdStrike migration story must execute as an atomic burst: delete `auth_plugin = "crowdstrike-oauth2"`,
  add `[auth_acquisition]`, implement `DeclarativeHttpAuthProvider`, rewrite `step9a_populate_adapter_registry` dispatch (`spec_driven_adapter.rs`),
  delete the plugin crate — these cannot land independently without an intermediate broken state
- The `crowdstrike-oauth2.prx` plugin unit tests (13 tests covering EC-001 through EC-006c,
  cache behavior, URL encoding, form body content) are lost when the plugin is deleted. The
  equivalent behavioral coverage must migrate to `DeclarativeHttpAuthProvider` unit tests before
  the retirement story merges. (The tests themselves are a high-quality reference for what to
  cover: the same EC-NNN test table applies to `DeclarativeHttpAuthProvider`.)
- `E-SPEC-028` adds a new validation rule to `spec_parser.rs` and a new error code to `error-taxonomy.md`.
  Sensor specs authored after this ADR lands must include `[auth_acquisition]` for
  `oauth2_client_credentials` and `token_exchange`; any existing spec without the block fails.
  The only existing affected spec is `crowdstrike.sensor.toml` (migrated in the retirement story).

### Status as of 2026-07-22

Accepted (D-1943, human Wave-A approval gate 2026-07-22). Amendments to ADR-023 §Rule 4,
ADR-026 §D3, ADR-028 §D13, and amends_dis DI-012 are now OPERATIVE. CrowdStrike oauth2 migration
and `crowdstrike-oauth2.prx` retirement per D5 may proceed. Implementation stories land after
the ADR-053 standalone Wave-A engine story per §D7 merge-dependency.

---

## Alternatives Considered

- **Option A (rejected — custom_via_plugin + armis-token-exchange.prx):** ADR-053 v0.6 D2
  proposed `auth_type = "custom_via_plugin"` with a new `armis-token-exchange.prx` plugin.
  Rejected by human decision D-1895: the token-exchange flow is not genuinely arbitrary auth;
  a WASM plugin is unnecessary overhead for a standard HTTP form-POST pattern.

- **Option B (rejected — extend oauth2_client_credentials to cover Armis):** Override the
  RFC 6749 response parsing for `oauth2_client_credentials` to accept both `$.access_token`
  (RFC 6749) and `$.data.access_token` (Armis). Rejected: these are distinct protocols.
  Conflating them under one auth_type creates an ambiguous spec where the same `auth_type`
  value produces different behavior based on undeclared conditions. `token_exchange` as a
  separate variant makes the distinction explicit and schema-enforced.

- **Option C (rejected — generate-only: auto-generate WASM plugin from TOML):** Author a
  code-generation tool that produces a `crowdstrike-oauth2.prx`-style plugin from a declarative
  template. Rejected: this is more complex than a native Rust implementation, maintains the
  WASM boundary for no benefit, and defers the problem rather than solving it.

---

## Source / Origin

- **D-1895 (2026-07-20):** Human architectural decision authorizing native declarative auth
- **crowdstrike-oauth2 plugin source:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`
  — `acquire_token`, `get_token`, `HostInterface` trait, MockHost test double (business logic
  to port to `DeclarativeHttpAuthProvider`)
- **ADR-053 D2 (v0.6):** Armis token-exchange flow specification (POST `/api/v1/access_token/`,
  `$.data.access_token`, `$.data.expiration_utc`, `header_scheme = "raw"`) — unchanged in v0.7
  except TOML block replaces `custom_via_plugin + auth_plugin` with `token_exchange + [auth_acquisition]`
- **BC-2.01.016:** Plugin auth provider contract (EC-016-002 happy path; EC-016-005 unregistered
  plugin) — the `PluginAuthProvider` path is preserved for `custom_via_plugin`; `DeclarativeHttpAuthProvider`
  introduces a parallel native path
- **BC-2.06.003:** Four-tier per-client credential resolution chain — `DeclarativeHttpAuthProvider`
  resolves credential_refs through this chain at `acquire_token()` time
- **ADR-023 §Rule 4:** The rule being partially amended — standard HTTP token-acquisition
  is no longer classified as "genuinely arbitrary sensor-specific logic"
- **ADR-026 §D3:** The closed AuthType enum whose `VALID_AUTH_TYPES` const gains `"token_exchange"`

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 0.54 | 2026-07-25 | architect | FB51a (F-WASE-P64-MED-015, consistency follow-through): §D10 ADR-055 §D3 reconciliation block — updated parenthetical `(status: proposed)` → `(status: accepted)` following ADR-055 ratification in this burst. The factual characterization of Rule 10 (interpolation-independent, executes inside `SpecLoader::parse()`) is unchanged; only the status annotation is updated. No behavioral or structural content altered. |
| 0.53 | 2026-07-25 | architect | FB50 (F-WASE-P64-CRIT-002): §D10 — added Rule 10 execution-site statement and ADR-055 §D3 reconciliation block. Establishes that Rule 10 executes inside `SpecLoader::parse()`, not `validate_sensor_spec()`, with per-sub-condition verification that all 8 Rule 10 sub-conditions are interpolation-independent (none reads `base_url` or any env-var-interpolated field; `token_path` is explicitly "a literal relative path string — no env-var interpolation" per §D3). Rationale: (1) BC-2.16.009 §Integration function is explicit; (2) Rule 10 reads no interpolated field; (3) ADR-055 §D3 scoping is limited to Rules 1–5; (4) `validate_sensor_spec()` has zero production callers per ADR-055 §Context. ADR-055 remains proposed; the factual characterization of Rule 10 is independent of ADR-055's ratification. modified: synced. |
| 0.52 | 2026-07-24 | architect | F-WASE-P56-MED-001 (WAVE-A FIX-BURST 43): §D11 `AuthAcquisitionConfig` constructor row amended — ratified `new_oauth2` (Option a adjudication). Root cause: v0.48 designated `new(token_path, expiry_mode, ttl_buffer_secs)` as "used for `oauth2_client_credentials`" but `expiry_mode` is a `token_exchange`-only field per §D3; §D10(h) rejects `expiry_mode` on an `oauth2_client_credentials` block at spec-load time; no valid `ExpiryMode` exists for an oauth2 config (the TOML field is absent; supplying any variant would be a semantic lie). `new_oauth2(token_path, ttl_buffer_secs)` (no ExpiryMode) precisely models §D3: oauth2 expiry is always relative seconds via `$.expires_in`, engine-internal. Three-constructor API: `new_oauth2` (oauth2_client_credentials, 2 params), `new` (token_exchange minimal harness, 3 params with ExpiryMode), `new_token_exchange` (full token_exchange, 6 params). D11 constructor row: "two named constructors" → "three named constructors"; `new(...)` re-scoped to token_exchange minimal harness only (ExpiryMode required — makes sense for token_exchange; not for oauth2); `new_oauth2` added as constructor (3) with §D3/§D10(h) rationale and F-WASE-P56-MED-001 provenance. FB42 changelog claim "No ADR-054 edits needed" was incorrect — VP-159 v1.25 introduced `new_oauth2` in the harness but ADR-054 was not correspondingly amended, creating a cross-artifact contradiction: an engine-story implementer building from §D11 would construct only `new` + `new_token_exchange` and the VP-159 AC-7d harness would not compile. POL-29 sweep: "two named constructors" phrase at v0.48 changelog row is historical (immutable per forward-correction pattern); zero other live-body constructor-count sites in ADR-054. VP-159 v1.26 (companion burst) adds traceability row. modified: synced. |
| 0.51 | 2026-07-23 | architect | RU-Q1/Q2/Q3 + RU-Scanner-1 amendments (Wave-A remove-uncertainty burst D-1944 step 5): §D4 step 4 `absolute_utc_string` parse — strict RFC-3339 (`parse_from_rfc3339`) replaced with lenient chrono relaxed `FromStr` (`s.parse::<DateTime<FixedOffset>>()`) accepting both `T`-separator ISO-8601 and space-separated UTC strings; rationale: Option B (proactive TTL + reactive P6 401-backstop) per RU-Q1; chrono Context7 docs confirm relaxed `FromStr` accepts space separator. §D2 `$.expires_in` — lenient deserialization added: accept JSON number (`as_u64()`) OR numeric string (str.parse::<u64>()); non-numeric/wrong type → default 1799 per RU-Q2 (Microsoft Entra ID confirmed emits string `"3599"`; RFC 6749 does not fix the JSON type). §D2 wire-encoding note added: reqwest `.form()` (serde_urlencoded) is byte-equivalent for `+/=` per RU-Q3. §D11 new row: non-exhaustive gate EXPECTED bump 92→95 for `AuthAcquisitionConfig`/`CachedAuthToken`/`ExpiryMode` (RU-Scanner-1); all-three-update-sites constraint documented. modified: synced (POL-27/32). |
| 0.50 | 2026-07-23 | architect | F-WASE-P43-MED-001 (FIX-BURST 35): §D11 VP-153 §Feasibility Assessment row (line 669) — corrected cross-artifact contradiction with VP-153 source of truth (Source-of-Truth Precedence rule 4). Removed erroneous claim "token_exchange reuses the ApiKey single-string shape (structural shapes stay at 5)" — this conflated the credential RECORD-schema notion with VP-153's identifier-based SHAPE model. Root cause: v0.30 OBS-3 rewrite introduced the conflation when separating "typed structural shapes" (5) from "string-harness auth_type identifiers" (6); but VP-153's identifier-based model (reported_shape IS auth_type identifier per vp153_rule_c_shaped_probe.rs) makes token_exchange a DISTINCT 6th shape, not an ApiKey alias. Internal incoherence in removed text: "5 mismatched shapes per variant" is impossible with only 5 total shapes (max 4). Replacement states: token_exchange is a DISTINCT 6th credential structural shape in the harness model; record-schema similarity (single-string secret) is a RECORD-schema observation that does NOT alter the shape model; total shapes = 6; mismatched per variant = 5 (6 − 1 matching); 6 × 5 = 30 ordered pairs — now coherent with VP-153 §Feasibility v0.26 ("of the 6 total credential structural shapes, excluding the matching one"). POL-29 sweep: one live-body hit at row 669 (fixed); two changelog hits (v0.30 line 885, v0.17 line 898) — exempt per forward-correction pattern. ADR-053 sweep: zero hits — clean. At-commit-time hash per POL-32. |
| 0.49 | 2026-07-23 | architect | F-WASE-P40-MED-001: §D10(c) trigger-scope adjudication — token_exchange-gating is canonical. Three artifacts had divergent trigger scope: §D10(c) (token_exchange-gated), BC-2.16.009 Rule 10(c) (any auth_type), error-taxonomy.md E-SPEC-028(c) (any auth_type). §D10(c) is declared authoritative for trigger logic per the §D10 meta-note (taxonomy wins on wording, §D10 wins on trigger logic). Four-reason rationale: (1) `expiry_mode` is token_exchange-only; sub-condition (h) already fires for wrong-position use and emitting (c) additionally would produce contradictory repair guidance for LLM-agent-consumed errors; (2) value-validity presupposes positional validity (structural parallel with §D10(b)'s scope); (3) co-fire list consistency — taxonomy illustrative list already omits `(c)∩(h)` and `(c)∩(g)`, consistent with narrow scope only; (4) clean (c)/(h) partition: value-validity for valid-position (`token_exchange`) vs position-validity for wrong-position (non-`token_exchange`). §D10(c) adjudication blockquote added (parallel form to §D10(b) F-WASE-P2-HIGH-001 note). PO sweep directives in adjudication note: (a) BC-2.16.009 Rule 10(c) narrowed to token_exchange-gated with exact replacement text; (b) BC-2.16.009 EC-009-038 auth_type clarification with exact replacement text; (c) error-taxonomy E-SPEC-028(c) description narrowed with exact replacement text; (d) co-fire list requires no changes. POL-29 sweep: VP-153/VP-159/ADR-053/ADR-026/ADR-028 have no (c)-trigger-scope prose — zero architect-owned sites beyond this ADR. At-commit-time hash per POL-32. |
| 0.48 | 2026-07-23 | architect | F-WASE-P38-MED-001 (FIX-BURST 33): §D11 two new rows added for `AuthAcquisitionConfig` and `CachedAuthToken` constructors — both structs require `#[non_exhaustive]` per CLAUDE.md convention for public TOML-deserialized types and new pub types in prism-spec-engine; external struct-literal construction is E0639-impossible from the `tests/` crate. `AuthAcquisitionConfig` gains two named constructors: `new(token_path: impl Into<String>, expiry_mode: ExpiryMode, ttl_buffer_secs: u64) -> Self` (common fields only; token-exchange-specific fields default to empty string; used for `oauth2_client_credentials` and minimal harness configs) and `new_token_exchange(token_path, credential_body_field, token_response_path, expiry_field, expiry_mode, ttl_buffer_secs) -> Self` (full form for `token_exchange` auth_type). `CachedAuthToken` gains `new(token: String, expires_at: u64) -> Self`; no credential parameter in the constructor signature enforces BC-2.16.014 P7 / AD-017 / INV-014-003 credential-opacity architecturally. VP-159 v1.22 (companion burst) rewrites 5 harness sites to use these constructors (base_config helper, AC-4b, AC-6, AC-6b) and replaces the old AC-8 structural-assertion test with an async runtime-inequality test (`test_vp159_ac8_cached_token_no_credential_value_stored`). Symmetric to the FetchStep row (F-WASE-P9-OBS-003). At-commit-time hash per POL-32. |
| 0.47 | 2026-07-23 | architect | F-WASE-P32-MED-001 (FIX-BURST 29): §D11 reverse-reconciliation sweep — marked 6 additional spec-doc rows EXECUTED that had been completed in prior bursts but whose D11 markers were not updated: (1) row 622 ADR-053 D5 manifest: [EXECUTED — ADR-053 v0.7, 2026-07-20] BC-2.01.008 amendment updated to token_exchange native provider; (2) row 623 ADR-026 §D3 (partial): [EXECUTED — ADR-026 v1.35, 2026-07-20] amended_by back-ref + §D3 amendment note + at-point annotation; (3) row 626 BC-2.16.001: [EXECUTED — BC-2.16.001 v1.9, Wave-A spec evolution burst 3, 2026-07-22] both §Postconditions parenthetical + §Auth Type Resolution sites updated to 6-value set; (4) row 627 BC-2.06.003: [EXECUTED — BC-2.06.003 v1.12, Wave-A spec evolution burst 3, 2026-07-22] CrowdStrike auth provider column updated; (5) row 647 E-SPEC-005: [EXECUTED — error-taxonomy.md v2.57, Wave-A spec evolution burst 3, 2026-07-22] notes column modernized to canonical 6-value set — this is the primary F-WASE-P32-MED-001 finding; (6) row 648 ADR-028 §D13: [EXECUTED — ADR-028 v1.18/v1.20, 2026-07-21] PluginAuthProvider marked spec-load-rejected, Armis blockquotes updated, conditional framing removed. Sweep completeness: all remaining unmarked D11 rows are either code/engine-story PENDING (legitimately awaiting implementation) or explicitly annotated (HUMAN-FOLLOW-UP for CLAUDE.md). At-commit-time hash per POL-32. |
| 0.46 | 2026-07-23 | architect | F-WASE-P31-MED-002 (FIX-BURST 28): §D11 execution state corrected — added [EXECUTED] markers to 12 spec-doc rows whose amendments had landed in prior bursts: BC-2.16.009 Rule 10 (burst 3), error-taxonomy.md E-SPEC-028 registration (burst 3), DI-012 v1.8 (burst 3), BC-2.01.016 §Related BCs (burst 3), BC-2.01.017 §Preconditions/§P3/§Related BCs (burst 3), BC-2.16.009 §Validation Rules (burst 3), VP-153 rows 633–636 (VP-153 v0.21 burst 3). Corrected false "current text ends...custom_via_plugin" snapshot in row 625 (DI-012): past-tense "amended text now reads...token_exchange" with back-ref execution note. Re-scoped row 645 (E-SPEC-012): Step 1 marked [EXECUTED — error-taxonomy.md v2.57 burst 3]; corrected "current text ends" to "amended text now reads"; POL-24 atomicity re-scoped — two prose sites (error-taxonomy.md v2.57 + VP-153 v0.21) already executed in burst 3; only `error.rs` `AuthTypeCrossComposition` `#[error(…)]` attribute remains pending at engine-story time. Added new code-alignment row for `error.rs` `MultipleCredentialRefs` Display (E-SPEC-013): as-built emits hardcoded "1" with no `{auth_type}` or `{expected}` parameters; implementer aligns to error-taxonomy.md v2.64 template at engine-story time (provenance: F-WASE-P31-MED-001). At-commit-time hash per POL-32. |
| 0.45 | 2026-07-23 | architect | F-WASE-P29-OBS-001 (FIX-BURST 27): §D11 new row added — `vp153_sensorauth_cross_composition.rs` test-inventory `//!` doc table row-4 fn-name correction routed to implementer at engine-story time. As-typed name `prop_rule_b_single_or_zero_credential_refs_accepted` does not match the defined `fn` `prop_rule_b_single_credential_ref_accepted` (confirmed at the `proptest!` fn declaration); one-word divergence (`single_or_zero` vs `single`). Behavioral anchor: `prop_rule_b_single_or_zero_credential_refs_accepted` string in the test-inventory table row-4 Name column (grep-recoverable by `single_or_zero`; TD-VSDD-091). No behavioral change — doc-string-only correction. At-commit-time hash per POL-32. |
| 0.44 | 2026-07-23 | architect | F-WASE-P23-LOW-001 (FIX-BURST 22): §Status "Current contract highlights" — Definition-1 purge of E-SPEC-028(b) unconditional language: "E-SPEC-028(b) unconditionally rejects `auth_plugin` for declarative auth_types per D10(b)" → "E-SPEC-028(b) rejects `auth_plugin` for `auth_type ∈ {oauth2_client_credentials, token_exchange}` regardless of `[auth_acquisition]` presence (Definition 1, D10(b))". At-commit-time hash per POL-32. |
| 0.43 | 2026-07-23 | architect | F-WASE-P21-HIGH-001(b): (1) §D4 body — "restoring the caching behavior of the retired `crowdstrike-oauth2.prx` plugin" → "restoring the caching behavior of the `crowdstrike-oauth2.prx` plugin (to be retired by the §D2/§D5 migration story)" — forward framing; plugin is not yet retired. (2) Frontmatter `amends:` ADR-028 note — "crowdstrike-oauth2.prx plugin retired per D5" → "crowdstrike-oauth2.prx plugin to be retired per D5" — same forward framing. Source: both sites must match ADR-054 §D5 retirement-story prose framing (future action). (POL-32) |
| 0.42 | 2026-07-23 | architect | F-WASE-P16-OBS-003: §D11 `error-taxonomy.md` E-SPEC-012 message template row amended — extended from "add `token_exchange` as 6th value" to cover the FULL wording delta between as-built `error.rs` `AuthTypeCrossComposition` Display and the taxonomy template. Row now: (1) states the divergence in BASE WORDING (not only missing `token_exchange`); (2) cites both current wordings verbatim as behavioral anchors (code `#[error(…)]` attribute wording vs taxonomy `message_template` column wording); (3) directs the engine story to rewrite the `AuthTypeCrossComposition` `#[error(…)]` attribute to match the taxonomy template VERBATIM under POL-24; (4) expands POL-24 atomicity obligation from two to three sites (error-taxonomy.md + error.rs + VP-153 §Property Statement, same commit). Field substitution variable divergence also called out (`{provided_value}` in code vs `{value}` in taxonomy). (POL-32) |
| 0.41 | 2026-07-22 | architect | F-WASE-P14-HIGH-001 (fix-burst 14): §D9 Tool description corrected — "MockHttpClient for network isolation — behavioral state-transition sequences, not combinatorial input generation; analogous to VP-033/VP-036" → "wiremock network-level interception — token_url routed to MockServer; now_fn clock seam via new_for_test for TTL determinism". Stale reference from v0.20 (FIX-BURST 20) that predated OPTION (b) ratification in v0.40; v0.40 eliminated MockHttpClient from §D4/Constructor but missed the §D9 Tool bullet. MockHttpClient sweep across .factory/specs/: all remaining hits in ADR-054, VP-159, and VP-INDEX.md are immutable changelog rows or explicit negative references ("no MockHttpClient needed") — no further architect-owned positive/design references remain. PO-owned hit (BC-2.16.014 §Verification Properties row) reported to PO for parallel fix. POL-32: at-commit-time hash wording. |
| 0.40 | 2026-07-22 | architect | F-WASE-P13-MED-001 + OBS-P13-001 (fix-burst 13): §D4 Internal state completeness + HTTP-client/clock interception resolution. (F-WASE-P13-MED-001) Ratified OPTION (b) — internal reqwest client, no HTTP injection seam in production constructor. §D4 Internal state list extended with three new fields: `token_url: String` (per-org derived at step 9A, immutable after construction); `http_client: reqwest::Client` (ADR-050-compliant, internally constructed via `build_http_client_with_timeout()`, not exposed in production constructor API); `now_fn: Arc<dyn Fn() -> u64 + Send + Sync>` (narrow clock seam — production default = real SystemTime; overridable ONLY via `new_for_test` test constructor gated `#[cfg(any(test, feature = "test-helpers"))]`). §D4 acquire_token step 3 updated: "using the workspace reqwest client per ADR-050" → "using `self.http_client` (internally constructed per ADR-050; not exposed in production constructor)". §D4 get_token step 2 updated: `unix_now()` → `(self.now_fn)()`. §D4 Constructor paragraph added documenting the 3-arg production constructor, the gated `new_for_test` constructor, and the wiremock test-interception pattern (token_url → wiremock server URI; no MockHttpClient). (OBS-P13-001) `token_url: String` field added to Internal state — it was referenced in §D4 algorithm prose ("POST to `self.token_url`") and BC-2.16.014 P1/INV-014-002 but absent from the enumeration prior to this version. BC-2.16.014 INV-014-007 is already consistent with OPTION (b) — no BC follow-up needed. |
| 0.39 | 2026-07-22 | architect | F-WASE-P9-OBS-003: §D11 new row added — `FetchStep` struct doc-comment + `Default for FetchStep` impl doc-comment correction: both carry struct-literal + `..Default::default()` construction guidance that is E0639-impossible from external crates (`FetchStep` is `#[non_exhaustive]`; confirmed at `spec_parser.rs`). The misleading doc guidance caused VP-159 AC-9b's harness skeleton to use E0639-invalid struct-literal syntax (F-WASE-P9-MED-002). New D11 row directs replacement with `FetchStep::new(...)` guidance at engine-story time. Behavioral anchors: "use the `Default` impl or builder pattern for external construction" sentence in `FetchStep` struct doc-comment; `FetchStep { name: "fetch".to_string(), ..Default::default() }` example in `Default for FetchStep` impl doc-comment. |
| 0.38 | 2026-07-22 | architect | F-WASE-P7-HIGH-001: §D4 amended — added `AuthProvider` trait extension subsection and `PipelineExecutor` call-site dispatch table. `get_token()` is added to the `AuthProvider` trait with a default impl delegating to `acquire_token()`; all 7 existing implementors are unaffected. `execute_impl` and `execute_step` normal eager paths change to `get_token()` (cache-aware; zero token-POST on warm cache); `issue_request_with_retry` 401-refresh path remains `acquire_token()` (force-refresh). §D11 four new engine-story rows added: (a) `AuthProvider` trait `get_token` addition in `auth_provider.rs`; (b) `execute_impl` normal-path call-site change; (c) `execute_step` normal-path call-site change; (d) `issue_request_with_retry` 401-path no-change note (intentional). |
| 0.37 | 2026-07-22 | architect | F-WASE-P4-OBS-002: §D11 engine-story gate row added — `VP-153` proof re-run obligation: after the engine story activates the `token_exchange` proptest arms (dropping `[PLANNED]` markers per the existing §Proof Harness Skeleton and §Feasibility Assessment D11 rows), the engine story MUST re-run all 8 VP-153 proptests with those arms active as an explicit story gate before the PR merges. Behavioral anchors: `Just("token_exchange")` in `arb_valid_auth_type()` (FILE 1); `Just("token_exchange")` in `arb_matching_auth_type()` and `(0usize..6, 0usize..5)` in `arb_mismatched_auth_type_pair()` (FILE 2). Companion change: VP-153 v0.22 §Re-verification Gate section added. |
| 0.36 | 2026-07-22 | architect | F-WASE-P3-HIGH-001: §Consequences Positive burst attribution corrected — "(both authored in D-1946; see D8/D9)" → "(BC-2.16.014 authored D-1946, VP-159 authored D-1947; see D8/D9)". Sibling of D9 blockquote fix at v0.34 (D-1950). |
| 0.35 | 2026-07-22 | architect | F-WASE-P2-HIGH-001 + F-WASE-P2-MED-001: (HIGH-001) §D10(b) adjudicated — Definition 1 (`auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` present, regardless of `[auth_acquisition]`) is canonical; Definition 2 (taxonomy's prior "auth_plugin + [auth_acquisition] both declared, regardless of auth_type") is superseded. Expanded §D10(b) with explicit "Fires when:" wording, adjudication note citing four rationale points (§D2 design intent, §D7 unreachable-claim dependency, (g) redundancy under Def-2, message template auth_type-centricity), and PO sweep directive for required taxonomy update. (MED-001) Added meta-note block at top of §D10 message templates: taxonomy wins on wording conflicts; §D10 is condition-authoring intent; §D10(b) is the stated exception where taxonomy must follow §D10. §D10(h) semantics aligned to taxonomy: changed from per-field emission (`{field_name}` singular) to single aggregated emission (`{field_list}`, matching taxonomy v2.57 E-SPEC-028(h) cardinality). |
| 0.34 | 2026-07-22 | architect | F-WASE-P1-MED-001: §D8 blockquote burst attribution corrected burst 2→burst 1 for BC-2.16.014 (D-1946 is burst 1); §D9 blockquote marker corrected [AUTHORED — D-1946]→[AUTHORED — D-1947] + "decision D-1946, simultaneously with BC-2.16.014"→"decision D-1947; authored after BC-2.16.014 (burst 1, D-1946)"; §D11 BC-2.16.014 row burst 2→burst 1; §D11 VP-INDEX.md row [REGISTERED — D-1946]→[REGISTERED — D-1947] + "simultaneously with BC-2.16.014 authoring" corrected. F-WASE-P1-MED-002: §D11 error-taxonomy.md E-SPEC-028(c) row and BC-2.16.009 Rule 10(c)/EC-009-038 row both updated [PO FOLLOW-UP REQUIRED — Wave-A burst 3 adjudication]→[COMPLETED — D-1948] with past-tense descriptions; §D10(c) note past-tensed and stale v2.57 reference updated to v2.58/v1.13 (D-1948). |
| 0.33 | 2026-07-22 | architect | Wave-A burst 3 expiry_mode adjudication (Task 2): D10(c) ratification note added confirming `absolute_utc_string` and `relative_seconds` as the sole ratified `expiry_mode` values — all three design-authority sites self-consistent: D3 field table (`"absolute_utc_string"` or `"relative_seconds"`), D4 algorithm (RFC-3339 parse / u64 TTL default 1799), and Armis wiring example (`expiry_mode = "absolute_utc_string"`). BC-2.01.008's use of `expiry_mode = "absolute_utc_string"` is CORRECT per D3. Root cause of conflict: error-taxonomy.md v2.57 E-SPEC-028(c) and BC-2.16.009 Rule 10(c)/EC-009-038 were authored with incorrect values `absolute_epoch_secs, ttl_secs` during burst 3 (PO authoring errors). Two D11 PO follow-up rows added: (1) error-taxonomy.md E-SPEC-028(c) `valid values:` clause correction → `absolute_utc_string, relative_seconds`; (2) BC-2.16.009 Rule 10(c)/EC-009-038 correction → `{absolute_utc_string, relative_seconds}`. Both routed to product-owner. |
| 0.32 | 2026-07-22 | architect | Wave-A spec evolution burst 2 (D-1946): §D4 error-variant corrected — acquisition-level failures use `AuthAcquisitionFailed` (E-AUTH-001), not `AuthRefreshFailed`; `AuthRefreshFailed` (E-AUTH-002) is double-401 only. BC-2.16.014 and VP-159 [PLANNED] markers cleared — 8 sites: frontmatter `related_bcs_planned` removed (BC-2.16.014 moved to `related_bcs`); D8 blockquote → [AUTHORED — D-1946]; D8 postconditions summary note updated to present tense; D9 blockquote → [AUTHORED — D-1946]; D9 prose "will cover" → "covers"; D9 tail "will be created as DRAFT" → "is registered as DRAFT; D11 manifest rows [PLANNED] → [AUTHORED/REGISTERED — D-1946]; Consequences [PLANNED] prefix removed. |
| 0.31 | 2026-07-22 | state-manager | STATUS → ACCEPTED — human Wave-A approval gate 2026-07-22 (D-1943). Amendments to ADR-023/026/028 and amends_dis DI-012 now EFFECTIVE (DI-012 content amendment itself executes in the spec-evolution story per D11). |
| 0.30 | 2026-07-22 | architect | FIX-BURST 30 (OBS-2 + OBS-3 + OBS-1 forward-correction): [OBS-2] Stripped 11 volatile line pins from live D11 rows + census note per TD-VSDD-091 — D11 rows for `validate_cross_composition` fn doc-comment, `validate_cross_composition` inline comment, vp153 `//!` module doc, and `AuthTypeCrossComposition` doc-comment; census note 7-site list. Pins replaced with grep-recoverable behavioral anchors (function names + quoted brace-list text). [OBS-3] VP-153 §Feasibility Assessment D11 row rationale rewritten to separate the two counting models: typed structural shapes stay at 5 (token_exchange reuses ApiKey single-string shape); string-harness auth_type identifier set grows 5→6; hence 6×5=30 pair space. Removed conflating sentence "Credential string-shape space stays at 5+1=6 string identifiers." [OBS-1 forward-correction] v0.26 changelog justification "VP-153 uses timestamp: not modified: (no mismatch possible)" was inaccurate — VP-153 carries BOTH `timestamp:` (line 7) AND `modified:` (line 27); the v0.26 sweep coincidentally found them in-sync (modified: 2026-07-21 matches top changelog row 0.20 \| 2026-07-21, verified). Historical v0.26 row is unchanged per forward-correction pattern. ADR-053 live-body line-pin sweep: zero hits (clean). |
| 0.29 | 2026-07-22 | architect | FIX-BURST 29 (HIGH-1): Census note wildcard-site symbol corrected — `build_request_with_auth` does not exist in the repo; the real symbol is the module-level free function `build_request` (`crates/prism-spec-engine/src/pipeline.rs`, 8 params, no `&self`, `_ =>` Bearer catch-all at auth_type match). Three occurrences in census note body updated: (1) grep-confirms sentence; (2) wildcard-site (2) bullet; (3) census completeness claim `(iii)` list. v0.28 changelog reference to `build_request_with_auth _ =>` is historical — left as-is per forward-correction pattern. |
| 0.28 | 2026-07-22 | architect | FIX-BURST 28 (MED-1 + compile-enforced class): `AuthType::as_str()` exhaustive no-wildcard match in `impl AuthType` (`spec_parser.rs`) added as D11 row — sole E0004-compile-enforced AuthType match site; `TokenExchange => "token_exchange"` arm required; row exists for story-checklist completeness (compiler catches omission). Census note row rescoped: per-site census scope covers docs/comments/string literals/spec prose (the NOT-E0004 domain); compile-enforced no-wildcard matches form a separate class listed in the census. Full no-wildcard match enumeration: only `as_str()` in `impl AuthType` (1 site); all other AuthType matches (`step9a_populate_adapter_registry other =>` in `spec_driven_adapter.rs`, `build_request_with_auth _ =>` in `pipeline.rs`) carry wildcards and are behavior-reviewed. |
| 0.27 | 2026-07-22 | architect | FIX-BURST 27 (HIGH-1 + census granularity correction): Three uncovered doc/inline-comment enumeration carriers in spec_parser.rs and vp153_sensorauth_cross_composition.rs added as D11 rows: (1) `validate_cross_composition` fn doc-comment brace list at lines 1369-1371; (2) inline comment repeating the same brace list at line 1392 immediately above `VALID_AUTH_TYPES`; (3) `vp153_sensorauth_cross_composition.rs` module-level `//!` doc brace list at line 8. Census note row corrected: file-level exclusion error documented — FB-26 excluded "spec_parser.rs and vp153_*.rs as whole files" when the correct exclusion granularity is per-CONSTRUCT (the enum/const/harness constructs already manifested by existing D11 rows); doc-comments and inline comments in those files were NOT excluded. Final per-site re-census: zero unmanifested normative enumeration carriers remain across .factory/ and crates/. |
| 0.26 | 2026-07-22 | state-manager | D-1930 pass-37 MED-1 fix: frontmatter `modified:` date synced 2026-07-21→2026-07-22. Root cause: v0.25 edit (FIX-BURST 26) crossed the date rollover from 2026-07-21 to 2026-07-22; the changelog top row was correctly dated 2026-07-22 but the frontmatter `modified:` field retained the pre-midnight value 2026-07-21. State-manager-owned frontmatter sync. Class sweep CLEAN: ADR-023/026/028/031/032/053 frontmatter modified dates all match their respective changelog top-row dates; VP-153 uses `timestamp:` not `modified:` (no mismatch possible). Zero sibling mismatches. |
| 0.25 | 2026-07-22 | architect | FIX-BURST 26 (MED-1 + census extension): `crates/` code doc-comment domain swept for enumeration carriers — census pass 3, tracers: `'custom_via_plugin'`, bearer_static/cookie_roundtrip adjacency, `'oauth2_client_credentials'` enumeration contexts, `'closed enumeration'`. Two new D11 rows added: (1) `crates/prism-sensors/src/auth/mod.rs` `SensorAuth::auth_type_name()` fn doc-comment — direction (b): reword to exclude `token_exchange` from `SensorAuth` scope with rationale (`token_exchange` handled by `DeclarativeHttpAuthProvider`, never produces a `SensorAuth` instance; ADR-054 D2/D10b); (2) `crates/prism-spec-engine/src/error.rs` `AuthTypeCrossComposition` variant doc-comment — add `token_exchange` as 6th variant in brace expression. Census note row added recording all hits with per-hit dispositions. Per-hit census: `bc_2_01_016_test.rs` line 48 comment (already-manifested, BC-2.01.016 story scope); all remaining `custom_via_plugin` test single-value occurrences (single-value-non-carrier). Zero uncovered normative enumeration carriers remain in `crates/`. |
| 0.24 | 2026-07-21 | architect | FIX-BURST 24 (MED manifest fix): D11 VP-153 §Feasibility Assessment row (line 516) and §Proof Harness Skeleton row (line 518) rewritten against as-built constructs. Root cause: FB-22 "names are as-built per VP-153 v0.18" claim was false — the as-built harness uses string-based `&'static str` generators, not typed-enum helpers. Phantom constructs removed: `is_coherent_pair`, `MockCredentialType::ApiKey`, `AuthType::TokenExchange`, `matching_credential_for`, `valid_auth_type_credential_pairs_accepted`, `mismatched_auth_type_credential_rejected`. §Feasibility Assessment row now states harness amendments in terms of real constructs: `VALID_AUTH_TYPES: &[&str]` (both files), `arb_valid_auth_type()` `prop_oneof!` arm, `arb_matching_auth_type()` `prop_oneof!` arm (prism-bin), `arb_mismatched_auth_type_pair()` range bounds `(0..5, 0..4)→(0..6, 0..5)`. §Proof Harness Skeleton row corrected: `arb_auth_type()` → `arb_valid_auth_type()`; `Just(AuthType::TokenExchange)` → `Just("token_exchange")`; no inline E-SPEC-012 comment or separate prop_filter edit needed (filter references VALID_AUTH_TYPES constant). POL-22 Phase C: all symbols verified against `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` and `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs`. |
| 0.23 | 2026-07-21 | architect | FIX-BURST 23 (MED-1): D11 fourth VP-153 row added — `VP-153` §Proof Method member-count sentence "has 5 members" → "6 members" was not covered by any existing D11 row; only the §Feasibility Assessment "25 pairs" and §Property Statement Rule A enumerated set were covered. VP-153 in-file count census: two count-bearing sites (line 86 §Proof Method — NEW; line 259 §Feasibility Assessment — already in D11); two other "members" occurrences (lines 145, 148) are array-cardinality description and prop_filter logic, not auth_type set size claims — no amendment needed. New row inserted between §Feasibility Assessment and §Proof Harness Skeleton rows. In-scope for same spec-evolution story as §Feasibility Assessment correction. |
| 0.22 | 2026-07-21 | architect | FIX-BURST 22 (OBS-1): D11 VP-153 §Feasibility Assessment row — Harness amendments implied (2) corrected and (3) added. (2) was "prop_filter exclusion list must also exclude (TokenExchange, ApiKey) from incoherent-pair generation" — mis-attributed and redundant: `mismatched_auth_type_credential_rejected` uses `prop_assume!(!is_coherent_pair(...))`, not prop_filter; once `is_coherent_pair` gains (TokenExchange, ApiKey) acceptance the pair auto-excludes with no separate mechanism. Load-bearing omission added as new (2): `matching_credential_for(&auth_type)` must gain a `TokenExchange → MockCredentialType::ApiKey` arm or `valid_auth_type_credential_pairs_accepted` fails for the new variant. New (3) explicitly documents that `mismatched_auth_type_credential_rejected` needs no separate change and why. Implementer note added directing validation against as-built test files (VP-153 v0.18). Helper names cited by actual function name per TD-VSDD-091. POL-29: sole phantom prop_filter-exclusion claim was this row; §Proof Harness Skeleton row's prop_filter mention (Rule A out-of-set string arm) is correct and unchanged. |
| 0.21 | 2026-07-21 | architect | FIX-BURST 21 (HIGH-1): §Consequences Negative/Trade-offs — `update boot.rs dispatch` → `rewrite step9a_populate_adapter_registry dispatch (spec_driven_adapter.rs)`; boot.rs is NOT modified (construction site is spec_driven_adapter.rs step9a per D5/D7/D11/Context); POL-29 boot.rs-dispatch sweep across entire file: sole false claim was line 630; all other boot.rs mentions are either "unchanged" statements or context notes — zero further fixes needed. (LOW-1, adjudicated): `amends_dis: ["DI-012"]` added to frontmatter (mirrors ADR-023 sibling convention); D11 DI-012 row extended with bidirectional back-ref direction — adds `ADR-054` to DI-012's `amended_by` metadata in `domain-spec/invariants.md` at execution time. (OBS-1 correction note): v0.20 changelog rationale "unit_test is not a valid VP-INDEX tool token" was factually inaccurate — VP-INDEX has no defined legend and unit_test appears on draft rows VP-157/VP-158; the `integration_test` tool decision (FIX-BURST 20) STANDS on the VP-033/VP-036 network-boundary analogy and harness accuracy grounds, not on vocabulary invalidity. |
| 0.20 | 2026-07-21 | architect | FIX-BURST 20 (LOW-1): §D9 VP-159 tool corrected — `unit_test` → `integration_test` (canonical VP-INDEX tool vocabulary is {kani, proptest, integration_test, fuzz}; `unit_test` is not a valid VP-INDEX tool token; VP-159's behavioral state-transition sequences — cold/warm/stale cache — are deterministic MockHttpClient assertions, not combinatorial proptest input generation; analogous to VP-033/VP-036 which use `integration_test` for network-boundary invariants). (OBS-2): §D10(a) firing-condition precedence parenthesized — `A AND B OR C` → `A AND (B OR C)` making `A = auth_type ∈ {...}` bind to both conditions; D10(b)–(h) class sweep: zero other AND/OR precedence-ambiguity instances in D10. |
| 0.19 | 2026-07-21 | architect | FIX-BURST 19 (OBS-1): D11 census note row citation normalized — "ADR-023 §Rule A" → "ADR-023 Rule 2 (Rule A)" at all three occurrences in the census note cell (row label + 2 body mentions); ADR-023's Decision Rules are numbered Rule 1–5; the auth_type enumerated set with the at-point annotation lives in Rule 2 (§SensorAuth Trait Un-Sealing); "Rule A" is the project semantic label for the single-auth_type invariant (E-SPEC-012/ADR-026 §D3/VP-153); dual form "Rule 2 (Rule A)" removes the fresh-reader ambiguity. POL-29 (ADR-054 + ADR-053): zero remaining bare "ADR-023 §Rule A" hits in live text. |
| 0.18 | 2026-07-21 | architect | FIX-BURST 18 (LOW-1, adjudicated): D11 census note added for ADR-023 §Rule A — covered-by-annotation-only by design (restating site, not defining site); defining-site amendment is the ADR-026 §D3 D11 row; at-point annotation already present on ADR-023 §Rule A is the correct mechanism; formal D11 row is intentionally absent. Note added inline in D11 table to make the intent explicit for future fresh-context reviewers. |
| 0.17 | 2026-07-21 | architect | OBS-2: VP-153 §Feasibility Assessment D11 row amended — shape decision stated: token_exchange credential is a single secret string (credential_body_field → [[credential_refs]] name="secret_key"; ADR-054 D3/ADR-053 D2), structurally matching MockCredentialType::ApiKey("key") (static single-string credential); token_exchange reuses ApiKey structural shape; no new MockCredentialType variant required; credential space stays at 5 shapes; 6×5=30 arithmetic confirmed. is_coherent_pair and prop_filter amendment directions added. OBS-1 (ADR-054 scope): BC-2.06.003 D11 row has no §Worked examples reference (crowdstrike row only updates auth provider column, not env-var derivation examples); no fix needed. |
| 0.16 | 2026-07-21 | architect | HIGH-1: BC-2.06.003 §Per-Sensor `[[credential_refs]]` Declarations (Canonical) table added to D11 amendment manifest — crowdstrike row stale post-D5: auth provider column `crowdstrike-oauth2 WASM plugin — resolves both` → `DeclarativeHttpAuthProvider(Oauth2ClientCredentials) — resolves both`; credential refs (client_id, client_secret) and auth_type (oauth2_client_credentials) unchanged. MED-1: §D4 interface mis-anchor corrected — `SensorAuth` trait (prism-sensors) + `validate_and_construct_auth_providers` replaced with canonical `AuthProvider` trait (`crates/prism-spec-engine/src/auth_provider.rs`; defines `acquire_token`) + `step9a_populate_adapter_registry` (`crates/prism-bin/src/spec_driven_adapter.rs`) as construction site. POL-29: sole SensorAuth hit in declarative context was line 258 (the fixed site); zero other hits in ADR-053. |
| 0.15 | 2026-07-21 | architect | MED-1: BC-2.16.001 §Postconditions + §Auth Type Resolution added to D11 amendment manifest — two legacy-shorthand sites missed by custom_via_plugin tracer: §Postconditions "auth_type (oauth2/bearer/cookie/api_key)" slash shorthand and §Auth Type Resolution 4-value example ("e.g., oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key") both missing custom_via_plugin and token_exchange; both require modernization to canonical 6-value set in same spec-evolution story as E-SPEC-005/E-SPEC-012/VP-153/DI-012/BC-2.01.016/017 rows. BC-2.16.001 added to related_bcs frontmatter. Second census (legacy shorthand tracers: slash-shorthand, 4-value partial lists, "must be one of:" phrases): BC-2.16.001 is the sole new uncovered carrier; ADR-026:27 is per-sensor assignment note (not a closed-set enumeration); all other hits are non-normative or already in D11. Corrected carrier claim: v0.14 "sole uncovered carrier" applied only to the custom_via_plugin tracer class; three additional tracer classes (slash-shorthand, 4-value partial example, legacy "must be one of:") required a second pass and yielded BC-2.16.001 as one further uncovered normative carrier. |
| 0.14 | 2026-07-21 | architect | HIGH-1: domain-spec/invariants.md DI-012 pipe-delimited enumeration added to D11 amendment manifest — DI-012 "Single auth_type per spec" rule text (`SensorSpec.auth_type` accepts exactly one value (`oauth2_client_credentials` \| `bearer_static` \| `cookie_roundtrip` \| `api_key` \| `custom_via_plugin`)) goes stale when token_exchange lands; D11 row directs DI-012 update in same spec-evolution story as error-taxonomy.md E-SPEC-012, VP-153, and BC amendment rows. Deeper sweep (POL-25/29): DI-012 is the sole uncovered normative carrier; BC-2.16.009/BC-2.01.016/017 already in D11; BC-2.01.013 and edge-cases.md are single-value rule references (not closed-set enumerations); stories in .factory/stories/ are shipped implementation artifacts (not normative spec artifacts requiring D11 rows); ARCH-INDEX entries are historical changelog rows. |
| 0.13 | 2026-07-21 | architect | MED-1: D11 VP-153 harness row corrected — phantom `proptest::sample::select()` and `prop_compose!` constructs replaced with actual VP-153 constructs: `Just(AuthType::TokenExchange)` added to `prop_oneof!` in `arb_auth_type()`; inline E-SPEC-012 comment update cited as inside Rule A `proptest!` test body (not `prop_compose!`); `prop_filter` exclusion list amendment added. POL-29 class sweep: both phantom citations were exclusively in the fixed row; `prop_oneof!` and `proptest!` have zero hits in both primary ADRs (no other phantom construct citations). |
| 0.12 | 2026-07-21 | architect | HIGH-1: error-taxonomy.md E-SPEC-012 message template (Rule A) added to D11 — "Valid values: …" enumeration must include token_exchange; atomicity constraint added to VP-153 §Property Statement Rule A manifest row: error-taxonomy source + VP-153 copies must be amended in same commit (POL-24 byte-verbatim). LOW-2: volatile line-number pins stripped from VP-153 D11 rows; behavioral anchors (§Property Statement Rule A, §Feasibility Assessment, arb_auth_type(), inline E-SPEC-012 comment) preserved. OBS-1: E-SPEC-005 doc-hygiene row added to D11 directing modernization of stale "oauth2, bearer, cookie, api_key" enumeration to canonical 6-value set in same spec-evolution story. |
| 0.11 | 2026-07-21 | architect | HIGH-1: VP-153 (vp-153-sensorauth-runtime-cross-composition-prevention) added to D11 amendment manifest with 3 stale-site classes when token_exchange lands: (1) §Property Statement Rule A enumerated set + E-SPEC-012 expected message string (lines 47-51) — add token_exchange to 5-value set and to "Valid values: …" message; (2) §Feasibility Assessment "25 pairs" → 6×5=30 pairs (line 259); (3) §Proof Harness Skeleton arb_auth_type() (lines 113-121) + inline E-SPEC-012 message comment (line 169) — add TokenExchange arm. POL-29 VP-layer sweep: only VP-153 has live auth_type enumeration / E-SPEC-012 message hits. LOW-1: E-SPEC-028 template (h) added to D10 — fires when token_exchange-only fields (credential_body_field, token_response_path, expiry_field, expiry_mode) are present in [auth_acquisition] block for non-token_exchange auth_type; prevents silent-ignore class (SOUL.md #4). MED-1: D7 sequencing note updated — engine story authors E-SPEC-027 + coherence matrix for 5 existing auth_type variants only; ADR-054 story adds AuthType::TokenExchange + coherence-matrix row + allowed_set entry atomically (prevents forward-reference where token_exchange coherence row predates its enum variant). |
| 0.10 | 2026-07-21 | architect | LOW-2: §Status refreshed to current-version highlights (6th variant, E-SPEC-028(f)/(b), D5 retirement, 5→6-value downstream BCs, story sequencing dependency). OBS-2: explicit sequencing coordination note added to §D7 — ADR-054 stories land AFTER ADR-053 standalone engine story (Rule 9/E-SPEC-027 before Rule 10/E-SPEC-028). |
| 0.9 | 2026-07-21 | architect | HIGH-1: E-SPEC-028(h) → E-SPEC-028(f) in §D2 (credential-ref rule for oauth2_client_credentials; D10 enumerates only (a)–(g); (f) is "oauth2_client_credentials missing required credential_refs client_id and client_secret"). MED-1: BC-2.01.017 added to `related_bcs` frontmatter (3 D11 amendment rows target BC-2.01.017; was missing). LOW-1: D11 ADR-053 D2 and ADR-053 Rationale rows marked COMPLETED (ADR-053 v0.7); stale §Why custom_via_plugin anchor updated to current heading. MED-2: D11 amendment-instruction row for ADR-028 §D13 updated — removed "when [auth_acquisition] present" conditional framing; substituted D10(b) unconditional spec-load-rejection framing (E-SPEC-028(b)). |
| 0.8 | 2026-07-21 | architect | HIGH-1: D11 manifest extended with 5 downstream BC amendment rows for "5-value → 6-value" auth_type-set count corrections triggered by D1 (token_exchange as 6th variant): BC-2.01.016 §Related BCs, BC-2.01.017 §Preconditions, BC-2.01.017 §P3 Auth Type Dispatch, BC-2.01.017 §Related BCs, BC-2.16.009 §Validation Rules Schema Validation auth_type rule — all confirmed live normative sites by POL-29 grep sweep. |
| 0.7 | 2026-07-21 | architect | OBS-1: D11 `.config/nextest.toml` retirement row extended to cover the profile-documentation comment enumerating `crowdstrike_oauth2_plugin_tests` (behavioral anchor: the profile-documentation comment enumerating `crowdstrike_oauth2_plugin_tests`) — prior row covered only the four filter-group expression occurrences. |
| 0.6 | 2026-07-21 | architect | HIGH-1: ADR-028 added to `amends:` frontmatter (bidirectional symmetry with ADR-028 `amended_by` back-ref). MED-1: D2/D7/D10 internal contradiction resolved — D2 auth_plugin prohibition strengthened to unconditional for `{oauth2_client_credentials, token_exchange}` (consistent with D10(b)); D7 dispatch table row 2 annotated as validation-unreachable (D10(b) rejects at spec-load before step 9A executes); D10 cross-reference note added. OBS-2: ADR-023 §Status + §Rule 4 stale "no in-repo .prx plugin required" claims swept: acknowledge `crowdstrike-oauth2.prx` existence and ADR-054 D5 retirement. |
| 0.5 | 2026-07-21 | architect | HIGH-2 (FIX-BURST): ADR-028 §D13 amendment row added to D11 manifest — `oauth2_client_credentials` consistency-table row must be updated from `PluginAuthProvider (WASM)` to `DeclarativeHttpAuthProvider (native)` when `[auth_acquisition]` present (per D2/D5); bidirectional: ADR-028 frontmatter carries `amended_by` back-ref for ADR-054. OBS-1 (FIX-BURST): ARCH-INDEX AD-001 crate-count note amended — clarifies 25 after ADR-054 alone; 24 once ADR-037 (prism-customer-config) also lands; whichever ADR lands second must update to the combined value. |
| 0.4 | 2026-07-21 | architect | HIGH-1: BC-2.16.014 anchoring corrected (prior drafts had mis-anchored the planned declarative-auth BC to non-existent SS-23; retargeted to next-free BC-2.16.014 in SS-16/prism-spec-engine); swept all occurrences in frontmatter `related_bcs_planned`, D8 section header + body, D9, D11 manifest row, Consequences section, and Changelog v0.3 entry. MED-1: `tests/fixtures/README.md` added to D11 retirement manifest (H2 section heading `crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx` becomes stale post-retirement; `wasi_snapshot_preview1.wasm` preserved for remaining `custom_via_plugin` plugins). LOW-1: `test_F_LP7_MED_001_host_dispatch_acquire_token_component_model_path_emits_audit_event` added to D11 (`#[ignore]`/`todo!()` integration test requires pre-built `crowdstrike-oauth2.prx`; post-retirement the binary no longer exists; behavioral coverage superseded by `DeclarativeHttpAuthProvider` unit tests). OBS-1: doc-hygiene-sweep row added to D11 covering five preserved-infrastructure files (`spec_parser.rs`, `plugin_auth_provider.rs`, `error.rs`, `plugin/discovery.rs`, `plugin/host_functions.rs`) with stale `crowdstrike-oauth2` doc examples. |
| 0.3 | 2026-07-21 | architect | HIGH-1: add `crates/prism-spec-engine/Cargo.toml` to D11 retirement manifest (`[[test]]` crowdstrike_oauth2_plugin_tests block + async-trait dev-dep). HIGH-2: add second ci.yml self-guard row (grep `F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike`). HIGH-3: add `bc_2_16_013_crowdstrike_multiregion.rs` to D11 — D-747-LOCKED auth_plugin assertion must be retargeted to native declarative auth. HIGH-4: add `scripts/demo-setup.sh` and `scripts/demo-run.sh` to D11 (crowdstrike-oauth2.prx copy + manifest steps break post-retirement). HIGH-5: D1/D2 section headers corrected from "(supersedes ADR-026/023…)" to "(amends …)"; Source §ADR-023 reference corrected from "partially superseded" to "partially amended". MED-1: add `scripts/t13-preflight-audit.py` to D11 (`[A20]` plugin_status probe for crowdstrike-oauth2 must be retargeted). MED-2: add `docs/DEMO-RUNBOOK.md` to D11 (.prx + manifest.toml documentation stale). LOW-1: fix D2 prose + D4 step-2 form-body field order to match plugin actual order `client_id={}&client_secret={}&grant_type=client_credentials`. LOW-2: split BC-2.16.014 out of `related_bcs` into `related_bcs_planned` to prevent POL-21 phantom-anchor false-positive. OBS-1: assign ADR-054 auth-coherence check as BC-2.16.009 Rule 10 (Rule 9 reserved by ADR-053 D2 for header_scheme); update D7/D10/D11 accordingly. OBS-2: replace volatile `~line NNN` anchors in D11 with behavioral anchors (recipe name / step title / grep string / function name / filter expression) per TD-VSDD-091. |
| 0.2 | 2026-07-20 | architect | CRIT-1: retarget D5/D7/D11 from `validate_and_construct_auth_providers` to `step9a_populate_adapter_registry` (the real auth_type dispatch site in `spec_driven_adapter.rs`). HIGH-1: replace absolute `token_url` with relative `token_path` derived per-org from `resolved_spec.spec.base_url` at step 9A construction time. HIGH-2: complete D11 retirement manifest with Justfile, ci.yml (build steps + CI self-guard + committed-.prx validation), nextest.toml binary filters, crowdstrike_oauth2_plugin_tests.rs, plugin_boot_tests.rs, helpers/mod.rs staging functions, fixtures, and ARCH-INDEX/CLAUDE.md crate-count staleness. MED-1: remove phantom api_key → ApiKeyAuthProvider (api_key falls into other=> branch at step 9A → E-SPEC-012 skip). MED-2: correct bearer_static provider name to BearerStaticCredentialAuthProvider. MED-3: supersedes→amends in frontmatter (partial sub-section change is an amendment per CLAUDE.md). LOW-1: remove "replicates exactly" language. LOW-2: fix VP-159 TTL formula to match plugin's saturating_sub arithmetic; remove dead .max(1). |
| 0.1 | 2026-07-20 | architect | Initial draft per human decision D-1895 |
