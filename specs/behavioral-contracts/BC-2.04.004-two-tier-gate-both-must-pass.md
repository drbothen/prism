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

# BC-2.04.004: Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation

## Description

Write operations in Prism require two independent gates to both pass before execution
proceeds. The first gate is the compile tier: registry-derived per BC-2.04.001 — the
sensor's TOML spec must declare a matching `[[write_endpoints]]` entry loaded into the
`WriteEndpointRegistry` at boot (BC-2.16.012). The second gate is runtime: the
`client_capabilities.is_enabled("sensor.{sensor}.{operation}")` check must return `true`
for the specific client (BC-2.04.002). If the compile tier denies (no write-endpoint
declaration), the write is rejected `DeniedCompileTime` (E-FLAG-002) regardless of runtime
config — the write code remains in the binary; the capability is structurally undeclared.
If the compile tier permits but the runtime flag denies, the tool exists but is not
registered for that client. (The alias-write path, BC-2.11.008, is the one genuinely
cfg-derived compile gate and produces the same denial class.)

Both tiers produce a distinct, clear denial reason to support operator debugging and audit
trail completeness.

## Preconditions
- A write operation tool is being registered or invoked
- The sensor's TOML spec does or does not declare a matching `[[write_endpoints]]` entry (compile tier, BC-2.04.001; for `alias.write`, the `alias-write` cfg feature is or is not compiled)
- The client has or does not have the runtime capability enabled

## Postconditions
- Two-tier check: the compile tier must permit (matching `[[write_endpoints]]` declaration in the `WriteEndpointRegistry`, BC-2.04.001) AND `client_capabilities.is_enabled("sensor.{sensor}.{operation}")` must return `true` at runtime
- If the compile tier denies, the result is `CapabilityCheckResult::DeniedCompileTime` → `PrismError::CapabilityDenied` (E-FLAG-002) regardless of runtime configuration — the runtime tier cannot override
- If the compile tier permits but the runtime flag denies, the tool is not registered for that client
- Both tiers produce a clear reason when they block: compile tier ("no write-endpoint declaration (no [[write_endpoints]] entry in the sensor's TOML spec)", E-FLAG-002) vs runtime ("Not enabled in client config", E-FLAG-001)

## Invariants
- DI-003: Deny-by-default at both tiers

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| E-FLAG-002 (`DeniedCompileTime`) | Compile tier denied — no matching `[[write_endpoints]]` declaration | Write rejected with structured denial regardless of runtime config; `list_capabilities` reports `compile_time_disabled` with the "no write-endpoint declaration" reason |
| N/A (tool hidden) | Runtime flag disabled | Tool exists in binary but not registered for this client; `list_capabilities` reports "Not enabled in client config" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-008 | Compile-time enabled, runtime enabled for Client A, disabled for Client B | Tool visibility is determined per-invocation based on the `client_id` parameter — there is no session-level "active client". A tool call with `client_id: "client_a"` sees the tool available; a subsequent call with `client_id: "client_b"` does not. Different `client_id` values in successive tool calls may see different tool availability based on per-client capability configuration. No `notifications/tools/list_changed` is sent because tool registration is static (all compile-time-enabled tools are registered); runtime gating is evaluated at invocation time. |
| EC-04-009 | All sensor TOML specs declare `[[write_endpoints]]` but all runtime flags deny | Compile tier passes everywhere but no client can write; effectively read-only deployment with latent write capability |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.004.

| Scenario | Compile Tier | Runtime Flag | Expected Result |
|----------|----------------|-------------|----------------|
| Both gates pass | CrowdStrike `[[write_endpoints]]` declared | `sensor.crowdstrike.containment: Allow` | Tool registered and executable |
| Compile tier denied | No `[[write_endpoints]]` entry in the CrowdStrike sensor TOML spec | N/A (cannot override) | Write denied `DeniedCompileTime` (E-FLAG-002); `list_capabilities` → `compile_time_disabled` ("no write-endpoint declaration") |
| Compile permits, runtime deny | CrowdStrike `[[write_endpoints]]` declared | `sensor.crowdstrike.containment` not in map | Tool exists in binary; `list_capabilities` → "Not enabled in client config" |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — Kani proof that the two-tier gate requires both conditions to pass; neither alone is sufficient. The compile-tier condition is registry-derived per BC-2.04.001.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Tier-1 (compile) framing rewritten from `#[cfg(feature = "sensor-write")]` cargo-feature semantics to registry-derived semantics, aligned with error-taxonomy v1.67 E-FLAG-002 row and BC-2.04.001 v1.2: compile tier = matching `[[write_endpoints]]` declaration in `WriteEndpointRegistry` (BC-2.16.012); denial = `DeniedCompileTime` → E-FLAG-002 with "no write-endpoint declaration" reason; "tool code does not exist in the binary" claims removed (write code remains in binary post-PLUGIN-MIGRATION-001-B; capability is structurally undeclared). Runtime-tier reason cited as E-FLAG-001. EC-04-009 and test vectors restated to declaration-present/absent semantics (EC IDs preserved). Alias-write (BC-2.11.008) noted as the surviving genuinely cfg-derived compile gate. Two-tier AND semantics and VP-020 citation unchanged. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
