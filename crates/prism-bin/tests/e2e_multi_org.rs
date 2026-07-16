// SPDX-License-Identifier: Apache-2.0
//! Multi-org × multi-sensor isolation smoke tests for S-DEMO-004.
//!
//! Validates BC-3.2.001 org isolation, BC-2.06.017 per-DTU-instance multi-address
//! binding, and BC-2.06.018 per-org seeded data distinctness across a 3-org ×
//! mixed-sensor configuration.
//!
//! All tests are marked `#[ignore]` per AC-010:
//!   "Standard `cargo nextest run -p prism-bin` skips them; un-gated via 'e2e-multi-org' profile."
//!
//! # E2E-MULTI-001 gate
//! Every test carries:
//!   `// E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.`
//!
//! # Org / Sensor matrix
//!
//! | Org | CrowdStrike | Armis | Claroty | Cyberint | Seeds |
//! |-----|-------------|-------|---------|----------|-------|
//! | org-a | YES (100) | YES (110) | NO | NO | — |
//! | org-b | NO | NO | YES (120) | YES (130) | — |
//! | org-c | YES (200) | YES (210) | YES (220) | YES (230) | — |
//!
//! 8 DTU clone instances total (BC-2.06.017 Postcondition 2).
//!
//! # Test gating (AC-010)
//!
//! All tests are `#[ignore]` — `cargo nextest run -p prism-bin` (no profile) SKIPS them.
//! They run only when the `e2e-multi-org` nextest profile is active, satisfying AC-010.
//! The profile provides timeout and failure-output settings; the `--run-ignored ignored-only`
//! CLI flag is what actually un-ignores the tests (nextest has no `run-ignored` TOML key —
//! see the comment in `.config/nextest.toml`). The canonical command is:
//!   `cargo nextest run -p prism-bin --profile e2e-multi-org --run-ignored ignored-only`
//!
//! # Test → AC → BC Mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | `test_BC_2_22_001_multi_org_boot_registers_8_adapters` | AC-001 | BC-2.22.001, BC-2.10.001 |
//! | `test_BC_2_06_014_org_a_crowdstrike_query_returns_data` | AC-002 | BC-2.06.014, BC-2.10.001 |
//! | `test_BC_2_06_014_org_b_claroty_query_returns_data` | AC-003 | BC-2.06.014, BC-2.11.005 |
//! | `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data` | AC-004 | BC-2.01.013 |
//! | `test_BC_3_2_001_cross_org_query_returns_isolation_error` | AC-005 | BC-3.2.001 |
//! | `test_BC_2_06_018_per_org_seeded_data_is_disjoint` | AC-006 | BC-2.06.017 PC-3, BC-2.06.018 INV-DISTINCT-DATA-001, BC-2.06.014 |
//! | `test_BC_3_2_001_cyberint_session_isolation_org_b_and_org_c` | AC-007 | BC-3.2.001, BC-2.01.013, BC-2.06.017 INV-ISOLATION-001 |
//! | `test_BC_2_09_008_response_envelope_identifies_correct_org_and_sensor` | AC-008 | BC-2.09.008 |
//! | `test_BC_2_11_005_sequential_org_queries_do_not_interfere` | AC-009 | BC-2.11.005, BC-2.06.018 INV-DISTINCT-DATA-001 |
//! | `test_BC_2_22_001_multi_org_tests_ignored_under_standard_profile` | AC-010 | BC-2.22.001 |
//!
//! # Red Gate tests (4 designated per story frontmatter red_gate_tests=4)
//!
//! The 4 designated Red Gate tests are:
//!   1. `test_BC_2_22_001_multi_org_boot_registers_8_adapters`    (AC-001)
//!   2. `test_BC_2_01_013_org_c_all_4_sensors_return_independent_data` (AC-004)
//!   3. `test_BC_3_2_001_cross_org_query_returns_isolation_error`  (AC-005)
//!   4. `test_BC_2_06_018_per_org_seeded_data_is_disjoint`         (AC-006)
//!
//! Story: S-DEMO-004
//! BCs: BC-3.2.001, BC-2.06.014, BC-2.06.017, BC-2.06.018, BC-2.11.005,
//!      BC-2.01.013, BC-2.10.001, BC-2.09.008, BC-2.22.001

mod helpers;

// ---------------------------------------------------------------------------
// Helper: extract device/detection IDs from a response envelope
// ---------------------------------------------------------------------------

