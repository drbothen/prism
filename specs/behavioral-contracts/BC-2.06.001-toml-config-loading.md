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
input-hash: "[pending-recompute]"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.001: TOML Configuration Loads and Deserializes at Startup

## Description

At startup, Prism loads and deserializes its TOML configuration file from the path specified
by CLI argument or `PRISM_CONFIG_PATH` environment variable. All `[clients.{id}]` sections
are deserialized into `ClientConfig` structs with validated `client_id` values. The
`[defaults]` section (if present) is loaded for capability merging. Configuration is
immutable after load — there is no hot-reload mechanism in the stdio per-analyst model, and
the loaded config is available to all subsystems throughout the session lifetime.

Fatal errors at load time (file not found, TOML syntax error, permission denied) terminate
startup immediately with a descriptive error message.

## Preconditions
- A TOML configuration file exists at the path specified by CLI argument or `PRISM_CONFIG_PATH` environment variable
- The file is readable by the Prism process

## Postconditions
- All `[clients.{id}]` sections are deserialized into `ClientConfig` structs
- Each `ClientConfig` has a validated `client_id` matching `[a-zA-Z0-9_-]+`
- The `[defaults]` section (if present) is loaded and available for capability merging
- Configuration is immutable after load -- no hot-reload mechanism exists in the stdio per-analyst model
- The loaded config is available to all subsystems (MCP tools, sensor adapters, credential resolution)

## Invariants
- DI-008: Client data separation -- each `ClientConfig` is independently loaded and scoped

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | TOML file not found at specified path | Fatal error: "Configuration file not found at '{path}'" with suggestion to check the path |
| `PrismError::Config` | TOML syntax error (invalid TOML) | Fatal error with the TOML parser error message including line and column |
| `PrismError::Config` | TOML file is not readable (permission denied) | Fatal error: "Cannot read configuration file at '{path}': permission denied" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-001 | Config file exists but contains zero `[clients.*]` sections | Prism starts successfully with an empty client list; queries return "no clients configured" |
| DEC-006 | Operator changes TOML while Prism is running | Running session continues with startup-time config; no hot-reload |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.001.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Valid config, 2 clients | Well-formed TOML with `[clients.acme]` and `[clients.beta]` | Prism starts; 2 `ClientConfig` structs loaded |
| File not found | Path does not exist | Fatal: "Configuration file not found at '{path}'" |
| TOML syntax error | `api_base = invalid url` (no quotes) | Fatal with TOML parser error including line/column |
| Zero clients | Only `[defaults]` section, no `[clients.*]` | Prism starts; empty client list; no error |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify TOML config loading. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
