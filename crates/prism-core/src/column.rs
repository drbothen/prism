//! Column type and options definitions shared across Prism crates.
//!
//! These are the canonical column primitives from S-1.01 (foundational types).
//! prism-spec-engine uses these for SensorTableDescriptor column schemas.

use serde::{Deserialize, Serialize};

/// The data type of a column in a sensor table.
///
/// Corresponds to the TOML `type` field in a `[[table.columns]]` entry.
/// Maps to Arrow schema types when prism-query registers DataFusion tables.
///
/// `#[non_exhaustive]`: forward-compat for TOML schema evolution — new column types
/// (e.g., `Binary`, `Uuid`, `Decimal`) may be added without a semver bump.
/// External matchers MUST include a wildcard `_ => {}` arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnType {
    /// UTF-8 string. Arrow: Utf8.
    String,
    /// 64-bit signed integer. Arrow: Int64.
    Integer,
    /// 64-bit float. Arrow: Float64.
    Float,
    /// Boolean. Arrow: Boolean.
    Boolean,
    /// Microsecond-precision UTC timestamp. Arrow: TimestampMicrosecond.
    Datetime,
    /// JSON blob stored as UTF-8 string. Arrow: Utf8 (JSON string).
    Json,
}

/// Column-level options controlling query engine and normalizer behavior.
///
/// Maps to the TOML `options` array in a `[[table.columns]]` entry.
/// Multiple options can be combined.
///
/// `#[non_exhaustive]`: forward-compat for TOML schema evolution — new options
/// may be added (e.g., `Encrypted`, `PII`) without a semver bump.
/// External matchers MUST include a wildcard `_ => {}` arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ColumnOptions {
    /// Column must be present in WHERE clause (DI-021).
    Required,
    /// Column has an index hint for push-down optimization.
    Index,
    /// Column triggers an enrichment step when queried.
    Additional,
    /// Column is excluded from schema introspection.
    Hidden,
    /// Column is optimized for range queries.
    Optimized,
}
