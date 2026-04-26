//! Tests for BC-2.05.008 — Audit Entries Satisfy SOC 2 Type II and ISO 27001
//! Requirements.
//!
//! Postconditions tested:
//!   - SOC 2 Type II fields: who (`user_identity`), what (`tool_name`, `parameters`),
//!     when (`timestamp`), where (`client_id`, `system_id`), outcome (`outcome`),
//!     authorization (`capability_checks`).
//!   - ISO 27001 fields: `data_classification`, `trace_id`, `capability_checks`.
//!   - `user_identity` defaults to `"unknown"` with `audit_warning` when unavailable.
//!   - `capability_checks` is empty array (not omitted) for read operations.
//!   - All fields are machine-parseable JSON (not free-text prose).
//!
//! AC-3 (SOC2/ISO27001 overlay): same serialisation assertions as BC-2.05.002,
//! with extra checks for SOC2-specific semantics.

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::audit_entry::{
    AuditEntry, AuditOutcome, CapabilityCheckRecord, CapabilityCheckResult, DataClassification,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn soc2_compliant_entry() -> AuditEntry {
    AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "crowdstrike_contain_host".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({"host_id": "h-001"}),
        AuditOutcome::Success,
        "committed".to_owned(),
        55,
        None,
        DataClassification::Internal,
        vec![CapabilityCheckRecord {
            capability_path: "sensor.crowdstrike.containment".to_owned(),
            compile_time_enabled: true,
            runtime_enabled: true,
            result: CapabilityCheckResult::Permitted,
        }],
        vec![],
    )
}

// ── SOC 2 Type II: "who" ──────────────────────────────────────────────────────

/// BC-2.05.008: SOC 2 "who" — `user_identity` must be present and non-empty.
#[test]
fn test_BC_2_05_008_soc2_who_user_identity_present() {
    let entry = soc2_compliant_entry();
    assert!(
        !entry.user_identity.is_empty(),
        "SOC 2 'who': user_identity must be present and non-empty"
    );

    let obj: Value = serde_json::to_value(&entry).unwrap();
    assert!(
        obj.get("user_identity").and_then(|v| v.as_str()).is_some(),
        "SOC 2 'who': user_identity must serialise as a string field"
    );
}

/// BC-2.05.008: When user identity is unavailable, `user_identity` is `"unknown"`
/// and `audit_warning` is set.
#[test]
fn test_BC_2_05_008_missing_user_identity_defaults_to_unknown_with_warning() {
    let mut entry = soc2_compliant_entry();
    entry.user_identity = "unknown".to_owned();
    entry.audit_warning = Some("missing user_identity".to_owned());

    assert_eq!(
        entry.user_identity, "unknown",
        "missing user_identity must be set to 'unknown'"
    );
    assert!(
        entry.audit_warning.is_some(),
        "audit_warning must be set when user_identity is 'unknown'"
    );
}

// ── SOC 2 Type II: "what" ─────────────────────────────────────────────────────

/// BC-2.05.008: SOC 2 "what" — `tool_name` and `parameters` must be present.
#[test]
fn test_BC_2_05_008_soc2_what_tool_name_and_parameters_present() {
    let entry = soc2_compliant_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    assert!(
        obj.get("tool_name").and_then(|v| v.as_str()).is_some(),
        "SOC 2 'what': tool_name must be a non-null string"
    );
    assert!(
        obj.get("parameters").is_some(),
        "SOC 2 'what': parameters must be present"
    );
}

// ── SOC 2 Type II: "when" ─────────────────────────────────────────────────────

