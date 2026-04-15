---
document_type: behavioral-contract
level: L3
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: "CAP-005, CAP-015"
---

# BC-2.10.002: Tool Registration via #[tool_router]

**Note:** This file replaces BC-2.10.002 v1.0. The tool surface has been reduced from per-sensor read+write tools to 15 total tools. Per-sensor read tools are removed; all data access goes through the `query` tool (CAP-015). Per-sensor tools exist only for write operations.

## Preconditions
- MCP tools are defined using rmcp's `#[tool_router]` macro
- Each tool handler function accepts a `Parameters<T: JsonSchema>` input type
- Input types follow `{ToolName}Input` naming convention

## Postconditions
- All 15 tools are registered in `tools/list` (subject to feature-flag gating for write tools per BC-2.10.003)
- Each tool has: `name`, `description` (following BC-2.09.006 template), `inputSchema` (derived from `JsonSchema`), `outputSchema`, and `annotations`

### Tool Inventory (15 tools)

**Read tools (always registered):**
| Tool | Purpose | Annotations |
|------|---------|-------------|
| `query` | All data access via PrismQL (CAP-015) | readOnly, idempotent, openWorld |
| `explain_query` | Query planning without execution (CAP-015) | readOnly, idempotent, openWorld |
| `check_sensor_health` | Operational sensor status (CAP-008) | readOnly, idempotent, openWorld |
| `list_capabilities` | Capability introspection per client (CAP-005) | readOnly, idempotent |
| `list_credentials` | Credential inventory per client (CAP-004) | readOnly, idempotent |
| `list_aliases` | Alias inventory (CAP-016) | readOnly, idempotent |
| `explain_alias` | Alias expansion preview (CAP-016) | readOnly, idempotent |

**Write tools (feature-flag gated per BC-2.10.003):**
| Tool | Purpose | Annotations |
|------|---------|-------------|
| `crowdstrike_contain_host` | CrowdStrike host containment | destructive |
| `crowdstrike_acknowledge_alert` | CrowdStrike alert acknowledgment | not destructive |
| `claroty_resolve_alert` | Claroty alert resolution | not destructive |
| `claroty_device_action` | Claroty device action | destructive |
| `cyberint_acknowledge_alert` | Cyberint alert acknowledgment | not destructive |
| `cyberint_close_alert` | Cyberint alert closure | not destructive |
| `armis_update_alert_status` | Armis alert status update | not destructive |
| `armis_device_action` | Armis device action | destructive |

**Management tools (always registered or feature-flag gated):**
| Tool | Purpose | Annotations |
|------|---------|-------------|
| `confirm_action` | Write confirmation flow (CAP-006) | destructive |
| `set_credential` | Credential mutation (CAP-004) | not destructive |
| `delete_credential` | Credential mutation (CAP-004) | destructive |
| `create_alias` | Alias mutation (CAP-016) | not destructive |
| `delete_alias` | Alias mutation (CAP-016) | destructive |

- Read tools set `annotations`: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true` (or `false` for internal-only tools)
- Write tools set `annotations`: `readOnlyHint: false`, `destructiveHint` per risk level, `idempotentHint` per operation semantics

## Invariants
- DI-003: Feature flag deny-by-default -- write tools registered conditionally (BC-2.10.003)
- Tool count: exactly 15 tools in the complete inventory (7 read + 8 write + 5 management = 20 potential, minus feature-flag-hidden write tools)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Tool input type does not implement `JsonSchema` | Build fails with clear error |
| Compile error | Tool handler signature does not match `#[tool_router]` expectations | Build fails |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-003 | Tool name conflicts between sensors | Write tool names are prefixed with sensor ID, preventing conflicts |
| EC-10-004 | Agent calls a tool not in `tools/list` | rmcp returns "tool not found" error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005, CAP-015 |
| L2 Invariants | DI-003 |
| Replaces | BC-2.10.002 v1.0 (per-sensor read + write tool registration) |
| Priority | P0 |
