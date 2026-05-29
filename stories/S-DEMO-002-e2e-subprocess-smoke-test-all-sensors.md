---
document_type: story
story_id: S-DEMO-002
title: "prism-bin: E2E Subprocess Smoke Test — All 4 Sensors via DTU Clones + MCP Round-Trip"
wave: 5
epic_id: E-DEMO
priority: P0
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-10, SS-11, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) is exercised end-to-end — SpecDrivenSensorAdapter dispatches
#     to each of the 4 DTU clone endpoints; the test verifies real data flows through.
#   SS-10 (MCP Interface) — test drives MCP stdio transport (initialize handshake, tools/call),
#     exercising the rmcp ServerHandler delivered by S-5.01-FOLLOWUP-MCP-BOOT.
#   SS-11 (Query Execution) — tool_query invocation exercises QueryEngine fan_out()
#     materialization pipeline, OCSF normalization, and ResponseEnvelope wrapping.
#   SS-22 (Binary Entrypoint) — test launches prism-bin start as a subprocess and validates
#     clean SIGTERM shutdown per BC-2.10.010.
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: [CAP-001, CAP-015, CAP-034]
behavioral_contracts:
  - BC-2.11.001  # `query` MCP Tool Accepts Scoping + PrismQL Query String — the primary tool
                 # under test; "FROM crowdstrike_detections LIMIT 5" exercises this BC's
                 # query parsing + execution path.
  - BC-2.11.005  # Ephemeral Materialization — fan_out() → DTU → OCSF normalization → Arrow
                 # RecordBatch → ResponseEnvelope; this test validates the full pipeline.
  - BC-2.09.008  # Response Envelope with Trust Annotations — test asserts `_meta.trust_level`
                 # and `_meta.data_source` per BC-2.09.008 postconditions.
  - BC-2.10.001  # rmcp ServerHandler Implementation — test drives MCP initialize + tools/call
                 # over stdio, exercising BC-2.10.001 postcondition 1.
  - BC-2.10.010  # Graceful Shutdown on SIGTERM/SIGINT — test sends SIGTERM after queries
                 # complete; verifies both prism-bin and DTU server exit cleanly.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — this test is the end-to-end validator that VP-148
            # was intended to enable; it exercises all 4 sensor DTU clones in a single test run.
depends_on:
  - S-DEMO-001   # AdapterRegistry must be populated before any MCP query can return data.
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Per-org overlay loading needed for DTU base_url routing.
blocks:
  - S-DEMO-003   # Demo setup scripts should not ship until the smoke test confirms the green path.
points: 8
# Points justification:
#   - Subprocess launch + port-ready polling for DTU demo server: ~1 pt
#   - Subprocess launch for prism-bin with temp config: ~1 pt
#   - MCP handshake over stdio (initialize → tools/list): ~1.5 pts
#   - 4 × tool_query assertions (one per sensor): ~2 pts
#   - ResponseEnvelope _meta field assertions: ~1 pt
#   - SIGTERM + clean exit assertions: ~0.5 pts
#   - CI profile setup (e2e nextest profile): ~1 pt
#   Total: 8 points (~1.5-2 days)
estimated_days: 2
risk: MEDIUM
# Risk justification: Subprocess integration tests have process timing sensitivity (port
# binding, ready signal). DTU server startup uses a URL file as the ready signal; polling
# for that file in the test harness introduces flake risk if timeout is too short. Mitigation:
# use a generous timeout (30s) with backoff polling. Subprocess test teardown (SIGTERM) must
# handle both clean exits and timeouts gracefully to avoid zombie processes in CI.
acceptance_criteria_count: 10
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "DTU ready signal: poll .prism-dtu-demo-server.urls.json with exponential backoff (max 30s)
    before launching prism-bin; fail fast if file never appears."
  - "Subprocess teardown: always kill both prism-bin and DTU server in test teardown, even
    if assertions fail. Use a drop guard (impl Drop) to ensure cleanup runs on panic."
  - "Port conflicts: use ephemeral ports (bind :0) for DTU server; read actual port from
    urls.json file; write per-test temp config with that port."
  - "No production credentials: test uses dummy credential values (client_id='test-ci',
    client_secret='test-ci-secret') bootstrapped directly into the OS keyring via a helper
    function; DTU clones accept any values."
  - "Windows CI skip: subprocess test uses SIGTERM; acceptable to gate with #[cfg(unix)] or
    cfg_attr(not(target_os = 'windows'), ignore) if Windows subprocess handling is complex.
    Confirm in open question 1."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-dtu-demo-server/src/main.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.09.008-response-envelope-trust-annotations.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.001-rmcp-server-handler.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.010-graceful-shutdown.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-DEMO-001-spec-driven-sensor-adapter-and-boot-step-9a.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-002 — prism-bin: E2E Subprocess Smoke Test (All 4 Sensors)

