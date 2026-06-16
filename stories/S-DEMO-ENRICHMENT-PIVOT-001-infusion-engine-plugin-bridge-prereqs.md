---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-001
title: "Infusion Engine Plugin-Bridge Prerequisites — Forward-Subset of S-1.14-REDO for Demo"
wave: 5
epic_id: E-DEMO
priority: P2
status: ready
version: "1.11"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-06-15T00:00:00Z"
tdd_mode: strict
subsystems: [SS-19, SS-11, SS-17]
# Subsystem anchor justifications:
#   SS-19 (Infusion Enrichment Framework) owns InfusionLoader and plugin_bridge per
#   ARCH-INDEX Subsystem Registry — the loader and bridge are core SS-19 scope.
#   SS-11 (Query Engine) owns DataFusion UDF registration; InfusionRegistry::udf_descriptors()
#   is consumed by prism-query to register enrichment UDFs in the SessionContext.
#   SS-17 (Plugin Runtime / WASM) owns the WASM plugin ABI; plugin_bridge delegates to
#   PluginRuntime::enrich_single which lives in SS-17 scope.
target_module: prism-spec-engine
crates_touched: [prism-spec-engine, prism-query]
# BC status: LIGHT PO CONFIRMATION REQUIRED (story-writer cannot do PO work)
# BC-2.19.001 v1.7 (active) and BC-2.19.003 v1.3 are the authoritative anchors for this
# FORWARD-SUBSET. BCs exist and bidirectional AC↔BC traces are present. PO confirmed
# version pins updated to v1.7 (PIVOT-001 LOW-2 regression fix, 2026-06-15). Two-phase
# wiring: load_all (PARSE PHASE) returns (Vec<InfusionSpec>, Vec<InfusionError>) — does NOT
# construct PluginInfusionSource; load_spec_with_runtime (RUNTIME PHASE) builds
# PluginInfusionSource and attaches it as descriptor.source.
# BC-2.19.003 v1.3. No further PO sign-off required before test-writer dispatch.
behavioral_contracts: [BC-2.19.001, BC-2.19.003]
# BC array propagation note: BC-2.19.001 covers UDF registration (DataFusion scalar UDF
# per field); BC-2.19.003 covers is_api_backed() gate for plugin-type infusions.
# Both are cited by ACs below (bidirectional trace satisfied).
verification_properties: []
graduates: []
# graduates_note: This story is a FORWARD-SUBSET of S-1.14-REDO. It implements only
# the plugin-type InfusionLoader path, plugin_bridge, and DataFusion UDF wiring for
# plugin-type infusions. When S-1.14-REDO lands in Wave 5, it extends this subset with
# the full engine (MMDB, CSV, JSON-lookup, three-tier cache, VP-048/049). S-1.14-REDO
# must be annotated to reflect that it now REDOs only what this story leaves unimplemented.
depends_on:
  - S-1.14
  # Dependency anchor: S-1.14 (partial-merge state) established the infusion scaffolding —
  # InfusionSpec, InfusionRegistry, InfusionSource trait, plugin_bridge.rs with
  # unimplemented!() stubs, and InfusionLoader::parse skeleton. This story builds on
  # that scaffolding by implementing only the plugin-type path. Without the S-1.14
  # scaffold, there is no compile target for the implementation.
blocks:
  - S-DEMO-ENRICHMENT-PIVOT-002
  # Blocks anchor: 002 authors threatintel.infusion.toml and nvd.infusion.toml with
  # type = "plugin" and expects InfusionLoader to parse and load them into a working
  # InfusionRegistry with registered DataFusion UDFs. Without the plugin-bridge
  # and UDF registration wired, 002 cannot write integration tests.
  - S-DEMO-ENRICHMENT-PIVOT-003
  # Blocks anchor: 003 depends transitively via 002.
points: 5
# Points justification: Forward-subset of S-1.14-REDO scope.
#   1. InfusionLoader::parse + load_all for source.type = "plugin" (subset only): 1.5 pts
#   2. plugin_bridge::enrich_via_plugin -> PluginRuntime::enrich_single wiring: 1.5 pts
#   3. DataFusion UDF registration for plugin-type infusion descriptors
#      (InfusionRegistry::udf_descriptors() wired into prism-query): 1.5 pts
#   4. InfusionRegistry::is_api_backed() check (BC-2.19.003 gate): 0.5 pts
#   Total: 5 pts (vs 8 pts for full S-1.14-REDO; reduced because MMDB/CSV/JSON-lookup
#   and three-tier cache are deferred to S-1.14-REDO)
estimated_days: 2
risk: MEDIUM
# Risk justification: plugin_bridge delegates to PluginRuntime (S-1.15 WASM runtime,
# partial-merge). If S-1.15 stubs are incomplete, enrich_via_plugin may need an
# annotated todo!(). DataFusion UDF registration requires reading DataFusion 53.1
# SessionState extension API before writing. The subsystem perimeter between
# prism-spec-engine and prism-query must be respected: spec-engine provides descriptors,
# query registers UDFs — no reverse dependency.
red_gate_tests: 7
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "PluginInfusionSource::enrich_single/enrich_batch (U5/Ruling 2): implement via the
     existing InfusionSource trait methods in crates/prism-spec-engine/src/infusion/plugin_bridge.rs
     (stubs at lines 24-41 with unimplemented!()). Do NOT add a new free-function
     plugin_bridge::enrich_via_plugin. Delegate to PluginRuntime::enrich_single
     (plugin/mod.rs ~904). If S-1.15 PluginRuntime is not operational at dispatch time (e.g.,
     source is NotLoaded), enrich_single returns None and logs a WARN — it does NOT return an
     error (map-log-None path per BC-2.19.001 v1.5 / CRIT-3 closure). InfusionError::PluginRuntimeNotAvailable
     does NOT exist in prism-core error.rs — do not reference it. Document in PR."
  - "DataFusion async UDF registration (U3/U4/U8 — RESEARCH CONFIRMED DF 53.1.0):
     Implement AsyncScalarUDFImpl with invoke_async_with_args (real enrichment call) and
     invoke_with_args returning not_impl_err!(...). Register via:
     ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf()).
     NO analyzer/optimizer/physical-optimizer rules are needed — DefaultPhysicalPlanner
     handles async UDFs natively in DF 53.1. DO NOT use ScalarUDF::from(impl) (sync-only).
     HALLUCINATED — DO NOT USE: AsyncFunctionRule, async_function_rule.rs, enable_async_udf
     flag, concurrent_async_udf_tasks option, GLOBAL_ASYNC_UDF_SEMAPHORE — none exist in DF 53.1.
     block_in_place+block_on is fallback-only (safe ONLY on multi-thread runtime per AD-013).
     Methods invoke/invoke_batch were REMOVED by DataFusion ~48 — do not cite them."
  - "wasmtime post_return (U2 — RETRACTED): post_return is NOT called in the merged runtime.
     plugin/mod.rs ~L970 carries the explicit comment: '// post_return removed — no longer
     needed in wasmtime >=44 (no-op, deprecated).' Do NOT call TypedFunc::post_return after
     enrich-single calls. The real runtime uses the UNTYPED component::Val path via
     instance.get_func(store, 'enrich-single').call(store, params, results) — NOT
     get_typed_func. Following the pre-merge TypedFunc pattern would reintroduce a removed
     API; conform to the existing plugin/mod.rs implementation exactly."
  - "is_api_backed() is ALREADY implemented (InfusionRegistry::is_api_backed) (U6): scope as a
     regression/confirmation test — NOT a red-gate-new test. The implementation exists;
     the test verifies it returns true for plugin-type UDFs. Adjust points note accordingly."
  - "Error surface (U7): use the existing InfusionError enum surface (loader.rs returns
     Result<_, InfusionError>) and E-INFUSE-NNN codes from error-taxonomy.md. Do NOT
     invent SpecEngineError::PluginRuntimeNotAvailable, ::UnsupportedInfusionSourceType,
     or ::InfusionValidation. If a new variant is needed, scope it as: add variant to
     InfusionError + add row to error-taxonomy.md — explicitly flagged in PR."
  - "Scope boundary: this story implements ONLY source.type = 'plugin' path in
     InfusionLoader. Do NOT implement mmdb/csv/json_lookup paths — those belong to
     S-1.14-REDO. InfusionLoader::load_all must route 'plugin' to PluginInfusionSource
     and return Err(InfusionError::UnknownSourceType { type_name }) / E-INFUSE-004 for
     unrecognized types (see error-taxonomy.md §E-INFUSE-004)."
  - "Forward-subset graduation contract: after this story merges, S-1.14-REDO's scope
     annotation MUST be updated to explicitly exclude the plugin-type loader path
     implemented here. See §S-1.14-REDO Annotation section."
