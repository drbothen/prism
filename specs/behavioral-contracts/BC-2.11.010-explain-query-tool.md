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
  - `sources`: optional data source scoping (same as `query` tool)

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
  - `estimated_cost`: structured cost estimate including:
    - Per-sensor estimated latency (based on historical API call timings, rolling average)
    - Estimated API call count per sensor (based on expected pagination depth)
    - Rate limit headroom per sensor (remaining calls within the current rate limit window)
    - Human-readable summary combining the above into an actionable estimate
    - This enables the query planner (and the AI agent via `explain_query`) to make informed decisions about query scope before committing to execution
- Parse errors, alias resolution errors, and field resolution errors are returned as structured errors (same format as the `query` tool)
- Security limit validation runs (the query must pass all limits even in explain mode)

## Invariants
- DI-019: Syntactic security limits (query length, nesting depth, pipe stages) apply in explain mode and cause errors if exceeded. The materialization limit (10K records) is an estimation-only warning in explain mode, not a failure, since no actual materialization occurs.
- No sensor API calls are made; no data is fetched or materialized
- DI-004: An audit entry IS emitted for `explain_query` invocations. Although it is a read-only tool, it is an MCP tool invocation and must be audited for SOC 2 compliance. The audit entry records the query, scoping parameters, and the explain result summary.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | Query cannot be parsed | Same structured error as `query` tool |
| `E-ALIAS-001` | Unknown alias reference | Same structured error as `query` tool |
| `E-QUERY-003` | Expanded query exceeds syntactic security limits (length, nesting, pipe stages) | Same structured error as `query` tool |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-025 | Explain a query that would exceed materialization limit | Explain succeeds (not an error); `estimated_cost` includes a warning that the estimated record count exceeds 10K and the query would fail at execution time |
| EC-11-026 | Explain a query with invalid field names | Error with `similar_fields` suggestions |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-004, DI-019 |
| Related BCs | BC-2.11.007 (push-down visible in explain output) |
| Priority | P0 |
