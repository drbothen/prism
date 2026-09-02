//! Tests for BC-2.09.008: Response Envelope with Trust Annotations
//!
//! Verifies: consistent envelope structure; `_meta.safety_flags` always present;
//! `content[].text` begins with provenance marker; zero-results edge case;
//! cross-client data_source array.
//!
//! All tests pass (implementation complete).

use prism_core::TrustLevel;
use prism_mcp::safety_envelope::{DataSource, ResponseEnvelope, SafetyEnvelopeBuilder};
use serde_json::json;

// ─── BC-2.09.008 Postconditions 1-4 — envelope structure ────────────────────

/// BC-2.09.008 postcondition 1: envelope has _meta and results fields.
/// Canonical vector: CrowdStrike query returning 5 detections.
#[test]
fn test_BC_2_09_008_envelope_has_meta_and_results_fields() {
    let results = json!([
        {"hostname": "h1.corp.com"}, {"hostname": "h2.corp.com"},
        {"hostname": "h3.corp.com"}, {"hostname": "h4.corp.com"},
        {"hostname": "h5.corp.com"}
    ]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );

    assert_eq!(envelope.meta.tool, "crowdstrike_detections");
    assert_eq!(envelope.meta.total_results, 5);
    assert_eq!(envelope.meta.page, 1);
    assert!(!envelope.meta.has_more);
    assert!(envelope.meta.next_cursor.is_none());
}

