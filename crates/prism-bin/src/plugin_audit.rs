//! RocksDbPluginAuditSink — production implementation of PluginLoadAuditSink.
//!
//! Writes durable audit entries to the `audit_buffer` RocksDB column family via
//! `append_audit_entry_sync` (WAL fsync) for each plugin load event.
//!
//! BC reference: BC-2.05.012 (Audit subsystem, postcondition 2: "confirmed durable"),
//! AC-4 of S-PLUGIN-PREREQ-D ("every unsigned plugin load must emit a DURABLE audit entry").
//!
//! # HIGH-002 closure (F-IMPL-LP1-HIGH-002)
//!
//! The previous implementation emitted only `tracing::warn!` for `plugin_load_unsigned`,
//! which is NOT a durable audit channel. This module provides the production wiring
//! so that the `audit_buffer` CF receives a persisted, fsync-confirmed entry for each
//! unsigned plugin load.

use std::collections::BTreeMap;
use std::sync::Arc;

use prism_spec_engine::plugin_audit_sink::PluginLoadAuditSink;
use prism_storage::audit_buffer::{AuditEntry, append_audit_entry_sync};
use prism_storage::rocksdb_backend::RocksDbBackend;
use uuid::Uuid;

/// Production `PluginLoadAuditSink` backed by RocksDB `audit_buffer` CF.
///
/// Writes durable, fsync-confirmed audit entries for each plugin load event.
/// Constructed in `plugin_load_step` from the `Arc<RocksDbBackend>` returned
/// by step 6 (`step6_init_audit`).
pub struct RocksDbPluginAuditSink {
    backend: Arc<RocksDbBackend>,
}

impl RocksDbPluginAuditSink {
    /// Construct a `RocksDbPluginAuditSink` from the given RocksDB backend.
    ///
    /// The `backend` MUST be the same `Arc<RocksDbBackend>` opened in step 6
    /// (all column families, including `audit_buffer`, are already open).
    pub fn new(backend: Arc<RocksDbBackend>) -> Self {
        Self { backend }
    }
}

impl PluginLoadAuditSink for RocksDbPluginAuditSink {
    /// Record a durable plugin load audit entry in the `audit_buffer` CF.
    ///
    /// Writes via `append_audit_entry_sync` which calls `flush_wal(true)` for
    /// synchronous, fsync-confirmed durability (BC-2.05.012 postcondition 2).
    fn record_plugin_load_event(
        &self,
        event_type: &str,
        plugin_path: &str,
        plugin_hash: &str,
        extra_fields: Option<&str>,
    ) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        let trace_id = Uuid::now_v7().to_string();

        let mut payload: BTreeMap<String, String> = BTreeMap::new();
        payload.insert("event_type".to_string(), event_type.to_string());
        payload.insert("plugin_path".to_string(), plugin_path.to_string());
        payload.insert("plugin_hash".to_string(), plugin_hash.to_string());
        if let Some(extra) = extra_fields {
            payload.insert("extra".to_string(), extra.to_string());
        }

        let entry = AuditEntry {
            timestamp_ns,
            trace_id,
            payload,
        };

        append_audit_entry_sync(&self.backend, &entry)
            .map_err(|e| format!("RocksDbPluginAuditSink write failed: {e}"))
    }
}
