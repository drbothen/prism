// SPDX-License-Identifier: Apache-2.0
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
//! Story: S-DEMO-002 v1.6
//! BCs: BC-2.22.001, BC-2.10.010, BC-3.2.001

use std::collections::HashMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// DtuPorts
// ---------------------------------------------------------------------------

/// Port map parsed from `.prism-dtu-demo-server.urls.json`.
///
/// Keys match clone names (`"crowdstrike"`, `"armis"`, `"claroty"`, `"cyberint"`).
/// Values are `"http://127.0.0.1:<port>"` strings.
///
/// DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
/// AdapterRegistry layer only (AC-013 scope clarification).
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

/// Write a prism.toml with 3 orgs configured for multi-tenant isolation tests (AC-011..013).
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
/// This is by design (AC-013).
///
/// AC-011 expected result: `AdapterRegistry.len() == 8` (2+2+4 entries).
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
    let prism_toml = format!(
        "# Generated by S-DEMO-002 E2E test harness — do not edit manually.\n\
         spec_dir   = \"{}\"\n\
         state_dir  = \"{}\"\n\
         plugin_dir = \"{}\"\n\
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
    /// including the `"error"` object when a JSON-RPC error is returned.
    ///
    /// Unlike `send_request`, this method does NOT treat a JSON-RPC error response
    /// as `Err`. Instead it returns `Ok(full_response_json)` so callers can inspect
    /// the `"error"` field.  Only I/O and parse failures are returned as `Err`.
    ///
    /// Used by AC-012 (cross-org isolation): the `query` handler returns a
    /// JSON-RPC error (code -32602) when `resolve_source_refs` raises E-QUERY-032
    /// (sensor not registered for the requesting org); the test must capture that
    /// error object and verify it carries the E-QUERY-032 signal — it must NOT panic.
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
    /// Used by AC-012/AC-013 to query from a specific org context (BC-2.11.001 scoping).
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

    /// Send `tools/call` for the `query` tool scoped to an org, expecting a JSON-RPC error.
    ///
    /// Used exclusively by AC-012 (cross-org isolation). When `resolve_source_refs`
    /// raises E-QUERY-032 (sensor not registered for the requesting org), the MCP server
    /// emits a JSON-RPC error response with code -32602.
    /// `tool_query_scoped` (which calls `send_request`) would propagate this as `Err` and
    /// `.expect()` would panic — hiding the error content the test needs to inspect.
    ///
    /// This method uses `send_request_allow_rpc_error` so the full JSON-RPC response
    /// (including the `"error"` object) is returned as `Ok(json)`. The test then asserts
    /// the error contains the E-QUERY-032 / "is not registered for org" signal.
    ///
    /// For success-path tests use `tool_query_scoped` instead.
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

        let result = self.send_request(
            "tools/call",
            serde_json::json!({
                "name": "query",
                "arguments": input
            }),
        )?;

        // Extract the text content from the MCP tools/call response.
        // MCP tools/call result shape: { "content": [{ "type": "text", "text": "<json>" }], ... }
        if let Some(content) = result.get("content") {
            if let Some(text) = content
                .as_array()
                .and_then(|a| a.first())
                .and_then(|c| c.get("text"))
                .and_then(|t| t.as_str())
            {
                return serde_json::from_str(text).map_err(|e| {
                    format!("Failed to parse tool_query response text as JSON: {e}; text: {text}")
                });
            }
        }

        // Return the raw result if content extraction failed.
        Ok(result)
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
/// # DTU-EXT-001 (SID-1 compliance)
/// This function requires a live boot sequence. It is called only from
/// `#[ignore]`'d E2E tests that are un-gated via the 'e2e' nextest profile.
pub async fn launch_prism_bin(
    config_dir: &Path,
) -> Result<(SubprocessGuard, McpStdioHandle), String> {
    let prism_bin = locate_binary("prism")?;

    // Spawn prism-bin with stdin/stdout pipes for MCP JSON-RPC communication.
    // PRISM_DISABLE_PLUGIN_LOAD=1: skip plugin loading in E2E tests (no .prx files present).
    let mut child = std::process::Command::new(&prism_bin)
        .arg("start")
        .arg("--config-dir")
        .arg(config_dir)
        .env("PRISM_DISABLE_PLUGIN_LOAD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // Suppress boot log noise; check exit code instead.
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

/// Locate a workspace binary by name.
///
/// Search order:
/// 1. `CARGO_BIN_EXE_<name>` env var (set by cargo for binaries in the same package).
///    NOTE: `CARGO_BIN_EXE_*` is only populated for binaries declared in the SAME
///    package as the integration test binary. Cross-package binaries (`prism`,
///    `prism-dtu-demo-server`) are NOT set by cargo from within `prism-bin`'s test
///    harness. The env-var path is kept as a forward-compatibility hook.
/// 2. Workspace `target/release/<name>` — the release binary is required by
///    Architecture Compliance Rule 5 (30-second subprocess timeout assumes release
///    performance). This is the documented precondition for running E2E tests.
/// 3. Workspace `target/debug/<name>` — fallback ONLY when release is absent.
///    NOT silent: emits a visible `eprintln!` diagnostic before returning the path.
///    Debug binaries may cause E2E timeout failures (30s limit assumes release speed).
/// 4. Returns `Err(...)` with an actionable `cargo build --release` message if
///    neither release nor debug binary exists.
///
/// OBS-1: There is NO silent fallback path. Every binary selection path either
/// returns `Ok` with a log/diagnostic or returns `Err` with a clear message.
///
/// # Precondition
/// Run `cargo build --release -p prism -p prism-dtu-demo-server` before running E2E tests.
/// The CI e2e profile ensures this; local runs require the manual build step.
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
