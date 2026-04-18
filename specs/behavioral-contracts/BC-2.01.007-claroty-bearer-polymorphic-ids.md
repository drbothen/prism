---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Adapters"
capability: "CAP-001"
---

# BC-2.01.007: Claroty Bearer Token Auth with Polymorphic ID Handling

## Preconditions
- Claroty xDome sensor is configured with a bearer token credential
- The target data source is one of the 9 Claroty endpoints (alerts, ot_activity_events, audit_logs, device_alert_relations, device_vulnerability_relations, servers, sites, devices, vulnerabilities)

## Postconditions
- All Claroty API requests use the bearer token in the Authorization header
- Claroty's POST-for-read pattern is followed (POST requests used for read operations)
- Polymorphic ID fields (JSON number `12345` or JSON string `"12345"`) are normalized to a `PolymorphicId` enum
- Cursor arity matches the source: 2-tuple for most sources, 3-tuple for sources requiring it

## Invariants
- DI-012: Sealed auth trait
- DI-001: Cursor forward progress across polymorphic ID types

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Bearer token rejected (HTTP 401) | `category: "authentication"`, suggestion: "Verify Claroty API token in credential store" |
| `PrismError::Sensor` | Claroty API returns HTTP 500 during paginated query | Return pages already fetched as partial result; cursor advances to last successful page |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-010 | Same ID appears as JSON number `12345` and string `"12345"` in the same response | `PolymorphicId` enum normalizes both to string for cursor comparison and deduplication; both representations treated as equivalent |
| EC-01-010 | Claroty API changes field name or nesting structure in a minor version | Adapter logs unmapped fields as warnings; preserves them in `raw_extensions`; does not fail on unknown fields |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-012 |
| Priority | P0 |
