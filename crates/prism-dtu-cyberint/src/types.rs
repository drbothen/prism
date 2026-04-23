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

/// Indicator of Compromise within a threat item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ioc {
    #[serde(rename = "type")]
    pub ioc_type: String,
    pub value: String,
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
