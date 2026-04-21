---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-024"
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
input-hash: "572c2a9"
traces_to:
  - "CAP-024"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.006: Resource Watchdog Initialization — Set Memory/CPU/Timeout Limits Based on Graduated Level

## Description

The resource watchdog is initialized at startup with limits derived from a three-level
graduated profile (normal, restrictive, permissive). The profile sets defaults for
process memory, per-query memory budget, query timeout, concurrent API calls, and
maximum materialized records. Individual limits can be overridden via TOML keys that
take precedence over the profile defaults. The watchdog cannot be disabled; even the
permissive level enforces finite resource bounds.

A background monitoring task spawned at startup checks process resource usage every
3 seconds. Current limits are logged at INFO level for operator visibility.

## Preconditions
- The Prism server is starting up
- The `watchdog.level` configuration is set in TOML (one of: `normal`, `restrictive`, `permissive`; default: `normal`)

## Postconditions
- The resource watchdog is initialized with limits based on the configured level:

  | Limit | Normal (default) | Restrictive | Permissive |
  |-------|---------|-------------|------------|
  | Memory limit (process RSS) | 512 MB | 256 MB | 2048 MB |
  | Per-query memory budget | 200 MB | 100 MB | 512 MB |
  | Query timeout | 30 s | 15 s | 120 s |
  | Max concurrent API calls per query | 10 | 5 | 32 |
  | Max materialized records | 10,000 | 5,000 | 50,000 |
  | Watchdog check interval | 3 s | 3 s | 3 s |

- Individual limits can be overridden via TOML configuration: `watchdog.memory_limit_mb`, `watchdog.query_timeout_seconds`, `watchdog.max_concurrent_api_calls`, `watchdog.max_materialized_records`
- Override values take precedence over the level defaults
- The watchdog spawns a background monitoring task that checks process resource usage every `check_interval` seconds
- Current limits are logged at startup at INFO level
- The watchdog exposes current limits, denylisted queries, and resource history via the dedicated `watchdog_status` MCP tool (SS-15, always-visible)

## Invariants
- The watchdog is always active: there is no way to disable it entirely (even `permissive` has limits)
- Level defaults are hardcoded; they cannot be removed, only overridden
- The watchdog check interval is not configurable (fixed at 3 seconds)

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — normal level | `watchdog.level: normal` (or absent) | 512MB RSS, 200MB/query, 30s timeout; logged at INFO |
| Override wins | `watchdog.level: restrictive, watchdog.query_timeout_seconds: 60` | timeout=60s; all other limits=restrictive defaults |
| Invalid level | `watchdog.level: extreme` | Fatal startup error with valid levels listed |
| Clamped override | `watchdog.memory_limit_mb: 16` | Warning; clamped to 64MB minimum |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Override-takes-precedence is a trivial Option::unwrap_or merge, unit test coverage sufficient; watchdog-cannot-be-disabled is a hardcoded constant, code review property; enforcement side covered by VP-014/015. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
