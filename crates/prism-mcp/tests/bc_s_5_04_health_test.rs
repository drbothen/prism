//! S-5.04 Red Gate tests — Sensor Health Subsystem (live probe) — REWRITTEN.
//!
//! All tests in this file are named `test_BC_S_5_04_*` or `test_BC_2_08_*` per the
//! VSDD test-writer naming convention (`test_BC_S_SS_NNN_xxx`).
//!
//! # Coverage matrix (external test file)
//!
//! | AC / BC clause | Test(s) |
//! |----------------|---------|
//! | BC-2.08.003 parse_retry_after — delta-seconds | test_BC_2_08_003_parse_retry_after_delta_seconds |
//! | BC-2.08.003 — whitespace trim | test_BC_2_08_003_parse_retry_after_delta_seconds_with_whitespace |
//! | BC-2.08.003 — u64 large value no overflow | test_BC_2_08_003_parse_retry_after_large_u64_does_not_overflow |
//! | BC-2.08.003 — IMF-fixdate future date | test_BC_2_08_003_parse_retry_after_imf_fixdate_form |
//! | BC-2.08.003 — past date clamps to zero | test_BC_2_08_003_parse_retry_after_past_date_clamps_to_zero |
//! | BC-2.08.003 — parse failure returns None | test_BC_2_08_003_parse_retry_after_invalid_returns_none |
//! | BC-2.08.003 — empty string returns None | test_BC_2_08_003_parse_retry_after_empty_returns_none |
//! | AC-3 (BC-2.08.003 postcondition 1) — 429+header→is_rate_limited | test_BC_2_08_003_rate_limit_state_from_http_429_with_header |
//! | EC-003 (BC-2.08.003) — no header → default 60s | test_BC_2_08_003_rate_limit_default_60s_when_no_header |
//! | BC-2.08.003 — is_cleared true after expiry | test_BC_2_08_003_rate_limit_is_cleared_after_expiry |
//! | BC-2.08.003 — is_cleared false before expiry | test_BC_2_08_003_rate_limit_not_cleared_before_expiry |
//! | BC-2.08.003 — not_limited state | test_BC_2_08_003_not_limited_state_is_cleared |
//! | BC-2.08.003 — rate_limited sets flag | test_BC_2_08_003_rate_limited_state_sets_flag |
//! | BC-2.08.004 — timestamp_key format | test_BC_2_08_004_timestamp_key_format |
//! | BC-2.08.004 — keys distinct per pair | test_BC_2_08_004_timestamp_key_is_distinct_per_pair |
//! | AC-4 (BC-2.08.004 postcondition 1) — write+read round-trip | test_BC_2_08_004_write_and_read_timestamp_roundtrip |
//! | AC-5 (BC-2.08.004) — None before write | test_BC_2_08_004_read_timestamp_returns_none_before_write |
//! | BC-2.08.004 — independent per key | test_BC_2_08_004_timestamps_are_independent_per_key |
//! | AC-6 (BC-2.08.007 postcondition 1) — 3up+1down→Partial | test_BC_2_08_007_aggregate_partial_when_some_up_some_down |
//! | BC-2.08.007 — all up→Healthy | test_BC_2_08_007_aggregate_healthy_when_all_up |
//! | BC-2.08.007 — all down→Unhealthy | test_BC_2_08_007_aggregate_unhealthy_when_all_down |
//! | BC-2.08.007 invariant — partial is success | test_BC_2_08_007_invariant_partial_is_not_error |
//! | BC-2.08.007 — auth_invalid counts as not healthy | test_BC_2_08_007_aggregate_auth_invalid_sensor_is_not_healthy |
//! | BC-2.08.007 — ALL auth-invalid → Unhealthy (F-S504-LP3-HIGH-001) | test_BC_2_08_007_aggregate_all_auth_invalid_is_unhealthy |
//! | BC-2.08.007 — ALL auth-invalid server response → "unhealthy" (F-S504-LP3-HIGH-001) | test_BC_2_08_007_all_auth_invalid_server_response_is_unhealthy |
//! | EC-08-001 (BC-2.08.001) — HTTP 503 connectivity probe → Degraded | test_BC_2_08_001_live_probe_503_connectivity_is_degraded |
//! | EC-08-009 (BC-2.08.002) — 503 check_one → reachable=true + no last_successful_query_at (HS-007) | test_BC_2_08_001_EC_08_001_503_probe_yields_reachable_true |
//! | HS-007 (EC-08-009) — all-503 fleet → OverallStatus::Partial (reachable=true, error set) | test_BC_2_08_001_EC_08_001_all_503_fleet_aggregate_partial |
//! | BC-2.08.007 — empty list → Unhealthy | test_BC_2_08_007_aggregate_empty_list_is_unhealthy |
//! | SensorHealthChecker::new GREEN | test_BC_S_5_04_sensor_health_checker_new_constructs_successfully |
//! | BC-2.08.004 checker — record+read round-trip | test_BC_2_08_004_checker_record_and_read_timestamp |
//! | BC-2.08.004 checker — None before record | test_BC_2_08_004_checker_last_successful_query_none_before_record |
//! | AC-5 (BC-2.08.004 postcondition 2) — survives checker reconstruction | test_BC_2_08_004_timestamp_survives_checker_reconstruction |
//! | AC-1 (BC-2.08.001) — 200→Up+latency | test_BC_2_08_001_live_probe_200_returns_up_with_latency |
//! | AC-1 consumer (F-S504-P1-MED-001) — check_one Up→latency_ms on SensorHealthResult | test_BC_2_08_001_check_one_surfaces_latency_ms_on_consumer_result_F_S504_P1_MED_001 |
//! | AC-2 (BC-2.08.002) — 401→auth_invalid not down | test_BC_2_08_002_live_probe_401_returns_auth_invalid_not_down |
//! | EC-004 (BC-2.08.002) — connection refused→Down+Unknown | test_BC_2_08_002_connection_refused_returns_down_auth_unknown |
//! | BC-2.08.002 — 403→auth_invalid | test_BC_2_08_002_live_probe_403_returns_auth_invalid |
//! | BC-2.08.006 postcondition 2 — sensors is keyed object | test_BC_2_08_006_sensors_health_resource_sensors_is_keyed_object |
//! | BC-2.08.006 S-5.04 — probe_level="live" in resource | test_BC_2_08_006_sensors_health_resource_live_probe_level |
//! | BC-2.08.006 S-5.04 — reachable is bool not null | test_BC_2_08_006_sensors_health_resource_reachable_is_boolean |
//! | RECONCILIATION-3 — cursor_count accessor | test_BC_2_08_005_query_engine_cursor_count_accessor |
//! | RECONCILIATION-3 — token_count accessor | test_BC_2_08_005_query_engine_token_count_accessor |
//! | FIX-001/v1.6 invariant — no direct reqwest::Client | test_BC_S_5_04_invariant_no_direct_reqwest_client_in_health_module |
//! | F-S504-P1-001/002 — 429→Up + rate_limit populated in SensorHealthResult | test_BC_2_08_003_live_probe_429_yields_up_and_populates_rate_limit |
//! | F-S504-P1-002 core — probe_connectivity RateLimited arm → ConnectivityStatus::Up | test_BC_2_08_001_live_probe_429_status_is_up_not_down |
//! | F-S504-P2-008 — sanitize_error truncates long body; strips control chars | test_BC_2_08_001_live_probe_error_body_is_sanitized |
//! | F-S504-P1-004 — with_token_store wired: token_count reflects live store | test_BC_2_08_005_query_engine_token_count_reflects_wired_store |
//! | F-S504-P1-005 — timestamp RocksDB read/write survives context reconstruction | test_BC_2_08_004_timestamp_survives_context_reconstruction_with_storage |
//! | F-S504-P2-RE-001 (1/3) — check_one + new_with_spec_map: probe_table=Some("detections") | test_BC_2_08_001_check_one_routes_to_probe_table_via_spec_map |
//! | F-S504-P2-RE-001 (2/3) — check_one + new_with_spec_map: probe_table=None, tables fallback | test_BC_2_08_001_check_one_falls_back_to_first_table_via_spec_map |
//! | F-S504-P2-RE-001 (3/3) — check_one key-miss: client_id not in spec_map → devices sentinel | test_BC_2_08_001_check_one_falls_back_to_devices_when_org_not_in_spec_map |
//! | EC-007/F-S504-P5-001 — aggregate: all-rate-limited → RateLimited (not Partial) | test_BC_2_08_007_EC_007_all_rate_limited_aggregate_yields_rate_limited |
//! | EC-007/F-S504-P5-001 — aggregate: mixed (some RL + some down) → Partial, not RateLimited | test_BC_2_08_007_EC_007_mixed_rate_limited_and_down_is_partial |
//! | F-S504-P5-002 — SensorHealthStructuredContent: overall_status + summary_counts + suggestion | test_BC_2_08_007_EC_007_response_shape_overall_status_summary_counts_suggestion |
//! | F-S504-LP1P3-MED-001 (1/2) — auth-invalid production-path suggestion (AC-12) | test_BC_2_08_007_EC_08_015_auth_invalid_production_path_suggestion |
//! | F-S504-LP1P3-MED-001 (2/2) — genuine-Down production-path suggestion (AC-12) | test_BC_2_08_007_EC_08_015_genuine_down_production_path_suggestion_distinct_from_5xx |
//! | BC-2.08.002 EC-08-005 — Down check_one sets reason="sensor_unreachable_cannot_verify" | test_BC_2_08_002_EC_08_005_down_check_one_sets_sensor_unreachable_cannot_verify_reason |
//!
//! # AC-7 (BC-2.08.005 live-probe path) tests
//! Tests requiring `PrismServer.health_checker` (a private field set only from
//! within `server.rs mod tests`) are located in `crates/prism-mcp/src/server.rs`
//! under `mod tests`, where private field access is available. See:
//! `test_BC_2_08_005_S504_live_probe_sets_probe_level_live` and siblings.
//!
//! # SID-1 compliance
//! Tests requiring a live DTU adapter are `#[ignore]`'d with explicit blocking-dependency
//! comments. Each has a companion in-process unit test driving the behavior via a mock
//! `SensorAdapter`.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_mcp::{
    context::PrismContext,
    health::{
        auth::{probe_auth, AuthStatus},
        connectivity::{probe_connectivity, probe_connectivity_with_routing, ConnectivityStatus},
        rate_limit::{
            extract_rate_limit_state, parse_retry_after, RateLimitState, DEFAULT_RETRY_AFTER_SECS,
        },
        timestamp::{read_timestamp, timestamp_key, write_timestamp, HEALTH_TS_KEY_PREFIX},
        HealthCheckResult, OverallStatus, SensorHealthChecker,
    },
    resources::{render_sensors_health_resource, RateLimitInfo, SensorHealthResult},
};
use prism_sensors::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    registry::AdapterRegistry,
};
use prism_spec_engine::{
    AuthType as EngAuthType, OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey,
    SensorInstanceOverlay, SensorSpec as EngSensorSpec, TableSpec,
};

// ─── Mock SensorAdapter implementations ──────────────────────────────────────
//
// Used by in-process unit tests at the adapter boundary (SID-1 §2).
// Each mock simulates a specific HTTP outcome without requiring a live DTU.

/// Mock adapter that simulates a successful HTTP 200 response (reachable, auth valid).
struct MockAdapterOk;

#[async_trait]
impl SensorAdapter for MockAdapterOk {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Simulate a successful empty probe (LIMIT 0).
        Ok(vec![])
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-ok"
    }
}

/// Mock adapter simulating HTTP 401 Unauthorized.
///
/// Returns `SensorError::HttpError { status: 401 }` to signal auth failure.
struct MockAdapterUnauthorized;

#[async_trait]
impl SensorAdapter for MockAdapterUnauthorized {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::HttpError {
            sensor: "crowdstrike".to_string(),
            status: 401,
            body: "mock 401 unauthorized".to_string(),
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-unauthorized"
    }
}

/// Mock adapter simulating HTTP 403 Forbidden.
struct MockAdapterForbidden;

#[async_trait]
impl SensorAdapter for MockAdapterForbidden {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::HttpError {
            sensor: "crowdstrike".to_string(),
            status: 403,
            body: "mock 403 forbidden".to_string(),
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-forbidden"
    }
}

/// Mock adapter simulating HTTP 401 Unauthorized for an "armis" sensor type.
///
/// Used by F-S504-LP3-HIGH-001 all-auth-invalid server-response test that needs two
/// distinct sensor types (crowdstrike + armis) both returning 401.
struct MockAdapterUnauthorizedArmis;

#[async_trait]
impl SensorAdapter for MockAdapterUnauthorizedArmis {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("armis")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::HttpError {
            sensor: "armis".to_string(),
            status: 401,
            body: "mock armis 401 unauthorized".to_string(),
        })
    }

    fn sensor_name(&self) -> &'static str {
        "armis-mock-unauthorized"
    }
}

/// Mock adapter simulating a connection error (sensor unreachable).
///
/// Returns `SensorError::Timeout` to signal that no HTTP exchange occurred.
struct MockAdapterConnectionRefused;

#[async_trait]
impl SensorAdapter for MockAdapterConnectionRefused {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::Timeout {
            sensor: "crowdstrike".to_string(),
            elapsed_ms: 30_000,
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-connection-refused"
    }
}

/// Mock adapter simulating a connection error for the "armis" sensor type.
///
/// Used by F-S504-LP1P3-MED-001 genuine-Down production-path test that needs two
/// distinct sensor types (crowdstrike + armis) both unreachable to prove the
/// "Sensor unreachable — verify network and endpoint configuration." branch fires
/// for all sensor types, and is entirely absent from 5xx tests.
struct MockAdapterConnectionRefusedArmis;

#[async_trait]
impl SensorAdapter for MockAdapterConnectionRefusedArmis {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("armis")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::Timeout {
            sensor: "armis".to_string(),
            elapsed_ms: 30_000,
        })
    }

    fn sensor_name(&self) -> &'static str {
        "armis-mock-connection-refused"
    }
}

/// Mock adapter simulating HTTP 429 Too Many Requests with a retry-after hint.
///
/// Returns `SensorError::RateLimited { retry_after_ms: 30_000 }` to verify:
/// - F-S504-P1-002: RateLimited → ConnectivityStatus::Up (NOT Down)
/// - F-S504-P1-001: `is_rate_limited=true` + `rate_limit_retry_after_ms=Some(30_000)` propagate
///   through ProbeOutcome → AuthProbeResult → SensorHealthResult.rate_limit
struct MockAdapterRateLimited;

#[async_trait]
impl SensorAdapter for MockAdapterRateLimited {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::RateLimited {
            sensor: "crowdstrike".to_string(),
            retry_after_ms: 30_000,
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-rate-limited"
    }
}

/// Mock adapter simulating HTTP 429 for an "armis" sensor type.
///
/// Used by EC-007 all-rate-limited tests that require two distinct sensor types
/// (crowdstrike + armis) to be registered under the same org_id.
struct MockAdapterRateLimitedArmis;

#[async_trait]
impl SensorAdapter for MockAdapterRateLimitedArmis {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("armis")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::RateLimited {
            sensor: "armis".to_string(),
            retry_after_ms: 30_000,
        })
    }

    fn sensor_name(&self) -> &'static str {
        "armis-mock-rate-limited"
    }
}

/// Mock adapter simulating an HTTP error with a hostile, oversized body.
///
/// Returns `SensorError::HttpError { status: 500, body: long_hostile }` to verify
/// F-S504-P2-008 sanitize_error: body is truncated to 512 chars and control chars
/// are replaced with spaces.
struct MockAdapterHostileBody;

#[async_trait]
impl SensorAdapter for MockAdapterHostileBody {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Hostile body: 2000 chars + embedded control characters (ANSI escape + null + newline)
        // The sanitizer should truncate at 512 chars and strip control chars.
        let long_body = format!("INJECTED\x1b[31mRED\x1b[0m\x00\n\r{}", "A".repeat(2000));
        Err(SensorError::HttpError {
            sensor: "crowdstrike".to_string(),
            status: 500,
            body: long_body,
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-hostile-body"
    }
}

/// Mock adapter that captures the `source_table` passed in the `SensorSpec`.
///
/// Used by F-S504-P2-009 test to assert that `probe_connectivity` constructs
/// a sensor-prefixed `source_table` (e.g. "armis_devices") not the historic
/// hardcoded "devices" string.
struct MockAdapterCapturingSpec {
    captured_source_table: std::sync::Mutex<Option<String>>,
    sensor_id: &'static str,
}

impl MockAdapterCapturingSpec {
    fn new(sensor_id: &'static str) -> Self {
        Self {
            captured_source_table: std::sync::Mutex::new(None),
            sensor_id,
        }
    }

