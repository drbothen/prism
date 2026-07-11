---
document_type: behavioral-contract
level: L3
version: "1.11"
status: active
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.005: Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable

## Description

Ephemeral materialization is the core execution mechanism for every `query` tool call. When a QueryPlan is ready, the engine fans out concurrently (default 10 parallel calls) to sensor APIs and RocksDB internal tables, normalizes external responses to OCSF via CAP-003, converts records to Arrow RecordBatch (with hot OCSF fields as flat columns, full event in `event_data`, and virtual fields injected), and registers all batches as a DataFusion MemTable named `events`. The 10K record streaming counter enforces DI-019 at fan-out time, aborting immediately on breach. The SessionContext and all materialized data are dropped when the tool call returns — there is no cross-call state. A per-query in-memory cache prevents redundant API calls when DataFusion's plan accesses the same source multiple times within one execution.

## Preconditions
- A `QueryPlan` has been produced with resolved clients, sensors, push-down filters, and post-filters
- Sensor credentials are available for all resolved (client, sensor) combinations
- The query has passed all security limit pre-checks

## Postconditions
- Fan-out to sensor APIs occurs concurrently for all resolved (client, sensor, source) combinations
  - Each sensor fetch checks the response cache (CAP-014) first; cache hits skip the API call
  - Cache misses trigger sensor API calls with push-down filters translated to sensor-native syntax
  - Max fan-out concurrency bounded (configurable, default 10 parallel sensor calls)
- Sensor responses are normalized to OCSF via the OCSF normalizer (CAP-003)
- OCSF-normalized records are converted to Arrow RecordBatch format:
  - Hot OCSF fields (severity, timestamp, device.ip, device.hostname, src_endpoint, dst_endpoint, etc.) as flat top-level Arrow columns
  - Full event serialized as JSON in an `event_data` string column for `json_extract_string()` UDF access
  - Virtual fields (`sensor`, `client_id`, `source`) injected as additional columns
  - **For spec-driven sensor adapters (`SpecDrivenSensorAdapter`):** OCSF normalization conformance is further specified in BC-2.01.013 OCSF Conformance Clause (v1.13). In particular: all spec-declared data columns from the sensor TOML spec must survive into the RecordBatch via `ColumnMapper`; `category_uid`/`class_uid` must be derived by `OcsfNormalizer` from the spec's declared `ocsf_class` (not copied from the raw vendor record); `_sensor` must be present. A RecordBatch containing only the three OCSF envelope columns (`category_uid`, `class_uid`, `_sensor`) with all spec-declared data columns absent is NON-CONFORMANT (S-DEMO-001 adversary F-001-R; D-924). Additionally, as of S-DEMO-QUERY-PUSHDOWN-001, cache misses on the first/query-plan step trigger sensor API calls with push-down filters from `FetchContext` translated to sensor-native syntax — see BC-2.01.013 v1.13 Pagination/Push-Down Scope Clause for the corrected per-sensor translation spec (CrowdStrike FQL injection via ADR-033 Option T1; Armis AQL passthrough; Cyberint/Claroty: no native time-window push-down in current DTU set).
