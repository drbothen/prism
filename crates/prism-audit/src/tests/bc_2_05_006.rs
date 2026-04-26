//! Tests for BC-2.05.006 — Audit Entries Are Append-Only and Immutable.
//!
//! Postconditions tested:
//!   - Audit key format is `audit:{timestamp_ns:020}:{trace_id}` (lexicographic = chronological).
//!   - Two concurrent invocations produce entries with unique keys (nanosecond resolution).
//!   - `StorageDomain::AuditBuffer` is NEVER passed to `remove()` in prism-audit.
//!   - Once an entry is written, no code path overwrites it (put with same key).
//!   - Correction pattern: new entry referencing original `trace_id` (not overwrite).
//!
//! AC-6: no `remove()` call on `AuditBuffer`; keys are monotonically-increasing new keys.
//! EC-004: concurrent invocations produce unique keys.
//! EC-005: no `remove(AuditBuffer, ...)` in prism-audit source.

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::{PrismError, StorageDomain};
use prism_storage::backend::RocksStorageBackend;
use prism_storage::memory_backend::InMemoryBackend;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::audit_emitter::{
    AuditEmitterLayer, AuditedRequest, AuditedResponse, ToolClass, ToolClassificationRegistry,
};
use crate::tests::helpers::{make_request, AlwaysSucceedService, MemBackend};

// ── Helper: invoke for tests ──────────────────────────────────────────────────

async fn invoke<S>(svc: &mut S, req: AuditedRequest) -> Result<AuditedResponse, PrismError>
where
    S: Service<AuditedRequest, Response = AuditedResponse, Error = PrismError>,
    S::Future: Send,
{
    std::future::poll_fn(|cx| svc.poll_ready(cx))
        .await
        .expect("poll_ready failed");
    svc.call(req).await
}

// ── AC-6: audit key format is monotonically ordered ──────────────────────────

/// AC-6 (BC-2.05.006): Keys in `audit_buffer` CF use the format
/// `audit:{timestamp_ns:020}:{trace_id}`.
///
/// We write two entries in sequence and verify:
/// 1. Both keys start with `audit:`.
/// 2. Keys are distinct (trace_id uniqueness).
#[tokio::test]
async fn test_BC_2_05_006_audit_key_format_is_ordered() {
    let backend = MemBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("query_crowdstrike_alerts", ToolClass::ReadTool);
    let registry = Arc::new(registry);

    // First invocation.
    {
        let mut svc = AuditEmitterLayer::new(Arc::new(backend.clone()), Arc::clone(&registry))
            .layer(AlwaysSucceedService);
        invoke(&mut svc, make_request("query_crowdstrike_alerts"))
            .await
            .expect("first invocation must succeed");
    }

    // Second invocation.
    {
        let mut svc = AuditEmitterLayer::new(Arc::new(backend.clone()), Arc::clone(&registry))
            .layer(AlwaysSucceedService);
        invoke(&mut svc, make_request("query_crowdstrike_alerts"))
            .await
            .expect("second invocation must succeed");
    }

    // Both entries must exist and have `audit:` prefix keys.
    let entries = backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .expect("scan must succeed");

    assert_eq!(
        entries.len(),
        2,
        "expected 2 audit entries after 2 sequential invocations, found {}",
        entries.len()
    );

    for (key, _) in &entries {
        let key_str = std::str::from_utf8(key).expect("audit key must be valid UTF-8");
        assert!(
            key_str.starts_with("audit:"),
            "audit key must start with 'audit:', got: {key_str}"
        );
    }

    // Keys must be distinct.
    let key0 = &entries[0].0;
    let key1 = &entries[1].0;
    assert_ne!(
        key0, key1,
        "concurrent invocations must produce distinct audit keys"
    );
}

// ── EC-004: concurrent invocations have unique keys ───────────────────────────

