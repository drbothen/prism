//! Foundational newtypes used across the Prism platform.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::tenant::TenantId;

/// Client identifier in the MSSP context — wraps `TenantId`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub TenantId);

/// Analyst identifier — validated same rules as TenantId (alphanumeric, `_`, `-`, 1–64 chars).
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
    /// Construct an `AnalystId`, applying the same validation as `TenantId`.
    pub fn new(s: &str) -> Result<Self, crate::error::PrismError> {
        todo!("S-1.01: implement AnalystId validation")
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

/// Enumeration of first-class sensor types.
///
/// Lives in prism-core to avoid circular dependency: error variants and
/// prism-query virtual fields reference sensor types; sensor adapters are
/// implemented in prism-sensors (S-2.06).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    CrowdStrike,
    Cyberint,
    Claroty,
    Armis,
}

impl std::fmt::Display for SensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SensorType::CrowdStrike => "crowdstrike",
            SensorType::Cyberint => "cyberint",
            SensorType::Claroty => "claroty",
            SensorType::Armis => "armis",
        };
        write!(f, "{s}")
    }
}

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
