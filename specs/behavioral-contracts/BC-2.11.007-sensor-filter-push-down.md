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

# BC-2.11.007: Sensor Filter Push-Down

## Preconditions
- An AxiQL query has been parsed into an AST with WHERE predicates
- The QueryPlan is being constructed with resolved sensor targets

## Postconditions
- Each WHERE predicate is classified as either push-down-capable or post-filter for each target sensor:
  - **Push-down capable**: The predicate references a field with a known sensor-native mapping AND the sensor API supports the comparison operator
  - **Post-filter**: The predicate references an OCSF-only field (exists only after normalization), or the sensor API does not support the operator
- Push-down filters are translated to sensor-native query syntax:
  - CrowdStrike: FQL filter syntax (e.g., `severity:>3+created_date:>'2026-04-01'`)
  - Cyberint: JSON body parameters on POST
  - Claroty xDome: POST body filter arrays
  - Armis: AQL WHERE clauses (most capable for push-down)
- Remaining post-filter predicates are applied by DataFusion over the materialized OCSF table
- The push-down classification is visible in `explain_query` output (see BC-2.11.010)
- Push-down reduces the volume of data fetched from sensor APIs, improving performance and reducing materialization size

## Invariants
- Push-down is an optimization only; the query result must be identical whether or not push-down occurs
- A predicate that cannot be pushed down is never silently dropped -- it is always applied as a post-filter
- Time range push-down is always attempted (all four sensors support time-based filtering)
- Push-down filter translation produces a canonical form (sorted parameter keys, normalized timestamp ISO8601 format, lowercase string values where applicable) before the result is used as cache key input. This ensures that semantically equivalent push-down filters produce identical cache keys regardless of the original predicate ordering in the AxiQL query.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Predicate cannot be pushed down | Normal path -- predicate is applied post-materialization via DataFusion |
| N/A | Push-down filter translation fails | Log warning, fall back to post-filter for that predicate |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-018 | Query predicate uses OCSF field `device.ip` which maps to different native fields per sensor | Cannot push down uniformly; applied as post-filter. Each sensor may partially push down the corresponding native field if the mapping is known. |
| EC-11-019 | Armis AQL supports the full predicate natively | Push down the entire predicate to Armis; no post-filter needed for Armis. Other sensors may still need post-filtering. |
| EC-11-020 | `severity >= "high"` pushed down to CrowdStrike (severity 1-5 scale) | Translate OCSF severity to CrowdStrike native scale before push-down: `"high"` -> CrowdStrike severity >= 4 |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Related BCs | BC-2.11.010 (explain_query shows push-down plan) |
| Priority | P0 |
