---
document_type: behavioral-contract
level: L3
version: "2.0"
status: removed
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Pagination & Cache"
capability: "CAP-011"
---

# BC-2.07.010: REMOVED -- State File Directory Structure

**This behavioral contract has been removed.** The `{state_dir}/{client_id}/{sensor_id}/{source_id}.json` directory structure was for persistent cursor state files. With ephemeral in-memory pagination, there are no cursor state files on disk.

- No state files for pagination
- In-memory state is keyed by `(client_id, sensor_id, source_id)` tuple (see BC-2.07.007 v2.0)
- Addresses: ADV-1-002, ADV-2-005

**Replacement:** BC-2.07.007 v2.0 (In-Memory Pagination and Cache State Isolated Per-Client, Per-Sensor)
