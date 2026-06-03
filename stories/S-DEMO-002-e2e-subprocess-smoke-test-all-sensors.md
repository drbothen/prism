---
document_type: story
story_id: S-DEMO-002
title: "prism-bin: E2E Subprocess Smoke Test — All 4 Sensors + Multi-Org Isolation via DTU Clones + MCP Round-Trip"
wave: 5
epic_id: E-DEMO
priority: P0
status: ready
version: "1.7"
level: "L4"
producer: story-writer
timestamp: "2026-06-02T00:00:00Z"
modified: "2026-06-02T00:00:00Z"
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
crates_touched: [prism-bin, prism-query, prism-spec-engine]
# crates_touched notes:
#   prism-bin: integration test file (tests/e2e_smoke.rs + helpers) + binary entrypoint
#   prism-query: AC-014 AQL push-down seeding — QueryEngine must populate
#                FetchContext.query_filters["aql"] from PrismQL WHERE predicates (D-934 scope)
#   prism-spec-engine: PipelineExecutor receives FetchContext; may need seeding if the
#                      query_filters plumbing runs through this crate rather than prism-query
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
points: 11
# Points justification (revised v1.0→v1.1: +3 pts for multi-org isolation scope):
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
#   Total: 11 points (~2-2.5 days)
estimated_days: 2
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
  - "crates/prism-spec-engine/src/pipeline_executor.rs"
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

# S-DEMO-002 v1.7 — prism-bin: E2E Subprocess Smoke Test (All 4 Sensors + Multi-Org Isolation)

