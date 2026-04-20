---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[pending-recompute]"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation

## Description

Config-driven sensor tables may require multi-step fetch sequences where each step's
output feeds into subsequent steps via variable interpolation. Steps execute sequentially
in spec-declared order; variables produced by a step are available to all later steps
but not earlier ones. When a variable resolves to an array, the step is executed in
batches (default batch size 100), with all batches concatenated into a single result set.

The final step's response records are collected into an Arrow RecordBatch using the
table's column definitions. The 10K materialization limit (DI-019) applies to the
final collected records, not intermediate step results. Rate limit hints from the
`SensorSpec` apply between API calls.

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
  - `variables_produced`: a list of variable names that downstream steps can reference
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

## Invariants
- Steps execute sequentially; no parallel step execution within a single table fetch (simplifies variable scoping)
- The 10K materialization limit (DI-019) applies to the final collected records, not to intermediate step results
- No BC-specific invariants beyond DI-019 and rate-limit behavior defined above.

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-010` | Variable interpolation failure at runtime (variable exists but field path does not match response structure) | `E-SPEC-010` with the step name, variable reference, and actual response structure hint |
| (sensor_error) | HTTP error on any step | Pipeline aborts for the current client; `sensor_error` reported in query response (consistent with BC-2.01.010 partial failure handling) |
| (no error) | Empty response from a non-final step | Subsequent steps receive empty arrays; produces zero results (not an error) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| Fan-out batch of 250 IDs | 250 IDs; batch_size=100 | 3 executions: 100, 100, 50; all results concatenated |
| Empty non-final step | step 1 returns empty; step 2 needs step 1 output | Step 2 receives empty variable; produces zero records |
| Pagination | step has cursor pagination | Iterates pages until cursor=null or empty page |
| Rate limiting | spec declares 5 req/s | 200ms delay between requests; burst allowed |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — 2-step | step 1 fetches IDs; step 2 fetches details by ID | Final RecordBatch with detail records |
| Fan-out | step 2 depends on array from step 1 (250 items) | 3 batched requests; all results concatenated |
| HTTP error on step 1 | step 1 returns 500 | Pipeline aborts; sensor_error in response |
| Empty step 1 | step 1 returns zero records | Step 2 produces zero records; no error |
| Interpolation failure | step 2 references `${step1.missing_field}` | `E-SPEC-010` with step name and field path |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify fan-out batch concatenation |
| (placeholder) | VP to be assigned — verify variable scoping (forward references rejected at validation) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-019 |
| Related BCs | BC-2.16.001 (spec loading), BC-2.01.014 (exponential backoff on HTTP calls), BC-2.01.010 (partial failure) |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
