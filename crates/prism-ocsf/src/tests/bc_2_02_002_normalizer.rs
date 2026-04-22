//! Tests for BC-2.02.002 — DynamicMessage Creation from Sensor Records.
//!
//! BC: `OcsfNormalizer::normalize()` creates a `DynamicMessage` wrapping the target OCSF
//! event class protobuf descriptor, then sets mapped fields via `prost-reflect` runtime
//! field access. The resulting `OcsfEvent` bundles the `DynamicMessage`, `raw_extensions`,
//! `source_sensor`, and `source_record_type`.
//!
//! Acceptance Criteria covered:
//! - AC-3: Raw CrowdStrike detection JSON → DynamicMessage with class_uid = 2004.
//! - AC-9: VP-016 proptest — all `Ok` outputs are valid protobuf (encode→decode round-trip).
//!
//! Test Vectors (BC-2.02.002):
//! - TV-BC-2.02.002-001: Well-formed CrowdStrike alert → DynamicMessage valid.
//! - TV-BC-2.02.002-002: Empty JSON `{}` → DynamicMessage with no fields; warning logged.
//! - TV-BC-2.02.002-003: Field type mismatch string "42" for severity_id → coercion succeeds.
//! - TV-BC-2.02.002-004: Non-numeric string for severity_id → coercion fails; raw_extensions.
//! - TV-BC-2.02.002-005: DynamicMessage encoding fails → record skipped; cursor advances.
//!
//! # Red Gate
//!
//! ALL tests in this file MUST FAIL until ocsf-proto-gen is provisioned AND S-1.05
//! field mappers are implemented. Expected failure mode:
//! - `normalize()` returns `Err(OcsfDescriptorNotFound)` because the descriptor pool is
//!   empty (stub). Tests that assert `Ok(DynamicMessage)` will panic on `unwrap()`.

use prost_reflect::ReflectMessage;
use serde_json::json;

use crate::normalizer::OcsfNormalizer;
use prism_core::PrismError;

/// BC-2.02.002 / AC-3 / TV-BC-2.02.002-001:
/// Raw CrowdStrike detection JSON produces a DynamicMessage.
///
/// # Red Gate
///
/// MUST FAIL — descriptor pool is empty (stub). normalize() returns OcsfDescriptorNotFound.
#[test]
fn test_BC_2_02_002_crowdstrike_detection_produces_dynamic_message() {
    let normalizer = OcsfNormalizer::new();
    let raw = json!({
        "detection_id": "abc123",
        "severity": "High"
    });

    let result = normalizer.normalize("crowdstrike", "detection", raw);

    // RED GATE: will fail because descriptor pool is empty (stub build.rs).
    // When ocsf-proto-gen is available, this should return Ok(DynamicMessage).
    assert!(
        result.is_ok(),
        "normalize('crowdstrike', 'detection', ...) must return Ok(DynamicMessage) \
         (AC-3, BC-2.02.002) — RED GATE: fails until ocsf-proto-gen + S-1.05 land; \
         actual error: {:?}",
        result.unwrap_err()
    );
}

/// BC-2.02.002 / AC-3: The returned DynamicMessage has the correct class_uid.
///
/// TV-BC-2.02.002-001: class_uid field on the message equals 2004.
///
/// # Red Gate
///
/// MUST FAIL — descriptor pool is empty (stub).
#[test]
fn test_BC_2_02_002_normalized_message_has_class_uid_2004() {
    let normalizer = OcsfNormalizer::new();
    let raw = json!({
        "detection_id": "abc123",
        "severity": "High"
    });

    let result = normalizer.normalize("crowdstrike", "detection", raw);

    // RED GATE
    let message = result.expect(
        "normalize() must succeed for well-formed CrowdStrike detection — \
         RED GATE: descriptor pool empty (stub)",
    );

    let class_uid_field = message
        .descriptor()
        .get_field_by_name("class_uid")
        .expect("DynamicMessage descriptor must have a 'class_uid' field (AC-3)");

    let class_uid_value = message.get_field(&class_uid_field);
    // The class_uid field value on an OCSF Detection Finding must be 2004.
    // (BC-2.02.002 postcondition, AC-3)
    assert_eq!(
        class_uid_value.as_u32(),
        Some(2004),
        "normalized message class_uid must be 2004 (Detection Finding) (AC-3, BC-2.02.002)"
    );
}

/// BC-2.02.002 / TV-BC-2.02.002-002: empty JSON record `{}` produces a DynamicMessage
/// with no mapped fields.
///
/// # Red Gate
///
/// MUST FAIL — descriptor pool is empty (stub).
#[test]
fn test_BC_2_02_002_empty_json_produces_dynamic_message() {
    let normalizer = OcsfNormalizer::new();
    let raw = json!({});

    // BC-2.02.002 EC-02-003: DynamicMessage created with no mapped fields;
    // all OCSF fields absent; raw_extensions is {}; valid but minimally useful.
    // RED GATE: descriptor pool is empty.
    let result = normalizer.normalize("crowdstrike", "detection", raw);
    assert!(
        result.is_ok(),
        "normalize() with empty JSON must return Ok(DynamicMessage) — \
         RED GATE: descriptor pool empty (stub); actual: {:?}",
        result.unwrap_err()
    );
}

