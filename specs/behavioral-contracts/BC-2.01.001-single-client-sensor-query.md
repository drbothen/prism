---
document_type: behavioral-contract
level: L3
version: "2.0"
status: removed
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: removed
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: []
extracted_from: "[tombstone]"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: cycle-1
removal_reason: "Replaced by BC-2.11.001 and BC-2.11.011; per-sensor read tools eliminated in favor of unified query engine interface"
---

## Description

Tombstone — replaced by BC-2.11.001 (`query` MCP tool) and BC-2.11.011 (cross-client query scoping). See Related BCs for redirect.

# BC-2.01.001: ~~Single-Client Sensor Query Returns Scoped Results~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Per-sensor read tools (`get_crowdstrike_alerts`, `get_claroty_devices`, etc.) no longer exist. Single-client data access is achieved via `query(clients: ["acme"], ...)`. Client scoping, result shaping, and pagination are handled by the query engine (subsystem 11).

- **Client scoping**: `query` tool's `clients` array parameter replaces the per-tool `client_id` parameter for reads
- **Sensor selection**: `query` tool's `sensors` array parameter replaces per-sensor tool names
- **Result shaping**: PrismQL query language replaces per-tool filter/sort parameters
- **Pagination**: Query engine uses `limit` + `total_available` instead of cursor-based pagination exposed to the agent

**Replacement:** BC-2.11.001 (`query` MCP tool), BC-2.11.011 (cross-client query scoping)

## Preconditions

_Tombstone — this contract is removed. No preconditions apply. See BC-2.11.001._

## Postconditions

_Tombstone — this contract is removed. No postconditions apply. See BC-2.11.001._

## Invariants

_Tombstone — this contract is removed. No invariants apply._

## Edge Cases

_Tombstone — this contract is removed. No edge cases apply._

## Canonical Test Vectors

_Tombstone — no test vectors. See BC-2.11.001 for replacement test vectors._

## Verification Properties

_Tombstone — no verification properties. See BC-2.11.001._

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Replaced by | BC-2.11.001, BC-2.11.011 |
| Removal cycle | cycle-1 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Tombstone: contract removed; per-sensor read tools eliminated in favor of query engine. |
| 2.0 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added required stub sections for tombstone compliance. |
