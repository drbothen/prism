# Plugin System Audit — 2026-05-08

**Driver:** S-3.09 SUT debate surfaced BUG-S309-PLUGIN (4 built-in adapters bypass spec-engine). User elevated to "core architecture concern" and requested full plugin-system audit.

**Verdict:** **The plugin/spec-engine subsystem is far more broken than BUG-S309-PLUGIN suggests.** Of 5 stories merged on the plugin axis (S-1.11 through S-1.15), at least 3 are stub-merges shipping `unimplemented!()` panics in production code. The 4 sensor TOMLs are write-side only (no read tables, no fetch pipelines). No production code path loads sensor TOMLs at all. WASM plugin runtime exists but its action-plugin dispatch is stubbed. Infusion framework is 100% `unimplemented!()`. Hot reload watcher is `unimplemented!()`.

**Counts:** ~14 P0/P1 deferrals (vs 1 in BUG-S309-PLUGIN). 8 P0, 6 P1. 7 plugin VPs status:draft (Red Gate).

## 1. Spec-Engine Module Inventory

| Module | LOC | Status |
|---|---|---|
| spec_parser.rs | 561 | complete |
| validation.rs | 459 | complete |
| column_mapping.rs | 149 | complete |
| interpolation.rs | 311 | complete |
| custom_adapter.rs | 158 | complete (trait + registry; never called in prod) |
| pipeline.rs | 105 | **STUB** — PipelineExecutor::execute returns empty (pipeline.rs:54-66, :72-80) |
| hot_reload.rs | 275 | **PARTIAL** — HotReloadWatcher::start/stop unimplemented!() (hot_reload.rs:66, 72); process_spec_changes implemented but no watcher invokes it |
| config_manager.rs | 170 | complete |
| reload_config.rs | 256 | complete (function form) |
| add_sensor_spec.rs | 309 | complete BUT uses DIFFERENT SensorSpec schema than spec_parser.rs (parallel types in types.rs:51 vs spec_parser.rs:189) |
| list_sensor_specs.rs | 81 | complete |
| write_endpoint.rs | 442 | complete |
| org_scoped_store.rs | 124 | complete |
| infusion/ (7 files) | 805 | **100% UNIMPLEMENTED** — every method body is unimplemented!() (~14 stubs) |
| plugin/ (6 files) | 1853 | MOSTLY REAL — wasmtime+component model loaded; fire_alert/fire_case/fire_report stubbed (plugin/mod.rs:399-461); URL allowlist deferred (plugin/mod.rs:171) |
| proofs/ (8 files) | — | 7 Red Gate stubs |

## 2. TOML Spec Completeness

All 4 specs at sensors/{crowdstrike,armis,claroty,cyberint}.sensor.toml are 41-71 LOC and contain only [write_endpoints.*] blocks — no [[tables]], no [[tables.steps]], no read pipelines, no auth flow steps. Comment crowdstrike.sensor.toml:5 literally says "Read-side tables: implement in S-1.11." — never delivered.

| Sensor | Write endpoints | Read tables | Auth round-trip | Pagination | Rate limits |
|---|---|---|---|---|---|
| crowdstrike | 4 verbs | MISSING | declared but no steps | none | none |
| armis | 2 verbs | MISSING | declared but no steps | none | none |
| claroty | 2 verbs | MISSING | declared but no steps | none | none |
| cyberint | 2 verbs | MISSING | declared but no steps | none | none |

Reference of "complete" spec exists only at crates/prism-spec-engine/examples/demo_spec_loading.rs:36-79 (in-source string literal, never persisted).

## 3. Production Wiring Map

**No production code path outside prism-spec-engine loads sensor TOMLs.** Search for `parse_spec_directory|SpecLoader|ConfigManager::new|PluginRuntime::new|InfusionRegistry::new` outside the crate returns: only fuzz/fuzz_targets/spec_parser.rs. The MCP crate (crates/prism-mcp/src/lib.rs:1-10) is a 10-line stub. **No binary exists that constructs ConfigManager/PluginRuntime/SpecLoader for runtime use.**