**Story ID:** S-DEMO-002
**Status:** draft
**Version:** v1.0
**Wave:** 5
**Priority:** P0
**Points:** 8

---

## Origin

New story required per E2E-DEMO-WIRING-PLAN.md §2 (h) "End-to-end smoke test harness".
User scope decision 2026-05-29: all 4 sensors exercised in the same test run.
CrowdStrike uses OAuth2 plugin path; Armis/Claroty/Cyberint use bearer_static path via
`SpecDrivenSensorAdapter` (no per-sensor WASM plugin needed).

---

## Narrative

As a Prism platform engineer, I want an end-to-end integration test that launches the DTU
demo server and prism-bin as real subprocesses, drives the complete MCP stdio round-trip
(initialize → tools/call → tool_query), and asserts live Arrow data returns from all 4
sensor DTU clones, so that regressions in the GAP-002-A closure are caught automatically
before merging.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.09.008 | Response Envelope with Trust Annotations |
| BC-2.10.001 | rmcp ServerHandler Implementation |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT |

---

## Acceptance Criteria

### AC-001: Test launches DTU server and prism-bin without manual intervention
Given: A clean CI environment with the workspace built in release mode
  (`cargo build --release -p prism-bin -p prism-dtu-demo-server`).
When: The test runs.
Then: Both subprocesses launch without error; DTU server writes `.prism-dtu-demo-server.urls.json`
within 10 seconds; prism-bin completes boot sequence and accepts MCP connections.
(traces to BC-2.22.001 postcondition: "The MCP server binds to stdio ONLY AFTER step 8 is complete")
Red Gate test: `test_BC_2_22_001_e2e_smoke_test_launches_dtu_and_prism_bin_without_error`

### AC-002: tools/list returns at least 1 tool (tool_query present)
Given: Both subprocesses are running and the MCP initialize handshake completes.
When: The test sends `tools/list` JSON-RPC over stdio.
Then: The response contains `tool_query` in the tools array.
(traces to BC-2.10.001 postcondition: "rmcp ServerHandler registers all tools via #[tool_router]")