    fn captured(&self) -> Option<String> {
        self.captured_source_table
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
}

#[async_trait]
impl SensorAdapter for MockAdapterCapturingSpec {
    fn sensor_type(&self) -> SensorId {
        SensorId::from(self.sensor_id)
    }

    async fn fetch(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        let mut guard = self
            .captured_source_table
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        *guard = Some(spec.source_table.clone());
        Ok(vec![])
    }

    fn sensor_name(&self) -> &'static str {
        "mock-capturing-spec"
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build a `SensorHealthResult` in live-probe scope (both fields = true).
fn live_result(sensor_id: &str, client_id: &str) -> SensorHealthResult {
    SensorHealthResult::new(sensor_id, client_id)
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(Utc::now())
}

/// Build a `SensorHealthResult` simulating an unreachable sensor.
fn down_result(sensor_id: &str, client_id: &str) -> SensorHealthResult {
    SensorHealthResult::new(sensor_id, client_id)
        .with_reachable(false)
        .with_auth_valid(false)
}

// ─── BC-2.08.003 — parse_retry_after ─────────────────────────────────────────

/// BC-2.08.003 postcondition: delta-seconds form `"30"` → Duration of 30 secs.
///
/// Story Dev Notes (S-5.04 v1.8): try `s.parse::<u64>()` first.
/// `parse_retry_after` is `todo!()` → RED.
#[test]
fn test_BC_2_08_003_parse_retry_after_delta_seconds() {
    let result = parse_retry_after("30");
    let dur = result.expect("parse_retry_after('30') must return Some(Duration)");
    assert_eq!(
        dur.as_secs(),
        30,
        "BC-2.08.003: delta-seconds '30' must yield Duration of 30 seconds; got {:?}",
        dur
    );
}

/// BC-2.08.003: whitespace padding is trimmed before parsing.
#[test]
fn test_BC_2_08_003_parse_retry_after_delta_seconds_with_whitespace() {
    let result = parse_retry_after("  60  ");
    let dur = result.expect("parse_retry_after with whitespace must return Some(Duration)");
    assert_eq!(
        dur.as_secs(),
        60,
        "BC-2.08.003: whitespace-padded '  60  ' must yield 60 seconds"
    );
}

/// BC-2.08.003: u64 range — value > u32::MAX must not overflow.
///
/// Story Dev Notes: parse as `u64`, not `u32`.
#[test]
fn test_BC_2_08_003_parse_retry_after_large_u64_does_not_overflow() {
    // u32::MAX + 1 = 4_294_967_296 — overflows u32 silently.
    let result = parse_retry_after("4294967296");
    let dur = result.expect("parse_retry_after large u64 must return Some(Duration)");
    assert_eq!(
        dur.as_secs(),
        4_294_967_296,
        "BC-2.08.003: u64 delta-seconds must not overflow; got {:?}",
        dur
    );
}

/// BC-2.08.003: IMF-fixdate (RFC 2822 / HTTP-date) form with future date returns positive duration.
///
/// Dev Notes: use `chrono::DateTime::parse_from_rfc2822` (NOT `parse_from_str` with `%Z`
/// — chrono issue #1575 fails on "GMT").
#[test]
fn test_BC_2_08_003_parse_retry_after_imf_fixdate_form() {
    // Far future — always a positive duration.
    let result = parse_retry_after("Sat, 01 Jan 2050 00:00:00 GMT");
    let dur = result.expect("parse_retry_after IMF-fixdate (future) must return Some(Duration)");
    assert!(
        dur.as_secs() > 0,
        "BC-2.08.003: IMF-fixdate far-future must return positive duration; got {:?}",
        dur
    );
}

/// BC-2.08.003: past IMF-fixdate clamps to Duration::ZERO ("retry immediately").
///
/// Dev Notes: "clamp past/negative to Duration::ZERO".
#[test]
fn test_BC_2_08_003_parse_retry_after_past_date_clamps_to_zero() {
    let result = parse_retry_after("Mon, 01 Jan 2001 00:00:00 GMT");
    let dur = result.expect("parse_retry_after past IMF-fixdate must return Some(Duration::ZERO)");
    assert_eq!(
        dur.as_secs(),
        0,
        "BC-2.08.003: past IMF-fixdate must clamp to Duration::ZERO; got {:?}",
        dur
    );
}

/// BC-2.08.003: parse failure on invalid value returns `None`.
///
/// Caller falls back to `DEFAULT_RETRY_AFTER_SECS`.
#[test]
fn test_BC_2_08_003_parse_retry_after_invalid_returns_none() {
    let result = parse_retry_after("not-valid");
    assert!(
        result.is_none(),
        "BC-2.08.003: invalid Retry-After value must return None (caller applies default fallback)"
    );
}

/// BC-2.08.003: empty string returns `None`.
#[test]
fn test_BC_2_08_003_parse_retry_after_empty_returns_none() {
    let result = parse_retry_after("");
    assert!(
        result.is_none(),
        "BC-2.08.003: empty Retry-After must return None"
    );
}

// ─── BC-2.08.003 — extract_rate_limit_state ──────────────────────────────────

/// AC-3 (BC-2.08.003 postcondition 1): HTTP 429 + `Retry-After: 30` header.
///
/// `extract_rate_limit_state` is `todo!()` → RED.
#[test]
fn test_BC_2_08_003_rate_limit_state_from_http_429_with_header() {
    let state = extract_rate_limit_state(Some("30"));
    assert!(
        state.is_rate_limited,
        "BC-2.08.003 AC-3: is_rate_limited must be true for HTTP 429 + Retry-After: 30"
    );
    let retry_after = state
        .retry_after
        .expect("BC-2.08.003 AC-3: retry_after must be Some when Retry-After header is present");
    let now = Utc::now();
    let delta = (retry_after - now).num_seconds();
    assert!(
        (25..=35).contains(&delta),
        "BC-2.08.003 AC-3: retry_after should be ~30s from now; got delta={delta}s"
    );
}

/// EC-003 (S-5.04): HTTP 429 without `Retry-After` header → default 60s backoff.
#[test]
fn test_BC_2_08_003_rate_limit_default_60s_when_no_header() {
    let state = extract_rate_limit_state(None);
    assert!(
        state.is_rate_limited,
        "BC-2.08.003 EC-003: is_rate_limited must be true even without Retry-After header"
    );
    let retry_after = state.retry_after.expect(
        "BC-2.08.003 EC-003: retry_after must be Some (defaulted to DEFAULT_RETRY_AFTER_SECS)",
    );
    let now = Utc::now();
    let delta = (retry_after - now).num_seconds();
    let default_secs = DEFAULT_RETRY_AFTER_SECS as i64;
    assert!(
        (default_secs - 5..=default_secs + 5).contains(&delta),
        "BC-2.08.003 EC-003: default retry_after should be ~{DEFAULT_RETRY_AFTER_SECS}s from now; got delta={delta}s"
    );
}

// ─── BC-2.08.003 — RateLimitState ────────────────────────────────────────────

/// BC-2.08.003 invariant: `is_cleared()` returns true when `retry_after` elapsed.
///
/// `RateLimitState::rate_limited` + `is_cleared` are `todo!()` → RED.
#[test]
fn test_BC_2_08_003_rate_limit_is_cleared_after_expiry() {
    let past = Utc::now() - ChronoDuration::seconds(120);
    let state = RateLimitState::rate_limited(past);
    assert!(
        state.is_cleared(),
        "BC-2.08.003: is_cleared() must return true when retry_after is in the past"
    );
}

/// BC-2.08.003: `is_cleared()` returns false when `retry_after` is in the future.
#[test]
fn test_BC_2_08_003_rate_limit_not_cleared_before_expiry() {
    let future = Utc::now() + ChronoDuration::seconds(120);
    let state = RateLimitState::rate_limited(future);
    assert!(
        !state.is_cleared(),
        "BC-2.08.003: is_cleared() must return false when retry_after is in the future"
    );
}

/// BC-2.08.003: `RateLimitState::not_limited()` is always cleared.
///
/// `not_limited` is `todo!()` → RED.
#[test]
fn test_BC_2_08_003_not_limited_state_is_cleared() {
    let state = RateLimitState::not_limited();
    assert!(
        !state.is_rate_limited,
        "BC-2.08.003: not_limited state must have is_rate_limited=false"
    );
    assert!(
        state.is_cleared(),
        "BC-2.08.003: not_limited state must always report is_cleared()=true"
    );
}

/// BC-2.08.003: `RateLimitState::rate_limited` sets `is_rate_limited=true`.
#[test]
fn test_BC_2_08_003_rate_limited_state_sets_flag() {
    let future = Utc::now() + ChronoDuration::seconds(30);
    let state = RateLimitState::rate_limited(future);
    assert!(
        state.is_rate_limited,
        "BC-2.08.003: rate_limited() must set is_rate_limited=true"
    );
}

// ─── BC-2.08.004 — timestamp_key ─────────────────────────────────────────────

/// BC-2.08.004: `timestamp_key("acme", "crowdstrike")` → `"health_ts/acme/crowdstrike"`.
///
/// `timestamp_key` is `todo!()` → RED.
#[test]
fn test_BC_2_08_004_timestamp_key_format() {
    let key = timestamp_key("acme", "crowdstrike");
    assert!(
        key.starts_with(HEALTH_TS_KEY_PREFIX),
        "BC-2.08.004: key must start with HEALTH_TS_KEY_PREFIX ({:?})",
        HEALTH_TS_KEY_PREFIX
    );
    let key_str =
        std::str::from_utf8(&key).expect("BC-2.08.004: timestamp_key must be valid UTF-8");
    assert_eq!(
        key_str, "health_ts/acme/crowdstrike",
        "BC-2.08.004: key format must be 'health_ts/{{client_id}}/{{sensor_id}}'"
    );
}

/// BC-2.08.004: distinct (client_id, sensor_id) pairs produce distinct keys.
#[test]
fn test_BC_2_08_004_timestamp_key_is_distinct_per_pair() {
    let ka = timestamp_key("acme", "crowdstrike");
    let kb = timestamp_key("globex", "crowdstrike");
    let kc = timestamp_key("acme", "armis");
    assert_ne!(
        ka, kb,
        "BC-2.08.004: different client_ids must yield different keys"
    );
    assert_ne!(
        ka, kc,
        "BC-2.08.004: different sensor_ids must yield different keys"
    );
    assert_ne!(
        kb, kc,
        "BC-2.08.004: all three key triples must be distinct"
    );
}

// ─── BC-2.08.004 — write_timestamp / read_timestamp ──────────────────────────

/// AC-4 (BC-2.08.004 postcondition 1): `write_timestamp` + `read_timestamp` round-trip.
///
/// Both functions are `todo!()` → RED.
#[test]
fn test_BC_2_08_004_write_and_read_timestamp_roundtrip() {
    let context = PrismContext::new();
    let expected = Utc.with_ymd_and_hms(2026, 4, 18, 10, 30, 0).unwrap();

    write_timestamp("acme", "crowdstrike", expected, &context);
    let actual = read_timestamp("acme", "crowdstrike", &context);

    assert_eq!(
        actual,
        Some(expected),
        "BC-2.08.004 AC-4: read_timestamp must return the timestamp written; \
         expected={expected:?}, got={actual:?}"
    );
}

/// BC-2.08.004: `read_timestamp` returns `None` before any write.
#[test]
fn test_BC_2_08_004_read_timestamp_returns_none_before_write() {
    let context = PrismContext::new();
    let result = read_timestamp("acme", "crowdstrike", &context);
    assert_eq!(
        result, None,
        "BC-2.08.004: read_timestamp must return None before any write; got={result:?}"
    );
}

/// BC-2.08.004: different (client_id, sensor_id) pairs are stored independently.
#[test]
fn test_BC_2_08_004_timestamps_are_independent_per_key() {
    let context = PrismContext::new();
    let ts_a = Utc.with_ymd_and_hms(2026, 4, 18, 10, 30, 0).unwrap();
    let ts_b = Utc.with_ymd_and_hms(2026, 4, 18, 11, 0, 0).unwrap();

    write_timestamp("acme", "crowdstrike", ts_a, &context);
    write_timestamp("globex", "crowdstrike", ts_b, &context);

    assert_eq!(read_timestamp("acme", "crowdstrike", &context), Some(ts_a));
    assert_eq!(
        read_timestamp("globex", "crowdstrike", &context),
        Some(ts_b)
    );
    assert_eq!(read_timestamp("acme", "armis", &context), None);
}

// ─── BC-2.08.007 — HealthCheckResult::aggregate ──────────────────────────────

/// AC-6 (BC-2.08.007 postcondition 1): 3 up + 1 down → `Partial`.
///
/// `HealthCheckResult::aggregate` is `todo!()` → RED.
#[test]
fn test_BC_2_08_007_aggregate_partial_when_some_up_some_down() {
    let results = vec![
        live_result("crowdstrike", "acme"),
        live_result("armis", "acme"),
        live_result("claroty", "acme"),
        down_result("cyberint", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    assert_eq!(
        status,
        OverallStatus::Partial,
        "BC-2.08.007 AC-6: 3 up + 1 down must yield OverallStatus::Partial"
    );
}

/// BC-2.08.007: all sensors up → `Healthy`.
#[test]
fn test_BC_2_08_007_aggregate_healthy_when_all_up() {
    let results = vec![
        live_result("crowdstrike", "acme"),
        live_result("armis", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    assert_eq!(
        status,
        OverallStatus::Healthy,
        "BC-2.08.007: all sensors up must yield OverallStatus::Healthy"
    );
}

/// BC-2.08.007: all sensors down → `Unhealthy`.
///
/// EC-005 (S-5.04): all down must still be a success response (not PrismError).
#[test]
fn test_BC_2_08_007_aggregate_unhealthy_when_all_down() {
    let results = vec![
        down_result("crowdstrike", "acme"),
        down_result("armis", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    assert_eq!(
        status,
        OverallStatus::Unhealthy,
        "BC-2.08.007: all sensors down must yield OverallStatus::Unhealthy"
    );
}

/// BC-2.08.007 invariant: `Partial` is a SUCCESS state (OverallStatus is not a Result).
///
/// Documents that the return type is never an error variant, per BC-2.08.007 postcondition 1.
#[test]
fn test_BC_2_08_007_invariant_partial_is_not_error() {
    let results = vec![
        live_result("crowdstrike", "acme"),
        down_result("armis", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    match status {
        OverallStatus::Partial => { /* BC-2.08.007 invariant satisfied */ }
        other => panic!("BC-2.08.007 invariant: 1 up + 1 down must be Partial; got {other:?}"),
    }
}

/// BC-2.08.007: `auth_invalid` sensor (reachable=true, auth_valid=false) counts as NOT healthy.
#[test]
fn test_BC_2_08_007_aggregate_auth_invalid_sensor_is_not_healthy() {
    let auth_invalid = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(false);
    let healthy = live_result("armis", "acme");
    let status = HealthCheckResult::aggregate(vec![auth_invalid, healthy]);
    assert_eq!(
        status,
        OverallStatus::Partial,
        "BC-2.08.007: auth_invalid sensor must not count toward Healthy"
    );
}

/// BC-2.08.007 F-S504-LP3-HIGH-001: ALL sensors auth-invalid (reachable=true, auth_valid=false,
/// no rate_limit) MUST return `OverallStatus::Unhealthy` — NOT `Partial`.
///
/// BC-2.08.007 §Postconditions canonical classification table:
/// - `"unhealthy"` = ALL sensors are unreachable OR auth-invalid (no rate-limited sensor present).
/// - `"partial"` = at least one sensor is unreachable OR auth-invalid (but not ALL).
///
/// Root cause: before this fix, `any_reachable = results.iter().any(|r| r.reachable != Some(false))`
/// treated auth-invalid sensors (reachable=true, auth_valid=false) as "partially available",
/// so a fleet of all-401 sensors returned `Partial`. This is a realistic MSSP scenario:
/// a client's entire fleet on an expired/rotated API key returns 401 from every sensor.
///
/// F-S504-LP3-HIGH-001 load-bearing test (TD-VSDD-059): uses sensors that are
/// `reachable=Some(true)` + `auth_valid=Some(false)` (NOT the `down_result` helper which also
/// sets `reachable=false` and would mask the bug).
///
/// RED GATE (pre-fix): returned `Partial` due to the `any_reachable` predicate bug — was RED
/// before the fix was applied. GREEN regression guard on current HEAD.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_08_007_aggregate_all_auth_invalid_is_unhealthy() {
    // ALL sensors are reachable-but-auth-invalid (401/403 pattern).
    // reachable=Some(true): connectivity probe returned Up (HTTP 4xx confirms exchange)
    // auth_valid=Some(false): auth probe returned Invalid
    // rate_limit=None: no rate-limiting observed
    let auth_invalid_cs = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(false);
    let auth_invalid_armis = SensorHealthResult::new("armis", "acme")
        .with_reachable(true)
        .with_auth_valid(false);
    let auth_invalid_claroty = SensorHealthResult::new("claroty", "acme")
        .with_reachable(true)
        .with_auth_valid(false);

    let status = HealthCheckResult::aggregate(vec![
        auth_invalid_cs,
        auth_invalid_armis,
        auth_invalid_claroty,
    ]);

    assert_eq!(
        status,
        OverallStatus::Unhealthy,
        "BC-2.08.007 F-S504-LP3-HIGH-001: when ALL sensors are auth-invalid \
         (reachable=true, auth_valid=false, no rate_limit), aggregate MUST return \
         OverallStatus::Unhealthy — NOT Partial. \
         'partial' = some sensors up, some down/auth-invalid; \
         'unhealthy' = ALL sensors unreachable or auth-invalid. \
         Got: {status:?}"
    );
}

/// BC-2.08.007 F-S504-LP3-HIGH-001 boundary: mixed (one auth-invalid + one healthy) → Partial.
///
/// This companion test verifies the fix does NOT break the existing partial case:
/// when SOME but not ALL sensors are auth-invalid, the result must still be Partial.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_08_007_aggregate_mixed_auth_invalid_and_healthy_is_partial() {
    let auth_invalid = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(false);
    let healthy = live_result("armis", "acme");

    let status = HealthCheckResult::aggregate(vec![auth_invalid, healthy]);

    assert_eq!(
        status,
        OverallStatus::Partial,
        "BC-2.08.007: mixed auth-invalid + healthy MUST remain Partial (not Unhealthy). \
         Only ALL-auth-invalid maps to Unhealthy. Got: {status:?}"
    );
}

/// BC-2.08.007: empty sensor list → `Unhealthy` (no healthy sensors).
#[test]
fn test_BC_2_08_007_aggregate_empty_list_is_unhealthy() {
    let status = HealthCheckResult::aggregate(vec![]);
    assert_eq!(
        status,
        OverallStatus::Unhealthy,
        "BC-2.08.007: empty sensor list must yield OverallStatus::Unhealthy"
    );
}

// ─── SensorHealthChecker::new (GREEN-BY-DESIGN) ───────────────────────────────

/// `SensorHealthChecker::new` is a plain struct constructor with no `todo!()`.
///
/// Expected to PASS against the stubs (GREEN-BY-DESIGN per BC-5.38.005 self-check).
/// Included to document that construction itself is not a Red Gate concern.
#[test]
fn test_BC_S_5_04_sensor_health_checker_new_constructs_successfully() {
    let registry = Arc::new(AdapterRegistry::new());
    let checker = SensorHealthChecker::new(Arc::clone(&registry));
    drop(checker); // Must not panic.
                   // GREEN-BY-DESIGN: SensorHealthChecker::new has no todo!() body.
}

// ─── BC-2.08.004 — SensorHealthChecker::{record_successful_query, last_successful_query} ─

/// AC-4 (BC-2.08.004 postcondition 1): `record_successful_query` + `last_successful_query`.
///
/// Both methods are `todo!()` → RED.
#[test]
fn test_BC_2_08_004_checker_record_and_read_timestamp() {
    let registry = Arc::new(AdapterRegistry::new());
    let checker = SensorHealthChecker::new(registry);
    let context = PrismContext::new();
    let sensor_id = SensorId::from("crowdstrike");
    let ts = Utc.with_ymd_and_hms(2026, 4, 18, 10, 30, 0).unwrap();

    checker.record_successful_query("acme", &sensor_id, ts, &context);

    let read_back = checker.last_successful_query("acme", &sensor_id, &context);
    assert_eq!(
        read_back,
        Some(ts),
        "BC-2.08.004 AC-4: last_successful_query must return the recorded timestamp; \
         expected={ts:?}, got={read_back:?}"
    );
}

/// BC-2.08.004: `last_successful_query` returns `None` before any `record_successful_query`.
#[test]
fn test_BC_2_08_004_checker_last_successful_query_none_before_record() {
    let registry = Arc::new(AdapterRegistry::new());
    let checker = SensorHealthChecker::new(registry);
    let context = PrismContext::new();
    let sensor_id = SensorId::from("crowdstrike");

    let result = checker.last_successful_query("acme", &sensor_id, &context);
    assert_eq!(
        result, None,
        "BC-2.08.004: last_successful_query must return None before any record call"
    );
}

/// AC-5 (BC-2.08.004 postcondition 2): timestamp persists across checker reconstruction.
///
/// Simulates a server restart by constructing a second `SensorHealthChecker` that reads
/// the same shared `PrismContext` / RocksDB storage.
///
/// Both methods are `todo!()` → RED.
#[test]
fn test_BC_2_08_004_timestamp_survives_checker_reconstruction() {
    let registry = Arc::new(AdapterRegistry::new());
    let checker_1 = SensorHealthChecker::new(Arc::clone(&registry));
    let context = PrismContext::new();
    let sensor_id = SensorId::from("crowdstrike");
    let ts = Utc.with_ymd_and_hms(2026, 4, 18, 10, 30, 0).unwrap();

    checker_1.record_successful_query("acme", &sensor_id, ts, &context);

    // Second checker (simulates restart) reads from the same shared context.
    let checker_2 = SensorHealthChecker::new(Arc::clone(&registry));
    let result = checker_2.last_successful_query("acme", &sensor_id, &context);
    assert_eq!(
        result,
        Some(ts),
        "BC-2.08.004 AC-5: timestamp must survive checker reconstruction on shared context"
    );
}

// ─── AC-1 (BC-2.08.001) — connectivity probe ─────────────────────────────────

/// AC-1 (BC-2.08.001 postcondition 1): 200-equivalent → `Up` + `latency_ms` non-zero.
///
/// SID-1 in-process companion: MockAdapterOk at the adapter boundary.
/// `probe_connectivity` is `todo!()` → RED.
#[tokio::test]
async fn test_BC_2_08_001_live_probe_200_returns_up_with_latency() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterOk));

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("BC-2.08.001: probe_connectivity must return Ok for reachable sensor");

    assert_eq!(
        outcome.status,
        ConnectivityStatus::Up,
        "BC-2.08.001 AC-1: 200-equivalent must yield ConnectivityStatus::Up"
    );
    assert!(
        outcome.latency_ms.is_some(),
        "BC-2.08.001 AC-1: latency_ms must be Some when sensor is Up"
    );
    assert!(
        outcome.latency_ms.unwrap() > 0,
        "BC-2.08.001 AC-1: latency_ms must be non-zero"
    );
}

/// AC-1 consumer boundary (F-S504-P1-MED-001): `check_one` for an `Up` sensor MUST
/// surface `latency_ms` as a non-zero `Some(u64)` on the CONSUMER-FACING `SensorHealthResult`.
///
/// This is the load-bearing test that closes finding F-S504-P1-MED-001.
/// The prior AC-1 test (`test_BC_2_08_001_live_probe_200_returns_up_with_latency`) verified
/// `ProbeOutcome.latency_ms` (internal). THIS test verifies the CONSUMER-FACING result from
/// `check_one` — the path that actually reaches the MCP response.
///
/// Thread:
///   probe_connectivity_inner → ProbeOutcome.latency_ms = Some(elapsed_ms.max(1))
///   → probe_auth_with_routing drops it (AuthProbeResult has no latency_ms field) [GAP]
///   → check_one never sees it → SensorHealthResult has no latency_ms field [GAP]
///   → consumer never receives latency.
///
/// RED GATE: fails until `latency_ms: Option<u64>` is threaded through
/// `AuthProbeResult` → `check_one` → `SensorHealthResult`.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_check_one_surfaces_latency_ms_on_consumer_result_F_S504_P1_MED_001() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterOk));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // Precondition: sensor must be Up (reachable) for latency_ms to be meaningful.
    assert_eq!(
        result.reachable,
        Some(true),
        "F-S504-P1-MED-001: precondition — sensor must be reachable (Up). Got: {:?}",
        result.reachable
    );

    // RED GATE (primary): latency_ms must be present and non-zero on SensorHealthResult.
    // This is the consumer-facing field. Before the fix, latency_ms was silently dropped
    // at the AuthProbeResult boundary and never reached SensorHealthResult.
    assert!(
        result.latency_ms.is_some(),
        "F-S504-P1-MED-001 RED GATE: SensorHealthResult.latency_ms MUST be Some(u64) for an \
         Up sensor — latency_ms was measured in probe_connectivity_inner but dropped at the \
         AuthProbeResult boundary, never reaching the consumer. Field must be threaded through \
         AuthProbeResult → check_one → SensorHealthResult. Got: None"
    );
    assert!(
        result.latency_ms.unwrap() > 0,
        "F-S504-P1-MED-001: SensorHealthResult.latency_ms MUST be non-zero (min 1ms). \
         Got: {:?}",
        result.latency_ms
    );
}

/// AC-1 (SID-1): integration test requiring live DTU (blocked).
///
/// Companion: `test_BC_2_08_001_live_probe_200_returns_up_with_latency` (above).
#[tokio::test]
#[ignore = "DTU-EXT-001: requires prism-dtu-crowdstrike clone; ungated after S-DEMO-001 wires boot step 9A"]
async fn test_BC_2_08_001_live_probe_200_returns_up_with_latency_dtu() {
    todo!("DTU-EXT-001: fill in after S-DEMO-001 wires AdapterRegistry at boot step 9A")
}

// ─── AC-2 (BC-2.08.002) — auth probe ─────────────────────────────────────────

/// AC-2 (BC-2.08.002 postcondition 1): HTTP 401 → `auth_invalid` (NOT `down`).
///
/// `probe_auth` is `todo!()` → RED.
#[tokio::test]
async fn test_BC_2_08_002_live_probe_401_returns_auth_invalid_not_down() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterUnauthorized));

