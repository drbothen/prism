//! Plugin host functions — the only interfaces available to WASM plugins.
//!
//! ## Allowlist enforcement (AC-7 / VP-PLUGIN-007)
//!
//! `host_http_request` enforces allowlist using host-only `==` comparison via `url::Url::parse`.
//! The allowlist is `Vec<String>` (not `Option`) — empty list = default-deny all outbound HTTP.
//! A blocked request returns HTTP 403 to the plugin and emits a single structured
//! `tracing::warn!(event_type = "plugin_http_request_blocked", ...)` per BC-2.16.002 catalog.
//!
//! ## Timeout (AC-9 / TD-S-PLUGIN-PREREQ-B-005 closure)
//!
//! Effective per-request timeout is 30-second, enforced by the shared `reqwest::Client`
//! constructed in `boot.rs` with `.timeout(Duration::from_secs(PLUGIN_HTTP_CLIENT_TIMEOUT_SECS))`.
//! No per-request `.timeout()` override is set here — the Client-level timeout is the source
//! of truth.

use std::time::Instant;

use tracing::{debug, error, info, trace, warn};
use url::Url;

use super::loader::HostState;

/// HTTP response type returned to the WASM plugin from `host::http_request`.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Log level mirroring the `log-level` enum in the WIT `host` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Execute an HTTP request on behalf of a plugin via the host's `reqwest::Client`.
///
/// - Validates the URL against `HostState.allowed_urls` (Vec<String>, default-deny).
/// - Enforces 30-second per-request timeout via shared `reqwest::Client` from boot.rs.
/// - Audit-logs `(plugin_id, method, url, status, latency_ms)` at `INFO` level.
///
/// Returns HTTP 403 equivalent (status 403, empty body) if URL host is not in allowlist.
/// Returns HTTP 408 equivalent (status 408) if the request times out.
///
/// ## Allowlist enforcement (AC-7 / VP-PLUGIN-007)
///
/// Host-only `==` comparison (not substring matching): `url::Url::parse` extracts the host
/// component, which is compared against each entry in `allowed_urls`. An empty `allowed_urls`
/// list blocks ALL outbound HTTP from the plugin (default-deny semantics).
///
/// Emits: `event_type = "plugin_http_request_blocked"` (WARN) on blocked requests per
/// BC-2.16.002 v1.12 Canonical Structured Event Catalog row (PG-LP11-001).
pub fn host_http_request(
    state: &HostState,
    method: &str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
) -> HttpResponse {
    // URL allowlist enforcement — host-only == comparison (AC-7 / VP-PLUGIN-007).
    // allowed_urls is Vec<String> (not Option): empty list = default-deny all outbound HTTP.
    // Substring matching (`url.contains(domain)`) is bypassable via query parameters
    // (e.g. `https://evil.com/?ref=allowed.com`). We parse the URL and compare only the
    // normalized host string against each allowlist entry (BC-2.17.002 / INV-PLUGIN-002).
    // MED-007 (F-IMPL-LP1-MED-007): guard against empty allowlist entries.
    // An empty string entry in allowed_urls would match host_str() == "" on unparseable
    // URLs, creating a de-facto allow-all bypass. Empty entries are rejected at manifest
    // parse time (parse_manifest), but we also filter here for defense-in-depth.
    let url_allowed = match Url::parse(url) {
        Ok(parsed) => {
            let url_host = parsed.host_str().unwrap_or("");
            state.allowed_urls.iter().any(|allowed_domain| {
                !allowed_domain.is_empty() && url_host == allowed_domain.as_str()
            })
        }
        Err(_) => false, // unparseable URL is never allowed
    };

    if !url_allowed {
        // Single structured emission per BC-2.16.002 v1.12 catalog row plugin_http_request_blocked.
        // WARN-level log and audit-channel routing are orthogonal via event_type field.
        warn!(
            event_type = "plugin_http_request_blocked",
            plugin_id = %state.plugin_id,
            url = %url,
            reason = "allowlist_mismatch",
            "Plugin HTTP request blocked: URL host not in allowed_urls allowlist"
        );
        return HttpResponse {
            status: 403,
            headers: vec![],
            body: vec![],
        };
    }

    // Make the actual HTTP request via the host's reqwest client.
    let start = Instant::now();

    // Build a blocking runtime for the async reqwest call.
    // We use block_in_place since this is called from a synchronous context.
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        // We're inside a tokio runtime — use block_in_place.
        let result = tokio::task::block_in_place(|| {
            handle.block_on(do_http_request(state, method, url, &headers, body))
        });
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let status = result.status;
        info!(
            plugin_id = %state.plugin_id,
            method = %method,
            url = %url,
            status = status,
            latency_ms = elapsed_ms,
            "Plugin HTTP request audit log"
        );
        return result;
    }

    // Fallback: create new runtime (for tests without tokio context).
    let result = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt.block_on(do_http_request(state, method, url, &headers, body)),
        Err(e) => HttpResponse {
            status: 500,
            headers: vec![],
            body: format!("runtime error: {}", e).into_bytes(),
        },
    };

    let elapsed_ms = start.elapsed().as_millis() as u64;
    info!(
        plugin_id = %state.plugin_id,
        method = %method,
        url = %url,
        status = result.status,
        latency_ms = elapsed_ms,
        "Plugin HTTP request audit log"
    );
    result
}

