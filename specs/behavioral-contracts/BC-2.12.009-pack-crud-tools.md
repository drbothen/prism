---
document_type: behavioral-contract
level: L3
version: "1.3"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "dc078d2"
traces_to: ["CAP-023"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.009: Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack`

## Description

Three MCP tools manage pack lifecycle: `create_pack` (gated by `pack.write`, validates all referenced schedules/rules, atomic `packs.toml` write), `list_packs` (read-only, always visible, returns active status per client), and `delete_pack` (gated by `pack.write`, requires confirmation token, atomic removal). Pack deletion does not cascade to referenced schedules or rules — they remain independently. A pack must reference at least one schedule or rule to be created. All tools emit audit entries per DI-004.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `create_pack(name="baseline", query_refs=["hourly_alerts"], client_id="acme")` | Pack persisted to packs.toml; query_count=1 | happy-path |
| `list_packs(client_id="acme")` | Pack summaries with acme-specific active status | happy-path |
| `create_pack(name="empty_pack", query_refs=[], detection_refs=[])` | `Err(E-PACK-002)` — must reference at least 1 item | error |
| `create_pack(name="existing_pack", ...)` | `Err(E-PACK-004)` | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-007 | Confirmation token expiry: expired at boundary (delete_pack) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-023 |
| L2 Invariants | DI-004, DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
