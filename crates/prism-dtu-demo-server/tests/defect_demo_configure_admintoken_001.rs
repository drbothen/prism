//! # DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 — Test suite
//!
//! ## Story
//!
//! DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001: cmd_configure missing X-Admin-Token header —
//! POST /dtu/configure returns 401.
//!
//! ## Root cause (fixed)
//!
//! `cmd_configure()` in `main.rs` called `POST /dtu/configure` WITHOUT the `X-Admin-Token`
//! header required by ADR-003 Amendment #5 item 5. Every clone returned HTTP 401. There was
//! also no mechanism for `cmd_configure` (a separate process invocation) to obtain the
//! per-clone admin token — no token sidecar was written at server-start time.
//!
//! ## Fixes applied (T-01..T-09)
//!
//! T-01: `TOKEN_FILE` / `TOKEN_MULTI_FILE` pub constants added to `lib.rs`.
//! T-02: `MultiInstanceServers::admin_token_map()` accessor + `token_map` field added.
//! T-03: `write_token_sidecar_to_path` helper added to `harness.rs`.
//! T-04: `write_token_sidecar` called in `cmd_start()` immediately after `write_url_sidecar`.
//! T-05: `write_multi_admin_token_sidecar_to_path` added to `multi_org_cmd.rs`.
//! T-06: `write_multi_admin_token_sidecar` called in `cmd_start_multi()`.
//! T-07: `resolve_configure_token` added to `multi_org_cmd.rs`.
//! T-08: `cmd_configure()` calls `resolve_configure_token` and attaches `X-Admin-Token` header.
//! T-09: Token sidecars removed on shutdown alongside URL sidecars.
//!
//! ## TD-VSDD-060 sibling sweep (AC-004)
//!
//! All POST /dtu/configure call sites in client code enumerated in §Root Cause:
//!
//! | Site | X-Admin-Token? | Status |
//! |------|----------------|--------|
//! | `cmd_configure()` — `main.rs` (exercised by Test E binary E2E) | YES (FIXED) | DEFECT → FIXED |
//! | `ac_3_configure_called_on_clone_port_directly` | YES | Correct |
//! | `ac_3_no_harness_proxy_for_configure` | YES | Correct |
//! | `prism-dtu-crowdstrike` `td_wv0_07_*` | YES | Correct |
//! | `prism-dtu-{claroty,cyberint,armis,...}` `td_wv0_07_*` | YES | Correct |
//! | `bc_2_06_019_scenario_progression.rs` | YES | Correct |
//!
//! Only `cmd_configure()` was missing the header. The sibling-sweep comment block at the
//! top of `cmd_configure()` in `main.rs` documents all enumerated sites (AC-004).
//!
//! ## Test inventory
//!
//! | Test | AC | Finding closed | Load-bearing: what revert breaks this |
//! |------|----|----------------|---------------------------------------|
//! | Test A: `test_BC_3_6_001_replicates_defect_401_without_admin_token` | AC-001 ¶3 | contract lock | Removing server-side 401 gate breaks this |
//! | Test B: `test_BC_2_06_017_token_sidecar_written_and_configure_with_token_returns_200` | AC-001/AC-002 | F-ADMTOK-P1-HIGH-003 | Removing `header("X-Admin-Token", token)` from POST → 401 assertion fails |
//! | Test C: `test_BC_3_6_001_e_demo_007_configure_no_sidecar_present` | AC-003 EC-004 | F-ADMTOK-P1-HIGH-003 | Removing E-DEMO-007 error return from `resolve_configure_token` fails |
//! | Test D: `test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name` | AC-003 EC-005 | — | Removing EC-005 ambiguity path from `resolve_configure_token` fails |
//! | Test E: `test_BC_3_6_001_ac001_binary_configure_with_sidecar_token_returns_200` | AC-001 ¶4 | F-ADMTOK-P1-HIGH-001 | Reverting T-08 (removing `X-Admin-Token` header in binary) → configure exits 1 |
//! | Test F: `test_BC_2_06_017_start_multi_admin_token_map_and_sidecar_written` | AC-002 | F-ADMTOK-P1-HIGH-002 | Removing T-02 (`admin_token_map`) or T-05 (sidecar write) fails |

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use prism_dtu_demo_server::{
    harness::{build_clone_pairs, DemoHarness},
    TOKEN_FILE, TOKEN_MULTI_FILE,
};

// ---------------------------------------------------------------------------
// Test A — GREEN / contract lock
// AC-001 (BC-3.6.001 Precondition 4)
// ---------------------------------------------------------------------------

