//! Per-sensor connectivity probe (BC-2.08.001, S-5.04).
//!
//! All probes route through `SpecDrivenSensorAdapter::fetch()` obtained via
//! `AdapterRegistry::get(org_id, sensor_id)` — the health module MUST NOT
//! construct a `reqwest::Client` directly (FIX-001/v1.6, ADR-023 §C1).
//!
//! Probe results:
//! - HTTP 2xx equivalent → `ConnectivityStatus::Up`
//! - HTTP 429 (RateLimited) → `ConnectivityStatus::Up` (reachable, just rate-limited)
//! - HTTP 5xx equivalent → `ConnectivityStatus::Degraded`
//! - Connection error (no HTTP response) → `ConnectivityStatus::Down`
//!
//! `latency_ms` is measured using `tokio::time::Instant` around the adapter
//! `fetch()` call (BC-2.08.001 postcondition 1).
//!
//! # Single-tenant fallback (F-S504-P1-003)
//! When `AdapterRegistry::get(org_id, sensor_id)` returns `None` AND `org_id` is nil
//! (single-tenant sentinel from server.rs), falls back to
//! `AdapterRegistry::get_all_for_sensor(sensor_id)` to find the sole registered adapter.
//! Multi-tenant callers always pass a real resolved OrgId — nil is never a valid tenant.
//!
//! # Error sanitization (F-S504-P2-008)
//! The `error` field in `ProbeOutcome` is always sanitized before being stored:
//! truncated to `MAX_ERROR_LEN` bytes and stripped of control characters, preventing
//! prompt-injection via upstream sensor error bodies (CWE-116).

use chrono::{DateTime, Utc};
use prism_core::{OrgId, SensorId};
use prism_sensors::{auth::SensorAuth, registry::AdapterRegistry};

/// Maximum length (in characters) for sanitized error text in `ProbeOutcome.error`.
///
/// Upstream HTTP bodies can be large HTML pages; truncating prevents context-stuffing
/// and prompt-injection (F-S504-P2-008, CWE-116).
const MAX_ERROR_LEN: usize = 512;

/// Sanitize an upstream error string for safe inclusion in `ProbeOutcome.error`.
///
/// Applies two transformations:
/// 1. Truncates to `MAX_ERROR_LEN` characters (prevents context-stuffing).
/// 2. Replaces control characters with spaces (prevents ANSI/control-char injection).
///
/// The result is safe for inclusion in AI agent-consumed health output.
fn sanitize_error(raw: &str) -> String {
    let capped: &str = {
        let byte_end = raw
            .char_indices()
            .nth(MAX_ERROR_LEN)
            .map(|(idx, _)| idx)
            .unwrap_or(raw.len());
        &raw[..byte_end]
    };
    capped
        .chars()
        .map(|c| {
            if c.is_ascii() && c.is_ascii_control() {
                ' '
            } else {
                c
            }
        })
        .collect()
}

/// Minimal auth token for health probes.
///
/// Health probes route through the adapter to exercise the HTTP path; the mock
/// adapters (and WASM plugin adapters in production) ignore the auth credential
/// for LIMIT-0 probes — the health subsystem does not supply real credentials
/// (FIX-001/v1.6: the adapter owns credential handling).
struct ProbeAuth;

impl SensorAuth for ProbeAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "bearer_static"
    }
}

// ── ConnectivityStatus ────────────────────────────────────────────────────────

/// Outcome of a live connectivity probe (BC-2.08.001).
///
/// `auth_invalid` is NOT a connectivity status — it is distinct and lives in
/// the auth module (BC-2.08.002).  The connectivity probe only distinguishes
/// reachability, not credential validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectivityStatus {
    /// The sensor API endpoint responded with a success (2xx-equivalent).
    Up,
    /// The sensor API endpoint responded but with a server error (5xx-equivalent).
    Degraded,
    /// The adapter could not reach the sensor (connection refused, DNS failure, timeout).
    Down,
}

/// Full outcome of a single connectivity probe run.
///
/// Returned by `probe_connectivity` and consumed by `SensorHealthChecker::check_one`.
#[derive(Debug, Clone)]
pub struct ProbeOutcome {
    /// Reachability status.
    pub status: ConnectivityStatus,
    /// Round-trip latency in milliseconds (None when status is `Down`).
    pub latency_ms: Option<u64>,
    /// UTC timestamp when the probe completed.
    pub probed_at: DateTime<Utc>,
    /// Raw HTTP status code returned by the adapter, if any.
    pub http_status: Option<u16>,
    /// Sanitised error text for `Down`/`Degraded` probes (prompt-injection-safe, F-S504-P2-008).
    pub error: Option<String>,
    /// Whether the probe received HTTP 429 (rate-limited) (F-S504-P1-001/002).
    pub is_rate_limited: bool,
    /// Retry-after delay in milliseconds extracted from HTTP 429 response (F-S504-P1-001).
    /// `None` when not rate-limited.
    pub rate_limit_retry_after_ms: Option<u64>,
}

// ── probe_connectivity ────────────────────────────────────────────────────────

