//! CrowdStrike OAuth2 client-credentials plugin — in-repo .prx WASM guest.
//!
//! Story: PLUGIN-MIGRATION-001-E
//! BCs: BC-2.01.016, BC-2.17.001, BC-2.17.006, BC-2.17.007, BC-2.16.013
//!
//! This plugin replaces `crates/prism-sensors/src/auth/crowdstrike.rs`
//! (CrowdStrikeAuth + token acquisition logic) with a WASM Component that
//! the PluginRuntime loads at boot step 7.5 (BC-2.22.001).
//!
//! ## Architecture
//!
//! In WASM target (`wasm32-wasip1` / `wasm32-unknown-unknown`):
//!   - Host functions are imported via `wit_bindgen::generate!` macro-generated bindings
//!     derived from `wit/sensor-auth.wit`. This ensures Component Model ABI alignment.
//!   - The Component Model ABI wires these at instantiation time from the registered
//!     linker functions in `host_functions.rs::register_host_functions`.
//!
//! In native lib target (cargo check / cargo test on host):
//!   - `#[cfg(not(target_arch = "wasm32"))]` gates stub implementations that allow
//!     `cargo check --target <native>` to succeed without linker errors.
//!   - The native stubs are gated under `#[cfg(test)]` proving non-production-reachability
//!     (F-LP2-HIGH-004 closure). Tests use WAT fixtures, not this native lib.
//!
//! ## Credential Handling (AD-017, ADR-028 §D11 Option C)
//!
//! The WASM guest NEVER holds the raw client_secret beyond the scope of a single dispatch call.
//! Credentials are injected into `PluginConfigMap` by `PluginAuthProvider::acquire_token`
//! (after resolving via `prism_credentials::resolve_credential`). The guest reads them via
//! `host::get-config("client_id")` and `host::get-config("client_secret")`.
//! The `credential_handle` param has been removed from `acquire-token` / `get-token` (Path 4a).
//!
//! In tests, MockHost's config map is pre-populated with `"client_id" → "id"` and
//! `"client_secret" → "secret"` by default (no DTU clone required; no credential_handle string).
//!
//! KV store keys (scoped by plugin_id automatically by PluginKvStore::set):
//!   "token"           — cached bearer token string
//!   "expires_at_secs" — Unix timestamp string (u64) after which cache is stale
//!   "token_endpoint"  — OAuth2 token endpoint URL (set in config by make_host_state)
//!
//! TTL: expires_in_seconds - 30 seconds (matching CachedToken::is_valid semantics
//!   from the legacy CrowdStrikeAdapter; RFC 6749 early-expiry recommendation).

// ---------------------------------------------------------------------------
// Auth error type
// ---------------------------------------------------------------------------

/// Errors returned by acquire_token and get_token.
///
/// Traces to: BC-2.01.016 §error cases; AC-001 error-path coverage.
/// `#[non_exhaustive]` per CLAUDE.md conventions — public enum.
#[non_exhaustive]
#[derive(Debug)]
pub enum AuthError {
    /// POST /oauth2/token returned 4xx with invalid credentials.
    ///
    /// Maps to PipelineExecutor propagating SpecEngineError::AuthRefreshFailed.
    InvalidCredentials,
    /// Token response body was not valid JSON or missing required fields,
    /// OR non-2xx non-401 non-5xx response status (e.g. 4xx client errors).
    ///
    /// Detail string contains the parse failure reason or HTTP status code description
    /// (not the token value or credential data).
    /// Distinct from Transient: indicates a structural/client-side problem, not a
    /// retryable server error.
    ResponseParse(String),
    /// Token endpoint returned 5xx — transient server-side error, safe to retry.
    ///
    /// Detail string contains the HTTP status code description (not credential data).
    /// Distinct from ResponseParse: the server responded with a server-error status,
    /// not a malformed body. Callers may retry after backoff (RFC 6749 §5.2).
    Transient(String),
    /// client_id or client_secret absent from host config, KV size limit exceeded,
    /// or other internal failure.
    ///
    /// Detail string describes the internal error (not credential data).
    Internal(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "invalid client credentials"),
            AuthError::ResponseParse(detail) => write!(f, "token response parse error: {detail}"),
            AuthError::Transient(detail) => {
                write!(f, "transient token endpoint error (5xx): {detail}")
            }
            AuthError::Internal(detail) => write!(f, "internal auth error: {detail}"),
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP response type
// ---------------------------------------------------------------------------

/// HTTP response type returned by host::http-request.
///
/// Mirrors the WIT `http-response` record (F-LP2-HIGH-002). `#[non_exhaustive]`
/// per CLAUDE.md conventions (F-LP1-HIGH-005 closure).
#[non_exhaustive]
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Host function implementations
//
// WASM target: generated by wit_bindgen::generate! from wit/sensor-auth.wit.
//   This replaces the prior manual #[link(wasm_import_module = "host")] extern
//   blocks with type-safe macro-generated bindings (F-LP2-HIGH-002 closure).
//   Component Model ABI is generated from the WIT spec — no manual encoding.
//
// Native target (cargo check / cargo test on host): gated under #[cfg(test)]
//   to prove these stubs are NOT production-reachable (F-LP2-HIGH-004 closure).
//   Tests use WAT fixtures, not this native lib build.
// ---------------------------------------------------------------------------

/// WASM target: wit-bindgen-generated host function bindings AND Component Model export wiring.
///
/// The `wit_bindgen::generate!` macro reads `wit/sensor-auth.wit` and generates:
/// - Safe Rust wrappers for all host-imported functions (http-request, kv-get,
///   kv-set, current-time-secs, get-config, log)
/// - The Component Model ABI — no manual (status << 32 | len) encoding
/// - The `exports::prism::crowdstrike_oauth2::sensor_auth::Guest` trait that the
///   plugin must implement (F-LP3-HIGH-001 closure: was missing, causing WIT
///   validation failure in `validate_wit_interface` — kebab-case exports absent)
///
/// F-LP2-HIGH-002 closure: replaces the prior manual #[link] extern "C" blocks.
/// F-LP3-HIGH-001 closure: adds `impl Guest for Component` + `export!(Component)`.
#[cfg(target_arch = "wasm32")]
mod host_impl {
    use super::AuthError;
    use super::HttpResponse;

    // wit-bindgen generates host function wrappers AND export trait from the WIT spec.
    // The generated code is placed at the current module scope.
    wit_bindgen::generate!({
        world: "crowdstrike-oauth2",
        path: "wit/sensor-auth.wit",
    });