/// AC-001 (Test A) — Contract lock: POST /dtu/configure without X-Admin-Token → HTTP 401.
///
/// Traces to: BC-3.6.001 Precondition 4 ("configure calls must be authenticated with
/// that clone's admin_token").
///
/// Load-bearing assertion: This test passes both before and after the fix. It locks the
/// server's 401 contract so no regression can accidentally remove the authentication
/// requirement. Removing the server-side admin-token check would break this test.
///
/// The fix is CLIENT-SIDE only (cmd_configure now sends the token via T-08). The SERVER
/// must always reject requests that omit X-Admin-Token — that contract is permanent.
///
/// AC-004 cross-reference: See `cmd_configure()` in `main.rs` for the TD-VSDD-060
/// sibling-sweep table that documents all /dtu/configure POST call sites.
#[tokio::test]
async fn test_BC_3_6_001_replicates_defect_401_without_admin_token() {
    // AC-001 (Test A): PASSES both before and after fix — contract lock.
    // Traces to: BC-3.6.001 Precondition 4.

    // Start a single-clone harness (CrowdStrike) to replicate the defect shape.
    let config = common::single_clone_config("crowdstrike");
    let pairs = build_clone_pairs(&config).expect("AC-001/Test-A: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);
    harness
        .start_all(&config, None)
        .await
        .expect("AC-001/Test-A: start_all must succeed");

    // Locate CrowdStrike's bound address.
    let cs_pair = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .expect("AC-001/Test-A: crowdstrike must be in pairs");
    let cs_addr = cs_pair
        .bound_addr
        .expect("AC-001/Test-A: crowdstrike must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/dtu/configure");

    // POST /dtu/configure WITHOUT X-Admin-Token — replicates the pre-fix cmd_configure defect.
    // ADR-003 Amendment #5: all callers of POST /dtu/configure must include
    // `.header("X-Admin-Token", clone.admin_token())`. cmd_configure omitted it (now fixed).
    // AC-001 specifies {"auth_mode": "accept"} as the representative payload shape for this
    // contract-lock test. The server rejects the request pre-parse (missing X-Admin-Token)
    // so the payload body is never inspected — but aligning to the AC literal is required.
    let payload = serde_json::json!({ "auth_mode": "accept" });
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("AC-001/Test-A: POST to /dtu/configure must not fail at transport level");

    // Contract lock: unauthenticated configure MUST return 401 before AND after fix.
    // The fix changes the CLIENT (cmd_configure sends the token); the SERVER 401 is permanent.
    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-001/Test-A: POST /dtu/configure without X-Admin-Token must return 401; \
         got: {}. This contract must hold both before and after the defect fix.",
        resp.status()
    );

    harness.stop_all().await;
}

// ---------------------------------------------------------------------------
// Test B — GREEN post-fix / F-ADMTOK-P1-HIGH-003 re-anchor
// AC-001 + AC-002 (BC-3.6.001 Precondition 4, BC-2.06.017 Postcondition 1)
// ---------------------------------------------------------------------------

