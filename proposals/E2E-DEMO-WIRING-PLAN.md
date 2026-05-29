---
document_type: architect-proposal
title: "E2E Demo Wiring Plan — Live DTU Round-Trip via Claude MCP"
author: architect
date: "2026-05-29"
status: DRAFT
version: "1.0"
anchor_gap: GAP-002-A
---

# E2E Demo Wiring Plan — Live DTU Round-Trip via Claude MCP

## Goal

Connect Claude Code to `prism-bin start`, issue `tool_query "FROM
crowdstrike_detections LIMIT 5"` via the MCP stdio transport, and receive real
Arrow data from the running `prism-dtu-demo-server` (CrowdStrike clone),
OCSF-normalized, in the response envelope.

---

## 1. Gap Analysis

### The Core Gap: PipelineExecutor is not bridged to AdapterRegistry (GAP-002-A)

The query execution path in `prism-query/src/materialization.rs` calls
`fan_out()` which calls `registry.get(org_id, sensor_id)` to obtain an
`Arc<dyn SensorAdapter>`, then calls `adapter.fetch()`. That is the ONLY path
to live sensor data.

`PipelineExecutor::execute()` in `prism-spec-engine/src/pipeline.rs` knows how
to talk to a sensor HTTP endpoint from a TOML spec (fetch steps, JSONPath,
pagination, auth via `AuthProvider`, retry) but it is NOT called from `fan_out`.

`AdapterRegistry` at boot step 9 is constructed empty with the comment `GAP-002-A
— spec-catalog dispatch deferred`. Queries parse correctly, plan correctly, and
reach the fan-out layer, but every `registry.get()` returns `None` (after
PLUGIN-MIGRATION-001-A deletes the hardcoded adapters entirely), which means the
fan-out emits `SensorError::AdapterNotFound` for every target, which produces
empty batches or partial-failure responses.

**The bridge that needs to be built: a `SpecDrivenSensorAdapter` struct that
implements `dyn SensorAdapter` and delegates to `PipelineExecutor::execute()`.**
At boot step 9 (or 9A), iterate the loaded `SensorSpec` map and register one
`SpecDrivenSensorAdapter` per `(OrgId, SensorId)` pair into `AdapterRegistry`.

### Surface-by-surface analysis

#### (a) AdapterRegistry populate-from-spec-catalog at boot — CLOSES GAP-002-A

Status: NOT covered by any existing story.

The comment at boot.rs line 1876 names `S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH`
as the future target, but that story ID does not exist in STORY-INDEX.md. The
actual S-5.04 story (`prism-mcp: Sensor Health Subsystem`) is scoped to BC-2.08
health-check contracts, not adapter population.

The sensor health story (S-5.04) depends on adapters already being registered so
it can ping them. It cannot close GAP-002-A because it is downstream of GAP-002-A.

New story required: **S-DEMO-001** (SpecDrivenSensorAdapter + boot step 9A wiring).
Priority: P0 — nothing else in the demo works without this.

#### (b) Per-sensor adapter implementation

Status: Partially addressed by existing infrastructure, but the bridge struct is
missing.

What exists:
- `PipelineExecutor::execute()` is fully implemented (PLUGIN-PREREQ-B, merged).
  It reads `SensorSpec`, runs HTTP steps, handles JSONPath, pagination, auth via
  `Arc<dyn AuthProvider>`, 401-refresh via plugin, returns `Vec<RecordBatch>`.
- `PluginAuthProvider` is implemented (PLUGIN-PREREQ-E + S-PLUGIN-CI-001, both
  merged). The `crowdstrike-oauth2.prx` plugin artifact is committed.
- TOML sensor specs for CrowdStrike, Claroty, Cyberint, Armis exist at
  `crates/prism-sensors/specs/` (PLUGIN-MIGRATION-001-D, merged). They include
  `auth_plugin = "crowdstrike-oauth2"` and DTU-grounded URLs.

What is missing:
- The `SpecDrivenSensorAdapter` struct that wraps `PipelineExecutor::execute()`
  to satisfy the `dyn SensorAdapter` interface.
- The boot-time loop that instantiates one `SpecDrivenSensorAdapter` per sensor
  spec and calls `AdapterRegistry::register()`.

ADR-023 explicitly permits `PipelineExecutor` in production; it is in the
Permitted Patterns section. Building a `SpecDrivenSensorAdapter` that calls
`PipelineExecutor` is architecturally correct — it is the intended end state of
PLUGIN-MIGRATION-001.

