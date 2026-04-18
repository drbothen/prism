---
document_type: adversarial-review
pass: 1
status: complete
novelty: HIGH
findings: 13
critical: 3
high: 5
medium: 4
low: 1
---

# Adversarial Review — Pass 1 Findings

## CRITICAL (3)

**ADV-1-001: Client context switch mechanism undefined.** Specs conflate per-call client_id scoping with session-level "active client" that controls tool visibility. No `set_client_context` tool exists. Hidden tools pattern is unimplementable as specified.

**ADV-1-002: Cursor persistence model mismatch.** NFR-008 says persist BEFORE in-memory update. BC-2.07.004 says persist AFTER delivery. Cursor state management is over-engineered for interactive MCP (designed for background poller). No consumer-side deduplication mechanism.

**ADV-1-003: Confirmation tokens in-memory only.** Server restart between token generation and confirm_action loses all tokens. No guidance for agent on recovery.

## HIGH (5)

**ADV-1-004:** `check_sensor_health` doesn't accept `client_id: null` for cross-client health overview.
**ADV-1-005:** No interface schemas for credential CRUD or write operation tools.
**ADV-1-006:** Poller-style persistent cursor vs interactive pagination cursor — two different concepts conflated.
**ADV-1-007:** Write operation idempotency not specified — token consumed before execution, retry creates duplicates.
**ADV-1-008:** Safety flag delivery mechanism ambiguous — per-field parallel fields vs _meta.safety_flags array.

## MEDIUM (4)

**ADV-1-009:** Two analysts sharing same config/state directory not addressed.
**ADV-1-010:** NFR-001 10s budget doesn't account for CrowdStrike two-step fetch (2+ HTTP calls).
**ADV-1-011:** Encrypted file backend: no KDF specified, salt/nonce terminology confused, nonce reuse risk.
**ADV-1-012:** No error code for write operation with `client_id: null`.

## LOW (1)

**ADV-1-013:** Token errors miscategorized under MCP instead of FLAG.
