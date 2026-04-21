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
input-hash: "dc078d2"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.004: Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation

## Description

Write operations in Prism require two independent gates to both pass before execution
proceeds. The first gate is compile-time: the `#[cfg(feature = "sensor-write")]` Cargo
feature must be present in the binary (BC-2.04.001). The second gate is runtime: the
`client_capabilities.is_enabled("sensor.{sensor}.{operation}")` check must return `true`
for the specific client (BC-2.04.002). If the compile-time feature is absent, the write
code simply does not exist in the binary. If the compile-time feature is present but the
runtime flag denies, the tool exists but is not registered for that client.

Both tiers produce a distinct, clear denial reason to support operator debugging and audit
trail completeness.

## Preconditions
- A write operation tool is being registered or invoked
- The binary was built with or without the corresponding cargo feature
- The client has or does not have the runtime capability enabled

## Postconditions
- Tool registration check: `#[cfg(feature = "sensor-write")]` gate must pass at compile time AND `client_capabilities.is_enabled("sensor.{sensor}.{operation}")` must return `true` at runtime
- If compile-time feature is absent, the tool code does not exist in the binary (no runtime check possible)
- If compile-time feature is present but runtime flag denies, the tool is not registered for that client
- Both tiers produce a clear reason when they block: compile-time ("Feature not compiled") vs runtime ("Not enabled in client config")

## Invariants
- DI-003: Deny-by-default at both tiers

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A (tool absent) | Compile-time feature disabled | Tool does not exist; agent cannot invoke it; `list_capabilities` reports "Feature not compiled" |
| N/A (tool hidden) | Runtime flag disabled | Tool exists in binary but not registered for this client; `list_capabilities` reports "Not enabled in client config" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-008 | Compile-time enabled, runtime enabled for Client A, disabled for Client B | Tool visibility is determined per-invocation based on the `client_id` parameter — there is no session-level "active client". A tool call with `client_id: "client_a"` sees the tool available; a subsequent call with `client_id: "client_b"` does not. Different `client_id` values in successive tool calls may see different tool availability based on per-client capability configuration. No `notifications/tools/list_changed` is sent because tool registration is static (all compile-time-enabled tools are registered); runtime gating is evaluated at invocation time. |
| EC-04-009 | All write features compiled in but all runtime flags deny | Binary has write code but no client can use it; effectively read-only deployment with latent write capability |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.004.

| Scenario | Compile Feature | Runtime Flag | Expected Result |
|----------|----------------|-------------|----------------|
| Both gates pass | `crowdstrike-write` present | `sensor.crowdstrike.containment: Allow` | Tool registered and executable |
| Compile absent | `crowdstrike-write` absent | N/A | Tool absent from binary; `list_capabilities` → "Feature not compiled" |
| Compile present, runtime deny | `crowdstrike-write` present | `sensor.crowdstrike.containment` not in map | Tool exists in binary; `list_capabilities` → "Not enabled in client config" |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — Kani proof that the two-tier gate requires both conditions to pass; neither alone is sufficient.

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
