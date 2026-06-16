# Demo Evidence Report — S-DEMO-ENRICHMENT-PIVOT-001

**Story:** Infusion Engine Plugin-Bridge Prerequisites — Forward-Subset of S-1.14-REDO for Demo
**Story version:** v1.12
**Branch:** feature/S-DEMO-ENRICHMENT-PIVOT-001
**Code under test:** feature/S-DEMO-ENRICHMENT-PIVOT-001 (current feature HEAD — pinning volatile SHAs violates TD-VSDD-091)
**Evidence date:** 2026-06-15
**Product type:** Library/infrastructure (no CLI surface — evidence is test-execution, not VHS/Playwright)
**Red Gate count:** 7 (rg7 — expanded from rg5 at D-1181; EC-006 validate_credentials + EC-007 validate_pipe_stage_columns implemented and wired)

---

## Coverage Summary

| # | Test Name | AC | BC Clause | Status |
|---|-----------|-----|-----------|--------|
| RG-1 | `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | AC-001 | BC-2.19.001 postcondition | PASS |
| RG-2 | `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | AC-002 | BC-2.19.001 postcondition (parse-phase) | PASS |
| RG-3 | `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | AC-003 | BC-2.19.001 postcondition | PASS |
| RG-4 | `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime` | AC-004 | BC-2.19.001 postcondition | PASS |
| RG-5 | `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | AC-005 | BC-2.19.003 postcondition | PASS |
| RG-6a | `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` | EC-006 (happy path) | BC-2.19.001 postcondition (`validate_credentials`) | PASS |
| RG-6b | `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` | EC-006 (error path) | BC-2.19.001 postcondition (`validate_credentials`) | PASS |
| RG-7a | `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` | EC-007 (happy path) | BC-2.19.001 postcondition (`validate_pipe_stage_columns`) | PASS |
| RG-7b | `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` | EC-007 (error path) | BC-2.19.001 postcondition (`validate_pipe_stage_columns`) | PASS |
| AC-006 | Grammar doc grep probe | AC-006 | BC-2.19.001 postcondition (query language surface) | PASS |

All 7 Red Gate test entries pass (RG-6 and RG-7 each comprise 2 test names per the story spec). AC-006 (doc amendment) verified by adversary grep probe.

---

## Commands Run and Output

