---
document_type: behavioral-contract
level: L3
version: "1.7"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]
input-hash: "05b3cd4"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
extracted_from: null
subsystem: "SS-10"
capability: "CAP-008, CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: ["cycle-1-burst-45", "cycle-1-burst-49", "pass-69-housekeeping", "pass-73-fix", "pass-79-fix"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.008: MCP Resources for Client List and Sensor Inventory

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | pass-79-fix | 2026-04-20 | state-manager | MED-001 fix: removed stale `pass-72-fix` entry from modified array (no corresponding changelog row existed; pass-72 did not touch this file). |
| 1.6 | pass-73-fix | 2026-04-20 | state-manager | Renumbered changelog to close v1.4 gap: old v1.5→v1.4; old v1.6→v1.5; this row closes the sequence at v1.6. Original v1.3→v1.5 spanned two distinct burst events that were conflated at authoring time. |
| 1.5 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. (originally recorded as v1.6; renumbered by pass-73-fix) |
| 1.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-050); normalized changelog schema to canonical 5-col form. (originally recorded as v1.5; renumbered by pass-73-fix) |
| 1.3 | cycle-1-burst-45 | 2026-04-20 | product-owner | pre-build-sweep: Template-compliance sweep — appended Changelog row (version bump 1.2→1.3). |
| 1.2 | cycle-1-burst-49 | 2026-04-19 | architect | Burst 49 / P3P48-A-HIGH-003: Canonicalized all resource URIs: `prism://clients` → `prism://config/clients`; `prism://clients/{client_id}/sensors` → `prism://config/clients/{client_id}/sensors`. Updated Description, Postconditions, Invariants, Error Cases, Edge Cases, Canonical Test Vectors, Verification Properties, and Architecture Anchors. |
| 1.1 | cycle-1-burst-45 | 2026-04-19 | product-owner | Burst 45 / P3P44-A-HIGH-003: Health resource reference updated from `prism://health/{client_id}` to `prism://sensors/health` (global matrix) to match api-surface.md. Added missing template sections: Description, Canonical Test Vectors, Verification Properties. |
| 1.0 | — | 2026-04-14 | product-owner | Initial draft |

## Description

This BC governs the MCP resources that expose client inventory and per-client sensor configuration: `prism://config/clients` (all configured clients) and `prism://config/clients/{client_id}/sensors` (sensor inventory for a specific client). It also references `prism://sensors/health` (governed by BC-2.08.006) as part of the complete resources registry. All three resources are read-only and reflect startup-time configuration; they never expose credential values.

## Preconditions

1. Prism has loaded configuration and initialized all client/sensor mappings
2. MCP resources are registered in `resources/list`

## Postconditions

1. `prism://config/clients` resource returns a JSON array of all configured clients with: `client_id`, `display_name`, `sensors` (list of enabled sensor IDs), `capabilities_summary` (count of enabled write capabilities)
2. `prism://config/clients/{client_id}/sensors` resource returns detailed sensor inventory for a specific client: sensor ID, API base URL (redacted to host only), enabled status, configured data sources
3. `prism://sensors/health` resource returns cached health status per BC-2.08.006 (global cross-client matrix)
4. Resource content uses `application/json` MIME type
5. Resources are read-only and reflect startup-time configuration (no live updates until `reload_config`)
6. Credential values and full API URLs are never exposed in resource content

## Invariants

- DI-002: Credential isolation — no credential values in resource responses; API URLs redacted to host only
- DI-008: Client data separation — `prism://config/clients/{client_id}/sensors` scoped to the specified client_id

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource not found | Invalid `client_id` in `prism://config/clients/{client_id}/sensors` URI | MCP resource error: "Client '{id}' not found" |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-014 | Zero clients configured | `prism://config/clients` returns empty JSON array `[]` |
| EC-10-015 | Client has sensors configured but all disabled | `prism://config/clients/{id}/sensors` lists sensors with `enabled: false` |
| EC-10-016 | Client has no sensors configured | `prism://config/clients/{id}/sensors` returns empty `sensors` array, not an error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://config/clients` with two clients ("acme", "globex") configured | JSON array with two objects, each containing `client_id`, `display_name`, `sensors`, `capabilities_summary` | happy-path |
| Read `prism://config/clients/acme/sensors` with CrowdStrike and Claroty configured | JSON with two sensor entries; API URL shows host only (e.g., `api.crowdstrike.com`), no full URL or credentials | happy-path |
| Read `prism://config/clients/nonexistent/sensors` | MCP resource error: "Client 'nonexistent' not found" | error |
| Read `prism://config/clients` with zero clients configured | `[]` (empty array) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-050 | `render_sensor_inventory_resource()` given a ClientSensorConfig containing full API base URLs and credential values produces a response JSON where: (a) no string matching an API key pattern appears; (b) the API base URL field contains only the host+port component | proptest |

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

- `architecture/api-surface.md#configuration-state-resources` — `prism://config/clients` and `prism://config/clients/{client_id}/sensors` are Configuration State resources
- `architecture/api-surface.md#event-feed-resources` — `prism://sensors/health` is an Event Feed resource

## Story Anchor

S-5.03 — Resources and Prompts