traces_to: [D-1109, WO-D1109]
supersedes: []
---

# S-DEMO-ENRICHMENT-PIVOT-001 v1.11: Infusion Engine Plugin-Bridge Prerequisites

Wire the plugin-backed infusion path forward-subset from S-1.14-REDO's scope so the
demo enrichment chain (S-DEMO-ENRICHMENT-PIVOT-002/003) can be built and tested before
the full S-1.14-REDO Wave 5 delivery.

**Sequencing context (D-1109, WO-D1109):** Slots AFTER the capability-discovery block
(S-5.02→S-5.03→S-5.04 + S-3.13, D-1107) and BEFORE S-DEMO-ENRICHMENT-PIVOT-002, as
part of the following demo objective chain:
```
T5 (PR #185) → T6 (S-DEMO-MULTI-TENANT-DTU-001) → T8
  → capability-discovery block (D-1107)
    → S-DEMO-ENRICHMENT-PIVOT-001 (this story — engine prereqs)
      → S-DEMO-ENRICHMENT-PIVOT-002 (infusion specs + plugins)
        → S-DEMO-ENRICHMENT-PIVOT-003 (IOC stamping + pivot query)
          → T11 → T13 capstone
```

**Scope (forward-subset of S-1.14-REDO):**
- `InfusionLoader::parse` and `load_all` for `source.type = "plugin"` specs ONLY
- `plugin_bridge::enrich_via_plugin` calling `PluginRuntime::enrich_single`
- DataFusion UDF registration wiring in `prism-query` for plugin-type infusion descriptors
- `InfusionRegistry::is_api_backed()` check for BC-2.19.003

**What this story does NOT implement** (deferred to S-1.14-REDO, Wave 5):
- `source.type = "maxmind_mmdb"` / `"csv"` / `"json_lookup"` paths in InfusionLoader
- Three-tier cache (`InfusionLruCache`, `QueryScopedInfusionCache`, RocksDB Tier 3)
- VP-048 Kani proof and VP-049 proptest (full dedup coverage)
- Hot reload integration (S-1.12-FOLLOWUP)

**Now implemented in PIVOT-001** (no longer deferred to S-1.14-REDO):
- `validate_credentials` — checks every `CredentialRef.env_var` is non-empty (spec-load-time, no live credential store needed); wired into `InfusionLoader::parse`; tested by `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` and `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential`
- `validate_pipe_stage_columns` — checks every `pipe_stage.adds_columns` name appears in `spec.fields`; wired into `InfusionLoader::parse`; tested by `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` and `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference`

---

## Narrative

As the Prism enrichment pipeline, I want `InfusionLoader` to parse plugin-type infusion
specs and `plugin_bridge::enrich_via_plugin` to call the WASM plugin runtime for enrichment
lookups — with DataFusion UDFs registered in the query engine's `SessionContext` — so that
analyst queries using `| enrich threat_intel(ioc_value)` resolve correctly against
plugin-backed DTU HTTP services without requiring the full three-tier cache or non-plugin
source types.

---

## Behavioral Contracts

| BC | Title | Key Clauses |
|----|-------|-------------|
| BC-2.19.001 v1.7 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | Postcondition: each field in `[[infusion.fields]]` produces exactly one `InfusionUdfDescriptor` entry registered in `SessionContext`. Two-phase wiring: PARSE PHASE (`InfusionLoader::load_all`) returns `(Vec<InfusionSpec>, Vec<InfusionError>)` — it does NOT construct `PluginInfusionSource` and does NOT attach it as `descriptor.source`; descriptors carry `Arc<NullSource>` as placeholder after this phase. RUNTIME PHASE (`InfusionRegistry::load_spec_with_runtime`) builds `PluginInfusionSource` (carrying `plugin_id`/`config` from the spec) and attaches it as `descriptor.source` — `plugin_id`/`config` live on `PluginInfusionSource` via `descriptor.source`, NOT on `InfusionUdfDescriptor` directly. A descriptor still carrying `Arc<NullSource>` at query execution time is a loading defect. AC-002 tests the parse-phase outcome only; AC-004 tests the runtime bridge delegation. |
| BC-2.19.003 v1.3 | API-Backed Infusion UDFs Rejected in Detection Rule Filters — E-RULE-012 | Postcondition: `is_api_backed("threat_score")` returns `true` for plugin-type infusions; detection rule loader rejects with E-RULE-012. PO-confirmed version v1.3 at D-1166 2026-06-14. |

---

