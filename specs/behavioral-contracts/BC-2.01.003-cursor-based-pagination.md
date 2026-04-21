---
document_type: behavioral-contract
level: L3
version: "2.5"
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
input-hash: "3eb97f3"
traces_to: []
extracted_from: "[tombstone]"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement:
  - BC-2.11.001
  - BC-2.07.001
  - BC-2.07.002
retired: null
removed: cycle-1
removal_reason: "MCP-exposed cursor pagination eliminated; internal pagination now handled by BC-2.07.001/BC-2.07.002"
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

## Preconditions

_Tombstone — this contract is removed. No preconditions apply. See BC-2.11.001._

## Postconditions

_Tombstone — this contract is removed. No postconditions apply. See BC-2.11.001._

## Invariants

_Tombstone — this contract is removed. No invariants apply._

## Edge Cases

_Tombstone — this contract is removed. No edge cases apply._

## Canonical Test Vectors

_Tombstone — no test vectors. See BC-2.11.001, BC-2.07.001, BC-2.07.002 for replacement test vectors._

## Verification Properties

_Tombstone — no verification properties. See BC-2.11.001._

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Replaced by | BC-2.11.001, BC-2.07.001, BC-2.07.002 |
| Removal cycle | cycle-1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.5 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 2.3 | pass-65-fix | 2026-04-20 | product-owner | Changed replacement: frontmatter from null to YAML list matching body-declared replacements (schema consistency with single-BC retired convention). |
| 2.2 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 BC scope extension). |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added required stub sections for tombstone compliance. |
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Tombstone: MCP-exposed cursor pagination eliminated; internal pagination handled by BC-2.07.001/BC-2.07.002. |
