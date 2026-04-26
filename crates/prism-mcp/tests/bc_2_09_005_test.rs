//! Tests for BC-2.09.005 (AC-2): Trust-Level Metadata — envelope-level tests.
//!
//! Verifies: every envelope carries `_meta.trust_level` at the correct wire format.
//!
//! All tests pass (implementation complete).

use prism_core::TrustLevel;
use prism_mcp::safety_envelope::{DataSource, SafetyEnvelopeBuilder};
use serde_json::json;

/// AC-2: query response envelope has trust_level = "untrusted_external" for sensor data.
#[test]
fn test_BC_2_09_005_ac2_envelope_trust_level_untrusted_external_for_sensor_data() {
    let results = json!([{"hostname": "server.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        false,
        None,
    );

    assert_eq!(
        envelope.meta.trust_level,
        TrustLevel::UntrustedExternal,
        "AC-2: sensor data envelope must have UntrustedExternal trust level"
    );

    let json_val = serde_json::to_value(&envelope).expect("serialize");
    let trust_wire = json_val["_meta"]["trust_level"]
        .as_str()
        .expect("trust_level string");
    assert_eq!(
        trust_wire, "untrusted_external",
        "AC-2: trust_level wire format must be 'untrusted_external'"
    );
}