## Acceptance Criteria

### AC-001 — InfusionLoader::parse accepts source.type = "plugin"
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor)

Given a `.infusion.toml` file with `source.type = "plugin"`, `plugin_ref = "some-plugin.prx"`,
and `[[infusion.fields]]` declaring output field(s),
when `InfusionLoader::parse` runs,
then it returns a valid `InfusionSpec` with `source.type = PluginSource` variant and the
declared fields present; no `SpecEngineError` is returned.

Red Gate: `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec`

### AC-002 — InfusionLoader::load_all builds InfusionRegistry with plugin-type descriptors (parse-phase outcome)
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor; v1.5 two-phase wiring)

Given a directory containing a valid plugin-type `.infusion.toml`,
when `InfusionLoader::load_all` runs,
then the returned `InfusionRegistry` contains `InfusionUdfDescriptor` entries for each
declared output field, and `registry.udf_descriptors()` returns a non-empty `Vec`.

**Parse-phase vs runtime-wiring distinction (BC-2.19.001 v1.5):**
`InfusionLoader::load_all` is the PARSE PHASE: it produces `InfusionUdfDescriptor` entries
with `plugin_id` and `config` populated from the spec TOML. At this stage the descriptor
carries `Arc<NullSource>` as a placeholder — the real `Arc<PluginInfusionSource>` (with
`plugin_id` and `config` wired as live runtime fields) is NOT attached by `load_all`; it is
attached by `InfusionRegistry::load_spec_with_runtime` (and future boot-time runtime wiring).

This test therefore asserts the PARSE-PHASE outcome only:
- `registry.udf_descriptors()` is non-empty (descriptors exist for each declared field)
- Each descriptor's SOURCE (`PluginInfusionSource`, accessed via `descriptor.source`) carries
  `plugin_id` and `config` populated from the TOML spec — these fields live on
  `PluginInfusionSource`, NOT on `InfusionUdfDescriptor` directly

A descriptor still carrying `Arc<NullSource>` at query-execution time (i.e., where
`load_spec_with_runtime` was not invoked) is a loading defect per BC-2.19.001 v1.5
postcondition; but that defect is NOT tested here — it is the responsibility of the
`load_spec_with_runtime` integration test.

**Test name scoping note:** `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`
names the parse-phase outcome correctly. Do NOT rename it to imply full runtime-wiring —
the test verifies `load_all` output, not `load_spec_with_runtime` output.

Red Gate: `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`

### AC-003 — DataFusion async UDF registration wires plugin descriptors into SessionContext
(traces to BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF)

Given an `InfusionRegistry` with plugin-type descriptors returned from `load_all`,
when `register_infusion_udfs(ctx, registry.udf_descriptors())` is called in `prism-query`,
then each descriptor is registered as a DataFusion scalar UDF in the `SessionContext`
via `ctx.register_udf(AsyncScalarUDF::new(Arc<dyn AsyncScalarUDFImpl>).into_scalar_udf())`,
and the UDF is callable by name in a DataFusion query plan.

**RESEARCH RESULT — DataFusion 53.1.0 async UDF path (authoritative):**

`ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` ALONE is sufficient
to make `invoke_async_with_args` execute. The `AsyncFuncExec` rewrite is built into
`DefaultPhysicalPlanner`. NO additional analyzer rule, optimizer rule, physical-optimizer
rule, or config flag is required. prism's `new_with_config_rt` + Tokio multithread
(AD-013) already satisfy the prerequisites.

Required async UDF trait impl:
```
impl AsyncScalarUDFImpl for InfusionAsyncUdf {
    async fn invoke_async_with_args(&self, args: ScalarFunctionArgs<'_>)
        -> Result<ColumnarValue> { /* real enrichment call */ }

    // Sync fallback — MUST return not_impl_err! to force async execution path
    fn invoke_with_args(&self, _args: ScalarFunctionArgs<'_>) -> Result<ColumnarValue> {
        not_impl_err!("InfusionAsyncUdf: use invoke_async_with_args (async context only)")
    }
}
```

**HALLUCINATED SYMBOLS — DO NOT USE (do not exist in DataFusion 53.1):**
These were produced by a prior research pass and are confirmed fictional:
- `AsyncFunctionRule` — does not exist; no such type in datafusion 53.1
- `async_function_rule.rs` — does not exist as a separate module
- `enable_async_udf` config flag — does not exist
- `concurrent_async_udf_tasks` config option — does not exist
- `GLOBAL_ASYNC_UDF_SEMAPHORE` — does not exist

If a stub returns a constant without calling `invoke_async_with_args`, the INCORRECTLY-wrapped
async UDF fails LOUDLY with the `not_impl_err!` from `invoke_with_args` — this is the
genuine false-green risk, not a silent pass. See anti-false-green hardening below.

**Anti-false-green hardening (AC-003):**
The Red Gate test MUST assert on an async-ONLY observable to prevent a no-op stub from
passing. The test must:
1. Register a test async UDF implementation backed by a mock `PluginRuntime`-like call
   that returns a known sentinel value (e.g., `"CVE-PIVOT-TEST-SENTINEL"`) that can ONLY
   originate from the async enrichment round-trip — not from any constant or sync path.
2. Execute a DataFusion query through `SessionContext` that invokes the registered UDF.
3. Assert the returned `RecordBatch` column contains the sentinel value.
4. Include a call counter (e.g., `Arc<AtomicUsize>`) in the mock that is asserted > 0 after
   execution — proving `invoke_async_with_args` was actually called, not `invoke_with_args`.

A stub that hard-codes a return value cannot pass both the sentinel assertion and the counter
assertion simultaneously.

Red Gate: `test_BC_2_19_001_plugin_udfs_registered_in_session_context`

### AC-004 — PluginInfusionSource::enrich_single delegates to PluginRuntime::enrich_single
(traces to BC-2.19.001 postcondition — plugin-type source executes via plugin bridge)

Given `PluginInfusionSource::enrich_single` (the InfusionSource trait impl in
`crates/prism-spec-engine/src/infusion/plugin_bridge.rs`, stubs at lines 24-41)
called with a valid plugin-type `InfusionSpec` and an input value,
when `PluginRuntime::enrich_single` is available (S-1.15 operational, plugin/mod.rs ~904),
then `enrich_single` delegates to `PluginRuntime::enrich_single` using the CONFIRMED
production signature: `runtime.enrich_single(plugin_id: &str, input_value: &str,
input_type: &str, config: &PluginConfigMap) -> Result<Option<Value>, PluginError>`.

