---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
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
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.009: Feature Flag Evaluations for Write Operations Are Audit-Logged

## Description

When a write/mutation MCP tool is invoked, the audit entry's `capability_checks` array
records each flag evaluated during hierarchical resolution. Entries are in evaluation order
(most-specific to least-specific, ending at the global deny default). Each entry includes
`capability_path`, `compile_time_enabled`, `runtime_enabled`, and `result`. The final
`result` reflects deny-by-default: `"permitted"` only if both compile-time and runtime tiers
allow the operation.

This full resolution chain in the audit record enables forensic analysis of why a write
was permitted or denied, supporting both SOC 2 and ISO 27001 least-privilege evidence.

## Preconditions
- A write/mutation MCP tool is invoked
- The feature flag system evaluates the capability path for the target client

## Postconditions
- The audit entry's `capability_checks` array records each flag evaluated during hierarchical resolution
- Each entry includes: `capability_path`, `compile_time_enabled`, `runtime_enabled`, `result`
- The resolution chain is recorded in evaluation order (most-specific to least-specific, ending at the global deny default)
- The final `result` field reflects the deny-by-default outcome: `"permitted"` only if both compile-time and runtime tiers allow it

## Invariants
- DI-003: Feature flag deny-by-default
- DI-004: Audit completeness

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile tier denied | The sensor's TOML spec declares no matching `[[write_endpoints]]` entry (registry-derived compile tier, BC-2.04.001/BC-2.16.012; for `alias.write`, the `alias-write` cfg feature is absent) | `capability_checks` records `compile_time_enabled: false` and `result: "denied"`; the denial surfaces as E-FLAG-002 (`PrismError::CapabilityDenied`, BC-2.04.015) and the denied check is audit-logged per BC-2.04.013 |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-015 | A write tool is invoked for a client where the capability is enabled at `sensor.crowdstrike.write` but not at the more-specific `sensor.crowdstrike.containment` | The audit records both paths evaluated and the final resolution per the hierarchy rules |
| EC-05-016 | `list_capabilities` meta-tool invocation | This is a read operation; `capability_checks` is empty; the tool's response data (the capability matrix) is recorded in `result_summary` |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.009.

| Scenario | Resolution Chain | `capability_checks` entries |
|----------|-----------------|----------------------------|
| Direct path match | `sensor.crowdstrike.containment: Allow` | One entry: `{capability_path: "sensor.crowdstrike.containment", compile_time_enabled: true, runtime_enabled: true, result: "permitted"}` |
| Parent match, child absent | `sensor.crowdstrike: Allow` | Two entries: child (no match), parent (`result: "permitted"`) |
| Override deny at child | `sensor.crowdstrike: Allow`, `sensor.crowdstrike.containment: Deny` | Two entries: child (`result: "denied"`) wins; parent not reached |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify the `capability_checks` resolution chain in audit entries. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Error Cases compile-tier row rewritten from cargo-feature framing ("The cargo feature for the sensor's write capability is not compiled in") to registry-derived semantics aligned with error-taxonomy v1.67 E-FLAG-002 row: condition = no matching `[[write_endpoints]]` entry in the sensor's TOML spec (BC-2.04.001/BC-2.16.012; alias-write cfg feature for `alias.write`). Removed the now-false "the tool is not registered and this path should not normally be reached" claim — post-PLUGIN-MIGRATION-001-B the `DeniedCompileTime` path is evaluated per invocation by `write_pipeline.rs` and surfaces as E-FLAG-002 (BC-2.04.015). Audit field schema (`compile_time_enabled`, `runtime_enabled`, `result`) unchanged. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