    let result = probe_auth(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("BC-2.08.002: probe_auth must return Ok on HTTP 401 (not an engine error)");

    assert_eq!(
        result.auth,
        AuthStatus::Invalid,
        "BC-2.08.002 AC-2: HTTP 401 must yield AuthStatus::Invalid (not Unknown)"
    );
    // MUST NOT be Down — BC-2.08.002 postcondition 1.
    assert_ne!(
        result.connectivity,
        ConnectivityStatus::Down,
        "BC-2.08.002 AC-2: HTTP 401 sensor is reachable — connectivity must not be Down"
    );
}

/// EC-004 (BC-2.08.002 postcondition 1): connection error → `connectivity=Down`, `auth=Unknown`.
///
/// MUST NOT conflate "unreachable" with "auth invalid".
#[tokio::test]
async fn test_BC_2_08_002_connection_refused_returns_down_auth_unknown() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterConnectionRefused));

    let result = probe_auth(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("BC-2.08.002 EC-004: probe_auth must return Ok on connection error");

    assert_eq!(
        result.connectivity,
        ConnectivityStatus::Down,
        "BC-2.08.002 EC-004: connection refused must yield connectivity=Down"
    );
    assert_eq!(
        result.auth,
        AuthStatus::Unknown,
        "BC-2.08.002 EC-004: connection refused must yield auth=Unknown (cannot determine)"
    );
}

/// BC-2.08.002: HTTP 403 must also yield `auth=Invalid`.
#[tokio::test]
async fn test_BC_2_08_002_live_probe_403_returns_auth_invalid() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterForbidden));

    let result = probe_auth(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("BC-2.08.002: probe_auth must return Ok on HTTP 403");

    assert_eq!(
        result.auth,
        AuthStatus::Invalid,
        "BC-2.08.002: HTTP 403 must yield AuthStatus::Invalid"
    );
}

/// AC-2 (SID-1): integration test requiring live DTU (blocked).
#[tokio::test]
#[ignore = "DTU-EXT-001: requires prism-dtu-crowdstrike clone; ungated after S-DEMO-001 wires boot step 9A"]
async fn test_BC_2_08_002_live_probe_401_returns_auth_invalid_dtu() {
    todo!("DTU-EXT-001: fill in after S-DEMO-001 wires AdapterRegistry at boot step 9A")
}

// ─── BC-2.08.006 — sensors health resource shape ─────────────────────────────

/// BC-2.08.006 postcondition 2: `sensors` must be a JSON OBJECT keyed by `sensor_id`.
///
/// Verifies the `BTreeMap` keyed-object shape introduced in BC-2.08.006 via a
/// live-scope cache entry (S-5.04 probe_level="live").
///
/// `render_sensors_health_resource` is implemented (not todo!()); this test uses a
/// live-probe result to exercise the S-5.04 shape contract.
#[test]
fn test_BC_2_08_006_sensors_health_resource_sensors_is_keyed_object() {
    let context = PrismContext::new();
    let mut result = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(Utc::now());
    // Force probe_level to "live" (new() sets it to "spec-only").
    result.probe_level = "live".to_string();

    context
        .health_cache
        .insert("acme".to_string(), "crowdstrike".to_string(), result);

    let resource_result = render_sensors_health_resource(&context)
        .expect("BC-2.08.006: render_sensors_health_resource must succeed");

    let text = resource_result
        .contents
        .iter()
        .find_map(|c| match c {
            rmcp::model::ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
            _ => None,
        })
        .expect("BC-2.08.006: resource must have text content");

    let payload: serde_json::Value =
        serde_json::from_str(&text).expect("BC-2.08.006: resource JSON must be valid");

    let sensors_value = &payload["clients"]["acme"]["sensors"];
    assert!(
        sensors_value.is_object(),
        "BC-2.08.006 postcondition 2: sensors must be a JSON object (keyed by sensor_id); \
         got: {sensors_value:?}"
    );
    assert!(
        !sensors_value.is_array(),
        "BC-2.08.006 postcondition 2: sensors MUST NOT be a JSON array"
    );
    assert!(
        sensors_value.get("crowdstrike").is_some(),
        "BC-2.08.006: 'crowdstrike' key must be present in sensors object"
    );
}