The `InfusionSource` trait's `enrich_single(&self, input: &str, input_type: &str)` does NOT
carry `plugin_id` or `PluginConfigMap` — `PluginInfusionSource` must hold them as fields
(e.g., `plugin_id: String`, `config: PluginConfigMap`) set at construction time in
`InfusionLoader::load_all`. The bridge maps `PluginError → InfusionError`.

IMPORTANT — DO NOT USE `TypedFunc`:
The runtime uses the UNTYPED `component::Val` path:
`instance.get_func(store, "enrich-single").call(store, &params, &mut results)`
using `wasmtime::component::Val::S32(...)` params. `get_typed_func` is NOT used and
`TypedFunc::post_return` is explicitly REMOVED (plugin/mod.rs ~L970 comment:
"post_return removed — no longer needed in wasmtime >=44 (no-op, deprecated)").
Conform exactly to the existing plugin/mod.rs implementation — do NOT call post_return.

When `PluginRuntime` is not yet available at dispatch time (e.g., source is `NotLoaded`),
`enrich_single` returns `None` and emits a `tracing::warn!` — it does NOT return an error
variant. This is the map-log-None path per BC-2.19.001 v1.5 / CRIT-3 closure.
`InfusionError::PluginRuntimeNotAvailable` does NOT exist in `prism-core/src/error.rs` and
MUST NOT be referenced. Real plugin wiring lands in PIVOT-002/003 / S-1.14-REDO;
the unavailable-runtime contract is `None` + WARN log.

Red Gate: `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`

### AC-005 — is_api_backed() returns true for plugin-type infusion UDFs (regression confirmation)
(traces to BC-2.19.003 postcondition — API-backed UDFs rejected in detection rule filters)

NOTE (U6): `InfusionRegistry::is_api_backed` is ALREADY IMPLEMENTED (the
`InfusionRegistry::is_api_backed` method in `crates/prism-spec-engine/src/infusion/mod.rs`).
This AC is a regression/confirmation test — NOT a red-gate-new implementation task. The test
exercises the existing code path to verify it returns `true` for plugin-type UDFs. Adjust
points estimate accordingly (0.5 pts → ~0.2 pts for test only).

Given an `InfusionRegistry` loaded with a plugin-type spec whose field is named `threat_score`,
when `registry.is_api_backed("threat_score")` is called,
then it returns `true` (verified against the existing `InfusionRegistry::is_api_backed` implementation).

Given an unknown UDF name not present in the registry,
when `registry.is_api_backed("unknown_field")` is called,
then it returns `false` (existing default behavior).

Red Gate: `test_BC_2_19_003_is_api_backed_true_for_plugin_type`

### AC-006 — Grammar doc updated: enrich pipe-stage table and EBNF use function-call form
(traces to BC-2.19.001 postcondition — query language surface matches implemented parser)

NOTE (U1/Ruling 1): D-1109 Ruling 1 establishes that the implemented PrismQL parser
(AD-020/S-1.14) uses `enrich infusion(field_path)` function-call form — NOT `ENRICH ident ON field`.
The grammar doc `.factory/specs/domain-spec/prismql-grammar.md` still contains the brownfield-era
`ENRICH ident ON field` form in its pipe-stage table (~line 330) and EBNF stage rule (~line 767).

Given `.factory/specs/domain-spec/prismql-grammar.md` after this story,
when the pipe-stage table (§Pipe Stages, ~line 330) is inspected,
then the `enrich` row reads: `enrich infusion(field_path)` — NOT `enrich X on Y`.

And when the EBNF stage rule (~line 767) is inspected,
then the stage production reads: `ENRICH ident "(" field_path ")"` — NOT `ENRICH ident ON field`.

SCOPE NOTE: The parser implementation requires NO change — `enrich infusion(field)` is already the
implemented form (AD-020/S-1.14 source of truth). Only the grammar doc artifact is amended.
This is a `.factory/` artifact edit — applied via Edit tool by story-writer/state-manager.

Red Gate: N/A (doc amendment — adversary verifies grammar doc consistency post-merge with a
text grep: `grep -n "ENRICH.*ON\|enrich.*\bon\b" .factory/specs/domain-spec/prismql-grammar.md`
MUST return 0 matches in the EBNF/pipe-stage sections after this story)

---

## Red Gate Test Plan

| # | Test Name | Crate | BC Clause | Type |
|---|-----------|-------|-----------|------|
| 1 | `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | prism-spec-engine | BC-2.19.001 postcondition | unit |
| 2 | `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | prism-spec-engine | BC-2.19.001 postcondition | unit |
| 3 | `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | prism-query | BC-2.19.001 postcondition | unit |
| 4 | `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime` | prism-spec-engine | BC-2.19.001 postcondition | unit (tests PluginInfusionSource::enrich_single, the InfusionSource trait impl — NOT a free function enrich_via_plugin) |
| 5 | `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | prism-spec-engine | BC-2.19.003 postcondition | unit (REGRESSION — InfusionRegistry::is_api_backed already implemented; test confirms existing behavior) |
| 6 | `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` / `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` | prism-spec-engine | BC-2.19.001 postcondition (`validate_credentials`) | unit — **IMPLEMENTED in PIVOT-001** (wired into `InfusionLoader::parse`; EC-006) |
| 7 | `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` / `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` | prism-spec-engine | BC-2.19.001 postcondition (`validate_pipe_stage_columns`) | unit — **IMPLEMENTED in PIVOT-001** (wired into `InfusionLoader::parse`; EC-007) |

---

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~3,500 |
| S-1.14 spec (InfusionSpec/Registry/Source types context) | ~1,500 |
| `prism-spec-engine/src/infusion/loader.rs` (plugin path only) | ~1,500 |
| `prism-spec-engine/src/infusion/plugin_bridge.rs` | ~600 |
| `prism-spec-engine/src/infusion/mod.rs` + registry | ~800 |
| `prism-query/src/engine.rs` (UDF registration wiring) | ~1,000 |
| BC files (BC-2.19.001, BC-2.19.003) | ~2,000 |
| S-1.14-REDO spec (graduation relationship context) | ~800 |
| Test files (5 red gate stubs × ~40 lines each; AC-005 regression ~20 lines; EC-006/EC-007 validator tests × 4 tests ~25 lines each) | ~720 |
| Tool outputs (nextest, clippy) | ~1,000 |
| **Total estimate** | **~13,400** |

At ~200k context window, this is ~6.7% — well within the 20-30% ceiling.

---

## Tasks

Implementation checklist (TDD order — write failing tests before each implementation step):

**Pre-flight: read substrate before writing anything**

- [ ] Read `crates/prism-spec-engine/src/infusion/loader.rs` — confirm `InfusionLoader::parse`,
  `load_all`, and the `InfusionSource` trait interface from S-1.14 partial-merge
