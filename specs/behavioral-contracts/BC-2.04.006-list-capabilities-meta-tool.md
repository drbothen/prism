---
document_type: behavioral-contract
level: L3
version: "1.1"
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
input-hash: "e5de7f9"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.006: list_capabilities Meta-Tool for Capability Discovery

## Description

The `list_capabilities` MCP tool is always registered (not gated by any feature flag) and
provides a complete capability matrix for AI agent introspection. For each capability path it
reports the combined enablement result, the compile-time flag status, the runtime TOML flag
status, and a human-readable reason when disabled. This meta-tool enables agents to answer
questions like "which clients can I contain hosts for?" before attempting write operations.

Its response is guaranteed to be consistent with what `tools/list` shows: if
`list_capabilities` reports a capability as enabled, the tool will appear in `tools/list`
and vice versa.

## Preconditions
- The `list_capabilities` MCP tool is always registered (not gated by any feature flag)
- The caller provides an optional `client_id` parameter

## Postconditions
- Returns a complete capability matrix showing all possible tools and their enablement status
- For each capability path, reports:
  - `enabled: bool` (the combined result of both tiers)
  - `compile_time: bool` (whether the cargo feature is present in the binary)
  - `runtime: bool` (whether the runtime TOML flag permits it for this client)
  - `reason: String` (human-readable explanation when disabled, e.g., "Feature not compiled (crowdstrike-write)" or "Not enabled in client config")
- If `client_id` is provided, shows capabilities for that specific client
- If `client_id` is null, shows capabilities for all clients in a per-client breakdown

## Invariants
- `list_capabilities` is always available regardless of feature flags
- The reported status is consistent with what `tools/list` shows

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | Provided `client_id` not found | Structured error: "Client '{id}' not found in configuration" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-012 | Agent calls `list_capabilities` with no client context | Returns global capability matrix showing all clients; useful for "which clients can I contain hosts for?" queries |
| EC-04-013 | Binary built with zero write features | All write capabilities show `compile_time: false, enabled: false` with reason "Feature not compiled" |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.006.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| All features compiled, runtime allow | `client_id: "acme"`, `crowdstrike-write` feature present, `sensor.crowdstrike.containment: Allow` | `{enabled: true, compile_time: true, runtime: true}` |
| Feature absent | `client_id: "acme"`, `crowdstrike-write` absent | `{enabled: false, compile_time: false, runtime: false, reason: "Feature not compiled (crowdstrike-write)"}` |
| Feature present, runtime deny | `client_id: "acme"`, feature present, no capability entry | `{enabled: false, compile_time: true, runtime: false, reason: "Not enabled in client config"}` |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify `list_capabilities` meta-tool behavior. Placeholder for future VP addition covering matrix consistency with `tools/list`.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
