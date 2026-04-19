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
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: cycle-1
removal_reason: null
---

## Description

Tombstone — replaced by BC-2.11.001 (`query` tool with `limit`/`total_available`) and BC-2.07.001/BC-2.07.002 (internal pagination). See Related BCs for redirect.

# BC-2.01.003: ~~Cursor-Based Forward-Only Pagination (MCP-Exposed)~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

MCP-exposed cursor-based pagination for per-sensor read tools no longer exists. The query engine handles pagination internally between Prism and sensor APIs. The agent-facing interface uses `limit` and `total_available` (no cursor tokens exposed to the agent).

- Internal sensor API pagination is handled by BC-2.07.001 (ephemeral pagination token structure) and BC-2.07.002 (pagination token lifecycle) as internal adapter mechanics
- The query engine fetches all pages from sensor APIs up to security limits (BC-2.11.006), materializes results, and applies PrismQL query logic before returning
- The agent controls result size via the `limit` parameter on the `query` tool

**Replacement:** BC-2.11.001 (`query` tool with `limit`/`total_available`), BC-2.07.001/BC-2.07.002 (internal pagination)
