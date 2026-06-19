# AC-008 — `prism://config/clients` reflects `TableRegistry` — excludes unregistered sensors

**AC:** AC-8 (BC-2.10.008 postcondition 1; S-3.13 `TableRegistry::registered_tables()`)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Test:** `test_BC_2_10_008_config_clients_resource_reflects_registered_tables`
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario

Config has all 4 sensors registered: crowdstrike, claroty, armis, cyberint.
`QueryEngine::table_registry()` has ONLY crowdstrike and claroty registered.

Expected: `prism://config/clients` returns ONLY crowdstrike and claroty entries.
Armis and cyberint must NOT appear (not in `TableRegistry`).

Also verified:
- `enabled_sensors` carries sensor IDs ("crowdstrike"), NOT table names ("crowdstrike_table")
  — BC-2.10.008 v1.12 postcondition 1 load-bearing assertion
- `client_id` is a real sensor ID — NOT the synthetic `"(all)"` aggregate (removed in S-5.03)

This test verifies that the resource handler calls `table_registry.registered_sensor_ids()`
(from the `TableRegistry` API delivered by S-3.13) rather than using a static config snapshot.

Prerequisite: S-3.13 merged (provides `TableRegistry::registered_tables()` / `registered_sensor_ids()` API).
S-3.13 IS merged: develop@60249ccc is the S-3.13 merge commit.

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_10_008_config_clients_resource_reflects_registered_tables)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
────────────
 Nextest run ID 70cb207e-c26c-4baa-92e4-e7752a12afe3 with nextest profile: default
    Starting 1 test across 8 binaries (243 tests skipped)
        PASS [   0.033s] (1/1) prism-mcp::resources test_BC_2_10_008_config_clients_resource_reflects_registered_tables
────────────
     Summary [   0.034s] 1 test run: 1 passed, 243 skipped
```

## Assertions verified

- `!enabled.contains("armis")` — not registered in TableRegistry, must not appear
- `!enabled.contains("cyberint")` — not registered in TableRegistry, must not appear
- `enabled_sensors` for "crowdstrike" contains `"crowdstrike"` (sensor ID), NOT `"crowdstrike_table"` (table name)
- `client_id != "(all)"` — synthetic aggregate removed; real sensor IDs used as client_id

## Observed result

PASS — `prism://config/clients` lists only sensors present in `TableRegistry.registered_sensor_ids()`; sensors absent from registry excluded; sensor IDs (not table names) in `enabled_sensors`.
