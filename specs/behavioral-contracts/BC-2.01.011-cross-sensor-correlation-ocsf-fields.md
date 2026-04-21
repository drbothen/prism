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
capability: "CAP-012"
lifecycle_status: removed
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "81c997e"
traces_to: []
extracted_from: "[tombstone]"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement:
  - BC-2.11.001
  - BC-2.11.005
  - BC-2.11.012
retired: null
removed: cycle-1
removal_reason: "Cross-sensor correlation is now the query engine itself; BC-2.11.001/2.11.005/2.11.012 cover this behavior"
---

## Description

Tombstone — replaced by BC-2.11.001 (`query` tool), BC-2.11.005 (ephemeral materialization), and BC-2.11.012 (virtual fields). See Related BCs for redirect.

# BC-2.01.011: ~~Cross-Sensor Correlation via OCSF Field Alignment~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Cross-sensor correlation IS the query engine. The query engine materializes OCSF-normalized data from multiple sensors into a unified Arrow table and executes PrismQL queries (including JOINs, WHERE clauses on OCSF fields, and cross-sensor aggregations) via DataFusion.

- **OCSF field alignment**: Still handled by subsystem 02 (OCSF Normalization) -- the mappers in BC-2.02.003 through BC-2.02.006 ensure all sensors map to common OCSF fields
- **Correlation queries**: Expressed in PrismQL, e.g., `SELECT * FROM events WHERE device.ip = '10.0.0.1' AND sensor IN ('crowdstrike', 'claroty')`
- **Virtual fields**: BC-2.11.012 adds `sensor`, `client_id`, `source` virtual fields for cross-sensor filtering
- **Materialization**: BC-2.11.005 handles the ephemeral data lake that enables correlation

**Replacement:** BC-2.11.001 (`query` tool), BC-2.11.005 (ephemeral materialization), BC-2.11.012 (virtual fields)

## Preconditions

_Tombstone — this contract is removed. No preconditions apply. See BC-2.11.001, BC-2.11.005._

## Postconditions

_Tombstone — this contract is removed. No postconditions apply. See BC-2.11.001, BC-2.11.005._

## Invariants

_Tombstone — this contract is removed. No invariants apply._

## Edge Cases

_Tombstone — this contract is removed. No edge cases apply._

## Canonical Test Vectors

_Tombstone — no test vectors. See BC-2.11.001 and BC-2.11.005 for replacement test vectors._

## Verification Properties

_Tombstone — no verification properties. See BC-2.11.001, BC-2.11.005._

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-012 |
| Replaced by | BC-2.11.001, BC-2.11.005, BC-2.11.012 |
| Removal cycle | cycle-1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.5 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 2.3 | pass-65-fix | 2026-04-20 | product-owner | Changed replacement: frontmatter from null to YAML list matching body-declared replacements (schema consistency with single-BC retired convention). |
| 2.2 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 BC scope extension). |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added required stub sections for tombstone compliance. |
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Tombstone: cross-sensor correlation subsumed by query engine materialization model. |
