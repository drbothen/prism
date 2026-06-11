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

// ─── P5-03 (2026-06-10 review pass-5) — BC-2.05.009 capability fields ────────
//
// BC-2.05.009 postcondition: each capability-check audit entry records
// `capability_path`, `compile_time_enabled`, `runtime_enabled`, and `result`.
// `BootAuditWriter::write_intent` previously bound `_capability_check` and
// dropped it; these tests drive the field derivation and the tracing emission.

/// Build a `WritePlan` with hyphenated sensor/verb names to prove the
/// capability-path derivation applies the same hyphen→underscore sanitization
/// as `WriteExecutor::execute` (prism-query write_pipeline).
fn make_capability_plan() -> prism_query::WritePlan {
    prism_query::WritePlan {
        verb: "contain-host".to_owned(),
        sensor: "test-sensor".to_owned(),
        target_table: "test_sensor_hosts".to_owned(),
        dml_operation: None,
        has_explicit_limit: true,
        explicit_limit: Some(1),
        has_where_clause: true,
        params: std::collections::HashMap::new(),
    }
}

fn make_capability_context() -> prism_query::QueryContext {
    prism_query::QueryContext {
        client_id: "acme".to_owned(),
        org_slug: prism_core::OrgSlug::new("acme").expect("valid org slug"),
        dry_run: false,
        confirmation_token_id: None,
        analyst_id: None,
    }
}

/// P5-03 / BC-2.05.009: `Allowed` → both tiers true, result "permitted",
/// capability path derived from the plan with hyphen→underscore sanitization.
#[test]
fn test_BC_2_05_009_capability_audit_fields_allowed() {
    let plan = make_capability_plan();
    let (path, compile_time_enabled, runtime_enabled, result) =
        BootAuditWriter::capability_audit_fields(
            &plan,
            &prism_security::feature_flag::CapabilityCheckResult::Allowed,
        );
    assert_eq!(
        path, "sensor.test_sensor.contain_host",
        "BC-2.05.009: Allowed capability path must be derived from the plan \
         with the write_pipeline hyphen→underscore sanitization"
    );
    assert!(
        compile_time_enabled && runtime_enabled,
        "BC-2.05.009: Allowed requires both tiers enabled (deny-by-default rule)"
    );
    assert_eq!(
        result, "permitted",
        "BC-2.05.009: final result must be 'permitted' only when both tiers allow"
    );
}

/// P5-03 / BC-2.05.009: `DeniedCompileTime` → compile tier false, runtime tier
/// false (never reached; deny-by-default), result "denied", path from variant.
#[test]
fn test_BC_2_05_009_capability_audit_fields_denied_compile_time() {
    let plan = make_capability_plan();
    let check = prism_security::feature_flag::CapabilityCheckResult::DeniedCompileTime {
        capability: "sensor.test_sensor.contain_host".to_owned(),
        client_id: "acme".to_owned(),
        resolution_trace: vec!["compile_gate=Absent".to_owned()],
    };
    let (path, compile_time_enabled, runtime_enabled, result) =
        BootAuditWriter::capability_audit_fields(&plan, &check);
    assert_eq!(path, "sensor.test_sensor.contain_host");
    assert!(
        !compile_time_enabled && !runtime_enabled,
        "BC-2.05.009: compile-tier denial records compile_time_enabled=false; \
         the runtime tier is never reached (deny-by-default → false)"
    );
    assert_eq!(result, "denied");
}

/// P5-03 / BC-2.05.009: `DeniedRuntime` → compile tier true (runtime tier is
/// only reached after it passes), runtime tier false, result "denied".
#[test]
fn test_BC_2_05_009_capability_audit_fields_denied_runtime() {
    let plan = make_capability_plan();
    let check = prism_security::feature_flag::CapabilityCheckResult::DeniedRuntime {
        capability: "sensor.test_sensor.contain_host".to_owned(),
        client_id: "acme".to_owned(),
        resolution_trace: vec!["sensor.test_sensor.contain_host=Deny".to_owned()],
    };
    let (path, compile_time_enabled, runtime_enabled, result) =
        BootAuditWriter::capability_audit_fields(&plan, &check);
    assert_eq!(path, "sensor.test_sensor.contain_host");
    assert!(
        compile_time_enabled && !runtime_enabled,
        "BC-2.05.009: runtime-tier denial implies the compile tier passed \
         (evaluation order) and the runtime tier denied"
    );
    assert_eq!(result, "denied");
}

/// Helper writer for tracing-subscriber capture (same pattern as
/// prism-mcp server.rs `WriterGuard` — F-PASS7-MED-1 precedent).
struct CaptureWriter(Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("capture buffer lock")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// P5-03 / BC-2.05.009 END-TO-END: the PRODUCTION `write_intent` emission
/// (`write.intent.recorded`) carries the capability fields — not just the
/// helper in isolation.
///
/// Mental-deletion proof: if `write_intent` reverts to binding
/// `_capability_check` and dropping it, the captured output loses the
/// capability_path / tier / result fields and this test fails.
#[tokio::test]
async fn test_BC_2_05_009_write_intent_emission_carries_capability_fields() {
    let (_state_dir, _backend, writer) = make_writer();
    let captured: Arc<std::sync::Mutex<Vec<u8>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let output = {
        let buf = Arc::clone(&captured);
        let make_writer = move || CaptureWriter(Arc::clone(&buf));
        let subscriber = tracing_subscriber::fmt()
            .with_writer(make_writer)
            .with_max_level(tracing::Level::INFO)
            // No ANSI escapes — the assertions below match `key=value` substrings.
            .with_ansi(false)
            .finish();
        // set_default (not with_default): the guard lives across .await points
        // in this async fn (prism-mcp F-PASS7-MED-1 capture precedent).
        let _guard = tracing::subscriber::set_default(subscriber);

        let plan = make_capability_plan();
        let context = make_capability_context();
        writer
            .write_intent(
                &plan,
                &context,
                &prism_security::feature_flag::CapabilityCheckResult::Allowed,
            )
            .await
            .expect("write_intent must succeed");

        let lock = captured.lock().expect("capture buffer lock");
        String::from_utf8_lossy(&lock).to_string()
    };

    assert!(
        output.contains("write.intent.recorded"),
        "write_intent must emit the write.intent.recorded event; got:\n{output}"
    );
    assert!(
        output.contains("sensor.test_sensor.contain_host"),
        "P5-03 / BC-2.05.009: emission must carry capability_path; got:\n{output}"
    );
    assert!(
        output.contains("compile_time_enabled=true"),
        "P5-03 / BC-2.05.009: emission must carry compile_time_enabled; got:\n{output}"
    );
    assert!(
        output.contains("runtime_enabled=true"),
        "P5-03 / BC-2.05.009: emission must carry runtime_enabled; got:\n{output}"
    );
    assert!(
        output.contains("permitted"),
        "P5-03 / BC-2.05.009: emission must carry result=permitted; got:\n{output}"
    );
}
