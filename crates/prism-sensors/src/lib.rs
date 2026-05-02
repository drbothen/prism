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

// SEC-P2-006: enforce at compile time that deprecated APIs (e.g. `init_registry`) are
// not called without an explicit `#[allow(deprecated)]` at the call site.  This gates
// migration to `init_registry_for_org` — any new caller that omits the allow gets a
// compile error (BC-3.2.001 invariant 1; AC-008).
#![deny(deprecated)]

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
pub mod types;

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
pub use secrecy::SecretString;
pub use timestamp::parse_timestamp;

// S-2.08 re-exports
pub use event_buffer::{EventBufferStore, NormalizedRecord};
pub use poller::{start_pollers, EventPoller, PollerDiagnostics, PollerId, PollerStatus};
pub use table_dispatch::{route_table_query, TableType, TableTypeRouteDecision};

// S-3.1.06 re-exports: OrgId is the canonical org key for all sensor dispatch
pub use prism_core::OrgId;

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
/// Each adapter is wrapped in `Arc` and stored in the registry under the
/// sentinel nil `OrgId`.
///
/// # Arguments
/// - `crowdstrike_auth` — pre-constructed auth credential for CrowdStrike.
/// - `cyberint_auth`    — pre-constructed auth credential for Cyberint.
/// - `claroty_auth`     — pre-constructed auth credential for Claroty.
/// - `claroty_token`    — bearer token for Claroty (wrapped as `SecretString`).
/// - `armis_auth`       — pre-constructed auth credential for Armis.
/// - `armis_token`      — bearer token for Armis (wrapped as `SecretString`).
///
/// Tokens are `SecretString` to enforce zeroing-on-drop and prevent heap-dump
/// exposure (WGS-W2-002, CWE-312).
///
/// # Deprecated
/// Use `init_registry_for_org` instead. Retained for backward compat during
/// the S-3.1.06 migration window (ADR-006). Uses a sentinel nil `OrgId`
/// internally; callers must migrate to `init_registry_for_org` before Wave 5.
///
/// Story: S-2.07 §Task 5
#[deprecated(
    since = "0.2.0",
    note = "use `init_registry_for_org(org_id, ...)` instead (S-3.1.06)"
)]
pub fn init_registry(
    crowdstrike_auth: &CrowdStrikeAuth,
    cyberint_auth: &CyberintAuth,
    claroty_auth: &ClarotyAuth,
    claroty_token: SecretString,
    armis_auth: &ArmisAuth,
    armis_token: SecretString,
) -> AdapterRegistry {
    // Thin wrapper: use the nil UUID as the sentinel OrgId for the migration
    // window (ADR-006). Callers must migrate to init_registry_for_org before
    // Wave 5 (AC-005).
    let sentinel = prism_core::OrgId::from_uuid(uuid::Uuid::nil());
    init_registry_for_org(
        sentinel,
        crowdstrike_auth,
        cyberint_auth,
        claroty_auth,
        claroty_token,
        armis_auth,
        armis_token,
    )
}

/// OrgId-keyed adapter registry initialization (S-3.1.06-ImplPhase).
///
/// This is the successor to `init_registry()` that accepts an explicit `OrgId`
/// so adapter dispatch is structurally keyed per org (BC-3.2.001 precondition 4).
///
/// Registers all four built-in sensor adapters under the composite
/// `(org_id, SensorType)` key, each constructed with the provided credentials
/// and the given `org_id`.
///
/// # Arguments
/// - `org_id`           — canonical org identity for this adapter set.
/// - `crowdstrike_auth` — pre-constructed auth credential for CrowdStrike.
/// - `cyberint_auth`    — pre-constructed auth credential for Cyberint.
/// - `claroty_auth`     — pre-constructed auth credential for Claroty.
/// - `claroty_token`    — bearer token for Claroty (wrapped as `SecretString`).
/// - `armis_auth`       — pre-constructed auth credential for Armis.
/// - `armis_token`      — bearer token for Armis (wrapped as `SecretString`).
///
/// Story: S-3.1.06-ImplPhase §Task 4 | BC: BC-3.2.001 precondition 4
pub fn init_registry_for_org(
    org_id: prism_core::OrgId,
    crowdstrike_auth: &CrowdStrikeAuth,
    cyberint_auth: &CyberintAuth,
    claroty_auth: &ClarotyAuth,
    claroty_token: SecretString,
    armis_auth: &ArmisAuth,
    armis_token: SecretString,
) -> AdapterRegistry {
    use std::sync::Arc;

    let mut registry = AdapterRegistry::new();

    registry.register(
        org_id,
        Arc::new(CrowdStrikeAdapter::new(org_id, crowdstrike_auth)),
    );
    registry.register(
        org_id,
        Arc::new(CyberintAdapter::new(org_id, cyberint_auth)),
    );
    registry.register(
        org_id,
        Arc::new(ClarotyAdapter::new(org_id, claroty_auth, claroty_token)),
    );
    registry.register(
        org_id,
        Arc::new(ArmisAdapter::new(org_id, armis_auth, armis_token)),
    );

    registry
}

/// `DEFAULT_ORG_ID` — sentinel `OrgId` for use in unit tests ONLY.
///
/// This constant is `#[cfg(test)]` gated so it cannot appear in production
/// code paths (BC-3.2.001 invariant 3, EC-005).  Any attempt to use it
/// outside a `#[cfg(test)]` context will result in a compile error.
///
/// The value is a fixed UUID v7 chosen for test-vector reproducibility.
/// Production code MUST obtain `OrgId` from `OrgRegistry::resolve()`.
#[cfg(test)]
pub const DEFAULT_ORG_ID_BYTES: [u8; 16] = [
    0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, // UUID v7 time-high + time-mid + time-low
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // variant + node
];
