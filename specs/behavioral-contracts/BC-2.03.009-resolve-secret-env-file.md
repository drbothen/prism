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
input-hash: "47125c0"
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

# BC-2.03.009: resolve_secret() for _FILE Env Var and K8s Secret Mount Compatibility

## Description

`resolve_secret(file_env, direct_env)` implements the `_FILE` suffix pattern used by all 4 reference Go pollers for K8s secret mount compatibility. It checks `{NAME}_FILE` first (file path containing the secret), then `{NAME}` (direct value). File contents take precedence over direct env vars. If neither is set, `None` is returned and the caller decides if this is an error. File contents are immediately loaded into a `SecretString` and trailing newlines stripped; the file is not re-read on each access.

## Preconditions
- A credential or encryption key needs to be resolved from the environment
- Two env var names are checked: `{NAME}_FILE` (file path) and `{NAME}` (direct value)

## Postconditions
- `resolve_secret(file_env, direct_env)` checks `{NAME}_FILE` first, then `{NAME}`
- If `{NAME}_FILE` is set, the file at that path is read and its contents used as the secret value (trailing newline stripped)
- If `{NAME}_FILE` is not set but `{NAME}` is set, the env var value is used directly
- If neither is set, returns `None` (caller decides if this is an error)
- File path takes precedence over direct env var (K8s secret mount pattern from all 4 Go pollers)

## Invariants
- File contents are loaded into `SecretString` immediately; file is not re-read on each access

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Credential` | `{NAME}_FILE` is set but the file does not exist | `category: "configuration"`, suggestion: "File '{path}' referenced by {NAME}_FILE does not exist" |
| `PrismError::Credential` | `{NAME}_FILE` is set but the file is not readable (permission denied) | `category: "configuration"`, suggestion: "Check file permissions on '{path}'" |
| `PrismError::Credential` | `{NAME}_FILE` points to a directory, not a file | `category: "configuration"`, suggestion: "{NAME}_FILE must point to a regular file, not a directory" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-022 | Both `{NAME}_FILE` and `{NAME}` are set | File takes precedence; direct env var ignored; debug-level log noting the precedence |
| EC-03-023 | `{NAME}_FILE` points to an empty file | Empty string resolved as the secret; likely causes auth failure downstream |
| EC-03-024 | File contains multiple lines | Only the first line is used (newlines stripped); remaining lines ignored |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.009-001 | `PRISM_KEY_FILE=/run/secrets/key` where file contains "abc\n" | Returns SecretString("abc") (trailing newline stripped) |
| TV-BC-2.03.009-002 | Only `PRISM_KEY=directvalue` set | Returns SecretString("directvalue") |
| TV-BC-2.03.009-003 | Both `PRISM_KEY_FILE` and `PRISM_KEY` set | File wins; debug log notes precedence |
| TV-BC-2.03.009-004 | `PRISM_KEY_FILE` points to nonexistent file | `PrismError::Credential` with file path and existence suggestion |
| TV-BC-2.03.009-005 | Neither env var set | Returns `None` |
| TV-BC-2.03.009-006 | `PRISM_KEY_FILE` points to a directory | `PrismError::Credential` with regular-file requirement message |

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
