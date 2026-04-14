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

# BC-2.07.009: REMOVED -- FileStore Default Implementation

**This behavioral contract has been removed.** The FileStore was the production cursor persistence backend. With ephemeral in-memory pagination, there is no file-based cursor store.

- No FileStore for pagination state
- State directory (`--state-dir`) is no longer used for cursor files (may still be used for other purposes such as cache persistence in future)
- Addresses: ADV-1-002, ADV-2-005

**Replacement:** BC-2.07.001 v2.0 (Ephemeral Pagination Token Structure), BC-2.07.007 v2.0 (In-Memory State Isolation)
