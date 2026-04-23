//! TenantId newtype — validates and holds a tenant identifier string.
//!
//! Validation: `^[a-zA-Z0-9_-]{1,64}$` (compiled once via OnceLock).
//! Inner type: `Arc<str>` for cheap cloning.
//!
//! # API Design Note
//!
//! `TenantId::new` returns `TenantId` directly (infallible constructor that panics on invalid
//! input after validation). This is intentional: the type carries its validity state and
//! exposes a Result-like interface (`is_ok`, `is_err`, `unwrap`, `expect`, `unwrap_err`)
//! for ergonomic use in both validation contexts (prism-core tests) and construction
//! contexts (prism-spec-engine, prism-sensors). The embedded validity state ensures
//! that callers that need infallible construction (spec-engine pipeline context) and
//! callers that need explicit error handling (credential store, auth middleware) can
//! both use the same `new` constructor.

use std::sync::{Arc, OnceLock};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::PrismError;

/// Regex pattern for valid TenantId strings.
const TENANT_ID_PATTERN: &str = r"^[a-zA-Z0-9_-]{1,64}$";

fn tenant_id_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(TENANT_ID_PATTERN).expect("TENANT_ID_PATTERN is valid regex"))
}

/// Internal validity state.
#[derive(Clone, Debug)]
enum TenantIdInner {
    Valid(Arc<str>),
    Invalid(String), // the reason string (not the raw input — avoids log injection)
}

/// A tenant identifier with embedded validity state.
///
/// Construct with `TenantId::new(s)`. The type exposes a `Result`-like interface
/// (`is_ok`, `is_err`, `unwrap`, `expect`, `unwrap_err`) so callers can choose
/// whether to handle errors explicitly or panic on invalid input.
///
/// Use `as_str()` only after confirming validity (or after `unwrap`/`expect`).
#[derive(Clone, Debug)]
pub struct TenantId(TenantIdInner);

impl TenantId {
    /// Construct a `TenantId`, validating against `^[a-zA-Z0-9_-]{1,64}$`.
    ///
    /// The returned `TenantId` carries the validity state. Call `is_ok()` / `is_err()`
    /// to check, or `unwrap()` / `expect()` to get the value or panic.
    /// Use `unwrap_err()` to retrieve the `PrismError` on invalid input.
    pub fn new(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        if tenant_id_regex().is_match(s) {
            TenantId(TenantIdInner::Valid(Arc::from(s)))
        } else {
            let reason = if s.is_empty() {
                "tenant ID must not be empty".to_string()
            } else if s.len() > 64 {
                format!("tenant ID length {} exceeds maximum of 64", s.len())
            } else {
                // Do NOT echo the raw input — it may contain attacker-controlled data
                // (null bytes, Unicode, shell metacharacters) that would constitute a
                // log-injection vector if forwarded to a log aggregator or MCP response.
                "tenant ID contains invalid characters; allowed: [a-zA-Z0-9_-]".to_string()
            };
            TenantId(TenantIdInner::Invalid(reason))
        }
    }

    /// Bypass validation — for test fixtures in downstream crates only.
    ///
    /// Constructs a valid-state `TenantId` without running the regex check.
    /// Used by `prism-credentials` test helpers before S-1.01 validation
    /// was finalized; kept for test-writer compatibility.
    ///
    /// MUST NOT be called from production code.
    pub fn new_unchecked(s: &str) -> Self {
        TenantId(TenantIdInner::Valid(Arc::from(s)))
    }

    /// Returns `true` if this `TenantId` was constructed from a valid string.
    pub fn is_ok(&self) -> bool {
        matches!(self.0, TenantIdInner::Valid(_))
    }

    /// Returns `true` if this `TenantId` was constructed from an invalid string.
    pub fn is_err(&self) -> bool {
        matches!(self.0, TenantIdInner::Invalid(_))
    }

    /// Returns `self` if valid; panics with a default message if invalid.
    pub fn unwrap(self) -> Self {
        match &self.0 {
            TenantIdInner::Valid(_) => self,
            TenantIdInner::Invalid(reason) => {
                panic!("called TenantId::unwrap() on an invalid TenantId: {reason}")
            }
        }
    }

    /// Returns `self` if valid; panics with `msg` if invalid.
    pub fn expect(self, msg: &str) -> Self {
        match &self.0 {
            TenantIdInner::Valid(_) => self,
            TenantIdInner::Invalid(reason) => {
                panic!("{msg}: {reason}")
            }
        }
    }

    /// Returns the `PrismError` if this `TenantId` is invalid; panics if valid.
    pub fn unwrap_err(self) -> PrismError {
        match self.0 {
            TenantIdInner::Invalid(reason) => PrismError::InvalidTenantId { reason },
            TenantIdInner::Valid(_) => {
                panic!("called TenantId::unwrap_err() on a valid TenantId")
            }
        }
    }

    /// Return the inner string slice. Panics if the TenantId is invalid.
    pub fn as_str(&self) -> &str {
        match &self.0 {
            TenantIdInner::Valid(s) => s,
            TenantIdInner::Invalid(reason) => {
                panic!("called TenantId::as_str() on an invalid TenantId: {reason}")
            }
        }
    }
}

impl PartialEq for TenantId {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (TenantIdInner::Valid(a), TenantIdInner::Valid(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for TenantId {}

impl std::hash::Hash for TenantId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            TenantIdInner::Valid(s) => s.hash(state),
            TenantIdInner::Invalid(_) => 0u8.hash(state),
        }
    }
}

impl Serialize for TenantId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            TenantIdInner::Valid(inner) => s.serialize_str(inner),
            TenantIdInner::Invalid(reason) => Err(serde::ser::Error::custom(format!(
                "cannot serialize invalid TenantId: {reason}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        let tid = TenantId::new(&raw);
        if tid.is_ok() {
            Ok(tid)
        } else {
            Err(serde::de::Error::custom(format!(
                "invalid tenant ID: {}",
                match &tid.0 {
                    TenantIdInner::Invalid(r) => r.as_str(),
                    _ => unreachable!(),
                }
            )))
        }
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            TenantIdInner::Valid(s) => f.write_str(s),
            TenantIdInner::Invalid(reason) => write!(f, "<invalid TenantId: {reason}>"),
        }
    }
}
