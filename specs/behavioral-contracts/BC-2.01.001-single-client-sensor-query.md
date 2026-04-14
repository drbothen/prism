---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.001: Single-Client Sensor Query Returns Scoped Results

## Preconditions
- A valid `client_id` (matching `[a-zA-Z0-9_-]+`) is provided in the MCP tool call
- The client exists in TOML configuration with at least one enabled sensor
- Credentials for the target sensor are available in the credential store

## Postconditions
- The response contains only records belonging to the specified `client_id`
- Each result item includes `source_sensor` and `record_type` fields
- Response metadata includes `trust_level: "untrusted_external"`
- Pagination metadata (`cursor`, `has_more`, `total_count`) is present in `_meta`

## Invariants
- DI-008: Client data separation -- no records from other clients appear in the response
- DI-004: Audit completeness -- exactly one AuditEntry is emitted for the tool invocation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` contains characters outside `[a-zA-Z0-9_-]` | Structured error with rejected value and allowed pattern |
| `PrismError::Config` | `client_id` not found in TOML configuration | Structured error: "Client '{id}' not found in configuration" with suggestion to check config |
| `PrismError::Credential` | Credentials for the target sensor cannot be resolved | Structured error with `category: "authentication"` and suggestion to verify credential store |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-004 | Client configured with zero sensors | Empty result set with metadata message "Client '{id}' has no sensors configured"; not an error |
| EC-01-001 | Sensor is configured but `enabled: false` | Sensor excluded from query; if no other sensors match, returns empty result set |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
