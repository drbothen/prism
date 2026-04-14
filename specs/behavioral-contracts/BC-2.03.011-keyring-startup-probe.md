---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Credential Management"
capability: "CAP-004"
---

# BC-2.03.011: Keyring Startup Probe for Permission Pre-Authorization

## Preconditions
- The `KeyringBackend` is configured as the active credential store
- Prism is starting up

## Postconditions
- A test read is performed against the OS keyring at startup to trigger any permission prompts (macOS Keychain authorization dialog)
- If the probe succeeds, all subsequent keyring operations proceed without user interaction
- If the probe fails, Prism fails fast with an actionable error message
- The probe consolidates permission prompts to process start (no mid-session authorization dialogs)

## Invariants
- Prism does not block indefinitely waiting for keyring unlock

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | macOS Keychain locked (user denied or did not respond to prompt) | Fail fast: "OS keyring is locked. Unlock your keychain and restart Prism, or configure encrypted file backend as fallback." (DEC-011) |
| `PrismError::Credential` | Keyring service not available on this platform | `category: "configuration"`, suggestion: "OS keyring not available. Configure encrypted file backend." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | macOS Keychain requires password entry | The startup probe triggers the dialog once; if user provides password, all subsequent operations succeed without re-prompting |
| EC-03-027 | Keyring available but empty (no Prism credentials stored yet) | Probe succeeds (keyring is accessible); individual credential lookups will fail with "not found" at query time |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
