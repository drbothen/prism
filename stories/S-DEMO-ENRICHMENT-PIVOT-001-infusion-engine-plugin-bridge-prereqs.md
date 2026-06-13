---
document_type: story
story_id: S-DEMO-ENRICHMENT-PIVOT-001
title: "Infusion Engine Plugin-Bridge Prerequisites — Forward-Subset of S-1.14-REDO for Demo"
wave: 5
epic_id: E-DEMO
priority: P2
status: draft
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-06-12T00:00:00Z"
created: "2026-06-12"
modified: "2026-06-12T18:00:00Z"
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
# BC status: pending PO authorship
# The infusion BCs (BC-2.19.*) exist and are the nearest anchors. This story implements
# a FORWARD-SUBSET of S-1.14-REDO scope: only plugin-type InfusionLoader + plugin_bridge
# + DataFusion UDF wiring for plugin-type infusions. The full BC-2.19.* contracts govern
# the complete engine; PO should anchor a story-specific BC or confirm BC-2.19.001/003
# cover this subset at materialization time.
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
red_gate_tests: 5
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "PluginInfusionSource::enrich_single/enrich_batch (U5/Ruling 2): implement via the
     existing InfusionSource trait methods in crates/prism-spec-engine/src/infusion/plugin_bridge.rs
     (stubs at lines 24-41 with unimplemented!()). Do NOT add a new free-function
     plugin_bridge::enrich_via_plugin. Delegate to PluginRuntime::enrich_single
     (plugin/mod.rs ~904). If S-1.15 PluginRuntime is not operational at dispatch time,
     implement as Err(InfusionError::PluginRuntimeNotAvailable) with annotated todo!(S-1.15) —
     same exemption as S-1.14-REDO AC-9. Document in PR."
  - "DataFusion UDF registration (U3/U4/U8): implement ScalarUDFImpl with required methods:
     name/signature/return_type/as_any and invoke_with_args(ScalarFunctionArgs) -> Result<ColumnarValue>.
     Methods invoke/invoke_batch were REMOVED by DataFusion ~48 — do not cite them.
     For the network-I/O enrichment path use AsyncScalarUDFImpl::invoke_async_with_args
     (native async UDFs since DataFusion 49.0) as the primary pattern; block_in_place+block_on
     is fallback-only (safe ONLY on multi-thread runtime per AD-013; panics on current-thread).
     Registration: ctx.register_udf(ScalarUDF::from(...)) — no SessionState extension API.
     Read DataFusion 53.1 docs before implementing."
  - "wasmtime TypedFunc::post_return (U2): after every successful sync call to
     Instance::get_typed_func::<(String, String), (Option<String>,)>(store, 'enrich-single')
     followed by TypedFunc::call, MUST call TypedFunc::post_return on the same store.
     Skip post_return only on Err paths. This is mandatory per wasmtime 44 TypedFunc ABI
     (component model post-return convention)."
  - "is_api_backed() is ALREADY implemented (infusion/mod.rs:619-628) (U6): scope as a
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
     and return E-INFUSE-003 (or equivalent InfusionError variant) for unrecognized types."
  - "Forward-subset graduation contract: after this story merges, S-1.14-REDO's scope
     annotation MUST be updated to explicitly exclude the plugin-type loader path
     implemented here. See §S-1.14-REDO Annotation section."
traces_to: [D-1109, WO-D1109]
supersedes: []
---

# S-DEMO-ENRICHMENT-PIVOT-001: Infusion Engine Plugin-Bridge Prerequisites

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
- Credential validation (`validate_credentials`)

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
| BC-2.19.001 v? | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | Postcondition: each field in `[[infusion.fields]]` produces exactly one `InfusionUdfDescriptor` entry registered in `SessionContext` |
| BC-2.19.003 v? | API-Backed Infusion UDFs Rejected in Detection Rule Filters — E-RULE-012 | Postcondition: `is_api_backed("threat_score")` returns `true` for plugin-type infusions; detection rule loader rejects with E-RULE-012 |

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

### AC-002 — InfusionLoader::load_all builds InfusionRegistry with plugin-type descriptors
(traces to BC-2.19.001 postcondition — each field registers exactly one UDF descriptor)

