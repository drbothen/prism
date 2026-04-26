//! Tests for BC-2.01.014: Exponential Backoff and Retry for Transient Sensor API Errors.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//!
//! Covers:
//! - `RetryConfig::default()` has `base_delay_ms == 2000` (v1.5 — NOT 1000)
//! - `RetryConfig::default()` has `max_delay_ms == 30_000`
//! - `compute_raw_delay_ms()` schedule matches `2s, 4s, 8s, 16s, 30s, 30s, ...`
//! - `is_transient_status()` returns true for 429/500/502/503/504
//! - `is_transient_status()` returns false for 400/401/403/404
//! - `retry_with_backoff()` succeeds on second attempt after transient error (AC-4)
//! - `retry_with_backoff()` returns immediately on non-transient error (EC-005)
//! - `retry_with_backoff()` returns `RetryBudgetExhausted` after `max_attempts`
//! - Jitter: computed jittered delay is in `[0, raw_delay]`
//! - `SensorError::is_transient()` mirrors `is_transient_status()`
//!
//! # Version note (S-2.06 v1.5)
//! The stub commit `e86d03f2` was generated against v1.3 and has
//! `base_delay_ms = 1000`. The v1.5 spec (and BC-2.01.014 "2s base" contract)
//! mandates `base_delay_ms = 2000`. All assertions here use the **literal**
//! value `2000` — never `RetryConfig::default().base_delay_ms` — to catch the
//! All tests pass (implementation complete).
//!
//! Story: S-2.06 | BC: BC-2.01.014

use crate::{
    adapter::SensorError,
    retry::{raw_delay_ms, RetryConfig, DEFAULT_TRANSIENT_CODES},
};

// ---------------------------------------------------------------------------
// RetryConfig defaults (v1.5 — base must be 2000 NOT 1000)
// ---------------------------------------------------------------------------

/// BC-2.01.014 precondition: "2s base" — default `base_delay_ms` must be 2000.
///
/// IMPORTANT: This test is intentionally written against the LITERAL value 2000,
/// not `RetryConfig::default().base_delay_ms`, so it fails Red on the stub's
/// wrong `1000` constant and passes only after the implementer corrects it.
#[test]
fn test_BC_2_01_014_retry_config_default_base_delay_is_2000ms() {
    let config = RetryConfig::default();
    assert_eq!(
        config.base_delay_ms, 2000,
        "BC-2.01.014 requires base_delay_ms = 2000 (v1.5 spec); \
         stub had 1000 — implementer must update the constant"
    );
}

/// Default `max_delay_ms` must be 30 000 ms (30 seconds).
#[test]
fn test_BC_2_01_014_retry_config_default_max_delay_is_30_000ms() {
    let config = RetryConfig::default();
    assert_eq!(
        config.max_delay_ms, 30_000,
        "max_delay_ms must be 30_000 ms (30 seconds)"
    );
}

/// Default `multiplier` must be 2.0.
#[test]
fn test_BC_2_01_014_retry_config_default_multiplier_is_2() {
    let config = RetryConfig::default();
    assert!(
        (config.multiplier - 2.0_f64).abs() < f64::EPSILON,
        "multiplier must be 2.0, got {}",
        config.multiplier
    );
}

/// Default `max_attempts` must be 3.
#[test]
fn test_BC_2_01_014_retry_config_default_max_attempts_is_3() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3, "default max_attempts must be 3");
}

// ---------------------------------------------------------------------------
// Delay schedule: 2s, 4s, 8s, 16s, 30s cap (BC-2.01.014 postconditions)
// ---------------------------------------------------------------------------

