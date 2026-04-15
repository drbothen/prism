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

# BC-2.01.001: REMOVED -- Single-Client Sensor Query Returns Scoped Results

**This behavioral contract has been removed.** Data access is now exclusively through the `query` tool (CAP-015). See BC-2.11.001.

Per-sensor read tools (`get_crowdstrike_alerts`, `get_claroty_devices`, etc.) no longer exist. Single-client data access is achieved via `query(clients: ["acme"], ...)`. Client scoping, result shaping, and pagination are handled by the query engine (subsystem 11).

- **Client scoping**: `query` tool's `clients` array parameter replaces the per-tool `client_id` parameter for reads
- **Sensor selection**: `query` tool's `sensors` array parameter replaces per-sensor tool names
- **Result shaping**: PrismQL query language replaces per-tool filter/sort parameters
- **Pagination**: Query engine uses `limit` + `total_available` instead of cursor-based pagination exposed to the agent

**Replacement:** BC-2.11.001 (`query` MCP tool), BC-2.11.011 (cross-client query scoping)
