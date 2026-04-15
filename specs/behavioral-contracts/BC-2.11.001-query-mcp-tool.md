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

# BC-2.11.001: `query` MCP Tool Accepts Scoping + AxiQL Query String

## Preconditions
- The `query` MCP tool is invoked with at minimum a `query` string parameter (required)
- Optional scoping parameters: `clients` (array of client IDs or null for all), `sensors` (array of sensor types or null for all), `time_range` (relative or absolute), `limit` (max results, default 25, max 1000)
- At least one client with at least one enabled sensor exists in configuration

## Postconditions
- The query string is parsed by the Chumsky parser (auto-detecting filter/SQL/pipe mode)
- Scoping parameters (`clients`, `sensors`) are resolved to concrete client/sensor combinations from config
- If the query contains `client_id`, `sensor`, or `source` predicates, they are intersected with tool parameters (narrowing, never widening)
- Fan-out to sensor APIs occurs for all resolved (client, sensor) combinations
- Sensor responses are normalized to OCSF, materialized as Arrow RecordBatch, registered as DataFusion MemTable
- Query is executed via DataFusion; results returned as OCSF-normalized events
- Response includes `query_context` with: `original_query`, `expanded_query` (after alias resolution), `clients_queried`, `sensors_queried`, `time_range_applied`, `total_results`, `returned_results`, `is_truncated`, `next_cursor`, `execution_time_ms`
- Response includes `sensor_errors` array for any sensors that failed (partial results are valid)
- Response includes `events` array of OCSF-normalized results with `trust_level: "untrusted_external"`

## Invariants
- DI-019: Query security limits enforced (64KB, nesting, pipe stages, 10K records, 30s timeout)
- DI-008: Client data separation -- cross-client results include per-event `client_id` provenance
- DI-004: Audit completeness -- exactly one AuditEntry emitted for the tool invocation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::QueryParse` | AxiQL query string cannot be parsed | Structured error with position, context, suggestion, and syntax help |
| `PrismError::QueryScopeTooBoard` | Materialization would exceed 10K records | Structured error with estimated counts and narrowing suggestions (DEC-023) |
| `PrismError::QueryTimeout` | Execution exceeds 30 seconds | Structured error with timeout duration and narrowing suggestions (DEC-026) |
| `PrismError::InvalidInput` | `clients` array contains invalid client ID | Structured error with rejected value |
| `PrismError::Config` | No matching clients/sensors found for scoping parameters | Structured error listing configured clients/sensors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-022 | All sensor API calls return empty results | Empty result set with `total_results: 0`, not an error |
| DEC-023 | Fan-out produces more than 10K records | Error with narrowing suggestions before materialization begins |
| DEC-026 | Query execution exceeds 30s timeout | Error, no partial results |
| EC-11-001 | `clients: ["acme"]` but query contains `client_id = "globex"` | Intersection is empty; return empty result set (not error) with metadata explaining intersection was empty |
| EC-11-002 | All sensors error for a single client in cross-client query | Client's results omitted; other clients' results returned; failed client listed in `sensor_errors` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-004, DI-008, DI-019 |
| L2 Edge Cases | DEC-022, DEC-023, DEC-026 |
| Priority | P0 |
