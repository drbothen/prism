---
document_type: story
story_id: S-DEMO-001
title: "prism-bin: SpecDrivenSensorAdapter + Boot Step 9A — Bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A)"
wave: 5
epic_id: E-DEMO
priority: P0
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns the SensorAdapter trait that SpecDrivenSensorAdapter implements;
#     this story closes the registry gap so fan_out() can resolve (org, sensor) → adapter.
#   SS-16 (Spec Engine) owns PipelineExecutor and the SensorSpec catalog that drives dispatch;
#     SpecDrivenSensorAdapter wraps PipelineExecutor, so SS-16 is a direct consumer.
#   SS-22 (Binary Entrypoint) owns boot.rs and the boot step sequencing contract (BC-2.22.001);
#     boot step 9A is a new sequencing invariant that must be registered in the ADR-022 §B table.
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: [CAP-001, CAP-015, CAP-029, CAP-034]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait Eliminates Per-Sensor Code Duplication — SpecDrivenSensorAdapter
                 # is the spec-driven implementation of the SensorAdapter interface. This story
                 # closes the spec-driven registration gap that BC-2.01.013 was always pointing toward.
  - BC-2.11.005  # Ephemeral Materialization — fan_out() calls registry.get(org_id, sensor_id);
                 # after this story, get() returns SpecDrivenSensorAdapter for every (org, sensor)
                 # pair in the spec catalog, enabling end-to-end live data flow.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — (org_id, sensor_id) tuple resolves
                 # to ResolvedSensorSpec; SpecDrivenSensorAdapter receives that ResolvedSensorSpec
                 # (with per-org base_url overlay from ADR-029) at construction time during boot 9A.
  - BC-2.22.001  # Boot Orchestration Sequencing — boot step 9A must be registered in the
                 # ADR-022 §B sequencing invariant table between step 7.5b and step 9.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — existing parity tests exercise the pipeline path that
            # SpecDrivenSensorAdapter delegates to; this story wires the adapter into the registry
            # so parity tests exercise the full round-trip path.
depends_on:
  - PLUGIN-MIGRATION-001-A   # Must merge first: cleans up legacy hardcoded adapter code so the
                              # registry is unambiguously empty and owned by spec-driven path only.
  - PLUGIN-MIGRATION-001-E   # Must merge first: provides CrowdStrike OAuth2 PluginAuthProvider
                              # at boot step 7.5b; SpecDrivenSensorAdapter for CrowdStrike holds
                              # this Arc<PluginAuthProvider> and ignores the SensorAuth argument.
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Must merge first: per-org overlay loading (ADR-029)
                                         # produces ResolvedSensorSpec map; boot 9A iterates this
                                         # map to construct one SpecDrivenSensorAdapter per (org, sensor).
blocks:
  - S-DEMO-002   # E2E smoke test cannot run without adapters in the registry.
  - S-5.04       # Sensor health checks call the registered adapter; health subsystem is
                 # downstream of GAP-002-A closure.
  - S-5.04-FIX-001  # Spec fix story depends on this story existing as the new S-5.04 dep.
points: 8
# Points justification:
#   - SpecDrivenSensorAdapter struct + SensorAdapter impl (plugin auth path + bearer_static path): ~2 pts
#   - BearerStaticAuthProvider wrapper for non-plugin sensors (Armis, Claroty, Cyberint): ~1 pt
#   - Boot step 9A loop + AdapterRegistry::register() calls: ~1.5 pts
#   - reqwest::Client construction with 30s timeout (AD-017 compliant): ~0.5 pts
#   - BC-2.16.002 catalog row for boot.step9a.adapter_registry_populated: ~0.5 pts
#   - Red Gate tests (2 required, see ACs): ~1.5 pts
#   - ADR-022 §B + ADR-023 §Permitted-Patterns amendment: ~0.5 pts
#   Total: 8 points (~2-3 days of focused TDD work)
estimated_days: 2
risk: MEDIUM
# Risk justification: Auth interface impedance between SensorAdapter::fetch(&dyn SensorAuth) and
# PipelineExecutor::execute(Arc<dyn AuthProvider>) is the primary risk. Plugin-authed sensors
# (CrowdStrike) ignore the auth arg and use the held PluginAuthProvider. Bearer-static sensors
# (Armis, Claroty, Cyberint) need a BearerStaticAuthProvider wrapper that extracts the token
# from the SensorAuth argument. Both paths are architecturally clean and bounded in scope.
acceptance_criteria_count: 10
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Crate boundary: SpecDrivenSensorAdapter MUST live in prism-bin (NOT prism-sensors). prism-sensors
    must not import prism-spec-engine per ADR-023 §D3 Forbidden Dependencies. The struct is defined
    in crates/prism-bin/src/spec_driven_adapter.rs where both prism-sensors and prism-spec-engine
    are already workspace deps."
  - "Auth interface impedance: SpecDrivenSensorAdapter holds Arc<dyn AuthProvider> acquired at boot
    step 7.5b for plugin-authed sensors. For bearer_static sensors (no auth_plugin field in spec),
    it constructs a BearerStaticAuthProvider that wraps the token extracted from the SensorAuth arg
    at fetch() call time per ADR-028 §D10."
  - "reqwest::Client timeout: SpecDrivenSensorAdapter::new() constructs reqwest::Client with
    .timeout(Duration::from_secs(30)) per CLAUDE.md conventions. Missing this is a P2 finding."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-sensors/src/lib.rs"
  - "crates/prism-sensors/src/traits.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/auth.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-001 — prism-bin: SpecDrivenSensorAdapter + Boot Step 9A (closes GAP-002-A)