`init_registry_for_org` at crates/prism-sensors/src/lib.rs:166-197 instantiates the 4 hardcoded Rust adapters — bypasses spec-engine entirely (BUG-S309-PLUGIN). `WriteEndpointRegistry::new()` is constructed only in tests (write_pipeline.rs:427, :491).

## 4. WASM Plugin Runtime (S-1.15 / AD-019)

Runtime exists with real wasmtime engine, Component Model linker, epoch-interruption, memory limits, sandbox, host-functions registration (plugin/mod.rs:84-117). load_plugin, enrich_single, enrich_batch are real. NOT wired into production — no caller outside tests. fire_alert/fire_case/fire_report action-plugin dispatch stubbed and deferred to S-4.08 (plugin/mod.rs:399-461). Per-plugin URL allowlist deferred (plugin/mod.rs:171). 7 of 8 proofs in proofs/ are Red Gate stubs.

## 5. Infusion Framework (SS-19)

**100% unimplemented.** crates/prism-spec-engine/src/infusion/mod.rs:13 documents: "All method bodies are unimplemented!(). Implementation lives in S-1.14." S-1.14 reports status: merged but the code shows ~14 distinct unimplemented!() panics across loader.rs, cache.rs, udf.rs, enrich_descriptor.rs, plugin_bridge.rs, and all 3 source types. **S-1.14 is a stub-merge.**

## 6. Hot Reload (BC-2.16.005..010)

HotReloadWatcher::start and stop are both unimplemented!() at hot_reload.rs:66, 72. process_spec_changes (hot_reload.rs:84-275) is real and would be the body of the watcher — but no watcher exists to call it. ArcSwap ConfigManager correct (config_manager.rs). reload_config (manual MCP-tool path) implemented. Mode-change detection "not yet wired into process_spec_changes" (hot_reload.rs:271). **Filesystem watcher per AD-018: NOT BUILT.** tests/hot_reload_tests.rs:1085 #[should_panic(expected = "not yet implemented")] codifies the hole.

## 7. Plugin-Related Stories

| Story | Status | Truth |
|---|---|---|
| S-1.11 (spec loading) | merged | partially true — parser real, PipelineExecutor::execute is stub |
| S-1.12 (hot reload) | merged | **STUB-MERGE** — watcher unimplemented |
| S-1.13 (write specs) | merged | true — write_endpoint.rs real |
| S-1.14 (infusion) | merged | **STUB-MERGE** — entire module unimplemented!() |
| S-1.15 (WASM runtime) | merged | partially true — runtime real, action dispatch stubbed |
| S-3.02 (query+materialization) | merged PR #129 | merged but doesn't load specs from disk |

## 8. Plugin-Related BCs

21 BCs declared (BC-2.16.001-010, BC-2.17.001-006, BC-2.19.001-005). BC-2.16.001 (parser) delivered. BC-2.16.002 (multi-step pipeline): postcondition unmet. BC-2.16.005 (reload_config tool): delivered as function. BC-2.16.007 (hot reload): partial. BC-2.16.008 (add_sensor_spec): delivered. BC-2.17.001-006 (plugin runtime): mostly delivered for sensor/infusion plugins, action-plugin path stubbed. BC-2.19.001-005 (infusion): all unmet — unimplemented!() panics.

## 9. Plugin-Related ADs

| AD | Decision | Status |
|---|---|---|
| AD-007 | ArcSwap for hot config reload | delivered |
| AD-015 | No DataFusion in spec-engine | delivered |
| AD-017 | AI-opaque credentials | delivered |
| AD-018 | Auto filesystem watching (notify crate, 500ms debounce) | **UNIMPLEMENTED** (hot_reload.rs:66) |
| AD-019 | WASM Component Model plugins | partial — runtime real, no production wiring, action dispatch stubbed |
| AD-022 | PrismQL write operations | delivered |

