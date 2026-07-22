---
document_type: adr
adr_id: "ADR-054"
title: "Native Declarative HTTP Auth Acquisition — TokenExchange and OAuth2ClientCredentials via DeclarativeHttpAuthProvider; Retire crowdstrike-oauth2.prx"
status: proposed
date: "2026-07-20"
modified: "2026-07-21"
version: "0.25"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes: null
superseded_by: null
amends:
  - "ADR-023 (partial — §Rule 4 walk-back: standard HTTP token-acquisition flows do not require WASM plugins; custom_via_plugin escape hatch preserved for genuinely non-standard auth)"
  - "ADR-026 (partial — §D3: AuthType closed enum gains token_exchange variant; affects E-SPEC-012 enum validation and step9a_populate_adapter_registry dispatch)"
  - "ADR-028 (partial — §D13 oauth2_client_credentials: PluginAuthProvider (WASM) path spec-load-rejected per D10(b) E-SPEC-028(b) unconditional; DeclarativeHttpAuthProvider (native) is the sole live path; §D2 + §D13 Armis blockquotes updated from custom_via_plugin to token_exchange; crowdstrike-oauth2.prx plugin retired per D5)"
related_adrs: [ADR-023, ADR-026, ADR-028, ADR-031, ADR-032, ADR-050, ADR-053]
related_bcs: [BC-2.01.016, BC-2.01.017, BC-2.06.003, BC-2.16.001, BC-2.16.009]
related_bcs_planned: [BC-2.16.014]
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

Proposed 2026-07-20. Current version per §Changelog. Amends ADR-023 §Rule 4 (standard HTTP token-acquisition flows do not require WASM plugins; `custom_via_plugin` preserved for genuinely non-standard auth) and ADR-026 §D3 (partial — `AuthType` gains `token_exchange` as 6th variant). Companion to ADR-053. Awaiting human approval gate before implementation begins.

Current contract highlights: D1 adds `token_exchange` as the 6th AuthType variant; E-SPEC-028(f) validates `client_id`/`client_secret` credential refs for `oauth2_client_credentials`; E-SPEC-028(b) unconditionally rejects `auth_plugin` for declarative auth_types per D10(b) — no "when `[auth_acquisition]` present" conditional. D2 makes `oauth2_client_credentials` native via `DeclarativeHttpAuthProvider`. D5 retires `crowdstrike-oauth2.prx`. D11 amendment manifest includes 5 downstream "5→6-value" BC count corrections (BC-2.01.016 §Related BCs, BC-2.01.017 §Preconditions/§P3/§Related BCs, BC-2.16.009 §Validation Rules). ADR-054 implementation stories land AFTER ADR-053's standalone Wave-A engine story (Rule 9/E-SPEC-027 must be registered before Rule 10/E-SPEC-028 — see §D7). See §Changelog for full revision history.

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
`crowdstrike-oauth2/src/lib.rs` `acquire_token()`). Response: `$.access_token` (string, required);
`$.expires_in` (u64 seconds, default 1799 when absent or zero); `ttl_buffer_secs` (default 30)
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

**CachedAuthToken** holds:
- `token: String` — the acquired access token (opaque bytes; never logged per AD-017)
- `expires_at: u64` — Unix timestamp after which the token must be re-acquired

**`acquire_token()` (force-refresh, bypasses cache):**
1. Resolve credential(s) from the credential store (lazy per AD-017 — no credential access at boot)
2. Build form body (RFC-3986 percent-encoded):
   - `oauth2_client_credentials`: `client_id={}&client_secret={}&grant_type=client_credentials`
   - `token_exchange`: `{credential_body_field}={resolved_value}`
3. POST to the per-org derived token URL (`format!("{}{}", resolved_spec.spec.base_url, config.token_path)`,
   computed at step 9A construction time and stored as `self.token_url` in the provider)
   with `Content-Type: application/x-www-form-urlencoded`
   (using the workspace reqwest client per ADR-050: `rustls-tls`, 30s timeout)