**Story ID:** S-DEMO-001
**Status:** draft
**Version:** v1.0
**Wave:** 5
**Priority:** P0
**Points:** 8

---

## Origin

New story required to close GAP-002-A per architect proposal E2E-DEMO-WIRING-PLAN.md §2.
The comment at `boot.rs` line 1876 named `S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH` as the
target, but that story ID does not exist. GAP-002-A is independent of S-5.04; it must
be closed here first. User scope decision 2026-05-29: all 4 sensors (CrowdStrike + Armis +
Claroty + Cyberint). CrowdStrike goes through WASM OAuth2 plugin (001-E path). The remaining
three use `bearer_static` auth via `BearerStaticAuthProvider` (no per-sensor WASM plugin needed).

---

## Narrative

As the Prism platform engineering team, I want `PipelineExecutor::execute()` bridged to the
`AdapterRegistry` via a `SpecDrivenSensorAdapter` struct, so that every sensor spec loaded at
boot produces a registered adapter and `tool_query "FROM crowdstrike_detections LIMIT 5"` (or
any of the other 3 sensors) returns real Arrow data from the DTU clone rather than `AdapterNotFound`.

---

## Story-Level Goal

After this story merges:
1. `crates/prism-bin/src/spec_driven_adapter.rs` exists with a `SpecDrivenSensorAdapter` struct
   implementing `dyn SensorAdapter` by delegating to `PipelineExecutor::execute()`.
2. Boot step 9A in `boot.rs` iterates the `ResolvedSensorSpec` map (from S-CONFIG-MULTI-TENANT-OVERRIDE-001)
   and registers one `SpecDrivenSensorAdapter` per (OrgId, SensorId) pair.
3. `fan_out()` in `prism-query/src/materialization.rs` returns live Arrow batches for all 4 sensors.
4. GAP-002-A comment in boot.rs is removed and replaced with a working implementation.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.06.014 | Instance Identity Resolution at Fanout — (org_id, sensor_id) Tuple Resolves to ResolvedSensorSpec |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |

---

## Acceptance Criteria

### AC-001: SpecDrivenSensorAdapter delegates to PipelineExecutor for CrowdStrike (plugin auth path)
Given: CrowdStrike sensor spec loaded at boot with `auth_plugin = "crowdstrike-oauth2"` and a
corresponding `PluginAuthProvider` constructed at step 7.5b.
When: `SpecDrivenSensorAdapter::fetch()` is called for CrowdStrike.
Then: The adapter calls `PipelineExecutor::execute()` passing its held `Arc<PluginAuthProvider>`
(the `SensorAuth` arg is ignored); returns `Vec<RecordBatch>` in OCSF-normalized shape.
(traces to BC-2.01.013 postcondition 4: "Adapter implementations are produced from TOML SensorSpec
declarations at runtime; no hand-written adapter code outside prism-sensors is required")
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor`

### AC-002: SpecDrivenSensorAdapter delegates to PipelineExecutor for bearer_static sensors
Given: Armis, Claroty, or Cyberint sensor spec loaded at boot (no `auth_plugin` field; auth_type
is `bearer_static`).
When: `SpecDrivenSensorAdapter::fetch()` is called with a `SensorAuth::BearerStatic { token }` arg.
Then: The adapter constructs a `BearerStaticAuthProvider` from the token in the `SensorAuth` arg,
calls `PipelineExecutor::execute()` with it, and returns `Vec<RecordBatch>`.
(traces to BC-2.01.013 postcondition 4)
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth`

### AC-003: Boot step 9A registers exactly N adapters (N = org count × spec count)
Given: `spec_catalog` has M resolved sensor specs and `org_registry` has K orgs.
When: Boot step 9A runs.
Then: `AdapterRegistry` contains exactly M × K entries (one per (OrgId, SensorId) pair). The
`boot.step9a.adapter_registry_populated` structured event is emitted with fields `sensor_count`
and `org_count` per BC-2.16.002 catalog row added in this story.
(traces to BC-2.22.001 postcondition: "all init steps complete in order before MCP accepts traffic")
Red Gate test: `test_BC_2_22_001_boot_step9a_registers_correct_adapter_count`

### AC-004: Adapter registration is per-org with correct overlay applied
Given: demo-org has a `customers/demo-org/crowdstrike.sensor.toml` overlay setting
`base_url = "http://127.0.0.1:<PORT>"`.
When: Boot step 9A constructs the CrowdStrike adapter for demo-org.
Then: The adapter's internal `PipelineExecutor` uses the overlay `base_url` (from the
`ResolvedSensorSpec`), not the production base URL from the type spec.
(traces to BC-2.06.014 precondition 1: "(org_id, sensor_id) tuple resolves to ResolvedSensorSpec")

### AC-005: Empty spec_catalog → AdapterRegistry remains empty; no error
Given: `spec_catalog` is empty at boot (no TOML specs loaded).
When: Boot step 9A runs.
Then: `AdapterRegistry` is empty; no error is emitted; boot continues to step 9 (MCP server start).
(traces to BC-2.22.001 postcondition: "steps 7 and 8 complete without error")
Red Gate test: `test_BC_2_22_001_boot_step9a_empty_spec_catalog_no_error`

### AC-006: AdapterRegistry::get(org_id, sensor_id) returns adapter for registered pairs
Given: Boot step 9A completed with M × K registrations.
When: `AdapterRegistry::get(org_id, sensor_id)` is called for any (org, sensor) pair present in
both `spec_catalog` and `org_registry`.
Then: Returns `Some(Arc<dyn SensorAdapter>)` — a `SpecDrivenSensorAdapter` for that (org, sensor).
(traces to BC-2.11.005 precondition: "Sensor credentials are available for all resolved
(client, sensor) combinations" — here extended to: adapter is available for all resolved pairs)

### AC-007: BearerStaticAuthProvider correctly converts SensorAuth to AuthHeader
Given: A `SpecDrivenSensorAdapter` for a bearer_static sensor is called with `SensorAuth::BearerStatic { token: "test-token-abc" }`.
When: The adapter internally constructs `BearerStaticAuthProvider` and calls `PipelineExecutor::execute()`.
Then: The HTTP request emitted by `PipelineExecutor` carries `Authorization: Bearer test-token-abc`
header per ADR-028 §D10 (bearer_static sensors bypass the legacy credential resolver).
(traces to BC-2.01.013 precondition: "A valid TOML SensorSpec declaration exists with `auth_type`")

### AC-008: Adapter fetch returns OCSF-normalized Arrow RecordBatches
Given: A `SpecDrivenSensorAdapter::fetch()` call is made for any of the 4 sensors.
When: The `PipelineExecutor` successfully fetches and normalizes data.
Then: The returned `Vec<RecordBatch>` contains at least the OCSF hot fields: `category_uid`,
`class_uid`; the `_sensor` virtual column identifies the sensor_id; no raw API response fields
transit the return value without OCSF normalization.
(traces to BC-2.11.005 postcondition: "Sensor responses are normalized to OCSF via the OCSF
normalizer (CAP-003)")

### AC-009: No `todo!()` or `unimplemented!()` in adapter or boot step 9A (POL-12)
Given: The implementation is complete and all 4 sensor paths are exercised by tests.
When: The codebase is searched for `todo!()` or `unimplemented!()` in
`crates/prism-bin/src/spec_driven_adapter.rs` and the boot step 9A block in `boot.rs`.
Then: Zero occurrences found. All auth paths (plugin auth + bearer_static) are fully implemented,
not stubbed.
(traces to BC-2.22.001 invariant: "Pre-traffic gate prevents MCP requests before step 8 completes"
— a partial implementation would silently drop queries for unimplemented auth paths)

### AC-010: Adapter handles plugin-auth-failure (double-401 → AuthExpired error)
Given: CrowdStrike DTU clone returns 401 on the initial fetch AND on the refresh attempt.
When: `SpecDrivenSensorAdapter::fetch()` calls `PipelineExecutor::execute()`.
Then: The adapter returns `Err(SpecEngineError::AuthRefreshFailed)` (or the canonical error code
from the error taxonomy); no panic; the response envelope wraps the error correctly per BC-2.10.007.
(traces to BC-2.01.013 error case: "`PrismError::Sensor` — unrecognized API response structure")

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and relevant ADRs. Violations are P1
adversarial findings.

| Rule | Source | Enforcement |
|------|--------|-------------|
| `SpecDrivenSensorAdapter` MUST live in `prism-bin`, NOT `prism-sensors` | ADR-023 §D3 Forbidden Dependencies | Build fails if `prism-sensors/Cargo.toml` gains dep on `prism-spec-engine` |
| `prism-bin` may import both `prism-sensors` and `prism-spec-engine` | ADR-023 §Permitted Patterns | Existing workspace deps already present in prism-bin |
| `SensorAdapter::fetch(&dyn SensorAuth)` arg ignored for plugin-authed sensors | ADR-028 §D10 | Documented in impl with inline comment citing ADR-028 §D10 |
| `reqwest::Client` MUST set `.timeout(Duration::from_secs(30))` | CLAUDE.md Conventions | Adversary probes for missing timeout on every pass |
| `boot.step9a.adapter_registry_populated` MUST have a BC-2.16.002 catalog row | SAP-1 (standing probe) | Adversary greps `event_type =` on every pass |
| Boot step 9A MUST appear between steps 7.5b and 9 in ADR-022 §B table | BC-2.22.001 + ADR-022 | ADR-022 §B amendment required in same PR |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | PipelineExecutor and AuthProvider trait |
| `prism-sensors` (workspace) | current workspace path | SensorAdapter trait and SensorAuth types |
| `reqwest` | workspace version | HTTP client inside PipelineExecutor (used transitively) |
| `tokio` | workspace version | async runtime for PipelineExecutor::execute() |
| `arrow` | workspace version | RecordBatch return type |
| `tracing` | workspace version | boot.step9a.adapter_registry_populated event emission |

Version source: `Cargo.toml` workspace `[dependencies]` table. Do not pin versions independently.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | CREATE | New module: `SpecDrivenSensorAdapter` struct + `SensorAdapter` impl + `BearerStaticAuthProvider` wrapper |
| `crates/prism-bin/src/lib.rs` (or `main.rs`) | MODIFY | Add `mod spec_driven_adapter;` |
| `crates/prism-bin/src/boot.rs` | MODIFY | Add boot step 9A between step 7.5b and step 9; remove GAP-002-A comment; emit `boot.step9a.adapter_registry_populated` event |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Add catalog row for `boot.step9a.adapter_registry_populated` event per SAP-1 obligation |
| `.factory/specs/architecture/decisions/ADR-022-*.md` | MODIFY | Add boot step 9A to §B sequencing table |
| `.factory/specs/architecture/decisions/ADR-023-*.md` | MODIFY | Add `SpecDrivenSensorAdapter` to §Permitted Patterns list |

---

## Tasks

1. **Read** `crates/prism-sensors/src/traits.rs` — understand `SensorAdapter::fetch()` and `SensorAuth` signatures before writing any code.
2. **Read** `crates/prism-spec-engine/src/pipeline.rs` — understand `PipelineExecutor::execute()` signature and auth provider contract.
3. **Read** `crates/prism-spec-engine/src/auth.rs` — understand `AuthProvider` trait and existing implementations.
4. **Read** `crates/prism-bin/src/boot.rs` — locate step 7.5b (`plugin_auth_providers` map) and the GAP-002-A comment at step 9. Understand the `ResolvedSensorSpec` map structure from S-CONFIG-MULTI-TENANT-OVERRIDE-001.
5. **Write stub** `crates/prism-bin/src/spec_driven_adapter.rs` with `todo!()` bodies (Red Gate setup).
6. **Write Red Gate tests** (see AC-001, AC-002, AC-003, AC-005 test names) — all must fail (RED) before implementation.
7. **Implement** `SpecDrivenSensorAdapter`:
   - Fields: `sensor_spec: Arc<ResolvedSensorSpec>`, `auth_provider: Arc<dyn AuthProvider>`, `executor: Arc<PipelineExecutor>`.
   - Constructor `new(sensor_spec, auth_provider, executor)` — builds with `.timeout(Duration::from_secs(30))` reqwest::Client if PipelineExecutor needs one.
   - `SensorAdapter::fetch()` — ignore `&dyn SensorAuth` arg; call `self.executor.execute(&self.sensor_spec, Arc::clone(&self.auth_provider)).await`.
   - `SensorAdapter::sensor_type()` — return `SensorId::from(self.sensor_spec.sensor_id.as_str())`.
8. **Implement** `BearerStaticAuthProvider`:
   - A thin wrapper that implements `AuthProvider` by extracting the bearer token from the `SensorAuth::BearerStatic { token }` variant.
   - Used when `sensor_spec.auth_plugin` is `None` (bearer_static sensors: Armis, Claroty, Cyberint).
9. **Implement boot step 9A** in `boot.rs`:
   - After step 7.5b: iterate `resolved_specs.iter()` (product of `org_registry.all_orgs()` × `spec_catalog.sensors()`).
   - For each `(org_id, resolved_spec)`: look up `plugin_auth_providers.get(&resolved_spec.sensor_id)` → if Some, construct `SpecDrivenSensorAdapter` with plugin auth; if None and auth_type is `bearer_static`, construct with a `BearerStaticAuthProvider` sentinel (actual token extracted at fetch time).
   - Call `adapter_registry.register(org_id, sensor_id, Arc::new(adapter))`.
   - Emit `tracing::info!(event_type = "boot.step9a.adapter_registry_populated", sensor_count = M, org_count = K)`.
10. **Amend BC-2.16.002** — add catalog row for `boot.step9a.adapter_registry_populated` per SAP-1.
11. **Amend ADR-022 §B** — add boot step 9A to the sequencing invariant table.
12. **Amend ADR-023 §Permitted Patterns** — add `SpecDrivenSensorAdapter` pattern note.
13. **Run tests**: `just iter prism-bin` — all 4 Red Gate tests must turn GREEN.
14. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** (depends_on): Must merge first. Provides the `ResolvedSensorSpec` map and per-org overlay loading (ADR-029). The `resolved_specs` map produced by that story is the iteration source for boot step 9A. Read that story's output types before writing step 9A.
- **PLUGIN-MIGRATION-001-A** (depends_on): Must merge first. Deletes legacy hardcoded adapter construction from `init_registry_for_org`. After it merges, `AdapterRegistry` is unambiguously empty at boot step 9. This story fills it.
- **PLUGIN-MIGRATION-001-E** (depends_on): Must merge first. Wires `crowdstrike-oauth2.prx` WASM plugin to `PluginAuthProvider` construction at boot step 7.5b. The `plugin_auth_providers` HashMap is this story's source for CrowdStrike auth.
- **S-PLUGIN-PREREQ-B** (merged): Delivered `PipelineExecutor::execute()`. Read its implementation to understand the signature and async contract before writing the adapter.
- **S-5.01-FOLLOWUP-MCP-BOOT** (merged): Delivered the complete rmcp MCP server. Boot step 9 (MCP server start) that boot step 9A precedes is now fully implemented.
- **S-3.02-FOLLOWUP-RUNTIME** (merged): Filled `QueryEngine` execution pipeline. The `fan_out()` call in `materialization.rs` that will be unblocked by this story is already implemented.

---

## Open Questions

1. **BearerStaticAuthProvider sentinel at registration time**: When a bearer_static sensor is registered in boot step 9A, the actual bearer token is not known at registration time (it comes from the `SensorAuth` arg at `fetch()` time). The adapter should hold an `Arc<dyn AuthProvider>` that is constructed per-call when the `SensorAuth` arg arrives. Two options: (a) make `SpecDrivenSensorAdapter::fetch()` construct a fresh `BearerStaticAuthProvider` from `auth: &dyn SensorAuth` each call; (b) add an enum auth-strategy field. Option (a) is simpler and avoids per-fetch allocation overhead. Architect should confirm before implementation.

2. **ResolvedSensorSpec iteration at boot step 9A**: S-CONFIG-MULTI-TENANT-OVERRIDE-001 produces a `HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>` (or similar). The exact type needs to be verified by reading that story's output before implementing step 9A. The iteration must convert from `OrgSlug` to `OrgId` via `OrgRegistry::id_for(slug)`.

3. **SensorAuth compatibility for bearer_static path**: `SensorAdapter::fetch()` receives `&dyn SensorAuth`. The concrete type at call time will be something from `prism-credentials`. Confirm that `SensorAuth::BearerStatic` variant exists or identify the correct variant name before implementing `BearerStaticAuthProvider`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Sensor spec exists in catalog but no PluginAuthProvider AND no bearer_static auth → ambiguous | Boot logs E-SPEC-012 (auth type mismatch); adapter is NOT registered for that sensor; boot continues |
| EC-002 | org_registry has orgs but spec_catalog is empty | Boot step 9A produces 0 registrations; no error; boot continues to step 9 |
| EC-003 | PluginAuthProvider fails at step 7.5b (plugin file not found) | Step 7.5b already handles this per BC-2.22.001 failure path; step 9A skips sensors with missing auth providers |
| EC-004 | ResolvedSensorSpec has per-org base_url overlay but PipelineExecutor is not passed the overlay | CRITICAL: PipelineExecutor MUST receive the ResolvedSensorSpec (with overlay applied), not the base SensorSpec. Verify at AC-004. |
| EC-005 | Double-401 from sensor API during SpecDrivenSensorAdapter::fetch() | PipelineExecutor handles the refresh attempt internally (PLUGIN-PREREQ-E behavior); on second 401, returns SpecEngineError::AuthRefreshFailed. Adapter propagates this. |
| EC-006 | AdapterRegistry already has an entry for (org_id, sensor_id) when step 9A tries to register | Overwrite is acceptable (idempotent boot); log a WARN if overwrite occurs |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| BC files (4 BCs) | ~6,000 |
| crates/prism-bin/src/boot.rs (boot.rs is large) | ~12,000 |
| crates/prism-spec-engine/src/pipeline.rs | ~6,000 |
| crates/prism-sensors/src/traits.rs | ~2,000 |
| crates/prism-spec-engine/src/auth.rs | ~3,000 |
| ADR-022, ADR-023, ADR-028, ADR-029 (relevant sections) | ~8,000 |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 story (type reference) | ~4,000 |
| PLUGIN-MIGRATION-001-A + 001-E stories (context) | ~6,000 |
| Test outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~53,000 tokens (~21% of 256K context)** |

Within the 20-30% budget. Single-story delivery is viable.

---

## Forbidden Dependencies

The following dependencies MUST NOT appear in `crates/prism-bin/src/spec_driven_adapter.rs`
imports or `crates/prism-sensors/Cargo.toml`:

| Forbidden | Reason |
|-----------|--------|
| `prism-spec-engine` in `prism-sensors/Cargo.toml` | ADR-023 §D3 Forbidden Dependencies |
| Any hardcoded sensor name string (`"crowdstrike"`, `"armis"`, etc.) in match arms | ADR-023 §Permitted Patterns — spec-driven dispatch only |
| `reqwest::Client::new()` without `.timeout()` | CLAUDE.md production HTTP client rule |

If the implementer needs to verify these invariants, add a compile-fail test in
`tests/external/perimeter-violation/` following the S-PLUGIN-PREREQ-A pattern.

---

## Spec Updates Required (same PR)

Per E2E-DEMO-WIRING-PLAN.md §5, the following ADR amendments must ship in the same PR as the implementation:

| Document | Amendment |
|----------|-----------|
| ADR-022 §B | Add boot step 9A (`spec_driven_adapter_registry_populate`) between step 7.5b and step 9 |
| ADR-023 §Permitted Patterns | Add `SpecDrivenSensorAdapter` as the architecturally intended bridge pattern |
| ADR-022 §F | Update comment that `adapter_registry` is empty at step 9 — post-story it is populated |
| BC-2.16.002 Structured Event Catalog | Add `boot.step9a.adapter_registry_populated` row with sensor_count, org_count fields |

These are in-scope for the implementer per the production-grade principle (CLAUDE.md §Canonical Principle Rule 4 — AI-built defects are the AI's responsibility to fix; spec updates are the implementer's responsibility when the spec lags the implementation).

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
