---
document_type: adr
adr_id: "ADR-054"
title: "Native Declarative HTTP Auth Acquisition — TokenExchange and OAuth2ClientCredentials via DeclarativeHttpAuthProvider; Retire crowdstrike-oauth2.prx"
status: proposed
date: "2026-07-20"
modified: "2026-07-21"
version: "0.3"
producer: architect
subsystems_affected: [SS-01, SS-06, SS-16, SS-17]
supersedes: null
superseded_by: null
amends:
  - "ADR-023 (partial — §Rule 4 walk-back: standard HTTP token-acquisition flows do not require WASM plugins; custom_via_plugin escape hatch preserved for genuinely non-standard auth)"
  - "ADR-026 (partial — §D3: AuthType closed enum gains token_exchange variant; affects E-SPEC-012 enum validation and step9a_populate_adapter_registry dispatch)"
related_adrs: [ADR-023, ADR-026, ADR-028, ADR-031, ADR-032, ADR-050, ADR-053]
related_bcs: [BC-2.01.016, BC-2.06.003, BC-2.16.009]
related_bcs_planned: [BC-2.23.001]
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

Proposed 2026-07-20, v0.1 (initial draft per D-1895); revised v0.2 (2026-07-20) — step 9A
retargeting (CRIT-1), per-org token_path schema (HIGH-1), complete retirement manifest (HIGH-2),
dispatch table corrections (MED-1/MED-2), supersedes→amends (MED-3). Revised v0.3 (2026-07-21)
— adversarial pass closures: D11 retirement manifest extended with Cargo.toml test block, second
ci.yml self-guard, bc_2_16_013_crowdstrike_multiregion.rs D-747-lock retarget, demo-setup/run.sh,
t13-preflight-audit.py, DEMO-RUNBOOK.md; D2/D4 form-body field order corrected to match plugin;
BC-2.16.009 Rule 10 assigned (Rule 9 reserved by ADR-053); related_bcs_planned split; volatile
~line anchors replaced with behavioral anchors. This ADR amends ADR-023 §Rule 4 and ADR-026 §D3
(partial sub-section changes; not supersessions). It is a companion to ADR-053 (Wave-A Sensor
Fidelity Remediation). Awaiting human approval gate before implementation begins.

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
field MUST NOT be present when `[auth_acquisition]` is declared; combining both is a D10 E-SPEC-028
validation error.

`oauth2_client_credentials` semantics in declarative mode: the engine performs an RFC 6749 §4.4
client credentials POST. Form body: `client_id={}&client_secret={}&grant_type=client_credentials`
(values URL-form-encoded per RFC 3986 §2.3; field order matches the plugin implementation in
`crowdstrike-oauth2/src/lib.rs` `acquire_token()`). Response: `$.access_token` (string, required);
`$.expires_in` (u64 seconds, default 1799 when absent or zero); `ttl_buffer_secs` (default 30)
subtracted from the computed `expires_at`. Credential refs MUST include one named `client_id`
and one named `client_secret` — validated by E-SPEC-028(h) at spec-load time.

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
It implements the `SensorAuth` trait (or the equivalent auth-provider interface used by
`validate_and_construct_auth_providers` in `boot.rs`).

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
| `Oauth2ClientCredentials` AND `auth_plugin.is_some()` (pre-migration / other sensors) | Build per-org `PluginAuthProvider` from global step-7.5b map — unchanged until plugin retired |
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

### D8 — BC-2.23.001: Declarative Auth Acquisition Token Lifecycle

> **[PLANNED — Wave-A spec evolution]** `BC-2.23.001` does not yet exist. It will be authored
> by the product-owner during the Wave-A implementation story that delivers
> `DeclarativeHttpAuthProvider`. The postconditions P1–P8 below are the **authoring source** —
> the product-owner uses this section as the behavioral specification when writing the BC file.
> Until that story ships, `BC-2.23.001` is a forward reference only; no
> `.factory/specs/behavioral-contracts/BC-2.23.001-*.md` file exists and no BC-INDEX row exists.

A new BC `BC-2.23.001` will be authored covering the behavioral contract for
`DeclarativeHttpAuthProvider`:

**Preconditions:**
- Sensor spec has `auth_type ∈ {oauth2_client_credentials (declarative), token_exchange}`
- `[auth_acquisition]` block is present and validated by E-SPEC-028 at spec-load time
- `DeclarativeHttpAuthProvider` is constructed per (org, sensor) during boot step 9A
  (`step9a_populate_adapter_registry` in `spec_driven_adapter.rs`)