### RG-1 — AC-001 — `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_infusion_loader_parses_plugin_type_spec)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.61s
    Starting 1 test across 38 binaries (614 tests skipped)
        PASS [   0.015s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_infusion_loader_parses_plugin_type_spec
     Summary [   0.016s] 1 test run: 1 passed, 614 skipped
```

**What is asserted:** `InfusionLoader::parse` given TOML with `source.type = "plugin"` and two
`[[infusion.fields]]` entries returns `Ok(InfusionSpec)` with `InfusionType::Plugin`, two
parsed fields (`threat_score`, `is_known_bad`), and `plugin_config.plugin_path` non-empty.

---

### RG-2 — AC-002 — `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
    Starting 1 test across 38 binaries (614 tests skipped)
        PASS [   0.042s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors
     Summary [   0.043s] 1 test run: 1 passed, 614 skipped
```

**What is asserted (parse-phase outcome per BC-2.19.001 v1.7 two-phase wiring):**
`InfusionLoader::load_all` on a directory containing a plugin-type `.infusion.toml` returns
an `InfusionRegistry` where `udf_descriptors()` is non-empty, and each descriptor has
`plugin_id` and `config` fields populated from the TOML. The test asserts the parse-phase
observable (descriptor count + field population) — not the runtime-wiring phase
(`load_spec_with_runtime`), per BC-2.19.001 v1.7 two-phase wiring disambiguation.

---

### RG-3 — AC-003 — `test_BC_2_19_001_plugin_udfs_registered_in_session_context`

**Command:**
```
cargo nextest run -p prism-query -E 'test(test_BC_2_19_001_plugin_udfs_registered_in_session_context)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.92s
    Starting 1 test across 13 binaries (971 tests skipped)
        PASS [   0.048s] (1/1) prism-query::bc_2_19_001_plugin_udf_registration_test test_BC_2_19_001_plugin_udfs_registered_in_session_context
     Summary [   0.049s] 1 test run: 1 passed, 971 skipped
```

**What is asserted (anti-false-green hardening per AC-003):**
The test registers an `InfusionAsyncUdf` via
`ctx.register_udf(AsyncScalarUDF::new(Arc::new(impl)).into_scalar_udf())` and executes a
DataFusion query through the `SessionContext`. It asserts: (a) the UDF is callable by name
in a DataFusion query plan, (b) the returned `RecordBatch` column contains the expected
sentinel value produced by `invoke_async_with_args`, and (c) an `Arc<AtomicUsize>` call
counter is asserted `> 0` after execution — proving `invoke_async_with_args` was called,
not the sync `invoke_with_args` fallback.

---

### RG-4 — AC-004 — `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
    Starting 1 test across 38 binaries (614 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime
     Summary [   0.011s] 1 test run: 1 passed, 614 skipped
```

**What is asserted:** `PluginInfusionSource::new("threat_intel", config, Arc<PluginRuntime>)`
constructs with `plugin_id = "threat_intel"`. `enrich_single("192.168.1.1", "ip")` returns
`None` after the runtime returns `PluginError::NotLoaded` (plugin not loaded in runtime) —
demonstrating the CRIT-3 closure: `NotLoaded` arm maps-log-None rather than panicking via
`todo!()`. The unavailable-runtime contract is `Ok(None)` + `tracing::warn!` per
BC-2.19.001 v1.5 / CRIT-3 closure.

---

### RG-5 — AC-005 — `test_BC_2_19_003_is_api_backed_true_for_plugin_type`

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_003_is_api_backed_true_for_plugin_type)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
    Starting 1 test across 38 binaries (614 tests skipped)
        PASS [   0.011s] (1/1) prism-spec-engine::infusion_tests test_BC_2_19_003_is_api_backed_true_for_plugin_type
     Summary [   0.012s] 1 test run: 1 passed, 614 skipped
```

**What is asserted (regression confirmation — `InfusionRegistry::is_api_backed` already
implemented):** `registry.is_api_backed("threat_score")` returns `true` for a plugin-type
infusion field, and `registry.is_api_backed("unknown_field")` returns `false` for a name
not present in the registry. Both BC-2.19.003 postcondition cases covered.

---

### RG-6 — EC-006 — `validate_credentials` (2 tests — happy path + error path)

EC-006 validates that `InfusionLoader::parse` rejects specs with empty `env_var` on a
`CredentialRef` at parse time. Implemented and wired into `InfusionLoader::parse` in
PIVOT-001 (D-1181).

**Command:**
```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference) + test(test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
    Starting 2 tests across 38 binaries (613 tests skipped)
        PASS [   0.011s] (1/2) prism-spec-engine::infusion_tests test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference
        PASS [   0.011s] (2/2) prism-spec-engine::infusion_tests test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential
     Summary [   0.012s] 2 tests run: 2 passed, 613 skipped
```

**What is asserted:**

- `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` (happy path):
  `InfusionLoader::parse` with a spec containing a `CredentialRef` whose `env_var` is a
  non-empty string returns `Ok(InfusionSpec)` — valid credential references pass through
  `validate_credentials` without error.

- `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` (error path):
  `InfusionLoader::parse` with a spec containing a `CredentialRef` whose `env_var` is an
  empty string returns `Err(InfusionError::...)` at parse time — empty `env_var` is rejected
  by `validate_credentials` before the spec is accepted.

---

### RG-7 — EC-007 — `validate_pipe_stage_columns` (2 tests — happy path + error path)

EC-007 validates that `InfusionLoader::parse` rejects specs where a `pipe_stage.adds_columns`
entry names a column not declared in `spec.fields`. Implemented and wired into
`InfusionLoader::parse` in PIVOT-001 (D-1181).

**Command:**
```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields) + test(test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
    Starting 2 tests across 38 binaries (613 tests skipped)
        PASS [   0.011s] (1/2) prism-spec-engine::infusion_tests test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference
        PASS [   0.012s] (2/2) prism-spec-engine::infusion_tests test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields
     Summary [   0.012s] 2 tests run: 2 passed, 613 skipped
```

**What is asserted:**

- `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` (happy path):
  `InfusionLoader::parse` with a spec where every column name in `pipe_stage.adds_columns`
  also appears in `spec.fields` returns `Ok(InfusionSpec)` — consistent pipe-stage column
  references pass `validate_pipe_stage_columns` without error.

- `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` (error path):
  `InfusionLoader::parse` with a spec where `pipe_stage.adds_columns` contains a name
  absent from `spec.fields` returns `Err(InfusionError::...)` at parse time — unknown column
  references are caught by `validate_pipe_stage_columns` before the spec is accepted.

---

### AC-006 — Grammar doc grep probe (doc amendment, no Red Gate test)

**Command:**
```
grep -n "ENRICH.*ON\|enrich.*\bon\b" .factory/specs/domain-spec/prismql-grammar.md
```

**Output:** (no output — 0 matches in EBNF/pipe-stage sections)

**Verification:** The pipe-stage table reads `enrich infusion(field_path)` and the EBNF
stage rule reads `"ENRICH" , ident , "(" , field_path , ")"` — the brownfield-era
`ENRICH ident ON field` form is absent from both sections.

---

## Error-Path Evidence

The following error-path behaviors are demonstrated by passing tests:

| Error Code | Trigger | Backing Test | Status |
|------------|---------|--------------|--------|
| E-INFUSE-004 | `source.type = "maxmind_mmdb"` (unsupported) passed to `load_all` — asserts `InfusionError::UnknownSourceType` variant | `test_BC_2_19_001_load_all_returns_error_for_unsupported_source_type` | PASS |
| E-INFUSE-003 | Plugin-type spec missing `plugin_ref` field | `test_BC_2_19_001_infusion_loader_rejects_plugin_spec_without_plugin_ref` | PASS |
| E-INFUSE-002 | Two plugin specs registering the same UDF field name | `test_register_infusion_udfs_duplicate_name_emits_e_infuse_002_with_infusion_id` | PASS |
| NULL short-circuit | `NULL` input row bypasses `enrich_single` without calling the plugin | `test_null_input_row_short_circuits_to_null_without_calling_enrich_single` | PASS |
| EC-006 empty `env_var` | `CredentialRef` with empty `env_var` rejected at parse time | `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` | PASS |
| EC-007 unknown column ref | `pipe_stage.adds_columns` names a column absent from `spec.fields` | `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` | PASS |

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
  -E 'test(test_register_infusion_udfs_duplicate_name_emits_e_infuse_002_with_infusion_id) + test(test_null_input_row_short_circuits_to_null_without_calling_enrich_single)'
```

**Output:**
```
Starting 2 tests across 13 binaries (970 tests skipped)
    PASS [   0.044s] (1/2) prism-query infusion_udf::tests::test_register_infusion_udfs_duplicate_name_emits_e_infuse_002_with_infusion_id
    PASS [   0.050s] (2/2) prism-query infusion_udf::tests::test_null_input_row_short_circuits_to_null_without_calling_enrich_single
Summary [   0.053s] 2 tests run: 2 passed, 970 skipped
```

---

## Full Infusion Suite (No Regression)

All infusion-related tests in both crates pass with no failures:

**prism-spec-engine `test(infusion)` suite:**
```
cargo nextest run -p prism-spec-engine -E 'test(infusion)' --no-fail-fast
Summary [   1.017s] 12 tests run: 12 passed, 599 skipped
```

**prism-query `test(infusion)` suite:**
```
cargo nextest run -p prism-query -E 'test(infusion)' --no-fail-fast
Summary [   0.541s] 6 tests run: 6 passed, 966 skipped
```

---

## BC Traceability

| Test | BC Clause | AC / EC |
|------|-----------|---------|
| `test_BC_2_19_001_infusion_loader_parses_plugin_type_spec` | BC-2.19.001 postcondition: each field registers exactly one UDF descriptor | AC-001 |
| `test_BC_2_19_001_load_all_plugin_type_produces_udf_descriptors` | BC-2.19.001 postcondition: parse-phase descriptor production | AC-002 |
| `test_BC_2_19_001_plugin_udfs_registered_in_session_context` | BC-2.19.001 postcondition: each field registered as DataFusion scalar UDF | AC-003 |
| `test_BC_2_19_001_plugin_bridge_delegates_to_plugin_runtime` | BC-2.19.001 postcondition: plugin-type source executes via plugin bridge | AC-004 |
| `test_BC_2_19_003_is_api_backed_true_for_plugin_type` | BC-2.19.003 postcondition: API-backed UDFs rejected in detection rule filters | AC-005 |
| Grammar doc grep probe | BC-2.19.001 postcondition: query language surface matches implemented parser | AC-006 |
| `test_BC_2_19_001_parse_accepts_spec_with_valid_credential_reference` | BC-2.19.001 postcondition (`validate_credentials` — happy path) | EC-006 |
| `test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential` | BC-2.19.001 postcondition (`validate_credentials` — error path) | EC-006 |
| `test_BC_2_19_001_parse_accepts_spec_with_pipe_stage_matching_fields` | BC-2.19.001 postcondition (`validate_pipe_stage_columns` — happy path) | EC-007 |
| `test_BC_2_19_001_parse_rejects_pipe_stage_with_unknown_column_reference` | BC-2.19.001 postcondition (`validate_pipe_stage_columns` — error path) | EC-007 |

---

## Notes

- This story is an infrastructure/library prereq with no CLI or browser surface. Evidence
  is test-execution output, not VHS or Playwright recordings. This is the correct form for
  this story type per the Demo Recorder operating procedure.
- **rg5 → rg7 expansion (D-1181):** Two validators were implemented and wired into
  `InfusionLoader::parse` in PIVOT-001: `validate_credentials` (EC-006) and
  `validate_pipe_stage_columns` (EC-007). Each validator has a happy-path test and an
  error-path test, adding 4 test names across RG-6 and RG-7. Story version bumped v1.7
  → v1.12 to capture this expansion (v1.10→v1.11→v1.12 were prose-only spec precision changes).
- AC-004 delegates to `PluginRuntime::enrich_single` via the confirmed production signature.
  When the plugin is not loaded in the runtime, `enrich_single` returns `None` after
  map-log (CRIT-3 closure) rather than panicking. The S-1.15 WASM runtime is not yet
  operational at merge time; the unavailable-runtime contract is `Ok(None)` + `tracing::warn!`
  per BC-2.19.001 v1.5 / CRIT-3 closure.
- AC-006 is a `.factory/` doc amendment verified by the adversary grep probe. No Red Gate
  test exists for a doc edit; the adversary probe is the canonical verification mechanism
  specified in the story.
