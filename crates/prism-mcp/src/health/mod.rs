//! Sensor health subsystem — live-probe phase (S-5.04).
//!
//! Provides `SensorHealthChecker`, which orchestrates per-sensor connectivity
//! and auth probes via `AdapterRegistry::get(org_id, sensor_id)`, rate-limit
//! state tracking, last-successful-query timestamp tracking, and partial-health
//! aggregation (BC-2.08.001 through BC-2.08.007).
//!
//! # Module layout
//! - `connectivity` — per-sensor connectivity probe (BC-2.08.001)
//! - `auth`         — auth validity detection (BC-2.08.002)
//! - `rate_limit`   — rate-limit state from HTTP 429 headers (BC-2.08.003)
//! - `timestamp`    — last-successful-query tracking (BC-2.08.004)
//!
//! # Key invariant (FIX-001/v1.6)
//! The health module MUST NOT construct a `reqwest::Client` directly.
//! All probes route through `SpecDrivenSensorAdapter::fetch()` obtained via
//! `AdapterRegistry::get(org_id, sensor_id)`.

pub mod auth;
pub mod connectivity;
pub mod rate_limit;
pub mod timestamp;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_sensors::registry::AdapterRegistry;
use prism_spec_engine::{ResolvedSensorSpec, ResolvedSpecKey};

use crate::context::{PrismContext, SensorKey};
use crate::health::connectivity::ConnectivityStatus;
use crate::health::{
    auth::{probe_auth_with_routing, AuthStatus},
    connectivity::ProbeOutcome,
    rate_limit::extract_rate_limit_state,
    rate_limit::RateLimitState,
};
use crate::resources::{RateLimitInfo, SensorHealthResult};

// ── HealthCheckResult ─────────────────────────────────────────────────────────

/// Aggregate result of a `check_sensor_health` invocation (S-5.04 scope).
///
/// Produced by `HealthCheckResult::aggregate(results)` from a batch of
/// per-sensor `SensorHealthResult` values.  The overall status determines
/// the prose summary phrasing in the MCP tool response (BC-2.08.007).
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Per-sensor health entries (one per sensor probed).
    pub sensors: Vec<SensorHealthResult>,
    /// Aggregate status across all sensors.
    pub overall: OverallStatus,
    /// UTC timestamp when the health check ran.
    pub checked_at: DateTime<Utc>,
}

/// Aggregate sensor health status across a client (BC-2.08.007).
///
/// - `Healthy` — all sensors are reachable and authenticated, none rate-limited.
/// - `Partial` — some sensors up, some down or degraded.
///   This is a SUCCESS state, not an error (BC-2.08.007 postcondition 1).
/// - `RateLimited` — ALL sensors are rate-limited and none are unreachable/auth-invalid
///   (EC-08-015 / BC-2.08.007 classification table).  This is a separate
///   status from `Partial` to give AI consumers an unambiguous signal that waiting
///   (not retrying immediately) is the correct remediation.
/// - `Unhealthy` — all sensors are unreachable or auth-invalid.
///   Returned as a success response, not an error (BC-2.08.007 postcondition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverallStatus {
    /// All sensors are reachable and authenticated, not rate-limited.
    Healthy,
    /// Some sensors are up, others are down or degraded.
    Partial,
    /// ALL sensors are rate-limited; none unreachable or auth-invalid (EC-08-015).
    RateLimited,
    /// All sensors are unreachable or auth-invalid.
    Unhealthy,
}

impl OverallStatus {
    /// Serialize to the canonical BC-2.08.007 wire string.
    ///
    /// - `"healthy"` — `Healthy`
    /// - `"partial"` — `Partial`
    /// - `"rate_limited"` — `RateLimited` (EC-08-015)
    /// - `"unhealthy"` — `Unhealthy`
    pub fn as_status_str(&self) -> &'static str {
        match self {
            OverallStatus::Healthy => "healthy",
            OverallStatus::Partial => "partial",
            OverallStatus::RateLimited => "rate_limited",
            OverallStatus::Unhealthy => "unhealthy",
        }
    }
}

