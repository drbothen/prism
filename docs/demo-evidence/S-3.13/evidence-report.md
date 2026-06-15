# S-3.13 Demo Evidence Report

**Story:** S-3.13 v1.15 — prism-query: Dynamic Table Availability
**Code under test:** f2b778b8405cbe111f9b5c96a2536e6046ba083c (feature/S-3.13)
**Evidence date:** 2026-06-15
**Product type:** CLI/Library (Rust crate — query-engine infrastructure)
**Recording method:** TEST-EXECUTION (no end-user UI; all acceptance criteria are validated via automated tests against the production codebase)
**ACs in scope:** AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-8 (7 total; AC-7 DEFERRED-TO-S-5.03)
**Red Gate tests:** 22 enrolled, 22 passing

---

## Execution Summary

| Test Suite | Command | Tests Run | Passed | Failed |
|------------|---------|-----------|--------|--------|
| prism-query table_registry | `cargo nextest run -p prism-query -E 'test(table_registry)'` | 20 | 20 | 0 |
| prism-mcp BC_2_16_001 boundary | `cargo nextest run -p prism-mcp -E 'test(BC_2_16_001)'` | 1 | 1 | 0 |
| prism-bin boot swap-listener | `cargo nextest run -p prism-bin -E 'test(wire_table_registry_swap_listener)'` | 3 | 3 | 0 |
| **Total** | | **24** | **24** | **0** |

---

## Acceptance Criteria Coverage

### AC-1: `is_registered()` reflects loaded specs

