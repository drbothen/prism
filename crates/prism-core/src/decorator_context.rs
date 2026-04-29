//! DecoratorContext — the three-phase metadata envelope injected into `_meta`
//! for every query result (S-2.03).
//!
//! Decorator fields are injected *after* DataFusion execution and are NOT
//! queryable in PrismQL predicates.  Missing values serialize as JSON `null`
//! (BC-2.15.009 — decorator injection cannot fail).
//!
//! The three phases and their fields:
//! - **Phase 1 (config-time):** `client_name`, `prism_version` — set at
//!   startup and on config reload (BC-2.15.010 Phase 1).
//! - **Phase 2 (query-time):** `analyst_id`, `query_source`,
//!   `sensor_instance` — computed fresh per query; never cached
//!   (BC-2.15.010 Phase 2).
//! - **Phase 3 (periodic):** `sensor_health_status` — refreshed on a
//!   configurable interval (default 300s); cached in RocksDB `decorators` CF
//!   for persistence across restarts (BC-2.15.010 Phase 3).
//!
//! The `merge` logic (config-time < query-time < periodic priority) lives in
//! `prism-storage::DecorationStore::merge` to keep this struct a pure
//! data-only type.

use serde::{Deserialize, Serialize};

/// All decorator fields across all three phases.
///
/// Every field is `Option<String>`; `None` serializes to JSON `null`.
/// Decorator injection never fails — callers set fields to `None` on
/// missing context rather than returning an error (BC-2.15.009).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecoratorContext {
    // ── Phase 1: config-time (static metadata) ───────────────────────────────
    /// Human-readable client name from the TOML `[clients.{id}]` section.
    /// NOT the OrgSlug — the display name (e.g., "Acme Corp").
    pub client_name: Option<String>,

    /// Running Prism version string (e.g., "0.1.0") from build metadata.
    pub prism_version: Option<String>,

    // ── Phase 2: query-time (per-invocation context) ─────────────────────────
    /// Analyst identifier from the current session context or tool parameter.
    /// `None` for automated / scheduled queries (EC-15-034).
    pub analyst_id: Option<String>,

    /// Provenance of the query invocation.
    ///
    /// Format strings (set by prism-query in S-3.02):
    /// - Interactive: `"interactive"`
    /// - Scheduled: `"schedule:{schedule_name}"` (e.g., `"schedule:check_alerts"`)
    /// - Pack: `"pack:{pack_name}.{query_name}"` (e.g.,
    ///   `"pack:incident-response.recent_detections"`)
    pub query_source: Option<String>,

    /// The sensor instance identifier (e.g., "us-1") for the adapter that
    /// fetched each record.
    pub sensor_instance: Option<String>,

    // ── Phase 3: periodic (refreshed on interval) ────────────────────────────
    /// Last-known health status of the sensor (e.g., "healthy", "degraded").
    /// `None` before the first periodic refresh completes (EC-15-039).
    pub sensor_health_status: Option<String>,
}
