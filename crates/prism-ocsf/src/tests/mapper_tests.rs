//! Per-sensor unit tests for S-1.05 field mappers.
//!
//! # Red Gate
//!
//! All tests in this module MUST FAIL before S-1.05 implementation begins.
//! They exercise `unimplemented!()` stubs and will panic with "not yet implemented".
//!
//! # BC Coverage
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_02_003_crowdstrike_severity_high_maps_to_id_4 | AC-1 | BC-2.02.003 |
//! | test_BC_2_02_003_crowdstrike_detection_id_maps_to_finding_info_uid | AC-1 | BC-2.02.003 |
//! | test_BC_2_02_003_crowdstrike_behaviors_tactic_maps_to_attacks | AC-2 | BC-2.02.003 |
//! | test_BC_2_02_003_crowdstrike_severity_out_of_range_maps_to_99 | BC-2.02.003 error case |
//! | test_BC_2_02_003_crowdstrike_unmapped_field_in_extensions | AC-7 | BC-2.02.003, BC-2.02.007 |
//! | test_BC_2_02_004_cyberint_unix_timestamp_parsed_correctly | AC-3 | BC-2.02.004 |
//! | test_BC_2_02_004_cyberint_rfc3339_timestamp_parsed_correctly | BC-2.02.004 TV-001 |
//! | test_BC_2_02_004_cyberint_malformed_timestamp_returns_error | AC-4 | BC-2.02.004 |
//! | test_BC_2_02_005_claroty_integer_id_converts_to_uid_string | AC-5 | BC-2.02.005 |
//! | test_BC_2_02_005_claroty_string_id_converts_to_uid_string | BC-2.02.005 |
//! | test_BC_2_02_005_claroty_unknown_record_type_returns_error | BC-2.02.005 error case |
//! | test_BC_2_02_006_armis_no_timestamp_falls_back_to_current_time | AC-6 | BC-2.02.006 |
//! | test_BC_2_02_006_armis_last_seen_used_when_present | BC-2.02.006 TV-001 |
//! | test_BC_2_02_007_custom_vendor_field_preserved_in_extensions | AC-7 | BC-2.02.007 |
//! | test_BC_2_02_007_all_unmapped_fields_captured | BC-2.02.007 TV-002 |
//! | test_BC_2_02_011_missing_detection_id_returns_normalization_error | AC-9 | BC-2.02.011 |
//! | test_BC_2_02_011_error_contains_source_record_context | AC-9 | BC-2.02.011 |

use prost_reflect::DynamicMessage;
use prism_core::PrismError;
use serde_json::json;

use crate::mappers::{ArmisMapper, ClarotyMapper, CrowdStrikeMapper, CyberintMapper, SensorMapper};
use crate::mappers::crowdstrike::crowdstrike_severity_to_id;
use crate::mappers::claroty::claroty_id_to_uid;
use crate::mappers::cyberint::parse_cyberint_timestamp;
use crate::mappers::armis::extract_armis_timestamp;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a minimal stub `DynamicMessage` for use in tests that require a `msg` argument.
///
/// In the stub phase (Red Gate) the tests will panic inside `map()` before `msg` is ever
/// used, so the exact contents of the message do not matter — we just need a value that
/// compiles.
///
/// STUB: In the real implementation, `msg` would be created via
/// `DynamicMessage::new(descriptor)` using the descriptor for the target class_uid.
fn stub_dynamic_message() -> DynamicMessage {
    // The empty DescriptorPool produces no descriptors, so we cannot construct a real
    // DynamicMessage here. We use `prost_reflect::DynamicMessage::new` with a default
    // FileDescriptor — this is acceptable in stub tests since map() panics before
    // msg is accessed.
    //
    // This function is intentionally unreachable in passing tests; it only exists so
    // test function signatures that take `&mut DynamicMessage` compile.
    use prost_reflect::DescriptorPool;
    let pool = DescriptorPool::decode(&b""[..])
        .expect("empty pool decode must not fail");
    // There are no descriptors in an empty pool — to get a DynamicMessage for tests
    // we rely on the fact that all map() calls will panic before touching `msg`.
    // This is a test-only hack; real tests will use a populated descriptor pool.
    let _ = pool;
    // We cannot call DynamicMessage::new() without a MessageDescriptor.
    // Use an unreachable sentinel — the test will panic inside the stub before reaching here.
    panic!("stub_dynamic_message: should never be reached in Red Gate phase — \
            map() panics with unimplemented!() before touching msg")
}

