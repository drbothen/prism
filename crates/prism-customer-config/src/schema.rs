use serde::Deserialize;
use uuid::Uuid;

/// Top-level customer configuration loaded from `customers/{org_slug}.toml`.
///
/// ADR-010 §2.2 required fields. `deny_unknown_fields` is mandatory per ADR-010 §2.2
/// and BC-3.3.004 postcondition R-CUST-010.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerConfig {
    /// Schema version gate — must equal 1 in Wave 3 (BC-3.3.003).
    pub schema_version: u64,
    /// UUID v7 unique organization identifier (R-CUST-003).
    pub org_id: Uuid,
    /// URL-safe slug matching the filename stem (R-CUST-002).
    pub org_slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// DTU adapter blocks (ADR-010 §2.3).
    #[serde(default)]
    pub dtu: Vec<DtuBlock>,
    /// Optional shared infrastructure block (ADR-010 §2.4).
    pub shared_infra: Option<SharedInfra>,
}

/// A single DTU adapter declaration within a customer config.
///
/// `deny_unknown_fields` rejects `allow_shared_override` (ADR-007 §7 OQ-1 DEFERRED)
/// and any other undeclared field, producing E-CFG-010.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DtuBlock {
    /// DTU type string (e.g. "claroty", "crowdstrike", "armis"). Checked against
    /// the compiled DTU_DEFAULT_MODE registry by the validator.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Deployment mode: "client" or "shared".
    pub mode: String,
    /// Opaque credential reference with a recognized scheme prefix (R-CRED-007).
    pub credential_ref: String,
    /// Path to the sensor spec file; required when mode = "client" (R-CUST-014).
    pub spec: Option<String>,
    /// Optional simulation / synthetic data parameters.
    pub data: Option<DtuData>,
}

/// Optional simulation data parameters for a DTU block.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DtuData {
    /// Archetype identifier (e.g. "enterprise-healthcare").
    pub archetype: Option<String>,
    /// Scale multiplier; must be a positive finite float (R-CUST-008).
    pub scale: Option<f64>,
    /// RNG seed for deterministic data generation (R-CUST-007).
    pub seed: Option<u64>,
}

/// Shared infrastructure block (ADR-010 §2.4).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedInfra {
    /// Opaque credential reference for shared infrastructure access.
    pub credential_ref: Option<String>,
    /// Optional endpoint URL.
    pub endpoint: Option<String>,
}
