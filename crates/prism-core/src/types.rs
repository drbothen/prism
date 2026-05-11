//! Foundational newtypes used across the Prism platform.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tenant::OrgSlug;

/// Client identifier in the MSSP context — wraps `OrgSlug`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub OrgSlug);

/// Analyst identifier — validated same rules as OrgSlug (alphanumeric, `_`, `-`, 1–64 chars).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnalystId(Arc<str>);

impl Serialize for AnalystId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AnalystId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        AnalystId::new(&raw).map_err(serde::de::Error::custom)
    }
}

impl AnalystId {
    /// Construct an `AnalystId`, applying the same validation as `OrgSlug`.
    pub fn new(s: &str) -> Result<Self, crate::error::PrismError> {
        // Re-use OrgSlug's validated regex: ^[a-zA-Z0-9_-]{1,64}$
        // OrgSlug::new returns an OrgSlug with embedded validity state.
        let slug = crate::tenant::OrgSlug::new(s);
        if slug.is_ok() {
            Ok(AnalystId(Arc::from(slug.as_str())))
        } else {
            Err(crate::error::PrismError::InvalidAnalystId {
                reason: if s.is_empty() {
                    "analyst ID must not be empty".to_string()
                } else if s.len() > 64 {
                    format!("analyst ID length {} exceeds maximum of 64", s.len())
                } else {
                    // Do NOT echo the raw input — same log-injection guard as OrgSlug.
                    "analyst ID contains invalid characters; allowed: [a-zA-Z0-9_-]".to_string()
                },
            })
        }
    }
}

/// OCSF severity_id (0–5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SeverityId(pub u32);

/// UTC timestamp. All Prism timestamps are UTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub DateTime<Utc>);

/// Column type used by `InternalTableDescriptor` (S-2.03) for schema
/// definitions without an Arrow dependency.
///
/// prism-query (S-3.02) converts these to Arrow `DataType` at query time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
    Text,
    Int64,
    UInt64,
    Float64,
    Bool,
    Timestamp,
    Json,
    Bytes,
}
