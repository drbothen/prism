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
/// BC-2.16.002 Canonical Structured Event Catalog row (PG-LP11-001).
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
        // Single structured emission per BC-2.16.002 catalog row plugin_http_request_blocked.
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

    // Fallback: create a current-thread runtime (for callers without a tokio context).
    // PR-LOW-003: use current_thread + enable_all (not the multi-thread default from
    // Runtime::new()) to match the expected minimal footprint for a one-shot HTTP call.
    // This code path is only exercised in tests without a tokio runtime context.
    let result = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt.block_on(do_http_request(state, method, url, &headers, body)),
        Err(e) => {
            // SEC-004 (CWE-209): route error detail to debug-level tracing only —
            // do NOT surface tokio runtime build errors (OS detail) to the plugin sandbox.
            debug!(
                plugin_id = %state.plugin_id,
                error = %e,
                "host_http_request: tokio runtime build failed (current-thread fallback)"
            );
            HttpResponse {
                status: 500,
                headers: vec![],
                body: b"runtime error".to_vec(),
            }
        }
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
    use std::str::FromStr;

    use reqwest::Method;

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
            // F6 (2026-06-10 review): header values are bytes per RFC 9110, but the
            // WIT `host` interface carries them as strings — a decode decision is
            // unavoidable. Decision: LOSSY decode (invalid bytes → U+FFFD), not skip
            // and not silent-empty. `v.to_str().unwrap_or("")` silently emptied any
            // non-UTF8 value, hiding the valid portion from the plugin; skipping the
            // header entirely would hide its presence. Lossy decoding preserves the
            // valid prefix/suffix and makes the corruption visible as U+FFFD.
            let resp_headers: Vec<(String, String)> = resp
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        String::from_utf8_lossy(v.as_bytes()).into_owned(),
                    )
                })
                .collect();
            // F6 (2026-06-10 review): a mid-body read failure (connection reset,
            // truncated Content-Length, timeout during streaming) must NOT surface
            // as the origin success status with a silently-empty body — plugins
            // could not distinguish that from a genuine 2xx-with-empty-body. Map it
            // to a synthetic error response exactly like the sibling send-error arm
            // below: 408 for timeouts, 500 with a diagnostic body otherwise.
            match resp.bytes().await {
                Ok(bytes) => HttpResponse {
                    status,
                    headers: resp_headers,
                    body: bytes.to_vec(),
                },
                Err(e) if e.is_timeout() => HttpResponse {
                    status: 408,
                    headers: vec![],
                    body: b"request timeout".to_vec(),
                },
                Err(e) => {
                    // SEC-004 (CWE-209): route reqwest error detail to debug-level
                    // tracing only — do NOT surface host network topology, TLS detail,
                    // or OS error text to the partially-trusted plugin sandbox.
                    debug!(
                        plugin_id = %state.plugin_id,
                        error = %e,
                        "host_http_request: mid-body read failure after successful response status"
                    );
                    HttpResponse {
                        status: 500,
                        headers: vec![],
                        body: b"body read error".to_vec(),
                    }
                }
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
                // SEC-004 (CWE-209): route reqwest send-error detail to debug-level
                // tracing only — URL/topology/TLS detail must not reach the plugin sandbox.
                debug!(
                    plugin_id = %state.plugin_id,
                    error = %e,
                    "host_http_request: send failed (non-timeout)"
                );
                HttpResponse {
                    status: 500,
                    headers: vec![],
                    body: b"request error".to_vec(),
                }
            }
        }
    }
}

