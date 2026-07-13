---
document_type: story
story_id: S-DEMO-002
title: "prism-bin: E2E Subprocess Smoke Test — All 4 Sensors + Multi-Org Isolation via DTU Clones + MCP Round-Trip"
wave: 5
epic_id: E-DEMO
priority: P0
status: ready
version: "2.7"
level: "L4"
producer: story-writer
timestamp: "2026-06-02T00:00:00Z"
modified: "2026-07-13T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-10, SS-11, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) is exercised end-to-end — SpecDrivenSensorAdapter dispatches
#     to each of the 4 DTU clone endpoints; the test verifies real data flows through.
#   SS-10 (MCP Interface) — test drives MCP stdio transport (initialize handshake, tools/call),
#     exercising the rmcp ServerHandler delivered by S-5.01-FOLLOWUP-MCP-BOOT.
#   SS-11 (Query Execution) — `query` MCP tool invocation exercises QueryEngine fan_out()
#     materialization pipeline, OCSF normalization, and ResponseEnvelope wrapping.
#   SS-22 (Binary Entrypoint) — test launches prism-bin start as a subprocess and validates
#     clean SIGTERM shutdown per BC-2.10.010.
crates_touched: [prism-bin, prism-core, prism-credentials, prism-dtu-armis, prism-dtu-cyberint, prism-dtu-demo-server, prism-mcp, prism-query, prism-sensors, prism-spec-engine]
# crates_touched notes (verified against git diff cd4a2211...81cf3678 per ADV-SDEMO002-PR-P08-HIGH-001):
#   prism-bin: SRC (src/boot.rs, src/spec_driven_adapter.rs) + TEST (tests/e2e_smoke.rs,
#              tests/helpers/mod.rs, tests/bc_2_01_013_spec_driven_adapter.rs,
#              tests/bc_2_03_013_credential_init.rs, tests/vp153_rule_c_shaped_probe.rs)
#              + CONFIG (Cargo.toml, fixtures/e2e-demo/demo.toml)
#   prism-core: SRC (src/error.rs — PrismError::SensorNotRegisteredForOrg / E-QUERY-032 variant)
#   prism-credentials: SRC (src/crud.rs, src/resolution.rs — OBS-001 fix; bootstrap helper)
#                      + TEST (tests/bc_2_03_005_credential_crud.rs,
#                              tests/bc_2_03_006_credential_resolution.rs)
#   prism-dtu-armis: SRC (src/routes/search.rs — AQL fidelity for AC-004 roundtrip)
#                    + TEST (tests/s_demo_armis_aql_001_red_gate.rs)
#   prism-dtu-cyberint: SRC (src/clone.rs — cookie-roundtrip auth support for DTU)
#   prism-dtu-demo-server: SRC (src/config.rs, src/harness.rs — multi-clone harness extensions)
#   prism-mcp: SRC (src/error_mapping.rs — E-QUERY-032 → -32602 mapping arm per AC-012;
#                   src/safety_envelope.rs — object-shaped rows injection-scan per AC-007)
#              + TEST (tests/tool_dispatch_tests.rs)
#   prism-query: SRC (src/engine.rs, src/materialization.rs — AQL push-down seeding D-934)
#                + TEST (src/tests/aql_pushdown_tests.rs, src/tests/mod.rs)
#   prism-sensors: CONFIG (specs/armis.sensor.toml, claroty.sensor.toml,
#                          crowdstrike.sensor.toml, cyberint.sensor.toml — TOML spec updates only)
#   prism-spec-engine: SRC (src/interpolation.rs, src/plugin/discovery.rs, src/plugin/mod.rs,
#                           src/plugin_auth_provider.rs, src/spec_parser.rs — WASM acquire-token
#                           nested-interface dispatch per LOW-002 resolution)
#                      + TEST (tests/bc_2_01_016_test.rs, tests/bc_2_16_001_bundled_spec_load.rs,
#                              tests/bc_2_16_012_test.rs, tests/crowdstrike_oauth2_plugin_tests.rs,
#                              tests/vp153_sensorauth_cross_composition.rs)
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
  - BC-3.2.001   # Per-Org Sensor Data Isolation — AC-011..013 verify that org-A queries cannot
                 # return org-B data and that AdapterNotFound is returned for sensors not
                 # registered for a given org. Multi-org registration is the structural proof.
  - BC-2.22.001  # Boot Orchestration — AC-001 and AC-010 trace to BC-2.22.001 postconditions
                 # ("The MCP server binds to stdio ONLY AFTER step 8 is complete"; "boot
                 # orchestration makes startup deterministic and testable"). AC-011 also traces
                 # to BC-2.22.001 postcondition for deterministic 3-org boot.
  - BC-2.11.007  # Sensor Filter Push-Down — AC-014 verifies end-to-end AQL push-down via
                 # Mechanism B (Verbatim-AQL Passthrough): user writes `aql = '<string>'`
                 # pseudo-column in PrismQL WHERE; prism-query seeds verbatim string into
                 # FetchContext.query_filters["aql"]; forwarded opaque to DTU
                 # /api/v1/search?aql=<value> per R-DTU-002 / ADR-031 §D8-a. The query-layer
                 # seeding (prism-query → PipelineExecutor) is S-DEMO-002 scope per D-934.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — this test is the end-to-end validator that VP-148
            # was intended to enable; it exercises all 4 sensor DTU clones in a single test run.
depends_on:
  - S-DEMO-001   # AdapterRegistry must be populated before any MCP query can return data.
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Per-org overlay loading needed for DTU base_url routing.
blocks:
  - S-DEMO-003   # Demo setup scripts should not ship until the smoke test confirms the green path.
points: 13
# Points justification (revised v1.0→v1.1: +3 pts for multi-org isolation scope;
#   v2.0: +2 pts for absorbed S-DEMO-CI-E2E-001 CI workflow scope):
#   - Subprocess launch + port-ready polling for DTU demo server: ~1 pt
#   - Subprocess launch for prism-bin with temp config: ~1 pt
#   - MCP handshake over stdio (initialize → tools/list): ~1.5 pts
#   - 4 × `query` tool assertions (one per sensor): ~2 pts
#   - ResponseEnvelope _meta field assertions: ~1 pt
#   - SIGTERM + clean exit assertions: ~0.5 pts
#   - CI profile setup (e2e nextest profile): ~1 pt
#   - AC-011: multi-org registration (3 orgs × different sensor combos): ~1 pt
#   - AC-012: cross-org isolation probe (AdapterNotFound for wrong-org sensor): ~1 pt
#   - AC-013: DTU multi-tenant scope documentation + single-clone multi-org wiring: ~1 pt
#   - Task 25: .github/workflows/e2e.yml CI workflow (PR+push triggers, DTU lifecycle,
#     release build, JUnit artifact upload): ~2 pts (absorbed from S-DEMO-CI-E2E-001)
#   Total: 13 points (~2.5-3 days)
estimated_days: 3
risk: MEDIUM
# Risk justification: Subprocess integration tests have process timing sensitivity (port
# binding, ready signal). DTU server startup uses a URL file as the ready signal; polling
# for that file in the test harness introduces flake risk if timeout is too short. Mitigation:
# use a generous timeout (30s) with backoff polling. Subprocess test teardown (SIGTERM) must
# handle both clean exits and timeouts gracefully to avoid zombie processes in CI.
acceptance_criteria_count: 14
red_gate_tests: 9
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
  - "crates/prism-query/src/lib.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - ".factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.09.008-response-envelope-trust-annotations.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.001-rmcp-server-handler.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.010-graceful-shutdown.md"
  - ".factory/specs/behavioral-contracts/BC-3.2.001-per-org-sensor-data-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-DEMO-001-spec-driven-sensor-adapter-and-boot-step-9a.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-002 v2.6 — prism-bin: E2E Subprocess Smoke Test (All 4 Sensors + Multi-Org Isolation)