4. Parse response:
   - `oauth2_client_credentials`: extract `$.access_token`; compute TTL from `$.expires_in` (default 1799 if absent/zero) minus `ttl_buffer_secs`
   - `token_exchange`: extract at `token_response_path`; parse expiry at `expiry_field` per `expiry_mode`:
     - `"relative_seconds"`: u64 seconds, default 1799, minus `ttl_buffer_secs`
     - `"absolute_utc_string"`: RFC-3339 parse → Unix timestamp, minus `ttl_buffer_secs`
5. Store `CachedAuthToken` in `cached_token` ArcSwap
6. Return `Ok(token_string)`

**`get_token()` (cache-aware):**
1. Load `cached_token` snapshot
2. If `Some(cached)` and `unix_now() < cached.expires_at` and `!cached.token.is_empty()` → return cached token (zero network calls)
3. Otherwise → call `acquire_token()` (refreshes cache atomically via ArcSwap)

**Error handling:** All acquisition errors propagate as `SpecEngineError::AuthRefreshFailed`
(reusing the existing error variant per error-taxonomy discipline — no new error variant needed
for the acquisition-level failure; E-SPEC-028 covers spec-load validation failures only).

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

> **[PLANNED — Wave-A spec evolution]** `BC-2.16.014` does not yet exist. It will be authored
> by the product-owner during the Wave-A implementation story that delivers
> `DeclarativeHttpAuthProvider`. The postconditions P1–P8 below are the **authoring source** —
> the product-owner uses this section as the behavioral specification when writing the BC file.
> Until that story ships, `BC-2.16.014` is a forward reference only; no
> `.factory/specs/behavioral-contracts/BC-2.16.014-*.md` file exists and no BC-INDEX row exists.

A new BC `BC-2.16.014` will be authored covering the behavioral contract for
`DeclarativeHttpAuthProvider`:

**Preconditions:**
- Sensor spec has `auth_type ∈ {oauth2_client_credentials (declarative), token_exchange}`
- `[auth_acquisition]` block is present and validated by E-SPEC-028 at spec-load time
- `DeclarativeHttpAuthProvider` is constructed per (org, sensor) during boot step 9A
  (`step9a_populate_adapter_registry` in `spec_driven_adapter.rs`)

**Postconditions (summary — BC-2.16.014 will be authoritative once authored):**
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

> **[PLANNED — Wave-A spec evolution]** `VP-159` does not yet exist. It will be registered in
> `VP-INDEX.md` by the architect during the Wave-A implementation story that delivers
> `DeclarativeHttpAuthProvider`, after `BC-2.16.014` is authored and its postconditions are
> confirmed. The properties listed below are the **authoring source** for VP-159. Until that
> story ships, VP-159 is a forward reference only; no `vp-159-*.md` file exists and no
> VP-INDEX row exists.

A new verification property `VP-159` will cover the network-call invariants of `DeclarativeHttpAuthProvider`:

