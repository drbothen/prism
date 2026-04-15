---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Scheduled Queries & Differential Results"
capability: "CAP-023"
---

# BC-2.12.009: Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack`

## Preconditions
- For `create_pack` and `delete_pack`: the `pack.write` capability is allowed for the invoking context
- For `create_pack`: required parameters are `name` (`[a-z0-9_-]{1,64}`), `queries` (map of query_name to `{query, interval}` objects); optional: `description`, `discovery`, `sensor_filter`, `shard`
- For `list_packs`: no required parameters; optional `client_id` to filter by shard eligibility
- For `delete_pack`: required parameter `name`

## Postconditions

### `create_pack`
- All queries in the pack pass AxiQL parsing and security limit validation
- The pack definition is persisted to `packs.toml` via atomic write (temp + fsync + rename)
- All pack queries are registered as scheduled queries with the execution loop
- Response includes: `pack_name`, `query_count`, `active` (based on discovery query if present)
- Gated by `pack.write` capability; follows hidden-tools pattern (BC-2.04.005)

### `list_packs`
- Returns array of pack summaries: `name`, `description`, `query_count`, `active`, `discovery` (query string or null), `shard`, `sensor_filter`, `queries` (array of query names with intervals)
- If `client_id` is provided, `active` reflects both discovery result and shard eligibility for that client
- Read-only; always visible in `tools/list`

### `delete_pack`
- Requires confirmation token (irreversible write, BC-2.04.009)
- Pack definition is removed from `packs.toml` via atomic write
- All pack queries are deregistered from the execution loop
- Differential results for pack queries are retained (orphaned but queryable)
- Gated by `pack.write` capability; follows hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry per tool invocation
- DI-019: All pack queries must pass security limits
- Atomic file writes: `packs.toml` is never left in a partial state

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PACK-004` | Pack `name` already exists (on create) | Structured error; use delete + create to replace |
| `E-PACK-005` | Pack `name` does not exist (on delete) | Structured error |
| `E-CAP-001` | `pack.write` capability denied | Structured error (BC-2.04.015) |
| `E-PACK-002` | Any query in the pack fails AxiQL parsing | Structured error identifying the failing query |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-026 | Create pack with 0 queries | Rejected; a pack must contain at least 1 query |
| EC-12-027 | Delete pack while pack queries are in-flight | In-flight executions complete; pack is then removed |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 |
| L2 Invariants | DI-004, DI-019 |
| Priority | P1 |
