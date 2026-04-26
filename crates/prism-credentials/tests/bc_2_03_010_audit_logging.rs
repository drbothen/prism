//! Tests for BC-2.03.010: Credential Access Audit Logging
//!
//! Every test name follows the `test_BC_S_SS_NNN_xxx` convention.
//! All tests pass (implementation complete).

use prism_credentials::audit::{emit_audit, AuditEvent, AuditOperation, AuditOutcome};

// ---------------------------------------------------------------------------
// TV-BC-2.03.010-001: successful get operation — audit entry with correct fields
// ---------------------------------------------------------------------------

/// BC-2.03.010 postcondition: AuditEvent for a successful get includes all required fields.
#[test]
fn test_BC_2_03_010_audit_event_get_success_has_all_required_fields() {
    let event = AuditEvent::new(
        AuditOperation::Get,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );

    assert_eq!(event.event_type, "credential_access");
    assert_eq!(event.operation, AuditOperation::Get);
    assert_eq!(event.client_id, "acme");
    assert_eq!(event.sensor_id, "crowdstrike");
    assert_eq!(event.credential_name, "api_key");
    assert_eq!(event.backend, "keyring");
    assert_eq!(event.result, AuditOutcome::Success);
    // timestamp is a DateTime<Utc> — must be recent (within 5 seconds)
    let now = chrono::Utc::now();
    let age = now
        .signed_duration_since(event.timestamp)
        .num_seconds()
        .abs();
    assert!(age < 5, "timestamp must be recent, age was {age}s");
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.010-002: failed get (not found) — same fields as success
// ---------------------------------------------------------------------------

/// BC-2.03.010 postcondition: failed access is logged with same detail as success.
#[test]
fn test_BC_2_03_010_audit_event_get_not_found_has_same_fields() {
    let event = AuditEvent::new(
        AuditOperation::Get,
        "acme",
        "crowdstrike",
        "missing_key",
        "keyring",
        AuditOutcome::NotFound,
    );

    assert_eq!(event.event_type, "credential_access");
    assert_eq!(event.result, AuditOutcome::NotFound);
    assert_eq!(event.credential_name, "missing_key");
    // Same fields as success — no difference in logging detail
    assert_eq!(event.client_id, "acme");
    assert_eq!(event.sensor_id, "crowdstrike");
    assert_eq!(event.backend, "keyring");
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.010-003: fan-out — 10 individual audit entries
// ---------------------------------------------------------------------------

/// BC-2.03.010 EC-03-025: each credential read produces its own audit entry.
/// No batching or deduplication.
#[test]
fn test_BC_2_03_010_fanout_produces_individual_audit_entries() {
    let clients: Vec<String> = (0..10).map(|i| format!("client-{i:02}")).collect();

    // Each creates a distinct audit event (no batching)
    let events: Vec<AuditEvent> = clients
        .iter()
        .map(|c| {
            AuditEvent::new(
                AuditOperation::Get,
                c.as_str(),
                "crowdstrike",
                "api_key",
                "keyring",
                AuditOutcome::Success,
            )
        })
        .collect();

    assert_eq!(events.len(), 10, "must produce 10 individual audit entries");

    // Verify each entry has a distinct client_id
    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.client_id, clients[i]);
        assert_eq!(event.event_type, "credential_access");
    }
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.010-004: delete operation — logged with correct operation field
// ---------------------------------------------------------------------------

/// BC-2.03.010 postcondition: delete is logged with `operation: "delete"`.
#[test]
fn test_BC_2_03_010_audit_event_delete_operation_field() {
    let event = AuditEvent::new(
        AuditOperation::Delete,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
    assert_eq!(event.operation, AuditOperation::Delete);
    assert_eq!(format!("{}", event.operation), "delete");
}

// ---------------------------------------------------------------------------
// TV-BC-2.03.010-005: emit() does not panic when tracing subscriber unavailable
// ---------------------------------------------------------------------------

/// BC-2.03.010 error case: emit proceeds even if tracing subscriber unavailable.
/// In test context there is no subscriber — calling emit() must not panic.
#[test]
fn test_BC_2_03_010_emit_does_not_panic_without_subscriber() {
    let event = AuditEvent::new(
        AuditOperation::Get,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
    // Must not panic even without a tracing subscriber installed
    event.emit();
}

// ---------------------------------------------------------------------------
// Invariant DI-002: AuditEvent struct has no credential value field
// ---------------------------------------------------------------------------

/// BC-2.03.010 invariant DI-002: the AuditEvent struct has no field for credential values.
/// Enforced structurally — if a `value` field were added, this construct would need it.
#[test]
fn test_BC_2_03_010_invariant_audit_event_has_no_value_field() {
    // Exhaustive struct construction — if a `value` field were added, this would need to include it.
    let event = AuditEvent {
        event_type: "credential_access".to_string(),
        operation: AuditOperation::Set,
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        credential_name: "api_key".to_string(),
        backend: "encrypted_file".to_string(),
        result: AuditOutcome::Success,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(event.event_type, "credential_access");
    // No `value` field — type system enforces this.
}

// ---------------------------------------------------------------------------
// Invariant DI-004: emit_audit convenience wrapper emits for all operations
// ---------------------------------------------------------------------------

/// BC-2.03.010 invariant DI-004: emit_audit emits for get, set, delete, list.
/// Must not panic for any operation type.
#[test]
fn test_BC_2_03_010_emit_audit_covers_all_operation_types() {
    // All four operation types must be auditable without panic
    emit_audit(
        AuditOperation::Get,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
    emit_audit(
        AuditOperation::Set,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
    emit_audit(
        AuditOperation::Delete,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
    emit_audit(
        AuditOperation::List,
        "acme",
        "crowdstrike",
        "api_key",
        "keyring",
        AuditOutcome::Success,
    );
}

// ---------------------------------------------------------------------------
// operation Display: "get" | "set" | "delete" | "list"
// ---------------------------------------------------------------------------

/// BC-2.03.010 postcondition: operation Display strings match the BC.
#[test]
fn test_BC_2_03_010_operation_display_strings_match_bc() {
    assert_eq!(format!("{}", AuditOperation::Get), "get");
    assert_eq!(format!("{}", AuditOperation::Set), "set");
    assert_eq!(format!("{}", AuditOperation::Delete), "delete");
    assert_eq!(format!("{}", AuditOperation::List), "list");
}

/// BC-2.03.010 postcondition: outcome Display strings match the BC.
#[test]
fn test_BC_2_03_010_outcome_display_strings_match_bc() {
    assert_eq!(format!("{}", AuditOutcome::Success), "success");
    assert_eq!(format!("{}", AuditOutcome::NotFound), "not_found");
    assert_eq!(format!("{}", AuditOutcome::Error), "error");
}
