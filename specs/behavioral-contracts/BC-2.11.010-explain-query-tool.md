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

# BC-2.11.010: `explain_query` MCP Tool

## Preconditions
- The `explain_query` MCP tool is invoked with:
  - `query`: AxiQL query string (required)
  - `clients`: optional client scoping (same as `query` tool)
  - `sensors`: optional sensor scoping (same as `query` tool)

## Postconditions
- The query is parsed and planned but NOT executed (no sensor API calls, no materialization)
- Response includes:
  - `parsed_mode`: the detected query mode (`filter`, `sql`, or `pipe`)
  - `original_query`: the raw query string as provided
  - `alias_expansion`: map of alias names to their expanded definitions (if any aliases were used)
  - `expanded_query`: the query after all alias expansion
  - `field_resolution`: map of field names used in the query to their OCSF paths and resolution method (direct, alias, virtual)
  - `execution_plan`:
    - `sensors_to_query`: list of sensors that would be queried
    - `api_filters_pushed`: per-sensor translated push-down filters in sensor-native syntax
    - `post_fetch_operations`: list of operations to be applied post-materialization (filter, group by, sort, limit, etc.)
  - `estimated_cost`: human-readable estimate of API calls and expected latency
- Parse errors, alias resolution errors, and field resolution errors are returned as structured errors (same format as the `query` tool)
- Security limit validation runs (the query must pass all limits even in explain mode)

## Invariants
- DI-019: Security limits apply even in explain mode (prevents using explain to bypass limits)
- No sensor API calls are made; no data is fetched or materialized
- No audit entry is emitted for explain (it is a read-only, no-side-effect operation)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::QueryParse` | Query cannot be parsed | Same structured error as `query` tool |
| `PrismError::AliasNotFound` | Unknown alias reference | Same structured error as `query` tool |
| `PrismError::QuerySecurityLimit` | Expanded query exceeds security limits | Same structured error as `query` tool |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-025 | Explain a query that would exceed materialization limit | Explain succeeds; `estimated_cost` notes the likely record count exceeding 10K |
| EC-11-026 | Explain a query with invalid field names | Error with `similar_fields` suggestions |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Related BCs | BC-2.11.007 (push-down visible in explain output) |
| Priority | P0 |
