//! Harness migration stubs — S-3.4.02 Red Gate
//!
//! Migrates all prism-dtu-armis tests to use `prism-dtu-harness::Harness`
//! instead of the single-tenant `ArmisClone::start()` pattern, and adds
//! multi-org logical isolation + network cross-credential 401 tests.
//!
//! ## Source tests (kept in place; migrated here in implementer phase)
//!
//! | Source file | Original test functions |
//! |-------------|------------------------|
//! | `ac_1_aql_capture_and_device_list.rs` | ac_1_get_devices_with_aql_returns_200_and_logs_aql, ac_1_post_devices_with_aql_body_returns_200_and_logs_aql, ec_001_aql_special_characters_stored_verbatim, ec_004_pagination_beyond_last_page_returns_empty_array, ac_1_devices_response_contains_pagination_fields |
//! | `ac_2_timestamp_fallback_fixture.rs` | ac_2_device_d001_has_null_last_seen_and_non_null_first_seen, ac_2_device_d002_has_both_timestamps_populated, ac_2_device_risk_endpoint_returns_risk_score, ec_002_risk_endpoint_returns_404_for_unknown_device |
//! | `ac_3_stateful_tag_add.rs` | ac_3_post_tag_returns_201_with_device_id_and_tag_key, ac_3_added_tag_appears_in_subsequent_device_query, ac_3_tag_endpoint_requires_bearer_auth_returns_403 |
//! | `ac_4_tag_delete.rs` | ac_4_delete_tag_returns_200_removed, ac_4_device_does_not_have_tag_after_delete, ec_003_delete_nonexistent_tag_returns_404, ac_4_delete_tag_endpoint_requires_bearer_auth |
//! | `ac_5_missing_bearer_403.rs` | ac_5_get_devices_without_auth_returns_403, ac_5_get_alerts_without_auth_returns_403, ac_5_get_device_activity_without_auth_returns_403, ac_5_get_device_risk_without_auth_returns_403, ac_5_empty_bearer_value_returns_403, ac_5_wrong_scheme_returns_403, ac_5_dtu_internal_endpoints_do_not_require_auth |
//! | `ac_6_rate_limit_429.rs` | ac_6_rate_limit_429_after_threshold_exceeded_via_configure, ac_6_rate_limit_allows_requests_before_threshold, ec_006_malformed_response_mode_returns_non_parseable_body |
//! | `fidelity_validator.rs` | fidelity_validator_passes |
//! | `reset_state_invariants.rs` | ac_story_7_reset_clears_tag_store_and_aql_log, ac_story_7_reset_does_not_remove_fixture_data, activity_endpoint_returns_200_with_activities_array, alerts_endpoint_returns_200_with_alerts_array, alerts_pagination_beyond_last_returns_empty_array, reset_clears_failure_mode_to_none |
//! | `td_wv0_04_configure_deny_unknown.rs` | configure_known_field_returns_200, configure_unknown_field_returns_400 |
//! | `td_wv0_07_configure_requires_admin_token.rs` | configure_without_token_returns_401, configure_with_wrong_token_returns_401, configure_with_correct_token_returns_200 |
//!
//! ## New tests (BC-3.5.001 + BC-3.5.002)
//!
//! - `test_BC_3_5_001_ac_multi_org_logical_isolation` (AC-003): 2-org Armis harness; pairwise-disjoint device sets.
//! - `test_BC_3_5_002_ac_network_cross_creds_401` (AC-004): Network isolation; cross-org credential mismatch → HTTP 401.
//!
//! # Red Gate
//!
//! ALL tests in this file MUST fail (todo!() panic) before implementation.
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern (Factory TDD spec).
#![cfg(feature = "dtu")]
#![allow(clippy::expect_used, non_snake_case, dead_code, unused_imports)]

use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// Helper: build a single-tenant Armis harness (logical isolation).
//
// Used by every migrated single-org test. The implementer replaces
// `ArmisClone::start()` with this helper and derives `base_url` from
// `harness.endpoints()` / `harness.endpoint_for("test-tenant", DtuType::Armis)`.
// ============================================================================

async fn build_single_armis_harness(slug: &str) -> prism_dtu_harness::Harness {
    todo!(
        "S-3.4.02: build single-tenant Armis harness via \
         Harness::builder().isolation(IsolationMode::Logical)\
         .with_customer_overrides({slug:?}, |spec| {{ spec.dtu_types = vec![DtuType::Armis]; }})\
         .build().await"
    )
}

