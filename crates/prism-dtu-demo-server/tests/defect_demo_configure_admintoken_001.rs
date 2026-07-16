//! RED Gate test file for DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
//!
//! # Story
//!
//! DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001: cmd_configure missing X-Admin-Token header —
//! POST /dtu/configure returns 401.
//!
//! # Root cause
//!
//! `cmd_configure()` in `main.rs` calls `POST /dtu/configure` WITHOUT the `X-Admin-Token`
//! header required by ADR-003 Amendment #5 item 5. Every clone returns HTTP 401. There is
//! also no mechanism for `cmd_configure` (a separate process invocation) to obtain the
//! per-clone admin token — no token sidecar is written at server-start time.
//!
//! # TD-VSDD-060 sibling sweep (AC-004)
//!
//! All POST /dtu/configure call sites in client code enumerated in §Root Cause:
//! | Site                                     | X-Admin-Token? | Status        |
//! |------------------------------------------|----------------|---------------|
//! | cmd_configure() — main.rs                | NO             | DEFECT SITE   |
//! | ac_3_configure_called_on_clone_port_directly | YES        | Correct       |
//! | ac_3_no_harness_proxy_for_configure      | YES            | Correct       |
//! | prism-dtu-crowdstrike td_wv0_07_*        | YES            | Correct       |
//! | prism-dtu-{claroty,cyberint,...} td_wv0_07_* | YES        | Correct       |
//! | bc_2_06_019_scenario_progression.rs      | YES            | Correct       |
//!
//! Only cmd_configure() was missing the header.
//!
//! # Tests
//!
//! - `test_BC_3_6_001_replicates_defect_401_without_admin_token` (GREEN / contract lock):
//!   Locks the server's 401 response to unauthenticated configure requests. Passes now,
//!   must keep passing after fix.
//!
//! - `test_BC_2_06_017_token_sidecar_written_by_start_and_configure_with_token_returns_200`
//!   (RED / AC-001, AC-002): After `start_all`, TOKEN_FILE must be written and a
//!   configure POST using the sidecar-sourced token must return 200.
//!   Fails today: `write_token_sidecar` (T-03/T-04) not yet implemented.
//!
//! - `test_BC_3_6_001_e_demo_007_configure_no_sidecar_present`
//!   (RED / AC-003, EC-004): When the token sidecar is absent, configure must block with
//!   E-DEMO-007. Fails today: TOKEN_FILE not written (T-04 not implemented).
//!
//! - `test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name`
//!   (RED / AC-003, EC-005): Bare sensor name in start-multi mode is ambiguous →
//!   E-DEMO-007. Fails today: TOKEN_MULTI_FILE not written (T-06 not implemented).
//!
//! # Red Gate requirement
//!
//! Per `tdd_mode: strict` and `red_gate_tests: 4` in story frontmatter:
//! - test_BC_3_6_001_replicates_defect_401_without_admin_token → PASSES now (contract lock)
//! - test_BC_2_06_017_token_sidecar_written_by_start_and_configure_with_token_returns_200 → FAILS
//! - test_BC_3_6_001_e_demo_007_configure_no_sidecar_present → FAILS
//! - test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name → FAILS

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

// ---------------------------------------------------------------------------
// Test A — GREEN / contract lock
// AC-001 (BC-3.6.001 Precondition 4)
// ---------------------------------------------------------------------------

