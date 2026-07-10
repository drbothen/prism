//! RED gate tests for DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Sub-defect 2
//! (empty MemTable registration / JOIN "Internal error").
//!
//! # Defect summary (from root-cause report)
//!
//! `register_mem_table` silently skips registration when `batches.is_empty()`.
//! When a mixed JOIN is attempted (one side has data, the other returned 0 batches),
//! DataFusion planning fails with `DataFusionError::Plan("table 'X' not found")`,
//! which the engine's catch-all maps to `PrismError::QueryExecutionFailed`
//! (`-32000 "Internal error"` to the MCP caller).
//!
//! # Ratified fix (D-1650 §Track B)
//!
//! Register a **schema-only empty `MemTable`** when the sensor result batch count
//! is 0. This allows DataFusion to plan a JOIN against a known schema, returning
//! 0 rows gracefully rather than erroring.
//!
//! # BC anchors
//!
//! - BC-2.11.005 edge case DEC-022: "All sensor API calls return empty" →
//!   "Empty RecordBatch registered; query returns empty result set"
//! - BC-2.01.010: empty result ≠ error (partial-failure handling)
//! - BC-2.11.005 canonical test vector: "QueryPlan where one of 3 sensors returns
//!   HTTP 503" → "Partial results from 2 sensors" (analogous: 1 sensor returns 0
//!   rows; the other sensor's query must still plan and execute).
//!
//! # Test inventory
//!
//! ## Test 1 — mixed JOIN: one populated table + one 0-batch table → 0 rows (no error)
//!
//! `test_BC_2_11_005_invariant_join_with_zero_batch_table_returns_empty_not_error`
//!
//! Simulates: `armis_devices` registered with data; `crowdstrike_devices` returns
//! 0 batches (so `register_mem_table` skips it). The JOIN SQL must return `Ok`
//! with 0 rows — not a DataFusion plan error.
//!
//! RED: `execute_against_session` returns `Err(PrismError::QueryExecutionFailed)`
//! because DataFusion cannot find `crowdstrike_devices` in the catalog.
//! PASSES after the fix registers a schema-only empty MemTable for 0-batch sources.
//!
//! ## Test 2 — LEFT JOIN: populated table LEFT JOIN empty table → rows from left, NULLs right
//!
//! `test_BC_2_11_005_left_join_zero_batch_right_table_returns_left_rows_with_nulls`
//!
//! More realistic production scenario: user queries
//!   `crowdstrike_detections LEFT JOIN crowdstrike_devices ON ...`
//! when the devices pipeline returns 0 rows. Must return all detection rows with
//! NULL device columns, not an error.
//!
//! RED: same DataFusion plan error as Test 1.
//!
//! ## Test 3 — solo SELECT on 0-batch table → 0 rows with correct schema (no error)
//!
//! `test_BC_2_01_010_solo_select_zero_batch_table_returns_empty_result_not_error`
//!
//! User queries `FROM crowdstrike_devices` when the devices sensor returns 0 rows.
//! Per BC-2.01.010 and BC-2.11.005 DEC-022: empty ≠ error; must return
//! `Ok` with 0 rows.
//!
//! RED: DataFusion plan error because the table was not registered.
//!
//! ---
//!
//! ## F-CSD-P1-002 (HIGH) — spec-column empty MemTable contract
//!
//! Tests 4-7 encode the FULL contract from D-1650 §Track B: the schema-only empty
//! MemTable must be built from the SENSOR SPEC's declared columns, not inferred
//! from JOIN-equality peer columns only. The current implementation (`pre_register_empty_tables_for_joins`
//! v1) passes Tests 1-3 but fails Tests 4-7 because it infers at most the JOIN key
//! column and cannot satisfy queries on non-JOIN-equality or datetime-typed columns.
//!
//! ## Test 4 — non-JOIN column from empty side: SELECT hostname must return NULL, not error
//!
//! `test_BC_2_11_005_DEFECT_CSD_P1_002_T1_non_join_col_from_empty_side_returns_null`
//!
//! Live demo symptom: `SELECT det.detection_id, dev.hostname ... LEFT JOIN crowdstrike_devices dev ...`
//! with devices=0 batches. `hostname` is NOT in the inferred schema (only `device_id`
//! from JOIN equality). DataFusion planning fails on `dev.hostname` → QueryExecutionFailed.
//!
//! RED: `result.is_ok()` fails — DataFusion cannot find `hostname`.
//! PASSES after spec-column fix: all 6 declared columns available; hostname=NULL per row.
//!
//! ## Test 5 — SELECT * schema width: full 6-column spec schema, not just join-key
//!
//! `test_BC_2_11_005_DEFECT_CSD_P1_002_T2_select_star_empty_side_returns_full_spec_schema`
//!
//! SELECT * LEFT JOIN with 0-batch right side: the result schema must include all 6
//! spec-declared devices columns (device_id, hostname, platform_name, status, first_seen,
//! last_seen), not just the JOIN-equality column (device_id).
//!
//! RED: result is Ok (SELECT * with {device_id} schema runs), but `schema.index_of("hostname")`
//! fails — hostname absent from the inference-only schema.
//!
//! ## Test 6 — two 0-batch tables joined: must plan and return empty, not error
//!
//! `test_BC_2_11_005_DEFECT_CSD_P1_002_T3_two_zero_batch_tables_joined_returns_empty`
//!
//! Both sides of the JOIN have 0 batches. Current `pre_register_empty_tables_for_joins`:
//! processes the FROM-table first (devices), looks up the JOIN peer (detections) — also
//! not yet registered — gets no schema hint → Schema::empty(). Second pass for detections:
//! peer (devices) is now registered with Schema::empty() — `field_with_name("device_id")`
//! fails — still no hint. Both tables get Schema::empty(). JOIN ON device_id fails.
//!
//! RED: `result.is_ok()` fails — neither empty schema has the join key column.
//! PASSES after spec-column fix: both tables get their full spec schemas.
//!
//! ## Test 7 — type fidelity: spec datetime columns must be Timestamp, not inferred String
//!
//! `test_BC_2_11_005_DEFECT_CSD_P1_002_T4_empty_side_datetime_cols_have_timestamp_type`
//!
//! The `crowdstrike_devices` spec declares `first_seen` and `last_seen` as
//! `column_type = "datetime"` → Arrow `Timestamp(Microsecond, UTC)`.
//! With inference-only approach, these columns are absent from the schema entirely
//! (not JOIN-equality columns) → query on them fails → QueryExecutionFailed.
//!
//! RED: `result.is_ok()` fails — `first_seen` not in inferred schema.
//! PASSES after spec-column fix AND type mapping: Timestamp type asserted post-fix.
//!
//! # Red Gate (BC-5.38.001)
//!
//! Tests 1-3: PASS after the inference-based fix (`pre_register_empty_tables_for_joins` v1).
//! Tests 4-7: FAIL against inference-based fix; PASS after spec-column fix (D-1650 §Track B).
//! Failure mode for 4-7: QueryExecutionFailed (DataFusion cannot resolve non-JOIN columns)
//! or assertion on schema field names/types fails.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::{
        array::{Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };

    use crate::{
        filter_parser::PrismQlParser,
        materialization::{execute_against_session, register_mem_table},
        memory::build_session_context,
    };

    // -----------------------------------------------------------------------
    // Fixtures
    // -----------------------------------------------------------------------

    /// Build a RecordBatch with a single `Utf8` column.
    fn make_batch(col: &str, values: &[&str]) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new(col, DataType::Utf8, true)]));
        let col_arr = Arc::new(StringArray::from(values.to_vec())) as _;
        RecordBatch::try_new(schema, vec![col_arr]).expect("batch build must succeed")
    }

    /// Build a two-column RecordBatch — simulates a sensor table with two string fields.
    fn make_two_col_batch(
        col_a: &str,
        vals_a: &[&str],
        col_b: &str,
        vals_b: &[&str],
    ) -> RecordBatch {
        assert_eq!(
            vals_a.len(),
            vals_b.len(),
            "test data: column lengths must match"
        );
        let schema = Arc::new(Schema::new(vec![
            Field::new(col_a, DataType::Utf8, true),
            Field::new(col_b, DataType::Utf8, true),
        ]));
        let arr_a = Arc::new(StringArray::from(vals_a.to_vec())) as _;
        let arr_b = Arc::new(StringArray::from(vals_b.to_vec())) as _;
        RecordBatch::try_new(schema, vec![arr_a, arr_b]).expect("two-col batch build must succeed")
    }

    // -----------------------------------------------------------------------
    // Test 1: Mixed JOIN — one populated table + one 0-batch table → 0 rows
    //
    // BC-2.11.005 DEC-022 / BC-2.01.010 empty-is-not-error invariant.
    //
    // Setup:
    //   - `armis_devices` registered with 2 rows (non-empty → `register_mem_table` wires it)
    //   - `crowdstrike_devices` receives empty batch list → `register_mem_table` SKIPS it
    //   - SQL: INNER JOIN crowdstrike_devices → should plan successfully, return 0 rows
    //
    // RED: `execute_against_session` returns `Err(QueryExecutionFailed)` because
    //      DataFusion can't find `crowdstrike_devices` in the catalog.
    //      Assertion `result.is_ok()` fails.
    //
    // PASSES after fix: empty-batch sources get a schema-only MemTable registered so
    //      DataFusion can plan the JOIN, returning 0 rows.
    // -----------------------------------------------------------------------

    /// BC-2.11.005 / BC-2.01.010: INNER JOIN where one table returned 0 batches must
    /// plan and execute successfully, returning 0 rows — not a DataFusion plan error.
    ///
    /// RED: `Err(QueryExecutionFailed { detail: "...crowdstrike_devices..." })` because
    ///      `register_mem_table` skips empty batches and DataFusion cannot plan the JOIN.
    #[tokio::test]
    async fn test_BC_2_11_005_invariant_join_with_zero_batch_table_returns_empty_not_error() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // armis_devices has data — registered normally.
        let armis_batch = make_batch("device_id", &["armis-001", "armis-002"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // crowdstrike_devices returned 0 batches — register_mem_table skips it.
        // This is the exact production path: empty Vec<RecordBatch> → skip.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error (it skips silently)");

        // At this point: armis_devices IS registered; crowdstrike_devices is NOT.
        // Verify the gap to confirm the RED precondition holds.
        let armis_registered = ctx
            .table_exist("armis_devices")
            .expect("table_exist must not error");
        let cs_registered = ctx
            .table_exist("crowdstrike_devices")
            .expect("table_exist must not error");
        assert!(
            armis_registered,
            "test setup: armis_devices must be registered"
        );
        // This assertion PASSES (confirms the pre-fix bug is present):
        assert!(
            !cs_registered,
            "test setup: crowdstrike_devices must NOT be registered before fix \
             (register_mem_table skips empty batches)"
        );

        let sql = "SELECT a.device_id FROM armis_devices a \
                   INNER JOIN crowdstrike_devices c ON a.device_id = c.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED behavior (post-fix): Ok with 0 rows.
        // RED assertion — this fails before the fix because result is Err.
        assert!(
            result.is_ok(),
            "BC-2.11.005 / BC-2.01.010: JOIN with 0-batch table must return Ok (0 rows), \
             not DataFusion plan error. \
             RED: currently Err(QueryExecutionFailed) — crowdstrike_devices not in catalog. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "BC-2.11.005: JOIN with empty right side must return 0 rows; got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 2: LEFT JOIN — populated left + 0-batch right → left rows with NULLs
    //
    // More realistic user query: "list all detections, with device info where available."
    // When crowdstrike_devices returned 0 rows for THIS query, a LEFT JOIN must
    // return all detection rows with NULL for the device columns — not an error.
    //
    // RED: same DataFusion plan error as Test 1.
    // -----------------------------------------------------------------------

    /// BC-2.11.005 / BC-2.01.010: LEFT JOIN where the right side returned 0 batches
    /// must return the left side rows with NULL right-side columns — not an error.
    ///
    /// Models the production query:
    ///   `crowdstrike_detections LEFT JOIN crowdstrike_devices ON ...`
    /// when the devices two-step pipeline returns 0 rows (e.g., step 1 returns
    /// 0 device IDs → step 2 is skipped → 0 batches materialized for devices).
    ///
    /// RED: `Err(QueryExecutionFailed)` — crowdstrike_devices not in catalog.
    #[tokio::test]
    async fn test_BC_2_11_005_left_join_zero_batch_right_table_returns_left_rows_with_nulls() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections has data (left side of the LEFT JOIN).
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-A"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices has 0 batches — register_mem_table skips it.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        let sql = "SELECT d.detection_id, d.device_id \
                   FROM crowdstrike_detections d \
                   LEFT JOIN crowdstrike_devices c ON d.device_id = c.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED behavior: Ok with 3 rows (one per detection; device columns NULL).
        // RED: Err(QueryExecutionFailed) because crowdstrike_devices not in catalog.
        assert!(
            result.is_ok(),
            "BC-2.11.005 / BC-2.01.010: LEFT JOIN with 0-batch right side must return Ok, \
             not DataFusion plan error. \
             RED: currently Err — crowdstrike_devices missing from DataFusion catalog. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        // LEFT JOIN: all 3 detection rows must appear; device columns are NULL.
        assert_eq!(
            total_rows, 3,
            "BC-2.11.005: LEFT JOIN with empty right side must return 3 rows \
             (all left-side detections with NULL device columns); got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: Solo SELECT on 0-batch table → 0 rows with correct schema (no error)
    //
    // BC-2.11.005 DEC-022 canonical test vector:
    //   "QueryPlan with all sensors returning empty" →
    //   "Empty RecordBatch registered; query returns empty result set"
    //
    // This is the non-JOIN case: a direct `FROM crowdstrike_devices` query when
    // the devices pipeline returned 0 rows.
    //
    // RED: DataFusion plan error (table not in catalog) because register_mem_table
    //      skips empty batches.
    //
    // Note: the solo-SELECT case reaches `execute_against_session` only when at
    // least one external table is registered (the `!any_external_table_registered`
    // early-return path in `run_materialization_pipeline` would short-circuit it).
    // For the unit test we call `execute_against_session` directly to isolate the
    // DataFusion-planning contract.
    // -----------------------------------------------------------------------

    /// BC-2.11.005 DEC-022 / BC-2.01.010: Direct SELECT on a table that returned
    /// 0 batches must return `Ok` with 0 rows — not a DataFusion plan error.
    ///
    /// Exercises the `execute_against_session` path directly (bypassing the
    /// `!any_external_table_registered` early-return in `run_materialization_pipeline`)
    /// to isolate the DataFusion-registration contract.
    ///
    /// RED: `Err(QueryExecutionFailed)` — `crowdstrike_devices` not registered.
    #[tokio::test]
    async fn test_BC_2_01_010_solo_select_zero_batch_table_returns_empty_result_not_error() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Simulate the 0-batch case: register_mem_table is called but silently skips.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // Confirm pre-fix state: table is absent from the catalog.
        let registered = ctx
            .table_exist("crowdstrike_devices")
            .expect("table_exist must not error");
        assert!(
            !registered,
            "test setup: crowdstrike_devices must NOT be registered before fix \
             (register_mem_table skips empty batches — confirmed)"
        );

        let sql = "SELECT * FROM crowdstrike_devices";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED behavior: Ok with 0 rows.
        // RED: Err(QueryExecutionFailed { detail: "...crowdstrike_devices..." }).
        assert!(
            result.is_ok(),
            "BC-2.01.010 / BC-2.11.005 DEC-022: SELECT on 0-batch table must return \
             Ok with 0 rows, not DataFusion plan error. \
             RED: currently Err because crowdstrike_devices not in DataFusion catalog. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "BC-2.11.005 DEC-022: empty-sensor query must return 0 rows; got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // F-CSD-P1-002 Tests 4-7 (RED against inference-based fix; GREEN after spec-column fix)
    //
    // The inference-based `pre_register_empty_tables_for_joins` passes Tests 1-3 but
    // fails Tests 4-7: it infers at most the JOIN-equality column (`device_id: Utf8`)
    // and cannot satisfy queries on non-JOIN columns or datetime-typed columns.
    //
    // D-1650 §Track B: the correct fix uses the sensor spec's declared columns to
    // build the empty MemTable schema (all 6 crowdstrike_devices columns with correct
    // Arrow types). Tests 4-7 encode this contract.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 4 (F-CSD-P1-002-T1): Non-JOIN column from empty side must return NULL,
    // not QueryExecutionFailed.
    //
    // Live demo symptom: SELECT det.detection_id, dev.hostname with devices=0 batches.
    //
    // Current inference: crowdstrike_devices gets schema `{device_id: Utf8}`.
    // DataFusion plan fails: `dev.hostname` not in `{device_id: Utf8}` →
    //   QueryExecutionFailed.
    //
    // DESIRED (post-fix): Ok with 3 rows; hostname column all NULL.
    // -----------------------------------------------------------------------

    /// F-CSD-P1-002-T1 / BC-2.11.005: LEFT JOIN selecting a non-JOIN-equality column
    /// from the 0-batch right side must return left rows with NULL — not error.
    ///
    /// RED: `Err(QueryExecutionFailed)` — `hostname` absent from inferred `{device_id}` schema.
    /// GREEN (post-fix): 3 rows returned with all hostname values NULL.
    #[tokio::test]
    async fn test_BC_2_11_005_DEFECT_CSD_P1_002_T1_non_join_col_from_empty_side_returns_null() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — 3 rows with detection_id + device_id (left side, populated).
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-A"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches (simulates empty two-step pipeline result).
        // pre_register_empty_tables_for_joins infers `{device_id: Utf8}` from the JOIN ON
        // clause — but `hostname` is not in that inferred schema.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // SELECT `dev.hostname` — a non-JOIN-equality column absent from the inferred schema.
        let sql = "SELECT det.detection_id, dev.hostname \
                   FROM crowdstrike_detections det \
                   LEFT JOIN crowdstrike_devices dev ON det.device_id = dev.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 3 rows; hostname column all NULL.
        // RED: Err(QueryExecutionFailed) — `No field named dev.hostname` because
        //      pre_register_empty_tables_for_joins inferred only `{device_id: Utf8}`.
        assert!(
            result.is_ok(),
            "F-CSD-P1-002-T1 / BC-2.11.005: SELECT on non-JOIN column from 0-batch \
             left-joined table must return Ok (left rows + NULL right columns), not error. \
             RED: currently Err — `hostname` absent from inference-only schema `{{device_id: Utf8}}`. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 3,
            "F-CSD-P1-002-T1: LEFT JOIN with 0-batch right side must return 3 rows \
             (one per detection); got {total_rows}"
        );

        // All hostname values must be NULL (empty side contributes no data).
        let non_empty: Vec<_> = batches.iter().filter(|b| b.num_rows() > 0).collect();
        assert!(
            !non_empty.is_empty(),
            "F-CSD-P1-002-T1: at least one non-empty batch expected after fix"
        );
        let hostname_col = non_empty[0]
            .column_by_name("hostname")
            .expect("hostname column must be present in result schema after fix");
        assert_eq!(
            hostname_col.null_count(),
            hostname_col.len(),
            "F-CSD-P1-002-T1: all hostname values must be NULL (right side is empty); \
             got {}/{} nulls",
            hostname_col.null_count(),
            hostname_col.len()
        );
    }

    // -----------------------------------------------------------------------
    // Test 5 (F-CSD-P1-002-T2): SELECT * schema width — all 6 spec columns required.
    //
    // SELECT * LEFT JOIN with 0-batch right side.
    // Current inference: crowdstrike_devices gets `{device_id: Utf8}`.
    // Result schema: only `device_id` from the devices side (plus detections columns).
    // `schema.index_of("hostname")` FAILS → RED.
    //
    // DESIRED (post-fix): full 6-column spec schema in result
    //   (device_id, hostname, platform_name, status, first_seen, last_seen).
    // -----------------------------------------------------------------------

    /// F-CSD-P1-002-T2 / BC-2.11.005: SELECT * across a LEFT JOIN with 0-batch right side
    /// must include ALL spec-declared columns of the empty table — not just the JOIN-key.
    ///
    /// RED: result is Ok (SELECT * with `{device_id}` schema executes), but
    ///      `schema.index_of("hostname")` fails — inference-only schema has only `device_id`.
    /// GREEN (post-fix): schema includes all 6 spec-declared devices columns.
    #[tokio::test]
    async fn test_BC_2_11_005_DEFECT_CSD_P1_002_T2_select_star_empty_side_returns_full_spec_schema()
    {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — 2 rows (left side, populated).
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002"],
            "device_id",
            &["dev-A", "dev-B"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches (right side, empty).
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // SELECT * — must include ALL 6 spec-declared devices columns in result schema.
        let sql = "SELECT * FROM crowdstrike_detections det \
                   LEFT JOIN crowdstrike_devices dev ON det.device_id = dev.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // With inference-based fix, result is Ok (SELECT * + {device_id} schema plans fine).
        // The RED failure is the schema width assertion below, not result.is_ok().
        assert!(
            result.is_ok(),
            "F-CSD-P1-002-T2 / BC-2.11.005: SELECT * LEFT JOIN with 0-batch right side \
             must execute without error. got: {result:?}"
        );

        let batches = result.unwrap();
        assert!(
            !batches.is_empty(),
            "F-CSD-P1-002-T2: at least one batch expected (even if 0 rows)"
        );
        let schema = batches[0].schema();

        // RED assertion: `hostname` absent from inference-only schema `{device_id: Utf8}`.
        // PASSES after spec-column fix: schema has hostname (plus 4 other spec columns).
        assert!(
            schema.index_of("hostname").is_ok(),
            "F-CSD-P1-002-T2: result schema must include `hostname` (spec-declared column \
             from crowdstrike_devices) — inference-only schema has only `device_id`. \
             RED: hostname absent from result schema. \
             actual schema fields: {:?}",
            schema.fields().iter().map(|f| f.name()).collect::<Vec<_>>()
        );

        // Post-fix lock: ALL 6 spec-declared devices columns must be present.
        // (crowdstrike.sensor.toml declares: device_id, hostname, platform_name,
        //  status, first_seen, last_seen)
        for col in &[
            "hostname",
            "platform_name",
            "status",
            "first_seen",
            "last_seen",
        ] {
            assert!(
                schema.index_of(col).is_ok(),
                "F-CSD-P1-002-T2: result schema must include spec-declared column `{col}`"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 6 (F-CSD-P1-002-T3): Two 0-batch tables joined — must plan and
    // return empty result, not error.
    //
    // Both crowdstrike_devices AND crowdstrike_detections have 0 batches.
    // Inference: both get Schema::empty() (no registered peer to infer from).
    // JOIN ON device_id fails: device_id absent from Schema::empty().
    //
    // DESIRED (post-fix): Ok with 0 rows.
    // -----------------------------------------------------------------------

    /// F-CSD-P1-002-T3 / BC-2.11.005: INNER JOIN of two 0-batch tables must plan
    /// and return an empty result — not a DataFusion plan error.
    ///
    /// RED: `Err(QueryExecutionFailed)` — both tables get `Schema::empty()` because no
    ///      registered peer can supply a schema hint; JOIN ON device_id fails.
    /// GREEN (post-fix): spec columns available for both → plan succeeds, 0 rows returned.
    #[tokio::test]
    async fn test_BC_2_11_005_DEFECT_CSD_P1_002_T3_two_zero_batch_tables_joined_returns_empty() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Both tables have 0 batches — register_mem_table skips both.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");
        register_mem_table(&ctx, "crowdstrike_detections", vec![])
            .expect("register_mem_table with empty batches must not error");

        // INNER JOIN across two empty tables: must plan and return 0 rows.
        // With both tables unregistered (or registered with Schema::empty()), DataFusion
        // cannot resolve `dev.device_id` in the JOIN ON clause → plan fails.
        let sql = "SELECT dev.device_id, det.detection_id \
                   FROM crowdstrike_devices dev \
                   INNER JOIN crowdstrike_detections det ON dev.device_id = det.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 0 rows.
        // RED: Err(QueryExecutionFailed) — inference gives both tables Schema::empty(),
        //      JOIN ON device_id fails (field not found in either schema).
        assert!(
            result.is_ok(),
            "F-CSD-P1-002-T3 / BC-2.11.005: INNER JOIN of two 0-batch tables must return \
             Ok (0 rows), not DataFusion plan error. \
             RED: currently Err — inference yields Schema::empty() for both tables; \
             JOIN ON device_id cannot be resolved. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "F-CSD-P1-002-T3: INNER JOIN of two empty tables must return 0 rows; \
             got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 7 (F-CSD-P1-002-T4): Spec-declared datetime columns must be
    // Timestamp-typed in the empty-side schema — not absent or inferred as String.
    //
    // crowdstrike_devices spec: first_seen, last_seen → `column_type = "datetime"`.
    // With inference-only: these columns absent from `{device_id: Utf8}`.
    // DataFusion plan fails on `dev.first_seen` → QueryExecutionFailed.
    //
    // DESIRED (post-fix): Arrow DataType::Timestamp(_, _) for both columns.
    // -----------------------------------------------------------------------

    /// F-CSD-P1-002-T4 / BC-2.11.005: Spec-declared `datetime` columns on the 0-batch
    /// side must be present in the result schema as Arrow Timestamp type — not absent.
    ///
    /// RED: `Err(QueryExecutionFailed)` — `first_seen`/`last_seen` absent from the
    ///      inference-only schema `{device_id: Utf8}`.
    /// GREEN (post-fix): columns present with `DataType::Timestamp(_, _)` type.
    #[tokio::test]
    async fn test_BC_2_11_005_DEFECT_CSD_P1_002_T4_empty_side_datetime_cols_have_timestamp_type() {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — 1 row (left side, drives the LEFT JOIN).
        let det_batch = make_two_col_batch("detection_id", &["det-001"], "device_id", &["dev-A"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches (right side, empty).
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // SELECT the spec-declared datetime columns from the 0-batch side.
        // crowdstrike.sensor.toml: first_seen = "datetime", last_seen = "datetime".
        let sql = "SELECT dev.first_seen, dev.last_seen \
                   FROM crowdstrike_detections det \
                   LEFT JOIN crowdstrike_devices dev ON det.device_id = dev.device_id";
        let ast = PrismQlParser::parse(sql).expect("SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 1 row; first_seen/last_seen are NULL Timestamp columns.
        // RED: Err(QueryExecutionFailed) — `first_seen`/`last_seen` absent from the
        //      inference-only schema (only `device_id` was inferred from the JOIN ON clause).
        assert!(
            result.is_ok(),
            "F-CSD-P1-002-T4 / BC-2.11.005: SELECT on spec-declared datetime columns from \
             0-batch table must return Ok (NULL Timestamp columns), not error. \
             RED: currently Err — `first_seen`/`last_seen` absent from inference-only schema \
             `{{device_id: Utf8}}`; DataFusion cannot plan the query. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        // At least one batch must exist (even if rows = 0) to inspect the schema.
        assert!(
            !batches.is_empty(),
            "F-CSD-P1-002-T4: at least one batch expected to verify schema types"
        );
        let schema = batches[0].schema();

        // Type fidelity: spec `datetime` → Arrow `Timestamp(_, _)`.
        let first_seen_type = schema
            .field_with_name("first_seen")
            .expect("first_seen must be present in result schema after spec-column fix")
            .data_type()
            .clone();
        assert!(
            matches!(first_seen_type, DataType::Timestamp(_, _)),
            "F-CSD-P1-002-T4: `first_seen` must have Arrow Timestamp type \
             (spec declares `column_type = \"datetime\"`); got {first_seen_type:?}"
        );

        let last_seen_type = schema
            .field_with_name("last_seen")
            .expect("last_seen must be present in result schema after spec-column fix")
            .data_type()
            .clone();
        assert!(
            matches!(last_seen_type, DataType::Timestamp(_, _)),
            "F-CSD-P1-002-T4: `last_seen` must have Arrow Timestamp type \
             (spec declares `column_type = \"datetime\"`); got {last_seen_type:?}"
        );
    }
}
