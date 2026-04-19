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

Tombstone — replaced by BC-2.11.002/003/004 (PrismQL parsing) and BC-2.11.007 (sensor filter push-down). See Related BCs for redirect.

# BC-2.01.009: ~~Query Filtering and Sorting Parameters~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Per-tool filter and sort parameters (`severity`, `status`, `time_range`, sort directives) no longer exist as MCP tool inputs. Filtering and sorting are expressed via the PrismQL query language and executed by the query engine (DataFusion).

- **Filter push-down**: BC-2.11.007 handles translating PrismQL predicates to sensor-native filters
- **Post-fetch filtering**: Predicates that cannot be pushed down are applied by DataFusion after materialization
- **Sorting**: Handled by PrismQL `ORDER BY` (SQL mode) or `sort` (pipe mode), executed by DataFusion
- **Query fingerprints**: Eliminated -- no persistent cursor state requiring fingerprint validation

**Replacement:** BC-2.11.002/003/004 (PrismQL parsing), BC-2.11.007 (sensor filter push-down)
