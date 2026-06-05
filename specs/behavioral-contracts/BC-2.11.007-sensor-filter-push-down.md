---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-05"
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

# BC-2.11.007: Sensor Filter Push-Down

## Description

Push-down translates WHERE predicates from the PrismQL AST into sensor-native API filter syntax (CrowdStrike FQL, Cyberint JSON body, Claroty POST arrays, Armis AQL) to minimize data transferred from sensor APIs. Each adapter column declares a push-down capability option (REQUIRED/INDEX/ADDITIONAL/OPTIMIZED/DEFAULT); only REQUIRED, INDEX, and ADDITIONAL columns are pushed down. Predicates on OPTIMIZED/DEFAULT columns are post-filtered by DataFusion after materialization. Column pruning (passing `columns_used` to adapters that support field selection) further reduces payload. Push-down is an optimization only — query correctness is identical whether push-down occurs or not. REQUIRED columns enforce DI-021: queries that omit a REQUIRED column are rejected before any API calls.

## Preconditions
- A PrismQL query has been parsed into an AST with WHERE predicates
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
- Push-down filters are applied to sensor-native query syntax via one of two distinct mechanisms:

  #### Mechanism A — Predicate Translation (CrowdStrike; Cyberint/Claroty time-window falls back to post-filter)
  The query planner translates each OCSF/PrismQL WHERE predicate into the sensor's native filter syntax when the sensor's DTU supports the corresponding parameter. The user writes standard PrismQL; the planner converts it:
  - CrowdStrike: FQL filter syntax (e.g., `severity = 'critical'` → `severity:5`). Time-window: injected as `filter` FQL param on Step 1 (`query_detection_ids`) via ADR-033 Option T1 heuristic — `created_timestamp:>'<ISO8601>'` for start, `created_timestamp:<'<ISO8601>'` for end. `limit` query param is also wired.
  - Cyberint: No time-window push-down against the current DTU. `AlertListParams` is cursor-only (GET, no body_template); `from_date`/`to_date` POST-body injection is NOT correct. Time predicates are post-filtered by DataFusion. Cursor pagination is handled by the existing pipeline.
  - Claroty xDome: No time-window push-down against the current DTU. `body_template: '{}'` is always empty; URL OffsetLimit params (`?offset=N&limit=M`) handle pagination via the existing OffsetLimit pipeline. Body-based offset/limit deferred to `S-DEMO-CLAROTY-PAGINATION-001` (Gap-CL-004). Time predicates are post-filtered by DataFusion.

  > **v1.6 Mechanism A grouping note (append-only, POLICY 1):** Prior versions listed Cyberint POST-body params and Claroty POST-body filter arrays as active Mechanism A push-down. These are incorrect against production DTU structs as of v1.7 (see pushdown-redesign.md §1.3+§1.4; ADR-033 §Rationale §5).

  #### Mechanism B — Verbatim-AQL Passthrough (Armis)
  Armis's native query language IS AQL; there is no OCSF-to-AQL translation layer. Instead, the user supplies the AQL string directly as a pseudo-column literal in the PrismQL WHERE clause:

  ```sql
  FROM armis_devices WHERE aql = 'in:devices' LIMIT 100
  ```

  The query planner recognizes `aql` as an INDEX pseudo-column on Armis tables (declared with `options = ["INDEX"]` in `armis.sensor.toml`). It extracts the literal string value and seeds it verbatim into `FetchContext.query_filters["aql"]`. The spec engine interpolates this value into the TOML `path_template` via `${query.filter.aql}`, forwarding it as a URL query parameter to the DTU endpoint `GET /api/v1/search?aql=<value>`. No translation of the AQL content occurs — the string is treated as opaque (R-DTU-002 / ADR-031 §D8-a).

  This passthrough convention was established by the merged S-DEMO-ARMIS-AQL-001 (PR #168, 2026-06-02) and is grounded in BC-2.16.013 §Postconditions §1 (`armis.sensor.toml` devices and alerts tables use `GET /api/v1/search` with AQL forwarded via `${query.filter.aql}`).

  Discrimination between entity types (`in:devices` vs `in:alerts`) is performed by the DTU clone via string pattern-matching on the received AQL value; the query planner does not inspect or validate AQL syntax.

- Remaining post-filter predicates are applied by DataFusion over the materialized OCSF table. This includes ALL time-window predicates for Armis, Cyberint, and Claroty sensors (which lack native DTU time params), and any other predicates that cannot be mapped to a supported sensor-native param.
- The push-down classification is visible in `explain_query` output (see BC-2.11.010)
- Push-down reduces the volume of data fetched from sensor APIs, improving performance and reducing materialization size. For sensors without native time-window params (Armis, Cyberint, Claroty), the result-equivalence invariant guarantees correctness at the cost of fetching a broader dataset that DataFusion then filters.

### Column Pruning

The query planner tracks which columns are referenced in the query (SELECT list + WHERE + ORDER BY + GROUP BY). This column usage set is passed to the sensor adapter, which uses it to populate API `fields`/`select` parameters where supported, minimizing response payload. Specifically:

- The planner computes a `columns_used: HashSet<String>` from all column references in the query AST
- This set is included in the `QueryContext` passed to each sensor adapter
- Adapters that support field selection (e.g., CrowdStrike's `fields` parameter, Armis's `fields` in AQL) translate the set to API-specific field selection syntax
- Adapters that do not support field selection ignore the set and return full records
- Column pruning is an optimization only; it does not affect query correctness

## REQUIRED Column Runtime Mechanism

REQUIRED columns enforce DI-021: queries against a sensor must constrain at least one REQUIRED column in the WHERE clause, or be rejected with `E-QUERY-009` before any API calls.

### Source of Truth

The canonical `ColumnOptions::Required` type is defined in `crates/prism-core/src/column.rs` (exported as `prism_core::ColumnOptions`). This enum variant (`ColumnOptions::Required`) is the single source of truth for REQUIRED classification — not a hardcoded column-name list.

Column options (including REQUIRED) are declared per-column in sensor spec TOML files using `options = ["REQUIRED"]` on `[[tables.columns]]` entries. The `prism-spec-engine` `SpecParser` parses these into `ColumnSpec.options: Vec<ColumnOptions>` (type `prism_spec_engine::spec_parser::ColumnSpec`).

### Runtime Lookup Interface

The query planner determines REQUIRED columns for a sensor at plan time by inspecting the loaded `ColumnSpec` entries from the active `ConfigSnapshot`:

```
ConfigSnapshot.sensor_specs[sensor_id]   // SensorSpec
  -> .tables[*]                           // SensorTableDescriptor (prism-spec-engine types.rs)
  -> .columns[*]                          // ColumnDef
```

For spec-parser-level detail (where options live):
```
SpecLoader::parse(toml) -> SensorSpec
  -> .tables[*]           // TableSpec (prism_spec_engine::spec_parser::TableSpec)
  -> .columns[*]          // ColumnSpec (prism_spec_engine::spec_parser::ColumnSpec)
  -> .options             // Vec<ColumnOptions> — contains ColumnOptions::Required if declared
```

The query planner collects REQUIRED column names by filtering:
```
columns.iter()
    .filter(|c| c.options.contains(&ColumnOptions::Required))
    .map(|c| &c.name)
```

No hardcoded REQUIRED column name list exists in the query engine. The set of REQUIRED columns is fully spec-driven.

### Canonical REQUIRED Column Names per Sensor (spec-driven, not hardcoded)

REQUIRED column names are declared in `.sensor.toml` files, not hardcoded in query-engine code. The following are representative examples from existing sensor specs and test fixtures — implementers must read the actual loaded `ColumnSpec` at runtime:

| Sensor | Example REQUIRED Columns | Notes |
|--------|--------------------------|-------|
| CrowdStrike | `detection_id`, `device_id` | Declared in `[[tables.columns]]` with `options = ["REQUIRED"]`; vary per table |
| Cyberint | `customer_id` | Required for all Cyberint API calls; prevents cross-tenant data leakage |
| Claroty xDome | `site_id` | Required to scope to a specific customer's Claroty instance |
| Armis | `organizationId` | Armis API requires org scoping on every request |

> **Important:** these are illustrative. The actual REQUIRED columns for any given sensor+table combination are determined exclusively by the `options = ["REQUIRED"]` declarations in that sensor's TOML spec file, read at load time by `SpecParser`. The test-writer should use spec fixtures that match production sensor specs, not assume a hardcoded name list.

### Relationship to BC-2.11.011

BC-2.11.011 (cross-client scoping) depends on REQUIRED column enforcement defined here. REQUIRED columns that encode client-scoping (e.g., `customer_id`, `organizationId`) are the push-down mechanism by which the query planner ensures no cross-client data leakage. The REQUIRED rejection (E-QUERY-009) fires before any API call, preventing unbounded scans that would return multi-client data. This is the primary DI-021 enforcement point and directly underpins the cross-client scoping invariant in BC-2.11.011.

## Invariants
- Push-down is an optimization only; the query result must be identical whether or not push-down occurs
- A predicate that cannot be pushed down is never silently dropped -- it is always applied as a post-filter
- **Time range push-down (qualified, v1.7 per pushdown-redesign.md §1 + ADR-033):** Time-window push-down is attempted only for sensors whose DTU exposes a native time-window parameter. In the current initial sensor set, **only CrowdStrike** has a usable native time param (`DetectionListParams.filter` FQL injection). Armis, Cyberint, and Claroty do NOT have native time-window params in their current DTU structs — for these sensors, time-window predicates are post-filtered by DataFusion after materialization (result-equivalence invariant is preserved). The spec-driven `options: Index` declaration on a datetime column indicates the column is eligible for time-window push-down IF the sensor's DTU supports it; declaration alone is not sufficient. > **v1.6 claim SUPERSEDED (append-only, POLICY 1):** The prior invariant "Time range push-down is always attempted (all initial sensors support time-based filtering)" was factually incorrect against production DTU structs and has been qualified above. The superseded text was: `Time range push-down is always attempted (all initial sensors support time-based filtering; spec-driven sensors declare push-down support per column via options: Index)`.
- Push-down filter translation produces a canonical form (sorted parameter keys, normalized timestamp ISO8601 format, lowercase string values where applicable) before the result is used as cache key input. This ensures that semantically equivalent push-down filters produce identical cache keys regardless of the original predicate ordering in the PrismQL query.
- **INV-REQUIRED-SPECDRIVEN:** The set of REQUIRED columns for any sensor+table is determined exclusively by `ColumnOptions::Required` entries in the loaded `ColumnSpec`, not by any hardcoded name list in the query engine.

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
| EC-11-019 | Armis query uses verbatim-AQL passthrough (`aql = 'in:devices status:Online'`) | The AQL string is forwarded verbatim to `/api/v1/search?aql=in:devices+status:Online`; no translation occurs; remaining non-`aql` predicates (e.g., OPTIMIZED columns) are applied as post-filters by DataFusion. Other sensors executing fan-out alongside Armis use Mechanism A translation independently. |
| EC-11-020 | `severity >= "high"` pushed down to CrowdStrike (severity 1-5 scale) | Translate OCSF severity to CrowdStrike native scale before push-down: `"high"` -> CrowdStrike severity >= 4 |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query with `severity = 'critical'` against CrowdStrike (INDEX column) | Push-down generated: `severity:5`; no post-filter for severity | happy-path |
| Query missing REQUIRED column `customer_id` for Cyberint | `Err(E-QUERY-009)` before any API calls | error |
| Query with `device.hostname = 'srv01'` (OPTIMIZED on all sensors) | Post-filter only; DataFusion filters after materialization | edge-case |
| `FROM armis_devices WHERE aql = 'in:devices' LIMIT 100` | `FetchContext.query_filters["aql"] = "in:devices"`; DTU receives `GET /api/v1/search?aql=in:devices`; Mechanism B passthrough; no AQL translation | happy-path (Armis passthrough) |
| `FROM armis_alerts WHERE aql = 'in:alerts status:Open' LIMIT 50` | `FetchContext.query_filters["aql"] = "in:alerts status:Open"`; DTU receives `GET /api/v1/search?aql=in:alerts+status:Open`; AQL content is opaque — no validation | happy-path (Armis passthrough with compound AQL) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-031 | Required column enforcement: rejects unconstrained | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-021 |
| Related BCs | BC-2.11.010 (explain_query shows push-down plan), BC-2.01.013 (per-sensor push-down translation table; this BC's result-equivalence invariant is referenced in BC-2.01.013 TV-006 and EC-01-027) |
| Related ADRs | ADR-033 (push-down time-window extraction strategy — pre-fan-out heuristic T1 for CrowdStrike FQL injection; establishes that Armis/Cyberint/Claroty have no native time-window params in current DTU set, consistent with Invariants §Time range push-down) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | S-DEMO-QUERY-PUSHDOWN-001-v2-bc-respec | 2026-06-05 | product-owner | S-DEMO-QUERY-PUSHDOWN-001 v2 re-spec (LOCAL adversary passes 5/6 factual correction). (1) Qualified the over-broad time-range invariant: "all initial sensors support time-based filtering" was factually wrong against production DTU structs. Corrected to: only CrowdStrike has a usable native time param (`DetectionListParams.filter` FQL injection, via ADR-033 Option T1 heuristic); Armis/Cyberint/Claroty have NO native time-window params in their current DTU structs — time predicates fall back to DataFusion post-filter for these sensors; result-equivalence invariant is preserved. Superseded v1.6 text retained append-only per POLICY 1. (2) §Mechanism A updated: Cyberint POST-body `from_date`/`to_date` injection and Claroty POST-body filter arrays removed as Mechanism A push-down targets; corrected prose reflects no-native-time-param reality for these two sensors. v1.6 grouping note retained append-only. (3) §Predicate Classification — post-filter sentence extended to explicitly name Armis/Cyberint/Claroty time-window as always-post-filter cases. (4) ADR-033 added to Traceability §Related ADRs. BC-2.01.013 added to Related BCs. |
| 1.6 | D-987-post-merge-POL14 | 2026-06-04 | state-manager | POL-14 status-field alignment: `status:` synced draft→active (anchor story S-DEMO-002 merged PR #171 develop@fdd12251 2026-06-04). `lifecycle_status` was already active (ground truth per ADR-025). BC-INDEX row 157 updated to v1.6 active. No body changes; changelog row added for version bump. |
| 1.5 | F-DEMO002-P1-MED-002-adjudication | 2026-06-02 | product-owner | AMENDMENT 4 — adjudicates finding F-DEMO002-P1-MED-002 (POL-4 semantic drift). Rewrote §Predicate Classification to document two distinct push-down mechanisms: Mechanism A (Predicate Translation, used by CrowdStrike/Cyberint/Claroty) and Mechanism B (Verbatim-AQL Passthrough, Armis only). Armis convention: user writes `aql = '<string>'` pseudo-column literal in PrismQL WHERE; query planner seeds verbatim string into `FetchContext.query_filters["aql"]`; spec engine interpolates via `${query.filter.aql}` into TOML path_template; DTU receives `GET /api/v1/search?aql=<value>` opaque per R-DTU-002 / ADR-031 §D8-a. Precedent: merged S-DEMO-ARMIS-AQL-001 PR #168 / BC-2.16.013 §Postconditions §1. Updated EC-11-019 to reflect passthrough semantics. Added two Armis Mechanism B canonical test vectors. CrowdStrike translation example (EC-11-020, test vectors) preserved intact. |
| 1.4 | pre-impl-amendments | 2026-05-06 | product-owner | AMENDMENT 3 — added REQUIRED Column Runtime Mechanism section: SoT is ColumnOptions::Required in prism-core/column.rs; lookup interface via ColumnSpec.options in spec-parser; no hardcoded column names; illustrative per-sensor examples; INV-REQUIRED-SPECDRIVEN invariant; relationship to BC-2.11.011 cross-client scoping. Resolves S-3.02 implementer blocker on REQUIRED column lookup. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
