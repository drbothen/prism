// SPDX-License-Identifier: Apache-2.0
// Items in this helpers module are used selectively by different test binaries
// (e2e_smoke, e2e_multi_org, plugin_boot_tests, bc_2_10_006_mcp_stdout_purity).
// Each item is used by at least one binary; the ones not used by all binaries
// are correctly reported as dead_code by the per-binary perspective.
#![allow(dead_code)]
//! Test helpers for S-DEMO-002 E2E subprocess smoke test.
//!
//! Provides:
//! - `SubprocessGuard` — drop guard that sends SIGTERM to a child process.
//! - `wait_for_file()` — async polling with exponential backoff.
//! - `write_demo_config()` — generates temp prism.toml with DTU port overlays.
//! - `write_multi_org_demo_config()` — 3-org config for multi-tenant isolation tests.
//! - `bootstrap_credentials()` — no-op for DTU tests (DTU clones accept any credentials).
//! - `DtuPorts` — port map parsed from `.prism-dtu-demo-server.urls.json`.
//!
//! # Design notes
//!
//! ## DTU-MULTI-001
//! The demo DTU operates in single-tenant mode; org isolation is at the
//! AdapterRegistry layer only. All orgs that share a sensor point to the same
//! DTU clone port and receive identical fixture data. True per-org HTTP
//! segregation is Wave 3 scope (BC-3.2.003/BC-3.2.004).
//!
//! ## Credential model (AD-017)
//! DTU clones accept any credential values — they don't validate auth.
//! `bootstrap_credentials` is a no-op for DTU-backed E2E tests.
//! Credential values MUST NOT appear in source files visible to AI context.
//!
//! ## Subprocess binary location
//! Tests locate the `prism` and `prism-dtu-demo-server` binaries via `locate_binary`:
//! 1. `CARGO_BIN_EXE_*` env var (populated by cargo for same-package bins only).
//! 2. Workspace `target/release/<name>` — preferred (Architecture Compliance Rule 5).
//! 3. Workspace `target/debug/<name>` — fallback with VISIBLE diagnostic (not silent);
//!    emits eprintln! warning. Debug binaries may cause 30s E2E timeout failures.
//! 4. Neither found — `Err(...)` with actionable `cargo build --release` instruction.
//!
//! OBS-1: The debug fallback is NOT silent — a clear diagnostic is always emitted
//! when the debug path is taken, so developers know they are not using the preferred
//! release binary.
//!
//! Run `cargo build --release -p prism -p prism-dtu-demo-server` before E2E tests.
//!
//! Story: S-DEMO-002
//! BCs: BC-2.22.001, BC-2.10.010, BC-3.2.001

use std::collections::HashMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// E2E test constants
// ---------------------------------------------------------------------------

/// Shared access token registered in the Cyberint DTU's allowlist and passed
/// via per-client env vars for E2E tests (ADR-032 / BC-2.06.003).
///
/// The Cyberint DTU validates the `access_token` cookie against an in-memory
/// allowlist (ADR-031 §D3-a). `launch_prism_bin()` sets
/// `PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_API_KEY` for each
/// org that uses Cyberint, so `StaticCookieAuthProvider` (via `PrismCredentialResolver`)
/// injects it as `Cookie: access_token=dtu-e2e-cyberint-access-token`.
///
/// Not a real credential — never reaches any external service.
/// Per AD-017: credential values must not transit AI context; this is a
/// test-harness-only placeholder, visible only in test-scope code.
const DTU_E2E_CYBERINT_ACCESS_TOKEN: &str = "dtu-e2e-cyberint-access-token";

/// Bearer token for Armis — set via per-client env vars for E2E tests (ADR-032).
///
/// Resolved by `BearerStaticCredentialAuthProvider` via
/// `resolve_credential(org_slug, "armis", "bearer_token")` → env var
/// `PRISM_CLIENTS_{ID}_SENSORS_ARMIS_BEARER_TOKEN` (BC-2.06.003 Tier 2).
/// The Armis DTU clone validates `Authorization: Bearer {non-empty}` — any non-empty value passes.
///
/// Not a real credential — never reaches any external service.
/// Per AD-017: credential values must not transit AI context; this is a
/// test-harness-only placeholder, visible only in test-scope code.
const DTU_E2E_ARMIS_BEARER_TOKEN: &str = "dtu-e2e-armis-bearer-token";

/// Bearer token for Claroty — set via per-client env vars for E2E tests (ADR-032).
///
/// Resolved by `BearerStaticCredentialAuthProvider` via
/// `resolve_credential(org_slug, "claroty", "bearer_token")` → env var
/// `PRISM_CLIENTS_{ID}_SENSORS_CLAROTY_BEARER_TOKEN` (BC-2.06.003 Tier 2).
/// The Claroty DTU clone validates `Authorization: Bearer {non-empty}` — any non-empty value passes.
///
/// Not a real credential — never reaches any external service.
/// Per AD-017: credential values must not transit AI context; this is a
/// test-harness-only placeholder, visible only in test-scope code.
const DTU_E2E_CLAROTY_BEARER_TOKEN: &str = "dtu-e2e-claroty-bearer-token";

// ---------------------------------------------------------------------------
// DtuPorts
// ---------------------------------------------------------------------------

/// Port map parsed from `.prism-dtu-demo-server.urls.json`.
///
/// Keys match clone names (`"crowdstrike"`, `"armis"`, `"claroty"`, `"cyberint"`).
/// Values are `"http://127.0.0.1:<port>"` strings.
///
/// DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
/// AdapterRegistry layer only.
#[derive(Debug, Default)]
pub struct DtuPorts {
    pub urls: HashMap<String, String>,
}

impl DtuPorts {
    /// Parse a `DtuPorts` from the JSON contents of `.prism-dtu-demo-server.urls.json`.
    ///
    /// The urls.json file is a flat `{"clone_name": "http://host:port", ...}` object.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse urls.json: {e}"))?;

        let obj = parsed
            .as_object()
            .ok_or_else(|| "urls.json must be a JSON object".to_string())?;

        let mut urls = HashMap::new();
        for (key, val) in obj {
            let url = val
                .as_str()
                .ok_or_else(|| format!("urls.json: value for key '{key}' is not a string"))?;
            urls.insert(key.clone(), url.to_string());
        }

        Ok(DtuPorts { urls })
    }

    /// Returns `http://127.0.0.1:<port>` for the named clone.
    pub fn base_url(&self, clone_name: &str) -> Option<&str> {
        self.urls.get(clone_name).map(|s| s.as_str())
    }
}

// ---------------------------------------------------------------------------
// SubprocessGuard
// ---------------------------------------------------------------------------

/// Drop guard that sends SIGTERM to a child process when dropped.
///
/// AC-008 (BC-2.10.010): ensures both prism-bin and DTU server exit cleanly
/// even if an assertion fails mid-test. If the process hasn't exited within
/// 3 seconds after SIGTERM, it is forcibly killed.
pub struct SubprocessGuard {
    pub child: Child,
    /// Diagnostic name for log messages during SIGTERM teardown.
    #[allow(dead_code)]
    pub name: String,
}

impl SubprocessGuard {
    pub fn new(child: Child, name: impl Into<String>) -> Self {
        Self {
            child,
            name: name.into(),
        }
    }
}

#[cfg(unix)]
impl Drop for SubprocessGuard {
    fn drop(&mut self) {
        // Send SIGTERM to the child process.
        let pid = self.child.id() as libc::pid_t;
        let _ = unsafe { libc::kill(pid, libc::SIGTERM) };

        // Wait up to 3 seconds for clean exit.
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return, // Exited cleanly.
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                _ => {
                    // Timeout or error — force kill.
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// wait_for_file
// ---------------------------------------------------------------------------

/// Poll for `path` to appear on the filesystem with exponential backoff.
///
/// Returns `Ok(contents)` when the file exists and is non-empty.
/// Returns `Err(String)` if the file does not appear within `timeout_secs`.
///
/// Uses `tokio::time::sleep` (never `std::thread::sleep`).
///
/// Risk mitigation: max 30s timeout with backoff per story risk_mitigations[0].
pub async fn wait_for_file(path: &Path, timeout_secs: u64) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut delay_ms = 100u64; // Start at 100ms, double each retry, cap at 2000ms.

    loop {
        if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(contents) if !contents.is_empty() => return Ok(contents),
                Ok(_) => {} // File exists but empty — wait for write to complete.
                Err(e) => {
                    // Transient read error (partial write race) — retry.
                    if Instant::now() >= deadline {
                        return Err(format!(
                            "Timeout waiting for file '{}': read error: {e}",
                            path.display()
                        ));
                    }
                }
            }
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "Timeout waiting for file '{}' to appear within {}s (EC-001)",
                path.display(),
                timeout_secs
            ));
        }

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms = (delay_ms * 2).min(2000);
    }
}

// ---------------------------------------------------------------------------
// bootstrap_credentials (no-op for DTU-backed E2E tests)
// ---------------------------------------------------------------------------

/// Insert dummy credentials for all 4 sensors.
///
/// DTU clones accept any credential values — they don't validate auth at the
/// HTTP level. This function is a no-op for DTU-backed E2E tests.
///
/// AD-017: credential values MUST NOT appear in source files visible to AI.
/// In a real production test environment, credentials would be seeded via
/// OS keyring CLI (`security add-generic-password` on macOS, `secret-tool` on Linux)
/// using values sourced from environment variables, not hardcoded here.
///
/// For DTU E2E tests, since the DTU clone accepts any auth token, no actual
/// credential seeding is required.
#[allow(dead_code)]
pub fn bootstrap_credentials(_config_dir: &Path) -> Result<(), String> {
    // No-op: DTU clones do not validate credentials.
    // If a real credential backend is required, inject via PRISM_CONFIG_DIR
    // and OS keyring commands using env-var-sourced values (AD-017).
    Ok(())
}

// ---------------------------------------------------------------------------
// Sensor spec template helpers
// ---------------------------------------------------------------------------

/// Return the path to the workspace-level sensor specs directory.
fn workspace_sensor_specs_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR is set by cargo for integration tests.
    // prism-bin/tests/../.. → workspace root → crates/prism-sensors/specs/
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during integration tests");
    PathBuf::from(manifest_dir)
        .parent() // prism-bin
        .expect("manifest dir must have a parent")
        .parent() // crates/
        .expect("crates dir must have a parent")
        .join("crates/prism-sensors/specs")
}

