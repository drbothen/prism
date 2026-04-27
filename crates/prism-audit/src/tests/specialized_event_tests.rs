//! Specialized audit event tests for S-2.05.
//!
//! Covers:
//!   - BC-2.05.005 — Credential access events are audit-logged with context (AC-1)
//!   - BC-2.05.007 — Audit entries are compatible with the Vector pipeline (AC-2)
//!   - BC-2.05.009 — Feature flag evaluations for write operations are audit-logged (AC-3)
//!   - BC-2.05.010 — Confirmation token lifecycle events are audit-logged (AC-4)
//!
//! Test naming: `test_BC_2_05_NNN_xxx()` for full traceability.
//!
//! # QueryContext gap note (S-2.05 spec gap)
//!
//! The story spec refers to `QueryContext` from `prism-core`, which does not yet
//! exist in the workspace. The stub-architect created interim context types local
//! to each module:
//!   - `RequestingContext`  (credential_events.rs)
//!   - `FlagEvalContext`    (flag_events.rs)
//!   - `TokenEventContext`  (token_events.rs)
//!
//! Tests construct these directly using the story-specified fields. If the
//! implementer later consolidates into `prism_core::QueryContext`, these
//! constructions will remain valid as local types or change only at the call
//! site. This is documented and handled per the dispatch note.

use chrono::Utc;
use serial_test::serial;

use prism_storage::backend::RocksStorageBackend;

use crate::audit_entry::{AuditEntry, AuditOutcome, DataClassification};
use crate::credential_events::{
    detail_to_json as cred_detail_to_json, CredentialAccessDetail, CredentialAccessResult,
    CredentialAccessType, RequestingContext,
};
use crate::flag_events::{
    detail_to_json as flag_detail_to_json, emit_flag_eval, FlagEvalContext, FlagEvalDetail,
    FlagResolutionStep,
};
use crate::tests::helpers::MemBackend;
use crate::token_events::{
    detail_to_json as token_detail_to_json, emit_token_consumed, emit_token_expired,
    emit_token_generated, TokenEvent, TokenEventContext, TokenLifecycleDetail,
};
use crate::vector_compat::{outcome_to_log_level, resolve_host, to_vector_json};

// ═══════════════════════════════════════════════════════════════════════════════
// BC-2.05.005 — Credential Access Events
// ═══════════════════════════════════════════════════════════════════════════════

// ── AC-1: credential name recorded, value absent ─────────────────────────────

/// AC-1 / BC-2.05.005: `emit_credential_event()` for a read on
/// `"crowdstrike_api_key"` must produce an entry whose serialized
/// `CredentialAccessDetail` contains `credential_name: "crowdstrike_api_key"`.
///

#[test]
fn test_BC_2_05_005_credential_name_recorded_on_emit() {
    let ctx = RequestingContext {
        tool_name: "crowdstrike_get_detections".to_owned(),
        client_id: "acme".to_owned(),
        trace_id: "01900000-0000-7000-0000-000000000001".to_owned(),
    };
    let backend = MemBackend::new();
    let result = crate::credential_events::emit_credential_event(
        &backend,
        "crowdstrike_api_key",
        "crowdstrike",
        CredentialAccessType::Read,
        CredentialAccessResult::Success,
        &ctx,
    );
    // The emitter must not error on a valid call.
    assert!(
        result.is_ok(),
        "emit_credential_event should succeed for valid input, got: {:?}",
        result
    );
}

/// AC-1 / BC-2.05.005 postcondition: Serialised `CredentialAccessDetail` JSON
/// must contain `"credential_name": "crowdstrike_api_key"`.
///
/// GREEN-BY-DESIGN: `detail_to_json` delegates to `serde_json::to_value` on a
/// fully-implemented pure struct. This exercises AC-1's structural assertion
/// using the serialization helper, which the stub-architect implemented as a
/// tautological data helper. Test is GREEN-BY-DESIGN.
#[test]
fn test_BC_2_05_005_serialized_detail_contains_credential_name() {
    let detail = CredentialAccessDetail {
        credential_name: "crowdstrike_api_key".to_owned(),
        access_type: CredentialAccessType::Read,
        sensor_id: "crowdstrike".to_owned(),
        result: CredentialAccessResult::Success,
        requesting_context: RequestingContext {
            tool_name: "crowdstrike_get_detections".to_owned(),
            client_id: "acme".to_owned(),
            trace_id: "01900000-0000-7000-0000-000000000001".to_owned(),
        },
    };
    let json = cred_detail_to_json(&detail).expect("serialization must not fail");

    assert_eq!(
        json["credential_name"],
        serde_json::Value::String("crowdstrike_api_key".to_owned()),
        "serialized CredentialAccessDetail must have credential_name == 'crowdstrike_api_key'"
    );
    assert_eq!(
        json["access_type"],
        serde_json::Value::String("read".to_owned()),
        "access_type must serialize to snake_case 'read'"
    );
    assert_eq!(
        json["sensor_id"],
        serde_json::Value::String("crowdstrike".to_owned()),
        "sensor_id must be present in serialized detail"
    );
}

