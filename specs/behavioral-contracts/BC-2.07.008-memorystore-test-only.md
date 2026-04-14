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

# BC-2.07.008: REMOVED -- MemoryStore Test-Only Restriction

**This behavioral contract has been removed.** The MemoryStore/FileStore distinction was part of the persistent cursor state model. With ephemeral in-memory pagination, all pagination state is in-memory by design -- there is no FileStore for cursors and no need for a separate test-only MemoryStore.

- All pagination state is in-memory in both production and test
- Addresses: ADV-1-002, ADV-2-005

**Replacement:** BC-2.07.001 v2.0 (Ephemeral Pagination Token Structure)
