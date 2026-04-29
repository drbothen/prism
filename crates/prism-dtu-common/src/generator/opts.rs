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
    pub time_anchor: DateTime<Utc>,
    /// JSON Merge Patch (RFC 7396) applied after generation (BC-3.4.001 precondition 5).
    pub overrides: Value,
}

impl Default for GenOpts {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 1.0,
            time_anchor: DateTime::UNIX_EPOCH,
            overrides: Value::Null,
        }
    }
}

impl GenOpts {
    /// Construct a validated `GenOpts`.
    ///
    /// Returns `Err(GenOptsError::InvalidScale)` if `scale` is not positive and finite.
    pub fn new(
        _seed: u64,
        _scale: f64,
        _time_anchor: DateTime<Utc>,
        _overrides: Value,
    ) -> Result<Self, GenOptsError> {
        todo!()
    }
}
