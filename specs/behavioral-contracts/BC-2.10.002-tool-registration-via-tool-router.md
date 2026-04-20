---
document_type: behavioral-contract
level: L3
version: "2.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: [CAP-005, CAP-015]
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.002: Tool Registration via #[tool_router]

**Note:** This file replaces BC-2.10.002 v1.0. Per-sensor read tools are removed; all data access goes through the `query` tool (CAP-015). Per-sensor tools exist only for write operations. The canonical tool inventory is maintained in `architecture/api-surface.md` (Tool Registry section) as the single source of truth for tool names, subsystem assignments, capability gates, and parameter shapes.

## Preconditions
- MCP tools are defined using rmcp's `#[tool_router]` macro
- Each tool handler function accepts a `Parameters<T: JsonSchema>` input type
- Input types follow `{ToolName}Input` naming convention

## Postconditions
- All tools listed in `architecture/api-surface.md` Tool Registry are registered in `tools/list` (subject to capability-gate rules per BC-2.10.003). As of api-surface.md v1.3: 28 always-visible read tools + 24 capability-gated write tools = 52 total in the registry. The authoritative count and names are owned by api-surface.md; this BC governs the structural registration rules.
- Each tool has: `name`, `description` (following BC-2.09.006 template), `inputSchema` (derived from `JsonSchema`), `outputSchema`, and `annotations`

### Tool Inventory — Structural Categories

The canonical per-tool list is in `architecture/api-surface.md` §"Tool Registry Details". This BC defines the invariant structural rules each category must satisfy:

**Always-Visible Read Tools (28 tools as of api-surface.md v1.3):**

Every tool in the "Always-Visible Tools" table of api-surface.md MUST:
- Appear in `tools/list` unconditionally (no capability gate, no feature flag)
- Set `readOnlyHint: true`, `destructiveHint: false`
- Set `idempotentHint: true` (reads are repeatable)
- Set `openWorldHint: true` for tools that query external sensor APIs; `false` for internal-only tools

**Capability-Gated Write Tools (24 tools as of api-surface.md v1.3):**

Every tool in the "Capability-Gated Tools" table of api-surface.md MUST:
- Be hidden from `tools/list` entirely when its capability path is denied for ALL configured clients (hidden-tools pattern, BC-2.04.005)
- Set `readOnlyHint: false`
- Set `destructiveHint: true` for Irreversible risk tier; `false` for Reversible risk tier
- Have a corresponding capability path in capabilities.md matching the `Capability Path` column of api-surface.md

- Read tools set `annotations`: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true` (or `false` for internal-only tools)
- Write tools set `annotations`: `readOnlyHint: false`, `destructiveHint` per risk level, `idempotentHint` per operation semantics

## Invariants
- DI-003: Feature flag deny-by-default -- write tools registered conditionally (BC-2.10.003)
- Tool registry completeness: every tool listed in `architecture/api-surface.md` Tool Registry is registered exactly once via `#[tool_router]`; no tool is registered outside the canonical registry
- Tool visibility rule: every always-visible tool MUST appear in `tools/list` unconditionally; every capability-gated tool MUST be hidden from `tools/list` when its capability path is denied for all clients
- Capability gate integrity: every capability-gated tool's `Capability Path` column in api-surface.md MUST have a corresponding entry in capabilities.md; orphan capability paths are a build-time or startup error
- Tool count is not fixed: the registry grows as new subsystems are added; structural rules above apply to all present and future tools regardless of count

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

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 2.0 | 2026-04-14 | Phase 1 | Reduced tool surface to 15 tools; per-sensor reads removed |
| 2.1 | 2026-04-19 | Burst 43 | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` in management tools inventory table |
| 2.2 | 2026-04-19 | Burst 44 | P3P43-A-HIGH-002: rewrote postcondition, tool inventory, and invariant to eliminate stale 15-tool hardcount and internally inconsistent arithmetic (7+8+5≠15). Replaced fixed count with structural policy: registry completeness, visibility rule, and capability gate integrity. Authoritative tool list deferred to api-surface.md v1.3 (28 always-visible + 24 capability-gated = 52 total). |
