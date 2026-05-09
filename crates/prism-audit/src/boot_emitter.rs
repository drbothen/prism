//! BootAuditEmitter — boot-time sentinel emitter for the audit subsystem init.
//!
//! # Purpose
//!
//! `AuditEmitterLayer` / `AuditEmitterService` are Tower middleware for the MCP
//! request lifecycle.  At boot step 6, no MCP request context exists — we need
//! a simpler handle that:
//!
//! 1. Is constructed from the `prism-audit` crate (BC-2.05.012 postcondition 1).
//! 2. Writes the `boot.audit.initialized` sentinel event to the `audit_buffer` CF.
//! 3. Enforces the sentinel schema: event_type, timestamp (RFC 3339), prism_version,
//!    config_dir (hash), org_count, boot_step (BC-2.05.012 §Sentinel Event Schema).
//! 4. Calls `append_audit_entry_sync` to guarantee durable write before returning
//!    (BC-2.05.012 Postcondition 2: "synchronous and confirmed durable").
//!
//! # Why Not `AuditEmitterLayer`?
//!
//! `AuditEmitterLayer` is a Tower `Layer` that wraps MCP tool invocations.  It
//! requires an `OrgRegistry`, a `ToolClassificationRegistry`, and an inner
//! Tower `Service`.  None of these exist at boot step 6 (OrgRegistry is in-memory,
//! no MCP transport is running).  `BootAuditEmitter` satisfies the BC postcondition
//! ("via AuditEmitter" from the prism-audit crate) with a minimal, boot-specific API.

use std::sync::Arc;

use chrono::Utc;
use prism_core::PrismError;
use prism_storage::audit_buffer::{append_audit_entry_sync, AuditEntry as StorageAuditEntry};
use prism_storage::rocksdb_backend::RocksDbBackend;
use uuid::Uuid;

/// Boot-time audit emitter.
///
/// Constructed at step 6 from the `prism-audit` crate (BC-2.05.012 postcondition 1).
/// Writes the `boot.audit.initialized` sentinel event synchronously and durably
/// to the `audit_buffer` column family (BC-2.05.012 postcondition 2).
///
/// Holds an `Arc<RocksDbBackend>` so it can be returned from step 6 and
/// used by step 7 once storage is needed for query operations.
pub struct BootAuditEmitter {
    backend: Arc<RocksDbBackend>,
}

/// Fields required for the `boot.audit.initialized` sentinel event.
///
/// All fields are specified by BC-2.05.012 §Postconditions lines 111-120.
pub struct BootSentinelFields<'a> {
    /// Semver string from CARGO_PKG_VERSION (e.g. "0.1.0").
    pub prism_version: &'a str,
    /// Hash or redacted identifier for the config directory
    /// (BC-2.05.012: "config_dir MUST be redacted — only a hash or basename").
    pub config_dir_hash: String,
    /// Number of orgs registered in the OrgRegistry (from step 3).
    pub org_count: usize,
}

impl BootAuditEmitter {
    /// Construct a `BootAuditEmitter` from the given RocksDB backend.
    ///
    /// BC-2.05.012 postcondition 1: "The `audit_buffer` RocksDB column family is
    /// opened and confirmed writable via `AuditEmitter`."
    ///
    /// This constructor does NOT perform any writes — it merely wraps the backend.
    /// Call [`emit_boot_sentinel`] to perform the actual sentinel write.
    pub fn new(backend: Arc<RocksDbBackend>) -> Self {
        Self { backend }
    }

    /// Emit the `boot.audit.initialized` sentinel event.
    ///
    /// Constructs the sentinel with all required BC-2.05.012 fields and writes
    /// it synchronously and durably via `append_audit_entry_sync`.
    ///
    /// # BC-2.05.012 Sentinel Schema (lines 111-120)
    ///
    /// Required fields:
    /// - `event_type`: `"boot.audit.initialized"`
    /// - `timestamp`: RFC 3339 string (F-PASS2-HIGH-2)
    /// - `prism_version`: semver from CARGO_PKG_VERSION
    /// - `config_dir`: redacted path hash (BC-2.05.012 invariant)
    /// - `org_count`: integer count of registered orgs
    /// - `boot_step`: `6` (ADR-022 §B step numbering)
    ///
    /// # Errors
    ///
    /// Returns `PrismError::AuditPersistenceFailed` if the write fails.
    /// Returns `PrismError::StorageWriteFailed` (with WAL detail) if the fsync fails.
    pub fn emit_boot_sentinel(&self, fields: BootSentinelFields<'_>) -> Result<(), PrismError> {
        // Read clock once so timestamp_ns and timestamp_rfc3339 are consistent.
        let now = Utc::now();
        // OBS-1 (S-WAVE5-PREP-01 fix-pass-5): harmonize with dominant crate pattern.
        // `.unwrap_or(0)` matches flag_events, credential_events, token_events, and
        // audit_emitter.  The boot sentinel also captures `timestamp_rfc3339` from the
        // same `now` capture, so even a 0 `timestamp_ns` (only possible for years
        // outside 1677–2262) does not lose the human-readable timestamp.
        let timestamp_ns = now.timestamp_nanos_opt().unwrap_or(0) as u64;

        // F-PASS2-HIGH-2: RFC 3339 timestamp field (BC-2.05.012 §Sentinel Schema).
        let timestamp_rfc3339 = now.to_rfc3339();

        let trace_id = Uuid::now_v7().to_string();

        let mut payload = std::collections::BTreeMap::new();
        payload.insert(
            "event_type".to_string(),
            "boot.audit.initialized".to_string(),
        );
        // F-PASS2-HIGH-2: timestamp is RFC 3339 (was missing in fix-pass-1).
        payload.insert("timestamp".to_string(), timestamp_rfc3339);
        payload.insert(
            "prism_version".to_string(),
            fields.prism_version.to_string(),
        );
        payload.insert("config_dir".to_string(), fields.config_dir_hash);
        payload.insert("org_count".to_string(), fields.org_count.to_string());
        payload.insert("boot_step".to_string(), "6".to_string());

        let sentinel = StorageAuditEntry {
            timestamp_ns,
            trace_id,
            payload,
        };

        // BC-2.05.012 Postcondition 2: synchronous and confirmed durable write.
        // Uses append_audit_entry_sync (F-PASS2-HIGH-1) which calls flush_wal(true).
        append_audit_entry_sync(&self.backend, &sentinel)
    }

    /// Return the `Arc<RocksDbBackend>` held by this emitter.
    ///
    /// Used by step 7 to re-use the already-opened RocksDB backend for storage
    /// operations without reopening the database.
    pub fn into_backend(self) -> Arc<RocksDbBackend> {
        self.backend
    }
}