Given a directory containing a valid plugin-type `.infusion.toml`,
when `InfusionLoader::load_all` runs,
then the returned `InfusionRegistry` contains `InfusionUdfDescriptor` entries for each
declared output field, and `registry.udf_descriptors()` returns a non-empty `Vec`.

Red Gate: `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`

### AC-003 — DataFusion UDF registration wires plugin descriptors into SessionContext
(traces to BC-2.19.001 postcondition — each field registers as a DataFusion scalar UDF)

Given an `InfusionRegistry` with plugin-type descriptors returned from `load_all`,
when `register_infusion_udfs(ctx, registry.udf_descriptors())` is called in `prism-query`,
then each descriptor is registered as a `ScalarUDF` in the `SessionContext`; the UDF
is callable by name in a DataFusion query plan.

Red Gate: `test_BC_2_19_001_plugin_udfs_registered_in_session_context`

### AC-004 — PluginInfusionSource::enrich_single delegates to PluginRuntime::enrich_single
(traces to BC-2.19.001 postcondition — plugin-type source executes via plugin bridge)

Given `PluginInfusionSource::enrich_single` (the InfusionSource trait impl in
`crates/prism-spec-engine/src/infusion/plugin_bridge.rs`, stubs at lines 24-41)
called with a valid plugin-type `InfusionSpec` and an input value,
when `PluginRuntime::enrich_single` is available (S-1.15 operational, plugin/mod.rs ~904),
then `enrich_single` delegates to `PluginRuntime::enrich_single` using
`Instance::get_typed_func::<(String, String), (Option<String>,)>(store, "enrich-single")` →
`TypedFunc::call` → process result → `TypedFunc::post_return` (mandatory after every
successful sync call per wasmtime 44 TypedFunc ABI; skip only on Err).

When `PluginRuntime` is not yet available at dispatch time, `enrich_single` returns
`Err(InfusionError::PluginRuntimeNotAvailable)` (or the closest equivalent variant in the
InfusionError enum) with an annotated `todo!("S-1.15")`.

Red Gate: `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`

### AC-005 — is_api_backed() returns true for plugin-type infusion UDFs (regression confirmation)
(traces to BC-2.19.003 postcondition — API-backed UDFs rejected in detection rule filters)

NOTE (U6): `InfusionRegistry::is_api_backed` is ALREADY IMPLEMENTED at
`crates/prism-spec-engine/src/infusion/mod.rs:619-628`. This AC is a regression/confirmation
test — NOT a red-gate-new implementation task. The test exercises the existing code path to
verify it returns `true` for plugin-type UDFs. Adjust points estimate accordingly (0.5 pts
→ ~0.2 pts for test only).

