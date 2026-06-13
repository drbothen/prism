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
///
/// F10 / finding ⑫ (2026-06-10 review): `deny_unknown_fields` on every config
/// struct — a typo'd key silently ignored means the demo runs with defaults
/// the operator believes they overrode. Unknown keys are a parse ERROR.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

// ---------------------------------------------------------------------------
// Story A: ScenarioConfig stub (BC-2.06.018 / ADR-036 §2.4)
// ---------------------------------------------------------------------------

/// Per-clone scenario configuration (the `[clones.<name>.scenario]` section).
///
/// Used by `build_clone_pairs` to determine whether to call `new_with_seed` and
/// to derive the `ScenarioEntityCatalog` for cross-DTU entity coherence.
///
/// # ADR-036 §2.4 — ScenarioConfig fields
///
/// All fields have defaults. When `enabled = false` (default), the clone uses the
/// backward-compatible `new()` static-JSON path.
///
/// # Story A stub
///
/// Fields present; `build_clone_pairs` integration is Gate 4's job.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ScenarioConfig {
    /// When `true`, `build_clone_pairs` calls `new_with_seed` instead of `new()`.
    ///
    /// Requires `org_id` to be set in `CloneConfig`; absence produces E-DEMO-004.
    #[serde(default)]
    pub enabled: bool,

    /// Scenario archetype string. Valid values: `"compromised_endpoint"`, `"healthy"`.
    ///
    /// Unrecognized values produce E-DEMO-003 at construction time.
    #[serde(default = "default_scenario_archetype")]
    pub archetype: String,

    /// Unix epoch seconds for scenario start time. `None` = start at construction time.
    #[serde(default)]
    pub scenario_start_secs: Option<i64>,

    /// 4-entry array of cumulative `activates_after_secs` thresholds for stages 1..=4.
    ///
    /// Stage 0 (Baseline) always activates at 0 (no entry needed).
    /// Empty = use archetype defaults `[60, 180, 360, 600]`.
    #[serde(default)]
    pub stage_duration_secs: Vec<u64>,
}

fn default_scenario_archetype() -> String {
    "compromised_endpoint".to_string()
}

/// Configuration for a single DTU clone (e.g. `[clones.crowdstrike]`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Initial access token to register in the static allowlist (Cyberint only).
    ///
    /// When set, `build_clone_pairs` calls `BehavioralClone::configure()` with
    /// `{"access_token": "<value>"}` on the Cyberint clone immediately after construction.
    /// This seeds the allowlist so the clone accepts `Cookie: access_token=<value>`
    /// on data requests without requiring a separate `/dtu/configure` POST.
    ///
    /// ADR-031 §D3-a: Cyberint uses static cookie auth; this is the test-harness
    /// mechanism for seeding the allowlist at startup time.
    #[serde(default)]
    pub initial_access_token: Option<String>,

    // -----------------------------------------------------------------------
    // Story A additions: org_id + scenario (BC-2.06.018 / ADR-036 §2.4)
    // -----------------------------------------------------------------------
    /// Org UUID (hyphenated string) for this demo client.
    ///
    /// Required when `scenario.enabled = true` for any clone in this config block.
    /// Parsed as `uuid::Uuid` and converted to `OrgId` (`[u8; 16]`) by `build_clone_pairs`.
    ///
    /// - Absence when scenario.enabled = true → E-DEMO-004 at construction time.
    /// - Non-UUID value → E-DEMO-005 at construction time.
    /// - Optional (may be `None`) when scenario.enabled = false — backward compatible.
    ///
    /// ADR-036 §2.4 / BC-2.06.018 "New Config Requirement"
    #[serde(default)]
    pub org_id: Option<String>,

    /// Per-clone scenario configuration (the `[clones.<name>.scenario]` subsection).
    ///
    /// When `None` or `enabled = false`, the clone uses the backward-compatible `new()` path.
    /// ADR-036 §2.4
    #[serde(default)]
    pub scenario: Option<ScenarioConfig>,
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
            initial_access_token: None,
            // Story A fields: default to None (backward-compatible path)
            org_id: None,
            scenario: None,
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
        toml::from_str(toml_str).map_err(|e| anyhow::anyhow!("Invalid TOML in demo config: {}", e))
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

// ---------------------------------------------------------------------------
// F10 / finding ⑫ (2026-06-10 review): deny_unknown_fields strictness tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::DemoConfig;

    /// Non-regression: a known-good minimal config still parses.
    #[test]
    fn test_f10_valid_config_parses() {
        let toml = r#"
            [harness]
            bind = "127.0.0.1"

            [clones.crowdstrike]
            enabled = true
            seed = 7

            [clones.crowdstrike.scenario]
            enabled = false
        "#;
        let cfg = DemoConfig::from_str(toml).expect("valid config must parse");
        assert_eq!(cfg.clones.crowdstrike.seed, 7);
    }

    /// F10: an unknown key anywhere in the demo TOML must be a parse ERROR —
    /// a typo'd key silently ignored means the demo runs with defaults the
    /// operator believes they overrode.
    #[test]
    fn test_f10_unknown_keys_rejected_at_every_level() {
        let cases: &[(&str, &str)] = &[
            ("top-level", "unknown_top = true\n"),
            ("[harness]", "[harness]\nbnd = \"127.0.0.1\"\n"), // typo'd 'bind'
            ("[clones]", "[clones]\nnotaclone = {}\n"),
            (
                "[clones.crowdstrike]",
                "[clones.crowdstrike]\nsede = 7\n", // typo'd 'seed'
            ),
            (
                "[clones.crowdstrike.scenario]",
                "[clones.crowdstrike.scenario]\narchetyp = \"compromised_endpoint\"\n", // typo'd 'archetype'
            ),
        ];
        for (level, toml) in cases {
            assert!(
                DemoConfig::from_str(toml).is_err(),
                "unknown key at {level} must be rejected (deny_unknown_fields — \
                 finding ⑫, 2026-06-10 review), but it parsed: {toml:?}"
            );
        }
    }
}