// ============================================================================
// AC-1 migrations — aql_capture_and_device_list
// (source: ac_1_aql_capture_and_device_list.rs)
// ============================================================================

/// AC-1: GET /api/v1/devices?aql=in:type=switch with Bearer → HTTP 200, AQL captured.
///
/// (BC-3.5.001 postcondition 1; AC-001; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_get_devices_with_aql_returns_200_and_logs_aql() {
    todo!(
        "S-3.4.02: replace ArmisClone::start() with harness; derive base_url from \
         harness.endpoint_for('test-tenant', DtuType::Armis)"
    )
}

/// AC-1 / EC-005: POST /api/v1/devices with AQL body → HTTP 200, AQL captured.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_post_devices_with_aql_body_returns_200_and_logs_aql() {
    todo!("S-3.4.02: migrate ac_1 POST variant to harness")
}

/// EC-001: AQL strings with special characters stored verbatim.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec001_aql_special_characters_stored_verbatim() {
    todo!("S-3.4.02: migrate ec_001 special-char AQL to harness")
}

/// EC-004: Pagination page beyond last returns empty devices array with correct total.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec004_pagination_beyond_last_page_returns_empty_array() {
    todo!("S-3.4.02: migrate ec_004 pagination to harness")
}

/// AC-1: Response contains pagination fields (data.devices, data.total, data.page).
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac1_devices_response_contains_pagination_fields() {
    todo!("S-3.4.02: migrate ac_1 pagination fields check to harness")
}

// ============================================================================
// AC-2 migrations — timestamp_fallback_fixture
// (source: ac_2_timestamp_fallback_fixture.rs)
// ============================================================================

/// AC-2: Device d-001 has last_seen:null and non-null first_seen (timestamp fallback).
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_d001_has_null_last_seen_and_non_null_first_seen() {
    todo!("S-3.4.02: migrate ac_2 timestamp fallback fixture test to harness")
}

/// AC-2 contrast: Device d-002 has both timestamps populated.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_d002_has_both_timestamps_populated() {
    todo!("S-3.4.02: migrate ac_2 contrast timestamp test to harness")
}

/// AC-2: Risk endpoint returns risk_score field.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac2_device_risk_endpoint_returns_risk_score() {
    todo!("S-3.4.02: migrate ac_2 risk endpoint test to harness")
}

/// EC-002: Risk endpoint returns 404 for unknown device.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec002_risk_endpoint_returns_404_for_unknown_device() {
    todo!("S-3.4.02: migrate ec_002 unknown device risk to harness")
}

// ============================================================================
// AC-3 migrations — stateful_tag_add
// (source: ac_3_stateful_tag_add.rs)
// ============================================================================

/// AC-3: POST tag returns 201 with device_id and tag_key.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_post_tag_returns_201_with_device_id_and_tag_key() {
    todo!("S-3.4.02: migrate ac_3 tag add to harness")
}

/// AC-3: Added tag appears in subsequent device query.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_added_tag_appears_in_subsequent_device_query() {
    todo!("S-3.4.02: migrate ac_3 tag state persistence to harness")
}

/// AC-3: Tag endpoint requires Bearer auth → 403 without auth.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac3_tag_endpoint_requires_bearer_auth_returns_403() {
    todo!("S-3.4.02: migrate ac_3 auth check to harness")
}

// ============================================================================
// AC-4 migrations — tag_delete
// (source: ac_4_tag_delete.rs)
// ============================================================================

/// AC-4: DELETE tag returns 200 with status "removed".
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_delete_tag_returns_200_removed() {
    todo!("S-3.4.02: migrate ac_4 tag delete to harness")
}

/// AC-4: Device does not have tag after delete.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_device_does_not_have_tag_after_delete() {
    todo!("S-3.4.02: migrate ac_4 tag delete state check to harness")
}

/// EC-003: DELETE non-existent tag returns 404.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec003_delete_nonexistent_tag_returns_404() {
    todo!("S-3.4.02: migrate ec_003 missing tag delete to harness")
}

/// AC-4: DELETE tag endpoint requires Bearer auth → 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac4_delete_tag_endpoint_requires_bearer_auth() {
    todo!("S-3.4.02: migrate ac_4 delete auth check to harness")
}

// ============================================================================
// AC-5 migrations — missing_bearer_403
// (source: ac_5_missing_bearer_403.rs)
// ============================================================================

/// AC-5: GET /api/v1/devices without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_devices_without_auth_returns_403() {
    todo!("S-3.4.02: migrate ac_5 no-auth devices to harness")
}

/// AC-5: GET /api/v1/alerts without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_alerts_without_auth_returns_403() {
    todo!("S-3.4.02: migrate ac_5 no-auth alerts to harness")
}

/// AC-5: GET /api/v1/devices/d-001/activity without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_device_activity_without_auth_returns_403() {
    todo!("S-3.4.02: migrate ac_5 no-auth activity to harness")
}

/// AC-5: GET /api/v1/devices/d-001/risk without auth returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_get_device_risk_without_auth_returns_403() {
    todo!("S-3.4.02: migrate ac_5 no-auth risk to harness")
}

/// AC-5: "Authorization: Bearer " (empty token value) returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_empty_bearer_value_returns_403() {
    todo!("S-3.4.02: migrate ac_5 empty bearer edge case to harness")
}

/// AC-5: "Authorization: Basic ..." (wrong scheme) returns 403.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_wrong_scheme_returns_403() {
    todo!("S-3.4.02: migrate ac_5 wrong scheme to harness")
}

/// AC-5: DTU internal /dtu/* endpoints do not require auth.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac5_dtu_internal_endpoints_do_not_require_auth() {
    todo!("S-3.4.02: migrate ac_5 dtu endpoint no-auth check to harness")
}

// ============================================================================
// AC-6 migrations — rate_limit_429
// (source: ac_6_rate_limit_429.rs)
// ============================================================================

/// AC-6: Rate limit 429 after threshold exceeded via /dtu/configure.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac6_rate_limit_429_after_threshold_exceeded_via_configure() {
    todo!("S-3.4.02: migrate ac_6 rate limit 429 to harness")
}

/// AC-6: Rate limit allows requests before threshold.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac6_rate_limit_allows_requests_before_threshold() {
    todo!("S-3.4.02: migrate ac_6 pre-threshold success to harness")
}

/// EC-006: MalformedResponse mode returns non-parseable body.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ec006_malformed_response_mode_returns_non_parseable_body() {
    todo!("S-3.4.02: migrate ec_006 malformed response to harness")
}

// ============================================================================
// Fidelity validator migration
// (source: fidelity_validator.rs)
// AC-002: fidelity_validator reports checks_failed == 0 under harness.
// ============================================================================

/// AC-002: FidelityValidator reports checks_failed == 0 for all Armis endpoints
/// when the clone is hosted via HarnessBuilder.
///
/// The `base_url` must come from `harness.endpoint_for("test-tenant", DtuType::Armis)`,
/// not a hardcoded address.
///
/// (BC-3.5.001 precondition 3; AC-002; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_ac002_fidelity_validator_passes_under_harness() {
    todo!(
        "S-3.4.02: replace ArmisClone::start() + clone.base_url() with harness endpoint; \
         pass harness endpoint address as base_url to FidelityValidator::run"
    )
}

// ============================================================================
// reset_state_invariants migration
// (source: reset_state_invariants.rs)
// AC-005: reset passes — state mutate + reset → clean state + all invariants hold.
// ============================================================================

/// AC-005: After state-mutating operation followed by reset, clone state is clean.
///
/// Uses harness reset pattern: rebuild harness between state-sensitive assertions,
/// or call `harness.reset_customer("test-tenant")` if exposed.
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_clears_tag_store_and_aql_log() {
    todo!(
        "S-3.4.02: migrate reset test — use harness reset_customer or rebuild harness; \
         assert AQL log empty and tags cleared after reset"
    )
}

/// AC-005: Reset does not remove fixture data (devices, alerts, activity remain).
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_does_not_remove_fixture_data() {
    todo!("S-3.4.02: migrate reset fixture-preservation check to harness")
}

/// AC-005: Activity endpoint returns 200 with activities array after harness build.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_activity_endpoint_returns_200_with_activities_array() {
    todo!("S-3.4.02: migrate activity endpoint shape test to harness")
}

/// AC-005: Alerts endpoint returns 200 with alerts array after harness build.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_alerts_endpoint_returns_200_with_alerts_array() {
    todo!("S-3.4.02: migrate alerts endpoint shape test to harness")
}

/// AC-005: Alerts pagination beyond last page returns empty array.
///
/// (BC-3.5.001 postcondition 1; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_alerts_pagination_beyond_last_returns_empty_array() {
    todo!("S-3.4.02: migrate alerts pagination edge case to harness")
}