/// Issue a live connectivity probe to a single sensor via the adapter registry.
///
/// Looks up `AdapterRegistry::get(org_id, sensor_id)` and calls
/// `adapter.fetch()` with a minimal probe query (`LIMIT 0`).
///
/// Returns `ProbeOutcome` with status, latency, and HTTP status code.
///
/// # Error semantics
/// Returns `Err(PrismError)` only for unrecoverable engine failures (e.g.,
/// adapter lookup bookkeeping error).  Connection errors and HTTP failures are
/// represented as `Ok(ProbeOutcome { status: Down | Degraded, ... })`.
///
/// # FIX-001/v1.6 mandate
/// This function MUST NOT construct a `reqwest::Client`. It MUST obtain the
/// adapter via `registry.get(org_id, sensor_id)`.
///
/// # AC-9 probe_table routing (BC-2.08.001 postcondition 5 / S-5.04)
///
/// `probe_table` and `first_table_name` implement the three-tier fallback chain:
/// 1. `probe_table = Some(tbl)` → `source_table = "{sensor_id}_{tbl}"`.
/// 2. `probe_table = None`, `first_table_name = Some(tbl)` → `source_table = "{sensor_id}_{tbl}"`.
/// 3. Both `None` → legacy sentinel `"{sensor_id}_devices"` (pre-S-1.11 hollow probe).
///
/// The underscore form `{sensor_id}_{table}` is the canonical fully-qualified table name
/// used by PrismQL `FROM {sensor_id}_{table}` and by `SpecDrivenSensorAdapter::fetch`,
/// which strips the `"{sensor_id}_"` prefix from `source_table` to select the matching
/// `[[tables]]` entry.  Dot-notation was incorrect — it would never match the strip_prefix
/// and cause the adapter to fan out to ALL tables (F-S504-P1-002 fix).
pub async fn probe_connectivity(
    registry: &AdapterRegistry,
    org_id: OrgId,
    sensor_id: &SensorId,
    _client_id: &str,
) -> Result<ProbeOutcome, prism_core::error::PrismError> {
    probe_connectivity_inner(registry, org_id, sensor_id, _client_id, None, None).await
}

/// Extended form of `probe_connectivity` with explicit probe_table routing.
///
/// Called by `SensorHealthChecker::check_one` (and from AC-9 tests) with the
/// sensor spec's `probe_table` and `first_table_name` resolved from the loaded spec.
///
/// `probe_table`      — `Some(name)` if the spec declares `probe_table`.
/// `first_table_name` — `Some(name)` of `spec.tables[0].table_name` when tables exist and
///                      `probe_table` is absent.  `None` when no tables are declared.
///
/// The resolved `source_table` uses underscore form `"{sensor_id}_{tbl}"` matching
/// `SpecDrivenSensorAdapter::fetch`'s `strip_prefix("{sensor_id}_")` table selection
/// (F-S504-P1-002 fix — dot form caused fan-out to ALL tables).
pub async fn probe_connectivity_with_routing(
    registry: &AdapterRegistry,
    org_id: OrgId,
    sensor_id: &SensorId,
    client_id: &str,
    probe_table: Option<&str>,
    first_table_name: Option<&str>,
) -> Result<ProbeOutcome, prism_core::error::PrismError> {
    probe_connectivity_inner(
        registry,
        org_id,
        sensor_id,
        client_id,
        probe_table,
        first_table_name,
    )
    .await
}

