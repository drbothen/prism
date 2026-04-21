---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-017"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "b1e4604"
traces_to: ["CAP-017"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.001: `create_schedule` MCP Tool — Create a Scheduled Query

## Description

The `create_schedule` tool persists a named scheduled query that the execution loop (BC-2.12.004) fires at the configured interval. Schedule creation is a reversible write with dry-run default: the analyst must explicitly set `dry_run: false` to activate. The PrismQL query is validated at creation time (same path as explain_query), ensuring no repeated parse errors at execution time. Splay offsets are computed per (name, client_id) to distribute API load. The active schedule cap (DI-028, default 500) is checked before persistence; at-cap requests are rejected with E-SCHED-008.

## Preconditions
- The `create_schedule` MCP tool is invoked with required parameters: `name` (unique identifier, `[a-z0-9_-]{1,64}`), `query` (PrismQL query string), `interval` (seconds, minimum 60, maximum 86400)
- Optional parameters: `clients` (array of client IDs or null for all), `sensors` (array), `sources` (array), `splay_percent` (0-25, default 10), `snapshot` (boolean, default false -- if true emit full results, not differential), `removed` (boolean, default true -- include removed rows in differential), `enabled` (boolean, default true), `dry_run` (boolean, default true -- validates and shows what would be created without activating; the analyst must set `dry_run: false` to actually persist and activate the schedule)
- The `schedule.write` capability is allowed for the invoking context. For schedules targeting multiple clients, `schedule.write` must be enabled for ALL targeted clients. For `clients: null`, `schedule.write` must be enabled for at least one client (same as hidden tools visibility rule). Schedule creation fails with `E-FLAG-001` if any targeted client lacks the capability.
- The `query` string is validated (parsed and security-limit-checked) at creation time using the same path as `explain_query` (BC-2.11.010). Invalid queries are rejected with `E-QUERY-001` before the schedule is created. This ensures that scheduling a malformed query does not produce repeated parse errors at execution time.

## Postconditions
- Schedule creation is classified as a **reversible write** with dry-run default (BC-2.04.008). When `dry_run: true` (the default), the tool validates all inputs, computes splay offsets, and returns a preview of the schedule configuration without persisting or activating it. The agent must explicitly set `dry_run: false` to activate.
- A new `ScheduledQuery` record is persisted to the RocksDB `schedules` domain (BC-2.12.010)
- The `splayed_interval` is computed as `interval + (interval * splay_percent / 100) * hash(client_id, name)` per client, spreading API load
- The splay offset per (name, client_id) pair is persisted for deterministic scheduling across restarts
- `next_run` is computed as `now + splayed_interval` for each targeted client
- `epoch` counter is initialized to 0
- The schedule is registered with the execution loop (BC-2.12.004) and begins ticking on the next loop iteration
- An audit entry is emitted for the tool invocation (DI-004)
- Response includes: `schedule_id`, `name`, `interval`, `splay_percent`, `splayed_intervals` (map of client_id to actual interval), `next_run` (map of client_id to next execution time), `enabled`
- The `create_schedule` tool is gated by `schedule.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- DI-008: Client data separation -- schedule scoping respects client boundaries
- DI-019: Query security limits enforced on the PrismQL query string at creation time
- DI-028: Schedule capacity cap enforced -- before persisting a new schedule, the scheduler checks the count of active (non-deleted) schedules against `max_schedules` (default 500, configurable via `[defaults.limits].max_schedules`). If the count is at or above the cap, the schedule is rejected with `E-SCHED-008` and is never persisted or registered with the execution loop.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SCHED-008` | Active schedule count is at or above `max_schedules` cap (default 500) | Structured error with `current_count`, `max_count`, and suggestion to delete unused schedules; schedule is never persisted (DI-028) |
| `E-SCHED-003` | Schedule `name` already exists | Structured error with existing schedule details; use `delete_schedule` + `create_schedule` to replace |
| `E-MCP-004` | `interval` < 60 or > 86400 | Structured error with valid range |
| `E-MCP-004` | `splay_percent` > 25 | Structured error; splay capped at 25% to prevent excessive drift |
| `E-QUERY-001` | PrismQL query string cannot be parsed | Structured error with position and suggestion |
| `E-FLAG-001` | `schedule.write` capability denied | Structured error (BC-2.04.015) |
| `E-MCP-004` | `name` contains invalid characters | Structured error with allowed character set |

## TOML Configuration Example

```toml
[clients.acme.capabilities]
"schedule.write" = "allow"    # enables create_schedule and delete_schedule for client acme
```

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-001 | `clients: null` with 50+ configured clients | Schedule created for all clients; splay spreads executions across the interval to avoid thundering herd |
| EC-12-002 | Query references a sensor not available for some targeted clients | Schedule created; unavailable sensors produce `sensor_errors` at execution time, not at creation time |
| EC-12-003 | `interval: 60` with `splay_percent: 25` | Splayed intervals range from 60-75 seconds depending on client_id hash |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `create_schedule(name="hourly_alerts", query="severity='critical'", interval=3600, dry_run=false)` | Schedule persisted; splay offsets computed; registered with execution loop | happy-path |
| `create_schedule(...)` with `dry_run=true` (default) | Preview returned; nothing persisted | happy-path |
| `create_schedule(...)` when active schedule count == 500 | `Err(E-SCHED-008)` | error |
| `create_schedule(name="hourly_alerts", ...)` when name already exists | `Err(E-SCHED-003)` | error |
| `create_schedule(interval=59, ...)` | `Err(E-MCP-004)` interval below minimum | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-030 | Schedule/rule count caps: rejects beyond limits | kani |
| VP-026 | Splay computation: deterministic per (query, client) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008, DI-019, DI-022, DI-028 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref. |
| 1.1 | deferred-cleanup-track-1 | 2026-04-19 | product-owner | Added DI-028 cap-check invariant, E-SCHED-008 error case |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
