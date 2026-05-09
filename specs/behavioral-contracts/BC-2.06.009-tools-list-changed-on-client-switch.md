---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "c36ec87"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.009: Config Reload Triggers notifications/tools/list_changed

## Description

When a configuration reload changes the resolved capability set, Prism sends an MCP
`notifications/tools/list_changed` notification to the connected AI agent. The agent
re-fetches `tools/list`, which now reflects the updated capabilities. Write tools newly
enabled appear; write tools no longer enabled are removed. Read tools are unaffected.

There is no session-level "client context switch" concept. The server is stateless. Tool
visibility is the union of capabilities across all configured clients; per-call `client_id`
determines authorization at invocation time. This contract is the config-subsystem counterpart
of BC-2.04.014 (which specifies the same behavior from the feature-flag subsystem's perspective).

## Preconditions
- The MCP server is running and a client is connected
- A configuration reload occurs (e.g., SIGHUP, config file change detection) that changes the resolved capability set

## Postconditions
- Prism sends an MCP `notifications/tools/list_changed` notification to the AI agent
- The notification causes the agent to re-fetch the `tools/list`, which now reflects the updated capabilities
- Write tools newly enabled (for any client) appear in the updated `tools/list`
- Write tools no longer enabled (for any client) are removed from the updated `tools/list`
- Read tools are unaffected (they are always available)
- There is no session-level "client context switch" concept. The server is stateless. Tool visibility is the union of capabilities across all configured clients; per-call `client_id` determines authorization at invocation time.

## Invariants
- DI-003: Feature flag deny-by-default -- the tools/list reflects deny-by-default resolution across all clients

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Notification delivery failure | MCP transport cannot deliver the notification (e.g., client disconnected) | Warning logged; the stale tool list may cause the agent to attempt now-unavailable tools, which return capability-denied errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-016 | Config reload produces the same capability set | `notifications/tools/list_changed` is NOT sent (no-op optimization) |
| EC-06-017 | Config reload adds a new client | Tool list re-evaluated; notification sent if the new client enables previously-hidden write tools |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.009.

| Scenario | Config Change | Notification Sent? | `tools/list` After |
|----------|-------------|-------------------|--------------------|
| New write capability enabled | Add `sensor.crowdstrike.containment: Allow` for Client A | Yes | Write tool appears |
| Write capability removed | Remove write capability from last client that had it | Yes | Write tool disappears |
| No effective change | Same TOML reloaded | No | `tools/list` unchanged |
| New client, write enabled | Add `[clients.new_client]` with write flag | Yes | Write tool newly visible |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify config-reload-triggered notification. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
