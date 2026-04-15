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

# BC-2.06.010: Client ID Validation Enforces Allowed Character Set

## Preconditions
- A `client_id` value is provided in TOML configuration (as a TOML key under `[clients.*]`)
- The config loader is constructing `TenantId` newtype instances

## Postconditions
- The `client_id` is validated against the pattern `[a-zA-Z0-9_-]+`
- The `client_id` is non-empty
- The `client_id` is unique across all configured clients
- Valid IDs are wrapped in the `TenantId` newtype, which enforces the invariant at the type level
- The same validation applies to `client_id` values in MCP tool call parameters at runtime

## Invariants
- DI-008: Client data separation -- validated `TenantId` prevents path traversal or injection via client IDs

## Reserved Identifiers
- `__global__` is a reserved identifier that cannot be used as a client name in configuration. It is used internally as a sentinel value for global-scope operations (aliases, schedules, packs, global-scope rules) in ConfirmationToken `client_id` fields.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` contains spaces, dots, slashes, or other disallowed characters | "Invalid client_id '{value}': must match [a-zA-Z0-9_-]+" |
| `PrismError::InvalidInput` | `client_id` is empty | "Invalid client_id: must be non-empty" |
| `PrismError::InvalidInput` | `client_id` is `__global__` (reserved) | "Invalid client_id '__global__': reserved identifier" |
| `PrismError::Config` | Duplicate `client_id` in TOML | "Duplicate client_id '{value}' in configuration" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-018 | `client_id` is a single character (e.g., `a`) | Valid; no minimum length beyond non-empty |
| EC-06-019 | `client_id` contains only hyphens and underscores (e.g., `--__`) | Valid per the pattern; unusual but not prohibited |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |
