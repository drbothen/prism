// SPDX-License-Identifier: Apache-2.0
//! Test helpers for S-DEMO-002 E2E subprocess smoke test.
//!
//! Provides:
//! - `SubprocessGuard` — drop guard that sends SIGTERM to both prism-bin and DTU server.
//! - `wait_for_file()` — async polling with exponential backoff.
//! - `write_demo_config()` — generates temp prism.toml with DTU port overlays.
//! - `write_multi_org_demo_config()` — 3-org config for multi-tenant isolation tests.
//! - `bootstrap_credentials()` — inserts dummy credentials via OS keyring CLI.
//! - `DtuPorts` — port map parsed from `.prism-dtu-demo-server.urls.json`.
//!
//! All helpers are stubs (compile but `todo!()` body) for the Red Gate phase.
//! The implementer fills the bodies in the TDD green phase.
//!
//! Story: S-DEMO-002 v1.3
//! BCs: BC-2.22.001, BC-2.10.010, BC-3.2.001

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Child;

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
    pub fn from_json(json: &str) -> Result<Self, String> {
        // Stub: implementer parses serde_json::from_str into HashMap<String, String>.
        // FAIL: this is a stub and does not parse anything.
        todo!("S-DEMO-002: DtuPorts::from_json — parse urls.json into port map")
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
/// even if an assertion fails mid-test.
pub struct SubprocessGuard {
    pub child: Child,
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
        use std::os::unix::process::ExitStatusExt;
        // Stub: implementer sends SIGTERM and waits up to 5s.
        // FAIL at Red Gate: does not actually terminate the process.
        let _ = unsafe { libc::kill(self.child.id() as libc::pid_t, libc::SIGTERM) };
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
    // Stub: implementer adds tokio::time::sleep + backoff loop.
    // FAIL at Red Gate: immediately returns Err.
    todo!("S-DEMO-002: wait_for_file — poll for file with exponential backoff")
}

// ---------------------------------------------------------------------------
// bootstrap_credentials
// ---------------------------------------------------------------------------

/// Insert dummy credentials for all 4 sensors into the OS keyring.
///
/// Uses the OS keyring CLI (macOS: `security add-generic-password`;
/// Linux: `secret-tool store`) or the `prism-credentials` test-helpers feature.
///
/// Dummy values per story risk_mitigations[3]: `client_id='test-ci'`,
/// `client_secret='test-ci-secret'`.
///
/// AD-017: credential values MUST NOT appear in source visible to AI context.
/// This helper uses env-var indirection — actual values are sourced from
/// `PRISM_TEST_CI_CLIENT_ID` / `PRISM_TEST_CI_CLIENT_SECRET` env vars in CI.
pub fn bootstrap_credentials(config_dir: &Path) -> Result<(), String> {
    // Stub: implementer adds actual keyring writes.
    // FAIL at Red Gate: returns Ok(()) without inserting anything.
    // Red Gate note: tests that call this will fail at the subsequent query assertion
    // because credentials were never inserted — that is the expected Red Gate failure mode.
    todo!("S-DEMO-002: bootstrap_credentials — insert dummy CI credentials into OS keyring")
}

// ---------------------------------------------------------------------------
// write_demo_config (single-org)
// ---------------------------------------------------------------------------

/// Write a prism.toml with a single `demo-org` org entry and per-sensor DTU overlays.
///
/// Generates:
/// - `<config_dir>/prism.toml` with `demo-org` org entry pointing at spec_dir, plugin_dir.
/// - `<config_dir>/customers/demo-org/crowdstrike.sensor.toml` overriding `base_url`
///   to `dtu_ports.base_url("crowdstrike")`.
/// - Same for `armis`, `claroty`, `cyberint`.
///
/// Calls `bootstrap_credentials(config_dir)` to insert dummy credentials.
pub fn write_demo_config(config_dir: &Path, dtu_ports: &DtuPorts) -> Result<(), String> {
    // Stub: implementer writes temp config files.
    // FAIL at Red Gate: returns Ok(()) without writing any files.
    todo!("S-DEMO-002: write_demo_config — write temp prism.toml + per-sensor overlays")
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
/// Each org gets a distinct `org_id` (UUIDv7) and `org_slug` with corresponding
/// `customers/{slug}/` overlay directories setting DTU clone `base_url` per sensor.
///
/// DTU-MULTI-001: demo DTU operates in single-tenant mode; org isolation is at
/// AdapterRegistry layer only. Two different orgs that both have CrowdStrike
/// point to the same DTU clone port — they receive the same fixture data.
/// This is by design (AC-013).
///
/// AC-011 expected result: `AdapterRegistry.len() == 8` (2+2+4 entries).
pub fn write_multi_org_demo_config(config_dir: &Path, dtu_ports: &DtuPorts) -> Result<(), String> {
    // Stub: implementer writes 3-org temp config files.
    // FAIL at Red Gate: returns Ok(()) without writing any files.
    todo!("S-DEMO-002: write_multi_org_demo_config — write 3-org temp prism.toml")
}

// ---------------------------------------------------------------------------
// launch_dtu_server
// ---------------------------------------------------------------------------

/// Launch `prism-dtu-demo-server start --config <fixture>` as a subprocess.
///
/// Polls for `.prism-dtu-demo-server.urls.json` via `wait_for_file()` with 30s timeout.
/// Returns `(SubprocessGuard, DtuPorts)`.
///
/// Uses the release binary (per Architecture Compliance Rule 5 in the story).
pub async fn launch_dtu_server(
    fixture_config: &Path,
    working_dir: &TempDir,
) -> Result<(SubprocessGuard, DtuPorts), String> {
    // Stub: implementer spawns subprocess and polls urls.json.
    // FAIL at Red Gate: returns Err immediately.
    todo!("S-DEMO-002: launch_dtu_server — spawn DTU server subprocess")
}

// ---------------------------------------------------------------------------
// McpStdioHandle
// ---------------------------------------------------------------------------

/// Handle to prism-bin's MCP stdio transport.
///
/// Wraps stdin/stdout for sending JSON-RPC messages and reading responses.
/// Raw JSON-RPC approach (Open Question 2 resolution: raw is more portable for Red Gate).
pub struct McpStdioHandle {
    pub stdin: std::process::ChildStdin,
    pub stdout: std::io::BufReader<std::process::ChildStdout>,
    pub next_id: u64,
}

impl McpStdioHandle {
    /// Send a JSON-RPC `method` with `params` and return the raw response string.
    pub fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Stub: implementer writes JSON-RPC to stdin and reads from stdout.
        todo!("S-DEMO-002: McpStdioHandle::send_request")
    }

    /// Send MCP `initialize` → `initialized` handshake. Returns server capabilities JSON.
    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        // Stub: implementer sends initialize request per rmcp 1.7 protocol.
        todo!("S-DEMO-002: McpStdioHandle::initialize")
    }

    /// Send `tools/list` and return the array of tool objects.
    pub fn tools_list(&mut self) -> Result<Vec<serde_json::Value>, String> {
        // Stub: implementer sends tools/list and extracts result.tools array.
        todo!("S-DEMO-002: McpStdioHandle::tools_list")
    }

    /// Send `tools/call` for `tool_query` with the given PrismQL string.
    /// Returns the raw ResponseEnvelope JSON.
    pub fn tool_query(&mut self, pql: &str) -> Result<serde_json::Value, String> {
        // Stub: implementer sends tools/call with tool_query input.
        todo!("S-DEMO-002: McpStdioHandle::tool_query")
    }

    /// Send `tools/call` for `tool_query` with an explicit org_slug scope.
    ///
    /// Used by AC-012 to query from a specific org context (BC-2.11.001 scoping).
    /// The `org_slug` is passed in the tool input parameters so the query engine
    /// routes via AdapterRegistry.get(org_id, sensor_id) for that specific org.
    pub fn tool_query_scoped(
        &mut self,
        pql: &str,
        org_slug: &str,
    ) -> Result<serde_json::Value, String> {
        // Stub: implementer sends tools/call with tool_query input including org_slug scoping.
        todo!("S-DEMO-002: McpStdioHandle::tool_query_scoped — include org_slug in tool input")
    }
}

// ---------------------------------------------------------------------------
// launch_prism_bin
// ---------------------------------------------------------------------------

/// Launch `prism start --config <config_dir>` as a subprocess with stdin/stdout pipes.
///
/// Waits for MCP `initialize` readiness (signals: prism logs "MCP server ready" or
/// the test drives the initialize handshake directly).
///
/// Returns `(SubprocessGuard, McpStdioHandle)`.
pub async fn launch_prism_bin(
    config_dir: &Path,
) -> Result<(SubprocessGuard, McpStdioHandle), String> {
    // Stub: implementer spawns prism-bin and wraps stdio.
    // FAIL at Red Gate: returns Err immediately.
    todo!("S-DEMO-002: launch_prism_bin — spawn prism-bin subprocess with stdio pipes")
}
