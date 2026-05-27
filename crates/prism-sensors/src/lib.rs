//! `prism-sensors` — Sensor adapter framework for the Prism platform.
//!
//! Defines the `SensorAdapter` async trait and supporting infrastructure that
//! makes the query engine's fan-out layer sensor-agnostic. As of
//! PLUGIN-MIGRATION-001-A, all four built-in concrete sensor implementations
//! (CrowdStrike, Cyberint, Claroty, Armis) have been deleted from this crate.
//! Sensors now run exclusively via TOML specs + WASM plugins through the spec
//! engine (ADR-023, ADR-028 §D10).
//!
//! # Modules
//! - [`adapter`]        — `SensorAdapter` trait, `SensorSpec`, `QueryParams`, `SensorError`
//! - [`auth`]           — `SensorAuth` open trait (built-in impls deleted in PLUGIN-MIGRATION-001-A)
//! - [`registry`]       — `AdapterRegistry` mapping `SensorId` → `Arc<dyn SensorAdapter>`
//! - [`fanout`]         — Cross-client fan-out orchestrator (`fan_out()`)
//! - [`retry`]          — `retry_with_backoff()` with full-jitter exponential backoff
//! - [`http`]           — Global HTTP connection semaphore (200 permits)
//! - [`pagination`]     — `OffsetCursor` + `paginate_claroty()` stream
//!   (S-2.07; function retained for potential WASM plugin use)
//! - [`timestamp`]      — Multi-format timestamp parsing (S-2.07)
//! - [`table_dispatch`] — `route_table_query()` routing dispatch; imports `prism_core::TableType` (S-2.08)
//! - [`event_buffer`]   — `EventBufferStore` RocksDB CF ops for event-stream tables (S-2.08)
//! - [`poller`]         — `EventPoller` background task + `start_pollers()` (S-2.08)
//!
//! # Architecture Compliance (S-2.06 + S-2.07 + PLUGIN-MIGRATION-001-A)
//! - `SensorAdapter` is object-safe — no generic methods (BC-2.01.013)
//! - `SensorAuth` is open — external crates (plugins) may implement it (BC-2.01.016, S-PLUGIN-PREREQ-E)
//! - Fan-out concurrency: 10 per query (BC-2.01.002) AND global HTTP cap: 200 (AC-5)
//! - Non-transient 4xx errors (400, 401, 403, 404) are NEVER retried (BC-2.01.014)
//! - Built-in sensor adapters deleted; spec-catalog dispatch is in prism-bin at boot (ADR-023)
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
// S-3.07 — per-record write result types for SensorAdapter::write()
pub mod write_result;

// ── Test modules (cfg-gated) ───────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ─────────────────────────────────────────────────────────────
pub use adapter::{is_transient_status, QueryParams, SensorAdapter, SensorError, SensorSpec};
pub use auth::SensorAuth;
pub use fanout::{
    error_to_retry_metadata, fan_out, fan_out_with_overlay_map, resolve_spec_for_fanout,
    CredentialResolver, FanOutError, FanOutResult, FanOutTarget, RetryMetadata,
    MAX_FANOUT_CONCURRENCY,
};
pub use http::{
    acquire_http_permit, available_http_permits, init_http_semaphore, HTTP_SEMAPHORE_PERMITS,
    HTTP_SEMAPHORE_TIMEOUT,
};
pub use pagination::{paginate_claroty, OffsetCursor};
pub use registry::AdapterRegistry;
pub use retry::{retry_with_backoff, RetryConfig, DEFAULT_TRANSIENT_CODES};
pub use timestamp::parse_timestamp;
pub use types::RequestParams;

// S-2.08 re-exports
pub use event_buffer::{EventBufferStore, NormalizedRecord};
pub use poller::{start_pollers, EventPoller, PollerDiagnostics, PollerId, PollerStatus};
pub use table_dispatch::{route_table_query, TableType, TableTypeRouteDecision};

// S-3.1.06 re-exports: OrgId is the canonical org key for all sensor dispatch
pub use prism_core::OrgId;

// S-3.07 re-exports: per-record write result types
pub use write_result::{RecordWriteResult, WriteStatus};

// ---------------------------------------------------------------------------
// init_registry — startup adapter registration (deprecated, migration window)
// ---------------------------------------------------------------------------