impl HealthCheckResult {
    /// Aggregate a batch of `SensorHealthResult` values into an `OverallStatus`.
    ///
    /// Rules (BC-2.08.007 classification table):
    /// - All `reachable: true, auth_valid: true, rate_limit: None, error: None` → `Healthy`
    /// - ALL sensors have `rate_limit: Some(...)` and none have `reachable: false`
    ///   or `auth_valid: false` → `RateLimited` (EC-08-015)
    /// - At least one healthy and at least one not → `Partial`
    /// - None healthy → `Unhealthy`
    /// - `Partial` is a SUCCESS state — callers MUST NOT return an error for it.
    pub fn aggregate(results: Vec<SensorHealthResult>) -> OverallStatus {
        if results.is_empty() {
            return OverallStatus::Unhealthy;
        }

        // EC-08-015: ALL sensors rate-limited, none unreachable/auth-invalid → RateLimited
        let all_rate_limited = results.iter().all(|r| r.rate_limit.is_some());
        let any_unreachable_or_auth_invalid = results
            .iter()
            .any(|r| r.reachable == Some(false) || r.auth_valid == Some(false));
        if all_rate_limited && !any_unreachable_or_auth_invalid {
            return OverallStatus::RateLimited;
        }

        // Standard healthy/partial/unhealthy classification.
        //
        // `is_fully_healthy()` is the AUTHORITATIVE single-source predicate (T-REFACTOR-1):
        // reachable=true AND auth_valid=true AND rate_limit.is_none() AND error.is_none().
        // See `SensorHealthResult::is_fully_healthy` in resources.rs for the full rationale
        // (BC-2.08.002 EC-08-009 / HS-007 / DEFECT-ADAPTER-TLS-XDOME-LIVE-001).
        //
        // A sensor is "partially available" (contributes to Partial, not Unhealthy) when:
        //   - it is reachable (connectivity Up or Degraded, i.e. reachable != Some(false)), AND
        //   - it is not auth-invalid (auth_valid != Some(false))
        //
        // BC-2.08.007 postcondition:
        //   "partial"   = at least one sensor is unreachable OR auth-invalid (but not ALL)
        //   "unhealthy" = ALL sensors are unreachable OR auth-invalid (no rate-limited sensor present)
        //
        // Treating auth-invalid sensors as NOT partially available is the key fix for
        // F-S504-LP3-HIGH-001: a fleet of all-401 sensors (reachable=true, auth_valid=false)
        // must classify as Unhealthy, not Partial. Rate-limited sensors (reachable=true,
        // auth_valid=true, rate_limit=Some) ARE partially available — they are reachable
        // and authenticated; the EC-08-015 all-rate-limited guard fires first for that case.
        let fully_healthy_count = results.iter().filter(|r| r.is_fully_healthy()).count();

        // A sensor is "partially available" if reachable AND not auth-invalid.
        // auth-invalid sensors (reachable=true, auth_valid=false) are NOT partially available —
        // they count toward Unhealthy, not Partial (BC-2.08.007 F-S504-LP3-HIGH-001 fix).
        let any_partially_available = results
            .iter()
            .any(|r| r.reachable != Some(false) && r.auth_valid != Some(false));

        if fully_healthy_count == results.len() {
            OverallStatus::Healthy
        } else if fully_healthy_count > 0 || any_partially_available {
            OverallStatus::Partial
        } else {
            OverallStatus::Unhealthy
        }
    }
}

// ── SensorHealthChecker ───────────────────────────────────────────────────────

/// Orchestrates live-probe sensor health checks (S-5.04 scope, BC-2.08.001–007).
///
/// Held in `PrismContext` (one instance per server) so timestamp and rate-limit
/// state is shared between `check_sensor_health` tool calls and the query engine's
/// success tracking.
///
/// # Probe path (FIX-001/v1.6 — canonical, non-waivable)
/// All probes call `AdapterRegistry::get(org_id, sensor_id)` and invoke
/// `adapter.fetch()` with a minimal probe query (e.g., `LIMIT 0`). The health
/// module MUST NOT construct a `reqwest::Client` directly.
///
/// # probe_table routing (F-S504-P1-001 fix)
/// When `resolved_spec_map` is wired (production path), `check_one` looks up
/// the sensor's `ResolvedSensorSpec` and extracts `probe_table` + first table name
/// to call `probe_auth_with_routing`. Without the spec map the probe falls back to
/// the legacy `{sensor_id}_devices` sentinel (hollow probe, AC-9 no-op path).
#[derive(Debug, Clone)]
pub struct SensorHealthChecker {
    /// Shared adapter registry for live probe dispatch (FIX-001 / ADR-023 §C1).
    adapter_registry: Arc<AdapterRegistry>,
    /// Resolved sensor spec map for probe_table routing (F-S504-P1-001).
    ///
    /// `Some` in production (wired at boot via `SensorHealthChecker::new_with_spec_map`).
    /// `None` in tests that only supply an `AdapterRegistry` without a spec catalog.
    /// When `None`, `check_one` falls back to the legacy `{sensor_id}_devices` sentinel.
    resolved_spec_map: Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>,
}

