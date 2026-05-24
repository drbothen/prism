# Credential Substitution Model — PLUGIN-MIGRATION-001-E PR-LEVEL CRIT #2

**Date:** 2026-05-24
**Author:** architect
**Source:** PR #154 pr-reviewer CRIT #2 adjudication; user-authorized fix-in-scope
**Normative document:** ADR-028 §D11 (this memo is the rationale companion)

---

## Decision Summary

**Option C selected:** Host resolves `credential_handle` to `(client_id, client_secret)` via `prism_credentials::resolve_credential` before dispatch. Resolved values are injected into `PluginConfigMap`. The WASM guest reads them via `host::get-config("client_id")` and `host::get-config("client_secret")`, then builds the OAuth2 form body explicitly.

The authoritative decision record and implementer contract are in ADR-028 §D11. This memo captures the adjudication reasoning and architectural constraints for session continuity.

---

## The Defect

`boot.rs::validate_and_construct_auth_providers` (line ~218) constructs:

```rust
let credential_handle = format!("sensor:{sensor_id}");
```

This opaque keyring reference (per AD-017) is forwarded through `PluginAuthProvider` to `dispatch_plugin_acquire_token`, which puts it in `PluginConfigMap["credential_handle"]`. The WASM guest's `acquire_token` does:

```rust
let form_body = format!("{}&grant_type=client_credentials", credential_handle);
```

Result: `sensor:crowdstrike&grant_type=client_credentials` — not a valid OAuth2 form body. CrowdStrike's `/oauth2/token` returns 4xx. Tests pass literal `"client_id=test&client_secret=test"` as `credential_handle`, masking the production bug.

The lib.rs docstring at lines 28-30 claims "host substitutes via host_http_request credential substitution" — but `host_http_request` performs zero substitution. The claim is architecturally aspirational but unimplemented. This is a standing rule 3 §3 violation: doc comment claiming behavior X with no corresponding implementation.

---

## Options Considered

### Option A: host_http_request sentinel substitution
The guest emits a body containing a sentinel (`${credential:handle}`) that `host_http_request` recognizes and substitutes inline by looking up the keyring.

**Rejected.** Three problems:
1. **Fragile pattern matching.** Any character escaping, URL encoding, or parameter ordering change in the POST body silently breaks the substitution. The sentinel is a string convention, not a typed interface.
2. **Concern mixing.** HTTP execution and credential resolution are orthogonal responsibilities. Mixing them in `host_http_request` violates single-responsibility and makes `host_http_request` context-sensitive (different behavior for bodies containing the sentinel vs not).
3. **Invisible contract.** The sentinel syntax is invisible to the WIT interface, the host function signature, and the guest type system. A future plugin author has no way to know the sentinel mechanism exists without reading `host_http_request` source.

### Option B: WIT param expansion to client_id/client_secret strings
Change the WIT `acquire-token` signature from `(credential-handle: string)` to `(client-id: string, client-secret: string)`.

**Rejected.** AD-017 prohibits credential values from transiting AI context. WIT params are visible to:
- wit-bindgen trace logging (enabled by default in debug builds)
- Component Model ABI introspection tools
- Any wasmtime debug/profiling hooks

The exposed value would be the raw client_secret string passed as a WIT string param. This is a direct AD-017 violation. Additionally, this change requires amending BC-2.17.006 (WIT validation gate) and the WIT file, which has wider blast radius and requires a separate BC amendment cycle.

### Option C: get_config injection at dispatch time (SELECTED)

Before `dispatch_plugin_acquire_token` is called, `PluginAuthProvider::acquire_token` calls `prism_credentials::resolve_credential` for both `"client_id"` and `"client_secret"`. The resolved `SecretString` values are exposed once (`expose_secret()`) to populate a `PluginConfigMap` that is passed into the dispatch call. The WASM guest reads via `host::get-config`.

---

## AD-017 Compliance Analysis

AD-017 principle: "Credentials never transit AI context. Reference-based model with CLI/env/vault paths."

In this context "AI context" means: tracing log output, MCP tool responses, error messages, and any string that could be captured in an AI conversation transcript.