/// BC-2.08.006 S-5.04 scope: cached live-probe result shows `probe_level="live"` in resource.
#[test]
fn test_BC_2_08_006_sensors_health_resource_live_probe_level() {
    let context = PrismContext::new();
    let mut result = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(Utc::now());
    result.probe_level = "live".to_string();
    context
        .health_cache
        .insert("acme".to_string(), "crowdstrike".to_string(), result);

    let resource_result = render_sensors_health_resource(&context).expect("must succeed");
    let text = resource_result
        .contents
        .iter()
        .find_map(|c| match c {
            rmcp::model::ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default();

    assert!(
        text.contains(r#""probe_level":"live""#) || text.contains(r#""probe_level": "live""#),
        "BC-2.08.006 S-5.04: resource must show probe_level='live'; got: {text:.200}"
    );
}

/// BC-2.08.006 S-5.04: `reachable` must be a boolean (not null) in live-probe scope.
#[test]
fn test_BC_2_08_006_sensors_health_resource_reachable_is_boolean() {
    let context = PrismContext::new();
    let mut result = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(Utc::now());
    result.probe_level = "live".to_string();
    context
        .health_cache
        .insert("acme".to_string(), "crowdstrike".to_string(), result);

    let resource_result = render_sensors_health_resource(&context).expect("must succeed");
    let text = resource_result
        .contents
        .iter()
        .find_map(|c| match c {
            rmcp::model::ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default();

    let payload: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    let reachable = &payload["clients"]["acme"]["sensors"]["crowdstrike"]["reachable"];
    assert!(
        reachable.is_boolean(),
        "BC-2.08.006 S-5.04: reachable must be a boolean in live-probe scope, not null; \
         got: {reachable:?}"
    );
    assert_eq!(
        reachable.as_bool(),
        Some(true),
        "BC-2.08.006 S-5.04: reachable must be true for a successful probe"
    );
}

// ─── BC-2.08.005 RECONCILIATION-3 — QueryEngine live accessors ────────────────

/// BC-2.08.005 RECONCILIATION-3: `QueryEngine::cursor_count()` returns a usize.
///
/// Fresh QueryEngine (no active queries) must return 0 when implemented.
/// Method is `todo!()` → RED.
///
/// `#[tokio::test]`: `QueryEngine::new()` spawns a background cursor-cleanup task
/// that requires a Tokio runtime. Without the attribute the test panics with
/// "no reactor running" even before reaching the `todo!()`.
#[tokio::test]
async fn test_BC_2_08_005_query_engine_cursor_count_accessor() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_query::{engine::QueryEngine, engine::QueryEngineConfig};

    let engine = QueryEngine::new(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    );

    // cursor_count() is todo!() → panics → RED.
    // When implemented, a fresh engine must return 0.
    let count = engine.cursor_count();
    assert_eq!(
        count, 0,
        "BC-2.08.005 RECONCILIATION-3: fresh QueryEngine cursor_count must be 0; got {count}"
    );
}

/// BC-2.08.005 RECONCILIATION-3: `QueryEngine::token_count()` returns a usize.
///
/// Method is `todo!()` → RED.
#[tokio::test]
async fn test_BC_2_08_005_query_engine_token_count_accessor() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_query::{engine::QueryEngine, engine::QueryEngineConfig};

    let engine = QueryEngine::new(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    );

    // token_count() is todo!() → panics → RED.
    // When implemented, a fresh engine must return 0.
    let count = engine.token_count();
    assert_eq!(
        count, 0,
        "BC-2.08.005 RECONCILIATION-3: fresh QueryEngine token_count must be 0; got {count}"
    );
}

// ─── FIX-001/v1.6 architectural invariant ─────────────────────────────────────

/// FIX-001/v1.6 mandate: health module source files must NOT contain `reqwest::Client::new()`.
///
/// This is a source-scan regression guard. It will PASS against the stubs (which contain
/// only `todo!()`) and will continue to PASS after implementation if the invariant is
/// honoured. It will FAIL if an implementer introduces a direct `reqwest::Client` call.
///
/// Note: GREEN-BY-DESIGN against the current stubs — included as a regression guard,
/// not a Red Gate test.
#[test]
fn test_BC_S_5_04_invariant_no_direct_reqwest_client_in_health_module() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo");
    let health_sources = [
        format!("{manifest_dir}/src/health/mod.rs"),
        format!("{manifest_dir}/src/health/connectivity.rs"),
        format!("{manifest_dir}/src/health/auth.rs"),
        format!("{manifest_dir}/src/health/rate_limit.rs"),
        format!("{manifest_dir}/src/health/timestamp.rs"),
    ];

    for path in &health_sources {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !source.contains("reqwest::Client::new()"),
            "FIX-001/v1.6 violation in {path}: \
             health module must NOT call reqwest::Client::new() directly. \
             All probes must route through AdapterRegistry::get(org_id, sensor_id).fetch()"
        );
        assert!(
            !source.contains("reqwest::ClientBuilder"),
            "FIX-001/v1.6 violation in {path}: \
             health module must NOT use reqwest::ClientBuilder"
        );
    }
}

// ─── F-S504-P1-001/002 — HTTP 429 probe_connectivity + check_one ────────────

/// F-S504-P1-002 (core): `probe_connectivity` with a `SensorError::RateLimited` mock
/// MUST return `ConnectivityStatus::Up` (NOT `Down`).
///
/// Closing test: the RateLimited explicit arm in connectivity.rs must not fall
/// through to the catch-all `Err(e) => Down` arm.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_live_probe_429_status_is_up_not_down() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterRateLimited));

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("F-S504-P1-002: probe_connectivity must return Ok for RateLimited response");

    assert_eq!(
        outcome.status,
        ConnectivityStatus::Up,
        "F-S504-P1-002: SensorError::RateLimited MUST yield ConnectivityStatus::Up (sensor is \
         reachable; HTTP 429 confirms an HTTP exchange occurred). Got: {:?}",
        outcome.status
    );
    assert_eq!(
        outcome.http_status,
        Some(429),
        "F-S504-P1-002: http_status must be 429 for RateLimited response. Got: {:?}",
        outcome.http_status
    );
    assert!(
        outcome.is_rate_limited,
        "F-S504-P1-001: ProbeOutcome.is_rate_limited must be true for RateLimited response"
    );
    assert_eq!(
        outcome.rate_limit_retry_after_ms,
        Some(30_000),
        "F-S504-P1-001: rate_limit_retry_after_ms must propagate from SensorError. \
         Got: {:?}",
        outcome.rate_limit_retry_after_ms
    );
}

/// F-S504-P1-001 (wiring): `check_one` with a `MockAdapterRateLimited` MUST populate
/// `SensorHealthResult.rate_limit` with a `RateLimitInfo` value (not None).
///
/// This verifies the full wiring path:
///   SensorError::RateLimited → ProbeOutcome.is_rate_limited=true
///   → extract_rate_limit_state → context.rate_limit_states persisted
///   → SensorHealthResult.rate_limit = Some(RateLimitInfo { reset_at: Some(...) })
///
/// Closing test for F-S504-P1-001: rate_limit_states and SensorHealthResult.rate_limit
/// were dead code before this wiring; this test forces the full production path.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_003_live_probe_429_yields_up_and_populates_rate_limit() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterRateLimited));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // F-S504-P1-002: sensor is reachable (Up, not Down) when rate-limited.
    assert_eq!(
        result.reachable,
        Some(true),
        "F-S504-P1-002: reachable must be Some(true) for HTTP 429 (sensor responded). \
         Got: {:?}",
        result.reachable
    );

    // F-S504-P1-001: rate_limit MUST be populated (was always None before the fix).
    assert!(
        result.rate_limit.is_some(),
        "F-S504-P1-001: SensorHealthResult.rate_limit MUST be Some(RateLimitInfo) when \
         HTTP 429 observed; was always None (dead-code path) before this fix. Got: {:?}",
        result.rate_limit
    );

    // The reset_at timestamp must be in the future (extract_rate_limit_state adds ~30s).
    let rate_limit = result.rate_limit.unwrap();
    if let Some(reset_at) = rate_limit.reset_at {
        assert!(
            reset_at > chrono::Utc::now(),
            "F-S504-P1-001: rate_limit.reset_at must be in the future (retry_after=30s). \
             Got: {reset_at:?}"
        );
    } else {
        panic!(
            "F-S504-P1-001: rate_limit.reset_at must be Some(DateTime) when \
             retry_after_ms=30_000. Got None"
        );
    }

    // F-S504-P1-001: context.rate_limit_states must be persisted.
    let guard = context.rate_limit_states.lock().unwrap();
    let key = prism_mcp::context::SensorKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
    };
    assert!(
        guard.contains_key(&key),
        "F-S504-P1-001: context.rate_limit_states must contain the (acme, crowdstrike) key \
         after a 429 probe. Keys present: {:?}",
        guard.keys().collect::<Vec<_>>()
    );
}

// ─── F-S504-P2-008 — sanitize_error load-bearing test ───────────────────────

/// F-S504-P2-008 (load-bearing): `probe_connectivity` with a hostile oversized error body
/// MUST produce a sanitized `ProbeOutcome.error` — truncated to 512 chars and stripped
/// of ASCII control characters.
///
/// This is the load-bearing test for sanitize_error() (CWE-116 / prompt-injection defense).
/// Without this test the sanitize_error path had no coverage verifying the output.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_live_probe_error_body_is_sanitized() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterHostileBody));

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("F-S504-P2-008: probe_connectivity must return Ok for HTTP 500");

    let error_str = outcome.error.as_deref().unwrap_or("");

    // 1. Length MUST be <= 512 chars (MAX_ERROR_LEN).
    assert!(
        error_str.chars().count() <= 512,
        "F-S504-P2-008 (CWE-116): sanitized error MUST be at most 512 chars; \
         got {} chars: {:?}",
        error_str.chars().count(),
        &error_str[..error_str.len().min(100)]
    );

    // 2. Control characters (ESC, NUL, newline, CR) MUST be replaced with spaces.
    assert!(
        !error_str.chars().any(|c| c.is_ascii_control()),
        "F-S504-P2-008 (CWE-116): sanitized error MUST contain no ASCII control chars. \
         Found control chars: {:?}",
        error_str
            .chars()
            .filter(|c| c.is_ascii_control())
            .collect::<String>()
    );

    // 3. ANSI escape sequence start (ESC = 0x1b) must not survive.
    assert!(
        !error_str.contains('\x1b'),
        "F-S504-P2-008: ESC character (ANSI escape start) must be stripped. \
         Got: {error_str:?}"
    );
}

// ─── F-S504-P1-004 — with_token_store wired in boot.rs ────────────────────

/// F-S504-P1-004 (load-bearing): `QueryEngine::token_count()` returns the actual live
/// count from the wired `ConfirmationTokenStore` (not always 0).
///
/// Before the boot.rs fix, `with_token_store()` was never called, so `token_store`
/// was always None and `token_count()` always returned 0 regardless of active tokens.
///
/// This test exercises the production code path: create a store, generate a token,
/// wire the store into a QueryEngine via `with_token_store()`, assert count == 1.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_005_query_engine_token_count_reflects_wired_store() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_query::{engine::QueryEngine, engine::QueryEngineConfig};
    use prism_security::confirmation_token::ConfirmationTokenStore;

    let store = Arc::new(ConfirmationTokenStore::new());

    // Generate a token so active_count() > 0.
    store
        .generate(
            "acme",
            "delete_host",
            serde_json::json!({"host_id": "H001"}),
            "Delete host H001 from acme",
        )
        .expect("generate must succeed (store is empty, below TOKEN_CAP)");

    // Before wiring: fresh engine with no token store returns 0.
    let engine_unwired = QueryEngine::new(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    );
    assert_eq!(
        engine_unwired.token_count(),
        0,
        "F-S504-P1-004: unwired engine (no with_token_store) must return 0"
    );

    // After wiring: engine with the populated store returns 1.
    let engine_wired = QueryEngine::new(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_token_store(Arc::clone(&store));

    assert_eq!(
        engine_wired.token_count(),
        1,
        "F-S504-P1-004: wired engine MUST return 1 when store has 1 active token; \
         before this fix with_token_store() was never called in boot.rs and \
         token_count() always returned 0. Got: {}",
        engine_wired.token_count()
    );
}

// ─── F-S504-P1-005 — RocksDB timestamp persistence (restart simulation) ──────

/// F-S504-P1-005 (load-bearing): timestamps written via `write_timestamp` MUST be
/// readable by a NEW `PrismContext` backed by the SAME `InMemoryBackend` — simulating
/// a server restart where the in-memory map is empty but RocksDB still holds the value.
///
/// This is the definitive closing test for AC-5 (BC-2.08.004 postcondition 2):
/// the old implementation only wrote to the in-memory map (a rationalized deferral).
/// This test catches any regression to in-memory-only behavior by using a SEPARATE
/// `PrismContext` that shares the storage backend but NOT the in-memory map.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_08_004_timestamp_survives_context_reconstruction_with_storage() {
    use prism_mcp::context::PrismContext;
    use prism_storage::memory_backend::InMemoryBackend;

    let storage: Arc<dyn prism_storage::backend::RocksStorageBackend> =
        Arc::new(InMemoryBackend::new());

    // ── Process 1 (pre-restart) ───────────────────────────────────────────────
    let context_1 = PrismContext::new_with_storage(Arc::clone(&storage));
    let ts_written = chrono::Utc.with_ymd_and_hms(2026, 6, 19, 12, 0, 0).unwrap();
    write_timestamp("acme", "crowdstrike", ts_written, &context_1);

    // Confirm in-memory map is populated (fast path passes).
    let in_memory_result = read_timestamp("acme", "crowdstrike", &context_1);
    assert_eq!(
        in_memory_result,
        Some(ts_written),
        "F-S504-P1-005 precondition: in-memory write+read must work in same context"
    );

    // ── Process 2 (post-restart) — fresh context, same storage ───────────────
    // New PrismContext starts with an EMPTY in-memory map — simulates restart.
    // If write_timestamp only updated the HashMap (old behavior), this would return None.
    let context_2 = PrismContext::new_with_storage(Arc::clone(&storage));
    let after_restart = read_timestamp("acme", "crowdstrike", &context_2);

    assert_eq!(
        after_restart,
        Some(ts_written),
        "F-S504-P1-005 (BC-2.08.004 postcondition 2 — AC-5 'survives restart'): \
         read_timestamp MUST recover the value from RocksDB when the in-memory map is empty \
         (new context = restart). Old implementation returned None here because it only wrote \
         to HashMap. Got: {:?}",
        after_restart
    );

    // Verify the cold-path cache repopulation: reading again hits the in-memory map.
    let second_read = read_timestamp("acme", "crowdstrike", &context_2);
    assert_eq!(
        second_read,
        Some(ts_written),
        "F-S504-P1-005: second read after cold-path must still return the timestamp \
         (repopulated into in-memory map on first read)"
    );
}

// ─── F-S504-P2-009 — source_table is sensor-prefixed, not hardcoded "devices" ──────────────

/// F-S504-P2-009 (BC-2.08.001): `probe_connectivity` MUST construct the `SensorSpec`
/// with a sensor-generic `source_table` of the form `{sensor_id}_devices`, NOT the
/// CrowdStrike-specific hardcoded `"devices"` string.
///
/// Regression guard: if this test fails, the hardcoded `"devices"` was re-introduced,
/// breaking probes for sensors where the spec-driven adapter strips the sensor prefix
/// to select the matching table entry (e.g. "armis_devices" → strips "armis_" → "devices").
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_live_probe_source_table_is_sensor_prefixed() {
    let adapter = Arc::new(MockAdapterCapturingSpec::new("armis"));
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("armis");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    let result = probe_connectivity(&registry, org_id, &sensor_id, "acme-client")
        .await
        .expect("probe must not error");

    // Probe must succeed (mock returns Ok([])).
    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "F-S504-P2-009: probe with capturing adapter must return Up"
    );

    // The source_table passed to the adapter MUST be sensor-prefixed.
    let captured = adapter
        .captured()
        .expect("adapter must have been called (source_table captured)");
    assert_eq!(
        captured, "armis_devices",
        "F-S504-P2-009: source_table MUST be '{{sensor_id}}_devices' ('armis_devices'), \
         NOT the historic hardcoded 'devices'. Got: '{captured}'"
    );
}

// ─── F-S504-P2-010 — ProbeAuth stub works because adapter handles credentials internally ────

/// F-S504-P2-010 (BC-2.08.001/002): the health probe MUST succeed even though `ProbeAuth`
/// carries no real credentials.
///
/// Rationale: in production, all `SpecDrivenSensorAdapter` instances use
/// `AdapterAuthStrategy::Plugin(...)` variants (including `BearerStaticCredentialAuthProvider`
/// for bearer_static sensors). All Plugin variants IGNORE the `SensorAuth` argument and
/// resolve credentials internally via their held `Arc<dyn AuthProvider>` (ADR-028 §D10;
/// ADV-SDEMO002-P01-CRIT-001).
///
/// `ProbeAuth` carries `auth_type_name = "bearer_static"` but is NOT a `BearerStaticSensorAuth`.
/// Mock adapters that ignore auth (simulating Plugin strategy) MUST succeed.
/// This is the confirming test for the "adapter handles credential injection internally" claim.
///
/// If this test fails, a Plugin-strategy adapter accidentally attempts to downcast
/// the `SensorAuth` arg — which would regress ADV-SDEMO002-P01-CRIT-001.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_probe_auth_stub_succeeds_when_adapter_handles_creds_internally() {
    // MockAdapterOk ignores auth entirely (simulates Plugin strategy behaviour).
    let adapter = Arc::new(MockAdapterOk);
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    let result = probe_connectivity(&registry, org_id, &sensor_id, "acme-client")
        .await
        .expect("probe must not error even with stub ProbeAuth");

    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "F-S504-P2-010: ProbeAuth stub (no real creds) MUST not cause a Down status — \
         adapter handles credential injection internally (ADR-028 §D10)"
    );
    assert!(
        result.error.is_none(),
        "F-S504-P2-010: no error expected when adapter ignores auth arg. Got: {:?}",
        result.error
    );
}

