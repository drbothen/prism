# Red Gate Log — S-DEMO-ENRICHMENT-PIVOT-001

**Story:** S-DEMO-ENRICHMENT-PIVOT-001 — Plugin-type infusion loader + PluginInfusionSource + DataFusion async UDF registration
**Phase:** 3 (TDD Implementation) — Red Gate Step
**Wave:** wave-5-e-demo-fidelity
**Date:** 2026-06-14
**Author:** test-writer
**Worktree:** .worktrees/S-DEMO-ENRICHMENT-PIVOT-001 (branch: feature/S-DEMO-ENRICHMENT-PIVOT-001, based on develop@664566e9)

---

## Red Gate Status: RED

4 of 5 Red Gate tests FAIL. Workspace COMPILES. Red Gate discipline satisfied per BC-5.39.001.

The 5th test (AC-003 async UDF sentinel test) uses a `MockInfusionAsyncUdf` to prove DataFusion's
async path works — this test PASSES at the Red Gate phase by design (the production
`InfusionAsyncUdf::invoke_async_with_args` has `todo!()` as its Red Gate stub). The companion
test (`test_BC_2_19_001_register_infusion_udfs_helper_compiles_and_registers`) verifies
registration only (not invocation), so the `todo!()` in `InfusionAsyncUdf` is never hit.
The actual production Red Gate is enforced structurally: the implementer's first task is to
replace `todo!()` in `InfusionAsyncUdf::invoke_async_with_args`.

---

## Stub Changes

| File | Change |
|------|--------|
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs` | REWRITTEN — `PluginInfusionSource` struct redesigned: old `plugin_path: String` + `infusion_id: String` fields replaced with `plugin_id: String`, `config: Arc<PluginConfigMap>`, `runtime: Arc<PluginRuntime>`. Manual `Debug` impl (PluginRuntime has no `#[derive(Debug)]`). `enrich_single` and `enrich_batch` are `todo!()` stubs — Red Gate. `map_plugin_error_to_infusion_error` helper added. |
| `crates/prism-spec-engine/src/lib.rs` | Added three re-exports: `pub use infusion::loader::InfusionLoader`, `pub use infusion::plugin_bridge::PluginInfusionSource`, `pub use plugin::PluginConfigMap`. |
| `crates/prism-query/src/infusion_udf.rs` | NEW — `InfusionAsyncUdf` struct with manual `PartialEq + Eq + Hash` (name-keyed, required by `ScalarUDFImpl: DynEq + DynHash`). Two-block impl: `ScalarUDFImpl` (with `invoke_with_args` returning `not_impl_err!`) + `#[async_trait] AsyncScalarUDFImpl` (with `invoke_async_with_args` body `todo!()`). `register_infusion_udfs` function. |
| `crates/prism-query/src/lib.rs` | Added `pub mod infusion_udf` declaration. |
| `crates/prism-query/Cargo.toml` | Added `[[test]]` entry for `bc_2_19_001_plugin_udf_registration_test`. |

**`cargo build -p prism-spec-engine -p prism-query --tests` result: PASS (exit 0)** — all files compile without error.

---

## DataFusion 53.1 Async UDF — Implementation Notes

The following were discovered and resolved during Red Gate stub authoring:

1. **`AsyncScalarUDF`/`AsyncScalarUDFImpl` import path:** These are NOT re-exported from
   `datafusion::logical_expr`. They live at `datafusion::logical_expr::async_udf::{AsyncScalarUDF, AsyncScalarUDFImpl}`.

2. **`AsyncScalarUDFImpl` requires `#[async_trait]` on impl blocks:** The trait itself is
   declared with `#[async_trait]` in DataFusion 53.1. Implementors MUST annotate their impl
   block with `#[async_trait]` to match the lifetime desugaring — without it, the compiler
   reports "lifetimes do not match method in trait".

