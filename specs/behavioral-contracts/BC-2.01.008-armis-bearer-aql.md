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

# BC-2.01.008: Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback

## Preconditions
- Armis Centrix sensor is configured with a static API key (bearer token)
- The target data source is one of the 7 Armis sources (alerts, activities, audit_logs, risk_factors, connections, devices, vulnerabilities)

## Postconditions
- All Armis API requests use the bearer token via the SDK's GetSearch endpoint
- Queries are expressed in AQL (Armis Query Language) and forwarded to the GetSearch API
- Timestamp extraction uses the per-source fallback chain (1-3 candidate fields)
- ID extraction uses the per-source fallback chain (2-4 candidate fields)
- Cursor is a `(Timestamp, TypeSpecificID)` 2-tuple

## Invariants
- DI-012: Sealed auth trait
- DI-001: Cursor forward progress

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Armis API key rejected (HTTP 401) | `category: "authentication"`, suggestion: "Verify Armis API key in credential store" |
| `PrismError::Sensor` | AQL syntax error (HTTP 400) | `category: "api_contract"`, include the AQL query and Armis error message in the structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-013 | Record has no valid timestamp in any of the fallback fields | Warning logged identifying the record; record included in response but with null cursor contribution; does not advance cursor |
| EC-01-011 | All records in a page lack valid timestamps | Page treated as having no cursor advancement; pagination halts to prevent infinite loops |
| EC-01-012 | ID fallback chain exhausted (no valid ID field found) | Record logged as warning and skipped; cursor does not account for this record |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-012 |
| Priority | P0 |
