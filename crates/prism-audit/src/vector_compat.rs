//! Vector pipeline compatibility for audit entries (BC-2.05.007, S-2.05).
//!
//! Provides [`VectorAuditEntry`] — a newtype wrapper around [`crate::AuditEntry`]
//! — and [`to_vector_json()`] which produces a Vector-compatible JSON object
//! without modifying the stored entry.
//!
//! # Architecture compliance (S-2.05)
//!
//! - `VectorAuditEntry` MUST NOT modify the stored `AuditEntry`.
//!   `to_vector_json()` is a read-only view; the entry in RocksDB always uses
//!   the canonical `AuditEntry` format.
//! - The canonical format uses `snake_case` field names (BC-2.05.007).
//! - The `@timestamp` field is an RFC 3339 copy of `AuditEntry.timestamp`.
//! - `"host"` falls back to `gethostname()` when `PRISM_HOST_ID` is unset
//!   (EC-002: never panics or returns an empty `host` field).
//! - `"log.level"` uses Vector canonical values: `"info"` for success,
//!   `"error"` for failure (not Rust tracing level names).
//! - The entire audit entry is emitted as single-line JSON — no multi-line
//!   pretty-printing (BC-2.05.007).

use crate::audit_entry::AuditEntry;
use serde_json::Value;

// ── Newtype wrapper ───────────────────────────────────────────────────────────

/// Newtype wrapper around `AuditEntry` with a Vector-compatible JSON view.
///
/// This type does NOT own a mutable copy of the entry — it borrows the
/// canonical entry and adds Vector-required fields in [`to_vector_json()`]
/// without modifying the original.
///
/// # Vector required fields (AC-2 / BC-2.05.007)
///
/// | Field | Source |
/// |-------|--------|
/// | `@timestamp` | RFC 3339 copy of `AuditEntry.timestamp` |
/// | `host` | `PRISM_HOST_ID` env var, falling back to `gethostname()` |
/// | `service` | Fixed string `"prism"` |
/// | `log.level` | `"info"` for `AuditOutcome::Success`, `"error"` for Failure |
pub struct VectorAuditEntry<'a> {
    /// The wrapped canonical audit entry (never mutated).
    pub entry: &'a AuditEntry,
}

impl<'a> VectorAuditEntry<'a> {
    /// Wrap an `AuditEntry` reference for Vector-compatible serialization.
    pub fn new(entry: &'a AuditEntry) -> Self {
        Self { entry }
    }

    /// Produce a Vector-compatible flat JSON object for this entry.
    ///
    /// See [`to_vector_json()`] for full semantics.
    pub fn to_json(&self) -> Value {
        to_vector_json(self.entry)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Produce a Vector-compatible flat JSON object from an `AuditEntry`.
///
/// The resulting JSON is:
/// - A flat `serde_json::Value::Object` (no nested structs — Vector's default
///   JSON parser expects flat fields)
/// - `parameters` is serialized as a JSON **string** value (not a nested object)
/// - Includes the four Vector-required extra fields: `@timestamp`, `host`,
///   `service`, `log.level`
///
/// # Read-only guarantee (S-2.05 Architecture Compliance)
///
/// This function never modifies `entry`. The original entry stored in RocksDB
/// is always the canonical `AuditEntry` format.
///
/// # Fallback for `host` (EC-002)
///
/// If `PRISM_HOST_ID` is unset, falls back to `gethostname::gethostname()`.
/// The `host` field is NEVER empty — if both fail, the sentinel `"unknown-host"`
/// is used.
///
/// # Arguments
///
/// - `entry` — the canonical audit entry to wrap
///
/// # Returns
///
/// A `serde_json::Value::Object` with all `AuditEntry` fields plus Vector
/// extra fields.
pub fn to_vector_json(entry: &AuditEntry) -> Value {
    // Serialize the entire AuditEntry to a JSON object first.
    let mut obj = match serde_json::to_value(entry) {
        Ok(Value::Object(map)) => map,
        Ok(other) => {
            // Fallback: wrap the value — should never happen for a struct.
            let mut m = serde_json::Map::new();
            m.insert("_raw".to_owned(), other);
            m
        }
        Err(e) => {
            let mut m = serde_json::Map::new();
            m.insert(
                "_serialization_error".to_owned(),
                Value::String(e.to_string()),
            );
            m
        }
    };

    // Serialize `parameters` as a JSON string (not a nested object) — Vector
    // default JSON parser expects flat fields (Dev Notes / BC-2.05.007).
    if let Some(params) = obj.get("parameters") {
        let params_string = serde_json::to_string(params).unwrap_or_else(|_| "{}".to_owned());
        obj.insert("parameters".to_owned(), Value::String(params_string));
    }

    // Add Vector-required extra fields (AC-2 / BC-2.05.007).
    obj.insert(
        "@timestamp".to_owned(),
        Value::String(entry.timestamp.to_rfc3339()),
    );
    obj.insert("host".to_owned(), Value::String(resolve_host()));
    obj.insert("service".to_owned(), Value::String("prism".to_owned()));
    obj.insert(
        "log.level".to_owned(),
        Value::String(outcome_to_log_level(&entry.outcome).to_owned()),
    );

    Value::Object(obj)
}

/// Resolve the `host` field value for Vector (EC-002 fallback chain).
///
/// Resolution order:
/// 1. `PRISM_HOST_ID` environment variable (if set and non-empty)
/// 2. `gethostname::gethostname()` OS call
/// 3. Sentinel `"unknown-host"` (never panics, never returns empty)
pub fn resolve_host() -> String {
    // Resolution order (EC-002):
    // 1. PRISM_HOST_ID environment variable (if set and non-empty)
    // 2. gethostname::gethostname() OS call
    // 3. Sentinel "unknown-host" (never panics, never returns empty)
    if let Ok(host) = std::env::var("PRISM_HOST_ID") {
        if !host.is_empty() {
            return host;
        }
    }

    // Fall back to gethostname().
    let hostname = gethostname::gethostname();
    let hostname_str = hostname.to_string_lossy();
    if !hostname_str.is_empty() {
        return hostname_str.into_owned();
    }

    // Final sentinel — never empty, never panics.
    "unknown-host".to_owned()
}

/// Map `AuditOutcome` to a Vector canonical log level string (BC-2.05.007 AC-2).
///
/// Returns `"info"` for `AuditOutcome::Success`, `"error"` for
/// `AuditOutcome::Failure`.
///
/// # GREEN-BY-DESIGN (pure data mapping, tautological test)
///
/// This trivial two-arm match is implemented because the test would be
/// tautological — asserting `"info" == "info"` proves nothing about behavior.
/// Flagged as GREEN-BY-DESIGN per stub architect protocol.
pub fn outcome_to_log_level(outcome: &crate::audit_entry::AuditOutcome) -> &'static str {
    match outcome {
        crate::audit_entry::AuditOutcome::Success => "info",
        crate::audit_entry::AuditOutcome::Failure { .. } => "error",
    }
}
