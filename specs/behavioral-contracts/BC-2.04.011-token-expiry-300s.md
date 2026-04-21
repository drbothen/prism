---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-006"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "abc4070"
traces_to: ["CAP-006"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.011: Token Expiry at 300 Seconds with Structured Error Recovery

## Description

Confirmation tokens expire exactly 300 seconds after creation. The expiry check computes
`Utc::now() >= expires_at`; the boundary is strict (>= 300s elapsed means expired). Expired
tokens return a structured error that includes the original `action_summary` so the agent can
re-request intelligently. The error marks the token as non-retryable — the agent must request
a new token from the original write tool.

The 300-second TTL is a security invariant (DI-007) and is not configurable per-client.
Expired tokens are cleaned up lazily on the next token creation request or on an explicit
cleanup sweep.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.011.

| Scenario | Elapsed Time | Expected Result |
|----------|-------------|----------------|
| Token valid | 0s elapsed | Token valid; no expiry error |
| Token valid at boundary minus 1 | 299s elapsed | Token valid |
| Token expired at boundary | exactly 300s elapsed (`now == expires_at`) | `PrismError::TokenExpired`; `retryable: false` |
| Token long expired | 600s elapsed | `PrismError::TokenExpired`; original `action_summary` included in error |

## Verification Properties

- **VP-007** (Confirmation token expiry: expired at boundary inclusive) — Kani proof that boundary condition `Utc::now() >= expires_at` is correctly evaluated (inclusive).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
