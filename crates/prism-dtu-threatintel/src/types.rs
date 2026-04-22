//! Response types for the Threat Intel Aggregator DTU.
//!
//! These structs define the unified aggregated output shape consumed by
//! Prism's infusion cache — not raw per-provider formats.

use serde::{Deserialize, Serialize};

/// Unified threat intelligence lookup response.
/// Returned by IP, domain, and hash lookup endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelResponse {
    /// The lookup value (IP address, domain, or hash) that was queried.
    pub lookup_value: String,
    /// Aggregated threat score from 0 (benign) to 100 (high confidence malicious).
    pub threat_score: u32,
    /// Whether any source confirmed this as a known malicious indicator.
    pub threat_is_known_malicious: bool,
    /// List of threat intelligence sources that contributed data.
    pub threat_sources: Vec<String>,
}

/// Key identifying which fixture to use for a lookup value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixtureKey {
    Malicious,
    Benign,
    Unknown,
}

/// Request body for `POST /dtu/configure`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureRequest {
    /// Set rate-limit threshold (requests before 429).
    pub rate_limit_after: Option<u32>,
    /// IP address to add to the fixture registry.
    pub ip: Option<String>,
    /// Fixture to assign to the given IP.
    pub fixture: Option<FixtureKey>,
}

/// Error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}