/// Write a per-sensor overlay TOML file that overrides `base_url`.
///
/// Overlay format follows BC-2.06.012 / ADR-029 §Hybrid Sensor Instance.
fn write_sensor_overlay(
    customers_dir: &Path,
    org_slug: &str,
    sensor_id: &str,
    base_url: &str,
) -> Result<(), String> {
    let org_dir = customers_dir.join(org_slug);
    std::fs::create_dir_all(&org_dir)
        .map_err(|e| format!("Failed to create overlay dir '{}': {e}", org_dir.display()))?;

    let overlay_path = org_dir.join(format!("{sensor_id}.sensor.toml"));
    let content = format!(
        "# Per-org overlay for {sensor_id} sensor — {org_slug} (DTU-backed E2E test)\n\
         # BC-2.06.012 / ADR-029: scalar-only overlay; no schema fields.\n\
         extends     = \"{sensor_id}\"\n\
         instance_id = \"{sensor_id}@{org_slug}\"\n\
         base_url    = \"{base_url}\"\n"
    );

    std::fs::write(&overlay_path, &content)
        .map_err(|e| format!("Failed to write overlay '{}': {e}", overlay_path.display()))
}

// ---------------------------------------------------------------------------
// write_demo_config (single-org)
// ---------------------------------------------------------------------------

/// Write a prism.toml with a single `demo-org` org entry and per-sensor DTU overlays.
///
/// Directory layout:
/// ```text
/// <config_dir>/
///   prism.toml                          — prism config with demo-org
///   specs/                              — sensor specs (symlink-copied from workspace)
///   specs/crowdstrike.sensor.toml       — canonical TYPE spec (copied)
///   specs/armis.sensor.toml             — canonical TYPE spec (copied)
///   specs/claroty.sensor.toml           — canonical TYPE spec (copied)
///   specs/cyberint.sensor.toml          — canonical TYPE spec (copied)
///   specs/customers/demo-org/           — overlay directory for demo-org
///     crowdstrike.sensor.toml           — base_url override → DTU port
///     armis.sensor.toml                 — base_url override → DTU port
///     claroty.sensor.toml               — base_url override → DTU port
///     cyberint.sensor.toml              — base_url override → DTU port
///   state/                              — RocksDB state directory
///   plugins/                            — empty plugin directory
/// ```
pub fn write_demo_config(config_dir: &Path, dtu_ports: &DtuPorts) -> Result<(), String> {
    // UUIDv7 for demo-org: fixed value for deterministic tests.
    // BC-2.21.001: org_id must be UUID v7.
    const DEMO_ORG_ID: &str = "019700a0-0000-7000-8000-000000000001";
    const DEMO_ORG_SLUG: &str = "demo-org";

    write_org_config(
        config_dir,
        &[(DEMO_ORG_ID, DEMO_ORG_SLUG)],
        &[
            (DEMO_ORG_SLUG, "crowdstrike"),
            (DEMO_ORG_SLUG, "armis"),
            (DEMO_ORG_SLUG, "claroty"),
            (DEMO_ORG_SLUG, "cyberint"),
        ],
        dtu_ports,
    )
}

// ---------------------------------------------------------------------------
// write_multi_org_demo_config (3-org)
// ---------------------------------------------------------------------------

/// Write a prism.toml with 3 orgs configured for multi-tenant isolation tests (S-DEMO-004 AC-001..AC-010).
///
/// Org layout:
/// - `demo-org-a` (UUIDv7): CrowdStrike + Armis (2 sensors)
/// - `demo-org-b` (UUIDv7): Claroty + Cyberint (2 sensors)
/// - `demo-org-c` (UUIDv7): all 4 sensors
///
/// Each org gets a distinct `org_id` (fixed UUIDv7) and `org_slug` with corresponding
/// `specs/customers/{slug}/` overlay directories setting DTU clone `base_url` per sensor.
///
/// DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
/// AdapterRegistry layer only. Two different orgs that both have CrowdStrike
/// point to the same DTU clone port — they receive the same fixture data.
/// This is by design (S-DEMO-002 scope; per-org DTU isolation is S-DEMO-004 scope).
pub fn write_multi_org_demo_config(config_dir: &Path, dtu_ports: &DtuPorts) -> Result<(), String> {
    // Fixed UUIDv7 values for deterministic multi-org tests.
    // BC-2.21.001: org_id must be UUID v7.
    const ORG_A_ID: &str = "019700a0-0000-7000-8000-000000000011";
    const ORG_A_SLUG: &str = "demo-org-a";
    const ORG_B_ID: &str = "019700a0-0000-7000-8000-000000000012";
    const ORG_B_SLUG: &str = "demo-org-b";
    const ORG_C_ID: &str = "019700a0-0000-7000-8000-000000000013";
    const ORG_C_SLUG: &str = "demo-org-c";

    write_org_config(
        config_dir,
        &[
            (ORG_A_ID, ORG_A_SLUG),
            (ORG_B_ID, ORG_B_SLUG),
            (ORG_C_ID, ORG_C_SLUG),
        ],
        &[
            // demo-org-a: CrowdStrike + Armis
            (ORG_A_SLUG, "crowdstrike"),
            (ORG_A_SLUG, "armis"),
            // demo-org-b: Claroty + Cyberint
            (ORG_B_SLUG, "claroty"),
            (ORG_B_SLUG, "cyberint"),
            // demo-org-c: all 4 sensors
            (ORG_C_SLUG, "crowdstrike"),
            (ORG_C_SLUG, "armis"),
            (ORG_C_SLUG, "claroty"),
            (ORG_C_SLUG, "cyberint"),
        ],
        dtu_ports,
    )
}

