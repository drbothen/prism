---
document_type: adversarial-review
pass: 26
status: complete-fixed
novelty: MEDIUM
findings: 7
critical: 0
high: 5
low: 2
---

# Pass 26 — final error code semantic sweep

## CRITICAL (0)

## HIGH (5) — all same class: BC error code vs taxonomy semantic mismatch
- ADV-26-001: BC-2.13.005 uses E-ALERT-001/002 for persistence/notification failures (taxonomy: not_found/already_ack)
- ADV-26-002: BC-2.12.007 uses E-SCHED-005 for "not found" (taxonomy: "still in-flight") — should be E-SCHED-001
- ADV-26-003: BC-2.16.002 uses E-SPEC-005 for variable interpolation failure (taxonomy: invalid auth_type)
- ADV-26-004: BC-2.12.006 uses E-SCHED-008 for RocksDB write failure (taxonomy: max schedule count)
- ADV-26-005: BC-2.15.001 uses E-STORE-004 for directory creation failure (taxonomy: column family not found)

## LOW (2)
- ADV-26-006: E-STORE-002 used for read/deserialization in 3 BCs (taxonomy says "write failed")
- ADV-26-007: BC-2.14.012 acknowledge_alert remains STUB (known, tracked)