| Concern | Analysis |
|---------|----------|
| `client_secret` in tracing logs | `host_get_config` is currently silent — it returns `Option<String>` without any `tracing::*!` emission. This must remain true; no logging of config values is permitted. |
| `client_secret` in error messages | `AuthError` variants carry structural descriptions ("invalid client credentials", "token endpoint returned HTTP 401") — never config values. EC-006b/EC-006c new errors use literal key names only. |
| `client_secret` in KV store | Only `token` (the bearer result) and `expires_at_secs` are KV-stored. `client_secret` is never written to KV. |
| `client_secret` retained across calls | `Arc<PluginConfigMap>` is constructed per-dispatch inside `dispatch_plugin_acquire_token`. The Store is dropped when the function returns, deallocating the Arc copy. |
| `client_secret` accessible from WASM linear memory after call | The WASM Store is dropped after `func.call` returns. Linear memory is deallocated with the Store. |
| `expose_secret()` call site | Single call at `PluginAuthProvider::acquire_token` when building `PluginConfigMap`. This is the minimum required exposure boundary. No other site materializes the credential value. |

**Verdict: Option C is AD-017 compliant** with one mandatory constraint: `host_get_config` must never emit tracing for the returned value.

Note on TD-S-PLUGIN-PREREQ-B-002 (AuthToken zeroize gap): `client_id` and `client_secret` as plain `String` in `PluginConfigMap` are subject to the same residual-in-heap gap as `AuthToken`. This is pre-existing and in scope for the TD. Option C does not worsen this gap relative to any other approach — the credential must materialize somewhere in host memory to POST it to the token endpoint.

---

## Implementation Sketch

### 1. `dispatch_plugin_acquire_token` signature change

Before:
```rust
pub fn dispatch_plugin_acquire_token(
    &self,
    plugin_id: &str,
    credential_handle: &str,
    token_endpoint: &str,
) -> Result<String, PluginError>
```

After:
```rust
pub fn dispatch_plugin_acquire_token(
    &self,
    plugin_id: &str,
    config: &PluginConfigMap,  // must contain "client_id", "client_secret", "token_endpoint"
) -> Result<String, PluginError>
```

Remove the internal `PluginConfigMap::from([("credential_handle"...), ("token_endpoint"...)])` construction inside the function. Use the caller-provided `config` directly.

### 2. `PluginAuthProvider::acquire_token` — credential resolution

```rust
// prism-spec-engine/src/auth_provider.rs
async fn acquire_token(&self) -> Result<AuthToken, SpecEngineError> {
    let client_id = prism_credentials::resolve_credential(
        &self.client_id_or_org,
        &self.sensor_id,
        "client_id",
    ).await.map_err(|e| SpecEngineError::AuthRefreshFailed {
        sensor_id: self.sensor_id.clone(),
        detail: e.to_string(),
    })?;

    let client_secret = prism_credentials::resolve_credential(
        &self.client_id_or_org,
        &self.sensor_id,
        "client_secret",
    ).await.map_err(|e| SpecEngineError::AuthRefreshFailed {
        sensor_id: self.sensor_id.clone(),
        detail: e.to_string(),
    })?;

    use secrecy::ExposeSecret;
    let config = PluginConfigMap::from([
        ("client_id".to_string(), client_id.expose_secret().to_string()),
        ("client_secret".to_string(), client_secret.expose_secret().to_string()),
        ("token_endpoint".to_string(), self.token_endpoint.clone()),
    ]);

    let token_str = self.runtime.dispatch_plugin_acquire_token(&self.plugin_id, &config)?;
    Ok(AuthToken::new(token_str))
}
```

### 3. WASM guest `acquire_token` — remove credential_handle usage

Before:
```rust
pub(crate) fn acquire_token(
    host: &impl HostInterface,
    credential_handle: &str,
    token_endpoint: &str,
) -> Result<String, AuthError> {
    let form_body = format!("{}&grant_type=client_credentials", credential_handle);
    ...
}
```

After:
```rust
pub(crate) fn acquire_token(
    host: &impl HostInterface,
    token_endpoint: &str,
) -> Result<String, AuthError> {
    let client_id = host.get_config("client_id")
        .ok_or_else(|| AuthError::Internal("client_id absent from host config (EC-006b)".to_string()))?;
    let client_secret = host.get_config("client_secret")
        .ok_or_else(|| AuthError::Internal("client_secret absent from host config (EC-006c)".to_string()))?;
    let form_body = format!(
        "client_id={}&client_secret={}&grant_type=client_credentials",
        client_id, client_secret
    );
    ...
}
```

