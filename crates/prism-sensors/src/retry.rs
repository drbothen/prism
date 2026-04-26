//! Exponential backoff with full jitter for transient sensor API errors.
//!
//! # Retry Policy (BC-2.01.014)
//! - Transient codes: 429, 500, 502, 503, 504, network timeout
//! - Non-transient codes (400, 401, 403, 404, …): returned immediately, no retry
//! - Jitter: full jitter — `delay = rand::random::<f64>() * computed_delay`
//! - HTTP 429 with a `Retry-After` header overrides the computed backoff
//!
//! Story: S-2.06 | BC: BC-2.01.014

use std::{future::Future, time::Duration};

use crate::adapter::SensorError;

// ---------------------------------------------------------------------------
// RetryConfig
// ---------------------------------------------------------------------------

/// Configuration for `retry_with_backoff`.
///
/// Defaults match the story spec:
/// - `base_delay_ms = 2000` (2 s) — BC-2.01.014 "2s base"
/// - `multiplier = 2.0`
/// - `max_delay_ms = 30_000` (30 s)
/// - `max_attempts = 3`
/// - `transient_codes` — the canonical set of transient HTTP status codes
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Base delay in milliseconds before the first retry.
    pub base_delay_ms: u64,
    /// Exponential growth factor applied to `base_delay_ms` each attempt.
    pub multiplier: f64,
    /// Ceiling on the computed delay (before jitter) in milliseconds.
    pub max_delay_ms: u64,
    /// Maximum number of total attempts (1 = no retries; 0 = unlimited).
    pub max_attempts: u32,
    /// HTTP status codes that trigger a retry.
    ///
    /// All other 4xx codes are NOT retried (BC-2.01.014 invariant).
    pub transient_codes: &'static [u16],
}

/// Canonical set of transient HTTP status codes for `RetryConfig::default()`.
///
/// 429 (rate limited), 500 (internal server error), 502 (bad gateway),
/// 503 (service unavailable), 504 (gateway timeout).
pub static DEFAULT_TRANSIENT_CODES: &[u16] = &[429, 500, 502, 503, 504];

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 2_000,
            multiplier: 2.0,
            max_delay_ms: 30_000,
            max_attempts: 3,
            transient_codes: DEFAULT_TRANSIENT_CODES,
        }
    }
}

impl RetryConfig {
    /// Returns `true` when `http_status` is in the `transient_codes` list.
    pub fn is_transient_status(&self, http_status: u16) -> bool {
        self.transient_codes.contains(&http_status)
    }

    /// Computes the raw (pre-jitter) delay in milliseconds for `attempt` (0-indexed).
    ///
    /// `delay = min(base_delay_ms * multiplier^attempt, max_delay_ms)`
    pub fn compute_raw_delay_ms(&self, attempt: u32) -> u64 {
        let raw = self.base_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        raw.min(self.max_delay_ms as f64) as u64
    }

    /// Computes a jittered delay for `attempt` using full-jitter strategy.
    ///
    /// `delay = rand::random::<f64>() * raw_delay` (BC-2.01.014).
    pub fn compute_jittered_delay_ms(&self, attempt: u32) -> u64 {
        let raw = self.compute_raw_delay_ms(attempt) as f64;
        (rand::random::<f64>() * raw) as u64
    }
}

// ---------------------------------------------------------------------------
// retry_with_backoff
// ---------------------------------------------------------------------------

/// Retries `op` with exponential backoff and full jitter.
///
/// Calls `op()` up to `config.max_attempts` times. On each `Err`:
/// 1. Checks whether the error is transient (via `SensorError::is_transient()`
///    and `config.is_transient_status()`).
/// 2. If non-transient → return the error immediately without further retries.
/// 3. If transient → sleep for a jittered delay, then retry.
/// 4. If `max_attempts` exceeded → return `SensorError::RetryBudgetExhausted`.
///
/// HTTP 429 responses with a `Retry-After` value can override the computed
/// backoff via `SensorError::RateLimited { retry_after_ms }`.
///
/// # Arguments
/// - `op` — an async closure that produces `Result<T, SensorError>`.
/// - `sensor_name` — name used in log spans and error messages.
/// - `config` — backoff parameters.
///
/// # AC-4
/// On a transient 429 first attempt, the second attempt waits at least
/// `base_delay_ms` milliseconds with jitter applied.
///
/// Story: S-2.06 | BC: BC-2.01.014
pub async fn retry_with_backoff<F, Fut, T>(
    op: F,
    sensor_name: &str,
    config: RetryConfig,
) -> Result<T, SensorError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, SensorError>>,
{
    let max = config.max_attempts;
    let mut attempt: u32 = 0;

    loop {
        match op().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                // Non-transient: return immediately — no retry
                if !err.is_transient() {
                    return Err(err);
                }

                attempt += 1;

                // Budget exhausted after max_attempts failures
                if attempt >= max {
                    return Err(SensorError::RetryBudgetExhausted {
                        sensor: sensor_name.to_string(),
                        attempts: attempt,
                    });
                }

                // Sleep the backoff duration before the next attempt
                let delay = sleep_duration(&err, attempt - 1, &config);
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Determines the sleep duration for a retry attempt.
///
/// If the error is `SensorError::RateLimited { retry_after_ms }`, that value
/// overrides the computed backoff (BC-2.01.014, EC-01-022).
/// Otherwise, full-jitter backoff from `config` is used.
fn sleep_duration(error: &SensorError, attempt: u32, config: &RetryConfig) -> Duration {
    if let SensorError::RateLimited { retry_after_ms, .. } = error {
        return Duration::from_millis(*retry_after_ms);
    }
    Duration::from_millis(config.compute_jittered_delay_ms(attempt))
}

// Re-export for tests
#[cfg(test)]
pub use self::compute_helpers::*;

#[cfg(test)]
mod compute_helpers {
    use super::*;

    /// Exposed for tests: compute raw delay without jitter.
    pub fn raw_delay_ms(config: &RetryConfig, attempt: u32) -> u64 {
        config.compute_raw_delay_ms(attempt)
    }
}
