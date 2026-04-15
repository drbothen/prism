---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-025"
---

# BC-2.15.006: Resource Watchdog Initialization — Set Memory/CPU/Timeout Limits Based on Graduated Level

## Preconditions
- The Prism server is starting up
- The `watchdog.level` configuration is set in TOML (one of: `normal`, `restrictive`, `permissive`; default: `normal`)

## Postconditions
- The resource watchdog is initialized with limits based on the configured level:

  | Limit | Normal (default) | Restrictive | Permissive |
  |-------|---------|-------------|------------|
  | Memory limit (process RSS) | 512 MB | 256 MB | 4096 MB |
  | Per-query memory budget | 128 MB | 64 MB | 1024 MB |
  | Query timeout | 30 s | 15 s | 120 s |
  | Max concurrent API calls | 16 | 8 | 64 |
  | Max materialized records | 10,000 | 5,000 | 50,000 |
  | Watchdog check interval | 3 s | 3 s | 3 s |

- Individual limits can be overridden via TOML configuration: `watchdog.memory_limit_mb`, `watchdog.query_timeout_seconds`, `watchdog.max_concurrent_api_calls`, `watchdog.max_materialized_records`
- Override values take precedence over the level defaults
- The watchdog spawns a background monitoring task that checks process resource usage every `check_interval` seconds
- Current limits are logged at startup at INFO level
- The watchdog exposes current limits and usage via the `check_sensor_health` tool's response (piggybacks on existing health tool)

## Invariants
- The watchdog is always active: there is no way to disable it entirely (even `permissive` has limits)
- Level defaults are hardcoded; they cannot be removed, only overridden
- The watchdog check interval is not configurable (fixed at 3 seconds)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-WATCH-001` | Invalid `watchdog.level` value | Fatal startup error with valid levels |
| `E-WATCH-002` | Override value below safe minimum (e.g., `memory_limit_mb: 32`) | Warning logged; value clamped to minimum (64 MB memory, 5s timeout, 1000 records) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-021 | `watchdog.level: restrictive` with `watchdog.query_timeout_seconds: 60` | Override wins; timeout is 60s despite restrictive level (other limits remain restrictive) |
| EC-15-022 | System has less RAM than the configured memory limit | Watchdog still monitors; OS may OOM-kill the process before watchdog triggers |
| EC-15-023 | No watchdog configuration in TOML | Defaults to `normal` level with all default limits |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 |
| L2 Invariants | DI-019 |
| Priority | P0 |