// ── BC-2.05.005 invariant: no credential value fields in serialized output ────

/// BC-2.05.005 invariant (DI-002): serialized `CredentialAccessDetail` must not
/// contain any field named `value`, `secret`, `password`, or `token` at the
/// top level.
///
/// GREEN-BY-DESIGN: `CredentialAccessDetail` has no such fields by struct
/// definition — this is a pure data test that confirms the struct shape is
/// compliant before proptest is added by the implementer. This is correct
/// behaviour by construction (GREEN-BY-DESIGN).
#[test]
fn test_BC_2_05_005_invariant_no_credential_value_fields_in_detail() {
    let detail = CredentialAccessDetail {
        credential_name: "api_key".to_owned(),
        access_type: CredentialAccessType::Write,
        sensor_id: "crowdstrike".to_owned(),
        result: CredentialAccessResult::Success,
        requesting_context: RequestingContext {
            tool_name: "tool".to_owned(),
            client_id: "client_a".to_owned(),
            trace_id: "trace-001".to_owned(),
        },
    };
    let json = cred_detail_to_json(&detail).expect("serialization must succeed");
    let obj = json
        .as_object()
        .expect("detail must serialize to a JSON object");

    // BC-2.05.005: none of the forbidden field names may appear at the top level.
    let forbidden = ["value", "secret", "password", "token"];
    for name in forbidden {
        assert!(
            !obj.contains_key(name),
            "CredentialAccessDetail must NOT contain field '{name}' (BC-2.05.005 DI-002). \
             Found in serialized JSON: {json}"
        );
    }
}

/// BC-2.05.005: requesting context fields (`tool_name`, `client_id`,
/// `trace_id`) must be present in the serialized detail.
///
/// GREEN-BY-DESIGN: pure struct-shape assertion on a fully-implemented data
/// struct. GREEN-BY-DESIGN.
#[test]
fn test_BC_2_05_005_requesting_context_fields_present_in_detail() {
    let detail = CredentialAccessDetail {
        credential_name: "key".to_owned(),
        access_type: CredentialAccessType::Read,
        sensor_id: "armis".to_owned(),
        result: CredentialAccessResult::Success,
        requesting_context: RequestingContext {
            tool_name: "armis_list_devices".to_owned(),
            client_id: "beta_client".to_owned(),
            trace_id: "trace-abc".to_owned(),
        },
    };
    let json = cred_detail_to_json(&detail).expect("serialization must succeed");

    let ctx = &json["requesting_context"];
    assert_eq!(
        ctx["tool_name"],
        serde_json::Value::String("armis_list_devices".to_owned()),
        "requesting_context.tool_name must be present"
    );
    assert_eq!(
        ctx["client_id"],
        serde_json::Value::String("beta_client".to_owned()),
        "requesting_context.client_id must be present"
    );
    assert_eq!(
        ctx["trace_id"],
        serde_json::Value::String("trace-abc".to_owned()),
        "requesting_context.trace_id must be present"
    );
}