/// Internal async HTTP request execution.
///
/// Relies on the 30-second timeout configured in the shared `reqwest::Client` at boot
/// (TD-S-PLUGIN-PREREQ-B-005 closure; `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS = 30` in mod.rs).
/// No per-request `.timeout()` override — the Client-level timeout is the source of truth.
async fn do_http_request(
    state: &HostState,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<Vec<u8>>,
) -> HttpResponse {
    use reqwest::Method;
    use std::str::FromStr;

    let method = match Method::from_str(method) {
        Ok(m) => m,
        Err(_) => {
            return HttpResponse {
                status: 400,
                headers: vec![],
                body: b"invalid HTTP method".to_vec(),
            };
        }
    };

    // url was already parsed (and host extracted) during allowlist check above; parse again
    // for reqwest. The `url` crate's `Url` and `reqwest::Url` are the same type (reqwest
    // re-exports it), so this is zero-cost in practice.
    let url_parsed = match reqwest::Url::parse(url) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse {
                status: 400,
                headers: vec![],
                body: b"invalid URL".to_vec(),
            };
        }
    };

    // No per-request .timeout() call: the 30s timeout is enforced at Client::builder() level.
    let mut request_builder = state.http_client.request(method, url_parsed);

    for (key, value) in headers {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(key.as_bytes()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            request_builder = request_builder.header(name, val);
        }
    }

    if let Some(body_bytes) = body {
        request_builder = request_builder.body(body_bytes);
    }

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let resp_headers: Vec<(String, String)> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body = resp.bytes().await.unwrap_or_default().to_vec();
            HttpResponse {
                status,
                headers: resp_headers,
                body,
            }
        }
        Err(e) => {
            if e.is_timeout() {
                HttpResponse {
                    status: 408,
                    headers: vec![],
                    body: b"request timeout".to_vec(),
                }
            } else {
                HttpResponse {
                    status: 500,
                    headers: vec![],
                    body: format!("request error: {}", e).into_bytes(),
                }
            }
        }
    }
}

/// Forward a plugin log message to `tracing` at the appropriate level.
///
/// Prefix format: `"[plugin:{plugin_id}] {message}"`.
pub fn host_log(state: &HostState, level: LogLevel, message: &str) {
    let prefixed = format!("[plugin:{}] {}", state.plugin_id, message);
    match level {
        LogLevel::Trace => trace!("{}", prefixed),
        LogLevel::Debug => debug!("{}", prefixed),
        LogLevel::Info => info!("{}", prefixed),
        LogLevel::Warn => warn!("{}", prefixed),
        LogLevel::Error => error!("{}", prefixed),
    }
}

/// Look up a key in the plugin's config map (`HostState.config`).
///
/// Returns `None` for unknown keys — never errors.
pub fn host_get_config(state: &HostState, key: &str) -> Option<String> {
    state.config.get(key).cloned()
}

/// Get a value from the plugin's KV store.
///
/// KV stores are scoped per plugin: the underlying key is `"{plugin_id}:{key}"`.
/// Returns `None` if the key has not been set.
pub fn host_kv_get(state: &HostState, key: &str) -> Option<String> {
    state.kv_store.get(&state.plugin_id, key)
}

