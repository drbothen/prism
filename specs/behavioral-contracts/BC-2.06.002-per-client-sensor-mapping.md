---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
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
input-hash: "[pending-recompute]"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.002: Per-Client Sensor Mapping from TOML Configuration

## Description

Each client in TOML can define a `[clients.{id}.sensors]` section mapping sensor IDs to
`SensorConfig` structs. Valid sensor IDs are the four supported sensors: `crowdstrike`,
`cyberint`, `claroty`, `armis`. Each `SensorConfig` includes `api_base` (valid URL),
`credential_ref` (reference to credential store), `enabled` (boolean, default `true`),
and `data_sources`. The `(client_id, sensor_id)` pair must be unique within the config.

Disabled sensors (`enabled: false`) are loaded but excluded from query and health check
operations, allowing operators to temporarily disable a sensor without removing its config.

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
| EC-06-002 | Client has all available sensors configured | Valid configuration; all sensor adapters (spec-driven) are instantiated for that client |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.002.

| Scenario | Input | Expected Result |
|----------|-------|----------------|
| Valid sensor | `[clients.acme.sensors.crowdstrike]` with valid URL and cred ref | `SensorConfig` loaded; `enabled: true` default |
| Unknown sensor | `sensor_id: "splunk"` | Validation error naming supported sensors |
| Invalid URL | `api_base = "not-a-url"` | Validation error with path and value |
| Disabled sensor | `enabled = false` | Loaded; excluded from query routing |
| Duplicate sensor | Two `crowdstrike` entries for same client | Validation error: "Duplicate sensor 'crowdstrike' for client 'acme'" |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify per-client sensor mapping. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