**Story ID:** S-DEMO-002
**Status:** ready
**Version:** v1.7
**Wave:** 5
**Priority:** P0
**Points:** 11

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
When: The test sends `tools/call` with tool name `query` and input `"FROM crowdstrike_detections LIMIT 5"` (i.e., `{"name": "query", "arguments": {"query": "FROM crowdstrike_detections LIMIT 5"}}`).
Then: The ResponseEnvelope contains at least 1 row of data; the `category_uid` and `class_uid`
fields are present and non-null; the `detection_id` column (the primary key per Gap-CS-001 fix —
NOT `id`) is present and non-null for each row; no error code in the response.
(traces to BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF via the OCSF normalizer")
Red Gate test: `test_BC_2_11_005_e2e_crowdstrike_query_returns_ocsf_data`

### AC-004: Armis query with AQL predicate returns non-empty Arrow batches
Given: Demo-org's Armis sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input
`"FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"`
(call shape: `{"name": "query", "arguments": {"query": "FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"}}`).
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
When: The test sends `tools/call` with tool name `query` and input `"FROM claroty_alerts LIMIT 5"` (call shape: `{"name": "query", "arguments": {"query": "..."}}`).
Then: The ResponseEnvelope contains at least 1 row; the `alert_type_name` column (not `type`)
and `detected_time` column (not `created_at`) are present per Gap-CL-005 TOML fix; no error code.
Also test `"FROM claroty_devices LIMIT 5"` — expects at least 1 row with `uid` column present
(Gap-CL-003 fix — devices table added to claroty.sensor.toml).
Note: queries beyond page 1 are limited by Gap-CL-004 (offset pagination sent as URL params, not
POST body) — assert only for page-1 result rows (≤ page_size=100).
(traces to BC-2.11.005 postcondition: same as AC-003 for Claroty sensor)

### AC-006: Cyberint query returns non-empty Arrow batches
Given: Demo-org's Cyberint sensor spec has `base_url` overlaid to point at the DTU clone.
When: The test sends `tools/call` with tool name `query` and input `"FROM cyberint_alerts LIMIT 5"`
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
least 1 row (so `results` is a non-empty array) and then assert `safety_flags == []`. This
verifies that `SafetyEnvelopeBuilder::wrap()` ran the injection scanner over the row data
(`results` array items) and found no patterns in the synthetic DTU fixture data.

**What `wrap()` scans:** `SafetyEnvelopeBuilder::wrap()` calls `collect_string_fields` only when
`results` is a JSON Array (not Object). For the `query` tool, `results` is the Arrow-serialized
rows array — each element is an object with sensor field values. The scanner recurses into each
row object via `collect_string_fields` (IMP-6 fix, MAX_SCAN_DEPTH=64). The assertion
`safety_flags == []` is meaningful only when `results` contains at least 1 row; with 0 rows the
array is empty and the scan trivially produces no flags regardless of implementation correctness.

**Implementation note for BC-2.09.008 v1.x follow-up:** when `results` is a JSON Object (not
Array) — e.g., for `explain_query`, `create_alias` tool responses — `wrap()` does NOT invoke
`collect_string_fields` and `safety_flags` is always empty (pre-existing scope-limitation
documented in the `wrap()` doc comment as "SS-09 hardening follow-up"). That case is NOT what
AC-007 exercises; AC-007 explicitly targets the `query` tool's array-shaped `results`.

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
running the full smoke test suite 5 consecutive times in the CI e2e profile job and confirming
zero failures. This is a CI-gate behavior, not a separate Rust `#[test]` function. The implementer
MUST document this in the PR description and confirm 5 consecutive local green runs before declaring
AC-009 satisfied. No additional `#[test]` function is required beyond the primary smoke test
functions — determinism is a *property* of the existing tests, verified by repetition.

### AC-010: Test is gated behind `#[ignore]` with explicit CI profile un-ignoring
Given: CI matrix runs standard nextest profile (no DTU server available).
When: Standard `cargo nextest run -p prism-bin` is executed.
Then: The E2E smoke test is skipped (marked `#[ignore]`). When the CI runs with
`cargo nextest run -p prism-bin --profile e2e` (a dedicated CI job), the test runs.
The `#[ignore]` annotation includes a code comment: `// E2E-001: requires DTU server
running; un-gated in CI via 'e2e' nextest profile.`
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
When: The test sends a `query` MCP tool call with `{"name": "query", "arguments": {"query": "FROM claroty_alerts LIMIT 5", "clients": ["demo-org-a"]}}`.
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
errors to "Internal error; see audit log" (AD-017 credential opacity). E-SENSOR-010 (AdapterNotFound)
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
When: The test issues `query` tool calls (`{"name": "query", "arguments": {"query": "FROM crowdstrike_detections LIMIT 5", "clients": ["<org_slug>"]}}`) for each of the 3 orgs.
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
tool call `{"name": "query", "arguments": {"query": "FROM crowdstrike_detections LIMIT 5", "clients": ["demo-org-a"]}}` and `{"clients": ["demo-org-c"]}` variants; asserts both succeed and
return data rows (verifying both orgs' CrowdStrike adapters are wired to the same DTU clone port);
includes `// DTU-MULTI-001:` comment per scope clarification above. This test is separate from
AC-011's adapter-count test (which verifies registration) — AC-013's test verifies query execution
reaches the correct port at runtime.

### AC-014: AQL filter seeded into FetchContext for Armis end-to-end push-down (Mechanism B passthrough)
Given: A `query` MCP tool call targets an Armis table using the verbatim-AQL pseudo-column convention
(e.g., `"FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"` — the user writes `aql = '<string>'`
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
| `.config/nextest.toml` | MODIFY | Add `[profile.e2e]` that un-ignores E2E-tagged tests |

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
9. **Implement** `launch_prism_bin()` — spawns `prism-bin start --config <temp_dir>` with stdin/stdout pipes; wraps stdio in a `rmcp` client or raw JSON-RPC writer; returns `SubprocessGuard` + IO handles.
10. **Implement** MCP handshake in test — send `initialize`, receive `initialized`, send `tools/list`, assert `query` present in the tools array (the canonical name registered by `PrismServer`; NOT `tool_query`).
11. **Implement** 4 × query assertions (AC-003..006) — each sends `tools/call` with appropriate query string; asserts non-empty data and OCSF fields. For Armis (AC-004), the query MUST supply the AQL predicate: `"FROM armis_devices WHERE aql = 'in:devices' LIMIT 5"` — bare `FROM armis_devices LIMIT 5` is invalid per AC-004.
12. **Implement** `_meta` assertions (AC-007) — parse `ResponseEnvelope` JSON; assert `trust_level` and `safety_flags`.
13. **Implement** SIGTERM teardown (AC-008) in `SubprocessGuard::drop()`.
14. **Add** `[profile.e2e]` to `.config/nextest.toml` — un-ignores tests tagged `// E2E-001:`. (Path is `.config/nextest.toml`, NOT `.cargo/nextest.toml` — LOW-001 corrected.)
15. **Implement** `write_multi_org_demo_config()` helper — generates 3-org config with overlapping
    and non-overlapping sensor sets (demo-org-a: CrowdStrike+Armis; demo-org-b: Claroty+Cyberint;
    demo-org-c: all 4). Writes per-org `customers/{slug}/` overlay directories for each sensor.
16. **Write Red Gate test** `test_BC_3_2_001_e2e_multi_org_boot_registers_correct_adapter_count`
    (AC-011): asserts `AdapterRegistry` contains exactly 8 entries after boot with 3-org config.
17. **Write Red Gate test** `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032`
    (AC-012): asserts that a `query` tool call `{"name": "query", "arguments": {"query": "FROM claroty_alerts LIMIT 5", "clients": ["demo-org-a"]}}` returns
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
    Write a failing Red Gate test first: `test_BC_2_11_007_e2e_armis_aql_pushdown_seeded_in_fetch_context`
    that drives `FROM armis_devices WHERE aql = 'in:devices' LIMIT 5` through the engine and asserts
    `FetchContext.query_filters["aql"] == "in:devices"` and that the DTU receives the correct
    `GET /api/v1/search?aql=in:devices` call. Read `crates/prism-query/src/` and
    `crates/prism-spec-engine/src/pipeline_executor.rs` to confirm the seeding site.
20. **Run** `cargo nextest run -p prism-bin --profile e2e` after S-DEMO-001 merges; all assertions must pass GREEN.
21. **Run** `just check` — final pre-push gate.
22. **Write Red Gate test** `test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port`
    (AC-013): issues `query` tool calls for `demo-org-a` and `demo-org-c` (both have CrowdStrike);
    asserts both succeed and return data rows; includes `// DTU-MULTI-001:` comment per AC-013
    scope clarification. This is a SEPARATE test from Task 16 (adapter-count) — it tests runtime
    query execution, not boot-time registration. (F-PC-002 gap closed.)
23. **Write test** `test_EC_004_e2e_limit_zero_returns_empty_not_error` (EC-004): sends
    `FROM crowdstrike_detections LIMIT 0` via `query` tool; asserts no error envelope and
    zero rows. (F-PC-002 gap closed.)
24. **Write test** `test_EC_005_e2e_limit_200_returns_paginated_rows` (EC-005): sends
    `FROM crowdstrike_detections LIMIT 200` via `query` tool; asserts no error envelope and
    ≤200 rows (see EC-005 coverage decision for assertion caveat on fixture row count).
    Read `crates/prism-dtu-demo-server/` fixture data to confirm row count before asserting.
    (F-PC-002 gap closed.)

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

3. **Canonical Armis/Claroty/Cyberint table names (RESOLVED v1.7)**: Canonical Armis table names are `armis_devices` and `armis_alerts` (per `armis.sensor.toml` `table_name = "devices"` and `table_name = "alerts"` + `sensor_id = "armis"` prefix). **Critical: both Armis tables require an AQL predicate** — `FROM armis_devices WHERE aql = 'in:devices'` and `FROM armis_alerts WHERE aql = 'in:alerts'`. A bare `FROM armis_devices LIMIT 5` (no AQL WHERE) will error because `path_template = "/api/v1/search?aql=${query.filter.aql}"` has no default (AC-004 rationale). Claroty: `claroty_alerts` + `claroty_devices`. Cyberint: `cyberint_alerts`. CrowdStrike: `crowdstrike_detections`. Implementer must confirm Cyberint/CrowdStrike names from their TOML specs; Armis and Claroty are confirmed.

4. **Credential bootstrap mechanism**: `prism-credentials` has a `test-helpers` feature that may expose a direct keyring write function. If that feature exists, use it. If not, the test should spawn a `security add-generic-password` (macOS) or `secret-tool store` (Linux) subprocess. Confirm the correct mechanism by reading `crates/prism-credentials/src/lib.rs`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU server fails to start (port conflict) | Test fails with clear message: "DTU server did not write urls.json within 30s" |
| EC-002 | prism-bin exits before MCP handshake completes | Test fails with clear message: "prism-bin exited unexpectedly with code N"; SubprocessGuard teardown logs stderr for diagnosis |
| EC-003 | A sensor query returns zero rows (DTU returned empty) | Test fails with AC assertion: "expected at least 1 row"; this is a data fidelity issue in the DTU clone, not a framework issue |
| EC-004 | `LIMIT 0` query variant | An additional edge-case test verifies that `LIMIT 0` returns empty-but-not-error response (E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation). **Coverage decision (F-PC-002 — in scope):** Implementer MUST write `test_EC_004_e2e_limit_zero_returns_empty_not_error` — sends `{"name": "query", "arguments": {"query": "FROM crowdstrike_detections LIMIT 0"}}` and asserts: response is not an error envelope; `events` array is empty (0 rows); `is_truncated: false` or `total_available: 0`. |
| EC-005 | `LIMIT 200` query variant | An additional edge-case test verifies that `LIMIT 200` triggers pagination in the DTU clone and returns 200 rows (exercises pagination at least one extra page per E2E-DEMO-WIRING-PLAN §6 Risk 3 mitigation). **Coverage decision (F-PC-002 — in scope):** Implementer MUST write `test_EC_005_e2e_limit_200_returns_paginated_rows` — sends `{"name": "query", "arguments": {"query": "FROM crowdstrike_detections LIMIT 200"}}` and asserts: response is not an error envelope; `events` array contains 200 rows (assuming the CrowdStrike DTU clone fixture has ≥200 rows — if it has fewer, assert `len(events) == fixture_row_count` and document this in a comment; do NOT assert exactly 200 if fixture data is smaller). Implementer must verify DTU fixture row count before writing the assertion. |
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
| `prism-query/src/` + `prism-spec-engine/src/pipeline_executor.rs` (AQL seeding site) | ~3,500 |
| **Total estimate** | **~43,800 tokens (~17% of 256K context)** |

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
| 1.7 | 2026-06-02 | product-owner | SPEC fix — LOCAL adversarial CRIT F-DEMO002-P2-CRIT-001 closure. Human decision: Option A — faithful to real Armis API; require AQL predicate. (1) AC-004 rewritten: query changed from bare `FROM armis_devices LIMIT 5` to `FROM armis_devices WHERE aql = 'in:devices' LIMIT 5`; added mandatory-AQL rationale block citing `armis.sensor.toml` path_template `"/api/v1/search?aql=${query.filter.aql}"` (no default), real Armis `/api/v1/search` API contract, and AC-014 Mechanism B convention; noted same requirement applies to `armis_alerts` table. (2) Task 11 updated: Armis query must include AQL predicate. (3) Open Question 3 resolved: Armis table names confirmed (`armis_devices`, `armis_alerts`); AQL-mandatory calling convention documented. AC-011 unchanged — its Armis verification is boot-time adapter registration (adapter count = 8), not query execution; no AQL predicate required. Rationale note: this aligns S-DEMO-002 to the merged Armis spec (armis.sensor.toml v1.0.0) per Source-of-Truth precedence (story spec defers to BC/TOML spec on contract semantics); the demo demonstrates the seeded-AQL path, faithful to the real Armis Centrix API. |
| 1.6 | 2026-06-02 | product-owner | SPEC-EVOLUTION burst — LOCAL adversarial CRIT-001 / MED-002 / LOW-001 closures. (1) CRIT-001: AC-012 matcher contract replaced — `response_has_adapter_not_found_error` pattern retired; new matcher asserts MCP error code `-32602` + message containing `"E-QUERY-032"`, `"claroty"`, `"demo-org-a"`. Red Gate test renamed: `test_BC_3_2_001_e2e_cross_org_sensor_query_returns_e_query_032`. Rationale: `map_prism_error` redacts all E-SENSOR-* to "Internal error" (AD-017); AC-012 could never pass with the old E-SENSOR-010 substring match. New E-QUERY-032 is credential-safe (org slug + sensor type only) and surfaces as -32602. Companion changes: error-taxonomy.md v1.58 (E-QUERY-032 definition), BC-3.2.001 v0.7 (postcondition 5, EC-006/007, TV-3.2.001-06). (2) MED-002: AC-007 injection-scan assertion adjudicated — assertion `safety_flags == []` is meaningful ONLY when `results` is a non-empty array; test must verify at least 1 row returned. Added "What `wrap()` scans" narrative clarifying `collect_string_fields` is invoked for Array results (query tool rows), NOT for Object results (explain/alias responses). Implementation note added for BC-2.09.008 v1.x follow-up (object-shape scanning is pre-existing out-of-scope per `wrap()` doc comment SS-09 hardening section). Code change to `wrap()` is implementer scope. (3) LOW-001: FSR `.cargo/nextest.toml` corrected to `.config/nextest.toml` (actual on-disk path). Task 14 updated to match. |
| 1.5 | 2026-06-02 | product-owner | Adversarial reconciliation (POL-27, POL-32). (1) F-*-CRIT-001 — all `tool_query` references replaced with canonical tool name `query` (BC-2.11.001 H1 source-of-truth): AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-012, AC-013, AC-014, Task 10, Task 17, Narrative, frontmatter comments. (2) F-*-CRIT-002 — AC-012 and Task 17 replace `org_slug="demo-org-a"` tool arg with canonical `clients: ["demo-org-a"]` per BC-2.11.001 Preconditions and `QueryToolParams` `#[serde(deny_unknown_fields)]` constraint; rationale note added inline. (3) F-*-MED — AC-014 seeding mechanism accuracy: added "Seeding mechanism accuracy" paragraph clarifying that INDEX column option is decorative (not a runtime gate today); generic `predicate_tree_to_filter_map` path extracts all `field='string'` equality predicates regardless of INDEX; implementer must NOT add `if column_is_index` branch. (4) F-PC-002 — coverage decisions for AC-009 (CI repetition, no new #[test]), AC-013 (new test Task 22: `test_BC_3_2_001_e2e_dtu_multi_tenant_each_org_reaches_correct_clone_port`), EC-004 (new test Task 23: `test_EC_004_e2e_limit_zero_returns_empty_not_error`), EC-005 (new test Task 24: `test_EC_005_e2e_limit_200_returns_paginated_rows`); red_gate_tests 5→9. (5) F-PB-MED-002 — FSR corrected: `crates/prism-bin/fixtures/e2e-demo/demo.toml` (not `tests/fixtures/demo.toml`); phantom `demo-prism.toml.template` removed (programmatic generation via `write_demo_config()`); Task 8 path updated. |
| 1.4 | 2026-06-02 | product-owner | F-DEMO002-P1-MED-002 adjudication (POL-4 semantic drift). AC-014 rewritten to reflect BC-2.11.007 v1.5 Mechanism B (Verbatim-AQL Passthrough): user writes `aql = '<string>'` pseudo-column in PrismQL WHERE (not raw AQL syntax); query planner seeds verbatim string into FetchContext.query_filters["aql"]; forwarded opaque to DTU per R-DTU-002 / ADR-031 §D8-a. Task 19 updated: canonical seeding site is predicate_tree_to_filter_map / extract_push_down_filters_as_map path; explicitly prohibits parallel extract_aql_filter_value_from_ast function (one code path only per implementer recommendation). Story H1 version block updated v1.3→v1.4. |
| 1.3 | 2026-06-02 | product-owner | Readiness flip draft→ready. (1) BC-2.22.001 (Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate) added to behavioral_contracts frontmatter + BC body table — AC-001/AC-010/AC-011 already traced to it; gap closed per POL-8. (2) BC-2.11.007 (Sensor Filter Push-Down) added to behavioral_contracts frontmatter — required by AC-014 AQL push-down seeding. (3) AC-014 added: end-to-end AQL push-down seeding — prism-query QueryEngine populates FetchContext.query_filters["aql"] from PrismQL WHERE predicate for Armis; D-934 confirmed scope boundary (architect+implementer). Task 19 added (AQL seeding implementation + Red Gate test). acceptance_criteria_count 13→14. Token budget updated: 8 BCs / +3,500 tokens for prism-query seeding site / total ~43,800. |
| 1.2 | 2026-05-29 | architect | AC-003: assert `detection_id` column (not `id`) per Gap-CS-001 TOML fix. AC-005: corrected query from `claroty_assets` (table does not exist) to `claroty_alerts`; added `claroty_devices` query per Gap-CL-003 fix; asserted `alert_type_name`/`detected_time` column names per Gap-CL-005 fix; noted Gap-CL-004 single-page limitation. Story bumped to v1.2 from v1.1. |
| 1.1 | 2026-05-29 | architect | Multi-org isolation scope added: AC-011 (3-org registration + 8-adapter count), AC-012 (cross-org AdapterNotFound isolation probe), AC-013 (DTU multi-tenant scope clarification + DTU-MULTI-001 comment requirement). BC-3.2.001 added to behavioral_contracts. Points 8→11 (+3 pts for multi-org ACs). acceptance_criteria_count 10→13. red_gate_tests 4→5. EC-006..008 added. Title updated to reflect multi-org scope. |
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