**Postconditions (summary — BC-2.23.001 will be authoritative once authored):**
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
> `DeclarativeHttpAuthProvider`, after `BC-2.23.001` is authored and its postconditions are
> confirmed. The properties listed below are the **authoring source** for VP-159. Until that
> story ships, VP-159 is a forward reference only; no `vp-159-*.md` file exists and no
> VP-INDEX row exists.

A new verification property `VP-159` will cover the network-call invariants of `DeclarativeHttpAuthProvider`:

- **Module:** `prism-spec-engine`
- **Tool:** `unit_test` (MockHttpClient for network isolation)
- **Phase:** P1
- **BC:** BC-2.23.001 (primary; forward reference — see D8)
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
Fires when: `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `[auth_acquisition]` absent OR `token_path` absent.

**(b) Conflicting auth_plugin:**
`"sensor '{sensor_id}': auth_type = '{auth_type}' uses native declarative provider and does not accept auth_plugin. Remove auth_plugin or change auth_type to custom_via_plugin."`
Fires when: `auth_type ∈ {oauth2_client_credentials, token_exchange}` AND `auth_plugin` is present.

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
| New `BC-2.23.001` `[PLANNED]` | Author Declarative Auth Acquisition Token Lifecycle contract (D8) during Wave-A implementation story; postconditions P1–P8 specified in §D8 above are the authoring source | D8 |
| `VP-INDEX.md` `[PLANNED]` | Register VP-159 (D9) during Wave-A implementation story, after BC-2.23.001 is authored | D9 |
| ADR-053 D2 | Rewrite Armis TOML block: `custom_via_plugin` + `auth_plugin` → `token_exchange` + `[auth_acquisition]` block; update coherence matrix to include `token_exchange → bearer, raw`; update Rationale §Why custom_via_plugin → §Why native declarative provider | D1, D3 |
| ADR-053 Rationale §Why custom_via_plugin | Update to reflect ADR-054's native declarative provider decision | D2 |
| ADR-053 D5 manifest | Update BC-2.01.008 amendment description from `custom_via_plugin` + plugin to `token_exchange` + native provider | D1 |
| ADR-026 §D3 (partial) | Note that `token_exchange` variant added to AuthType enum per ADR-054 D1; cross-reference frontmatter `amended_by` | D1 |
| **--- CrowdStrike plugin retirement blast-radius (atomic with D5 migration story) ---** | | |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | Drop `auth_plugin = "crowdstrike-oauth2"`; add `[auth_acquisition]` block with `token_path = "/oauth2/token"` and `credential_refs` (D2) | D5 |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` (entire crate dir) | Delete source, `Cargo.toml`, WIT, committed `.prx` binary | D5 |
| `Cargo.toml` workspace `members` array | Remove `"crates/prism-spec-engine/plugins/crowdstrike-oauth2"` entry | D5 |
| `crates/prism-spec-engine/Cargo.toml` | Remove `[[test]]` block with `name = "crowdstrike_oauth2_plugin_tests"` (build target anchored by test target name `crowdstrike_oauth2_plugin_tests`); remove `async-trait` dev-dep and its comment `# async-trait: used in crowdstrike_oauth2_plugin_tests.rs for NullTestOrgIdStore stub` — both stale post-retirement; leaving the build target causes a cargo build error (missing `.rs` source file) | D5 |
| `Justfile` `build-plugin-crowdstrike-oauth2` recipe | Delete entire `build-plugin-crowdstrike-oauth2` recipe (multi-step wasm32 build + wasm-tools validate + wasm-pack chain; recipe name is the behavioral anchor) | D5 |
| `.github/workflows/ci.yml` `wasm32-compile-check` job | Remove all four crowdstrike plugin steps (anchored by step names): "Check crowdstrike-oauth2-plugin compiles", "Validate committed crowdstrike-oauth2.prx structural integrity (F-MCPRS-PRL14-LOW-001)", "Build crowdstrike-oauth2.prx", "Upload crowdstrike-oauth2.prx artifact" | D5 |
| `.github/workflows/ci.yml` first CI self-guard in `validate-workflow-structure` job | Remove `grep -qE 'just build-plugin-crowdstrike-oauth2'` + `exit 1` reachability assertion (grep pattern is the behavioral anchor) — **this guard fails CI BY DESIGN once the build step is removed; must be removed in the same change** | D5 |
| `.github/workflows/ci.yml` second CI self-guard in `validate-workflow-structure` job | Remove `grep -q 'F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike'` + `exit 1` reachability assertion (grep string `F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike` is the behavioral anchor) — **this guard fails CI BY DESIGN once the committed-.prx wasm-tools step is removed; must be removed in the same change** | D5 |
| `.config/nextest.toml` binary filter entries | Remove `binary(crowdstrike_oauth2_plugin_tests)` from all filter group expressions (grep `binary(crowdstrike_oauth2_plugin_tests)` to locate all affected filter entries — currently four occurrences in two profile blocks) | D5 |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | Port behavioral coverage (EC-001 through EC-006c, cache-hit, cache-miss, URL encoding, form body) to `DeclarativeHttpAuthProvider` unit tests before retiring; delete the WASM plugin test file | D5 |
| `crates/prism-spec-engine/tests/bc_2_16_013_crowdstrike_multiregion.rs` | Retarget the `D-747 LOCKED: auth_plugin must remain 'crowdstrike-oauth2'` assertion in the `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly` and `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works` tests (postcondition 3, `assert_eq!(spec.auth_plugin.as_deref(), Some("crowdstrike-oauth2"), …)`): remove the `auth_plugin` check and replace with an assertion confirming `spec.auth_acquisition` is present with `token_path = "/oauth2/token"` (native declarative config). D-1895/ADR-054 walks back the D-747 lock. (Behavioral anchor: the `AC-005 / D-747 LOCKED` assert_eq! message string.) | D5 |
| `crates/prism-bin/tests/plugin_boot_tests.rs` | Audit and remove CrowdStrike plugin staging paths: all call sites referencing `crowdstrike-oauth2.prx` staging and `auth_plugin = "crowdstrike-oauth2"` boot assertions (grep `crowdstrike-oauth2` in the file to locate) | D5 |
| `crates/prism-bin/tests/helpers/mod.rs` | Remove `stage_crowdstrike_plugin` function (behavioral anchor: function name `stage_crowdstrike_plugin`) and all call sites that invoke it (grep `stage_crowdstrike_plugin` to locate; currently includes `write_org_config` and integration test setup functions) | D5 |
| `crates/prism-bin/fixtures/multi-org-prism.toml.template` | Remove comment referencing `crowdstrike-oauth2.prx` staging (behavioral anchor: comment text `crowdstrike-oauth2.prx staged for SEC-003`) | D5 |
| `scripts/demo-setup.sh` | Remove Step 5 (behavioral anchor: section header `# Step 5: Copy crowdstrike-oauth2.prx plugin`): delete `PLUGIN_ARTIFACT` variable definition, the `crowdstrike-oauth2.prx` copy command, the `crowdstrike-oauth2.manifest.toml` heredoc write, and the EC-003 hard-`exit 1` guard (`if [[ ! -f "${PLUGIN_ARTIFACT}" ]]`) — this guard exits 1 at demo time once the crate directory is deleted | D5 |
| `scripts/demo-run.sh` | Remove or update the `CROWDSTRIKE_BASE_URL must be "http://127.0.0.1"` comment block that references the `crowdstrike-oauth2 plugin manifest allowed_urls` SEC-003 constraint (behavioral anchor: comment text `crowdstrike-oauth2 plugin manifest allowed_urls`); the SEC-003 host-check restriction goes away with the plugin | D5 |
| `scripts/t13-preflight-audit.py` | Retarget or remove the `[A20] HANG-FIX: plugin_status returns promptly` probe: it issues `plugin_status` for `plugin_id = "crowdstrike-oauth2"` (behavioral anchor: the `plugin_id: "crowdstrike-oauth2"` argument in the `[A20]` params block); post-retirement this plugin_id no longer exists, so the probe must be retargeted to the remaining plugin (e.g., `threatintel-lookup`) or removed from the coverage matrix | D5 |
| `docs/DEMO-RUNBOOK.md` | Update the setup step list: remove item "Copies `crowdstrike-oauth2.prx` plugin artifact" and "Writes `crowdstrike-oauth2.manifest.toml`" (behavioral anchors: the step description strings); remove the `crowdstrike-oauth2` plugin manifest SEC-003 `allowed_urls` explanation; update the `CROWDSTRIKE_BASE_URL` constraint note to remove the plugin-manifest host-check rationale | D5 |
| ARCH-INDEX `adr_registry` AD-001 | Update "26 member workspace" / "crowdstrike-oauth2 plugin member" narrative → 25-crate workspace; `root Cargo.toml members` is source of truth | D5 (architect-owned) |
| `CLAUDE.md` "26-crate workspace" count | Update `26-crate workspace (25 once ADR-037 retires...)` count — **HUMAN-FOLLOW-UP**: CLAUDE.md is human-maintained per project git rules; do NOT auto-edit; flag to human at PR time | D5 (human-follow-up) |

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
- `[PLANNED]` BC-2.23.001 will formalize the token-lifecycle contract that was previously only implicit in the
  WASM plugin source, making it testable and verifiable via VP-159 (both authored during Wave-A; see D8/D9)

