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

# BC-2.03.002: OS Keyring Backend via keyring-rs

## Preconditions
- The OS keyring service is available (macOS Keychain, Windows Credential Vault, or Linux libsecret)
- The `KeyringBackend` is selected as the active credential store

## Postconditions
- Credentials are stored in the OS keyring using the namespace `prism/{client_id}/{sensor_id}/{credential_name}` as the service/account key
- Credential values are encrypted by the OS keyring (hardware-backed where available)
- The keyring is probed at startup to consolidate OS permission prompts (macOS Keychain authorization dialog)

## Invariants
- DI-002: Credential isolation -- keyring entries are namespaced by `client_id`

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | macOS Keychain is locked and requires user password | `category: "configuration"`, suggestion: "OS keyring is locked. Unlock your keychain and restart Prism, or configure encrypted file backend as fallback." (DEC-011) |
| `PrismError::Credential` | Linux libsecret not installed or D-Bus unavailable | `category: "configuration"`, suggestion: "Install libsecret and ensure D-Bus is running, or configure encrypted file backend" |
| `PrismError::Credential` | Keyring entry corrupted or unreadable | `category: "data"`, suggestion: "Delete and re-create the credential for client '{client}' sensor '{sensor}'" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | macOS Keychain requires password at startup | Fail fast with actionable error; Prism does not block waiting for keyring unlock |
| EC-03-004 | Keyring entry exists from a previous Prism version with different namespace format | Not found by current namespace; treated as missing credential |
| EC-03-005 | Keyring service becomes unavailable mid-session (e.g., user locks screen) | Subsequent credential operations fail with `PrismError::Credential`; previously loaded credentials remain in memory |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
