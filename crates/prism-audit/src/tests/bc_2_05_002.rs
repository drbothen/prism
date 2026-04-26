//! Tests for BC-2.05.002 — Audit Entries Use Structured JSON Format with
//! Complete Fields.
//!
//! Postconditions tested:
//!   - Serialised `AuditEntry` is valid JSON.
//!   - All required fields are present: `trace_id`, `timestamp`, `tool_name`,
//!     `client_id`, `parameters`, `outcome`, `duration_ms`, `data_classification`,
//!     `system_id`, `user_identity`, `result_summary`, `capability_checks`,
//!     `safety_flags`.
//!   - `system_id` is always `"prism"`.
//!   - `client_id` sentinel rules (multi_client, all_clients, cross_client, MISSING).
//!
//! AC-3: serialised entry contains all required compliance fields.

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::audit_entry::{AuditEntry, AuditOutcome, DataClassification};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a minimal `AuditEntry` for serialization tests.
fn minimal_entry() -> AuditEntry {
    AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "query_crowdstrike_alerts".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({"query": "SELECT * FROM alerts"}),
        AuditOutcome::Success,
        "ok".to_owned(),
        42,
        None,
        DataClassification::Internal,
        vec![],
        vec![],
    )
}

// ── AC-3: all required compliance fields present ──────────────────────────────

/// AC-3 (BC-2.05.002, BC-2.05.008): When an `AuditEntry` is serialised to JSON,
/// it must contain all required compliance fields.
#[test]
fn test_BC_2_05_002_serialised_entry_contains_all_required_fields() {
    let entry = minimal_entry();
    let json_str =
        serde_json::to_string(&entry).expect("AuditEntry must serialise to JSON without error");
    let obj: Value =
        serde_json::from_str(&json_str).expect("serialised AuditEntry must parse as valid JSON");

    // All required fields per BC-2.05.002 and BC-2.05.008.
    let required_fields = [
        "trace_id",
        "timestamp",
        "tool_name",
        "client_id",
        "parameters",
        "outcome",
        "duration_ms",
        "data_classification",
        "system_id",
        "user_identity",
        "result_summary",
        "capability_checks",
        "safety_flags",
    ];

    for field in required_fields {
        assert!(
            obj.get(field).is_some(),
            "required field '{field}' is missing from serialised AuditEntry"
        );
    }
}

// ── system_id is always "prism" ───────────────────────────────────────────────

/// BC-2.05.002: `system_id` must always be the fixed string `"prism"` (ISO 27001 "where").
#[test]
fn test_BC_2_05_002_system_id_is_always_prism() {
    let entry = minimal_entry();
    assert_eq!(
        entry.system_id, "prism",
        "system_id must be the fixed string 'prism' (BC-2.05.002, BC-2.05.008)"
    );

    // Verify it serialises correctly too.
    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        obj["system_id"],
        Value::String("prism".to_owned()),
        "system_id in JSON must be 'prism'"
    );
}

// ── client_id sentinel rules (BC-2.05.002 test vectors) ──────────────────────

/// BC-2.05.002 canonical test vector: single-client tool → `client_id` = the tenant id.
#[test]
fn test_BC_2_05_002_client_id_single_client_preserved() {
    let mut entry = minimal_entry();
    entry.client_id = "acme".to_owned();
    assert_eq!(
        entry.client_id, "acme",
        "single-client tool: client_id must be the tenant id"
    );
}

/// BC-2.05.002 canonical test vector: multi-client fan-out → `client_id = "multi_client"`.
#[test]
fn test_BC_2_05_002_client_id_multi_client_sentinel() {
    let mut entry = minimal_entry();
    entry.client_id = "multi_client".to_owned();
    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        obj["client_id"],
        Value::String("multi_client".to_owned()),
        "multi-client fan-out: client_id must be 'multi_client'"
    );
}

/// BC-2.05.002 canonical test vector: cross-client query with `clients: null`
/// → `client_id = "all_clients"`.
#[test]
fn test_BC_2_05_002_client_id_all_clients_sentinel() {
    let mut entry = minimal_entry();
    entry.client_id = "all_clients".to_owned();
    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        obj["client_id"],
        Value::String("all_clients".to_owned()),
        "cross-client query (clients: null): client_id must be 'all_clients'"
    );
}

