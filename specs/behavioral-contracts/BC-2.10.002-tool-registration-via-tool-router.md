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
capability: "CAP-001, CAP-005"
---

# BC-2.10.002: Tool Registration via #[tool_router]

## Preconditions
- MCP tools are defined using rmcp's `#[tool_router]` macro
- Each tool handler function accepts a `Parameters<T: JsonSchema>` input type
- Input types follow `{ToolName}Input` naming convention

## Postconditions
- All read-only query tools are registered unconditionally in `tools/list`
- Each tool has: `name`, `description` (following BC-2.09.006 template), `inputSchema` (derived from `JsonSchema`), `outputSchema`, and `annotations`
- Read-only tools set `annotations`: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`
- Write tools set `annotations`: `readOnlyHint: false`, `destructiveHint` per risk level, `idempotentHint` per operation semantics
- Every tool input type includes `client_id: String` as a required field (BC-2.10.005)
- Tool names follow `{action}_{sensor}_{entity}` convention (e.g., `get_crowdstrike_alerts`, `contain_crowdstrike_host`)

## Invariants
- DI-003: Feature flag deny-by-default -- write tools registered conditionally (BC-2.10.003)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Tool input type does not implement `JsonSchema` | Build fails with clear error |
| Compile error | Tool handler signature does not match `#[tool_router]` expectations | Build fails |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-003 | Tool name conflicts between sensors | Names are prefixed with sensor ID, preventing conflicts |
| EC-10-004 | Agent calls a tool not in `tools/list` | rmcp returns "tool not found" error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001, CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