/// Initializes an empty `AdapterRegistry` (deprecated compatibility shim).
///
/// As of PLUGIN-MIGRATION-001-A, all four built-in sensor adapters have been
/// deleted. Sensors are now driven exclusively by TOML specs + WASM plugins
/// through the spec engine at boot time (ADR-023, ADR-028 §D10).
///
/// Spec-catalog dispatch wiring belongs in `prism-bin` at boot (boot.rs step 7)
/// rather than here, because `prism-sensors` MUST NOT gain any NEW `prism-spec-engine`
/// imports for spec-catalog dispatch wiring (ADR-028 §D3); the existing dependency
/// is limited to `WriteEndpointSpec` and overlay resolution per S-3.07 /
/// S-CONFIG-MULTI-TENANT-OVERRIDE-001.
///
/// # Deprecated
/// Use `init_registry_for_org` instead. Retained for backward compat during
/// the S-3.1.06 migration window (ADR-006). Use the nil `OrgId` internally;
/// callers must migrate to `init_registry_for_org` before Wave 5.
///
/// Story: PLUGIN-MIGRATION-001-A §AC-003/AC-005 | BC: BC-3.2.001
#[deprecated(
    since = "0.2.0",
    note = "use `init_registry_for_org(org_id)` instead (S-3.1.06 / PLUGIN-MIGRATION-001-A)"
)]
pub fn init_registry() -> AdapterRegistry {
    let sentinel = prism_core::OrgId::from_uuid(uuid::Uuid::nil());
    init_registry_for_org(sentinel)
}

/// OrgId-keyed adapter registry initialization (PLUGIN-MIGRATION-001-A post-deletion).
///
/// Returns an empty `AdapterRegistry` keyed to `org_id`. All four built-in sensor
/// adapters (CrowdStrike, Cyberint, Claroty, Armis) have been deleted as of
/// PLUGIN-MIGRATION-001-A (ADR-028 §D10 co-merge contract satisfied).
///
/// Spec-catalog dispatch (populating the registry from the loaded `SensorSpec`
/// catalog via `PluginRegistry` open dispatch, BC-2.16.012 INV-SPEC-PARSER-OPEN-001)
/// is wired in `prism-bin/src/boot.rs` at boot time, not here. This function
/// returns an empty registry as the structural baseline; `prism-bin` populates
/// adapters from the spec catalog after plugin load (step 7 / step 7.5).
///
/// # PLUGIN-MIGRATION-001-A Known Gap (GAP-002-A)
///
/// Spec-catalog dispatch wiring is deferred to `S-WAVE5-PREP-01` /
/// `S-3.02-FOLLOWUP-RUNTIME`. The wiring belongs in `prism-bin` (boot-time
/// orchestration concern) because `prism-sensors` MUST NOT gain any NEW
/// `prism-spec-engine` imports for spec-catalog dispatch wiring (ADR-028 §D3);
/// the existing dependency is limited to `WriteEndpointSpec` and overlay
/// resolution per S-3.07 / S-CONFIG-MULTI-TENANT-OVERRIDE-001.
///
/// # Arguments
/// - `org_id` — canonical org identity for this adapter set (BC-3.2.001 precondition 4).
///
/// Story: S-3.1.06-ImplPhase → PLUGIN-MIGRATION-001-A §AC-004 | BC: BC-3.2.001 precondition 4
pub fn init_registry_for_org(org_id: prism_core::OrgId) -> AdapterRegistry {
    // All four built-in adapter construction calls removed per AC-004:
    //   - CrowdStrikeAdapter::new (deleted: PLUGIN-MIGRATION-001-A AC-006)
    //   - CyberintAdapter::new    (deleted: PLUGIN-MIGRATION-001-A AC-003)
    //   - ClarotyAdapter::new     (deleted: PLUGIN-MIGRATION-001-A AC-003)
    //   - ArmisAdapter::new       (deleted: PLUGIN-MIGRATION-001-A AC-003)
    //
    // Spec-catalog dispatch (BC-2.16.012) is the replacement path.
    // Wiring is a boot-time concern in prism-bin, not prism-sensors, because
    // prism-sensors MUST NOT gain any NEW prism-spec-engine imports for
    // spec-catalog dispatch wiring (ADR-028 §D3); the existing dependency
    // is limited to WriteEndpointSpec and overlay resolution (S-3.07 /
    // S-CONFIG-MULTI-TENANT-OVERRIDE-001).
    //
    // GAP-002-A: Full spec-catalog dispatch wiring deferred to S-WAVE5-PREP-01.
    let _ = org_id; // org_id preserved in signature per BC-3.2.001 precondition 4
    AdapterRegistry::new()
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
