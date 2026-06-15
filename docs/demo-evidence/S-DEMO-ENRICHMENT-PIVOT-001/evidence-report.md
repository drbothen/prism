# Demo Evidence Report — S-DEMO-ENRICHMENT-PIVOT-001

**Story:** Infusion Engine Plugin-Bridge Prerequisites — Forward-Subset of S-1.14-REDO for Demo
**Story version:** v1.7
**Branch:** feature/S-DEMO-ENRICHMENT-PIVOT-001
**Code under test (LOCAL-converged):** 0d4978c3
**Evidence date:** 2026-06-15
**Product type:** Library/infrastructure (no CLI surface — evidence is test-execution, not VHS/Playwright)

---

## Coverage Summary

| AC | Title | Backing Test(s) | Status |
|----|-------|-----------------|--------|
| AC-001 | InfusionLoader::parse accepts source.type = "plugin" | `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | PASS |
| AC-002 | load_all builds InfusionRegistry with plugin-type descriptors (parse-phase) | `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | PASS |
| AC-003 | DataFusion async UDF registration wires plugin descriptors into SessionContext | `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | PASS |
| AC-004 | PluginInfusionSource::enrich_single delegates to PluginRuntime::enrich_single | `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime` | PASS |
| AC-005 | is_api_backed() returns true for plugin-type infusion UDFs | `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | PASS |
| AC-006 | Grammar doc updated: enrich pipe-stage table and EBNF use function-call form | `grep -n "ENRICH.*ON\|enrich.*\bon\b" prismql-grammar.md` → 0 matches | PASS |

All 5 Red Gate tests pass. AC-006 (doc amendment) verified by adversary grep probe.

---

## Commands Run and Output