// ---------------------------------------------------------------------------
// AC-9 (BC-2.08.001 postcondition 5) — probe_table routing via probe_connectivity_with_routing
// S-5.04 IMPLEMENTED (implementer pass complete)
//
// These tests exercise the probe_table routing behavior implemented in
// probe_connectivity_inner (connectivity.rs). The routing uses UNDERSCORE form:
//   1. probe_table = Some("tbl") → source_table = "{sensor_id}_{probe_table}"
//   2. probe_table = None, first_table_name = Some("tbl") → source_table = "{sensor_id}_{tbl}"
//   3. Both None → legacy sentinel "{sensor_id}_devices"
//
// UNDERSCORE contract: `SpecDrivenSensorAdapter::fetch` selects a single table by calling
// `spec.source_table.strip_prefix("{sensor_id}_")`. Dot form "{sensor_id}.{table}" would
// never match the strip_prefix and would cause fan-out to ALL tables (F-S504-P1-002 fix).
//
// Tests call probe_connectivity_with_routing (the extended form that accepts probe_table
// and first_table_name explicitly). Assertions correctly assert underscore form.
// See F-S504-P2-RE-001 tests below for the check_one / new_with_spec_map end-to-end path.
// ---------------------------------------------------------------------------

/// AC-9 (S-5.04): When `probe_table = Some("detections")`, `probe_connectivity_with_routing`
/// MUST route the LIMIT-0 fetch to `"crowdstrike_detections"` (underscore form, BC-2.08.001 §5).
///
/// UNDERSCORE contract: `SpecDrivenSensorAdapter::fetch` calls
/// `spec.source_table.strip_prefix("crowdstrike_")` to select the table.  Dot form
/// "crowdstrike.detections" would never match the strip_prefix and would cause fan-out to
/// all tables (F-S504-P1-002 fix).  The assertion correctly checks for underscore form.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_probe_routes_to_probe_table_when_set() {
    // AC-9 IMPLEMENTED: calls probe_connectivity_with_routing with probe_table = Some("detections").
    // Assertion checks for UNDERSCORE form "crowdstrike_detections" (canonical for
    // SpecDrivenSensorAdapter::fetch's strip_prefix selection).
    let adapter = Arc::new(MockAdapterCapturingSpec::new("crowdstrike"));
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    // AC-9: use probe_connectivity_with_routing to pass probe_table = Some("detections").
    // crowdstrike.sensor.toml declares probe_table = "detections" (S-5.04 AC-10).
    let result = probe_connectivity_with_routing(
        &registry,
        org_id,
        &sensor_id,
        "acme",
        Some("detections"),
        None,
    )
    .await
    .expect("probe_connectivity must not error");

    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "AC-9: probe must succeed (Up) regardless of routing behavior"
    );

    let captured = adapter
        .captured()
        .expect("adapter must have been called (source_table captured)");

    // F-S504-P1-002: source_table MUST use underscore form to match SpecDrivenSensorAdapter::fetch's
    // strip_prefix("{sensor_id}_") table selection. Dot form "crowdstrike.detections" would
    // never match strip_prefix("crowdstrike_") and would cause fan-out to ALL tables.
    assert_eq!(
        captured, "crowdstrike_detections",
        "AC-9 (BC-2.08.001 §5): when probe_table = Some(\"detections\"), source_table MUST be \
         \"crowdstrike_detections\" (underscore form, canonical for SpecDrivenSensorAdapter). \
         Got '{captured}' — F-S504-P1-002 fix required"
    );
}

/// AC-9 (S-5.04): When `probe_table` is absent but tables exist, `probe_connectivity_with_routing`
/// MUST fall back to `"{sensor_id}_{spec.tables[0].table_name}"` (underscore form, BC-2.08.001 §5).
///
/// For a sensor with `tables = ["alerts", "incidents"]`, the fallback is
/// `"cyberint_alerts"` (first declared table in TOML order, underscore-joined).
///
/// UNDERSCORE contract: same as the probe_table case — `SpecDrivenSensorAdapter::fetch`
/// uses `strip_prefix("cyberint_")` for table selection; dot form "cyberint.alerts" would
/// not match and would fan-out to all tables (F-S504-P1-002 fix).
/// The assertion correctly checks `captured == "cyberint_alerts"`.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_probe_falls_back_to_first_table_when_probe_table_absent() {
    // AC-9 IMPLEMENTED: calls probe_connectivity_with_routing with probe_table = None,
    // first_table_name = Some("alerts"). Assertion checks for UNDERSCORE form "cyberint_alerts".
    let adapter = Arc::new(MockAdapterCapturingSpec::new("cyberint"));
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("cyberint");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    // AC-9 fallback: probe_table = None, first_table_name = Some("alerts").
    // cyberint.sensor.toml: probe_table absent, tables[0].table_name = "alerts".
    let result = probe_connectivity_with_routing(
        &registry,
        org_id,
        &sensor_id,
        "acme",
        None,
        Some("alerts"),
    )
    .await
    .expect("probe_connectivity must not error");

    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "AC-9 fallback: probe must succeed (Up)"
    );

    let captured = adapter
        .captured()
        .expect("adapter must have been called (source_table captured)");

    // F-S504-P1-002: underscore form is canonical for SpecDrivenSensorAdapter::fetch.
    // "cyberint.alerts" (dot form) would fail strip_prefix("cyberint_") → fan-out to ALL tables.
    assert_eq!(
        captured, "cyberint_alerts",
        "AC-9 (BC-2.08.001 §5): when probe_table is absent, source_table MUST fall back to \
         \"{{sensor_id}}_{{tables[0].table_name}}\" = \"cyberint_alerts\" (underscore form). \
         Got '{captured}' — F-S504-P1-002 fix required"
    );
}

// ─── F-S504-P1-002 integration test: strip_prefix stand-in ───────────────────
//
// The MockAdapterCapturingSpec above confirms that probe_connectivity_with_routing
// passes the correct source_table string.  That is a necessary but NOT sufficient
// guarantee: we also need to verify that the canonical UNDERSCORE form is what
// SpecDrivenSensorAdapter::fetch would accept.
//
// SpecDrivenSensorAdapter::fetch (crates/prism-bin/src/spec_driven_adapter.rs §647-649):
//
//   let sensor_id_str = self.sensor_spec.spec.sensor_id.as_str();
//   let queried_table_name: Option<&str> =
//       spec.source_table.strip_prefix(&format!("{sensor_id_str}_"));
//
// The strip_prefix call resolves a single table from source_table.  The result is
// Some(table_name) when source_table == "{sensor_id}_{table_name}" — i.e. exactly the
// underscore form.  The result is None when source_table uses dot form or any other
// convention, which causes the adapter to fan-out to all declared tables instead of
// routing to a single table.
//
// This test uses MockAdapterStripPrefix (below): a stand-in that replicates the
// strip_prefix logic and returns Err only when strip_prefix finds no match.  The test
// proves end-to-end that probe_connectivity_with_routing produces a source_table that
// strip_prefix("{sensor_id}_") resolves to a non-None table name.
// ─────────────────────────────────────────────────────────────────────────────

/// Mock adapter simulating HTTP 503 Service Unavailable.
///
/// Returns `SensorError::HttpError { status: 503 }` to drive EC-08-009 coverage:
/// BC-2.08.002 EC-08-009 (HS-007): HTTP 503 → `reachable: true` (network-reachable, erroring),
/// `reason: "service_unavailable"`. Only `Down` (no HTTP exchange) → `reachable: false`.
struct MockAdapterServiceUnavailable;

#[async_trait]
impl SensorAdapter for MockAdapterServiceUnavailable {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::HttpError {
            sensor: "crowdstrike".to_string(),
            status: 503,
            body: "Service Unavailable".to_string(),
        })
    }

    fn sensor_name(&self) -> &'static str {
        "crowdstrike-mock-service-unavailable"
    }
}

/// Mock adapter simulating HTTP 503 Service Unavailable for "armis" sensor type.
///
/// Used by the all-503-fleet aggregate test requiring two distinct sensor types
/// (crowdstrike + armis) both returning 503 Service Unavailable.
struct MockAdapterServiceUnavailableArmis;

#[async_trait]
impl SensorAdapter for MockAdapterServiceUnavailableArmis {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("armis")
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Err(SensorError::HttpError {
            sensor: "armis".to_string(),
            status: 503,
            body: "Service Unavailable".to_string(),
        })
    }

    fn sensor_name(&self) -> &'static str {
        "armis-mock-service-unavailable"
    }
}

/// Stand-in adapter that applies `strip_prefix("{sensor_id}_")` to `spec.source_table`,
/// faithfully modelling `SpecDrivenSensorAdapter::fetch`'s single-table selection.
///
/// Returns `Ok(vec![])` when strip_prefix succeeds (underscore form).
/// Returns `Err(SensorError::Other("strip_prefix_mismatch: ..."))` when strip_prefix fails
/// (dot form or other convention), making the test assertion observable.
struct MockAdapterStripPrefix {
    sensor_id: &'static str,
    resolved_table: std::sync::Mutex<Option<String>>,
}

impl MockAdapterStripPrefix {
    fn new(sensor_id: &'static str) -> Self {
        Self {
            sensor_id,
            resolved_table: std::sync::Mutex::new(None),
        }
    }

    /// Returns the table name resolved by strip_prefix, if the last fetch succeeded.
    fn resolved(&self) -> Option<String> {
        self.resolved_table
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
}

#[async_trait]
impl SensorAdapter for MockAdapterStripPrefix {
    fn sensor_type(&self) -> SensorId {
        SensorId::from(self.sensor_id)
    }

    async fn fetch(
        &self,
        spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Replicate SpecDrivenSensorAdapter::fetch strip_prefix logic (spec_driven_adapter.rs §647-649).
        let prefix = format!("{}_", self.sensor_id);
        match spec.source_table.strip_prefix(&prefix) {
            Some(table_name) => {
                // Underscore form matched — record resolved table name.
                *self
                    .resolved_table
                    .lock()
                    .unwrap_or_else(|p| p.into_inner()) = Some(table_name.to_owned());
                Ok(vec![])
            }
            None => {
                // source_table did NOT start with "{sensor_id}_" — the probe used the wrong form.
                // This would cause SpecDrivenSensorAdapter to fan-out to ALL declared tables
                // instead of routing to a single table.
                Err(SensorError::Internal {
                    detail: format!(
                        "strip_prefix_mismatch: source_table '{}' does not start with '{prefix}'",
                        spec.source_table
                    ),
                })
            }
        }
    }

    fn sensor_name(&self) -> &'static str {
        "mock-strip-prefix"
    }
}

/// F-S504-P1-002 integration: `probe_connectivity_with_routing` with `probe_table=Some("detections")`
/// produces `source_table = "crowdstrike_detections"` that passes `strip_prefix("crowdstrike_")`.
///
/// This test proves the underscore form is not just passed through (MockAdapterCapturingSpec)
/// but is CORRECT for the SpecDrivenSensorAdapter table selection path.
///
/// If the routing produced "crowdstrike.detections" (dot form), the mock would return
/// `SensorError::Other("strip_prefix_mismatch: ...")` and the probe would record Down,
/// causing this test to fail with a clear message.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_probe_table_underscore_form_resolves_via_strip_prefix() {
    let adapter = Arc::new(MockAdapterStripPrefix::new("crowdstrike"));
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    // Probe with probe_table = Some("detections").
    // probe_connectivity_with_routing MUST produce source_table = "crowdstrike_detections".
    let result = probe_connectivity_with_routing(
        &registry,
        org_id,
        &sensor_id,
        "acme",
        Some("detections"),
        None,
    )
    .await
    .expect("probe_connectivity_with_routing must not error");

    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "F-S504-P1-002 integration: probe must return Up — if Down, source_table used dot \
         form and strip_prefix returned None (fan-out to all tables instead of single table). \
         Probe outcome: {:?}",
        result.error
    );

    // The stand-in's resolved() field holds the table name that strip_prefix extracted.
    // It is Some("detections") when underscore form was used, None when strip_prefix found no match.
    assert_eq!(
        adapter.resolved(),
        Some("detections".to_owned()),
        "F-S504-P1-002 integration: strip_prefix MUST resolve to 'detections'. \
         None means source_table used dot form ('crowdstrike.detections'), which would \
         cause SpecDrivenSensorAdapter to fan-out to all tables instead of a single table."
    );
}

/// F-S504-P1-002 integration: `probe_connectivity_with_routing` with `probe_table=None`,
/// `first_table_name=Some("alerts")` produces `source_table = "cyberint_alerts"` that
/// passes `strip_prefix("cyberint_")`.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_probe_table_fallback_underscore_form_resolves_via_strip_prefix() {
    let adapter = Arc::new(MockAdapterStripPrefix::new("cyberint"));
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("cyberint");

    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    // probe_table absent, first declared table = "alerts".
    let result = probe_connectivity_with_routing(
        &registry,
        org_id,
        &sensor_id,
        "acme",
        None,
        Some("alerts"),
    )
    .await
    .expect("probe_connectivity_with_routing must not error");

    assert_eq!(
        result.status,
        ConnectivityStatus::Up,
        "F-S504-P1-002 integration (fallback): probe must return Up. Error: {:?}",
        result.error
    );

    assert_eq!(
        adapter.resolved(),
        Some("alerts".to_owned()),
        "F-S504-P1-002 integration (fallback): strip_prefix MUST resolve to 'alerts'. \
         None means dot form was used."
    );
}

// ─── F-S504-P2-RE-001 — check_one / new_with_spec_map end-to-end probe routing ─
//
// These tests close F-S504-P2-RE-001 (MED): the production chain
//   server.rs::with_deps
//     → SensorHealthChecker::new_with_spec_map(registry, Arc::new(spec_map))
//       → check_one(org_id, client_id, sensor_id, context)
//         → resolves resolved.spec.probe_table via spec_map.get
//           → probe_auth_with_routing(probe_table_ref, first_table_ref)
//             → probe_connectivity_inner → adapter.fetch(SensorSpec { source_table })
//
// Tests 1 + 2 prove that check_one correctly EXTRACTS probe_table / first_table_name
// from a wired ResolvedSensorSpec and forms the underscore source_table.
// Test 3 proves the OrgSlug key-miss degradation path: when client_id is not in
// the spec map, check_one falls back to "{sensor_id}_devices" (legacy sentinel).
//
// LOAD-BEARING: these tests fail if the (OrgSlug, SensorId) key construction in
// check_one is refactored incorrectly (e.g., key shape changed), or if the
// probe_table / tables field extraction logic is broken.
// ─────────────────────────────────────────────────────────────────────────────────

/// Helper: build a minimal `ResolvedSensorSpec` with the given probe_table and table list.
///
/// Construction path (avoids #[non_exhaustive] struct-literal restrictions for both
/// `SensorSpec` and `ResolvedSensorSpec`):
///   1. `SensorSpec::new()` for the base spec (no probe_table).
///   2. Direct field assignment `spec.probe_table = ...` (all SensorSpec fields are pub).
///   3. `TableSpec::new_point_in_time()` for each table (forward-compatible constructor).
///   4. `OverlayLoader::merge_overlay_onto_type_spec` to produce the `ResolvedSensorSpec`
///      (the canonical factory for this type, as used in prism-bin tests).
fn make_resolved_spec(
    sensor_id: &str,
    org_slug: OrgSlug,
    probe_table: Option<&str>,
    table_names: &[&str],
) -> ResolvedSensorSpec {
    let tables: Vec<TableSpec> = table_names
        .iter()
        .map(|name| {
            TableSpec::new_point_in_time(
                name.to_string(),
                "security_finding".to_string(),
                vec![],
                vec![],
            )
        })
        .collect();
    // SensorSpec::new does not accept probe_table; set it via direct field assignment
    // (all SensorSpec fields are pub — direct assignment avoids the non-exhaustive restriction).
    let mut spec = EngSensorSpec::new(
        sensor_id.to_string(),
        sensor_id.to_string(),
        EngAuthType::ApiKey,
        format!("https://{sensor_id}.example.com"),
        tables,
        None,
        "1.0.0",
        vec![],
    );
    spec.probe_table = probe_table.map(|s| s.to_string());

    // Use OverlayLoader::merge_overlay_onto_type_spec (the canonical factory — same path
    // as prism-bin tests) to produce a ResolvedSensorSpec without #[non_exhaustive] issues.
    let overlay_toml = format!(
        "extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"",
        org = org_slug.as_str()
    );
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("make_resolved_spec: SensorInstanceOverlay TOML parse failed");
    OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug)
}