    /// Issue POST via the wit-bindgen-generated host::http-request.
    ///
    /// Token endpoint is the URL; body is the OAuth2 form-encoded credentials.
    pub fn http_request(method: &str, url: &str, body: &[u8]) -> HttpResponse {
        // wit-bindgen generates prism::crowdstrike_oauth2::host::http_request.
        // The WIT interface takes method, url, headers, body: option<list<u8>>.
        let response = prism::crowdstrike_oauth2::host::http_request(
            method,
            url,
            &[], // no extra headers for OAuth2 token endpoint (form body is self-describing)
            Some(body),
        );
        HttpResponse {
            status: response.status,
            body: response.body,
        }
    }

    /// Read from host::kv-get. Returns None if key not present.
    pub fn kv_get(key: &str) -> Option<String> {
        prism::crowdstrike_oauth2::host::kv_get(key)
    }

    /// Write to host::kv-set. Returns Err(msg) if KV limit exceeded.
    pub fn kv_set(key: &str, value: &str) -> Result<(), String> {
        prism::crowdstrike_oauth2::host::kv_set(key, value)
    }

    /// Get current wall-clock time as Unix seconds.
    pub fn current_time_secs() -> u64 {
        prism::crowdstrike_oauth2::host::current_time_secs()
    }

    /// Read a config value from the host config map.
    pub fn get_config(key: &str) -> Option<String> {
        prism::crowdstrike_oauth2::host::get_config(key)
    }

    /// Convert our crate-level `AuthError` to the wit-bindgen-generated
    /// `exports::prism::crowdstrike_oauth2::sensor_auth::AuthError` variant.
    fn to_wit_auth_error(
        e: AuthError,
    ) -> exports::prism::crowdstrike_oauth2::sensor_auth::AuthError {
        use exports::prism::crowdstrike_oauth2::sensor_auth::AuthError as WitAuthError;
        match e {
            AuthError::InvalidCredentials => WitAuthError::InvalidCredentials,
            AuthError::ResponseParse(msg) => WitAuthError::ResponseParse(msg),
            // Transient 5xx → map to Internal for WIT transport (callers may inspect the detail).
            // WIT auth-error variant does not have a Transient arm; Internal carries the detail.
            AuthError::Transient(msg) => WitAuthError::Internal(format!("transient(5xx): {msg}")),
            AuthError::Internal(msg) => WitAuthError::Internal(msg),
        }
    }

    /// Component Model export implementation.
    ///
    /// F-LP3-HIGH-001 closure: `wit_bindgen::generate!` with `export sensor-auth` in the WIT
    /// world generates an `exports::prism::crowdstrike_oauth2::sensor_auth::Guest` trait.
    /// The plugin MUST implement this trait and register via `export!(Component)` so that
    /// wit-bindgen emits the properly named kebab-case WIT exports (`auth-type-name`,
    /// `acquire-token`, `get-token`) instead of the hand-rolled snake_case `*_export`
    /// wrappers that compile to the wrong export names and fail `validate_wit_interface`.
    ///
    /// Production caller: `PluginRuntime::dispatch_plugin_acquire_token` (mod.rs:720)
    /// which calls `get_func(&mut store, "acquire-token")` — the kebab-case name is
    /// required for this lookup to succeed on the Component Model path.
    struct Component;

    impl exports::prism::crowdstrike_oauth2::sensor_auth::Guest for Component {
        /// Return the canonical auth type name.
        ///
        /// MUST return "oauth2_client_credentials" per INV-AUTH-OPEN-003 Rule A (BC-2.01.016).
        fn auth_type_name() -> String {
            super::auth_type_name().to_string()
        }

        /// Force-acquire a fresh OAuth2 token (bypass KV cache).
        ///
        /// ADR-028 §D11 Option C (Path 4a): credential-handle param removed.
        /// Reads `token_endpoint`, `client_id`, `client_secret` from host config map
        /// via `get-config(...)`. Delegates to `super::acquire_token` via WasmHost.
        ///
        /// EC-006: absent `token_endpoint` in config → `AuthError::Internal`.
        fn acquire_token()
        -> Result<String, exports::prism::crowdstrike_oauth2::sensor_auth::AuthError> {
            let token_endpoint = get_config("token_endpoint").ok_or_else(|| {
                to_wit_auth_error(AuthError::Internal(
                    "token_endpoint absent from host config (EC-006)".to_string(),
                ))
            })?;
            super::acquire_token(&super::WasmHost, &token_endpoint).map_err(to_wit_auth_error)
        }

        /// Get a cached or freshly-acquired token.
        ///
        /// ADR-028 §D11 Option C (Path 4a): credential-handle param removed.
        /// Reads `token_endpoint` from host config map, delegates to `super::get_token`.
        fn get_token() -> Result<String, exports::prism::crowdstrike_oauth2::sensor_auth::AuthError>
        {
            let token_endpoint = get_config("token_endpoint").ok_or_else(|| {
                to_wit_auth_error(AuthError::Internal(
                    "token_endpoint absent from host config (EC-006)".to_string(),
                ))
            })?;
            super::get_token(&super::WasmHost, &token_endpoint).map_err(to_wit_auth_error)
        }
    }

