//! Rate-limit state detection from HTTP 429 headers (BC-2.08.003, S-5.04).
//!
//! Tracks HTTP 429 responses per (client_id, sensor_id) and extracts
//! `Retry-After` header values to compute when the rate limit clears.
//!
//! # Retry-After parsing (confirmed approach — uncertainty-pivot003-s504-2026-06-19.md §3)
//! 1. Trim whitespace; try `s.parse::<u64>()` → `Duration::from_secs(n)` (delta-seconds).
//! 2. On failure, try `chrono::DateTime::parse_from_rfc2822(s)` → absolute HTTP-date.
//!    Compute duration relative to `SystemTime::now()`; clamp past/negative to zero.
//!
//! CRITICAL: Do NOT use `parse_from_str` with `%Z` — chrono issue #1575 fails on "GMT".
//! Use `parse_from_rfc2822` ONLY for HTTP-date form.
//! Use `u64` (not `u32`) to avoid integer overflow on hostile delta-seconds values.
//!
//! Parse failure is advisory: log at debug level, fall back to 60-second default.
//!
//! # Rate-limit clearing
//! Clearing is time-based: when `retry_after` elapses, the next health check
//! sees a cleared state automatically — no explicit reset needed (BC-2.08.003).

use std::time::Duration;

use chrono::{DateTime, Utc};

/// Default backoff when no `Retry-After` header is present or parse fails.
pub const DEFAULT_RETRY_AFTER_SECS: u64 = 60;

/// Rate-limit state for a (client_id, sensor_id) pair (BC-2.08.003).
///
/// Tracked in `PrismContext::rate_limit_states` and written on every HTTP 429
/// observation.  Cleared automatically when `retry_after` elapses.
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Whether the sensor is currently rate-limited.
    pub is_rate_limited: bool,
    /// UTC time after which the rate limit is expected to clear.
    /// `None` if the sensor is not rate-limited.
    pub retry_after: Option<DateTime<Utc>>,
    /// UTC time when the 429 was observed.
    pub observed_at: DateTime<Utc>,
}

impl RateLimitState {
    /// Construct a rate-limited state with the given retry-after timestamp.
    ///
    /// Sets `observed_at` to `Utc::now()` (clock I/O).
    pub fn rate_limited(retry_after: DateTime<Utc>) -> Self {
        Self {
            is_rate_limited: true,
            retry_after: Some(retry_after),
            observed_at: Utc::now(),
        }
    }

    /// Construct a "not rate-limited" state.
    ///
    /// Sets `observed_at` to `Utc::now()` (clock I/O).
    pub fn not_limited() -> Self {
        Self {
            is_rate_limited: false,
            retry_after: None,
            observed_at: Utc::now(),
        }
    }

    /// Returns `true` if the rate limit has already cleared (retry_after < now).
    ///
    /// Used for auto-expiry: when this returns `true`, the caller clears the
    /// stored state and treats the sensor as no longer rate-limited.
    pub fn is_cleared(&self) -> bool {
        if !self.is_rate_limited {
            return true;
        }
        match self.retry_after {
            Some(retry_at) => Utc::now() >= retry_at,
            None => true,
        }
    }
}

/// Parse a raw `Retry-After` header value into a `Duration` from now.
///
/// Algorithm (confirmed approach — uncertainty-pivot003-s504-2026-06-19.md §3):
/// 1. `s.trim().parse::<u64>()` → delta-seconds form.
/// 2. `chrono::DateTime::parse_from_rfc2822(s)` → absolute HTTP-date form.
///    Clamps past dates to `Duration::ZERO`.
///
/// Returns `None` on parse failure (caller falls back to `DEFAULT_RETRY_AFTER_SECS`).
///
/// # CRITICAL: No `%Z` in `parse_from_str`
/// Do NOT call `chrono::NaiveDateTime::parse_from_str` with a format string
/// containing `%Z` — chrono issue #1575 causes it to fail on `"GMT"` timezone
/// tokens.  Always use `parse_from_rfc2822` for the HTTP-date branch.
///
/// # Integer overflow safety
/// Delta-seconds are parsed as `u64`, not `u32`, to avoid overflow on hostile
/// (unrealistically large) values.
pub fn parse_retry_after(header_value: &str) -> Option<Duration> {
    let trimmed = header_value.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Branch 1: delta-seconds (u64 to prevent overflow on hostile values)
    if let Ok(secs) = trimmed.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Branch 2: IMF-fixdate (RFC 2822 / HTTP-date form)
    // Use parse_from_rfc2822 — NOT parse_from_str with %Z (chrono #1575 fails on "GMT")
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(trimmed) {
        let future = dt.with_timezone(&Utc);
        let now = Utc::now();
        if future > now {
            let delta = (future - now).num_seconds().max(0) as u64;
            return Some(Duration::from_secs(delta));
        } else {
            // Past date clamps to zero ("retry immediately")
            return Some(Duration::ZERO);
        }
    }

    None
}

/// Extract `RateLimitState` from an HTTP 429 response.
///
/// `retry_after_header` is the raw `Retry-After` header value (if present).
/// Falls back to `DEFAULT_RETRY_AFTER_SECS` when the header is absent or fails
/// to parse (BC-2.08.003 EC-003 / story EC-003).
///
/// Returns a `RateLimitState` with `is_rate_limited: true` and the computed
/// `retry_after` timestamp.
pub fn extract_rate_limit_state(retry_after_header: Option<&str>) -> RateLimitState {
    let duration = retry_after_header
        .and_then(parse_retry_after)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_RETRY_AFTER_SECS));

    let chrono_duration = chrono::Duration::seconds(duration.as_secs() as i64);
    let retry_after = Utc::now() + chrono_duration;
    RateLimitState::rate_limited(retry_after)
}

// BC-5.38.005 self-check (S-5.04 implementation complete):
// parse_retry_after — non-trivial branching (delta-seconds + IMF-fixdate). IMPLEMENTED.
// extract_rate_limit_state — non-trivial (Duration → chrono::Duration → DateTime). IMPLEMENTED.
// RateLimitState::is_cleared — non-trivial match on is_rate_limited + Option<DateTime>. IMPLEMENTED.
