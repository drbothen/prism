---
document_type: adversarial-review
pass: 2
status: complete
novelty: HIGH
findings: 13
critical: 2
high: 4
medium: 5
low: 2
---

# Adversarial Review — Pass 2 Findings

## CRITICAL (2)

**ADV-2-001: Feature flag hierarchy cannot deny specific while allowing parent.** HashSet only has enabled paths. No deny mechanism. Parent grant = blanket grant for all children. Breaks principle of least privilege.

**ADV-2-002: Confirmation token store unbounded memory.** No rate limit on token creation. Lazy cleanup only. 300s window allows thousands of tokens. Memory exhaustion vector.

## HIGH (4)

**ADV-2-003:** Client context switch mechanism entirely unspecified (confirms ADV-1-001).
**ADV-2-004:** Cross-client `client_id: null` tool list in hidden tools pattern is contradictory — union vs intersection semantics undefined.
**ADV-2-005:** Cursor state designed for polling but Prism is interactive request/response (confirms ADV-1-006).
**ADV-2-006:** BC-2.02.008 title says "Three-Tier" but body and entities say "Four-Tier" field resolution.

## MEDIUM (5)

**ADV-2-007:** AES-256-GCM no KDF specified, no key rotation procedure (confirms ADV-1-011).
**ADV-2-008:** Prompt injection regex patterns trivially bypassable via Unicode homoglyphs. Not configurable.
**ADV-2-009:** Audit failure non-blocking contradicts SOC 2 compliance claim for write operations.
**ADV-2-010:** Concurrent MCP tool invocations — no synchronization specified for cursor or token store.
**ADV-2-011:** Credential CRUD not gated behind feature flags — prompt injection could delete credentials.

## LOW (2)

**ADV-2-012:** NFR-008 wording omits delivery step in ordering description.
**ADV-2-013:** 7 launch-day sensor data sources have no OCSF mapping (fall through to Base Event).