    // Register Component as the export implementation.
    // wit-bindgen emits properly named kebab-case WIT export symbols:
    //   auth-type-name, acquire-token, get-token
    // These are the names that PluginRuntime::dispatch_plugin_acquire_token looks up
    // via get_func(&mut store, "acquire-token") on the Component Model path.
    export!(Component);
}

// Note: the native `host_impl` stub module (which previously contained panic!() bodies for
// native builds) has been REMOVED as part of F-LP5-HIGH-002 closure. With the HostInterface
// trait refactor, `acquire_token` and `get_token` no longer call `host_impl::*` directly on
// any target. Native test builds use MockHost (defined in #[cfg(test)] mod tests) instead of
// the panic-stub host_impl. The wasm32 production path uses WasmHost → host_impl::*.
// Removing the stub eliminates dead code and clarifies the ownership boundary:
//   - wasm32 target: host_impl module generated by wit_bindgen (in host_impl block above)
//   - native test target: MockHost in #[cfg(test)] mod tests (no host_impl module needed)

// ---------------------------------------------------------------------------
// HostInterface trait — testability abstraction over host_impl::* calls
//
// F-LP5-HIGH-002 closure: `acquire_token` and `get_token` previously called
// `host_impl::*` directly, making unit testing impossible (native stubs panic).
//
// The `HostInterface` trait wraps all host function calls. On wasm32, `WasmHost`
// delegates to the wit-bindgen-generated `host_impl::*` functions. In test builds,
// `MockHost` (defined inside `#[cfg(test)] mod tests`) records calls and returns
// canned responses — enabling behavioral unit tests without a WASM runtime.
//
// This is the SID-1-compliant fix: unit tests at production module's `#[cfg(test)]
// mod tests` block driving the behavior via mock at the dependency boundary, no
// external dependency required.
// ---------------------------------------------------------------------------

/// Abstraction over host I/O functions called by `acquire_token` and `get_token`.
///
/// Implemented by:
///   - `WasmHost` (wasm32 target): delegates to wit-bindgen-generated `host_impl::*`.
///   - `MockHost` (test builds): records calls + returns canned responses for assertions.
///
/// F-LP5-HIGH-002 closure: extraction of this trait enables ≥8 unit tests covering
/// EC-001 through EC-005, cache-hit, cache-miss, and cache-empty-token paths.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) trait HostInterface {
    /// Issue an HTTP request. Returns the HTTP response.
    fn http_request(&self, method: &str, url: &str, body: &[u8]) -> HttpResponse;
    /// Read a value from the plugin KV store. Returns None if absent.
    fn kv_get(&self, key: &str) -> Option<String>;
    /// Write a value to the plugin KV store. Returns Err(msg) if KV limit exceeded.
    fn kv_set(&self, key: &str, value: &str) -> Result<(), String>;
    /// Return current wall-clock time as Unix seconds (u64).
    fn current_time_secs(&self) -> u64;
    /// Read a config value from the host config map.
    ///
    /// Called by the wasm32 Guest impl to look up `token_endpoint` from the
    /// plugin config map (EC-006). Native test builds use MockHost which returns None
    /// (config lookup not exercised in unit tests; WasmHost handles it on wasm32).
    #[allow(dead_code)]
    fn get_config(&self, key: &str) -> Option<String>;
}

/// WASM production host — delegates all calls to wit-bindgen-generated `host_impl::*`.
///
/// On wasm32 targets, this struct wraps the imports generated by `wit_bindgen::generate!`.
/// The Guest impl (`impl Guest for Component`) creates a `WasmHost` and passes it to
/// `acquire_token` / `get_token`, keeping the business logic testable on native targets.
#[cfg(target_arch = "wasm32")]
pub(crate) struct WasmHost;

#[cfg(target_arch = "wasm32")]
impl HostInterface for WasmHost {
    fn http_request(&self, method: &str, url: &str, body: &[u8]) -> HttpResponse {
        host_impl::http_request(method, url, body)
    }

    fn kv_get(&self, key: &str) -> Option<String> {
        host_impl::kv_get(key)
    }

    fn kv_set(&self, key: &str, value: &str) -> Result<(), String> {
        host_impl::kv_set(key, value)
    }

    fn current_time_secs(&self) -> u64 {
        host_impl::current_time_secs()
    }

    fn get_config(&self, key: &str) -> Option<String> {
        host_impl::get_config(key)
    }
}

// ---------------------------------------------------------------------------
// URL encoding helper (RFC 3986 percent-encoding for OAuth2 form bodies)
// ---------------------------------------------------------------------------