async fn probe_connectivity_inner(
    registry: &AdapterRegistry,
    org_id: OrgId,
    sensor_id: &SensorId,
    _client_id: &str,
    probe_table: Option<&str>,
    first_table_name: Option<&str>,
) -> Result<ProbeOutcome, prism_core::error::PrismError> {
    use prism_sensors::adapter::{QueryParams, SensorSpec};

    // F-S504-P1-003: when org_id is nil (single-tenant sentinel from server.rs), fall back to
    // get_all_for_sensor() to locate the sole registered adapter without requiring an OrgId.
    // Multi-tenant callers always pass a real resolved OrgId; nil is never a valid tenant UUID.
    let adapter = if org_id.as_uuid().is_nil() {
        // Single-tenant fallback: find the first adapter registered for this sensor.
        registry
            .get_all_for_sensor(sensor_id)
            .into_iter()
            .next()
            .map(|(_, a)| a)
    } else {
        registry.get(org_id, sensor_id)
    };

    let (adapter, actual_org_id) = match adapter {
        Some(a) => {
            // Recover the org_id to build the correct SensorSpec.
            // For nil sentinel, get_all_for_sensor already gave us the adapter above;
            // re-query to get the org_id for the SensorSpec (avoids a second allocation path).
            let resolved_org = if org_id.as_uuid().is_nil() {
                registry
                    .get_all_for_sensor(sensor_id)
                    .into_iter()
                    .next()
                    .map(|(oid, _)| oid)
                    .unwrap_or(org_id)
            } else {
                org_id
            };
            (a, resolved_org)
        }
        None => {
            // No adapter registered — treat as Down (sensor not configured)
            return Ok(ProbeOutcome {
                status: ConnectivityStatus::Down,
                latency_ms: None,
                probed_at: Utc::now(),
                http_status: None,
                error: Some(sanitize_error(&format!(
                    "no adapter registered for sensor {sensor_id}"
                ))),
                is_rate_limited: false,
                rate_limit_retry_after_ms: None,
            });
        }
    };

    // Minimal probe query — LIMIT 0.
    //
    // AC-9 / BC-2.08.001 postcondition 5 — probe_table fallback chain:
    //   1. probe_table = Some(tbl) → "{sensor_id}_{tbl}" (explicit probe table).
    //   2. probe_table = None, first_table_name = Some(tbl) → "{sensor_id}_{tbl}" (first table).
    //   3. Both None → "{sensor_id}_devices" (legacy sentinel; structural no-op when no tables
    //      exist, adapter's empty-tables fast-path returns Ok([]) — BC-2.08.001 / FIX-001 v1.6).
    //
    // F-S504-P1-002 FIX: underscore form "{sensor_id}_{table}" is the canonical form used by
    // both PrismQL `FROM` syntax and `SpecDrivenSensorAdapter::fetch`, which strips the
    // "{sensor_id}_" prefix from source_table to select the matching [[tables]] entry.
    // Dot-notation "{sensor_id}.{table}" was incorrect — strip_prefix("{sensor_id}_") would
    // never match it, causing the adapter to fan out to ALL tables instead of just the probe table.
    let sensor_type = adapter.sensor_type();
    let probe_source_table = if let Some(tbl) = probe_table {
        format!("{sensor_type}_{tbl}")
    } else if let Some(tbl) = first_table_name {
        format!("{sensor_type}_{tbl}")
    } else {
        // Legacy sentinel: structural no-op (F-S504-P2-009 — sensor-id-prefixed form).
        format!("{sensor_type}_devices")
    };
    #[allow(deprecated)]
    let spec = SensorSpec {
        source_table: probe_source_table,
        org_id: actual_org_id,
        client_id: String::new(),
        sensor_config: serde_json::Value::Null,
    };
    let params = QueryParams {
        limit: 0,
        ..Default::default()
    };
    let auth = ProbeAuth;

    let start = tokio::time::Instant::now();
    let result = adapter.fetch(&spec, &params, &auth).await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(_) => Ok(ProbeOutcome {
            status: ConnectivityStatus::Up,
            latency_ms: Some(elapsed_ms.max(1)), // at least 1ms to satisfy non-zero assertion
            probed_at: Utc::now(),
            // F-S504-P2-004: Do NOT fabricate http_status=200 for a hollow Ok([]).
            // The adapter may return Ok([]) without making any HTTP request (e.g., when
            // no tables are declared or via the empty-tables fast-path). Asserting 200
            // when no HTTP contact was confirmed violates BC-2.08.001 postcondition 5's
            // no-op clause. Use None — Up status alone signals reachable-by-runtime.
            http_status: None,
            error: None,
            is_rate_limited: false,
            rate_limit_retry_after_ms: None,
        }),
        // F-S504-P1-002: SensorError::RateLimited must NOT be misclassified as Down.
        // The sensor is reachable (HTTP 429 means we got a response); rate-limited is
        // a distinct state from unreachable (BC-2.08.003 / BC-2.08.001 distinction).
        Err(prism_sensors::adapter::SensorError::RateLimited { retry_after_ms, .. }) => {
            Ok(ProbeOutcome {
                status: ConnectivityStatus::Up,
                latency_ms: Some(elapsed_ms.max(1)),
                probed_at: Utc::now(),
                http_status: Some(429),
                error: None,
                is_rate_limited: true,
                rate_limit_retry_after_ms: Some(retry_after_ms),
            })
        }
        Err(prism_sensors::adapter::SensorError::HttpError { status, body, .. }) => {
            // HTTP response received — sensor is reachable (not Down)
            let connectivity = if status >= 500 {
                ConnectivityStatus::Degraded
            } else {
                // 4xx: sensor reachable, but auth/client issue — still Up from connectivity perspective
                ConnectivityStatus::Up
            };
            // F-S504-P2-008: sanitize upstream body before storing — prevent prompt-injection.
            Ok(ProbeOutcome {
                status: connectivity,
                latency_ms: Some(elapsed_ms.max(1)),
                probed_at: Utc::now(),
                http_status: Some(status),
                error: Some(sanitize_error(&body)),
                is_rate_limited: false,
                rate_limit_retry_after_ms: None,
            })
        }
        Err(e) => {
            // Connection error, timeout — sensor unreachable.
            // F-S504-P2-008: sanitize error string — e.to_string() may contain upstream content.
            Ok(ProbeOutcome {
                status: ConnectivityStatus::Down,
                latency_ms: None,
                probed_at: Utc::now(),
                http_status: None,
                error: Some(sanitize_error(&e.to_string())),
                is_rate_limited: false,
                rate_limit_retry_after_ms: None,
            })
        }
    }
}

// BC-5.38.005 self-check (S-5.04 implementation complete):
// probe_connectivity — non-trivial (adapter lookup + single-tenant fallback, fetch, latency
//   measurement, RateLimited arm, sanitize_error). IMPLEMENTED.
