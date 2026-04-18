---
document_type: behavioral-contract
level: L3
version: "2.0"
status: removed
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Adapters"
capability: "CAP-001"
---

# BC-2.01.012: REMOVED -- Query Fingerprint Validation at Startup

**This behavioral contract has been removed.** Query fingerprint validation was part of the persistent cursor state model. With ephemeral in-memory pagination, there are no stored cursor state files and no fingerprints to validate at startup.

- Each query starts fresh with the current configuration
- Configuration changes take effect immediately on the next query invocation
- No startup-time fingerprint validation is needed
- Addresses: ADV-1-002, ADV-2-005

**Replacement:** No direct replacement needed. See BC-2.07.005 (removed) and BC-2.07.006 (removed).
