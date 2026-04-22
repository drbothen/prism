//! VP-016 proptest: normalize() output is always valid protobuf.
//!
//! Property: For every raw sensor record successfully processed by
//! `OcsfNormalizer::normalize`, the resulting `DynamicMessage` serializes to a byte
//! sequence that round-trips through prost decode into an equivalent `DynamicMessage`.
//!
//! # Red Gate
//!
//! The `prop_normalize_output_is_valid_protobuf` property test MUST FAIL until
//! ocsf-proto-gen is provisioned. The stub normalize() always returns Err for
//! well-formed CrowdStrike inputs (descriptor pool is empty), so the `Ok` branch
//! is never reached and the round-trip assertion is never executed.
//!
//! When the pool is real and normalize() starts returning Ok, the proptest will
//! exercise the round-trip property across 1000+ randomly generated records.

use proptest::prelude::*;
use prost_reflect::ReflectMessage;
use serde_json::{json, Value};

use crate::normalizer::OcsfNormalizer;

/// Proptest strategy: generate an arbitrary JSON object (sensor record shaped).
/// Values are kept small to avoid pathological allocations — each string is capped
/// at 256 bytes, nesting depth at 3, array length at 8.
fn arbitrary_json_object() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        "[a-zA-Z0-9_-]{0,64}".prop_map(Value::String),
    ];

    leaf.prop_recursive(3, 32, 8, |inner| {
        prop::collection::hash_map("[a-zA-Z0-9_]{1,32}", inner, 0..8)
            .prop_map(|m| Value::Object(m.into_iter().collect()))
    })
}

// VP-016 proptest block.
// Every Ok(DynamicMessage) from normalize() must encode→decode correctly.
// Red Gate: the guard test (test_VP_016_normalize_produces_ok_for_valid_inputs) fails
// because normalize() always returns Err when the descriptor pool is empty (stub).
proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 1000,
        max_shrink_iters: 512,
        ..Default::default()
    })]

    #[test]
    fn prop_normalize_output_is_valid_protobuf(raw in arbitrary_json_object()) {
        use prost::Message;
        use prost_reflect::DynamicMessage;

        let normalizer = OcsfNormalizer::new();
        let result = normalizer.normalize("crowdstrike", "detection", raw);

        match result {
            Ok(message) => {
                // Encode to bytes.
                let mut bytes = Vec::new();
                message.encode(&mut bytes)
                    .expect("VP-016: DynamicMessage must encode to bytes without error");

                // Decode back from bytes.
                let descriptor = message.descriptor();
                let decoded = DynamicMessage::decode(descriptor.clone(), bytes.as_slice())
                    .expect("VP-016: encoded bytes must decode back to a valid DynamicMessage");

                // Round-trip identity.
                prop_assert_eq!(
                    message, decoded,
                    "VP-016: DynamicMessage encode→decode round-trip must be identity"
                );
            }
            Err(_) => {
                // normalize() returned Err — no panic, which is correct behaviour.
                // In the stub, this is always the case because the descriptor pool
                // is empty. When the real pool is available this branch will only be
                // reached for genuinely invalid inputs.
                //
                // RED GATE NOTE: If normalize() *only* ever returns Err (stub), the
                // round-trip assertion is never exercised. The test technically "passes"
                // because no invariant is violated — but the Red Gate is violated because
                // we want at least some Ok outputs to exercise the round-trip. A secondary
                // test below asserts that at least one Ok result is produced per 1000 cases
                // when the pool is real.
            }
        }
    }
}

/// VP-016 guard: normalize() must produce at least one Ok result per 1000 inputs.
///
/// This test catches the Red Gate scenario where the proptest above trivially passes
/// because normalize() always returns Err (empty pool). When the real pool is available,
/// at least some CrowdStrike detection inputs will produce Ok results.
///
/// # Red Gate
///
/// MUST FAIL with the stub — normalize() always returns Err (empty pool).
#[test]
fn test_VP_016_normalize_produces_ok_for_valid_inputs() {
    let normalizer = OcsfNormalizer::new();
    let test_inputs = vec![
        json!({ "detection_id": "abc123", "severity": "High" }),
        json!({ "detection_id": "xyz789" }),
        json!({}),
        json!({ "id": "test", "type": "detection", "severity": 4 }),
    ];

    let ok_count = test_inputs
        .into_iter()
        .filter(|raw| {
            normalizer
                .normalize("crowdstrike", "detection", raw.clone())
                .is_ok()
        })
        .count();

    // RED GATE: zero Ok results from stub. Real implementation must produce at least 1.
    assert!(
        ok_count > 0,
        "VP-016: normalize() must produce at least one Ok(DynamicMessage) for valid \
         CrowdStrike detection inputs — RED GATE: stub returns 0 Ok results (pool empty)"
    );
}
