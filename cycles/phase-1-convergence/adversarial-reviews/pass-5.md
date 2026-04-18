---
document_type: adversarial-review
pass: 5
status: complete
novelty: MEDIUM
findings: 11
critical: 2
high: 2
medium: 4
low: 3
---

# Adversarial Review — Pass 5 Findings

## CRITICAL (2)
- ADV-5-001: Credential CRUD tools bypass feature flag system entirely — delete_credential is a DoS vector
- ADV-5-002: ConfirmationToken entity missing client_id attribute — cross-client replay possible

## HIGH (2)  
- ADV-5-003: E-FLAG-007 retryability contradiction (taxonomy: No, edge case: Yes)
- ADV-5-004: BC-2.07.004 cache invalidation BC file missing (old persistence-after-delivery file still exists)

## MEDIUM (4)
- ADV-5-005: Cross-client fan-out generates cursors that could exceed 200-cursor cap
- ADV-5-006: query_hash composition unspecified (cursor/page_size/force_refresh included?)
- ADV-5-007: DEC-008 still uses per-field safety flags, contradicts centralized _meta pattern
- ADV-5-008: confirm_action execution mechanism unspecified — token doesn't store action params

## LOW (3)
- ADV-5-009: Removed BC files still exist on disk with stale content
- ADV-5-010: TOML true/false → Allow/Deny mapping undocumented
- ADV-5-011: BC-2.10.003 references removed BC-2.10.005
