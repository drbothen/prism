---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "572c2a9"
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

# BC-2.03.011: Keyring Startup Probe for Permission Pre-Authorization

## Description

When the `KeyringBackend` is configured, Prism performs a test read against the OS keyring at startup to trigger and consolidate any OS permission prompts (e.g., macOS Keychain authorization dialog) before beginning normal operation. If the probe succeeds, all subsequent keyring operations proceed without user interaction. If the probe fails (keyring locked, not available), Prism fails fast with an actionable error message. Prism does not block indefinitely waiting for keyring unlock.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.011-001 | macOS Keychain available and unlocked | Probe succeeds; startup continues; no mid-session prompts |
| TV-BC-2.03.011-002 | macOS Keychain locked at startup (DEC-011) | Fail fast with actionable error; no blocking |
| TV-BC-2.03.011-003 | Keyring available but no Prism credentials stored | Probe succeeds (keyring accessible); query-time lookups will return "not found" |
| TV-BC-2.03.011-004 | Keyring service unavailable (Linux, no D-Bus) | `PrismError::Credential` with encrypted file backend suggestion |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