- Records are fetched with a running counter. If the total fetched record count across all sensors exceeds 10K during fan-out, the fetch is aborted and an error is returned. Partial memory consumption during fetch is accepted (bounded by the 10K record limit). No pre-estimation of record counts is required; the limit is enforced as records arrive.
- RecordBatches are registered as a DataFusion `MemTable` named `events` in a fresh `SessionContext`
- **Empty-table pre-registration (DEC-022 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Sub-defect 2 expansion):** When a referenced sensor table returns 0 batches, a schema-only empty `MemTable` is pre-registered for that table before DataFusion query planning (`pre_register_empty_tables`). Pre-registration covers: `FROM`, `JOIN`, `WHERE Predicate::InSubquery`, and nested subqueries at arbitrary depth (via `walk_sql_query` recursive walk). **Projection-position `Expr::InSubquery` (`SELECT`, `GROUP BY`, `ORDER BY`) is NOT pre-registered** — the plan-time gate `check_expr_insubquery_projection` rejects such queries with `E-QUERY-043 / ExprInSubqueryProjectionNotSupported` before DataFusion planning occurs (Option A adjudication D-1650 / F-CSD-P4-001, 2026-07-10; pre-registration for projection position is dead code under Option A). Schema is resolved via a **3-priority ladder**: (P1) live `TableRegistry` schema if available; (P2) bundled TOML spec schemas (compile-time `include_str!` embedding of `prism-sensors/specs/` TOMLs, keyed `{sensor_id}_{table_name}`, backed by `BUNDLED_SPEC_SCHEMAS: OnceLock`); (P3) JOIN ON equality inference from peer table Arrow columns for any fields not covered by P1/P2. Spec-declared `datetime` columns are mapped to Arrow `Timestamp(Microsecond, UTC)` to preserve type fidelity. **After the schema ladder resolves, `virtual_fields::append_virtual_fields_to_schema` appends `_sensor` (Utf8, nullable=true), `_client` (Utf8, nullable=true), `_source_table` (Utf8, nullable=true), and `_source_type` (Utf8, nullable=true) to every empty-table schema** — mirroring the virtual-field injection performed by `inject_virtual_fields` on the populated batch path. A spoofed-column dedup guard (mirroring `remove_spoofed_virtual_columns`) strips any pre-existing fields named `_sensor`, `_client`, or `_source_table` before appending, preventing duplicate virtual columns. Virtual fields use `nullable=true` (diverging from the populated path's `nullable=false`) to support LEFT JOIN NULL propagation: when the empty table is the right side of a LEFT JOIN, DataFusion propagates NULL for the virtual field columns rather than producing a schema mismatch error. See BC-2.11.012 §Invariants "Empty MemTable schema parity" for the full contract. This guarantees DataFusion can plan queries that JOIN or subquery against empty sensor tables, returning 0 rows gracefully instead of `DataFusionError::Plan("table not found")` mapped to `PrismError::QueryExecutionFailed`.
- The `SessionContext` (and all materialized data) is dropped when the query tool call returns. There is no cross-call pagination for query results; each `query` call re-materializes from scratch (the response cache mitigates re-fetch cost). The `limit` tool parameter truncates DataFusion results after execution; `is_truncated` and `total_available` are set in the response when results exceed `limit`.

## In-Query Cache

Within a single query execution, the query engine maintains a per-query cache of materialized sensor data. If the same `(client_id, sensor_id, source_id, push_down_params)` tuple is requested multiple times within one query (e.g., due to DataFusion plan structure such as self-joins, subqueries referencing the same source, or aggregation plans that re-scan), the second fetch reuses the first fetch's data instead of making a redundant API call. This in-query cache is distinct from the cross-query TTL cache (CAP-014):

- **Scope:** Single query execution only. The cache is created when the query begins and dropped when the `SessionContext` is torn down.
- **Key:** `(client_id, sensor_id, source_id, canonicalized_push_down_params)` -- the same canonicalization used for the cross-query cache key.
- **Lifetime:** Exists only for the duration of the query. No TTL -- entries are valid for the entire query execution.
- **Purpose:** Prevents redundant API calls when DataFusion's execution plan touches the same sensor data source multiple times. This is critical for the federated model where each "table scan" translates to a remote API call.

## Invariants
- DI-019: Materialization limit of 10K records enforced via streaming counter during fetch (abort on breach)
- DI-008: Client data separation -- each record includes `client_id` provenance in the materialized table
- The transient memory peak (both DynamicMessage and Arrow representations in memory simultaneously during conversion) is bounded by the 10K record limit

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-005` | Fan-out fetched record count exceeds 10K during streaming | Fetch aborted; error includes per-sensor fetched counts and narrowing suggestions. `PrismError::QueryMaterializationLimitExceeded { count, max }` — `"E-QUERY-005: materialization limit exceeded: fetched {count} records (max {max})"` (materialization records limit per BC-2.11.006 canonical mapping, error-taxonomy v1.68+) |
| `E-SENSOR-001` | One or more sensor API calls fail | Partial materialization: successful sensors contribute data; failed sensors listed in `sensor_errors` |
| `E-AUTH-005` | Credentials unavailable for a sensor | Sensor excluded from fan-out; listed in `sensor_errors` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-022 | All sensor API calls return empty — empty MemTable pre-registration for supported positions | Schema-only empty `MemTable` pre-registered for every referenced table that returned 0 batches. Pre-registration covers: FROM, JOIN, WHERE `Predicate::InSubquery`, nested subqueries at arbitrary depth. **Projection-position `Expr::InSubquery` (`SELECT`, `GROUP BY`, `ORDER BY`) is NOT pre-registered** — `check_expr_insubquery_projection` rejects these queries with E-QUERY-043 before DataFusion planning (Option A, D-1650). Schema resolved via 3-priority ladder: (P1) live `TableRegistry`; (P2) bundled TOML spec schemas; (P3) JOIN-equality inference fallback. Spec-declared `datetime` columns mapped to Arrow `Timestamp(Microsecond, UTC)`. **After the ladder, `virtual_fields::append_virtual_fields_to_schema` appends `_sensor`, `_client`, `_source_table`, `_source_type` (all Utf8, nullable=true) with spoofed-column dedup guard; nullable=true enables LEFT JOIN NULL propagation on the empty-table side.** See BC-2.11.012 §Invariants "Empty MemTable schema parity". Result: graceful 0-row DataFusion execution; no `DataFusionError::Plan("table not found")`. 14 locking tests in `crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs` (11 original + T32/T33/T34 virtual-field parity tests). |
| EC-11-013 | Cache hit for some sensors, cache miss for others | Mix of cached and fresh data is valid; cache hits avoid API calls |
| EC-11-014 | A single sensor returns more than 10K records | Per-sensor API response pagination limits apply; the 10K limit is across all sensors combined |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| QueryPlan with 2 sensors, both return 500 records each | 1000 records materialized; MemTable registered | happy-path |
| QueryPlan where total fan-out reaches 10001 records | `Err(E-QUERY-005)` with per-sensor counts | error |
| QueryPlan where one of 3 sensors returns HTTP 503 | Partial results from 2 sensors; failed sensor in `sensor_errors` | edge-case |
| QueryPlan with all sensors returning empty | Empty MemTable; query returns empty result set | edge-case |
| INNER JOIN: left table has 3 rows, right table returns 0 batches | 0 rows returned; no `QueryExecutionFailed`; right MemTable pre-registered with P2/P3 schema | edge-case (DEC-022) — `test_BC_2_11_005_invariant_join_with_zero_batch_table_returns_empty_not_error` |
| LEFT JOIN: left has 3 rows, right returns 0 batches | 3 rows returned; right-side columns NULL; no error | edge-case (DEC-022) — `test_BC_2_11_005_left_join_zero_batch_right_table_returns_left_rows_with_nulls` |
| Solo SELECT on 0-batch table via `execute_against_session` | `Ok` with 0 rows; no plan error | edge-case (DEC-022) — `test_BC_2_01_010_solo_select_zero_batch_table_returns_empty_result_not_error` |
| LEFT JOIN selecting non-JOIN-key column from 0-batch right side | Left rows returned; non-key column NULL (not absent) | edge-case (DEC-022) — `test_BC_2_11_005_DEFECT_CSD_P1_002_T1_non_join_col_from_empty_side_returns_null` |
| SELECT * with 0-batch right side | Result schema includes ALL spec-declared columns, not just JOIN-key column | edge-case (DEC-022) — `test_BC_2_11_005_DEFECT_CSD_P1_002_T2_select_star_empty_side_returns_full_spec_schema` |
| INNER JOIN of two 0-batch tables | 0 rows; no plan error; both tables pre-registered from spec schemas | edge-case (DEC-022) — `test_BC_2_11_005_DEFECT_CSD_P1_002_T3_two_zero_batch_tables_joined_returns_empty` |
| 0-batch right side with spec-declared `datetime` columns | Result schema has Arrow `Timestamp(Microsecond, UTC)` columns; not absent | edge-case (DEC-022) — `test_BC_2_11_005_DEFECT_CSD_P1_002_T4_empty_side_datetime_cols_have_timestamp_type` |
| WHERE col IN (SELECT col FROM zero-batch table) | 0 rows; no plan error; IN-subquery table pre-registered via recursive walk | edge-case (DEC-022) — `test_BC_2_11_005_F_CSD_P3_001_T1_predicate_insubquery_empty_table_returns_empty_not_error` |
| SELECT (col IN (SELECT col FROM zero-batch table)) AS alias | `Err(E-QUERY-043 / ExprInSubqueryProjectionNotSupported)` — plan-time gate `check_expr_insubquery_projection` fires; no COUNT-rewrite; no pre-registration for projection position | edge-case (DEC-022) — `test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_returns_e_query_043_not_internal_error` |
| Nested depth-2 IN-subquery; both tables zero-batch | 0 rows; no plan error; recursive walk discovers both tables | edge-case (DEC-022) — `test_BC_2_11_005_F_CSD_P3_001_T3_nested_insubquery_depth2_both_empty_returns_empty_not_error` |
| WHERE non-key col IN (SELECT non-key col FROM zero-batch table) | 0 rows; no plan error; non-key col present via P2 spec schema | edge-case (DEC-022) — `test_BC_2_11_005_F_CSD_P3_001_T4_insubquery_nonkey_col_where_empty_table_returns_empty_not_error` |
| LEFT JOIN with empty right table; SELECT `_sensor` from right side | Right-side `_sensor` column is NULL (not absent); schema includes all three virtual fields via `append_virtual_fields_to_schema`; no schema-mismatch error | edge-case (DEC-022 / F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T32_left_join_empty_side_virtual_fields_return_null_not_error` |
| SELECT * from both populated and zero-batch table in same query; check schema parity | Both table schemas include `_sensor`, `_client`, `_source_table`; empty path uses nullable=true, populated path uses nullable=false | edge-case (DEC-022 / F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T33_select_star_empty_side_schema_includes_virtual_fields_parity` |
| LEFT JOIN populated-left / populated-right; both sides have virtual fields | Virtual fields present with nullable=false on both sides; non-null values returned | green-lock (F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T34_left_join_populated_side_virtual_fields_green_lock` |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-014 | Query security limits: rejects oversized queries | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-008, DI-019 |
| L2 Edge Cases | DEC-022, DEC-023 |
| Related BCs | BC-2.07.003 (cache), BC-2.02.002 (OCSF normalization), BC-2.01.013 v1.13 (SpecDrivenSensorAdapter OCSF Conformance Clause — spec-declared column survival and envelope derivation requirements; Pagination/Push-Down Scope Clause — push-down now active on first/query-plan step per S-DEMO-QUERY-PUSHDOWN-001; per-sensor translation table corrected in v1.13 per pushdown-redesign.md §6 + ADR-033) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.11 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P22-001 append_virtual_fields_to_schema four-field correction | 2026-07-10 | product-owner | **Closes F-CSD-P22-001 (MED) — BC-2.11.005 empty-path virtual-field enumeration stale (3 fields vs 4).** Code-verified against `virtual_fields::append_virtual_fields_to_schema` (`crates/prism-query/src/virtual_fields.rs`): the function appends four fields (`_sensor`, `_client`, `_source_table`, `_source_type`, all Utf8 nullable=true). BC-2.11.012 §Invariants "Empty MemTable schema parity" independently mandates all four. (1) **§Postconditions DEC-022 bullet:** `appends _sensor, _client, and _source_table` → `appends _sensor, _client, _source_table, and _source_type` (all Utf8, nullable=true). (2) **§Edge Cases DEC-022 row:** same correction. No change to Preconditions, Invariants, Error Cases, Canonical Test Vectors, Verification Properties, or Traceability. |
| 1.10 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P16-001 virtual-field spec anchor | 2026-07-10 | product-owner | **Closes F-CSD-P16-001 (MED) — empty-MemTable virtual-field behavior has no spec anchor.** (1) **§Postconditions DEC-022 bullet:** added explicit statement that after the 3-priority schema ladder resolves, `virtual_fields::append_virtual_fields_to_schema` appends `_sensor`, `_client`, `_source_table` (all Utf8, nullable=true) to every empty-table schema; named the spoofed-column dedup guard and the nullable=true LEFT JOIN NULL propagation rationale; added cross-reference to BC-2.11.012 §Invariants "Empty MemTable schema parity". (2) **§DEC-022 edge case row:** same virtual-field append step added inline; test count updated 11 → 14 to reflect T32/T33/T34. (3) **§Canonical Test Vectors:** added T32 (left-join empty-side virtual fields return NULL), T33 (SELECT * schema parity between empty and populated paths), T34 (populated-path green-lock). No change to Preconditions, Invariants, Error Cases, Verification Properties, or Traceability. |
| 1.9 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P5-003 + F-CSD-P5-004 spec correction — Option A adjudication propagation | 2026-07-10 | product-owner | **Corrections per Option A architect adjudication (D-1650 / F-CSD-P4-001, 2026-07-10) — closes F-CSD-P5-003 (MED) and F-CSD-P5-004 (MED).** (1) **§Postconditions empty-table pre-registration bullet:** removed "SELECT `Expr::InSubquery`" from the covered-positions list; added explicit statement that projection-position `Expr::InSubquery` (`SELECT`, `GROUP BY`, `ORDER BY`) is NOT pre-registered — plan-time gate `check_expr_insubquery_projection` rejects such queries with `E-QUERY-043 / ExprInSubqueryProjectionNotSupported` before DataFusion planning occurs. (2) **§DEC-022 edge case row:** same correction — removed "SELECT `Expr::InSubquery`" from the position list; added positive rejection statement and D-1650 Option A reference. DEC-022 title updated from "position-invariant schema-only MemTable registration" to "empty MemTable pre-registration for supported positions" to reflect the corrected scope. (3) **§Canonical Test Vectors T2 row (F-CSD-P5-003):** stale test name `..._T2_expr_insubquery_projection_empty_table_returns_false_col_not_error` and stale postcondition "All-false boolean column; no plan error; Expr::InSubquery table pre-registered" replaced with actual test name `test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_returns_e_query_043_not_internal_error` and correct postcondition `Err(E-QUERY-043 / ExprInSubqueryProjectionNotSupported)` — plan-time gate fires; no COUNT-rewrite; no pre-registration for projection position. **Note on v1.8:** v1.8's "SELECT `Expr::InSubquery`" in the pre-registration position list was erroneous — it encoded the COUNT-rewrite behavior that was subsequently REJECTED by architect adjudication D-1650. v1.8 is preserved as historical record; this v1.9 entry corrects the record. No change to Preconditions, Invariants, Error Cases, Traceability, or Verification Properties. |
| 1.8 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P4-003 DEC-022 enforcement-expansion spec amendment | 2026-07-10 | product-owner | DEC-022 semantics expanded to document the position-invariant empty-MemTable pre-registration added in DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Sub-defect 2 fix (fix/csdevices-empty-pipeline branch). **Postconditions:** new bullet documents `pre_register_empty_tables` — covers FROM, JOIN, WHERE `Predicate::InSubquery`, SELECT `Expr::InSubquery`, nested subqueries at arbitrary depth via `walk_sql_query`; 3-priority schema ladder (P1 live `TableRegistry` → P2 bundled TOML spec schemas via `BUNDLED_SPEC_SCHEMAS: OnceLock` → P3 JOIN-equality inference); spec-declared `datetime` → Arrow `Timestamp(Microsecond, UTC)` type fidelity. **DEC-022 edge case row:** expanded from one-line description to full semantics with ladder detail and result guarantee. **Canonical Test Vectors:** 11 locking tests added (INNER JOIN zero-batch, LEFT JOIN nulls, solo SELECT zero-batch, SELECT * full spec schema, datetime type fidelity, WHERE IN-subquery, Expr IN-subquery, depth-2 nested IN-subquery, non-key col IN-subquery) — all from `crates/prism-query/src/tests/defect_csdevices_empty_memtable_tests.rs`. Closes F-CSD-P4-003 (MED). No change to Preconditions, Invariants, Error Cases, Traceability, or Verification Properties. |
| 1.7 | QRY cascade P5-02 adjudication sweep (review-2026-06-10 PO micro-burst; error-taxonomy v1.71) | 2026-06-10 | product-owner | Record-cap error code resynced E-QUERY-003 → E-QUERY-005: the 10K streaming-counter condition migrated to `PrismError::QueryMaterializationLimitExceeded { count, max }` / E-QUERY-005 at QRY-01 (BC-2.11.006 v1.18 canonical mapping, taxonomy v1.68 D2 adjudication), but this BC's Error Cases row and the 10001-record canonical vector still cited E-QUERY-003 from the v1.4 reconciliation (which matched the then-current code). Error Cases row updated with the verbatim shipped variant + display per ADR-035 canonical-row convention; stale `§EC-003` cross-ref into BC-2.11.006 replaced with the canonical-mapping citation. E-QUERY-003 is now security-limits-only (P5-02; `QuerySecurityLimitExceeded`, `-32602`) — no condition in this BC qualifies. No postcondition/edge-case/invariant changes. |
| 1.6 | S-DEMO-QUERY-PUSHDOWN-001-v2-bc-respec | 2026-06-05 | product-owner | Cite-pin sweep: BC-2.01.013 v1.12 → v1.13 in Postconditions OCSF note (body line) and Traceability §Related BCs table. Amended note now references the corrected per-sensor translation spec in BC-2.01.013 v1.13 (CrowdStrike FQL via ADR-033 T1; Armis AQL passthrough; Cyberint/Claroty no native time-window). No semantic change to this BC's own postconditions or invariants. |
| 1.5 | D-924-bc-amendment | 2026-05-31 | product-owner | S-DEMO-001 adversary pass-2 F-001-R cross-reference: added forward-reference to BC-2.01.013 v1.8 OCSF Conformance Clause in the OCSF normalization postcondition bullet (spec-driven adapters must pass ColumnMapper + OcsfNormalizer; envelope-only RecordBatch is NON-CONFORMANT). Added BC-2.01.013 to Related BCs in Traceability. No semantic change to this BC's own invariants — the conformance detail lives in BC-2.01.013; this BC carries the pointer. |
| 1.4 | PR-129-pass-1 | 2026-05-06 | product-owner | Adversary F-PR129-PR-MED-A remediation: error-code reconciliation per BC-2.11.006 v1.12 canonical SoT mapping. E-QUERY-005 (timeout) → E-QUERY-003 (records limit) for the 10K record cap rows. Implementation already emits E-QUERY-003 at materialization.rs:186; this BC update closes spec↔code drift. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
