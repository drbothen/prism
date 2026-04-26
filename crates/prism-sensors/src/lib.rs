//! `prism-sensors` — Sensor adapter framework for the Prism platform.
//!
//! Defines the `SensorAdapter` async trait and supporting infrastructure that
//! makes the query engine's fan-out layer sensor-agnostic. Concrete sensor
//! implementations (CrowdStrike, Cyberint, Claroty, Armis) live in S-2.07.
//!
//! # Modules
//! - [`adapter`]        — `SensorAdapter` trait, `SensorSpec`, `QueryParams`, `SensorError`
//! - [`auth`]           — `SensorAuth` sealed trait + per-sensor auth subtypes + adapters
//! - [`registry`]       — `AdapterRegistry` mapping `SensorType` → `Arc<dyn SensorAdapter>`
//! - [`fanout`]         — Cross-client fan-out orchestrator (`fan_out()`)
//! - [`retry`]          — `retry_with_backoff()` with full-jitter exponential backoff
//! - [`http`]           — Global HTTP connection semaphore (200 permits)
//! - [`pagination`]     — `OffsetCursor` + `paginate_claroty()` stream (S-2.07)
//! - [`timestamp`]      — Multi-format timestamp parsing (S-2.07)
//! - [`table_dispatch`] — `route_table_query()` routing dispatch; imports `prism_core::TableType` (S-2.08)
//! - [`event_buffer`]   — `EventBufferStore` RocksDB CF ops for event-stream tables (S-2.08)
//! - [`poller`]         — `EventPoller` background task + `start_pollers()` (S-2.08)
//!
//! # Architecture Compliance (S-2.06 + S-2.07)
//! - `SensorAdapter` is object-safe — no generic methods (BC-2.01.013)
//! - `SensorAuth` is sealed — external crates cannot add auth types (BC-2.01.013, DI-012)
//! - Fan-out concurrency: 10 per query (BC-2.01.002) AND global HTTP cap: 200 (AC-5)
//! - Non-transient 4xx errors (400, 401, 403, 404) are NEVER retried (BC-2.01.014)
//!
//! Subsystem: SS-01 (Sensor Adapters) | Layer: 1 (Infrastructure)

// ── Module declarations ────────────────────────────────────────────────────
pub mod adapter;
pub mod auth;
pub mod event_buffer;
pub mod fanout;
pub mod http;
pub mod pagination;
pub mod poller;
pub mod registry;
pub mod retry;
pub mod table_dispatch;
pub mod timestamp;

// ── Test modules (cfg-gated) ───────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ─────────────────────────────────────────────────────────────
pub use adapter::{is_transient_status, QueryParams, SensorAdapter, SensorError, SensorSpec};
pub use auth::armis::ArmisAdapter;
pub use auth::claroty::{ClarotyAdapter, ClarotyId};
pub use auth::crowdstrike::CrowdStrikeAdapter;
pub use auth::cyberint::CyberintAdapter;
pub use auth::{ArmisAuth, ClarotyAuth, CrowdStrikeAuth, CyberintAuth, SensorAuth};
pub use fanout::{
    error_to_retry_metadata, fan_out, CredentialResolver, FanOutError, FanOutResult, FanOutTarget,
    RetryMetadata, MAX_FANOUT_CONCURRENCY,
};
pub use http::{
    acquire_http_permit, available_http_permits, init_http_semaphore, HTTP_SEMAPHORE_PERMITS,
    HTTP_SEMAPHORE_TIMEOUT,
};
pub use pagination::{paginate_claroty, OffsetCursor};
pub use registry::AdapterRegistry;
pub use retry::{retry_with_backoff, RetryConfig, DEFAULT_TRANSIENT_CODES};
pub use timestamp::parse_timestamp;

// S-2.08 re-exports
pub use event_buffer::{EventBufferStore, NormalizedRecord};
pub use poller::{start_pollers, EventPoller, PollerDiagnostics, PollerId, PollerStatus};
pub use table_dispatch::{route_table_query, TableType, TableTypeRouteDecision};

// ---------------------------------------------------------------------------
// init_registry — startup adapter registration
// ---------------------------------------------------------------------------

/// Initializes the `AdapterRegistry` with all four built-in sensor adapters.
///
/// Called once at process startup (e.g., from `main()` or the MCP server
/// initialization path).  Registers:
/// - `CrowdStrikeAdapter` for `SensorType::CrowdStrike`
/// - `CyberintAdapter` for `SensorType::Cyberint`
/// - `ClarotyAdapter` for `SensorType::Claroty`
/// - `ArmisAdapter` for `SensorType::Armis`
///
/// Each adapter is wrapped in `Arc` and stored in the registry.
///
/// # Arguments
/// - `crowdstrike_auth` — pre-constructed auth credential for CrowdStrike.
/// - `cyberint_auth`    — pre-constructed auth credential for Cyberint.
/// - `claroty_auth`     — pre-constructed auth credential for Claroty.
/// - `claroty_token`    — bearer token string for Claroty (from credential store).
/// - `armis_auth`       — pre-constructed auth credential for Armis.
/// - `armis_token`      — bearer token string for Armis (from credential store).
///
/// Story: S-2.07 §Task 5
pub fn init_registry(
    crowdstrike_auth: &CrowdStrikeAuth,
    cyberint_auth: &CyberintAuth,
    claroty_auth: &ClarotyAuth,
    claroty_token: String,
    armis_auth: &ArmisAuth,
    armis_token: String,
) -> AdapterRegistry {
    use std::sync::Arc;
    let mut registry = AdapterRegistry::new();

    registry.register(Arc::new(CrowdStrikeAdapter::new(crowdstrike_auth)));
    registry.register(Arc::new(CyberintAdapter::new(cyberint_auth)));
    registry.register(Arc::new(ClarotyAdapter::new(claroty_auth, claroty_token)));
    registry.register(Arc::new(ArmisAdapter::new(armis_auth, armis_token)));

    registry
}