**Story ID:** S-DEMO-002
**Status:** ready
**Version:** v2.7
**Wave:** 5
**Priority:** P0
**Points:** 13

---

## Origin

New story required per E2E-DEMO-WIRING-PLAN.md §2 (h) "End-to-end smoke test harness".
User scope decision 2026-05-29: all 4 sensors exercised in the same test run.

v1.1 expansion (architect 2026-05-29): Multi-org isolation ACs added per user requirement:
"multiple client orgs registered in prism, each with a DIFFERENT sensor combo" exercises
ADR-029 per-org overlay resolution and BC-3.2.001 isolation. S-DEMO-002 v1.0 had no ACs
for multi-org registration or cross-org isolation probing — a confirmed spec gap.

Auth model (corrected per S-DEMO-001 v1.1 architect revision 2026-05-29):
- CrowdStrike uses OAuth2 WASM plugin path (`PluginAuthProvider`).
- Armis + Claroty use `bearer_static` path via `BearerStaticAuthProvider` (no WASM plugin).
- Cyberint uses `cookie_roundtrip` path via `CookieLoginAuthProvider` (POST /login → cyberint_session cookie).

Previous v1.0 statement "Armis/Claroty/Cyberint use bearer_static path" was incorrect for
Cyberint. `cyberint.sensor.toml` declares `auth_type = "cookie_roundtrip"` (D-737 LOCKED).

---

## Narrative

As a Prism platform engineer, I want an end-to-end integration test that (1) launches the
DTU demo server and prism-bin as real subprocesses with multiple registered orgs, (2) drives
the complete MCP stdio round-trip (initialize → tools/call → `query`) for all 4 sensor
DTU clones, (3) verifies that each org's query reaches the correct sensor, and (4) verifies
that querying a sensor NOT registered for an org returns an isolation error rather than data,
so that regressions in both GAP-002-A closure and BC-3.2.001 org isolation are caught
automatically before merging.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.09.008 | Response Envelope with Trust Annotations |
| BC-2.10.001 | rmcp ServerHandler Implementation |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |
| BC-2.11.007 | Sensor Filter Push-Down |

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

### AC-002: tools/list returns at least 1 tool (`query` present)
Given: Both subprocesses are running and the MCP initialize handshake completes.
When: The test sends `tools/list` JSON-RPC over stdio.
Then: The response contains `query` in the tools array (the canonical name registered by `PrismServer` via `#[tool_router]`; NOT `tool_query`).
(traces to BC-2.10.001 postcondition: "rmcp ServerHandler registers all tools via #[tool_router]")

### AC-003: CrowdStrike query returns non-empty Arrow batches with OCSF fields
Given: Demo-org's CrowdStrike sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input `"SELECT * FROM crowdstrike_detections LIMIT 5"` (i.e., `{"name": "query", "arguments": {"query": "SELECT * FROM crowdstrike_detections LIMIT 5"}}`).
Then: The ResponseEnvelope contains at least 1 row of data; the `category_uid` and `class_uid`
fields are present and non-null; the `detection_id` column (the primary key per Gap-CS-001 fix —
NOT `id`) is present and non-null for each row; no error code in the response.
(traces to BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF via the OCSF normalizer")
Red Gate test: `test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data`

### AC-004: Armis query with AQL predicate returns non-empty Arrow batches
Given: Demo-org's Armis sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input
`"SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"`
(call shape: `{"name": "query", "arguments": {"query": "SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"}}`).
Then: The ResponseEnvelope contains at least 1 row; no error code; the DTU clone receives
`GET /api/v1/search?aql=in:devices` (AQL value forwarded opaque via `${query.filter.aql}`
interpolation in `armis.sensor.toml` `fetch_devices` path_template).

**AQL predicate is mandatory for Armis queries.** The `armis.sensor.toml` `fetch_devices` step
declares `path_template = "/api/v1/search?aql=${query.filter.aql}"` with NO default for
`${query.filter.aql}`. A bare `FROM armis_devices LIMIT 5` (no WHERE aql = ...) is NOT a
supported query for the Armis sensor — the `${query.filter.aql}` interpolation variable is absent
and the fetch errors before any HTTP call. This mirrors the real Armis Centrix `/api/v1/search`
API contract, which requires an `aql` query parameter. `WHERE aql = 'in:devices'` is the correct
pattern (AC-014 AQL push-down convention; `in:devices` is the real Armis entity discriminator
per research artifact 2026-06-01 and `armis.sensor.toml` DTU-EXT-003 comment).

Note: the same AQL requirement applies to the `armis_alerts` table — any test querying
`FROM armis_alerts` must supply `WHERE aql = 'in:alerts'` (or a valid Armis AQL filter string).

(traces to BC-2.11.005 postcondition: same as AC-003 for Armis sensor; BC-2.11.007 §Mechanism B
Verbatim-AQL Passthrough — AQL predicate is the mandatory calling convention for Armis tables)
Red Gate test: `test_BC_2_11_005_e2e_armis_query_returns_data`

### AC-005: Claroty query returns non-empty Arrow batches
Given: Demo-org's Claroty sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input `"SELECT * FROM claroty_alerts LIMIT 5"` (call shape: `{"name": "query", "arguments": {"query": "..."}}`).
Then: The ResponseEnvelope contains at least 1 row; the `alert_type_name` column (not `type`)
and `detected_time` column (not `created_at`) are present per Gap-CL-005 TOML fix; no error code.
Also test `"SELECT * FROM claroty_devices LIMIT 5"` — expects at least 1 row with `uid` column present
(Gap-CL-003 fix — devices table added to claroty.sensor.toml).
Note: queries beyond page 1 are limited by Gap-CL-004 (offset pagination sent as URL params, not
POST body) — assert only for page-1 result rows (≤ page_size=100).
(traces to BC-2.11.005 postcondition: same as AC-003 for Claroty sensor)

### AC-006: Cyberint query returns non-empty Arrow batches
Given: Demo-org's Cyberint sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input `"SELECT * FROM cyberint_alerts LIMIT 5"`
(or the canonical table name for Cyberint per the TOML spec; call shape: `{"name": "query", "arguments": {"query": "..."}}`).
Then: The ResponseEnvelope contains at least 1 row; no error code.
(traces to BC-2.11.005 postcondition: same as AC-003 for Cyberint sensor)

### AC-007: ResponseEnvelope _meta fields are correct
Given: A successful `query` tool response for any of the 4 sensors.
When: The test inspects the ResponseEnvelope JSON.
Then: `_meta.trust_level == "untrusted_external"` and `_meta.data_source` contains the sensor name
(e.g., `["crowdstrike"]` for a CrowdStrike query).

**Injection-scan assertion (MED-002 resolution):** `_meta.safety_flags` is an empty array on
synthetic DTU data. The assertion MUST be meaningful — the test must send a query that returns at
least 1 row (so `results.rows` is a non-empty array) and then assert `safety_flags == []`. This
verifies that `SafetyEnvelopeBuilder::wrap()` ran the injection scanner over the row data
and found no patterns in the synthetic DTU fixture data.

**What `wrap()` scans:** For the `query` tool, `results` in the `ResponseEnvelope` is a JSON
Object with shape `{"rows": [...], "returned_results": N, "total_available": N, "is_truncated": bool}`.
`SafetyEnvelopeBuilder::wrap()` detects this object shape via `results.get("rows")` and scans
the `rows` array. Each row element is passed through `collect_string_fields` (IMP-6 fix,
MAX_SCAN_DEPTH=64). The assertion `safety_flags == []` is meaningful only when `results.rows`
contains at least 1 element; with 0 rows the scan trivially produces no flags regardless of
implementation correctness.

**Implementation note:** when `results` is a metadata-only JSON Object with no `"rows"` key
— e.g., for `explain_query`, `create_alias` tool responses — `wrap()` does NOT invoke
`collect_string_fields` and `safety_flags` is always empty (no attacker-controllable sensor
field values present; documented in `wrap()` doc comment). That case is NOT what AC-007
exercises; AC-007 explicitly targets the `query` tool's object-shaped `results` (with `rows`).

(traces to BC-2.09.008 postcondition: "ResponseEnvelope carries trust_level and data_source fields"
and "`_meta.safety_flags` is always present (empty array when no flags triggered)")

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
**Coverage decision (F-PC-002 — in scope, implementer must verify):** AC-009 is validated by
running the full smoke test suite 5 consecutive times in the dedicated `.github/workflows/e2e.yml`
CI job (delivered in this story's PR #171) and confirming zero failures. The `[profile.e2e]`
nextest configuration includes `retries = 1` to absorb single transient flakes; a double failure is
a real regression. This is a CI-gate behavior, not a separate Rust `#[test]` function. The
implementer MUST confirm 5 consecutive local green runs before declaring AC-009 satisfied; the CI
job then provides ongoing regression protection on every PR and push. No additional `#[test]`
function is required beyond the primary smoke test functions — determinism is a *property* of the
existing tests, verified by repetition.

### AC-010: Test is gated behind `#[ignore]` with automatic CI execution via dedicated e2e job
Given: The standard CI matrix (`ci.yml`) runs `cargo nextest run --workspace` without a DTU
server available.
When: Standard `cargo nextest run -p prism-bin` is executed (no `--profile e2e` flag).
Then: The E2E smoke test functions in `crates/prism-bin/tests/e2e_smoke.rs` are skipped
(each is marked `#[ignore]`). The `#[ignore]` attribute on each test includes the code comment:
`// E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.`

A dedicated GitHub Actions workflow (`.github/workflows/e2e.yml`, delivered in this PR) runs the
suite automatically. The canonical workflow attributes are:
- **job name:** "E2E smoke"
- **runs-on:** `ubuntu-latest`
- **triggers:** `pull_request` and `push` to `[develop, main]`, plus `workflow_dispatch`
  (no schedule/cron — PR+push gating is the contract; a scheduled drift-detection run is a
  future maintenance PR if desired)
- **timeout-minutes:** 45
- **release build:** `cargo build --release -p prism-bin -p prism-dtu-demo-server`
- **canonical command:** `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only --no-tests=fail`
  (un-ignoring is via the CLI flag `--run-ignored ignored-only`, NOT a profile config key;
  `--no-tests=fail` causes the job to fail if zero tests are selected — prevents a false-green if
  a future refactor removes the `#[ignore]` attributes or renames the test file)
- launches the DTU demo server and polls for `.prism-dtu-demo-server.urls.json` (30s timeout)
- runs DTU teardown unconditionally (`if: always()`) to prevent zombie processes
- uploads JUnit XML artifacts on failure for post-mortem diagnosis

The `[profile.e2e]` section in `.config/nextest.toml` sets `slow-timeout = { period = "120s" }`
and `retries = 1`. The `retries = 1` setting absorbs single transient flakes; a double failure is
a real regression. Note: nextest does NOT support a `run-ignored` key in the profile — un-ignoring
is always specified via the CLI flag `--run-ignored ignored-only`, not in `.config/nextest.toml`.

The net effect: E2E tests are invisible to developers running standard `just check` locally
(no DTU server required), but run automatically on every PR and push to develop and main so
regressions are caught before merge, not discovered during adversarial review.

(traces to BC-2.22.001 invariant: "boot orchestration makes startup deterministic and testable")

### AC-011: Multiple orgs with different sensor combos can be registered simultaneously
Given: Prism starts with 3 orgs configured:
  - `demo-org-a`: CrowdStrike + Armis (2 sensors)
  - `demo-org-b`: Claroty + Cyberint (2 sensors)
  - `demo-org-c`: all 4 sensors (CrowdStrike + Armis + Claroty + Cyberint)
Each org has a distinct `org_id` (UUIDv7) and `org_slug` with corresponding
`customers/{org_slug}/` overlay directories setting DTU clone `base_url` per sensor.
When: Boot step 9A completes.
Then: `AdapterRegistry` contains 8 entries (2+2+4); each (org_id, sensor_id) pair resolves
to the correct `SpecDrivenSensorAdapter`; no cross-org aliasing exists in the registry.
(traces to BC-2.22.001 postcondition + ADR-029 per-org overlay resolution)
Red Gate test: `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count`

### AC-012: Cross-org isolation — querying a sensor not registered for an org returns E-QUERY-032
Given: `demo-org-a` has CrowdStrike + Armis but NOT Claroty or Cyberint.
When: The test sends a `query` MCP tool call with `{"name": "query", "arguments": {"query": "SELECT * FROM claroty_alerts LIMIT 5", "clients": ["demo-org-a"]}}`.
Note: The scoping param is **`clients`** (array of org slug strings), NOT `org_slug` — `QueryToolParams`
uses `clients: Option<Vec<String>>` per BC-2.11.001 Preconditions and the as-built `QueryToolParams`
in `prism-mcp/server.rs` (which has `#[serde(deny_unknown_fields)]`; passing `org_slug` would be
rejected at deserialization before isolation logic runs).
Then: The MCP response is an **error** (not a success envelope with zero rows). The error response
MUST satisfy ALL of the following matcher assertions:
1. The MCP response contains `"error"` at the top level (JSON-RPC error object, NOT a success
   `result` containing an empty `results` array).
2. The error `code` is `-32602` (INVALID_PARAMS — the org+sensor combination is an invalid query
   scope parameter, per E-QUERY-032 `map_prism_error` routing).
3. The error `message` contains the substring `"E-QUERY-032"`.
4. The error `message` contains the substring `"claroty"` (the sensor_id that is not registered
   for demo-org-a).
5. The error `message` contains the substring `"demo-org-a"` (the org_slug for which the sensor
   is not registered).
6. Zero data rows are returned (error response has no `results` array with row data).
7. No Claroty data from `demo-org-b` is present anywhere in the response.

**Why E-QUERY-032 (not E-SENSOR-010):** `map_prism_error` deliberately redacts all E-SENSOR-*
errors: caller-visible `message` is `"Internal error"` and `suggestion` carries `"See audit log for details."` (BC-2.10.007 message/suggestion split; AD-017 credential opacity). E-SENSOR-010 (AdapterNotFound)
also goes through the partial-failure path as a `sensor_errors` string, never surfaced as an MCP
error response at all. E-QUERY-032 is a new credential-safe error raised by `resolve_source_refs`
in `prism-query` when a clients-scoped query targets a sensor not registered for the specified org —
it carries only the public sensor type name and org slug (no credentials, tokens, or auth chain
details) and surfaces as MCP -32602. See error-taxonomy.md §E-QUERY-032 and BC-3.2.001
postcondition 5 for full rationale.

(traces to BC-3.2.001 postcondition 5: "cross-org query to unregistered sensor returns E-QUERY-032
error envelope, not an empty success envelope"; ADR-007 §2.2 adapter dispatch OrgId verification)
Red Gate test: `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032`

### AC-013: DTU multi-tenant emulation — each org's queries reach the correct DTU clone port
Given: The demo DTU server runs with all 4 sensor clones, each on an ephemeral port; each org's
`customers/{slug}/{sensor}.sensor.toml` overlay points to the correct port for that sensor clone.
When: The test issues `query` tool calls (`{"name": "query", "arguments": {"query": "SELECT * FROM crowdstrike_detections LIMIT 5", "clients": ["<org_slug>"]}}`) for each of the 3 orgs.
Then: All 3 queries succeed (each org's CrowdStrike adapter points to the same DTU clone port
for CrowdStrike — different orgs can share the same DTU clone port in the demo context because
the DTU clone operates in single-tenant mode without org-level data segregation at the HTTP layer;
org isolation is enforced at the `AdapterRegistry` dispatch layer per BC-3.2.001, not at the DTU
HTTP layer in the demo configuration).

**Scope clarification (DTU multi-tenant architecture):** The `prism-dtu-*` clones in the demo
run in single-tenant mode — each clone instance serves all HTTP requests without per-org data
segregation at the transport layer. The multi-tenant isolation guarantee in BC-3.2.001 is
structural at the `AdapterRegistry` keying layer: two different orgs that both have CrowdStrike
can point to the same DTU clone port and receive the same synthetic data — what is isolated is
the *adapter lookup* (Org A cannot accidentally get Org B's adapter), not the *data content* (both
orgs' adapters return the same DTU fixture data). True per-org data segregation at the DTU HTTP
layer is a Wave 3 S-3.2.xx story scope (BC-3.2.003/BC-3.2.004); it is NOT required for the demo.
This scoping decision MUST be documented in a code comment in the test helper: `// DTU-MULTI-001:
demo DTU operates in single-tenant mode; org isolation is at AdapterRegistry layer only.`
(traces to BC-3.2.001 postcondition 3; ADR-006 §3.1 cross-tenant threat model)
**Coverage decision (F-PC-002 — explicit test required, in scope):** AC-013 requires a dedicated
test function. The implementer MUST write:
`test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port` — issues a `query`
tool call `{"name": "query", "arguments": {"query": "SELECT * FROM crowdstrike_detections LIMIT 5", "clients": ["demo-org-a"]}}` and `{"clients": ["demo-org-c"]}` variants; asserts both succeed and
return data rows (verifying both orgs' CrowdStrike adapters are wired to the same DTU clone port);
includes `// DTU-MULTI-001:` comment per scope clarification above. This test is separate from
AC-011's adapter-count test (which verifies registration) — AC-013's test verifies query execution
reaches the correct port at runtime.

### AC-014: AQL filter seeded into FetchContext for Armis end-to-end push-down (Mechanism B passthrough)
Given: A `query` MCP tool call targets an Armis table using the verbatim-AQL pseudo-column convention
(e.g., `"SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"` — the user writes `aql = '<string>'`
as a literal pseudo-column in the PrismQL WHERE clause; `aql` is declared as an INDEX column in
`armis.sensor.toml`).
When: prism-query constructs the FetchContext passed to PipelineExecutor for the Armis adapter.
Then: `FetchContext.query_filters["aql"]` is populated with the verbatim AQL string value extracted
from the `aql = '<string>'` predicate (e.g., `"in:devices"`); the DTU clone receives
`GET /api/v1/search?aql=in:devices` (the string is forwarded opaque — no translation occurs;
per BC-2.11.007 §Predicate Classification Mechanism B and R-DTU-002 / ADR-031 §D8-a); the
response contains rows from the AQL-matched entity type. This verifies end-to-end push-down from
PrismQL → QueryEngine (seeding) → PipelineExecutor (interpolation via `${query.filter.aql}`) → DTU.

**Seeding mechanism accuracy (F-*-MED):** The `aql` INDEX column declaration in `armis.sensor.toml`
is **not** what triggers the seeding path. The INDEX option is currently decorative — it documents
push-down intent but does not gate a special code path at runtime. The actual seeding is performed
by the **generic equality-extraction path** (`predicate_tree_to_filter_map` /
`extract_push_down_filters_as_map`), which extracts ALL `field = 'string'` equality predicates from
the PrismQL WHERE clause regardless of whether the column is declared INDEX. The implementer MUST
use this generic path — do NOT add a special `if column_is_index` branch. The INDEX declaration in
the TOML documents the semantic intent (push-down candidate) and will gate a future optimization
path (per ADR-031 §D8-a); it is NOT a runtime gate today.

Scope note (D-934): The Armis DTU/TOML/interpolation layer was confirmed correctly wired in
S-DEMO-ARMIS-AQL-001 (PR #168; `${query.filter.aql}` path_template variable + `armis.sensor.toml`
fetch_devices path = `/api/v1/search`). The production query-layer seeding — prism-query
QueryEngine extracting the literal `aql` pseudo-column value from the parsed PrismQL WHERE predicate
and populating `FetchContext.query_filters["aql"]` — is this story's scope. This is not a defer-pattern
violation: the seeding is the final piece of the Armis AQL push-down pipeline and belongs to this
E2E integration story.

(traces to BC-2.11.007 §Predicate Classification Mechanism B — Verbatim-AQL Passthrough: `aql = '<string>'`
pseudo-column convention; BC-2.11.001 precondition: `query` MCP tool accepts and forwards WHERE predicates
through to the query engine)

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
| `crates/prism-bin/tests/e2e_smoke.rs` | CREATE | Integration test file with all ACs |
| `crates/prism-bin/tests/helpers/mod.rs` | CREATE | `SubprocessGuard` (drop → SIGTERM), `wait_for_file()`, `write_demo_config()`, `bootstrap_credentials()` |
| `crates/prism-bin/fixtures/e2e-demo/demo.toml` | CREATE | DTU demo server config for test (same 4 sensors). Path is `fixtures/e2e-demo/` (NOT `tests/fixtures/`) — accessed via `CARGO_MANIFEST_DIR/fixtures/e2e-demo/demo.toml` in the test harness (F-PB-MED-002 corrected; phantom `demo-prism.toml.template` removed — `prism.toml` is generated programmatically by `write_demo_config()` helper into a `TempDir`, not from a template file). |
| `.config/nextest.toml` | MODIFY | Add `[profile.e2e]` with `slow-timeout = { period = "120s" }` and `retries = 1`. Note: nextest does NOT support a `run-ignored` profile key — un-ignoring is specified via the CLI flag `--run-ignored ignored-only`, not here. |
| `.github/workflows/e2e.yml` | CREATE | Dedicated e2e CI workflow — job name "E2E smoke", runs-on `ubuntu-latest`, timeout-minutes 45; triggers on `pull_request` + `push` to `[develop, main]` + `workflow_dispatch` (no cron); builds release binaries, launches DTU, runs `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only --no-tests=fail`; closes ADV-SDEMO002-PR-P02-HIGH-001, OBS-2, and ADV-SDEMO002-PR-P05-OBS-001 |
| `crates/prism-credentials/src/` | MODIFY | OBS-001 fix: credential bootstrap helper modifications (src + tests) [SRC+TEST] |
| `crates/prism-core/src/error.rs` | MODIFY | `PrismError::SensorNotRegisteredForOrg` variant (E-QUERY-032) required for AC-012 isolation error [SRC] |
| `crates/prism-mcp/src/error_mapping.rs` | MODIFY | E-QUERY-032 → JSON-RPC `-32602` mapping arm; AC-012 error surface [SRC] |
| `crates/prism-mcp/src/safety_envelope.rs` | MODIFY | Object-shaped `rows` injection-scan path; AC-007 `safety_flags == []` assertion [SRC] |
| `crates/prism-query/src/engine.rs, src/materialization.rs` | MODIFY | E-QUERY-032 cross-org isolation raise (`resolve_source_refs`, BC-3.2.001 postcondition 5) + AQL push-down seeding (`extract_push_down_filters_as_map`, AC-014 / BC-2.11.007 Mechanism B) [SRC] |
| `crates/prism-spec-engine/src/plugin/mod.rs` | MODIFY | WASM acquire-token nested-interface dispatch [SRC] (LOW-002 resolved) |
| `crates/prism-spec-engine/src/plugin/discovery.rs` | MODIFY | Plugin discovery for nested WASM interface [SRC] |
| `crates/prism-spec-engine/src/plugin_auth_provider.rs` | MODIFY | Auth provider wiring for WASM token acquisition [SRC] |
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | Spec parser updates for multi-tenant WASM auth [SRC] |
| `crates/prism-spec-engine/src/interpolation.rs` | MODIFY | Interpolation path update [SRC] |
| `crates/prism-dtu-armis/src/routes/search.rs` | MODIFY | AQL fidelity fix for AC-004 DTU roundtrip verification [SRC] |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Cookie-roundtrip auth support for Cyberint DTU clone [SRC] |
| `crates/prism-dtu-demo-server/src/config.rs` | MODIFY | Multi-clone config extensions for 3-org demo setup [SRC] |
| `crates/prism-dtu-demo-server/src/harness.rs` | MODIFY | Harness extensions for multi-sensor demo server [SRC] |
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Sensor TOML spec updates for AQL path + column parity [CONFIG] |
| `crates/prism-sensors/specs/claroty.sensor.toml` | MODIFY | Sensor TOML spec updates [CONFIG] |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | MODIFY | Sensor TOML spec updates [CONFIG] |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | MODIFY | Sensor TOML spec updates [CONFIG] |

---

## Tasks

1. **Read** `crates/prism-dtu-demo-server/src/main.rs` and `harness.rs` — understand how to launch the demo server with a config file and how `.prism-dtu-demo-server.urls.json` is written.
2. **Read** `crates/prism-bin/src/boot.rs` — understand prism-bin start invocation signature and any `--config` flag that points to the config directory.
3. **Read** `crates/prism-sensors/specs/` — identify the canonical table names for each sensor (e.g., `crowdstrike_detections`, `armis_devices`, `claroty_assets`, `cyberint_alerts`) before writing the query strings in AC-003..006.
4. **Write** `crates/prism-bin/tests/helpers/mod.rs` — `SubprocessGuard`, `wait_for_file()`, `write_demo_config()`.
5. **Write Red Gate tests** in `crates/prism-bin/tests/e2e_smoke.rs` — all 4 Red Gate tests (AC-001, AC-003, AC-004, AC-009 shape) fail RED before S-DEMO-001 is merged.
6. **Implement** `bootstrap_credentials()` helper — uses `prism-credentials` test-helpers feature or OS keyring CLI to write dummy credentials for all 4 sensors.
7. **Implement** `write_demo_config()` — generates `prism.toml` with `demo-org` org entry, correct `spec_dir`, `plugin_dir`, `state_dir`, and per-sensor `customers/demo-org/*.sensor.toml` overlay files pointing at DTU ports read from `urls.json`.
8. **Implement** `launch_dtu_server()` — spawns `prism-dtu-demo-server start --config <CARGO_MANIFEST_DIR>/fixtures/e2e-demo/demo.toml` with temp state dir; polls for `urls.json` with 30s timeout; returns `SubprocessGuard` + parsed ports. (Config path is `fixtures/e2e-demo/demo.toml` per FSR corrected by F-PB-MED-002.)
9. **Implement** `launch_prism_bin()` — spawns `prism-bin start --config-dir <temp_dir>` with stdin/stdout pipes; wraps stdio in a `rmcp` client or raw JSON-RPC writer; returns `SubprocessGuard` + IO handles.
10. **Implement** MCP handshake in test — send `initialize`, receive `initialized`, send `tools/list`, assert `query` present in the tools array (the canonical name registered by `PrismServer`; NOT `tool_query`).
11. **Implement** 4 × query assertions (AC-003..006) — each sends `tools/call` with appropriate query string; asserts non-empty data and OCSF fields. For Armis (AC-004), the query MUST supply the AQL predicate: `"SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"` — bare `SELECT * FROM armis_devices LIMIT 5` (no AQL WHERE) is not a supported query for Armis per AC-004.
12. **Implement** `_meta` assertions (AC-007) — parse `ResponseEnvelope` JSON; assert `trust_level` and `safety_flags`.
13. **Implement** SIGTERM teardown (AC-008) in `SubprocessGuard::drop()`.
14. **Add** `[profile.e2e]` to `.config/nextest.toml` — set `slow-timeout = { period = "120s" }` and `retries = 1`. Do NOT add `run-ignored = "all"` — nextest does not support this key in profiles; un-ignoring is done via the CLI flag `--run-ignored ignored-only`. (Path is `.config/nextest.toml`, NOT `.cargo/nextest.toml` — LOW-001 corrected.)
15. **Implement** `write_multi_org_demo_config()` helper — generates 3-org config with overlapping
    and non-overlapping sensor sets (demo-org-a: CrowdStrike+Armis; demo-org-b: Claroty+Cyberint;
    demo-org-c: all 4). Writes per-org `customers/{slug}/` overlay directories for each sensor.
16. **Write Red Gate test** `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count`
    (AC-011): asserts `AdapterRegistry` contains exactly 8 entries after boot with 3-org config.
17. **Write Red Gate test** `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032`
    (AC-012): asserts that a `query` tool call `{"name": "query", "arguments": {"query": "SELECT * FROM claroty_alerts LIMIT 5", "clients": ["demo-org-a"]}}` returns
    an MCP error response (not a data envelope) because Claroty is not registered for demo-org-a.
    The matcher MUST assert: (1) response has `"error"` field (not `"result"`); (2) error `code == -32602`;
    (3) error `message` contains `"E-QUERY-032"`; (4) message contains `"claroty"`; (5) message contains
    `"demo-org-a"`. See AC-012 for full matcher contract and rationale.
    Note: pass `clients: ["demo-org-a"]` (NOT `org_slug`) — see AC-012 for rationale.
18. **Implement** AC-013 assertion and document DTU-MULTI-001 comment per scope clarification.
19. **Implement** AQL push-down seeding (AC-014, D-934 scope, Mechanism B passthrough): in
    `prism-query` QueryEngine or the adapter dispatch layer, handle the `aql = '<string>'`
    pseudo-column pattern from the PrismQL AST. The canonical seeding site is the
    `predicate_tree_to_filter_map` / `extract_push_down_filters_as_map` path (which already
    extracts the `aql` key from the filter map per adversary finding F-DEMO002-P1-MED-002).
    Wire this path to populate `FetchContext.query_filters["aql"]` with the literal string value
    before dispatching to PipelineExecutor. Do NOT add a parallel `extract_aql_filter_value_from_ast`
    function — one seeding code path only (see Implementer Recommendation in adjudication report).
    Write a failing Red Gate test first: `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip`
    that drives `SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5` through the engine and asserts
    both (A) non-empty rows are returned (full pipeline: PQL parse → FilterMap → FetchContext →
    DTU `/api/v1/search?aql=in:devices`) and (B) the Armis DTU's `GET /dtu/aql-log` endpoint
    confirms the verbatim AQL "in:devices" was received (BC-2.11.007 Mechanism B; R-DTU-002;
    ADR-031 §D8-a). Read `crates/prism-query/src/` and
    `crates/prism-spec-engine/src/pipeline.rs` to confirm the seeding site.
20. **Run** `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only` after S-DEMO-001 merges; all assertions must pass GREEN locally. (Local invocation omits `--no-tests=fail` — the flag is only required in the CI job where a zero-test result indicates a test-selection regression. The CI job runs `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only --no-tests=fail` automatically via `.github/workflows/e2e.yml` on every PR and push to develop.)
21. **Run** `just check` — final pre-push gate.
22. **Write Red Gate test** `test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port`
    (AC-013): issues `query` tool calls for `demo-org-a` and `demo-org-c` (both have CrowdStrike);
    asserts both succeed and return data rows; includes `// DTU-MULTI-001:` comment per AC-013
    scope clarification. This is a SEPARATE test from Task 16 (adapter-count) — it tests runtime
    query execution, not boot-time registration. (F-PC-002 gap closed.)
23. **Write test** `test_EC_004_e2e_limit_zero_returns_empty_not_error` (EC-004): sends
    `SELECT * FROM crowdstrike_detections LIMIT 0` via `query` tool; asserts no error envelope and
    zero rows. (F-PC-002 gap closed.)
24. **Write test** `test_EC_005_e2e_limit_200_returns_paginated_rows` (EC-005): sends
    `SELECT * FROM crowdstrike_detections LIMIT 200` via `query` tool; asserts no error envelope and
    ≤200 rows (see EC-005 coverage decision for assertion caveat on fixture row count).
    Read `crates/prism-dtu-demo-server/` fixture data to confirm row count before asserting.
    (F-PC-002 gap closed.)
25. **Create** `.github/workflows/e2e.yml` (devops-engineer scope, in-scope for this PR per human
    decision 2026-06-03 absorbing S-DEMO-CI-E2E-001): the dedicated e2e CI workflow. Read
    `.github/workflows/ci.yml` in full to extract all action SHA pins verbatim. Canonical structure:
    - **job name:** `E2E smoke`
    - **runs-on:** `ubuntu-latest` (Linux runner; SIGTERM works on Linux; no macOS runner needed)
    - **timeout-minutes:** `45`
    - **triggers:** `pull_request` and `push` to `[develop, main]`, plus `workflow_dispatch`
      (no schedule/cron — PR+push gating is the contract; a scheduled drift-detection run can be
      a future maintenance PR; OBS-2 gap is closed by PR+push gating alone)
    - steps: checkout → rust-toolchain → setup-protoc → rust-cache (shared-key: `e2e-release`) →
      install nextest → `cargo build --release -p prism-bin -p prism-dtu-demo-server` →
      credential bootstrap (env-var shim via `prism-credentials` test-helpers feature, or
      Linux `secret-tool store` / env-var injection) → launch DTU + poll `urls.json` (30s timeout) →
      `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only --no-tests=fail` →
      upload JUnit XML on failure → DTU teardown (`if: always()`)
    - header comment: `# E2E Red Gate workflow — closes ADV-SDEMO002-PR-P02-HIGH-001 +
      OBS-2 (S-DEMO-002 LOCAL cascade 2026-06-02). Absorbed from S-DEMO-CI-E2E-001 per human
      decision 2026-06-03. Runs the #[ignore]'d E2E suite against a live DTU + release binary.`
    - DTU teardown step comment: `# OBS-2: DTU teardown must always run — zombie processes consume runner ports on retries.`

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

3. **Canonical Armis/Claroty/Cyberint table names (RESOLVED v1.7)**: Canonical Armis table names are `armis_devices` and `armis_alerts` (per `armis.sensor.toml` `table_name = "devices"` and `table_name = "alerts"` + `sensor_id = "armis"` prefix). **Critical: both Armis tables require an AQL predicate** — `SELECT * FROM armis_devices WHERE aql = 'in:devices' LIMIT 5` and `SELECT * FROM armis_alerts WHERE aql = 'in:alerts' LIMIT 5`. A bare `SELECT * FROM armis_devices LIMIT 5` (no AQL WHERE) will error because `path_template = "/api/v1/search?aql=${query.filter.aql}"` has no default (AC-004 rationale). Claroty: `claroty_alerts` + `claroty_devices`. Cyberint: `cyberint_alerts`. CrowdStrike: `crowdstrike_detections`. All queries use SQL form `SELECT * FROM <source> LIMIT N` — the bare `FROM ... LIMIT N` form is invalid PrismQL (pipe mode requires `|` before `LIMIT`). Implementer must confirm Cyberint/CrowdStrike names from their TOML specs; Armis and Claroty are confirmed.

4. **Credential bootstrap mechanism**: `prism-credentials` has a `test-helpers` feature that may expose a direct keyring write function. If that feature exists, use it. If not, the test should spawn a `security add-generic-password` (macOS) or `secret-tool store` (Linux) subprocess. Confirm the correct mechanism by reading `crates/prism-credentials/src/lib.rs`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU server fails to start (port conflict) | Test fails with clear message: "DTU server did not write urls.json within 30s" |
| EC-002 | prism-bin exits before MCP handshake completes | Test fails with clear message: "prism-bin exited unexpectedly with code N"; SubprocessGuard teardown logs stderr for diagnosis |
| EC-003 | A sensor query returns zero rows (DTU returned empty) | Test fails with AC assertion: "expected at least 1 row"; this is a data fidelity issue in the DTU clone, not a framework issue |
| EC-004 | `LIMIT 0` query variant | An additional edge-case test verifies that `LIMIT 0` returns empty-but-not-error response (E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation). **Coverage decision (F-PC-002 — in scope):** Implementer MUST write `test_EC_004_e2e_limit_zero_returns_empty_not_error` — sends `{"name": "query", "arguments": {"query": "SELECT * FROM crowdstrike_detections LIMIT 0"}}` and asserts: response is not an error envelope; `rows` array is empty (0 rows); `is_truncated: false` or `total_available: 0`. |
| EC-005 | `LIMIT 200` query variant | An additional edge-case test verifies that `LIMIT 200` triggers pagination in the DTU clone and returns 200 rows (exercises pagination at least one extra page per E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation). **Coverage decision (F-PC-002 — in scope):** Implementer MUST write `test_EC_005_e2e_limit_200_returns_paginated_rows` — sends `{"name": "query", "arguments": {"query": "SELECT * FROM crowdstrike_detections LIMIT 200"}}` and asserts: response is not an error envelope; `rows` array contains ≤200 rows (assuming the CrowdStrike DTU clone fixture has ≥1 row — if it has fewer than 200, assert `len(rows) == fixture_row_count` and document this in a comment; do NOT assert exactly 200 if fixture data is smaller). Implementer must verify DTU fixture row count before writing the assertion. |
| EC-006 | Org registered with zero sensors (all overlays missing) | Boot step 9A produces 0 adapters for that org; the org entry itself remains in `OrgRegistry`; no error; org-specific query returns `AdapterNotFound` |
| EC-007 | Two orgs both have CrowdStrike; same DTU clone port in both overlays | Both orgs' CrowdStrike queries succeed and return identical fixture data (DTU single-tenant mode — DTU-MULTI-001 scope); no cross-org data is modified |
| EC-008 | Org A queries sensor registered for Org C but not Org A | Returns `AdapterNotFound` error envelope; Org C's adapter is not accessible from Org A's call context per BC-3.2.001 |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| BC files (8 BCs: BC-2.11.001, BC-2.11.005, BC-2.09.008, BC-2.10.001, BC-2.10.010, BC-3.2.001, BC-2.22.001, BC-2.11.007) | ~12,800 |
| prism-dtu-demo-server/src/main.rs + harness.rs | ~6,000 |
| crates/prism-sensors/specs/ (4 TOML files, table names) | ~4,000 |
| crates/prism-bin/src/boot.rs (CLI invocation) | ~4,000 |
| S-DEMO-001 story (dependency context) | ~4,000 |
| S-5.01-FOLLOWUP-MCP-BOOT story (rmcp version context) | ~3,000 |
| Test output during iteration | ~3,000 |
| `prism-query/src/` + `prism-spec-engine/src/pipeline.rs` (AQL seeding site) | ~3,500 |
| `crates/prism-credentials/src/` (credential bootstrap helper surface) | ~1,200 |
| `crates/prism-core/src/error.rs` (E-QUERY-032 variant) | ~500 |
| `crates/prism-mcp/src/error_mapping.rs` + `safety_envelope.rs` (AC-012 error surface + AC-007 injection scan) | ~1,800 |
| `crates/prism-spec-engine/src/plugin/` + `plugin_auth_provider.rs` + `interpolation.rs` (WASM nested-interface dispatch) | ~2,500 |
| `crates/prism-dtu-armis/src/routes/search.rs` + `crates/prism-dtu-cyberint/src/clone.rs` (DTU SRC changes) | ~1,500 |
| `crates/prism-dtu-demo-server/src/config.rs` + `harness.rs` (multi-clone extensions) | ~1,000 |
| **Total estimate** | **~53,300 tokens (~21% of 256K context)** |

Within budget; additional crate context (+8,300 tokens) from HIGH-001 reconciliation remains comfortably below 25% threshold.

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
| 2.7 | 2026-07-13 | story-writer | DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P1-HIGH-001 residual — retire composed string `"Internal error; see audit log"` in AC-012 rationale block. Updated to BC-2.10.007 message/suggestion split contract: caller-visible `message` is `"Internal error"`; `suggestion` carries `"See audit log for details."` (error-taxonomy v2.40). TD-VSDD-060 sibling sweep confirmed no other live occurrences of the retired string in this file (v1.6 changelog row at former line 704 already used the abbreviated `"Internal error"` form only — historical context, not a live citation). All version pins updated: frontmatter version 2.6→2.7, frontmatter modified, body **Version:** block. |
| 2.6 | 2026-06-03 | product-owner | ADV-SDEMO002-PR-P11-MED-001 closure: body `**Version:**` block synced (v2.4→v2.6); recurring sibling-pin drift (2nd occurrence, P04+P11) — all version pins (frontmatter, H1, body block, modified) now verified consistent |
| 2.5 | 2026-06-03 | product-owner | Fix-burst closing ADV-SDEMO002-PR-P10-MED-001 (prism-query had no FSR row despite being in crates_touched with load-bearing SRC changes). Added FSR row: `crates/prism-query/src/engine.rs, src/materialization.rs` MODIFY — E-QUERY-032 cross-org isolation raise + AQL push-down seeding (BC-3.2.001 postcondition 5, AC-014 / BC-2.11.007 Mechanism B). Full FSR/Token-Budget/crates_touched bidirectional cross-check performed against all 10 crates; zero additional asymmetries found (see 10-row checklist in commit message). Token Budget total unchanged at ~53,300 tokens (prism-query was already covered by the "prism-query/src/ + prism-spec-engine/src/pipeline.rs" ~3,500 row). |
| 2.4 | 2026-06-03 | product-owner | Fix-burst closing ADV-SDEMO002-PR-P08-HIGH-001 (crates_touched was incomplete — reconciled to actual 10-crate diff set: prism-bin, prism-core, prism-credentials, prism-dtu-armis, prism-dtu-cyberint, prism-dtu-demo-server, prism-mcp, prism-query, prism-sensors, prism-spec-engine; verified via `git diff cd4a2211...81cf3678 --stat -- crates/<name>/`; FSR rows and Token Budget rows added for 6 newly documented crates; per-crate SRC/TEST/CONFIG tags added to crates_touched comments). Closes ADV-SDEMO002-PR-P08-LOW-001 (phantom path `prism-spec-engine/src/pipeline_executor.rs` → `prism-spec-engine/src/pipeline.rs` corrected in frontmatter inputs, Task 19, and Token Budget). Closes ADV-SDEMO002-PR-P08-LOW-002 (prism-spec-engine confirmed genuinely touched via WASM acquire-token nested-interface dispatch in `src/plugin/{mod,discovery}.rs`, `src/plugin_auth_provider.rs`, `src/spec_parser.rs`; kept in crates_touched with SRC tag). Token Budget total revised ~45,000 → ~53,300 tokens (~21% of 256K). |
| 2.3 | 2026-06-03 | product-owner | Closes ADV-SDEMO002-PR-P05-OBS-001 (e2e CI positive-coverage guard). AC-010 canonical command updated: `--no-tests=fail` appended to the CI job command (`cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only --no-tests=fail`); purpose documented inline — job fails on zero selected tests, preventing a false-green if `#[ignore]` attrs are removed or the test file is renamed. Task 25 FSR and step list updated to match. Task 20 clarifies local invocation omits the guard (CI-only flag). |
| 2.2 | 2026-06-03 | product-owner | Fix-burst closing ADV-SDEMO002-PR-P04-MED-001 (body H1 `**Points:** 11` → `**Points:** 13`; frontmatter `points: 13` was correct since v2.0 but body H1 pin was not updated). Comprehensive body-pin sibling sweep (TD-VSDD-060): body H1 Version updated v2.1→v2.2; all other body pins (Status, Wave, Priority, Story ID, acceptance_criteria_count 14, red_gate_tests 9, behavioral_contracts 8, crates_touched 4) verified correct against frontmatter — no additional corrections required. |
| 2.1 | 2026-06-03 | product-owner | Fix-burst closing ADV-SDEMO002-PR-P03-HIGH-001/MED-001/MED-002 + OBS-001 + AC-009/retries reconcile. (1) HIGH-001/OBS-001: AC-010 Then-clause rewritten to match canonical e2e.yml spec exactly — job name "E2E smoke", runs-on `ubuntu-latest`, timeout-minutes 45, triggers `pull_request`+`push`[develop,main]+`workflow_dispatch` (no cron), release build, canonical command `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only`. Task 25 updated: `macos-latest`→`ubuntu-latest`, `timeout-minutes: 30`→`45`, trigger set corrected (add `main` + `workflow_dispatch`, drop schedule); job name set to "E2E smoke". Changelog v2.0 narrative corrected. (2) MED-002: `run-ignored = "all"` profile key struck from AC-010 profile description and FSR `.config/nextest.toml` row — nextest does NOT support this key; un-ignoring is via CLI flag `--run-ignored ignored-only` only; both locations now state this explicitly. (3) MED-001 (POL-29 step 3d): `prism-credentials` added to frontmatter `crates_touched` array; FSR row added for `crates/prism-credentials/src/`; Token Budget row added (~1,200 tokens); frontmatter comment updated. |
| 2.0 | 2026-06-03 | product-owner | Reconcile AC-010 to delivered e2e CI job; disposition S-DEMO-CI-E2E-001 (ADV-SDEMO002-PR-P02-HIGH-001). AC-010 rewritten: the `#[ignore]` gate is now paired with a described dedicated GitHub Actions workflow (`.github/workflows/e2e.yml`) that triggers on `pull_request` + `push` to `develop`, builds release binaries, launches DTU, and runs `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only` automatically. AC-009 updated: CI job replaces the prior vague "CI e2e profile job" reference with a concrete citation to `e2e.yml`. File Structure Requirements table updated: `.config/nextest.toml` clarified with `run-ignored / slow-timeout / retries` settings; `.github/workflows/e2e.yml` added as a CREATE deliverable of this PR. Task 25 added: devops-engineer scope for creating `e2e.yml`, absorbed from S-DEMO-CI-E2E-001 per human decision 2026-06-03. S-DEMO-CI-E2E-001 superseded (see that story's v1.1 changelog). |
| 1.9 | 2026-06-03 | story-writer | SPEC fix — human-authorized spec amendment per Source-of-Truth §7 (code canonical for query syntax). Closes ADV-SDEMO002-P01-MED-001: all bare `FROM <source> LIMIT N` query strings in ACs/Tasks/Open-Questions replaced with SQL form `SELECT * FROM <source> LIMIT N` to match the working code in `crates/prism-bin/tests/e2e_smoke.rs` (bare-FROM is invalid PrismQL pipe syntax; parser requires `\|` before `LIMIT` in pipe mode). Affected locations: AC-003, AC-005 (claroty_alerts + claroty_devices), AC-006, AC-012, AC-013 (two instances), Task 11, Task 17, Open Question 3. AC-004 EXCEPTION: `WHERE aql = 'in:devices' LIMIT 5` form retained verbatim (D-963 Option-A locked decision); SQL form already in place from v1.7. Closes ADV-SDEMO002-P01-MED-002: Task 9 `prism-bin start --config <temp_dir>` corrected to `prism-bin start --config-dir <temp_dir>` (clap CLI declares `--config-dir`; confirmed in `helpers/mod.rs` line 976 `.arg("--config-dir")`). Task 8 (`prism-dtu-demo-server start --config`) LEFT UNCHANGED — DTU binary genuinely uses `--config` (confirmed in `prism-dtu-demo-server/src/main.rs` line 37: `#[arg(long, short = 'c'` named `config`). |
| 1.8 | 2026-06-02 | product-owner | SPEC fix — comprehensive prose audit + F-DEMO002-P3-MED-001 closure. (1) EC-004: `events` field corrected to `rows` (non-existent `events` field; actual ResponseEnvelope payload field is `rows` per `server.rs:1322` `"rows": rows`; envelope keys are `rows / returned_results / total_available / is_truncated`). (2) EC-005: both `events` occurrences corrected to `rows`; assertion description aligned to `≤200 rows` (consistent with as-built test `rows.len() <= 200`). (3) AC-007 "What `wrap()` scans" block corrected: prose previously claimed `results` is a "JSON Array (not Object)" for the query tool — as-built `safety_envelope.rs` shows `results` is an Object `{"rows": [...], "returned_results": N, "total_available": N, "is_truncated": bool}`; `wrap()` extracts the inner `rows` array via `results.get("rows")` for scanning. Prose rewritten to match actual code path. (4) Task 19 test name corrected: `test_BC_2_11_007_e2e_armis_aql_pushdown_seeded_in_fetch_context` → `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip` (as-built name in `e2e_smoke.rs`); description updated to match the as-built dual-assertion pattern (rows non-empty + DTU aql-log verification). COORDINATION NOTE: implementer separately directed to make `e2e_smoke.rs` doc-comment story reference version-agnostic (`Story: S-DEMO-002` with no version) per TD-VSDD-091 anti-volatile-pin — no code change required from this burst. |
| 1.7 | 2026-06-02 | product-owner | SPEC fix — LOCAL adversarial CRIT F-DEMO002-P2-CRIT-001 closure. Human decision: Option A — faithful to real Armis API; require AQL predicate. (1) AC-004 rewritten: query changed from bare `FROM armis_devices LIMIT 5` to `FROM armis_devices WHERE aql = 'in:devices' LIMIT 5`; added mandatory-AQL rationale block citing `armis.sensor.toml` path_template `"/api/v1/search?aql=${query.filter.aql}"` (no default), real Armis `/api/v1/search` API contract, and AC-014 Mechanism B convention; noted same requirement applies to `armis_alerts` table. (2) Task 11 updated: Armis query must include AQL predicate. (3) Open Question 3 resolved: Armis table names confirmed (`armis_devices`, `armis_alerts`); AQL-mandatory calling convention documented. AC-011 unchanged — its Armis verification is boot-time adapter registration (adapter count = 8), not query execution; no AQL predicate required. Rationale note: this aligns S-DEMO-002 to the merged Armis spec (armis.sensor.toml v1.0.0) per Source-of-Truth precedence (story spec defers to BC/TOML spec on contract semantics); the demo demonstrates the seeded-AQL path, faithful to the real Armis Centrix API. |
| 1.6 | 2026-06-02 | product-owner | SPEC-EVOLUTION burst — LOCAL adversarial CRIT-001 / MED-002 / LOW-001 closures. (1) CRIT-001: AC-012 matcher contract replaced — `response_has_adapter_not_found_error` pattern retired; new matcher asserts MCP error code `-32602` + message containing `"E-QUERY-032"`, `"claroty"`, `"demo-org-a"`. Red Gate test renamed: `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032`. Rationale: `map_prism_error` redacts all E-SENSOR-* to "Internal error" (AD-017); AC-012 could never pass with the old E-SENSOR-010 substring match. New E-QUERY-032 is credential-safe (org slug + sensor type only) and surfaces as -32602. Companion changes: error-taxonomy.md v1.58 (E-QUERY-032 definition), BC-3.2.001 v0.7 (postcondition 5, EC-006/007, TV-3.2.001-06). (2) MED-002: AC-007 injection-scan assertion adjudicated — assertion `safety_flags == []` is meaningful ONLY when `results` is a non-empty array; test must verify at least 1 row returned. Added "What `wrap()` scans" narrative clarifying `collect_string_fields` is invoked for Array results (query tool rows), NOT for Object results (explain/alias responses). Implementation note added for BC-2.09.008 v1.x follow-up (object-shape scanning is pre-existing out-of-scope per `wrap()` doc comment SS-09 hardening section). Code change to `wrap()` is implementer scope. (3) LOW-001: FSR `.cargo/nextest.toml` corrected to `.config/nextest.toml` (actual on-disk path). Task 14 updated to match. |
| 1.5 | 2026-06-02 | product-owner | Adversarial reconciliation (POL-27, POL-32). (1) F-*-CRIT-001 — all `tool_query` references replaced with canonical tool name `query` (BC-2.11.001 H1 source-of-truth): AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-012, AC-013, AC-014, Task 10, Task 17, Narrative, frontmatter comments. (2) F-*-CRIT-002 — AC-012 and Task 17 replace `org_slug="demo-org-a"` tool arg with canonical `clients: ["demo-org-a"]` per BC-2.11.001 Preconditions and `QueryToolParams` `#[serde(deny_unknown_fields)]` constraint; rationale note added inline. (3) F-*-MED — AC-014 seeding mechanism accuracy: added "Seeding mechanism accuracy" paragraph clarifying that INDEX column option is decorative (not a runtime gate today); generic `predicate_tree_to_filter_map` path extracts all `field='string'` equality predicates regardless of INDEX; implementer must NOT add `if column_is_index` branch. (4) F-PC-002 — coverage decisions for AC-009 (CI repetition, no new #[test]), AC-013 (new test Task 22: `test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port`), EC-004 (new test Task 23: `test_EC_004_e2e_limit_zero_returns_empty_not_error`), EC-005 (new test Task 24: `test_EC_005_e2e_limit_200_returns_paginated_rows`); red_gate_tests 5→9. (5) F-PB-MED-002 — FSR corrected: `crates/prism-bin/fixtures/e2e-demo/demo.toml` (not `tests/fixtures/demo.toml`); phantom `demo-prism.toml.template` removed (programmatic generation via `write_demo_config()`); Task 8 path updated. |
| 1.4 | 2026-06-02 | product-owner | F-DEMO002-P1-MED-002 adjudication (POL-4 semantic drift). AC-014 rewritten to reflect BC-2.11.007 v1.5 Mechanism B (Verbatim-AQL Passthrough): user writes `aql = '<string>'` pseudo-column in PrismQL WHERE (not raw AQL syntax); query planner seeds verbatim string into FetchContext.query_filters["aql"]; forwarded opaque to DTU per R-DTU-002 / ADR-031 §D8-a. Task 19 updated: canonical seeding site is predicate_tree_to_filter_map / extract_push_down_filters_as_map path; explicitly prohibits parallel extract_aql_filter_value_from_ast function (one code path only per implementer recommendation). Story H1 version block updated v1.3→v1.4. |
| 1.3 | 2026-06-02 | product-owner | Readiness flip draft→ready. (1) BC-2.22.001 (Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate) added to behavioral_contracts frontmatter + BC body table — AC-001/AC-010/AC-011 already traced to it; gap closed per POL-8. (2) BC-2.11.007 (Sensor Filter Push-Down) added to behavioral_contracts frontmatter — required by AC-014 AQL push-down seeding. (3) AC-014 added: end-to-end AQL push-down seeding — prism-query QueryEngine populates FetchContext.query_filters["aql"] from PrismQL WHERE predicate for Armis; D-934 confirmed scope boundary (architect+implementer). Task 19 added (AQL seeding implementation + Red Gate test). acceptance_criteria_count 13→14. Token budget updated: 8 BCs / +3,500 tokens for prism-query seeding site / total ~43,800. |
| 1.2 | 2026-05-29 | architect | AC-003: assert `detection_id` column (not `id`) per Gap-CS-001 TOML fix. AC-005: corrected query from `claroty_assets` (table does not exist) to `claroty_alerts`; added `claroty_devices` query per Gap-CL-003 fix; asserted `alert_type_name`/`detected_time` column names per Gap-CL-005 fix; noted Gap-CL-004 single-page limitation. Story bumped to v1.2 from v1.1. |
| 1.1 | 2026-05-29 | architect | Multi-org isolation scope added: AC-011 (3-org registration + 8-adapter count), AC-012 (cross-org AdapterNotFound isolation probe), AC-013 (DTU multi-tenant scope clarification + DTU-MULTI-001 comment requirement). BC-3.2.001 added to behavioral_contracts. Points 8→11 (+3 pts for multi-org ACs). acceptance_criteria_count 10→13. red_gate_tests 4→5. EC-006..008 added. Title updated to reflect multi-org scope. |
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