/// Set a value in the plugin's KV store.
///
/// Returns `E-PLUGIN-003` if the 1MB per-plugin KV limit is exceeded.
pub fn host_kv_set(
    state: &HostState,
    key: &str,
    value: &str,
) -> Result<(), prism_core::PluginError> {
    state.kv_store.set(&state.plugin_id, key, value)
}

/// Register all host functions into the `Linker<HostState>`.
///
/// This is called once during `PluginRuntime::build_linker()`. After this call,
/// the linker is ready to pre-instantiate any plugin component that uses only
/// the Prism host interface — including production `.prx` files built with WIT
/// bindings that import from the `"host"` instance namespace.
///
/// Registered functions (all in the `"host"` instance namespace):
/// - `"http-request"` — outbound HTTP via the allowlisted `reqwest::Client` (AC-7)
/// - `"log"` — structured logging forwarded to `tracing` (AC-8)
/// - `"get-config"` — key-value config map lookup (AC-8)
/// - `"kv-get"` — per-plugin persistent KV get (AC-8)
/// - `"kv-set"` — per-plugin persistent KV set (AC-8)
///
/// # Architecture Compliance
/// MUST NOT call any `wasmtime_wasi::add_to_linker_*` function — WASI MUST NOT
/// be added to plugin instances (BC-2.17.002 / VP-040 / INV-PLUGIN-002).
///
/// # WIT Integration Note
/// These registrations use `func_new` (untyped `Val`-based callbacks) which are
/// compatible with both bare WIT bindgen-generated calls and dynamically-typed
/// plugin interfaces. When full WIT bindgen is adopted (S-4.08-manifest-embedding),
/// these may be replaced by typed `func_wrap` bindings generated from the WIT IDL.
pub fn register_host_functions(
    linker: &mut wasmtime::component::Linker<HostState>,
) -> Result<(), prism_core::PrismError> {
    use prism_core::PrismError;
    use wasmtime::StoreContextMut;
    use wasmtime::component::Val;
    use wasmtime::component::types::ComponentFunc;

    let map_err = |e: wasmtime::Error| PrismError::Internal {
        detail: format!("failed to register host function in Linker: {e}"),
    };

    // Create the "host" instance namespace.
    let mut host = linker.instance("host").map_err(map_err)?;

    // ------ host::http-request ------
    // Signature (WIT-compatible):
    //   (method: string, url: string, headers: list<tuple<string,string>>, body: option<list<u8>>)
    //   -> (status: u32, response_headers: list<tuple<string,string>>, body: list<u8>)
    //
    // Note: WIT `u16` maps to Val::U32 in wasmtime's Component Model Val representation.
    // Note: WIT `list<u8>` maps to Val::List(Vec<Val::U8(_)>).
    // Note: WIT `tuple<string,string>` maps to Val::Tuple(vec![Val::String, Val::String]).
    //
    // Delegates to `host_http_request` — the allowlist gate (AC-7 / VP-PLUGIN-007) and
    // 30-second timeout enforcement (AC-9) are exercised through this call path.
    host.func_new(
        "http-request",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            // Deserialize params: (method: string, url: string,
            //   headers: list<tuple<string,string>>, body: option<list<u8>>)
            let method = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => "GET".to_string(),
            };
            let url = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => String::new(),
            };
            // headers: list<tuple<string,string>>
            let headers: Vec<(String, String)> = match params.get(2) {
                Some(Val::List(items)) => items
                    .iter()
                    .filter_map(|item| match item {
                        Val::Tuple(fields)
                            if matches!(
                                (fields.first(), fields.get(1)),
                                (Some(Val::String(_)), Some(Val::String(_)))
                            ) =>
                        {
                            match (fields.first(), fields.get(1)) {
                                (Some(Val::String(k)), Some(Val::String(v))) => {
                                    Some((k.as_str().to_string(), v.as_str().to_string()))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            };
            // body: option<list<u8>>
            let body: Option<Vec<u8>> = match params.get(3) {
                Some(Val::Option(Some(inner))) => {
                    if let Val::List(bytes) = inner.as_ref() {
                        Some(
                            bytes
                                .iter()
                                .filter_map(|b| {
                                    if let Val::U8(byte) = b {
                                        Some(*byte)
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        )
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // Delegate to production allowlist gate + HTTP execution.
            // AC-7: allowlist check runs here, not in a stub.
            // AC-9: 30-second timeout is enforced by the shared reqwest::Client.
            let state = ctx.data();
            let response = host_http_request(state, &method, &url, headers, body);

            // Serialize results: (status: u32, headers: list<tuple<string,string>>, body: list<u8>)
            // Note: WIT u16 → Val::U32 in wasmtime 44 component model Val enum.
            let status_val = Val::U32(u32::from(response.status));
            let headers_val = Val::List(
                response
                    .headers
                    .into_iter()
                    .map(|(k, v)| Val::Tuple(vec![Val::String(k), Val::String(v)]))
                    .collect(),
            );
            let body_val = Val::List(response.body.into_iter().map(Val::U8).collect());

            if results.len() >= 3 {
                results[0] = status_val;
                results[1] = headers_val;
                results[2] = body_val;
            } else if !results.is_empty() {
                // Fallback: if the Component Model result type doesn't match the full
                // tuple (e.g., simplified WIT with different binding), at minimum set
                // the status code so the plugin can check for 403 allowlist rejection.
                results[0] = status_val;
            }

            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::log ------
    // Signature: (level: u8, message: string) -> ()
    //
    // Delegates to `host_log` — the tracing-level dispatch (AC-8) runs through this call path.
    host.func_new(
        "log",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         _results: &mut [Val]| {
            let state = ctx.data();
            // Deserialize level: u8 (mapped to WIT log-level enum ordinal)
            // 0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error
            let level = match params.first() {
                Some(Val::U8(n)) => match n {
                    0 => LogLevel::Trace,
                    1 => LogLevel::Debug,
                    2 => LogLevel::Info,
                    3 => LogLevel::Warn,
                    4 => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                Some(Val::U32(n)) => match n {
                    0 => LogLevel::Trace,
                    1 => LogLevel::Debug,
                    2 => LogLevel::Info,
                    3 => LogLevel::Warn,
                    4 => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                _ => LogLevel::Info,
            };
            let msg = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => "<non-string log message>".to_string(),
            };
            // Delegate to production host_log — tracing-level dispatch runs here.
            host_log(state, level, &msg);
            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::get-config ------
    // Signature: (key: string) -> option<string>
    host.func_new(
        "get-config",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            let state = ctx.data();
            let key = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => String::new(),
            };
            let value = host_get_config(state, &key);
            results[0] = match value {
                Some(v) => Val::Option(Some(Box::new(Val::String(v)))),
                None => Val::Option(None),
            };
            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::kv-get ------
    // Signature: (key: string) -> option<string>
    host.func_new(
        "kv-get",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            let state = ctx.data();
            let key = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => String::new(),
            };
            let value = host_kv_get(state, &key);
            results[0] = match value {
                Some(v) => Val::Option(Some(Box::new(Val::String(v)))),
                None => Val::Option(None),
            };
            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::kv-set ------
    // Signature: (key: string, value: string) -> result<(), string>
    //
    // Delegates to `host_kv_set`. Errors are propagated via Val::Result — NOT silently
    // discarded via `let _ = ...` (fixes F-PASS2-HIGH-003: Standing Rule 3 §2 violation).
    // A plugin that attempts a >1MB write through host::kv-set sees an Err result.
    host.func_new(
        "kv-set",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            let state = ctx.data();
            let key = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => String::new(),
            };
            let value = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                _ => String::new(),
            };
            // Delegate to production host_kv_set. Propagate Err as Val::Result(Err(..)).
            // F-PASS2-HIGH-003: the prior `let _ = host_kv_set(...)` was a Standing Rule 3 §2
            // violation — partial-failure data (KV limit exceeded) must propagate to the plugin.
            match host_kv_set(state, &key, &value) {
                Ok(()) => {
                    if !results.is_empty() {
                        // result<(), string>::Ok(()) — empty ok payload
                        results[0] = Val::Result(Ok(None));
                    }
                }
                Err(e) => {
                    if !results.is_empty() {
                        // result<(), string>::Err(error_message)
                        results[0] = Val::Result(Err(Some(Box::new(Val::String(e.to_string())))));
                    }
                }
            }
            Ok(())
        },
    )
    .map_err(map_err)?;

    Ok(())
}