/// AC-001/AC-002 (Test B) — Re-anchored: token sidecar written via pub helper; configure
/// with sidecar-sourced token returns 200.
///
/// Traces to:
///   BC-3.6.001 Precondition 4: configure requests must be authenticated with the per-clone
///     admin_token. This test verifies the sidecar-read → header-attach path works end-to-end.
///   BC-2.06.017 Postcondition 1: `TOKEN_FILE` / `TOKEN_MULTI_FILE` are the token sidecar
///     artifacts parallel to the URL sidecars governed by this BC.
///
/// Load-bearing assertion: Removing `.header("X-Admin-Token", &token)` from the POST in
/// this test would yield HTTP 401 and break the assertion. This directly mirrors the defect
/// pattern (T-08 fix) — the test verifies that a correctly-sourced token authorises the call.
///
/// Re-anchored from the original RED gate which used a shared CWD path (TOKEN_FILE as a
/// literal on-disk file at the process cwd) causing cross-test races. This version uses
/// a per-test temp directory and `write_token_sidecar_to_path` (pub helper, T-03) to write
/// and `resolve_configure_token` (T-07) to read, with no shared-CWD file paths.
#[tokio::test]
async fn test_BC_2_06_017_token_sidecar_written_and_configure_with_token_returns_200() {
    // AC-001/AC-002 (Test B): GREEN post-fix.
    // Uses per-test temp dir — no shared CWD paths (cross-test race fix, F-ADMTOK-P1-HIGH-003).

    let tmp = tempfile::tempdir().expect("Test-B: tempdir must be created");
    let token_sidecar_path = tmp.path().join(TOKEN_FILE);

    // Start a single-clone harness (CrowdStrike) so we have a live clone to POST to.
    let config = common::single_clone_config("crowdstrike");
    let pairs = build_clone_pairs(&config).expect("Test-B: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);
    harness
        .start_all(&config, None)
        .await
        .expect("Test-B: start_all must succeed");

    // Write token sidecar to a per-test temp path using the pub helper (T-03).
    // In the binary, this is called by `write_token_sidecar()` in `cmd_start()` (T-04).
    // The helper writes atomically (tmp+rename) per GAP-3 sidecar-availability guarantee.
    let token_map = harness.token_map();
    prism_dtu_demo_server::write_token_sidecar_to_path(&token_map, &token_sidecar_path)
        .expect("Test-B: write_token_sidecar_to_path must succeed");

    // Verify sidecar was written (sanity check on the pub helper).
    assert!(
        token_sidecar_path.exists(),
        "Test-B/AC-002: TOKEN_FILE must exist after write_token_sidecar_to_path. \
         Pub helper T-03 must write the sidecar atomically."
    );

    // Resolve the token via the same function used by cmd_configure (T-07).
    let token = prism_dtu_demo_server::resolve_configure_token(
        "crowdstrike",
        Some(&token_sidecar_path),
        None,
    )
    .expect("Test-B: resolve_configure_token must succeed with a valid sidecar");

    // Token must be non-empty (UUID v4).
    assert!(
        !token.is_empty(),
        "Test-B: resolved token must be non-empty (UUID v4 from CrowdstrikeClone::new)"
    );

    // Locate CrowdStrike's bound address.
    let cs_pair = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .expect("Test-B: crowdstrike must be in pairs");
    let cs_addr = cs_pair
        .bound_addr
        .expect("Test-B: crowdstrike must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/dtu/configure");

    // POST /dtu/configure WITH the sidecar-sourced token must return 200.
    //
    // LOAD-BEARING (F-ADMTOK-P1-HIGH-001 complement): removing `.header("X-Admin-Token", &token)`
    // from this POST yields HTTP 401 → assertion fails. This mirrors the T-08 fix in cmd_configure.
    let payload = serde_json::json!({ "seed": 42 });
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Admin-Token", &token)
        .json(&payload)
        .send()
        .await
        .expect("Test-B: POST /dtu/configure with sidecar token must not fail at transport");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-002: POST /dtu/configure with token from TOKEN_FILE must return 200; got: {}. \
         Load-bearing: remove the X-Admin-Token header → 401.",
        resp.status()
    );

    harness.stop_all().await;
    // tmp is dropped here — temp dir auto-cleaned by tempfile.
}

// ---------------------------------------------------------------------------
// Test C — GREEN post-fix / F-ADMTOK-P1-HIGH-003 re-anchor
// AC-003, EC-004 (BC-3.6.001 Precondition 4)
// ---------------------------------------------------------------------------