impl SensorHealthChecker {
    /// Construct a `SensorHealthChecker` with the provided adapter registry (no spec map).
    ///
    /// Backward-compatible constructor for tests and MVP mode. Probes use the legacy
    /// `{sensor_id}_devices` sentinel (hollow, returns Up without HTTP contact for sensors
    /// with no declared tables). Use `new_with_spec_map` for production wiring.
    pub fn new(adapter_registry: Arc<AdapterRegistry>) -> Self {
        Self {
            adapter_registry,
            resolved_spec_map: None,
        }
    }

    /// Construct a `SensorHealthChecker` wired with both adapter registry and resolved
    /// sensor spec map (production path — F-S504-P1-001 fix).
    ///
    /// With the spec map wired, `check_one` resolves `probe_table` and first table name
    /// from the sensor's `ResolvedSensorSpec` before calling `probe_auth_with_routing`.
    /// This ensures the probe routes to the correct single table (BC-2.08.001 postcondition 5).
    ///
    /// Called from `PrismServer::with_deps` at boot step 9.
    pub fn new_with_spec_map(
        adapter_registry: Arc<AdapterRegistry>,
        resolved_spec_map: Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
    ) -> Self {
        Self {
            adapter_registry,
            resolved_spec_map: Some(resolved_spec_map),
        }
    }

    /// Run a live health probe for all sensors registered for `org_id`.
    ///
    /// For each sensor, calls `AdapterRegistry::get(org_id, sensor_id)` and issues
    /// a minimal probe query via `adapter.fetch()`.  Populates `SensorHealthResult`
    /// with `probe_level: "live"`, `reachable`, `auth_valid`, and
    /// `last_successful_query_at` fields (BC-2.08.005 live-probe postcondition).
    ///
    /// Returns a `HealthCheckResult` containing per-sensor results and the aggregate
    /// `OverallStatus`.  The caller writes results to the health cache.
    ///
    /// # Error handling
    /// Partial failure is normal — if sensor A is down but sensor B is up, the
    /// overall result is `OverallStatus::Partial` (a success, not an error).
    /// Only truly unrecoverable engine errors return `Err`.
    pub async fn check_all(
        &self,
        org_id: OrgId,
        client_id: &str,
        sensor_ids: &[SensorId],
        context: &PrismContext,
    ) -> Result<HealthCheckResult, prism_core::error::PrismError> {
        let mut sensors = Vec::with_capacity(sensor_ids.len());
        for sensor_id in sensor_ids {
            let result = self.check_one(org_id, client_id, sensor_id, context).await;
            sensors.push(result);
        }
        let overall = HealthCheckResult::aggregate(sensors.clone());
        Ok(HealthCheckResult {
            sensors,
            overall,
            checked_at: chrono::Utc::now(),
        })
    }