/// Percent-encode a string for use in an `application/x-www-form-urlencoded` body.
///
/// Encodes all characters that are NOT unreserved per RFC 3986 §2.3:
/// A-Z a-z 0-9 `-` `_` `.` `~`
///
/// Used to safely encode client_id and client_secret values in OAuth2 form bodies
/// (ADR-028 §D11 Option C; F-PR154-LOW-004 closure).
/// Prevents injection if values contain `&`, `=`, `+`, or other special characters.
///
/// This is a pure no-std-compatible implementation using only character classification —
/// no external crate required (the `url` and `urlencoding` crates are not yet workspace deps;
/// this bounded helper covers the OAuth2 use case without adding dependencies).
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3 / 2 + 1);
    for byte in input.as_bytes() {
        match byte {
            // RFC 3986 §2.3 unreserved characters — pass through unchanged.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            // All other bytes → percent-encode as %XX (uppercase hex).
            b => {
                out.push('%');
                out.push(
                    char::from_digit((b >> 4) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                out.push(
                    char::from_digit((b & 0x0f) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// SensorAuth WIT interface exports (sensor-auth interface)
// ---------------------------------------------------------------------------

/// Return the canonical auth type name for this plugin.
///
/// MUST return `"oauth2_client_credentials"` per INV-AUTH-OPEN-003 Rule A
/// (BC-2.01.016). This value is matched against `crowdstrike.sensor.toml`
/// `auth_type` field at spec-load time (ADR-028 §D2 LOCKED value).
///
/// AC-002 Red Gate Test 2 drives this function.
///
/// # Example
/// ```
/// use crowdstrike_oauth2_plugin::auth_type_name;
/// assert_eq!(auth_type_name(), "oauth2_client_credentials");
/// ```
pub fn auth_type_name() -> &'static str {
    "oauth2_client_credentials"
}

/// Force-acquire a fresh OAuth2 token by calling POST /oauth2/token.
///
/// Gated under `#[cfg(any(target_arch = "wasm32", test))]`:
/// - On wasm32: production path via WasmHost (wit-bindgen-generated host function bindings).
/// - On native: only accessible from tests via MockHost (no host_impl::* panic paths).
/// - On non-wasm, non-test: not compiled — `cargo check` does not include this function,
///   proving it is NOT production-reachable in native builds (F-LP2-HIGH-004).
///
/// This is the FORCED-REFRESH entrypoint — it MUST bypass the KV cache and
/// always issue a new token request regardless of cache state. Called by
/// `PipelineExecutor::issue_request_with_retry` on HTTP 401 (VP-150).
///
/// Steps (AC-003, AC-006):
///   1. Read client_id + client_secret from host config via get-config (ADR-028 §D11 Option C).
///   2. URL-encode client_id and client_secret values (RFC 3986 form-encoding).
///   3. Call host.http_request POST to the OAuth2 token endpoint.
///   4. Parse access_token + expires_in from the JSON response.
///   5. Cache the new token in PluginKvStore via host.kv_set.
///   6. Return Ok(access_token).
///
/// Error cases:
///   - client_id absent from config → AuthError::Internal (EC-006b).
///   - client_secret absent from config → AuthError::Internal (EC-006c).
///   - 401 response → AuthError::InvalidCredentials (EC-001).
///   - 5xx response → AuthError::Transient (MED-3: distinct from ResponseParse).
///   - Non-2xx, non-401, non-5xx → AuthError::ResponseParse (EC-002).
///   - Non-JSON or missing access_token → AuthError::ResponseParse (EC-002, EC-003).
///     The host-side `emit_acquire_token_parse_error_and_fail` (in `prism-spec-engine/src/plugin/mod.rs`)
///     emits `plugin_auth_token_parse_error` when the dispatch completes without a cached token.
///     The WASM guest does NOT emit tracing events — the host owns the tracing subscriber.
///   - expires_in missing or zero → default TTL 1799s per CrowdStrikeAdapter::acquire_token
///     `unwrap_or(1799)` semantics (EC-004).
///   - KV store full → AuthError::Internal("kv_store size limit exceeded") (EC-005).
///
/// Security invariant (AD-017): credentials live in PluginConfigMap for this call only.
/// They are NEVER stored in the KV store, NEVER logged by the host, and NEVER returned
/// in error messages (EC-006b/EC-006c use key names only, not values).
///
/// AC-003 Red Gate Test 3 / AC-006 Red Gate Test 6 drive this function.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn acquire_token(
    host: &impl HostInterface,
    token_endpoint: &str,
) -> Result<String, AuthError> {
    // ADR-028 §D11 Option C: read client_id and client_secret from host config.
    // EC-006b: client_id absent → AuthError::Internal (key name only in message, never value).
    let client_id = host.get_config("client_id").ok_or_else(|| {
        AuthError::Internal("client_id absent from host config (EC-006b)".to_string())
    })?;
    // EC-006c: client_secret absent → AuthError::Internal.
    let client_secret = host.get_config("client_secret").ok_or_else(|| {
        AuthError::Internal("client_secret absent from host config (EC-006c)".to_string())
    })?;

    // URL-encode client_id and client_secret values per RFC 3986 form-encoding.
    // This prevents injection if values contain '&', '=', or other special chars.
    let encoded_client_id = url_encode(&client_id);
    let encoded_client_secret = url_encode(&client_secret);

    // Build form body with properly encoded values.
    let form_body = format!(
        "client_id={}&client_secret={}&grant_type=client_credentials",
        encoded_client_id, encoded_client_secret
    );

    // Issue POST /oauth2/token via HostInterface.
    let response = host.http_request("POST", token_endpoint, form_body.as_bytes());

    // 401 → invalid credentials (EC-001).
    if response.status == 401 {
        return Err(AuthError::InvalidCredentials);
    }

    // 5xx → transient server error, safe to retry (MED-3: distinct from ResponseParse).
    if response.status >= 500 {
        return Err(AuthError::Transient(format!(
            "token endpoint returned HTTP {} (server error, retry after backoff)",
            response.status
        )));
    }

    // Non-2xx, non-401, non-5xx → ResponseParse (EC-002: structural client-side error).
    if response.status < 200 || response.status >= 300 {
        return Err(AuthError::ResponseParse(format!(
            "token endpoint returned HTTP {}",
            response.status
        )));
    }

    // Parse JSON body for access_token + expires_in.
    // F-LP2-HIGH-005 closure: use checked from_utf8 (not unchecked) — defense-in-depth.
    let body_str = std::str::from_utf8(&response.body)
        .map_err(|e| AuthError::ResponseParse(format!("invalid UTF-8 in response body: {e}")))?;

    let json: serde_json::Value =
        serde_json::from_str(body_str).map_err(|e| AuthError::ResponseParse(e.to_string()))?;

    let access_token = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::ResponseParse("missing access_token field".to_string()))?
        .to_string();

    // expires_in: default 1799s when missing or zero (EC-004 — matches CrowdStrikeAdapter semantics).
    let expires_in: u64 = json
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .filter(|&v| v > 0)
        .unwrap_or(1799);

    // Compute expires_at = now + expires_in - 30s buffer (CachedToken::is_valid semantics).
    let now = host.current_time_secs();
    let expires_at = now + expires_in.saturating_sub(30);

    // Cache token + expiry in KV store.
    host.kv_set("token", &access_token)
        .map_err(AuthError::Internal)?;
    host.kv_set("expires_at_secs", &expires_at.to_string())
        .map_err(AuthError::Internal)?;

    Ok(access_token)
}

/// Get a cached token, or acquire a fresh one if the cache is stale.
///
/// Steps (AC-004, AC-005):
///   1. Read expires_at_secs from KV via host.kv_get("expires_at_secs").
///   2. If present and current_time_secs() < expires_at_secs → return cached token.
///   3. Otherwise → fall through to acquire_token(host, token_endpoint).
///
/// TTL check: expires_at_secs was written as `token_issue_unix + expires_in - 30`.
/// The 30-second buffer matches CachedToken::is_valid() semantics in the legacy adapter.
///
/// ADR-028 §D11 Option C: credential-handle param removed. Credentials are read from
/// host config by acquire_token if a fresh token is needed.
///
/// AC-004 Red Gate Test 4 / AC-005 Red Gate Test 5 drive this function.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn get_token(
    host: &impl HostInterface,
    token_endpoint: &str,
) -> Result<String, AuthError> {
    // Step 1: Check cache validity.
    let now = host.current_time_secs();

    if let Some(expires_at_str) = host.kv_get("expires_at_secs")
        && let Ok(expires_at) = expires_at_str.parse::<u64>()
        && now < expires_at
    {
        // Cache hit — return cached token if non-empty.
        if let Some(cached_token) = host.kv_get("token")
            && !cached_token.is_empty()
        {
            return Ok(cached_token);
        }
    }

    // Cache miss or stale — acquire fresh token.
    acquire_token(host, token_endpoint)
}

// ---------------------------------------------------------------------------
// WASM export entrypoints
//
// F-LP3-HIGH-001 closure: The prior hand-rolled `*_export` functions (auth_type_name_export,
// acquire_token_export, get_token_export) compiled to snake_case WASM export names, which
// do NOT match the kebab-case names required by `validate_wit_interface` (discovery.rs:26):
//   SENSOR_AUTH_REQUIRED_EXPORTS = ["auth-type-name", "acquire-token", "get-token"]
//
// These hand-rolled wrappers are DELETED. The WIT exports are now generated by
// `wit_bindgen::generate!` + `impl Guest for Component` + `export!(Component)` inside
// `host_impl` (see above). wit-bindgen emits properly named kebab-case WASM export symbols.
//
// Production callers:
//   - PluginRuntime::dispatch_plugin_acquire_token (mod.rs:720): get_func(&mut store, "acquire-token")
//   - PluginRuntime::load_plugin (mod.rs:208): validate_wit_interface checks all 3 exports
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// WAT-compatible exports for WIT validation compatibility
// ---------------------------------------------------------------------------

/// Required WIT export: `name` — returns plugin identity string.
///
/// Used by PluginRuntime::load_plugin → discovery::validate_wit_interface.
/// Returns "crowdstrike-oauth2" (must match plugin_id in plugin.toml).
pub fn plugin_name() -> &'static str {
    "crowdstrike-oauth2"
}

/// Required WIT export: `version` — returns plugin semver string.
///
/// Used by PluginRuntime::load_plugin → discovery::validate_wit_interface.
pub fn plugin_version() -> &'static str {
    "0.1.0"
}

// ---------------------------------------------------------------------------
// Unit tests (native target only — these test the pure Rust logic, not WASM ABI)
//
// F-LP5-HIGH-002 closure: MockHost implements HostInterface using std::cell::RefCell
// to record calls and return canned responses. This allows unit tests to drive
// acquire_token() and get_token() on native targets without any WASM runtime.
//
// Test coverage:
//   EC-001 → test_acquire_token_EC_001_401_returns_invalid_credentials
//   EC-002 → test_acquire_token_EC_002_invalid_json_returns_response_parse  (200 + invalid JSON)
//   EC-002 → test_acquire_token_non_2xx_returns_response_parse              (defense-in-depth: non-2xx)
//   EC-003 → test_acquire_token_EC_003_missing_access_token_returns_response_parse
//   EC-004 → test_acquire_token_EC_004_missing_expires_in_defaults_to_1799
//   EC-004 → test_acquire_token_EC_004_zero_expires_in_defaults_to_1799
//   EC-005 → test_acquire_token_EC_005_kv_set_error_propagates
//   cache  → test_get_token_cache_hit_returns_cached_value
//   cache  → test_get_token_cache_miss_calls_acquire_token
//   cache  → test_get_token_cache_hit_but_empty_token_treats_as_miss
//   bonus  → test_acquire_token_form_body_contains_required_params
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)] // EC-NNN naming convention required by story §Edge Cases table
mod tests {
    use super::*;
    use std::cell::RefCell;

