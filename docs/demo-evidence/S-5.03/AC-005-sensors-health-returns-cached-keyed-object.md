# AC-005 — `prism://sensors/health` returns cached data in keyed-object schema

**AC:** AC-5 (BC-2.08.006 v1.5 postconditions 1, 2)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Tests:**
- `test_BC_2_08_006_sensors_health_resource_returns_cached_data` — cache populated, response includes result
- `test_BC_2_08_006_sensors_health_resource_keyed_object_shape` — `sensors` is JSON object, not array
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario

After a `check_sensor_health` run at 10:30 that populated the health cache with a result for
`(client_id: "acme", sensor_id: "crowdstrike")`, `prism://sensors/health` must return:

```json
{
  "clients": {
    "acme": {
      "sensors": {
        "crowdstrike": { "sensor_id": "crowdstrike", "client_id": "acme", ... }
      }
    }
  },
  "stale": false
}
```

Key requirement: `sensors` MUST be a JSON object keyed by `sensor_id` (BC-2.08.006 v1.5
postcondition 2) — NOT a JSON array. The prior implementation emitted `"sensors": [array]`
which violated the contract; that was corrected during S-5.03.

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_08_006_sensors_health)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
────────────
 Nextest run ID 5a6a8530-0652-4223-85f9-bc71f1d10021 with nextest profile: default
    Starting 4 tests across 8 binaries (240 tests skipped)
        PASS [   0.032s] (1/4) prism-mcp::resources test_BC_2_08_006_sensors_health_zero_clients_returns_empty_object
        PASS [   0.032s] (2/4) prism-mcp::resources test_BC_2_08_006_sensors_health_resource_keyed_object_shape
        PASS [   0.032s] (3/4) prism-mcp::resources test_BC_2_08_006_sensors_health_resource_returns_cached_data
        PASS [   0.032s] (4/4) prism-mcp::resources test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check
────────────
     Summary [   0.032s] 4 tests run: 4 passed, 240 skipped
```

## Assertions verified

- `content_text.contains("crowdstrike")` — cached result present
- `sensors` field is a JSON object (BTreeMap), not an array (keyed-object schema)
- Response structure: `{ clients: { [client_id]: { sensors: { [sensor_id]: SensorHealthResult } } } }`

## Observed result

PASS — cached health data returned in correct keyed-object schema after `check_sensor_health` populates cache.
