//! Request/response types for the Cyberint DTU clone.

use serde::{Deserialize, Serialize};

/// Status record for an alert — stored in `CyberintState::alert_store`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatus {
    pub alert_id: String,
    pub status: String,
    pub closed: bool,
}

/// Alert object as returned by the Cyberint API (loaded from fixture).
///
/// AC-001 (S-DEMO-ENRICHMENT-PIVOT-003): adds real-schema IOC fields:
/// `ioc: Option<Ioc>` (singleton — flagged PENDING-LIVE-VALIDATION, may be removed),
/// `iocs: Vec<Ioc>` (list form — CONFIRMED via Check Point sk182975),
/// `alert_data: Option<AlertData>` (network IOC surface — url CONFIRMED).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: String,
    pub title: String,
    pub severity: String,
    pub status: String,
    pub created_at: serde_json::Value,
    pub source: String,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub affected_assets: Vec<serde_json::Value>,
    /// Singleton top-level IOC field — PENDING-LIVE-VALIDATION.
    /// No public-documentation basis found (BC-2.06.019 Cyberint row).
    /// Retained pending live-tenant validation; remove if absent in live API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ioc: Option<Ioc>,
    /// List of IOC objects — CONFIRMED via Check Point sk182975 + FortiSOAR connector.
    /// Each element uses serde dual-alias to tolerate both key conventions
    /// (`type`/`value` OR `ioc_type`/`ioc_value`) — see `Ioc` struct.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iocs: Vec<Ioc>,
    /// Network IOC surface: ip, domain, url fields on the alert.
    /// `url` CONFIRMED; `ip`/`domain` UNCONFIRMED-plausible (BC-2.06.019).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_data: Option<AlertData>,
}

/// Threat intelligence item (loaded from fixture).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatItem {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub threat_type: String,
    pub severity: String,
    pub confidence: String,
    pub iocs: Vec<Ioc>,
}

/// Indicator of Compromise — used on both `ThreatItem.iocs` and `Alert.iocs` / `Alert.ioc`.
///
/// AC-001 (S-DEMO-ENRICHMENT-PIVOT-003): serde dual-alias required per BC-2.06.019
/// INCONCLUSIVE inner-key determination. The real Cyberint alerts API inner-key form is
/// not confirmed in any public documentation. DTU tolerates both forms at deserialization:
/// - `"type"` (short form) AND `"ioc_type"` (feed convention) → `ioc_type` field
/// - `"value"` (short form) AND `"ioc_value"` (feed convention) → `value` field
///
/// Serialization output uses the primary key names (`"type"`, `"value"`) — the Rust field
/// names drive fixture generation output regardless of which wire form the live API uses.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ioc {
    /// IOC type string — deserialized from `"type"` (short form, serde rename) or
    /// `"ioc_type"` (feed convention, serde alias). BC-2.06.019 dual-alias requirement.
    #[serde(rename = "type", alias = "ioc_type")]
    pub ioc_type: String,
    /// IOC value string — deserialized from `"value"` (default) or `"ioc_value"` (alias).
    #[serde(alias = "ioc_value")]
    pub value: String,
}

/// Network IOC surface on a Cyberint alert record.
///
/// AC-001 (S-DEMO-ENRICHMENT-PIVOT-003): added per BC-2.06.019 Per-Sensor IOC-Surface
/// Matrix. `url` CONFIRMED via FortiSOAR connector; `ip`/`domain` UNCONFIRMED-plausible.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Domain-level error type for Cyberint DTU operations.
#[derive(Debug)]
pub enum CyberintError {
    AlertNotFound(String),
    AlertAlreadyClosed(String),
    Unauthorized,
}

impl std::fmt::Display for CyberintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CyberintError::AlertNotFound(id) => write!(f, "alert not found: {id}"),
            CyberintError::AlertAlreadyClosed(id) => write!(f, "alert already closed: {id}"),
            CyberintError::Unauthorized => write!(f, "unauthorized"),
        }
    }
}

impl std::error::Error for CyberintError {}