/// Internal: write prism.toml + sensor specs + per-org overlay directories.
///
/// `orgs`: slice of `(org_id, org_slug)` pairs.
/// `org_sensors`: slice of `(org_slug, sensor_id)` pairs for overlay generation.
fn write_org_config(
    config_dir: &Path,
    orgs: &[(&str, &str)],
    org_sensors: &[(&str, &str)],
    dtu_ports: &DtuPorts,
) -> Result<(), String> {
    // Create directory layout.
    let specs_dir = config_dir.join("specs");
    let state_dir = config_dir.join("state");
    let plugins_dir = config_dir.join("plugins");

    for dir in [&specs_dir, &state_dir, &plugins_dir] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory '{}': {e}", dir.display()))?;
    }

    // Stage the crowdstrike-oauth2 plugin for E2E tests.
    //
    // The crowdstrike.sensor.toml TYPE spec requires auth_type = "oauth2_client_credentials"
    // with auth_plugin = "crowdstrike-oauth2" (D-747 LOCKED). Without the plugin, boot step
    // 7.5b fails with BootError::UnknownAuthPlugin — the prism process exits before the MCP
    // server binds, producing a "Broken pipe" EC-002 failure.
    //
    // The production plugin manifest (plugin.toml) has allowed_urls = ["api.crowdstrike.com"].
    // SEC-003 validates the token endpoint host (127.0.0.1 for the DTU clone) against this
    // allowlist at boot time. To pass SEC-003 in the E2E test harness, we write a DTU-safe
    // manifest that extends the allowlist with "127.0.0.1" (the DTU bind address per demo.toml).
    //
    // The production .prx binary is used unchanged — only the manifest's allowed_urls changes.
    // The production manifest (plugin.toml) is NOT modified; only the test-staging copy differs.
    //
    // Precedent: plugin_boot_tests.rs SENSOR_AUTH_MANIFEST constant uses the same pattern
    // (adds "localhost" to allowed_urls for in-process unit tests; see plugin_boot_tests.rs L1357).
    stage_crowdstrike_plugin(&plugins_dir)?;

    // Copy canonical TYPE specs from workspace into temp specs_dir.
    let workspace_specs = workspace_sensor_specs_dir();
    for sensor_id in ["crowdstrike", "armis", "claroty", "cyberint"] {
        let src = workspace_specs.join(format!("{sensor_id}.sensor.toml"));
        let dst = specs_dir.join(format!("{sensor_id}.sensor.toml"));
        std::fs::copy(&src, &dst).map_err(|e| {
            format!(
                "Failed to copy sensor spec '{}' → '{}': {e}",
                src.display(),
                dst.display()
            )
        })?;
    }

    // Write per-org overlay files into specs/customers/{org_slug}/.
    let customers_dir = specs_dir.join("customers");
    for (org_slug, sensor_id) in org_sensors {
        let base_url = dtu_ports
            .base_url(sensor_id)
            .ok_or_else(|| format!("No DTU port for sensor '{sensor_id}' in urls.json"))?;
        write_sensor_overlay(&customers_dir, org_slug, sensor_id, base_url)?;
    }

    // Build [[orgs]] section for prism.toml.
    let mut orgs_toml = String::new();
    for (org_id, org_slug) in orgs {
        orgs_toml.push_str(&format!(
            "\n[[orgs]]\norg_id = \"{org_id}\"\norg_slug = \"{org_slug}\"\n"
        ));
    }

    // Write prism.toml.
    // Windows-safe path serialization: {:?} emits a quoted string with backslashes
    // escaped as \\, producing valid TOML basic-string values on Windows paths
    // (e.g. C:\Users\... → "C:\\Users\\..."). Pattern matches all make_valid_config_dir()
    // helpers throughout the prism-bin test suite.
    let prism_toml = format!(
        "# Generated by S-DEMO-002 E2E test harness — do not edit manually.\n\
         spec_dir   = {:?}\n\
         state_dir  = {:?}\n\
         plugin_dir = {:?}\n\
         {}\n",
        specs_dir.display(),
        state_dir.display(),
        plugins_dir.display(),
        orgs_toml.trim()
    );

    let prism_toml_path = config_dir.join("prism.toml");
    std::fs::write(&prism_toml_path, &prism_toml)
        .map_err(|e| format!("Failed to write prism.toml: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// stage_crowdstrike_plugin (E2E test helper)
// ---------------------------------------------------------------------------

/// Stage the `crowdstrike-oauth2` plugin into the given plugins directory.
///
/// The crowdstrike.sensor.toml TYPE spec requires `auth_plugin = "crowdstrike-oauth2"` (D-747).
/// Boot step 7.5b calls `validate_and_construct_auth_providers`, which fails with
/// `BootError::UnknownAuthPlugin` if the plugin is not loaded. The process then exits before
/// the MCP server binds, producing a "Broken pipe" EC-002 error in the test harness.
///
/// The production plugin binary (`.prx`) is copied unchanged from
/// `crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx`.
///
/// The production `plugin.toml` manifest has `allowed_urls = ["api.crowdstrike.com"]`.
/// SEC-003 validates the token endpoint host (`127.0.0.1` for the DTU clone, per demo.toml
/// `bind = "127.0.0.1"`) against this allowlist at boot time. To pass SEC-003 in the
/// E2E harness, a DTU-safe companion manifest is written instead, extending the allowlist
/// with `"127.0.0.1"`.
///
/// The production `plugin.toml` is **NOT modified**. Only the test-staging copy in the
/// temp plugins directory uses the extended allowlist. This mirrors the pattern in
/// `plugin_boot_tests.rs::SENSOR_AUTH_MANIFEST` (L1357 adds `"localhost"` for unit tests).
///
/// # Manifest companion naming
/// The companion file is `{prx_stem}.manifest.toml` per `load_all_plugins` convention
/// (`path.with_extension("manifest.toml")`).
fn stage_crowdstrike_plugin(plugins_dir: &Path) -> Result<(), String> {
    // Locate the production .prx file relative to CARGO_MANIFEST_DIR.
    // CARGO_MANIFEST_DIR for prism-bin tests is `crates/prism-bin`.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during integration tests");
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent() // prism-bin
        .expect("manifest dir must have a parent")
        .parent() // crates/
        .expect("crates dir must have a parent")
        .to_path_buf();

    let prx_src = workspace_root
        .join("crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx");
    let prx_dst = plugins_dir.join("crowdstrike-oauth2.prx");

    std::fs::copy(&prx_src, &prx_dst).map_err(|e| {
        format!(
            "Failed to copy crowdstrike-oauth2.prx from '{}' → '{}': {e}\n\
             Hint: verify the plugin was compiled (it ships pre-built in the repo).",
            prx_src.display(),
            prx_dst.display()
        )
    })?;

    // Write a DTU-safe companion manifest that adds "127.0.0.1" to allowed_urls.
    // The DTU demo server always binds to 127.0.0.1 (demo.toml: bind = "127.0.0.1").
    // SEC-003 validates the token endpoint host (base_url + "/oauth2/token") against
    // allowed_urls at boot time; the DTU overlay sets base_url = "http://127.0.0.1:<port>".
    //
    // plugin_type = "sensor_auth" matches the production plugin.toml (required by
    // PluginRuntime::load_all_plugins for correct type dispatch).
    let manifest_content = r#"# DTU-safe companion manifest for crowdstrike-oauth2 E2E tests.
# Extends production allowed_urls with "127.0.0.1" so SEC-003 passes when the
# CrowdStrike DTU clone is the token endpoint (demo.toml: bind = "127.0.0.1").
# The production plugin.toml is NOT modified — only this test-staging copy differs.
name = "crowdstrike-oauth2"
version = "0.1.0"
format_version = 1
plugin_type = "sensor_auth"
allowed_urls = ["api.crowdstrike.com", "127.0.0.1"]
"#;

    let manifest_dst = plugins_dir.join("crowdstrike-oauth2.manifest.toml");
    std::fs::write(&manifest_dst, manifest_content).map_err(|e| {
        format!(
            "Failed to write crowdstrike-oauth2 DTU manifest to '{}': {e}",
            manifest_dst.display()
        )
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// launch_dtu_server
// ---------------------------------------------------------------------------

/// Launch `prism-dtu-demo-server start --config <fixture>` as a subprocess.
///
/// Polls for `.prism-dtu-demo-server.urls.json` in the working directory via
/// `wait_for_file()` with 30s timeout.
/// Returns `(SubprocessGuard, DtuPorts)`.
///
/// Uses the release binary (per Architecture Compliance Rule 5 in the story):
/// locate via `CARGO_BIN_EXE_prism-dtu-demo-server` env var or workspace target dir.
///
/// # DTU binary location
/// Integration tests set `CARGO_BIN_EXE_*` for bins in the same package.
/// For cross-package binaries, we use the workspace `target/debug` or `target/release` dir
/// derived from `CARGO_MANIFEST_DIR`.
///
/// # DTU-EXT-001 (SID-1 compliance)
/// This function requires a live DTU binary and network. It is called only from
/// `#[ignore]`'d E2E tests that are un-gated via the 'e2e' nextest profile.
pub async fn launch_dtu_server(
    fixture_config: &Path,
    working_dir: &TempDir,
) -> Result<(SubprocessGuard, DtuPorts), String> {
    let dtu_bin = locate_binary("prism-dtu-demo-server")?;

    // Spawn DTU server subprocess.
    // The DTU server writes `.prism-dtu-demo-server.urls.json` in its working directory
    // (cwd = working_dir.path()). We poll for that file with 30s timeout.
    let child = std::process::Command::new(&dtu_bin)
        .arg("start")
        .arg("--config")
        .arg(fixture_config)
        .current_dir(working_dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn DTU server '{}': {e}", dtu_bin.display()))?;

    let guard = SubprocessGuard::new(child, "prism-dtu-demo-server");

    // Poll for urls.json (written atomically after all clones bind).
    let urls_file = working_dir.path().join(".prism-dtu-demo-server.urls.json");
    let urls_json = wait_for_file(&urls_file, 30)
        .await
        .map_err(|e| format!("DTU server did not write urls.json within 30s (EC-001): {e}"))?;

    let dtu_ports = DtuPorts::from_json(&urls_json)?;
    Ok((guard, dtu_ports))
}

// ---------------------------------------------------------------------------
// McpStdioHandle
// ---------------------------------------------------------------------------

/// Handle to prism-bin's MCP stdio transport.
///
/// Wraps stdin/stdout for sending JSON-RPC messages and reading responses.
/// Uses raw JSON-RPC over stdio (Open Question 2 resolution: portable approach).
///
/// Protocol: each JSON-RPC message is a single line terminated with `\n`.
/// Responses are read line-by-line from stdout.
///
/// The `Child` process is owned by `SubprocessGuard` for SIGTERM on drop.
/// `McpStdioHandle` only holds the I/O handles (stdin/stdout taken from the child).
pub struct McpStdioHandle {
    pub stdin: std::process::ChildStdin,
    pub stdout: std::io::BufReader<std::process::ChildStdout>,
    pub next_id: u64,
}

impl McpStdioHandle {
    /// Send a JSON-RPC `method` with `params` and return the parsed response.
    ///
    /// Writes one line to stdin; reads one line from stdout.
    /// Returns the parsed JSON-RPC `result` field on success, or `Err` on
    /// JSON-RPC error or I/O failure.
    pub fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        // Write request line to stdin.
        let line = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize JSON-RPC request: {e}"))?;

        writeln!(self.stdin, "{line}")
            .map_err(|e| format!("Failed to write to prism-bin stdin: {e}"))?;

        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush prism-bin stdin: {e}"))?;

        // Read response line from stdout.
        let mut response_line = String::new();
        self.stdout
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read from prism-bin stdout: {e}"))?;

        if response_line.is_empty() {
            return Err(
                "prism-bin closed stdout unexpectedly (process may have exited)".to_string(),
            );
        }

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse JSON-RPC response '{response_line}': {e}"))?;

        // Check for JSON-RPC error.
        if let Some(err) = response.get("error") {
            return Err(format!("JSON-RPC error from prism-bin: {err}"));
        }

        // Return the result field.
        Ok(response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    /// Send a JSON-RPC `method` with `params` and return the full parsed response,
    /// including the `"error"` object when a JSON-RPC protocol-level error is returned.
    ///
    /// Unlike `send_request`, this method does NOT treat a JSON-RPC error response
    /// as `Err`. Instead it returns `Ok(full_response_json)` so callers can inspect
    /// the `"error"` field.  Only I/O and parse failures are returned as `Err`.
    ///
    /// # When to use
    ///
    /// Use this for tests that need to assert on genuine JSON-RPC protocol-level errors
    /// (e.g., unknown method, malformed request, fatal pre-handler failures). For user-visible
    /// domain errors (E-QUERY-032, validation, permission, etc.), use `send_request` directly —
    /// post BC-2.10.007 (F-2 fix), domain errors return `{ "result": { "isError": true, ... } }`
    /// which `send_request` handles correctly without error propagation.
    fn send_request_allow_rpc_error(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let line = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize JSON-RPC request: {e}"))?;

        writeln!(self.stdin, "{line}")
            .map_err(|e| format!("Failed to write to prism-bin stdin: {e}"))?;

        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush prism-bin stdin: {e}"))?;

        let mut response_line = String::new();
        self.stdout
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read from prism-bin stdout: {e}"))?;

        if response_line.is_empty() {
            return Err(
                "prism-bin closed stdout unexpectedly (process may have exited)".to_string(),
            );
        }

        serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse JSON-RPC response '{response_line}': {e}"))
    }

    /// Send MCP `initialize` → send `notifications/initialized` → return server capabilities.
    ///
    /// Protocol per MCP 2024-11-05 spec (rmcp 1.7):
    /// 1. Client → Server: `initialize` request with clientInfo + protocolVersion
    /// 2. Server → Client: `initialize` result with serverInfo + capabilities
    /// 3. Client → Server: `notifications/initialized` notification (no response)
    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        let capabilities = self.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "prism-e2e-test-harness",
                    "version": "0.1.0"
                }
            }),
        )?;

        // Send initialized notification (fire-and-forget; no response expected).
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        let line = serde_json::to_string(&notification)
            .map_err(|e| format!("Failed to serialize initialized notification: {e}"))?;
        let _ = writeln!(self.stdin, "{line}");
        let _ = self.stdin.flush();

        Ok(capabilities)
    }

    /// Send `tools/list` and return the array of tool objects.
    pub fn tools_list(&mut self) -> Result<Vec<serde_json::Value>, String> {
        let result = self.send_request("tools/list", serde_json::json!({}))?;
        result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .ok_or_else(|| format!("tools/list response missing 'tools' array; got: {result:?}"))
    }

    /// Send `tools/call` for the `query` MCP tool with the given PrismQL string.
    ///
    /// Returns the raw ResponseEnvelope JSON parsed from the MCP tool response.
    ///
    /// The canonical tool name is `"query"` (registered via `pub async fn query` under
    /// `#[tool_router]` in `prism-mcp/src/server.rs`; BC-2.11.001 H1 source-of-truth).
    pub fn tool_query(&mut self, pql: &str) -> Result<serde_json::Value, String> {
        self.tool_query_with_params(pql, None)
    }

    /// Send `tools/call` for the `query` MCP tool scoped to a specific org.
    ///
    /// Used by AC-002..AC-009 (S-DEMO-004) to query from a specific org context (BC-2.11.001 scoping).
    /// The org scope is passed via `clients: [org_slug]` (array of strings) — NOT `org_slug`.
    /// `QueryToolParams` uses `clients: Option<Vec<String>>` and has `#[serde(deny_unknown_fields)]`;
    /// passing `org_slug` would be rejected at deserialization before isolation logic runs.
    pub fn tool_query_scoped(
        &mut self,
        pql: &str,
        org_slug: &str,
    ) -> Result<serde_json::Value, String> {
        self.tool_query_with_params(pql, Some(org_slug))
    }

    /// Send `tools/call` for the `query` tool scoped to an org, returning the full raw
    /// JSON-RPC response object (including any top-level `"error"` field).
    ///
    /// # When to use
    ///
    /// Use this method ONLY when testing genuine **JSON-RPC protocol-level errors** — i.e.,
    /// errors where the server returns `{ "error": { "code": N, "message": "..." } }` at the
    /// JSON-RPC transport layer (e.g., malformed requests, unknown methods, fatal boot failures
    /// that prevent the server from calling the tool handler at all).
    ///
    /// For **user-visible domain errors** (E-QUERY-032 cross-org isolation, validation errors,
    /// permission errors, etc.), use `tool_query_scoped` instead. Post BC-2.10.007 (F-2
    /// fix), domain errors return `Ok(CallToolResult { isError: true, structuredContent: {...} })`
    /// — a JSON-RPC success with `isError=true` in the result — NOT a protocol-level error.
    /// `send_request` handles those correctly; this method is not needed for them.
    ///
    /// # What it returns
    ///
    /// Returns `Ok(full_json_rpc_response)` for both success and protocol-level error responses.
    /// The caller must inspect `response.get("error")` vs `response.get("result")` directly.
    /// Only transport-level I/O and parse failures are returned as `Err`.
    pub fn tool_query_scoped_expect_rpc_error(
        &mut self,
        pql: &str,
        org_slug: &str,
    ) -> Result<serde_json::Value, String> {
        let input = serde_json::json!({
            "query": pql,
            "clients": [org_slug],
        });
        self.send_request_allow_rpc_error(
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": input
            }),
        )
    }

    /// Internal: send `tools/call` for the `query` tool with optional org scoping.
    ///
    /// Returns a normalized ResponseEnvelope with:
    /// - `rows` at the top level (from `results.rows` or `structuredContent.results.rows`)
    /// - `_meta` at the top level with `data_source` normalized to a string
    ///   (if the server returns `data_source` as an array, we extract the first element)
    /// - All other top-level fields preserved for assertion by tests
    ///
    /// This normalization is necessary because:
    /// 1. MCP `tools/call` wraps the result in `{ "content": [...], "structuredContent": {...} }`.
    /// 2. Prism's `query` tool may return `content[0].text` as a human summary ("N results found")
    ///    rather than a JSON blob — so we cannot rely on parsing the text field as JSON.
    /// 3. Tests in `e2e_multi_org.rs` access `result.get("rows")` (top-level) and
    ///    `result.get("_meta").get("data_source").as_str()` — requiring this normalization.
    fn tool_query_with_params(
        &mut self,
        pql: &str,
        org_slug: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let mut input = serde_json::json!({ "query": pql });
        if let Some(slug) = org_slug {
            // BC-2.11.001: scoping param is `clients` (array of org slug strings).
            // QueryToolParams.clients: Option<Vec<String>>; deny_unknown_fields rejects `org_slug`.
            input["clients"] = serde_json::json!([slug]);
        }

        let raw = self.send_request(
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": input
            }),
        )?;

        // Prism's query MCP tool embeds the full ResponseEnvelope JSON as a string
        // inside content[0].text.  The outer MCP `tools/call` result wrapper looks like:
        //
        //   raw = {
        //     "content": [{"type": "text", "text": "<ResponseEnvelope JSON string>"}],
        //     "structuredContent": { ... },
        //     "isError": false,
        //   }
        //
        // where the ResponseEnvelope JSON string is:
        //
        //   {
        //     "_meta": { "data_source": ["sensor_name"], "total_results": N, ... },
        //     "content": [{"type": "text", "text": "N results found"}],
        //     "results": { "rows": [...], "returned_results": N, ... },
        //     "structuredContent": { "results": { ... } }
        //   }
        //
        // We pick the richest source available (text_json if it parses, else raw),
        // then normalize to the shape tests expect:
        //   { "rows": [...], "_meta": { "data_source": "sensor_name", ... }, ... }

        // Unwrap content[0].text if it's a valid JSON object (the ResponseEnvelope path).
        let envelope: serde_json::Value = raw
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
            .filter(|v| v.is_object())
            .unwrap_or(raw);

        // Extract rows: envelope.results.rows (primary) → envelope.structuredContent.results.rows.
        let rows = envelope
            .get("results")
            .and_then(|r| r.get("rows"))
            .cloned()
            .or_else(|| {
                envelope
                    .get("structuredContent")
                    .and_then(|sc| sc.get("results"))
                    .and_then(|r| r.get("rows"))
                    .cloned()
            });

        // Normalize _meta.data_source from Array → String (first element).
        // Tests use `.as_str()` which requires a JSON String, not an Array.
        let normalized_meta = envelope.get("_meta").cloned().map(|mut meta| {
            if let Some(ds_array) = meta
                .get("data_source")
                .and_then(|ds| ds.as_array())
                .map(|a| a.to_owned())
                && let Some(first) = ds_array.first().and_then(|s| s.as_str())
            {
                meta["data_source"] = serde_json::Value::String(first.to_string());
            }
            meta
        });

        // Build normalized result: start from envelope, add top-level `rows` and
        // replace `_meta` with the normalized version.
        let mut normalized = envelope;
        if let Some(rows) = rows {
            normalized["rows"] = rows;
        }
        if let Some(meta) = normalized_meta {
            normalized["_meta"] = meta;
        }
        Ok(normalized)
    }
}