No concrete sensor-specific Rust adapters need to be written. The TOML spec
files already encode all sensor-specific behavior.

There is one complication: `SensorAdapter::fetch()` takes `&dyn SensorAuth`
(from the legacy auth trait), while `PipelineExecutor::execute()` takes
`Arc<dyn AuthProvider>` (the newer plugin-auth trait). The `SpecDrivenSensorAdapter`
must hold the `Arc<PluginAuthProvider>` from boot step 7.5b
(`plugin_result.plugin_auth_providers`) and use it via `AuthProvider` rather than
routing through `SensorAuth`. This means the `SensorAuth` argument to
`SensorAdapter::fetch()` should be ignored when the adapter holds a plugin
auth provider. This is architecturally clean: plugin-authed sensors bypass the
legacy credential resolver entirely per ADR-028 §D10.

Covered in new story S-DEMO-001 scope.

#### (c) Auth resolution per-sensor

Status: The plugin auth path is the correct production path (ADR-028 §D10).

For the demo, CrowdStrike uses `auth_plugin = "crowdstrike-oauth2"`. At boot
step 7.5b, `validate_and_construct_auth_providers()` already constructs a
`PluginAuthProvider` for CrowdStrike. The `plugin_result.plugin_auth_providers`
HashMap carries this. The `SpecDrivenSensorAdapter` for CrowdStrike must hold
this `Arc<PluginAuthProvider>` and pass it to `PipelineExecutor::execute()`.

For sensors without `auth_plugin` (if any for the demo), the
`ProductionCredentialResolver` path applies. For the initial demo scope (CrowdStrike
only), the plugin path is sufficient.

S-2.07 (per-sensor auth + pagination) was superseded by ADR-023 (PLUGIN-MIGRATION-001-H).
The plugin auth path IS the production auth path; S-2.07 patterns are retired.

Auth resolution is in-scope for S-DEMO-001.

#### (d) DTU clone orchestration for the demo

Status: EXISTS — `prism-dtu-demo-server` binary already handles this.

`prism-dtu-demo-server start --config <path>` launches CrowdStrike, Claroty,
Cyberint, Armis, ThreatIntel, and NVD clones concurrently per config. It writes
`.prism-dtu-demo-server.urls.json` with the bound addresses.

CrowdStrike, Claroty, Cyberint, and Armis clones are wired via
`harness.rs::build_clones_from_config()`. All four are available.

No new story required for orchestration itself. The demo runbook story
(S-DEMO-003) will encode the launch procedure as ACs.

#### (e) Sensor spec configuration for demo

Status: PARTIALLY EXISTS — specs at `crates/prism-sensors/specs/` have production
`base_url = "https://api.crowdstrike.com"`. For a demo against DTU clones, these
must either (1) be overridden via per-org overlay in `customers/demo/` or (2) a
demo-specific variant spec must point at `http://127.0.0.1:<PORT>`.

The ADR-029 overlay mechanism is the correct approach: write a
`customers/demo-org/crowdstrike.sensor.toml` overlay with
`base_url = "http://127.0.0.1:<crowdstrike_port>"`. This leaves the base spec
unchanged while letting the demo org hit the DTU clone.

The port is available from `.prism-dtu-demo-server.urls.json` after the demo
server starts. The demo setup script reads that file and writes the overlay.

This is in-scope for the demo setup runbook story (S-DEMO-003). The overlay
mechanism itself is already implemented (S-CONFIG-MULTI-TENANT-OVERRIDE-001, merged).

#### (f) Credential bootstrap for demo

Status: NOT covered by any existing story for the demo context.

AD-017 mandates reference-only credentials; values must never transit AI context.
For the demo, credentials need to exist in the system keyring under the namespaced
form `prism/{sensor_id}/{ref_name}` that `KeyringCredentialProbe` checks at boot.

For DTU clones, the auth credential is the OAuth2 client_id and client_secret
that the CrowdStrike DTU's `/oauth2/token` endpoint accepts. Looking at the DTU
clone, the default admin token for the DTU is the credential to store, not a
production secret.

The DTU clone's OAuth2 endpoint accepts any `client_id`/`client_secret` pair
(the clone is a fidelity model, not a real auth enforcer). The
`crowdstrike-oauth2.prx` plugin calls the DTU's `base_url/oauth2/token` to
acquire a token, which the DTU clone will return.

A `demo-credential-setup.sh` script needs to call `prism credential set` (or
directly use the `keyring` crate CLI equivalent) to pre-populate the namespaced
entries. This is in-scope for S-DEMO-003.