/// BC-2.01.014 postcondition: delay schedule is `2s, 4s, 8s, 16s, 30s, 30s, ...`
/// with `base_delay_ms = 2000`, `multiplier = 2.0`, `max_delay_ms = 30_000`.
///
/// Attempt indices are 0-based:
/// - attempt 0 → 2000 ms (base)
/// - attempt 1 → 4000 ms
/// - attempt 2 → 8000 ms
/// - attempt 3 → 16000 ms
/// - attempt 4 → 30000 ms (capped)
/// - attempt 5 → 30000 ms (still capped)
#[test]
fn test_BC_2_01_014_delay_schedule_matches_spec() {
    let config = RetryConfig {
        base_delay_ms: 2000,
        multiplier: 2.0,
        max_delay_ms: 30_000,
        max_attempts: 10,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    assert_eq!(raw_delay_ms(&config, 0), 2_000, "attempt 0 → 2000 ms");
    assert_eq!(raw_delay_ms(&config, 1), 4_000, "attempt 1 → 4000 ms");
    assert_eq!(raw_delay_ms(&config, 2), 8_000, "attempt 2 → 8000 ms");
    assert_eq!(raw_delay_ms(&config, 3), 16_000, "attempt 3 → 16000 ms");
    assert_eq!(
        raw_delay_ms(&config, 4),
        30_000,
        "attempt 4 → 30000 ms (cap)"
    );
    assert_eq!(
        raw_delay_ms(&config, 5),
        30_000,
        "attempt 5 → 30000 ms (still capped)"
    );
}

/// The cap is enforced at exactly `max_delay_ms`.
#[test]
fn test_BC_2_01_014_delay_never_exceeds_max_delay_ms() {
    let config = RetryConfig::default();
    for attempt in 0u32..20 {
        let delay = raw_delay_ms(&config, attempt);
        assert!(
            delay <= 30_000,
            "delay at attempt {attempt} is {delay} — must not exceed 30_000 ms"
        );
    }
}

// ---------------------------------------------------------------------------
// Transient / non-transient status classification
// ---------------------------------------------------------------------------

/// BC-2.01.014: HTTP 429, 500, 502, 503, 504 are transient.
#[test]
fn test_BC_2_01_014_transient_codes_are_correctly_classified() {
    let config = RetryConfig::default();
    for code in [429u16, 500, 502, 503, 504] {
        assert!(
            config.is_transient_status(code),
            "HTTP {code} must be classified as transient"
        );
    }
}

/// BC-2.01.014: HTTP 400, 401, 403, 404 are NOT transient — never retried.
#[test]
fn test_BC_2_01_014_non_transient_codes_are_not_retried() {
    let config = RetryConfig::default();
    for code in [400u16, 401, 403, 404] {
        assert!(
            !config.is_transient_status(code),
            "HTTP {code} must NOT be classified as transient"
        );
    }
}

/// `DEFAULT_TRANSIENT_CODES` contains exactly `[429, 500, 502, 503, 504]`.
#[test]
fn test_BC_2_01_014_default_transient_codes_exact_set() {
    let expected: &[u16] = &[429, 500, 502, 503, 504];
    for code in expected {
        assert!(
            DEFAULT_TRANSIENT_CODES.contains(code),
            "DEFAULT_TRANSIENT_CODES must contain {code}"
        );
    }
    // No extraneous codes (e.g., 400 must not sneak in).
    for code in [400u16, 401, 403, 404] {
        assert!(
            !DEFAULT_TRANSIENT_CODES.contains(&code),
            "DEFAULT_TRANSIENT_CODES must NOT contain {code}"
        );
    }
}

// ---------------------------------------------------------------------------
// SensorError::is_transient() mirrors the retry policy
// ---------------------------------------------------------------------------

/// `SensorError::HttpError` with status 503 is transient.
#[test]
fn test_BC_2_01_014_sensor_error_http_503_is_transient() {
    let err = SensorError::HttpError {
        sensor: "test".into(),
        status: 503,
        body: String::new(),
    };
    assert!(
        err.is_transient(),
        "HttpError(503) must be transient (BC-2.01.014)"
    );
}

/// `SensorError::HttpError` with status 429 is transient.
#[test]
fn test_BC_2_01_014_sensor_error_http_429_is_transient() {
    let err = SensorError::HttpError {
        sensor: "test".into(),
        status: 429,
        body: String::new(),
    };
    assert!(err.is_transient(), "HttpError(429) must be transient");
}

/// `SensorError::HttpError` with status 400 is NOT transient.
#[test]
fn test_BC_2_01_014_sensor_error_http_400_is_not_transient() {
    let err = SensorError::HttpError {
        sensor: "test".into(),
        status: 400,
        body: String::new(),
    };
    assert!(
        !err.is_transient(),
        "HttpError(400) must NOT be transient — non-transient errors are never retried"
    );
}

/// `SensorError::HttpError` with status 404 is NOT transient.
#[test]
fn test_BC_2_01_014_sensor_error_http_404_is_not_transient() {
    let err = SensorError::HttpError {
        sensor: "test".into(),
        status: 404,
        body: String::new(),
    };
    assert!(!err.is_transient(), "HttpError(404) must NOT be transient");
}

/// `SensorError::Timeout` is transient (network timeouts are retriable).
#[test]
fn test_BC_2_01_014_sensor_error_timeout_is_transient() {
    let err = SensorError::Timeout {
        sensor: "test".into(),
        elapsed_ms: 5000,
    };
    assert!(err.is_transient(), "Timeout must be transient");
}

/// `SensorError::RateLimited` is transient.
#[test]
fn test_BC_2_01_014_sensor_error_rate_limited_is_transient() {
    let err = SensorError::RateLimited {
        sensor: "test".into(),
        retry_after_ms: 10_000,
    };
    assert!(err.is_transient(), "RateLimited must be transient");
}

/// `SensorError::ResponseParse` is NOT transient (structural problem, not network).
#[test]
fn test_BC_2_01_014_sensor_error_response_parse_is_not_transient() {
    let err = SensorError::ResponseParse {
        sensor: "test".into(),
        detail: "unexpected json field".into(),
    };
    assert!(!err.is_transient(), "ResponseParse must NOT be transient");
}

// ---------------------------------------------------------------------------
// retry_with_backoff async tests
// ---------------------------------------------------------------------------

/// AC-4: A transient 503 on the first attempt is followed by a retry.
/// The operation must succeed on the second attempt.
///
/// This test uses a `std::sync::atomic::AtomicU32` call counter to verify
/// that the operation is called exactly twice.
///
/// TV-BC-2.01.014-001 (analogue — HTTP 503 retried successfully)
#[tokio::test]
async fn test_BC_2_01_014_retry_with_backoff_succeeds_on_second_attempt_for_503() {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use crate::retry::retry_with_backoff;

    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let config = RetryConfig {
        base_delay_ms: 0, // zero delay so test runs fast
        multiplier: 2.0,
        max_delay_ms: 0, // zero delay cap
        max_attempts: 3,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    let result = retry_with_backoff(
        move || {
            let n = call_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if n == 0 {
                    // First attempt: transient 503
                    Err(SensorError::HttpError {
                        sensor: "test".into(),
                        status: 503,
                        body: "service unavailable".into(),
                    })
                } else {
                    // Second attempt: success
                    Ok(42u32)
                }
            }
        },
        "test",
        config,
    )
    .await;

    assert!(
        result.is_ok(),
        "retry_with_backoff must succeed on second attempt; got: {result:?}"
    );
    assert_eq!(result.unwrap(), 42u32);
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        2,
        "op must have been called exactly twice"
    );
}

