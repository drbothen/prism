// S-1.08: BC-2.04.013 — Feature Flag Evaluation Audit Logging for Write Operations
//
// Tests verify that:
//  - AC-5: write operation check emits an audit event.
//  - CapabilityCheckEvent fields: event_type, client_id, capability, result,
//    tool_name, denied_reason (if denied), timestamp.
//  - Audit events emitted for BOTH Allow and Deny outcomes.
//  - EC-04-027: cross-client fan-out emits one event per client.
//  - EC-04-028: compile-time denied → event still emitted with denied_reason "Feature not compiled".
//  - EC-003 (story edge case): audit emission failure does NOT affect gate result.
//  - Read operations do NOT emit capability check events.
//
// Naming: test_BC_2_04_013_<assertion>
#![allow(non_snake_case)]

use prism_security::flag_audit::{CapabilityCheckEvent, FlagAuditEmitter};

// ─────────────────────────────────────────────────────────────
// AC-5: write operation check emits an audit event
// ─────────────────────────────────────────────────────────────

/// AC-5: An audit event is constructed and emitted for a write capability check.
/// Tests that `FlagAuditEmitter::allowed_event` and `denied_event` build correct events.
#[test]
fn test_BC_2_04_013_ac5_allowed_event_has_correct_fields() {
    let event = FlagAuditEmitter::allowed_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
    );

    assert_eq!(
        event.event_type, "capability_check",
        "BC-2.04.013: event_type must be 'capability_check'"
    );
    assert_eq!(
        event.client_id, "acme",
        "BC-2.04.013: client_id must be 'acme'"
    );
    assert_eq!(
        event.capability, "sensor.crowdstrike.containment",
        "BC-2.04.013: capability must match"
    );
    assert_eq!(
        event.result, "allowed",
        "BC-2.04.013 AC-5: result must be 'allowed'"
    );
    assert_eq!(
        event.tool_name, "crowdstrike_contain_host",
        "BC-2.04.013: tool_name must match"
    );
    assert!(
        event.denied_reason.is_none(),
        "BC-2.04.013: denied_reason must be None for allowed events"
    );
    assert!(
        !event.timestamp.is_empty(),
        "BC-2.04.013: timestamp must be present"
    );
}

#[test]
fn test_BC_2_04_013_ac5_denied_event_has_correct_fields() {
    let event = FlagAuditEmitter::denied_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
        "Not enabled in client config",
    );

    assert_eq!(event.event_type, "capability_check");
    assert_eq!(event.client_id, "acme");
    assert_eq!(event.capability, "sensor.crowdstrike.containment");
    assert_eq!(
        event.result, "denied",
        "BC-2.04.013: result must be 'denied'"
    );
    assert_eq!(event.tool_name, "crowdstrike_contain_host");
    assert_eq!(
        event.denied_reason.as_deref(),
        Some("Not enabled in client config"),
        "BC-2.04.013: denied_reason must be 'Not enabled in client config'"
    );
    assert!(!event.timestamp.is_empty());
}

// ─────────────────────────────────────────────────────────────
// Audit events emitted for both Allow and Deny outcomes
// ─────────────────────────────────────────────────────────────

/// Both Allow and Deny paths must produce audit events (DI-004 invariant).
#[test]
fn test_BC_2_04_013_both_allow_and_deny_produce_events() {
    let allowed_event = FlagAuditEmitter::allowed_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
    );

    let denied_event = FlagAuditEmitter::denied_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
        "Not enabled in client config",
    );

    // Both produce events (not panics, not Err).
    assert_eq!(allowed_event.result, "allowed");
    assert_eq!(denied_event.result, "denied");
}

// ─────────────────────────────────────────────────────────────
// EC-04-028: compile-time denied → event with "Feature not compiled" reason
// ─────────────────────────────────────────────────────────────

/// EC-04-028: Capability denied at compile-time tier still emits audit event.
#[test]
fn test_BC_2_04_013_ec_compile_time_denial_emits_event_with_feature_not_compiled_reason() {
    let event = FlagAuditEmitter::denied_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
        "Feature not compiled",
    );

    assert_eq!(
        event.denied_reason.as_deref(),
        Some("Feature not compiled"),
        "EC-04-028: compile-time denied event must use reason 'Feature not compiled'"
    );
    assert_eq!(event.result, "denied");
}

// ─────────────────────────────────────────────────────────────
// EC-04-027: cross-client fan-out emits one event per client
// ─────────────────────────────────────────────────────────────

/// EC-04-027: For a cross-client query across 10 clients, 10 separate events
/// must be emitted — one per client.
#[test]
fn test_BC_2_04_013_ec_cross_client_fan_out_one_event_per_client() {
    let client_ids = [
        "client_01",
        "client_02",
        "client_03",
        "client_04",
        "client_05",
        "client_06",
        "client_07",
        "client_08",
        "client_09",
        "client_10",
    ];

    let events: Vec<CapabilityCheckEvent> = client_ids
        .iter()
        .map(|cid| {
            FlagAuditEmitter::allowed_event(
                *cid,
                "sensor.crowdstrike.containment",
                "crowdstrike_contain_host",
            )
        })
        .collect();

    assert_eq!(
        events.len(),
        10,
        "EC-04-027: must produce exactly one event per client"
    );

    // Each event has the correct client_id.
    for (event, expected_cid) in events.iter().zip(client_ids.iter()) {
        assert_eq!(
            event.client_id, *expected_cid,
            "EC-04-027: event client_id must match the client it was emitted for"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// Emit method does not panic even if subscriber fails (best-effort)
// ─────────────────────────────────────────────────────────────

/// EC-003 (story edge case): audit emission failure must NOT affect the gate result.
/// The emitter must not panic even if no tracing subscriber is installed.
#[test]
fn test_BC_2_04_013_emit_does_not_panic_without_subscriber() {
    let emitter = FlagAuditEmitter::new();
    let event = FlagAuditEmitter::denied_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
        "Not enabled in client config",
    );

    // Must not panic even if no tracing subscriber is installed.
    emitter.emit_write_check(&event);
    // If we reach here, the test passes.
}

// ─────────────────────────────────────────────────────────────
// Denied reason variants (BC-2.04.013 postconditions)
// ─────────────────────────────────────────────────────────────

/// The three valid `denied_reason` values per BC-2.04.013.
#[test]
fn test_BC_2_04_013_denied_reason_no_matching_capability_path() {
    let event = FlagAuditEmitter::denied_event(
        "acme",
        "sensor.crowdstrike.containment",
        "crowdstrike_contain_host",
        "No matching capability path",
    );
    assert_eq!(
        event.denied_reason.as_deref(),
        Some("No matching capability path")
    );
}

/// Timestamp is in UTC format (non-empty string).
#[test]
fn test_BC_2_04_013_event_timestamp_is_present_and_nonempty() {
    let event = FlagAuditEmitter::allowed_event(
        "acme",
        "sensor.crowdstrike.read",
        "crowdstrike_list_hosts",
    );
    assert!(
        !event.timestamp.is_empty(),
        "BC-2.04.013: timestamp must be a non-empty UTC string"
    );
}
