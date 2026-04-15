---
document_type: behavioral-contract
level: L3
version: "2.0"
status: removed
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.009: REMOVED -- Query Filtering and Sorting Parameters

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Per-tool filter and sort parameters (`severity`, `status`, `time_range`, sort directives) no longer exist as MCP tool inputs. Filtering and sorting are expressed via the AxiQL query language and executed by the query engine (DataFusion).

- **Filter push-down**: BC-2.11.007 handles translating AxiQL predicates to sensor-native filters
- **Post-fetch filtering**: Predicates that cannot be pushed down are applied by DataFusion after materialization
- **Sorting**: Handled by AxiQL `ORDER BY` (SQL mode) or `sort` (pipe mode), executed by DataFusion
- **Query fingerprints**: Eliminated -- no persistent cursor state requiring fingerprint validation

**Replacement:** BC-2.11.002/003/004 (AxiQL parsing), BC-2.11.007 (sensor filter push-down)