- [ ] Read `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` — confirm
  `PluginInfusionSource::enrich_single` and `enrich_batch` stub signatures (InfusionSource
  trait impl, stubs at lines 24-41 with unimplemented!()) and `PluginRuntime` import path
- [ ] Read `crates/prism-query/src/engine.rs` — identify SessionContext construction site
  where `register_infusion_udfs` call should be inserted
- [ ] Read `crates/prism-spec-engine/src/infusion/mod.rs` — confirm `InfusionRegistry`,
  `InfusionUdfDescriptor`, and `udf_descriptors()` method exist from S-1.14

**Phase 1: plugin-type InfusionLoader path**

- [ ] Write failing test 1 (FAIL first): `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec`
- [ ] Implement `InfusionLoader::parse` for `source.type = "plugin"`:
  validate `plugin_ref` field present, at least one `[[infusion.fields]]` entry, return `InfusionSpec`
- [ ] Write failing test 2 (FAIL first): `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`
- [ ] Implement `InfusionLoader::load_all` plugin branch:
  route `source.type = "plugin"` to `PluginInfusionSource`; other types return
  `Err(InfusionError::UnknownSourceType { type_name })` / E-INFUSE-004 (stub — full impl
  in S-1.14-REDO; see error-taxonomy.md §E-INFUSE-004 and error.rs InfusionError::UnknownSourceType)
- [ ] Verify tests 1-2 pass

**Phase 2: plugin_bridge (InfusionSource trait impl)**

- [ ] Write failing test 4 (FAIL first): `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`
- [ ] Extend `PluginInfusionSource` struct fields to carry `plugin_id: String` and
  `config: PluginConfigMap` (required because the `InfusionSource` trait signature only
  receives `input` and `input_type`; `plugin_id` and config must be captured at construction)
- [ ] Implement `PluginInfusionSource::enrich_single` (and `enrich_batch`) in
  `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` (stubs at lines 24-41):
  - if S-1.15 PluginRuntime is operational (plugin/mod.rs ~904):
    delegate via `runtime.enrich_single(self.plugin_id, input, input_type, &self.config)`
    using the UNTYPED `component::Val` path (see AC-004 for confirmed signature).
    DO NOT use `get_typed_func` or call `post_return` — removed in merged runtime.
    Map `PluginError → InfusionError` at the boundary.
  - if PluginRuntime not yet operational (e.g., source is `NotLoaded`): return `Ok(None)` and
    emit `tracing::warn!(plugin_id = %self.plugin_id, "plugin runtime unavailable — enrichment skipped")`.
    DO NOT return `Err(InfusionError::PluginRuntimeNotAvailable)` — that variant does not exist.
    DO NOT use `todo!("S-1.15")`. The map-log-None path is the correct production contract
    (BC-2.19.001 v1.5 / CRIT-3 closure). Real wiring lands in PIVOT-002/003 / S-1.14-REDO.
- [ ] Write regression test for AC-005 (NOT failing-first — already implemented):
  `test_BC_2_19_003_is_api_backed_true_for_plugin_type` — exercises the existing
  `InfusionRegistry::is_api_backed` implementation; confirms `true` for plugin-type UDFs and `false` for unknown names
- [ ] Verify tests 4-5 pass

**Phase 3: DataFusion UDF registration wiring**

- [ ] Write failing test 3 (FAIL first): `test_BC_2_19_001_plugin_udfs_registered_in_session_context`
  with anti-false-green hardening: mock async impl with sentinel value + call counter (see AC-003)
- [ ] In `prism-query`, add `register_infusion_udfs(ctx: &SessionContext, descriptors: Vec<InfusionUdfDescriptor>)`:
  for each descriptor, construct an `InfusionAsyncUdf` implementing `AsyncScalarUDFImpl`
  with: `invoke_async_with_args` (real enrichment call) and `invoke_with_args` returning
  `not_impl_err!(...)`. Register via `ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())`.
  DO NOT use `ScalarUDF::from(impl)` (sync-only path) for network-I/O UDFs.
  DO NOT wire `AsyncFunctionRule`, `enable_async_udf`, or any analyzer/optimizer rule —
  none of these exist in DataFusion 53.1 (see AC-003 hallucination list).
- [ ] **NullSource replacement (CRITICAL — net-new work):**
  `udf_descriptors()` in `infusion/mod.rs` hardwires every descriptor's `source` field to
  `Arc::new(NullSource)` (confirmed at mod.rs lines ~500, ~535, ~558, ~662). For plugin-type
  descriptors, the loader/registry MUST thread a REAL `PluginInfusionSource` (with populated
  `plugin_id` and `config` fields) into each descriptor's `source`. Without this, all UDFs
  are registered but enrich with nothing — NullSource always returns `None`.
  Implementation: `InfusionLoader::load_all` must construct `Arc<PluginInfusionSource>` and
  pass it into the descriptor's `source` field when building plugin-type descriptors.
- [ ] Wire `register_infusion_udfs` call into `prism-query/src/engine.rs` at BOTH
  SessionContext construction sites: (a) the `execute` path and (b) the `new_full`
  materialized path (or consolidate into `build_session_context` if one exists). Registering
  at only one site leaves the other path without enrichment UDFs.
  MERGE-COORDINATION NOTE: `engine.rs` is also touched by S-3.13 (capability-discovery block).
  Sequence merges or coordinate with the S-3.13 implementer to avoid conflicts at the
  SessionContext construction site. Preferred: add `register_infusion_udfs` in
  `build_session_context` (single site) if that helper exists post-S-3.13.
- [ ] Verify test 3 passes

**Phase 4: Final gates**

- [ ] SAP-1 probe (CLAUDE.md §SAP-1): `rg 'event_type\s*=' crates/ --type rust` —
  verify any new `event_type` emissions have BC-2.16.002 catalog rows
- [ ] Run `just iter prism-spec-engine` — all 4 spec-engine tests pass
- [ ] Run `just iter prism-query` — test 3 passes; no regression
- [ ] Perimeter check: `prism-spec-engine` must NOT depend on `prism-query`; dependency
  direction is `prism-query` → `prism-spec-engine` (verify no Cargo.toml change violates this)
- [ ] Confirm `plugin_bridge.rs` has NO production `unwrap()` or `expect()` on Result paths

---

## Previous Story Intelligence

