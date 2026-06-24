---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: demo-readiness-2026-06-24
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
input-hash: "TBD"
traces_to: ["CAP-034"]
extracted_from: null
---

# BC-2.10.017: Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking

## Description

Tools in the `NOT_YET_AVAILABLE_TOOLS` set (`list_infusions`, `plugin_status`, `infusion_status`) MUST return a fast-fail JSON-RPC error response within 1 second of the tool invocation request arriving at the MCP server. The audit channel write (`emit_tool_audit`) MUST NOT be on the blocking path before the fast-fail guard fires; the guard MUST fire before any audit emission for tools in this set.

## Preconditions

- A `tools/call` request arrives for a tool whose name is in the `NOT_YET_AVAILABLE_TOOLS` set
- The MCP server's audit channel (`audit_writer: mpsc::Sender<AuditEntry>`) is active

## Postconditions

- The tool invocation returns a structured JSON-RPC `-32003` error response within **1 second** of request receipt
- The response body uses the `not_yet_available_msg` pattern: a structured JSON-RPC error with code `-32003`, message indicating the tool is not yet available, and a `content[].text` with human-readable guidance
- The audit channel write does NOT block the fast-fail response:
  - **Option A (preferred for production correctness):** the `NOT_YET_AVAILABLE_TOOLS` check fires BEFORE `emit_tool_audit` — no audit event is emitted for a not-yet-available tool invocation (no tool execution occurred; nothing to audit)
  - **Option B (acceptable if A is architecturally infeasible):** `emit_tool_audit` uses `try_send` (non-blocking) with a `tracing::warn!` on `Err(Full)` — audit buffer full condition MUST NOT block the fast-fail path
- If `emit_tool_audit` currently uses blocking `send()`, it MUST be changed to `try_send()` for the fast-fail path OR the fast-fail guard MUST be moved before the audit call

## Invariants

- **INV-NOT-YET-AVAILABLE-GUARD-ORDER:** For tools in `NOT_YET_AVAILABLE_TOOLS`, the guard check (`is_not_yet_available(tool_name)`) MUST evaluate before any blocking operation (including `emit_tool_audit`) in the tool dispatch path.
- **INV-AUDIT-NON-BLOCKING:** `emit_tool_audit` on the fast-fail path MUST use `mpsc::Sender::try_send` (non-blocking) NOT `mpsc::Sender::send` (blocking). An audit buffer full condition generates a `tracing::warn!("audit buffer full; dropping tool audit entry for not-yet-available tool {tool_name}")` and does NOT block the response.
- The set `NOT_YET_AVAILABLE_TOOLS` is a compile-time constant — not configurable at runtime
- Tool names in `NOT_YET_AVAILABLE_TOOLS` are registered in `tools/list` (visible to clients) but invoke the fast-fail handler, consistent with the existing not-yet-available pattern

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| JSON-RPC `-32003` | Tool in `NOT_YET_AVAILABLE_TOOLS` is invoked | Fast-fail within 1s: `{"code": -32003, "message": "Tool '{tool_name}' is not yet available in this release. …"}` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-017-001 | `list_infusions` called when audit channel buffer is full | Returns `-32003` within 1s; audit drop warning logged; NO hang |
| EC-10-017-002 | `plugin_status` called with valid `plugin_name` argument | Returns `-32003` fast-fail within 1s; argument value ignored |
| EC-10-017-003 | `infusion_status` called | Returns `-32003` fast-fail within 1s |
| EC-10-017-004 | Concurrent invocations of `list_infusions` (e.g., 5 simultaneous) | All return `-32003` within 1s each; no head-of-line blocking |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `tools/call list_infusions` | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |
| `tools/call plugin_status {"plugin_name": "crowdstrike-oauth2.prx"}` | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |
| `tools/call infusion_status {"infusion_name": "threat_score"}` | JSON-RPC error `-32003` within 1s | happy-path (fast-fail) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| (none allocated) | Fast-fail within 1s | integration test (timing assertion) |

## Related BCs

- **BC-2.10.011** (related — list_capabilities: NOT in NOT_YET_AVAILABLE set): `list_capabilities` is always-available; this BC governs a different set of tools
- **BC-2.10.009** (unrelated — prompts): prompt fast-fail is governed by BC-2.10.016; this BC governs tool fast-fail

## Architecture Anchors

- `crates/prism-mcp/src/server.rs` — `emit_tool_audit` (change to `try_send` or reorder guard before audit call)
- `crates/prism-mcp/src/server.rs` — `NOT_YET_AVAILABLE_TOOLS` constant and fast-fail handler path
- ADR-046 §BLOCKER-004 root-cause hypothesis (audit channel saturation diagnosis)

## Story Anchor

TBD

## VP Anchors

(none allocated; timing test is integration test scope)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| Capability Anchor Justification | CAP-034 ("MCP Server & Transport") per capabilities.md §CAP-034 — this BC governs the tool dispatch middleware in `PrismServer`. CAP-034 describes "Tool dispatch errors surface as structured MCP error responses, never as raw panics" and the middleware layer on every tool dispatch. Not-yet-available tools are a class of conditional tool registration that the MCP server must handle without blocking. |
| L2 Invariants | DI-004 (audit completeness — audit MUST NOT block the fast-fail path; try_send preserves the intent) |
| Priority | P0 |
| Closes findings | BLOCKER-004 (`list_infusions`, `plugin_status`, `infusion_status` hang indefinitely) |
| ADR traces | ADR-046 (root-cause hypothesis for BLOCKER-004) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | Initial contract. Authored per demo-readiness-remediation-design-2026-06-24.md. Closes BLOCKER-004. Root cause: `emit_tool_audit` blocking `send()` before fast-fail guard; fix: reorder guard before audit OR change to `try_send`. |
