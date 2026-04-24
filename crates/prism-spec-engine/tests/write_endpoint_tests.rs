//! Write endpoint tests for S-1.13.
//!
//! Every test exercises a specific BC clause, AC, or edge case from the story spec.
//! Test naming: `test_BC_S_SS_NNN_xxx()` for BC-traced tests.
//!
//! Red Gate: ALL tests must fail (via unimplemented! panic) before implementation.
//!
//! # Test Coverage Map
//!
//! | Test | BC | AC | EC |
//! |------|----|----|-----|
//! | test_BC_2_16_001_write_registry_get_crowdstrike_contain | BC-2.16.001 | AC-1 | — |
//! | test_BC_2_16_001_write_table_descriptor_write_only_flag | BC-2.16.001 | AC-4 | — |
//! | test_BC_2_16_001_all_four_sensors_ten_write_verbs | BC-2.16.001 | AC-5 | — |
//! | test_BC_2_16_001_verbs_for_crowdstrike_returns_four | BC-2.16.001 | AC-5 | — |
//! | test_BC_2_16_009_reserved_keyword_where_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_reserved_keyword_sort_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_reserved_keyword_limit_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_reserved_keyword_join_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_reserved_keyword_enrich_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_reserved_keyword_head_returns_e_spec_011 | BC-2.16.009 | AC-2 | EC-001 |
//! | test_BC_2_16_009_batch_limit_zero_irreversible_emits_warning | BC-2.16.009 | AC-3 | EC-003 |
//! | test_BC_2_16_009_batch_limit_zero_irreversible_spec_loads | BC-2.16.009 | AC-3 | EC-003 |
//! | test_BC_2_16_009_empty_steps_rejected | BC-2.16.009 | — | EC-004 |
//! | test_BC_2_16_009_record_id_field_uppercase_rejected | BC-2.16.009 | — | EC-005 |
//! | test_BC_2_16_009_record_id_field_special_chars_rejected | BC-2.16.009 | — | EC-005 |
//! | test_BC_2_16_009_record_id_field_valid_lowercase | BC-2.16.009 | — | EC-005 |
//! | test_BC_2_16_009_cross_sensor_verb_uniqueness_collision | BC-2.16.009 | — | EC-002 |
//! | test_BC_2_16_009_all_errors_collected_no_fail_fast | BC-2.16.009 | — | — |
//! | test_BC_2_16_009_valid_spec_no_errors_no_warnings | BC-2.16.009 | — | — |
//! | test_BC_2_16_001_crowdstrike_risk_tier_irreversible | BC-2.16.001 | AC-1 | — |
//! | test_BC_2_16_001_crowdstrike_batch_limit_ten | BC-2.16.001 | AC-1 | — |
//! | test_BC_2_16_001_write_table_descriptor_sql_table_name | BC-2.16.001 | AC-4 | — |
//! | test_interpolation_record_ids_resolved_in_body_template | S-1.13 Task 6 | — | — |
//! | test_interpolation_params_key_resolved | S-1.13 Task 6 | — | — |
//! | test_interpolation_params_key_default_used_when_missing | S-1.13 Task 6 | — | — |
//! | test_interpolation_url_context_percent_encodes | S-1.13 Task 6 | — | — |
//! | test_BC_2_16_009_risk_tier_invalid_string_parse_error | BC-2.16.009 | — | — |
//! | test_BC_2_16_001_registry_is_empty_before_register | BC-2.16.001 | — | — |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_map_or,
    unused_imports,
    unused_variables,
    dead_code,
    unused_mut
)]
use prism_core::{RiskTier, SpecErrorCode};
use prism_spec_engine::{
    interpolation::{InterpolationContext, Interpolator},
    write_endpoint::{
        check_reserved_keyword, validate_write_endpoints, BatchMode, WriteEndpointRegistry,
        WriteEndpointSpec, WriteStep, WriteTableDescriptor,
    },
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid WriteEndpointSpec for testing.
fn minimal_write_endpoint(pipe_verb: &str, risk_tier: RiskTier) -> WriteEndpointSpec {
    WriteEndpointSpec {
        pipe_verb: pipe_verb.to_string(),
        sql_table: format!("sensor_{}_table", pipe_verb),
        risk_tier,
        capability_path: "sensor.resource.write".to_string(),
        batch_limit: 10,
        batch_mode: BatchMode::Serial,
        record_id_field: "resource_id".to_string(),
        steps: vec![WriteStep {
            method: "POST".to_string(),
            url: "/api/v1/action".to_string(),
            body_template: Some(r#"{"ids": ${record_ids}}"#.to_string()),
            response_path: None,
        }],
    }
}

/// Build the CrowdStrike contain endpoint from spec (canonical test vector — AC-1).
fn crowdstrike_contain_endpoint() -> WriteEndpointSpec {
    WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "crowdstrike_contained_hosts".to_string(),
        risk_tier: RiskTier::Irreversible,
        capability_path: "crowdstrike.hosts.write".to_string(),
        batch_limit: 10,
        batch_mode: BatchMode::Serial,
        record_id_field: "device_id".to_string(),
        steps: vec![WriteStep {
            method: "POST".to_string(),
            url: "/devices/entities/host-actions/v2".to_string(),
            body_template: Some(r#"{"action_name": "contain", "ids": ${record_ids}}"#.to_string()),
            response_path: None,
        }],
    }
}

fn crowdstrike_endpoints() -> Vec<WriteEndpointSpec> {
    vec![
        crowdstrike_contain_endpoint(),
        WriteEndpointSpec {
            pipe_verb: "uncontain".to_string(),
            sql_table: "crowdstrike_uncontained_hosts".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "crowdstrike.hosts.write".to_string(),
            batch_limit: 10,
            batch_mode: BatchMode::Serial,
            record_id_field: "device_id".to_string(),
            steps: vec![WriteStep {
                method: "POST".to_string(),
                url: "/devices/entities/host-actions/v2".to_string(),
                body_template: Some(
                    r#"{"action_name": "lift_containment", "ids": ${record_ids}}"#.to_string(),
                ),
                response_path: None,
            }],
        },
        WriteEndpointSpec {
            pipe_verb: "update_status".to_string(),
            sql_table: "crowdstrike_status_updates".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "crowdstrike.detections.write".to_string(),
            batch_limit: 100,
            batch_mode: BatchMode::Serial,
            record_id_field: "detection_id".to_string(),
            steps: vec![WriteStep {
                method: "PATCH".to_string(),
                url: "/detects/entities/detect/v2".to_string(),
                body_template: Some(
                    r#"{"ids": ${record_ids}, "status": "${params.status}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
        WriteEndpointSpec {
            pipe_verb: "assign".to_string(),
            sql_table: "crowdstrike_assignments".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "crowdstrike.detections.write".to_string(),
            batch_limit: 100,
            batch_mode: BatchMode::Serial,
            record_id_field: "detection_id".to_string(),
            steps: vec![WriteStep {
                method: "PATCH".to_string(),
                url: "/detects/entities/detect/v2".to_string(),
                body_template: Some(
                    r#"{"ids": ${record_ids}, "assigned_to_uid": "${params.user_id}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
    ]
}

fn cyberint_endpoints() -> Vec<WriteEndpointSpec> {
    vec![
        WriteEndpointSpec {
            pipe_verb: "acknowledge".to_string(),
            sql_table: "cyberint_acknowledged_alerts".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "cyberint.alerts.write".to_string(),
            batch_limit: 50,
            batch_mode: BatchMode::Serial,
            record_id_field: "alert_id".to_string(),
            steps: vec![WriteStep {
                method: "POST".to_string(),
                url: "/v1/alerts/acknowledge".to_string(),
                body_template: Some(r#"{"alert_ids": ${record_ids}}"#.to_string()),
                response_path: None,
            }],
        },
        WriteEndpointSpec {
            pipe_verb: "close_alert".to_string(),
            sql_table: "cyberint_closed_alerts".to_string(),
            risk_tier: RiskTier::Irreversible,
            capability_path: "cyberint.alerts.write".to_string(),
            batch_limit: 20,
            batch_mode: BatchMode::Serial,
            record_id_field: "alert_id".to_string(),
            steps: vec![WriteStep {
                method: "POST".to_string(),
                url: "/v1/alerts/close".to_string(),
                body_template: Some(
                    r#"{"alert_ids": ${record_ids}, "reason": "${params.reason|default:resolved}"}"#
                        .to_string(),
                ),
                response_path: None,
            }],
        },
    ]
}

fn claroty_endpoints() -> Vec<WriteEndpointSpec> {
    vec![
        WriteEndpointSpec {
            pipe_verb: "tag".to_string(),
            sql_table: "claroty_tagged_assets".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "claroty.assets.write".to_string(),
            batch_limit: 50,
            batch_mode: BatchMode::Serial,
            record_id_field: "asset_id".to_string(),
            steps: vec![WriteStep {
                method: "POST".to_string(),
                url: "/v1/assets/tag".to_string(),
                body_template: Some(
                    r#"{"asset_ids": ${record_ids}, "tag": "${params.tag}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
        WriteEndpointSpec {
            pipe_verb: "remove_tag".to_string(),
            sql_table: "claroty_untagged_assets".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "claroty.assets.write".to_string(),
            batch_limit: 50,
            batch_mode: BatchMode::Serial,
            record_id_field: "asset_id".to_string(),
            steps: vec![WriteStep {
                method: "DELETE".to_string(),
                url: "/v1/assets/tag".to_string(),
                body_template: Some(
                    r#"{"asset_ids": ${record_ids}, "tag": "${params.tag}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
    ]
}

fn armis_endpoints() -> Vec<WriteEndpointSpec> {
    vec![
        WriteEndpointSpec {
            // Armis uses "label" (not "tag") to satisfy EC-002 global pipe_verb uniqueness.
            // "tag" is already claimed by claroty; "label" is semantically appropriate
            // for Armis device categorization (Armis API calls these "labels" internally).
            pipe_verb: "label".to_string(),
            sql_table: "armis_labeled_devices".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "armis.devices.write".to_string(),
            batch_limit: 50,
            batch_mode: BatchMode::Serial,
            record_id_field: "device_id".to_string(),
            steps: vec![WriteStep {
                method: "POST".to_string(),
                url: "/api/v1/devices/labels".to_string(),
                body_template: Some(
                    r#"{"device_ids": ${record_ids}, "label": "${params.label}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
        WriteEndpointSpec {
            // "remove_label" pairs with "label" — unique, semantically correct for Armis.
            pipe_verb: "remove_label".to_string(),
            sql_table: "armis_unlabeled_devices".to_string(),
            risk_tier: RiskTier::Reversible,
            capability_path: "armis.devices.write".to_string(),
            batch_limit: 50,
            batch_mode: BatchMode::Serial,
            record_id_field: "device_id".to_string(),
            steps: vec![WriteStep {
                method: "DELETE".to_string(),
                url: "/api/v1/devices/labels".to_string(),
                body_template: Some(
                    r#"{"device_ids": ${record_ids}, "label": "${params.label}"}"#.to_string(),
                ),
                response_path: None,
            }],
        },
    ]
}

// ---------------------------------------------------------------------------
// BC-2.16.001 — WriteEndpointRegistry get / describe
// ---------------------------------------------------------------------------

/// AC-1: registry.get("crowdstrike", "contain") returns the spec with
/// risk_tier=Irreversible and batch_limit=10.
///
/// BC-2.16.001 postcondition: each TableSpec within a SensorSpec is registered.
#[test]
fn test_BC_2_16_001_write_registry_get_crowdstrike_contain() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("valid endpoints should register without error");

    let spec = registry
        .get("crowdstrike", "contain")
        .expect("contain endpoint must be present after registration");

    assert_eq!(spec.risk_tier, RiskTier::Irreversible);
    assert_eq!(spec.batch_limit, 10);
}

/// AC-1: risk_tier is Irreversible for the contain endpoint.
///
/// BC-2.16.001 postcondition: WriteEndpointSpec fields faithfully reflect the spec.
#[test]
fn test_BC_2_16_001_crowdstrike_risk_tier_irreversible() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("registration succeeds");

    let spec = registry.get("crowdstrike", "contain").unwrap();
    assert_eq!(spec.risk_tier, RiskTier::Irreversible);
}

/// AC-1: batch_limit is 10 for the contain endpoint.
///
/// BC-2.16.001 postcondition.
#[test]
fn test_BC_2_16_001_crowdstrike_batch_limit_ten() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("registration succeeds");

    let spec = registry.get("crowdstrike", "contain").unwrap();
    assert_eq!(spec.batch_limit, 10);
}

/// AC-4: WriteEndpointRegistry contains a WriteTableDescriptor with
/// sql_table="crowdstrike_contained_hosts" and write_only=true.
///
/// BC-2.16.001 postcondition: table registration exports descriptors.
#[test]
fn test_BC_2_16_001_write_table_descriptor_write_only_flag() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("registration succeeds");

    let descriptors = registry.table_descriptors();
    let descriptor = descriptors
        .iter()
        .find(|d| d.sql_table == "crowdstrike_contained_hosts")
        .expect("crowdstrike_contained_hosts descriptor must be present");

    assert!(
        descriptor.write_only,
        "write endpoint tables must have write_only=true"
    );
}

/// AC-4: WriteTableDescriptor sql_table matches the spec's sql_table field.
///
/// BC-2.16.001 postcondition.
#[test]
fn test_BC_2_16_001_write_table_descriptor_sql_table_name() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("registration succeeds");

    let descriptors = registry.table_descriptors();
    let names: Vec<&str> = descriptors.iter().map(|d| d.sql_table.as_str()).collect();
    assert!(
        names.contains(&"crowdstrike_contained_hosts"),
        "expected crowdstrike_contained_hosts in descriptors, got: {:?}",
        names
    );
}

/// AC-5: At least 10 write verbs are available after loading all four sensors.
///
/// BC-2.16.001 postcondition: 4 CrowdStrike + 2 Cyberint + 2 Claroty + 2 Armis = 10.
#[test]
fn test_BC_2_16_001_all_four_sensors_ten_write_verbs() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("crowdstrike registration succeeds");
    registry
        .register("cyberint", cyberint_endpoints())
        .expect("cyberint registration succeeds");
    registry
        .register("claroty", claroty_endpoints())
        .expect("claroty registration succeeds");
    registry
        .register("armis", armis_endpoints())
        .expect("armis registration succeeds");

    let total = registry.len();
    assert!(
        total >= 10,
        "expected at least 10 write verbs across 4 sensors, got {}",
        total
    );
}

/// AC-5: verbs_for_sensor("crowdstrike") returns the expected 4 verbs.
///
/// BC-2.16.001 postcondition.
#[test]
fn test_BC_2_16_001_verbs_for_crowdstrike_returns_four() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("registration succeeds");

    let verbs = registry.verbs_for_sensor("crowdstrike");
    assert_eq!(
        verbs.len(),
        4,
        "expected 4 verbs for crowdstrike, got {:?}",
        verbs
    );

    let expected_verbs = ["contain", "uncontain", "update_status", "assign"];
    for verb in &expected_verbs {
        assert!(
            verbs.contains(verb),
            "expected verb '{}' in crowdstrike verbs: {:?}",
            verb,
            verbs
        );
    }
}

/// Registry is empty before any registration.
///
/// BC-2.16.001 precondition.
#[test]
fn test_BC_2_16_001_registry_is_empty_before_register() {
    let registry = WriteEndpointRegistry::new();
    // is_empty() does not call unimplemented! — it checks the internal HashMap directly.
    // len() does call unimplemented!, so we test via table_descriptors().
    let descriptors = registry.table_descriptors();
    assert!(
        descriptors.is_empty(),
        "fresh registry must have no descriptors"
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — Reserved keyword rejection (E-SPEC-011)
// ---------------------------------------------------------------------------

/// AC-2 / EC-001: pipe_verb="where" must return E-SPEC-011.
///
/// BC-2.16.009 validation rule 1: pipe_verb must not collide with RESERVED_KEYWORDS.
#[test]
fn test_BC_2_16_009_reserved_keyword_where_returns_e_spec_011() {
    let error = check_reserved_keyword(
        "where",
        "test_sensor",
        Some("write_endpoints.where.pipe_verb"),
    )
    .expect("'where' is a reserved keyword; error must be returned");

    assert_eq!(
        error.code,
        SpecErrorCode::ESpec011,
        "reserved keyword collision must produce E-SPEC-011, got {:?}",
        error.code
    );
    assert!(
        error.message.contains("where"),
        "error message must name the conflicting keyword, got: {}",
        error.message
    );
}

/// EC-001: pipe_verb="sort" must return E-SPEC-011.
#[test]
fn test_BC_2_16_009_reserved_keyword_sort_returns_e_spec_011() {
    let error = check_reserved_keyword("sort", "test_sensor", None)
        .expect("'sort' is reserved; error required");
    assert_eq!(error.code, SpecErrorCode::ESpec011);
}

/// EC-001: pipe_verb="limit" must return E-SPEC-011.
#[test]
fn test_BC_2_16_009_reserved_keyword_limit_returns_e_spec_011() {
    let error = check_reserved_keyword("limit", "test_sensor", None)
        .expect("'limit' is reserved; error required");
    assert_eq!(error.code, SpecErrorCode::ESpec011);
}

/// EC-001: pipe_verb="join" must return E-SPEC-011.
#[test]
fn test_BC_2_16_009_reserved_keyword_join_returns_e_spec_011() {
    let error = check_reserved_keyword("join", "test_sensor", None)
        .expect("'join' is reserved; error required");
    assert_eq!(error.code, SpecErrorCode::ESpec011);
}

/// EC-001: pipe_verb="enrich" must return E-SPEC-011.
#[test]
fn test_BC_2_16_009_reserved_keyword_enrich_returns_e_spec_011() {
    let error = check_reserved_keyword("enrich", "test_sensor", None)
        .expect("'enrich' is reserved; error required");
    assert_eq!(error.code, SpecErrorCode::ESpec011);
}

/// EC-001: pipe_verb="head" must return E-SPEC-011.
#[test]
fn test_BC_2_16_009_reserved_keyword_head_returns_e_spec_011() {
    let error = check_reserved_keyword("head", "test_sensor", None)
        .expect("'head' is reserved; error required");
    assert_eq!(error.code, SpecErrorCode::ESpec011);
}

/// Non-reserved verb must NOT return an error from check_reserved_keyword.
///
/// BC-2.16.009: valid verbs pass through.
#[test]
fn test_BC_2_16_009_non_reserved_verb_passes() {
    let result = check_reserved_keyword("contain", "crowdstrike", None);
    assert!(
        result.is_none(),
        "'contain' is not reserved; expected None, got {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — Batch limit + irreversible warning (AC-3, EC-003)
// ---------------------------------------------------------------------------

/// AC-3: batch_limit=0 + risk_tier=Irreversible emits a structured warning.
///
/// BC-2.16.009 validation rule: warning, not error.
#[test]
fn test_BC_2_16_009_batch_limit_zero_irreversible_emits_warning() {
    let endpoint = WriteEndpointSpec {
        batch_limit: 0,
        risk_tier: RiskTier::Irreversible,
        ..minimal_write_endpoint("quarantine", RiskTier::Irreversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    let warnings = result.expect("batch_limit=0+irreversible must not fail validation");
    assert!(
        !warnings.is_empty(),
        "expected at least one warning for batch_limit=0+irreversible, got none"
    );
    let has_relevant_warning = warnings.iter().any(|w| {
        w.message.contains("batch_limit")
            || w.message.contains("irreversible")
            || w.message.contains("unlimited")
    });
    assert!(
        has_relevant_warning,
        "warning must mention batch_limit or irreversible, got: {:?}",
        warnings
    );
}

/// AC-3: batch_limit=0 + risk_tier=Irreversible — spec LOADS (returns Ok, not Err).
///
/// BC-2.16.009 invariant: warnings never block loading.
#[test]
fn test_BC_2_16_009_batch_limit_zero_irreversible_spec_loads() {
    let endpoint = WriteEndpointSpec {
        batch_limit: 0,
        risk_tier: RiskTier::Irreversible,
        ..minimal_write_endpoint("quarantine", RiskTier::Irreversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    assert!(
        result.is_ok(),
        "batch_limit=0+irreversible is a warning, not an error — spec must load, got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — Empty steps (EC-004)
// ---------------------------------------------------------------------------

/// EC-004: empty steps array must be rejected.
///
/// BC-2.16.009 validation rule: steps must be non-empty.
#[test]
fn test_BC_2_16_009_empty_steps_rejected() {
    let endpoint = WriteEndpointSpec {
        steps: vec![],
        ..minimal_write_endpoint("quarantine", RiskTier::Reversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    assert!(
        result.is_err(),
        "empty steps array must produce a validation error"
    );
    let errors = result.unwrap_err();
    assert!(
        !errors.is_empty(),
        "expected at least one error for empty steps"
    );
    // Error must identify the location
    let has_steps_error = errors.iter().any(|e| {
        e.message.contains("step") || e.toml_path.as_deref().map_or(false, |p| p.contains("step"))
    });
    assert!(
        has_steps_error,
        "error must reference steps, got: {:?}",
        errors
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — record_id_field validation (EC-005)
// ---------------------------------------------------------------------------

/// EC-005: record_id_field with uppercase characters must be rejected.
///
/// BC-2.16.009: record_id_field must match [a-z0-9_]+.
#[test]
fn test_BC_2_16_009_record_id_field_uppercase_rejected() {
    let endpoint = WriteEndpointSpec {
        record_id_field: "DeviceID".to_string(),
        ..minimal_write_endpoint("quarantine", RiskTier::Reversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    assert!(
        result.is_err(),
        "uppercase record_id_field must be rejected"
    );
    let errors = result.unwrap_err();
    let has_id_error = errors.iter().any(|e| {
        e.message.contains("record_id_field")
            || e.toml_path
                .as_deref()
                .map_or(false, |p| p.contains("record_id"))
    });
    assert!(
        has_id_error,
        "error must reference record_id_field, got: {:?}",
        errors
    );
}

/// EC-005: record_id_field with special characters (dash) must be rejected.
///
/// BC-2.16.009: record_id_field must match [a-z0-9_]+ (underscore only, no dash).
#[test]
fn test_BC_2_16_009_record_id_field_special_chars_rejected() {
    let endpoint = WriteEndpointSpec {
        record_id_field: "device-id".to_string(),
        ..minimal_write_endpoint("quarantine", RiskTier::Reversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    assert!(
        result.is_err(),
        "record_id_field with dash must be rejected (only [a-z0-9_] allowed)"
    );
}

/// EC-005: record_id_field matching [a-z0-9_]+ must be accepted.
///
/// BC-2.16.009: valid record_id_field passes validation.
#[test]
fn test_BC_2_16_009_record_id_field_valid_lowercase() {
    let endpoint = WriteEndpointSpec {
        record_id_field: "device_id".to_string(),
        ..minimal_write_endpoint("quarantine", RiskTier::Reversible)
    };

    let result = validate_write_endpoints("test_sensor", &[endpoint]);
    assert!(
        result.is_ok(),
        "valid record_id_field 'device_id' must pass validation, got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — Cross-sensor verb uniqueness (EC-002)
// ---------------------------------------------------------------------------

/// EC-002: two sensors registering the same pipe_verb must fail at registry level.
///
/// BC-2.16.009: pipe_verb must be unique across all sensors.
/// Using "contain" — registered by crowdstrike; second sensor claiming it must fail.
#[test]
fn test_BC_2_16_009_cross_sensor_verb_uniqueness_collision() {
    let mut registry = WriteEndpointRegistry::new();
    registry
        .register("crowdstrike", crowdstrike_endpoints())
        .expect("crowdstrike registration succeeds");

    // A second sensor claiming "contain" must fail.
    let conflicting_endpoint = WriteEndpointSpec {
        pipe_verb: "contain".to_string(),
        sql_table: "other_sensor_contain".to_string(),
        risk_tier: RiskTier::Irreversible,
        capability_path: "other_sensor.hosts.write".to_string(),
        batch_limit: 5,
        batch_mode: BatchMode::Serial,
        record_id_field: "host_id".to_string(),
        steps: vec![WriteStep {
            method: "POST".to_string(),
            url: "/api/contain".to_string(),
            body_template: Some(r#"{"ids": ${record_ids}}"#.to_string()),
            response_path: None,
        }],
    };

    let result = registry.register("other_sensor", vec![conflicting_endpoint]);
    assert!(
        result.is_err(),
        "registering a second sensor with verb 'contain' must fail (EC-002 global uniqueness)"
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — All-errors-collected (VP-059 invariant)
// ---------------------------------------------------------------------------

/// BC-2.16.009 invariant (VP-059): all errors collected in a single pass, no fail-fast.
///
/// Two invalid endpoints in the same slice must both produce errors.
#[test]
fn test_BC_2_16_009_all_errors_collected_no_fail_fast() {
    let bad_endpoint_1 = WriteEndpointSpec {
        pipe_verb: "where".to_string(), // reserved keyword — E-SPEC-011
        ..minimal_write_endpoint("where", RiskTier::Reversible)
    };
    let bad_endpoint_2 = WriteEndpointSpec {
        record_id_field: "BadFieldName".to_string(), // uppercase — E-SPEC-001
        steps: vec![],                               // empty steps — E-SPEC-001
        ..minimal_write_endpoint("good_verb", RiskTier::Reversible)
    };

    let result = validate_write_endpoints("test_sensor", &[bad_endpoint_1, bad_endpoint_2]);
    assert!(
        result.is_err(),
        "multiple invalid endpoints must fail validation"
    );

    let errors = result.unwrap_err();
    assert!(
        errors.len() >= 2,
        "all errors must be collected (no fail-fast); expected >= 2, got {} errors: {:?}",
        errors.len(),
        errors
    );
}

/// BC-2.16.009: a fully valid spec produces no errors and no warnings.
#[test]
fn test_BC_2_16_009_valid_spec_no_errors_no_warnings() {
    let endpoints = vec![minimal_write_endpoint("quarantine", RiskTier::Reversible)];
    let result = validate_write_endpoints("test_sensor", &endpoints);

    let warnings = result.expect("valid spec must produce no errors");
    assert!(
        warnings.is_empty(),
        "valid spec must produce no warnings, got: {:?}",
        warnings
    );
}

// ---------------------------------------------------------------------------
// BC-2.16.009 — Risk tier validation
// ---------------------------------------------------------------------------

/// Invalid risk_tier string in TOML must produce a parse error.
///
/// BC-2.16.009 validation rule 3: risk_tier must be "reversible" or "irreversible".
/// This is enforced by the serde enum deserialization of RiskTierSpec.
#[test]
fn test_BC_2_16_009_risk_tier_invalid_string_parse_error() {
    // Attempt to deserialize a WriteEndpointSpec with an invalid risk_tier.
    let toml_snippet = r#"
        pipe_verb = "quarantine"
        sql_table = "sensor_quarantine_table"
        risk_tier = "read"
        capability_path = "sensor.resource.write"
        batch_limit = 10
        batch_mode = "serial"
        record_id_field = "resource_id"

        [[steps]]
        method = "POST"
        url = "/api/v1/action"
    "#;

    let result: Result<WriteEndpointSpec, _> = toml::from_str(toml_snippet);
    assert!(
        result.is_err(),
        "risk_tier='read' must fail deserialization; got Ok({:?})",
        result.ok()
    );
}

// ---------------------------------------------------------------------------
// S-1.13 Task 6 — Write-side interpolation
// ---------------------------------------------------------------------------

/// ${record_ids} in a JSON body template must be resolved to a JSON array.
///
/// S-1.13 Task 6: interpolate_record_ids with JsonBody context.
#[test]
fn test_interpolation_record_ids_resolved_in_body_template() {
    let template = r#"{"action_name": "contain", "ids": ${record_ids}}"#;
    let record_ids = vec![
        serde_json::Value::String("device-001".to_string()),
        serde_json::Value::String("device-002".to_string()),
    ];

    let result = Interpolator::interpolate_record_ids(
        template,
        &InterpolationContext::JsonBody,
        &record_ids,
    )
    .expect("interpolation must succeed");

    assert!(
        result.contains("device-001"),
        "interpolated template must contain device-001, got: {}",
        result
    );
    assert!(
        result.contains("device-002"),
        "interpolated template must contain device-002, got: {}",
        result
    );
    // The ${record_ids} placeholder must be replaced.
    assert!(
        !result.contains("${record_ids}"),
        "interpolated template must not contain the placeholder, got: {}",
        result
    );
}

/// ${params.KEY} in a write template must be resolved to the provided value.
///
/// S-1.13 Task 6: interpolate_write_params.
#[test]
fn test_interpolation_params_key_resolved() {
    let template = r#"{"ids": ${record_ids}, "status": "${params.status}"}"#;
    let mut params = std::collections::HashMap::new();
    params.insert("status".to_string(), "closed".to_string());

    let result =
        Interpolator::interpolate_write_params(template, &InterpolationContext::JsonBody, &params)
            .expect("interpolation must succeed");

    assert!(
        result.contains("closed"),
        "interpolated template must contain param value 'closed', got: {}",
        result
    );
    assert!(
        !result.contains("${params.status}"),
        "interpolated template must not contain the placeholder, got: {}",
        result
    );
}

/// ${params.KEY|default:VALUE} uses the default when the key is not in params.
///
/// S-1.13 Task 6: default value resolution.
#[test]
fn test_interpolation_params_key_default_used_when_missing() {
    let template = r#"{"reason": "${params.reason|default:resolved}"}"#;
    let params = std::collections::HashMap::new(); // empty — no "reason" key

    let result =
        Interpolator::interpolate_write_params(template, &InterpolationContext::JsonBody, &params)
            .expect("interpolation with default must succeed");

    assert!(
        result.contains("resolved"),
        "default value 'resolved' must be used when param is missing, got: {}",
        result
    );
    assert!(
        !result.contains("${params.reason"),
        "placeholder must not remain in result, got: {}",
        result
    );
}

/// URL context interpolation must percent-encode record ids.
///
/// S-1.13 Task 6: URL context applies percent-encoding.
#[test]
fn test_interpolation_url_context_percent_encodes() {
    let template = "/api/v1/action?ids=${record_ids}";
    let record_ids = vec![
        // A value with a space — must be percent-encoded in URL context.
        serde_json::Value::String("device id with spaces".to_string()),
    ];

    let result =
        Interpolator::interpolate_record_ids(template, &InterpolationContext::UrlPath, &record_ids)
            .expect("url interpolation must succeed");

    assert!(
        !result.contains(' '),
        "URL context must percent-encode spaces; got: {}",
        result
    );
    assert!(
        result.contains("%20") || result.contains("+"),
        "URL context must encode spaces as %20 or +; got: {}",
        result
    );
}