#### (g) Multi-tenant config for demo

Status: Partially addressed by S-CONFIG-MULTI-TENANT-OVERRIDE-001 (merged).

The `prism.toml` `[[orgs]]` section needs at least one entry with a valid UUID v7
`org_id` and a `org_slug` matching the `customers/` subdirectory name for the
overlay resolution. For the demo, a single org `demo-org` is sufficient.

This is config file setup that belongs in the demo runbook (S-DEMO-003).

#### (h) End-to-end smoke test harness

Status: NOT covered by any existing story.

A test that:
1. Launches `prism-dtu-demo-server` with demo config
2. Runs `prism-bin start` with demo prism.toml
3. Sends `tool_query "FROM crowdstrike_detections LIMIT 5"` via a mock MCP client
4. Asserts: response contains non-empty data, fields match OCSF schema, no
   error in response envelope

This is more than a unit test — it is a subprocess integration test (spans
`prism-bin` + `prism-dtu-demo-server`). It belongs in a new story (S-DEMO-002).

The MCP client in the test can use the `rmcp` crate directly (same as the server)
to send `tools/call` JSON-RPC over stdio.

#### (i) Install + setup runbook

Status: NOT covered by any existing story.

Belongs in S-DEMO-003.

#### (j) Sensor health check end-to-end (S-5.04)

Status: S-5.04 (draft) depends on S-5.03 and S-2.07. S-2.07 is superseded.

S-5.04 needs to be re-evaluated because its dependency on S-2.07 is broken
(S-2.07 patterns are superseded by the plugin path per ADR-023). The story needs
an updated `depends_on` list.

More importantly: S-5.04's health check implementation in the MCP server currently
returns a stub response citing GAP-002-A (see `server.rs` lines 2337-2343 and
2382-2387). Once GAP-002-A is closed by S-DEMO-001, S-5.04 can implement real
pings via the registered `SpecDrivenSensorAdapter`.

S-5.04 is NOT on the critical path for the initial demo. The demo goal is
`tool_query` returning real data, not sensor health pings. S-5.04 belongs in a
follow-up wave after S-DEMO-001 ships.

A `depends_on` update to S-5.04 (replace S-2.07 with S-DEMO-001) is needed
before S-5.04 can be worked. This is an in-scope spec fix, not a new story.

#### (k) Additional gaps surfaced by scope analysis

**PLUGIN-MIGRATION-001-A (status: ready, not yet merged):**
This story deletes the 4 legacy Rust auth modules and replaces `init_registry_for_org`.
Until it merges, the old adapter code exists alongside the new plugin path but is
no longer called (GAP-002-A is independent — the registry is empty whether or not
001-A has merged). However, for cleanliness and to avoid confusion, 001-A should
merge before or alongside S-DEMO-001. GAP: 001-A is `ready` but unmerged — it is
blocking PLUGIN-MIGRATION-001-B.

**PLUGIN-MIGRATION-001-B (status: planned):**
Converts 5 sensor-name dispatch sites in `prism-query` to spec-catalog lookups.
This is downstream of 001-A. The dispatch sites it touches are in the write path
and query routing, not the materialization pipeline. For the read-only demo
(`FROM crowdstrike_detections LIMIT 5`), 001-B is NOT on the critical path.
It needs to ship before the demo can be called production-complete.

**S-CONFIG-MULTI-TENANT-OVERRIDE-001 (status: draft):**
Implements per-org overlay loading — critical for pointing the demo org at DTU
clone URLs. This story is ALREADY DRAFTED. It must merge before the demo can
route queries to DTU clone addresses. This is on the critical path.

**PLUGIN-MIGRATION-001-E (status: ready, not yet merged):**
Delivers the CrowdStrike OAuth2 WASM plugin integration. S-PLUGIN-CI-001 has
already committed the `.prx` artifact. PLUGIN-MIGRATION-001-E connects the plugin
to the boot auth provider construction. Until it merges, the `auth_plugin =
"crowdstrike-oauth2"` in the TOML spec will cause boot step 7.5b to construct a
`PluginAuthProvider` for CrowdStrike, but... actually wait. Looking at the code,
`validate_and_construct_auth_providers` already does construct `PluginAuthProvider`
instances (this is implemented in `run_boot_sequence`). The question is whether
`crowdstrike-oauth2` is registered in `PluginRuntime`.

