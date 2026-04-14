---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Client Configuration"
capability: "CAP-009"
---

# BC-2.06.002: Per-Client Sensor Mapping from TOML Configuration

## Preconditions
- TOML configuration has been loaded successfully
- A `[clients.{id}.sensors]` section exists for a client

## Postconditions
- Each sensor entry under `clients.{id}.sensors.{sensor_id}` is deserialized into a `SensorConfig` with:
  - `sensor_id` (one of: `crowdstrike`, `cyberint`, `claroty`, `armis`)
  - `api_base` (valid URL)
  - `credential_ref` (reference to a credential in the credential store)
  - `enabled` (boolean, defaults to `true`)
  - `data_sources` (list of source IDs specific to the sensor)
- The `(client_id, sensor_id)` pair is unique -- no client has duplicate sensor entries
- Disabled sensors (`enabled: false`) are loaded but excluded from query and health check operations

## Invariants
- DI-008: Client data separation -- sensor configs are scoped per client

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Config` | Unknown `sensor_id` value (not one of the four supported sensors) | Validation error: "Unknown sensor '{value}' for client '{id}'. Supported: crowdstrike, cyberint, claroty, armis" |
| `PrismError::Config` | `api_base` is not a valid URL | Validation error: "Invalid API base URL for clients.{id}.sensors.{sensor}: '{value}'" |
| `PrismError::Config` | Duplicate `sensor_id` for the same client | Validation error: "Duplicate sensor '{sensor}' for client '{id}'" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-004 | Client configured with zero sensors (empty `[clients.acme.sensors]`) | Valid configuration; client loads with an empty sensor map; queries return empty results |
| EC-06-002 | Client has all four sensors configured | Valid configuration; all four sensor adapters are instantiated for that client |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |
