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

use std::sync::Arc;

use chrono::{DateTime, Utc};
use prism_core::{OrgId, SensorId};
use prism_sensors::registry::AdapterRegistry;

use crate::context::{PrismContext, SensorKey};
use crate::health::connectivity::ConnectivityStatus;
use crate::health::{
    auth::AuthStatus, connectivity::ProbeOutcome, rate_limit::extract_rate_limit_state,
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
/// - `Healthy` — all sensors are reachable and authenticated.
/// - `Partial` — some sensors up, some down or degraded.
///   This is a SUCCESS state, not an error (BC-2.08.007 postcondition 1).
/// - `Unhealthy` — all sensors are unreachable or auth-invalid.
///   Returned as a success response, not an error (BC-2.08.007 postcondition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverallStatus {
    /// All sensors are reachable and authenticated.
    Healthy,
    /// Some sensors are up, others are down or degraded.
    Partial,
    /// All sensors are unreachable or auth-invalid.
    Unhealthy,
}

impl HealthCheckResult {
    /// Aggregate a batch of `SensorHealthResult` values into an `OverallStatus`.
    ///
    /// Rules (BC-2.08.007 postcondition 1):
    /// - All `reachable: true, auth_valid: true` → `Healthy`
    /// - At least one healthy and at least one not → `Partial`
    /// - None healthy → `Unhealthy`
    /// - `Partial` is a SUCCESS state — callers MUST NOT return an error for it.
    pub fn aggregate(results: Vec<SensorHealthResult>) -> OverallStatus {
        if results.is_empty() {
            return OverallStatus::Unhealthy;
        }

        let healthy_count = results
            .iter()
            .filter(|r| r.reachable == Some(true) && r.auth_valid == Some(true))
            .count();

        if healthy_count == results.len() {
            OverallStatus::Healthy
        } else if healthy_count > 0 {
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
#[derive(Debug, Clone)]
pub struct SensorHealthChecker {
    /// Shared adapter registry for live probe dispatch (FIX-001 / ADR-023 §C1).
    #[allow(dead_code)]
    adapter_registry: Arc<AdapterRegistry>,
}

impl SensorHealthChecker {
    /// Construct a `SensorHealthChecker` with the provided adapter registry.
    ///
    /// Called at server construction time (boot step 9 / `PrismServer::with_deps`).
    pub fn new(adapter_registry: Arc<AdapterRegistry>) -> Self {
        Self { adapter_registry }
    }

    /// Run a live health probe for all sensors registered for `org_id`.
    ///
    /// For each sensor, calls `AdapterRegistry::get(org_id, sensor_id)` and issues
    /// a minimal probe query via `adapter.fetch()`.  Populates `SensorHealthResult`
    /// with `probe_level: "live"`, `reachable`, `auth_valid`, and
    /// `last_successful_query_at` fields (BC-2.08.005 v1.5 live-probe postcondition).
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
    /// Delegates to `auth::probe_auth` using the adapter obtained from
    /// `AdapterRegistry::get(org_id, sensor_id)`.
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
        match auth::probe_auth(&self.adapter_registry, org_id, sensor_id, client_id).await {
            Ok(probe) => {
                let reachable = probe.connectivity != ConnectivityStatus::Down;
                let auth_valid = matches!(probe.auth, AuthStatus::Valid);

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

                // Record successful query timestamp when both reachable and auth valid
                let last_successful_query_at = if reachable && auth_valid && !probe.is_rate_limited
                {
                    let now = chrono::Utc::now();
                    self.record_successful_query(client_id, sensor_id, now, context);
                    Some(now)
                } else {
                    self.last_successful_query(client_id, sensor_id, context)
                };

                let mut result = SensorHealthResult::new(sensor_id.as_ref(), client_id)
                    .with_reachable(reachable)
                    .with_auth_valid(auth_valid);
                result.probe_level = "live".to_string();
                result.rate_limit = rate_limit_info;
                if let Some(ts) = last_successful_query_at {
                    result = result.with_last_successful_query_at(ts);
                }
                result
            }
            Err(_) => {
                // Engine error — sensor unreachable
                let mut result = SensorHealthResult::new(sensor_id.as_ref(), client_id)
                    .with_reachable(false)
                    .with_auth_valid(false);
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
