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

# BC-2.04.015: Structured Error When Write Capability Is Denied

## Description

When an agent invokes a write tool whose capability is not enabled for the specified client,
Prism returns a structured error — not a generic "unknown tool" error. The structured error
includes the `CAPABILITY_DENIED` code, the exact capability path checked, the `client_id`,
the denial reason, and actionable guidance (exact remediation path plus instruction to
restart Prism). The contract covers both denial tiers: the runtime tier (capability not
enabled in the client's TOML config, E-FLAG-001) and the compile tier, which for sensor
writes is registry-derived per BC-2.16.012 — a sensor has compile-tier write capability if
and only if its TOML spec declares a matching `[[write_endpoints]]` entry loaded into the
`WriteEndpointRegistry` at boot (E-FLAG-002). The `{sensor}-write` Cargo features are empty
test-gating declarations in `prism-query`, NOT the gate; the one genuinely cfg-gated case
is the alias-write path (`alias.write`, BC-2.11.008), which produces the same
`DeniedCompileTime` denial class.

The error is also audit-logged as a denied capability check per BC-2.04.013.

## Preconditions
- A write operation is attempted for a client where the capability is not enabled
- This can occur if the agent calls a tool that exists in the binary but is not registered for the current client (race condition or direct JSON-RPC call bypassing tool list)

## Postconditions
- The response is a structured error (not a generic "unknown tool" error) containing:
  - `code: "CAPABILITY_DENIED"`
  - `capability`: the path that was checked (e.g., `sensor.crowdstrike.containment`)
  - `client_id`: the client context
  - `reason`: "Not enabled in client config" (runtime tier) or, for the compile tier, the registry-derived E-FLAG-002 denial: "Write capability '{path}' denied: no write-endpoint declaration (no [[write_endpoints]] entry in the sensor's TOML spec)"; the alias-write path (BC-2.11.008) produces the same `DeniedCompileTime` class with its cfg-gate reason
  - `suggestion`: actionable guidance (e.g., "Enable 'sensor.crowdstrike.containment' in [clients.acme.capabilities] and restart Prism")
- The error is audit-logged as a denied capability check

## Invariants
- DI-003: Denied operations produce actionable errors, not silent failures

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Structured error | Runtime flag denies the capability | Error with TOML path to enable and restart instruction |
| Structured error | Compile tier denies a sensor write — no `[[write_endpoints]]` entry in the sensor's TOML spec (registry-derived, BC-2.16.012) | E-FLAG-002 error explaining that the sensor's TOML spec declares no write endpoint for this capability; remediation is adding the `[[write_endpoints]]` declaration and restarting Prism — not a rebuild |
| Structured error | Compile tier denies an alias write — binary built without the `alias-write` Cargo feature (BC-2.11.008, the one genuinely cfg-gated case) | Same `DeniedCompileTime` denial class; remediation is rebuilding with the `alias-write` Cargo feature |

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
| Compile tier — sensor write (registry-derived) | No `[[write_endpoints]]` entry for the capability in the crowdstrike sensor TOML spec | `CAPABILITY_DENIED` | "Add a [[write_endpoints]] declaration for 'sensor.crowdstrike.containment' to the crowdstrike sensor TOML spec and restart Prism" |
| Compile tier — alias write (cfg-gated) | Binary built without the `alias-write` Cargo feature | `CAPABILITY_DENIED` | "Rebuild Prism with the `alias-write` Cargo feature to enable alias mutation tools" |

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
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Stale cargo-feature framing rewritten to registry-derived compile-tier semantics, aligned with error-taxonomy v1.67 E-FLAG-002 row. Description: dropped "(compile-time feature present)"; documents both denial tiers — runtime (E-FLAG-001) and compile (registry-derived per BC-2.16.012, `[[write_endpoints]]` in sensor TOML); names `{sensor}-write` Cargo features as empty test-gating declarations and alias-write (BC-2.11.008) as the surviving genuinely cfg-gated case. Postcondition `reason`: "Feature not compiled" replaced with the E-FLAG-002 "no write-endpoint declaration" message format. Error Cases: "rebuild with the feature flag" row split into registry-derived sensor-write row (remediation = add `[[write_endpoints]]` declaration + restart, not rebuild) and accurate alias-write cfg-gated row. Test vectors: "Rebuild Prism with the `crowdstrike-write` Cargo feature" vector replaced with registry-derived declaration vector + alias-write rebuild vector. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
