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

/// Canonical set of sensor IDs supported by `start-multi`.
///
/// This is the **single source of truth** for valid sensor names used by BOTH:
/// 1. `MultiOrgDemoConfig::from_str` — validates each `OrgConfig.sensors` entry at
///    config parse time (LOW finding: unsupported sensor must yield clean `Err`, not
///    a worker-thread panic in `build_multi_clone_factory`'s EC-008 arm).
/// 2. `build_multi_clone_factory` in `multi_org_cmd.rs` — dispatches to the correct
///    seeded clone constructor by matching sensor_id against this set.
///
/// TD-VSDD-060 sibling-awareness: adding a new sensor requires updating ONLY this
/// constant — both validation (here) and dispatch (`multi_org_cmd.rs`) will pick up
/// the change automatically once the `match` arm is added to `build_multi_clone_factory`.
pub const KNOWN_SENSORS: &[&str] = &["crowdstrike", "armis", "claroty", "cyberint"];

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

    /// Scenario configuration shared across ALL sensors in this org's DTU fleet.
    ///
    /// When `Some` and `enabled = true`, `build_multi_clone_factory` calls
    /// `new_with_scenario` (with a shared `Arc<IncidentTimeline>`) instead of
    /// `new_with_seed`. All sensors in the org share the same timeline and
    /// `ScenarioEntityCatalog` (derived from this org's `seed` + `org_id`).
    ///
    /// Mirrors the `CloneConfig.scenario` field used by the `start` path's
    /// `build_clone_pairs` (harness.rs) — same `ScenarioConfig` type.
    ///
    /// When `None` or `enabled = false`, falls back to `new_with_seed` (backward compatible).
    #[serde(default)]
    pub scenario: Option<ScenarioConfig>,
}