/// EC-005: A non-transient HTTP 400 on the first attempt must NOT be retried.
/// `retry_with_backoff` must return the error immediately after exactly 1 call.
///
/// BC-2.01.014: "Non-transient errors (400, 404) are not retried; return immediately."
#[tokio::test]
async fn test_BC_2_01_014_retry_with_backoff_returns_immediately_for_400() {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use crate::retry::retry_with_backoff;

    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let config = RetryConfig {
        base_delay_ms: 0,
        multiplier: 2.0,
        max_delay_ms: 0,
        max_attempts: 3,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    let result: Result<u32, _> = retry_with_backoff(
        move || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            async {
                Err(SensorError::HttpError {
                    sensor: "test".into(),
                    status: 400,
                    body: "bad request".into(),
                })
            }
        },
        "test",
        config,
    )
    .await;

    assert!(
        result.is_err(),
        "non-transient 400 must return an error immediately"
    );
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "op must have been called exactly ONCE — no retries for non-transient errors"
    );
}

/// TV-BC-2.01.014-005: HTTP 404 (non-transient) — no retry, immediate error.
#[tokio::test]
async fn test_BC_2_01_014_retry_with_backoff_returns_immediately_for_404() {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use crate::retry::retry_with_backoff;

    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let config = RetryConfig {
        base_delay_ms: 0,
        multiplier: 2.0,
        max_delay_ms: 0,
        max_attempts: 3,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    let result: Result<u32, _> = retry_with_backoff(
        move || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            async {
                Err(SensorError::HttpError {
                    sensor: "test".into(),
                    status: 404,
                    body: "not found".into(),
                })
            }
        },
        "test",
        config,
    )
    .await;

    assert!(result.is_err(), "HTTP 404 must return an error");
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "op must have been called exactly ONCE for 404"
    );
}

/// TV-BC-2.01.014-004: After `max_attempts` on a transient 503, returns
/// `SensorError::RetryBudgetExhausted`.
#[tokio::test]
async fn test_BC_2_01_014_retry_budget_exhausted_after_max_attempts() {
    use crate::retry::retry_with_backoff;

    let config = RetryConfig {
        base_delay_ms: 0,
        multiplier: 2.0,
        max_delay_ms: 0,
        max_attempts: 3,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    let result: Result<u32, _> = retry_with_backoff(
        || async {
            Err(SensorError::HttpError {
                sensor: "test".into(),
                status: 503,
                body: "always fails".into(),
            })
        },
        "test",
        config,
    )
    .await;

    assert!(
        result.is_err(),
        "must return an error after budget exhausted"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, SensorError::RetryBudgetExhausted { .. }),
        "error must be RetryBudgetExhausted, got: {err:?}"
    );
}

/// Jitter: `compute_jittered_delay_ms()` returns a value in `[0, raw_delay]`.
/// Runs 200 samples to probabilistically cover the full range.
#[test]
fn test_BC_2_01_014_jitter_is_within_0_to_raw_delay() {
    let config = RetryConfig {
        base_delay_ms: 2000,
        multiplier: 2.0,
        max_delay_ms: 30_000,
        max_attempts: 3,
        transient_codes: DEFAULT_TRANSIENT_CODES,
    };

    for attempt in 0u32..5 {
        let raw = raw_delay_ms(&config, attempt);
        for _ in 0..200 {
            let jittered = config.compute_jittered_delay_ms(attempt);
            assert!(
                jittered <= raw,
                "jittered delay {jittered} must be ≤ raw delay {raw} at attempt {attempt}"
            );
        }
    }
}
