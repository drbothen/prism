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
capability: "CAP-026"
---

# BC-2.15.010: Decorator Three-Phase Model — Config-Time, Query-Time, Periodic

## Preconditions
- The Prism server has started and configuration is loaded
- Decorator sources are available from configuration, session context, and periodic refresh

## Postconditions
- Decorators are populated through three distinct phases:

  **Phase 1: Config-time (static metadata)**
  - Populated once at startup and on config reload
  - Sources: TOML configuration file
  - Fields: `_client_name` (from `[clients.{id}]` section), `_prism_version` (from build metadata), sensor endpoint metadata
  - Stored in a `DecorationStore` (thread-safe map) keyed by client_id
  - Updated only on config reload (not per-query)

  **Phase 2: Query-time (per-invocation context)**
  - Populated fresh for every query execution
  - Sources: MCP tool parameters, session state
  - Fields: `_analyst_id` (from session or tool parameter), `_query_source` (from invocation context: interactive/schedule/pack), `_sensor_type`, `_sensor_instance` (from the specific adapter that fetched each record)
  - Not cached; computed inline during result materialization

  **Phase 3: Periodic (refreshed on interval)**
  - Populated on a configurable interval (default 300 seconds)
  - Sources: computed values, sensor health state
  - Fields: `_sensor_health_status` (last known health per sensor per client, refreshed periodically)
  - Cached in the RocksDB `decorators` column family (BC-2.15.001) for persistence across restarts
  - Stale values are used if refresh fails (last-known-good pattern)

- Decorator phases are applied in order: config-time values are overridden by query-time values which are overridden by periodic values (for any overlapping keys)
- Only the first row of a periodic decorator query is used (multi-row produces a warning)

## Invariants
- Config-time decorators are always available after startup (no delay)
- Query-time decorators are always fresh (computed per-query)
- Periodic decorators may be up to `refresh_interval` seconds stale; this is acceptable
- Decorator key conflicts across phases: last-write-wins (periodic > query-time > config-time)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DECOR-001` | Periodic refresh fails (e.g., health check times out) | Stale cached values used; warning logged; retry on next interval |
| `E-DECOR-002` | Config-time decorator references missing config field | Decorator value set to null; warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-037 | Config reload changes `_client_name` | New name takes effect for subsequent queries; in-flight queries use the old name |
| EC-15-038 | Periodic refresh interval set to 0 | Periodic decorators are refreshed before every query (expensive; warning logged) |
| EC-15-039 | First query executes before first periodic refresh completes | Periodic decorator values are null for that query; config-time and query-time values are present |
| EC-15-040 | 50 clients with periodic health checks | 50 health checks every 300 seconds; bounded concurrency (max 8 concurrent checks) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 |
| L2 Invariants | DI-008 |
| Priority | P0 |
