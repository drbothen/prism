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

# BC-2.01.015: REMOVED -- MCP Tool Response Envelope Structure

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

The per-sensor tool response envelope (with `_meta`, `results`, `content_summary`, cursor-based pagination fields) is replaced by the query engine's response format. The query engine defines its own response structure with `query_context`, `events`, `_meta`, and `sensor_errors`.

- **Query response format**: Defined in the `query` tool's `outputSchema` (see interface-definitions.md section 1.9)
- **Trust annotations**: Still present via `_meta.trust_level` and `_meta.safety_flags` (BC-2.09.008)
- **Prompt injection defense**: Still enforced -- sensor data in `events` array, never interpolated into prose (BC-2.09.001)
- **Write tool envelopes**: Write tools retain their own response envelopes (confirmation tokens, execution results)

**Replacement:** BC-2.11.001 (`query` tool response format), BC-2.09.008 (trust annotations)
