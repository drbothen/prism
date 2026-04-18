---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Feature Flags"
capability: "CAP-006"
---

# BC-2.04.011: Token Expiry at 300 Seconds with Structured Error Recovery

## Preconditions
- A `ConfirmationToken` has been generated with `created_at` and `expires_at = created_at + 300s`

## Postconditions
- Tokens are valid for exactly 300 seconds from creation
- Validation computes `Utc::now() >= expires_at` to determine expiry
- Expired tokens return a structured error with the original `action_summary` so the agent can re-request intelligently
- The error includes `retryable: false` (the token itself is not retryable; a new token must be requested)
- Expired tokens are cleaned up lazily (on next token creation or on explicit cleanup sweep)

## Invariants
- DI-007: 300-second TTL is fixed and not configurable per-client (security invariant)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::TokenExpired` | `Utc::now() >= token.expires_at` | Returns the original `action_summary` and suggestion to re-request |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-022 | System clock skew (NTP adjustment) makes token appear expired prematurely | Token expiry is based on wall-clock time; clock skew can cause early expiry; no mitigation (clock is trusted) |
| EC-04-023 | Token created at 299s remaining, confirm_action called at 301s | Expired; the 300s boundary is strict |
| EC-04-024 | Prism process restarts between token creation and confirmation | All tokens lost (in-memory store); agent must re-request; this is acceptable for the stdio per-analyst model |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Priority | P1 |