/// F-S504-P2-RE-001 (1/3): `check_one` via `new_with_spec_map` with `probe_table = Some("detections")`.
///
/// Constructs `SensorHealthChecker::new_with_spec_map` with a spec_map containing a
/// `ResolvedSensorSpec` for `(acme, crowdstrike)` with `probe_table = Some("detections")`.
/// Calls `check_one` — the PRODUCTION entry point — and asserts the
/// `MockAdapterStripPrefix` received `source_table == "crowdstrike_detections"`.
///
/// LOAD-BEARING: fails if:
/// - `check_one` does not look up the spec_map (bypasses spec lookup)
/// - The `(OrgSlug, SensorId)` key shape changes without updating `check_one`
/// - `probe_table` extraction from `resolved.spec.probe_table` is broken
/// - The underscore form (not dot form) is no longer used
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_check_one_routes_to_probe_table_via_spec_map() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme");
    let sensor_id = SensorId::from("crowdstrike");

    // Build spec_map: (acme, crowdstrike) → probe_table = Some("detections")
    let resolved = make_resolved_spec(
        "crowdstrike",
        org_slug.clone(),
        Some("detections"),
        &["detections", "incidents"],
    );
    let key: ResolvedSpecKey = (org_slug, sensor_id.clone());
    let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    spec_map.insert(key, resolved);

    // Wire the MockAdapterStripPrefix — it returns Err if source_table doesn't start with
    // "crowdstrike_", making the assertion observable via check_one's result.
    let adapter = Arc::new(MockAdapterStripPrefix::new("crowdstrike"));
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    let checker = SensorHealthChecker::new_with_spec_map(Arc::new(registry), Arc::new(spec_map));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // The probe must succeed (Up): MockAdapterStripPrefix returns Ok when strip_prefix matches.
    // If the routing produced dot form "crowdstrike.detections", strip_prefix("crowdstrike_")
    // would fail → SensorError::Internal → probe records Down here.
    assert_eq!(
        result.reachable,
        Some(true),
        "F-S504-P2-RE-001 (1/3): check_one with probe_table=Some('detections') MUST record \
         reachable=true. Down means source_table used wrong form (not 'crowdstrike_detections'). \
         Got: {:?}",
        result.reachable
    );

    // The table resolved by strip_prefix must be "detections" (not "detections" of dot form).
    assert_eq!(
        adapter.resolved(),
        Some("detections".to_owned()),
        "F-S504-P2-RE-001 (1/3): MockAdapterStripPrefix MUST resolve table 'detections' via \
         strip_prefix('crowdstrike_'). None means source_table used dot form \
         'crowdstrike.detections' instead of 'crowdstrike_detections'. \
         check_one probe_table extraction broken."
    );
}

/// F-S504-P2-RE-001 (2/3): `check_one` via `new_with_spec_map` with `probe_table = None`,
/// tables = ["alerts", "incidents"] → fallback to first table "alerts".
///
/// Asserts the adapter received `source_table == "crowdstrike_alerts"`.
///
/// LOAD-BEARING: fails if:
/// - `first_table_name` extraction from `resolved.spec.tables.first()` is broken
/// - The fallback chain does not use the first declared table when probe_table is None
/// - The underscore form is not used in the fallback path
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_check_one_falls_back_to_first_table_via_spec_map() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme");
    let sensor_id = SensorId::from("crowdstrike");

    // probe_table = None, tables[0] = "alerts"
    let resolved = make_resolved_spec(
        "crowdstrike",
        org_slug.clone(),
        None,
        &["alerts", "incidents"],
    );
    let key: ResolvedSpecKey = (org_slug, sensor_id.clone());
    let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    spec_map.insert(key, resolved);

    let adapter = Arc::new(MockAdapterStripPrefix::new("crowdstrike"));
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    let checker = SensorHealthChecker::new_with_spec_map(Arc::new(registry), Arc::new(spec_map));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    assert_eq!(
        result.reachable,
        Some(true),
        "F-S504-P2-RE-001 (2/3): check_one with probe_table=None, tables=['alerts','incidents'] \
         MUST record reachable=true (fallback to first table 'alerts' → source_table \
         'crowdstrike_alerts' passes strip_prefix). Down means first_table_name not extracted \
         or wrong form used. Got: {:?}",
        result.reachable
    );

    assert_eq!(
        adapter.resolved(),
        Some("alerts".to_owned()),
        "F-S504-P2-RE-001 (2/3): strip_prefix MUST resolve to 'alerts'. None means \
         first_table_name was not extracted from spec.tables[0] or dot form was used."
    );
}

// ─── RED GATE HELPER — rate-limited SensorHealthResult ────────────────────────

/// Build a `SensorHealthResult` simulating a rate-limited sensor (HTTP 429).
///
/// Matches the output of `check_one` when `SensorError::RateLimited` is observed:
/// - `reachable = Some(true)` (sensor responded — Up, not Down)
/// - `auth_valid = Some(true)` (no auth failure observed)
/// - `rate_limit = Some(RateLimitInfo { ... })` (rate-limit state populated)
///
/// Uses `RateLimitInfo::with_reset_at` builder (avoids `#[non_exhaustive]` struct-literal
/// restriction in external test crates). Used by EC-007 aggregate tests.
fn rate_limited_result(sensor_id: &str, client_id: &str) -> SensorHealthResult {
    let mut r = SensorHealthResult::new(sensor_id, client_id)
        .with_reachable(true)
        .with_auth_valid(true);
    r.rate_limit = Some(RateLimitInfo::new_with_reset_at(
        Utc::now() + ChronoDuration::seconds(60),
    ));
    r
}

/// F-S504-P2-RE-001 (3/3): OrgSlug key-miss degradation — `client_id` not in spec_map
/// → `check_one` falls back to the legacy `"{sensor_id}_devices"` sentinel.
///
/// The spec_map contains a resolved spec keyed by `(acme, crowdstrike)`. `check_one` is
/// called with `client_id = "unknown-org"` (not in the map). The fallback path must use
/// `probe_table = None, first_table_name = None` → `"crowdstrike_devices"`.
///
/// This test documents and locks the intended degradation: a client_id that does not have
/// a matching spec entry gets a hollow probe (no specific table routing). It asserts this
/// loudly so a future key-shape refactor (e.g., changing OrgSlug construction in check_one)
/// fails loudly rather than silently producing wrong routing.
///
/// LOAD-BEARING: fails if:
/// - The OrgSlug/SensorId key shape changes and check_one incorrectly matches non-existent orgs
/// - The fallback path does not produce "{sensor_id}_devices" when spec_map.get returns None
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_check_one_falls_back_to_devices_when_org_not_in_spec_map() {
    let org_id = OrgId::new();
    // Spec map only knows "acme" — the probe is for "unknown-org".
    let known_slug = OrgSlug::new("acme");
    let sensor_id = SensorId::from("crowdstrike");

    let resolved = make_resolved_spec(
        "crowdstrike",
        known_slug.clone(),
        Some("detections"),
        &["detections"],
    );
    let key: ResolvedSpecKey = (known_slug, sensor_id.clone());
    let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    spec_map.insert(key, resolved);

    // MockAdapterCapturingSpec: captures the source_table verbatim (no strip_prefix validation).
    // Used here because the legacy sentinel "crowdstrike_devices" would cause
    // MockAdapterStripPrefix to resolve "devices" (valid underscore form but wrong table),
    // making the assertion harder to read. We want to directly assert the sentinel value.
    let adapter = Arc::new(MockAdapterCapturingSpec::new("crowdstrike"));
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::clone(&adapter) as Arc<dyn SensorAdapter>);

    let checker = SensorHealthChecker::new_with_spec_map(Arc::new(registry), Arc::new(spec_map));
    let context = PrismContext::new();

    // Call with "unknown-org" — NOT in the spec_map keyed by (acme, crowdstrike).
    let _result = checker
        .check_one(org_id, "unknown-org", &sensor_id, &context)
        .await;

    let captured = adapter
        .captured()
        .expect("adapter must have been called even for key-miss path");

    assert_eq!(
        captured, "crowdstrike_devices",
        "F-S504-P2-RE-001 (3/3): when client_id='unknown-org' is not in spec_map, \
         check_one MUST fall back to legacy sentinel 'crowdstrike_devices'. \
         Got '{captured}'. If this breaks, the OrgSlug key construction or the \
         None-spec fallback path in check_one was changed."
    );
}

// ─── EC-007 / F-S504-P5-001 — all-rate-limited aggregate (RED GATE) ─────────

/// EC-007 (BC-2.08.007 / F-S504-P5-001): When ALL sensors are rate-limited
/// (`rate_limit.is_some()`) and none are unreachable or auth-invalid,
/// `HealthCheckResult::aggregate` MUST return `OverallStatus::RateLimited`.
///
/// BC-2.08.007 postcondition: `"rate_limited"` — ALL sensors are rate-limited,
/// none unreachable or auth-invalid. NOT `"partial"` (which implies connectivity/auth failure
/// requiring different remediation). NOT `"unhealthy"` (sensors ARE reachable/auth-valid).
///
/// RED GATE: `OverallStatus::RateLimited` variant does not exist yet → compile error / assert fails.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_08_007_EC_007_all_rate_limited_aggregate_yields_rate_limited() {
    let results = vec![
        rate_limited_result("crowdstrike", "acme"),
        rate_limited_result("armis", "acme"),
        rate_limited_result("claroty", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    assert_eq!(
        status,
        OverallStatus::RateLimited,
        "EC-007 (BC-2.08.007 / F-S504-P5-001): when ALL sensors are rate-limited, \
         aggregate MUST return OverallStatus::RateLimited (not Partial, not Unhealthy). \
         Got: {status:?}"
    );
}

/// EC-007 boundary: mixed (some rate-limited + some unreachable/down) → `Partial`, not `RateLimited`.
///
/// The `RateLimited` aggregate only applies when ALL sensors are rate-limited AND
/// NONE are unreachable or auth-invalid. A mix falls into `Partial`.
///
/// BC-2.08.007 postcondition:
/// `"partial"` — at least one sensor is unreachable or auth-invalid (regardless of RL state on others).
///
/// RED GATE: `OverallStatus::RateLimited` variant does not exist yet → compile error.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_08_007_EC_007_mixed_rate_limited_and_down_is_partial() {
    // One sensor is rate-limited (reachable=true, auth_valid=true, rate_limit=Some(...))
    // One sensor is down (reachable=false, auth_valid=false) — connectivity failure.
    let results = vec![
        rate_limited_result("crowdstrike", "acme"),
        down_result("armis", "acme"),
    ];
    let status = HealthCheckResult::aggregate(results);
    assert_eq!(
        status,
        OverallStatus::Partial,
        "EC-007 boundary: mixed rate-limited + down MUST be Partial (connectivity failure \
         present). Got: {status:?}"
    );
}

// ─── F-S504-P5-002 — response-shape postconditions (RED GATE) ─────────────────