    /// Run a live health probe for a single sensor.
    ///
    /// When `resolved_spec_map` is wired (production path), resolves the sensor's
    /// `probe_table` and first table name from the spec catalog and delegates to
    /// `auth::probe_auth_with_routing` (F-S504-P1-001 fix — routes to the correct
    /// single table, not the legacy `{sensor_id}_devices` sentinel).
    ///
    /// When `resolved_spec_map` is `None` (test / legacy path), falls back to
    /// `auth::probe_auth` using the legacy sentinel.
    ///
    /// Updates the last-successful-query timestamp in `context` on success
    /// (BC-2.08.004 postcondition 1).
    pub async fn check_one(
        &self,
        org_id: OrgId,
        client_id: &str,
        sensor_id: &SensorId,
        context: &PrismContext,
    ) -> SensorHealthResult {
        // F-S504-P1-001: resolve probe_table and first_table_name from the sensor spec
        // when the resolved_spec_map is wired (production path).
        // OrgSlug is derived from client_id (string passed from server.rs call site).
        let probe_table_owned: Option<String>;
        let first_table_name_owned: Option<String>;
        if let Some(ref spec_map) = self.resolved_spec_map {
            let org_slug = OrgSlug::new(client_id);
            let spec_key = (org_slug, sensor_id.clone());
            if let Some(resolved) = spec_map.get(&spec_key) {
                probe_table_owned = resolved.spec.probe_table.clone();
                first_table_name_owned = resolved.spec.tables.first().map(|t| t.table_name.clone());
            } else {
                probe_table_owned = None;
                first_table_name_owned = None;
            }
        } else {
            probe_table_owned = None;
            first_table_name_owned = None;
        }
        let probe_table_ref: Option<&str> = probe_table_owned.as_deref();
        let first_table_ref: Option<&str> = first_table_name_owned.as_deref();

        match auth::probe_auth_with_routing(
            &self.adapter_registry,
            org_id,
            sensor_id,
            client_id,
            probe_table_ref,
            first_table_ref,
        )
        .await
        {
            Ok(probe) => {
                // BC-2.08.002 EC-08-009 (HS-007 / DEFECT-ADAPTER-TLS-XDOME-LIVE-001):
                // `reachable` reflects whether the sensor endpoint returned ANY HTTP response
                // (network-level reachability), NOT whether the response was healthy.
                // Only `Down` (connection refused / timeout — no HTTP exchange at all) means the
                // sensor is unreachable (`reachable: false`).
                // `Degraded` (5xx response received) means the sensor IS network-reachable but
                // erroring — it set `reachable: true` per EC-08-009, distinguishable from Down.
                // The `error` field is set to "service_unavailable" for Degraded by the branch
                // below, preventing false-positive fully-healthy classification via the
                // `r.error.is_none()` gate in `HealthCheckResult::aggregate` (HS-007 edit 2).
                let reachable = probe.connectivity != ConnectivityStatus::Down;
                // BC-2.08.002 EC-08-005 / F-S504-LP1P3-MED-001: auth_valid is THREE-VALUED:
                //   AuthStatus::Valid   → Some(true)   — credentials accepted
                //   AuthStatus::Invalid → Some(false)  — HTTP 401/403 received
                //   AuthStatus::Unknown → None         — sensor unreachable, auth was NEVER attempted
                // A Down sensor (connection refused/timeout) MUST NOT set auth_valid=Some(false);
                // that would conflate "unreachable" with "auth failure", misdirecting the suggestion
                // ladder to "Check credentials" instead of "verify network".
                let auth_valid_opt: Option<bool> = match probe.auth {
                    AuthStatus::Valid => Some(true),
                    AuthStatus::Invalid => Some(false),
                    AuthStatus::Unknown => None,
                };

                // F-S504-P1-001: when HTTP 429 observed, extract rate-limit state and persist
                // to context.rate_limit_states (BC-2.08.003 postcondition).
                let rate_limit_info = if probe.is_rate_limited {
                    // Convert retry_after_ms from the adapter to a Retry-After header string
                    // for extract_rate_limit_state — use delta-seconds form.
                    let retry_after_secs = probe
                        .rate_limit_retry_after_ms
                        .map(|ms| (ms / 1000).max(1))
                        .unwrap_or(crate::health::rate_limit::DEFAULT_RETRY_AFTER_SECS);
                    let header_str = retry_after_secs.to_string();
                    let state = extract_rate_limit_state(Some(header_str.as_str()));
                    // Persist to context for subsequent queries to read (BC-2.08.003 postcondition)
                    let key = SensorKey {
                        client_id: client_id.to_owned(),
                        sensor_id: sensor_id.as_ref().to_owned(),
                    };
                    let mut guard = match context.rate_limit_states.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    let reset_at = state.retry_after;
                    guard.insert(key, state);
                    // Populate SensorHealthResult.rate_limit from the observed state
                    Some(RateLimitInfo {
                        remaining: None,
                        limit: None,
                        reset_at,
                    })
                } else {
                    // Clear stale rate-limit state if it has expired (auto-expiry BC-2.08.003)
                    let key = SensorKey {
                        client_id: client_id.to_owned(),
                        sensor_id: sensor_id.as_ref().to_owned(),
                    };
                    let mut guard = match context.rate_limit_states.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    if let Some(state) = guard.get(&key) {
                        if state.is_cleared() {
                            guard.remove(&key);
                        }
                    }
                    None
                };

                // Record successful query timestamp only when FULLY successful:
                // reachable (any HTTP response), auth valid, not rate-limited, AND
                // connectivity is Up (not Degraded/5xx).
                //
                // HS-007 edit 3 (companion to edit 1): the `probe.connectivity == Up`
                // guard is REQUIRED because edit 1 changed `reachable` to `!= Down`,
                // meaning Degraded (5xx) probes now have `reachable = true`. Without
                // this guard, a 503 sensor (reachable=true, auth_valid=Some(true),
                // !is_rate_limited) would falsely record a `last_successful_query_at`
                // timestamp, making it appear the sensor last answered successfully at
                // the 503 probe time. Only `Up` (2xx/non-auth-error) constitutes a
                // genuinely successful query (BC-2.08.002 EC-08-009).
                let last_successful_query_at = if reachable
                    && auth_valid_opt == Some(true)
                    && !probe.is_rate_limited
                    && probe.connectivity == ConnectivityStatus::Up
                {
                    let now = chrono::Utc::now();
                    self.record_successful_query(client_id, sensor_id, now, context);
                    Some(now)
                } else {
                    self.last_successful_query(client_id, sensor_id, context)
                };

                let mut result = SensorHealthResult::new(sensor_id.as_ref(), client_id)
                    .with_reachable(reachable);
                // Only set auth_valid when we actually know it — Down sensors (auth_valid_opt=None)
                // leave auth_valid as null (BC-2.08.002 EC-08-005: "sensor_unreachable_cannot_verify").
                if let Some(av) = auth_valid_opt {
                    result = result.with_auth_valid(av);
                }
                result.probe_level = "live".to_string();
                result.rate_limit = rate_limit_info;
                // BC-2.08.001 EC-08-001 (F-S504-LP1P1-MED-001): when the probe returned a 5xx
                // (ConnectivityStatus::Degraded), set reason="service_unavailable" per the BC
                // canonical contract.  This is the authoritative string — it is not the raw
                // upstream body (which may be a large HTML page); the BC specifies the reason
                // string for this edge case explicitly.
                //
                // BC-2.08.002 EC-08-005: when the sensor is unreachable (ConnectivityStatus::Down),
                // set reason="sensor_unreachable_cannot_verify".  This is the BC-2.08.002 canonical
                // string for "auth could not be verified because the sensor was unreachable".
                // It pairs with auth_valid=None (set above) to give AI consumers an unambiguous
                // signal that the sensor was not reachable — NOT that credentials were rejected.
                if probe.connectivity == ConnectivityStatus::Degraded {
                    result = result.with_error("service_unavailable");
                } else if probe.connectivity == ConnectivityStatus::Down {
                    result = result.with_error("sensor_unreachable_cannot_verify");
                }
                if let Some(ts) = last_successful_query_at {
                    result = result.with_last_successful_query_at(ts);
                }
                // F-S504-P1-MED-001 (AC-1): surface probe latency on the consumer-facing result.
                // latency_ms is Some(ms) when connectivity is Up (sensor responded).
                // It is None when connectivity is Down (no HTTP exchange, no meaningful latency).
                result.latency_ms = probe.latency_ms;
                result
            }
            Err(_) => {
                // Engine error — sensor unreachable; auth was never attempted (BC-2.08.002 EC-08-005).
                // MUST NOT set auth_valid=Some(false): that conflates "unreachable" with auth failure.
                // auth_valid remains None; reason="sensor_unreachable_cannot_verify" (BC-2.08.002 EC-08-005).
                let mut result = SensorHealthResult::new(sensor_id.as_ref(), client_id)
                    .with_reachable(false)
                    .with_error("sensor_unreachable_cannot_verify");
                result.probe_level = "live".to_string();
                result
            }
        }
    }