3. **`ScalarUDFImpl: DynEq + DynHash`:** The base trait requires `Eq + Hash + Any` (auto-impl'd
   via blanket impls in `datafusion-expr-common`). `InfusionUdfDescriptor` contains
   `Arc<dyn InfusionSource>` which doesn't implement `Hash`, so we use manual `PartialEq + Eq + Hash`
   impls keyed on the UDF name (globally unique within a `SessionContext` per DataFusion's
   registration invariant).

4. **Two-block impl pattern required:** `AsyncScalarUDFImpl` is a supertrait of `ScalarUDFImpl`.
   Both must be implemented in SEPARATE impl blocks. Mixing `ScalarUDFImpl` methods into the
   `AsyncScalarUDFImpl` block fails to satisfy the base trait requirement.

5. **`ctx.udfs()` requires `FunctionRegistry` in scope:** `SessionContext::udfs()` is provided by
   the `datafusion::execution::FunctionRegistry` trait. The test must `use datafusion::execution::FunctionRegistry`.
   The return type is `HashSet<String>` (not `HashMap`); use `.contains("name")` not `.contains_key("name")`.

6. **`ScalarFunctionArgs` has no lifetime parameter** in DataFusion 53.1. The story spec erroneously
   cited `ScalarFunctionArgs<'_>`. The correct signature is `fn invoke_async_with_args(&self, args: ScalarFunctionArgs)`.

---

## Anti-False-Green Design (AC-003)

`test_BC_2_19_001_plugin_udfs_registered_in_session_context` uses `MockInfusionAsyncUdf` with:
- **Sentinel value** `"CVE-PIVOT-TEST-SENTINEL"`: returned from `invoke_async_with_args` only.
  A sync fallback (`not_impl_err!`) cannot produce it.
- **`Arc<AtomicUsize>` call counter**: asserted `> 0` after query execution.
  Proves `invoke_async_with_args` was actually invoked via DataFusion's async path.

Two-assertion guard means: a stub that hard-codes a return value in `invoke_with_args` CANNOT
satisfy both assertions simultaneously. An incorrectly-wrapped UDF (sync-only) fails loudly.

---

## Test Files Created

| File | BC | Test Name | Failure Mode |
|------|----|-----------|--------------|
| `crates/prism-spec-engine/tests/infusion_tests.rs` (appended) | BC-2.19.001 | `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | `unimplemented!()` in `InfusionLoader::parse` |
| `crates/prism-spec-engine/tests/infusion_tests.rs` (appended) | BC-2.19.001 EC-002 | `test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref` | `unimplemented!()` in `InfusionLoader::parse` |
| `crates/prism-spec-engine/tests/infusion_tests.rs` (appended) | BC-2.19.001 | `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | `unimplemented!()` in `InfusionLoader::load_all` |
| `crates/prism-spec-engine/tests/infusion_tests.rs` (appended) | BC-2.19.001 EC-001 | `test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type` | `unimplemented!()` in `InfusionLoader::load_all` |
| `crates/prism-query/tests/bc_2_19_001_plugin_udf_registration_test.rs` | BC-2.19.001 AC-003 | `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | PASSES (mock UDF) — production Red Gate is `todo!()` in `InfusionAsyncUdf::invoke_async_with_args` |
| `crates/prism-query/tests/bc_2_19_001_plugin_udf_registration_test.rs` | BC-2.19.001 | `test_BC_2_19_001_register_infusion_udfs_helper_compiles_and_registers` | PASSES (registration only, no invocation) |
| `crates/prism-spec-engine/tests/infusion_tests.rs` (appended) | BC-2.19.003 | `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | PASSES (already implemented — regression guard) |

---

## Red Gate Test Results

### prism-spec-engine

```
Summary [6.817s] 598 tests run: 594 passed, 4 failed, 13 skipped
    FAIL prism-spec-engine::infusion_tests test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref
    FAIL prism-spec-engine::infusion_tests test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type
    FAIL prism-spec-engine::infusion_tests test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors
    FAIL prism-spec-engine::infusion_tests test_BC_2_19_001_infusion_loader_parses_plugin_type_spec
```

All 4 failing tests panic at `unimplemented!()` in `loader.rs`. Zero regressions (594 pre-existing tests still pass).