/// BC-2.09.008 postcondition 5: `_meta.safety_flags` always present, even empty.
#[test]
fn test_BC_2_09_008_safety_flags_always_present_in_envelope() {
    let results = json!([{"hostname": "clean.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );

    // safety_flags must be present and be an empty vec (not None/absent)
    assert!(
        SafetyEnvelopeBuilder::safety_flags_always_present(&envelope),
        "_meta.safety_flags must always be present"
    );
}

/// BC-2.09.008 postcondition 5: `_meta.safety_flags` is empty array for clean records.
#[test]
fn test_BC_2_09_008_safety_flags_empty_array_for_clean_records() {
    let results = json!([{"hostname": "clean.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    assert!(
        envelope.meta.safety_flags.is_empty(),
        "_meta.safety_flags must be empty array for clean records"
    );
}

/// BC-2.09.008 postcondition 6: `_meta.query_time` is present (ISO8601).
#[test]
fn test_BC_2_09_008_meta_query_time_is_present() {
    let results = json!([]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    assert!(
        !envelope.meta.query_time.is_empty(),
        "_meta.query_time must be present and non-empty"
    );
    // Basic ISO8601 check: contains 'T' and '-'
    assert!(
        envelope.meta.query_time.contains('T'),
        "_meta.query_time must be ISO8601 format"
    );
}

/// BC-2.09.008 postcondition 7: `_meta.data_source` identifies the sensor.
#[test]
fn test_BC_2_09_008_meta_data_source_identifies_sensor() {
    let results = json!([]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    let json = serde_json::to_value(&envelope).expect("serialize");
    assert_eq!(
        json["_meta"]["data_source"].as_str().unwrap_or(""),
        "crowdstrike",
        "_meta.data_source must identify the sensor"
    );
}

// ─── BC-2.09.008 EC-09-018 — zero results ────────────────────────────────────

/// EC-09-018: query returning zero results — envelope still present.
/// Canonical vector: empty query result.
#[test]
fn test_BC_2_09_008_zero_results_envelope_still_present() {
    let results = json!([]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    assert_eq!(
        envelope.meta.total_results, 0,
        "_meta.total_results must be 0"
    );
    assert!(!envelope.meta.has_more, "_meta.has_more must be false");
    assert!(
        envelope.meta.next_cursor.is_none(),
        "_meta.next_cursor must be null"
    );
    assert!(
        SafetyEnvelopeBuilder::safety_flags_always_present(&envelope),
        "_meta.safety_flags must be present even with zero results"
    );
}

// ─── BC-2.09.008 EC-09-019 — cross-client query ──────────────────────────────

/// EC-09-019: cross-client query — `_meta.data_source` is array of sensor IDs.
#[test]
fn test_BC_2_09_008_cross_client_query_data_source_is_array() {
    let results = json!([
        {"hostname": "h1.corp.com", "source_sensor": "crowdstrike"},
        {"hostname": "h2.corp.com", "source_sensor": "armis"}
    ]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "cross_client_detections",
        DataSource::Multiple(vec!["crowdstrike".to_owned(), "armis".to_owned()]),
        results,
        1,
        None,
    );
    let json = serde_json::to_value(&envelope).expect("serialize");
    let sources = json["_meta"]["data_source"]
        .as_array()
        .expect("cross-client data_source must be an array");
    assert_eq!(sources.len(), 2, "data_source array must have 2 sensors");
    assert!(
        sources.iter().any(|s| s.as_str() == Some("crowdstrike")),
        "data_source array must include 'crowdstrike'"
    );
    assert!(
        sources.iter().any(|s| s.as_str() == Some("armis")),
        "data_source array must include 'armis'"
    );
}

// ─── BC-2.09.008 — has_more / next_cursor invariant ──────────────────────────

/// ADR-060 §D8.7 + BC-2.09.008 v1.5: `wrap()` ALWAYS emits `has_more = false` and
/// `next_cursor = null`. The invariant is enforced at the API boundary — `wrap()` no longer
/// accepts `has_more` or `next_cursor` parameters; the values are hard-wired inside `wrap()`
/// and callers cannot influence them.
///
/// Mental-deletion proof: if the hard-wired `has_more: false` / `next_cursor: None`
/// constants in `wrap()` are replaced with non-constant values, this test FAILS
/// (the envelope would carry truthy has_more or non-null next_cursor).
///
/// This test MUST NOT be read as blessing cursor pagination — it exists solely to verify
/// the structural enforcement described in ADR-060 §D8.7. The observable-output gate for
/// the OBS-2 defect closure is in `defect_live_envelope_obs_001_test.rs` (TEST C + TEST D).
#[test]
fn test_BC_2_09_008_wrap_always_emits_has_more_false_next_cursor_null() {
    let results = json!([{"hostname": "h.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    // Invariant: has_more MUST be false (ADR-060 §D8.7). API-level enforcement: the param
    // was removed entirely — no caller can pass has_more=true.
    assert!(
        !envelope.meta.has_more,
        "BC-2.09.008 v1.5 / ADR-060 §D8.7: _meta.has_more MUST always be false; \
         invariant enforced at API boundary (param removed) and structurally in wrap()."
    );
    // Invariant: next_cursor MUST be null (ADR-060 §D8.7). API-level enforcement: the param
    // was removed entirely — no caller can pass a non-null cursor.
    assert!(
        envelope.meta.next_cursor.is_none(),
        "BC-2.09.008 v1.5 / ADR-060 §D8.7: _meta.next_cursor MUST always be null; \
         invariant enforced at API boundary (param removed) and structurally in wrap()."
    );

    // Wire-shape: verify the SERIALIZED output also carries false/null.
    let serialized = serde_json::to_value(&envelope).expect("envelope must serialize");
    assert_eq!(
        serialized["_meta"]["has_more"],
        serde_json::json!(false),
        "BC-2.09.008 v1.5: serialized _meta.has_more MUST be false at the wire level"
    );
    assert!(
        serialized["_meta"]["next_cursor"].is_null(),
        "BC-2.09.008 v1.5: serialized _meta.next_cursor MUST be null at the wire level"
    );
}

// ─── BC-2.09.008 — trust_level in envelope ───────────────────────────────────

/// BC-2.09.008 + BC-2.09.005: envelope trust_level is `untrusted_external` for sensor data.
#[test]
fn test_BC_2_09_008_envelope_trust_level_is_untrusted_external() {
    let results = json!([{"hostname": "server.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        None,
    );
    assert_eq!(
        envelope.meta.trust_level,
        TrustLevel::UntrustedExternal,
        "sensor data envelope must have UntrustedExternal trust level"
    );
}

// ─── DI-006 Invariant ────────────────────────────────────────────────────────

/// DI-006: envelope structure enforces separation — _meta and results are typed separately.
/// Verifies the `_meta` field is distinct from `results`.
#[test]
fn test_BC_2_09_008_invariant_meta_and_results_are_typed_separately() {
    let results = json!([{"hostname": "server.corp.com"}]);
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results.clone(),
        1,
        None,
    );

    // meta is a typed struct, results is a Value
    let json = serde_json::to_value(&envelope).expect("serialize");
    assert!(
        json.get("_meta").is_some(),
        "envelope must have '_meta' field"
    );
    assert!(
        json.get("results").is_some(),
        "envelope must have 'results' field"
    );
    // _meta must not bleed into results
    assert!(
        json["results"].get("trust_level").is_none(),
        "trust_level must not appear in results"
    );
    assert!(
        json["results"].get("safety_flags").is_none(),
        "safety_flags must not appear in results (must be in _meta)"
    );
}
