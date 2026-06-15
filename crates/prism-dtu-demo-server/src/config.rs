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
// S-DEMO-LAUNCHER-CONSOLIDATION-001: MultiOrgDemoConfig + OrgConfig
//
// These are NEW top-level config types for `start-multi`. They MUST NOT modify
// DemoConfig (which has #[serde(deny_unknown_fields)] with 6 fixed ClonesConfig
// fields and cannot accept [orgs.*] without a parse error).
//
// Architecture Compliance Rule: MultiOrgDemoConfig is parsed ONLY by cmd_start_multi.
// The `start` subcommand continues to parse DemoConfig only.
// ---------------------------------------------------------------------------

/// Top-level config for `start-multi`. Loaded from `scripts/demo.toml`.
///
/// Separate from `DemoConfig` to avoid `deny_unknown_fields` clash — `DemoConfig`
/// has a fixed 6-sensor `ClonesConfig`; adding `[orgs.*]` to it would fail parsing.
///
/// # Architecture Compliance (S-DEMO-LAUNCHER-CONSOLIDATION-001)
///
/// `MultiOrgDemoConfig` is parsed ONLY in `cmd_start_multi`. The existing `start`
/// subcommand and `DemoConfig` are UNTOUCHED.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiOrgDemoConfig {
    /// Global harness settings (reuses existing `HarnessConfig`).
    #[serde(default)]
    pub harness: HarnessConfig,
    /// Per-org DTU clone fleet configs, keyed by org slug (e.g. `"org-a"`).
    ///
    /// Corresponds to the `[orgs.<slug>]` TOML section.
    pub orgs: std::collections::HashMap<String, OrgConfig>,
}

/// Configuration for one org's DTU clone fleet.
///
/// Corresponds to a `[orgs.<slug>]` TOML subsection within `MultiOrgDemoConfig`.
///
/// All fields use `deny_unknown_fields` — typo'd keys are a parse error.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrgConfig {
    /// UUID v7 hyphenated string for this org (e.g. `"0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"`).
    pub org_id: String,
    /// Sensor IDs for this org's DTU fleet (e.g. `["crowdstrike", "armis"]`).
    ///
    /// Valid values: `"crowdstrike"`, `"armis"`, `"claroty"`, `"cyberint"`.
    pub sensors: Vec<String>,
    /// RNG seed for deterministic, distinct fixture generation (INV-DISTINCT-DATA-001).
    ///
    /// org-a: 100, org-c: 200 (matching S-DEMO-004 seed assignments per the spec).
    pub seed: u64,
    /// Cyberint-only: initial access token registered in the clone's allowlist via
    /// `configure({"access_token": token})` post-construction (GAP-2 composite path).
    ///
    /// When `None`, the Cyberint clone's allowlist is empty at startup.
    #[serde(default)]
    pub initial_access_token: Option<String>,
}