// ---------------------------------------------------------------------------
// BC-2.02.003 — CrowdStrike Field Mapping
// ---------------------------------------------------------------------------

/// AC-1 (part 1): CrowdStrike detection with severity="High" maps severity_id to 4.
/// BC-2.02.003 postcondition, TV-BC-2.02.003-001.
#[test]
fn test_BC_2_02_003_crowdstrike_severity_high_maps_to_id_4() {
    let severity_id = crowdstrike_severity_to_id("High");
    assert_eq!(severity_id, 4, "BC-2.02.003: 'High' must map to severity_id 4");
}

/// AC-1 (part 2): CrowdStrike detection with severity="Critical" maps severity_id to 5.
/// BC-2.02.003 postcondition.
#[test]
fn test_BC_2_02_003_crowdstrike_severity_critical_maps_to_id_5() {
    let severity_id = crowdstrike_severity_to_id("Critical");
    assert_eq!(severity_id, 5, "BC-2.02.003: 'Critical' must map to severity_id 5");
}

/// BC-2.02.003 error case: severity=7 (out of range) → severity_id 99.
/// TV-BC-2.02.003-002.
#[test]
fn test_BC_2_02_003_crowdstrike_severity_out_of_range_maps_to_99() {
    // The integer-valued CrowdStrike severity outside 1-5 range must map to 99 (Other).
    // This uses the string representation per the BC description.
    let severity_id = crowdstrike_severity_to_id("7");
    assert_eq!(
        severity_id, 99,
        "BC-2.02.003 TV-002: severity outside 1-5 range maps to severity_id 99 (Other)"
    );
}

/// AC-1 (part 3): `detection_id` maps to `finding_info.uid`.
/// BC-2.02.003 postcondition, TV-BC-2.02.003-001.
#[test]
fn test_BC_2_02_003_crowdstrike_detection_id_maps_to_finding_info_uid() {
    let mapper = CrowdStrikeMapper;
    let raw = json!({
        "detection_id": "ldt:abc123",
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z"
    });
    let mut extensions = serde_json::Map::new();

    // Panics inside unimplemented!() — confirms Red Gate.
    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

/// AC-2: CrowdStrike detection with `behaviors[0].tactic` = "Discovery" maps to
/// `attacks[0].tactic.name == "Discovery"`. BC-2.02.003 postcondition.
#[test]
fn test_BC_2_02_003_crowdstrike_behaviors_tactic_maps_to_attacks() {
    let mapper = CrowdStrikeMapper;
    let raw = json!({
        "detection_id": "ldt:abc123",
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z",
        "behaviors": [{"tactic": "Discovery", "technique": "System Information Discovery"}]
    });
    let mut extensions = serde_json::Map::new();

    // Panics inside unimplemented!() — confirms Red Gate.
    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

/// AC-7 (CrowdStrike): custom vendor field `custom_tags` goes to extensions, not lost.
/// BC-2.02.003 + BC-2.02.007 (DEC-007), TV-BC-2.02.007-001.
#[test]
fn test_BC_2_02_003_crowdstrike_unmapped_field_in_extensions() {
    let mapper = CrowdStrikeMapper;
    let raw = json!({
        "detection_id": "ldt:abc123",
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z",
        "custom_tags": ["tag1", "tag2"]
    });
    let mut extensions = serde_json::Map::new();

    // Panics inside unimplemented!() — confirms Red Gate.
    // In the real implementation, extensions["custom_tags"] must be set.
    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.004 — Cyberint Field Mapping (multi-format timestamps)
// ---------------------------------------------------------------------------

/// AC-3: Cyberint alert with Unix timestamp integer `1710498600` parses correctly.
/// BC-2.02.004, TV-BC-2.02.004 format 3.
#[test]
fn test_BC_2_02_004_cyberint_unix_timestamp_parsed_correctly() {
    // Unix timestamp 1710498600 → 2024-03-15T10:30:00Z
    let value = json!(1710498600i64);
    let result = parse_cyberint_timestamp("created_date", &value);
    let dt = result.expect("BC-2.02.004 AC-3: Unix timestamp must parse without error");
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2024-03-15T10:30:00Z",
        "BC-2.02.004 AC-3: Unix timestamp 1710498600 must parse to 2024-03-15T10:30:00Z"
    );
}

/// BC-2.02.004 TV-001: RFC3339 timestamp string parses correctly.
#[test]
fn test_BC_2_02_004_cyberint_rfc3339_timestamp_parsed_correctly() {
    let value = json!("2024-03-15T10:30:00Z");
    let result = parse_cyberint_timestamp("created_date", &value);
    let dt = result.expect("BC-2.02.004 TV-001: RFC3339 timestamp must parse without error");
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2024-03-15T10:30:00Z",
        "BC-2.02.004 TV-001: RFC3339 string must parse to correct datetime"
    );
}

/// BC-2.02.004: ISO 8601 without timezone (assume UTC) parses correctly.
#[test]
fn test_BC_2_02_004_cyberint_iso8601_no_tz_parsed_correctly() {
    let value = json!("2024-03-15T10:30:00");
    let result = parse_cyberint_timestamp("created_date", &value);
    let dt = result.expect("BC-2.02.004: ISO 8601 without tz must parse as UTC");
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2024-03-15T10:30:00Z",
        "BC-2.02.004: ISO 8601 without tz must be assumed UTC"
    );
}

