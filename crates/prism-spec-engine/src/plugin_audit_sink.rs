//! Plugin load audit sink — injectable callback for durable audit entries on plugin load.
//!
//! # Purpose
//!
//! `PluginLoadAuditSink` is an abstraction over the durable audit channel for plugin
//! load events. The production implementation (`RocksDbPluginAuditSink` in `prism-bin`)
//! writes to the `audit_buffer` RocksDB column family via `append_audit_entry_sync`.
//! Test implementations use `NoOpPluginAuditSink` (no I/O, zero side-effects).
//!
//! # Why a trait?
//!
//! `prism-spec-engine` must not depend on `prism-storage` (RocksDB) or `prism-audit`
//! (Tower middleware). The trait inverts the dependency: `prism-bin` provides the
//! production implementation; `prism-spec-engine` defines the interface.
//!
//! # BC Reference
//!
//! BC-2.05.012 (Audit subsystem) + AC-4 of S-PLUGIN-PREREQ-D:
//! "every unsigned plugin load must emit a DURABLE audit entry — not just a tracing::warn!".
//! HIGH-002 (F-IMPL-LP1-HIGH-002): closes the audit channel gap identified in impl-pass-1.

use std::sync::Arc;

/// Audit sink for plugin load events.
///
/// Implementations write a durable, persistent audit record for each plugin
/// load event (success or failure). The record is NOT just a `tracing::warn!` —
/// it must be persisted to the audit_buffer column family (RocksDB WAL fsync)
/// or an equivalent durable write path.
///
/// The trait is `Send + Sync` so `PluginRuntime` can hold `Arc<dyn PluginLoadAuditSink>`.
pub trait PluginLoadAuditSink: Send + Sync {
    /// Record a durable audit entry for a plugin load event.
    ///
    /// # Parameters
    ///
    /// - `event_type`: Catalog event_type string (e.g., `"plugin_load_unsigned"`).
    ///   Must match a row in BC-2.16.002 Canonical Structured Event Catalog.
    /// - `plugin_path`: Filesystem path to the `.prx` file being loaded.
    /// - `plugin_hash`: SHA-256 hex hash of the `.prx` bytes (empty string if unavailable).
    /// - `extra_fields`: Optional JSON-encoded additional fields (error message, etc.).
    ///
    /// # Contract
    ///
    /// - The implementation MUST persist the record durably before returning `Ok(())`.
    /// - On persistence failure, return `Err(String)` with a descriptive message.
    ///   The caller (load_all_plugins) treats Err as a log-and-continue (n-1 survivor rule).
    /// - MUST NOT panic.
    fn record_plugin_load_event(
        &self,
        event_type: &str,
        plugin_path: &str,
        plugin_hash: &str,
        extra_fields: Option<&str>,
    ) -> Result<(), String>;
}

/// No-op audit sink for tests and production paths where RocksDB is not available.
///
/// Always returns `Ok(())` without performing any I/O. Used by:
/// - Test builds (no RocksDB available in unit test context).
/// - PRISM_DISABLE_PLUGIN_LOAD=1 path (no plugins loaded; no audit needed).
pub struct NoOpPluginAuditSink;

impl PluginLoadAuditSink for NoOpPluginAuditSink {
    fn record_plugin_load_event(
        &self,
        _event_type: &str,
        _plugin_path: &str,
        _plugin_hash: &str,
        _extra_fields: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Convenience constructor: `Arc<dyn PluginLoadAuditSink>` wrapping `NoOpPluginAuditSink`.
pub fn noop_sink() -> Arc<dyn PluginLoadAuditSink> {
    Arc::new(NoOpPluginAuditSink)
}
