---
document_type: adversarial-review
pass: 6
status: complete
novelty: HIGH
findings: 11
critical: 3
high: 5
medium: 0
low: 3
---

# Adversarial Review — Pass 6 Findings

## CRITICAL (3)
- ADV-6-001: Subsystem 07 BC filenames not renamed — PRD links all broken (6 files)
- ADV-6-002: BC-2.07.005 and BC-2.07.006 still contain removal stubs, no cache BC content
- ADV-6-003: PRD references CACHE error category but taxonomy uses STATE — no cache error codes

## HIGH (5)
- ADV-6-004: credential.write capability path not in TOML config example
- ADV-6-005: source_id mapping for cache invalidation unspecified (write tools don't take source_id)
- ADV-6-006: Cross-client pagination output schema has single cursor, not per-client cursors map
- ADV-6-007: confirm_action re-dispatch mechanism unspecified (bypass middleware? double audit?)
- ADV-6-008: Cursor cap + token cap combined pressure not addressed

## LOW (3)
- ADV-6-009: Cache BCs are P0 but CAP-014 is P1 — priority mismatch
- ADV-6-010: confirm_action produces 1 or 2 audit entries? DI-004 says exactly 1
- ADV-6-011: DEC-020 not in L2-INDEX cross-references
