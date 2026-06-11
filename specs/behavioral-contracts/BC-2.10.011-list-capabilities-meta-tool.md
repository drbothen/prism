---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
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
---

# BC-2.10.011: list_capabilities Meta-Tool

## Description

The `list_capabilities` tool is always registered (never gated) and returns the full capability matrix for a specified client (or all clients when `client_id` is null), showing each hierarchical capability path with its status: `enabled`, `runtime_disabled`, or `compile_time_disabled`. The response includes the resolution chain showing which hierarchy level determined the result, uses `trust_level: "internal"`, and is annotated `readOnlyHint: true`, `idempotentHint: true`, `openWorldHint: false`. This tool reveals the complete capability state regardless of what is visible in `tools/list`.

## Preconditions
- The `list_capabilities` tool is always registered (not gated by feature flags)
- The tool accepts `client_id: Option<String>` (required)

## Postconditions
- When `client_id` is provided: returns the full capability matrix for that client, showing all capabilities with their status:
  - `enabled`: both compile-time and runtime flags permit the capability
  - `runtime_disabled`: compile tier permits but runtime TOML denies it
  - `compile_time_disabled`: compile tier denies — no `[[write_endpoints]]` entry in the sensor's TOML spec (registry-derived, BC-2.04.001/BC-2.16.012); for `alias.write`, the `alias-write` cfg feature is not compiled in (BC-2.11.008)
- When `client_id` is null: returns per-client capability summaries for all clients
- Response includes the hierarchical capability path (e.g., `sensor.crowdstrike.containment`) and whether it resolves to enabled or denied
- Response includes the resolution chain showing which level of the hierarchy determined the result
- Response uses `trust_level: "internal"` (capability data is Prism-generated)
- Tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: false`

## Invariants
- DI-003: Feature flag deny-by-default -- the capability matrix reflects deny-by-default semantics

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Invalid `client_id` format | Structured error with validation details |
| `PrismError::Config` | `client_id` not found | Structured error with suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-021 | Client with zero capabilities enabled | Returns full matrix with all capabilities showing `runtime_disabled` or `compile_time_disabled` |
| EC-10-022 | All sensor TOML specs declare `[[write_endpoints]]` but all capabilities runtime-disabled | Matrix shows all write capabilities as `runtime_disabled` with TOML paths for enabling |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_capabilities("acme")` with containment enabled | `sensor.crowdstrike.containment: "enabled"` with resolution chain | happy-path |
| `list_capabilities("acme")` with no `[[write_endpoints]]` declarations loaded (empty `WriteEndpointRegistry`) | All sensor write capabilities `compile_time_disabled` | edge-case |
| `list_capabilities(null)` | Per-client summary for all clients | happy-path |
| Invalid `client_id` format | Structured validation error | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Capability resolution: deny-by-default | kani |
| VP-003 | Capability resolution: most-specific-path wins | kani |
| VP-004 | Capability resolution: deny overrides allow at same specificity | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Stale cargo-feature framing rewritten to registry-derived compile-tier semantics, aligned with error-taxonomy v1.67 E-FLAG-002 row and BC-2.04.001 v1.2: `compile_time_disabled` status redefined as registry-derived compile-tier denial (no `[[write_endpoints]]` entry in the sensor's TOML spec, BC-2.16.012; alias-write cfg feature for `alias.write`); `runtime_disabled` definition reworded "compile-time feature present" → "compile tier permits"; EC-10-022 and the compile-tier test vector restated to declaration-based conditions (EC ID preserved). Status labels (`enabled`/`runtime_disabled`/`compile_time_disabled`), trust_level, annotations, and VP-002/003/004 citations unchanged. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