The `.prx` artifact is committed (`crates/prism-spec-engine/wasm/`?). Boot step
7.5 calls `plugin_load_step_with_audit` which loads plugins from `config.plugin_dir`.
The demo `prism.toml` must set `plugin_dir` to a directory containing
`crowdstrike-oauth2.prx`. S-PLUGIN-CI-001 committed the artifact — need to verify
the path and ensure the demo setup script copies it to the right location.

---

## 2. Story Decomposition

### New stories required

#### S-DEMO-001: SpecDrivenSensorAdapter — Bridge PipelineExecutor to AdapterRegistry (CLOSES GAP-002-A)

- **Story ID:** S-DEMO-001
- **Title:** `prism-sensors + prism-bin: SpecDrivenSensorAdapter — Bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A)`
- **Scope:**
  - Implement `SpecDrivenSensorAdapter` in `prism-sensors/src/spec_driven.rs`:
    - Holds `SensorSpec` + `Arc<dyn AuthProvider>` (PluginAuthProvider for plugin-authed sensors)
    - Implements `dyn SensorAdapter` by delegating `fetch()` to `PipelineExecutor::execute()`
    - `sensor_type()` returns `SensorId` from `SensorSpec.sensor_id`
    - `sensor_name()` returns `SensorSpec.name`
    - The `auth` argument to `fetch()` is ignored; auth flows through the held `Arc<dyn AuthProvider>`
  - Add boot step 9A to `boot.rs` (between step 7.5b and step 9): iterate
    `plugin_result.plugin_auth_providers` + loaded sensor specs, construct one
    `SpecDrivenSensorAdapter` per sensor spec, register into `AdapterRegistry`
  - Wire the populated `AdapterRegistry` into `step9_start_mcp_server` (currently
    passed as `AdapterRegistry::new()`)
  - Add `reqwest::Client` construction with 30-second timeout inside
    `SpecDrivenSensorAdapter::new()` (production HTTP client, AD-017 compliant)
  - Add BC-2.16.002 catalog row for `boot.step9a.adapter_registry_populated` event
  - Red Gate tests: `test_BC_2_11_spec_driven_adapter_non_empty_registry_wires_into_query_engine`
    and `test_BC_2_11_spec_driven_adapter_fetch_delegates_to_pipeline_executor`
- **depends_on:** S-5.01-FOLLOWUP-MCP-BOOT (merged), S-PLUGIN-CI-001 (merged),
  PLUGIN-MIGRATION-001-D (merged), S-CONFIG-MULTI-TENANT-OVERRIDE-001 (must merge first)
- **estimated_days:** 2-3
- **priority:** P0
- **anchor_bcs:** BC-2.11.001 (query execution), BC-2.11.005 (materialization pipeline), BC-2.01.013 (adapter pattern)
- **Note on crate boundary:** `SpecDrivenSensorAdapter` should live in `prism-bin`
  rather than `prism-sensors` because it imports both `prism-spec-engine`
  (for `PipelineExecutor`) and `prism-sensors` (for `dyn SensorAdapter`).
  `prism-sensors` must NOT import `prism-spec-engine` per ADR-023 §D3 Forbidden
  Dependencies. The struct can be defined inline in `boot.rs` or in a new
  `crates/prism-bin/src/spec_driven_adapter.rs` module.

#### S-DEMO-002: E2E Smoke Test Harness

- **Story ID:** S-DEMO-002
- **Title:** `prism-bin: E2E Smoke Test — tool_query returns real data from DTU clone via MCP`
- **Scope:**
  - Integration test in `crates/prism-bin/tests/e2e_demo_smoke.rs`
  - Launches `prism-dtu-demo-server` as a subprocess, waits for ready signal
  - Spawns `prism-bin start` with demo `prism.toml` (temp-dir config, DTU base_url overlay)
  - Connects via `rmcp` stdio client (or writes raw JSON-RPC to the prism-bin stdin)
  - Issues `tools/call` for `tool_query` with `FROM crowdstrike_detections LIMIT 5`
  - Asserts: response envelope `status == "ok"`, at least 1 row, fields `id` and
    `created_timestamp` present, no error code
  - Test is `#[ignore]` by default (requires DTU server and full boot); un-ignored
    in CI via a dedicated `e2e` profile job
  - Demo credential bootstrap: test uses `prism credential set` CLI or
    directly calls the keyring crate with test values before boot
- **depends_on:** S-DEMO-001, S-CONFIG-MULTI-TENANT-OVERRIDE-001
- **estimated_days:** 1.5-2
- **priority:** P0
- **anchor_bcs:** BC-2.11.001, BC-2.10.001 (tool_query BC)