- **Module:** `prism-spec-engine`
- **Tool:** `integration_test` (MockHttpClient for network isolation — behavioral state-transition sequences, not combinatorial input generation; analogous to VP-033/VP-036)
- **Phase:** P1
- **BC:** BC-2.16.014 (primary; forward reference — see D8)
- **Properties:**
  - `acquire_token()` makes exactly one HTTP POST (no cached-token bypass)
  - `get_token()` on cold cache → exactly one HTTP POST; on warm cache → zero HTTP POSTs
  - `get_token()` on stale cache (expired TTL) → exactly one HTTP POST (re-acquisition)
  - `get_token()` on cache-hit with empty token string → exactly one HTTP POST (same as cold cache)
  - TTL arithmetic for `absolute_utc_string` expiry mode: `expires_at = parse_rfc3339(expiry_str).as_unix_secs().saturating_sub(ttl_buffer_secs)`
  - TTL arithmetic for `relative_seconds` expiry mode: `expires_at = now + expires_in.saturating_sub(ttl_buffer_secs)` where `expires_in` is defaulted to 1799 when absent or zero (matches the plugin's `saturating_sub(30)` arithmetic; `.max(1)` is omitted as dead code when the absent/zero default is already 1799)
  - Credential values are not stored in `CachedAuthToken` (AD-017 assertion)

VP-159 will be created as DRAFT; promoted to ACTIVE when the implementation story (D5 retirement story)
ships and the unit tests are green.

### D10 — E-SPEC-028: Declarative Auth Acquisition Validation Errors

`E-SPEC-028` (next free after E-SPEC-027 reserved by ADR-053) is registered in `error-taxonomy.md`
covering validation errors for `[auth_acquisition]` blocks. Validation runs in the same multi-error
pass as other spec-file validation rules (BC-2.16.009 **Rule 10** — after ADR-053 D2's Rule 9 for
`header_scheme` validation). Spec rejected on any E-SPEC-028; boot fails exit code 2.

**Message templates:**

**(a) Required block absent:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' requires an [auth_acquisition] block with token_path. Add an [auth_acquisition] block."`
Fires when: `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND (`[auth_acquisition]` absent OR `token_path` absent).

**(b) Conflicting auth_plugin:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."`
Fires when: `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` is present.

> **D2/D7 consistency note:** D2 states the `auth_plugin` prohibition **unconditionally** for these
> auth_types (not only when `[auth_acquisition]` is declared). D7's dispatch table row for
> `Oauth2ClientCredentials AND auth_plugin.is_some()` is therefore validation-unreachable from any
> valid spec — this arm exists as defense-in-depth only. D2, D7, and D10(b) are now consistent.

**(c) Unknown expiry_mode:**
`"sensor '{sensor_id}': [auth_acquisition].expiry_mode = '{value}' is not valid. Accepted values: absolute_utc_string, relative_seconds."`
Fires for `token_exchange` when `expiry_mode` is present but not in the two-value closed set.

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
`"sensor '{sensor_id}': [auth_acquisition].{field_name} is only valid when auth_type = 'token_exchange'. Remove '{field_name}' or change auth_type to 'token_exchange'."`
Fires when any of `credential_body_field`, `token_response_path`, `expiry_field`, or `expiry_mode` is present in an `[auth_acquisition]` block whose `auth_type` is not `token_exchange`. Prevents token_exchange-only fields from being silently ignored when misconfigured on an `oauth2_client_credentials` block (SOUL.md #4 violation class).

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
| New `crates/prism-spec-engine/src/auth/declarative.rs` | Implement `DeclarativeHttpAuthProvider` + `AuthAcquisitionConfig` + `ExpiryMode` | D4 |
| `crates/prism-bin/src/spec_driven_adapter.rs` `step9a_populate_adapter_registry` `Oauth2ClientCredentials` arm | Rewrite from per-org `PluginAuthProvider` construction to `DeclarativeHttpAuthProvider` construction with `token_url = base_url + token_path` | D4, D7 |
| `crates/prism-bin/src/spec_driven_adapter.rs` `step9a_populate_adapter_registry` | Add new `TokenExchange` arm: construct `DeclarativeHttpAuthProvider(TokenExchange)` with `token_url = base_url + token_path` | D1, D7 |
| `BC-2.16.009` Rule set | Add `[auth_acquisition]` coherence validation as **Rule 10** (after ADR-053 D2's Rule 9 for `header_scheme`); E-SPEC-028 error suite per D10 | D10 |
| `error-taxonomy.md` | Register E-SPEC-028 with all message templates (D10) | D10 |
| New `BC-2.16.014` `[PLANNED]` | Author Declarative Auth Acquisition Token Lifecycle contract (D8) during Wave-A implementation story; postconditions P1–P8 specified in §D8 above are the authoring source | D8 |
| `VP-INDEX.md` `[PLANNED]` | Register VP-159 (D9) during Wave-A implementation story, after BC-2.16.014 is authored | D9 |
| ADR-053 D2 | **[COMPLETED — ADR-053 v0.7]** Armis TOML block rewritten from `custom_via_plugin` + `auth_plugin` to `token_exchange` + `[auth_acquisition]` block; coherence matrix updated with `token_exchange → bearer, raw` row; rationale section rewritten. | D1, D3 |
| ADR-053 §Why native `token_exchange` for Armis (not `custom_via_plugin` + plugin)? | **[COMPLETED — ADR-053 v0.7/v0.12]** Rationale updated to reflect native declarative provider decision; heading renamed from `§Why custom_via_plugin` (retired per POL-21) to current heading `§Why native token_exchange for Armis (not custom_via_plugin + plugin)?`. | D2 |
| ADR-053 D5 manifest | Update BC-2.01.008 amendment description from `custom_via_plugin` + plugin to `token_exchange` + native provider | D1 |
| ADR-026 §D3 (partial) | Note that `token_exchange` variant added to AuthType enum per ADR-054 D1; cross-reference frontmatter `amended_by` | D1 |
| ADR-023 Rule 2 (Rule A) (census note — annotation-only, no amendment row) | **Covered-by-annotation-only by design — no D11 amendment row required.** ADR-023 Rule 2 (Rule A) restates the auth_type set but is not the defining site; the defining-site amendment is the ADR-026 §D3 row above. ADR-023 Rule 2 (Rule A) already carries an at-point annotation noting the 6th variant; a formal D11 amendment row would duplicate the change. This asymmetry with ADR-026 §D3 is intentional. | D1 (annotation only) |
| `domain-spec/invariants.md` DI-012 "Single auth_type per spec" enumeration | Add `token_exchange` to the pipe-delimited enumeration in DI-012 rule 1: current text — `SensorSpec.auth_type` accepts exactly one value (`oauth2_client_credentials` \| `bearer_static` \| `cookie_roundtrip` \| `api_key` \| `custom_via_plugin`) — update to include `token_exchange` as 6th variant. DI-012 is the canonical domain root that VP-153 traces to (VP-153 `source_invariant: DI-012`) and BC-2.01.016 operationalizes. Update in same spec-evolution story as error-taxonomy.md E-SPEC-012, VP-153, and BC amendment rows (all must be consistent at every commit boundary). Behavioral anchor: DI-012 rule 1 "Single auth_type per spec" pipe-delimited value list. **Bidirectional back-ref at execution time:** In the same amendment commit, add `ADR-054` to DI-012's `amended_by` (or equivalent) back-ref metadata in `domain-spec/invariants.md` — symmetric with ADR-023's `amends_dis: ["DI-012"]` convention (ADR-023 appears as a back-ref in DI-012; ADR-054 must too). | D1 |
| `BC-2.16.001` §Postconditions + §Auth Type Resolution (doc-hygiene) | Two legacy-shorthand sites require modernization to the canonical 6-value set when D1 lands: (1) §Postconditions `SensorSpec` field description — update `auth_type` parenthetical "(oauth2/bearer/cookie/api_key)" slash shorthand to canonical 6-value set; (2) §Auth Type Resolution first bullet — update 4-value example "e.g., `oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`" to include `custom_via_plugin` and `token_exchange`. Behavioral anchors: §Postconditions `auth_type` parenthetical; §Auth Type Resolution example list. Update in same spec-evolution story as E-SPEC-005, E-SPEC-012, VP-153, DI-012, and BC-2.01.016/017 rows — all auth_type enumeration sites must be consistent at every commit boundary. | D1 (doc-hygiene) |
| BC-2.06.003 §Per-Sensor `[[credential_refs]]` Declarations (Canonical) table | CrowdStrike row stale post-D5 plugin retirement: update auth provider column from `crowdstrike-oauth2 WASM plugin — resolves both` to `DeclarativeHttpAuthProvider(Oauth2ClientCredentials) — resolves both`. Credential ref names (`client_id`, `client_secret`) and `auth_type` (`oauth2_client_credentials`) are UNCHANGED — only the auth provider implementation changes. Behavioral anchor: §Per-Sensor `[[credential_refs]]` Declarations (Canonical) table `crowdstrike` row "Auth provider" column. In-scope for the CrowdStrike plugin retirement / Armis token-exchange story (same as other D5 retirement artifacts). Armis and Cyberint rows in this same table are stale via ADR-053 D2/D3 and are covered by the ADR-053 D5 manifest. | D5 |
| BC-2.01.016 §Related BCs | Update "one entry in the 5-value canonical auth_type set" → 6-value canonical auth_type set (`token_exchange` is the 6th variant per D1) | D1 |
| BC-2.01.017 §Preconditions | Update "The 5-value canonical auth_type set (BC-2.01.016 §Postconditions)" → 6-value canonical auth_type set | D1 |
| BC-2.01.017 §P3 (Auth Type Dispatch) | Update "the 5-value canonical auth_type set per BC-2.01.016 §Postconditions" → 6-value canonical auth_type set | D1 |
| BC-2.01.017 §Related BCs | Update "the 5-value canonical auth_type set (including `"cookie_roundtrip"`)" → 6-value canonical auth_type set | D1 |
| BC-2.16.009 §Validation Rules (Schema Validation, `auth_type` rule) | Add `token_exchange` to the enumerated allowed-values list; update "(5-value canonical set)" → "(6-value canonical set)" in the parenthetical | D1 |
| `VP-153` §Property Statement Rule A enumerated set + E-SPEC-012 expected message string | Add `token_exchange` to the Rule A 5-value enumerated set `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}`; update E-SPEC-012 expected message "Valid values: oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin" to include `token_exchange` as the 6th variant. Both sites in `vp-153-sensorauth-runtime-cross-composition-prevention.md` §Property Statement Rule A section. **Atomicity with error-taxonomy.md E-SPEC-012:** VP-153 and error-taxonomy.md E-SPEC-012 MUST be updated in the same commit — POL-24 byte-verbatim invariant breaks if source and copies diverge across a commit boundary. | D1 |
| `VP-153` §Feasibility Assessment | Update "5 auth_type variants × 5 credential structural shapes = 25 pairs; all enumerable" → "6 auth_type variants × 5 credential structural shapes = 30 pairs; all enumerable". **Shape decision:** `token_exchange`'s credential is a single secret string consumed via `credential_body_field` → `[[credential_refs]] name="secret_key"` (ADR-054 D3 / ADR-053 D2). The as-built Rule C harness uses auth_type identifier strings as shape labels via `ShapedProbe.reported_shape` (no typed credential-shape enum); `token_exchange` introduces the 6th auth_type identifier; Rule C mismatch coverage grows from 20 ordered pairs (5×4) to 30 ordered pairs (6×5) once strategy bounds are updated. Credential string-shape space stays at 5+1=6 string identifiers. **Harness amendments (as-built constructs):** (1) `VALID_AUTH_TYPES: &[&str]` constant in both `crates/prism-spec-engine/tests/vp153_sensorauth_cross_composition.rs` and `crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs` — add `"token_exchange"` as 6th entry; (2) `arb_valid_auth_type()` `prop_oneof!` in `vp153_sensorauth_cross_composition.rs` — add `Just("token_exchange")` as 6th arm; (3) `arb_invalid_auth_type()` filter predicate already references `VALID_AUTH_TYPES` (`!VALID_AUTH_TYPES.contains(&s.as_str())`) — auto-expands when (1) lands, no separate edit needed; (4) Rule B `allowed_count` = `if valid_type == "oauth2_client_credentials" { 2 } else { 1 }` — `token_exchange` gets `allowed_count = 1` via `else` branch, no separate change; (5) `arb_matching_auth_type()` `prop_oneof!` in `vp153_rule_c_shaped_probe.rs` — add `Just("token_exchange")` as 6th arm; (6) `arb_mismatched_auth_type_pair()` range bounds `(0usize..5, 0usize..4)` → `(0usize..6, 0usize..5)` — covers 6×5=30 ordered mismatched pairs. | D1 |
| `VP-153` §Proof Method member-count sentence | Update "The valid `auth_type` enumerated set has 5 members" → "6 members". Behavioral anchor: §Proof Method feasibility prose block, sentence beginning "The valid `auth_type` enumerated set has". In-scope for the same spec-evolution story as the §Feasibility Assessment "25 pairs"→"30 pairs" count correction — both count-bearing sites must be consistent at every commit boundary. | D1 |
| `VP-153` §Proof Harness Skeleton `arb_valid_auth_type()` + `VALID_AUTH_TYPES` constants | In `vp153_sensorauth_cross_composition.rs`: (1) add `"token_exchange"` as 6th entry in `VALID_AUTH_TYPES: &[&str]` constant; (2) add `Just("token_exchange")` as 6th arm to the `prop_oneof!` in `arb_valid_auth_type()`. In `vp153_rule_c_shaped_probe.rs`: (3) add `"token_exchange"` as 6th entry in `VALID_AUTH_TYPES`; (4) add `Just("token_exchange")` as 6th arm to `arb_matching_auth_type()` `prop_oneof!`; (5) update `arb_mismatched_auth_type_pair()` range bounds `(0usize..5, 0usize..4)` → `(0usize..6, 0usize..5)`. **No inline "Valid values:" comment update needed** — `prop_rule_a_invalid_auth_type_rejected_with_e_spec_012` asserts `err_str.contains("E-SPEC-012")` only; no "Valid values:" literal appears in the test body. **No separate `prop_filter` edit needed** — `arb_invalid_auth_type()` references `VALID_AUTH_TYPES` in its filter predicate; adding `"token_exchange"` to the constant auto-expands the exclusion. Behavioral anchors: `VALID_AUTH_TYPES` constant in both test files; `arb_valid_auth_type()` `prop_oneof!` arm list; `arb_matching_auth_type()` `prop_oneof!` arm list; `arb_mismatched_auth_type_pair()` range tuple. | D1 |
| `error-taxonomy.md` E-SPEC-012 message template (Rule A — auth_type closed-enum validation) | Add `token_exchange` to the "Valid values: …" enumeration in the E-SPEC-012 `message_template` column: current text ends "…oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin"; add `token_exchange` as 6th value. **POL-24 atomicity:** `error-taxonomy.md` E-SPEC-012 (source) + VP-153 §Property Statement Rule A prose (copy) MUST be updated in the SAME commit — byte-verbatim invariant breaks if source and copy diverge across a commit boundary. Note: VP-153 §Proof Harness Skeleton `arb_valid_auth_type()` and `VALID_AUTH_TYPES` are updated separately per their own D11 row; no "Valid values:" literal appears in the test code, so no additional POL-24 copy site exists in the harness. | D1 |
| `error-taxonomy.md` E-SPEC-005 notes column (doc-hygiene) | Modernize stale pre-PREREQ-E enumeration in E-SPEC-005 notes column — "Auth type must be one of: oauth2, bearer, cookie, api_key." → canonical 6-value set (oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange post-D1). Behavioral anchor: E-SPEC-005 `notes` column legacy text. In-scope for the same spec-evolution story that amends E-SPEC-012 and VP-153 — prevents the error-taxonomy table from carrying pre-PREREQ-E legacy names after the Wave-A enum expansion. | D1 (doc-hygiene) |
| ADR-028 §D13 consistency table | Update `oauth2_client_credentials` row: mark `PluginAuthProvider` (WASM) path as **spec-load-rejected** per D10(b) — E-SPEC-028(b) unconditional rejection for `auth_type ∈ {oauth2_client_credentials, token_exchange}` + `auth_plugin` present; `DeclarativeHttpAuthProvider` (native) is the sole live path; drop the conditional "when `[auth_acquisition]` present" framing (superseded by D10(b)'s unconditional rule). Mark `crowdstrike-oauth2.prx` as retired per D5. Update §D2 and §D13 Armis blockquotes from `custom_via_plugin` + `armis-token-exchange.prx` → `token_exchange` + native `DeclarativeHttpAuthProvider`. Update ADR-028 frontmatter `amended_by` framing to reflect D10(b) unconditional rejection. ADR-028 `amended_by` back-ref already present (ADR-028 v1.18); ADR-028 in ADR-054 `related_adrs` (already present). | D2, D5, D10 |
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
| `crates/` code doc-comment domain — census pass 3 (FIX-BURST 26 domain extension) | **Census note — new carrier domain.** Tracers applied across entire `crates/` tree (excluding already-manifested `spec_parser.rs` and `vp153_*.rs` sites, and single-value test assertion strings): `'custom_via_plugin'` (canonical tracer), `'bearer_static.*cookie_roundtrip'` adjacency, `'oauth2_client_credentials'` enumeration contexts, `'closed enumeration'`. Per-hit dispositions: (1) `crates/prism-sensors/src/auth/mod.rs` `SensorAuth::auth_type_name()` fn doc-comment — **new-D11-row** (direction b, see row below); (2) `crates/prism-spec-engine/src/error.rs` `AuthTypeCrossComposition` variant doc-comment lines 182-183 — **new-D11-row** (see row below); (3) `crates/prism-spec-engine/tests/bc_2_01_016_test.rs` line 48 test comment listing 5-value set — **already-manifested** (covered implicitly by `spec_parser.rs` D11 row and BC-2.01.016 story scope; test comments are non-normative); (4) all remaining `custom_via_plugin` / `oauth2_client_credentials` occurrences in test assertion strings — **single-value-non-carrier**; (5) `crates/prism-spec-engine/tests/bc_2_16_009_bundled_spec_validation.rs` line 80 — "canonical enumerated set" reference with no value listing — **single-value-non-carrier**. Zero uncovered normative enumeration carriers remain in `crates/`. | D1 (census note) |
| `crates/prism-sensors/src/auth/mod.rs` `SensorAuth::auth_type_name()` fn doc-comment (FIX-BURST 26) | **Direction (b) — SensorAuth-scope clarification (not token_exchange addition).** Reword the doc-comment to exclude `token_exchange` from `SensorAuth` scope. Updated doc (behavioral anchor: the "Must return one of the closed enumeration values defined in ADR-026 §D3" sentence, per TD-VSDD-091): "Must return one of the auth-type discriminator values from ADR-026 §D3 that are applicable to `SensorAuth` credentials: `\"oauth2_client_credentials\"`, `\"bearer_static\"`, `\"cookie_roundtrip\"`, `\"api_key\"`, `\"custom_via_plugin\"`. Note: `\"token_exchange\"` (ADR-054 D1) is NOT in `SensorAuth` scope — `token_exchange` auth acquisition is handled natively by `DeclarativeHttpAuthProvider` (prism-spec-engine, ADR-054 D2/D10b) and never produces a `SensorAuth` instance." Direction (b) rationale: a `SensorAuth` impl returning `"token_exchange"` would have no caller — the `DeclarativeHttpAuthProvider` path bypasses the credential store → `SensorAuth` resolution chain entirely. Adding `token_exchange` to the doc (direction a) would mislead external `SensorAuth` implementors into providing a meaningless `"token_exchange"` value. Update in same spec-evolution story as `spec_parser.rs` D1 extension. | D1 (doc-hygiene) |
| `crates/prism-spec-engine/src/error.rs` `AuthTypeCrossComposition` variant doc-comment (FIX-BURST 26) | Add `token_exchange` to the closed-enum brace expression in the `AuthTypeCrossComposition` variant doc-comment: current text at lines 182-183 reads `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` → update to `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin, token_exchange}`. Behavioral anchor: `AuthTypeCrossComposition` variant doc-comment brace expression at lines 182-183 of `error.rs` (TD-VSDD-091). This is distinct from the existing D5 doc-hygiene row for `error.rs` — that row covers `UnknownAuthPlugin` and `PluginAuthDispatchError::plugin_id` doc-comments (which cite `"crowdstrike-oauth2"` as a plugin example). This row covers the E-SPEC-012 validation set enumeration in `AuthTypeCrossComposition`. Update in same spec-evolution story as `spec_parser.rs` D1 extension and `error-taxonomy.md` E-SPEC-012 update (POL-24 atomicity: keep the runtime error code doc consistent with the `error-taxonomy.md` message template). | D1 (doc-hygiene) |

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
- `[PLANNED]` BC-2.16.014 will formalize the token-lifecycle contract that was previously only implicit in the
  WASM plugin source, making it testable and verifiable via VP-159 (both authored during Wave-A; see D8/D9)

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

### Status as of 2026-07-20

Proposed. Not in effect. Implementation gated on this ADR reaching Accepted status (human
approval gate). Until accepted, ADR-053 v0.7 (Armis TOML block rewritten to use `token_exchange`
+ `[auth_acquisition]`) captures the intent, but the CrowdStrike migration and plugin retirement
do not proceed.

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