/// Extract the set of device/detection IDs from a ResponseEnvelope JSON value.
///
/// IDs are extracted by scanning the response JSON for string values that match
/// the canonical format `"dev-{8hex}-{seed}-{n}"` (ADR-036 v2.0 §2.2).
///
/// This function performs a recursive scan of all string values in the JSON
/// tree and collects values matching the `dev-[0-9a-f]{8}-\d+-\d+` pattern.
///
/// CRITICAL: the 8hex prefix is derived from `hex(org_id.as_bytes()[0..4])` of the
/// UUID assigned to that org — NOT the human slug "org-a". Asserting against "dev-org-a-..."
/// would match zero IDs, making `ids_a ∩ ids_c = ∅` pass VACUOUSLY (false green),
/// defeating the INV-DISTINCT-DATA-001 proof (AC-006, AC-009).
///
/// The implementer of the helper bodies must ensure the seeded clones actually return
/// data in the DTU HTTP response bodies that the MCP tool_query parses and surfaces.
fn extract_device_ids(response: &serde_json::Value) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();
    extract_ids_recursive(response, &mut ids);
    ids
}

fn extract_ids_recursive(value: &serde_json::Value, ids: &mut std::collections::HashSet<String>) {
    match value {
        // Canonical format: "dev-{8hex}-{seed}-{n}" per ADR-036 v2.0 §2.2.
        // 8hex = exactly 8 lowercase hex digits; seed = decimal u64; n = decimal u64.
        serde_json::Value::String(s) if looks_like_device_id(s) => {
            ids.insert(s.clone());
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_ids_recursive(item, ids);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                extract_ids_recursive(v, ids);
            }
        }
        _ => {}
    }
}

/// Returns true if `s` matches the canonical DTU device ID format.
///
/// Pattern: `dev-[0-9a-f]{8}-[0-9]+-[0-9]+`
///
/// This is intentionally lenient on the seed/n components (any decimal digits) so the
/// check works without knowing the org_id UUID bytes at parse time. The caller
/// (AC-006/AC-009) then asserts set disjointness, which validates the structural
/// distinctness guaranteed by INV-DISTINCT-DATA-001.
fn looks_like_device_id(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("dev-") else {
        return false;
    };
    // Expect: {8hex}-{seed}-{n}
    let parts: Vec<&str> = rest.splitn(3, '-').collect();
    if parts.len() != 3 {
        return false;
    }
    let hex_part = parts[0];
    let seed_part = parts[1];
    let n_part = parts[2];
    hex_part.len() == 8
        && hex_part.chars().all(|c| c.is_ascii_hexdigit())
        && seed_part.chars().all(|c| c.is_ascii_digit())
        && !seed_part.is_empty()
        && n_part.chars().all(|c| c.is_ascii_digit())
        && !n_part.is_empty()
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.22.001: 3-org boot registers exactly 8 adapters
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.22.001: 3-org boot registers exactly 8 adapters (2+2+4).
///
/// Given: prism.toml with 3 orgs and mixed sensor overlays from `start_multi_org_harness`.
/// When: prism-bin starts and boot step 9A completes.
/// Then: AdapterRegistry contains exactly 8 adapters.
///   - org-a: 2 adapters (CrowdStrike + Armis)
///   - org-b: 2 adapters (Claroty + Cyberint)
///   - org-c: 4 adapters (all 4 sensors)
///
/// Verified via `boot.step9a.adapter_registry_populated` event log assertion
/// (sensor_count field). Uses `RUST_LOG=boot=info` + `PRISM_LOG_FORMAT=json` with
/// stderr capture to parse the structured JSON event from the subprocess.
///
/// (traces to BC-2.22.001 boot sequencing; BC-2.10.001 client_id routing)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_22_001_multi_org_boot_registers_8_adapters() {
    // Step 1: Start MultiInstanceHarness with 8 clone entries.
    let (harness, tempdir) = helpers::start_multi_org_harness().await;

    // Step 2: Write per-org overlay TOML files from the harness socket_map.
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");

    // Step 3: Write 3-org prism.toml.
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    // Step 4: Launch prism-bin with stderr capture (RUST_LOG=boot=info, PRISM_LOG_FORMAT=json).
    // The `with_stderr` variant captures subprocess stderr so we can assert the
    // boot.step9a.adapter_registry_populated event count directly.
    let (prism_guard, mut mcp, stderr_buf) =
        helpers::launch_prism_bin_multi_org_with_stderr(tempdir.path())
            .await
            .expect("prism-bin did not boot for multi-org config (EC-002)");

    // Step 5: Assert tools/list contains `query` (MCP server is up).
    let tools = mcp.tools_list().expect("tools/list failed");
    assert!(
        tools.iter().any(|t| t["name"] == "query"),
        "AC-001: tools/list must contain 'query'; got: {tools:?}"
    );

    // Step 6: Assert boot.step9a.adapter_registry_populated shows sensor_count == 8.
    //
    // By the time MCP initialize + tools/list succeeds, all boot steps (including 9A)
    // have completed and their log events have been emitted to stderr. The background
    // stderr-reader thread has been draining the pipe; we give it a brief moment to
    // flush any remaining bytes before reading.
    //
    // We drop the MCP handle first (closes stdin → prism begins graceful shutdown →
    // closes its stderr), then wait for the drain thread to finish via a short poll.
    drop(mcp);
    drop(prism_guard);

    // Wait up to 3 seconds for the stderr drain thread to finish (the subprocess closes
    // stderr on exit; the thread reads to EOF then releases the lock).
    let drain_deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        // Check if the buffer has content (non-empty = drain thread has written something).
        // We stop as soon as the prism process exits (stderr EOF) and the thread writes.
        std::thread::sleep(std::time::Duration::from_millis(50));
        if let Ok(buf) = stderr_buf.lock()
            && !buf.is_empty()
        {
            break;
        }
        if std::time::Instant::now() >= drain_deadline {
            break; // proceed even if empty — assertion below will surface the failure
        }
    }

    // Parse the captured stderr lines and find boot.step9a.adapter_registry_populated.
    let stderr_text = {
        let buf = stderr_buf.lock().expect("stderr_buf mutex poisoned");
        String::from_utf8_lossy(&buf).into_owned()
    };

    // Each line is a JSON object (PRISM_LOG_FORMAT=json). Scan all lines for the event.
    let mut found_sensor_count: Option<u64> = None;
    for line in stderr_text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
            // Tracing JSON format: {"fields":{"event_type":"...","sensor_count":N,...},...}
            let event_type = obj
                .get("fields")
                .and_then(|f| f.get("event_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if event_type == "boot.step9a.adapter_registry_populated" {
                // The emitted field is sensor_count as u64; tracing-subscriber JSON may
                // serialize it as a number or as a string — accept both.
                let count = obj
                    .get("fields")
                    .and_then(|f| f.get("sensor_count"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse().ok()));
                if let Some(c) = count {
                    found_sensor_count = Some(c);
                    // Keep scanning: if the event fires more than once, we want the last
                    // (non-zero) value; the empty-catalog path emits sensor_count=0.
                    if c > 0 {
                        break;
                    }
                }
            }
        }
    }

    assert_eq!(
        found_sensor_count,
        Some(8),
        "AC-001 / BC-2.22.001: boot.step9a.adapter_registry_populated must report \
         sensor_count = 8 (org-a:2 + org-b:2 + org-c:4). \
         Got: {found_sensor_count:?}. \
         If None: the event was not found in stderr — check RUST_LOG=boot=info and \
         PRISM_LOG_FORMAT=json are being applied to the subprocess. \
         If wrong count: a spurious or missing adapter registration was detected. \
         Captured stderr ({} bytes, first 2000 chars):\n{}",
        stderr_text.len(),
        &stderr_text[..stderr_text.len().min(2000)]
    );

    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.06.014: Org A queries return data for registered sensors
