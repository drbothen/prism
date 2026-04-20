---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: ["CAP-004"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.03.002: OS Keyring Backend via keyring-rs

## Description

The `KeyringBackend` stores credentials in the OS keyring (macOS Keychain, Windows Credential Vault, Linux libsecret) using the namespace `prism/{client_id}/{sensor_id}/{credential_name}` as the service/account key. Credentials are encrypted at rest by the OS keyring (hardware-backed where available). A startup probe triggers any OS permission prompts (e.g., macOS Keychain authorization dialog) at process start, consolidating them to a single interaction rather than allowing mid-session prompts.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.002-001 | Store and retrieve credential via macOS Keychain | Namespace `prism/acme/crowdstrike/client_secret`; value retrieved correctly |
| TV-BC-2.03.002-002 | macOS Keychain locked at startup (DEC-011) | Fail fast with `PrismError::Credential` and unlock suggestion; no blocking |
| TV-BC-2.03.002-003 | Linux D-Bus unavailable | `PrismError::Credential` with libsecret/D-Bus install suggestion |
| TV-BC-2.03.002-004 | Keyring entry from old namespace format | Not found; treated as missing credential |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