Given `armis.sensor.toml` is loaded and `crowdstrike.sensor.toml` is NOT loaded, `is_registered("crowdstrike_alerts")` returns `false` and `is_registered("armis_alerts")` returns `true`. (Traces to BC-2.16.001.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_16_001_register_sensor_populates_is_registered` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_16_001_unregistered_sensor_is_not_registered` | `prism-query/src/tests/table_registry_tests.rs` | PASS |

**Key assertions:** `is_registered("armis_alerts") == true` after `register_sensor(armis_spec)`; `is_registered("crowdstrike_alerts") == false` when crowdstrike spec not registered.

---

### AC-2: Unregistered table query returns E-QUERY-037 before fan-out

Given `SELECT * FROM crowdstrike_alerts` when CrowdStrike is not configured, the query planner returns `PrismError::TableNotAvailable` (E-QUERY-037) with sensor context before any fan-out occurs. (Traces to BC-2.11.001.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_11_001_table_not_available_returns_e_query_037` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_no_sensors_configured_returns_e_query_037_empty_list` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params` | `prism-query/src/tests/table_registry_tests.rs` | PASS |

**Key assertions:** `PrismError::TableNotAvailable` returned; error contains `"Sensor 'crowdstrike' is not configured"`; MCP error code maps to `-32602` (INVALID_PARAMS, not `-32000` InternalError); no fan-out initiated.

**Error path:** With no sensors configured, `available_sensors = "[]"` and `did_you_mean = ""` — verified by `test_BC_2_11_001_no_sensors_configured_returns_e_query_037_empty_list`.

---

### AC-3: Levenshtein `did_you_mean` suggestion within distance ≤ 3

Given `SELECT * FROM crowdstrike_alert` (missing `s`), E-QUERY-037 includes `did_you_mean: " Did you mean: 'crowdstrike_alerts'?"` (distance 1 ≤ threshold 3 via `strsim::levenshtein`). Beyond distance 3, `did_you_mean = ""`. (Traces to BC-2.11.001.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_11_001_did_you_mean_suggestion_within_threshold` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_did_you_mean_empty_when_distance_exceeds_threshold` | `prism-query/src/tests/table_registry_tests.rs` | PASS |

**Key assertions (success path):** `did_you_mean` field contains `" Did you mean: 'crowdstrike_alerts'?"` for typo `crowdstrike_alert` (distance 1).
**Key assertions (error path):** `did_you_mean == ""` for `totallywrong` (min Levenshtein > 3 across all registered tables).

---

### AC-4: Hot-reload add registers tables immediately

Given `armis.sensor.toml` is added via hot-reload, after the config swap completes `is_registered("armis_alerts")` returns `true`. Boot glue (`wire_table_registry_swap_listener`) correctly dispatches `register_sensor` on the add path. (Traces to BC-2.16.007; MCP notification dispatch DEFERRED-TO-S-5.03.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_16_007_hot_reload_add_sensor_registers_tables` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_wire_table_registry_swap_listener_add_sensor_registers_tables` | `prism-bin/src/boot.rs::table_registry_swap_listener_tests` | PASS |
| `test_wire_table_registry_swap_listener_add_then_remove_across_swaps` | `prism-bin/src/boot.rs::table_registry_swap_listener_tests` | PASS |

**Key assertions:** After config swap with new sensor spec, `is_registered("armis_alerts") == true`; production `wire_table_registry_swap_listener` boot glue exercises the real registration path.

---

### AC-5: Hot-reload remove deregisters tables; in-flight queries unaffected

Given `claroty.sensor.toml` is removed via hot-reload, after the config swap `is_registered("claroty_devices")` returns `false`. Queries started before removal complete against the old `ConfigSnapshot` (CI-007 in-flight isolation). (Traces to BC-2.16.007.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_16_007_hot_reload_remove_sensor_deregisters_tables` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_wire_table_registry_swap_listener_remove_sensor_deregisters_tables` | `prism-bin/src/boot.rs::table_registry_swap_listener_tests` | PASS |
| `test_wire_table_registry_swap_listener_add_then_remove_across_swaps` | `prism-bin/src/boot.rs::table_registry_swap_listener_tests` | PASS |

**Key assertions:** After removal swap, `is_registered("claroty_devices") == false`; the add-then-remove lifecycle test confirms correct state across multiple sequential swaps.

---

### AC-6: `explain_query` lists only live registered tables

Given `explain_query` is called, `"available_tables"` in the response reflects `table_registry.registered_tables()` — not a static list. The MCP JSON serialization boundary does not drop the field. (Traces to BC-2.16.001.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_16_001_explain_query_lists_only_registered_tables` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_16_001_AC6_explain_query_json_response_contains_available_tables` | `prism-mcp/src/server.rs::tests` | PASS |

**Key assertions (success path):** `explain_query` response `available_tables` matches the live registry (contains only sensors present in the current `ConfigSnapshot`; does not include unconfigured sensors).
**Key assertions (error path / serialization boundary):** Serialized MCP JSON response contains the `available_tables` field; field is not dropped at the MCP serialization layer — verified by the `prism-mcp` server-level test added at fix-burst `ddcf16bf`.

---

### AC-7: DEFERRED-TO-S-5.03

`prism://config/clients` MCP resource backed by `TableRegistry::registered_tables()` is an SS-10 MCP resource-framework deliverable. S-3.13 delivers the `TableRegistry` API; S-5.03 wires it into `resources.rs`. No test in this story's scope. Orchestrator adjudication: ACCEPT-MOVE (2026-06-14).

---

### AC-8: Plan-time gate is mode-agnostic (SQL, filter, pipe, DML subquery)

SQL mode, filter mode, and pipe mode queries referencing unregistered tables all return E-QUERY-037 before fan-out. DML WHERE-IN subqueries referencing external unregistered tables are also caught by `extract_sources_from_ast_for_gate`. (Traces to BC-2.11.001.)

| Test Name | Location | Result |
|-----------|----------|--------|
| `test_BC_2_11_001_mode_agnostic_plan_time_gate_sql` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_mode_agnostic_plan_time_gate_filter` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_mode_agnostic_plan_time_gate_pipe` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_unregistered` | `prism-query/src/tests/table_registry_tests.rs` | PASS |
| `test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_registered_passes` | `prism-query/src/tests/table_registry_tests.rs` | PASS |

**Key assertions (success path — registered table):** DML WHERE-IN subquery with a registered table passes the gate without error (`test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_registered_passes` — no spurious E-QUERY-037 on valid queries).
**Key assertions (error path — unregistered table):** All four query modes (SQL, filter, pipe, DML subquery) return `PrismError::TableNotAvailable` before any fan-out materializes; gate fires at validation pass in `engine.rs` before `materialize_query`.

---

## Supplementary Tests (Non-AC)

These tests back future story S-5.03 by exercising registry accessor APIs, and confirm atomicity of re-registration:

| Test Name | Coverage | Result |
|-----------|----------|--------|
| `test_BC_2_16_001_registered_sets_reflect_only_configured_sensors` | `registered_sensor_ids()` + `registered_tables()` accessor correctness | PASS |
| `test_BC_2_16_001_register_sensor_reregistration_atomic_no_transient_absence` | 4 reader threads × 400 re-registration cycles; asserts `is_registered("crowdstrike_alerts")` is never `false` during re-registration (no transient table absence) — mutation-confirmed | PASS |
| `test_BC_2_16_007_hot_reload_schema_change_reregisters` | Spec update triggers deregister + re-register; `is_registered` reflects updated state | PASS |
| `test_BC_2_11_001_table_not_available_display_format` | `PrismError::TableNotAvailable` `Display` impl assembles correct human-readable message | PASS |

---

## Complete Red Gate Test Roster (22/22 PASS)

All 22 Red Gate tests enrolled in story v1.15 frontmatter are confirmed passing:

| # | Test Name | AC | Location | Result |
|---|-----------|----|-----------| -------|
| 1 | `test_BC_2_11_001_table_not_available_returns_e_query_037` | AC-2 | prism-query | PASS |
| 2 | `test_BC_2_11_001_did_you_mean_suggestion_within_threshold` | AC-3 | prism-query | PASS |
| 3 | `test_BC_2_11_001_did_you_mean_empty_when_distance_exceeds_threshold` | EC-11-126 | prism-query | PASS |
| 4 | `test_BC_2_11_001_mode_agnostic_plan_time_gate_sql` | AC-8 | prism-query | PASS |
| 5 | `test_BC_2_11_001_mode_agnostic_plan_time_gate_filter` | AC-8 | prism-query | PASS |
| 6 | `test_BC_2_11_001_mode_agnostic_plan_time_gate_pipe` | AC-8 | prism-query | PASS |
| 7 | `test_BC_2_16_001_register_sensor_populates_is_registered` | AC-1 | prism-query | PASS |
| 8 | `test_BC_2_16_001_unregistered_sensor_is_not_registered` | AC-1 | prism-query | PASS |
| 9 | `test_BC_2_16_007_hot_reload_add_sensor_registers_tables` | AC-4 | prism-query | PASS |
| 10 | `test_BC_2_16_007_hot_reload_remove_sensor_deregisters_tables` | AC-5 | prism-query | PASS |
| 11 | `test_BC_2_16_007_hot_reload_schema_change_reregisters` | EC-11-123 | prism-query | PASS |
| 12 | `test_BC_2_11_001_no_sensors_configured_returns_e_query_037_empty_list` | EC-11-125 | prism-query | PASS |
| 13 | `test_BC_2_11_001_e_query_037_mcp_maps_to_invalid_params` | AC-2 | prism-query | PASS |
| 14 | `test_BC_2_16_001_explain_query_lists_only_registered_tables` | AC-6 | prism-query | PASS |
| 15 | `test_BC_2_16_001_AC6_explain_query_json_response_contains_available_tables` | AC-6 | prism-mcp | PASS |
| 16 | `test_BC_2_16_001_register_sensor_reregistration_atomic_no_transient_absence` | EC-11-123 | prism-query | PASS |
| 17 | `test_BC_2_16_001_registered_sets_reflect_only_configured_sensors` | supplementary | prism-query | PASS |
| 18 | `test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_unregistered` | AC-8 | prism-query | PASS |
| 19 | `test_BC_2_11_001_mode_agnostic_gate_dml_filter_in_subquery_registered_passes` | AC-8 | prism-query | PASS |
| 20 | `test_wire_table_registry_swap_listener_add_sensor_registers_tables` | AC-4 | prism-bin | PASS |
| 21 | `test_wire_table_registry_swap_listener_remove_sensor_deregisters_tables` | AC-5 | prism-bin | PASS |
| 22 | `test_wire_table_registry_swap_listener_add_then_remove_across_swaps` | AC-4, AC-5 | prism-bin | PASS |

---

## Key Architecture Assertions Confirmed

- **Plan-time gate fires before fan-out:** E-QUERY-037 returned from validation pass in `engine.rs` before `materialize_query` is invoked — confirmed by all mode-agnostic gate tests (AC-8).
- **RwLock read non-blocking:** `is_registered()` acquires `RwLock::read()` — non-exclusive, does not block concurrent query execution.
- **Atomic re-registration:** Concurrency test (`test_BC_2_16_001_register_sensor_reregistration_atomic_no_transient_absence`) confirms 4 reader threads see no transient `false` across 400 re-registration cycles — mutation-confirmed (a non-atomic impl fails this test).
- **MCP error code correctness:** `PrismError::TableNotAvailable` maps to `-32602` (INVALID_PARAMS) via explicit arm in `prism-mcp/src/error_mapping.rs` — not the `#[non_exhaustive]` catch-all which would produce `-32000`.
- **strsim dependency:** `strsim = "0.11"` added as direct dep to `prism-query/Cargo.toml`; `strsim::levenshtein` used (not `edit-distance`) per architect decision D-1163.
- **HotReloadWatcher not introduced:** Integration uses `ConfigManager` surface (`register_swap_listener`, `process_spec_changes`, `add_sensor_spec`) per D-1163 constraint.

---

## Execution Commands (Reproducible)

```bash
# All table_registry tests in prism-query (20 tests, covers AC-1 through AC-6, AC-8)
cargo nextest run -p prism-query -E 'test(table_registry)'

# AC-6 serialization boundary test in prism-mcp (1 test)
cargo nextest run -p prism-mcp -E 'test(BC_2_16_001)'

# AC-4/AC-5 boot swap-listener wiring tests in prism-bin (3 tests)
cargo nextest run -p prism-bin -E 'test(wire_table_registry_swap_listener)'

# E-QUERY-037 plan-time gate tests specifically (13 tests, BC-2.11.001 coverage)
cargo nextest run -p prism-query -E 'test(BC_2_11_001)'
```

All commands executed from worktree root: `/Users/jmagady/Dev/prism/.worktrees/S-3.13/`