#### S-DEMO-003: Demo Setup Runbook + Scripts

- **Story ID:** S-DEMO-003
- **Title:** `scripts: Demo Setup Runbook — credential bootstrap, DTU launch, prism config, MCP connect`
- **Scope:**
  - `scripts/demo-setup.sh` — idempotent setup:
    1. `cargo build --release -p prism-bin -p prism-dtu-demo-server --features dtu`
    2. Create demo config dir (`~/.config/prism-demo/`)
    3. Write `prism.toml` with one org `demo-org`, `spec_dir`, `plugin_dir`, `state_dir`
    4. Copy `crowdstrike-oauth2.prx` to `plugin_dir`
    5. Run `prism-dtu-demo-server start --config scripts/demo.toml` (background)
    6. Read `.prism-dtu-demo-server.urls.json` for CrowdStrike port
    7. Write `customers/demo-org/crowdstrike.sensor.toml` overlay with `base_url = "http://127.0.0.1:<port>"`
    8. Call `prism credential set crowdstrike client_id demo-client` and
       `prism credential set crowdstrike client_secret demo-secret`
  - `scripts/demo.toml` — demo harness config (CrowdStrike enabled, others optional)
  - `scripts/demo-teardown.sh` — stop DTU server, remove temp state
  - `docs/demo-runbook.md` — step-by-step human walkthrough for Claude MCP session
  - ACs cover: script exits 0 on fresh machine, credential bootstrap AD-017 compliant,
    DTU URL written correctly, prism starts without error
- **depends_on:** S-DEMO-001, S-DEMO-002
- **estimated_days:** 0.5-1
- **priority:** P1
- **anchor_bcs:** (no new BCs — infrastructure only)

#### S-5.04-FIX-001: Fix S-5.04 depends_on (S-2.07 superseded)

- **Story ID:** S-5.04-FIX-001
- **Title:** `.factory: Fix S-5.04 depends_on — replace superseded S-2.07 with S-DEMO-001`
- **Scope:**
  - Update S-5.04 story frontmatter `depends_on: [S-5.03, S-DEMO-001]`
    (removes S-2.07 which is superseded per PLUGIN-MIGRATION-001-H)
  - Add implementation note that health-check pings use `SpecDrivenSensorAdapter`
    once registered in `AdapterRegistry` (GAP-002-A closure)
  - STORY-INDEX update for S-5.04 row
  - This is a spec-only story (factory-artifacts branch), no code changes
- **depends_on:** S-DEMO-001
- **estimated_days:** 0.25
- **priority:** P2
- **anchor_bcs:** BC-2.08.001 through BC-2.08.007

### Stories needed before S-DEMO-001 can merge

These stories are NOT new — they already exist and are blocking:

| Existing Story | Status | Why it blocks the demo |
|---|---|---|
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 | draft | Overlay loading needed for DTU base_url routing per demo-org |
| PLUGIN-MIGRATION-001-A | ready | Cleans up legacy adapter code; creates unambiguous state for SpecDrivenSensorAdapter wiring |
| PLUGIN-MIGRATION-001-E | ready | Connects crowdstrike-oauth2 plugin to PipelineExecutor auth path at boot |

---

## 3. Execution Sequencing

### Dependency graph

```
S-CONFIG-MULTI-TENANT-OVERRIDE-001  (draft → must ship)
    │
    ├── PLUGIN-MIGRATION-001-A  (ready → must ship)
    │       │
    │       └── PLUGIN-MIGRATION-001-E  (ready → must ship for CrowdStrike OAuth2)
    │               │
    │               └── S-DEMO-001  ← KEYSTONE
    │                       │
    │                       ├── S-DEMO-002  (E2E smoke test)
    │                       │       │
    │                       │       └── S-DEMO-003  (runbook + scripts)
    │                       │
    │                       └── S-5.04-FIX-001  (spec fix, can run parallel to S-DEMO-002)
```

### Critical path

1. S-CONFIG-MULTI-TENANT-OVERRIDE-001 (must merge — overlay loading for DTU URLs)
2. PLUGIN-MIGRATION-001-A (must merge — legacy adapter deletion)
3. PLUGIN-MIGRATION-001-E (must merge — OAuth2 plugin integration at boot)
4. S-DEMO-001 (keystone — SpecDrivenSensorAdapter + AdapterRegistry population)
5. S-DEMO-002 (E2E smoke test)
6. S-DEMO-003 (runbook)