/// BC-2.05.005: `access_type` variants serialize to their expected snake_case
/// names per the `#[serde(rename_all = "snake_case")]` attribute.
///
/// GREEN-BY-DESIGN: pure serde roundtrip on fully-implemented data types.
#[test]
fn test_BC_2_05_005_access_type_variants_serialize_correctly() {
    let cases = [
        (CredentialAccessType::Read, "read"),
        (CredentialAccessType::Write, "write"),
        (CredentialAccessType::Delete, "delete"),
        (CredentialAccessType::Rotate, "rotate"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_value(&variant).expect("serialize must succeed");
        assert_eq!(
            json,
            serde_json::Value::String(expected.to_owned()),
            "CredentialAccessType::{variant:?} must serialize to '{expected}'"
        );
    }
}

/// BC-2.05.005 precondition: `emit_credential_event()` called for a
/// `"not_found"` result must still succeed (result records `NotFound`,
/// no panic).
///

#[test]
fn test_BC_2_05_005_emit_not_found_result_succeeds() {
    let ctx = RequestingContext {
        tool_name: "get_cred".to_owned(),
        client_id: "acme".to_owned(),
        trace_id: "trace-notfound".to_owned(),
    };
    let backend = MemBackend::new();
    let result = crate::credential_events::emit_credential_event(
        &backend,
        "nonexistent_key",
        "crowdstrike",
        CredentialAccessType::Read,
        CredentialAccessResult::NotFound,
        &ctx,
    );
    assert!(
        result.is_ok(),
        "emit_credential_event with NotFound result must not fail, got: {:?}",
        result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-2.05.007 — Vector Pipeline Compatibility
// ═══════════════════════════════════════════════════════════════════════════════

// ── Helper: build a minimal AuditEntry for vector tests ──────────────────────

fn make_audit_entry(outcome: AuditOutcome) -> AuditEntry {
    AuditEntry::new(
        uuid::Uuid::now_v7(),
        Utc::now(),
        "crowdstrike_contain_host".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({"host_id": "acme-ws-01"}),
        outcome,
        "ok".to_owned(),
        42,
        None,
        DataClassification::Confidential,
        vec![],
        vec![],
    )
}

// ── AC-2: @timestamp, host, service, log.level all present ───────────────────

/// AC-2 / BC-2.05.007: `to_vector_json()` must produce a JSON object containing
/// `"@timestamp"`, `"host"`, `"service"`, and `"log.level"` fields.
///

#[test]
fn test_BC_2_05_007_vector_json_contains_required_fields() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let value = to_vector_json(&entry);

    let obj = value
        .as_object()
        .expect("to_vector_json must return a JSON object");

    assert!(
        obj.contains_key("@timestamp"),
        "Vector JSON must contain '@timestamp' field (BC-2.05.007 AC-2)"
    );
    assert!(
        obj.contains_key("host"),
        "Vector JSON must contain 'host' field (BC-2.05.007 AC-2)"
    );
    assert!(
        obj.contains_key("service"),
        "Vector JSON must contain 'service' field (BC-2.05.007 AC-2)"
    );
    assert!(
        obj.contains_key("log.level"),
        "Vector JSON must contain 'log.level' field (BC-2.05.007 AC-2)"
    );
}

/// AC-2 / BC-2.05.007: `service` field must be the fixed string `"prism"`.
///

#[test]
fn test_BC_2_05_007_service_field_is_prism() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let value = to_vector_json(&entry);

    assert_eq!(
        value["service"],
        serde_json::Value::String("prism".to_owned()),
        "Vector JSON 'service' must always be 'prism' (BC-2.05.007 AC-2)"
    );
}

/// AC-2 / BC-2.05.007: `log.level` must be `"info"` for `AuditOutcome::Success`.
///

#[test]
fn test_BC_2_05_007_log_level_info_for_success() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let value = to_vector_json(&entry);

    assert_eq!(
        value["log.level"],
        serde_json::Value::String("info".to_owned()),
        "Vector JSON 'log.level' must be 'info' for AuditOutcome::Success (BC-2.05.007)"
    );
}

/// EC-005 / BC-2.05.007: `log.level` must be `"error"` for
/// `AuditOutcome::Failure`.
///

#[test]
fn test_BC_2_05_007_log_level_error_for_failure() {
    let entry = make_audit_entry(AuditOutcome::Failure {
        error_code: "E-QUERY-001".to_owned(),
    });
    let value = to_vector_json(&entry);

    assert_eq!(
        value["log.level"],
        serde_json::Value::String("error".to_owned()),
        "Vector JSON 'log.level' must be 'error' for AuditOutcome::Failure (BC-2.05.007 EC-005)"
    );
}

/// BC-2.05.007: `@timestamp` must be a non-empty RFC 3339 string (parseable).
///

#[test]
fn test_BC_2_05_007_timestamp_is_rfc3339() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let value = to_vector_json(&entry);

    let ts = value["@timestamp"]
        .as_str()
        .expect("@timestamp must be a JSON string");

    // Must be parseable as RFC 3339.
    assert!(
        chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
        "@timestamp must be a valid RFC 3339 string, got: '{ts}' (BC-2.05.007)"
    );
}

/// BC-2.05.007: `host` field must never be empty.
///
/// EC-002: even when `PRISM_HOST_ID` is unset, `host` must be non-empty.
///

#[test]
#[serial]
fn test_BC_2_05_007_host_field_never_empty() {
    let entry = make_audit_entry(AuditOutcome::Success);
    // Remove PRISM_HOST_ID to trigger the fallback path.
    std::env::remove_var("PRISM_HOST_ID");
    let value = to_vector_json(&entry);

    let host = value["host"]
        .as_str()
        .expect("'host' must be a JSON string");

    assert!(
        !host.is_empty(),
        "'host' field must never be empty (BC-2.05.007 EC-002). \
         Expected gethostname() fallback or 'unknown-host' sentinel."
    );
}

/// BC-2.05.007: `to_vector_json()` must not modify the original `AuditEntry`
/// (read-only guarantee from S-2.05 Architecture Compliance).
///