/// Forward a plugin log message to `tracing` at the appropriate level.
///
/// Prefix format: `"[plugin:{plugin_id}] {message}"`.
///
/// # AD-017 Security Note
///
/// The `message` string is forwarded **unredacted** from the plugin guest to the host's
/// tracing subscriber. Plugin code is responsible for NOT including credential values in
/// log messages (the WIT interface is a trust boundary, not a redaction boundary).
///
/// Credential values are injected via `host::get-config` (which does NOT log values) and
/// must NEVER be logged by plugin guest code — this is a guest-side discipline enforced
/// by the plugin author's contract, not by the host. Log scrubbing at the tracing subscriber
/// level is the production guard (external to prism-spec-engine; e.g., log pipeline).
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
///
/// ## SEC-008 / AD-017
///
/// `PluginConfigMap` values are `SecretString`. We call `.expose_secret()` here —
/// at the last possible moment before handing the value to the WASM guest — so the
/// plaintext `String` exists only for the duration of this call frame. The returned
/// `String` is owned by the caller (the `register_host_functions` closure) and is
/// dropped after it is copied into the WIT `Val::String` result slot.
pub fn host_get_config(state: &HostState, key: &str) -> Option<String> {
    use secrecy::ExposeSecret;
    state.config.get(key).map(|s| s.expose_secret().to_owned())
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

/// Return the current wall-clock time as Unix seconds (u64).
///
/// Used by sensor-auth plugins for token TTL management — the plugin stores
/// `expires_at = current_time + expires_in - 30` in the KV store and checks
/// this value on subsequent `get-token` calls to detect cache staleness.
///
/// PLUGIN-MIGRATION-001-E Known Gap resolution: this host function was not
/// present in PREREQ-D; added in-scope per story Known Gaps §host_current_time_secs.
///
/// # Monotonicity
///
/// Uses `std::time::SystemTime::UNIX_EPOCH` (wall clock, not monotonic). This is
/// intentional: the TTL calculation is wall-clock-based (expires_in comes from the
/// OAuth2 server which uses wall time), and the 30-second buffer absorbs minor clock
/// skew. Precision loss up to ~1 second is acceptable for TTL checks.
pub fn host_current_time_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Register all host functions under a specific instance namespace.
///
/// Called by `register_host_functions` with:
/// - `"host"` — bare namespace, used by WAT test fixtures
/// - `"prism:crowdstrike-oauth2/host@0.1.0"` — WIT-namespaced interface,
///   used by real Component Model .prx binaries produced by `wasm-tools component new`
///   (S-PLUGIN-CI-001: real component imports are WIT-package-scoped).
///
/// # Val-type correctness (F-PASS3-CRIT-002)
/// - WIT `u16`  → `Val::U16(u16)` (NOT Val::U32)
/// - WIT `enum` (log-level) → `Val::Enum(String)` (NOT Val::U8 or Val::U32)
/// - WIT `-> http-response` (record) → ONE result slot: `Val::Record(Vec<(String, Val)>)`
///   (NOT three scalar slots)
/// - Schema violations (wrong Val variant from plugin) → `Err(wasmtime::Error::msg(...))`
///   (Component Model trap; NOT silent default coercion)
fn register_host_functions_for_namespace(
    linker: &mut wasmtime::component::Linker<HostState>,
    namespace: &str,
) -> Result<(), prism_core::PrismError> {
    use prism_core::PrismError;
    use wasmtime::{
        StoreContextMut,
        component::{Val, types::ComponentFunc},
    };

    let map_err = |e: wasmtime::Error| PrismError::Internal {
        detail: format!("failed to register host function in Linker (namespace={namespace}): {e}"),
    };

    let mut host = linker.instance(namespace).map_err(map_err)?;

    // ------ host::http-request ------
    // WIT signature:
    //   http-request: func(
    //     method: string, url: string,
    //     headers: list<tuple<string, string>>,
    //     body: option<list<u8>>,
    //   ) -> http-response;
    //
    // where http-response is a RECORD:
    //   record http-response { status: u16, headers: list<tuple<string,string>>, body: list<u8> }
    //
    // Component Model result slot layout: 1 slot → Val::Record(Vec<(String, Val)>)
    // WIT u16 → Val::U16(u16)   (NOT Val::U32 — F-PASS3-CRIT-002 Violation A fix)
    // WIT list<u8> → Val::List(Vec<Val::U8(_)>)
    // WIT tuple<string,string> → Val::Tuple(vec![Val::String, Val::String])
    //
    // Delegates to `host_http_request` — the allowlist gate (AC-7 / VP-PLUGIN-007) and
    // 30-second timeout enforcement (AC-9) are exercised through this call path.
    host.func_new(
        "http-request",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            // Deserialize params. Schema violations → trap (F-PASS3-MED-002 / SOUL.md #4).
            // A plugin calling host::http-request with wrong Val types has a WIT binding bug;
            // trapping is the correct production-grade response (surfaces the bug immediately).

            // method: string
            let method = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::http-request: schema violation: expected Val::String for \
                         'method' param; got {other:?}"
                    )));
                }
            };

            // url: string
            let url = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::http-request: schema violation: expected Val::String for \
                         'url' param; got {other:?}"
                    )));
                }
            };

            // headers: list<tuple<string,string>>
            let headers: Vec<(String, String)> = match params.get(2) {
                Some(Val::List(items)) => {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items.iter() {
                        match item {
                            Val::Tuple(fields)
                                if matches!(
                                    (fields.first(), fields.get(1)),
                                    (Some(Val::String(_)), Some(Val::String(_)))
                                ) =>
                            {
                                if let (Some(Val::String(k)), Some(Val::String(v))) =
                                    (fields.first(), fields.get(1))
                                {
                                    out.push((k.as_str().to_string(), v.as_str().to_string()));
                                }
                            }
                            other => {
                                return Err(wasmtime::Error::msg(format!(
                                    "host::http-request: schema violation: expected \
                                     Val::Tuple([String, String]) for 'headers' list entry; \
                                     got {other:?}"
                                )));
                            }
                        }
                    }
                    out
                }
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::http-request: schema violation: expected Val::List for \
                         'headers' param; got {other:?}"
                    )));
                }
            };

            // body: option<list<u8>>
            let body: Option<Vec<u8>> = match params.get(3) {
                Some(Val::Option(Some(inner))) => match inner.as_ref() {
                    Val::List(bytes) => {
                        let mut out = Vec::with_capacity(bytes.len());
                        for b in bytes.iter() {
                            match b {
                                Val::U8(byte) => out.push(*byte),
                                other => {
                                    return Err(wasmtime::Error::msg(format!(
                                        "host::http-request: schema violation: expected \
                                         Val::U8 in body list; got {other:?}"
                                    )));
                                }
                            }
                        }
                        Some(out)
                    }
                    other => {
                        return Err(wasmtime::Error::msg(format!(
                            "host::http-request: schema violation: expected Val::List \
                             inside body option; got {other:?}"
                        )));
                    }
                },
                Some(Val::Option(None)) => None,
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::http-request: schema violation: expected Val::Option for \
                         'body' param; got {other:?}"
                    )));
                }
            };

            // Delegate to production allowlist gate + HTTP execution.
            // AC-7: allowlist check runs here, not in a stub.
            // AC-9: 30-second timeout is enforced by the shared reqwest::Client.
            let state = ctx.data();
            let response = host_http_request(state, &method, &url, headers, body);

            // Serialize result: ONE slot — Val::Record matching WIT http-response record.
            // F-PASS3-CRIT-002 Violation A: WIT u16 → Val::U16(u16), not Val::U32.
            // F-PASS3-CRIT-002 Violation C: single Val::Record slot, not 3 scalar slots.
            let status_val = Val::U16(response.status);
            let headers_val = Val::List(
                response
                    .headers
                    .into_iter()
                    .map(|(k, v)| Val::Tuple(vec![Val::String(k), Val::String(v)]))
                    .collect(),
            );
            let body_val = Val::List(response.body.into_iter().map(Val::U8).collect());

            // WIT record http-response has 3 named fields; Component Model gives 1 result slot.
            results[0] = Val::Record(vec![
                ("status".to_string(), status_val),
                ("headers".to_string(), headers_val),
                ("body".to_string(), body_val),
            ]);

            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::log ------
    // WIT signature: log: func(level: log-level, message: string);
    //
    // WIT enum log-level (trace|debug|info|warn|error) → Val::Enum(String).
    // F-PASS3-CRIT-002 Violation B: WIT enum → Val::Enum(String) (NOT Val::U8 or Val::U32).
    // F-PASS3-HIGH-001: unrecognized enum string → emit `plugin_log_level_unrecognized`
    //   (BC-2.16.002 row 32) with explicit observability, then safe-default to Info.
    // F-PASS3-MED-002: non-Enum level param → trap (schema violation).
    //
    // Delegates to `host_log` — the tracing-level dispatch (AC-8) runs through this call path.
    host.func_new(
        "log",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         _results: &mut [Val]| {
            let state = ctx.data();

            // level: log-level enum → Val::Enum(String)
            let level = match params.first() {
                Some(Val::Enum(name)) => match name.as_str() {
                    "trace" => LogLevel::Trace,
                    "debug" => LogLevel::Debug,
                    "info" => LogLevel::Info,
                    "warn" => LogLevel::Warn,
                    "error" => LogLevel::Error,
                    unrecognized => {
                        // Emit observable warning per BC-2.16.002 row 32
                        // (plugin_log_level_unrecognized) then safe-default to Info.
                        // Unrecognized enum name means plugin uses a future log-level variant;
                        // trapping here would break forward-compat; emitting then defaulting
                        // preserves observability without losing the message.
                        warn!(
                            event_type = "plugin_log_level_unrecognized",
                            plugin_id = %state.plugin_id,
                            received_name = %unrecognized,
                            "Plugin sent unrecognized log-level enum value; defaulting to Info"
                        );
                        LogLevel::Info
                    }
                },
                other => {
                    // Schema violation: level param must be Val::Enum for WIT enum log-level.
                    // Trap so the plugin WIT binding bug is surfaced immediately.
                    return Err(wasmtime::Error::msg(format!(
                        "host::log: schema violation: expected Val::Enum for 'level' param; \
                         got {other:?}"
                    )));
                }
            };

            // message: string
            let msg = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::log: schema violation: expected Val::String for 'message' param; \
                         got {other:?}"
                    )));
                }
            };

            // Delegate to production host_log — tracing-level dispatch runs here.
            host_log(state, level, &msg);
            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::get-config ------
    // WIT signature: get-config: func(key: string) -> option<string>;
    host.func_new(
        "get-config",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            let state = ctx.data();
            let key = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::get-config: schema violation: expected Val::String for \
                         'key' param; got {other:?}"
                    )));
                }
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
    // WIT signature: kv-get: func(key: string) -> option<string>;
    host.func_new(
        "kv-get",
        |ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         params: &[Val],
         results: &mut [Val]| {
            let state = ctx.data();
            let key = match params.first() {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::kv-get: schema violation: expected Val::String for \
                         'key' param; got {other:?}"
                    )));
                }
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
    // WIT signature: kv-set: func(key: string, value: string) -> result<_, string>;
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
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::kv-set: schema violation: expected Val::String for \
                         'key' param; got {other:?}"
                    )));
                }
            };
            let value = match params.get(1) {
                Some(Val::String(s)) => s.as_str().to_string(),
                other => {
                    return Err(wasmtime::Error::msg(format!(
                        "host::kv-set: schema violation: expected Val::String for \
                         'value' param; got {other:?}"
                    )));
                }
            };
            // Delegate to production host_kv_set. Propagate Err as Val::Result(Err(..)).
            // F-PASS2-HIGH-003: the prior `let _ = host_kv_set(...)` was a Standing Rule 3 §2
            // violation — partial-failure data (KV limit exceeded) must propagate to the plugin.
            match host_kv_set(state, &key, &value) {
                Ok(()) => {
                    if !results.is_empty() {
                        // result<_, string>::Ok(()) — empty ok payload
                        results[0] = Val::Result(Ok(None));
                    }
                }
                Err(e) => {
                    if !results.is_empty() {
                        // result<_, string>::Err(error_message)
                        results[0] = Val::Result(Err(Some(Box::new(Val::String(e.to_string())))));
                    }
                }
            }
            Ok(())
        },
    )
    .map_err(map_err)?;

    // ------ host::current-time-secs ------
    // WIT signature: current-time-secs: func() -> u64;
    //
    // Returns current Unix timestamp as u64 (wall-clock seconds since epoch).
    // Used by sensor-auth plugins for token TTL management (PLUGIN-MIGRATION-001-E).
    // PLUGIN-MIGRATION-001-E Known Gap: this function did not exist in PREREQ-D;
    // added in-scope per story Known Gaps §host_current_time_secs.
    host.func_new(
        "current-time-secs",
        |_ctx: StoreContextMut<'_, HostState>,
         _func_type: ComponentFunc,
         _params: &[Val],
         results: &mut [Val]| {
            let now_secs = host_current_time_secs();
            // WIT u64 → Val::U64(u64)
            if !results.is_empty() {
                results[0] = Val::U64(now_secs);
            }
            Ok(())
        },
    )
    .map_err(map_err)?;

    Ok(())
}

