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
capability: "CAP-012"
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

Tombstone — replaced by BC-2.11.001 (`query` tool), BC-2.11.005 (ephemeral materialization), and BC-2.11.012 (virtual fields). See Related BCs for redirect.

# BC-2.01.011: ~~Cross-Sensor Correlation via OCSF Field Alignment~~

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Cross-sensor correlation IS the query engine. The query engine materializes OCSF-normalized data from multiple sensors into a unified Arrow table and executes PrismQL queries (including JOINs, WHERE clauses on OCSF fields, and cross-sensor aggregations) via DataFusion.

- **OCSF field alignment**: Still handled by subsystem 02 (OCSF Normalization) -- the mappers in BC-2.02.003 through BC-2.02.006 ensure all sensors map to common OCSF fields
- **Correlation queries**: Expressed in PrismQL, e.g., `SELECT * FROM events WHERE device.ip = '10.0.0.1' AND sensor IN ('crowdstrike', 'claroty')`
- **Virtual fields**: BC-2.11.012 adds `sensor`, `client_id`, `source` virtual fields for cross-sensor filtering
- **Materialization**: BC-2.11.005 handles the ephemeral data lake that enables correlation

**Replacement:** BC-2.11.001 (`query` tool), BC-2.11.005 (ephemeral materialization), BC-2.11.012 (virtual fields)
