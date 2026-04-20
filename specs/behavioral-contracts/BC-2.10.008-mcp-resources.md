---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
extracted_from: null
subsystem: "SS-10"
capability: ["CAP-008", "CAP-009"]
lifecycle_status: active
introduced: cycle-1
modified: ["cycle-1-burst-45"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.008: MCP Resources for Client List and Sensor Inventory

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-04-19 | product-owner | Burst 45 / P3P44-A-HIGH-003: Health resource reference updated from `prism://health/{client_id}` to `prism://sensors/health` (global matrix) to match api-surface.md. Added missing template sections: Description, Canonical Test Vectors, Verification Properties. |
| 1.0 | 2026-04-14 | product-owner | Initial draft |

## Description

This BC governs the MCP resources that expose client inventory and per-client sensor configuration: `prism://clients` (all configured clients) and `prism://clients/{client_id}/sensors` (sensor inventory for a specific client). It also references `prism://sensors/health` (governed by BC-2.08.006) as part of the complete resources registry. All three resources are read-only and reflect startup-time configuration; they never expose credential values.

## Preconditions

1. Prism has loaded configuration and initialized all client/sensor mappings
2. MCP resources are registered in `resources/list`

## Postconditions

1. `prism://clients` resource returns a JSON array of all configured clients with: `client_id`, `display_name`, `sensors` (list of enabled sensor IDs), `capabilities_summary` (count of enabled write capabilities)
2. `prism://clients/{client_id}/sensors` resource returns detailed sensor inventory for a specific client: sensor ID, API base URL (redacted to host only), enabled status, configured data sources
3. `prism://sensors/health` resource returns cached health status per BC-2.08.006 (global cross-client matrix)
4. Resource content uses `application/json` MIME type
5. Resources are read-only and reflect startup-time configuration (no live updates until `reload_config`)
6. Credential values and full API URLs are never exposed in resource content

## Invariants

- DI-002: Credential isolation — no credential values in resource responses; API URLs redacted to host only
- DI-008: Client data separation — `prism://clients/{client_id}/sensors` scoped to the specified client_id

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource not found | Invalid `client_id` in `prism://clients/{client_id}/sensors` URI | MCP resource error: "Client '{id}' not found" |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-014 | Zero clients configured | `prism://clients` returns empty JSON array `[]` |
| EC-10-015 | Client has sensors configured but all disabled | `prism://clients/{id}/sensors` lists sensors with `enabled: false` |
| EC-10-016 | Client has no sensors configured | `prism://clients/{id}/sensors` returns empty `sensors` array, not an error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://clients` with two clients ("acme", "globex") configured | JSON array with two objects, each containing `client_id`, `display_name`, `sensors`, `capabilities_summary` | happy-path |
| Read `prism://clients/acme/sensors` with CrowdStrike and Claroty configured | JSON with two sensor entries; API URL shows host only (e.g., `api.crowdstrike.com`), no full URL or credentials | happy-path |
| Read `prism://clients/nonexistent/sensors` | MCP resource error: "Client 'nonexistent' not found" | error |
| Read `prism://clients` with zero clients configured | `[]` (empty array) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `prism://clients/{id}/sensors` response never contains a string matching an API key or token pattern | proptest / fuzz |
| VP-TBD | `prism://clients/{id}/sensors` full API base URL never appears; only the host component | manual / integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008, CAP-009 (dual anchor) |
| L2 Invariants | DI-002, DI-008 |
| Architecture Module | SS-10 (filled by architect) |
| Stories | S-5.03 |

## Related BCs

- BC-2.08.006 — composes with: `prism://sensors/health` is part of the resource registry covered by this BC
- BC-2.08.005 — depends on: `check_sensor_health` tool populates the health data exposed via `prism://sensors/health`

## Architecture Anchors

- `architecture/api-surface.md#configuration-state-resources` — `prism://clients` is a Configuration State resource
- `architecture/api-surface.md#event-feed-resources` — `prism://sensors/health` is an Event Feed resource

## Story Anchor

S-5.03 — Resources and Prompts