/// F-S504-P5-002 (BC-2.08.007): `SensorHealthStructuredContent` MUST carry:
///   1. `overall_status: String` — serialized aggregate status (e.g., `"rate_limited"`).
///   2. `summary_counts` object — `healthy_count`, `unhealthy_count`, `total_count` (integers).
///   3. Per-sensor `suggestion: Option<String>` on unhealthy/rate-limited sensors.
///
/// This test exercises the full production path via `check_sensor_health` response JSON
/// (not just `aggregate()` in isolation — TD-VSDD-059 load-bearing requirement).
///
/// RED GATE (1): `SensorHealthStructuredContent.overall_status` field does not exist yet.
/// RED GATE (2): `SensorHealthStructuredContent.summary_counts` field does not exist yet.
/// RED GATE (3): `SensorHealthResult.suggestion` field does not exist yet.
///
/// When all sensors in the response are rate-limited (EC-08-015):
/// - `overall_status` == `"rate_limited"`
/// - `summary_counts.healthy_count` == 0
/// - `summary_counts.unhealthy_count` == N (all sensors)
/// - `summary_counts.total_count` == N
/// - each sensor entry has `suggestion: "Rate limit in effect — wait before retrying."`
///   (verbatim per BC-2.08.007 / EC-08-015 / EC-007 in S-5.04)
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_007_EC_007_response_shape_overall_status_summary_counts_suggestion() {
    // ── Build a server with health_checker wired + 2 rate-limited mock sensors ──
    // This exercises the PRODUCTION check_sensor_health path (not just aggregate()).
    // Two sensors: both are rate-limited (MockAdapterRateLimited + MockAdapterRateLimitedArmis).
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Register two sensors under the same org_id with rate-limited adapters.
    let org_id = OrgId::new();

    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(MockAdapterRateLimited));
    adapter_registry.register(org_id, Arc::new(MockAdapterRateLimitedArmis));
    let adapter_registry = Arc::new(adapter_registry);

    // Build a TableRegistry with both sensors registered so check_sensor_health
    // enumerates them (single-tenant fallback path: table_registry.registered_sensor_ids()).
    let table_registry = TableRegistry::new();
    let cs_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "crowdstrike",
        "CrowdStrike (rate-limited mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    let armis_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "armis",
        "Armis (rate-limited mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.armis.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    table_registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike");
    table_registry
        .register_sensor(&armis_spec)
        .expect("register armis");

    // Build a QueryEngine wired with the table_registry (for sensor ID enumeration).
    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    // Build SensorHealthChecker with the populated adapter registry.
    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    // Build a PrismServer with the health_checker + query_engine wired.
    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    // ── Build params: client_id = "acme", sensor_id = None (all sensors) ─────
    let params = CheckSensorHealthParams::for_client("acme");

    let call_result = server
        .check_sensor_health(Parameters(params))
        .await
        .expect("F-S504-P5-002: check_sensor_health MUST succeed for all-rate-limited sensors");

    // The simplest extraction: serialize CallToolResult to JSON and check for key-value pairs.
    // CallToolResult::structured() embeds the structured value in the response JSON.
    let json_str = serde_json::to_string(&call_result)
        .expect("F-S504-P5-002: CallToolResult must serialize to JSON");

    // ── Assert overall_status == "rate_limited" ─────────────────────────────
    // structuredContent field in CallToolResult JSON (rmcp CallToolResult shape).
    assert!(
        json_str.contains(r#""overall_status":"rate_limited""#)
            || json_str.contains(r#""overall_status": "rate_limited""#),
        "F-S504-P5-002 (RED GATE 1 — F-S504-P5-001): \
         `structuredContent.overall_status` MUST be \"rate_limited\" when all sensors are rate-limited. \
         BC-2.08.007 EC-08-015: all-rate-limited MUST NOT be reported as Partial or Healthy. \
         Got JSON: {:.500}",
        json_str
    );

    // ── Assert summary_counts present with correct values ───────────────────
    assert!(
        json_str.contains(r#""healthy_count":0"#) || json_str.contains(r#""healthy_count": 0"#),
        "F-S504-P5-002 (RED GATE 2): \
         `summary_counts.healthy_count` MUST be 0 when all sensors are rate-limited. \
         Got JSON: {:.500}",
        json_str
    );
    assert!(
        json_str.contains(r#""total_count":2"#) || json_str.contains(r#""total_count": 2"#),
        "F-S504-P5-002 (RED GATE 2): \
         `summary_counts.total_count` MUST be 2 (two sensors probed). \
         Got JSON: {:.500}",
        json_str
    );
    assert!(
        json_str.contains(r#""unhealthy_count":2"#) || json_str.contains(r#""unhealthy_count": 2"#),
        "F-S504-P5-002 (RED GATE 2): \
         `summary_counts.unhealthy_count` MUST be 2 when all sensors are rate-limited. \
         Got JSON: {:.500}",
        json_str
    );

    // ── Assert per-sensor suggestion == verbatim BC text ────────────────────
    // BC-2.08.007 EC-08-015: suggestion for rate-limited sensor:
    // "Rate limit in effect — wait before retrying." (verbatim, em-dash U+2014)
    let expected_suggestion = "Rate limit in effect \u{2014} wait before retrying.";
    assert!(
        json_str.contains(expected_suggestion),
        "F-S504-P5-002 (RED GATE 3): \
         each rate-limited sensor entry MUST carry suggestion: \"{expected_suggestion}\" \
         (verbatim per BC-2.08.007 EC-08-015). \
         Got JSON: {:.500}",
        json_str
    );

    // ── Assert prose summary matches AC-11 verbatim ─────────────────────────
    // AC-11 (BC-2.08.007 EC-08-015): prose summary is
    // "0 of N sensors healthy for client 'X' — all rate-limited"
    // where N = sensor count, X = client identifier, — = U+2014 em-dash.
    // OBS-1 fix: assert the FULL verbatim string (em-dash + "for client 'acme'" clause),
    // not just fragment subsets that would miss a regression dropping the client-id clause
    // or swapping the em-dash for an ASCII hyphen.
    let expected_summary = "0 of 2 sensors healthy for client 'acme' \u{2014} all rate-limited";
    assert!(
        json_str.contains(expected_summary),
        "F-S504-P5-002 (OBS-1 tightened): prose summary MUST be verbatim AC-11 string: \
         \"{expected_summary}\" (em-dash U+2014, 'for client X' clause). \
         Got JSON: {:.500}",
        json_str
    );
}

// ─── F-S504-LP3-HIGH-001 — all-auth-invalid server response shape ─────────────

/// F-S504-LP3-HIGH-001 (BC-2.08.007): `check_sensor_health` for an all-401 client MUST
/// emit `overall_status: "unhealthy"` (NOT `"partial"`).
///
/// Canonical test-vector: "All sensors unhealthy (unreachable/auth-invalid) → overall_status: 'unhealthy'".
///
/// RED GATE (pre-fix): returned `"partial"` due to the `any_reachable` predicate bug.
/// After fix, the all-auth-invalid fleet MUST produce (GREEN regression guard on current HEAD):
/// - `overall_status: "unhealthy"`
/// - `summary_counts.unhealthy_count == 2`
/// - `summary_counts.healthy_count == 0`
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_007_all_auth_invalid_server_response_is_unhealthy() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Register two sensors both returning 401 (all-auth-invalid fleet).
    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(MockAdapterUnauthorized));
    adapter_registry.register(org_id, Arc::new(MockAdapterUnauthorizedArmis));
    let adapter_registry = Arc::new(adapter_registry);

    // Build a TableRegistry with both sensors so check_sensor_health enumerates them.
    let table_registry = TableRegistry::new();
    let cs_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "crowdstrike",
        "CrowdStrike (all-401 mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    let armis_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "armis",
        "Armis (all-401 mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.armis.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    table_registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike");
    table_registry
        .register_sensor(&armis_spec)
        .expect("register armis");

    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    let params = CheckSensorHealthParams::for_client("acme");
    let call_result = server.check_sensor_health(Parameters(params)).await.expect(
        "F-S504-LP3-HIGH-001: check_sensor_health MUST succeed for all-auth-invalid sensors",
    );

    let json_str = serde_json::to_string(&call_result)
        .expect("F-S504-LP3-HIGH-001: CallToolResult must serialize to JSON");

    // overall_status MUST be "unhealthy" — NOT "partial"
    assert!(
        json_str.contains(r#""overall_status":"unhealthy""#)
            || json_str.contains(r#""overall_status": "unhealthy""#),
        "F-S504-LP3-HIGH-001 (BC-2.08.007 canonical test-vector): \
         all sensors auth-invalid MUST produce overall_status=\"unhealthy\" (not \"partial\"). \
         Canonical test-vector: 'All sensors unhealthy (unreachable/auth-invalid) → \
         overall_status: unhealthy'. Got JSON: {:.500}",
        json_str
    );

    // summary_counts.unhealthy_count == 2, healthy_count == 0
    assert!(
        json_str.contains(r#""unhealthy_count":2"#) || json_str.contains(r#""unhealthy_count": 2"#),
        "F-S504-LP3-HIGH-001: summary_counts.unhealthy_count MUST be 2 for all-auth-invalid fleet. \
         Got JSON: {:.500}",
        json_str
    );
    assert!(
        json_str.contains(r#""healthy_count":0"#) || json_str.contains(r#""healthy_count": 0"#),
        "F-S504-LP3-HIGH-001: summary_counts.healthy_count MUST be 0 for all-auth-invalid fleet. \
         Got JSON: {:.500}",
        json_str
    );
}

// ─── EC-08-009 / HS-007 — HTTP 5xx/Degraded → reachable=true (network-reachable, erroring) ─────

/// EC-08-001 (BC-2.08.001): HTTP 503 during health probe → `ConnectivityStatus::Degraded`.
///
/// `probe_connectivity` already maps status >= 500 to `Degraded`.  This test verifies
/// that mapping at the raw connectivity layer so the `check_one` test below can rely on
/// the input condition being correct.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_live_probe_503_connectivity_is_degraded() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterServiceUnavailable));

    let outcome = probe_connectivity(&registry, org_id, &sensor_id, "acme")
        .await
        .expect("BC-2.08.001 EC-08-001: probe_connectivity must return Ok for HTTP 503");

    assert_eq!(
        outcome.status,
        ConnectivityStatus::Degraded,
        "BC-2.08.001 EC-08-001: HTTP 503 must yield ConnectivityStatus::Degraded. Got: {:?}",
        outcome.status
    );
    assert_eq!(
        outcome.http_status,
        Some(503),
        "BC-2.08.001 EC-08-001: http_status must be Some(503). Got: {:?}",
        outcome.http_status
    );
}

/// EC-08-009 (BC-2.08.002 / HS-007): HTTP 503 → `SensorHealthResult.reachable == Some(true)`.
///
/// HS-007 load-bearing test (TD-VSDD-059):
/// `check_one` with a `MockAdapterServiceUnavailable` (HTTP 503) MUST produce
/// `reachable = Some(true)` — the sensor IS network-reachable (returned HTTP response),
/// it is just erroring (5xx).
///
/// CORRECTION of F-S504-LP3P5-HIGH-001:
/// The OLD behavior was `reachable = probe.connectivity == ConnectivityStatus::Up`,
/// which yielded `reachable=Some(false)` for `Degraded` (503) — a FALSE-NEGATIVE.
///
/// The CORRECT behavior per EC-08-009:
///   `let reachable = probe.connectivity != ConnectivityStatus::Down;`
/// so that `Degraded` (5xx) → `reachable = true` (reachable, erroring, distinguishable from Down).
/// Only `Down` (no HTTP exchange) → `reachable = false`.
///
/// The sensor is prevented from counting as fully-healthy by the `r.error.is_none()` gate
/// in `HealthCheckResult::aggregate` (HS-007 edit 2): error="service_unavailable" is set
/// for Degraded, so `fully_healthy_count` correctly excludes 503 sensors.
///
/// Additional assertions:
/// - `last_successful_query_at` MUST be `None` (HS-007 edit 3: `Up`-only guard prevents
///   false success timestamp for Degraded probes).
/// - `auth_valid` is `Some(true)` (503 is not an auth error — sensor returned an HTTP response).
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_EC_08_001_503_probe_yields_reachable_true() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterServiceUnavailable));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // BC-2.08.002 EC-08-009 (HS-007): HTTP 503 MUST yield reachable=true.
    // Degraded != Down → network-reachable (returned HTTP response), only Down → false.
    assert_eq!(
        result.reachable,
        Some(true),
        "BC-2.08.002 EC-08-009 (HS-007): HTTP 503 MUST yield reachable=Some(true). \
         ConnectivityStatus::Degraded (5xx) means the sensor IS network-reachable but \
         erroring — distinct from Down (no HTTP exchange). Got: {:?}.",
        result.reachable
    );

    // HS-007 edit 3: MUST NOT record a successful-query timestamp for a 5xx (Degraded) probe.
    // The `probe.connectivity == Up` gate in check_one prevents false success timestamps.
    assert_eq!(
        result.last_successful_query_at, None,
        "HS-007 edit 3: HTTP 503 (Degraded) probe MUST NOT record last_successful_query_at \
         (probe.connectivity != Up → timestamp gate skips recording). Got: {:?}",
        result.last_successful_query_at
    );

    // ADV-4 (SID-2 wire-shape): assert "reachable":true at the wire level, not just struct level.
    // Guards against a serde regression (e.g., skip_serializing_if or renamed field) that would
    // cause the key to be absent or wrong in the JSON consumed by LLM agents.
    let json =
        serde_json::to_string(&result).expect("ADV-4: SensorHealthResult must serialize to JSON");
    assert!(
        json.contains("\"reachable\":true"),
        "ADV-4 (SID-2 wire-shape): JSON MUST contain '\"reachable\":true' for HTTP 503 \
         (ConnectivityStatus::Degraded) at the wire level. \
         EC-08-009 (HS-007): Degraded != Down → reachable=Some(true). Got JSON: {json}"
    );
}

/// HS-007 / EC-08-009 (aggregate path): an all-503 sensor fleet MUST produce
/// `OverallStatus::Partial` via `HealthCheckResult::aggregate`.
///
/// After HS-007 edit 1 (`reachable = probe.connectivity != Down`):
/// - Degraded (5xx) → `reachable=Some(true)` — sensor IS network-reachable.
/// - `error="service_unavailable"` is set for Degraded sensors.
///
/// After HS-007 edit 2 (`r.error.is_none()` gate in `fully_healthy_count`):
/// - `fully_healthy_count == 0` (error is Some → excluded from fully-healthy).
/// - `any_partially_available == true` (reachable != Some(false) && auth_valid != Some(false)).
/// - `aggregate` → `OverallStatus::Partial`.
///
/// Before HS-007, the old fix (`reachable = connectivity == Up`) would have yielded
/// `reachable=Some(false)` for each 503 → `Unhealthy`. But that was a false-negative:
/// the sensor IS reachable (returned HTTP), it's just erroring. `Partial` is correct:
/// sensors are reachable but not serving useful data.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_EC_08_001_all_503_fleet_aggregate_partial() {
    let org_id = OrgId::new();
    let sensor_id_cs = SensorId::from("crowdstrike");
    let sensor_id_armis = SensorId::from("armis");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterServiceUnavailable));
    registry.register(org_id, Arc::new(MockAdapterServiceUnavailableArmis));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result_cs = checker
        .check_one(org_id, "acme", &sensor_id_cs, &context)
        .await;
    let result_armis = checker
        .check_one(org_id, "acme", &sensor_id_armis, &context)
        .await;

    // Both sensors must be reachable=true (EC-08-009: Degraded → network-reachable).
    assert_eq!(
        result_cs.reachable,
        Some(true),
        "EC-08-009 (HS-007): crowdstrike 503 probe must yield reachable=true"
    );
    assert_eq!(
        result_armis.reachable,
        Some(true),
        "EC-08-009 (HS-007): armis 503 probe must yield reachable=true"
    );

    // ADV-4 (SID-2 wire-shape): assert reachable:true and error:service_unavailable at wire level
    // for both sensors, BEFORE aggregate() consumes (moves) the values.
    // Guards against a serde regression that would emit wrong field names/values in the wire.
    let json_cs =
        serde_json::to_string(&result_cs).expect("ADV-4: result_cs must serialize to JSON");
    assert!(
        json_cs.contains("\"reachable\":true"),
        "ADV-4 (SID-2 wire-shape): crowdstrike 503 JSON MUST contain '\"reachable\":true'. \
         Got JSON: {json_cs}"
    );
    assert!(
        json_cs.contains("\"error\":\"service_unavailable\""),
        "ADV-4 (SID-2 wire-shape): crowdstrike 503 JSON MUST contain \
         '\"error\":\"service_unavailable\"'. Got JSON: {json_cs}"
    );
    let json_armis =
        serde_json::to_string(&result_armis).expect("ADV-4: result_armis must serialize to JSON");
    assert!(
        json_armis.contains("\"reachable\":true"),
        "ADV-4 (SID-2 wire-shape): armis 503 JSON MUST contain '\"reachable\":true'. \
         Got JSON: {json_armis}"
    );

    // Aggregate of all-503 fleet → Partial (reachable=true, error set → not fully-healthy).
    // fully_healthy_count=0 (error.is_none() gate); any_partially_available=true (reachable=true).
    let overall = HealthCheckResult::aggregate(vec![result_cs, result_armis]);
    assert_eq!(
        overall,
        OverallStatus::Partial,
        "HS-007 / EC-08-009: an all-503 sensor fleet MUST aggregate to \
         OverallStatus::Partial. Sensors ARE network-reachable (returned HTTP 503) but \
         erroring — r.error.is_none() gate excludes them from fully_healthy; \
         reachable=true makes them partially_available. Got: {overall:?}"
    );
}

// ─── F-S504-LP1P1-MED-001 — EC-08-001 reason string + degraded suggestion ────

/// EC-08-009 (BC-2.08.002 / HS-007): HTTP 503 probe MUST set `result.error = Some("service_unavailable")`.
///
/// BC-2.08.002 EC-08-009 canonical contract:
///   "Health status reports `reachable: true` (network-reachable), `error: "service_unavailable"`"
///
/// The `error` field is carried in `SensorHealthResult.error`. This is the mechanism that
/// distinguishes Degraded (5xx: reachable=true, error set) from fully-healthy (reachable=true,
/// error=None), preventing false-positive `fully_healthy_count` via the `r.error.is_none()` gate
/// (HS-007 edit 2). The `service_unavailable` string is the BC canonical value.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_EC_08_001_503_probe_sets_service_unavailable_reason() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterServiceUnavailable));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // Prerequisite: reachable=true (EC-08-009: Degraded → network-reachable; already tested above).
    assert_eq!(
        result.reachable,
        Some(true),
        "EC-08-009 (HS-007) prerequisite: HTTP 503 must yield reachable=Some(true)"
    );

    // PRIMARY assertion (F-S504-LP1P1-MED-001 part 1):
    // BC-2.08.002 EC-08-009 requires error="service_unavailable" for Degraded (5xx) probes.
    // The error field in SensorHealthResult is what the `r.error.is_none()` gate reads.
    assert_eq!(
        result.error.as_deref(),
        Some("service_unavailable"),
        "BC-2.08.002 EC-08-009 (HS-007): HTTP 503 probe MUST set \
         result.error = Some(\"service_unavailable\"). \
         This is the BC canonical value for Degraded (5xx) sensors, and it gates the \
         fully_healthy_count predicate (r.error.is_none() = false → not fully-healthy). \
         Current: result.error = {:?}",
        result.error
    );

    // ADV-4 (SID-2 wire-shape): assert "error":"service_unavailable" at the wire level.
    // Guards against a serde regression where the field is present in the struct but absent
    // or differently named in the serialized JSON consumed by LLM agents.
    let json =
        serde_json::to_string(&result).expect("ADV-4: SensorHealthResult must serialize to JSON");
    assert!(
        json.contains("\"error\":\"service_unavailable\""),
        "ADV-4 (SID-2 wire-shape): JSON MUST contain '\"error\":\"service_unavailable\"' \
         for HTTP 503 (Degraded) at the wire level. Got JSON: {json}"
    );
}

// ─── BC-2.08.002 EC-08-005 — Down reason string ───────────────────────────────

