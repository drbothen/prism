//! OrgSlug newtype — validates and holds an org slug string.
//!
//! Validation: `^[a-zA-Z0-9_-]{1,64}$` (compiled once via OnceLock).
//! Inner type: `Arc<str>` for cheap cloning.
//!
//! # API Design Note
//!
//! `OrgSlug::new` returns `OrgSlug` directly (infallible constructor that panics on invalid
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

/// Regex pattern for valid OrgSlug strings.
pub const ORG_SLUG_PATTERN: &str = r"^[a-zA-Z0-9_-]{1,64}$";

fn org_slug_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(ORG_SLUG_PATTERN).expect("ORG_SLUG_PATTERN is valid regex"))
}

/// Internal validity state.
#[derive(Clone, Debug)]
enum OrgSlugInner {
    Valid(Arc<str>),
    Invalid(String), // the reason string (not the raw input — avoids log injection)
}

/// An org slug with embedded validity state.
///
/// The analyst-visible display identifier for an organization (e.g., "acme-corp").
/// This is the human-readable counterpart to `OrgId` (internal UUID v7 canonical key).
///
/// Construct with `OrgSlug::new(s)`. The type exposes a `Result`-like interface
/// (`is_ok`, `is_err`, `unwrap`, `expect`, `unwrap_err`) so callers can choose
/// whether to handle errors explicitly or panic on invalid input.
///
/// Use `as_str()` only after confirming validity (or after `unwrap`/`expect`).
#[derive(Clone, Debug)]
pub struct OrgSlug(OrgSlugInner);

impl OrgSlug {
    /// Construct an `OrgSlug`, validating against `^[a-zA-Z0-9_-]{1,64}$`.
    ///
    /// The returned `OrgSlug` carries the validity state. Call `is_ok()` / `is_err()`
    /// to check, or `unwrap()` / `expect()` to get the value or panic.
    /// Use `unwrap_err()` to retrieve the `PrismError` on invalid input.
    pub fn new(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        if org_slug_regex().is_match(s) {
            OrgSlug(OrgSlugInner::Valid(Arc::from(s)))
        } else {
            let reason = if s.is_empty() {
                "org slug must not be empty".to_string()
            } else if s.len() > 64 {
                format!("org slug length {} exceeds maximum of 64", s.len())
            } else {
                // Do NOT echo the raw input — it may contain attacker-controlled data
                // (null bytes, Unicode, shell metacharacters) that would constitute a
                // log-injection vector if forwarded to a log aggregator or MCP response.
                "org slug contains invalid characters; allowed: [a-zA-Z0-9_-]".to_string()
            };
            OrgSlug(OrgSlugInner::Invalid(reason))
        }
    }

    /// Bypass-validation constructor for test fixtures only.
    ///
    /// # Precondition
    /// The caller MUST ensure `s` satisfies `^[a-zA-Z0-9_-]{1,64}$`. Passing an
    /// invalid string creates an `OrgSlug` in `Valid` state with invalid content
    /// — incorrect behavior downstream (credential store lookups, audit logging).
    ///
    /// Feature-gating is NOT applied: test fixtures across crate boundaries
    /// (`prism-credentials/`, `prism-query/tests/`, `prism-query/src/write_dispatch.rs`)
    /// need this function and `#[cfg(test)]` does not propagate downstream.
    ///
    /// # History (MED-001 update, S-PLUGIN-PREREQ-C fix-burst-2)
    /// The prior production caller in `prism-query/src/materialization.rs` was migrated
    /// to `OrgSlug::new()` with `"synthetic-unmapped"` sentinel fallback (HIGH-006 closure,
    /// fix-burst-1). No production caller of `new_unchecked` remains.
    ///
    /// # Audit guardrail
    /// Locked by `crates/prism-core/tests/new_unchecked_audit.rs` allowlist tuple
    /// `("tenant.rs", "OrgSlug")`. Any new `*::new_unchecked` in the workspace requires
    /// an explicit allowlist entry or feature-gate.
    pub fn new_unchecked(s: &str) -> Self {
        OrgSlug(OrgSlugInner::Valid(Arc::from(s)))
    }

    /// Returns `true` if this `OrgSlug` was constructed from a valid string.
    pub fn is_ok(&self) -> bool {
        matches!(self.0, OrgSlugInner::Valid(_))
    }

    /// Returns `true` if this `OrgSlug` was constructed from an invalid string.
    pub fn is_err(&self) -> bool {
        matches!(self.0, OrgSlugInner::Invalid(_))
    }

    /// Returns `self` if valid; panics with a default message if invalid.
    pub fn unwrap(self) -> Self {
        match &self.0 {
            OrgSlugInner::Valid(_) => self,
            OrgSlugInner::Invalid(reason) => {
                panic!("called OrgSlug::unwrap() on an invalid OrgSlug: {reason}")
            }
        }
    }

    /// Returns `self` if valid; panics with `msg` if invalid.
    pub fn expect(self, msg: &str) -> Self {
        match &self.0 {
            OrgSlugInner::Valid(_) => self,
            OrgSlugInner::Invalid(reason) => {
                panic!("{msg}: {reason}")
            }
        }
    }

    /// Returns the `PrismError` if this `OrgSlug` is invalid; panics if valid.
    pub fn unwrap_err(self) -> PrismError {
        match self.0 {
            OrgSlugInner::Invalid(reason) => PrismError::InvalidOrgSlug { reason },
            OrgSlugInner::Valid(_) => {
                panic!("called OrgSlug::unwrap_err() on a valid OrgSlug")
            }
        }
    }

    /// Return the inner string slice. Panics if the OrgSlug is invalid.
    pub fn as_str(&self) -> &str {
        match &self.0 {
            OrgSlugInner::Valid(s) => s,
            OrgSlugInner::Invalid(reason) => {
                panic!("called OrgSlug::as_str() on an invalid OrgSlug: {reason}")
            }
        }
    }
}

impl PartialEq for OrgSlug {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (OrgSlugInner::Valid(a), OrgSlugInner::Valid(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for OrgSlug {}

impl std::hash::Hash for OrgSlug {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            OrgSlugInner::Valid(s) => s.hash(state),
            OrgSlugInner::Invalid(_) => 0u8.hash(state),
        }
    }
}

impl Serialize for OrgSlug {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            OrgSlugInner::Valid(inner) => s.serialize_str(inner),
            OrgSlugInner::Invalid(reason) => Err(serde::ser::Error::custom(format!(
                "cannot serialize invalid OrgSlug: {reason}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for OrgSlug {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        let slug = OrgSlug::new(&raw);
        if slug.is_ok() {
            Ok(slug)
        } else {
            Err(serde::de::Error::custom(format!(
                "invalid org slug: {}",
                match &slug.0 {
                    OrgSlugInner::Invalid(r) => r.as_str(),
                    _ => unreachable!(),
                }
            )))
        }
    }
}

impl std::fmt::Display for OrgSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            OrgSlugInner::Valid(s) => f.write_str(s),
            OrgSlugInner::Invalid(reason) => write!(f, "<invalid OrgSlug: {reason}>"),
        }
    }
}

// ---------------------------------------------------------------------------
// Wave-3 deprecation alias — removed in Wave 4.
// ---------------------------------------------------------------------------

/// Deprecated: use [`OrgSlug`] instead.
///
/// This alias exists for one migration wave (Wave 3) to ease the mechanical rename.
/// All call sites should be updated to `OrgSlug` before Wave 4 begins.
#[deprecated(since = "3.0.0", note = "use OrgSlug")]
pub type TenantId = OrgSlug;
