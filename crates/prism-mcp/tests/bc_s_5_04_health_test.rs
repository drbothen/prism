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
//! | BC-2.08.007 — empty list → Unhealthy | test_BC_2_08_007_aggregate_empty_list_is_unhealthy |
//! | SensorHealthChecker::new GREEN | test_BC_S_5_04_sensor_health_checker_new_constructs_successfully |
//! | BC-2.08.004 checker — record+read round-trip | test_BC_2_08_004_checker_record_and_read_timestamp |
//! | BC-2.08.004 checker — None before record | test_BC_2_08_004_checker_last_successful_query_none_before_record |
//! | AC-5 (BC-2.08.004 postcondition 2) — survives checker reconstruction | test_BC_2_08_004_timestamp_survives_checker_reconstruction |
//! | AC-1 (BC-2.08.001) — 200→Up+latency | test_BC_2_08_001_live_probe_200_returns_up_with_latency |
//! | AC-2 (BC-2.08.002) — 401→auth_invalid not down | test_BC_2_08_002_live_probe_401_returns_auth_invalid_not_down |
//! | EC-004 (BC-2.08.002) — connection refused→Down+Unknown | test_BC_2_08_002_connection_refused_returns_down_auth_unknown |
//! | BC-2.08.002 — 403→auth_invalid | test_BC_2_08_002_live_probe_403_returns_auth_invalid |
//! | BC-2.08.006 postcondition 2 — sensors is keyed object | test_BC_2_08_006_sensors_health_resource_sensors_is_keyed_object |
//! | BC-2.08.006 S-5.04 — probe_level="live" in resource | test_BC_2_08_006_sensors_health_resource_live_probe_level |
//! | BC-2.08.006 S-5.04 — reachable is bool not null | test_BC_2_08_006_sensors_health_resource_reachable_is_boolean |
//! | RECONCILIATION-3 — cursor_count accessor | test_BC_2_08_005_query_engine_cursor_count_accessor |
//! | RECONCILIATION-3 — token_count accessor | test_BC_2_08_005_query_engine_token_count_accessor |
//! | FIX-001/v1.6 invariant — no direct reqwest::Client | test_BC_S_5_04_invariant_no_direct_reqwest_client_in_health_module |
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

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use prism_core::{OrgId, SensorId};
use prism_mcp::{
    context::PrismContext,
    health::{
        auth::{probe_auth, AuthStatus},
        connectivity::{probe_connectivity, ConnectivityStatus},
        rate_limit::{
            extract_rate_limit_state, parse_retry_after, RateLimitState, DEFAULT_RETRY_AFTER_SECS,
        },
        timestamp::{read_timestamp, timestamp_key, write_timestamp, HEALTH_TS_KEY_PREFIX},
        HealthCheckResult, OverallStatus, SensorHealthChecker,
    },
    resources::{render_sensors_health_resource, SensorHealthResult},
};
use prism_sensors::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    registry::AdapterRegistry,
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
/// Verifies the `BTreeMap` keyed-object shape introduced in BC-2.08.006 v1.5 via a
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