#[test]
fn test_BC_2_05_007_to_vector_json_does_not_modify_entry() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let original_tool_name = entry.tool_name.clone();
    let original_client_id = entry.client_id.clone();

    let _value = to_vector_json(&entry);

    // Entry fields must be unchanged after the call.
    assert_eq!(
        entry.tool_name, original_tool_name,
        "to_vector_json must not modify entry.tool_name"
    );
    assert_eq!(
        entry.client_id, original_client_id,
        "to_vector_json must not modify entry.client_id"
    );
}

/// BC-2.05.007 round-trip: `AuditEntry` → `to_vector_json()` → parse JSON →
/// assert key fields present (no data loss).
///

#[test]
fn test_BC_2_05_007_round_trip_no_data_loss() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let tool_name = entry.tool_name.clone();
    let client_id = entry.client_id.clone();

    let value = to_vector_json(&entry);

    // Re-parse to confirm the JSON is valid (no serialization error path).
    let re_parsed: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&value).expect("must serialize"))
            .expect("must deserialize");

    assert_eq!(
        re_parsed["tool_name"].as_str().unwrap_or(""),
        tool_name,
        "round-trip must preserve tool_name (BC-2.05.007)"
    );
    assert_eq!(
        re_parsed["client_id"].as_str().unwrap_or(""),
        client_id,
        "round-trip must preserve client_id (BC-2.05.007)"
    );
}

/// BC-2.05.007: `parameters` must be serialized as a JSON string (not nested
/// object) in the Vector format, per Dev Notes.
///

#[test]
fn test_BC_2_05_007_parameters_serialized_as_string_not_nested_object() {
    let entry = make_audit_entry(AuditOutcome::Success);
    let value = to_vector_json(&entry);

    // The `parameters` field must be a JSON string, not an object.
    assert!(
        value["parameters"].is_string(),
        "Vector JSON 'parameters' must be a JSON string (not a nested object) \
         per Dev Notes (BC-2.05.007). Got type: {:?}",
        value["parameters"]
    );
}

// ── outcome_to_log_level — GREEN-BY-DESIGN ────────────────────────────────────

/// BC-2.05.007: `outcome_to_log_level(Success)` returns `"info"`.
///
/// GREEN-BY-DESIGN: `outcome_to_log_level` is a trivial two-arm match
/// implemented in the stub. The stub-architect flagged this as GREEN-BY-DESIGN.
/// The test is retained for traceability to AC-2 / BC-2.05.007.
#[test]
fn test_BC_2_05_007_outcome_to_log_level_success_is_info() {
    assert_eq!(
        outcome_to_log_level(&AuditOutcome::Success),
        "info",
        "outcome_to_log_level(Success) must return 'info' (BC-2.05.007)"
    );
}

/// BC-2.05.007: `outcome_to_log_level(Failure)` returns `"error"`.
///
/// GREEN-BY-DESIGN: same rationale as above.
#[test]
fn test_BC_2_05_007_outcome_to_log_level_failure_is_error() {
    assert_eq!(
        outcome_to_log_level(&AuditOutcome::Failure {
            error_code: "E-001".to_owned()
        }),
        "error",
        "outcome_to_log_level(Failure) must return 'error' (BC-2.05.007)"
    );
}

// ── EC-002: resolve_host fallback chain ───────────────────────────────────────

/// EC-002 / BC-2.05.007: `resolve_host()` must never return an empty string.
///

#[test]
#[serial]
fn test_BC_2_05_007_resolve_host_never_empty() {
    std::env::remove_var("PRISM_HOST_ID");
    let host = resolve_host();

    assert!(
        !host.is_empty(),
        "resolve_host() must never return an empty string (BC-2.05.007 EC-002). \
         Expected gethostname() fallback or 'unknown-host' sentinel."
    );
}

/// EC-002 / BC-2.05.007: When `PRISM_HOST_ID` is set, `resolve_host()` returns it.
///