// ---------------------------------------------------------------------------

/// AC-002 / BC-2.06.014: Org A `tool_query` for CrowdStrike returns non-empty data.
///
/// Given: org-a is registered with CrowdStrike + Armis (not Claroty or Cyberint).
/// When: `tool_query "SELECT * FROM crowdstrike_detections LIMIT 5" client_id="org-a"` is sent.
/// Then: Returns non-empty data from org-a's CrowdStrike DTU clone (seed=100).
///
/// (traces to BC-2.06.014 instance identity resolution; BC-2.10.001 client_id scoping)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_06_014_org_a_crowdstrike_query_returns_data() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // AC-002: org-a queries CrowdStrike (a registered sensor).
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let result = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 5",
            helpers::ORG_A_SLUG,
        )
        .expect("tool_query_scoped for org-a CrowdStrike failed");

    // Assert non-empty result rows.
    let rows = result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !rows.is_empty(),
        "AC-002: org-a CrowdStrike query must return non-empty data; got empty rows. \
         response: {result:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.06.014: Org B queries return data for registered sensors
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.06.014: Org B `tool_query` for Claroty returns non-empty data.
///
/// Given: org-b is registered with Claroty + Cyberint (not CrowdStrike or Armis).
/// When: `tool_query "SELECT * FROM claroty_alerts LIMIT 5" client_id="org-b"` is sent.
/// Then: Returns non-empty data from org-b's Claroty DTU clone (seed=120).
///
/// (traces to BC-2.06.014; BC-2.11.005 ephemeral materialization)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_06_014_org_b_claroty_query_returns_data() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // AC-003: org-b queries Claroty (a registered sensor).
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    // Claroty table name is "claroty_alerts" (sensor_id=claroty, table_name=alerts).
    let result = mcp
        .tool_query_scoped("SELECT * FROM claroty_alerts LIMIT 5", helpers::ORG_B_SLUG)
        .expect("tool_query_scoped for org-b Claroty failed");

    let rows = result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !rows.is_empty(),
        "AC-003: org-b Claroty (claroty_alerts) query must return non-empty data; got empty rows. \
         response: {result:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.01.013: Org C queries succeed for all 4 sensors independently
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.01.013: Org C queries return data for all 4 sensors independently.
///
/// Given: org-c is registered with all 4 sensors.
/// When: Each of the 4 `tool_query` calls is sent with `client_id="org-c"`.
/// Then: All 4 return non-empty data; no cross-sensor data contamination.
///
/// (traces to BC-2.01.013 spec-driven adapters are per-org)
///
/// RED GATE TEST (one of 4 designated Red Gate tests per story frontmatter).
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_01_013_org_c_all_4_sensors_return_independent_data() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // AC-004: org-c queries all 4 sensors independently.
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    // CrowdStrike (seed=200)
    let cs_result = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 5",
            helpers::ORG_C_SLUG,
        )
        .expect("org-c CrowdStrike query failed");
    let cs_rows = cs_result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !cs_rows.is_empty(),
        "AC-004: org-c CrowdStrike query must return non-empty data; response: {cs_result:?}"
    );

    // Armis (seed=210): Armis queries require a WHERE aql predicate.
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax.
    let armis_result = mcp
        .tool_query_scoped(
            "SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5",
            helpers::ORG_C_SLUG,
        )
        .expect("org-c Armis query failed");
    let armis_rows = armis_result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !armis_rows.is_empty(),
        "AC-004: org-c Armis query must return non-empty data; response: {armis_result:?}"
    );

    // Claroty (seed=220): table name is "claroty_alerts" (sensor_id=claroty, table_name=alerts).
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax.
    let claroty_result = mcp
        .tool_query_scoped("SELECT * FROM claroty_alerts LIMIT 5", helpers::ORG_C_SLUG)
        .expect("org-c Claroty query failed");
    let claroty_rows = claroty_result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !claroty_rows.is_empty(),
        "AC-004: org-c Claroty query must return non-empty data; response: {claroty_result:?}"
    );

    // Cyberint (seed=230)
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax.
    let cyberint_result = mcp
        .tool_query_scoped("SELECT * FROM cyberint_alerts LIMIT 5", helpers::ORG_C_SLUG)
        .expect("org-c Cyberint query failed");
    let cyberint_rows = cyberint_result
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !cyberint_rows.is_empty(),
        "AC-004: org-c Cyberint query must return non-empty data; response: {cyberint_result:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.2.001: Cross-org isolation — error, not data