/// Register all Prism host functions with the `Linker<HostState>`.
///
/// Registers only under the bare `"host"` namespace — used by WAT test fixtures and
/// the legacy host-side test path. Production `.prx` Component Model binaries (produced
/// by `wasm-tools component new --adapt wasi_snapshot_preview1`) import the WIT-package-
/// scoped namespace (`"prism:crowdstrike-oauth2/host@0.1.0"`); those are registered
/// by `register_host_functions_for_component` which also adds WASI trap stubs.
///
/// # Architecture Compliance
/// MUST NOT call any `wasmtime_wasi::add_to_linker_*` function — WASI MUST NOT
/// be added to plugin instances (BC-2.17.002 / VP-040 / INV-PLUGIN-002).
pub fn register_host_functions(
    linker: &mut wasmtime::component::Linker<HostState>,
) -> Result<(), prism_core::PrismError> {
    register_host_functions_for_namespace(linker, "host")
}

/// Build a per-component linker that satisfies a real Component Model `.prx` import set.
///
/// Real `.prx` files produced by `cargo build --target wasm32-wasip1` + `wasm-tools
/// component new --adapt wasi_snapshot_preview1` import TWO interface families:
/// 1. `"prism:crowdstrike-oauth2/host@0.1.0"` — Prism host functions (production I/O)
/// 2. Various `"wasi:*"` interfaces — from the WASI reactor adapter wrapping Rust std
///
/// This function clones `base_linker` (which has bare `"host"` functions registered),
/// then:
/// 1. Registers Prism host functions under the WIT-namespaced interface name.
/// 2. Calls `define_unknown_imports_as_traps(component)` to satisfy remaining WASI
///    imports with trap stubs (BC-2.17.002: no real WASI access, trapping = deny).
///
/// S-PLUGIN-CI-001 AC-001: enables `test_PLUGIN_MIGRATION_001_E_med_001` to pass.
pub(crate) fn build_component_linker(
    base_linker: &wasmtime::component::Linker<HostState>,
    component: &wasmtime::component::Component,
    wit_namespace: &str,
) -> Result<wasmtime::component::Linker<HostState>, prism_core::PrismError> {
    // Strategy (S-PLUGIN-CI-001 AC-001):
    // 1. Call define_unknown_imports_as_traps on the clone to satisfy ALL unresolved
    //    imports (WASI + the WIT-namespaced host interface) with trap stubs.
    //    This is done BEFORE registering the real Prism host functions.
    // 2. Enable allow_shadowing so that the subsequent register call can overwrite
    //    the trap stub for wit_namespace with the real implementations.
    //    (Wasmtime's Linker::instance returns an error on duplicate name unless
    //    allow_shadowing is true.)
    // 3. Register the real Prism host functions under wit_namespace — overwrites trap stub.
    //
    // BC-2.17.002: WASI stubs trap on call — no real filesystem/network access exposed.
    // The crowdstrike plugin may call WASI for Rust std init (environ_get, clock_time_get,
    // etc.) — those will trap. The auth path (acquire-token) only needs host::http-request,
    // host::kv-set, host::get-config which ARE real implementations registered in step 3.
    let mut linker = base_linker.clone();
    // Step 1: Add trap stubs for all unresolved imports (WASI + wit_namespace).
    linker
        .define_unknown_imports_as_traps(component)
        .map_err(|e| prism_core::PrismError::Internal {
            detail: format!(
                "failed to define WASI trap stubs for component ({wit_namespace}): {e}"
            ),
        })?;
    // Step 2: Enable shadowing so we can overwrite the trap stub for wit_namespace.
    linker.allow_shadowing(true);
    // Step 3: Register real Prism host functions under wit_namespace (overwrites trap stub).
    register_host_functions_for_namespace(&mut linker, wit_namespace)?;
    // Restore: disable shadowing after the overwrite to prevent accidental re-registration.
    linker.allow_shadowing(false);
    Ok(linker)
}
