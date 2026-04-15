---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "Query Engine & Aliases"
capability: "CAP-015"
---

# BC-2.11.005: Ephemeral Materialization -- Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable

## Preconditions
- A `QueryPlan` has been produced with resolved clients, sensors, push-down filters, and post-filters
- Sensor credentials are available for all resolved (client, sensor) combinations
- The query has passed all security limit pre-checks

## Postconditions
- Fan-out to sensor APIs occurs concurrently for all resolved (client, sensor, source) combinations
  - Each sensor fetch checks the response cache (CAP-014) first; cache hits skip the API call
  - Cache misses trigger sensor API calls with push-down filters translated to sensor-native syntax
  - Max fan-out concurrency bounded (configurable, default 10 parallel sensor calls)
- Sensor responses are normalized to OCSF via the OCSF normalizer (CAP-003)
- OCSF-normalized records are converted to Arrow RecordBatch format:
  - Hot OCSF fields (severity, timestamp, device.ip, device.hostname, src_endpoint, dst_endpoint, etc.) as flat top-level Arrow columns
  - Full event serialized as JSON in an `event_data` string column for `json_extract_string()` UDF access
  - Virtual fields (`sensor`, `client_id`, `source`) injected as additional columns
- Total record count is checked against 10K materialization limit before Arrow conversion
- RecordBatches are registered as a DataFusion `MemTable` named `events` in a fresh `SessionContext`
- The `SessionContext` (and all materialized data) is dropped when the query tool call returns

## Invariants
- DI-019: Materialization limit of 10K records enforced pre-conversion
- DI-008: Client data separation -- each record includes `client_id` provenance in the materialized table
- The transient memory peak (both DynamicMessage and Arrow representations in memory simultaneously during conversion) is bounded by the 10K record limit

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::QueryScopeTooBoard` | Fan-out would produce more than 10K records | Error before materialization; includes per-sensor estimated counts and narrowing suggestions |
| `PrismError::Sensor` | One or more sensor API calls fail | Partial materialization: successful sensors contribute data; failed sensors listed in `sensor_errors` |
| `PrismError::Credential` | Credentials unavailable for a sensor | Sensor excluded from fan-out; listed in `sensor_errors` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-022 | All sensor API calls return empty | Empty RecordBatch registered; query returns empty result set |
| EC-11-013 | Cache hit for some sensors, cache miss for others | Mix of cached and fresh data is valid; cache hits avoid API calls |
| EC-11-014 | A single sensor returns more than 10K records | Per-sensor API response pagination limits apply; the 10K limit is across all sensors combined |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-008, DI-019 |
| L2 Edge Cases | DEC-022, DEC-023 |
| Related BCs | BC-2.07.003 (cache), BC-2.02.002 (OCSF normalization) |
| Priority | P0 |
