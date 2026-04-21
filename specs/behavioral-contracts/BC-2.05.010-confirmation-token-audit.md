---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
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
input-hash: "b1e4604"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.010: Confirmation Token Lifecycle Events Are Audit-Logged

## Description

Every event in the confirmation token lifecycle — issuance, successful consumption, and
all rejection paths (expired, already consumed, not found, hash mismatch) — produces an
audit entry with a distinct `result_summary` value. Token IDs are intentionally excluded
from issuance audit entries to prevent correlation by log readers. All token lifecycle audit
entries include `client_id`, `sensor`, and `tool_name` of the original write operation,
enabling forensic reconstruction of the full two-step write flow from the audit trail.

## Preconditions
- An irreversible write operation produces a `ConfirmationToken` (issuance) or a `confirm_action` call consumes/rejects a token

## Postconditions
- **Token issuance**: audit entry records `result_summary: "confirmation_token_issued"` with the `action_summary` and token expiry time, but NOT the token ID
- **Token consumption (success)**: audit entry for `confirm_action` records `result_summary: "confirmed_and_executed"` with the action outcome
- **Token rejection (expired)**: audit entry records `result_summary: "token_expired"` with the original action summary
- **Token rejection (consumed)**: audit entry records `result_summary: "token_already_consumed"` with the original action summary
- All token lifecycle events include the `client_id`, `sensor`, and `tool_name` of the original write operation

## Invariants
- DI-007: Confirmation token expiry
- DI-004: Audit completeness

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Invalid token ID | `confirm_action` called with a token ID that does not exist | Audit entry records `result_summary: "token_not_found"` |
| Action hash mismatch | Token's `action_hash` does not match the `confirm_action` parameters | Audit entry records `result_summary: "action_hash_mismatch"` with both hashes for forensic review |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-009 | Token expired at exactly 300 seconds | Token is rejected; audit entry records `result_summary: "token_expired"` |
| EC-05-017 | Token issued but never consumed (analyst abandons the flow) | Only the issuance audit entry exists; no consumption entry; the token expires silently with no additional audit event |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.010.

| Scenario | `result_summary` | Token ID in Entry? |
|----------|-----------------|-------------------|
| Token issued | `"confirmation_token_issued"` + `action_summary` + expiry | No |
| Token consumed successfully | `"confirmed_and_executed"` + action outcome | Yes (in sub-fields) |
| Token expired | `"token_expired"` + original `action_summary` | No |
| Token already consumed | `"token_already_consumed"` + original `action_summary` | No |
| Token not found | `"token_not_found"` | No |
| Hash mismatch | `"action_hash_mismatch"` + both hashes | No |

## Verification Properties

- **VP-008** (Confirmation token: single-use enforcement) — verifies that double-consumption produces a distinct audit event (`token_already_consumed`).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004, DI-007 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