/// BC-2.02.002: normalize() with unknown sensor returns Err(OcsfUnknownEventClass).
///
/// This test exercises the AC-8 path through normalize().
/// Red Gate: PASSES with the stub — EventClassSelector returns Err before descriptor lookup.
#[test]
fn test_BC_2_02_002_unknown_sensor_returns_err() {
    let normalizer = OcsfNormalizer::new();
    let raw = json!({ "id": "test" });

    let result = normalizer.normalize("vendor_x", "unknown_type", raw);

    assert!(result.is_err(), "normalize() with unknown sensor must return Err");
    match result.unwrap_err() {
        PrismError::OcsfUnknownEventClass { sensor, record_type } => {
            assert_eq!(sensor, "vendor_x");
            assert_eq!(record_type, "unknown_type");
        }
        other => panic!(
            "Expected OcsfUnknownEventClass, got {:?} (BC-2.02.002 / AC-8)",
            other
        ),
    }
}

/// BC-2.02.002: normalize() never panics on malformed JSON (VP-022 unit-level check).
///
/// Red Gate: PASSES with the stub — normalize() propagates errors via Result.
#[test]
fn test_BC_2_02_002_malformed_input_does_not_panic() {
    let normalizer = OcsfNormalizer::new();

    // Null value
    let _ = normalizer.normalize("crowdstrike", "detection", serde_json::Value::Null);

    // Boolean
    let _ = normalizer.normalize("crowdstrike", "detection", json!(true));

    // Array instead of object
    let _ = normalizer.normalize("crowdstrike", "detection", json!([1, 2, 3]));

    // Deeply nested
    let _ = normalizer.normalize("crowdstrike", "detection", json!({"a": {"b": {"c": {}}}}));

    // Very long string value
    let long_str = "x".repeat(1_000_000);
    let _ = normalizer.normalize("crowdstrike", "detection", json!({ "field": long_str }));
}

/// BC-2.02.002 postcondition: normalize() for a valid known sensor returns either
/// `Ok(DynamicMessage)` or a typed `Err` — never an unstructured panic or unwrap.
///
/// This verifies the error variant taxonomy from the story spec is enforced.
/// Red Gate: PASSES for the error-path check; the Ok-path check is Red Gate.
#[test]
fn test_BC_2_02_002_known_sensor_returns_typed_error_or_ok() {
    let normalizer = OcsfNormalizer::new();
    let raw = json!({ "detection_id": "x" });
    let result = normalizer.normalize("crowdstrike", "detection", raw);

    match result {
        Ok(_msg) => {
            // The real implementation will reach here.
            // No additional assertions — the round-trip test covers Ok quality.
        }
        Err(PrismError::OcsfDescriptorNotFound { class_uid }) => {
            // Expected from the stub — descriptor pool is empty.
            assert_eq!(
                class_uid, 2004,
                "OcsfDescriptorNotFound class_uid must be 2004 for crowdstrike/detection"
            );
        }
        Err(PrismError::OcsfNormalizationFailed { .. }) => {
            // Also acceptable — catch-all normalization failure.
        }
        Err(other) => {
            panic!(
                "normalize() returned unexpected error variant {:?} \
                 (BC-2.02.002 — only OcsfDescriptorNotFound or OcsfNormalizationFailed allowed \
                 for a known sensor+record_type)",
                other
            );
        }
    }
}

/// VP-016 / BC-2.02.002 postcondition: `Ok(DynamicMessage)` encodes to bytes and decodes
/// back to an equivalent message (valid protobuf round-trip).
///
/// # Red Gate
///
/// MUST FAIL — normalize() returns Err from the stub, so the round-trip assertion
/// is never reached. Proptest coverage is in `proptest_normalizer.rs`.
#[test]
fn test_BC_2_02_002_vp016_dynamic_message_round_trips() {
    use prost::Message;
    use prost_reflect::DynamicMessage;

    let normalizer = OcsfNormalizer::new();
    let raw = json!({
        "detection_id": "round_trip_test",
        "severity": "High"
    });

    // RED GATE: will fail because normalize() returns Err from the stub.
    let message = normalizer
        .normalize("crowdstrike", "detection", raw)
        .expect(
            "normalize() must succeed for VP-016 round-trip — \
             RED GATE: fails until ocsf-proto-gen + real pool is available",
        );

    // Encode to bytes.
    let mut bytes = Vec::new();
    message.encode(&mut bytes).expect("DynamicMessage must encode to bytes (VP-016)");

    // Decode back.
    let descriptor = message.descriptor();
    let decoded = DynamicMessage::decode(descriptor.clone(), bytes.as_slice())
        .expect("Decoded bytes must produce a valid DynamicMessage (VP-016)");

    assert_eq!(
        message, decoded,
        "DynamicMessage encode→decode round-trip must be identity (VP-016, BC-2.02.002)"
    );
}
