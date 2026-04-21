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
input-hash: "ac6b633"
traces_to: []
extracted_from: "[tombstone]"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement:
  - BC-2.11.002
  - BC-2.11.003
  - BC-2.11.004
  - BC-2.11.007
retired: null
removed: cycle-1
removal_reason: "Per-tool filter/sort parameters eliminated; filtering/sorting now expressed via PrismQL and executed by query engine (DataFusion)"
---

## Description

Tombstone — replaced by BC-2.11.002/003/004 (PrismQL parsing) and BC-2.11.007 (sensor filter push-down). See Related BCs for redirect.

# BC-2.01.009: ~~Query Filtering and Sorting Parameters~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Per-tool filter and sort parameters (`severity`, `status`, `time_range`, sort directives) no longer exist as MCP tool inputs. Filtering and sorting are expressed via the PrismQL query language and executed by the query engine (DataFusion).

- **Filter push-down**: BC-2.11.007 handles translating PrismQL predicates to sensor-native filters
- **Post-fetch filtering**: Predicates that cannot be pushed down are applied by DataFusion after materialization
- **Sorting**: Handled by PrismQL `ORDER BY` (SQL mode) or `sort` (pipe mode), executed by DataFusion
- **Query fingerprints**: Eliminated -- no persistent cursor state requiring fingerprint validation

**Replacement:** BC-2.11.002/003/004 (PrismQL parsing), BC-2.11.007 (sensor filter push-down)

## Preconditions

_Tombstone — this contract is removed. No preconditions apply. See BC-2.11.002, BC-2.11.007._

## Postconditions

_Tombstone — this contract is removed. No postconditions apply. See BC-2.11.002, BC-2.11.007._

## Invariants

_Tombstone — this contract is removed. No invariants apply._

## Edge Cases

_Tombstone — this contract is removed. No edge cases apply._

## Canonical Test Vectors

_Tombstone — no test vectors. See BC-2.11.002/003/004 and BC-2.11.007 for replacement test vectors._

## Verification Properties

_Tombstone — no verification properties. See BC-2.11.002, BC-2.11.007._

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Replaced by | BC-2.11.002, BC-2.11.003, BC-2.11.004, BC-2.11.007 |
| Removal cycle | cycle-1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.5 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 2.3 | pass-65-fix | 2026-04-20 | product-owner | Changed replacement: frontmatter from null to YAML list matching body-declared replacements (schema consistency with single-BC retired convention). |
| 2.2 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 BC scope extension). |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added required stub sections for tombstone compliance. |
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Tombstone: per-tool filter/sort parameters eliminated; PrismQL and DataFusion handle all filtering/sorting. |
