---
document_type: adversarial-review
pass: 4
status: complete
novelty: HIGH
findings: 10
critical: 1
high: 2
medium: 5
low: 2
---

# Adversarial Review — Pass 4 Findings

## CRITICAL (1)
- ADV-4-001: confirm_action has no client_id — breaks audit and capability invariants

## HIGH (2)
- ADV-4-002: Error codes STILL inconsistent — BC-2.04.010 uses TOKEN_EXPIRED/TOKEN_CONSUMED/TOKEN_NOT_FOUND instead of E-FLAG-xxx
- ADV-4-004: user_identity sourcing undefined — how does Prism know which analyst is using it? SOC 2 compliance gap

## MEDIUM (5)
- ADV-4-003: Cache invalidation too broad — all entries for (client_id, sensor_id) wiped on any write
- ADV-4-005: Cross-client query pagination unspecified — P0 capability gap
- ADV-4-006: HKDF no salt — weakens key derivation for low-entropy key material
- ADV-4-007: set_credential create vs update contradiction between brief and interface-definitions
- ADV-4-008: No concurrency limit for cross-client fan-out

## LOW (2)
- ADV-4-009: Cursor TTL 600s vs token TTL 300s — rationale not documented, no cursor cap
- ADV-4-010: E-STATE-002 (cache miss) shouldn't be an error code
