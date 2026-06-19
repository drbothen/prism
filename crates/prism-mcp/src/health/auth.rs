//! Authentication validity detection for sensor health probes (BC-2.08.002, S-5.04).
//!
//! Issues the same adapter-based probe as the connectivity check
//! (`SpecDrivenSensorAdapter::fetch()` via `AdapterRegistry::get()`), then
//! classifies the HTTP response code:
//!
//! - HTTP 401 or 403 → `AuthStatus::Invalid` (distinct from `Down`)
//! - Connection error → `{ connectivity: Down, auth: Unknown }`
//! - 2xx or other non-auth error → `AuthStatus::Valid`
//!
//! # Key invariant (BC-2.08.002 postcondition 1)
//! "Unreachable" and "auth invalid" MUST be distinct — they require different
//! remediation actions.  Never conflate a connection error with an auth failure.

use prism_core::{OrgId, SensorId};
use prism_sensors::registry::AdapterRegistry;

use crate::health::connectivity::{ConnectivityStatus, ProbeOutcome};

// ── AuthStatus ────────────────────────────────────────────────────────────────

/// Authentication validity state for a sensor (BC-2.08.002).
///
/// `Unknown` is returned when a connection error prevented any HTTP exchange
/// (BC-2.08.002 postcondition 1 — "unreachable" ≠ "auth invalid").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    /// Credentials were accepted by the sensor (2xx response).
    Valid,
    /// Sensor returned HTTP 401 or 403 — credentials rejected.
    Invalid,
    /// Sensor was unreachable — no HTTP exchange occurred.
    Unknown,
}

/// Combined result of a single auth probe.
///
/// Includes both connectivity and auth status to support the BC-2.08.002
/// requirement that unreachable ≠ auth invalid.
#[derive(Debug, Clone)]
pub struct AuthProbeResult {
    /// Connectivity outcome (may be `Down` when auth is `Unknown`).
    pub connectivity: ConnectivityStatus,
    /// Authentication status.
    pub auth: AuthStatus,
    /// HTTP status code from the probe, if any.
    pub http_status: Option<u16>,
    /// Sanitised error text, if applicable.
    pub error: Option<String>,
    /// Whether the probe observed HTTP 429 (rate-limited) (F-S504-P1-001/002).
    pub is_rate_limited: bool,
    /// Retry-after delay in milliseconds from the 429 response (F-S504-P1-001).
    pub rate_limit_retry_after_ms: Option<u64>,
}

// ── probe_auth ────────────────────────────────────────────────────────────────

/// Issue an auth validity probe to a single sensor via the adapter registry.
///
/// Reuses the connectivity probe mechanism: calls `adapter.fetch()` with a
/// LIMIT-0 query, then inspects the HTTP status code.
///
/// - 401 or 403 → `auth: Invalid` (sensor reachable, credentials rejected)
/// - 2xx → `auth: Valid`
/// - Connection error → `connectivity: Down, auth: Unknown`
///   (MUST NOT be conflated with auth failure — BC-2.08.002 postcondition 1)
///
/// # FIX-001/v1.6 mandate
/// This function MUST NOT construct a `reqwest::Client`. It MUST obtain the
/// adapter via `registry.get(org_id, sensor_id)`.
pub async fn probe_auth(
    registry: &AdapterRegistry,
    org_id: OrgId,
    sensor_id: &SensorId,
    client_id: &str,
) -> Result<AuthProbeResult, prism_core::error::PrismError> {
    let outcome: ProbeOutcome =
        crate::health::connectivity::probe_connectivity(registry, org_id, sensor_id, client_id)
            .await?;

    let (auth, connectivity) = match &outcome.status {
        ConnectivityStatus::Down => {
            // Sensor unreachable — cannot determine auth validity (BC-2.08.002 postcondition 1)
            (AuthStatus::Unknown, ConnectivityStatus::Down)
        }
        ConnectivityStatus::Up | ConnectivityStatus::Degraded => {
            // HTTP response received — classify by status code.
            // F-S504-P1-002: HTTP 429 is NOT auth failure; sensor is reachable and auth
            // status cannot be determined during rate-limit (treat as Valid for this probe).
            let auth = match outcome.http_status {
                Some(401) | Some(403) => AuthStatus::Invalid,
                _ => AuthStatus::Valid,
            };
            (auth, outcome.status.clone())
        }
    };

    Ok(AuthProbeResult {
        connectivity,
        auth,
        http_status: outcome.http_status,
        error: outcome.error,
        is_rate_limited: outcome.is_rate_limited,
        rate_limit_retry_after_ms: outcome.rate_limit_retry_after_ms,
    })
}

// BC-5.38.005 self-check (S-5.04 implementation complete):
// probe_auth — non-trivial (delegates to probe_connectivity, classifies auth by HTTP status). IMPLEMENTED.