### AC-003: CrowdStrike query returns non-empty Arrow batches with OCSF fields
Given: Demo-org's CrowdStrike sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with `tool_query` and input `"FROM crowdstrike_detections LIMIT 5"`.
Then: The ResponseEnvelope contains at least 1 row of data; the `category_uid` and `class_uid`
fields are present and non-null; no error code in the response.
(traces to BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF via the OCSF normalizer")
Red Gate test: `test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data`

### AC-004: Armis query returns non-empty Arrow batches
Given: Demo-org's Armis sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with `tool_query` and input `"FROM armis_devices LIMIT 5"`
(or the canonical table name for Armis per the TOML spec).
Then: The ResponseEnvelope contains at least 1 row; no error code.
(traces to BC-2.11.005 postcondition: same as AC-003 for Armis sensor)
Red Gate test: `test_BC_2_11_005_e2e_armis_query_returns_data`

### AC-005: Claroty query returns non-empty Arrow batches
Given: Demo-org's Claroty sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with `tool_query` and input `"FROM claroty_assets LIMIT 5"`
(or the canonical table name for Claroty per the TOML spec).
Then: The ResponseEnvelope contains at least 1 row; no error code.
(traces to BC-2.11.005 postcondition: same as AC-003 for Claroty sensor)

### AC-006: Cyberint query returns non-empty Arrow batches
Given: Demo-org's Cyberint sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with `tool_query` and input `"FROM cyberint_alerts LIMIT 5"`
(or the canonical table name for Cyberint per the TOML spec).
Then: The ResponseEnvelope contains at least 1 row; no error code.
(traces to BC-2.11.005 postcondition: same as AC-003 for Cyberint sensor)

### AC-007: ResponseEnvelope _meta fields are correct
Given: A successful tool_query response for any of the 4 sensors.
When: The test inspects the ResponseEnvelope JSON.
Then: `_meta.trust_level == "untrusted_external"` and `_meta.safety_flags` is an empty array
(no injection flags on synthetic DTU data); `_meta.data_source` contains the sensor name
(e.g., `["crowdstrike"]` for a CrowdStrike query).
(traces to BC-2.09.008 postcondition: "ResponseEnvelope carries trust_level and data_source fields")

### AC-008: SIGTERM cleanly shuts down both subprocesses
Given: All 4 sensor queries have completed successfully.
When: The test sends SIGTERM to prism-bin and then to the DTU demo server.
Then: Both processes exit within 5 seconds with status code 0; no `thread panicked` or
`tokio runtime dropped` error messages on stderr; no zombie processes remain.
(traces to BC-2.10.010 postcondition: "Graceful Shutdown on SIGTERM/SIGINT")

### AC-009: Test runs deterministically (no flake)
Given: The test is run 5 consecutive times without changing code.
When: Each run completes.
Then: All 5 runs pass; no race conditions between subprocess startup and MCP connection;
ready-signal polling (`.prism-dtu-demo-server.urls.json` appearance) is the synchronization point.
(traces to BC-2.11.005 invariant: "The SessionContext and all materialized data are dropped
when the tool call returns — no cross-call state")

### AC-010: Test is gated behind `#[ignore]` with explicit CI profile un-ignoring
Given: CI matrix runs standard nextest profile (no DTU server available).
When: Standard `cargo nextest run -p prism-bin` is executed.
Then: The E2E smoke test is skipped (marked `#[ignore]`). When the CI runs with
`cargo nextest run -p prism-bin --profile e2e` (a dedicated CI job), the test runs.
The `#[ignore]` annotation includes a code comment: `// E2E-001: requires DTU server
running; un-gated in CI via 'e2e' nextest profile.`
(traces to BC-2.22.001 invariant: "boot orchestration makes startup deterministic and testable")

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Test lives in `crates/prism-bin/tests/e2e_smoke.rs` (integration test dir) | CLAUDE.md "In-process vs subprocess tests" | Must not be in `src/` — subprocess tests belong in `tests/` |
| Temp config must use `tempfile::TempDir` for cleanup | CLAUDE.md production-grade rule | Test drops TempDir in teardown; no hardcoded `/tmp/prism-demo` paths |
| Credential bootstrap must NOT hardcode values in source visible to AI | AD-017 — AI-opaque credential model | Use `prism-credentials` test-helpers feature gate or OS keyring CLI in test setup, not `std::env::var()` with plaintext values |
| No `std::thread::sleep()` for subprocess ready polling | CLAUDE.md async discipline | Use `tokio::time::sleep()` with exponential backoff in `async fn wait_for_file(path)` |
| DTU subprocess uses Release binary | E2E correctness | `cargo build --release` in test setup; debug binary too slow for 30s timeout |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `tokio` | workspace version | async subprocess management (`tokio::process::Command`) |
| `tempfile` | workspace version | `TempDir` for per-test config isolation |
| `serde_json` | workspace version | Parse DTU urls.json and MCP JSON-RPC responses |
| `rmcp` | `1.7` (per S-5.01-FOLLOWUP-MCP-BOOT) | MCP client in test harness (or raw JSON-RPC over stdin/stdout) |
| `prism-credentials` | workspace path | Credential bootstrap helper (test-helpers feature gate) |

Version source: workspace `Cargo.toml`. `rmcp` version confirmed from S-5.01-FOLLOWUP-MCP-BOOT changelog.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/tests/e2e_smoke.rs` | CREATE | Integration test file with all 10 ACs |
| `crates/prism-bin/tests/helpers/mod.rs` | CREATE | `SubprocessGuard` (drop → SIGTERM), `wait_for_file()`, `write_demo_config()`, `bootstrap_credentials()` |
| `crates/prism-bin/tests/fixtures/demo-prism.toml.template` | CREATE | Template prism.toml with `{{DTU_CROWDSTRIKE_PORT}}` etc. placeholders filled by test setup |
| `crates/prism-bin/tests/fixtures/demo.toml` | CREATE | DTU demo server config for test (same 4 sensors) |
| `.cargo/nextest.toml` (or `Cargo.toml` nextest section) | MODIFY | Add `[profile.e2e]` that un-ignores E2E-tagged tests |

---

## Tasks

1. **Read** `crates/prism-dtu-demo-server/src/main.rs` and `harness.rs` — understand how to launch the demo server with a config file and how `.prism-dtu-demo-server.urls.json` is written.
2. **Read** `crates/prism-bin/src/boot.rs` — understand prism-bin start invocation signature and any `--config` flag that points to the config directory.
3. **Read** `crates/prism-sensors/specs/` — identify the canonical table names for each sensor (e.g., `crowdstrike_detections`, `armis_devices`, `claroty_assets`, `cyberint_alerts`) before writing the query strings in AC-003..006.
4. **Write** `crates/prism-bin/tests/helpers/mod.rs` — `SubprocessGuard`, `wait_for_file()`, `write_demo_config()`.
5. **Write Red Gate tests** in `crates/prism-bin/tests/e2e_smoke.rs` — all 4 Red Gate tests (AC-001, AC-003, AC-004, AC-009 shape) fail RED before S-DEMO-001 is merged.
6. **Implement** `bootstrap_credentials()` helper — uses `prism-credentials` test-helpers feature or OS keyring CLI to write dummy credentials for all 4 sensors.
7. **Implement** `write_demo_config()` — generates `prism.toml` with `demo-org` org entry, correct `spec_dir`, `plugin_dir`, `state_dir`, and per-sensor `customers/demo-org/*.sensor.toml` overlay files pointing at DTU ports read from `urls.json`.
8. **Implement** `launch_dtu_server()` — spawns `prism-dtu-demo-server start --config tests/fixtures/demo.toml` with temp state dir; polls for `urls.json` with 30s timeout; returns `SubprocessGuard` + parsed ports.
9. **Implement** `launch_prism_bin()` — spawns `prism-bin start --config <temp_dir>` with stdin/stdout pipes; wraps stdio in a `rmcp` client or raw JSON-RPC writer; returns `SubprocessGuard` + IO handles.
10. **Implement** MCP handshake in test — send `initialize`, receive `initialized`, send `tools/list`, assert `tool_query` present.
11. **Implement** 4 × query assertions (AC-003..006) — each sends `tools/call` with appropriate query string; asserts non-empty data and OCSF fields.
12. **Implement** `_meta` assertions (AC-007) — parse `ResponseEnvelope` JSON; assert `trust_level` and `safety_flags`.
13. **Implement** SIGTERM teardown (AC-008) in `SubprocessGuard::drop()`.
14. **Add** `[profile.e2e]` to nextest config — un-ignores tests tagged `// E2E-001:`.
15. **Run** `cargo nextest run -p prism-bin --profile e2e` after S-DEMO-001 merges; all assertions must pass GREEN.
16. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-DEMO-001** (depends_on): This story's test will fail RED until S-DEMO-001 merges (AdapterRegistry populated). That is correct and expected. The Red Gate tests should be written to validate the desired behavior, not the current empty-registry behavior.
- **S-5.01-FOLLOWUP-MCP-BOOT** (merged): Delivered the rmcp PrismServer. The test's MCP client must use rmcp 1.7 (same version as server) for handshake compatibility. Confirm the exact transport: the server uses stdio.
- **S-6.20** (merged): `prism-dtu-demo-server` multi-clone demo harness is already implemented. The test harness uses this binary directly; no new demo server code needed.
- **PLUGIN-MIGRATION-001-D** (merged): Delivered the 4 production TOML sensor specs at `crates/prism-sensors/specs/`. Read these to get the canonical table names for AC-003..006 query strings.

---

## Open Questions

1. **Windows CI**: The SIGTERM-based subprocess teardown is Unix-specific. Should the E2E test be `#[cfg(unix)]`-gated entirely, or should a Windows-compatible teardown be implemented using `taskkill /F /PID`? Architect to confirm. If Windows is low-priority for the demo, `#[cfg(unix)]` gating is acceptable with a TODO comment.

2. **MCP client in test**: Should the test use `rmcp` as an MCP client (requires adding `rmcp` as a dev-dependency) or should it write raw JSON-RPC messages to stdin and parse stdout? The rmcp approach is higher-fidelity but adds a dev dep. The raw JSON-RPC approach is simpler and more portable. Architect to confirm.

3. **Canonical Armis/Claroty/Cyberint table names**: The query strings in AC-004..006 need the exact table names as declared in the TOML specs (e.g., is it `armis_devices` or `armis_asset_vulnerabilities`?). Story-writer deferred to implementer to read `crates/prism-sensors/specs/` for the canonical names. Story-writer notes that using the wrong table name produces an `AdapterNotFound` error that masks the AC-002-A closure test; implementer must verify before writing the query strings.

4. **Credential bootstrap mechanism**: `prism-credentials` has a `test-helpers` feature that may expose a direct keyring write function. If that feature exists, use it. If not, the test should spawn a `security add-generic-password` (macOS) or `secret-tool store` (Linux) subprocess. Confirm the correct mechanism by reading `crates/prism-credentials/src/lib.rs`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU server fails to start (port conflict) | Test fails with clear message: "DTU server did not write urls.json within 30s" |
| EC-002 | prism-bin exits before MCP handshake completes | Test fails with clear message: "prism-bin exited unexpectedly with code N"; SubprocessGuard teardown logs stderr for diagnosis |
| EC-003 | A sensor query returns zero rows (DTU returned empty) | Test fails with AC assertion: "expected at least 1 row"; this is a data fidelity issue in the DTU clone, not a framework issue |
| EC-004 | `LIMIT 0` query variant | An additional edge-case test verifies that `LIMIT 0` returns empty-but-not-error response (E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation) |
| EC-005 | `LIMIT 200` query variant | An additional edge-case test verifies that `LIMIT 200` triggers pagination in the DTU clone and returns 200 rows (exercises pagination at least one extra page per E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation) |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| BC files (5 BCs) | ~8,000 |
| prism-dtu-demo-server/src/main.rs + harness.rs | ~6,000 |
| crates/prism-sensors/specs/ (4 TOML files, table names) | ~4,000 |
| crates/prism-bin/src/boot.rs (CLI invocation) | ~4,000 |
| S-DEMO-001 story (dependency context) | ~4,000 |
| S-5.01-FOLLOWUP-MCP-BOOT story (rmcp version context) | ~3,000 |
| Test output during iteration | ~3,000 |
| **Total estimate** | **~36,500 tokens (~14% of 256K context)** |

Well within budget; second-cheapest story in the E-DEMO epic.

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| Hardcoded production credentials or real API keys in test fixtures | AD-017 AI-opaque credential model |
| `std::thread::sleep()` for subprocess synchronization | CLAUDE.md async discipline |
| Hardcoded port numbers (e.g., `127.0.0.1:8080`) | DTU server binds to ephemeral port; port read from urls.json |
| Inline `unwrap()` in test helper code | CLAUDE.md Error handling rule; test helpers should propagate errors clearly |

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