/// Returns `true` if `slug` is a path-safe org slug.
///
/// Allowed charset: `[a-zA-Z0-9][a-zA-Z0-9\-]*`
/// - Must be non-empty.
/// - First character: ASCII alphanumeric only (no leading hyphen).
/// - Remaining characters: ASCII alphanumeric or `-` (hyphen).
/// - Explicitly rejects `/`, `.`, `..`, `\`, null bytes, and any character that
///   could escape a `path.join(customers_dir, slug)` boundary.
///
/// This is a char-level scan with no regex dependency — the `prism-dtu-demo-server`
/// crate does not depend on `regex`, and the allowed set is small enough to check
/// with `char::is_ascii_alphanumeric` + a single `== '-'` guard.
///
/// SEC-001 (CWE-22): single enforcement point called from `MultiOrgDemoConfig::from_str`
/// before any filesystem path is constructed from the slug value.
fn is_path_safe_slug(slug: &str) -> bool {
    if slug.is_empty() {
        return false;
    }
    let mut chars = slug.chars();
    // First character: alphanumeric only (no leading hyphen)
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    // Remaining characters: alphanumeric or hyphen
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
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
    /// After TOML deserialization, validates every org slug key, `org_id` field, and
    /// `sensors` list in `orgs`:
    ///
    /// - **SEC-001 (CWE-22):** Each org slug (HashMap key) must match
    ///   `[a-zA-Z0-9][a-zA-Z0-9-]*` — alphanumeric start, alphanumeric-or-hyphen body.
    ///   A slug containing `/`, `..`, a leading hyphen, or any other path-unsafe character
    ///   returns `Err` naming the offending slug before any filesystem path is constructed.
    ///   This is the single enforcement point: the same slug flows into
    ///   `os.path.join(customers_dir, org_slug)` + `os.makedirs` in the shell overlay
    ///   script, so a crafted `[orgs."../../../tmp/evil"]` would write outside
    ///   `customers_dir` if not blocked here.
    ///
    /// - **MED-B:** Each `org_id` must be a well-formed UUID.
    ///
    /// - **LOW sensor validation:** Each sensor name must appear in `KNOWN_SENSORS`.
    ///
    /// Mirrors the `DemoConfig::from_str` inherent method pattern.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(toml_str: &str) -> anyhow::Result<Self> {
        let cfg: Self = toml::from_str(toml_str)
            .map_err(|e| anyhow::anyhow!("Invalid TOML in multi-org demo config: {}", e))?;

        for (slug, org_cfg) in &cfg.orgs {
            // SEC-001 (CWE-22): validate the org slug is path-safe before it can reach
            // any filesystem join. Allowed charset: [a-zA-Z0-9][a-zA-Z0-9-]*.
            // This rejects "../../../tmp/evil", "/abs/path", "leading-dash" (-foo),
            // and any other slug that could escape `customers_dir` in the overlay script.
            if !is_path_safe_slug(slug) {
                return Err(anyhow::anyhow!(
                    "Invalid multi-org demo config: org slug '{}' contains characters that are \
                     not path-safe. Org slugs must start with an alphanumeric character and \
                     contain only alphanumeric characters and hyphens (e.g. 'org-a', 'acme-corp'). \
                     Slugs containing '/', '..', a leading hyphen, or other special characters \
                     are rejected to prevent path traversal (CWE-22).",
                    slug
                ));
            }

            // MED-B: validate all org_id fields as UUIDs at parse time.
            // This ensures the `.expect()` in `build_multi_clone_factory` is a true
            // programming-error guard (not a user-input panic on a typo'd org_id).
            uuid::Uuid::parse_str(&org_cfg.org_id).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid multi-org demo config: org '{}' has org_id '{}' which is not a \
                     valid UUID. Expected a hyphenated UUID v7 string, e.g. \
                     '0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000'.",
                    slug,
                    org_cfg.org_id
                )
            })?;

            // LOW fix: validate every sensors entry against the known supported set at parse
            // time. Without this check, an unsupported sensor (e.g. "foo") passes from_str,
            // builds an InstanceEntry named "{org_slug}-foo", then hits the EC-008 `other =>
            // panic!` arm inside build_multi_clone_factory on a tokio worker thread —
            // the same operator-config-error→worker-panic pattern MED-B was introduced to
            // prevent for org_id. Mirrors the MED-B pattern: clean Err naming the offending
            // org AND sensor, never a panic.
            for sensor in &org_cfg.sensors {
                if !KNOWN_SENSORS.contains(&sensor.as_str()) {
                    return Err(anyhow::anyhow!(
                        "Invalid multi-org demo config: org '{}' lists sensor '{}' which is not \
                         supported. Valid sensors: {}.",
                        slug,
                        sensor,
                        KNOWN_SENSORS.join(", ")
                    ));
                }
            }
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

    // ---------------------------------------------------------------------------
    // SEC-001 (CWE-22): path-traversal-safe org slug validation load-bearing tests.
    //
    // The org slug (HashMap key in MultiOrgDemoConfig.orgs) flows unsanitized into
    // `os.path.join(customers_dir, org_slug)` + `os.makedirs` in the shell overlay
    // script. A crafted `[orgs."../../../tmp/evil"]` TOML key would escape
    // `customers_dir` if not blocked at parse time.
    //
    // After the fix, MultiOrgDemoConfig::from_str calls `is_path_safe_slug` for every
    // slug key BEFORE any UUID or sensor validation. Invalid slugs return `Err` naming
    // the offending slug — no panic, no filesystem operation, no subprocess needed.
    //
    // Proves the fix is load-bearing: if `is_path_safe_slug` is removed or weakened
    // to return `true` for path-unsafe inputs, the path-traversal cases below fail
    // at assertion 1 (from_str returns Ok instead of Err).
    // ---------------------------------------------------------------------------

    /// SEC-001 (CWE-22): org slug path traversal must be rejected at parse time.
    ///
    /// Asserts:
    /// 1. `from_str` returns `Err` (not `Ok`) for slugs containing path-traversal
    ///    sequences (`../`), absolute-path prefixes (`/`), leading hyphens, empty
    ///    strings, or any character outside `[a-zA-Z0-9\-]`.
    /// 2. The error message names the offending slug.
    /// 3. Valid slugs (`"org-a"`, `"acme-corp"`, `"tenant123"`) still parse without error.
    #[test]
    fn test_sec_001_org_slug_path_traversal_rejected() {
        // A valid org_id and sensor to use across all cases — the slug is the only variable.
        let valid_org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000";
        let valid_sensor = "crowdstrike";

        // --- Cases that MUST be rejected ---

        // Path traversal sequences
        let traversal_cases: &[&str] = &[
            "../../../tmp/evil",
            "../sibling",
            "parent/../escape",
            "evil/subdir",
            "/absolute/path",
            "leading-ok/but-slash",
        ];
        for slug in traversal_cases {
            let toml = format!(
                "[orgs.\"{}\"]\norg_id = \"{}\"\nsensors = [\"{}\"]\nseed = 42\n",
                slug, valid_org_id, valid_sensor
            );
            let result = MultiOrgDemoConfig::from_str(&toml);
            // Note: TOML may reject some of these slugs as invalid TOML keys before
            // Rust validation even runs (e.g. bare `../` in a key). Both outcomes
            // (TOML parse error OR our validation Err) are acceptable — the invariant
            // is that from_str NEVER returns Ok for these slugs.
            assert!(
                result.is_err(),
                "SEC-001: from_str must return Err for path-traversal slug '{slug}', got Ok"
            );
        }

        // Leading hyphen — first char must be alphanumeric
        let leading_hyphen = "-leading-hyphen";
        let toml_leading_hyphen = format!(
            "[orgs.\"{}\"]\norg_id = \"{}\"\nsensors = [\"{}\"]\nseed = 42\n",
            leading_hyphen, valid_org_id, valid_sensor
        );
        let result_lh = MultiOrgDemoConfig::from_str(&toml_leading_hyphen);
        assert!(
            result_lh.is_err(),
            "SEC-001: from_str must return Err for leading-hyphen slug '{leading_hyphen}', got Ok"
        );
        // The error message must name the offending slug (our Err path)
        // OR it's a TOML parse error (which also rejects it) — both are acceptable.
        // If our validator fired, verify the slug is named:
        if let Err(ref e) = result_lh {
            let msg = e.to_string();
            // If the error is from our validator (not the TOML parser), slug must be named.
            if msg.contains("path-safe") || msg.contains("org slug") {
                assert!(
                    msg.contains(leading_hyphen),
                    "SEC-001: error must name the offending slug '{leading_hyphen}', got: {msg}"
                );
            }
        }

        // Dot-only / double-dot — special filesystem entries
        let dot_cases: &[&str] = &[".", ".."];
        for slug in dot_cases {
            let toml = format!(
                "[orgs.\"{}\"]\norg_id = \"{}\"\nsensors = [\"{}\"]\nseed = 42\n",
                slug, valid_org_id, valid_sensor
            );
            // TOML may or may not parse these as bare keys; either Err form is acceptable.
            let result = MultiOrgDemoConfig::from_str(&toml);
            assert!(
                result.is_err(),
                "SEC-001: from_str must return Err for dot-slug '{slug}', got Ok"
            );
        }

        // --- Control cases that MUST parse successfully ---

        let valid_slugs: &[&str] = &["org-a", "acme-corp", "tenant123", "a", "X9", "org-b-west"];
        for slug in valid_slugs {
            let toml = format!(
                "[orgs.{}]\norg_id = \"{}\"\nsensors = [\"{}\"]\nseed = 100\n",
                slug, valid_org_id, valid_sensor
            );
            MultiOrgDemoConfig::from_str(&toml).unwrap_or_else(|e| {
                panic!("SEC-001: valid slug '{slug}' must parse without error, got: {e}")
            });
        }
    }

    // ---------------------------------------------------------------------------
    // LOW fix load-bearing: unsupported sensor name must yield clean Err at parse
    // time, not a panic on a tokio worker thread inside build_multi_clone_factory's
    // EC-008 `other => panic!` arm.
    //
    // Before the fix, MultiOrgDemoConfig::from_str validated org_id (UUID) but NOT
    // sensors. A config with sensors = ["foo"] parsed Ok, built an InstanceEntry named
    // "{org_slug}-foo", then hit the EC-008 panic at runtime — the same
    // operator-config-error→worker-panic pattern MED-B was introduced to prevent
    // for org_id. After the fix, from_str validates every sensors entry against
    // KNOWN_SENSORS and returns Err with an actionable message naming org + sensor.
    //
    // Parallel to test_med_b_malformed_org_id_yields_clean_err_not_panic.
    // ---------------------------------------------------------------------------

    /// LOW fix: an unsupported sensor in OrgConfig.sensors must be caught at parse time.
    ///
    /// Asserts:
    /// 1. `from_str` returns `Err` (not `Ok`) when any org's sensors list contains an
    ///    unsupported value (i.e. not in KNOWN_SENSORS: crowdstrike/armis/claroty/cyberint).
    /// 2. The error message names the offending org entry (here `"org-bad"`).
    /// 3. The error message names the offending sensor value (`"foo"`).
    /// 4. The valid-sensor case (all sensors in KNOWN_SENSORS) still parses without error.
    ///
    /// Proves the fix is load-bearing: if sensor validation is removed from `from_str`,
    /// this test fails at assertion 1 (parse returns `Ok` instead of `Err`).
    #[test]
    fn test_low_unsupported_sensor_yields_clean_err_not_panic() {
        // Case 1: unsupported sensor — must be a clean Err at from_str call, not a panic.
        let toml_bad = r#"
            [orgs.org-bad]
            org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
            sensors = ["foo"]
            seed = 42
        "#;
        let result = MultiOrgDemoConfig::from_str(toml_bad);
        assert!(
            result.is_err(),
            "LOW fix: from_str must return Err for unsupported sensor 'foo', got Ok"
        );
        let err_msg = result
            .expect_err("LOW fix: from_str must return Err for unsupported sensor 'foo'")
            .to_string();
        assert!(
            err_msg.contains("org-bad"),
            "LOW fix: error must name the offending org entry ('org-bad'), got: {err_msg}"
        );
        assert!(
            err_msg.contains("foo"),
            "LOW fix: error must name the offending sensor value ('foo'), got: {err_msg}"
        );

        // Case 2: all-valid sensors — must still parse successfully.
        let toml_good = r#"
            [orgs.org-a]
            org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
            sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
            seed = 100
        "#;
        MultiOrgDemoConfig::from_str(toml_good)
            .expect("LOW fix: all KNOWN_SENSORS values must parse without error");

        // Case 3: mixed valid+invalid sensors in same org — Err must still fire.
        let toml_mixed = r#"
            [orgs.org-mixed]
            org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
            sensors = ["crowdstrike", "unknown-sensor"]
            seed = 42
        "#;
        let result_mixed = MultiOrgDemoConfig::from_str(toml_mixed);
        assert!(
            result_mixed.is_err(),
            "LOW fix: from_str must return Err when sensors list contains a mix of valid and \
             invalid entries, got Ok"
        );
        let err_mixed = result_mixed
            .expect_err("LOW fix: must return Err for mixed valid/invalid sensors")
            .to_string();
        assert!(
            err_mixed.contains("unknown-sensor"),
            "LOW fix: error must name the offending sensor ('unknown-sensor'), got: {err_mixed}"
        );
    }
}