### Negative / Trade-offs

- CrowdStrike migration story must execute as an atomic burst: delete `auth_plugin = "crowdstrike-oauth2"`,
  add `[auth_acquisition]`, implement `DeclarativeHttpAuthProvider`, update `boot.rs` dispatch,
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
| 0.3 | 2026-07-21 | architect | HIGH-1: add `crates/prism-spec-engine/Cargo.toml` to D11 retirement manifest (`[[test]]` crowdstrike_oauth2_plugin_tests block + async-trait dev-dep). HIGH-2: add second ci.yml self-guard row (grep `F-MCPRS-PRL14-LOW-001 PASS: committed crowdstrike`). HIGH-3: add `bc_2_16_013_crowdstrike_multiregion.rs` to D11 — D-747-LOCKED auth_plugin assertion must be retargeted to native declarative auth. HIGH-4: add `scripts/demo-setup.sh` and `scripts/demo-run.sh` to D11 (crowdstrike-oauth2.prx copy + manifest steps break post-retirement). HIGH-5: D1/D2 section headers corrected from "(supersedes ADR-026/023…)" to "(amends …)"; Source §ADR-023 reference corrected from "partially superseded" to "partially amended". MED-1: add `scripts/t13-preflight-audit.py` to D11 (`[A20]` plugin_status probe for crowdstrike-oauth2 must be retargeted). MED-2: add `docs/DEMO-RUNBOOK.md` to D11 (.prx + manifest.toml documentation stale). LOW-1: fix D2 prose + D4 step-2 form-body field order to match plugin actual order `client_id={}&client_secret={}&grant_type=client_credentials`. LOW-2: split BC-2.23.001 out of `related_bcs` into `related_bcs_planned` to prevent POL-21 phantom-anchor false-positive. OBS-1: assign ADR-054 auth-coherence check as BC-2.16.009 Rule 10 (Rule 9 reserved by ADR-053 D2 for header_scheme); update D7/D10/D11 accordingly. OBS-2: replace volatile `~line NNN` anchors in D11 with behavioral anchors (recipe name / step title / grep string / function name / filter expression) per TD-VSDD-091. |
| 0.2 | 2026-07-20 | architect | CRIT-1: retarget D5/D7/D11 from `validate_and_construct_auth_providers` to `step9a_populate_adapter_registry` (the real auth_type dispatch site in `spec_driven_adapter.rs`). HIGH-1: replace absolute `token_url` with relative `token_path` derived per-org from `resolved_spec.spec.base_url` at step 9A construction time. HIGH-2: complete D11 retirement manifest with Justfile, ci.yml (build steps + CI self-guard + committed-.prx validation), nextest.toml binary filters, crowdstrike_oauth2_plugin_tests.rs, plugin_boot_tests.rs, helpers/mod.rs staging functions, fixtures, and ARCH-INDEX/CLAUDE.md crate-count staleness. MED-1: remove phantom api_key → ApiKeyAuthProvider (api_key falls into other=> branch at step 9A → E-SPEC-012 skip). MED-2: correct bearer_static provider name to BearerStaticCredentialAuthProvider. MED-3: supersedes→amends in frontmatter (partial sub-section change is an amendment per CLAUDE.md). LOW-1: remove "replicates exactly" language. LOW-2: fix VP-159 TTL formula to match plugin's saturating_sub arithmetic; remove dead .max(1). |
| 0.1 | 2026-07-20 | architect | Initial draft per human decision D-1895 |