### Parallelizable work

- PLUGIN-MIGRATION-001-A and S-CONFIG-MULTI-TENANT-OVERRIDE-001 can merge in
  parallel (no dependency between them).
- PLUGIN-MIGRATION-001-E depends on PLUGIN-MIGRATION-001-A (per STORY-INDEX).
- S-DEMO-002 and S-5.04-FIX-001 can run in parallel after S-DEMO-001.
- S-DEMO-003 should start immediately after S-DEMO-001 merges (no blocking dep
  on S-DEMO-002 for the script structure, but should not ship until S-DEMO-002
  confirms the green path).

---

## 4. Install + Setup Outline

Steps a fresh user would follow. Story-writer encodes these as ACs in S-DEMO-003.

1. **Build:**
   ```
   cargo build --release -p prism-bin -p prism-dtu-demo-server --features dtu
   ```

2. **Create demo config directory:**
   ```
   mkdir -p ~/.config/prism-demo/specs/customers/demo-org/
   mkdir -p ~/.config/prism-demo/plugins/
   mkdir -p ~/.config/prism-demo/state/
   ```

3. **Copy production sensor specs:**
   ```
   cp crates/prism-sensors/specs/crowdstrike.sensor.toml ~/.config/prism-demo/specs/
   ```

4. **Copy WASM plugin:**
   ```
   cp <plugin_artifact_path>/crowdstrike-oauth2.prx ~/.config/prism-demo/plugins/
   ```
   (Plugin artifact path to be determined — S-PLUGIN-CI-001 committed it but the
   human-accessible path needs to be documented in S-DEMO-003.)

5. **Write `prism.toml`:**
   ```toml
   state_dir = "~/.config/prism-demo/state"
   spec_dir = "~/.config/prism-demo/specs"
   plugin_dir = "~/.config/prism-demo/plugins"
   [[orgs]]
   org_id = "<uuid-v7>"
   org_slug = "demo-org"
   [credential_backend]
   type = "keyring"
   ```

6. **Start DTU server:**
   ```
   ./target/release/prism-dtu-demo-server start --config scripts/demo.toml &
   ```
   Wait for `.prism-dtu-demo-server.urls.json` to appear.

7. **Write CrowdStrike overlay with DTU address:**
   Read CrowdStrike port from `.prism-dtu-demo-server.urls.json`, write:
   ```toml
   # ~/.config/prism-demo/specs/customers/demo-org/crowdstrike.sensor.toml
   extends = "crowdstrike"
   base_url = "http://127.0.0.1:<PORT>"
   ```

8. **Bootstrap demo credentials (AD-017 compliant):**
   ```
   prism credential set crowdstrike client_id demo-client
   prism credential set crowdstrike client_secret demo-secret
   ```
   These are not real credentials; the DTU clone accepts any value.

9. **Start Prism:**
   ```
   ./target/release/prism start --config ~/.config/prism-demo/
   ```

10. **Connect Claude MCP client:**
    In Claude Code: add prism-bin as an MCP server in `settings.json`, or run
    the demo query interactively via `claude-mcp connect stdio ./target/release/prism start`.

11. **Run demo query:**
    ```
    tool_query "FROM crowdstrike_detections LIMIT 5" client_id="demo-org"
    ```
    Expected: 5 rows of OCSF-normalized detection data from the CrowdStrike DTU clone.

---

## 5. ADR Updates Required

The following ADR amendments are candidates; no ADRs need to be written as
prerequisites for the stories, but they should be drafted in parallel with
S-DEMO-001:

| ADR | Amendment needed |
|-----|-----------------|
| ADR-022 §B | Add boot step 9A (`spec_driven_adapter_registry_populate`) to the sequencing invariant table. Between step 7.5b (auth provider construction) and step 9 (MCP server start). |
| ADR-023 §Permitted Patterns | Add `SpecDrivenSensorAdapter` (struct in `prism-bin` implementing `dyn SensorAdapter` via `PipelineExecutor`) to the permitted patterns list. This clarifies that the bridge pattern is architecturally intended. |
| ADR-022 §F (dependency wiring) | Update the comment that `adapter_registry` is empty at step 9 — after S-DEMO-001, it is populated from the spec catalog. |
| ADR-028 | Add §D11 note: demo credential setup uses dummy values against DTU clones; the OAuth2 flow is exercised but no real secrets are required. |

Story-writer should note these in S-DEMO-001 under "Spec Updates" section.

