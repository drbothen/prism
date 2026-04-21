---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
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
input-hash: "8bd996e"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.007: Sensor Filter Push-Down

## Description

Push-down translates WHERE predicates from the PrismQL AST into sensor-native API filter syntax (CrowdStrike FQL, Cyberint JSON body, Claroty POST arrays, Armis AQL) to minimize data transferred from sensor APIs. Each adapter column declares a push-down capability option (REQUIRED/INDEX/ADDITIONAL/OPTIMIZED/DEFAULT); only REQUIRED, INDEX, and ADDITIONAL columns are pushed down. Predicates on OPTIMIZED/DEFAULT columns are post-filtered by DataFusion after materialization. Column pruning (passing `columns_used` to adapters that support field selection) further reduces payload. Push-down is an optimization only — query correctness is identical whether push-down occurs or not. REQUIRED columns enforce DI-021: queries that omit a REQUIRED column are rejected before any API calls.

## Preconditions
- An PrismQL query has been parsed into an AST with WHERE predicates
- The QueryPlan is being constructed with resolved sensor targets

## Postconditions

### Column Push-Down Capability Taxonomy

Each sensor adapter column declares its push-down capability using the following taxonomy (inspired by osquery's ColumnOptions pattern, adapted for remote API-backed data sources):

| Option | Meaning | Query Planner Behavior | Adapter Contract |
|--------|---------|----------------------|------------------|
| `REQUIRED` | The sensor API **requires** this parameter; queries cannot execute without it | Query rejected with `E-QUERY-009` if column is not constrained in WHERE clause. Rejection occurs before any API calls. Error message lists the required columns and example usage. | Adapter MUST have this constraint to generate any results. Prevents full-scan of unbounded remote APIs. |
| `INDEX` | The sensor API supports this as a native filter parameter | Constraint is pushed down to the sensor API. Cost estimation favors queries with INDEX constraints. | Adapter SHOULD use this constraint for efficient lookup. Improves performance but is not mandatory. |
| `ADDITIONAL` | The sensor API uses this for secondary/supplemental filtering | Constraint is pushed down when present. Does not affect cost estimation as strongly as INDEX. | Adapter uses this to request additional or different data from the API (e.g., include resolved alerts when `status = resolved` is constrained). |
| `OPTIMIZED` | Prism can optimize this locally but the sensor API does not support it as a filter | Constraint is NOT pushed down. Applied as a post-filter by DataFusion. Marked in `explain_query` as locally-optimized. | Adapter ignores this constraint. DataFusion handles filtering after materialization. |
| `DEFAULT` | No special push-down behavior | Constraint is NOT pushed down. Applied as a post-filter by DataFusion. | Adapter does not receive this constraint. Standard post-materialization filtering. |

Column options are declared per-column, per-sensor-adapter in the adapter's schema definition. The same OCSF field may have different options across sensors (e.g., `severity` may be INDEX on CrowdStrike but DEFAULT on Cyberint).

### Predicate Classification

- Each WHERE predicate is classified as either push-down-capable or post-filter for each target sensor:
  - **Push-down capable**: The predicate references a field with a known sensor-native mapping AND the sensor API supports the comparison operator AND the column is declared as REQUIRED, INDEX, or ADDITIONAL
  - **Post-filter**: The predicate references an OCSF-only field (exists only after normalization), or the sensor API does not support the operator, or the column is DEFAULT or OPTIMIZED
- Push-down filters are translated to sensor-native query syntax:
  - CrowdStrike: FQL filter syntax (e.g., `severity:>3+created_date:>'2026-04-01'`)
  - Cyberint: JSON body parameters on POST
  - Claroty xDome: POST body filter arrays
  - Armis: AQL WHERE clauses (most capable for push-down)
- Remaining post-filter predicates are applied by DataFusion over the materialized OCSF table
- The push-down classification is visible in `explain_query` output (see BC-2.11.010)
- Push-down reduces the volume of data fetched from sensor APIs, improving performance and reducing materialization size

### Column Pruning

The query planner tracks which columns are referenced in the query (SELECT list + WHERE + ORDER BY + GROUP BY). This column usage set is passed to the sensor adapter, which uses it to populate API `fields`/`select` parameters where supported, minimizing response payload. Specifically:

- The planner computes a `columns_used: HashSet<String>` from all column references in the query AST
- This set is included in the `QueryContext` passed to each sensor adapter
- Adapters that support field selection (e.g., CrowdStrike's `fields` parameter, Armis's `fields` in AQL) translate the set to API-specific field selection syntax
- Adapters that do not support field selection ignore the set and return full records
- Column pruning is an optimization only; it does not affect query correctness

## Invariants
- Push-down is an optimization only; the query result must be identical whether or not push-down occurs
- A predicate that cannot be pushed down is never silently dropped -- it is always applied as a post-filter
- Time range push-down is always attempted (all initial sensors support time-based filtering; spec-driven sensors declare push-down support per column via `options: Index`)
- Push-down filter translation produces a canonical form (sorted parameter keys, normalized timestamp ISO8601 format, lowercase string values where applicable) before the result is used as cache key input. This ensures that semantically equivalent push-down filters produce identical cache keys regardless of the original predicate ordering in the PrismQL query.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-009` | Query does not constrain a REQUIRED column for a target sensor | Query rejected before any API calls. Structured error includes: the sensor name, the list of REQUIRED columns, and example WHERE clause syntax. See DI-021. |
| N/A | Predicate cannot be pushed down | Normal path -- predicate is applied post-materialization via DataFusion |
| N/A | Push-down filter translation fails | Log warning, fall back to post-filter for that predicate |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-018 | Query predicate uses OCSF field `device.ip` which maps to different native fields per sensor | Cannot push down uniformly; applied as post-filter. Each sensor may partially push down the corresponding native field if the mapping is known. |
| EC-11-019 | Armis AQL supports the full predicate natively | Push down the entire predicate to Armis; no post-filter needed for Armis. Other sensors may still need post-filtering. |
| EC-11-020 | `severity >= "high"` pushed down to CrowdStrike (severity 1-5 scale) | Translate OCSF severity to CrowdStrike native scale before push-down: `"high"` -> CrowdStrike severity >= 4 |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query with `severity = 'critical'` against CrowdStrike (INDEX column) | Push-down generated: `severity:5`; no post-filter for severity | happy-path |
| Query missing REQUIRED column `customer_id` for Cyberint | `Err(E-QUERY-009)` before any API calls | error |
| Query with `device.hostname = 'srv01'` (OPTIMIZED on all sensors) | Post-filter only; DataFusion filters after materialization | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-031 | Required column enforcement: rejects unconstrained | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-021 |
| Related BCs | BC-2.11.010 (explain_query shows push-down plan) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
