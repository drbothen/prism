//! NVD API 2.0 response types.
//!
//! Structs mirror the NVD REST API 2.0 JSON schema exactly so that Prism's
//! `vuln_context` infusion plugin can deserialize DTU responses identically to
//! live NVD responses.

use serde::{Deserialize, Serialize};

/// Top-level NVD CVE list response (single or bulk fetch).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CveResponse {
    pub results_per_page: u32,
    pub start_index: u32,
    pub total_results: u32,
    pub format: String,
    pub version: String,
    pub timestamp: String,
    pub vulnerabilities: Vec<VulnerabilityWrapper>,
}

/// Outer wrapper returned in the `vulnerabilities` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityWrapper {
    pub cve: CveRecord,
}

/// A single CVE record per NVD API 2.0 shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CveRecord {
    pub id: String,
    pub source_identifier: String,
    pub published: String,
    pub last_modified: String,
    pub vuln_status: String,
    pub descriptions: Vec<LangValue>,
    pub metrics: CveMetrics,
    pub weaknesses: Vec<Weakness>,
    pub configurations: Vec<serde_json::Value>,
    pub references: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cisa_kev_vuln_added: Option<String>,
}

/// `{"lang": "en", "value": "..."}` pair used in descriptions and weakness lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangValue {
    pub lang: String,
    pub value: String,
}

/// CVSS metrics block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CveMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvss_metric_v31: Option<Vec<CvssMetricV31>>,
}

/// One CVSS v3.1 metric entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CvssMetricV31 {
    pub source: String,
    pub r#type: String,
    pub cvss_data: CvssData,
    pub exploitability_score: f64,
    pub impact_score: f64,
}

/// Core CVSS scoring data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CvssData {
    pub version: String,
    pub vector_string: String,
    pub base_score: f64,
    pub base_severity: String,
}

/// CWE weakness entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weakness {
    pub source: String,
    pub r#type: String,
    pub description: Vec<LangValue>,
}

/// Error response body returned for 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvdError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cveId")]
    pub cve_id: Option<String>,
}

/// Request-count response from the `GET /dtu/request-count/{cve_id}` test API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCountResponse {
    pub cve_id: String,
    pub count: u32,
}

/// Rate-limit bucket tracking state.
#[derive(Debug, Clone)]
pub struct RateLimitBucket {
    pub count: u32,
    pub window_start: std::time::Instant,
    pub limit: u32,
}

impl RateLimitBucket {
    /// Unauthenticated: 5 requests per 30-second window.
    pub fn unauthenticated() -> Self {
        Self {
            count: 0,
            window_start: std::time::Instant::now(),
            limit: 5,
        }
    }

    /// Authenticated: 50 requests per 30-second window.
    pub fn authenticated() -> Self {
        Self {
            count: 0,
            window_start: std::time::Instant::now(),
            limit: 50,
        }
    }
}
