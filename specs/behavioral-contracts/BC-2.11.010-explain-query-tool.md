---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: 2026-05-07
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.010: `explain_query` MCP Tool

## Description

The `explain_query` tool parses and plans a PrismQL query without executing it — no sensor API calls are made, no data is materialized. It returns the detected mode, alias expansions, field resolution, per-sensor push-down filters (predicate strings; sensor-native translation deferred to S-3.X via TD-S303-PUSH-DOWN-TRANSLATION-001), post-materialization operations, and a cost estimate (per-sensor latency history, API call count, rate limit headroom). Syntactic security limits are enforced in explain mode; the materialization limit produces a warning (not an error) since no actual fetch occurs. An audit entry is emitted for every invocation per DI-004.

## Postconditions
- The query is parsed and planned but NOT executed (no sensor API calls, no materialization)
- Response includes:
  - `parsed_mode`: the detected query mode (`filter`, `sql`, or `pipe`)
  - `original_query`: the raw query string as provided
  - `alias_expansion`: map of alias names to their expanded definitions (if any aliases were used)
  - `expanded_query`: the query after all alias expansion
  - `field_resolution`: map of field names used in the query to their OCSF paths and resolution method (direct, alias, virtual)
  - `execution_plan`:
    - `sensors_to_query`: list of sensors that would be queried; each entry (`ExplainSource`) contains:
      - `source_ref`: the data source reference string
      - `sensor_type`: the sensor type identifier
      - `api_filters_pushed`: predicate strings that would be pushed to the sensor API (sensor-native translation deferred to S-3.X via TD-S303-PUSH-DOWN-TRANSLATION-001; currently emits PrismQL-native generic predicate strings)
      - `post_filter_predicates`: predicates applied post-materialization for this source
      - `estimated_row_count`: optional row-count hint (populated from `count_hint()` if the adapter provides one)
    - `post_fetch_operations`: list of operations to be applied post-materialization across all sources (filter, group by, sort, limit, etc.)
    - `clients_to_query`: list of all client IDs in scope for this query (populated from the `ClientRegistry` via `resolve_clients()`; empty when no registry is wired; no fan-out occurs — AC-5)
  - `estimated_cost`: structured cost estimate including:
    - Per-sensor estimated latency (based on historical API call timings, rolling average)
    - Estimated API call count per sensor (based on expected pagination depth)
    - Rate limit headroom per sensor (remaining calls within the current rate limit window)
    - Human-readable summary combining the above into an actionable estimate
    - This enables the query planner (and the AI agent via `explain_query`) to make informed decisions about query scope before committing to execution
- Parse errors, alias resolution errors, and field resolution errors are returned as structured errors (same format as the `query` tool)
- Security limit validation runs (the query must pass all limits even in explain mode)

## Preconditions
- The `explain_query` MCP tool is invoked with:
  - `query`: PrismQL query string (required)
  - `clients`: optional client scoping (same as `query` tool)
  - `sensors`: optional sensor scoping (same as `query` tool)
  - `sources`: optional data source scoping (same as `query` tool)
- Implementation note (I-LOCAL-002): the `sensors` scope filter applies only to external sensor sources (SourceRefKind::External). Non-external sources (internal, composite) are sensor-agnostic and are dropped when a sensor scope filter is active, because they cannot be validated against any specific sensor type. This behavior is intentional.

## Invariants
- DI-019: Syntactic security limits (query length, nesting depth, pipe stages) apply in explain mode and cause errors if exceeded. The materialization limit (10K records) is an estimation-only warning in explain mode, not a failure, since no actual materialization occurs.
- No sensor API calls are made; no data is fetched or materialized
- DI-004: An audit entry IS emitted for `explain_query` invocations. Although it is a read-only tool, it is an MCP tool invocation and must be audited for SOC 2 compliance. The audit entry records the query, scoping parameters, and the explain result summary.
- DataFusion logical plan elision (I-LOCAL-001): the S-3.03 implementation uses an AST-derived `post_fetch_operations` list rather than invoking DataFusion `SessionContext::create_logical_plan()` against schema-only MemTables. This satisfies the BC postcondition (showing post-materialization operations without executing the query). Full DataFusion plan visibility is deferred to TD-S303-DATAFUSION-PLAN-001 (a later story) if it becomes user-required.
- Sensor-native filter translation deferral (INV-PUSH-001): `api_filters_pushed` currently emits PrismQL-native generic predicate strings (e.g. `"severity = 'critical'"`) rather than sensor-native syntax (FQL, KQL, etc.). Sensor-native translation is deferred to a later story via TD-S303-PUSH-DOWN-TRANSLATION-001. The BC postcondition is met — push-down filter visibility is present; sensor-native rendering is a future enhancement.

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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `explain_query(query="severity = 'critical'", sensors=["crowdstrike"])` | `parsed_mode: "filter"`, push-down shown as PrismQL-native predicate strings (sensor-native translation deferred — see INV-PUSH-001 / TD-S303-PUSH-DOWN-TRANSLATION-001) | happy-path |
| `explain_query(query="SELECT count(*) FROM events GROUP BY _sensor")` | `parsed_mode: "sql"`, aggregation plan shown | happy-path |
| `explain_query(query="<syntactically invalid>")` | Same parse error as `query` tool | error |
| `explain_query(query="...")` where estimated record count > 10K | Succeeds; `estimated_cost` includes 10K warning | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-004, DI-019 |
| Related BCs | BC-2.11.007 (push-down visible in explain output) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | S-3.03-adversary-local-pass4 | 2026-05-07 | implementer | C-2: Canonical test vector corrected — "push-down shown as FQL" replaced with "push-down shown as PrismQL-native predicate strings (sensor-native translation deferred — see INV-PUSH-001 / TD-S303-PUSH-DOWN-TRANSLATION-001)". I-2: DI-PUSH-001 renamed to INV-PUSH-001 (BC-local prefix) across all occurrences to avoid orphan-DI detector false positives. I-3: v1.5 note added — softening propagation was incomplete at v1.5; v1.6 closes the canonical test vector site. |
| 1.5 | S-3.03-adversary-local-pass3 | 2026-05-07 | implementer | I-LOCAL-PASS3-2: Restructured `execution_plan` postcondition tree — `api_filters_pushed` moved from sibling of `sensors_to_query` to nested inside each `ExplainSource` entry (correct struct layout); added `source_ref`, `sensor_type`, `post_filter_predicates`, `estimated_row_count` sub-fields. I-LOCAL-PASS3-3: Softened "sensor-native syntax" claim in Description and postcondition to "predicate strings (sensor-native translation deferred to S-3.X via TD-S303-PUSH-DOWN-TRANSLATION-001)"; added INV-PUSH-001 invariant documenting the deferral explicitly. Filed TD-S303-PUSH-DOWN-TRANSLATION-001. NOTE: softening propagation was incomplete at v1.5 — the Canonical Test Vectors section still referenced "push-down shown as FQL"; v1.6 closes that remaining site. |
| 1.4 | S-3.03-adversary-local-pass1 | 2026-05-07 | implementer | Added `clients_to_query: Vec<OrgSlug>` postcondition to ExecutionPlan (C-LOCAL-002 / AC-5). Added Preconditions with I-LOCAL-002 sensor-scope filter note. Added DataFusion plan elision invariant (I-LOCAL-001 / TD-S303-DATAFUSION-PLAN-001). |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