**S-1.14 (partial-merge predecessor):**
- `InfusionSpec`, `InfusionRegistry`, `InfusionSource` trait, `plugin_bridge.rs` stubs are present
- `InfusionLoader::parse` skeleton existed with `unimplemented!()` at parse/load_all/validate_credentials; **both `validate_credentials` and `validate_pipe_stage_columns` are now implemented and wired into `parse` in PIVOT-001** (see §Edge Cases EC-006/EC-007)
- `PluginInfusionSource::enrich_single` and `enrich_batch` (InfusionSource trait impl) have
  `unimplemented!()` stubs at `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` lines 24-41
  (U5/Ruling 2: NOT a free function — trait impl on PluginInfusionSource)
- Test fixture file `test.mmdb` and `geoip.infusion.toml` are present — reuse context only; do NOT
  implement MMDB path (deferred to S-1.14-REDO)

**S-1.14-REDO (sibling wave-5 story):**
- This story is a forward-subset of S-1.14-REDO. After this story merges, S-1.14-REDO must be
  annotated to explicitly exclude the plugin-type loader and bridge (see §S-1.14-REDO Annotation).
- S-1.14-REDO owns: MMDB/CSV/JSON-lookup sources, three-tier cache, VP-048/049, hot reload
  integration, credential validation.