    // -------------------------------------------------------------------------
    // MockHost — test double for HostInterface
    //
    // F-LP5-HIGH-002 closure: MockHost wraps all host I/O behind HostInterface,
    // enabling unit tests to drive acquire_token/get_token on native targets without
    // any WASM runtime. SID-1 compliant: no external dependency, no #[ignore]'d tests.
    // -------------------------------------------------------------------------

    /// Canned HTTP response for MockHost.
    struct CannedResponse {
        status: u16,
        body: Vec<u8>,
    }

    /// MockHost records http_request calls and returns pre-configured canned responses.
    ///
    /// kv_* operations use an in-memory HashMap backed by RefCell for interior mutability.
    /// config holds key/value pairs for get_config() (ADR-028 §D11 Option C).
    struct MockHost {
        /// Responses returned by http_request() in FIFO order.
        /// Each call pops the first element; panics if the queue is empty (unexpected call).
        http_responses: RefCell<Vec<CannedResponse>>,
        /// All http_request calls recorded as (method, url, body_string).
        http_calls: RefCell<Vec<(String, String, String)>>,
        /// In-memory KV store.
        kv_store: RefCell<std::collections::HashMap<String, String>>,
        /// If Some(err_msg), kv_set returns Err with this message.
        kv_set_error: Option<String>,
        /// Simulated current time (Unix seconds).
        current_time: u64,
        /// Config map for get_config() lookups (ADR-028 §D11 Option C).
        ///
        /// Tests prime this with `"client_id"` and `"client_secret"` entries so that
        /// `acquire_token` can read them via `host.get_config(...)` instead of a
        /// credential_handle positional parameter.
        config: std::collections::HashMap<String, String>,
    }

    impl MockHost {
        fn new(current_time: u64) -> Self {
            // Pre-populate with default test credentials per ADR-028 §D11 test transition.
            // These are test values only — no real credentials.
            let mut config = std::collections::HashMap::new();
            config.insert("client_id".to_string(), "id".to_string());
            config.insert("client_secret".to_string(), "secret".to_string());
            Self {
                http_responses: RefCell::new(Vec::new()),
                http_calls: RefCell::new(Vec::new()),
                kv_store: RefCell::new(std::collections::HashMap::new()),
                kv_set_error: None,
                current_time,
                config,
            }
        }

        /// Queue a canned HTTP response.
        fn push_http_response(&mut self, status: u16, body: &str) {
            self.http_responses.borrow_mut().push(CannedResponse {
                status,
                body: body.as_bytes().to_vec(),
            });
        }

        /// Pre-populate KV store with a key/value pair.
        fn set_kv(&mut self, key: &str, value: &str) {
            self.kv_store
                .borrow_mut()
                .insert(key.to_string(), value.to_string());
        }

        /// Configure the next kv_set to fail with the given error message.
        fn fail_kv_set_with(&mut self, err: &str) {
            self.kv_set_error = Some(err.to_string());
        }

        /// Return the number of http_request calls made.
        fn http_call_count(&self) -> usize {
            self.http_calls.borrow().len()
        }

        /// Return the body of the Nth http_request call (0-indexed), as a String.
        fn http_call_body(&self, idx: usize) -> String {
            self.http_calls.borrow()[idx].2.clone()
        }
    }

    impl HostInterface for MockHost {
        fn http_request(&self, method: &str, url: &str, body: &[u8]) -> HttpResponse {
            self.http_calls.borrow_mut().push((
                method.to_string(),
                url.to_string(),
                String::from_utf8_lossy(body).into_owned(),
            ));
            let response = self.http_responses.borrow_mut().remove(0); // FIFO; panics if queue empty
            HttpResponse {
                status: response.status,
                body: response.body,
            }
        }

        fn kv_get(&self, key: &str) -> Option<String> {
            self.kv_store.borrow().get(key).cloned()
        }

