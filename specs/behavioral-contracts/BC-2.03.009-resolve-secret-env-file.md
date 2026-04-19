---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 |
| L2 Invariants | DI-002 |
| Priority | P0 |