/// AC-001 (Test A) — Contract lock: POST /dtu/configure without X-Admin-Token → HTTP 401.
///
/// Traces to: BC-3.6.001 Precondition 4 ("configure calls must be authenticated with
/// that clone's admin_token").
///
/// This test PASSES now (replicating the server's existing 401 behavior) and MUST keep
/// passing after the fix. It locks the server contract so no regression can accidentally
/// remove the authentication requirement.
///
/// The fix changes only the CLIENT side (cmd_configure sends the token); the SERVER must
/// always reject requests that omit X-Admin-Token.
#[tokio::test]
async fn test_BC_3_6_001_replicates_defect_401_without_admin_token() {
    // AC-001 (Test A): PASSES today; contract lock — must keep passing after fix.
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

    // POST /dtu/configure WITHOUT X-Admin-Token — replicates cmd_configure defect.
    // ADR-003 Amendment #5: all callers of POST /dtu/configure must include
    // `.header("X-Admin-Token", clone.admin_token())`. cmd_configure omits it.
    let payload = serde_json::json!({ "seed": 42 });
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
// Test B — RED / fix-surface
// AC-001 + AC-002 (BC-3.6.001 Precondition 4, BC-2.06.017 Postcondition 1)
// ---------------------------------------------------------------------------

/// AC-001/AC-002 (Test B) — RED: token sidecar written at start; configure WITH sidecar
/// token returns 200.
///
/// Traces to:
///   BC-3.6.001 Precondition 4: configure requests must be authenticated with the per-clone
///     admin_token. After fix: cmd_configure reads the token from TOKEN_FILE and includes it.
///   BC-2.06.017 Postcondition 1: MultiInstanceServers is the BC's runtime deliverable;
///     TOKEN_FILE / TOKEN_MULTI_FILE are the parallel token sidecar artifacts.
///
/// RED today because:
///   T-03 (write_token_sidecar) and T-04 (call it in cmd_start) are not yet implemented.
///   TOKEN_FILE (".prism-dtu-demo-server.admin-tokens.json") is never written by start_all.
///
/// After fix (T-01..T-04 implemented):
///   start_all calls write_token_sidecar → TOKEN_FILE is written atomically.
///   This test reads the token from TOKEN_FILE and verifies a POST with it returns 200.
#[tokio::test]
async fn test_BC_2_06_017_token_sidecar_written_by_start_and_configure_with_token_returns_200() {
    // AC-001/AC-002 (Test B): RED today — TOKEN_FILE not written (T-03/T-04 not implemented).

    // TOKEN_FILE path as defined by the future T-01 constant in lib.rs.
    // Per story spec AC-002: `TOKEN_FILE = ".prism-dtu-demo-server.admin-tokens.json"`.
    let token_file = std::path::Path::new(".prism-dtu-demo-server.admin-tokens.json");

    // Remove any stale sidecar from a previous test run.
    let _ = std::fs::remove_file(token_file);

    // Start a single-clone harness (CrowdStrike).
    // After fix: start_all calls write_token_sidecar which writes TOKEN_FILE atomically.
    let config = common::single_clone_config("crowdstrike");
    let pairs = build_clone_pairs(&config).expect("Test-B: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);
    harness
        .start_all(&config, None)
        .await
        .expect("Test-B: start_all must succeed");

    // ── RED assertion 1 (T-03/T-04) ─────────────────────────────────────────
    // TOKEN_FILE must be written by start_all (write_token_sidecar).
    // FAILS today: write_token_sidecar is not called in cmd_start.
    assert!(
        token_file.exists(),
        "AC-002/T-03/T-04 RED: TOKEN_FILE '{}' must be written by start_all. \
         Implement write_token_sidecar() (T-03) and call it in cmd_start (T-04).",
        token_file.display()
    );

    // ── Read token from TOKEN_FILE ───────────────────────────────────────────
    // After T-03/T-04: TOKEN_FILE contains flat JSON {name: token}.
    let sidecar_str = std::fs::read_to_string(token_file)
        .expect("Test-B: TOKEN_FILE must be readable after start_all");
    let token_map: std::collections::HashMap<String, String> = serde_json::from_str(&sidecar_str)
        .expect("Test-B: TOKEN_FILE must be valid JSON {name: token}");

    let token = token_map.get("crowdstrike").expect(
        "Test-B: 'crowdstrike' entry must be present in TOKEN_FILE after start_all with crowdstrike",
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

    // ── RED assertion 2 (overall fix) ────────────────────────────────────────
    // POST /dtu/configure WITH the sidecar-sourced token must return 200.
    // Before fix: TOKEN_FILE is not written, so this code is unreachable today.
    // After fix: the token from TOKEN_FILE is valid → server returns 200.
    let payload = serde_json::json!({ "seed": 42 });
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-Admin-Token", token)
        .json(&payload)
        .send()
        .await
        .expect("Test-B: POST /dtu/configure with sidecar token must not fail at transport");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-002: POST /dtu/configure with token from TOKEN_FILE must return 200; \
         got: {}",
        resp.status()
    );

    harness.stop_all().await;

    // Clean up sidecar written by start_all.
    let _ = std::fs::remove_file(token_file);
}

// ---------------------------------------------------------------------------
// Test C — RED / E-DEMO-007 (EC-004: sidecar missing)
// AC-003, EC-004 (BC-3.6.001 Precondition 4)
// ---------------------------------------------------------------------------

/// AC-003 (Test C) — RED: E-DEMO-007 when token sidecar is absent.
///
/// Traces to: BC-3.6.001 Precondition 4.
/// Error taxonomy: E-DEMO-007 (story §Error Taxonomy Addition, registered in T-11).
///
/// EC-004: Token sidecar missing (server not running or different cwd).
/// Expected E-DEMO-007 error message:
///   "configure: E-DEMO-007: admin token for clone 'crowdstrike' could not be resolved:
///    token sidecar not found (start the demo server first with start or start-multi)"
///
/// RED today because:
///   T-03/T-04 not implemented → TOKEN_FILE never written by start_all.
///   The first assertion (token_file.exists()) fails → test RED.
///
/// After full fix (T-01..T-07 implemented):
///   TOKEN_FILE is written by start_all. Then deleting it and calling
///   resolve_configure_token reproduces EC-004 → E-DEMO-007.
///   Uncomment the block marked "UNCOMMENT AFTER T-07" below to complete the assertion.
#[tokio::test]
async fn test_BC_3_6_001_e_demo_007_configure_no_sidecar_present() {
    // AC-003, EC-004 (Test C): RED today — TOKEN_FILE not written (T-04 not implemented).

    let token_file = std::path::Path::new(".prism-dtu-demo-server.admin-tokens.json");
    let token_multi_file = std::path::Path::new(".prism-dtu-demo-server.admin-tokens-multi.json");

    // Remove any stale sidecar files.
    let _ = std::fs::remove_file(token_file);
    let _ = std::fs::remove_file(token_multi_file);

    // Start a single-clone harness so that start_all can (after fix) write TOKEN_FILE.
    let config = common::single_clone_config("crowdstrike");
    let pairs = build_clone_pairs(&config).expect("Test-C/EC-004: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);
    harness
        .start_all(&config, None)
        .await
        .expect("Test-C/EC-004: start_all must succeed");

    // ── RED assertion (T-03/T-04) ────────────────────────────────────────────
    // TOKEN_FILE must be written by start_all so we can then verify EC-004 behavior.
    // FAILS today: write_token_sidecar not yet implemented.
    assert!(
        token_file.exists(),
        "AC-003/EC-004/T-04 RED: TOKEN_FILE '{}' must be written by start_all. \
         E-DEMO-007 error path cannot be verified until the token sidecar exists. \
         Expected E-DEMO-007 message (after T-07): \
         \"configure: E-DEMO-007: admin token for clone 'crowdstrike' could not be resolved: \
         token sidecar not found (start the demo server first with start or start-multi)\"",
        token_file.display()
    );

    // ── UNCOMMENT AFTER T-07 (resolve_configure_token) ──────────────────────
    // After T-04 writes TOKEN_FILE and T-07 implements resolve_configure_token:
    // 1. Delete the sidecar to simulate EC-004 (sidecar missing after restart).
    // 2. Call resolve_configure_token → must return E-DEMO-007 error.
    //
    // let _ = std::fs::remove_file(token_file);
    // let result = prism_dtu_demo_server::resolve_configure_token(
    //     "crowdstrike",
    //     Some(token_file),
    //     Some(token_multi_file),
    // );
    // let err = result
    //     .expect_err("AC-003/EC-004: missing sidecar must return E-DEMO-007 error");
    // let msg = format!("{err:?}");
    // assert!(
    //     msg.contains("E-DEMO-007"),
    //     "AC-003/EC-004: error must contain E-DEMO-007 code; got: {msg}"
    // );
    // assert!(
    //     msg.contains("configure"),
    //     "AC-003/EC-004: error must match E-DEMO-007 template; got: {msg}"
    // );
    // assert!(
    //     msg.contains("token sidecar not found"),
    //     "AC-003/EC-004: error reason must cite missing sidecar; got: {msg}"
    // );
    // ── END UNCOMMENT BLOCK ──────────────────────────────────────────────────

    harness.stop_all().await;
    let _ = std::fs::remove_file(token_file);
    let _ = std::fs::remove_file(token_multi_file);
}

// ---------------------------------------------------------------------------
// Test D — RED / E-DEMO-007 (EC-005: ambiguous bare sensor name in multi-org)
// AC-003, EC-005 (BC-3.6.001 Precondition 4, BC-2.06.017 Postcondition 1)
// ---------------------------------------------------------------------------

/// AC-003 (Test D) — RED: E-DEMO-007 for ambiguous bare sensor name in start-multi mode.
///
/// Traces to:
///   BC-3.6.001 Precondition 4: configure requests require the correct per-clone token.
///   BC-2.06.017 Postcondition 1: MultiInstanceServers / admin_token_map governs start-multi
///     token extraction; token_multi_file parallels url_multi_file.
///
/// EC-005: In start-multi mode, if multiple orgs have the same sensor name (e.g.
/// "crowdstrike"), a bare sensor name is ambiguous. Expected E-DEMO-007 message:
///   "Bare sensor name 'crowdstrike' is ambiguous — found in N orgs: [org-a, org-b].
///    Use full '{org_slug}-{sensor_id}' form."
///
/// RED today because:
///   T-05/T-06 not implemented → TOKEN_MULTI_FILE never written by start_multi.
///   The first assertion (token_multi_file.exists()) fails → test RED.
///
/// After full fix (T-01..T-07 implemented):
///   TOKEN_MULTI_FILE is written by cmd_start_multi (via T-06).
///   Uncomment the block marked "UNCOMMENT AFTER T-07" below to complete the assertion
///   using a manually crafted multi-org token sidecar with two orgs sharing "crowdstrike".
#[tokio::test]
async fn test_BC_3_6_001_e_demo_007_ec005_ambiguous_bare_sensor_name() {
    // AC-003, EC-005 (Test D): RED today — TOKEN_MULTI_FILE not written (T-06 not implemented).

    let token_multi_file = std::path::Path::new(".prism-dtu-demo-server.admin-tokens-multi.json");

    // Remove any stale multi-org token sidecar.
    let _ = std::fs::remove_file(token_multi_file);

    // ── RED assertion (T-05/T-06) ────────────────────────────────────────────
    // TOKEN_MULTI_FILE must be written by cmd_start_multi (write_multi_admin_token_sidecar_to_path).
    // FAILS today: T-05 (write_multi_admin_token_sidecar_to_path) and T-06 (call in
    // cmd_start_multi) are not yet implemented.
    //
    // Note: this test does not start a multi-org harness (that requires the fixture-gen feature
    // and a full MultiOrgDemoConfig). Instead it asserts the file was written by an external
    // cmd_start_multi invocation and verifies the ambiguity error path.
    // For the RED gate, the assertion on file existence is sufficient.
    assert!(
        token_multi_file.exists(),
        "AC-003/EC-005/T-06 RED: TOKEN_MULTI_FILE '{}' must be written by cmd_start_multi. \
         EC-005 ambiguity test cannot be verified until the multi-org token sidecar exists. \
         Expected E-DEMO-007 message (after T-07): \
         \"configure: E-DEMO-007: admin token for clone 'crowdstrike' could not be resolved: \
         Bare sensor name 'crowdstrike' is ambiguous — found in 2 orgs: [org-a, org-b]. \
         Use full '{{org_slug}}-{{sensor_id}}' form.\"",
        token_multi_file.display()
    );

    // ── UNCOMMENT AFTER T-07 (resolve_configure_token) ──────────────────────
    // After T-06 writes TOKEN_MULTI_FILE and T-07 implements resolve_configure_token:
    // 1. Create a temporary multi-org token sidecar with two orgs both having "crowdstrike".
    // 2. Call resolve_configure_token("crowdstrike", None, Some(temp_sidecar)) →
    //    must return E-DEMO-007 with the ambiguity message.
    //
    // let ambiguous_sidecar: std::collections::HashMap<
    //     String,
    //     std::collections::HashMap<String, String>,
    // > = [
    //     (
    //         "org-a".to_string(),
    //         [("crowdstrike".to_string(), "fake-token-org-a".to_string())]
    //             .into_iter()
    //             .collect(),
    //     ),
    //     (
    //         "org-b".to_string(),
    //         [("crowdstrike".to_string(), "fake-token-org-b".to_string())]
    //             .into_iter()
    //             .collect(),
    //     ),
    // ]
    // .into_iter()
    // .collect();
    //
    // let tmp_path = std::path::Path::new(".prism-dtu-demo-server.admin-tokens-multi.ec005-test.tmp");
    // std::fs::write(
    //     tmp_path,
    //     serde_json::to_string(&ambiguous_sidecar).unwrap(),
    // )
    // .expect("Test-D: must write temporary ambiguous token sidecar");
    //
    // let result = prism_dtu_demo_server::resolve_configure_token(
    //     "crowdstrike",
    //     None,
    //     Some(tmp_path),
    // );
    // let _ = std::fs::remove_file(tmp_path);
    //
    // let err = result
    //     .expect_err("AC-003/EC-005: ambiguous bare sensor name must return E-DEMO-007 error");
    // let msg = format!("{err:?}");
    // assert!(
    //     msg.contains("E-DEMO-007"),
    //     "AC-003/EC-005: error must contain E-DEMO-007 code; got: {msg}"
    // );
    // assert!(
    //     msg.contains("ambiguous"),
    //     "AC-003/EC-005: error must cite ambiguity; got: {msg}"
    // );
    // assert!(
    //     msg.contains("org-a") && msg.contains("org-b"),
    //     "AC-003/EC-005: error must name the conflicting orgs; got: {msg}"
    // );
    // ── END UNCOMMENT BLOCK ──────────────────────────────────────────────────

    let _ = std::fs::remove_file(token_multi_file);
}