### test_BC_2_19_001_infusion_loader_parses_plugin_type_spec

**Failure reason:** `InfusionLoader::parse` body is `unimplemented!()`.

```
thread 'test_BC_2_19_001_infusion_loader_parses_plugin_type_spec' panicked at
crates/prism-spec-engine/src/infusion/loader.rs:42:9:
not implemented: InfusionLoader::parse — implement in S-1.14 (BC-2.19.001)
```

### test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref

**Failure reason:** `InfusionLoader::parse` body is `unimplemented!()` — never reaches the error-path assertion.

```
thread 'test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref' panicked at
crates/prism-spec-engine/src/infusion/loader.rs:42:9:
not implemented: InfusionLoader::parse — implement in S-1.14 (BC-2.19.001)
```

### test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors

**Failure reason:** `InfusionLoader::load_all` body is `unimplemented!()`.

```
thread 'test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors' panicked at
crates/prism-spec-engine/src/infusion/loader.rs:50:9:
not implemented: InfusionLoader::load_all — implement in S-1.14 (BC-2.19.001)
```

### test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type

**Failure reason:** `InfusionLoader::load_all` body is `unimplemented!()` — never reaches the error-path assertion.

```
thread 'test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type' panicked at
crates/prism-spec-engine/src/infusion/loader.rs:50:9:
not implemented: InfusionLoader::load_all — implement in S-1.14 (BC-2.19.001)
```

### prism-query

```
Summary [3.550s] 962 tests run: 962 passed, 6 skipped
```

All 962 prism-query tests pass. Zero regressions.

---

## NullSource Wiring Gap (for implementer awareness)

`infusion/mod.rs` hardwires `Arc::new(NullSource)` at 4 call sites:
- `validate_spec_against` (~line 500)
- `load_spec` (~line 535)
- `udf_descriptors` (~line 563)
- `hot_reload` (~line 662)

BC-2.19.001 v1.4 postcondition requires: plugin-type `InfusionUdfDescriptor` values MUST carry
a real `Arc<PluginInfusionSource>`, not `Arc<NullSource>`. The implementer must replace
`Arc::new(NullSource)` with `Arc::new(PluginInfusionSource::new(...))` at the relevant sites
when wiring `InfusionLoader::load_all` and `InfusionRegistry::load_spec`.

---

## Implementer Handoff Instructions

**Next step:** Implement the stubs in order. Make each Red Gate test pass, one at a time.

**Micro-commit order:**
1. Implement `InfusionLoader::parse` in `loader.rs` — parses `.infusion.toml` TOML into `InfusionSpec`.
   → `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` turns green.
   → `test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref` turns green.
2. Implement `InfusionLoader::load_all` in `loader.rs` — for `source.type = "plugin"`, constructs
   `PluginInfusionSource` (with `plugin_id`, `config`, `runtime`) and wires into `InfusionRegistry`.
   → `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` turns green.
   → `test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type` turns green.
3. Replace `Arc::new(NullSource)` with real `Arc<PluginInfusionSource>` at the 4 call sites in `mod.rs`.
4. Implement `InfusionAsyncUdf::invoke_async_with_args` in `infusion_udf.rs` — calls
   `self.descriptor.source.enrich_single` for each row, maps result to Arrow `StringArray`.
5. Implement `PluginInfusionSource::enrich_single` in `plugin_bridge.rs` — delegates to
   `PluginRuntime::enrich_single(plugin_id, input, input_type, config)`.
6. Wire `register_infusion_udfs` into `engine.rs` at both `build_session_context` call sites
   (AC-003 / merge-coordination with S-3.13).
7. Run `just check` for final pre-push gate.

**Forbidden patterns:**
- NEVER add `tracing::*!(event_type=...)` without a BC-2.16.002 catalog row (SAP-1).
- NEVER add `Arc::new(SomeThing::placeholder())` in the boot path (Standing Rule 3 §4).
- NEVER call `reqwest::Client::new()` without `.timeout(Duration::from_secs(30))`.
- NEVER add `InfusionError::PluginCallFailed` without updating the error-taxonomy.md.
