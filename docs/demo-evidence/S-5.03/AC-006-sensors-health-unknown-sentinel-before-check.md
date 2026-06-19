# AC-006 — `prism://sensors/health` returns unknown sentinel before any health check

**AC:** AC-6 (BC-2.08.006 v1.5 postcondition 5)
**Modality:** Test-execution transcript — MCP stdio server (Rust)
**Test:** `test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check`
**File:** `crates/prism-mcp/tests/resources.rs`

---

## Scenario

`prism://sensors/health` is read before any `check_sensor_health` call has been made.
The health cache in `PrismContext` is empty.

Expected response: the sentinel `{ "status": "unknown", "message": "Run check_sensor_health to populate this resource." }`

This sentinel shape is distinct from the normal keyed-object response (`{ clients: {...} }`).
The handler must return `Ok(...)` — NOT an MCP error.

## Command

```
cargo nextest run -p prism-mcp -E 'test(BC_2_08_006_sensors_health_resource_returns_unknown_before_check)'
```

## Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
────────────
 Nextest run ID 5a6a8530-0652-4223-85f9-bc71f1d10021 with nextest profile: default
    Starting 4 tests across 8 binaries (240 tests skipped)
        PASS [   0.032s] (4/4) prism-mcp::resources test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check
────────────
     Summary [   0.032s] 4 tests run: 4 passed, 240 skipped
```

## Assertions verified

- `result.is_ok()` — NOT an error (BC-2.08.006: sentinel is not an error)
- `content_text.contains("unknown")` — sentinel status present
- `content_text.to_lowercase().contains("check_sensor_health")` — instructional message present

## Observed result

PASS — sentinel `{ "status": "unknown", ... }` returned (not error, not empty `clients` object) before any health check.