        fn kv_set(&self, key: &str, value: &str) -> Result<(), String> {
            if let Some(ref err) = self.kv_set_error {
                return Err(err.clone());
            }
            self.kv_store
                .borrow_mut()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn current_time_secs(&self) -> u64 {
            self.current_time
        }

        fn get_config(&self, key: &str) -> Option<String> {
            // ADR-028 §D11 Option C: acquire_token now reads client_id + client_secret
            // from config. MockHost returns values from the `config` HashMap.
            self.config.get(key).cloned()
        }
    }

    // -------------------------------------------------------------------------
    // auth_type_name tests (pre-existing)
    // -------------------------------------------------------------------------

    /// auth_type_name() MUST return "oauth2_client_credentials" per INV-AUTH-OPEN-003 Rule A.
    #[test]
    fn test_auth_type_name_returns_canonical_value() {
        assert_eq!(
            auth_type_name(),
            "oauth2_client_credentials",
            "auth_type_name() must return 'oauth2_client_credentials' per INV-AUTH-OPEN-003 Rule A \
             (BC-2.01.016); the value MUST match crowdstrike.sensor.toml auth_type field"
        );
    }

    /// Verify auth_type_name() length matches the canonical string (25 bytes).
    ///
    /// "oauth2_client_credentials" has 25 characters.
    /// The adversary pass-1 report cited 24 bytes — that is incorrect.
    #[test]
    fn test_auth_type_name_byte_length_is_25() {
        let name = auth_type_name();
        assert_eq!(
            name.len(),
            25,
            "auth_type_name() must be exactly 25 bytes ('oauth2_client_credentials'); got {} bytes",
            name.len()
        );
    }

    // -------------------------------------------------------------------------
    // acquire_token behavioral tests (EC-001 through EC-005 + bonus)
    // -------------------------------------------------------------------------

    /// EC-001: POST /oauth2/token returns HTTP 401 → AuthError::InvalidCredentials.
    ///
    /// Asserts variant-level match (not just is_err()). Asserts no KV-set occurred
    /// (token must not be cached when credentials are invalid).
    #[test]
    fn test_acquire_token_EC_001_401_returns_invalid_credentials() {
        let mut host = MockHost::new(1_000_000);
        host.push_http_response(401, r#"{"error": "invalid_client"}"#);

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(result, Err(AuthError::InvalidCredentials)),
            "EC-001: 401 response MUST return AuthError::InvalidCredentials, got: {:?}",
            result
        );
        // No token should be cached on credential failure.
        assert_eq!(
            host.kv_store.borrow().get("token"),
            None,
            "EC-001: token MUST NOT be cached when credentials are invalid"
        );
        assert_eq!(
            host.http_call_count(),
            1,
            "EC-001: exactly one HTTP call expected"
        );
    }

