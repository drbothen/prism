//! TenantId newtype — validates and holds a tenant identifier string.
//!
//! Validation: `^[a-zA-Z0-9_-]{1,64}$` (compiled once via OnceLock).
//! Inner type: `Arc<str>` for cheap cloning.

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

/// A validated, immutable tenant identifier.
///
/// Use `TenantId::new(s)` to construct. Cheap to clone (`Arc<str>` inner).
/// Rejects: empty string, length > 64, characters outside `[a-zA-Z0-9_-]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TenantId(Arc<str>);

impl Serialize for TenantId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        TenantId::new(&raw).map_err(serde::de::Error::custom)
    }
}

impl TenantId {
    /// Validate and construct a `TenantId` from a string slice.
    ///
    /// Returns `Err(PrismError::InvalidTenantId)` if `s` does not match
    /// `^[a-zA-Z0-9_-]{1,64}$`.
    pub fn new(s: &str) -> Result<Self, PrismError> {
        if tenant_id_regex().is_match(s) {
            Ok(TenantId(Arc::from(s)))
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
            Err(PrismError::InvalidTenantId { reason })
        }
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
