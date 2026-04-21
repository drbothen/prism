---
document_type: behavioral-contract
level: L3
version: "1.4"
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
input-hash: "7948920"
traces_to: ["CAP-006"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.009: Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap)

## Description

Irreversible write tools (e.g., `contain_host`, `quarantine_file`, `configure_credential_source`)
do not execute their write operation on the first call. Instead they return a `ConfirmationToken`
containing cryptographic random `token_id`, `client_id`, `tool_name`, `action_params`,
`action_summary`, `action_hash` (SHA-256), and `expires_at` (created_at + 300s). The agent
must call `confirm_action(token_id)` to execute. Tokens are held in an in-memory store
(not persisted) with a hard cap of 100 active tokens.

On each token creation request, expired tokens are swept from the store before checking the
cap. If the cap is still reached after cleanup, creation fails with `E-FLAG-007`. This prevents
unbounded token accumulation across long-running sessions.

## Preconditions
- An irreversible write tool is invoked (e.g., `contain_host`, `quarantine_file`)
- The tool is enabled (both compile-time and runtime feature flags pass)
- The caller's parameters are valid

## Postconditions
- The tool does NOT execute the write operation
- Instead, it returns a `ConfirmationToken` containing:
  - `token_id`: cryptographic random string
  - `client_id`: the `TenantId` of the client this action targets (prevents cross-client token replay)
  - `tool_name`: the originating write tool name (e.g., `crowdstrike_contain_host`, `configure_credential_source`)
  - `action_params`: the original tool parameters as `serde_json::Value` (enables `confirm_action` to re-dispatch the write without the caller re-supplying params)
  - `action_summary`: human-readable description (e.g., "Isolate host abc (10.0.1.5) from network for client acme-corp")
  - `action_hash`: SHA-256 of the action parameters (client_id, tool, params)
  - `expires_at`: `created_at + 300s` (5 minutes)
- The token is stored in Prism's in-memory token store (not persisted to disk)
- The response clearly instructs the agent: "Call `confirm_action(token)` to execute, or let the token expire to cancel"
- Hard cap: the token store enforces a maximum of 100 active (non-expired) tokens. If the cap is reached, token creation fails with `E-FLAG-007`.
- Proactive cleanup: on each token creation request, expired tokens are swept from the store before checking the cap.

## Invariants
- DI-007: Token is valid for exactly 300 seconds
- The action is NOT executed during this step
- Token store never exceeds 100 active tokens

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Action parameters fail validation | Structured error before token generation; no token created |
| `PrismError::Sensor` | Cannot verify the target entity exists (e.g., host ID not found) | Structured error; no token created; prevents generating tokens for nonexistent targets |
| `PrismError::Flag` | Token cap reached (100 active tokens after cleanup) | Structured error: `code: "E-FLAG-007"`, `message: "Token store capacity reached (100 active tokens)"`, `suggestion: "Wait for existing tokens to expire or confirm/cancel pending actions before requesting new ones"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-018 | Agent requests a token for the same action twice | Two independent tokens are created; both are valid until consumed or expired (subject to cap) |
| EC-04-019 | Token store at 99 active tokens, new request arrives | Expired tokens cleaned up first; if cleanup frees space, token is created; if still at 100 after cleanup, `E-FLAG-007` |
| EC-04-034 | Server restart loses all in-memory tokens | All pending confirmations are lost; agent must re-request tokens. This is acceptable: tokens are short-lived (5 min) and the agent can retry. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.009.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Normal token generation | Valid params, store empty | `ConfirmationToken` returned; write NOT executed; `expires_at = created_at + 300s` |
| Cap reached | 100 active tokens, no expired | `E-FLAG-007` error; no token created |
| Cap reached, expired exist | 100 tokens but some expired | Expired swept; token created if slot freed |
| Invalid params | Host ID not found | Structured error before token creation; no token |

## Verification Properties

- **VP-007** (Confirmation token expiry: expired at boundary inclusive) — verifies the 300s boundary condition.
- **VP-008** (Confirmation token: single-use enforcement) — verifies tokens cannot be consumed twice.
- **VP-010** (Token cap: store rejects at 100 active tokens) — verifies the hard cap enforcement.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 |
| L2 Invariants | DI-007, DI-015 |
| Addresses | ADV-1-003, ADV-2-002, ADV-5-002, ADV-5-008 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.2 | Burst 43 | 2026-04-19 | product-owner | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` in `tool_name` example in postconditions |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
