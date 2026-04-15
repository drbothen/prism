---
document_type: adversarial-review
pass: 31
status: complete-fixed
novelty: MEDIUM
findings: 2
critical: 0
high: 2
low: 0
---

# Pass 31 — __global__ sentinel scope restriction, startup-only notification

## CRITICAL (none)

## HIGH (2)
- ADV-31-001: __global__ sentinel described as "alias operations only" in 3 BCs but used by delete_schedule/pack/rule
- ADV-31-002: BC-2.10.003 says notifications/tools/list_changed is startup-only, contradicting 4 reload BCs

## LOW (none)
