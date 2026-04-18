---
document_type: adversarial-review
pass: 23
status: complete-fixed
novelty: HIGH
findings: 12
critical: 4
high: 7
low: 1
---

# Pass 23 — systematic cross-referential integrity sweep

## CRITICAL (4)

- ADV-23-001: BC-2.12.001 has 3 SCHED error codes with swapped semantics vs taxonomy
- ADV-23-002: BC-2.12.003 uses E-SCHED-005 (in-flight) for "not found" — should be E-SCHED-001
- ADV-23-003: BC-2.12.005 DIFF error codes have wrong semantics vs taxonomy
- ADV-23-004: get_case interface missing client_id — DI-008 security violation

## HIGH (7)

- ADV-23-005: Case entity missing severity and updated_at fields used by 5+ BCs
- ADV-23-006: Case entity uses alert_ids but all BCs use source_alert_ids
- ADV-23-007: create_pack/delete_pack tools missing from interface definitions
- ADV-23-008: list_cases interface missing 4 parameters (severity, assignee, sort_by, sort_order)
- ADV-23-009: Sequence config field names differ between entity and interface (steps/stages, name/label, condition/predicate)
- ADV-23-010: acknowledge_alert has no behavioral contract
- ADV-23-011: 4 orphan invariants (DI-022, DI-028, DI-029, DI-032) with no enforcing BC

## LOW (1)

- ADV-23-012: E-SPEC-005 lists custom as valid auth_type but entity omits it