---

## 6. Risk Assessment

### Risk 1: crate dependency boundary violation (HIGH probability if not caught early)

`SpecDrivenSensorAdapter` must NOT live in `prism-sensors`. If the implementer
naively puts it there, they import `prism-spec-engine` into `prism-sensors`, which
violates ADR-023 §D3 Forbidden Dependencies (`prism-sensors` must not import
`prism-spec-engine`). The struct belongs in `prism-bin/src/spec_driven_adapter.rs`
(or inline in `boot.rs`) where both deps are already present.

Mitigation: S-DEMO-001 must explicitly state the module location in its Task table.

### Risk 2: AuthProvider vs SensorAuth interface impedance (MEDIUM)

`SensorAdapter::fetch()` takes `&dyn SensorAuth`, but `PipelineExecutor::execute()`
takes `Arc<dyn AuthProvider>`. The `SpecDrivenSensorAdapter` holds the
`Arc<PluginAuthProvider>` acquired at step 7.5b and ignores the `auth` argument.
This is clean for plugin-authed sensors but requires care: if a sensor spec has
no `auth_plugin` (e.g., a future sensor using the `bearer_static` type without a
plugin), the adapter must fall back to resolving auth from the passed `SensorAuth`
argument.

For the initial demo (CrowdStrike only), this is not a risk — the plugin path
is always present. For full generality, the adapter needs a discriminated
union of auth paths.

Mitigation: Document the plugin vs non-plugin auth paths in S-DEMO-001 tasks.
For non-plugin sensors, `PipelineExecutor` accepts a `BearerStaticAuthProvider`
or similar thin wrapper that extracts the token from the passed `SensorAuth`.
This is bounded scope (one day of implementation).

### Risk 3: DTU clone data fidelity (LOW for demo, MEDIUM for production validation)

DTU clones return synthetic data that matches the real API schema but not the real
API's business logic (e.g., pagination cursor semantics may differ). The demo
exercises the happy path; edge cases (empty result sets, pagination exhaustion,
rate limiting) are not validated by the demo.

Mitigation: S-DEMO-002 should include a test that sends `LIMIT 0` and verifies
empty-but-not-error response, and a test that sends `LIMIT 200` to exercise
pagination at least one extra page.

### Risk 4: Credential bootstrap for demo without real cloud accounts (LOW)

DTU clones do not validate OAuth2 credential values beyond the presence of
`client_id` and `client_secret` parameters in the token request body. The
`crowdstrike-oauth2.prx` plugin calls `base_url/oauth2/token` with the credentials
from the keyring. The DTU clone returns a synthetic bearer token regardless of
credential values. This means the demo credential setup is trivially reproducible
with dummy values.

The risk is that the demo appears to validate the credential pipeline but does
not prove real-credential auth works. This is by design for the demo context.
Production validation requires real cloud credentials in a separate DTU validation
session.

### Risk 5: Multi-tenant complexity for single-user demo (LOW)

The OrgRegistry requires at least one org entry (BC-2.21.001). The demo uses
a single `demo-org` org. The overlay mechanism requires a `customers/demo-org/`
directory. This is simple to set up but easy to misconfigure (wrong slug,
non-v7 UUID). The setup script must be correct and tested.

Mitigation: S-DEMO-003 acceptance criteria include: script runs on a fresh
machine, `prism start` exits 0, demo query returns data.

### Risk 6: PLUGIN-MIGRATION-001-A timing (MEDIUM)

001-A is `ready` but unmerged. Its AC-006 is gated on 001-E merging first (per
STORY-INDEX: "AC-006 GATED-ON-001-E for CrowdStrike module deletion"). 001-E is
also `ready`. Both need to merge before S-DEMO-001 to avoid working against
code that is scheduled for deletion. If the orchestrator dispatches S-DEMO-001
before 001-A/E, the `SpecDrivenSensorAdapter` implementation will coexist with
the legacy adapters which are registered nowhere (empty registry), which is
actually fine — but confusing to review.

Recommendation: Merge 001-E → 001-A → then dispatch S-DEMO-001.

---

## 7. Cycle Estimate

Calibration reference: S-5.01-FOLLOWUP-MCP-BOOT was approximately 1 focused
session to converge (19 LOCAL passes, 16 fix-bursts). The stories here are
narrower in scope.

