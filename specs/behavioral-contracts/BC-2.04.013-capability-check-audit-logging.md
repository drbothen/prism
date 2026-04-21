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
input-hash: "abc4070"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.013: Feature Flag Evaluation Audit Logging for Write Operations

## Description

Every write capability check emits a `CapabilityCheckEvent` via `tracing::info!` with
structured fields capturing the event type, client ID, capability path, result (allowed or
denied), tool name, and denial reason (if denied). This provides a complete audit trail of
all write access decisions for SOC 2 and ISO 27001 compliance purposes. Read operations do
not emit capability check events since they are always allowed.

The event is emitted regardless of whether the capability check allows or denies, making
the audit trail complete even for attempted unauthorized access.

## Preconditions
- A write operation is about to be evaluated (either at tool registration or at invocation time)

## Postconditions
- A `CapabilityCheckEvent` is emitted via `tracing::info!` with structured fields:
  - `event_type: "capability_check"`
  - `client_id`: the tenant
  - `capability`: the checked path (e.g., `sensor.crowdstrike.containment`)
  - `result`: "allowed" | "denied"
  - `tool_name`: the MCP tool that triggered the check
  - `denied_reason` (if denied): "Feature not compiled" or "Not enabled in client config" or "No matching capability path"
  - `timestamp`: UTC
- Read operations do not emit capability check events (they are always allowed)

## Invariants
- DI-004: Audit completeness -- every write capability check is logged
- DI-003: The audit trail proves deny-by-default behavior

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Tracing subscriber fails | Capability check still proceeds; best-effort stderr warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-027 | Cross-client query triggers capability checks for 10 clients | 10 separate capability check events emitted (one per client) |
| EC-04-028 | Capability denied at compile-time tier | Event still emitted with `result: "denied"` and `denied_reason: "Feature not compiled"` |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.013.

| Scenario | Check Result | Expected Event Fields |
|----------|-------------|----------------------|
| Allowed | `sensor.crowdstrike.containment: Allow` | `result: "allowed"`, no `denied_reason` |
| Denied runtime | Capability path not in client map | `result: "denied"`, `denied_reason: "Not enabled in client config"` |
| Denied compile-time | Feature not compiled | `result: "denied"`, `denied_reason: "Feature not compiled"` |
| Cross-client fan-out (10 clients) | Mix of allow/deny per client | 10 separate events, one per client |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify `CapabilityCheckEvent` emission completeness. Placeholder for future VP covering all write paths emit the event.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