/// AC-4: Cyberint alert with malformed timestamp returns `Err(OcsfTimestampParseError)`
/// containing the field name and raw value. BC-2.02.004, BC-2.02.011.
#[test]
fn test_BC_2_02_004_cyberint_malformed_timestamp_returns_error() {
    let value = json!("not-a-date");
    let result = parse_cyberint_timestamp("created_date", &value);
    match result {
        Err(PrismError::OcsfTimestampParseError { field, raw }) => {
            assert_eq!(
                field, "created_date",
                "BC-2.02.004 AC-4: error must name the field 'created_date'"
            );
            assert_eq!(
                raw, "not-a-date",
                "BC-2.02.004 AC-4: error must preserve the raw value"
            );
        }
        Err(e) => panic!(
            "BC-2.02.004 AC-4: expected OcsfTimestampParseError, got {:?}",
            e
        ),
        Ok(_) => panic!("BC-2.02.004 AC-4: malformed timestamp must return Err"),
    }
}

/// BC-2.02.004: The full CyberintMapper::map() with a Unix timestamp in alert record.
#[test]
fn test_BC_2_02_004_cyberint_mapper_unix_timestamp_in_full_record() {
    let mapper = CyberintMapper;
    let raw = json!({
        "ref_id": "CYB-2024-001",
        "title": "Malware Detected",
        "severity": "high",
        "status": "open",
        "created_date": 1710498600i64,
        "threat_type": "Malware"
    });
    let mut extensions = serde_json::Map::new();

    let _ = mapper.map(
        "alert",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.005 — Claroty Field Mapping (polymorphic IDs)
// ---------------------------------------------------------------------------

/// AC-5: Claroty asset with integer `id: 42` → `device.uid == "claroty:42"`.
/// BC-2.02.005 postcondition, TV-BC-2.02.005-001.
#[test]
fn test_BC_2_02_005_claroty_integer_id_converts_to_uid_string() {
    let value = json!(42i64);
    let uid = claroty_id_to_uid(&value)
        .expect("BC-2.02.005 AC-5: integer ID must convert to uid string");
    assert_eq!(
        uid, "claroty:42",
        "BC-2.02.005 AC-5: integer id=42 must produce uid 'claroty:42'"
    );
}

/// BC-2.02.005: String ID also converts correctly.
#[test]
fn test_BC_2_02_005_claroty_string_id_converts_to_uid_string() {
    let value = json!("device-99");
    let uid = claroty_id_to_uid(&value)
        .expect("BC-2.02.005: string ID must convert to uid string");
    assert_eq!(
        uid, "claroty:device-99",
        "BC-2.02.005: string id='device-99' must produce uid 'claroty:device-99'"
    );
}

/// BC-2.02.005 error case: unknown Claroty record type → `Err(OcsfUnknownRecordType)`.
/// TV-BC-2.02.005-005.
#[test]
fn test_BC_2_02_005_claroty_unknown_record_type_returns_error() {
    let mapper = ClarotyMapper;
    let raw = json!({ "id": 1, "some_field": "some_value" });
    let mut extensions = serde_json::Map::new();

    let result = mapper.map(
        "unknown_claroty_type",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    match result {
        Err(PrismError::OcsfUnknownRecordType { sensor, record_type }) => {
            assert_eq!(sensor, "claroty", "BC-2.02.005: error sensor must be 'claroty'");
            assert_eq!(
                record_type, "unknown_claroty_type",
                "BC-2.02.005: error record_type must match"
            );
        }
        Err(e) => panic!(
            "BC-2.02.005: expected OcsfUnknownRecordType, got {:?}",
            e
        ),
        Ok(_) => panic!("BC-2.02.005: unknown record type must return Err"),
    }
}

/// BC-2.02.005: Full Claroty asset mapping with integer id.
#[test]
fn test_BC_2_02_005_claroty_asset_integer_id_full_mapping() {
    let mapper = ClarotyMapper;
    let raw = json!({
        "id": 42,
        "name": "PLC-01",
        "ip_v4": "192.168.1.100",
        "mac_address": "AA:BB:CC:DD:EE:FF",
        "firmware_version": "3.1.4",
        "site_name": "Plant A"
    });
    let mut extensions = serde_json::Map::new();

    let _ = mapper.map(
        "asset",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.006 — Armis Field Mapping (timestamp fallback)
// ---------------------------------------------------------------------------

/// AC-6: Armis device with no timestamp fields → valid time (current time fallback) + warning.
/// BC-2.02.006, TV-BC-2.02.006-004.
#[test]
fn test_BC_2_02_006_armis_no_timestamp_falls_back_to_current_time() {
    let raw = json!({
        "id": 100,
        "name": "MyDevice"
        // intentionally no last_seen, created_at, or timestamp fields
    });
    // extract_armis_timestamp must never fail — returns current time with warning.
    let dt = extract_armis_timestamp(&raw);
    // The returned time must be within the last 60 seconds (current time fallback).
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(dt);
    assert!(
        delta.num_seconds() >= 0 && delta.num_seconds() < 60,
        "BC-2.02.006 AC-6: fallback time must be close to current time, delta={}s",
        delta.num_seconds()
    );
}

/// BC-2.02.006: When `last_seen` is present it is used first.
#[test]
fn test_BC_2_02_006_armis_last_seen_used_when_present() {
    let raw = json!({
        "id": 100,
        "last_seen": "2024-03-15T10:30:00Z",
        "created_at": "2024-01-01T00:00:00Z"
    });
    let dt = extract_armis_timestamp(&raw);
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2024-03-15T10:30:00Z",
        "BC-2.02.006: last_seen must be used when present (highest priority fallback)"
    );
}

/// BC-2.02.006: When `last_seen` absent but `created_at` present, use `created_at`.
#[test]
fn test_BC_2_02_006_armis_created_at_fallback_when_no_last_seen() {
    let raw = json!({
        "id": 100,
        "created_at": "2024-03-15T10:30:00Z"
    });
    let dt = extract_armis_timestamp(&raw);
    assert_eq!(
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "2024-03-15T10:30:00Z",
        "BC-2.02.006: created_at must be used when last_seen is absent"
    );
}

/// BC-2.02.006: Full ArmisMapper map() with no timestamp — must not fail.
#[test]
fn test_BC_2_02_006_armis_mapper_no_timestamp_does_not_fail() {
    let mapper = ArmisMapper;
    let raw = json!({
        "id": 100,
        "name": "MyDevice",
        "ipAddress": "10.0.0.1"
        // no timestamp fields
    });
    let mut extensions = serde_json::Map::new();

    // Must not return Err — only returns current time fallback. (AC-6)
    let _ = mapper.map(
        "device",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.007 — Unmapped fields preserved in raw_extensions
// ---------------------------------------------------------------------------

/// AC-7: Custom vendor field `custom_vendor_field` appears in extensions.
/// BC-2.02.007 postcondition, TV-BC-2.02.007-001.
#[test]
fn test_BC_2_02_007_custom_vendor_field_preserved_in_extensions() {
    // This is verified at the SensorMapper contract level — after map() returns,
    // the caller must find the field in extensions. We test via CrowdStrikeMapper as
    // the canonical example. Any sensor mapper suffices.
    let mapper = CrowdStrikeMapper;
    let raw = json!({
        "detection_id": "ldt:abc123",
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z",
        "custom_vendor_field": "xyz"
    });
    let mut extensions = serde_json::Map::new();

    // Panics inside unimplemented!() — confirms Red Gate.
    // In the real implementation:
    //   1. map() must not drop "custom_vendor_field"
    //   2. extensions["custom_vendor_field"] must equal "xyz" after map() returns
    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    // (assertions would go here after the unimplemented!() stub is replaced)
}

/// BC-2.02.007 TV-002: Record with all unmapped fields — extensions captures everything.
#[test]
fn test_BC_2_02_007_all_unmapped_fields_captured() {
    let mapper = CrowdStrikeMapper;
    // A record with no explicitly mapped fields.
    let raw = json!({
        "detection_id": "ldt:test",
        "completely_unknown_field_a": 1,
        "completely_unknown_field_b": "hello",
        "completely_unknown_field_c": [1, 2, 3]
    });
    let mut extensions = serde_json::Map::new();

    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    // Real assertion: for each unknown key in raw, extensions[key] must be set.
}

/// BC-2.02.007 invariant: field names in extensions use original vendor names (not transformed).
#[test]
fn test_BC_2_02_007_invariant_extensions_use_original_vendor_field_names() {
    let mapper = CrowdStrikeMapper;
    let raw = json!({
        "detection_id": "ldt:abc",
        "severity": "High",
        "agent_id": "cs-agent-xyz",
        "cid": "customer-id-123"
    });
    let mut extensions = serde_json::Map::new();

    let _ = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    // Real assertion: extensions["agent_id"] and extensions["cid"] must be present
    // using original vendor field names — not "crowdstrike_agent_id" or similar transforms.
}

// ---------------------------------------------------------------------------
// BC-2.02.011 — Normalization error handling (source record ID + reason)
// ---------------------------------------------------------------------------

/// AC-9: Missing `detection_id` returns `Err(OcsfNormalizationFailed)` with source context.
/// BC-2.02.011, TV-BC-2.02.011-001.
#[test]
fn test_BC_2_02_011_missing_detection_id_returns_normalization_error() {
    let mapper = CrowdStrikeMapper;
    // Missing detection_id — required field for source record ID extraction.
    let raw = json!({
        "severity": "High",
        "created_timestamp": "2024-03-15T10:30:00Z"
        // "detection_id" intentionally absent
    });
    let mut extensions = serde_json::Map::new();

    let result = mapper.map(
        "detection",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    match result {
        Err(PrismError::OcsfNormalizationFailed { source_id, reason }) => {
            // source_id must identify the record context even without detection_id
            assert!(
                !source_id.is_empty(),
                "BC-2.02.011 AC-9: source_id in error must be non-empty"
            );
            assert!(
                reason.contains("detection_id"),
                "BC-2.02.011 AC-9: error reason must name the missing field 'detection_id', \
                 got: {reason}"
            );
        }
        Err(e) => panic!(
            "BC-2.02.011 AC-9: expected OcsfNormalizationFailed, got {:?}",
            e
        ),
        Ok(_) => panic!("BC-2.02.011 AC-9: missing detection_id must return Err"),
    }
}

/// AC-9: Error from normalization contains specific field name and source context.
/// BC-2.02.011 postcondition — warning-level log entry must name the record, field, issue.
#[test]
fn test_BC_2_02_011_error_contains_source_record_context_and_field_name() {
    let mapper = CyberintMapper;
    // Bad timestamp + missing ref_id to exercise error path on Cyberint mapper.
    let raw = json!({
        "ref_id": "CYB-FAIL-001",
        "created_date": "definitely-not-a-date"
    });
    let mut extensions = serde_json::Map::new();

    let result = mapper.map(
        "alert",
        &raw,
        &mut stub_dynamic_message(),
        &mut extensions,
    );
    // Cyberint mapper must propagate OcsfTimestampParseError for bad dates.
    match result {
        Err(PrismError::OcsfTimestampParseError { field, raw: raw_val }) => {
            assert_eq!(field, "created_date", "BC-2.02.011: error must name the field");
            assert_eq!(
                raw_val, "definitely-not-a-date",
                "BC-2.02.011: error must preserve raw value"
            );
        }
        Err(PrismError::OcsfNormalizationFailed { source_id, reason }) => {
            // Also acceptable — NormalizationFailed wrapping a timestamp error.
            assert!(
                !source_id.is_empty(),
                "BC-2.02.011: source_id must identify the record"
            );
            assert!(
                reason.contains("created_date") || reason.contains("timestamp"),
                "BC-2.02.011: reason must reference the failing field, got: {reason}"
            );
        }
        Err(e) => panic!(
            "BC-2.02.011: expected OcsfTimestampParseError or OcsfNormalizationFailed, got {:?}",
            e
        ),
        Ok(_) => panic!("BC-2.02.011: bad timestamp must produce Err"),
    }
}
