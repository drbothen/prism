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
- Optional scoping parameters: `clients` (array of client IDs or null for all), `sensors` (array of sensor types or null for all), `sources` (array of data source names or null for all), `time_range` (relative or absolute), `limit` (max results, default 25, max 1000), `force_refresh` (boolean, default false -- bypass response cache)
- At least one client with at least one enabled sensor exists in configuration

## Postconditions
- The query string is parsed by the Chumsky parser (auto-detecting filter/SQL/pipe mode)
- Scoping parameters (`clients`, `sensors`) are resolved to concrete client/sensor combinations from config
- If the query contains `client_id`, `sensor`, or `source` predicates, they are intersected with tool parameters (narrowing, never widening)
- Fan-out to sensor APIs occurs for all resolved (client, sensor) combinations
- Sensor responses are normalized to OCSF, materialized as Arrow RecordBatch, registered as DataFusion MemTable
- Query is executed via DataFusion; results returned as OCSF-normalized events
- **No cross-call pagination for query results.** The ephemeral model means the SessionContext (and all materialized data) cannot be held across calls. Each `query` call re-materializes from scratch (the response cache mitigates re-fetch cost). The `limit` parameter truncates results. If more results exist than `limit`, the response includes `is_truncated: true` and `total_available` (count of all matching records before truncation). The user narrows their query or increases `limit` (up to 1000) to see more results. There is no cursor or offset-based pagination for query results.
- **Dual limit semantics (tool-level vs SQL-level).** Tool-level `limit` is applied after DataFusion execution (which may include SQL-level LIMIT). `total_available` reflects count after DataFusion execution but before tool-level truncation. SQL LIMIT reduces `total_available`; tool-level limit causes `is_truncated: true`.
- Response includes `query_context` with: `original_query`, `expanded_query` (after alias resolution), `clients_queried`, `sensors_queried`, `time_range_applied`, `total_available` (total matching records before limit), `returned_results` (actual count returned, <= limit), `is_truncated` (true if total_available > returned_results), `execution_time_ms`
- Response includes `sensor_errors` array for any sensors that failed (partial results are valid)
- Response includes `events` array of OCSF-normalized results with `trust_level: "untrusted_external"`

## Invariants
- DI-019: Query security limits enforced (64KB, nesting, pipe stages, 10K records, 30s timeout)
- DI-008: Client data separation -- cross-client results include per-event `client_id` provenance
- DI-004: Audit completeness -- exactly one AuditEntry emitted for the tool invocation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | AxiQL query string cannot be parsed | Structured error with position, context, suggestion, and syntax help |
| `E-QUERY-006` | Materialization would exceed 10K records | Structured error with estimated counts and narrowing suggestions (DEC-023) |
| `E-QUERY-004` | Execution exceeds 30 seconds | Structured error with timeout duration and narrowing suggestions (DEC-026) |
| `E-MCP-004` | `clients` array contains invalid client ID | Structured error with rejected value |
| `E-CFG-001` | No matching clients/sensors found for scoping parameters | Structured error listing configured clients/sensors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-022 | All sensor API calls return empty results | Empty result set with `total_results: 0`, not an error |
| DEC-023 | Fan-out produces more than 10K records | Fetch aborted when streaming counter hits 10K; error with per-sensor counts and narrowing suggestions |
| DEC-026 | Query execution exceeds 30s timeout | Error, no partial results |
| EC-11-001 | `clients: ["acme"]` but query contains `client_id = "globex"` | Intersection is empty; return empty result set (not error) with metadata explaining intersection was empty |
| EC-11-002 | All sensors error for a single client in cross-client query | Client's results omitted; other clients' results returned; failed client listed in `sensor_errors` |
| EC-11-032 | Query matches 500 records but `limit` is 25 | Returns 25 records with `is_truncated: true`, `total_available: 500`. User can re-query with `limit: 500` or narrow the query. No cross-call pagination state is held. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-004, DI-008, DI-019 |
| L2 Edge Cases | DEC-022, DEC-023, DEC-026 |
| Priority | P0 |
