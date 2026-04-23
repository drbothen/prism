//! `DemoConfig` — TOML schema for the demo harness configuration file.
//!
//! Canonical config location: `configs/demo.toml`.
//!
//! All fields have defaults. Minimal config is `[clones.<name>] enabled = true`.
//!
//! # Security (R-DEMO-001)
//!
//! Setting any `bind` field to a non-loopback address requires BOTH the `--bind-any`
//! CLI flag AND `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK`.
//! A `[SECURITY WARNING]` log message is printed at startup listing all admin URLs.

use serde::{Deserialize, Serialize};

/// Top-level demo harness configuration.
///
/// Loaded from a TOML file, e.g. `configs/demo.toml`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DemoConfig {
    /// Global harness settings.
    #[serde(default)]
    pub harness: HarnessConfig,
    /// Per-clone configurations.
    #[serde(default)]
    pub clones: ClonesConfig,
}

/// Global harness configuration (the `[harness]` section).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HarnessConfig {
    /// Bind IP for the admin/health listener. Defaults to loopback.
    #[serde(default = "default_bind_ip")]
    pub bind: String,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            bind: default_bind_ip(),
        }
    }
}

fn default_bind_ip() -> String {
    "127.0.0.1".to_string()
}

/// Per-clone configuration container (the `[clones]` section).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ClonesConfig {
    #[serde(default)]
    pub crowdstrike: CloneConfig,
    #[serde(default)]
    pub claroty: CloneConfig,
    #[serde(default)]
    pub cyberint: CloneConfig,
    #[serde(default)]
    pub armis: CloneConfig,
    #[serde(default)]
    pub threatintel: CloneConfig,
    #[serde(default)]
    pub nvd: CloneConfig,
}

/// Configuration for a single DTU clone (e.g. `[clones.crowdstrike]`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloneConfig {
    /// Whether this clone is enabled. When `false`, the clone is not started.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Bind IP for this clone. Defaults to loopback.
    #[serde(default = "default_bind_ip")]
    pub bind: String,
    /// Port to bind. `0` means OS-assigned ephemeral.
    #[serde(default)]
    pub port: u16,
    /// Fixture set identifier (e.g. `"default"`).
    #[serde(default = "default_fixture_set")]
    pub fixture_set: String,
    /// Initial failure mode (e.g. `"None"`, `"RateLimit"`, etc.).
    #[serde(default = "default_failure_mode")]
    pub initial_failure_mode: String,
    /// RNG seed for deterministic response generation.
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Whether to use TLS for this clone.
    #[serde(default)]
    pub tls: bool,
    /// When `true`: a bind failure logs WARN and skips this clone; others continue.
    /// When `false` (default): a bind failure aborts startup (AC-11 cleanup path).
    #[serde(default)]
    pub continue_on_error: bool,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: default_bind_ip(),
            port: 0,
            fixture_set: default_fixture_set(),
            initial_failure_mode: default_failure_mode(),
            seed: default_seed(),
            tls: false,
            continue_on_error: false,
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_fixture_set() -> String {
    "default".to_string()
}

fn default_failure_mode() -> String {
    "None".to_string()
}

fn default_seed() -> u64 {
    42
}

impl std::str::FromStr for DemoConfig {
    type Err = anyhow::Error;

    fn from_str(toml_str: &str) -> anyhow::Result<Self> {
        toml::from_str(toml_str)
            .map_err(|e| anyhow::anyhow!("Invalid TOML in demo config: {}", e))
    }
}

impl DemoConfig {
    /// Load configuration from a TOML file at `path`.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config {:?}: {}", path, e))?;
        contents.parse()
    }

    /// Parse configuration from a TOML string.
    ///
    /// This inherent method exists so callers do not need to import
    /// `std::str::FromStr` explicitly. It delegates to the `FromStr` impl.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(toml_str: &str) -> anyhow::Result<Self> {
        toml_str.parse()
    }
}
