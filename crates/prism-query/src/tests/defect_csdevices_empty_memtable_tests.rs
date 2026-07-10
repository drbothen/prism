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

    // -----------------------------------------------------------------------
    // F-CSD-P3-001 Tests 8-11 (RED at HEAD 4a19a22d — LOCAL adversary pass-3)
    //
    // Finding: `pre_register_empty_tables_for_joins` (materialization.rs) builds
    // `all_table_names` from ONLY `sql_query.from` + `sql_query.joins` (lines
    // 2905-2916). It does NOT recurse into subqueries. `walk_sql_query` (the
    // `collect_external_table_names` helper) DOES recurse, but
    // `pre_register_empty_tables_for_joins` is independent and does not call it.
    //
    // Consequence: any 0-batch table referenced EXCLUSIVELY inside an IN-subquery
    // (at any position: WHERE, SELECT projection, nested depth ≥ 2, or subquery
    // WHERE referencing non-key columns) is never added to `all_table_names`, never
    // pre-registered, and DataFusion plan fails with "table not found" →
    // `PrismError::QueryExecutionFailed` (E-QUERY internal error to the MCP caller).
    //
    // Defect class: BC-2.11.005 DEC-022 / BC-2.01.010 (empty ≠ error),
    // position-invariant. The existing fix (Tests 1-7) closed FROM/JOIN positions;
    // these 4 tests lock the IN-subquery positions.
    //
    // All 4 tests FAIL at HEAD for the documented reason; ALL must pass after the
    // implementer extends `pre_register_empty_tables_for_joins` with recursive
    // subquery walking.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 8 (F-CSD-P3-001-T1): Predicate-position WHERE IN-subquery — primary exploit
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)
    //
    // crowdstrike_devices appears ONLY in the WHERE IN-subquery. The outer query
    // has no JOINs referencing it. `pre_register_empty_tables_for_joins` processes
    // only `crowdstrike_detections` (from FROM) → `crowdstrike_devices` never added
    // to `all_table_names` → not pre-registered → DataFusion plan error.
    //
    // DESIRED (post-fix): Ok with 0 rows — empty IN-set → no detection rows match.
    // RED: Err(QueryExecutionFailed) — "table not found: crowdstrike_devices".
    // -----------------------------------------------------------------------

    /// F-CSD-P3-001-T1 / BC-2.11.005 / BC-2.01.010: WHERE IN-subquery referencing
    /// a 0-batch table must return Ok (0 rows), not a DataFusion plan error.
    ///
    /// Primary exploit from LOCAL adversary pass-3 finding F-CSD-P3-001 (HIGH):
    /// `pre_register_empty_tables_for_joins` scans only FROM + JOIN positions in
    /// the outer query; a table referenced exclusively inside a WHERE IN-subquery
    /// is invisible to it and never pre-registered.
    ///
    /// RED: `Err(QueryExecutionFailed)` — DataFusion cannot find `crowdstrike_devices`
    ///      because it was only referenced inside the IN-subquery, not in the outer
    ///      FROM/JOIN list that `pre_register_empty_tables_for_joins` processes.
    #[tokio::test]
    async fn test_BC_2_11_005_F_CSD_P3_001_T1_predicate_insubquery_empty_table_returns_empty_not_error(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data (left side of the query).
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-A"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches; register_mem_table silently skips it.
        // ONLY referenced inside the IN-subquery, NOT in outer FROM/JOIN.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // Confirm pre-fix state: crowdstrike_devices absent from the DataFusion catalog.
        let cs_registered = ctx
            .table_exist("crowdstrike_devices")
            .expect("table_exist must not error");
        assert!(
            !cs_registered,
            "test setup: crowdstrike_devices must NOT be registered before fix \
             (register_mem_table skips empty batches — confirmed)"
        );

        // crowdstrike_devices is referenced ONLY in the IN-subquery.
        // pre_register_empty_tables_for_joins builds all_table_names from
        // `sql_query.from` (crowdstrike_detections) + `sql_query.joins` (none).
        // crowdstrike_devices is never added to all_table_names → not pre-registered.
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)";
        let ast = PrismQlParser::parse(sql).expect("predicate IN-subquery SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 0 rows — empty IN-set yields 0 matching detections.
        // RED: Err(QueryExecutionFailed) — DataFusion: "table not found: crowdstrike_devices"
        //      because pre_register_empty_tables_for_joins did not recurse into
        //      the IN-subquery to discover and pre-register crowdstrike_devices.
        assert!(
            result.is_ok(),
            "F-CSD-P3-001-T1 / BC-2.11.005 / BC-2.01.010: WHERE IN-subquery referencing \
             a 0-batch table must return Ok (0 rows), not a DataFusion plan error. \
             RED: Err — crowdstrike_devices invisible to pre_register_empty_tables_for_joins \
             (only outer FROM/JOIN positions are scanned; IN-subquery positions are skipped). \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "F-CSD-P3-001-T1: empty IN-set (crowdstrike_devices has 0 rows) → \
             0 matching detection rows; got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 9 (F-CSD-P3-001-T2): Expression-position IN-subquery (SELECT projection)
    //
    // Grammar-reach note: PrismQlParser parses projection-position IN-subquery of
    // the form `SELECT (col IN (SELECT col FROM t)) AS alias FROM outer_t`.
    // Confirmed by `test_med1_expr_insubquery_select_projection_temporal_folded`
    // (high002_plan_pinning_tests.rs:907) which parses:
    //   `SELECT (host_id IN (SELECT host_id FROM armis_alerts WHERE ...)) AS flagged
    //    FROM crowdstrike_detections`
    // Grammar reach: CONFIRMED — test uses the same (col IN (SELECT ...)) AS alias form.
    //
    // SQL: SELECT (det.device_id IN (SELECT device_id FROM crowdstrike_devices)) AS is_known
    //      FROM crowdstrike_detections det
    //
    // crowdstrike_devices appears ONLY in the SELECT projection Expr::InSubquery.
    // Same gap: pre_register_empty_tables_for_joins does not process Expr nodes
    // in the SELECT projection list.
    //
    // DESIRED (post-fix): Ok with 3 rows; is_known column all false (empty IN-set).
    // RED: Err(QueryExecutionFailed) — "table not found: crowdstrike_devices".
    // -----------------------------------------------------------------------

    /// F-CSD-P3-001-T2 / BC-2.11.005 / BC-2.01.010: Expression-position IN-subquery
    /// in SELECT projection referencing a 0-batch table must return Ok (all-false
    /// column), not a DataFusion plan error.
    ///
    /// Grammar reach: CONFIRMED by `test_med1_expr_insubquery_select_projection_temporal_folded`
    /// (high002_plan_pinning_tests.rs:907). The `(col IN (SELECT col FROM t)) AS alias`
    /// form is a legal PrismQL projection expression.
    ///
    /// RED: `Err(QueryExecutionFailed)` — `crowdstrike_devices` not in catalog because
    ///      `pre_register_empty_tables_for_joins` does not walk `Expr::InSubquery`
    ///      nodes in the SELECT projection list.
    #[tokio::test]
    async fn test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_empty_table_returns_false_col_not_error(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data.
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-A"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches; ONLY referenced in the SELECT projection subquery.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // Grammar-reach verified (see doc comment): (col IN (SELECT col FROM t)) AS alias
        // is a legal PrismQL projection expression.
        let sql =
            "SELECT (det.device_id IN (SELECT device_id FROM crowdstrike_devices)) AS is_known \
                   FROM crowdstrike_detections det";
        let ast = PrismQlParser::parse(sql)
            .expect("projection-position IN-subquery SQL must parse (grammar reach confirmed)");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 3 rows; is_known all false (empty IN-set).
        // RED: Err(QueryExecutionFailed) — DataFusion: "table not found: crowdstrike_devices"
        //      because pre_register_empty_tables_for_joins does not walk Expr::InSubquery
        //      nodes in the SELECT projection list.
        assert!(
            result.is_ok(),
            "F-CSD-P3-001-T2 / BC-2.11.005: Projection-position IN-subquery (SELECT clause) \
             with 0-batch table must return Ok (all-false is_known column), not plan error. \
             RED: Err — crowdstrike_devices not pre-registered; Expr::InSubquery in SELECT \
             not walked by pre_register_empty_tables_for_joins. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 3,
            "F-CSD-P3-001-T2: all 3 detection rows must appear (outer table populated); \
             is_known column all-false since crowdstrike_devices is empty; got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 10 (F-CSD-P3-001-T3): Nested IN-subquery depth 2 — locks recursion
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (
    //          SELECT device_id FROM crowdstrike_devices dev
    //          WHERE dev.device_id IN (SELECT device_id FROM armis_devices)
    //      )
    //
    // Depth-0 (outer FROM): crowdstrike_detections (has data)
    // Depth-1 subquery FROM: crowdstrike_devices (0 batches)
    // Depth-2 nested subquery FROM: armis_devices (0 batches)
    //
    // Locking recursion: a depth-1-only fix registers crowdstrike_devices from the
    // outer WHERE IN-subquery, but armis_devices at depth-2 remains missing.
    // DataFusion then fails on armis_devices instead. Only a fully recursive walk
    // (recurse into the subquery's own WHERE predicate) passes this test.
    //
    // RED at HEAD: Err(QueryExecutionFailed) on crowdstrike_devices (depth-1 missing).
    // RED post-depth-1-only-fix: Err(QueryExecutionFailed) on armis_devices (depth-2).
    // GREEN post-recursive-fix: Ok with 0 rows.
    // -----------------------------------------------------------------------

    /// F-CSD-P3-001-T3 / BC-2.11.005 / BC-2.01.010: Nested IN-subquery (depth 2) with
    /// 0-batch tables at both levels must return Ok (0 rows) — not a plan error.
    ///
    /// This test locks that the fix must be FULLY RECURSIVE. A shallow single-level
    /// patch that only walks depth-1 IN-subquery FROM positions still fails because
    /// `armis_devices` at depth 2 is not discovered.
    ///
    /// RED at HEAD: `Err(QueryExecutionFailed)` — `crowdstrike_devices` (depth-1) and
    ///              `armis_devices` (depth-2) both absent from catalog.
    #[tokio::test]
    async fn test_BC_2_11_005_F_CSD_P3_001_T3_nested_insubquery_depth2_both_empty_returns_empty_not_error(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data (depth-0).
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002"],
            "device_id",
            &["dev-A", "dev-B"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches; depth-1 subquery FROM table.
        // A depth-1-only fix would register this table but still miss armis_devices.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // armis_devices — 0 batches; depth-2 nested subquery FROM table.
        // This table is the recursion-lock: only reachable by recursing into the
        // depth-1 subquery's own WHERE predicate (Predicate::InSubquery at depth-1).
        register_mem_table(&ctx, "armis_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (\
                       SELECT device_id FROM crowdstrike_devices dev \
                       WHERE dev.device_id IN (SELECT device_id FROM armis_devices)\
                   )";
        let ast = PrismQlParser::parse(sql).expect("depth-2 nested IN-subquery SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-recursive-fix): Ok with 0 rows.
        // RED at HEAD: Err(QueryExecutionFailed) — crowdstrike_devices (depth-1) not
        //     in catalog; pre_register_empty_tables_for_joins found neither table.
        // RED post-depth-1-only-fix: Err(QueryExecutionFailed) — armis_devices (depth-2)
        //     still not in catalog; shallow fix covered only the outer IN-subquery FROM.
        assert!(
            result.is_ok(),
            "F-CSD-P3-001-T3 / BC-2.11.005: Nested IN-subquery (depth 2) with 0-batch \
             tables at both levels must return Ok (0 rows), not a DataFusion plan error. \
             RED at HEAD: crowdstrike_devices (depth-1) not pre-registered. \
             RED post-depth-1-fix: armis_devices (depth-2) not pre-registered. \
             Fix must recurse into subquery WHERE predicates to discover all levels. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "F-CSD-P3-001-T3: nested IN-subquery with both empty tables → \
             0 rows returned; got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 11 (F-CSD-P3-001-T4): Subquery WHERE references non-key column of empty table
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (
    //          SELECT device_id FROM crowdstrike_devices WHERE hostname = 'x'
    //      )
    //
    // The subquery WHERE references `hostname` — a spec-declared column of
    // crowdstrike_devices that is NOT inferrable from the outer query (no JOIN
    // predicates exist). This test locks two requirements in one:
    //   (a) subquery FROM tables must be discovered by pre_register (as in T1)
    //   (b) pre-registration must use spec-declared schema (Priority-2 bundled TOML),
    //       not just inference — `hostname` must be present for the subquery to plan
    //
    // Failure modes covered by the RED assertion `result.is_ok()`:
    //   Mode A: crowdstrike_devices not pre-registered at all →
    //           DataFusion: "table not found: crowdstrike_devices"
    //   Mode B: crowdstrike_devices pre-registered with inference-only schema
    //           (no `hostname`) → DataFusion: "No field named hostname"
    //
    // DESIRED (post-fix): Ok with 0 rows — subquery returns 0 matching device_ids
    //     (hostname='x' matches nothing in empty table → empty IN-set → 0 detections).
    // -----------------------------------------------------------------------

    /// F-CSD-P3-001-T4 / BC-2.11.005 / BC-2.01.010: IN-subquery WHERE references a
    /// non-key column (`hostname`) of a 0-batch table. Pre-registration must use the
    /// full spec-declared schema — not just inference from surrounding context (no JOINs
    /// exist here to infer from) — so the subquery planner can resolve `hostname`.
    ///
    /// Analogous to F-CSD-P1-002-T1 (which locked spec-schema fidelity for the JOIN-side
    /// case), but at the IN-subquery level.
    ///
    /// RED: `Err(QueryExecutionFailed)` — either (a) `crowdstrike_devices` not in catalog,
    ///      or (b) pre-registered with inference schema that lacks `hostname`.
    ///      Both modes are bugs that the fix must close.
    #[tokio::test]
    async fn test_BC_2_11_005_F_CSD_P3_001_T4_insubquery_nonkey_col_where_empty_table_returns_empty_not_error(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data.
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-C"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 0 batches; ONLY in the IN-subquery.
        // The subquery WHERE references `hostname` — a spec-declared column
        // (crowdstrike.sensor.toml: device_id, hostname, platform_name, status,
        //  first_seen, last_seen) that is NOT inferrable from the outer query
        // because there are no JOIN predicates to derive column types from.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (\
                       SELECT device_id FROM crowdstrike_devices \
                       WHERE hostname = 'x'\
                   )";
        let ast = PrismQlParser::parse(sql)
            .expect("IN-subquery with non-key WHERE column SQL must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Ok with 0 rows — hostname='x' matches nothing in the
        //     empty crowdstrike_devices table → empty IN-set → 0 detection rows.
        // RED failure modes:
        //   (a) current HEAD: crowdstrike_devices not pre-registered →
        //       QueryExecutionFailed "table not found: crowdstrike_devices"
        //   (b) post-fix-without-spec-schema: crowdstrike_devices pre-registered with
        //       inference-only schema (column count = 0, no hostname) →
        //       QueryExecutionFailed "No field named hostname"
        // Both (a) and (b) must be closed. Spec-schema pre-registration (Priority-2
        // bundled TOML) includes all 6 declared columns and correct Arrow types.
        assert!(
            result.is_ok(),
            "F-CSD-P3-001-T4 / BC-2.11.005: IN-subquery WHERE referencing non-key column \
             `hostname` of 0-batch table must return Ok (0 rows), not a plan error. \
             RED (a): crowdstrike_devices not pre-registered → table not found. \
             RED (b): pre-registered with inference schema (no hostname) → field not found. \
             Fix requires spec-schema pre-registration for subquery tables. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 0,
            "F-CSD-P3-001-T4: hostname='x' matches 0 rows in empty crowdstrike_devices → \
             empty IN-set → 0 detection rows returned; got {total_rows}"
        );
    }
}
