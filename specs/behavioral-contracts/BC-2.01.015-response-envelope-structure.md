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
input-hash: "572c2a9"
traces_to: []
extracted_from: "[tombstone]"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement:
  - BC-2.11.001
  - BC-2.09.008
retired: null
removed: cycle-1
removal_reason: "Per-sensor tool response envelope replaced by query engine response format defined in BC-2.11.001 and interface-definitions.md"
---

## Description

Tombstone — replaced by BC-2.11.001 (`query` tool response format) and BC-2.09.008 (trust annotations). See Related BCs for redirect.

# BC-2.01.015: ~~MCP Tool Response Envelope Structure~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

The per-sensor tool response envelope (with `_meta`, `results`, `content_summary`, cursor-based pagination fields) is replaced by the query engine's response format. The query engine defines its own response structure with `query_context`, `events`, `_meta`, and `sensor_errors`.

- **Query response format**: Defined in the `query` tool's `outputSchema` (see interface-definitions.md section 1.9)
- **Trust annotations**: Still present via `_meta.trust_level` and `_meta.safety_flags` (BC-2.09.008)
- **Prompt injection defense**: Still enforced -- sensor data in `events` array, never interpolated into prose (BC-2.09.001)
- **Write tool envelopes**: Write tools retain their own response envelopes (confirmation tokens, execution results)

**Replacement:** BC-2.11.001 (`query` tool response format), BC-2.09.008 (trust annotations)

## Preconditions

_Tombstone — this contract is removed. No preconditions apply. See BC-2.11.001._

## Postconditions

_Tombstone — this contract is removed. No postconditions apply. See BC-2.11.001._

## Invariants

_Tombstone — this contract is removed. No invariants apply._

## Edge Cases

_Tombstone — this contract is removed. No edge cases apply._

## Canonical Test Vectors

_Tombstone — no test vectors. See BC-2.11.001 and BC-2.09.008 for replacement test vectors._

## Verification Properties

_Tombstone — no verification properties. See BC-2.11.001, BC-2.09.008._

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Replaced by | BC-2.11.001, BC-2.09.008 |
| Removal cycle | cycle-1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.5 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 2.3 | pass-65-fix | 2026-04-20 | product-owner | Changed replacement: frontmatter from null to YAML list matching body-declared replacements (schema consistency with single-BC retired convention). |
| 2.2 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-001 BC scope extension). |
| 2.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added required stub sections for tombstone compliance. |
| 2.0 | cycle-1 | 2026-04-14 | product-owner | Tombstone: per-sensor tool response envelope eliminated; query engine response format defined in BC-2.11.001. |
