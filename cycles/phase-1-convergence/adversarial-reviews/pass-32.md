---
document_type: adversarial-review
pass: 32
status: complete-fixed
novelty: MEDIUM
findings: 2
critical: 0
high: 2
low: 0
---

# Pass 32 — entities.md blind spot: __global__ and context switch stale text

## CRITICAL (none)
## HIGH (2) — both fixed inline
- ADV-32-001: ConfirmationToken entity __global__ still said "alias operations only" — fixed to "aliases, schedules, packs, global-scope rules"
- ADV-32-002: ClientCapability entity referenced "client context switch" — fixed to "config reload"
## LOW (none)
