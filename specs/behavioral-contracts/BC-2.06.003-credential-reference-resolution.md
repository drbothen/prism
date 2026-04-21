---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
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
input-hash: "e5de7f9"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.003: Credential References in Config Resolve to Credential Store Entries

## Description

Sensor configuration entries include a `credential_ref` field whose value is resolved
against the credential store using a three-tier priority chain: `_FILE` env var suffix
(K8s secret mount pattern) > env var > credential store lookup by
`(client_id, sensor_id, credential_ref)`. The resolved credential is a `SecretString`
that is never logged or serialized. The `credential_ref` value is validated against
`[a-zA-Z0-9_\-\.]+` at config load time (DI-014).

Resolution is scoped to `client_id`, satisfying the credential isolation invariant
(DI-002): credentials for client A can never be resolved in the context of client B.

## Preconditions
- A sensor config entry includes a `credential_ref` field
- The credential store backend (keyring or encrypted file) is accessible

## Postconditions
- The `credential_ref` value is resolved against the credential store using the `(client_id, sensor_id, credential_name)` namespace
- Resolution follows the priority chain: `_FILE` env var suffix > env var > credential store
  - `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL_FILE` reads the secret from a file path (K8s secret mount pattern)
  - `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL` reads the secret from the env var value
  - Fallback: credential store lookup by `(client_id, sensor_id, credential_ref)`
- The resolved credential is available as a `SecretString` that is never logged or serialized

## Invariants
- DI-002: Credential isolation per client -- resolution is always scoped by `client_id`
- DI-014: Credential name sanitization -- `credential_ref` is validated against `[a-zA-Z0-9_\-\.]+`

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | Credential not found in any resolution tier | Error: "Credential '{ref}' for clients.{id}.sensors.{sensor} not found. Check credential store or set PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_CREDENTIAL env var." |
| `PrismError::InvalidInput` | `credential_ref` contains characters outside `[a-zA-Z0-9_\-\.]+` | Error: "Invalid credential reference '{ref}': must match [a-zA-Z0-9_\\-\\.]+" |
| `PrismError::Credential` | `_FILE` env var points to a non-existent or unreadable file | Error: "Credential file '{path}' referenced by PRISM_...CREDENTIAL_FILE not found or unreadable" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-011 | OS keyring is locked at startup | `PrismError::Credential` with suggestion to unlock keychain or use encrypted file fallback |
| EC-06-003 | `_FILE` env var is set but file content has trailing newline | File content is trimmed of leading/trailing whitespace before use as credential value |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.003.

| Scenario | Resolution Tier | Expected Result |
|----------|----------------|----------------|
| `_FILE` env var set | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CREDENTIAL_FILE=/run/secrets/cs_key` | File content read (trimmed) as `SecretString` |
| Bare env var set | `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CREDENTIAL=abc123` | Env var value used as `SecretString` |
| Credential store fallback | No env vars; credential in keyring | Keyring lookup succeeds |
| Not found | No env vars; credential absent from store | `PrismError::Credential` with suggestion |
| Invalid ref name | `credential_ref = "my key!"` (space and exclamation) | `PrismError::InvalidInput`: must match pattern |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify credential reference resolution. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-002, DI-014 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