/// AC-005: POST /dtu/reset also resets failure mode to None.
///
/// (BC-3.5.001 precondition 5; AC-005; S-3.4.02 Task 5)
#[tokio::test]
async fn test_BC_3_5_001_ac005_reset_clears_failure_mode_to_none() {
    todo!("S-3.4.02: migrate failure-mode reset check to harness")
}

// ============================================================================
// TD migrations — configure endpoint
// (source: td_wv0_04_configure_deny_unknown.rs, td_wv0_07_configure_requires_admin_token.rs)
// ============================================================================

/// TD-WV0-04: /dtu/configure rejects unknown fields (deny_unknown_fields) — known field → 200.
///
/// (S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_known_field_returns_200() {
    todo!("S-3.4.02: migrate td_wv0_04 known field check to harness")
}

/// TD-WV0-04: /dtu/configure rejects unknown fields (deny_unknown_fields) — unknown field → 400.
///
/// (S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_unknown_field_returns_400() {
    todo!("S-3.4.02: migrate td_wv0_04 unknown field rejection to harness")
}

/// TD-WV0-07: /dtu/configure without X-Admin-Token → 401.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_without_token_returns_401() {
    todo!("S-3.4.02: migrate td_wv0_07 no-token check to harness")
}

/// TD-WV0-07: /dtu/configure with wrong X-Admin-Token → 401.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_with_wrong_token_returns_401() {
    todo!("S-3.4.02: migrate td_wv0_07 wrong-token check to harness")
}

/// TD-WV0-07: /dtu/configure with correct X-Admin-Token → 200.
///
/// (ADR-003 Amendment #5; S-3.4.02 Task 2)
#[tokio::test]
async fn test_BC_3_5_001_td_configure_with_correct_token_returns_200() {
    todo!("S-3.4.02: migrate td_wv0_07 correct-token check to harness")
}

// ============================================================================
// NEW: AC-003 — Multi-org logical isolation (BC-3.5.001 postcondition 2; TV-2)
//
// Build a 2-org logical harness ("test-tenant" + "other-tenant"), both Armis.
// Query each org; assert device sets are pairwise-disjoint.
// ============================================================================

/// AC-003: 2-org Armis logical harness returns pairwise-disjoint device sets.
///
/// "test-tenant" devices must not appear in "other-tenant" response and vice versa.
///
/// (BC-3.5.001 postcondition 2; TV-2; AC-003; S-3.4.02 Task 3; VP-122, VP-123)
#[tokio::test]
async fn test_BC_3_5_001_ac_multi_org_logical_isolation() {
    todo!(
        "S-3.4.02: build 2-org Armis harness via \
         Harness::builder().isolation(IsolationMode::Logical)\
         .with_customer_overrides('test-tenant', |s| {{ s.dtu_types = vec![DtuType::Armis]; }})\
         .with_customer_overrides('other-tenant', |s| {{ s.dtu_types = vec![DtuType::Armis]; }})\
         .build().await; \
         GET /api/v1/devices from each endpoint; assert pairwise-disjoint device_id sets"
    )
}

// ============================================================================
// NEW: AC-004 — Network isolation cross-credential 401 (BC-3.5.002 postcondition 2; TV-3)
//
// Build a 2-org Network harness. Use org-A's credentials against org-B's endpoint.
// Expect HTTP 401.
// ============================================================================

/// AC-004: 2-org Network harness; cross-org credential mismatch returns HTTP 401.
///
/// Org-A's Bearer token sent to org-B's Armis clone endpoint must be rejected
/// with HTTP 401 (network isolation enforced).
///
/// (BC-3.5.002 postcondition 2; TV-3; AC-004; S-3.4.02 Task 4; VP-125, VP-126, VP-127)
#[tokio::test]
async fn test_BC_3_5_002_ac_network_cross_creds_401() {
    todo!(
        "S-3.4.02: build 2-org Armis Network harness via \
         Harness::builder().isolation(IsolationMode::Network)\
         .with_customer_overrides('test-tenant', |s| {{ s.dtu_types = vec![DtuType::Armis]; }})\
         .with_customer_overrides('other-tenant', |s| {{ s.dtu_types = vec![DtuType::Armis]; }})\
         .build().await; \
         send test-tenant Bearer token to other-tenant endpoint; assert HTTP 401"
    )
}
