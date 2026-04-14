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

# BC-2.06.001: TOML Configuration Loads and Deserializes at Startup

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |
