---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-034"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
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

# BC-2.10.001: rmcp ServerHandler Implementation

## Description

The `prism-mcp` crate implements `rmcp::ServerHandler` against the rmcp 1.4 SDK (version-pinned in `Cargo.toml`). The handler delegates tool dispatch to the tool router, resource reads to resource handlers, and prompt retrieval to prompt handlers; `capabilities()` declares support for tools, resources, prompts, and notifications. All tool dispatch flows through a middleware layer that validates `client_id`, emits AuditEntry per DI-004, and evaluates feature flags for write tools per DI-003. One server handler instance per stdio session (one per analyst).

## Preconditions
- The `prism-mcp` crate implements the `rmcp::ServerHandler` trait
- The rmcp 1.4 SDK is pinned to an exact version in `Cargo.toml`

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| MCP `initialize` handshake | `server_info` with name "prism", version from Cargo.toml, capabilities listing tools/resources/prompts/notifications | happy-path |
| Tool call dispatched through middleware | AuditEntry emitted; feature flag check executed before write tool dispatch | happy-path |
| Invalid JSON-RPC request | rmcp handles at protocol level; Prism not reached | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-020 | Feature flag: compile AND runtime must both permit | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-003, DI-004 |
| L2 Risk | R-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