    /// Update the last-successful-query timestamp for a (client_id, sensor_id) pair.
    ///
    /// Called by the query engine on every successful sensor fetch (not just health
    /// checks) so that `last_successful_query_at` reflects real query activity
    /// (BC-2.08.004 postcondition 1).
    pub fn record_successful_query(
        &self,
        client_id: &str,
        sensor_id: &SensorId,
        at: DateTime<Utc>,
        context: &PrismContext,
    ) {
        timestamp::write_timestamp(client_id, sensor_id.as_ref(), at, context);
    }

    /// Returns the last successful query timestamp for (client_id, sensor_id).
    ///
    /// Checks the in-memory timestamp map first; falls back to the RocksDB-persisted
    /// value if the in-memory map has no entry (BC-2.08.004 postcondition 2 — survives restart).
    pub fn last_successful_query(
        &self,
        client_id: &str,
        sensor_id: &SensorId,
        context: &PrismContext,
    ) -> Option<DateTime<Utc>> {
        timestamp::read_timestamp(client_id, sensor_id.as_ref(), context)
    }
}

// BC-5.38.005 self-check (S-5.04 implementation complete):
// aggregate — non-trivial (empty list + count logic). IMPLEMENTED.
// check_all / check_one — non-trivial (async probe delegation). IMPLEMENTED.
// record_successful_query / last_successful_query — non-trivial (context delegation). IMPLEMENTED.