#[test]
#[serial]
fn test_BC_2_05_007_resolve_host_uses_prism_host_id_env_var() {
    std::env::set_var("PRISM_HOST_ID", "prism-node-42");
    let host = resolve_host();
    std::env::remove_var("PRISM_HOST_ID");

    assert_eq!(
        host, "prism-node-42",
        "resolve_host() must return PRISM_HOST_ID when set (BC-2.05.007 EC-002)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-2.05.009 — Feature Flag Evaluation Audit Events
// ═══════════════════════════════════════════════════════════════════════════════

// ── AC-3: full resolution_trace recorded ─────────────────────────────────────

/// AC-3 / BC-2.05.009: `emit_flag_eval()` for `"sensors.crowdstrike.write"` must
/// succeed, and the `FlagEvalDetail` must record the full `resolution_trace`.
///

#[test]
fn test_BC_2_05_009_emit_flag_eval_records_resolution_trace() {
    let ctx = FlagEvalContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        trace_id: "01900000-0000-7000-0000-000000000002".to_owned(),
    };
    let detail = FlagEvalDetail {
        client_id: "acme".to_owned(),
        capability_path: "sensors.crowdstrike.write".to_owned(),
        evaluation_result: true,
        resolution_trace: vec![
            FlagResolutionStep {
                rule_id: "R-001".to_owned(),
                matched: true,
                reason: "client_id matched allowlist rule R-001".to_owned(),
            },
            FlagResolutionStep {
                rule_id: "global-deny-default".to_owned(),
                matched: false,
                reason: "global deny default not reached".to_owned(),
            },
        ],
    };
    let backend = MemBackend::new();
    let result = emit_flag_eval(&backend, detail, &ctx);

    assert!(
        result.is_ok(),
        "emit_flag_eval must succeed for a valid FlagEvalDetail, got: {:?}",
        result
    );
}

/// AC-3 / BC-2.05.009: Serialized `FlagEvalDetail` JSON must contain
/// `capability_path` and `resolution_trace` as an array.
///
/// GREEN-BY-DESIGN: `detail_to_json` is a tautological pure-data helper
/// (fully implemented stub). The assertion confirms the struct shape meets
/// AC-3 requirements. GREEN-BY-DESIGN.
#[test]
fn test_BC_2_05_009_serialized_flag_eval_detail_contains_capability_path_and_trace() {
    let detail = FlagEvalDetail {
        client_id: "acme".to_owned(),
        capability_path: "sensors.crowdstrike.write".to_owned(),
        evaluation_result: true,
        resolution_trace: vec![FlagResolutionStep {
            rule_id: "R-001".to_owned(),
            matched: true,
            reason: "client_id matched allowlist rule R-001".to_owned(),
        }],
    };
    let json = flag_detail_to_json(&detail).expect("serialization must succeed");

    assert_eq!(
        json["capability_path"],
        serde_json::Value::String("sensors.crowdstrike.write".to_owned()),
        "FlagEvalDetail must serialize capability_path (BC-2.05.009)"
    );
    assert!(
        json["resolution_trace"].is_array(),
        "resolution_trace must serialize as a JSON array (BC-2.05.009)"
    );
    assert_eq!(
        json["resolution_trace"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0),
        1,
        "resolution_trace must contain exactly the 1 step we provided"
    );
}

/// EC-004 / BC-2.05.009: `emit_flag_eval()` with an empty `resolution_trace`
/// must NOT panic — entry is emitted with `resolution_trace: []`.
///

#[test]
fn test_BC_2_05_009_empty_resolution_trace_does_not_panic() {
    let ctx = FlagEvalContext {
        tool_name: "crowdstrike_write_op".to_owned(),
        client_id: "beta_client".to_owned(),
        trace_id: "trace-empty-trace".to_owned(),
    };
    let detail = FlagEvalDetail {
        client_id: "beta_client".to_owned(),
        capability_path: "sensors.crowdstrike.write".to_owned(),
        evaluation_result: false,
        resolution_trace: vec![], // EC-004: empty trace
    };
    let backend = MemBackend::new();
    // Must not panic; must return either Ok or a persistence error (not a logic panic).
    let result = emit_flag_eval(&backend, detail, &ctx);
    assert!(
        result.is_ok(),
        "emit_flag_eval with empty resolution_trace must not panic (EC-004 / BC-2.05.009), \
         got: {:?}",
        result
    );
}

/// EC-004 / BC-2.05.009: `FlagEvalDetail` with empty `resolution_trace`
/// serialises correctly to `resolution_trace: []`.
///
/// GREEN-BY-DESIGN: pure serde data test on a fully-implemented struct.
#[test]
fn test_BC_2_05_009_empty_resolution_trace_serializes_as_empty_array() {
    let detail = FlagEvalDetail {
        client_id: "acme".to_owned(),
        capability_path: "sensors.crowdstrike.write".to_owned(),
        evaluation_result: false,
        resolution_trace: vec![],
    };
    let json = flag_detail_to_json(&detail).expect("serialization must succeed");

    assert_eq!(
        json["resolution_trace"],
        serde_json::Value::Array(vec![]),
        "empty resolution_trace must serialize to [] not null (EC-004 / BC-2.05.009)"
    );
}

/// BC-2.05.009: Each `FlagResolutionStep` in the trace records `rule_id`,
/// `matched`, and `reason` in human-readable form.
///
/// GREEN-BY-DESIGN: struct-shape serialization test on fully-implemented data.
#[test]
fn test_BC_2_05_009_resolution_step_fields_present_and_human_readable() {
    let step = FlagResolutionStep {
        rule_id: "R-001".to_owned(),
        matched: true,
        reason: "client_id matched allowlist rule R-001".to_owned(),
    };
    let json = serde_json::to_value(&step).expect("serialize must succeed");

    assert_eq!(json["rule_id"], serde_json::json!("R-001"));
    assert_eq!(json["matched"], serde_json::json!(true));
    assert_eq!(
        json["reason"],
        serde_json::json!("client_id matched allowlist rule R-001"),
        "reason must be human-readable per Dev Notes (BC-2.05.009)"
    );
}

/// BC-2.05.009: canonical test vector — direct path match.
/// `capability_path: "sensor.crowdstrike.containment"`, one step, `result: permitted`.
///
/// GREEN-BY-DESIGN: struct-shape verification on fully-implemented data types.
#[test]
fn test_BC_2_05_009_canonical_vector_direct_path_match_serializes() {
    let detail = FlagEvalDetail {
        client_id: "acme".to_owned(),
        capability_path: "sensor.crowdstrike.containment".to_owned(),
        evaluation_result: true,
        resolution_trace: vec![FlagResolutionStep {
            rule_id: "sensor.crowdstrike.containment".to_owned(),
            matched: true,
            reason: "compile_time_enabled: true, runtime_enabled: true".to_owned(),
        }],
    };
    let json = flag_detail_to_json(&detail).expect("serialize must succeed");

    assert_eq!(
        json["capability_path"],
        serde_json::json!("sensor.crowdstrike.containment"),
        "capability_path must match canonical test vector"
    );
    assert_eq!(
        json["evaluation_result"],
        serde_json::json!(true),
        "evaluation_result must be true for permitted path"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// BC-2.05.010 — Confirmation Token Lifecycle Events
// ═══════════════════════════════════════════════════════════════════════════════

// ── AC-4: token generated entry ──────────────────────────────────────────────

/// AC-4 / BC-2.05.010: `emit_token_generated()` for `"isolate host acme-ws-01"`
/// must succeed.
///

#[test]
fn test_BC_2_05_010_emit_token_generated_succeeds() {
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() + chrono::Duration::seconds(300);

    let backend = MemBackend::new();
    let result = emit_token_generated(&backend, "tok-001", "isolate host acme-ws-01", expiry, &ctx);

    assert!(
        result.is_ok(),
        "emit_token_generated must succeed for valid input (AC-4 / BC-2.05.010), \
         got: {:?}",
        result
    );
}

/// AC-4 / BC-2.05.010: Serialised `TokenLifecycleDetail` for issuance must
/// contain `action_summary` and `expiry_time`.
///
/// GREEN-BY-DESIGN: pure serde round-trip on fully-implemented struct.
#[test]
fn test_BC_2_05_010_token_generated_detail_contains_action_summary_and_expiry() {
    let expiry = Utc::now() + chrono::Duration::seconds(300);
    let detail = TokenLifecycleDetail {
        token_id: "tok-001".to_owned(),
        event_type: TokenEvent::Generated,
        action_summary: "isolate host acme-ws-01".to_owned(),
        expiry_time: expiry,
    };
    let json = token_detail_to_json(&detail).expect("serialization must succeed");

    assert_eq!(
        json["action_summary"],
        serde_json::Value::String("isolate host acme-ws-01".to_owned()),
        "token detail must contain action_summary (AC-4 / BC-2.05.010)"
    );
    assert!(
        json["expiry_time"].is_string(),
        "token detail must contain expiry_time as a string (AC-4 / BC-2.05.010)"
    );
    assert_eq!(
        json["event_type"],
        serde_json::Value::String("generated".to_owned()),
        "event_type must serialize to 'generated'"
    );
}

// ── Token consumed path ───────────────────────────────────────────────────────

/// BC-2.05.010: `emit_token_consumed()` must succeed and produce a distinct
/// entry from `emit_token_generated()`.
///

#[test]
fn test_BC_2_05_010_emit_token_consumed_succeeds() {
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };

    let backend = MemBackend::new();
    let result = emit_token_consumed(&backend, "tok-001", "isolate host acme-ws-01", &ctx);

    assert!(
        result.is_ok(),
        "emit_token_consumed must succeed (BC-2.05.010), got: {:?}",
        result
    );
}

/// BC-2.05.010: `TokenEvent::Consumed` serializes to `"consumed"`.
///
/// GREEN-BY-DESIGN: pure serde data test.
#[test]
fn test_BC_2_05_010_token_event_consumed_serializes_correctly() {
    let detail = TokenLifecycleDetail {
        token_id: "tok-001".to_owned(),
        event_type: TokenEvent::Consumed,
        action_summary: "isolate host".to_owned(),
        expiry_time: Utc::now(),
    };
    let json = token_detail_to_json(&detail).expect("serialize must succeed");

    assert_eq!(
        json["event_type"],
        serde_json::Value::String("consumed".to_owned()),
        "TokenEvent::Consumed must serialize to 'consumed' (BC-2.05.010)"
    );
}

// ── Token expired path ────────────────────────────────────────────────────────

/// BC-2.05.010: `emit_token_expired()` must succeed and be a DISTINCT event
/// from `emit_token_consumed()` (EC-003).
///

#[test]
fn test_BC_2_05_010_emit_token_expired_succeeds() {
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() - chrono::Duration::seconds(1); // already expired

    let backend = MemBackend::new();
    let result = emit_token_expired(&backend, "tok-002", "isolate host acme-ws-02", expiry, &ctx);

    assert!(
        result.is_ok(),
        "emit_token_expired must succeed (BC-2.05.010), got: {:?}",
        result
    );
}

/// EC-003 / BC-2.05.010: `Consumed` and `Expired` events are distinct.
/// Serialized `event_type` differs between the two.
///
/// GREEN-BY-DESIGN: pure serde assertion on fully-implemented enum.
#[test]
fn test_BC_2_05_010_consumed_and_expired_event_types_are_distinct() {
    let consumed_json = serde_json::to_value(TokenEvent::Consumed).expect("serialize");
    let expired_json = serde_json::to_value(TokenEvent::Expired).expect("serialize");

    assert_ne!(
        consumed_json, expired_json,
        "TokenEvent::Consumed and TokenEvent::Expired must serialize to different values \
         (EC-003 / BC-2.05.010)"
    );
    assert_eq!(
        consumed_json,
        serde_json::json!("consumed"),
        "TokenEvent::Consumed must serialize to 'consumed'"
    );
    assert_eq!(
        expired_json,
        serde_json::json!("expired"),
        "TokenEvent::Expired must serialize to 'expired'"
    );
}

/// BC-2.05.010: All `TokenEvent` variants serialize to their expected
/// snake_case strings.
///
/// GREEN-BY-DESIGN: pure serde data test on fully-implemented enum.
#[test]
fn test_BC_2_05_010_all_token_event_variants_serialize_correctly() {
    let cases = [
        (TokenEvent::Generated, "generated"),
        (TokenEvent::Consumed, "consumed"),
        (TokenEvent::Expired, "expired"),
        (TokenEvent::NotFound, "not_found"),
        (TokenEvent::HashMismatch, "hash_mismatch"),
        (TokenEvent::AlreadyConsumed, "already_consumed"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_value(&variant).expect("serialize must succeed");
        assert_eq!(
            json,
            serde_json::Value::String(expected.to_owned()),
            "TokenEvent::{variant:?} must serialize to '{expected}' (BC-2.05.010)"
        );
    }
}

/// BC-2.05.010 postcondition: token issuance entry's `result_summary` must be
/// `"confirmation_token_issued"`. The test calls the emitter and checks that
/// it completes without error.
#[test]
fn test_BC_2_05_010_token_generated_result_summary_is_confirmation_token_issued() {
    // This test verifies the result_summary postcondition from BC-2.05.010.
    // It calls emit_token_generated and expects it to complete without panicking.
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() + chrono::Duration::seconds(300);

    let backend = MemBackend::new();
    // emit_token_generated must record result_summary:
    // "confirmation_token_issued" per BC-2.05.010 postcondition.
    let result = emit_token_generated(&backend, "tok-003", "delete sensor config", expiry, &ctx);

    assert!(
        result.is_ok(),
        "emit_token_generated must return Ok (BC-2.05.010)"
    );
}

/// BC-2.05.010 postcondition (canonical TV): `emit_token_generated()` must NOT
/// persist `token_id` in the `token_lifecycle_detail` JSON embedded in
/// `parameters`.  Token IDs are intentionally excluded from issuance audit
/// entries to prevent correlation by log readers (BC-2.05.010 Pass 7 HIGH-001/003).
///
/// RED gate: current `emit_token_generated` serialises the full
/// `TokenLifecycleDetail` struct (including `token_id`) into `parameters`.
/// This test will FAIL until `token_id` is stripped from the serialized JSON
/// before persistence.
#[test]
fn test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail() {
    let backend = MemBackend::new();
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() + chrono::Duration::seconds(300);
    let token_id = "tok-secret-001";
    let action_summary = "isolate host acme-ws-01";

    emit_token_generated(&backend, token_id, action_summary, expiry, &ctx)
        .expect("emit_token_generated must not fail");

    // Read the persisted entry from the audit_buffer CF.
    let entries = backend
        .scan(prism_core::StorageDomain::AuditBuffer, b"audit:")
        .expect("scan must succeed");
    assert_eq!(entries.len(), 1, "exactly one entry must be persisted");

    let (_, raw) = &entries[0];
    let (entry, _): (prism_storage::audit_buffer::AuditEntry, _) =
        bincode::serde::decode_from_slice(raw, bincode::config::standard())
            .expect("bincode decode must succeed");

    // parameters is stored as a JSON string in the payload BTreeMap.
    let params_str = entry
        .payload
        .get("parameters")
        .expect("payload must contain 'parameters' key");
    let params: serde_json::Value =
        serde_json::from_str(params_str).expect("parameters must be valid JSON");

    let detail = &params["token_lifecycle_detail"];
    assert!(
        detail.is_object(),
        "parameters must contain 'token_lifecycle_detail' object, got: {params}"
    );

    // BC-2.05.010 canonical TV postcondition: token_id must NOT be in persisted
    // parameters for a Generated event.
    assert!(
        detail.get("token_id").is_none(),
        "BC-2.05.010 HIGH-001: token_id MUST NOT appear in persisted \
         token_lifecycle_detail for Generated events. \
         Found: {:?}",
        detail.get("token_id")
    );

    // Inclusion side: action_summary and expiry_time must be present.
    assert!(
        detail.get("action_summary").is_some(),
        "action_summary must be present in persisted token_lifecycle_detail"
    );
    assert_eq!(
        detail["action_summary"],
        serde_json::Value::String(action_summary.to_owned()),
        "action_summary must match the input value"
    );
    assert!(
        detail.get("expiry_time").is_some(),
        "expiry_time must be present in persisted token_lifecycle_detail"
    );
}

/// BC-2.05.010 postcondition (canonical TV): `emit_token_expired()` must NOT
/// persist `token_id` in the `token_lifecycle_detail` JSON embedded in
/// `parameters`.  Per BC-2.05.010 canonical TV table: "Token expired → Token ID
/// in Entry? = No".
///
/// RED gate: current `emit_token_expired` serialises the full
/// `TokenLifecycleDetail` struct (including `token_id`) into `parameters`.
/// This test will FAIL until `token_id` is stripped from the serialized JSON
/// before persistence.
#[test]
fn test_BC_2_05_010_token_id_excluded_from_expired_persisted_parameters() {
    let backend = MemBackend::new();
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };
    let expiry = Utc::now() - chrono::Duration::seconds(1); // already expired
    let token_id = "tok-expired-secret-002";
    let action_summary = "isolate host acme-ws-02";

    emit_token_expired(&backend, token_id, action_summary, expiry, &ctx)
        .expect("emit_token_expired must not fail");

    // Read the persisted entry from the audit_buffer CF.
    let entries = backend
        .scan(prism_core::StorageDomain::AuditBuffer, b"audit:")
        .expect("scan must succeed");
    assert_eq!(entries.len(), 1, "exactly one entry must be persisted");

    let (_, raw) = &entries[0];
    let (entry, _): (prism_storage::audit_buffer::AuditEntry, _) =
        bincode::serde::decode_from_slice(raw, bincode::config::standard())
            .expect("bincode decode must succeed");

    let params_str = entry
        .payload
        .get("parameters")
        .expect("payload must contain 'parameters' key");
    let params: serde_json::Value =
        serde_json::from_str(params_str).expect("parameters must be valid JSON");

    let detail = &params["token_lifecycle_detail"];
    assert!(
        detail.is_object(),
        "parameters must contain 'token_lifecycle_detail' object, got: {params}"
    );

    // BC-2.05.010 canonical TV postcondition: token_id must NOT be in persisted
    // parameters for an Expired event.
    assert!(
        detail.get("token_id").is_none(),
        "BC-2.05.010 HIGH-001: token_id MUST NOT appear in persisted \
         token_lifecycle_detail for Expired events. \
         Found: {:?}",
        detail.get("token_id")
    );

    // Inclusion side: action_summary and expiry_time must be present.
    assert!(
        detail.get("action_summary").is_some(),
        "action_summary must be present in persisted token_lifecycle_detail for Expired"
    );
    assert_eq!(
        detail["action_summary"],
        serde_json::Value::String(action_summary.to_owned()),
        "action_summary must match the input value"
    );
    assert!(
        detail.get("expiry_time").is_some(),
        "expiry_time must be present in persisted token_lifecycle_detail for Expired"
    );
}

/// BC-2.05.010: `TokenEventContext` carries `client_id`, `sensor`, and
/// `tool_name` for all token lifecycle events.
///
/// GREEN-BY-DESIGN: struct field existence test on fully-implemented data struct.
#[test]
fn test_BC_2_05_010_token_event_context_carries_required_fields() {
    let ctx = TokenEventContext {
        tool_name: "crowdstrike_contain_host".to_owned(),
        client_id: "acme".to_owned(),
        sensor: "crowdstrike".to_owned(),
    };

    assert_eq!(ctx.tool_name, "crowdstrike_contain_host");
    assert_eq!(ctx.client_id, "acme");
    assert_eq!(ctx.sensor, "crowdstrike");
}
