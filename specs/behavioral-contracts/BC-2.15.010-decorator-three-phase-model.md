---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-026"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to:
  - "CAP-026"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.010: Decorator Three-Phase Model — Config-Time, Query-Time, Periodic

## Description

Decorator values are sourced from three phases with distinct refresh cadences.
Config-time decorators are computed once at startup and on reload, providing stable
metadata about the runtime environment. Query-time decorators are computed fresh for
every query, providing per-invocation context. Periodic decorators are refreshed on a
configurable interval (default 300s) and cached in RocksDB for persistence across
restarts, using last-known-good values when refresh fails.

Phases apply in order with last-write-wins for overlapping keys: periodic > query-time
> config-time. Only the first row of a periodic decorator query is used.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — all phases | startup complete, periodic refresh done, interactive query | All three phase values present; periodic wins on overlapping keys |
| Periodic refresh fails | health check times out | Stale cached values used; warning logged; no error to query caller |
| Config reload | `_client_name` changed in TOML | Next query uses new name; in-flight query unaffected |
| Pre-first-refresh query | query before periodic refresh completes | Periodic fields null; config-time + query-time fields present |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify phase priority ordering (periodic > query-time > config-time) |
| (placeholder) | VP to be assigned — verify stale-on-failure for periodic phase |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