Given an `InfusionRegistry` loaded with a plugin-type spec whose field is named `threat_score`,
when `registry.is_api_backed("threat_score")` is called,
then it returns `true` (verified against existing implementation at infusion/mod.rs:619-628).

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
| 5 | `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | prism-spec-engine | BC-2.19.003 postcondition | unit (REGRESSION — is_api_backed already implemented at infusion/mod.rs:619-628; test confirms existing behavior) |

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
| Test files (5 red gate stubs × ~40 lines each; AC-005 regression ~20 lines) | ~620 |
| Tool outputs (nextest, clippy) | ~1,000 |
| **Total estimate** | **~13,300** |

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
  `SpecEngineError::UnsupportedInfusionSourceType` (stub — full impl in S-1.14-REDO)
- [ ] Verify tests 1-2 pass

**Phase 2: plugin_bridge (InfusionSource trait impl)**

- [ ] Write failing test 4 (FAIL first): `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`
- [ ] Implement `PluginInfusionSource::enrich_single` (and `enrich_batch`) in
  `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` (stubs at lines 24-41):
  - if S-1.15 PluginRuntime is operational (plugin/mod.rs ~904):
    delegate using `Instance::get_typed_func::<(String, String), (Option<String>,)>(store, "enrich-single")`
    → `TypedFunc::call` → process result → call `TypedFunc::post_return` (MANDATORY after
    every successful call; skip only on Err paths — wasmtime 44 TypedFunc ABI requirement)
  - if PluginRuntime not yet operational: `Err(InfusionError::PluginRuntimeNotAvailable)`
    (use the closest variant in the existing InfusionError enum) with `todo!("S-1.15")`
- [ ] Write regression test for AC-005 (NOT failing-first — already implemented):
  `test_BC_2_19_003_is_api_backed_true_for_plugin_type` — reads existing implementation at
  `infusion/mod.rs:619-628`; confirms `true` for plugin-type UDFs and `false` for unknown names
- [ ] Verify tests 4-5 pass

**Phase 3: DataFusion UDF registration wiring**

- [ ] Write failing test 3 (FAIL first): `test_BC_2_19_001_plugin_udfs_registered_in_session_context`
- [ ] In `prism-query`, add `register_infusion_udfs(ctx: &SessionContext, descriptors: Vec<InfusionUdfDescriptor>)`:
  for each descriptor, construct `ScalarUDF::from(InfusionScalarUdf::new(desc))` and call
  `ctx.register_udf(udf)`. `InfusionScalarUdf` implements DataFusion `ScalarUDFImpl` with
  required methods: `name()`, `signature()`, `return_type()`, `as_any()`, and
  `invoke_with_args(ScalarFunctionArgs) -> Result<ColumnarValue>` — do NOT implement
  `invoke` or `invoke_batch` (removed by DataFusion ~48). For the network-I/O enrichment
  path implement `AsyncScalarUDFImpl::invoke_async_with_args` as the primary pattern
  (native async UDFs, DataFusion 49.0+); `block_in_place`+`block_on` is fallback-only and
  safe ONLY on multi-thread runtime (AD-013). Registration uses
  `ctx.register_udf(ScalarUDF::from(...))` — no SessionState extension API.
- [ ] Wire call into `QueryEngine::new` or equivalent construction site (read engine.rs first)
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
- `InfusionLoader::parse` skeleton exists with `unimplemented!()` at parse/load_all/validate_credentials
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
  (stubs at lines 24-41). Use annotated `todo!("S-1.15")` if not operational.

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
| `wasmtime` | 44.x (already in prism-spec-engine/Cargo.toml) | PluginRuntime WASM bridge (S-1.15 delegation) via `Instance::get_typed_func` / `TypedFunc::call` + `TypedFunc::post_return` (mandatory after every successful sync call per wasmtime 44 component model ABI) |
| DataFusion | 53.1 (workspace — confirm exact pin in Cargo.toml before using) | `ScalarUDFImpl` (methods: name/signature/return_type/as_any/invoke_with_args) + `AsyncScalarUDFImpl::invoke_async_with_args` for network I/O; registered via `ctx.register_udf(ScalarUDF::from(...))`. Methods `invoke`/`invoke_batch` removed by DF ~48 — do not use. |
| `tokio` | 1.x (workspace) | Async UDF invocation context |

**MSRV:** Rust stable per `rust-toolchain.toml`.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/infusion/loader.rs` | MODIFY | Implement parse/load_all for `source.type = "plugin"` only; unimplemented!() remains for other types (deferred to S-1.14-REDO) |
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` | MODIFY | Implement `PluginInfusionSource::enrich_single` and `enrich_batch` (InfusionSource trait stubs at lines 24-41) delegating to PluginRuntime::enrich_single via TypedFunc + post_return (or annotated todo! if S-1.15 unavailable) |
| `crates/prism-spec-engine/src/infusion/mod.rs` | MODIFY | Implement `InfusionRegistry::is_api_backed(udf_name: &str) -> bool` |
| `crates/prism-query/src/engine.rs` | MODIFY | Add `register_infusion_udfs` call at SessionContext construction site |
| `crates/prism-spec-engine/tests/infusion_tests.rs` | MODIFY | Add Red Gate tests 1, 2, 4, 5 |
| `crates/prism-query/tests/` or `src/` | MODIFY | Add Red Gate test 3 (UDF registration) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `source.type = "maxmind_mmdb"` passed to load_all | Returns `Err(InfusionError::UnsupportedSourceType)` or nearest equivalent InfusionError variant (stub — full impl in S-1.14-REDO); E-INFUSE-003 if that code exists in error-taxonomy.md |
| EC-002 | Plugin-type spec with no `plugin_ref` field | `Err(InfusionError::...)` validation error from parse — plugin_ref is required for type=plugin; use nearest existing InfusionError variant; if none fits, add variant to InfusionError enum + row to error-taxonomy.md |
| EC-003 | `PluginInfusionSource::enrich_single` with S-1.15 unavailable | Returns `Err(InfusionError::PluginRuntimeNotAvailable)` (or nearest variant) — NOT a panic |
| EC-004 | Two plugin specs with the same field name (duplicate UDF) | `SpecEngineError` at load_all with named conflict (BC-2.19.001 invariant: UDF names globally unique) |
| EC-005 | `is_api_backed` called with UDF name not in registry | Returns `false` (unknown UDFs are not API-backed by default) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Anchor |
|-----------|--------|---------------|--------|
| `InfusionLoader::parse` (plugin path) | `prism-spec-engine` (SS-19) | Pure (TOML parsing + structural validation) | BC-2.19.001 postcondition |
| `InfusionLoader::load_all` (plugin branch) | `prism-spec-engine` (SS-19) | Effectful (constructs PluginInfusionSource) | BC-2.19.001 postcondition |
| `PluginInfusionSource::enrich_single` / `enrich_batch` (InfusionSource trait impl) | `prism-spec-engine` (SS-17, SS-19) — `src/infusion/plugin_bridge.rs` | Effectful (WASM runtime call via TypedFunc + post_return, or Err) | WO-D1109 §Q1, Ruling 2 |
| `InfusionRegistry::is_api_backed` | `prism-spec-engine` (SS-19) | Pure (HashMap lookup) | BC-2.19.003 postcondition |
| `register_infusion_udfs` | `prism-query` (SS-11) | Effectful (SessionContext mutation) | BC-2.19.001 postcondition |

---

## S-1.14-REDO Annotation

Per WO-D1109 §Story 1 note, S-1.14-REDO must be updated after this story merges to
reflect that the plugin-type InfusionLoader path and `plugin_bridge::enrich_via_plugin`
are no longer unimplemented stubs (they were implemented by this story). S-1.14-REDO's
scope becomes:
- MMDB, CSV, JSON-lookup `InfusionSource` implementations
- Three-tier cache (`InfusionLruCache`, `QueryScopedInfusionCache`, RocksDB Tier 3)
- VP-048 Kani proof and VP-049 proptest (per-query dedup)
- Hot reload integration (S-1.12-FOLLOWUP watcher)
- Credential validation (`validate_credentials`)
- (Plugin-type loader + bridge: ALREADY DONE by S-DEMO-ENRICHMENT-PIVOT-001)

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
| v1.1 | 2026-06-12 | D-1109 remove-uncertainty closure: U1/U2/U3/U4/U5/U6/U7/U8 applied (scanner + research-agent + architect rulings 1-4, WO-D1109 v1.1). enrich syntax → function-call form throughout; grammar doc AC-006 added. plugin_bridge free-function replaced with PluginInfusionSource::enrich_single trait impl (Ruling 2). TypedFunc::post_return mandatory wiring added. DataFusion ScalarUDFImpl rewritten: invoke_with_args + AsyncScalarUDFImpl::invoke_async_with_args (invoke/invoke_batch removed DF~48). is_api_backed scoped as regression test (already implemented). InfusionError enum replaces invented SpecEngineError variants. Wrong path cite src/plugin_bridge.rs → src/infusion/plugin_bridge.rs fixed. |
| v1.0 | 2026-06-12 | Initial draft per WO-D1109 §Story 1. Forward-subset of S-1.14-REDO for demo enrichment chain. Depends on S-1.14 partial-merge scaffolding; blocks 002 and 003. BC status: BC-2.19.001 + BC-2.19.003 as nearest anchors pending PO confirmation. Sequencing note added per D-1109 demo objective chain. |
