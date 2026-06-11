---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
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
  - `compile_time: bool` (whether the compile tier permits — registry-derived: the sensor's TOML spec declares a matching `[[write_endpoints]]` entry loaded into the `WriteEndpointRegistry` at boot, BC-2.04.001/BC-2.16.012; for `alias.write`, whether the `alias-write` cfg feature is compiled in, BC-2.11.008)
  - `runtime: bool` (whether the runtime TOML flag permits it for this client)
  - `reason: String` (human-readable explanation when disabled, e.g., "no write-endpoint declaration (no [[write_endpoints]] entry in the sensor's TOML spec)" or "Not enabled in client config")
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
| EC-04-013 | No sensor TOML spec declares `[[write_endpoints]]` entries (empty `WriteEndpointRegistry`) | All sensor write capabilities show `compile_time: false, enabled: false` with the "no write-endpoint declaration" reason |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.006.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Compile tier permits, runtime allow | `client_id: "acme"`, CrowdStrike `[[write_endpoints]]` declared, `sensor.crowdstrike.containment: Allow` | `{enabled: true, compile_time: true, runtime: true}` |
| Declaration absent | `client_id: "acme"`, no `[[write_endpoints]]` entry in the CrowdStrike sensor TOML spec | `{enabled: false, compile_time: false, runtime: false, reason: "no write-endpoint declaration (no [[write_endpoints]] entry in the sensor's TOML spec)"}` |
| Compile tier permits, runtime deny | `client_id: "acme"`, declaration present, no capability entry | `{enabled: false, compile_time: true, runtime: false, reason: "Not enabled in client config"}` |

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
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Stale cargo-feature framing rewritten to registry-derived compile-tier semantics, aligned with error-taxonomy v1.67 E-FLAG-002 row and BC-2.04.001 v1.2: `compile_time: bool` redefined as registry-derived compile-tier permission (`[[write_endpoints]]` in sensor TOML, BC-2.16.012; alias-write cfg feature for `alias.write`); `reason` example "Feature not compiled (crowdstrike-write)" replaced with the "no write-endpoint declaration" message format; EC-04-013 and test vectors restated to declaration-present/absent semantics (EC ID preserved). Response field names (`compile_time`, `runtime`, `enabled`, `reason`) unchanged. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
