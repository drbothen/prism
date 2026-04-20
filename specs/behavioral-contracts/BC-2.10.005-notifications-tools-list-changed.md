---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-005", "CAP-009"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-005, CAP-009"
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

# BC-2.10.005: notifications/tools/list_changed on Config Reload

## Description

When a configuration reload changes the resolved set of available tools (write tools added or removed due to capability flag changes, or sensor spec changes alter query engine sources), Prism sends `notifications/tools/list_changed` so the MCP client can re-fetch `tools/list`. If the tool set is unchanged after reload, no notification is sent (idempotent). There is no "client context switch" trigger — the server is stateless; per-call `client_id` determines capability resolution at invocation time, not at `tools/list` time.

## Preconditions
- The MCP client has connected and received an initial `tools/list`
- A configuration reload is triggered (e.g., SIGHUP, config file watch, or explicit reload command)

## Postconditions
- Prism re-evaluates capabilities from the updated TOML configuration
- If the resolved set of available tools changes (write tools added or removed), Prism sends a `notifications/tools/list_changed` notification to the MCP client
- The MCP client re-fetches `tools/list` and sees the updated set of tools
- If the tool set does not change after reload, no notification is sent (idempotent)
- There is no "client context switch" trigger. The server is stateless with respect to client context. Per-call `client_id` determines capability resolution at invocation time, not at `tools/list` time.

## Invariants
- DI-003: Feature flag deny-by-default -- tool set reflects resolved capabilities across all clients
- Notifications triggered only by config reload, not by per-call client_id routing

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | Notification delivery failure (MCP transport issue) | Best-effort; if notification fails, the client may have stale tool list until next `tools/list` request |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-008 | Config reload removes a client that had unique write capabilities | Write tools that are no longer enabled for any client disappear from `tools/list`; notification sent |
| EC-10-009 | Rapid config reloads in quick succession | Each reload evaluates capabilities; notification sent only when the tool set actually changes |
| DEC-006 | Config file changes on disk without explicit reload signal | Prism does not auto-detect file changes unless a file watcher is configured; the running session uses the last-loaded config |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Config reload enables a new write capability | `notifications/tools/list_changed` sent; agent re-fetches sees new tool | happy-path |
| Config reload with no tool set change | No notification sent | edge-case |
| Rapid back-to-back reloads, only last changes tool set | Notification sent once for the actual change | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-020 | Feature flag: compile AND runtime must both permit | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005, CAP-009 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Author | Change |
|---------|------|-------|--------|--------|
| 1.0 | 2026-04-14 | cycle-1 | product-owner | Initial draft |
| 1.1 | (prior) | product-owner | Prior remediation |
| 1.2 | 2026-04-20 | pre-build-sweep | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended Changelog row. |
| 1.3 | 2026-04-20 | pre-build-sweep | product-owner | Normalized capability frontmatter from YAML array to string scalar per corpus convention (IMP-006). |