// ---------------------------------------------------------------------------
// launch_prism_bin
// ---------------------------------------------------------------------------

/// Launch `prism start --config-dir <config_dir>` as a subprocess with stdin/stdout pipes.
///
/// Waits for the MCP server to become ready by attempting the `initialize` handshake
/// with up to 30s timeout.
///
/// Returns `(SubprocessGuard, McpStdioHandle)`.
///
/// # Env vars set for TYPE-spec parse
///
/// The canonical sensor TYPE specs use `${env.VAR}` interpolation for `base_url`:
///   - `claroty.sensor.toml`:      `base_url = "${env.CLAROTY_INSTANCE_URL}"`
///   - `armis.sensor.toml`:        `base_url = "${env.ARMIS_INSTANCE_URL}"`
///   - `cyberint.sensor.toml`:     `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io"`
///   - `crowdstrike.sensor.toml`:  `base_url = "${env.CROWDSTRIKE_BASE_URL}"` (S-DEMO-CROWDSTRIKE-MULTIREGION-001)
///
/// `env_resolver.rs` resolves these at spec-load time (before per-org overlays override
/// `base_url` to the DTU URL). If the env var is absent or empty, the spec-engine emits
/// E-SPEC-024 and boot fails. The placeholder values below are deliberately set to valid
/// non-empty strings — the per-org overlay (`specs/customers/{org}/sensor.toml`) always
/// overrides `base_url` to the DTU clone URL before any HTTP request is made.
///
/// `CYBERINT_ENVIRONMENT` is embedded inside the URL string:
/// `"https://demo.cyberint.io"` (demo is a valid hostname component).
/// The overlay replaces the full `base_url` with the DTU URL anyway.
///
/// `CROWDSTRIKE_BASE_URL` must be a URL whose host is in the crowdstrike-oauth2 plugin's
/// `allowed_urls` to pass SEC-003 at step 7.5b. The DTU-safe manifest has
/// `allowed_urls = ["api.crowdstrike.com", "127.0.0.1"]`. We use `"http://127.0.0.1"` —
/// the overlay replaces it with the actual DTU clone port URL before any HTTP contact.
///
/// # Plugin loading
///
/// PRISM_DISABLE_PLUGIN_LOAD is NOT set. The crowdstrike-oauth2 plugin is staged into
/// the temp plugins dir by `stage_crowdstrike_plugin` (called from `write_org_config`).
/// crowdstrike.sensor.toml requires auth_plugin = "crowdstrike-oauth2" (D-747 LOCKED).
/// Without the plugin, step 7.5b fails with BootError::UnknownAuthPlugin before the MCP
/// server binds, producing the EC-002 "Broken pipe" error.
///
/// # DTU-EXT-001 (SID-1 compliance)
/// This function requires a live boot sequence. It is called only from
/// `#[ignore]`'d E2E tests that are un-gated via the 'e2e' nextest profile.
pub async fn launch_prism_bin(
    config_dir: &Path,
) -> Result<(SubprocessGuard, McpStdioHandle), String> {
    let prism_bin = locate_binary("prism")?;

    // Spawn prism-bin with stdin/stdout pipes for MCP JSON-RPC communication.
    //
    // Env vars required for TYPE-spec ${env.VAR} interpolation (resolved at spec-load time,
    // before per-org overlays override base_url to the DTU URL):
    //   CLAROTY_INSTANCE_URL    — placeholder; overlay overrides base_url to DTU clone URL.
    //   ARMIS_INSTANCE_URL      — placeholder; overlay overrides base_url to DTU clone URL.
    //   CYBERINT_ENVIRONMENT    — "demo" produces "https://demo.cyberint.io"; override to DTU URL.
    //   CROWDSTRIKE_BASE_URL    — placeholder; overlay overrides base_url to DTU clone URL.
    //     (S-DEMO-CROWDSTRIKE-MULTIREGION-001 changed base_url to ${env.CROWDSTRIKE_BASE_URL};
    //      E-SPEC-024 fires if absent. Value is irrelevant — overlay replaces it.)
    //
    // PRISM_DISABLE_PLUGIN_LOAD is intentionally NOT set: the crowdstrike-oauth2 plugin is
    // staged by write_org_config/stage_crowdstrike_plugin into the temp plugins dir, and
    // boot step 7.5b requires it to resolve auth_plugin = "crowdstrike-oauth2" (D-747).
    //
    // RUST_LOG=off: The tracing subscriber (step1_init_tracing) writes to stdout by default
    // (tracing_subscriber::fmt::layer() → stdout). The MCP stdio transport also uses stdout
    // for JSON-RPC messages. Without log suppression, boot log lines go to stdout and are
    // interleaved with JSON-RPC responses, making McpStdioHandle.send_request() fail with
    // "Failed to parse JSON-RPC response" on every line that contains a log entry.
    // Setting RUST_LOG=off eliminates all tracing output in the subprocess so the stdout
    // stream carries only clean JSON-RPC protocol messages.
    //
    // This is correct for E2E tests: we test protocol behavior, not log output. Log output
    // correctness is covered by unit tests (step1_init_tracing, BC-2.06.011 AC-5 first-log-line).
    // Per-client env-var convention (ADR-032 / BC-2.06.003):
    // Format: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}
    // where {ID} = org_slug uppercased with hyphens → underscores.
    //
    // Set per-client env vars for ALL orgs used by the E2E tests:
    //   - write_demo_config:       1 org  — demo-org  → DEMO_ORG
    //   - write_multi_org_demo_config: 3 orgs — demo-org-a → DEMO_ORG_A,
    //                                           demo-org-b → DEMO_ORG_B,
    //                                           demo-org-c → DEMO_ORG_C
    //
    // Multi-org sensor assignment (write_multi_org_demo_config):
    //   demo-org-a: CrowdStrike + Armis
    //   demo-org-b: Claroty + Cyberint
    //   demo-org-c: all 4 sensors
    //
    // Extra vars set for unused orgs are harmless (probe wildcard scan: any org succeeds).
    let mut child = std::process::Command::new(&prism_bin)
        .arg("start")
        .arg("--config-dir")
        .arg(config_dir)
        .env("CLAROTY_INSTANCE_URL", "http://placeholder.claroty.invalid")
        .env("ARMIS_INSTANCE_URL", "http://placeholder.armis.invalid")
        .env("CYBERINT_ENVIRONMENT", "demo")
        // CROWDSTRIKE_BASE_URL: required by crowdstrike.sensor.toml TYPE spec since
        // S-DEMO-CROWDSTRIKE-MULTIREGION-001 changed base_url to "${env.CROWDSTRIKE_BASE_URL}".
        // E-SPEC-024 fires at spec-load time if absent.
        //
        // IMPORTANT: SEC-003 validates the token_endpoint host (base_url + "/oauth2/token")
        // against the plugin manifest's allowed_urls at step 7.5b. The TYPE spec base_url
        // is validated BEFORE per-org overlays are applied, so the value must be a host
        // present in the crowdstrike-oauth2 DTU-safe manifest's allowed_urls:
        // ["api.crowdstrike.com", "127.0.0.1"] (see stage_crowdstrike_plugin).
        //
        // Using "http://127.0.0.1" satisfies SEC-003 (host "127.0.0.1" is in the allowlist).
        // The per-org overlay then replaces base_url with the actual DTU clone port URL
        // (http://127.0.0.1:<ephemeral_port>) before any HTTP request is made.
        .env("CROWDSTRIKE_BASE_URL", "http://127.0.0.1")
        // ---------- Armis bearer_token (orgs: demo-org, demo-org-a, demo-org-c) ----------
        // Resolved by BearerStaticCredentialAuthProvider via
        // resolve_credential(org_slug, "armis", "bearer_token") (BC-2.06.003 Tier 2).
        // The Armis DTU clone validates Authorization: Bearer {non-empty}.
        // ADR-031 §D3-b / ADR-032 per-client convention.
        .env(
            "PRISM_CLIENTS_DEMO_ORG_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_A_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_C_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        // ---------- Claroty bearer_token (orgs: demo-org, demo-org-b, demo-org-c) ----------
        // Resolved by BearerStaticCredentialAuthProvider via
        // resolve_credential(org_slug, "claroty", "bearer_token") (BC-2.06.003 Tier 2).
        // The Claroty DTU clone validates Authorization: Bearer {non-empty}.
        .env(
            "PRISM_CLIENTS_DEMO_ORG_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_B_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_C_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        // ---------- Cyberint api_key (orgs: demo-org, demo-org-b, demo-org-c) ----------
        // Must match the access_token registered in the Cyberint DTU's allowlist via
        // initial_access_token in demo.toml. The DTU validates the `access_token` cookie
        // against its allowlist; this value must be identical on both sides.
        // ADR-031 §D3-a: static cookie auth; no login roundtrip.
        // Resolved via resolve_credential(org_slug, "cyberint", "api_key") (BC-2.06.003 Tier 2).
        .env(
            "PRISM_CLIENTS_DEMO_ORG_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_B_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_C_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        // ---------- CrowdStrike client_id (orgs: demo-org, demo-org-a, demo-org-c) ----------
        // Used by the crowdstrike-oauth2 WASM plugin to POST client credentials to the DTU's
        // /oauth2/token endpoint. The CrowdStrike DTU accepts any non-empty client_id/secret pair.
        // Resolved via resolve_credential(org_slug, "crowdstrike", "client_id") (BC-2.06.003).
        .env(
            "PRISM_CLIENTS_DEMO_ORG_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        // ---------- CrowdStrike client_secret (orgs: demo-org, demo-org-a, demo-org-c) ----------
        // Resolved via resolve_credential(org_slug, "crowdstrike", "client_secret") (BC-2.06.003).
        .env(
            "PRISM_CLIENTS_DEMO_ORG_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_DEMO_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env("RUST_LOG", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // Boot log noise already suppressed by RUST_LOG=off.
        .spawn()
        .map_err(|e| format!("Failed to spawn prism-bin '{}': {e}", prism_bin.display()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or("prism-bin stdin not available after spawn")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("prism-bin stdout not available after spawn")?;

    // SubprocessGuard owns the child (for SIGTERM on drop).
    // McpStdioHandle owns only the I/O handles (taken from child before guard creation).
    let guard = SubprocessGuard::new(child, "prism");

    let mut handle = McpStdioHandle {
        stdin,
        stdout: std::io::BufReader::new(stdout),
        next_id: 1,
    };

    // Wait for MCP server to become ready by polling initialize handshake.
    // EC-002: if prism-bin exits before MCP handshake, return clear error.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match handle.initialize() {
            Ok(_) => return Ok((guard, handle)),
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(format!(
                        "prism-bin MCP server did not become ready within 30s (EC-002): {e}"
                    ));
                }
                // Brief pause before retry.
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// locate_binary
// ---------------------------------------------------------------------------

// Locate a workspace binary by name.
//
// Search order:
// 1. `CARGO_BIN_EXE_<name>` env var (set by cargo for binaries in the same package).
//    NOTE: `CARGO_BIN_EXE_*` is only populated for binaries declared in the SAME
//    package as the integration test binary. Cross-package binaries (`prism`,
//    `prism-dtu-demo-server`) are NOT set by cargo from within `prism-bin`'s test
//    harness. The env-var path is kept as a forward-compatibility hook.
// 2. Workspace `target/release/<name>` — the release binary is required by
//    Architecture Compliance Rule 5 (30-second subprocess timeout assumes release
//    performance). This is the documented precondition for running E2E tests.
// 3. Workspace `target/debug/<name>` — fallback ONLY when release is absent.
//    NOT silent: emits a visible `eprintln!` diagnostic before returning the path.
//    Debug binaries may cause E2E timeout failures (30s limit assumes release speed).
// 4. Returns `Err(...)` with an actionable `cargo build --release` message if
//    neither release nor debug binary exists.
//
// OBS-1: There is NO silent fallback path. Every binary selection path either
// returns `Ok` with a log/diagnostic or returns `Err` with a clear message.
//
// Precondition:
// Run `cargo build --release -p prism -p prism-dtu-demo-server` before running E2E tests.
// The CI e2e profile ensures this; local runs require the manual build step.
// ---------------------------------------------------------------------------
// S-DEMO-004: Multi-org harness helpers
// ---------------------------------------------------------------------------
//
// BackgroundHarness: wraps a MultiInstanceHarness running on a dedicated
// multi-thread tokio runtime in a background thread. This is REQUIRED because
// the E2E tests use `#[tokio::test]` (current-thread runtime) and synchronous
// `std::io::BufReader::read_line` to read prism's stdout. That blocking call
// starves all tokio tasks in the current-thread runtime, including the in-process
// DTU clone axum server tasks — causing every sensor HTTP request from prism to
// time out (30s) because the DTU server cannot accept connections.
//
// Root cause: current-thread tokio runtime + blocking `read_line` = deadlock on
// any in-process async server (DTU clones) that prism is trying to reach.
//
// Fix: run the MultiInstanceHarness (and its axum server tasks) on a separate
// multi-thread tokio runtime in a background thread. The blocking `read_line`
// can't affect that background runtime. The socket_map is extracted and copied
// for use from the test thread (write_multi_org_overlays only needs socket addresses).

/// A guard that keeps a `MultiInstanceHarness` alive on a dedicated background
/// tokio multi-thread runtime, decoupled from the test's current-thread runtime.
///
/// `BackgroundHarness` exposes the `socket_map()` extracted at start time.
/// When dropped, it sends a shutdown signal that terminates the background thread
/// and its runtime (which in turn shuts down the DTU clone axum servers via the
/// harness's broadcast shutdown channel).
///
/// # Why this is needed (current-thread deadlock prevention)
///
/// `#[tokio::test]` uses a current-thread runtime. When the test calls the
/// synchronous `send_request` / `read_line`, the current thread is blocked.
/// Since only one thread runs the tokio tasks, the in-process DTU clone axum servers
/// cannot accept connections while the test is waiting for prism's response.
/// Running the harness on a SEPARATE multi-thread runtime in a background thread
/// ensures the DTU clone tasks run independently of any blocking on the test thread.
pub struct BackgroundHarness {
    /// Extracted socket_map from MultiInstanceHarness — (org_slug, sensor_id) → SocketAddr.
    /// Exposed via socket_map() for use by write_multi_org_overlays.
    socket_map: std::collections::HashMap<(String, String), std::net::SocketAddr>,
    /// Shutdown signal sender. Wrapped in `Option<_>` so the explicit `Drop` impl
    /// can take ownership (to drop it first) before joining the thread.
    ///
    /// The sender is always `Some` except after `Drop::drop` has taken it.
    _shutdown_tx: Option<std::sync::mpsc::SyncSender<()>>,
    /// Background thread handle. `Option<_>` so that `Drop::drop` can take
    /// ownership for the join call (O-02: deterministic teardown).
    ///
    /// Always `Some` except after `Drop::drop` has taken it.
    _thread: Option<std::thread::JoinHandle<()>>,
}

impl BackgroundHarness {
    /// Return the socket_map from the MultiInstanceHarness: (org_slug, sensor_id) → SocketAddr.
    ///
    /// Used by `write_multi_org_overlays` to write per-org overlay TOML files.
    pub fn socket_map(&self) -> &std::collections::HashMap<(String, String), std::net::SocketAddr> {
        &self.socket_map
    }
}

impl Drop for BackgroundHarness {
    fn drop(&mut self) {
        // O-02: deterministic teardown.
        //
        // ORDERING IS CRITICAL:
        //   1. Drop `_shutdown_tx` FIRST — this closes the sync_channel, causing
        //      the background thread's `shutdown_rx.recv()` to return `Err(RecvError)`.
        //      The thread then exits (dropping the harness + axum servers).
        //   2. Join `_thread` AFTER — by then the thread is already exiting/exited,
        //      so join returns quickly. Ensures the axum DTU clone servers have
        //      fully drained before the test harness tears down.
        //
        // Without this explicit ordering, a naive `Drop` impl that called `join()`
        // first would deadlock because `_shutdown_tx` would still be alive (not yet
        // dropped by field-drop, which runs after `Drop::drop` returns).
        drop(self._shutdown_tx.take()); // closes channel → thread exits
        if let Some(handle) = self._thread.take() {
            let _ = handle.join(); // wait for thread to finish
        }
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-004 multi-org harness helpers (GREEN — implemented)
//
// Seeds (fixed per story risk_mitigations):
//   org-a crowdstrike: 100  |  org-a armis: 110
//   org-b claroty:     120  |  org-b cyberint: 130
//   org-c crowdstrike: 200  |  org-c armis:    210
//   org-c claroty:     220  |  org-c cyberint: 230
//
// Org UUIDs (fixed UUIDv7 for deterministic tests, BC-2.21.001):
//   org-a: "019700a0-0000-7000-8000-000000000021"
//   org-b: "019700a0-0000-7000-8000-000000000022"
//   org-c: "019700a0-0000-7000-8000-000000000023"
//
// CRITICAL: 8hex prefix for device ID assertions MUST be derived from
// hex(org_id.as_bytes()[0..4]) of the UUID assigned above — NOT the human slug.
// E.g., for "019700a0-0000-7000-8000-000000000021", bytes[0..4] = [0x01, 0x97, 0x00, 0xa0]
// → 8hex = "019700a0". Device IDs for org-a CrowdStrike match "dev-019700a0-100-\d+".
//
// Story: S-DEMO-004
// BCs: BC-3.2.001, BC-2.06.017, BC-2.06.018
// ---------------------------------------------------------------------------

/// Fixed UUIDv7 org IDs used in all S-DEMO-004 multi-org tests (BC-2.21.001).
///
/// These MUST match the org_id values written in write_multi_org_prism_toml()
/// and the OrgId bytes passed to new_with_seed() in start_multi_org_harness().
///
/// Device ID 8hex prefix is derived from hex(org_id.as_bytes()[0..4]):
///   ORG_A_ID bytes[0..4] = [0x01, 0x97, 0x00, 0xa0] → "019700a0"
///   ORG_B_ID bytes[0..4] = [0x01, 0x97, 0x00, 0xa0] → "019700a0"  (same first 4 bytes)
///   ORG_C_ID bytes[0..4] = [0x01, 0x97, 0x00, 0xa0] → "019700a0"  (same first 4 bytes)
///
/// IMPORTANT: since these UUIDs share the same first 4 bytes (they are all v7
/// UUIDs in the same time bucket), INV-DISTINCT-DATA-001 is proven by the SEED
/// component of the ID ("dev-{8hex}-{seed}-{n}") rather than the 8hex alone:
///   org-a IDs: "dev-019700a0-100-N"
///   org-c IDs: "dev-019700a0-200-N"
/// The seed component (100 vs 200) makes the sets structurally disjoint.
pub const ORG_A_ID: &str = "019700a0-0000-7000-8000-000000000021";
pub const ORG_A_SLUG: &str = "org-a";
pub const ORG_B_ID: &str = "019700a0-0000-7000-8000-000000000022";
pub const ORG_B_SLUG: &str = "org-b";
pub const ORG_C_ID: &str = "019700a0-0000-7000-8000-000000000023";
pub const ORG_C_SLUG: &str = "org-c";

/// Seeds per (org, sensor) pair for S-DEMO-004.
///
/// Seeds are DISTINCT across orgs sharing the same sensor type (org-a and org-c both
/// have CrowdStrike; seeds 100 vs 200 satisfy INV-DISTINCT-DATA-001 per BC-2.06.018).
pub const SEED_ORG_A_CROWDSTRIKE: u64 = 100;
pub const SEED_ORG_A_ARMIS: u64 = 110;
pub const SEED_ORG_B_CLAROTY: u64 = 120;
pub const SEED_ORG_B_CYBERINT: u64 = 130;
pub const SEED_ORG_C_CROWDSTRIKE: u64 = 200;
pub const SEED_ORG_C_ARMIS: u64 = 210;
pub const SEED_ORG_C_CLAROTY: u64 = 220;
pub const SEED_ORG_C_CYBERINT: u64 = 230;

/// Start a `MultiInstanceHarness` with 8 DTU clone instances for the 3-org test matrix.
///
/// Org/sensor matrix:
///   org-a: crowdstrike (seed=100), armis (seed=110)
///   org-b: claroty (seed=120), cyberint (seed=130)
///   org-c: crowdstrike (seed=200), armis (seed=210), claroty (seed=220), cyberint (seed=230)
///
/// Each clone is constructed via `new_with_seed(seed, Archetype::HealthyOtEnvironment, org_id)`.
/// `ArmisClone::new_with_seed` and `CyberintClone::new_with_seed` are fallible (return
/// `anyhow::Result<Self>`); `CrowdstrikeClone` and `ClarotyClone` are infallible.
///
/// Returns (harness, tempdir) — tempdir is kept alive by the caller for the test duration.
/// The harness socket_map is keyed by (org_slug, sensor_id) plain strings.
///
/// # E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
///
/// # Current-thread deadlock prevention
///
/// `#[tokio::test]` uses a current-thread runtime. Calling blocking `read_line` on
/// prism's stdout starves all tasks in that runtime — including any in-process axum
/// server tasks. To avoid deadlock, this function builds the harness on a SEPARATE
/// multi-thread tokio runtime in a background thread, then extracts the socket_map
/// and returns a `BackgroundHarness` guard. The test thread can then block freely.
///
/// The function is `async` so test callers can use `.await` (tests use `#[tokio::test]`).
/// Internally, the background thread + its own dedicated multi-thread runtime are set up via
/// `std::thread::spawn`, which creates a raw OS thread that owns a new
/// `tokio::runtime::Builder::new_multi_thread()` runtime. The runtime runs the harness's
/// axum server tasks independently of anything on the test thread.
pub async fn start_multi_org_harness() -> (BackgroundHarness, TempDir) {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_claroty::ClarotyClone;
    use prism_dtu_common::{Archetype, BehavioralClone as _, OrgId};
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_dtu_cyberint::CyberintClone;
    use prism_dtu_harness::multi_instance::HarnessEntry;

    // Parse org UUIDs into OrgId([u8; 16]) — pattern from harness.rs parse_org_id.
    let make_org_id = |uuid_str: &str| -> OrgId {
        let uuid = uuid::Uuid::parse_str(uuid_str).expect("org UUID must be valid");
        OrgId(*uuid.as_bytes())
    };

    let org_id_a = make_org_id(ORG_A_ID);
    let org_id_b = make_org_id(ORG_B_ID);
    let org_id_c = make_org_id(ORG_C_ID);

    let archetype = Archetype::HealthyOtEnvironment;

    // Build 8 HarnessEntry items — one per (org_slug, sensor_id) pair.
    // HarnessEntry is #[non_exhaustive]; MUST use HarnessEntry::new() (not struct-literal).
    // ArmisClone::new_with_seed and CyberintClone::new_with_seed are fallible — propagate
    // via .expect() (test code; panic is acceptable per #[allow(unwrap_used)] in lints).
    // OrgId is Clone (not Copy) — clone when reusing across multiple constructors.
    // Archetype is Copy — no clone needed.
    let entries: Vec<HarnessEntry> = vec![
        // org-a: CrowdStrike (seed=100) — infallible constructor
        HarnessEntry::new(
            ORG_A_SLUG,
            "crowdstrike",
            Box::new(CrowdstrikeClone::new_with_seed(
                SEED_ORG_A_CROWDSTRIKE,
                archetype,
                org_id_a.clone(),
            )),
        ),
        // org-a: Armis (seed=110) — fallible constructor
        HarnessEntry::new(
            ORG_A_SLUG,
            "armis",
            Box::new(
                ArmisClone::new_with_seed(SEED_ORG_A_ARMIS, archetype, org_id_a)
                    .expect("ArmisClone::new_with_seed for org-a must succeed"),
            ),
        ),
        // org-b: Claroty (seed=120) — infallible constructor
        HarnessEntry::new(
            ORG_B_SLUG,
            "claroty",
            Box::new(ClarotyClone::new_with_seed(
                SEED_ORG_B_CLAROTY,
                archetype,
                org_id_b.clone(),
            )),
        ),
        // org-b: Cyberint (seed=130) — fallible constructor
        // Register the E2E access token before boxing so prism's Cookie header passes auth.
        // CyberintClone::new_with_seed does NOT pre-register any access token; without this
        // step the access_token_allowlist is empty and every query returns 0 results (AC-007).
        HarnessEntry::new(ORG_B_SLUG, "cyberint", {
            let cy_b = CyberintClone::new_with_seed(SEED_ORG_B_CYBERINT, archetype, org_id_b)
                .expect("CyberintClone::new_with_seed for org-b must succeed");
            cy_b.configure(serde_json::json!({"access_token": DTU_E2E_CYBERINT_ACCESS_TOKEN}))
                .await
                .expect("CyberintClone configure (org-b access_token) must succeed");
            Box::new(cy_b) as Box<dyn prism_dtu_common::BehavioralClone>
        }),
        // org-c: CrowdStrike (seed=200) — infallible constructor; DISTINCT seed from org-a (100≠200)
        HarnessEntry::new(
            ORG_C_SLUG,
            "crowdstrike",
            Box::new(CrowdstrikeClone::new_with_seed(
                SEED_ORG_C_CROWDSTRIKE,
                archetype,
                org_id_c.clone(),
            )),
        ),
        // org-c: Armis (seed=210) — fallible constructor
        HarnessEntry::new(
            ORG_C_SLUG,
            "armis",
            Box::new(
                ArmisClone::new_with_seed(SEED_ORG_C_ARMIS, archetype, org_id_c.clone())
                    .expect("ArmisClone::new_with_seed for org-c must succeed"),
            ),
        ),
        // org-c: Claroty (seed=220) — infallible constructor
        HarnessEntry::new(
            ORG_C_SLUG,
            "claroty",
            Box::new(ClarotyClone::new_with_seed(
                SEED_ORG_C_CLAROTY,
                archetype,
                org_id_c.clone(),
            )),
        ),
        // org-c: Cyberint (seed=230) — fallible constructor
        // Register the E2E access token before boxing (same reason as org-b above).
        HarnessEntry::new(ORG_C_SLUG, "cyberint", {
            let cy_c = CyberintClone::new_with_seed(SEED_ORG_C_CYBERINT, archetype, org_id_c)
                .expect("CyberintClone::new_with_seed for org-c must succeed");
            cy_c.configure(serde_json::json!({"access_token": DTU_E2E_CYBERINT_ACCESS_TOKEN}))
                .await
                .expect("CyberintClone configure (org-c access_token) must succeed");
            Box::new(cy_c) as Box<dyn prism_dtu_common::BehavioralClone>
        }),
    ];

    let tempdir = TempDir::new().expect("failed to create temp dir for multi-org harness");

    // --- BackgroundHarness: run DTU clones on a dedicated multi-thread runtime ---
    //
    // The test uses #[tokio::test] (current-thread runtime). When the test blocks on
    // synchronous `read_line`, no tokio tasks in that runtime can execute — including
    // any in-process axum server tasks. To prevent deadlock, start the harness on a
    // SEPARATE multi-thread tokio runtime in a background thread.
    //
    // Channel protocol:
    //   (a) `socket_tx` / `socket_rx` — tokio oneshot: background thread sends the
    //       extracted socket_map after the harness starts; async fn `.await`s on it.
    //   (b) `shutdown_tx` — std SyncSender: when BackgroundHarness is dropped (test
    //       teardown), this is dropped, closing the channel. The background thread
    //       receives `Err(RecvError)` from `shutdown_rx.recv()` and exits, dropping
    //       the harness (which fires MultiInstanceHarness::drop → graceful-shutdown).
    let (socket_tx, socket_rx) = tokio::sync::oneshot::channel::<
        std::collections::HashMap<(String, String), std::net::SocketAddr>,
    >();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::sync_channel::<()>(0);

    let thread = std::thread::spawn(move || {
        // Build a multi-thread tokio runtime in this background thread.
        // This runtime owns the DTU clone axum server tasks — it runs independently
        // of anything happening on the test thread.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("BackgroundHarness: failed to build multi-thread tokio runtime");

        // Start the harness inside the runtime.
        let harness = rt
            .block_on(prism_dtu_harness::multi_instance::MultiInstanceHarness::start(entries))
            .expect("MultiInstanceHarness::start must succeed for all 8 clone entries");

        // Extract and send the socket_map to the test thread via oneshot.
        let map = harness.socket_map().clone();
        // send() is infallible on the happy path (receiver alive until .await completes).
        let _ = socket_tx.send(map);

        // Keep the harness alive (and thus the axum server tasks running) until
        // the test is done. Block here waiting for the shutdown signal.
        // When BackgroundHarness is dropped, shutdown_tx is dropped, closing the
        // channel, and `recv()` returns Err(RecvError) → we exit, dropping harness.
        let _ = shutdown_rx.recv(); // Err(RecvError) on shutdown_tx drop = expected
        // harness dropped here → MultiInstanceHarness::drop fires shutdown_tx.send(())
        // → axum graceful-shutdown receivers wake → servers drain → thread exits.
        drop(harness);
    });

    // Receive the socket_map from the background thread (.await is non-blocking to the
    // current-thread runtime — it just yields until the background thread sends the map).
    let socket_map = socket_rx
        .await
        .expect("BackgroundHarness: socket_rx must receive socket_map (background thread alive)");

    let bg_harness = BackgroundHarness {
        socket_map,
        _shutdown_tx: Some(shutdown_tx),
        _thread: Some(thread),
    };

    (bg_harness, tempdir)
}

/// Write per-org overlay TOML files from the harness socket_map into `tempdir/specs/customers/`.
///
/// Calls `prism_dtu_harness::overlay_wiring::write_overlay_from_socket_map(harness.socket_map(), specs_dir)`
/// where `specs_dir = tempdir.path().join("specs")`.
///
/// The overlay files are written under `{tempdir}/specs/customers/{org_slug}/{sensor_id}.sensor.toml`
/// per BC-2.06.017 Postcondition 3 (overlay integration end-to-end). The `specs` directory is
/// created if not present. prism.toml sets `spec_dir = {tempdir}/specs` so the overlay walk
/// resolves to the correct customer sub-directory.
///
/// # Note on harness type
///
/// Takes `&BackgroundHarness` (not `&MultiInstanceHarness`) because the harness
/// runs on a background thread's multi-thread tokio runtime to avoid the
/// current-thread deadlock (see `start_multi_org_harness` doc comment).
pub fn write_multi_org_overlays(
    harness: &BackgroundHarness,
    tempdir: &TempDir,
) -> Result<(), String> {
    // Create {tempdir}/specs if not yet present (write_overlay_from_socket_map writes
    // {specs_dir}/customers/{org}/{sensor}.sensor.toml).
    let specs_dir = tempdir.path().join("specs");
    std::fs::create_dir_all(&specs_dir)
        .map_err(|e| format!("Failed to create specs dir '{}': {e}", specs_dir.display()))?;

    prism_dtu_harness::overlay_wiring::write_overlay_from_socket_map(
        harness.socket_map(),
        specs_dir.as_path(),
    )
    .map_err(|e| {
        format!(
            "write_overlay_from_socket_map failed for multi-org harness \
             (BC-2.06.017 Postcondition 3): {e}"
        )
    })
}

/// Write a 3-org prism.toml to `tempdir` for the S-DEMO-004 multi-org test.
///
/// Writes:
///   - `{tempdir}/prism.toml` with 3 `[[orgs]]` entries (org-a, org-b, org-c)
///     using the fixed UUIDv7 IDs from ORG_A_ID / ORG_B_ID / ORG_C_ID.
///   - Sets `spec_dir = {tempdir}/specs`, `state_dir`, `plugin_dir` paths within the tempdir.
///
/// The per-org overlay TOML files are already written by `write_multi_org_overlays` under
/// `{tempdir}/specs/customers/{org_slug}/{sensor_id}.sensor.toml`; this function writes the
/// top-level prism.toml that references `{tempdir}/specs` as `spec_dir`.
///
/// Also copies canonical TYPE specs from the workspace sensor specs directory
/// (`crates/prism-sensors/specs/`) into `{tempdir}/specs/`, stages the
/// crowdstrike-oauth2 plugin, and creates the `state/` and `plugins/` directories.
///
/// # Pattern
/// Mirrors `write_org_config()` — see the S-DEMO-002 helper for the established pattern.
///
/// Story: S-DEMO-004 AC-001..AC-010
pub fn write_multi_org_prism_toml(tempdir: &TempDir) -> Result<(), String> {
    let config_dir = tempdir.path();
    let specs_dir = config_dir.join("specs");
    let state_dir = config_dir.join("state");
    let plugins_dir = config_dir.join("plugins");

    // Create required directories (specs_dir may already exist from write_multi_org_overlays).
    for dir in [&specs_dir, &state_dir, &plugins_dir] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory '{}': {e}", dir.display()))?;
    }

    // Stage the crowdstrike-oauth2 plugin (required by crowdstrike.sensor.toml auth_plugin field).
    // Without this, boot step 7.5b raises BootError::UnknownAuthPlugin before MCP binds (EC-002).
    stage_crowdstrike_plugin(&plugins_dir)?;

    // Copy canonical TYPE specs from workspace into temp specs_dir.
    let workspace_specs = workspace_sensor_specs_dir();
    for sensor_id in ["crowdstrike", "armis", "claroty", "cyberint"] {
        let src = workspace_specs.join(format!("{sensor_id}.sensor.toml"));
        let dst = specs_dir.join(format!("{sensor_id}.sensor.toml"));
        std::fs::copy(&src, &dst).map_err(|e| {
            format!(
                "Failed to copy sensor spec '{}' → '{}': {e}",
                src.display(),
                dst.display()
            )
        })?;
    }

    // Build [[orgs]] TOML section for all 3 orgs.
    // org_id UUIDs MUST match those used in start_multi_org_harness() / new_with_seed() calls
    // so device ID 8hex prefixes derive consistently from the same UUID bytes.
    let orgs_toml = format!(
        "\n[[orgs]]\norg_id = \"{ORG_A_ID}\"\norg_slug = \"{ORG_A_SLUG}\"\n\
         \n[[orgs]]\norg_id = \"{ORG_B_ID}\"\norg_slug = \"{ORG_B_SLUG}\"\n\
         \n[[orgs]]\norg_id = \"{ORG_C_ID}\"\norg_slug = \"{ORG_C_SLUG}\"\n"
    );

    // Write prism.toml — Windows-safe path serialization via {:?} (matches write_org_config pattern).
    let prism_toml = format!(
        "# Generated by S-DEMO-004 multi-org E2E test harness — do not edit manually.\n\
         spec_dir   = {:?}\n\
         state_dir  = {:?}\n\
         plugin_dir = {:?}\n\
         {}\n",
        specs_dir.display(),
        state_dir.display(),
        plugins_dir.display(),
        orgs_toml.trim()
    );

    let prism_toml_path = config_dir.join("prism.toml");
    std::fs::write(&prism_toml_path, &prism_toml)
        .map_err(|e| format!("Failed to write prism.toml: {e}"))
}

/// Launch `prism start` with per-org credential env vars for the 3-org multi-tenant test.
///
/// Sets PRISM_CLIENTS_ORG_{A,B,C}_SENSORS_* env vars for all sensors active per org:
///   org-a: crowdstrike (client_id, client_secret), armis (bearer_token)
///   org-b: claroty (bearer_token), cyberint (api_key)
///   org-c: all 4 sensors
///
/// The org slug env prefix is ORG_A_SLUG.replace('-', '_').to_uppercase() → "ORG_A", etc.
/// Per ADR-032 / BC-2.06.003: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}.
///
/// Also sets RUST_LOG=off, CROWDSTRIKE_BASE_URL=http://127.0.0.1, and the
/// sensor placeholder env vars (CLAROTY_INSTANCE_URL, ARMIS_INSTANCE_URL,
/// CYBERINT_ENVIRONMENT) following the same pattern as launch_prism_bin().
///
/// # E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
pub async fn launch_prism_bin_multi_org(
    config_dir: &Path,
) -> Result<(SubprocessGuard, McpStdioHandle), String> {
    let prism_bin = locate_binary("prism")?;

    // Env prefix per org slug (ADR-032 / BC-2.06.003):
    //   "org-a" → replace '-' with '_' → "org_a" → uppercase → "ORG_A"
    //   "org-b" → "ORG_B"
    //   "org-c" → "ORG_C"
    //
    // Full env var: PRISM_CLIENTS_{PREFIX}_SENSORS_{SENSOR}_{REF}

    let mut child = std::process::Command::new(&prism_bin)
        .arg("start")
        .arg("--config-dir")
        .arg(config_dir)
        // TYPE-spec ${env.VAR} interpolation placeholders (same as launch_prism_bin).
        // Per-org overlays replace base_url with the actual DTU clone URL before any HTTP call.
        .env("CLAROTY_INSTANCE_URL", "http://placeholder.claroty.invalid")
        .env("ARMIS_INSTANCE_URL", "http://placeholder.armis.invalid")
        .env("CYBERINT_ENVIRONMENT", "demo")
        // CROWDSTRIKE_BASE_URL: SEC-003 validates this against the plugin's allowed_urls at step 7.5b.
        // DTU-safe manifest has allowed_urls = ["api.crowdstrike.com", "127.0.0.1"].
        // Using "http://127.0.0.1" satisfies SEC-003; per-org overlay replaces it for actual DTU calls.
        .env("CROWDSTRIKE_BASE_URL", "http://127.0.0.1")
        // ---------- org-a credentials (sensors: crowdstrike + armis) ----------
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        // ---------- org-b credentials (sensors: claroty + cyberint) ----------
        .env(
            "PRISM_CLIENTS_ORG_B_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_B_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        // ---------- org-c credentials (all 4 sensors) ----------
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        // Suppress all tracing output so MCP JSON-RPC stdout is not corrupted by log lines.
        // prism-bin step1_init_tracing uses fmt::layer() which defaults to stdout;
        // any non-off RUST_LOG level will inject log lines into the MCP stdio stream,
        // causing McpStdioHandle to read a WARN line instead of the JSON-RPC response
        // and shift all subsequent reads by one (tools/list reads init response, etc.).
        .env("RUST_LOG", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn prism-bin '{}': {e}", prism_bin.display()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or("prism-bin stdin not available after spawn")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("prism-bin stdout not available after spawn")?;

    let guard = SubprocessGuard::new(child, "prism-multi-org");

    let mut handle = McpStdioHandle {
        stdin,
        stdout: std::io::BufReader::new(stdout),
        next_id: 1,
    };

    // Poll for MCP server readiness (same pattern as launch_prism_bin — 30s timeout).
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match handle.initialize() {
            Ok(_) => return Ok((guard, handle)),
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(format!(
                        "prism-bin multi-org MCP server did not become ready within 30s (EC-002): {e}"
                    ));
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

/// Launch `prism start` for multi-org tests and capture stderr for boot-event assertion.
///
/// Identical to `launch_prism_bin_multi_org` except:
/// - `RUST_LOG=boot=info` (instead of `off`) so that the `boot.step9a.adapter_registry_populated`
///   tracing event is emitted to stderr.
/// - `PRISM_LOG_FORMAT=json` so each event is a machine-readable JSON object on its own line.
/// - `stderr(Stdio::piped())` — stderr is captured.
/// - A background thread buffers the subprocess stderr into a `Vec<u8>` shared via
///   `Arc<Mutex<_>>`. The caller MUST poll the arc AFTER MCP readiness is established —
///   at that point all boot-phase log lines (including step9a) have been emitted.
///
/// Returns `(prism_guard, mcp_handle, stderr_buf)`.
///
/// The caller asserts on `stderr_buf` after MCP readiness, then drops everything.
///
/// # AC-001 / BC-2.22.001 use case
///
/// The `boot.step9a.adapter_registry_populated` event carries `sensor_count` (total across
/// all orgs) and `org_count`. The event is emitted once per boot (in JSON format, one line).
/// The test parses all lines for the event and asserts `sensor_count == 8` (2+2+4).
///
/// # E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
pub async fn launch_prism_bin_multi_org_with_stderr(
    config_dir: &Path,
) -> Result<
    (
        SubprocessGuard,
        McpStdioHandle,
        std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    ),
    String,
> {
    let prism_bin = locate_binary("prism")?;

    let stderr_buf: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut child = std::process::Command::new(&prism_bin)
        .arg("start")
        .arg("--config-dir")
        .arg(config_dir)
        // Env var placeholders (same as launch_prism_bin_multi_org).
        .env("CLAROTY_INSTANCE_URL", "http://placeholder.claroty.invalid")
        .env("ARMIS_INSTANCE_URL", "http://placeholder.armis.invalid")
        .env("CYBERINT_ENVIRONMENT", "demo")
        .env("CROWDSTRIKE_BASE_URL", "http://127.0.0.1")
        // org-a credentials
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_ORG_A_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        // org-b credentials
        .env(
            "PRISM_CLIENTS_ORG_B_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_B_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        // org-c credentials
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_ID",
            "dtu-e2e-crowdstrike-client-id",
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CROWDSTRIKE_CLIENT_SECRET",
            "dtu-e2e-crowdstrike-client-secret",
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_ARMIS_BEARER_TOKEN",
            DTU_E2E_ARMIS_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CLAROTY_BEARER_TOKEN",
            DTU_E2E_CLAROTY_BEARER_TOKEN,
        )
        .env(
            "PRISM_CLIENTS_ORG_C_SENSORS_CYBERINT_API_KEY",
            DTU_E2E_CYBERINT_ACCESS_TOKEN,
        )
        // M-01: capture boot-phase events at info level; suppress all non-boot targets.
        // "boot=info" enables just the "boot" tracing target (used by spec_driven_adapter.rs
        // step9a_populate_adapter_registry) without flooding stdout with debug-level frames.
        // Other targets stay silent so the test does not time out waiting for an MCP response
        // mixed with verbose log output.
        .env("RUST_LOG", "boot=info")
        // JSON log format: one event per line — easy to parse for boot.step9a assertion.
        .env("PRISM_LOG_FORMAT", "json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn prism-bin '{}': {e}", prism_bin.display()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or("prism-bin stdin not available after spawn")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("prism-bin stdout not available after spawn")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("prism-bin stderr not available after spawn")?;

    let guard = SubprocessGuard::new(child, "prism-multi-org-with-stderr");

    // Spawn a background thread to drain prism's stderr into `stderr_buf`.
    //
    // Without this drain, the subprocess can block when its stderr OS pipe buffer
    // fills (~64 KiB on Linux/macOS), causing MCP polling to time out.
    // The thread holds an Arc clone of `stderr_buf`; the test reads it after
    // MCP readiness is established (by which time all boot-phase events are emitted).
    let buf_clone = std::sync::Arc::clone(&stderr_buf);
    std::thread::spawn(move || {
        use std::io::Read as _;
        let mut buf = Vec::new();
        let mut stderr_reader = stderr;
        // Read to EOF (the subprocess closes stderr when it exits).
        // We accumulate all bytes; the test consumes them after MCP handshake.
        let _ = stderr_reader.read_to_end(&mut buf);
        if let Ok(mut locked) = buf_clone.lock() {
            locked.extend_from_slice(&buf);
        }
    });

    let mut handle = McpStdioHandle {
        stdin,
        stdout: std::io::BufReader::new(stdout),
        next_id: 1,
    };

    // Poll for MCP server readiness (same 30s timeout as launch_prism_bin_multi_org).
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match handle.initialize() {
            Ok(_) => return Ok((guard, handle, stderr_buf)),
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(format!(
                        "prism-bin multi-org MCP server did not become ready within 30s \
                         (EC-002, launch_prism_bin_multi_org_with_stderr): {e}"
                    ));
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

fn locate_binary(name: &str) -> Result<PathBuf, String> {
    // Env var name: replace hyphens with underscores per cargo convention.
    // NOTE: CARGO_BIN_EXE_* is only set for cross-package bins if declared as
    // [[bin]] dev-dep or build-dep — not currently the case for `prism` / DTU.
    let env_name = format!("CARGO_BIN_EXE_{}", name.replace('-', "_"));
    if let Ok(path) = std::env::var(&env_name) {
        let pb = PathBuf::from(&path);
        if pb.exists() {
            return Ok(pb);
        }
    }

    // Derive workspace root from CARGO_MANIFEST_DIR.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set; cannot locate binary".to_string())?;
    let workspace_root = PathBuf::from(manifest_dir)
        .parent() // prism-bin
        .ok_or("manifest dir has no parent")?
        .parent() // crates/
        .ok_or("crates dir has no parent")?
        .to_path_buf();

    // Prefer release build (Architecture Compliance Rule 5: 30-second E2E timeout
    // assumes release-build performance). Stale debug binaries next to a fresh release
    // binary would produce non-deterministic test timing — release-first eliminates this.
    let release_bin = workspace_root.join("target/release").join(name);
    if release_bin.exists() {
        return Ok(release_bin);
    }

    // Debug fallback: permitted ONLY when no release binary exists. NOT silent — emit a
    // clear diagnostic so developers know they are running a potentially slower binary.
    // If the debug binary is stale, the E2E timeout (30s) will surface the issue.
    // OBS-1 resolution: no silent fallback path; every binary selection is accounted for.
    let debug_bin = workspace_root.join("target/debug").join(name);
    if debug_bin.exists() {
        // PRECONDITION VIOLATION — emit a visible diagnostic, not a silent fallback.
        // eprintln! is permitted in test helpers (not production code paths).
        eprintln!(
            "[E2E PRECONDITION WARNING] locate_binary: release binary not found for '{name}'. \
             Falling back to debug binary at {debug_bin:?}. Debug binaries may cause E2E \
             timeout failures (30s limit assumes release performance). \
             Run `cargo build --release -p {name}` before running E2E tests \
             (Architecture Compliance Rule 5)."
        );
        return Ok(debug_bin);
    }

    // Neither release nor debug binary found — fail with a clear actionable error.
    Err(format!(
        "Binary '{name}' not found in release or debug target directories. \
         Run `cargo build --release -p {name}` before running E2E tests \
         (release build required for 30s E2E timing; Architecture Compliance Rule 5). \
         Searched: {release_bin:?}, {debug_bin:?}"
    ))
}
