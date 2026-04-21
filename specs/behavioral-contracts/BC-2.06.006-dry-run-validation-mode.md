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
input-hash: "5b48b9c"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.006: --dry-run Flag Validates Config and Prints Redacted Summary

## Description

When invoked with `--dry-run`, Prism runs the full configuration validation pipeline
(identical to normal startup validation) but does not start the MCP server. If validation
passes, a redacted summary is printed to stdout listing configured clients (with
`client_id` and `display_name`), per-client sensor configurations (with `api_base` and
`enabled` status), per-client resolved capabilities, and credential references (names only,
values shown as `[REDACTED]`). Prism then exits with code 0.

If validation fails, the multi-error report (per BC-2.06.005) is printed and Prism exits
with a non-zero code. Credential values are never present in `--dry-run` output (DI-002).

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.006.

| Scenario | Config State | Expected Output |
|----------|-------------|----------------|
| Valid config | 2 clients, 1 sensor each | Summary printed to stdout; credential values → `[REDACTED]`; exit 0 |
| Validation errors | Invalid URL in one sensor | Multi-error report printed; no summary; exit non-zero |
| Zero clients | Only `[defaults]` | Summary shows empty client list; exit 0 |
| Keyring locked | Credential ref unresolvable | Credential error in validation report; no summary; exit non-zero |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify `--dry-run` behavior. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-002 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
