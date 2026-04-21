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

# BC-2.06.008: Default Values Apply and Environment Variables Override TOML

## Description

Configuration resolution is layered: CLI args > env vars > TOML values > built-in defaults.
Environment variables use the `PRISM_` prefix with `_` separating hierarchy levels (e.g.,
`PRISM_CLIENTS_ACME_SENSORS_CROWDSTRIKE_CREDENTIAL`). The `_FILE` suffix pattern enables
K8s secret mount patterns by reading the credential from a file path. `_FILE` takes
precedence over the bare env var. Built-in defaults apply for optional fields: sensors
default to `enabled: true` and `data_sources: all sources for that sensor type`.

Standard validation is applied to all values after resolution — an env var override that
produces an invalid value (e.g., non-URL for `api_base`) is a validation error, not a
silent override.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.008.

| Scenario | Source | Expected Resolution |
|----------|--------|---------------------|
| Both `_FILE` and bare env var set | `PRISM_..._CREDENTIAL_FILE=/path` and `PRISM_..._CREDENTIAL=abc` | `_FILE` wins; file content used |
| Env var overrides TOML | TOML `api_base = "https://old.example.com"`; `PRISM_..._API_BASE=https://new.example.com` | Env var value used |
| Default applied | `enabled` not in TOML | `enabled: true` (built-in default) |
| Invalid env var value | `PRISM_..._API_BASE=not-a-url` | Validation error after resolution |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify configuration layering. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | -- |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
