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
input-hash: "[pending-recompute]"
traces_to: ["CAP-006"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.012: Token Content Hash Verification Prevents Action Tampering

## Description

The `action_hash` field in a `ConfirmationToken` is a SHA-256 digest of the sorted,
canonically serialized action parameters (including `client_id`, tool name, and all
action-specific parameters). When `confirm_action` is called, the hash stored in the token
is compared against the hash recomputed from the parameters at consumption time. A mismatch
is rejected, preventing the scenario where an agent creates a token for one action and
attempts to use it for a different action (e.g., "contain host A" → "contain host B").

Hash computation is deterministic by always sorting keys before hashing and using canonical
JSON serialization.

## Preconditions
- A `ConfirmationToken` was created with an `action_hash` (SHA-256 of sorted action parameters)
- `confirm_action` is invoked to execute the operation

## Postconditions
- The `action_hash` stored in the token is compared against the hash that was computed at token creation time
- If the hash does not match (indicating the action parameters were modified), the token is rejected
- This prevents a scenario where an agent could create a token for "contain host A" and then use it to "contain host B"
- The hash includes: `client_id`, tool name, and all action-specific parameters (sorted and serialized to canonical JSON)

## Invariants
- DI-007: The confirmed action must match the original request
- Hash computation is deterministic (sorted keys, canonical JSON serialization)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `action_hash` mismatch | `code: "E-FLAG-005"`, `retryable: false`, suggestion: "The confirmation token does not match the requested action. Request a new token for the intended action." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-025 | Same action parameters produce different hashes due to JSON serialization order | Prevented by sorting keys before hashing; canonical JSON ensures deterministic output |
| EC-04-026 | Token created for `contain_host(host_id: "abc")`, confirmed for `contain_host(host_id: "xyz")` | Hash mismatch; rejected; agent must request a new token for host "xyz" |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.012.

| Scenario | Token Params | Confirm Params | Expected Result |
|----------|-------------|---------------|----------------|
| Match | `{host_id: "abc"}` | Same params, same `client_id` | Hash matches; execution proceeds |
| Host ID tampered | `{host_id: "abc"}` | `{host_id: "xyz"}` | `E-FLAG-005`; execution rejected |
| Key order variation | `{b: 2, a: 1}` | `{a: 1, b: 2}` | Hash matches (sorted keys normalize ordering) |

## Verification Properties

- **VP-009** (Confirmation token: content hash mismatch rejects) — verifies that a hash mismatch causes token rejection without executing the write.

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