### 4. WIT interface

The WIT `sensor-auth.wit` `acquire-token` export currently takes `credential-handle: string`. Two paths:

**Path 4a (preferred):** Remove `credential-handle` param from WIT entirely — it is now unused by the production guest. The host passes credentials through the config map, not as WIT params. Update BC-2.17.006 WIT validation gate accordingly.

**Path 4b (compatibility stub):** Retain `credential-handle: string` in WIT for backward compatibility during the migration window, but the guest ignores it. The implementer may choose this if WIT versioning is a concern, but the `credential-handle` param must be explicitly documented as `[deprecated — use host::get-config("client_id") and host::get-config("client_secret")]`.

The architect recommendation is **Path 4a** (clean removal) since no other plugin currently uses this WIT signature and the migration window is bounded.

---

## Test Strategy

### What the current tests get wrong

All unit tests in `lib.rs` (EC-001 through EC-005, cache tests) pass:
```rust
acquire_token(&host, "client_id=my-id&client_secret=my-secret", "https://...")
```

After the fix, `acquire_token` reads from config — not from a positional string. Tests must be updated to prime `MockHost::get_config` returns:

```rust
// Update MockHost::get_config from "return None" to reading from a config HashMap:
struct MockHost {
    ...
    config: HashMap<String, String>,  // add this field
}

impl HostInterface for MockHost {
    fn get_config(&self, key: &str) -> Option<String> {
        self.config.get(key).cloned()  // was: return None
    }
}

// Test setup:
host.config.insert("client_id".to_string(), "my-id".to_string());
host.config.insert("client_secret".to_string(), "my-secret".to_string());
```

The `test_acquire_token_form_body_contains_required_params` test is updated to:
```rust
assert!(body.contains("client_id=my-id"), ...);
assert!(body.contains("client_secret=my-secret"), ...);
assert!(body.contains("grant_type=client_credentials"), ...);
```

### New required tests

1. **EC-006b:** `get_config("client_id")` returns `None` → `AuthError::Internal("client_id absent")`.
2. **EC-006c:** `get_config("client_secret")` returns `None` → `AuthError::Internal("client_secret absent")`.
3. **`dispatch_plugin_acquire_token` integration test:** Pass a `PluginConfigMap` with explicit `client_id`/`client_secret`. Assert the WAT fixture token is returned. (WAT path is test-only; Component Model integration requires a real `.prx`.)
4. **Form body correctness test (production path proof):** An integration test using wiremock or the DTU clone that verifies the POST body received by the OAuth2 endpoint contains `client_id=...&client_secret=...&grant_type=client_credentials`. This is the class-level test that catches the F-LP12-PR-CRIT-2 defect family.

### DTU integration test (SID-1 compliance)

Per SID-1, the `#[ignore]`-tagged DTU integration test must cite the blocking dependency:

```rust
#[test]
#[ignore = "DTU-EXT-002: requires prism-dtu-crowdstrike running; ungated in CI after PLUGIN-MIGRATION-001-A deploys"]
fn test_dispatch_plugin_acquire_token_crowdstrike_full_oauth2_flow() { ... }
```

The unit tests above (EC-006b, EC-006c, form body correctness via MockHost) are NOT `#[ignore]`'d and provide the SID-1 required unit coverage without external dependency.

---

## Files Affected (Absolute Paths)

| File | Change Type |
|------|------------|
| `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md` | §D11 added (this burst) |
| `crates/prism-spec-engine/src/plugin/mod.rs` | `dispatch_plugin_acquire_token` signature change |
| `crates/prism-spec-engine/src/auth_provider.rs` | Add credential resolution before dispatch |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` | `acquire_token` form body construction; remove `credential_handle` param |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/sensor-auth.wit` | Remove `credential-handle` param from `acquire-token` (Path 4a) |
| `crates/prism-bin/src/boot.rs` | `validate_and_construct_auth_providers`: update `PluginAuthProvider::new` call if constructor changes |
| Unit test files in above crates | MockHost `get_config` update; EC-006b/EC-006c tests; form body assertion update |

---

## ADR-028 Amendment Required

Yes — §D11 added to ADR-028 v1.11 (this burst). The amendment covers the normative decision, AD-017 analysis, data flow, implementer contract, and test strategy. No new ADR number is required — this is a natural extension of ADR-028's plugin dispatch authority.
