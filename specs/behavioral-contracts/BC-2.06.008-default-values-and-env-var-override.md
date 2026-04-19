---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.06.008: Default Values Apply and Environment Variables Override TOML

## Preconditions
- TOML configuration has been parsed
- Environment variables matching the `PRISM_{DOMAIN}_{FIELD}` pattern may be set

## Postconditions
- Layered config resolution follows the priority order: CLI args > env vars > TOML values > built-in defaults
- Built-in defaults for optional fields:
  - `sensors.{sensor}.enabled`: `true`
  - `sensors.{sensor}.data_sources`: all sources for that sensor type
- Environment variables use the `PRISM_` prefix with `_` separating hierarchy levels
- The `_FILE` suffix pattern (e.g., `PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CREDENTIAL_FILE`) reads the value from the file at the specified path, supporting K8s secret mount patterns
- `_FILE` suffix takes precedence over the bare env var (e.g., `_FILE` > `_CREDENTIAL`)

## Invariants
- None specific -- this is a configuration layering contract

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | `_FILE` env var points to unreadable file | "Cannot read secret file '{path}' referenced by {env_var}: {io_error}" |
| `PrismError::Config` | Env var override results in an invalid value (e.g., non-URL for `api_base`) | Standard validation error applied after override resolution |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-013 | Both `PRISM_..._CREDENTIAL` and `PRISM_..._CREDENTIAL_FILE` are set | `_FILE` wins; the bare env var is ignored |
| EC-06-014 | Env var overrides a TOML value that was explicitly set | Env var wins per the layered precedence; the TOML value is shadowed |
| EC-06-015 | `_FILE` env var points to a file with only whitespace | Credential value is empty after trimming; validation error: "Credential file '{path}' is empty after trimming whitespace" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | -- |
| Priority | P0 |
