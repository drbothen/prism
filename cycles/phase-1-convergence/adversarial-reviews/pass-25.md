---
document_type: adversarial-review
pass: 25
status: complete-fixed
novelty: MEDIUM
findings: 5
critical: 0
high: 4
low: 1
---

# Pass 25 — stale references, error category mismatches, BC ID collision

## CRITICAL (0)

## HIGH (4)
- ADV-25-001: FM-003/FM-009 reference removed invariants (DI-009), FileStore, persistent cursors
- ADV-25-002: DEC-032 uses nonexistent E-STORAGE-001 (should be E-STORE-002) and invalid category "infrastructure"
- ADV-25-003: DEC-017 uses invalid error category "rate_limit" (should be "permission")
- ADV-25-004: BC-2.14.010 ID collision — INDEX says acknowledge_alert STUB, file says case_metrics

## LOW (1)
- ADV-25-005: BC-2.10.010 EC-10-020 references "cursor persisted" (stale ephemeral cursor language)
