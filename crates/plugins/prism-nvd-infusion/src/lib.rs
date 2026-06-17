//! NVD CVSS enrichment WASM plugin — S-DEMO-ENRICHMENT-PIVOT-002.
//!
//! Calls the NVD DTU clone to look up CVSS data for a given CVE ID:
//! `GET /rest/json/cves/2.0?cveId=<cve_id>` (confirmed 2026-06-12 cves.rs — NOT /nvd/cves/{id}).
//!
//! Auth: `?apiKey=<key>` query param (confirmed cves.rs CveQueryParams).
//!
//! Response envelope (confirmed 2026-06-12 from prism-dtu-nvd/src/types.rs,
//! all serde rename_all=camelCase wire names):
//! - `vulnerabilities[0].cve.id` (String — NOT cve_id)
//! - `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore` (f64 → Float)
//! - `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseSeverity` (String)
//! - `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.vectorString` (String)
//!
//! NOTE (U17/Ruling 1b): the enrich input field is `device_cves_first` (scalar String
//! projected by S-DEMO-ENRICHMENT-PIVOT-003 Armis fixture generator), NOT `device_cves[0]`.
//!
//! HTTP goes through host WIT import `host.http-request` (U9: WASM guests have no sockets).
//! The 30s timeout applies to the HOST reqwest client in host_functions.rs, not this crate.
//!
//! # Build
//! ```sh
//! cargo build --manifest-path crates/plugins/prism-nvd-infusion/Cargo.toml \
//!   --target wasm32-wasip1 --release
//! wasm-tools component new target/wasm32-wasip1/release/prism_nvd_infusion.wasm \
//!   --adapt wasi_snapshot_preview1=tests/fixtures/wasi_snapshot_preview1.wasm \
//!   -o crates/prism-spec-engine/plugins/nvd-lookup/nvd-lookup.prx
//! ```
//! See Justfile recipe `build-plugin-nvd-infusion`.

// ---------------------------------------------------------------------------
// Response deserialization types (camelCase wire names via serde)
// ---------------------------------------------------------------------------
// These mirror the NVD API response shape confirmed from prism-dtu-nvd/src/types.rs
// (all serde rename_all=camelCase). The WASM guest deserializes the HTTP response body
// from the host WIT import call.

/// Top-level NVD API response envelope.
///
/// Wire name confirmed from types.rs: `vulnerabilities` array (camelCase).
#[derive(Debug, serde::Deserialize)]
pub struct NvdCveResponse {
    /// Array of CVE vulnerability objects.
    pub vulnerabilities: Vec<NvdVulnerability>,
}

/// A single vulnerability entry in the NVD response.
#[derive(Debug, serde::Deserialize)]
pub struct NvdVulnerability {
    /// The CVE record with metrics.
    pub cve: NvdCveRecord,
}

/// CVE record fields used for enrichment.
///
/// Wire field `id` (NOT `cve_id`) per types.rs (rename_all=camelCase).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NvdCveRecord {
    /// CVE identifier (e.g. "CVE-2024-12345"). Wire name: `id` (NOT cve_id).
    pub id: String,
    /// CVSS metric arrays. Wire name: `metrics` (camelCase).
    pub metrics: NvdMetrics,
}

/// CVSS metrics container.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NvdMetrics {
    /// CVSS v3.1 metric array. Wire name: `cvssMetricV31` (camelCase).
    pub cvss_metric_v31: Vec<NvdCvssMetricV31>,
}

/// A single CVSS v3.1 metric entry.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NvdCvssMetricV31 {
    /// The CVSS data block. Wire name: `cvssData` (camelCase).
    pub cvss_data: NvdCvssData,
}

/// CVSS v3.1 score data.
///
/// Wire names are camelCase (confirmed from types.rs serde rename_all=camelCase):
/// - `baseScore` → base_score (f64)
/// - `baseSeverity` → base_severity (String)
/// - `vectorString` → vector_string (String)
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NvdCvssData {
    /// CVSS base score (0.0–10.0). Wire name: `baseScore` (camelCase).
    pub base_score: f64,
    /// CVSS base severity (e.g. "HIGH", "CRITICAL"). Wire name: `baseSeverity`.
    pub base_severity: String,
    /// CVSS vector string. Wire name: `vectorString`.
    pub vector_string: String,
}

/// Output from the NVD enrichment call.
///
/// JSON-encoded by the plugin and decoded by the host before injecting into
/// the DataFusion row per the infusion field schema.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnrichOutput {
    /// CVSS base score (f64 → Float in infusion TOML).
    pub cvss_base_score: f64,
    /// CVSS severity string (e.g. "HIGH").
    pub cvss_severity: String,
    /// CVSS vector string.
    pub cvss_vector: String,
}

// ---------------------------------------------------------------------------
// Plugin entry point
// ---------------------------------------------------------------------------

/// Enrich a single CVE ID via the NVD DTU clone endpoint.
///
/// Calls `GET /rest/json/cves/2.0?cveId=<cve_id>&apiKey=<api_key>`.
/// Parses the camelCase response envelope to extract CVSS data from
/// `vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData`.
///
/// Returns JSON-encoded `EnrichOutput` on success, or an error string.
///
/// # WIT ABI
/// This function will be bound to the WIT export defined in prism-infusion-plugin.wit.
/// The signature mirrors the WIT interface (to be added at implementation time).
pub fn enrich_cve(cve_id: &str, api_key: &str) -> Result<String, String> {
    todo!(
        "enrich_cve stub — S-DEMO-ENRICHMENT-PIVOT-002 implementer: \
         call GET /rest/json/cves/2.0?cveId={cve_id}&apiKey=<key> via host.http-request WIT import; \
         parse camelCase response envelope (baseScore/baseSeverity/vectorString); \
         return JSON-encoded EnrichOutput; \
         handle EC-002: CVE not in cve_registry → 404 → return None (not panic)"
    )
}