    /// EC-002: POST /oauth2/token returns HTTP 200 but body is not valid JSON →
    /// AuthError::ResponseParse.
    ///
    /// This is the spec'd EC-002 scenario: the token endpoint returns a success status
    /// code but the response body cannot be parsed as JSON. The serde_json::from_str
    /// error branch is exercised (separate from the non-2xx status branch).
    ///
    /// Asserts: (a) variant-level match on AuthError::ResponseParse, (b) error detail
    /// contains a serde-json parse indication, (c) no token cached.
    #[test]
    fn test_acquire_token_EC_002_invalid_json_returns_response_parse() {
        let mut host = MockHost::new(1_000_000);
        // HTTP 200 with syntactically invalid JSON body — triggers serde_json::from_str error.
        host.push_http_response(200, "this is not JSON {[");

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::ResponseParse(_))),
            "EC-002: 200 + invalid JSON body MUST return AuthError::ResponseParse, got: {:?}",
            result
        );
        // Verify the error detail contains serde-json parse information (not just an empty string).
        if let Err(AuthError::ResponseParse(detail)) = &result {
            assert!(
                !detail.is_empty(),
                "EC-002: ResponseParse detail MUST be non-empty (serde_json error description)"
            );
        }
        assert_eq!(
            host.kv_store.borrow().get("token"),
            None,
            "EC-002: token MUST NOT be cached when JSON parsing fails"
        );
    }

    /// MED-3: POST /oauth2/token returns 5xx → AuthError::Transient (NOT ResponseParse).
    ///
    /// 5xx responses are transient server errors distinct from ResponseParse (which
    /// indicates a structural parse failure). Callers can safely retry 5xx after backoff.
    /// Asserts: (a) variant-level match on AuthError::Transient, (b) detail contains "503",
    /// (c) no token cached.
    #[test]
    fn test_acquire_token_5xx_returns_transient() {
        let mut host = MockHost::new(1_000_000);
        host.push_http_response(503, "Service Unavailable");

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::Transient(msg)) if msg.contains("503")),
            "5xx response MUST return AuthError::Transient with status code, got: {:?}",
            result
        );
        assert_eq!(
            host.kv_store.borrow().get("token"),
            None,
            "5xx: token MUST NOT be cached on 5xx response"
        );
    }

    /// Defense-in-depth: POST /oauth2/token returns non-2xx, non-401, non-5xx (e.g. 400) →
    /// AuthError::ResponseParse.
    ///
    /// Note: This test covers the residual non-2xx path (4xx other than 401).
    /// 5xx → Transient (see test_acquire_token_5xx_returns_transient).
    /// 401 → InvalidCredentials (EC-001).
    /// This covers 4xx like 400, 403, 404 → ResponseParse.
    ///
    /// Asserts variant match with detail string containing the status code.
    /// Asserts no KV-set occurred (token must not be cached on error response).
    #[test]
    fn test_acquire_token_4xx_non_401_returns_response_parse() {
        let mut host = MockHost::new(1_000_000);
        host.push_http_response(400, "Bad Request");

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::ResponseParse(msg)) if msg.contains("400")),
            "4xx (non-401) response MUST return AuthError::ResponseParse with status code, got: {:?}",
            result
        );
        assert_eq!(
            host.kv_store.borrow().get("token"),
            None,
            "4xx: token MUST NOT be cached on non-2xx response"
        );
    }

    /// EC-003: Token response is missing `access_token` field → AuthError::ResponseParse.
    ///
    /// Asserts: (a) variant match with detail "missing access_token field",
    /// (b) no token cached (mirrors EC-001 and EC-002 no-cache invariant per spec EC-003 row).
    #[test]
    fn test_acquire_token_EC_003_missing_access_token_returns_response_parse() {
        let mut host = MockHost::new(1_000_000);
        host.push_http_response(200, r#"{"token_type": "bearer", "expires_in": 3600}"#);

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::ResponseParse(msg)) if msg.contains("missing access_token")),
            "EC-003: missing access_token MUST return AuthError::ResponseParse('missing access_token field'), got: {:?}",
            result
        );
        // F-LP7-LOW-001 closure: sibling-symmetry with EC-001 and EC-002 no-cache assertions.
        // Production code short-circuits before any kv_set call when access_token is absent,
        // but a regression moving access_token extraction after kv_set would not be caught
        // without this assertion. Mirrors pattern in test_acquire_token_EC_001_* and test_acquire_token_EC_002_*.
        assert_eq!(
            host.kv_store.borrow().get("token"),
            None,
            "EC-003: token MUST NOT be cached when access_token field is missing"
        );
    }

    /// EC-004: `expires_in` field is missing → defaults to 1799s TTL.
    ///
    /// Asserts the KV-set value for expires_at_secs equals now + 1799 - 30.
    /// The TTL math is: now + expires_in.saturating_sub(30) where expires_in defaults to 1799.
    #[test]
    fn test_acquire_token_EC_004_missing_expires_in_defaults_to_1799() {
        let now: u64 = 1_000_000;
        let mut host = MockHost::new(now);
        // expires_in field absent — must trigger the unwrap_or(1799) default.
        host.push_http_response(
            200,
            r#"{"access_token": "tok-abc", "token_type": "bearer"}"#,
        );

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            result.is_ok(),
            "EC-004: absent expires_in must succeed, got: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            "tok-abc",
            "EC-004: returned token must match access_token field"
        );

        let expected_expires_at = now + 1799u64.saturating_sub(30); // = now + 1769
        let stored_expires_at: u64 = host
            .kv_store
            .borrow()
            .get("expires_at_secs")
            .expect("EC-004: expires_at_secs must be set in KV store")
            .parse()
            .expect("EC-004: expires_at_secs must be a valid u64");
        assert_eq!(
            stored_expires_at, expected_expires_at,
            "EC-004: expires_at_secs MUST equal now({now}) + 1799 - 30 = {expected_expires_at}, got {stored_expires_at}"
        );
    }

    /// EC-004 (zero case): `expires_in` field is zero → defaults to 1799s TTL.
    ///
    /// Story spec EC-004: "Token response `expires_in` field is missing **or zero**".
    /// The `.filter(|&v| v > 0).unwrap_or(1799)` chain handles both cases:
    ///   - missing: `and_then` returns None → unwrap_or fires
    ///   - zero: filter rejects v==0 → unwrap_or fires
    ///
    /// A regression removing `.filter(|&v| v > 0)` would compile clean and pass all other
    /// tests. Zero `expires_in` would produce `expires_at = now + 0.saturating_sub(30) = now`,
    /// immediately stale → infinite token-refresh loops in production.
    ///
    /// Asserts expires_at_secs == now + 1799 - 30 (== now + 1769), same as the missing case.
    #[test]
    fn test_acquire_token_EC_004_zero_expires_in_defaults_to_1799() {
        let now: u64 = 1_000_000;
        let mut host = MockHost::new(now);
        // expires_in is present but zero — must trigger .filter(|&v| v > 0) rejection.
        host.push_http_response(
            200,
            r#"{"access_token": "tok-abc", "token_type": "bearer", "expires_in": 0}"#,
        );

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            result.is_ok(),
            "EC-004 (zero): zero expires_in must succeed (defaults to 1799s TTL), got: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            "tok-abc",
            "EC-004 (zero): returned token must match access_token field"
        );

        let expected_expires_at = now + 1799u64.saturating_sub(30); // = now + 1769
        let stored_expires_at: u64 = host
            .kv_store
            .borrow()
            .get("expires_at_secs")
            .expect("EC-004 (zero): expires_at_secs must be set in KV store")
            .parse()
            .expect("EC-004 (zero): expires_at_secs must be a valid u64");
        assert_eq!(
            stored_expires_at, expected_expires_at,
            "EC-004 (zero): expires_at_secs MUST equal now({now}) + 1799 - 30 = {expected_expires_at} \
             (zero expires_in MUST be treated same as missing), got {stored_expires_at}"
        );
    }

    /// EC-005: kv_set returns error (KV size limit exceeded) → AuthError::Internal propagated.
    ///
    /// Asserts variant match with Internal variant.
    #[test]
    fn test_acquire_token_EC_005_kv_set_error_propagates() {
        let mut host = MockHost::new(1_000_000);
        host.push_http_response(200, r#"{"access_token": "tok-abc", "expires_in": 3600}"#);
        host.fail_kv_set_with("kv_store size limit exceeded");

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::Internal(msg)) if msg.contains("kv_store size limit exceeded")),
            "EC-005: kv_set error MUST return AuthError::Internal with the error message, got: {:?}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // get_token cache behavior tests
    // -------------------------------------------------------------------------

    /// Cache hit: get_token returns the cached value without issuing an HTTP request.
    ///
    /// Asserts: (a) Ok with cached token, (b) zero http_request calls made.
    #[test]
    fn test_get_token_cache_hit_returns_cached_value() {
        let now: u64 = 1_000_000;
        let mut host = MockHost::new(now);
        // Pre-populate cache with a valid, non-expired token.
        let expires_at = now + 3600; // well in the future
        host.set_kv("token", "cached-bearer-token");
        host.set_kv("expires_at_secs", &expires_at.to_string());
        // No HTTP responses queued — if http_request is called, MockHost will panic.

        let result = get_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Ok(tok) if tok == "cached-bearer-token"),
            "cache hit: get_token MUST return cached token without HTTP call, got: {:?}",
            result
        );
        assert_eq!(
            host.http_call_count(),
            0,
            "cache hit: ZERO http_request calls expected (cache was valid)"
        );
    }

    /// Cache miss: get_token calls acquire_token when no cache entry exists.
    ///
    /// Asserts: (a) Ok with newly acquired token, (b) exactly one http_request call.
    #[test]
    fn test_get_token_cache_miss_calls_acquire_token() {
        let mut host = MockHost::new(1_000_000);
        // No KV entries — cache miss.
        host.push_http_response(
            200,
            r#"{"access_token": "fresh-token", "expires_in": 3600}"#,
        );

        let result = get_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Ok(tok) if tok == "fresh-token"),
            "cache miss: get_token MUST call acquire_token and return new token, got: {:?}",
            result
        );
        assert_eq!(
            host.http_call_count(),
            1,
            "cache miss: exactly ONE http_request call expected"
        );
    }

    /// Cache-hit-but-empty-token: cached token string is empty → treated as cache miss.
    ///
    /// The `!cached_token.is_empty()` branch in get_token must treat an empty cached
    /// token as a miss and fall through to acquire_token. This asserts that path.
    ///
    /// Asserts: (a) Ok with freshly-acquired token, (b) exactly one http_request call.
    #[test]
    fn test_get_token_cache_hit_but_empty_token_treats_as_miss() {
        let now: u64 = 1_000_000;
        let mut host = MockHost::new(now);
        // expires_at is in the future (valid TTL) but token value is empty string.
        let expires_at = now + 3600;
        host.set_kv("token", ""); // empty token — must treat as miss
        host.set_kv("expires_at_secs", &expires_at.to_string());
        host.push_http_response(
            200,
            r#"{"access_token": "fresh-after-empty-cache", "expires_in": 3600}"#,
        );

        let result = get_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Ok(tok) if tok == "fresh-after-empty-cache"),
            "empty-token cache: get_token MUST fall through to acquire_token, got: {:?}",
            result
        );
        assert_eq!(
            host.http_call_count(),
            1,
            "empty-token cache: exactly ONE http_request call expected (fell through to acquire)"
        );
    }

    // -------------------------------------------------------------------------
    // Bonus: form body content assertion
    // -------------------------------------------------------------------------

    /// Form body correctness: acquire_token form body contains client_id, client_secret,
    /// and grant_type=client_credentials, read from host config (ADR-028 §D11 Option C).
    ///
    /// Asserts:
    ///   (a) grant_type=client_credentials present
    ///   (b) client_id=my-id present (from host config, URL-encoded)
    ///   (c) client_secret=my-secret present (from host config, URL-encoded)
    ///
    /// This is the LOAD-BEARING test for F-PR154-CRIT-2: the production defect was
    /// `credential_handle = "sensor:crowdstrike"` producing an invalid form body.
    /// With Option C, the form body is built from explicit client_id + client_secret config keys.
    #[test]
    fn test_acquire_token_form_body_contains_required_params() {
        let mut host = MockHost::new(1_000_000);
        // Override default config with specific named values.
        host.config
            .insert("client_id".to_string(), "my-id".to_string());
        host.config
            .insert("client_secret".to_string(), "my-secret".to_string());
        host.push_http_response(200, r#"{"access_token": "tok-xyz", "expires_in": 3600}"#);

        let _ = acquire_token(&host, "https://example.com/oauth2/token");

        assert_eq!(host.http_call_count(), 1, "exactly one HTTP call expected");
        let body = host.http_call_body(0);
        assert!(
            body.contains("grant_type=client_credentials"),
            "form body MUST contain grant_type=client_credentials, got: {body:?}"
        );
        assert!(
            body.contains("client_id=my-id"),
            "form body MUST contain client_id from host config (EC-006b path), got: {body:?}"
        );
        assert!(
            body.contains("client_secret=my-secret"),
            "form body MUST contain client_secret from host config (EC-006c path), got: {body:?}"
        );
    }

    /// EC-006b: client_id absent from host config → AuthError::Internal.
    ///
    /// ADR-028 §D11 error code EC-006b: acquire_token MUST fail with Internal when
    /// client_id is not in the host config map.
    #[test]
    fn test_acquire_token_EC_006b_missing_client_id_returns_internal() {
        let mut host = MockHost::new(1_000_000);
        // Remove client_id from config.
        host.config.remove("client_id");
        // No HTTP response queued — should fail before making any HTTP call.

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::Internal(msg)) if msg.contains("client_id absent") && msg.contains("EC-006b")),
            "EC-006b: missing client_id MUST return AuthError::Internal with EC-006b code, got: {:?}",
            result
        );
        assert_eq!(
            host.http_call_count(),
            0,
            "EC-006b: NO HTTP call should be made when client_id is absent"
        );
    }

    /// EC-006c: client_secret absent from host config → AuthError::Internal.
    ///
    /// ADR-028 §D11 error code EC-006c: acquire_token MUST fail with Internal when
    /// client_secret is not in the host config map.
    #[test]
    fn test_acquire_token_EC_006c_missing_client_secret_returns_internal() {
        let mut host = MockHost::new(1_000_000);
        // Remove client_secret from config.
        host.config.remove("client_secret");
        // No HTTP response queued — should fail before making any HTTP call.

        let result = acquire_token(&host, "https://example.com/oauth2/token");

        assert!(
            matches!(&result, Err(AuthError::Internal(msg)) if msg.contains("client_secret absent") && msg.contains("EC-006c")),
            "EC-006c: missing client_secret MUST return AuthError::Internal with EC-006c code, got: {:?}",
            result
        );
        assert_eq!(
            host.http_call_count(),
            0,
            "EC-006c: NO HTTP call should be made when client_secret is absent"
        );
    }

    /// URL encoding: special characters in client_id and client_secret are percent-encoded.
    ///
    /// Client credentials may contain `+`, `=`, `&`, `%` or other special chars.
    /// These MUST be percent-encoded before splicing into the form body to prevent injection.
    #[test]
    fn test_acquire_token_url_encodes_credentials() {
        let mut host = MockHost::new(1_000_000);
        // Credentials with special characters that require encoding.
        host.config
            .insert("client_id".to_string(), "id+with spaces".to_string());
        host.config.insert(
            "client_secret".to_string(),
            "secret&with=special".to_string(),
        );
        host.push_http_response(200, r#"{"access_token": "tok", "expires_in": 3600}"#);

        let _ = acquire_token(&host, "https://example.com/oauth2/token");

        let body = host.http_call_body(0);
        // "id+with spaces" → "id%2Bwith%20spaces" (+ is %2B, space is %20)
        assert!(
            !body.contains("id+with spaces"),
            "URL-encoding: literal '+' and space in client_id MUST be percent-encoded, got: {body:?}"
        );
        // "secret&with=special" → percent-encoded (& and = must be encoded)
        assert!(
            !body.contains("secret&with=special"),
            "URL-encoding: literal '&' and '=' in client_secret MUST be percent-encoded, got: {body:?}"
        );
        // grant_type must still be present
        assert!(
            body.contains("grant_type=client_credentials"),
            "URL-encoding: grant_type must still be present after encoding, got: {body:?}"
        );
    }
}
