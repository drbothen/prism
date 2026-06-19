# AC-001 — `prism://config/clients` returns all configured clients

**AC:** AC-1 (BC-2.10.008 v1.12 postcondition 1)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Test:** `test_BC_2_10_008_config_clients_returns_all_clients`
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario

Prism instance configured with two clients:
- "acme" with `name = "Acme Corp"` in `[[orgs]]` → `display_name: "Acme Corp"`
- "globex" with no `name` field → `display_name: null`

Both sensors registered in `QueryEngine::table_registry()`.

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_10_008_config_clients_returns_all_clients)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
────────────
 Nextest run ID 736565b3-3704-489a-997f-112acd2c7154 with nextest profile: default
    Starting 1 test across 8 binaries (243 tests skipped)
        PASS [   0.032s] (1/1) prism-mcp::resources test_BC_2_10_008_config_clients_returns_all_clients
────────────
     Summary [   0.033s] 1 test run: 1 passed, 243 skipped
```

## Assertions verified

- Response is a JSON array (BC-2.10.008 postcondition 1)
- At least 2 entries for a 2-sensor config
- Each entry has `sensor_count > 0`
- Both "crowdstrike" and "claroty" appear by `client_id`
- `capabilities_summary` field absent (BC-2.10.011 surface only)

## Observed result

PASS — `prism://config/clients` returns a JSON array with entries for all registered sensors.
