---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Config-Driven Adapters & Hot Reload"
capability: "CAP-029"
---

# BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation

## Preconditions
- A spec-driven table has been registered (BC-2.16.001) with one or more `FetchStep` entries in its `steps` array
- A query targeting this table has been dispatched by the query engine (CAP-015)
- Credentials for the sensor's `auth_type` have been resolved for the target `client_id`

## Postconditions
- Steps are executed sequentially in the order defined in the spec's `[[table.steps]]` array
- Each step produces an HTTP request using:
  - `method`: GET or POST as declared
  - `path_template`: interpolated against variables from prior steps and query parameters (e.g., `${query_ids.resource_ids}` resolves to the `resource_ids` field from the step named `query_ids`)
  - `body_template` (if present): interpolated identically to `path_template`, then sent as the request body
- Variable interpolation uses the syntax `${step_name.field}` where:
  - `step_name` is the `name` attribute of a prior step
  - `field` is a JSONPath-like dot-notation path into that step's response (e.g., `${query_ids.resources[*].id}` extracts all IDs from the array)
  - Array-valued variables trigger fan-out: the step is executed once per batch of values (batch size configurable per step, default 100)
- Each step's response is parsed according to:
  - `response_path`: a JSONPath expression pointing to the results array in the JSON response (e.g., `$.resources`, `$.data.items`)
  - `pagination_cursor_path` (if present): a JSONPath expression pointing to the pagination cursor in the response for automatic page iteration
  - `variables_produced`: a list of variable names that downstream steps can reference via `${this_step.variable_name}`
- Pagination within a step follows the sensor spec's declared pagination config (cursor/offset/none), iterating until the API returns an empty page or the cursor is null
- The final step's response records are collected into an Arrow RecordBatch using the table's column definitions
- Rate limit hints from the `SensorSpec` are applied between API calls: inter-request delay = `1 / requests_per_second`, with burst allowance from `burst_size`

## Variable Scope and Lifetime
- Variables produced by a step are available to all subsequent steps but not to prior steps
- Variables from the most recent execution of a step overwrite previous values (relevant when a step is re-executed due to fan-out)
- Query-time variables are available to all steps: `${query.client_id}`, `${query.filter.*}` (push-down filter values extracted by the query planner)

## Fan-Out Behavior
- When a variable interpolation resolves to an array, the step is executed in batches
- Batch size is configurable per step via `fan_out_batch_size` (default 100)
- Fan-out results are concatenated into a single result set for the step
- Fan-out respects rate limit hints — each batch counts as a separate request

## Error Handling
- HTTP error on any step: the pipeline aborts for the current client and reports a `sensor_error` in the query response (consistent with BC-2.01.010 partial failure handling)
- Variable interpolation failure at runtime (variable exists but field path does not match response structure): `E-SPEC-010` with the step name, variable reference, and actual response structure hint
- Empty response from a non-final step: subsequent steps that depend on its variables receive empty arrays, effectively producing zero results (not an error)

## Invariants
- Steps execute sequentially; no parallel step execution within a single table fetch (simplifies variable scoping)
- The 10K materialization limit (DI-019) applies to the final collected records, not to intermediate step results

## Traces
- CAP-029 (Config-Driven Sensor Adapters)
- BC-2.01.014 (Exponential backoff applies to individual HTTP calls within the pipeline)
- DI-019 (Materialization limit)
