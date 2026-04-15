---
document_type: adversarial-review
pass: 3
status: complete
novelty: MEDIUM-HIGH
findings: 13
critical: 2
high: 3
medium: 4
low: 4
---

# Adversarial Review — Pass 3 Findings

## CRITICAL (2)
- ADV-3-001: DEC-014 contradicts DI-004/DI-016 on audit fail-closed for writes
- ADV-3-002: BC-2.01.010 references removed DI-009

## HIGH (3)
- ADV-3-003: NFR-017 cache default 50 vs CAP-014/DI-018 default 1000
- ADV-3-004: BC-2.06.004 still says HashSet instead of BTreeMap
- ADV-3-005: BC-2.07.002 contradicts CAP-011/DI-001 on forward-only pagination scope

## MEDIUM (4)
- ADV-3-006: Error codes inconsistent (E-FLAG-xxx vs ACTION_MISMATCH vs TOKEN_EXPIRED)
- ADV-3-007: No pagination token TTL defined
- ADV-3-008: BC-2.08.005 health check missing cross-client postconditions
- ADV-3-009: Credential tool names inconsistent (store_ vs set_)

## LOW (4)
- ADV-3-010: set_credential creates without confirmation — asymmetry not documented
- ADV-3-011: --state-dir CLI flag references removed cursor state files
- ADV-3-012: Exit code 3 references removed fingerprint mismatch
- ADV-3-013: confirm_action tool has no interface definition
