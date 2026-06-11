//! P1-03 (2026-06-10 review pass-1) — `BootAuditWriter` production-path CF-readback tests.
//!
//! `BootAuditWriter` is the production `AuditWriter` wired into `WriteExecutor` +
//! `PrismServer` at boot step 9. Its `write_tool_call` method (MCP-02, 2026-06-10
//! review) persists MCP tool-call audit records durably to the RocksDB
//! `audit_buffer` CF (BC-2.05.001 — one audit entry per tool invocation —
//! / CRIT-005).
//!
//! These tests follow the repo's own CF-readback precedent:
//! - `plugin_boot_tests.rs::test_AC_4_VP_PLUGIN_004_unsigned_plugin_durable_audit_entry`
//! - `bc_2_05_012_audit_init.rs::test_BC_2_05_012_sentinel_schema_has_required_fields`
//!
//! Each test constructs `BootAuditWriter` over a real temp `RocksDbBackend`
//! (MED-5 precedent: isolated per-test dirs to avoid parallel RocksDB LOCK
//! collisions), calls `write_tool_call`, then scans the `audit_buffer` CF and
//! asserts the payload fields survive the round-trip.

#![allow(non_snake_case)]

use std::sync::Arc;

use prism_bin::boot::BootAuditWriter;
use prism_core::StorageDomain;
use prism_query::write_dispatch::AuditWriter;
use prism_storage::{
    audit_buffer::AuditEntry as StorageAuditEntry, backend::RocksStorageBackend,
    rocksdb_backend::RocksDbBackend,
};

/// Open a fresh temp RocksDB backend and wrap it in a `BootAuditWriter`.
fn make_writer() -> (tempfile::TempDir, Arc<RocksDbBackend>, BootAuditWriter) {
    let state_dir = tempfile::tempdir().expect("create temp state dir");
    let backend = Arc::new(
        RocksDbBackend::open(state_dir.path().to_path_buf())
            .expect("RocksDbBackend::open must succeed"),
    );
    let writer = BootAuditWriter::new(Arc::clone(&backend));
    (state_dir, backend, writer)
}

/// Scan the `audit_buffer` CF and return all decoded `mcp.tool.called` entries.
fn read_tool_call_entries(backend: &RocksDbBackend) -> Vec<StorageAuditEntry> {
    let entries = backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .expect("scan of audit_buffer CF must succeed");

    let mut decoded_entries = Vec::new();
    for (_key, value) in &entries {
        let decoded: Result<(StorageAuditEntry, _), _> =
            bincode::serde::decode_from_slice(value, bincode::config::standard());
        if let Ok((entry, _)) = decoded {
            if entry
                .payload
                .get("event_type")
                .map(|v| v == "mcp.tool.called")
                .unwrap_or(false)
            {
                decoded_entries.push(entry);
            }
        }
    }
    decoded_entries
}

/// BC-2.05.001 / CRIT-005 / MCP-02: `write_tool_call` with outcome
/// `"invoked"` and a present `client_id` persists a durable `mcp.tool.called`
/// entry to the `audit_buffer` CF whose payload fields (tool_name, client_id,
/// outcome) survive the round-trip.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_call_invoked_cf_readback() {
    let (_state_dir, backend, writer) = make_writer();

    writer
        .write_tool_call("prism_query", Some("acme"), "invoked")
        .await
        .expect("write_tool_call must succeed against a healthy backend");

    let entries = read_tool_call_entries(&backend);
    assert_eq!(
        entries.len(),
        1,
        "P1-03 (MCP-02): exactly one mcp.tool.called entry must be durably \
         persisted in the audit_buffer CF after one write_tool_call"
    );

    let payload = &entries[0].payload;
    assert_eq!(
        payload.get("event_type").map(String::as_str),
        Some("mcp.tool.called"),
        "P1-03: payload event_type must be 'mcp.tool.called'"
    );
    assert_eq!(
        payload.get("tool_name").map(String::as_str),
        Some("prism_query"),
        "P1-03: payload tool_name must round-trip"
    );
    assert_eq!(
        payload.get("client_id").map(String::as_str),
        Some("acme"),
        "P1-03: payload client_id must round-trip when present"
    );
    assert_eq!(
        payload.get("outcome").map(String::as_str),
        Some("invoked"),
        "P1-03: payload outcome must round-trip as 'invoked'"
    );
    assert!(
        !entries[0].trace_id.is_empty(),
        "P1-03: durable entry must carry a non-empty trace_id"
    );
    assert!(
        entries[0].timestamp_ns > 0,
        "P1-03: durable entry must carry a non-zero timestamp_ns"
    );
}

/// BC-2.05.001 / CRIT-005 / MCP-02 + BC-2.05.002: `write_tool_call`
/// with outcome `"rejected_injection"` and NO client_id persists a durable
/// entry whose `client_id` field carries the BC-2.05.002 `"MISSING"` sentinel.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_call_rejected_injection_missing_client_id_cf_readback() {
    let (_state_dir, backend, writer) = make_writer();

    writer
        .write_tool_call("prism_query", None, "rejected_injection")
        .await
        .expect("write_tool_call must succeed against a healthy backend");

    let entries = read_tool_call_entries(&backend);
    assert_eq!(
        entries.len(),
        1,
        "P1-03 (MCP-03): exactly one mcp.tool.called entry must be durably \
         persisted in the audit_buffer CF after one rejected_injection write"
    );

    let payload = &entries[0].payload;
    assert_eq!(
        payload.get("outcome").map(String::as_str),
        Some("rejected_injection"),
        "P1-03: payload outcome must round-trip as 'rejected_injection'"
    );
    assert_eq!(
        payload.get("client_id").map(String::as_str),
        Some("MISSING"),
        "P1-03 (BC-2.05.002): payload client_id must carry the 'MISSING' \
         sentinel when the request had no client_id"
    );
    assert_eq!(
        payload.get("tool_name").map(String::as_str),
        Some("prism_query"),
        "P1-03: payload tool_name must round-trip"
    );
}

/// BC-2.05.001 / MCP-02: two sequential `write_tool_call` invocations
/// (one per outcome) persist two distinct durable entries — the append pattern
/// must not overwrite prior records.
#[tokio::test]
async fn test_BC_2_05_001_write_tool_call_appends_distinct_entries() {
    let (_state_dir, backend, writer) = make_writer();

    writer
        .write_tool_call("list_capabilities", Some("acme"), "invoked")
        .await
        .expect("first write_tool_call must succeed");
    writer
        .write_tool_call("prism_query", None, "rejected_injection")
        .await
        .expect("second write_tool_call must succeed");

    let entries = read_tool_call_entries(&backend);
    assert_eq!(
        entries.len(),
        2,
        "P1-03: both tool-call records must be present (append, not overwrite)"
    );

    let outcomes: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.payload.get("outcome").map(String::as_str))
        .collect();
    assert!(
        outcomes.contains(&"invoked") && outcomes.contains(&"rejected_injection"),
        "P1-03: both outcomes must be present in the audit_buffer CF; got: {outcomes:?}"
    );

    let trace_ids: std::collections::BTreeSet<&str> =
        entries.iter().map(|e| e.trace_id.as_str()).collect();
    assert_eq!(
        trace_ids.len(),
        2,
        "P1-03: each tool-call record must carry a distinct trace_id"
    );
}
