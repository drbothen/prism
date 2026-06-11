//! `GenOpts` — generator parameterisation struct (BC-3.4.001 preconditions 2–5).

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Error returned when `GenOpts` validation fails.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GenOptsError {
    /// `scale` is not positive and finite (BC-3.4.001 precondition 3).
    #[error("GenOpts.scale must be positive and finite")]
    InvalidScale,
}

/// Demo-era time anchor epoch seconds: `2026-01-01T00:00:00Z`.
///
/// Fix for review-2026-06-10 finding P1-01: `GenOpts::default().time_anchor`
/// was `DateTime::UNIX_EPOCH`, so every seeded DTU clone served records whose
/// timestamps were anchored at 1970 — realistic demo time-window queries
/// returned 0 rows. The anchor is now a FIXED, DOCUMENTED constant matching the
/// static fixtures' era (they carry 2026-01 timestamps). It is deterministic —
/// NOT wall-clock — so BC-3.4.001 invariant 1 (no `SystemTime::now()` in the
/// generator call stack) and the byte-identity postconditions are preserved.
///
/// BC-3.4.001 v0.9 (PO consolidated amendment burst, 2026-06-10) records this
/// default: precondition 4 now reads "default is
/// `prism_dtu_common::demo_time_anchor()` (`2026-01-01T00:00:00Z`, epoch
/// `1_767_225_600`)". The determinism contract itself is unaffected.
///
/// Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) wires `scenario_start_secs` →
/// `time_anchor`; until then this constant is the anchor for all seeded clones.
pub const DEMO_TIME_ANCHOR_EPOCH_SECS: i64 = 1_767_225_600;

/// The demo-era time anchor as a `DateTime<Utc>`: `2026-01-01T00:00:00Z`.
///
/// See [`DEMO_TIME_ANCHOR_EPOCH_SECS`] for rationale (review-2026-06-10 P1-01).
/// Deterministic constant — no wall-clock involvement (BC-3.4.001 invariant 1).
pub fn demo_time_anchor() -> DateTime<Utc> {
    // Panic-free: the delta is a fixed in-range constant; `TimeDelta::seconds`
    // only panics outside ±i64::MAX/1000 seconds.
    DateTime::UNIX_EPOCH + chrono::TimeDelta::seconds(DEMO_TIME_ANCHOR_EPOCH_SECS)
}

/// Generator parameterisation (ADR-009 §2.3).
///
/// Construct via `GenOpts::default()` for test defaults or `GenOpts::new()` for
/// validated construction.
#[derive(Clone, Debug)]
pub struct GenOpts {
    /// Deterministic seed for `ChaCha20Rng` (BC-3.4.001 precondition 2).
    pub seed: u64,
    /// Record-count scale factor; must be positive and finite (BC-3.4.001 precondition 3).
    pub scale: f64,
    /// Time anchor for all generated timestamps (BC-3.4.001 precondition 4).
    /// Defaults to [`demo_time_anchor`] (2026-01-01T00:00:00Z) per
    /// review-2026-06-10 P1-01 — fixed demo-era constant, not wall-clock.
    pub time_anchor: DateTime<Utc>,
    /// JSON Merge Patch (RFC 7396) applied after generation (BC-3.4.001 precondition 5).
    pub overrides: Value,
}

impl Default for GenOpts {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 1.0,
            time_anchor: demo_time_anchor(),
            overrides: Value::Null,
        }
    }
}

impl GenOpts {
    /// Construct a validated `GenOpts`.
    ///
    /// Returns `Err(GenOptsError::InvalidScale)` if `scale` is not positive and finite
    /// (BC-3.4.001 precondition 3: rejects 0, negative, inf, NaN).
    pub fn new(
        seed: u64,
        scale: f64,
        time_anchor: DateTime<Utc>,
        overrides: Value,
    ) -> Result<Self, GenOptsError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(GenOptsError::InvalidScale);
        }
        Ok(Self {
            seed,
            scale,
            time_anchor,
            overrides,
        })
    }
}
