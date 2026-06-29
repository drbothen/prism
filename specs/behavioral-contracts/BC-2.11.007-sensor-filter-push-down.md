---
document_type: behavioral-contract
level: L3
version: "1.9"
status: active
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-28"  # v1.9 S-DEMO-FIDELITY-REMEDIATION-001 — Mechanism B extended with §B.1 Planner-Side Entity-Discriminator Auto-Seeding postcondition; governs seed_armis_entity_discriminator(); closes F-L3-MED-001 POL-4 mis-anchor; AC-DISC anchors here
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

  #### Mechanism B — AQL Passthrough + Time-Window AQL-Clause Augmentation (Armis)
  Armis's native query language IS AQL; there is no OCSF-to-AQL translation layer. The user supplies the AQL entity string directly as a pseudo-column literal in the PrismQL WHERE clause:

  ```sql
  FROM armis_devices WHERE aql = 'in:devices' LIMIT 100
  ```

  The query planner recognizes `aql` as an INDEX pseudo-column on Armis tables (declared with `options = ["INDEX"]` in `armis.sensor.toml`). It extracts the literal string value and seeds it verbatim into `FetchContext.query_filters["aql"]`. The spec engine interpolates this value into the TOML `path_template` via `${query.filter.aql}`, forwarding it as a URL query parameter to the DTU endpoint `GET /api/v1/search?aql=<value>`. Base AQL content is opaque (R-DTU-002 / ADR-031 §D8-a).

  **Time-window AQL-clause augmentation (in scope, v1.8 per human directive 2026-06-05):** When the PrismQL WHERE clause also contains a time-window Compare predicate (Gt/Ge/Lt/Le) on a datetime column declared `options = ["INDEX"]` in `armis.sensor.toml` (e.g., `last_seen`, `created_at`), the query-engine layer appends the canonical Armis AQL time clause to the user's base AQL string before forwarding:

  - Bounded range: `<base_aql> after:2026-01-01T00:00:00 before:2026-01-02T00:00:00`
  - Lower bound only: `<base_aql> after:<ts>`
  - Relative (when expressed as timeFrame): `<base_aql> timeFrame:"<N> <unit>"` (e.g. `timeFrame:"3 Hours"`)
  - AQL syntax confirmed by research-doc `armis-aql-time-window-syntax-2026-06.md` (HIGH confidence, 6 independent sources): bare, unquoted, timezone-naive `YYYY-MM-DDTHH:MM:SS` timestamps; space-separated keywords; no `AND`/parens/operators. The `lastSeen:>"T"` comparison-operator form is NOT confirmed and MUST NOT be used.

  **Anti-double-filter guard:** If the user's base AQL string already contains any of `after:`, `before:`, or `timeFrame:`, the string is forwarded verbatim without augmentation. The user's explicit time scope is preserved. This guard prevents duplicate time clauses when the user has already embedded temporal filtering in their AQL.

  **DTU-honors-AQL-time-clause contract:** The prism-dtu-armis clone MUST parse and honor the `after:`/`before:` / `timeFrame:` clauses by filtering its fixture dataset on `last_seen` (devices) and `created_at` (alerts). Without this change, time-window scenarios are vacuous (DTU returns full dataset regardless of time clause). The verbatim `capture_aql()` call (R-DTU-002) is unaffected — the augmented AQL string is captured as-is.

  The combined AQL string (base + time clause) is forwarded via the existing `${query.filter.aql}` path. No new DTU struct fields are needed; time bounds arrive embedded in the AQL string. DataFusion post-filter on the same time column STILL runs as the correctness backstop (result-equivalence invariant preserved).

  > **v1.7 "post-filter-only for Armis" text SUPERSEDED (append-only, POLICY 1):** Prior versions stated Armis time-window predicates "are post-filtered by DataFusion after materialization" due to lack of native DTU time params. This was the correct assessment before the human directive (2026-06-05) to fully wire Armis AQL time-window. The AQL-clause augmentation path now provides native-equivalent time filtering for Armis. The superseded text read: "Remaining post-filter predicates are applied by DataFusion over the materialized OCSF table. This includes ALL time-window predicates for Armis, Cyberint, and Claroty sensors (which lack native DTU time params)."

  Discrimination between entity types (`in:devices` vs `in:alerts`) is performed by the DTU clone via string pattern-matching on the received AQL value; the query planner does not inspect or validate AQL syntax beyond extracting the base AQL literal and the time-window bounds from the AST.

  #### Mechanism B.1 — Planner-Side Entity-Discriminator Auto-Seeding (absent-aql case)

  **PC-DISC-001:** When the resolved `aql` filter for an Armis table is absent or empty (i.e., the user wrote no `WHERE aql = '...'` predicate), the query planner synthesizes the entity discriminator from the source table name and seeds it into `FilterMap["aql"]` BEFORE constructing the fan-out `QueryParams`:

  | Source Table | Seeded `aql` value |
  |---|---|
  | `armis_alerts` | `"in:alerts"` |
  | `armis_devices` | `"in:devices"` |

  The seeding is performed by `seed_armis_entity_discriminator()` in `prism-query/src/materialization.rs` (S-DEMO-FIDELITY-REMEDIATION-001). The function is called per-target immediately before `QueryParams` construction so cross-target contamination is impossible (each target receives its own `FilterMap` clone).

  **PC-DISC-002:** A user-supplied non-empty `WHERE aql = '<value>'` predicate is preserved verbatim. The planner DOES NOT overwrite a non-empty `aql` filter value. The auto-seeding fires ONLY when `filters["aql"]` is absent or the value is whitespace-only after `trim()`.

  **PC-DISC-003:** Non-Armis source tables (e.g., `crowdstrike_detections`, `cyberint_alerts`, `claroty_devices`) are unaffected. `seed_armis_entity_discriminator()` passes `FilterMap` through unchanged for any `source_table` that does not match `"armis_alerts"` or `"armis_devices"`.

  **Mechanism rationale:** The Armis DTU (`GET /api/v1/search`) uses the AQL `in:alerts` / `in:devices` prefix as the sole entity discriminator (EC-001 in `prism-dtu-armis/src/routes/search.rs`: absent AQL defaults to device records). Without auto-seeding, a `FROM armis_alerts` query with no explicit `WHERE aql = '...'` sends `?aql=` (blank) to the DTU, which returns device records; after OCSF normalization the alert-severity/status filter post-filter discards all rows, yielding silently 0 rows (F-L2-CRIT-001). Auto-seeding eliminates this silent failure by ensuring the correct entity discriminator is always present before fan-out.

  > **v1.8 Mechanism B prose correction (append-only, POLICY 1):** The v1.8 Mechanism B description states "the user supplies the AQL entity string directly... the planner does NOT synthesize." This was accurate for the user-supplied case but failed to document the absent-aql planner-synthesis path. The §B.1 postcondition above is the authoritative governing contract for that path; the v1.8 prose is supplemented (not replaced) by §B.1. The user-supplied passthrough behavior described in v1.8 Mechanism B remains accurate and is unaffected.

  This passthrough convention was established by the merged S-DEMO-ARMIS-AQL-001 (PR #168, 2026-06-02) and is grounded in BC-2.16.013 §Postconditions §1 (`armis.sensor.toml` devices and alerts tables use `GET /api/v1/search` with AQL forwarded via `${query.filter.aql}`). Time-window augmentation is the S-DEMO-QUERY-PUSHDOWN-001 v2 scope addition.

- Remaining post-filter predicates are applied by DataFusion over the materialized OCSF table. This includes ALL time-window predicates for Cyberint and Claroty sensors (which lack native DTU time params), and any other predicates that cannot be mapped to a supported sensor-native param. **Armis time-window is no longer in this category** — it is handled via Mechanism B AQL-clause augmentation, with DataFusion post-filter as the correctness backstop only.
- The push-down classification is visible in `explain_query` output (see BC-2.11.010)
- Push-down reduces the volume of data fetched from sensor APIs, improving performance and reducing materialization size. For sensors without native time-window params (Cyberint, Claroty), the result-equivalence invariant guarantees correctness at the cost of fetching a broader dataset that DataFusion then filters. For Armis, the AQL-clause augmentation provides time-window filtering at the DTU level, with DataFusion post-filter as the correctness backstop.

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
- **Time range push-down (qualified, v1.8 per pushdown-redesign.md §8 + human directive 2026-06-05 + research-doc armis-aql-time-window-syntax-2026-06.md):** Time-window push-down is supported by **CrowdStrike** (FQL injection via `DetectionListParams.filter` query param, ADR-033 Option T1 heuristic) AND **Armis** (AQL-clause augmentation via appending `after:`/`before:` or `timeFrame:` keywords to the base AQL string, forwarded via the existing `${query.filter.aql}` path, with anti-double-filter guard). For Armis, the prism-dtu-armis clone MUST honor the AQL time clause by filtering its dataset (load-bearing contract). **Cyberint and Claroty** do NOT have native time-window params in their current DTU structs — time-window predicates for these sensors are post-filtered by DataFusion after materialization (result-equivalence invariant preserved). The spec-driven `options: Index` declaration on a datetime column indicates the column is eligible for time-window push-down IF the sensor's push-down mechanism supports it; declaration alone is not sufficient. > **v1.7 "only CrowdStrike" qualifier SUPERSEDED (append-only, POLICY 1):** The v1.7 invariant stated "only CrowdStrike has a usable native time param." This was correct at v1.7. Armis is now added via AQL-clause augmentation per the human directive (2026-06-05). Superseded v1.7 text: "only CrowdStrike has a usable native time param (`DetectionListParams.filter` FQL injection). Armis, Cyberint, and Claroty do NOT have native time-window params." > **v1.6 claim SUPERSEDED (append-only, POLICY 1):** The prior invariant "Time range push-down is always attempted (all initial sensors support time-based filtering)" was factually incorrect against production DTU structs and was qualified in v1.7. The superseded text was: `Time range push-down is always attempted (all initial sensors support time-based filtering; spec-driven sensors declare push-down support per column via options: Index)`.
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
| EC-11-021 | `FROM armis_alerts` with no `WHERE aql` predicate (absent-aql auto-seeding) | `seed_armis_entity_discriminator("armis_alerts", empty_filters)` seeds `filters["aql"] = "in:alerts"`. DTU receives `GET /api/v1/search?aql=in:alerts` and returns alert records. Without seeding the DTU would default to device records → 0 alert rows after OCSF post-filter (F-L2-CRIT-001). |
| EC-11-022 | `FROM armis_alerts WHERE aql = 'in:alerts status:Open'` — user-supplied non-empty AQL | `seed_armis_entity_discriminator` detects non-empty existing `aql = "in:alerts status:Open"` and passes `FilterMap` through unchanged. DTU receives `GET /api/v1/search?aql=in:alerts+status:Open`. User's AQL is never overwritten (PC-DISC-002). |
| EC-11-023 | Non-Armis table with `WHERE aql = 'anything'` (e.g., `FROM crowdstrike_detections WHERE aql = 'test'`) | `seed_armis_entity_discriminator` source_table is `"crowdstrike_detections"` — does not match either Armis table name. `FilterMap` is returned unchanged (PC-DISC-003). The `aql` filter is forwarded as a regular push-down predicate or dropped per CrowdStrike's column classification. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query with `severity = 'critical'` against CrowdStrike (INDEX column) | Push-down generated: `severity:5`; no post-filter for severity | happy-path |
| Query missing REQUIRED column `customer_id` for Cyberint | `Err(E-QUERY-009)` before any API calls | error |
| Query with `device.hostname = 'srv01'` (OPTIMIZED on all sensors) | Post-filter only; DataFusion filters after materialization | edge-case |
| `FROM armis_devices WHERE aql = 'in:devices' LIMIT 100` | `FetchContext.query_filters["aql"] = "in:devices"`; DTU receives `GET /api/v1/search?aql=in:devices`; Mechanism B passthrough; no AQL translation | happy-path (Armis passthrough) |
| `FROM armis_alerts WHERE aql = 'in:alerts status:Open' LIMIT 50` | `FetchContext.query_filters["aql"] = "in:alerts status:Open"`; DTU receives `GET /api/v1/search?aql=in:alerts+status:Open`; AQL content is opaque — no validation | happy-path (Armis passthrough with compound AQL) |
| `FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00' AND last_seen < '2026-02-01T00:00:00'` — `last_seen` is `options = ["INDEX"]` datetime in `armis.sensor.toml` | `FetchContext.query_filters["aql"] = "in:devices after:2026-01-01T00:00:00 before:2026-02-01T00:00:00"`; DTU receives bounded-range AQL clause; DTU returns PROPER SUBSET of unfiltered device fixture (filtered_count < unfiltered_count — load-bearing); DataFusion post-filter on `last_seen` also applies | happy-path (Armis time-window AQL-clause augmentation — bounded range) |
| `FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00'` | `FetchContext.query_filters["aql"] = "in:devices after:2026-01-01T00:00:00"`; bare unquoted timestamp; no `before:` clause | happy-path (Armis time-window AQL-clause augmentation — lower bound only) |
| `FROM armis_devices WHERE aql = 'in:devices after:2024-06-01T00:00:00' AND last_seen > '2026-01-01T00:00:00'` — base AQL already contains `after:` | `FetchContext.query_filters["aql"] = "in:devices after:2024-06-01T00:00:00"` — anti-double-filter guard fires; no second `after:` appended; user's explicit time scope preserved | edge-case (Armis anti-double-filter guard) |
| `FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00'` — WITH DataFusion post-filter only (no AQL augmentation) AND WITH AQL augmentation | Both executions return identical row sets — result-equivalence invariant holds for Armis time-window push-down path | invariant (Armis result-equivalence) |
| `FROM armis_alerts` with NO `WHERE aql` predicate — absent-aql entity-discriminator auto-seeding (PC-DISC-001 / EC-11-021) | `seed_armis_entity_discriminator("armis_alerts", {})` → `filters["aql"] = "in:alerts"`; DTU receives `GET /api/v1/search?aql=in:alerts`; DTU returns alert records (not device records); result set contains rows with alert-entity shape. LOAD-BEARING: result count > 0 (fixtures contain at least one alert record). Without auto-seeding the DTU returns device records and alert OCSF post-filter discards all → 0 rows. | happy-path (absent-aql auto-seeding, armis_alerts) |
| `FROM armis_alerts WHERE aql = 'in:alerts status:Open'` — user-supplied non-empty AQL preserved verbatim (PC-DISC-002 / EC-11-022) | `seed_armis_entity_discriminator("armis_alerts", {"aql": "in:alerts status:Open"})` detects non-empty existing value and returns `FilterMap` unchanged; DTU receives `GET /api/v1/search?aql=in:alerts+status:Open`; user's compound AQL is forwarded intact | happy-path (user-aql preserved, no overwrite) |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-031 | Required column enforcement: rejects unconstrained | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("PrismQL Query Execution Engine") per capabilities.md §CAP-015 — this BC defines the filter push-down optimization contract that is a direct sub-behavior of the query execution capability. |
| L2 Invariants | DI-021 |
| Related BCs | BC-2.11.010 (explain_query shows push-down plan), BC-2.01.013 v1.14 (per-sensor push-down translation table; this BC's result-equivalence invariant is referenced in BC-2.01.013 TV-006/TV-007/TV-008 and EC-01-027) |
| Related ADRs | ADR-033 (push-down time-window extraction strategy — pre-fan-out heuristic T1; covers CrowdStrike FQL injection AND Armis AQL-clause augmentation path; Cyberint and Claroty confirmed as no-native-time-param) |
| Related Research | `.factory/research/armis-aql-time-window-syntax-2026-06.md` (HIGH confidence; canonical Armis AQL time-window syntax confirmed — `after:<ts>` / `before:<ts>` bare unquoted timezone-naive ISO8601; `timeFrame:"<N> <unit>"` relative; `lastSeen:>"T"` comparison-operator form NOT confirmed and MUST NOT be used) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.9 | S-DEMO-FIDELITY-REMEDIATION-001 | 2026-06-28 | product-owner | Closes F-L3-MED-001 (POL-4 semantic mis-anchor): §Mechanism B.1 "Planner-Side Entity-Discriminator Auto-Seeding" postcondition authored. (1) PC-DISC-001: when `aql` filter is absent or empty for an Armis table, the query planner synthesizes `"in:alerts"` for `armis_alerts` / `"in:devices"` for `armis_devices` and seeds into `FilterMap["aql"]` before fan-out. (2) PC-DISC-002: user-supplied non-empty `WHERE aql = '...'` is preserved verbatim — auto-seeding never overwrites. (3) PC-DISC-003: non-Armis source tables are unaffected. (4) Mechanism rationale: absent-AQL → blank `?aql=` → DTU defaults to device records → 0 alert rows after OCSF post-filter (F-L2-CRIT-001). Implementation: `seed_armis_entity_discriminator()` in `prism-query/src/materialization.rs`. (5) Added EC-11-021 (absent-aql seeding), EC-11-022 (user-aql preserved), EC-11-023 (non-Armis table unaffected). (6) Added 2 canonical test vectors: absent-aql happy-path (LOAD-BEARING: result count > 0), user-supplied AQL preservation. (7) v1.8 Mechanism B prose correction note added append-only per POLICY 1: v1.8 stated "the planner does NOT synthesize" — accurate for user-supplied path, but omitted the absent-aql synthesis path now documented in §B.1. AC-DISC anchor: §Mechanism B.1 / PC-DISC-001. |
| 1.8 | S-DEMO-QUERY-PUSHDOWN-001-v2-armis-aql-full-wiring | 2026-06-05 | product-owner | Human directive 2026-06-05: "fully wire Armis AQL into our DTU and our scenarios." (1) Mechanism B extended: added AQL-clause augmentation design — query-engine appends canonical Armis AQL time clause (`after:<ts>` / `before:<ts>` / `timeFrame:"<N> <unit>"`) to user's base AQL string when WHERE clause has time Compare predicates on `options = ["INDEX"]` datetime columns in `armis.sensor.toml`. Confirmed AQL syntax from research-doc `armis-aql-time-window-syntax-2026-06.md` (HIGH confidence, 6 independent sources): bare unquoted timezone-naive `YYYY-MM-DDTHH:MM:SS`; space-separated keywords; no `AND`/parens. Anti-double-filter guard specified. DTU-honors-AQL-time-clause contract specified (prism-dtu-armis must filter dataset on `last_seen`/`created_at`; load-bearing). (2) Invariant updated: "CrowdStrike (FQL injection) AND Armis (AQL-clause augmentation)" now both support time-window push-down; Cyberint and Claroty remain post-filter-only. Superseded v1.7 "only CrowdStrike" text preserved append-only per POLICY 1. (3) Post-filter sentence updated: Armis removed from the "lacks native DTU time params" list. (4) Six new canonical test vectors added: Armis bounded-range augmentation (load-bearing), lower-bound augmentation, anti-double-filter guard, result-equivalence invariant for Armis path. (5) Traceability updated: research-doc added to Related Research; BC-2.01.013 citation updated to v1.14; ADR-033 note updated to cover Armis augmentation path. Capability Anchor Justification row added. |
| 1.7 | S-DEMO-QUERY-PUSHDOWN-001-v2-bc-respec | 2026-06-05 | product-owner | S-DEMO-QUERY-PUSHDOWN-001 v2 re-spec (LOCAL adversary passes 5/6 factual correction). (1) Qualified the over-broad time-range invariant: "all initial sensors support time-based filtering" was factually wrong against production DTU structs. Corrected to: only CrowdStrike has a usable native time param (`DetectionListParams.filter` FQL injection, via ADR-033 Option T1 heuristic); Armis/Cyberint/Claroty have NO native time-window params in their current DTU structs — time predicates fall back to DataFusion post-filter for these sensors; result-equivalence invariant is preserved. Superseded v1.6 text retained append-only per POLICY 1. (2) §Mechanism A updated: Cyberint POST-body `from_date`/`to_date` injection and Claroty POST-body filter arrays removed as Mechanism A push-down targets; corrected prose reflects no-native-time-param reality for these two sensors. v1.6 grouping note retained append-only. (3) §Predicate Classification — post-filter sentence extended to explicitly name Armis/Cyberint/Claroty time-window as always-post-filter cases. (4) ADR-033 added to Traceability §Related ADRs. BC-2.01.013 added to Related BCs. |
| 1.6 | D-987-post-merge-POL14 | 2026-06-04 | state-manager | POL-14 status-field alignment: `status:` synced draft→active (anchor story S-DEMO-002 merged PR #171 develop@fdd12251 2026-06-04). `lifecycle_status` was already active (ground truth per ADR-025). BC-INDEX row 157 updated to v1.6 active. No body changes; changelog row added for version bump. |
| 1.5 | F-DEMO002-P1-MED-002-adjudication | 2026-06-02 | product-owner | AMENDMENT 4 — adjudicates finding F-DEMO002-P1-MED-002 (POL-4 semantic drift). Rewrote §Predicate Classification to document two distinct push-down mechanisms: Mechanism A (Predicate Translation, used by CrowdStrike/Cyberint/Claroty) and Mechanism B (Verbatim-AQL Passthrough, Armis only). Armis convention: user writes `aql = '<string>'` pseudo-column literal in PrismQL WHERE; query planner seeds verbatim string into `FetchContext.query_filters["aql"]`; spec engine interpolates via `${query.filter.aql}` into TOML path_template; DTU receives `GET /api/v1/search?aql=<value>` opaque per R-DTU-002 / ADR-031 §D8-a. Precedent: merged S-DEMO-ARMIS-AQL-001 PR #168 / BC-2.16.013 §Postconditions §1. Updated EC-11-019 to reflect passthrough semantics. Added two Armis Mechanism B canonical test vectors. CrowdStrike translation example (EC-11-020, test vectors) preserved intact. |
| 1.4 | pre-impl-amendments | 2026-05-06 | product-owner | AMENDMENT 3 — added REQUIRED Column Runtime Mechanism section: SoT is ColumnOptions::Required in prism-core/column.rs; lookup interface via ColumnSpec.options in spec-parser; no hardcoded column names; illustrative per-sensor examples; INV-REQUIRED-SPECDRIVEN invariant; relationship to BC-2.11.011 cross-client scoping. Resolves S-3.02 implementer blocker on REQUIRED column lookup. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
