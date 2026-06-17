//! ThreatIntel enrichment WASM plugin — S-DEMO-ENRICHMENT-PIVOT-002.
//!
//! Dispatches an IOC value to the correct prism-dtu-threatintel HTTP route based on
//! input type:
//! - IP: `GET /v3/ip/:ip` (confirmed 2026-06-12 lookup.rs:162)
//! - domain: `GET /v3/domain/:domain` (lookup.rs:187)
//! - hash: `GET /v3/hash/:hash` (lookup.rs:214)
//!
//! Auth: `?key=<api_key>` query param OR `Authorization: Bearer <token>` header
//! (NOT X-Admin-Token — that is admin-surface only per lookup.rs lines 20-53).
//!
//! Response shape (confirmed 2026-06-12 from prism-dtu-threatintel/src/types.rs):
//! - `threat_score`: Integer (i64)
//! - `threat_is_known_malicious`: Boolean (bool)
//! - `threat_sources`: Json array of source strings (NOT `threat_source` singular — SAP-2)
//! - `greynoise_classification`: String
//! - `abuseipdb_confidence_score`: Integer
//! - `virustotal_detections`: Integer
//!
//! HTTP goes through host WIT import `host.http-request` (U9: WASM guests have no sockets).
//! The 30s timeout applies to the HOST reqwest client in host_functions.rs, not this crate.
//!
//! # Build
//! ```sh
//! cargo build --manifest-path crates/plugins/prism-threatintel-infusion/Cargo.toml \
//!   --target wasm32-wasip1 --release
//! wasm-tools component new target/wasm32-wasip1/release/prism_threatintel_infusion.wasm \
//!   --adapt wasi_snapshot_preview1=tests/fixtures/wasi_snapshot_preview1.wasm \
//!   -o crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
//! ```
//! See Justfile recipe `build-plugin-threatintel-infusion`.

// ---------------------------------------------------------------------------
// WIT bindings — host.http-request import
// ---------------------------------------------------------------------------
// wit_bindgen::generate! block will be added at implementation time once the
// WIT file path is confirmed. For stubs, we declare the minimal types needed
// to make the function signatures correct.

/// Input to the ThreatIntel enrichment call, received via the host WIT ABI.
///
/// The WASM host calls `enrich(ioc_value, ioc_type, api_key)` and receives
/// a JSON-encoded result object.
#[derive(Debug)]
pub struct EnrichInput {
    /// The IOC value to look up (IP address, domain, or file hash).
    pub ioc_value: String,
    /// The IOC type discriminant: `"ip"`, `"domain"`, or `"hash"`.
    pub ioc_type: String,
    /// The API key for ThreatIntel authentication (resolved by host from keyring).
    pub api_key: String,
}

/// Output from the ThreatIntel enrichment call.
///
/// JSON-encoded by the plugin and decoded by the host before injecting into
/// the DataFusion row as per the infusion field schema.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnrichOutput {
    /// Threat intelligence score (0–100).
    pub threat_score: i64,
    /// True if the IOC is in the known-malicious registry.
    pub threat_is_known_malicious: bool,
    /// Array of source names that flagged this IOC as malicious.
    /// Declared as Json type in infusion TOML (confirmed 2026-06-12: array, not string).
    pub threat_sources: Vec<String>,
}

// ---------------------------------------------------------------------------
// Plugin entry point
// ---------------------------------------------------------------------------

/// Enrich a single IOC value via the DTU lookup endpoint.
///
/// Dispatches on `ioc_type` to the correct DTU route:
/// - `"ip"` → `GET /v3/ip/:ip`
/// - `"domain"` → `GET /v3/domain/:domain`
/// - `"hash"` → `GET /v3/hash/:hash`
///
/// Auth: `?key=<api_key>` query param (or Authorization: Bearer <token>).
/// HTTP call goes through host WIT import `host.http-request` (U9).
///
/// Returns JSON-encoded `EnrichOutput` on success, or an error string.
///
/// # WIT ABI
/// This function will be bound to the WIT export defined in prism-infusion-plugin.wit
/// once the WIT binding code generation is added. The signature mirrors the WIT interface.
pub fn enrich_ioc(ioc_value: &str, ioc_type: &str, api_key: &str) -> Result<String, String> {
    todo!(
        "enrich_ioc stub — S-DEMO-ENRICHMENT-PIVOT-002 implementer: \
         dispatch on ioc_type ({ioc_type}) to correct DTU route via host.http-request WIT import; \
         auth via ?key= param or Bearer header; \
         parse threat_sources as JSON array (NOT singular threat_source string); \
         return JSON-encoded EnrichOutput"
    )
}

/// Validate that the given string is a plausible IP address format.
///
/// Used to distinguish IP IOCs from domain/hash IOCs before routing to the
/// correct DTU endpoint path.
pub fn is_ip_address(value: &str) -> bool {
    todo!(
        "is_ip_address stub — classify IOC as IP for DTU route dispatch"
    )
}

/// Validate that the given string is a plausible domain name format.
pub fn is_domain(value: &str) -> bool {
    todo!(
        "is_domain stub — classify IOC as domain for DTU route dispatch"
    )
}

/// Validate that the given string is a plausible file hash (MD5/SHA1/SHA256).
pub fn is_hash(value: &str) -> bool {
    todo!(
        "is_hash stub — classify IOC as file hash for DTU route dispatch"
    )
}