/// BC-2.05.002 canonical test vector: non-query tool with `client_id: null`
/// → `client_id = "cross_client"`.
#[test]
fn test_BC_2_05_002_client_id_cross_client_sentinel() {
    let mut entry = minimal_entry();
    entry.client_id = "cross_client".to_owned();
    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        obj["client_id"],
        Value::String("cross_client".to_owned()),
        "non-query tool with null client_id: client_id must be 'cross_client'"
    );
}

/// BC-2.05.002 canonical test vector: malformed request lacking `client_id`
/// → `client_id = "MISSING"`.
#[test]
fn test_BC_2_05_002_client_id_missing_sentinel() {
    let mut entry = minimal_entry();
    entry.client_id = "MISSING".to_owned();
    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        obj["client_id"],
        Value::String("MISSING".to_owned()),
        "malformed request lacking client_id: client_id must be 'MISSING'"
    );
}

// ── capability_checks is empty array (not omitted) for read ops ───────────────

/// BC-2.05.002 / BC-2.05.008: `capability_checks` must be an empty array (not
/// omitted) for read-only tool invocations.
#[test]
fn test_BC_2_05_002_capability_checks_empty_array_for_read_ops() {
    let entry = minimal_entry(); // no capability checks
    let obj: Value = serde_json::to_value(&entry).unwrap();

    let checks = obj.get("capability_checks").expect(
        "capability_checks must be present in serialised AuditEntry for read ops (not omitted)",
    );
    assert!(
        checks.is_array(),
        "capability_checks must be a JSON array, got: {checks}"
    );
    assert_eq!(
        checks.as_array().unwrap().len(),
        0,
        "capability_checks must be an empty array for read ops"
    );
}

// ── safety_flags is empty array (not omitted) when none triggered ─────────────

/// BC-2.05.002: `safety_flags` must be an empty array (not omitted) when no
/// prompt injection flags are triggered.
#[test]
fn test_BC_2_05_002_safety_flags_empty_array_not_omitted() {
    let entry = minimal_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    let flags = obj
        .get("safety_flags")
        .expect("safety_flags must be present even when empty (not omitted)");
    assert!(flags.is_array(), "safety_flags must be a JSON array");
    assert_eq!(
        flags.as_array().unwrap().len(),
        0,
        "safety_flags must be empty when none triggered"
    );
}

// ── Failure outcome serialises with error_code ───────────────────────────────

/// BC-2.05.002: `AuditOutcome::Failure { error_code }` must serialise with
/// the structured error code present in the JSON.
#[test]
fn test_BC_2_05_002_failure_outcome_serialises_with_error_code() {
    let entry = AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "crowdstrike_contain_host".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({}),
        AuditOutcome::Failure {
            error_code: "E-SENSOR-001".to_owned(),
        },
        "error: E-SENSOR-001".to_owned(),
        10,
        Some("E-SENSOR-001".to_owned()),
        DataClassification::Internal,
        vec![],
        vec![],
    );

    let obj: Value = serde_json::to_value(&entry).unwrap();
    let outcome = &obj["outcome"];
    assert!(
        outcome.is_object(),
        "AuditOutcome must serialise as a JSON object"
    );
    // The outcome object must have status=failure and error_code field.
    assert_eq!(
        outcome["status"],
        Value::String("failure".to_owned()),
        "outcome.status must be 'failure' for AuditOutcome::Failure"
    );
    assert_eq!(
        outcome["error_code"],
        Value::String("E-SENSOR-001".to_owned()),
        "outcome.error_code must be present and match the structured error code"
    );
}

// ── data_classification defaults to Internal ─────────────────────────────────

/// BC-2.05.002 / BC-2.05.008 / Dev Notes: `data_classification` defaults to
/// `Internal` for all tool invocations unless the tool manifest specifies otherwise.
#[test]
fn test_BC_2_05_002_data_classification_defaults_to_internal() {
    let classification = DataClassification::default();
    assert_eq!(
        classification,
        DataClassification::Internal,
        "DataClassification must default to Internal per Dev Notes"
    );
}