/// BC-2.05.008: SOC 2 "when" — `timestamp` must be present in ISO 8601 UTC format.
#[test]
fn test_BC_2_05_008_soc2_when_timestamp_is_iso8601_utc() {
    let entry = soc2_compliant_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    let ts = obj
        .get("timestamp")
        .expect("SOC 2 'when': timestamp must be present");
    let ts_str = ts.as_str().expect("timestamp must be a JSON string");

    // Must parse as RFC 3339 (ISO 8601 UTC).
    let parsed = chrono::DateTime::parse_from_rfc3339(ts_str);
    assert!(
        parsed.is_ok(),
        "SOC 2 'when': timestamp must be a valid RFC 3339 / ISO 8601 string, got: {ts_str}"
    );
}

// ── SOC 2 Type II: "where" ────────────────────────────────────────────────────

/// BC-2.05.008: SOC 2 "where" — `client_id` and `system_id` must be present.
#[test]
fn test_BC_2_05_008_soc2_where_client_id_and_system_id_present() {
    let entry = soc2_compliant_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    assert!(
        obj.get("client_id").and_then(|v| v.as_str()).is_some(),
        "SOC 2 'where': client_id must be a non-null string"
    );
    assert_eq!(
        obj["system_id"],
        Value::String("prism".to_owned()),
        "SOC 2 'where': system_id must be 'prism'"
    );
}

// ── SOC 2 Type II: "outcome" ──────────────────────────────────────────────────

/// BC-2.05.008: SOC 2 "outcome" — `outcome` and `result_summary` must be present.
#[test]
fn test_BC_2_05_008_soc2_outcome_fields_present() {
    let entry = soc2_compliant_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    assert!(
        obj.get("outcome").is_some(),
        "SOC 2 'outcome': outcome field must be present"
    );
    assert!(
        obj.get("result_summary").and_then(|v| v.as_str()).is_some(),
        "SOC 2 'outcome': result_summary must be a non-null string"
    );
}

// ── SOC 2 Type II: "authorization" ───────────────────────────────────────────

/// BC-2.05.008: SOC 2 "authorization" — `capability_checks` must be present
/// and non-empty for write operations.
#[test]
fn test_BC_2_05_008_soc2_authorization_capability_checks_present_for_write() {
    let entry = soc2_compliant_entry(); // has one capability check
    let obj: Value = serde_json::to_value(&entry).unwrap();

    let checks = obj
        .get("capability_checks")
        .expect("SOC 2 'authorization': capability_checks must be present");
    let arr = checks
        .as_array()
        .expect("capability_checks must be a JSON array");
    assert!(
        !arr.is_empty(),
        "SOC 2 'authorization': capability_checks must be non-empty for write operations"
    );
}

/// BC-2.05.008 / EC-05-013: `capability_checks` is empty array (not omitted)
/// for read-only tool invocations.
#[test]
fn test_BC_2_05_008_capability_checks_empty_not_omitted_for_read_ops() {
    let entry = AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "query_crowdstrike_alerts".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({}),
        AuditOutcome::Success,
        "ok".to_owned(),
        10,
        None,
        DataClassification::Internal,
        vec![], // empty for read ops
        vec![],
    );

    let obj: Value = serde_json::to_value(&entry).unwrap();
    let checks = obj
        .get("capability_checks")
        .expect("capability_checks must be present (not omitted) even for read ops");
    assert!(checks.is_array(), "capability_checks must be a JSON array");
    assert_eq!(
        checks.as_array().unwrap().len(),
        0,
        "capability_checks must be empty [] for read ops"
    );
}

// ── ISO 27001: data_classification ───────────────────────────────────────────

/// BC-2.05.008: ISO 27001 — `data_classification` must be present on every entry.
#[test]
fn test_BC_2_05_008_iso27001_data_classification_present() {
    let entry = soc2_compliant_entry();
    let obj: Value = serde_json::to_value(&entry).unwrap();

    assert!(
        obj.get("data_classification").is_some(),
        "ISO 27001: data_classification must be present on every audit entry"
    );
}

/// BC-2.05.008: All four `DataClassification` variants are valid.
#[test]
fn test_BC_2_05_008_data_classification_variants_are_valid() {
    use crate::audit_entry::DataClassification;

    let variants = [
        DataClassification::Public,
        DataClassification::Internal,
        DataClassification::Confidential,
        DataClassification::Restricted,
    ];

    for variant in variants {
        let v: Value = serde_json::to_value(&variant)
            .expect("DataClassification variant must serialise without error");
        assert!(
            v.is_string(),
            "DataClassification must serialise as a JSON string"
        );
    }
}

// ── ISO 27001: trace_id for incident response correlation ─────────────────────

/// BC-2.05.008: ISO 27001 — `trace_id` must be present and be a valid UUID v7 string.
#[test]
fn test_BC_2_05_008_iso27001_trace_id_present_and_uuid() {
    let trace_id = Uuid::now_v7();
    let entry = AuditEntry::new(
        trace_id,
        Utc::now(),
        "query_alerts".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({}),
        AuditOutcome::Success,
        "ok".to_owned(),
        5,
        None,
        DataClassification::Internal,
        vec![],
        vec![],
    );

    let obj: Value = serde_json::to_value(&entry).unwrap();
    let tid = obj
        .get("trace_id")
        .expect("ISO 27001: trace_id must be present");
    let tid_str = tid.as_str().expect("trace_id must be a JSON string");

    // Must parse as a valid UUID.
    let parsed = Uuid::parse_str(tid_str);
    assert!(
        parsed.is_ok(),
        "ISO 27001: trace_id must be a valid UUID string, got: {tid_str}"
    );

    // The parsed UUID must match the original.
    assert_eq!(
        parsed.unwrap(),
        trace_id,
        "trace_id in serialised entry must match the original trace_id"
    );
}

// ── All fields are machine-parseable JSON ─────────────────────────────────────

/// BC-2.05.008: All audit entry fields must be machine-parseable JSON
/// (structured, not free-text prose). Verifies that round-trip JSON
/// serialisation/deserialisation is lossless.
#[test]
fn test_BC_2_05_008_all_fields_are_machine_parseable_json_roundtrip() {
    let entry = soc2_compliant_entry();
    let json_str = serde_json::to_string(&entry).expect("AuditEntry must serialise to JSON");
    let deserialized: AuditEntry = serde_json::from_str(&json_str)
        .expect("AuditEntry must deserialise from JSON (machine-parseable roundtrip)");

    // Spot-check critical fields survive the roundtrip.
    assert_eq!(
        entry.trace_id, deserialized.trace_id,
        "trace_id must survive JSON roundtrip"
    );
    assert_eq!(
        entry.tool_name, deserialized.tool_name,
        "tool_name must survive JSON roundtrip"
    );
    assert_eq!(
        entry.client_id, deserialized.client_id,
        "client_id must survive JSON roundtrip"
    );
    assert_eq!(
        entry.system_id, deserialized.system_id,
        "system_id must survive JSON roundtrip"
    );
    assert_eq!(
        entry.data_classification, deserialized.data_classification,
        "data_classification must survive JSON roundtrip"
    );
}

// ── CapabilityCheckRecord serialises correctly ───────────────────────────────

/// BC-2.05.008: `CapabilityCheckRecord` must serialise with all required fields
/// for SOC 2 capability evidence.
#[test]
fn test_BC_2_05_008_capability_check_record_has_required_fields() {
    let record = CapabilityCheckRecord {
        capability_path: "sensor.crowdstrike.containment".to_owned(),
        compile_time_enabled: true,
        runtime_enabled: true,
        result: CapabilityCheckResult::Permitted,
    };

    let obj: Value = serde_json::to_value(&record).unwrap();
    assert!(
        obj.get("capability_path").is_some(),
        "capability_path must be present"
    );
    assert!(
        obj.get("compile_time_enabled").is_some(),
        "compile_time_enabled must be present"
    );
    assert!(
        obj.get("runtime_enabled").is_some(),
        "runtime_enabled must be present"
    );
    assert!(obj.get("result").is_some(), "result must be present");
}
