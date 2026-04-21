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
input-hash: "67e5667"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.015: Structured Error When Write Capability Is Denied

## Description

When an agent invokes a write tool that exists in the binary (compile-time feature present)
but the capability is not enabled for the specified client, Prism returns a structured error
— not a generic "unknown tool" error. The structured error includes the `CAPABILITY_DENIED`
code, the exact capability path checked, the `client_id`, the denial reason, and actionable
guidance (exact TOML path to enable the capability plus instruction to restart Prism).

The error is also audit-logged as a denied capability check per BC-2.04.013.

## Preconditions
- A write operation is attempted for a client where the capability is not enabled
- This can occur if the agent calls a tool that exists in the binary but is not registered for the current client (race condition or direct JSON-RPC call bypassing tool list)

## Postconditions
- The response is a structured error (not a generic "unknown tool" error) containing:
  - `code: "CAPABILITY_DENIED"`
  - `capability`: the path that was checked (e.g., `sensor.crowdstrike.containment`)
  - `client_id`: the client context
  - `reason`: "Not enabled in client config" or "Feature not compiled"
  - `suggestion`: actionable guidance (e.g., "Enable 'sensor.crowdstrike.containment' in [clients.acme.capabilities] and restart Prism")
- The error is audit-logged as a denied capability check

## Invariants
- DI-003: Denied operations produce actionable errors, not silent failures

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Structured error | Runtime flag denies the capability | Error with TOML path to enable and restart instruction |
| Structured error | Compile-time feature absent | Error explaining that the binary must be rebuilt with the feature flag |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-032 | Agent receives denial and asks "how do I enable this?" | The structured error already contains the exact config path and action needed |
| EC-04-033 | Capability path partially matches (e.g., `sensor.crowdstrike` enabled but `sensor.crowdstrike.containment` specifically denied) | If the capability system supports explicit deny entries, the deny wins; otherwise, parent match enables child |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.015.

| Scenario | Denial Tier | Expected Error Code | Expected `suggestion` |
|----------|------------|--------------------|-----------------------|
| Runtime deny | Capability not in client map | `CAPABILITY_DENIED` | "Enable 'sensor.crowdstrike.containment' in [clients.acme.capabilities] and restart Prism" |
| Compile-time absent | Feature not compiled | `CAPABILITY_DENIED` | "Rebuild Prism with the `crowdstrike-write` Cargo feature to enable this operation" |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — verifies that denied write operations produce structured errors, not silent failures or unexpected panics.

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