/// AC-003 (Test C) — Re-anchored: `resolve_configure_token` with absent sidecar paths
/// returns E-DEMO-007.
///
/// Traces to: BC-3.6.001 Precondition 4.
/// Error taxonomy: E-DEMO-007 (story §Error Taxonomy Addition, registered in T-11).
///
/// EC-004: Token sidecar missing (server not running or different cwd).
/// Expected E-DEMO-007 error message template:
///   "configure: E-DEMO-007: admin token for clone 'crowdstrike' could not be resolved:
///    token sidecar not found (start the demo server first with start or start-multi)"
///
/// Load-bearing assertion: Removing the E-DEMO-007 error return from `resolve_configure_token`
/// (i.e., returning Ok("") instead) would break the `expect_err` assertion.
///
/// Re-anchored from the original RED gate which started a harness, wrote TOKEN_FILE to CWD,
/// and then deleted it. This version uses nonexistent temp paths directly — no harness
/// needed, no CWD pollution.
#[tokio::test]
async fn test_BC_3_6_001_e_demo_007_configure_no_sidecar_present() {
    // AC-003, EC-004 (Test C): GREEN post-fix.
    // Uses nonexistent temp paths — no harness, no shared CWD files.

    let tmp = tempfile::tempdir().expect("Test-C: tempdir must be created");
    // These paths do NOT exist — simulating EC-004 (sidecar absent).
    let flat_path = tmp.path().join(TOKEN_FILE);
    let nested_path = tmp.path().join(TOKEN_MULTI_FILE);

    // Both paths are absent — neither sidecar was written. EC-004: sidecar missing.
    assert!(
        !flat_path.exists(),
        "Test-C: flat path must not exist for EC-004 scenario"
    );
    assert!(
        !nested_path.exists(),
        "Test-C: nested path must not exist for EC-004 scenario"
    );

    // resolve_configure_token with both paths absent → E-DEMO-007 (EC-004).
    //
    // LOAD-BEARING: removing the EC-004 error return from `resolve_configure_token` and
    // returning Ok("") instead would cause `expect_err` to panic, failing this test.
    let result = prism_dtu_demo_server::resolve_configure_token(
        "crowdstrike",
        Some(&flat_path),
        Some(&nested_path),
    );
    let err = result.expect_err("AC-003/EC-004: absent sidecar must return E-DEMO-007 error");
    let msg = format!("{err:?}");

    assert!(
        msg.contains("E-DEMO-007"),
        "AC-003/EC-004: error must contain E-DEMO-007 code per story §Error Taxonomy; got: {msg}"
    );
    assert!(
        msg.contains("configure"),
        "AC-003/EC-004: error must match E-DEMO-007 message template 'configure: E-DEMO-007: ...'; got: {msg}"
    );
    assert!(
        msg.contains("token sidecar not found"),
        "AC-003/EC-004: error reason must cite missing sidecar per AC-003 template; got: {msg}"
    );
    // tmp is dropped here — temp dir auto-cleaned by tempfile.
}

// ---------------------------------------------------------------------------
// Test D — GREEN post-fix
// AC-003, EC-005 (BC-3.6.001 Precondition 4, BC-2.06.017 Postcondition 1)
// ---------------------------------------------------------------------------

/// AC-003 (Test D) — GREEN post-fix: E-DEMO-007 for ambiguous bare sensor name in start-multi mode.
///
/// Traces to:
///   BC-3.6.001 Precondition 4: configure requests require the correct per-clone token.
///   BC-2.06.017 Postcondition 1: MultiInstanceServers / admin_token_map governs start-multi
///     token extraction; token_multi_file parallels url_multi_file.
///
/// EC-005: In start-multi mode, if multiple orgs have the same sensor name (e.g.
/// "crowdstrike"), a bare sensor name is ambiguous. Expected E-DEMO-007 message (per
/// story §Edge Cases EC-005 and the multi-match ambiguity arm of `resolve_configure_token`):
///   "Bare sensor name 'crowdstrike' is ambiguous — found in N orgs: ["org-a", "org-b"].
///    Use full '{org_slug}-{sensor_id}' form."
///
/// Load-bearing assertion: Removing the EC-005 ambiguity branch from `resolve_configure_token`
/// and returning the first match instead would break the `expect_err` and `contains("ambiguous")`.
///
/// Uses a per-test temp dir and tempfile::TempDir for isolation (no shared CWD paths).
#[tokio::test]
async fn test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name() {
    // AC-003, EC-005 (Test D): GREEN post-fix.
    // Uses per-test temp dir — no shared CWD files.

    let tmp = tempfile::tempdir().expect("Test-D: tempdir must be created");
    let token_multi_path = tmp.path().join("admin-tokens-multi.ec005-test.json");

    // Create a self-contained multi-org token sidecar with two orgs both having
    // "crowdstrike". This isolates the EC-005 path without needing a live multi-org harness.
    // T-07 (resolve_configure_token) handles bare-sensor disambiguation and must return
    // E-DEMO-007 with the ambiguity message.
    let ambiguous_sidecar: std::collections::HashMap<
        String,
        std::collections::HashMap<String, String>,
    > = [
        (
            "org-a".to_string(),
            [("crowdstrike".to_string(), "fake-token-org-a".to_string())]
                .into_iter()
                .collect(),
        ),
        (
            "org-b".to_string(),
            [("crowdstrike".to_string(), "fake-token-org-b".to_string())]
                .into_iter()
                .collect(),
        ),
    ]
    .into_iter()
    .collect();

    std::fs::write(
        &token_multi_path,
        serde_json::to_string(&ambiguous_sidecar).unwrap(),
    )
    .expect("Test-D: must write temporary ambiguous token sidecar");

    // LOAD-BEARING (EC-005): removing the ambiguity branch from resolve_configure_token
    // and returning the first bare match instead would break `expect_err` → test fails.
    let result = prism_dtu_demo_server::resolve_configure_token(
        "crowdstrike",
        None,
        Some(&token_multi_path),
    );

    let err =
        result.expect_err("AC-003/EC-005: ambiguous bare sensor name must return E-DEMO-007 error");
    let msg = format!("{err:?}");

    assert!(
        msg.contains("E-DEMO-007"),
        "AC-003/EC-005: error must contain E-DEMO-007 code; got: {msg}"
    );
    assert!(
        msg.contains("ambiguous"),
        "AC-003/EC-005: error must cite ambiguity; got: {msg}"
    );
    assert!(
        msg.contains("org-a") && msg.contains("org-b"),
        "AC-003/EC-005: error must name the conflicting orgs; got: {msg}"
    );
    // tmp is dropped here — temp dir auto-cleaned by tempfile.
}

