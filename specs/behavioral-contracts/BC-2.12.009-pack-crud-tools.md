---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-023"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.009: Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack`

## Preconditions
- For `create_pack` and `delete_pack`: the `pack.write` capability is allowed for the invoking context
- For `create_pack`: required parameters are `name` (`[a-z0-9_-]{1,64}`), `client_id`; optional: `query_refs` (array of existing schedule names), `detection_refs` (array of existing rule_ids), `discovery_query`, `description`, `enabled` (default `true`)
- For `list_packs`: no required parameters; optional `client_id` to filter by shard eligibility
- For `delete_pack`: required parameter `pack_id`

## Postconditions

### `create_pack`
- All referenced schedules (`query_refs`) and rules (`detection_refs`) must exist; invalid references produce a structured error
- The pack definition is persisted to `packs.toml` via atomic write (temp + fsync + rename)
- Referenced schedules and rules are associated with the pack
- Response includes: `pack_name`, `query_count`, `detection_count`, `active` (based on `enabled` flag and discovery query if present)
- Gated by `pack.write` capability; follows hidden-tools pattern (BC-2.04.005)

### `list_packs`
- Returns array of pack summaries: `name`, `description`, `query_count`, `detection_count`, `active`, `discovery_query` (string or null), `enabled`, `query_refs` (array of schedule names), `detection_refs` (array of rule_ids)
- If `client_id` is provided, `active` reflects both discovery result and shard eligibility for that client
- Read-only; always visible in `tools/list`

### `delete_pack`
- Requires confirmation token (irreversible write, BC-2.04.009)
- Pack definition is removed from `packs.toml` via atomic write
- All pack schedule and rule associations are removed
- Referenced schedules and rules themselves are not deleted (they remain independently)
- Gated by `pack.write` capability; follows hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry per tool invocation
- DI-019: All referenced schedules and rules must exist and pass validation
- Atomic file writes: `packs.toml` is never left in a partial state

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-PACK-004` | Pack `name` already exists (on create) | Structured error; use delete + create to replace |
| `E-PACK-005` | Pack `name` does not exist (on delete) | Structured error |
| `E-FLAG-001` | `pack.write` capability denied | Structured error (BC-2.04.015) |
| `E-PACK-002` | A referenced schedule or rule does not exist | Structured error identifying the invalid reference |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-026 | Create pack with 0 refs (no query_refs and no detection_refs) | Rejected; a pack must reference at least 1 schedule or rule |
| EC-12-027 | Delete pack while pack queries are in-flight | In-flight executions complete; pack is then removed |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 |
| L2 Invariants | DI-004, DI-019 |
| Priority | P0 |