impl MultiOrgDemoConfig {
    /// Load configuration from a TOML file at `path`.
    ///
    /// Mirrors the `DemoConfig::from_file` pattern. Validates org_id UUID strings after
    /// TOML deserialization — a malformed org_id returns `Err` with an actionable message
    /// naming the offending entry, rather than panicking later inside the factory closure
    /// (MED-B: the `.expect()` in `build_multi_clone_factory` is guarded by this validation).
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read multi-org config {:?}: {}", path, e))?;
        Self::from_str(&contents)
    }

    /// Parse configuration from a TOML string.
    ///
    /// Uses `deny_unknown_fields` on all nested structs — a typo'd key is a parse error,
    /// not a silently-ignored default (BC-2.06.001 invariant).
    ///
    /// After TOML deserialization, validates every `org_id` field in `orgs` as a
    /// well-formed UUID. A malformed org_id (e.g. `"not-a-uuid"`) returns `Err` with
    /// an actionable message naming the offending org entry and value (MED-B fix).
    /// This makes the claim in `build_multi_clone_factory`'s `.expect()` TRUE: by the
    /// time the factory runs, all org_ids have been validated as valid UUIDs at parse time.
    ///
    /// Mirrors the `DemoConfig::from_str` inherent method pattern.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(toml_str: &str) -> anyhow::Result<Self> {
        let cfg: Self = toml::from_str(toml_str)
            .map_err(|e| anyhow::anyhow!("Invalid TOML in multi-org demo config: {}", e))?;

        // MED-B: validate all org_id fields as UUIDs at parse time.
        // This ensures the `.expect()` in `build_multi_clone_factory` is a true
        // programming-error guard (not a user-input panic on a typo'd org_id).
        for (entry_name, org_cfg) in &cfg.orgs {
            uuid::Uuid::parse_str(&org_cfg.org_id).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid multi-org demo config: org '{}' has org_id '{}' which is not a \
                     valid UUID. Expected a hyphenated UUID v7 string, e.g. \
                     '0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000'.",
                    entry_name,
                    org_cfg.org_id
                )
            })?;
        }

        Ok(cfg)
    }
}

// ---------------------------------------------------------------------------
// F10 / finding ⑫ (2026-06-10 review): deny_unknown_fields strictness tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{DemoConfig, MultiOrgDemoConfig};

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

    // ---------------------------------------------------------------------------
    // MED-B load-bearing: malformed org_id must yield clean Err at parse time,
    // not a panic inside the factory closure (which runs on a tokio worker thread
    // where a panic produces an unrecoverable stack trace rather than an
    // actionable error message).
    //
    // Before the fix, MultiOrgDemoConfig::from_str did ONLY TOML deserialization —
    // no UUID validation. A typo'd org_id passed parse silently, then panicked in
    // build_multi_clone_factory when parse_org_id was called inside start_instances.
    //
    // After the fix, from_str validates all org_id fields as UUIDs and returns Err
    // with an actionable message naming the offending entry.
    // ---------------------------------------------------------------------------

    /// MED-B: a malformed org_id in MultiOrgDemoConfig must be caught at parse time.
    ///
    /// Asserts:
    /// 1. `from_str` returns `Err` (not `Ok`) when any org_id is not a valid UUID.
    /// 2. The error message names the offending org entry (here `"org-bad"`).
    /// 3. The error message names the offending value (`"not-a-uuid"`).
    /// 4. The valid-config case (well-formed UUID) still parses without error.
    ///
    /// Proves the fix is load-bearing: if UUID validation is removed from `from_str`,
    /// this test fails at assertion 1 (parse returns `Ok` instead of `Err`).
    #[test]
    fn test_med_b_malformed_org_id_yields_clean_err_not_panic() {
        // Case 1: malformed org_id — must be a clean Err at from_str call, not a panic.
        let toml_bad = r#"
            [orgs.org-bad]
            org_id = "not-a-uuid"
            sensors = ["crowdstrike"]
            seed = 42
        "#;
        let result = MultiOrgDemoConfig::from_str(toml_bad);
        assert!(
            result.is_err(),
            "MED-B: from_str must return Err for malformed org_id 'not-a-uuid', got Ok"
        );
        let err_msg = result
            .expect_err("MED-B: from_str must return Err for malformed org_id 'not-a-uuid'")
            .to_string();
        assert!(
            err_msg.contains("org-bad"),
            "MED-B: error must name the offending org entry ('org-bad'), got: {err_msg}"
        );
        assert!(
            err_msg.contains("not-a-uuid"),
            "MED-B: error must name the offending value ('not-a-uuid'), got: {err_msg}"
        );

        // Case 2: well-formed org_id — must still parse successfully.
        let toml_good = r#"
            [orgs.org-a]
            org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
            sensors = ["crowdstrike"]
            seed = 100
        "#;
        MultiOrgDemoConfig::from_str(toml_good)
            .expect("MED-B: valid UUID org_id must parse without error");
    }
}