// ---------------------------------------------------------------------------
// Test E — GREEN post-fix / F-ADMTOK-P1-HIGH-001 (binary-level E2E)
// AC-001 ¶4 (BC-3.6.001 Precondition 4)
// Only compiled and run on Unix (uses SIGTERM for clean shutdown).
// ---------------------------------------------------------------------------

/// AC-001 ¶4 (Test E) — Binary-level E2E: `configure` subcommand with sidecar token → HTTP 200.
///
/// Traces to: BC-3.6.001 Precondition 4.
///
/// F-ADMTOK-P1-HIGH-001: The primary defect line (`.header("X-Admin-Token", &admin_token)` in
/// `cmd_configure`, main.rs T-08) had NO load-bearing test before this. This test is the
/// definitive binary-level gate for the fix.
///
/// Load-bearing assertion: Reverting T-08 — removing `.header("X-Admin-Token", &admin_token)`
/// from `cmd_configure` in `main.rs` — causes the configure subprocess to receive HTTP 401,
/// which is non-success, so the binary exits with code 1. This test asserts exit code 0,
/// so it FAILS when the header is absent.
///
/// Subprocess cost: ~400-800ms (start binary + wait for sidecars + run configure).
/// One scoped E2E (happy path) per subprocess cost budget.
///
/// Pattern mirrors `td_wv1_04_binary_tls_e2e.rs` (test precedent for binary subprocess tests).
#[tokio::test]
#[cfg(unix)]
async fn test_BC_3_6_001_ac001_binary_configure_with_sidecar_token_returns_200() {
    use std::time::Duration;

    // AC-001 ¶4 (Test E): binary-level E2E. GREEN post-fix; RED if T-08 is reverted.

    let tmp = tempfile::tempdir().expect("AC-001-E2E: tempdir must be created");

    // Write a minimal single-CrowdStrike config to the temp dir.
    let config_path = e2e_write_single_crowdstrike_config(tmp.path());

    let bin = e2e_binary_path();

    // Spawn `prism-dtu-demo-server start --config <config>` in tmp.path() as cwd.
    // The binary will write TOKEN_FILE and URL_FILE into that cwd.
    let mut start_child = std::process::Command::new(&bin)
        .args(["start", "--config", config_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("AC-001-E2E: start binary must spawn");

    let start_pid = start_child.id();

    // RAII kill-guard (F-ADMTOK-P2-OBS-002): if this test panics before the SIGTERM line,
    // Drop sends SIGKILL so the spawned demo server is not leaked.
    // Safety: SIGKILL to our own subprocess; the OS silently ignores SIGKILL to an
    // already-exited process (returns ESRCH, which we discard).
    struct KillGuard(u32);
    impl Drop for KillGuard {
        fn drop(&mut self) {
            unsafe { libc::kill(self.0 as libc::pid_t, libc::SIGKILL) };
        }
    }
    let _kill_guard = KillGuard(start_pid);

    // Poll for TOKEN_FILE in tmp.path() (written after URL_FILE, so polling TOKEN_FILE
    // is the stricter gate — both sidecars are present once TOKEN_FILE appears).
    // Timeout: 20s (binary needs to bind all clones, write sidecars, complete startup).
    let token_sidecar = tmp.path().join(TOKEN_FILE);
    e2e_wait_for_json_file(&token_sidecar, Duration::from_secs(20));

    // Run `prism-dtu-demo-server configure crowdstrike '{"seed": 99}'` in the same cwd.
    // The configure subcommand reads TOKEN_FILE from cwd to obtain the X-Admin-Token (T-08).
    let configure_output = std::process::Command::new(&bin)
        .args(["configure", "crowdstrike", r#"{"seed": 99}"#])
        .current_dir(tmp.path())
        .output()
        .expect("AC-001-E2E: configure subprocess must complete");

    // Clean shutdown: SIGTERM to the start server.
    e2e_send_sigterm(start_pid);
    let _ = start_child.wait();

    // LOAD-BEARING assertion: configure must exit 0.
    // Revert T-08 (remove X-Admin-Token header in cmd_configure) → server returns 401
    // → binary exits 1 → this assertion fails.
    assert_eq!(
        configure_output.status.code(),
        Some(0),
        "AC-001-E2E: configure must exit 0 with valid X-Admin-Token from TOKEN_FILE sidecar. \
         Load-bearing: reverting T-08 (removing .header(\"X-Admin-Token\", ...) in \
         cmd_configure) causes the server to return HTTP 401 → binary exits 1.",
    );

    // Spot-check: stdout must contain "200" (cmd_configure prints `println!("HTTP {status}")`).
    let stdout = String::from_utf8_lossy(&configure_output.stdout);
    assert!(
        stdout.contains("200"),
        "AC-001-E2E: configure stdout must contain '200'; got stdout: {stdout}"
    );
    // tmp is dropped here — temp dir auto-cleaned by tempfile.
}

// ---------------------------------------------------------------------------
// Test F — GREEN post-fix / F-ADMTOK-P1-HIGH-002 (start-multi admin_token_map)
// AC-002 (BC-2.06.017 Postcondition 1)
// ---------------------------------------------------------------------------

/// AC-002 (Test F) — In-process multi-org test: `start_instances` populates
/// `admin_token_map()`; `write_multi_admin_token_sidecar_to_path` produces the correct
/// nested `{org_slug: {sensor_id: token}}` shape; fail-loud on missing map entry.
///
/// Traces to: BC-2.06.017 Postcondition 1 (MultiInstanceServers is the BC's runtime
/// deliverable; `admin_token_map()` and `TOKEN_MULTI_FILE` are the token parallel to
/// `socket_map()` and `URL_MULTI_FILE`).
///
/// Load-bearing assertions:
/// - Removing T-02 (`token_map` field / `admin_token_map()` from `start_instances`) causes
///   `admin_token_map()` to return empty → `contains_key` assertions fail.
/// - Removing T-05 (`write_multi_admin_token_sidecar_to_path`) causes compile failure.
/// - Removing the fail-loud error return in `write_multi_admin_token_sidecar_to_path`
///   (replacing the `?` with silent skip) causes the "must fail loudly" assertion to fail.
///
/// Uses `start_instances` directly with `CrowdstrikeClone::new()` (non-seeded) — no
/// `fixture-gen` feature required. This tests the admin_token_map wiring independently
/// of fixture data generation.
#[tokio::test]
async fn test_BC_2_06_017_start_multi_admin_token_map_and_sidecar_written() {
    // AC-002 (Test F): GREEN post-fix. No fixture-gen required.
    // Uses per-test temp dir — no shared CWD files.

    use prism_dtu_demo_server::{
        multi_instance::{InstanceEntry, MultiInstanceConfig},
        MultiOrgDemoConfig,
    };

    let tmp = tempfile::tempdir().expect("Test-F: tempdir must be created");
    let token_multi_path = tmp.path().join(TOKEN_MULTI_FILE);

    // Build a 2-instance config: org-a-crowdstrike, org-b-crowdstrike.
    // Instance names mirror the {org_slug}-{sensor_id} convention used by start_multi_for_config.
    let cfg = MultiInstanceConfig::new(vec![
        InstanceEntry::new("org-a-crowdstrike", "127.0.0.1:0".parse().unwrap()),
        InstanceEntry::new("org-b-crowdstrike", "127.0.0.1:0".parse().unwrap()),
    ]);

    // Factory: CrowdstrikeClone::new() — no fixture-gen required (non-seeded).
    // Each new() generates a distinct UUID v4 admin_token, so tokens will differ.
    let servers = prism_dtu_demo_server::start_instances(cfg, |_entry| {
        Box::new(prism_dtu_crowdstrike::CrowdstrikeClone::new())
    })
    .await
    .expect("Test-F: start_instances must succeed for 2-crowdstrike config");

    // --- Assert admin_token_map() has both entries (T-02) ---
    //
    // LOAD-BEARING: removing `token_map` field or `admin_token_map()` from start_instances
    // causes the map to be empty → these assertions fail.
    let token_map = servers.admin_token_map();

    assert_eq!(
        token_map.len(),
        2,
        "Test-F/AC-002: admin_token_map must have exactly 2 entries (one per started instance); \
         got {} entries: {:?}",
        token_map.len(),
        token_map.keys().collect::<Vec<_>>()
    );
    assert!(
        token_map.contains_key("org-a-crowdstrike"),
        "Test-F/AC-002: admin_token_map must contain 'org-a-crowdstrike'; \
         got keys: {:?}",
        token_map.keys().collect::<Vec<_>>()
    );
    assert!(
        token_map.contains_key("org-b-crowdstrike"),
        "Test-F/AC-002: admin_token_map must contain 'org-b-crowdstrike'; \
         got keys: {:?}",
        token_map.keys().collect::<Vec<_>>()
    );
    assert!(
        !token_map["org-a-crowdstrike"].is_empty(),
        "Test-F: org-a-crowdstrike token must be non-empty (UUID v4)"
    );
    assert!(
        !token_map["org-b-crowdstrike"].is_empty(),
        "Test-F: org-b-crowdstrike token must be non-empty (UUID v4)"
    );
    // Each CrowdstrikeClone::new() generates a distinct UUID v4 — tokens must differ.
    assert_ne!(
        token_map["org-a-crowdstrike"], token_map["org-b-crowdstrike"],
        "Test-F: org-a and org-b tokens must be distinct (each CrowdstrikeClone::new() \
         generates a fresh UUID v4)"
    );

    // --- Write multi-org token sidecar to temp path (T-05) and assert nested shape ---
    //
    // Build a matching MultiOrgDemoConfig (2 orgs, each with crowdstrike sensor).
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a1001"
        sensors = ["crowdstrike"]
        seed = 100

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a1002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;
    let multi_org_cfg =
        MultiOrgDemoConfig::from_str(toml).expect("Test-F: MultiOrgDemoConfig must parse");

    // LOAD-BEARING: removing T-05 (write_multi_admin_token_sidecar_to_path) causes
    // compile failure or missing behavior.
    prism_dtu_demo_server::write_multi_admin_token_sidecar_to_path(
        &servers,
        &multi_org_cfg,
        &token_multi_path,
    )
    .expect("Test-F: write_multi_admin_token_sidecar_to_path must succeed");

    // Assert sidecar was written.
    assert!(
        token_multi_path.exists(),
        "Test-F: TOKEN_MULTI_FILE must exist after write_multi_admin_token_sidecar_to_path"
    );

    // Assert nested {org_slug: {sensor_id: token}} shape.
    let sidecar_str = std::fs::read_to_string(&token_multi_path)
        .expect("Test-F: TOKEN_MULTI_FILE must be readable");
    let nested: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        serde_json::from_str(&sidecar_str).expect(
            "Test-F: TOKEN_MULTI_FILE must be valid nested JSON {org_slug: {sensor_id: token}}",
        );

    assert!(
        nested.contains_key("org-a"),
        "Test-F: nested sidecar must contain 'org-a' key; got keys: {:?}",
        nested.keys().collect::<Vec<_>>()
    );
    assert!(
        nested.contains_key("org-b"),
        "Test-F: nested sidecar must contain 'org-b' key; got keys: {:?}",
        nested.keys().collect::<Vec<_>>()
    );
    assert!(
        nested["org-a"].contains_key("crowdstrike"),
        "Test-F: nested['org-a'] must contain 'crowdstrike'; got: {:?}",
        nested["org-a"].keys().collect::<Vec<_>>()
    );
    assert!(
        nested["org-b"].contains_key("crowdstrike"),
        "Test-F: nested['org-b'] must contain 'crowdstrike'; got: {:?}",
        nested["org-b"].keys().collect::<Vec<_>>()
    );

    // Values in the nested sidecar must match what admin_token_map() returned.
    assert_eq!(
        nested["org-a"]["crowdstrike"], token_map["org-a-crowdstrike"],
        "Test-F: nested sidecar token for org-a/crowdstrike must match admin_token_map entry"
    );
    assert_eq!(
        nested["org-b"]["crowdstrike"], token_map["org-b-crowdstrike"],
        "Test-F: nested sidecar token for org-b/crowdstrike must match admin_token_map entry"
    );

    // --- Assert fail-loud on a missing map entry ---
    //
    // Pass a config that claims an "armis" sensor exists for org-a, but only crowdstrike
    // was started. write_multi_admin_token_sidecar_to_path must Err() loudly (not silently skip).
    //
    // LOAD-BEARING: replacing the `?` error return in write_multi_admin_token_sidecar_to_path
    // with a silent skip (filter_map pattern) would allow a missing sensor to be dropped
    // without error — causing configure to fail with 401 for that sensor later. This assertion
    // catches that regression.
    let toml_missing = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a1001"
        sensors = ["crowdstrike", "armis"]
        seed = 100

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a1002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;
    let cfg_missing = MultiOrgDemoConfig::from_str(toml_missing)
        .expect("Test-F: missing-entry config must parse (armis is a valid sensor name)");

    let fail_result = prism_dtu_demo_server::write_multi_admin_token_sidecar_to_path(
        &servers,
        &cfg_missing,
        &token_multi_path,
    );
    assert!(
        fail_result.is_err(),
        "Test-F/AC-002: write_multi_admin_token_sidecar_to_path must Err() loudly when a \
         sensor declared in MultiOrgDemoConfig has no token in admin_token_map(). \
         Load-bearing: a silent skip would cause downstream configure calls to return 401."
    );
    let err_msg = format!("{:?}", fail_result.unwrap_err());
    assert!(
        err_msg.contains("org-a-armis"),
        "Test-F: error must name the missing entry 'org-a-armis'; got: {err_msg}"
    );
    // tmp is dropped here — temp dir auto-cleaned by tempfile.
}

// ---------------------------------------------------------------------------
// Helpers for binary E2E (Test E) — Unix-only
// ---------------------------------------------------------------------------

/// Return the path to the `prism-dtu-demo-server` binary built by cargo.
///
/// `CARGO_BIN_EXE_prism-dtu-demo-server` is set by `cargo test` for every `[[bin]]` target
/// in the crate. Falls back to a workspace-root derivation for IDE/ad-hoc runs.
#[cfg(unix)]
fn e2e_binary_path() -> std::path::PathBuf {
    let var = "CARGO_BIN_EXE_prism-dtu-demo-server";
    std::env::var(var)
        .unwrap_or_else(|_| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set by cargo test");
            let ws_root = std::path::PathBuf::from(&manifest)
                .parent()
                .and_then(|p| p.parent())
                .expect("could not locate workspace root from CARGO_MANIFEST_DIR")
                .to_path_buf();
            ws_root
                .join("target")
                .join("debug")
                .join("prism-dtu-demo-server")
                .to_string_lossy()
                .into_owned()
        })
        .into()
}

/// Write a minimal single-CrowdStrike `DemoConfig` TOML to `dir` and return its path.
///
/// Uses `port = 0` so the OS assigns an ephemeral port (no hard-coded port conflicts).
/// Only CrowdStrike is enabled; all other clones are disabled.
#[cfg(unix)]
fn e2e_write_single_crowdstrike_config(dir: &std::path::Path) -> std::path::PathBuf {
    let toml = r#"
[harness]
bind = "127.0.0.1"

[clones.crowdstrike]
enabled = true
bind = "127.0.0.1"
port = 0
fixture_set = "default"
initial_failure_mode = "None"
seed = 42
tls = false
continue_on_error = false

[clones.claroty]
enabled = false

[clones.cyberint]
enabled = false

[clones.armis]
enabled = false

[clones.threatintel]
enabled = false

[clones.nvd]
enabled = false
"#;
    let path = dir.join("e2e-test-demo.toml");
    std::fs::write(&path, toml).expect("AC-001-E2E: failed to write config TOML");
    path
}

/// Poll `path` until it exists and contains valid non-empty JSON, or panic after `timeout`.
///
/// Used to wait for the binary's TOKEN_FILE (and URL_FILE) to be written atomically.
#[cfg(unix)]
fn e2e_wait_for_json_file(path: &std::path::Path, timeout: std::time::Duration) {
    use std::time::Instant;
    let deadline = Instant::now() + timeout;
    loop {
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if !contents.trim().is_empty() && contents.trim().starts_with('{') {
                    // Sidecar written — JSON present.
                    return;
                }
            }
        }
        if Instant::now() >= deadline {
            panic!(
                "AC-001-E2E: sidecar file {:?} not populated within {:?}",
                path, timeout
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Send SIGTERM to `pid` (Unix only).
#[cfg(unix)]
fn e2e_send_sigterm(pid: u32) {
    // SAFETY: calling libc::kill with a valid pid and SIGTERM is safe.
    let ret = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    assert_eq!(ret, 0, "AC-001-E2E: kill(SIGTERM) to pid {pid} failed");
}
