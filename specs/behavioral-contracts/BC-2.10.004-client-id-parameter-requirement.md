---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: "CAP-001, CAP-002"
---

# BC-2.10.004: client_id Parameter on Every Tool

## Preconditions
- An MCP tool call is received by the server handler

## Postconditions
- Every MCP tool input schema includes `client_id: Option<String>` as a required parameter in the JSON Schema
- When `client_id` is a non-null string: the tool operates on that specific client only
- When `client_id` is null: the tool operates in cross-client mode (CAP-002), fanning out to all configured clients
- The `client_id` value is validated against `[a-zA-Z0-9_-]+` before any processing
- The `client_id` is passed to the sensor adapter, state store, credential store, and audit logger for every operation
- The `client_id` is included in the tracing span for the entire tool invocation

### Exception: Query Engine Tools (Subsystem 11)

Query engine tools (`query`, `explain_query`, `create_alias`, `list_aliases`, `delete_alias`, `explain_alias`) use a `clients` array parameter instead of the scalar `client_id`. Audit entries log `clients` as an array. Feature flag evaluation runs per-client for the clients in the array. These tools are the documented exception to the universal client_id contract.

## Invariants
- DI-008: Client data separation -- `client_id` scopes every downstream operation
- Stateless: there is no session-level "active client" context. Each tool call is self-contained; the `client_id` parameter on that call determines the client scope.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` contains invalid characters | Structured error: `code: "E-MCP-001"`, `message: "Invalid client_id format"`, `allowed_pattern: "[a-zA-Z0-9_-]+"` |
| `PrismError::Config` | Non-null `client_id` not found in config | Structured error: `code: "E-CFG-001"`, `message: "Client '{id}' not found"`, `suggestion: "Check TOML config for available clients"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-005 | Cross-client query where some clients lack the target sensor | Clients without the sensor are silently skipped; response metadata lists `clients_skipped` with reason |
| DEC-003 | Cross-client query where one client has expired credentials | Partial results returned; `partial_failures` array in response metadata |
| EC-10-007 | `client_id` is an empty string | Treated as invalid input (fails `[a-zA-Z0-9_-]+` validation which requires at least one character) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001, CAP-002 |
| L2 Invariants | DI-008 |
| L2 Edge Cases | DEC-003, DEC-005 |
| Priority | P0 |
