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
capability: "CAP-009, CAP-010"
---

# BC-2.10.001: rmcp ServerHandler Implementation

## Preconditions
- The `prism-mcp` crate implements the `rmcp::ServerHandler` trait
- The rmcp 0.8 SDK is pinned to an exact version in `Cargo.toml`

## Postconditions
- `server_info()` returns server name (`"prism"`), version (from `Cargo.toml`), and protocol version
- The `ServerHandler` impl delegates to the tool router for tool dispatch, resource handlers for resource reads, and prompt handlers for prompt retrieval
- `capabilities()` declares support for: tools, resources, prompts, and notifications
- The server handler is instantiated once per stdio session (one per analyst)
- All tool dispatch goes through a middleware layer that: validates `client_id`, emits AuditEntry, evaluates feature flags for write tools

## Invariants
- DI-004: Audit completeness -- middleware ensures every tool call is audit-logged
- DI-003: Feature flag deny-by-default -- write tools are gated before dispatch

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| MCP protocol error | Invalid JSON-RPC request | rmcp handles protocol-level errors; Prism receives only well-formed requests |
| `PrismError::Config` | ServerHandler initialization fails (bad config) | Fatal startup error; Prism exits before accepting MCP connections |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-001 | rmcp SDK version mismatch at compile time | Build fails with clear dependency error; pinned version prevents accidental upgrades |
| EC-10-002 | Client sends a method not supported by Prism | rmcp returns standard JSON-RPC "method not found" error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009, CAP-010 |
| L2 Invariants | DI-003, DI-004 |
| L2 Risk | R-001 |
| Priority | P0 |