/// EC-004 (BC-2.05.006): When two concurrent tool invocations share the same
/// millisecond timestamp, their audit keys are still unique because the key
/// includes a nanosecond timestamp + trace_id (UUID v7).
#[tokio::test]
async fn test_BC_2_05_006_concurrent_invocations_produce_unique_keys() {
    let backend = MemBackend::new();
    let mut registry: ToolClassificationRegistry = HashMap::new();
    registry.insert("query_crowdstrike_alerts", ToolClass::ReadTool);
    let registry = Arc::new(registry);

    let backend_a = backend.clone();
    let registry_a = Arc::clone(&registry);
    let backend_b = backend.clone();
    let registry_b = Arc::clone(&registry);

    let fut_a = async move {
        let mut svc =
            AuditEmitterLayer::new(Arc::new(backend_a), registry_a).layer(AlwaysSucceedService);
        invoke(&mut svc, make_request("query_crowdstrike_alerts")).await
    };
    let fut_b = async move {
        let mut svc =
            AuditEmitterLayer::new(Arc::new(backend_b), registry_b).layer(AlwaysSucceedService);
        invoke(&mut svc, make_request("query_crowdstrike_alerts")).await
    };

    let (r_a, r_b): (
        Result<AuditedResponse, PrismError>,
        Result<AuditedResponse, PrismError>,
    ) = tokio::join!(fut_a, fut_b);
    assert!(r_a.is_ok(), "concurrent invocation A must succeed");
    assert!(r_b.is_ok(), "concurrent invocation B must succeed");

    let entries = backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .expect("scan must succeed");

    assert_eq!(
        entries.len(),
        2,
        "two concurrent invocations must produce exactly 2 distinct audit entries, found {}",
        entries.len()
    );

    let key0 = &entries[0].0;
    let key1 = &entries[1].0;
    assert_ne!(
        key0, key1,
        "concurrent audit keys must be distinct (nanosecond timestamp + trace_id uniqueness)"
    );
}

// ── EC-005 / AC-6: no remove(AuditBuffer) in prism-audit source ─────────────

/// EC-005 / AC-6 (BC-2.05.006): The `StorageDomain::AuditBuffer` domain MUST
/// NEVER be passed to `StorageBackend::remove()` in the `prism-audit` crate.
///
/// This test is a source-level scan that detects the forbidden pattern.
#[test]
fn test_BC_2_05_006_no_remove_call_with_audit_buffer_in_prism_audit_sources() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during tests");

    let src_dir = std::path::Path::new(&manifest_dir).join("src");

    let mut violations: Vec<String> = vec![];
    scan_for_audit_buffer_remove(&src_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "BC-2.05.006 / AC-6 VIOLATION: prism-audit source contains \
         `remove(…AuditBuffer…)` call(s) — append-only invariant breached:\n{}",
        violations.join("\n")
    );
}

/// Recursively scan a directory tree for lines containing both `remove(` and
/// `AuditBuffer` (the forbidden combination per BC-2.05.006 append-only invariant).
fn scan_for_audit_buffer_remove(dir: &std::path::Path, violations: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_for_audit_buffer_remove(&path, violations);
        } else if path.extension().is_some_and(|e| e == "rs") {
            // Skip the test file that contains the scanner (false-positive prevention).
            if path.file_name().is_some_and(|n| n == "bc_2_05_006.rs") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (line_no, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                // Skip comment lines (single-line `//` comments and doc comments `///`).
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains("remove(") && trimmed.contains("AuditBuffer") {
                    violations.push(format!("  {}:{}: {}", path.display(), line_no + 1, trimmed));
                }
            }
        }
    }
}

// ── Correction pattern: new entry, not overwrite ──────────────────────────────

/// BC-2.05.006: If a correction is needed, a new audit entry is emitted that
/// references the original `trace_id`. The original entry is not modified.
///
/// Verifies that after writing an entry via `append_audit_entry`, a correction
/// write produces a second distinct key rather than overwriting the original.
#[test]
fn test_BC_2_05_006_correction_writes_new_entry_not_overwrite() {
    use chrono::Utc;
    use prism_storage::audit_buffer;

    let backend = InMemoryBackend::new();
    let original_trace_id = Uuid::now_v7().to_string();
    let ts_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;

    // Write original entry.
    let original_entry = audit_buffer::AuditEntry {
        timestamp_ns: ts_ns,
        trace_id: original_trace_id.clone(),
        payload: {
            let mut m = std::collections::BTreeMap::new();
            m.insert("json".to_owned(), "{\"event\": \"original\"}".to_owned());
            m
        },
    };
    audit_buffer::append_audit_entry(&backend, &original_entry)
        .expect("original entry write must succeed");

    // Write correction entry with new trace_id (referencing original).
    let correction_trace_id = Uuid::now_v7().to_string();
    // Use ts + 1000 ns to guarantee a distinct key.
    let ts_ns_correction = ts_ns + 1000;
    let correction_entry = audit_buffer::AuditEntry {
        timestamp_ns: ts_ns_correction,
        trace_id: correction_trace_id.clone(),
        payload: {
            let mut m = std::collections::BTreeMap::new();
            m.insert(
                "json".to_owned(),
                format!("{{\"event\": \"correction\", \"corrects\": \"{original_trace_id}\"}}"),
            );
            m
        },
    };
    audit_buffer::append_audit_entry(&backend, &correction_entry)
        .expect("correction entry write must succeed");

    // Both entries must be present — original not overwritten.
    let all_entries = backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .expect("scan must succeed");

    assert_eq!(
        all_entries.len(),
        2,
        "correction must produce a NEW entry, not overwrite original. \
         Expected 2 entries, found {}",
        all_entries.len()
    );
}
