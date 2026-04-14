---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Client Configuration"
capability: "CAP-009"
---

# BC-2.06.006: --dry-run Flag Validates Config and Prints Redacted Summary

## Preconditions
- Prism is invoked with the `--dry-run` CLI flag
- A TOML configuration file is specified

## Postconditions
- The full configuration validation pipeline runs (same as normal startup)
- If validation passes, a redacted summary is printed to stdout showing:
  - List of configured clients with their `client_id` and `display_name`
  - Per-client sensor list with `sensor_id`, `api_base`, and `enabled` status
  - Per-client resolved capabilities (the merged capability set)
  - Credential references (names only, values shown as `[REDACTED]`)
- Prism exits with code 0 after printing the summary (the MCP server is NOT started)
- If validation fails, the multi-error report is printed and Prism exits with a non-zero code

## Invariants
- DI-002: Credential isolation -- credential values never appear in dry-run output

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Config validation errors | Any validation errors exist | Multi-error report printed (per BC-2.06.005); exit code non-zero |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-009 | `--dry-run` with a config that has zero clients | Prints summary showing empty client list; exits with code 0 (valid but empty config) |
| EC-06-010 | `--dry-run` with credentials that cannot be resolved (keyring locked, env var missing) | Credential resolution errors are reported as part of the validation error list; the summary is not printed if validation fails |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-002 |
| Priority | P0 |