## 10. Plugin-Related VPs (all status:draft, Red Gate)

VP-023 (parser no-panic), VP-032 (hot-reload atomicity), VP-040..043 (plugin sandbox), VP-048/049 (infusion). 7 of 8 proofs are Red Gate stubs.

## P0/P1 Deferrals Discovered

**P0:**
1. TD-PLUGIN-P0-001: PipelineExecutor::execute is stub (pipeline.rs:54-66). BC-2.16.002 unmet.
2. TD-PLUGIN-P0-002: Infusion framework 100% unimplemented!(). S-1.14 stub-merged.
3. TD-PLUGIN-P0-003: HotReloadWatcher::start/stop unimplemented!() (hot_reload.rs:66, 72). AD-018 not delivered.
4. TD-PLUGIN-P0-004: Sensor TOMLs lack read-side declarations — all 4 are write-only.
5. TD-PLUGIN-P0-005: Two parallel SensorSpec types (spec_parser.rs:189 vs types.rs:51) — silent drift.
6. TD-PLUGIN-P0-006: WriteEndpointRegistry::new() never constructed in production.
7. TD-PLUGIN-P0-007: No binary loads sensors/*.toml. prism-mcp/src/lib.rs is 10-line stub. (Umbrella over BUG-S309-PLUGIN.)
8. TD-PLUGIN-P0-008: Action-plugin WASM dispatch stubbed (plugin/mod.rs:399-461) deferred to S-4.08. BC-2.17.x partially unmet.

**P1:**
9. TD-PLUGIN-P1-001: 7 Red Gate VP proofs unwritten (VP-032/040/041/042/043/048/049).
10. TD-PLUGIN-P1-002: Per-plugin URL allowlist deferred (plugin/mod.rs:171). Sandbox open-by-default.
11. TD-PLUGIN-P1-003: CustomAdapterRegistry never registered in production — escape hatch unwired.
12. TD-PLUGIN-P1-004: Mode-change detection not wired into process_spec_changes (hot_reload.rs:271).
13. TD-PLUGIN-P1-005: infusion/plugin_bridge.rs:6 admits "If S-1.15 is not yet built, this stub panics" — fragile coupling.

## Recommendation (Ordered Remediation)

1. BUG-S309-PLUGIN-fix (file MCP-server binary that constructs ConfigManager + parse_spec_directory(./sensors/)).
2. S-1.11-FOLLOWUP — implement PipelineExecutor::execute with real HTTP client injection.
3. Backfill 4 sensor TOMLs with read-side [[tables]]/[[tables.steps]] matching demo_spec_loading.rs:36 reference.
4. Unify SensorSpec types — pick spec_parser::SensorSpec as canonical, retire types::SensorSpec.
5. S-1.14-REDO — implement infusion framework. Or formally retire to Wave 4+.
6. S-1.12-FOLLOWUP — wire notify crate into HotReloadWatcher::start per AD-018.
7. S-1.15-FOLLOWUP — implement action-plugin fire_* dispatch + URL allowlist.
8. Write 7 Red Gate Kani/proptest proofs.
9. Story-index hygiene sweep — flip S-1.12/S-1.14/S-1.15 status from merged to partial-merge.

## Honest Assessment

The "unify disparate APIs via plugins to a unified query language" core promise is **not delivered**. The spec engine has the parser, validator, descriptor types, column mapper, write-endpoint registry — all real and well-built. But the *runtime* — the part that actually fetches data through declared pipelines, hot-reloads specs, enriches with infusions, and routes via spec-driven custom adapters — is stub-grade. The 4 hardcoded Rust adapters do all the work in production with TOMLs nobody reads. **This is closer to "we built the schema for plugins" than "we have plugins."**