### AC-001 — `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_infusion_loader_parses_plugin_type_spec)'
```

**Output (key lines):**
```
Starting 1 test across 38 binaries (610 tests skipped)
    PASS [   0.012s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_infusion_loader_parses_plugin_type_spec
Summary [   0.013s] 1 test run: 1 passed, 610 skipped
```

**What is asserted:** `InfusionLoader::parse` given TOML with `source.type = "plugin"` and two
`[[infusion.fields]]` entries returns `Ok(InfusionSpec)` with `InfusionType::Plugin`, two
parsed fields (`threat_score`, `is_known_bad`), and `plugin_config.plugin_path` non-empty.

---

### AC-002 — `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors)'
```

**Output (key lines):**
```
Starting 1 test across 38 binaries (610 tests skipped)
    PASS [   0.016s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors
Summary [   0.016s] 1 test run: 1 passed, 610 skipped
```

**What is asserted (parse-phase outcome per BC-2.19.001 v1.5 two-phase wiring):**
`InfusionLoader::load_all` on a directory containing a plugin-type `.infusion.toml` returns
an `InfusionRegistry` where `udf_descriptors()` is non-empty, and each descriptor has
`plugin_id` and `config` fields populated from the TOML. The test does NOT assert on
`Arc<NullSource>` downcast — it asserts the parse-phase observable (descriptor count +
field population), not the runtime-wiring phase (`load_spec_with_runtime`).

---

### AC-003 — `test_BC_2_19_001_plugin_udfs_registered_in_session_context`

**Command:**
```
cargo nextest run -p prism-query -E 'test(test_BC_2_19_001_plugin_udfs_registered_in_session_context)'
```

**Output (key lines):**
```
Starting 1 test across 13 binaries (971 tests skipped)
    PASS [   0.051s] (1/1) prism-query::bc_2_19_001_plugin_udf_registration_test test_BC_2_19_001_plugin_udfs_registered_in_session_context
Summary [   0.052s] 1 test run: 1 passed, 971 skipped
```

**What is asserted (anti-false-green hardening per AC-003):**
The test registers an `InfusionAsyncUdf` via `ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` and executes a DataFusion query through the `SessionContext`. It asserts: (a) the UDF is callable by name in a DataFusion query plan, (b) the returned `RecordBatch` column contains the expected sentinel value produced by `invoke_async_with_args`, and (c) an `Arc<AtomicUsize>` call counter is asserted `> 0` after execution — proving `invoke_async_with_args` was called, not the sync `invoke_with_args` fallback.

---

### AC-004 — `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime)'
```

**Output (key lines):**
```
Starting 1 test across 38 binaries (610 tests skipped)
    PASS [   0.112s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime
Summary [   0.122s] 1 test run: 1 passed, 610 skipped
```

**What is asserted:** `PluginInfusionSource::new("threat_intel", config, Arc<PluginRuntime>)`
constructs with `plugin_id = "threat_intel"` (structural assertion distinguishing it from
`NullSource`, which has no `plugin_id` field). `enrich_single("192.168.1.1", "ip")` returns
`None` after the runtime returns `PluginError::NotLoaded` (plugin not loaded in runtime) —
demonstrating the CRIT-3 fix: `NotLoaded` arm maps-log-None rather than panicking via
`todo!()`. The `reqwest::Client` is constructed with 30s timeout per CLAUDE.md convention.

---

### AC-005 — `test_BC_2_19_003_is_api_backed_true_for_plugin_type`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_003_is_api_backed_true_for_plugin_type)'
```

**Output (key lines):**
```
Starting 1 test across 38 binaries (610 tests skipped)
    PASS [   0.015s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_003_is_api_backed_true_for_plugin_type
Summary [   0.016s] 1 test run: 1 passed, 610 skipped
```

**What is asserted (regression confirmation — `InfusionRegistry::is_api_backed` is already
implemented):** `registry.is_api_backed("threat_score")` returns `true` for a plugin-type
infusion field, and `registry.is_api_backed("unknown_field")` returns `false` for a name
not present in the registry. Both BC-2.19.003 postcondition cases covered.

---

### AC-006 — Grammar doc grep probe (doc amendment, no Red Gate test)

**Command:**
```
grep -n "ENRICH.*ON\|enrich.*\bon\b" .factory/specs/domain-spec/prismql-grammar.md
```

**Output:** (no output — 0 matches)

**Verification:** Checked that the pipe-stage table at line 330 reads
`enrich infusion(field_path)` and the EBNF stage rule at line 767 reads
`"ENRICH" , ident , "(" , field_path , ")"` — neither the brownfield-era
`ENRICH ident ON field` form nor `enrich X on Y` is present.

---

## Error-Path Evidence

The following error-path behaviors are demonstrated by passing tests:

| Error Code | Trigger | Backing Test | Status |
|------------|---------|--------------|--------|
| E-INFUSE-004 | `source.type = "maxmind_mmdb"` (unsupported) passed to `load_all` — asserts `InfusionError::UnknownSourceType` variant and error message containing `E-INFUSE-004` | `test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type` | PASS |
| E-INFUSE-003 | Plugin-type spec missing `plugin_ref` field | `test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref` | PASS |
| E-INFUSE-007 | Two plugin specs registering the same UDF field name | `test_register_infusion_udfs_duplicate_name_emits_e_infuse_007_with_infusion_id` | PASS |
| NULL short-circuit | `NULL` input row bypasses `enrich_single` without calling the plugin | `test_null_input_row_short_circuits_to_null_without_calling_enrich_single` | PASS |

**Command (error-path batch, prism-spec-engine):**
```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref) + test(test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type) + test(test_BC_2_19_001_rejects_duplicate_udf_name_across_specs) + test(test_BC_2_19_003_is_api_backed_true_for_plugin_type)' \
  --no-fail-fast
```

**Output:**
```
Starting 4 tests across 38 binaries (607 tests skipped)
    PASS [   0.014s] (1/4) prism-spec-engine::infusion_tests test_BC_2_19_001_rejects_duplicate_udf_name_across_specs
    PASS [   0.014s] (2/4) prism-spec-engine::infusion_tests test_BC_2_19_003_is_api_backed_true_for_plugin_type
    PASS [   0.017s] (3/4) prism-spec-engine::infusion_tests test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref
    PASS [   0.020s] (4/4) prism-spec-engine::infusion_tests test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type
Summary [   0.021s] 4 tests run: 4 passed, 607 skipped
```

**Command (error-path + NULL short-circuit, prism-query):**
```
cargo nextest run -p prism-query \
  -E 'test(test_register_infusion_udfs_duplicate_name_emits_e_infuse_007_with_infusion_id) + test(test_null_input_row_short_circuits_to_null_without_calling_enrich_single)'
```

**Output:**
```
Starting 2 tests across 13 binaries (970 tests skipped)
    PASS [   0.044s] (1/2) prism-query infusion_udf::tests::test_register_infusion_udfs_duplicate_name_emits_e_infuse_007_with_infusion_id
    PASS [   0.050s] (2/2) prism-query infusion_udf::tests::test_null_input_row_short_circuits_to_null_without_calling_enrich_single
Summary [   0.053s] 2 tests run: 2 passed, 970 skipped
```

---

## Full Infusion Suite (No Regression)

All infusion-related tests in both crates passed with no failures:

**prism-spec-engine `test(infusion)` suite — 12 tests:**
```
cargo nextest run -p prism-spec-engine -E 'test(infusion)' --no-fail-fast
Summary [   1.017s] 12 tests run: 12 passed, 599 skipped
```

**prism-query `test(infusion)` suite — 6 tests:**
```
cargo nextest run -p prism-query -E 'test(infusion)' --no-fail-fast
Summary [   0.541s] 6 tests run: 6 passed, 966 skipped
```

---

## BC Traceability

| Test | BC Clause | AC |
|------|-----------|----|
| `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | BC-2.19.001 postcondition: each field registers exactly one UDF descriptor | AC-001 |
| `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | BC-2.19.001 postcondition: parse-phase descriptor production | AC-002 |
| `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | BC-2.19.001 postcondition: each field registered as DataFusion scalar UDF | AC-003 |
| `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime` | BC-2.19.001 postcondition: plugin-type source executes via plugin bridge | AC-004 |
| `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | BC-2.19.003 postcondition: API-backed UDFs rejected in detection rule filters | AC-005 |
| grammar doc grep probe | BC-2.19.001 postcondition: query language surface matches implemented parser | AC-006 |

---

## Notes

- This story is an infrastructure/library prereq with no CLI or browser surface. Evidence
  is test-execution output, not VHS or Playwright recordings. This is the correct form for
  this story type per the Demo Recorder operating procedure.
- AC-004 delegates to `PluginRuntime::enrich_single` via the confirmed production signature.
  When the plugin is not loaded in the runtime, `enrich_single` returns `None` after
  map-log (CRIT-3 fix) rather than panicking. The S-1.15 WASM runtime is not yet
  operational at merge time; the annotated `todo!("S-1.15")` path has been replaced with
  the `map-log-None` pattern per the CRIT-3 adversary finding closure.
- AC-006 is a `.factory/` doc amendment verified by the adversary grep probe. No Red Gate
  test exists for a doc edit; the adversary probe (`grep -n "ENRICH.*ON\|enrich.*\bon\b"`)
  is the canonical verification mechanism specified in the story.