| Story | Estimated sessions | Confidence |
|-------|-------------------|------------|
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 | 0.75 | MEDIUM (already drafted, needs TDD implementation) |
| PLUGIN-MIGRATION-001-A | 0.5 | HIGH (ready, narrowly scoped: delete + replace init_registry_for_org) |
| PLUGIN-MIGRATION-001-E | 0.75 | HIGH (ready, plugin integration at boot) |
| S-DEMO-001 (SpecDrivenSensorAdapter) | 1.0-1.5 | MEDIUM (new struct + boot wiring; auth interface impedance is the unknown) |
| S-DEMO-002 (E2E smoke test) | 0.5-0.75 | MEDIUM (subprocess integration test; DTU boot timing is the uncertainty) |
| S-DEMO-003 (runbook + scripts) | 0.25 | HIGH (bash scripts + markdown) |
| S-5.04-FIX-001 (spec fix) | 0.1 | HIGH (frontmatter change only) |
| **Total** | **~3.85-4.35 sessions** | |

Rounded: **~4 focused sessions** from today to a reproducible E2E demo.

If S-CONFIG-MULTI-TENANT-OVERRIDE-001, 001-A, and 001-E are delivered in a
single burst (they can be dispatched in sequence within one session), the
remaining work is:

- Session 1: S-CONFIG-MULTI-TENANT-OVERRIDE-001 + 001-A + 001-E (prereqs cleared)
- Session 2: S-DEMO-001 (SpecDrivenSensorAdapter — keystone)
- Session 3: S-DEMO-002 + S-DEMO-003 (smoke test + runbook)

---

## 8. Human Review Items Before Story-Writer Starts

The following require human decision before the story-writer drafts S-DEMO-001:

1. **Crate location for `SpecDrivenSensorAdapter`:** Recommendation is
   `crates/prism-bin/src/spec_driven_adapter.rs`. Confirm or override.

2. **Scope of initial demo:** This plan targets CrowdStrike-only for the first
   demo round-trip (it has the `crowdstrike-oauth2.prx` plugin already committed).
   Armis, Claroty, and Cyberint use `bearer_static` auth (no WASM plugin needed;
   the TOML spec will need a `BearerStaticAuthProvider` wrapper or a non-plugin
   auth path in `SpecDrivenSensorAdapter`). Confirm: CrowdStrike-only for demo
   scope, or all four sensors?

3. **Merging order for 001-A and 001-E:** Both are `ready`. The recommended
   order is 001-E first (S-PLUGIN-CI-001 is already merged; 001-E builds on it),
   then 001-A. Confirm authorization to dispatch both.

4. **PLUGIN-MIGRATION-001-B:** Not on the critical path for the read-only demo.
   Confirm that the write-path dispatch cleanup (5 sites in prism-query) can wait
   until after the demo ships.

5. **`prism credential set` CLI command:** Does this command exist in the current
   `prism-bin` CLI? The boot sequence validates credential refs at step 5 via
   `KeyringCredentialProbe`, which reads from the system keyring under
   `prism/{sensor_id}/{ref_name}`. If `prism credential set` is not yet
   implemented (it is S-5.xx scope), the demo setup script needs to use the
   platform keyring CLI (`secret-tool` on Linux, `security` on macOS) or a
   direct Rust helper binary. Confirm the credential-set mechanism available today.

---

## Summary

**New stories proposed:** 4
- S-DEMO-001 (P0, ~1-1.5 sessions): SpecDrivenSensorAdapter — closes GAP-002-A
- S-DEMO-002 (P0, ~0.5-0.75 sessions): E2E smoke test
- S-DEMO-003 (P1, ~0.25 sessions): runbook + scripts
- S-5.04-FIX-001 (P2, ~0.1 sessions): spec fix for S-5.04 depends_on

**Existing stories that must merge first:**
- S-CONFIG-MULTI-TENANT-OVERRIDE-001 (draft) — P0 blocker
- PLUGIN-MIGRATION-001-E (ready) — P0 blocker
- PLUGIN-MIGRATION-001-A (ready) — P0 blocker

**Total estimated sessions to live demo:** ~4 focused sessions

**Critical path:** S-CONFIG-MULTI-TENANT-OVERRIDE-001 → 001-E → 001-A → S-DEMO-001 → S-DEMO-002 → S-DEMO-003

**Architectural correctness:** Building `SpecDrivenSensorAdapter` in `prism-bin`
(not `prism-sensors`) using `PipelineExecutor` from `prism-spec-engine` is the
correct ADR-023-compliant architecture. It is not a workaround; it IS the
intended end state of the plugin migration architecture.
