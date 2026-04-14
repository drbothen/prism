---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: "CAP-008, CAP-009"
---

# BC-2.10.008: MCP Resources for Client List and Sensor Inventory

## Preconditions
- Prism has loaded configuration and initialized all client/sensor mappings
- MCP resources are registered in `resources/list`

## Postconditions
- `prism://clients` resource returns a JSON array of all configured clients with: `client_id`, `display_name`, `sensors` (list of enabled sensor IDs), `capabilities_summary` (count of enabled write capabilities)
- `prism://clients/{client_id}/sensors` resource returns detailed sensor inventory for a specific client: sensor ID, API base URL (redacted to host only), enabled status, configured data sources
- `prism://health/{client_id}` resource returns cached health status (BC-2.08.006)
- Resource content uses `application/json` MIME type
- Resources are read-only and reflect startup-time configuration (no live updates)
- Credential values and full API URLs are never exposed in resource content

## Invariants
- DI-002: Credential isolation -- no credential values in resource responses
- DI-008: Client data separation -- per-client resources scoped by client_id path parameter

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource not found | Invalid `client_id` in URI | MCP resource error: "Client '{id}' not found" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-014 | Zero clients configured | `prism://clients` returns empty array |
| EC-10-015 | Client has sensors configured but all disabled | `prism://clients/{id}/sensors` lists sensors with `enabled: false` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008, CAP-009 |
| L2 Invariants | DI-002, DI-008 |
| Priority | P0 |