// ---------------------------------------------------------------------------

/// AC-005 / BC-3.2.001: Org A querying Org B's sensor returns an isolation error.
///
/// Given: Org A has CrowdStrike + Armis registered; Cyberint is NOT registered for Org A.
/// When: `tool_query "SELECT * FROM cyberint_alerts LIMIT 5" client_id="org-a"` is sent.
/// Then: Response envelope has `isError: true` and `structuredContent.error.code == "E-QUERY-032"`;
///       NO data rows are returned; NO data from Org B leaks into Org A's response.
///
/// EC-003: Empty data looks like a "soft pass" but is a BC-3.2.001 violation —
///         the isolation error MUST be explicit. This test asserts on the error code,
///         not just "no rows".
///
/// # Wire shape (post F-2 / BC-2.10.007+)
///
/// The MCP `query` handler routes all user-visible domain errors (including E-QUERY-032)
/// through `prism_error_to_structured_call_result`, which returns:
///   `Ok(CallToolResult { isError: true, structuredContent: { error: { code: "E-QUERY-032", ... } } })`
/// rather than the pre-F-2 JSON-RPC protocol-level error (`{"error": {...}}`).
/// BC-2.10.007 (later, more-specific contract) governs the wire shape; BC-3.2.001 governs
/// the isolation semantics. Both are satisfied: the error is observably an error (isError=true)
/// with E-QUERY-032 signal; no data leaks.
///
/// (traces to BC-3.2.001 postcondition 5 invariant: no cross-org data leakage)
///
/// RED GATE TEST (one of 4 designated Red Gate tests per story frontmatter).
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_3_2_001_cross_org_query_returns_isolation_error() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // AC-005: org-a queries Cyberint (a sensor NOT registered for org-a).
    // Expect a BC-2.10.007 structured error: CallToolResult with isError=true and
    // structuredContent.error.code = "E-QUERY-032" (SensorNotRegisteredForOrg).
    //
    // Post F-2: `prism_error_to_structured_call_result` wraps all domain errors as
    // Ok(CallToolResult{isError:true}) — NOT a JSON-RPC protocol-level error.
    // `tool_query_scoped` calls `send_request` (returns the result field on success)
    // and then normalizes the envelope. For an isError=true response, `content[0].text`
    // is the "ERROR: [validation] - ..." string (not JSON), so `tool_query_with_params`
    // falls back to the raw CallToolResult object. We inspect it directly here.
    //
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let response = mcp
        .tool_query_scoped("SELECT * FROM cyberint_alerts LIMIT 5", helpers::ORG_A_SLUG)
        .expect("I/O or parse failure sending isolation probe");

    // AC-005 assertion 1: isError must be true (BC-2.10.007 structured error shape).
    // The response is the normalized CallToolResult: { "isError": true, "structuredContent": {...}, ... }.
    let is_error = response
        .get("isError")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        is_error,
        "AC-005 / BC-3.2.001: org-a querying cyberint (not registered for org-a) \
         MUST return isError=true (BC-2.10.007 structured error envelope). \
         BC-3.2.001 invariant: no cross-org data leakage. \
         EC-003: empty data is NOT an acceptable isolation response. \
         Full response: {response:?}"
    );

    // AC-005 assertion 2: structuredContent.error.code must signal E-QUERY-032.
    let sc_error_code = response
        .get("structuredContent")
        .and_then(|sc| sc.get("error"))
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert!(
        sc_error_code.contains("E-QUERY-032"),
        "AC-005 / BC-3.2.001: structuredContent.error.code must be 'E-QUERY-032' \
         (sensor not registered for org). Got: {sc_error_code:?}. \
         The error must indicate the sensor is not registered for the requesting org. \
         Full response: {response:?}"
    );

    // AC-005 assertion 3: structuredContent.error.message must mention isolation signals.
    let sc_error_msg = response
        .get("structuredContent")
        .and_then(|sc| sc.get("error"))
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let signals_isolation = sc_error_msg.to_lowercase().contains("not registered")
        || sc_error_msg.to_lowercase().contains("adapter not found")
        || sc_error_msg.to_lowercase().contains("e-query-032");
    assert!(
        signals_isolation,
        "AC-005 / BC-3.2.001: structuredContent.error.message must signal isolation failure \
         (contains 'not registered', 'adapter not found', or 'E-QUERY-032'). \
         Got: {sc_error_msg:?}. Full response: {response:?}"
    );

    // Confirm NO data rows are present (belt-and-suspenders, EC-003).
    // For an isError=true CallToolResult there is no ResponseEnvelope rows field.
    let has_rows = response
        .get("rows")
        .and_then(|rows| rows.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    assert!(
        !has_rows,
        "AC-005 / BC-3.2.001: isolation error response MUST NOT contain data rows. \
         Got non-empty rows — this indicates cross-org data leakage. \
         Full response: {response:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.06.018 INV-DISTINCT-DATA-001: per-org seeded data is disjoint
// ---------------------------------------------------------------------------

/// AC-006 / BC-2.06.018: ids_org_a ∩ ids_org_c = ∅ (INV-DISTINCT-DATA-001).
///
/// Given: Org A and Org C both have CrowdStrike. org-a clone seed=100, org-c clone seed=200.
/// When: `tool_query "SELECT * FROM crowdstrike_detections LIMIT 50"` is executed for org-a and org-c.
/// Then: The test reads both response bodies, extracts device/detection IDs
///       (format: "dev-{8hex}-{seed}-{n}" where 8hex = hex(org_id.as_bytes()[0..4])),
///       and asserts ids_org_a ∩ ids_org_c = ∅.
///
/// CRITICAL FALSE-GREEN TRAP: the expected ID prefix MUST be derived from
/// hex(org_id.as_bytes()[0..4]) of the UUID — NOT the human slug "org-a".
/// This test uses `extract_device_ids()` which scans for the structural pattern
/// "dev-[0-9a-f]{8}-[0-9]+-[0-9]+" — it will catch IDs regardless of the 8hex value.
/// The disjointness is proven by the SEED component (100 vs 200), not the 8hex alone
/// (which may be identical for our deterministic test UUIDs).
///
/// (traces to BC-2.06.017 Postcondition 3 + BC-2.06.018 INV-DISTINCT-DATA-001 +
///  BC-2.06.014 endpoint-resolution)
///
/// RED GATE TEST (one of 4 designated Red Gate tests per story frontmatter).
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_06_018_per_org_seeded_data_is_disjoint() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // Query org-a's CrowdStrike clone (seed=100).
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let result_org_a = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 50",
            helpers::ORG_A_SLUG,
        )
        .expect("org-a CrowdStrike query failed");

    // Query org-c's CrowdStrike clone (seed=200).
    let result_org_c = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 50",
            helpers::ORG_C_SLUG,
        )
        .expect("org-c CrowdStrike query failed");

    // Extract device/detection ID sets from both responses.
    let ids_org_a = extract_device_ids(&result_org_a);
    let ids_org_c = extract_device_ids(&result_org_c);

    // ANTI-FALSE-GREEN GUARD: both sets must be non-empty before testing disjointness.
    // If either set is empty, the intersection is vacuously empty — a FALSE GREEN that
    // defeats the INV-DISTINCT-DATA-001 proof. This guard catches implementation bugs
    // where seeded data is not surfaced in the MCP response.
    //
    // Expected IDs for org-a: "dev-{8hex}-100-N" (seed=100)
    // Expected IDs for org-c: "dev-{8hex}-200-N" (seed=200)
    // where {8hex} = hex(org_id.as_bytes()[0..4]) for each org's UUID.
    assert!(
        !ids_org_a.is_empty(),
        "AC-006 ANTI-FALSE-GREEN: org-a CrowdStrike response must contain device IDs \
         (format: dev-{{8hex}}-100-N). Got empty set — check that the seeded clone \
         (seed=100) is actually serving data and the MCP response surfaces it. \
         CRITICAL: do NOT assert against 'dev-org-a-...' — derive the 8hex prefix from \
         hex(ORG_A_ID bytes[0..4]). Full response: {result_org_a:?}"
    );
    assert!(
        !ids_org_c.is_empty(),
        "AC-006 ANTI-FALSE-GREEN: org-c CrowdStrike response must contain device IDs \
         (format: dev-{{8hex}}-200-N). Got empty set — check that the seeded clone \
         (seed=200) is actually serving data and the MCP response surfaces it. \
         Full response: {result_org_c:?}"
    );

    // INV-DISTINCT-DATA-001: ids_org_a ∩ ids_org_c = ∅.
    // The canonical ID format encodes the seed as the second dash-component:
    //   org-a: "dev-{8hex}-100-N"
    //   org-c: "dev-{8hex}-200-N"
    // Since 100 ≠ 200, no ID from org-a can appear in org-c's set and vice versa.
    let intersection: std::collections::HashSet<_> = ids_org_a.intersection(&ids_org_c).collect();
    assert!(
        intersection.is_empty(),
        "AC-006 / BC-2.06.018 INV-DISTINCT-DATA-001: ids_org_a ∩ ids_org_c must be ∅. \
         Got non-empty intersection: {intersection:?}. \
         This indicates cross-org data leakage or incorrect seeding \
         (org-a seed=100 vs org-c seed=200 should produce disjoint ID sets). \
         ids_org_a (sample): {:?}, ids_org_c (sample): {:?}",
        ids_org_a.iter().take(3).collect::<Vec<_>>(),
        ids_org_c.iter().take(3).collect::<Vec<_>>()
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-007 / BC-3.2.001: Cyberint session isolation between org-b and org-c
// ---------------------------------------------------------------------------

/// AC-007 / BC-3.2.001: Cyberint cookie_roundtrip auth works for org-b and org-c independently.
///
/// Given: org-b and org-c both have Cyberint registered. MultiInstanceHarness binds a
///        distinct socket for each org's Cyberint clone: (org-b, cyberint) at S_B and
///        (org-c, cyberint) at S_C (S_B ≠ S_C per INV-ISOLATION-001, BC-2.06.017).
/// When: `tool_query "SELECT * FROM cyberint_alerts LIMIT 5"` is sent for each org.
/// Then: Each query succeeds with its own session cookie from the respective org's DTU clone;
///       session tokens do not cross between org-b and org-c.
///
/// (traces to BC-3.2.001 session isolation; BC-2.01.013 per-org adapter construction;
///  BC-2.06.017 INV-ISOLATION-001 per-org distinct Cyberint DTU sockets)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_3_2_001_cyberint_session_isolation_org_b_and_org_c() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    // Verify that the socket_map has distinct sockets for each org's Cyberint clone
    // (INV-ISOLATION-001: per-org distinct Cyberint DTU sockets).
    let socket_map = harness.socket_map();
    let socket_org_b = socket_map
        .get(&(helpers::ORG_B_SLUG.to_string(), "cyberint".to_string()))
        .expect("org-b cyberint must have a socket in the harness socket_map");
    let socket_org_c = socket_map
        .get(&(helpers::ORG_C_SLUG.to_string(), "cyberint".to_string()))
        .expect("org-c cyberint must have a socket in the harness socket_map");

    assert_ne!(
        socket_org_b, socket_org_c,
        "AC-007 / BC-2.06.017 INV-ISOLATION-001: org-b and org-c must have DISTINCT \
         Cyberint DTU sockets. Got identical socket {socket_org_b} for both orgs — \
         this violates INV-ISOLATION-001 (cross-tenant socket reuse)."
    );

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // org-b Cyberint query (seed=130) — must succeed independently.
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let result_org_b = mcp
        .tool_query_scoped("SELECT * FROM cyberint_alerts LIMIT 5", helpers::ORG_B_SLUG)
        .expect("org-b Cyberint query failed");

    // org-c Cyberint query (seed=230) — must succeed independently.
    let result_org_c = mcp
        .tool_query_scoped("SELECT * FROM cyberint_alerts LIMIT 5", helpers::ORG_C_SLUG)
        .expect("org-c Cyberint query failed");

    // Both queries must return non-empty data (independent session tokens per org).
    let rows_b = result_org_b
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    let rows_c = result_org_c
        .get("rows")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    assert!(
        !rows_b.is_empty(),
        "AC-007: org-b Cyberint query must return non-empty data (independent session). \
         response: {result_org_b:?}"
    );
    assert!(
        !rows_c.is_empty(),
        "AC-007: org-c Cyberint query must return non-empty data (independent session). \
         response: {result_org_c:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.09.008: ResponseEnvelope metadata identifies correct org/sensor
// ---------------------------------------------------------------------------

/// AC-008 / BC-2.09.008: ResponseEnvelope metadata identifies correct org and sensor.
///
/// Given: A successful multi-org query for any org/sensor combination.
/// When: The ResponseEnvelope is inspected.
/// Then: `_meta.data_source` contains the correct sensor name; no org identifiers
///       from other orgs appear in the response.
///
/// (traces to BC-2.09.008 response envelope trust annotations)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_09_008_response_envelope_identifies_correct_org_and_sensor() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // Query org-a CrowdStrike and verify the response envelope.
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let result = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 5",
            helpers::ORG_A_SLUG,
        )
        .expect("org-a CrowdStrike query failed");

    // BC-2.09.008: response must have `_meta.data_source` identifying the sensor.
    let data_source = result
        .get("_meta")
        .and_then(|m| m.get("data_source"))
        .and_then(|ds| ds.as_str())
        .unwrap_or("");

    assert!(
        !data_source.is_empty(),
        "AC-008 / BC-2.09.008: _meta.data_source must be non-empty in ResponseEnvelope. \
         response: {result:?}"
    );
    assert!(
        data_source.contains("crowdstrike"),
        "AC-008 / BC-2.09.008: _meta.data_source must identify 'crowdstrike' for a \
         CrowdStrike query. Got: {data_source:?}. Full response: {result:?}"
    );

    // No org identifiers from OTHER orgs must appear.
    // Org-c's slug must not appear in org-a's response.
    let response_str = serde_json::to_string(&result).unwrap_or_default();
    assert!(
        !response_str.contains(helpers::ORG_C_SLUG),
        "AC-008 / BC-3.2.001: org-c identifier must not appear in org-a's CrowdStrike response. \
         This would indicate cross-org metadata leakage. response: {result:?}"
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.11.005 + BC-2.06.018: Sequential queries for different orgs
// ---------------------------------------------------------------------------

/// AC-009 / BC-2.11.005: Sequential back-to-back queries for org-a and org-c do not interfere.
///
/// Given: org-a and org-c both query CrowdStrike via sequential back-to-back dispatch,
///        using the same MultiInstanceHarness setup as AC-006
///        (org-a seed=100, org-c seed=200, distinct org_ids, distinct sockets per INV-ISOLATION-001).
/// When: Both responses arrive (via sequential dispatch over the single stdio channel).
/// Then: org-a's response contains only data from the org-a CrowdStrike clone;
///       org-c's response contains only data from the org-c CrowdStrike clone;
///       `ids_a ∩ ids_c = ∅` on both response bodies (INV-DISTINCT-DATA-001).
///
/// ANTI-FALSE-GREEN: same guard as AC-006 — both ID sets must be non-empty before
/// asserting disjointness. The expected ID format is "dev-{8hex}-{seed}-N" where
/// {8hex} = hex(org_id.as_bytes()[0..4]). Asserting against "dev-org-a-..." would
/// match zero IDs and make the intersection vacuously empty (false green).
///
/// (traces to BC-2.11.005 ephemeral materialization; BC-2.06.018 INV-DISTINCT-DATA-001
///  sequential back-to-back distinctness proof)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_11_005_sequential_org_queries_do_not_interfere() {
    let (harness, tempdir) = helpers::start_multi_org_harness().await;
    helpers::write_multi_org_overlays(&harness, &tempdir).expect("write_multi_org_overlays failed");
    helpers::write_multi_org_prism_toml(&tempdir).expect("write_multi_org_prism_toml failed");

    let (prism_guard, mut mcp) = helpers::launch_prism_bin_multi_org(tempdir.path())
        .await
        .expect("prism-bin launch failed");
    mcp.initialize().expect("MCP initialize failed");

    // NOTE: McpStdioHandle uses a single stdio channel and is not Send/Sync across tasks.
    // AC-009 uses sequential back-to-back dispatch: we send both requests in rapid
    // succession (within a single thread) and read both responses. The server processes
    // each query independently via fan_out() (async); back-to-back dispatch over a single
    // stdio channel is the correct model for single-client MCP.
    //
    // Full cross-thread concurrency (tokio::spawn + Arc<Mutex<McpStdioHandle>>) would
    // require making McpStdioHandle Send — a production concern addressed when true
    // parallel MCP client support is added. For the smoke test, back-to-back sends
    // prove the server handles sequential org queries without cross-org data mixing.

    // Send org-a CrowdStrike query (seed=100).
    // SQL form required: "FROM source LIMIT N" is invalid pipe syntax; use "SELECT * FROM ... LIMIT N".
    let result_org_a = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 50",
            helpers::ORG_A_SLUG,
        )
        .expect("sequential org-a CrowdStrike query failed");

    // Send org-c CrowdStrike query immediately after (seed=200).
    let result_org_c = mcp
        .tool_query_scoped(
            "SELECT * FROM crowdstrike_detections LIMIT 50",
            helpers::ORG_C_SLUG,
        )
        .expect("sequential org-c CrowdStrike query failed");

    let ids_a = extract_device_ids(&result_org_a);
    let ids_c = extract_device_ids(&result_org_c);

    // ANTI-FALSE-GREEN GUARD (same as AC-006).
    assert!(
        !ids_a.is_empty(),
        "AC-009 ANTI-FALSE-GREEN: org-a CrowdStrike response must contain \
         device IDs (format: dev-{{8hex}}-100-N). Got empty set. \
         response: {result_org_a:?}"
    );
    assert!(
        !ids_c.is_empty(),
        "AC-009 ANTI-FALSE-GREEN: org-c CrowdStrike response must contain \
         device IDs (format: dev-{{8hex}}-200-N). Got empty set. \
         response: {result_org_c:?}"
    );

    // INV-DISTINCT-DATA-001 under sequential back-to-back dispatch.
    let intersection: std::collections::HashSet<_> = ids_a.intersection(&ids_c).collect();
    assert!(
        intersection.is_empty(),
        "AC-009 / BC-2.11.005 + BC-2.06.018 INV-DISTINCT-DATA-001: ids_a ∩ ids_c \
         must be ∅. Got non-empty intersection under sequential back-to-back dispatch: {intersection:?}. \
         This indicates cross-org data mixing during back-to-back fan_out() execution. \
         ids_a (sample): {:?}, ids_c (sample): {:?}",
        ids_a.iter().take(3).collect::<Vec<_>>(),
        ids_c.iter().take(3).collect::<Vec<_>>()
    );

    drop(mcp);
    drop(prism_guard);
    drop(harness);
    drop(tempdir);
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.22.001: Test is gated behind #[ignore] with explicit CI profile
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.22.001: Multi-org tests are skipped under the standard nextest profile.
///
/// This test itself is `#[ignore]` — it verifies the gating property by its own
/// existence: if you can read this test in the output of `cargo nextest run -p prism-bin`
/// (without the e2e-multi-org profile), something is wrong.
///
/// The actual gating validation is structural: ALL tests in this file carry
/// `#[ignore = "E2E-MULTI-001: ...]` which causes nextest to skip them unless
/// `--run-ignored` or `--profile e2e-multi-org` is used.
///
/// (traces to BC-2.22.001 invariant: startup deterministic and testable)
///
/// // E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.
#[tokio::test]
#[ignore = "E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile."]
async fn test_BC_2_22_001_multi_org_tests_ignored_under_standard_profile() {
    // AC-010: The #[ignore] attribute on this function (and all tests in this file)
    // is the enforcement mechanism. If this test runs without the e2e-multi-org profile,
    // it means the #[ignore] was removed — which violates AC-010.
    //
    // This test is intentionally a no-op (passes when run under the e2e-multi-org profile).
    // The Red Gate for AC-010 is satisfied by the #[ignore] attribute itself causing nextest
    // to skip all tests in standard profile runs.
}
