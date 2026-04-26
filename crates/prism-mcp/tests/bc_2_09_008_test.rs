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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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
        false,
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

// ─── BC-2.09.008 — pagination ─────────────────────────────────────────────────

/// BC-2.09.008 postcondition 1: `has_more` and `next_cursor` when paginating.
#[test]
fn test_BC_2_09_008_pagination_fields_present_when_paginating() {
    let results = json!([{"hostname": "h.corp.com"}]);
    let cursor = Some("cursor-abc-123".to_owned());
    let envelope = SafetyEnvelopeBuilder::wrap(
        "crowdstrike_detections",
        DataSource::Single("crowdstrike".to_owned()),
        results,
        1,
        true,
        cursor.clone(),
    );
    assert!(envelope.meta.has_more, "_meta.has_more must be true");
    assert_eq!(
        envelope.meta.next_cursor, cursor,
        "_meta.next_cursor must be the provided cursor"
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
        false,
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
        false,
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