/// BC-2.08.002 EC-08-005: A genuinely-unreachable sensor (connection refused / timeout)
/// MUST set `result.error = Some("sensor_unreachable_cannot_verify")` through `check_one`.
///
/// BC-2.08.002 EC-08-005 canonical contract:
///   "Sensor API is unreachable (auth cannot be verified) →
///    `auth_valid: null` with `reason: "sensor_unreachable_cannot_verify"`"
///
/// The `reason` is carried in `SensorHealthResult.error`.  The previous implementation
/// did NOT set this — `result.error` stayed `None` for Down sensors, leaving AI consumers
/// with no machine-readable reason string to distinguish "unreachable" from "no error recorded".
///
/// RED GATE (TD-VSDD-059): removing the `probe.connectivity == Down` branch in check_one
/// that calls `with_error("sensor_unreachable_cannot_verify")` causes this assertion to fail.
///
/// Distinctness from 503: `service_unavailable` and `sensor_unreachable_cannot_verify` are
/// mutually exclusive — Degraded (5xx, HTTP response received) sets the former; Down (no
/// HTTP exchange) sets the latter.  The 503 test above covers the Degraded arm separately.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_002_EC_08_005_down_check_one_sets_sensor_unreachable_cannot_verify_reason() {
    let org_id = OrgId::new();
    let sensor_id = SensorId::from("crowdstrike");
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(MockAdapterConnectionRefused));

    let checker = SensorHealthChecker::new(Arc::new(registry));
    let context = PrismContext::new();

    let result = checker
        .check_one(org_id, "acme", &sensor_id, &context)
        .await;

    // Prerequisite: reachable=false, auth_valid=None (from prior fix).
    assert_eq!(
        result.reachable,
        Some(false),
        "BC-2.08.002 EC-08-005 prerequisite: Down sensor must yield reachable=Some(false)"
    );
    assert_eq!(
        result.auth_valid,
        None,
        "BC-2.08.002 EC-08-005 prerequisite: Down sensor must yield auth_valid=None (never attempted)"
    );

    // PRIMARY assertion (BC-2.08.002 EC-08-005):
    // reason="sensor_unreachable_cannot_verify" must be set in result.error.
    assert_eq!(
        result.error.as_deref(),
        Some("sensor_unreachable_cannot_verify"),
        "BC-2.08.002 EC-08-005: Down/timeout sensor MUST set \
         result.error = Some(\"sensor_unreachable_cannot_verify\"). \
         The BC-2.08.002 EC-08-005 canonical contract specifies this reason string \
         for sensors that were unreachable (auth could not be verified). \
         Current: result.error = {:?}",
        result.error
    );

    // Distinctness: "service_unavailable" MUST NOT appear for a Down sensor.
    // A timeout never produces a 5xx HTTP response — it never reached the server.
    assert_ne!(
        result.error.as_deref(),
        Some("service_unavailable"),
        "BC-2.08.002 EC-08-005 distinctness: Down sensor MUST NOT carry \
         reason=\"service_unavailable\" (that is for HTTP 503 Degraded, not Down). \
         Current: result.error = {:?}",
        result.error
    );
}

/// EC-08-001 + BC-2.08.007: The 503-Degraded suggestion fires through the REAL production
/// handler — not through a test-local duplicate of the ladder logic.
///
/// F-S504-LP1P2-MED-001 (fix): previous coverage used `apply_suggestion_ladder`, a
/// test-local duplicate of server.rs logic.  Reverting/collapsing the server.rs
/// `service_unavailable` branch would leave that test green while the production path
/// silently regressed.  This test exercises `PrismServer::check_sensor_health` directly
/// (same pattern as `test_BC_2_08_007_EC_007_response_shape_overall_status_summary_counts_suggestion`).
///
/// Production path for HTTP 503:
///   `MockAdapterServiceUnavailable` → `SensorError::HttpError { status: 503 }`
///   → `probe_connectivity` → `ConnectivityStatus::Degraded`
///   → `check_one` → `reachable=false`, `auth_valid=true` (503 is not 401/403),
///                    `error=Some("service_unavailable")`
///   → `server.rs` suggestion ladder → `reachable == Some(false)` +
///     `error == "service_unavailable"` → "Sensor returned a server error (5xx) …"
///
/// RED GATE: if the server.rs `service_unavailable` branch is reverted or collapsed
/// into the generic "verify network" fallback, the first assertion below fails.
///
/// A second sensor (`MockAdapterServiceUnavailableArmis`, also 503) confirms the 5xx
/// branch fires for both, and that "verify network" is absent from the entire response —
/// proving the 5xx branch is distinct from the Down/unreachable branch.
///
/// Verbatim BC-2.08.007 EC-08-015 suggestion:
/// "Sensor returned a server error (5xx) \u{2014} service may be temporarily unavailable."
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_001_EC_08_001_503_production_path_suggestion_distinct_from_down() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Register two 503 sensors (crowdstrike + armis) — both trigger the 5xx branch.
    // Using two distinct sensors proves the branch fires for sensor-type-agnostic cases.
    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(MockAdapterServiceUnavailable));
    adapter_registry.register(org_id, Arc::new(MockAdapterServiceUnavailableArmis));
    let adapter_registry = Arc::new(adapter_registry);

    // TableRegistry — enumerate both sensors in check_sensor_health.
    let table_registry = TableRegistry::new();
    let cs_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "crowdstrike",
        "CrowdStrike (503-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    let armis_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "armis",
        "Armis (503-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.armis.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    table_registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike");
    table_registry
        .register_sensor(&armis_spec)
        .expect("register armis");

    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    let params = CheckSensorHealthParams::for_client("acme");
    let call_result = server
        .check_sensor_health(Parameters(params))
        .await
        .expect("F-S504-LP1P2-MED-001: check_sensor_health MUST succeed");

    let json_str = serde_json::to_string(&call_result)
        .expect("F-S504-LP1P2-MED-001: CallToolResult must serialize to JSON");

    // ── RED GATE (primary): 503 sensors MUST get the 5xx-specific suggestion ─────────────
    // This assertion FAILS if the server.rs `service_unavailable` branch is reverted.
    // If reverted: 503 sensors have reachable=false + auth_valid=true + no service_unavailable
    // error marker → fall to the "Sensor unreachable — verify network" fallback.
    // (Actually check_one always sets error="service_unavailable" for Degraded; the revert
    //  would need to remove both the server.rs branch AND the check_one assignment.)
    // Minimum revert that breaks this test: remove the `service_unavailable` check in server.rs
    // so all reachable=false sensors get "verify network" — then this assert fails because
    // "verify network" appears instead of the 5xx string.
    let expected_5xx_suggestion =
        "Sensor returned a server error (5xx) \u{2014} service may be temporarily unavailable.";
    assert!(
        json_str.contains(expected_5xx_suggestion),
        "F-S504-LP1P2-MED-001 (RED GATE — production handler): \
         a 503-Degraded sensor MUST carry suggestion: \"{expected_5xx_suggestion}\" \
         (verbatim per BC-2.08.007 EC-08-015) through PrismServer::check_sensor_health. \
         If the server.rs `service_unavailable` branch is removed, this fails. \
         Got JSON: {:.500}",
        json_str
    );

    // ── "verify network" MUST NOT appear: 503 sensors are NOT network-unreachable ───────
    // Both sensors return HTTP 503 (they ARE reachable at the network layer).
    // The "verify network" text belongs to ConnectivityStatus::Down sensors only.
    // With only 503 sensors in this test, "verify network" MUST be absent entirely.
    assert!(
        !json_str.contains("verify network"),
        "F-S504-LP1P2-MED-001 (distinctness from Down/unreachable): \
         503 sensors are network-reachable (HTTP 503 = server-side error, NOT connection failure). \
         The 'verify network' suggestion MUST NOT appear. \
         Got JSON: {:.500}",
        json_str
    );

    // ── Sanity: the 5xx and "verify network" strings are distinct constants ─────────────
    let would_be_down_suggestion =
        "Sensor unreachable \u{2014} verify network and endpoint configuration.";
    assert_ne!(
        expected_5xx_suggestion, would_be_down_suggestion,
        "F-S504-LP1P2-MED-001: the 5xx and Down suggestion constants must differ (sanity)"
    );
}

// ─── F-S504-LP1P3-MED-001 — AC-12 suggestion ladder: auth-invalid + genuine-Down ──

/// F-S504-LP1P3-MED-001 (1/2): `check_sensor_health` for an all-401 (auth-invalid) fleet
/// MUST emit the verbatim AC-12 suggestion string through the production handler.
///
/// AC-12 (BC-2.08.007 EC-08-015): auth-invalid sensors carry:
///   `"Check credentials \u{2014} sensor rejected authentication."` (em-dash U+2014)
///
/// Production path:
///   `MockAdapterUnauthorized` → `SensorError::HttpError { status: 401 }`
///   → `probe_auth` → `AuthStatus::Invalid`
///   → `check_one` → `auth_valid = Some(false)`, `reachable = Some(true)`
///   → `server.rs` suggestion ladder → `auth_valid == Some(false)` arm
///   → `"Check credentials \u{2014} sensor rejected authentication."`
///
/// RED GATE (TD-VSDD-059): reverting/paraphrasing the `auth_valid == Some(false)` branch in
/// server.rs causes this assertion to fail because the JSON will not contain the verbatim string.
///
/// A second sensor (`MockAdapterUnauthorizedArmis`, also 401) confirms the branch fires for
/// both crowdstrike and armis sensor types, proving the arm is sensor-type-agnostic.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_007_EC_08_015_auth_invalid_production_path_suggestion() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Register two 401 sensors (crowdstrike + armis) — both trigger the auth-invalid branch.
    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(MockAdapterUnauthorized));
    adapter_registry.register(org_id, Arc::new(MockAdapterUnauthorizedArmis));
    let adapter_registry = Arc::new(adapter_registry);

    // TableRegistry — enumerate both sensors in check_sensor_health.
    let table_registry = TableRegistry::new();
    let cs_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "crowdstrike",
        "CrowdStrike (401-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    let armis_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "armis",
        "Armis (401-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.armis.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    table_registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike");
    table_registry
        .register_sensor(&armis_spec)
        .expect("register armis");

    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    let params = CheckSensorHealthParams::for_client("acme");
    let call_result = server.check_sensor_health(Parameters(params)).await.expect(
        "F-S504-LP1P3-MED-001 (1/2): check_sensor_health MUST succeed for all-auth-invalid sensors",
    );

    let json_str = serde_json::to_string(&call_result)
        .expect("F-S504-LP1P3-MED-001 (1/2): CallToolResult must serialize to JSON");

    // ── RED GATE (primary): auth-invalid sensors MUST get the verbatim AC-12 suggestion ──
    // Reverting the `auth_valid == Some(false)` branch in server.rs causes this to fail.
    let expected_auth_suggestion = "Check credentials \u{2014} sensor rejected authentication.";
    assert!(
        json_str.contains(expected_auth_suggestion),
        "F-S504-LP1P3-MED-001 (1/2) RED GATE — production handler: \
         a 401-auth-invalid sensor MUST carry suggestion: \"{expected_auth_suggestion}\" \
         (verbatim AC-12 / BC-2.08.007 EC-08-015, em-dash U+2014) through PrismServer::check_sensor_health. \
         If the server.rs auth_valid==Some(false) branch is removed or paraphrased, this fails. \
         Got JSON: {:.500}",
        json_str
    );

    // ── The "verify network" Down-suggestion MUST NOT appear: 401 sensors ARE reachable ──
    // A 401 response requires a successful HTTP exchange — the sensor is network-reachable.
    assert!(
        !json_str.contains("verify network"),
        "F-S504-LP1P3-MED-001 (1/2) distinctness: \
         401-auth-invalid sensors are network-reachable. \
         'verify network' (Down-suggestion) MUST NOT appear for an all-401 fleet. \
         Got JSON: {:.500}",
        json_str
    );

    // ── overall_status: "unhealthy" — both sensors auth-invalid, none healthy ──────────
    assert!(
        json_str.contains(r#""overall_status":"unhealthy""#)
            || json_str.contains(r#""overall_status": "unhealthy""#),
        "F-S504-LP1P3-MED-001 (1/2): all-auth-invalid fleet MUST produce overall_status=\"unhealthy\". \
         Got JSON: {:.500}",
        json_str
    );
}

/// F-S504-LP1P3-MED-001 (2/2): `check_sensor_health` for a genuine-Down (connection refused)
/// fleet MUST emit the verbatim AC-12 "verify network" suggestion through the production handler,
/// and MUST NOT emit the 5xx-specific suggestion.
///
/// AC-12 (BC-2.08.007 EC-08-015): unreachable sensors carry:
///   `"Sensor unreachable \u{2014} verify network and endpoint configuration."` (em-dash U+2014)
///
/// Production path:
///   `MockAdapterConnectionRefused` → `SensorError::Timeout`
///   → `probe_connectivity` → `ConnectivityStatus::Down`
///   → `check_one` → `reachable = Some(false)`, `auth_valid = None`,
///                    `error` is NOT `"service_unavailable"` (no HTTP exchange)
///   → `server.rs` suggestion ladder → `reachable == Some(false)` +
///     `error != "service_unavailable"` → "Sensor unreachable \u{2014} verify network …"
///
/// RED GATE (TD-VSDD-059): reverting or collapsing the `else` arm in server.rs (the fallthrough
/// after the `service_unavailable` check) causes this assertion to fail.
///
/// Distinctness guarantee: this test also asserts the 5xx string is absent — confirming the
/// Down branch and the Degraded (5xx) branch are mutually exclusive, which is the precise
/// invariant F-S504-LP1P1-MED-001 and F-S504-LP1P2-MED-001 depend on.
#[tokio::test]
#[allow(non_snake_case)]
async fn test_BC_2_08_007_EC_08_015_genuine_down_production_path_suggestion_distinct_from_5xx() {
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{CheckSensorHealthParams, PrismServer};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        table_registry::TableRegistry,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // Register two genuinely-unreachable sensors (crowdstrike + armis, both timeout/refused).
    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, Arc::new(MockAdapterConnectionRefused));
    adapter_registry.register(org_id, Arc::new(MockAdapterConnectionRefusedArmis));
    let adapter_registry = Arc::new(adapter_registry);

    // TableRegistry — enumerate both sensors in check_sensor_health.
    let table_registry = TableRegistry::new();
    let cs_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "crowdstrike",
        "CrowdStrike (timeout-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    let armis_spec = prism_spec_engine::spec_parser::SensorSpec::new(
        "armis",
        "Armis (timeout-mock)",
        prism_spec_engine::spec_parser::AuthType::ApiKey,
        "https://api.armis.com",
        vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![],
                vec![],
            ),
        ],
        None,
        "1.0.0",
        vec![],
    );
    table_registry
        .register_sensor(&cs_spec)
        .expect("register crowdstrike");
    table_registry
        .register_sensor(&armis_spec)
        .expect("register armis");

    let engine = QueryEngine::new(
        Arc::clone(&adapter_registry),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
    )
    .with_table_registry(Arc::new(table_registry));

    let checker = SensorHealthChecker::new(Arc::clone(&adapter_registry));

    let server = PrismServer::new()
        .with_query_engine(Arc::new(engine))
        .with_health_checker(checker);

    let params = CheckSensorHealthParams::for_client("acme");
    let call_result = server
        .check_sensor_health(Parameters(params))
        .await
        .expect(
            "F-S504-LP1P3-MED-001 (2/2): check_sensor_health MUST succeed for genuinely-unreachable sensors",
        );

    let json_str = serde_json::to_string(&call_result)
        .expect("F-S504-LP1P3-MED-001 (2/2): CallToolResult must serialize to JSON");

    // ── RED GATE (primary): Down sensors MUST get the verbatim AC-12 "verify network" suggestion
    // Reverting the `else` fallthrough arm in server.rs (after service_unavailable check) causes this.
    let expected_down_suggestion =
        "Sensor unreachable \u{2014} verify network and endpoint configuration.";
    assert!(
        json_str.contains(expected_down_suggestion),
        "F-S504-LP1P3-MED-001 (2/2) RED GATE — production handler: \
         a genuinely-unreachable sensor (connection refused / timeout) MUST carry \
         suggestion: \"{expected_down_suggestion}\" \
         (verbatim AC-12 / BC-2.08.007 EC-08-015, em-dash U+2014) through PrismServer::check_sensor_health. \
         If the server.rs else-fallthrough arm is removed or the Down path is conflated with 5xx, \
         this fails. Got JSON: {:.500}",
        json_str
    );

    // ── The 5xx-specific suggestion MUST NOT appear: timeout sensors had NO HTTP exchange ──
    // A connection timeout never produces a 5xx HTTP response — it never reached the server.
    let would_be_5xx_suggestion =
        "Sensor returned a server error (5xx) \u{2014} service may be temporarily unavailable.";
    assert!(
        !json_str.contains(would_be_5xx_suggestion),
        "F-S504-LP1P3-MED-001 (2/2) distinctness: \
         genuinely-unreachable sensors (timeout, no HTTP exchange) MUST NOT carry the 5xx suggestion. \
         The 5xx suggestion is for HTTP 503 (service-reachable but degraded). \
         Got JSON: {:.500}",
        json_str
    );

    // ── overall_status: "unhealthy" — both sensors down, none healthy ────────────────────
    assert!(
        json_str.contains(r#""overall_status":"unhealthy""#)
            || json_str.contains(r#""overall_status": "unhealthy""#),
        "F-S504-LP1P3-MED-001 (2/2): all-down fleet MUST produce overall_status=\"unhealthy\". \
         Got JSON: {:.500}",
        json_str
    );
}