**S-1.15 (WASM runtime, partial-merge):**
- `PluginRuntime::enrich_single` may or may not be operational at dispatch. Read
  `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` to confirm current stub state
  (stubs at lines 24-41). If not operational, the correct behavior is map-log-None:
  `enrich_single` returns `Ok(None)` and emits a `tracing::warn!` — NOT `todo!("S-1.15")`
  and NOT `Err(InfusionError::PluginRuntimeNotAvailable)` (that variant does not exist).
  See AC-004 and CRIT-3 closure for the authoritative contract.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism-spec-engine` MUST NOT depend on `prism-query` | S-1.14-REDO Architecture Compliance | Cargo.toml check; dependency direction is query→spec-engine |
| `PluginInfusionSource::enrich_single` (InfusionSource trait impl in plugin_bridge.rs) delegates to `PluginRuntime::enrich_single` (plugin/mod.rs ~904) | BC-2.19.001 postcondition + WO-D1109 §Q1 Ruling 2 | AC-004 |
| Plugin-type infusion UDFs MUST have `is_api_backed() = true` | BC-2.19.003 postcondition | AC-005 |
| Plugin-type infusion UDFs MUST NOT be registered for detection rule execution path | BC-2.19.003 invariant | `is_api_backed()` wired to S-4.03 detection rule loader |
| No `unwrap()` / `expect()` on Result in production code paths | CLAUDE.md §Conventions | Adversary |
| All `event_type =` tracing emissions require BC-2.16.002 catalog rows | SAP-1 / CLAUDE.md §SAP-1 | Adversary SAP-1 probe |

**Forbidden Dependencies:**
- `prism-spec-engine` MUST NOT depend on `prism-query`, `prism-mcp`, or `prism-bin`
- `prism-query` MUST NOT depend on `prism-dtu-*` directly (DTU clones are test infrastructure only)

---

## Library & Framework Requirements

| Crate | Version | Usage |
|-------|---------|-------|
| `toml` | 0.8.x (workspace) | Infusion spec TOML parsing |
| `serde` / `serde_json` | 1.x (workspace) | InfusionSpec deserialization + enrichment result values |
| `wasmtime` | 44.x (already in prism-spec-engine/Cargo.toml) | PluginRuntime WASM bridge (S-1.15 delegation). The bridge uses the UNTYPED `component::Val` path via `instance.get_func(store, "enrich-single").call(store, params, results)`. DO NOT use `get_typed_func` or `TypedFunc::post_return` — post_return is removed in merged runtime (plugin/mod.rs ~L970: "post_return removed — no longer needed in wasmtime >=44 (no-op, deprecated)"). |
| DataFusion | 53.1 (workspace — confirm exact pin in Cargo.toml before using) | `AsyncScalarUDFImpl` (primary for network-I/O): `invoke_async_with_args` + sync `invoke_with_args` returning `not_impl_err!`. Registered via `ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` — NO analyzer/optimizer rules required; `DefaultPhysicalPlanner` handles async UDFs natively. Methods `invoke`/`invoke_batch` removed by DF ~48 — do not use. `AsyncFunctionRule`, `enable_async_udf`, `concurrent_async_udf_tasks` do NOT exist. |
| `tokio` | 1.x (workspace) | Async UDF invocation context |

**MSRV:** Rust stable per `rust-toolchain.toml`.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/infusion/loader.rs` | MODIFY | Implement parse/load_all for `source.type = "plugin"` only; unimplemented!() remains for other types (deferred to S-1.14-REDO) |
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` | MODIFY | (1) Add `plugin_id: String` and `config: PluginConfigMap` fields to `PluginInfusionSource`. (2) Implement `enrich_single`/`enrich_batch` (stubs at lines 24-41) delegating via the UNTYPED `component::Val` path to `PluginRuntime::enrich_single`. DO NOT use `TypedFunc` or `post_return`. Map `PluginError → InfusionError`. |
| `crates/prism-spec-engine/src/infusion/mod.rs` | MODIFY | (1) `InfusionRegistry::is_api_backed` is ALREADY IMPLEMENTED (`InfusionRegistry::is_api_backed` method) — no re-implementation needed. (2) NET-NEW: Replace `Arc::new(NullSource)` with real `Arc<PluginInfusionSource>` in `udf_descriptors()` and `load_all` for plugin-type descriptors (multiple `NullSource` sites in this file today). |
| `crates/prism-query/src/engine.rs` | MODIFY | Add `register_infusion_udfs` call at BOTH SessionContext construction sites (execute path + new_full/materialized path). Merge-coordinate with S-3.13 (capability-discovery block also modifies engine.rs). |
| `crates/prism-spec-engine/tests/infusion_tests.rs` | MODIFY | Add Red Gate tests 1, 2, 4, 5 |
| `crates/prism-query/tests/` or `src/` | MODIFY | Add Red Gate test 3 (UDF registration) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `source.type = "maxmind_mmdb"` passed to load_all | Returns `Err(InfusionError::UnknownSourceType { type_name: "maxmind_mmdb".into() })` / E-INFUSE-004: "Unknown source type 'maxmind_mmdb'. Valid types: maxmind_mmdb, csv, json_lookup, plugin." (stub — full impl in S-1.14-REDO; see error-taxonomy.md §E-INFUSE-004 and prism-core error.rs `InfusionError::UnknownSourceType`) |
| EC-002 | Plugin-type spec with no `plugin_ref` field | `Err(InfusionError::MissingRequiredField { field: "plugin_ref".into(), spec_path })` / E-INFUSE-003 from parse — plugin_ref is required for type=plugin (see error-taxonomy.md §E-INFUSE-003 and prism-core error.rs `InfusionError::MissingRequiredField`) |
| EC-003 | `PluginInfusionSource::enrich_single` with S-1.15 unavailable (e.g., source is `NotLoaded`) | Returns `Ok(None)` and emits a `tracing::warn!` — map-log-None path per BC-2.19.001 v1.5 / CRIT-3 closure. NOT an error return; NOT a panic. `InfusionError::PluginRuntimeNotAvailable` does not exist — do not use it. |
| EC-004 | Two plugin specs with the same field name (duplicate UDF) | `InfusionError::DuplicateUdfName` / E-INFUSE-002 returned from load_all for the second spec; first spec retained (BC-2.19.001 Error Cases: E-INFUSE-002; error message: "Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'.") |
| EC-005 | `is_api_backed` called with UDF name not in registry | Returns `false` (unknown UDFs are not API-backed by default) |
| EC-006 | `validate_credentials` — spec with empty `env_var` on a `CredentialRef` | `Err(InfusionError::...)` at parse time; `tracing::warn!` / structured error. **IMPLEMENTED in PIVOT-001** — wired into `InfusionLoader::parse`. Tests: `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` (happy path), `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` (reject path). |
| EC-007 | `validate_pipe_stage_columns` — `pipe_stage.adds_columns` entry names a column not in `spec.fields` | `Err(InfusionError::...)` at parse time. **IMPLEMENTED in PIVOT-001** — wired into `InfusionLoader::parse`. Tests: `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` (happy path), `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` (reject path). |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `InfusionLoader::parse` (plugin path) | `prism-spec-engine` (SS-19) | Pure (TOML parsing + structural validation) | BC-2.19.001 postcondition |
| `InfusionLoader::load_all` (plugin branch) | `prism-spec-engine` (SS-19) | Effectful (constructs PluginInfusionSource) | BC-2.19.001 postcondition |
| `PluginInfusionSource::enrich_single` / `enrich_batch` (InfusionSource trait impl) | `prism-spec-engine` (SS-17, SS-19) — `src/infusion/plugin_bridge.rs` | Effectful (delegates via UNTYPED `component::Val` path to PluginRuntime::enrich_single; maps PluginError → InfusionError; post_return NOT called — removed in merged runtime) | WO-D1109 §Q1, Ruling 2 |
| `InfusionRegistry::is_api_backed` | `prism-spec-engine` (SS-19) | Pure (HashMap lookup) | BC-2.19.003 postcondition |
| `register_infusion_udfs` | `prism-query` (SS-11) | Effectful (SessionContext mutation) | BC-2.19.001 postcondition |

---

## S-1.14-REDO Annotation

Per WO-D1109 §Story 1 note, S-1.14-REDO must be updated after this story merges to
reflect that the plugin-type InfusionLoader path and `PluginInfusionSource::enrich_single`
are no longer unimplemented stubs (they were implemented by this story). S-1.14-REDO's
scope becomes:
- MMDB, CSV, JSON-lookup `InfusionSource` implementations
- Three-tier cache (`InfusionLruCache`, `QueryScopedInfusionCache`, RocksDB Tier 3)
- VP-048 Kani proof and VP-049 proptest (per-query dedup)
- Hot reload integration (S-1.12-FOLLOWUP watcher)
- (Plugin-type loader + bridge: ALREADY DONE by S-DEMO-ENRICHMENT-PIVOT-001)
- (`validate_credentials`: ALREADY DONE by S-DEMO-ENRICHMENT-PIVOT-001 — wired into `parse`; tests: `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference`, `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential`)
- (`validate_pipe_stage_columns`: ALREADY DONE by S-DEMO-ENRICHMENT-PIVOT-001 — wired into `parse`; tests: `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields`, `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference`)

The story-writer notes that S-1.14-REDO should add a `forward_subset_implemented_by:
[S-DEMO-ENRICHMENT-PIVOT-001]` frontmatter annotation and a body note in its §Objective
section. This annotation should be applied by state-manager at post-merge burst time.

---

## SAP-1 Compliance

Per CLAUDE.md §SAP-1, any `tracing::*!(event_type = "...")` emission added in this story
requires a BC-2.16.002 catalog row in the same commit.

Anticipated emissions (implementer must enumerate actual sites):
- Potentially `event_type = "infusion.plugin_enrich"` in plugin_bridge
- If NO new `event_type` emissions are added, state explicitly in PR description.

---

## Story Changelog

| Version | Date | Change |
|---------|------|--------|
| v1.11 | 2026-06-15 | OBS-1 closure: EC-004 prose corrected — replaced "SpecEngineError at load_all with named conflict" with `InfusionError::DuplicateUdfName` / E-INFUSE-002, per canonical error taxonomy (E-INFUSE-002 row: "Duplicate UDF name '{udf_name}' in '{path2}' — already registered from '{path1}'"; second spec rejected, first retained) and BC-2.19.001 Error Cases row. BC conflict check: BC-2.19.001 explicitly contracts E-INFUSE-002 for this condition (not SpecEngineError); no BC conflict found. Source-of-Truth Precedence: taxonomy + BC govern; EC prose was stale. |
| v1.10 | 2026-06-15 | Validator-implemented closure: `validate_credentials` and `validate_pipe_stage_columns` are now IMPLEMENTED + WIRED into `InfusionLoader::parse` + TESTED in PIVOT-001 (feature/S-DEMO-ENRICHMENT-PIVOT-001 @e87e44ea). Removed both from S-1.14-REDO deferral list (§"What this story does NOT implement" and §S-1.14-REDO Annotation scope list). Added §"Now implemented in PIVOT-001" note with test names (TD-VSDD-091). Updated Previous Story Intelligence to reflect both validators implemented. Added EC-006 (`validate_credentials` — empty `env_var`) and EC-007 (`validate_pipe_stage_columns` — unknown column ref) with 4 new test-name anchors. Red Gate count 5 → 7; tests 6-7 added to Red Gate Test Plan table. |
| v1.9 | 2026-06-15 | LOW-1 BC-table sync (PIVOT-001 LOW-1): BC-2.19.001 row at line ~174 updated — version pin bumped v1.5→v1.7; carrier-struct phrasing corrected: PARSE PHASE (`load_all`) returns specs+errors and does NOT construct `PluginInfusionSource`; RUNTIME PHASE (`load_spec_with_runtime`) builds `PluginInfusionSource` and attaches it as `descriptor.source`; `plugin_id`/`config` live on `PluginInfusionSource` via `descriptor.source`, not on `InfusionUdfDescriptor` directly. Aligns with BC-2.19.001 v1.7 ground truth. |
| v1.8 | 2026-06-15 | OBS prose-precision fix: AC-002 second "then" bullet clarified — `plugin_id`/`config` live on `PluginInfusionSource` (accessed via `descriptor.source`), not on `InfusionUdfDescriptor` directly. Implementation and tests are correct and unchanged; AC prose now matches the struct layout. |
| v1.7 | 2026-06-15 | LOW spec↔impl drift fix (CRIT-3 closure): reconciled all pre-CRIT-3 prose to BC-2.19.001 v1.5 map-log-None semantics. Five sites corrected — (1) frontmatter risk_mitigations U5/Ruling-2 bullet, (2) AC-004 unavailable-runtime clause, (3) Tasks Phase-2 `if PluginRuntime not yet operational` bullet, (4) Previous Story Intelligence S-1.15 paragraph, (5) EC-003. All sites now specify: runtime-unavailable → `Ok(None)` + `tracing::warn!` (map-log-None). Removed all references to phantom `InfusionError::PluginRuntimeNotAvailable` variant (does not exist in prism-core/src/error.rs) and removed all `todo!("S-1.15")` directives. E-INFUSE-003/004/007 EC entries unchanged. |
| v1.6 | 2026-06-14 | MED-1 fix: EC-001 corrected from non-existent `InfusionError::UnsupportedSourceType` / `E-INFUSE-003` to real variant `InfusionError::UnknownSourceType` / `E-INFUSE-004` (verified against prism-core error.rs line 1135 and error-taxonomy.md line 435 and BC-2.19.001 §Error table). `E-INFUSE-003` is `MissingRequiredField` — the wrong code for unknown-source-type. EC-002 corrected to cite `InfusionError::MissingRequiredField` / `E-INFUSE-003` (correct for missing `plugin_ref`). Tasks Phase-1 bullet corrected: `SpecEngineError::UnsupportedInfusionSourceType` (non-existent, forbidden by risk_mitigations U7) replaced with `Err(InfusionError::UnknownSourceType { type_name })` / E-INFUSE-004. risk_mitigations Scope-boundary bullet corrected to same. OBS-2 fix: all line-pin citations `infusion/mod.rs:619-628` de-pinned to behavioral anchor `InfusionRegistry::is_api_backed` per TD-VSDD-091 (three sites: risk_mitigations U6, AC-005 note, §File Structure mod.rs row, Red Gate table test-5 parenthetical). |
| v1.5 | 2026-06-14 | OBS-3 fix: AC-002 third bullet removed unsupported `Arc<NullSource>` type assertion. The test `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` asserts descriptor count + `plugin_id`/`config` field population (parse-phase outcome) — it does NOT make a `NullSource` type assertion, which would require downcasting and is not part of the test's observable contract. AC-002 "then" bullets now match what the test actually asserts. |
| v1.4 | 2026-06-14 | AC-002 aligned to BC-2.19.001 v1.5 two-phase wiring. AC-002 "then" clause now distinguishes: `load_all` (parse phase) produces descriptors with `plugin_id`/`config` populated but `source = Arc<NullSource>` placeholder; the full runtime-wired `Arc<PluginInfusionSource>` is the output of `load_spec_with_runtime` (runtime phase). AC-002 scoping note added: test asserts parse-phase outcome only; runtime bridge delegation is AC-004's scope. Test name `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` retained (names parse phase correctly — do not rename to imply full runtime wiring). BC table row updated: BC-2.19.001 v1.4 → v1.5 with two-phase wiring description. Frontmatter BC status comment updated to v1.5. |
| v1.3 | 2026-06-14 | BC version-pin sync (D-1167 state-manager burst — citation sync only). BC-2.19.001 v1.3 → v1.4 (NullSource gap closed; PO confirmed at D-1166); BC-2.19.003 v? → v1.3 (PO-confirmed at D-1166). Frontmatter BC status comment updated to remove pending-PO language. Behavioral Contracts table rows updated. No AC/scope changes. |
| v1.2 | 2026-06-14 | Pre-TDD scan corrections (verified against develop@664566e9). (1) Correction 1 — post_return RETRACTED: plugin/mod.rs ~L970 removed post_return as deprecated in wasmtime >=44; risk_mitigations + AC-004 + Library table + Architecture Mapping updated to reflect UNTYPED component::Val path with no post_return. (2) Correction 2 — enrich_single delegation signature fixed: real signature is PluginRuntime::enrich_single(plugin_id, input_value, input_type, config: &PluginConfigMap) -> Result<Option<Value>, PluginError>; PluginInfusionSource must carry plugin_id + config as fields; bridge maps PluginError → InfusionError; AC-004 + task + file-structure updated. (3) Correction 3 — async UDF research applied: DataFusion 53.1.0 confirmed ctx.register_udf(AsyncScalarUDF::new(Arc<dyn AsyncScalarUDFImpl>).into_scalar_udf()) is sufficient; DefaultPhysicalPlanner handles async natively; no analyzer/optimizer rules needed; hallucinated symbols catalogued (AsyncFunctionRule, enable_async_udf, concurrent_async_udf_tasks, GLOBAL_ASYNC_UDF_SEMAPHORE — do not exist); AC-003 rewritten, hallucination list added, anti-false-green hardening added (sentinel value + call counter). (4) Correction 4 — NullSource wiring gap: udf_descriptors() hardwires Arc::new(NullSource) at mod.rs ~L500/535/558/662; net-new Phase 3 task added to replace with real PluginInfusionSource; engine.rs two-site registration noted (execute + new_full paths); S-3.13 merge-coordination note added; file-structure table corrected (is_api_backed already implemented — not net-new). Status: draft → ready. |
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U2/U3/U4/U5/U6/U7/U8 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form throughout; grammar doc AC-006 added. plugin_bridge free-function replaced with PluginInfusionSource::enrich_single trait impl (Ruling 2). TypedFunc::post_return mandatory wiring added. DataFusion ScalarUDFImpl rewritten: invoke_with_args + AsyncScalarUDFImpl::invoke_async_with_args (invoke/invoke_batch removed DF~48). is_api_backed scoped as regression test (already implemented). InfusionError enum replaces invented SpecEngineError variants. Wrong path cite src/plugin_bridge.rs → src/infusion/plugin_bridge.rs fixed. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 1. Forward-subset of S-1.14-REDO for demo enrichment chain. Depends on S-1.14 partial-merge scaffolding; blocks 002 and 003. BC status: BC-2.19.001 + BC-2.19.003 as nearest anchors pending PO confirmation. Sequencing note added per D-1109 demo objective chain. |
