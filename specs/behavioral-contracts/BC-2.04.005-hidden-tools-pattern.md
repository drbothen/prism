---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
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
input-hash: "dc078d2"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.005: Hidden Tools Pattern -- Stateless Tool List Based on Configured Capabilities

## Description

The MCP `tools/list` response is stateless: it shows the union of all read tools plus any
write tools that are enabled for at least one configured client. Per-client write capability
is not pre-filtered at the `tools/list` level — authorization is enforced at invocation time
via the `client_id` parameter. Write tools disabled for all clients are completely absent from
`tools/list` (not visible-but-disabled). When a write tool is invoked with a `client_id` that
lacks the required capability, a structured error `E-FLAG-001` is returned rather than an
"unknown tool" error.

The server is stateless with respect to client context: there is no session-level "active client"
concept, and `tools/list` is the same regardless of prior tool calls.

## Preconditions
- An MCP client requests `tools/list`
- The server has resolved capabilities for all configured clients

## Postconditions
- Read-only tools always appear in the `tools/list` response regardless of client or feature flags. The tool list shows the union of all read tools across all configured clients.
- **General rule:** A write tool appears in `tools/list` if it is allowed for at least one configured client. Per-client denial is enforced at invocation time via `E-FLAG-001`. This applies to all write tools, not just credential tools. Specifically:
  - Credential mutation tools (`configure_credential_source`, `delete_credential`) are gated by `credential.write`
  - Alias mutation tools (`create_alias`, `delete_alias`) are gated by `alias.write`
  - Schedule mutation tools (`create_schedule`, `delete_schedule`) are gated by `schedule.write`
  - Detection rule mutation tools (`create_rule`, `delete_rule`) are gated by `detection.write`
  - Case mutation tools (`create_case`, `update_case`) are gated by `case.write`
  - Sensor write tools (e.g., `crowdstrike_contain_host`) are gated by their respective sensor capability paths
- Write tools are not pre-filtered by any session-level client context; the `client_id` parameter at invocation time determines capability resolution.
- When a write tool is invoked, the `client_id` parameter determines whether the caller has the required capability. If the capability is denied for that client, a structured error is returned (not "unknown tool").
- There is no session-level "active client" concept. The server is stateless with respect to client context.
- Disabled write tools (disabled for ALL clients) are completely absent from the response (not visible to the AI agent)

## Invariants
- DI-003: Disabled tools are hidden, not visible-but-disabled
- Tool visibility is stateless: the `tools/list` response is the same regardless of prior tool calls

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Flag` | Agent invokes a write tool with a `client_id` that lacks the required capability | Structured error: `code: "E-FLAG-001"`, with the denied capability path and suggestion |
| N/A | Agent invokes a tool hidden from `tools/list` (disabled for all clients) | MCP protocol returns "unknown tool" error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-010 | Write tool enabled for Client A but not Client B | Tool appears in `tools/list`. Invocation with `client_id: "a"` succeeds capability check. Invocation with `client_id: "b"` returns `E-FLAG-001`. |
| EC-04-011 | No clients have any write capabilities enabled | Only read tools appear in `tools/list`; all write tools are hidden |
| EC-04-033 | Write tool invoked with `client_id: null` (cross-client) | Write operations with `client_id: null` are not supported; returns `E-FLAG-006` structured error |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.005.

| Scenario | Config | `tools/list` Includes Write Tool? | Invocation Result |
|----------|--------|----------------------------------|-------------------|
| Write enabled for Client A | `acme` has `sensor.crowdstrike.containment: Allow` | Yes | Invocation with `client_id: "acme"` succeeds; with `client_id: "other"` → `E-FLAG-001` |
| No clients have write enabled | All clients deny writes | No | Tool absent; invocation → "unknown tool" |
| Write tool invoked cross-client | `client_id: null` | N/A | `E-FLAG-006` |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — verifies that runtime-only tools in `tools/list` still require capability check at invocation.

No additional VPs in VP-INDEX v1.5 directly verify the `tools/list` stateless pattern; placeholder for future VP addition.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Addresses | ADV-1-001, ADV-2-003, ADV-2-004, ADV-5-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.2 | Burst 43 | 2026-04-19 | product-owner | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` in postconditions credential mutation tool list |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
