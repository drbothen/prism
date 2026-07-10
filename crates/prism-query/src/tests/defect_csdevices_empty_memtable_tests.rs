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
//! from JOIN-equality peer columns only. The current implementation (`pre_register_empty_tables`
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
//! Both sides of the JOIN have 0 batches. Current `pre_register_empty_tables`:
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
//! Tests 1-3: PASS after the inference-based fix (`pre_register_empty_tables` v1).
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
    // The inference-based `pre_register_empty_tables` passes Tests 1-3 but
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
        // pre_register_empty_tables infers `{device_id: Utf8}` from the JOIN ON
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
        //      pre_register_empty_tables inferred only `{device_id: Utf8}`.
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
    // Finding: `pre_register_empty_tables` (materialization.rs) builds
    // `all_table_names` from ONLY `sql_query.from` + `sql_query.joins` (lines
    // 2905-2916). It does NOT recurse into subqueries. `walk_sql_query` (the
    // `collect_external_table_names` helper) DOES recurse, but
    // `pre_register_empty_tables` is independent and does not call it.
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
    // implementer extends `pre_register_empty_tables` with recursive
    // subquery walking.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 8 (F-CSD-P3-001-T1): Predicate-position WHERE IN-subquery — primary exploit
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)
    //
    // crowdstrike_devices appears ONLY in the WHERE IN-subquery. The outer query
    // has no JOINs referencing it. `pre_register_empty_tables` processes
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
    /// `pre_register_empty_tables` scans only FROM + JOIN positions in
    /// the outer query; a table referenced exclusively inside a WHERE IN-subquery
    /// is invisible to it and never pre-registered.
    ///
    /// RED: `Err(QueryExecutionFailed)` — DataFusion cannot find `crowdstrike_devices`
    ///      because it was only referenced inside the IN-subquery, not in the outer
    ///      FROM/JOIN list that `pre_register_empty_tables` processes.
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
        // pre_register_empty_tables builds all_table_names from
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
        //      because pre_register_empty_tables did not recurse into
        //      the IN-subquery to discover and pre-register crowdstrike_devices.
        assert!(
            result.is_ok(),
            "F-CSD-P3-001-T1 / BC-2.11.005 / BC-2.01.010: WHERE IN-subquery referencing \
             a 0-batch table must return Ok (0 rows), not a DataFusion plan error. \
             RED: Err — crowdstrike_devices invisible to pre_register_empty_tables \
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
    // UPDATED per F-CSD-P4-001 Option A architect adjudication 2026-07-10.
    // The original COUNT-rewrite approach was rejected due to NULL semantics divergence.
    //
    // Grammar reach: CONFIRMED — `(col IN (SELECT col FROM t)) AS alias` is a legal
    // PrismQL projection expression (verified by high002_plan_pinning_tests.rs).
    //
    // SQL: SELECT (det.device_id IN (SELECT device_id FROM crowdstrike_devices)) AS is_known
    //      FROM crowdstrike_detections det
    //
    // DESIRED (post Option A fix): Err(ExprInSubqueryProjectionNotSupported { .. }).
    // E-QUERY-043 plan-time gate fires before DataFusion planning; returns structured error
    // with rewrite directive ("Use WHERE field IN (SELECT ...)").
    // -----------------------------------------------------------------------

    /// F-CSD-P3-001-T2 / BC-2.11.005: Expression-position IN-subquery in SELECT projection
    /// must return `E-QUERY-043 ExprInSubqueryProjectionNotSupported`, NOT a silent
    /// `QueryExecutionFailed` ("Internal error").
    ///
    /// # Architect adjudication (F-CSD-P4-001 Option A, 2026-07-10)
    ///
    /// The original COUNT-rewrite approach (normalize_expr Expr::InSubquery → scalar COUNT
    /// subquery) was REJECTED by the architect due to NULL semantics divergence: OCSF columns
    /// are nullable=true; the COUNT rewrite always returns TRUE/FALSE, collapsing the NULL
    /// case to FALSE. Standard SQL three-valued logic requires NULL when `x IS NULL` or when
    /// the IN-set contains NULL. LLM analysts relying on standard SQL semantics would receive
    /// wrong answers on nullable OCSF fields (agent-harness design goal).
    ///
    /// Option A (plan-time structured rejection) is strictly better than the original silent
    /// "Internal error" / `QueryExecutionFailed` — it explains what is unsupported and how
    /// to fix the query (`WHERE field IN (SELECT ...)`).
    ///
    /// Grammar reach: CONFIRMED by `test_med1_expr_insubquery_select_projection_temporal_folded`
    /// (high002_plan_pinning_tests.rs). The `(col IN (SELECT col FROM t)) AS alias` form is a
    /// legal PrismQL projection expression.
    #[tokio::test]
    async fn test_BC_2_11_005_F_CSD_P3_001_T2_expr_insubquery_projection_returns_e_query_043_not_internal_error(
    ) {
        use prism_core::error::PrismError;

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

        // DESIRED (post Option A fix): Err(ExprInSubqueryProjectionNotSupported).
        // The plan-time gate check_expr_insubquery_projection fires before DataFusion
        // planning and returns E-QUERY-043, giving the analyst a clear rewrite directive.
        //
        // The COUNT-rewrite approach was REJECTED (NULL semantics divergence — see doc comment).
        // The original error was Err(QueryExecutionFailed) — "Internal error" via catch-all.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P3-001-T2 / BC-2.11.005: Projection-position IN-subquery must return \
             E-QUERY-043 (ExprInSubqueryProjectionNotSupported), not an internal plan error. \
             The COUNT-rewrite was REJECTED by architect adjudication 2026-07-10 due to NULL \
             semantics divergence (nullable OCSF fields, agent-harness SQL semantics goal). \
             Use WHERE clause subquery form for equivalent filtering. got: {result:?}"
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
        //     in catalog; pre_register_empty_tables found neither table.
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

    // -----------------------------------------------------------------------
    // F-CSD-P4-001 Tests 12-17 (GREEN locks — E-QUERY-043 position gate, F-CSD-P4-001 closure)
    //
    // These tests lock the E-QUERY-043 plan-time gate for ALL positions where
    // `Expr::InSubquery` can appear in a PrismQL SELECT query (F-CSD-P4-001 Option A
    // adjudication 2026-07-10, D-1650).
    //
    // Background: Test 9 (F-CSD-P3-001-T2) above already locks the SELECT-projection
    // position. These tests add locks for:
    //   - GROUP BY position (F-CSD-P4-001-T1)
    //   - ORDER BY position (F-CSD-P4-001-T2)
    //   - WHERE-position negative control with populated tables (F-CSD-P4-001-T3)
    //   - Gate-ordering: temporal preempts E-QUERY-043 (F-CSD-P4-001-T4)
    //   - JOIN ON-position scope boundary: NOT gated by E-QUERY-043 (F-CSD-P4-001-T5)
    //   - Error-content POL-24 byte-consistency lock (F-CSD-P4-001-T6)
    //
    // Grammar reach (all positions confirmed by sql_parser.rs inspection):
    //   GROUP BY: group_by_clause uses expr.clone() which includes in_subquery atom.
    //   ORDER BY: order_expr uses expr.clone() → OrderExpr { expr: Expr::InSubquery }
    //   JOIN ON:  join_clause uses .then(expr.clone()) → Join { on: Expr::InSubquery }
    //   SELECT projection: covered by T2 above.
    //
    // All 6 tests expect GREEN (gate is implemented at HEAD 842b029e).
    // If any position unexpectedly returns a non-E-QUERY-043 error, the test is
    // left RED and reported as an implementer handoff.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 12 (F-CSD-P4-001-T1): GROUP BY-position IN-subquery → E-QUERY-043
    //
    // Grammar reach: CONFIRMED.
    // `group_by_clause` parser uses `expr.clone()` (sql_parser.rs build_sql_query_parser,
    // line ~454). The `in_subquery` atom (`field_path IN (sql_query)`) is included in
    // `expr` via the `atom = choice(( ..., in_subquery, ... ))` combinator.
    //
    // Query: `SELECT count(*) FROM crowdstrike_detections
    //         GROUP BY device_id IN (SELECT device_id FROM crowdstrike_devices)`
    // Parsed group_by: [Expr::InSubquery { field: device_id, subquery: SELECT ... }]
    //
    // DESIRED: Err(ExprInSubqueryProjectionNotSupported) — check_expr_insubquery_projection
    //   walks SqlQuery.group_by and fires E-QUERY-043 before DataFusion planning.
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T1 / BC-2.11.003: GROUP BY-position `Expr::InSubquery` must return
    /// E-QUERY-043 (ExprInSubqueryProjectionNotSupported), NOT a DataFusion plan error.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `group_by_clause` in `build_sql_query_parser` (sql_parser.rs) uses
    /// `expr.clone()` for each GROUP BY key. The `in_subquery` atom is part of `expr`
    /// (via `atom = choice((..., in_subquery, ...))`), so `field IN (SELECT ...)` is
    /// valid as a GROUP BY expression.
    ///
    /// # Gate contract
    ///
    /// `check_expr_insubquery_projection` walks `SqlQuery.group_by` (materialization.rs
    /// lines ~3190-3195) and fires E-QUERY-043 on the first `Expr::InSubquery` found.
    /// Gate fires before DataFusion planning, AFTER temporal checks in the production
    /// path (preserves F-EQ42-P2-001 ordering).
    ///
    /// This test exercises the gate via `execute_against_session` (no temporal check wrapper).
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T1_group_by_insubquery_returns_e_query_043() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Register both tables with data — E-QUERY-043 fires before DataFusion planning,
        // so table contents do not affect the gate result.
        let det_batch = make_batch("detection_id", &["det-001", "det-002"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-confirmed: `GROUP BY device_id IN (SELECT ...)` parses as
        // `group_by: [Expr::InSubquery { field: device_id, subquery: ... }]`
        let sql = "SELECT count(*) FROM crowdstrike_detections \
                   GROUP BY device_id IN (SELECT device_id FROM crowdstrike_devices)";
        let ast = PrismQlParser::parse(sql).expect(
            "GROUP BY IN-subquery must parse (grammar reach confirmed: \
             group_by_clause uses expr.clone() which includes in_subquery atom)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (GREEN): E-QUERY-043 fires via check_expr_insubquery_projection
        // walking SqlQuery.group_by before DataFusion planning.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T1 / BC-2.11.003: GROUP BY IN-subquery must return E-QUERY-043 \
             (ExprInSubqueryProjectionNotSupported). \
             check_expr_insubquery_projection walks group_by and gates Expr::InSubquery. \
             If this fails with another error type, the gate has a gap at GROUP BY position \
             → implementer handoff required. got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 13 (F-CSD-P4-001-T2): ORDER BY-position IN-subquery → E-QUERY-043
    //
    // Grammar reach: CONFIRMED.
    // `order_by_clause` parser: `order_expr` = `expr.clone().then(order_direction)`.
    // The `in_subquery` atom is part of `expr`, so `field IN (SELECT ...)` is valid
    // as an ORDER BY expression.
    //
    // Query: `SELECT device_id FROM crowdstrike_detections
    //         ORDER BY device_id IN (SELECT device_id FROM crowdstrike_devices)`
    // Parsed order_by: [OrderExpr { expr: Expr::InSubquery { ... }, direction: Asc }]
    //
    // DESIRED: Err(ExprInSubqueryProjectionNotSupported).
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T2 / BC-2.11.003: ORDER BY-position `Expr::InSubquery` must return
    /// E-QUERY-043 (ExprInSubqueryProjectionNotSupported), NOT a DataFusion plan error.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `order_by_clause` builds `order_expr = expr.clone().then(order_direction)`.
    /// The `in_subquery` atom is part of `expr`, so `field IN (SELECT ...)` is syntactically
    /// valid in ORDER BY position.
    ///
    /// # Gate contract
    ///
    /// `check_expr_insubquery_projection` walks `SqlQuery.order_by` (materialization.rs
    /// lines ~3197-3201: `for order_item in &q.order_by { if contains_insubquery(&order_item.expr) }`)
    /// and fires E-QUERY-043 on the first match.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T2_order_by_insubquery_returns_e_query_043() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        let det_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-confirmed: `ORDER BY device_id IN (SELECT ...)` parses as
        // `order_by: [OrderExpr { expr: Expr::InSubquery { field: device_id, ... }, direction: Asc }]`
        let sql = "SELECT device_id FROM crowdstrike_detections \
                   ORDER BY device_id IN (SELECT device_id FROM crowdstrike_devices)";
        let ast = PrismQlParser::parse(sql).expect(
            "ORDER BY IN-subquery must parse (grammar reach confirmed: \
             order_expr = expr.clone().then(order_direction), in_subquery is part of expr)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (GREEN): E-QUERY-043 fires via check_expr_insubquery_projection
        // walking SqlQuery.order_by before DataFusion planning.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T2 / BC-2.11.003: ORDER BY IN-subquery must return E-QUERY-043 \
             (ExprInSubqueryProjectionNotSupported). \
             check_expr_insubquery_projection walks order_by and gates Expr::InSubquery. \
             If this fails with another error type, the gate has a gap at ORDER BY position \
             → implementer handoff required. got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 14 (F-CSD-P4-001-T3): Negative control — WHERE-position IN-subquery
    // with POPULATED tables → Ok with correct three-valued filtering.
    //
    // WHERE/HAVING use `Predicate::InSubquery` (not `Expr::InSubquery`).
    // The E-QUERY-043 gate only checks `select.items`, `group_by`, and `order_by`.
    // WHERE `Predicate::InSubquery` is DataFusion-native — decorrelate_predicate_subquery
    // handles it with standard three-valued SQL semantics.
    //
    // Setup: crowdstrike_detections has 3 rows (device_ids: dev-A, dev-B, dev-C).
    //        crowdstrike_devices has 2 rows (device_ids: dev-A, dev-B only).
    // SQL: WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)
    // DESIRED: Ok with 2 rows — det-001/dev-A and det-002/dev-B match; det-003/dev-C does not.
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T3 / BC-2.11.003 negative control: WHERE-position `Predicate::InSubquery`
    /// with POPULATED tables must execute successfully with correct three-valued filtering.
    ///
    /// This locks two invariants simultaneously:
    ///   (a) E-QUERY-043 does NOT fire for WHERE-position IN-subquery
    ///       (only SELECT/GROUP BY/ORDER BY positions are gated by check_expr_insubquery_projection)
    ///   (b) WHERE `Predicate::InSubquery` executes via DataFusion's
    ///       `decorrelate_predicate_subquery` optimizer with correct three-valued semantics:
    ///       - TRUE: det.device_id ∈ crowdstrike_devices.device_id set → row included
    ///       - FALSE: det.device_id ∉ set (AND set has no NULLs) → row excluded
    ///
    /// The T8 test (F-CSD-P3-001-T1) covers the EMPTY-tables WHERE case (0 rows);
    /// this test covers the POPULATED-tables path (2 of 3 rows match).
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T3_where_insubquery_populated_executes_ok() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — 3 rows; device_id values: dev-A, dev-B, dev-C
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-C"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 2 rows: dev-A and dev-B only (dev-C absent)
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)";
        let ast = PrismQlParser::parse(sql).expect("WHERE IN-subquery must parse");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // Negative control assertion (a): E-QUERY-043 must NOT fire for WHERE position.
        assert!(
            !matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T3 / BC-2.11.003: WHERE-position IN-subquery must NOT return \
             E-QUERY-043 — only SELECT/GROUP BY/ORDER BY positions are gated by \
             check_expr_insubquery_projection (it does NOT walk SqlQuery.where_). got: {result:?}"
        );

        // Assertion (b): WHERE IN-subquery with populated tables must execute successfully.
        assert!(
            result.is_ok(),
            "F-CSD-P4-001-T3: WHERE IN-subquery with populated tables must return Ok \
             (DataFusion three-valued semantics via decorrelate_predicate_subquery). \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        // det-001 (dev-A) and det-002 (dev-B) are in crowdstrike_devices;
        // det-003 (dev-C) is NOT → 2 rows returned.
        assert_eq!(
            total_rows, 2,
            "F-CSD-P4-001-T3: WHERE IN-subquery must return exactly 2 rows \
             (det-001/dev-A and det-002/dev-B match; det-003/dev-C does not). \
             got {total_rows}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 15 (F-CSD-P4-001-T4): Gate-ordering lock — temporal (E-QUERY-042) preempts
    // E-QUERY-043 when both violations are present.
    //
    // Gate ordering contract (error-taxonomy v2.38 §E-QUERY-043):
    //   E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → check_temporal_literals (E-QUERY-041/042)
    //     → fan-out → execute_against_session_with_registry → check_expr_insubquery_projection (E-QUERY-043)
    //
    // Query shape: outer SELECT has Expr::InSubquery → E-QUERY-043 if reached.
    //              inner subquery GROUP BY has RFC-3339 literal → E-QUERY-042 via temporal walker.
    //
    // In the production path (run_materialization_pipeline), check_temporal_literals fires
    // at step 1c BEFORE execute_against_session_with_registry (which has E-QUERY-043).
    // check_expr_temporal_pos recurses into Expr::InSubquery subqueries (MED-1 / F-P4-MED-2 fix):
    //   check_select_items_raw_temporal → check_expr_temporal(Expr::InSubquery) →
    //   check_expr_temporal_pos on subquery.group_by → TemporalCheckPos::GroupBy →
    //   Literal::Timestamp in GroupBy → E-QUERY-042.
    //
    // Proven by two-step test:
    //   Step 1: check_temporal_literals(ast) → E-QUERY-042 (GroupBy) [production ordering]
    //   Step 2: execute_against_session(ast) → E-QUERY-043 [skips temporal check, hits 043 gate]
    //   Combined conclusion: E-QUERY-042 fires BEFORE E-QUERY-043 in production path.
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T4 / BC-2.11.003 gate-ordering lock: when a query violates BOTH
    /// E-QUERY-042 (temporal literal in subquery GROUP BY) AND E-QUERY-043 (SELECT
    /// projection IN-subquery), the temporal check (E-QUERY-042) fires FIRST in the
    /// production pipeline, preempting E-QUERY-043.
    ///
    /// # Gate ordering contract
    ///
    /// In `run_materialization_pipeline` (step 1c):
    ///   `check_temporal_literals` fires E-QUERY-042 via GROUP BY temporal walk
    ///   → returns Err BEFORE `execute_against_session_with_registry` is called
    ///   → `check_expr_insubquery_projection` (E-QUERY-043) is never reached.
    ///
    /// # Two-step verification
    ///
    /// Step 1: calls `check_temporal_literals` directly (pub(crate)) — simulates the
    ///   production gate order. Asserts E-QUERY-042 (GroupBy) fires on the subquery's
    ///   GROUP BY temporal literal.
    ///
    /// Step 2: calls `execute_against_session` (no temporal check wrapper) — asserts
    ///   E-QUERY-043 fires for the outer SELECT projection's IN-subquery.
    ///
    /// Combined: E-QUERY-042 < E-QUERY-043 in production gate order.
    ///
    /// Traces to: F-CSD-P4-001 adjudication 2026-07-10; error-taxonomy v2.38 §E-QUERY-043
    /// gate-ordering contract; F-EQ42-P2-001 (temporal ordering pin).
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T4_gate_ordering_temporal_preempts_e_query_043() {
        use prism_core::error::{PrismError, TemporalLiteralPosition};

        // Query with BOTH violations simultaneously:
        //   Outer SELECT projection: (device_id IN (SELECT ...)) AS flag → Expr::InSubquery
        //   Inner subquery GROUP BY: '2026-07-01T00:00:00Z' → Literal::Timestamp (RFC-3339)
        //     parsed as RawTemporalLiteral or Literal::Timestamp → E-QUERY-042 (GroupBy)
        //
        // Grammar reach:
        //   Outer SELECT InSubquery: confirmed by T2 test above and high002 MED-1 tests.
        //   GROUP BY RFC-3339 literal: confirmed by test_F_EQ42_P1_002_subquery_in_where_group_by_timestamp.
        let sql = concat!(
            "SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices ",
            "GROUP BY '2026-07-01T00:00:00Z')) AS flag ",
            "FROM crowdstrike_detections"
        );

        // --- Step 1: production gate ordering ---
        // check_temporal_literals (pub(crate)) simulates what run_materialization_pipeline
        // calls at step 1c. With no registry, temporal checks fail-open on column type
        // (Unknown → skip), but the GROUP BY bare literal triggers E-QUERY-042 regardless
        // of registry state (arm 6: TemporalCheckPos::GroupBy → E-QUERY-042).
        //
        // Recursion path:
        //   Ast::Sql(Select) → check_select_items_raw_temporal (skip_projection=false)
        //     → check_expr_temporal(Expr::InSubquery { subquery }) [item is SELECT projection]
        //       → check_expr_temporal_pos(Expr::InSubquery, Other)
        //         → for subquery.group_by: check_expr_temporal_pos(GroupBy)
        //           → Literal::Timestamp in GroupBy → E-QUERY-042 (GroupBy)
        let mut ast_for_temporal = PrismQlParser::parse(sql)
            .expect("both-violation query must parse for gate-ordering step 1");

        let temporal_result =
            crate::materialization::check_temporal_literals(&mut ast_for_temporal, None, false);

        assert!(
            matches!(
                &temporal_result,
                Err(PrismError::TemporalLiteralInvalidPosition {
                    position: TemporalLiteralPosition::GroupBy,
                    ..
                })
            ),
            "F-CSD-P4-001-T4 step 1 (gate ordering): check_temporal_literals must fire \
             E-QUERY-042 (TemporalLiteralInvalidPosition::GroupBy) for the RFC-3339 \
             temporal literal inside the IN-subquery GROUP BY clause. \
             This confirms check_temporal_literals recurses into Expr::InSubquery subqueries \
             (MED-1 / F-P4-MED-2 fix) and that E-QUERY-042 fires before E-QUERY-043 \
             in the production pipeline. got: {temporal_result:?}"
        );

        // --- Step 2: execute_against_session companion verification ---
        // execute_against_session calls execute_against_session_with_registry directly,
        // which has check_expr_insubquery_projection (E-QUERY-043) but NOT
        // check_temporal_literals. So E-QUERY-043 fires here for the outer SELECT
        // projection's Expr::InSubquery — proving E-QUERY-043 WOULD have been reached
        // if temporal check were bypassed.
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
        // Fresh parse: ast_for_temporal may have been partially mutated by step 1
        // (check_temporal_literals mutates in-place for coercion arms; the GroupBy arm
        // returns Err before mutation, but defensive fresh parse is safer).
        let ast_for_043 = PrismQlParser::parse(sql)
            .expect("both-violation query must parse for gate-ordering step 2");

        let result_043 =
            execute_against_session(&ctx, sql, &ast_for_043, std::collections::HashMap::new())
                .await;

        assert!(
            matches!(
                &result_043,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T4 step 2 (companion): execute_against_session (no temporal check) \
             must fire E-QUERY-043 for the outer SELECT projection IN-subquery. \
             Combined with step 1: in run_materialization_pipeline, check_temporal_literals \
             fires E-QUERY-042 BEFORE execute_against_session_with_registry is called, \
             so E-QUERY-043 is preempted. got: {result_043:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 16 (F-CSD-P4-001-T5): JOIN ON-position scope boundary — NOT gated by E-QUERY-043
    //
    // Grammar reach: CONFIRMED.
    // `join_clause` parser uses `.then(expr.clone())` for the ON condition (sql_parser.rs
    // line ~434). The `in_subquery` atom is part of `expr`, so `field IN (SELECT ...)` is
    // syntactically valid in JOIN ON position.
    //
    // Architect adjudication (F-CSD-P4-001 Option A, 2026-07-10):
    //   JOIN ON Expr::InSubquery is deliberately NOT gated by E-QUERY-043.
    //   check_expr_insubquery_projection walks only select.items, group_by, order_by.
    //   It does NOT walk `q.joins`. JOIN ON is a predicate position (consistent with WHERE
    //   semantics); DataFusion's decorrelate_predicate_subquery handles it natively.
    //
    // DESIRED: NOT Err(ExprInSubqueryProjectionNotSupported); Ok with correct rows.
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T5 / BC-2.11.003 JOIN ON scope boundary: JOIN ON-position
    /// `Expr::InSubquery` must NOT return E-QUERY-043.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `join_clause` parser uses `.then(expr.clone().padded())` for the ON
    /// condition (sql_parser.rs `join_clause` definition). The `in_subquery` atom
    /// (`field_path IN (sql_query)`) is part of `expr`, so:
    ///   `INNER JOIN t2 ON field IN (SELECT col FROM t3)` is grammar-valid.
    ///
    /// # Gate scope boundary
    ///
    /// `check_sql_query` in `check_expr_insubquery_projection` checks:
    ///   - `q.select.items` ✓ (gated)
    ///   - `q.group_by` ✓ (gated)
    ///   - `q.order_by` ✓ (gated)
    ///   - `q.joins` — NOT checked (predicate position, DataFusion-plannable)
    ///
    /// This test locks the deliberate scope boundary: the gate does NOT fire for JOIN ON.
    /// DataFusion's `decorrelate_predicate_subquery` optimizer handles JOIN ON IN-subquery
    /// natively (same optimizer path as WHERE IN-subquery).
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T5_join_on_insubquery_not_gated_by_e_query_043() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Register both tables with data so DataFusion can plan and execute the query.
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002"],
            "device_id",
            &["dev-A", "dev-B"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B", "dev-C"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-confirmed: `JOIN t2 ON field IN (SELECT ...)` is parseable.
        // join_clause uses expr.clone() for ON; in_subquery is part of expr.
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   INNER JOIN crowdstrike_devices dev \
                   ON det.device_id IN (SELECT device_id FROM crowdstrike_devices)";
        let ast = PrismQlParser::parse(sql).expect(
            "JOIN ON IN-subquery must parse (grammar reach confirmed: \
             join_clause uses expr.clone() for ON condition, in_subquery is part of expr)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // Core assertion: E-QUERY-043 must NOT fire for JOIN ON position.
        // check_expr_insubquery_projection explicitly excludes q.joins.
        assert!(
            !matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T5 / BC-2.11.003: JOIN ON-position IN-subquery must NOT return \
             E-QUERY-043 — check_expr_insubquery_projection does not walk q.joins \
             (deliberate scope boundary: JOIN ON is a predicate position). got: {result:?}"
        );

        // DataFusion must be able to plan and execute the JOIN ON IN-subquery.
        // DataFusion's decorrelate_predicate_subquery optimizer handles this natively
        // (same path as WHERE field IN (SELECT ...)).
        assert!(
            result.is_ok(),
            "F-CSD-P4-001-T5: JOIN ON IN-subquery must execute successfully via DataFusion \
             (predicate position handled by decorrelate_predicate_subquery). \
             If this fails with a non-E-QUERY-043 error, report as implementer handoff: \
             DataFusion may not support IN-subquery in JOIN ON position (gate gap check). \
             got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 17 (F-CSD-P4-001-T6 / F-CSD-P5-002): Error-content POL-24 byte-strict lock
    //
    // UPDATED by F-CSD-P5-002 (MED) — LOCAL adversary pass-5.
    //
    // Previously asserted three substrings; now asserts FULL-STRING equality against the
    // exact error-taxonomy v2.38 §E-QUERY-043 template (single preamble + both sentences).
    //
    // Error-taxonomy v2.38 §E-QUERY-043 full message (source of truth):
    //   "E-QUERY-043: IN subquery in projection position is not supported. Use a WHERE
    //    clause subquery instead: `WHERE field IN (SELECT ...)`. Alternatively, a JOIN
    //    achieves the same result: `SELECT * FROM t JOIN (SELECT col FROM src) s ON
    //    t.field = s.col`."
    //
    // #[error(...)] template in prism_core::error::PrismError:
    //   "E-QUERY-043: IN subquery in projection position is not supported. {hint}"
    //
    // For Display to match the taxonomy, `hint` must be EXACTLY:
    //   "Use a WHERE clause subquery instead: `WHERE field IN (SELECT ...)`. \
    //    Alternatively, a JOIN achieves the same result: \
    //    `SELECT * FROM t JOIN (SELECT col FROM src) s ON t.field = s.col`."
    //
    // Current `hint` in materialization.rs (~3205):
    //   "IN subquery in SELECT projection position is not currently supported. \
    //    Use a WHERE clause subquery instead: `WHERE field IN (SELECT ...)`."
    //
    // Current Display (BUGGY — differs in two ways):
    //   "E-QUERY-043: IN subquery in projection position is not supported. IN subquery \
    //    in SELECT projection position is not currently supported. Use a WHERE clause \
    //    subquery instead: `WHERE field IN (SELECT ...)`."
    //
    // RED: assert_eq! FAILS because:
    //   (a) hint has a doubled preamble ("IN subquery in SELECT projection position is
    //       not currently supported.") after the fixed prefix already says "not supported"
    //   (b) hint omits the JOIN alternative sentence
    //
    // GREEN (post-fix): implementer must update `hint` in check_expr_insubquery_projection
    //   (materialization.rs) to exactly:
    //   "Use a WHERE clause subquery instead: `WHERE field IN (SELECT ...)`. \
    //    Alternatively, a JOIN achieves the same result: \
    //    `SELECT * FROM t JOIN (SELECT col FROM src) s ON t.field = s.col`."
    // -----------------------------------------------------------------------

    /// F-CSD-P4-001-T6 / F-CSD-P5-002 / BC-2.11.003 error-content byte-strict lock:
    /// E-QUERY-043 Display message must match the FULL template from error-taxonomy v2.38
    /// §E-QUERY-043 byte-for-byte.
    ///
    /// # Taxonomy template (authoritative)
    ///
    /// ```text
    /// E-QUERY-043: IN subquery in projection position is not supported. Use a WHERE
    /// clause subquery instead: `WHERE field IN (SELECT ...)`. Alternatively, a JOIN
    /// achieves the same result: `SELECT * FROM t JOIN (SELECT col FROM src) s ON
    /// t.field = s.col`.
    /// ```
    ///
    /// # RED state (current Display — two bugs)
    ///
    /// 1. **Doubled preamble**: The `#[error]` macro already emits "not supported.", then
    ///    the current `hint` starts with "IN subquery in SELECT projection position is not
    ///    currently supported." — a near-duplicate of the fixed prefix.
    /// 2. **Missing JOIN alternative**: The hint ends after the WHERE form, omitting
    ///    "Alternatively, a JOIN achieves the same result: `SELECT * FROM t JOIN ...`."
    ///
    /// # POL-24 constraint
    ///
    /// The full Display string is byte-pinned to the error-taxonomy v2.38 template.
    /// Any future change to `check_expr_insubquery_projection`'s `hint` value must keep
    /// the combined `#[error(...)] + {hint}` output byte-identical to the taxonomy.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P4_001_T6_e_query_043_display_contains_actionable_hint() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Minimal setup — just enough to reach the gate (1 row, tables registered).
        let det_batch = make_batch("detection_id", &["det-001"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        // crowdstrike_devices can be empty — E-QUERY-043 fires before DataFusion planning.
        register_mem_table(&ctx, "crowdstrike_devices", vec![])
            .expect("empty registration must not error");

        // SELECT projection IN-subquery — the canonical position for E-QUERY-043.
        let sql = "SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices)) AS is_known \
                   FROM crowdstrike_detections";
        let ast = PrismQlParser::parse(sql).expect(
            "SELECT projection IN-subquery must parse (grammar reach confirmed by T2 above)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // Must be E-QUERY-043 (gate is already implemented; this part stays GREEN).
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P4-001-T6: projection IN-subquery must produce E-QUERY-043. got: {result:?}"
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        // F-CSD-P5-002 byte-strict lock: full-string equality against error-taxonomy v2.38 template.
        //
        // Source: error-taxonomy.md v2.38 §E-QUERY-043 Message Format column (single preamble,
        //   both sentences: WHERE-clause form + JOIN alternative).
        //
        // RED: assert_eq! FAILS because the current Display is:
        //   "E-QUERY-043: IN subquery in projection position is not supported. IN subquery
        //    in SELECT projection position is not currently supported. Use a WHERE clause
        //    subquery instead: `WHERE field IN (SELECT ...)`."
        // Expected (post-fix):
        //   "E-QUERY-043: IN subquery in projection position is not supported. Use a WHERE
        //    clause subquery instead: `WHERE field IN (SELECT ...)`. Alternatively, a JOIN
        //    achieves the same result: `SELECT * FROM t JOIN (SELECT col FROM src) s ON
        //    t.field = s.col`."
        let expected = concat!(
            "E-QUERY-043: IN subquery in projection position is not supported. ",
            "Use a WHERE clause subquery instead: `WHERE field IN (SELECT ...)`. ",
            "Alternatively, a JOIN achieves the same result: ",
            "`SELECT * FROM t JOIN (SELECT col FROM src) s ON t.field = s.col`."
        );
        assert_eq!(
            display, expected,
            "F-CSD-P5-002 / F-CSD-P4-001-T6 byte-strict (POL-24): E-QUERY-043 Display \
             must match error-taxonomy.md v2.38 §E-QUERY-043 template byte-for-byte. \
             RED (a): current hint has doubled preamble — 'IN subquery in SELECT projection \
             position is not currently supported.' duplicates the fixed #[error] prefix. \
             RED (b): current hint omits the JOIN alternative sentence. \
             Fix: update `hint` in check_expr_insubquery_projection (materialization.rs) to \
             'Use a WHERE clause subquery instead: `WHERE field IN (SELECT ...)`. \
             Alternatively, a JOIN achieves the same result: \
             `SELECT * FROM t JOIN (SELECT col FROM src) s ON t.field = s.col`.' \
             expected: {expected:?} \
             actual:   {display:?}"
        );
    }

    // -----------------------------------------------------------------------
    // F-CSD-P5-001 Tests 18-19 (RED — LOCAL adversary pass-5)
    //
    // Finding: `contains_insubquery` in `check_expr_insubquery_projection`
    // (materialization.rs ~3169) recurses only into Compare/Logical/Not.
    // It does NOT recurse into `Expr::FuncCall` args or `Expr::TimestampArithmetic` base.
    //
    // Consequence: a query with `(Expr::InSubquery)` nested INSIDE a FuncCall argument
    // in a gated position (SELECT projection or GROUP BY) evades the E-QUERY-043 gate.
    // `contains_insubquery` returns `false` for the FuncCall wrapper → gate does NOT
    // fire → DataFusion receives the query → DataFusion cannot plan InSubquery in scalar
    // expression position → `PrismError::QueryExecutionFailed` (opaque -32000 to MCP).
    //
    // Grammar reach (CONFIRMED by sql_parser.rs analysis):
    //   `scalar_call` uses `expr.clone()` for each arg (sql_parser.rs lines ~1011-1019).
    //   `in_subquery` is an atom within `expr` via `atom = choice((..., in_subquery, ...))`.
    //   Therefore `ident(field IN (SELECT ...))` parses as
    //   `FuncCall::Scalar { func: Unknown("ident"), args: [Expr::InSubquery { ... }] }`.
    //
    // TimestampArithmetic base: NOT reachable via the sql_parser. The `TimestampArithmetic`
    //   variant is only constructed in filter_parser.rs (not sql_parser.rs) and always with
    //   `base: Expr::Now`. No path from any SQL grammar production yields a
    //   `TimestampArithmetic` with an `InSubquery` base. Not tested.
    //
    // Both tests FAIL at HEAD for the documented reason; both must PASS after the
    // implementer adds `Expr::FuncCall(FuncCall::Scalar { args, .. }) => args.iter().any(contains_insubquery)`,
    // and the equivalent arm for `FuncCall::Aggregate`, to `contains_insubquery`.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 18 (F-CSD-P5-001-T1): FuncCall-wrapped InSubquery in SELECT projection
    //
    // SQL: SELECT coalesce(device_id IN (SELECT device_id FROM crowdstrike_devices))
    //      FROM crowdstrike_detections
    //
    // SELECT item: Expr::FuncCall(FuncCall::Scalar { func: Unknown("coalesce"),
    //              args: [Expr::InSubquery { field: device_id, subquery: ... }] })
    //
    // contains_insubquery(Expr::FuncCall(...)) → `_ => false` → gate returns false.
    // Gate does NOT fire E-QUERY-043 → DataFusion receives query → QueryExecutionFailed.
    //
    // RED: matches!(result, Err(ExprInSubqueryProjectionNotSupported { .. })) FAILS.
    // GREEN (post-fix): contains_insubquery recurses into FuncCall.args → gate fires.
    // -----------------------------------------------------------------------

    /// F-CSD-P5-001-T1 / BC-2.11.003: `Expr::InSubquery` nested as a `FuncCall::Scalar`
    /// arg in SELECT projection must return E-QUERY-043
    /// (`ExprInSubqueryProjectionNotSupported`) — not an opaque DataFusion internal error.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `scalar_call` in the PrismQL expr parser uses `expr.clone()` for each
    /// argument (sql_parser.rs `build_expr_parser`, lines ~1011-1019). The `in_subquery`
    /// atom (`field IN (SELECT ...)`) is part of `expr` via the `atom = choice(...)` list.
    /// Any `ident(field IN (SELECT ...))` parses as
    /// `FuncCall::Scalar { func: Unknown("ident"), args: [Expr::InSubquery { ... }] }`.
    ///
    /// # Defect
    ///
    /// `contains_insubquery` in `check_expr_insubquery_projection` (materialization.rs
    /// ~3169-3176) handles three cases explicitly — `InSubquery` (direct), `Compare`,
    /// `Logical`, `Not` — and falls through to `_ => false` for all other variants,
    /// including `Expr::FuncCall`. It does NOT recurse into `FuncCall.args`. An
    /// `Expr::InSubquery` node inside a FuncCall arg is invisible to the gate.
    ///
    /// # RED reason
    ///
    /// The gate returns `false` for the FuncCall wrapper → E-QUERY-043 does NOT fire →
    /// DataFusion receives the normalized SQL (e.g., `coalesce(device_id IN (SELECT ...))`)
    /// → DataFusion cannot plan `InSubquery` in scalar expression position
    /// (`not_impl_err!`) → maps to `Err(QueryExecutionFailed)`.
    /// The assertion `matches!(result, Err(ExprInSubqueryProjectionNotSupported { .. }))` FAILS.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P5_001_T1_funcall_arg_insubquery_projection_misses_gate() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Register both tables with data — E-QUERY-043 should fire before DataFusion planning;
        // table contents are irrelevant if the gate works, but populated tables make the
        // DataFusion error more deterministic when the gate is absent.
        let det_batch = make_batch("detection_id", &["det-001"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-reach confirmed: scalar_call uses expr.clone() for args; in_subquery is
        // an atom within expr. `coalesce` becomes ScalarFunc::Unknown("coalesce").
        // Result AST: FuncCall::Scalar { func: Unknown("coalesce"),
        //             args: [Expr::InSubquery { field: device_id,
        //                    subquery: SELECT device_id FROM crowdstrike_devices }] }
        //
        // If this expect() panics, grammar reach is NOT confirmed — revise the SQL shape.
        let sql = "SELECT coalesce(device_id IN (SELECT device_id FROM crowdstrike_devices)) \
                   FROM crowdstrike_detections";
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P5-001-T1: scalar_call with InSubquery arg must parse \
             (scalar_call uses expr.clone() for args; in_subquery is part of atom within expr)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported { .. }).
        // contains_insubquery must recurse into FuncCall::Scalar args → gate fires E-QUERY-043
        // before DataFusion planning.
        //
        // RED: contains_insubquery hits `_ => false` for Expr::FuncCall → misses the InSubquery
        //      node inside args → gate does NOT fire → DataFusion receives the normalized SQL
        //      (`coalesce(device_id IN (SELECT ...))`) → DataFusion fails with
        //      Err(QueryExecutionFailed) (not_impl_err for InSubquery in scalar position).
        //      The assertion FAILS because result is NOT ExprInSubqueryProjectionNotSupported.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P5-001-T1 / BC-2.11.003: FuncCall-wrapped InSubquery in SELECT projection \
             must return E-QUERY-043 (ExprInSubqueryProjectionNotSupported). \
             RED: contains_insubquery does not recurse into Expr::FuncCall args \
             (`_ => false` arm in materialization.rs ~3175) — gate misses InSubquery nested \
             inside FuncCall::Scalar args → DataFusion receives query → \
             Err(QueryExecutionFailed) (opaque -32000 to MCP caller). \
             Fix: add FuncCall::Scalar {{ args, .. }} and FuncCall::Aggregate {{ args, .. }} \
             arms to contains_insubquery that call args.iter().any(contains_insubquery). \
             got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 19 (F-CSD-P5-001-T2): FuncCall-wrapped InSubquery in GROUP BY — gate miss
    //
    // Second nesting shape: GROUP BY position.
    //
    // SQL: SELECT count(*) FROM crowdstrike_detections
    //      GROUP BY coalesce(device_id IN (SELECT device_id FROM crowdstrike_devices))
    //
    // GROUP BY item: Expr::FuncCall(FuncCall::Scalar { func: Unknown("coalesce"),
    //                args: [Expr::InSubquery { ... }] })
    //
    // `check_sql_query` walks q.group_by and calls contains_insubquery on each expr.
    // But contains_insubquery(Expr::FuncCall(...)) → `_ => false` → gate misses.
    //
    // Comparison with Test 12 (F-CSD-P4-001-T1): bare `Expr::InSubquery` in GROUP BY
    // DOES fire the gate (T12 is GREEN). Wrapping it in FuncCall EVADES the gate (T19 RED).
    //
    // RED: matches!(result, Err(ExprInSubqueryProjectionNotSupported { .. })) FAILS.
    // GREEN (post-fix): same FuncCall arms added to contains_insubquery fix both T18/T19.
    // -----------------------------------------------------------------------

    /// F-CSD-P5-001-T2 / BC-2.11.003: `Expr::InSubquery` nested inside `FuncCall::Scalar`
    /// in GROUP BY position must return E-QUERY-043 — not an opaque DataFusion internal error.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED (extends T18 proof): `group_by_clause` in `build_sql_query_parser` uses
    /// `expr.clone()` for each GROUP BY key. `scalar_call` is part of `atom` within `expr`,
    /// and `scalar_call` args use `expr.clone()` which includes `in_subquery`. Therefore
    /// `GROUP BY coalesce(field IN (SELECT ...))` is syntactically valid and produces
    /// `group_by: [Expr::FuncCall(FuncCall::Scalar { args: [Expr::InSubquery { ... }] })]`.
    ///
    /// # Contrast with Test 12 (F-CSD-P4-001-T1)
    ///
    /// T12 asserts that a BARE `Expr::InSubquery` in GROUP BY fires E-QUERY-043 (GREEN).
    /// This test asserts that the SAME node WRAPPED in `FuncCall::Scalar` also fires E-QUERY-043.
    /// Currently it does NOT — `contains_insubquery` returns `false` for `Expr::FuncCall`,
    /// exposing a one-level-of-wrapping bypass. The fix is identical to T18.
    ///
    /// # RED reason
    ///
    /// `contains_insubquery(Expr::FuncCall(...))` → `_ => false` → GROUP BY gate does not
    /// detect the nested InSubquery → DataFusion receives the query → `QueryExecutionFailed`.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P5_001_T2_funcall_arg_insubquery_group_by_misses_gate() {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Register both tables with data.
        let det_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-confirmed (extends T18): group_by_clause uses expr.clone() for keys;
        // scalar_call is part of atom within expr; scalar_call args include in_subquery.
        //
        // If this expect() panics, grammar reach is NOT confirmed — revise the SQL shape.
        let sql = "SELECT count(*) FROM crowdstrike_detections \
                   GROUP BY coalesce(device_id IN (SELECT device_id FROM crowdstrike_devices))";
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P5-001-T2: scalar_call with InSubquery arg in GROUP BY must parse \
             (group_by_clause uses expr.clone(); scalar_call+in_subquery are both atoms in expr)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported { .. }).
        // contains_insubquery must recurse into FuncCall args → fires for GROUP BY item.
        //
        // RED: contains_insubquery hits `_ => false` for Expr::FuncCall on the GROUP BY expr
        //      → misses InSubquery inside args → gate does NOT fire → DataFusion receives
        //      the query → Err(QueryExecutionFailed) (DataFusion not_impl_err for InSubquery
        //      in scalar position). Assertion FAILS: result is NOT ExprInSubqueryProjectionNotSupported.
        //
        // Note: T12 (F-CSD-P4-001-T1) is GREEN for bare InSubquery in GROUP BY.
        // This test locks that wrapping in FuncCall is NOT a bypass.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P5-001-T2 / BC-2.11.003: FuncCall-wrapped InSubquery in GROUP BY \
             must return E-QUERY-043 (ExprInSubqueryProjectionNotSupported). \
             RED: contains_insubquery does not recurse into Expr::FuncCall args \
             (`_ => false` arm in materialization.rs ~3175) — GROUP BY FuncCall(InSubquery) \
             evades the gate → DataFusion receives the query → Err(QueryExecutionFailed). \
             Contrast: T12 (bare InSubquery in GROUP BY) is GREEN; wrapping in FuncCall \
             is the bypass. Fix: add FuncCall::Scalar {{ args, .. }} and \
             FuncCall::Aggregate {{ args, .. }} arms to contains_insubquery. got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 20 (F-CSD-P6-001-T1): DML source_select InSubquery in projection
    // bypasses E-QUERY-043 gate
    //
    // Finding F-CSD-P6-001 (LOW): `check_expr_insubquery_projection` routes
    // `Ast::Sql(SqlStatement::Dml(_))` to `_ => false`, claiming
    // "no E-QUERY-043 cases possible" — falsifiable:
    //   `INSERT INTO armis_tags
    //    SELECT device_id IN (SELECT device_id FROM crowdstrike_devices)
    //    FROM crowdstrike_detections LIMIT 10`
    // The source_select.select.items holds `Expr::InSubquery`. Without the
    // explicit DML arm, `check_sql_query` is never called on source_select
    // and the gate does not fire.
    //
    // Precedent: sibling walker `check_temporal_literals` already walks DML
    // source_select as defense-in-depth (F-P4-LOW-1, materialization.rs ~3415-3505).
    //
    // RED: result is `Ok(vec![])` — `_ => false` arm bypasses gate for all DML.
    //      DML execution path returns Ok(Vec::new()) pending S-3.06 wiring.
    // GREEN (post-fix): DML arm `dml.source_select.as_ref().is_some_and(|sq|
    //      check_sql_query(sq))` fires → `found = true` → E-QUERY-043.
    // -----------------------------------------------------------------------

    /// F-CSD-P6-001-T1 / BC-2.11.003: `INSERT INTO … SELECT (field IN (SELECT …)) FROM …`
    /// where the source SELECT's projection contains `Expr::InSubquery` must return
    /// `E-QUERY-043` (`ExprInSubqueryProjectionNotSupported`), not `Ok(vec![])`.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `in_subquery` is part of the `atom` choice in the Expr grammar
    /// (sql_parser.rs, the `choice((…, in_subquery, …))` block). `select_item` uses
    /// `expr [AS alias]`, so `device_id IN (SELECT device_id FROM crowdstrike_devices)`
    /// is a valid SELECT projection item that parses to
    /// `SelectItem::Expr { expr: Expr::InSubquery { field: device_id, subquery: … } }`.
    /// `LIMIT 10` satisfies `check_unbounded_write` (INSERT SELECT must have WHERE or LIMIT).
    ///
    /// # Execution path
    ///
    /// DML reaches `execute_against_session_with_registry` → `check_expr_insubquery_projection`
    /// at line 1177. Currently: `_ => false` → `found = false` → gate returns `Ok(())`.
    /// DML then falls into `_ => { Ok(Vec::new()) }` at ~1590 → `Ok(vec![])`.
    ///
    /// # RED observation
    ///
    /// `execute_against_session` returns `Ok(vec![])`. The analyst gets an empty success
    /// response instead of an actionable `E-QUERY-043` rewrite directive.
    ///
    /// Source: F-CSD-P6-001 (LOW), LOCAL pass-6, 2026-07-10.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P6_001_T1_dml_source_select_insubquery_projection_bypasses_e_query_043_gate(
    ) {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // Register tables — E-QUERY-043 fires before DataFusion planning when the gate works;
        // populated tables make the fallback DataFusion error more deterministic in RED state.
        let det_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");
        let dev_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // Grammar-reach confirmed: `device_id IN (SELECT device_id FROM crowdstrike_devices)`
        // is a valid SELECT projection item (in_subquery atom within Expr grammar).
        // `LIMIT 10` satisfies check_unbounded_write (INSERT SELECT must have WHERE or LIMIT).
        //
        // OBS-004 (gate-before-target-validation ordering): `armis_tags` is intentionally
        // NOT registered in this test's session context. The E-QUERY-043 gate fires in
        // `check_expr_insubquery_projection` (Step 1d of `run_materialization_pipeline`,
        // line ~1177) BEFORE table-availability validation (E-QUERY-037, Step 1c). Because
        // the gate fires first, the test receives `ExprInSubqueryProjectionNotSupported`
        // regardless of whether the DML target exists. This is correct gate-ordering
        // behaviour (E-QUERY-043 rewrite directive supersedes table-not-found). Switching to
        // a registered target would also work but would mask the ordering dependency —
        // the unregistered target makes the ordering invariant explicit.
        //
        // If this expect() panics, grammar reach is NOT confirmed — revise the SQL shape.
        let sql = "INSERT INTO armis_tags \
                   SELECT device_id IN (SELECT device_id FROM crowdstrike_devices) \
                   FROM crowdstrike_detections LIMIT 10";
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P6-001-T1: INSERT SELECT with InSubquery in projection must parse \
             (in_subquery is an atom in the Expr grammar; select_item uses expr [AS alias]; \
             LIMIT 10 satisfies check_unbounded_write)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported { .. }).
        // DML arm must walk source_select via check_sql_query → gate fires E-QUERY-043
        // before DataFusion planning, giving the analyst an actionable rewrite directive.
        //
        // RED: `_ => false` arm in check_expr_insubquery_projection bypasses the gate for
        //      all DML variants → `found = false` → gate returns Ok(()) → DML execution
        //      falls into `_ => { Ok(Vec::new()) }` → result is Ok(vec![]).
        //      The comment "no E-QUERY-043 cases possible" is falsified by this INSERT shape.
        //
        // F-P4-LOW-1 precedent: sibling walker check_temporal_literals already walks
        // DML source_select (~3415-3505) as defense-in-depth. This closes the equivalent
        // gap in check_expr_insubquery_projection.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P6-001-T1 / BC-2.11.003: DML source_select with InSubquery in projection \
             must return E-QUERY-043 (ExprInSubqueryProjectionNotSupported), not Ok(vec[]). \
             RED: check_expr_insubquery_projection `_ => false` arm bypasses gate for DML — \
             source_select is not walked, gate never fires, result is Ok(vec[]). \
             Fix: add arm `Ast::Sql(SqlStatement::Dml(dml)) => \
             dml.source_select.as_ref().is_some_and(|sq| check_sql_query(sq))`. \
             (F-CSD-P6-001 LOW, mirrors F-P4-LOW-1 check_temporal_literals precedent). \
             got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // F-CSD-P7-001 Tests 21-24 (RED — LOCAL adversary pass-7)
    //
    // Finding: `check_expr_insubquery_projection`'s inner `check_sql_query`
    // (materialization.rs ~3211) walks only the TOP-LEVEL select.items, group_by,
    // and order_by. It does NOT recurse into subqueries reached via:
    //   - q.where_  Predicate::InSubquery.subquery
    //   - q.having  Predicate::InSubquery.subquery
    //   - q.joins[].on Expr::InSubquery.subquery
    //   - the .subquery of any Expr::InSubquery node it finds
    //
    // Consequence: a projection-position Expr::InSubquery NESTED inside a supported
    // WHERE/HAVING Predicate::InSubquery slips the gate entirely.
    // DataFusion receives the unsupported shape → not_impl_err → QueryExecutionFailed
    // (-32000 "Internal error" to the MCP caller).
    //
    // Sibling walkers (walk_sql_query, check_temporal_literals) DO recurse into
    // WHERE/HAVING predicates. check_sql_query is the lagging walker.
    //
    // Test 21 (F-CSD-P7-001-T1): Primary exploit shape — WHERE IN-subquery whose
    //   inner SELECT projection has Expr::InSubquery. Gate must fire E-QUERY-043.
    // Test 22 (F-CSD-P7-001-T2): Depth variant — WHERE IN-subquery whose inner
    //   GROUP BY has Expr::InSubquery. Locks that the fix uses full check_sql_query
    //   (all positions), not just a minimal "check inner select.items" patch.
    // Test 23 (F-CSD-P7-001-T3): HAVING-path variant — HAVING IN-subquery whose
    //   inner SELECT projection has Expr::InSubquery. Exercises the HAVING recursion
    //   path (q.having), orthogonal to the WHERE path of T21/T22.
    // Test 24 (F-CSD-P7-001-T4): Negative control — nested WHERE-in-WHERE with
    //   NO projection-position InSubquery anywhere, populated tables. Must execute
    //   fine. Locks that the recursive fix does not over-reject supported query shapes.
    //
    // Tests 21-23: RED at HEAD (Err(QueryExecutionFailed) instead of E-QUERY-043).
    // Test 24:     GREEN at HEAD and GREEN post-fix (negative control).
    //
    // All 4 pass after the implementer extends check_sql_query to call itself
    // recursively on WHERE/HAVING Predicate::InSubquery.subquery.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Test 21 (F-CSD-P7-001-T1): Primary exploit — WHERE IN-subquery with
    // SELECT projection InSubquery inside slips the gate.
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (
    //          SELECT (dev.device_id IN (SELECT device_id FROM armis_devices)) AS is_known
    //          FROM crowdstrike_devices dev
    //      )
    //
    // Gate walk on outer query:
    //   select.items = [detection_id] → no InSubquery
    //   group_by = [] → ok
    //   order_by = [] → ok
    //   where_ = Predicate::InSubquery { subquery: inner_q } → NOT walked
    //
    // inner_q.select.items has Expr::InSubquery → gate never called on inner_q.
    // DataFusion receives query → not_impl_err for InSubquery in scalar projection
    // → QueryExecutionFailed (opaque -32000 to MCP).
    //
    // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported).
    // RED: Err(QueryExecutionFailed).
    // -----------------------------------------------------------------------

    /// F-CSD-P7-001-T1 / BC-2.11.003: `Expr::InSubquery` in the SELECT projection of a
    /// subquery nested inside a WHERE `Predicate::InSubquery` must return E-QUERY-043
    /// (`ExprInSubqueryProjectionNotSupported`), NOT a silent `QueryExecutionFailed`.
    ///
    /// # Defect (F-CSD-P7-001, LOCAL adversary pass-7, MED)
    ///
    /// `check_sql_query` in `check_expr_insubquery_projection` (materialization.rs ~3211)
    /// walks only the TOP-LEVEL query's `select.items`, `group_by`, and `order_by`.
    /// It does NOT recurse into subqueries reached via `q.where_`
    /// `Predicate::InSubquery.subquery`. A projection-position `Expr::InSubquery` nested
    /// inside a WHERE IN-subquery's SELECT clause therefore slips the gate undetected.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: T9 (F-CSD-P3-001-T2) verified `(field IN (SELECT …)) AS alias` is a valid
    /// PrismQL SELECT projection. T8 / T14 confirmed `WHERE field IN (SELECT …)` parses as
    /// `Predicate::InSubquery`. Combining them (outer WHERE IN → inner SELECT projection IN)
    /// is grammatically valid and confirmed by the `PrismQlParser::parse` call below.
    ///
    /// # RED state
    ///
    /// Currently returns `Err(QueryExecutionFailed)` because the gate returns `false`
    /// (check_sql_query only walks the outer query and WHERE is not walked).
    /// DataFusion receives the inner `SELECT (col IN (SELECT …)) AS flag FROM …`
    /// and emits `not_impl_err!("InSubquery")` in scalar projection context.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P7_001_T1_where_insubquery_projection_insubquery_returns_e_query_043(
    ) {
        use prism_core::error::PrismError;

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

        // crowdstrike_devices — referenced in the outer WHERE IN-subquery FROM clause.
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // armis_devices — referenced in the innermost IN-subquery (projection position).
        let armis_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // Exploit shape from F-CSD-P7-001 finding description (2026-07-10):
        //   Outer WHERE: Predicate::InSubquery { field: det.device_id, subquery: inner_q }
        //   inner_q SELECT projection: SelectItem::Expr { expr: Expr::InSubquery { ... } }
        //
        // Gate walk on outer check_sql_query(outer_q):
        //   select.items = [detection_id Field] → contains_insubquery → false
        //   group_by = [] → ok
        //   order_by = [] → ok
        //   where_ = Predicate::InSubquery { subquery: inner_q } → NOT walked
        //
        // inner_q.select.items = [(dev.device_id IN (SELECT device_id FROM armis_devices)) AS is_known]
        // → Expr::InSubquery → gate WOULD fire if check_sql_query(inner_q) were called.
        // It is NOT called → gate returns false → DataFusion receives full query.
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (\
                       SELECT (dev.device_id IN (SELECT device_id FROM armis_devices)) AS is_known \
                       FROM crowdstrike_devices dev\
                   )";
        // Grammar reach: outer WHERE Predicate::InSubquery confirmed by T8/T14;
        // inner SELECT projection Expr::InSubquery confirmed by T9.
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P7-001-T1: combined WHERE IN + inner SELECT projection IN must parse — \
             outer Predicate::InSubquery and inner Expr::InSubquery in SELECT are both \
             established PrismQL grammar forms (T8 + T9 grammar-reach verification)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported { .. }).
        // check_sql_query must recurse into WHERE Predicate::InSubquery.subquery and
        // call check_sql_query(inner_q), where inner_q.select.items contains Expr::InSubquery.
        //
        // RED: currently Err(QueryExecutionFailed) — check_sql_query only walks the
        //      outer query's top-level positions; the WHERE predicate is not walked.
        //      DataFusion receives the query → not_impl_err for InSubquery in scalar
        //      projection context → PrismError::QueryExecutionFailed (opaque -32000 to MCP).
        //
        // This finding is the primary exploit described in F-CSD-P7-001 (LOCAL pass-7, MED).
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P7-001-T1 / BC-2.11.003: Projection-position InSubquery nested inside \
             a WHERE IN-subquery must return E-QUERY-043 \
             (ExprInSubqueryProjectionNotSupported), not a silent QueryExecutionFailed. \
             RED: check_sql_query does not recurse into WHERE Predicate::InSubquery.subquery \
             — inner SELECT projection Expr::InSubquery slips the gate → DataFusion receives \
             the unsupported shape → Err(QueryExecutionFailed) (opaque -32000 to MCP). \
             Fix: check_sql_query must also call itself recursively on the subquery found \
             in q.where_ when q.where_ is (or contains) Predicate::InSubquery.subquery. \
             Sibling walkers walk_sql_query and check_temporal_literals already recurse here. \
             got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 22 (F-CSD-P7-001-T2): Depth variant — WHERE IN-subquery whose inner
    // GROUP BY has Expr::InSubquery.
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (
    //          SELECT count(*) FROM crowdstrike_devices
    //          GROUP BY (device_id IN (SELECT device_id FROM armis_devices))
    //      )
    //
    // Gate walk on outer query:
    //   select.items = [detection_id] → no InSubquery
    //   group_by = [] → empty
    //   order_by = [] → empty
    //   where_ = Predicate::InSubquery { subquery: inner_q } → NOT walked
    //
    // inner_q.group_by = [Expr::InSubquery { field: device_id, subquery: armis_q }]
    // → gate WOULD fire if check_sql_query(inner_q) were called.
    // It is NOT called → gate returns false.
    //
    // Implementer note: a minimal patch that only checks inner_q.select.items
    // (without calling full check_sql_query which also checks group_by) passes T21
    // but fails this test. The fix must call the FULL check_sql_query on subqueries
    // reached via WHERE, not just check their select.items inline.
    //
    // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported).
    // RED: Err(QueryExecutionFailed).
    // -----------------------------------------------------------------------

    /// F-CSD-P7-001-T2 / BC-2.11.003: `Expr::InSubquery` in the GROUP BY clause of a
    /// subquery nested inside a WHERE `Predicate::InSubquery` must return E-QUERY-043.
    ///
    /// # Purpose: lock the full recursive walk, not a single-position patch
    ///
    /// A minimal fix that only checks `inner_q.select.items` (without calling
    /// `check_sql_query(inner_q)` which also walks `group_by` and `order_by`) would
    /// pass T21 (SELECT projection) but fail this test (GROUP BY). The fix must invoke
    /// the full `check_sql_query` on all subqueries reached via WHERE predicates —
    /// not a per-position selective check.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: T12 (F-CSD-P4-001-T1) verified `GROUP BY device_id IN (SELECT …)` parses
    /// as `group_by: [Expr::InSubquery { … }]`. The outer `WHERE field IN (SELECT …)` is
    /// an established PrismQL pattern (T8/T14). Combining them is grammatically valid.
    ///
    /// # RED state
    ///
    /// `check_sql_query` never called on inner_q → inner GROUP BY InSubquery undetected
    /// → gate returns false → DataFusion receives inner `SELECT count(*) FROM … GROUP BY
    /// (device_id IN (SELECT …))` → DataFusion cannot plan correlated subquery in GROUP BY
    /// → `Err(QueryExecutionFailed)`.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P7_001_T2_where_insubquery_group_by_insubquery_returns_e_query_043(
    ) {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002"],
            "device_id",
            &["dev-A", "dev-B"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        let armis_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // Depth variant: inner subquery has GROUP BY InSubquery (not SELECT projection InSubquery).
        //
        // Gate walk on outer check_sql_query(outer_q):
        //   outer_q.select.items = [detection_id Field] → no InSubquery
        //   outer_q.group_by = [] → empty (outer query has no GROUP BY)
        //   outer_q.order_by = [] → empty
        //   outer_q.where_ = Predicate::InSubquery { subquery: inner_q } → NOT walked
        //
        // inner_q.group_by = [Expr::InSubquery { field: device_id, subquery: armis_q }]
        //   → check_sql_query(inner_q) WOULD fire E-QUERY-043 (T12 confirmed this for outer GROUP BY).
        //   → check_sql_query(inner_q) is NOT called → gate misses.
        //
        // Grammar reach: T12 confirmed GROUP BY InSubquery; T14 confirmed WHERE InSubquery.
        // `count(*)` is used in inner SELECT to avoid SELECT/GROUP BY column mismatch
        // at the DataFusion semantic level (pure syntax test; DataFusion may or may not
        // validate this, but E-QUERY-043 fires before DataFusion planning post-fix).
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (\
                       SELECT count(*) FROM crowdstrike_devices \
                       GROUP BY (device_id IN (SELECT device_id FROM armis_devices))\
                   )";
        // Grammar reach: outer WHERE Predicate::InSubquery (T8/T14);
        // inner GROUP BY Expr::InSubquery (T12).
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P7-001-T2: WHERE IN + inner GROUP BY IN must parse — \
             outer Predicate::InSubquery (T8/T14) and inner Expr::InSubquery in GROUP BY \
             (T12) are both established PrismQL grammar forms",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported).
        // Fix must call full check_sql_query(inner_q) — not just check inner_q.select.items.
        // check_sql_query walks group_by → finds Expr::InSubquery → fires E-QUERY-043.
        //
        // RED: Err(QueryExecutionFailed) — inner GROUP BY InSubquery slips the gate.
        //
        // Implementer note (lock): a fix that only checks inner_q.select.items
        // without calling full check_sql_query passes T21 (SELECT projection) but fails here.
        // Correct fix: call check_sql_query(inner_q) recursively for any Predicate::InSubquery
        // found in q.where_, so ALL positions (select.items, group_by, order_by) are checked.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P7-001-T2 / BC-2.11.003: GROUP BY InSubquery inside a WHERE IN-subquery \
             must return E-QUERY-043 (ExprInSubqueryProjectionNotSupported). \
             RED: check_sql_query is not called on inner_q — inner group_by Expr::InSubquery \
             slips the gate → DataFusion receives unsupported GROUP BY InSubquery shape → \
             Err(QueryExecutionFailed). \
             DEPTH LOCK: a minimal patch that only checks inner_q.select.items (not full \
             check_sql_query) passes T21 but fails here. Fix must call full check_sql_query \
             on subqueries reached via WHERE Predicate::InSubquery.subquery. got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 23 (F-CSD-P7-001-T3): HAVING-path variant — HAVING IN-subquery whose
    // inner SELECT projection has Expr::InSubquery.
    //
    // SQL: SELECT device_id FROM crowdstrike_detections
    //      HAVING device_id IN (
    //          SELECT (dev.device_id IN (SELECT device_id FROM armis_devices)) AS is_known
    //          FROM crowdstrike_devices dev
    //      )
    //
    // Grammar reach verification: HAVING uses `build_having_predicate_parser` which
    // wraps the base predicate grammar (sql_parser.rs line ~555). The base predicate
    // grammar includes Predicate::InSubquery (`field IN (SELECT …)`). HAVING
    // Predicate::InSubquery is confirmed reachable.
    //
    // Gate walk on outer query:
    //   select.items = [device_id Field] → no InSubquery
    //   group_by = [] → ok
    //   order_by = [] → ok
    //   having = Predicate::InSubquery { subquery: inner_q } → NOT walked
    //
    // inner_q.select.items = [(dev.device_id IN (…)) AS is_known] → Expr::InSubquery
    // → gate WOULD fire if check_sql_query(inner_q) were called.
    //
    // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported).
    // RED: Err(QueryExecutionFailed).
    // -----------------------------------------------------------------------

    /// F-CSD-P7-001-T3 / BC-2.11.003: `Expr::InSubquery` in the SELECT projection of a
    /// subquery nested inside a HAVING `Predicate::InSubquery` must return E-QUERY-043.
    ///
    /// # Grammar reach
    ///
    /// CONFIRMED: `build_having_predicate_parser` (sql_parser.rs ~line 629) wraps the base
    /// predicate grammar (line ~555). The base predicate grammar includes `Predicate::InSubquery`
    /// (`field IN (SELECT …)`, from the `in_subquery_predicate` combinator). HAVING therefore
    /// supports `HAVING field IN (SELECT …)` — the same InSubquery predicate form as WHERE.
    ///
    /// The inner subquery SELECT projection InSubquery form is confirmed by T9 (F-CSD-P3-001-T2).
    ///
    /// # HAVING path (orthogonal to WHERE path of T21/T22)
    ///
    /// `check_sql_query` does not walk `q.having` (only `select.items`, `group_by`, `order_by`).
    /// This test exercises the `q.having` path — orthogonal to the `q.where_` path tested by
    /// T21 and T22. The fix must recurse into BOTH `q.where_` AND `q.having` predicates.
    ///
    /// # RED state
    ///
    /// `check_sql_query` never called on inner_q (HAVING not walked) → inner SELECT projection
    /// InSubquery undetected → gate returns false → DataFusion receives the query →
    /// not_impl_err for InSubquery in scalar projection context →
    /// `Err(QueryExecutionFailed)`.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P7_001_T3_having_insubquery_projection_insubquery_returns_e_query_043(
    ) {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data.
        let det_batch = make_batch("device_id", &["dev-A", "dev-B", "dev-C"]);
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — HAVING IN-subquery FROM table.
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // armis_devices — innermost subquery FROM table.
        let armis_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // HAVING-path exploit shape:
        //   outer HAVING: Predicate::InSubquery { field: device_id, subquery: inner_q }
        //   inner_q SELECT projection: Expr::InSubquery { field: dev.device_id, subquery: armis_q }
        //
        // Gate walk on outer check_sql_query(outer_q):
        //   select.items = [device_id Field] → no InSubquery
        //   group_by = [] → empty
        //   order_by = [] → empty
        //   having = Predicate::InSubquery { subquery: inner_q } → NOT walked
        //
        // inner_q.select.items has Expr::InSubquery → gate never called on inner_q.
        //
        // Grammar reach: HAVING Predicate::InSubquery uses the base predicate grammar
        // (build_having_predicate_parser wraps build_sql_predicate_parser). The `in_subquery`
        // predicate form is part of the base grammar → HAVING field IN (SELECT …) parses.
        // Inner SELECT projection Expr::InSubquery confirmed by T9.
        let sql = "SELECT device_id \
                   FROM crowdstrike_detections \
                   HAVING device_id IN (\
                       SELECT (dev.device_id IN (SELECT device_id FROM armis_devices)) AS is_known \
                       FROM crowdstrike_devices dev\
                   )";
        // Grammar reach verification: if this expect() panics, HAVING InSubquery is not
        // reachable — revise test shape and document grammar gap instead of forcing.
        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P7-001-T3: HAVING IN + inner SELECT projection IN must parse — \
             HAVING uses base predicate grammar which includes Predicate::InSubquery; \
             inner Expr::InSubquery in SELECT confirmed by T9",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // DESIRED (post-fix): Err(ExprInSubqueryProjectionNotSupported).
        // check_sql_query must also recurse into q.having Predicate::InSubquery.subquery
        // (same pattern as q.where_ recursion from T21/T22 fix, applied to HAVING).
        //
        // RED: currently Err(QueryExecutionFailed) — check_sql_query does not walk q.having;
        //      inner SELECT projection InSubquery slips the gate → DataFusion receives the
        //      unsupported shape → not_impl_err → PrismError::QueryExecutionFailed.
        //
        // Note: WHERE and HAVING are both Option<Predicate> fields in SqlQuery. The fix
        // for T21/T22 (recurse into q.where_ Predicate::InSubquery.subquery) must also
        // apply to q.having for this test to pass.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P7-001-T3 / BC-2.11.003: Projection-position InSubquery nested inside \
             a HAVING IN-subquery must return E-QUERY-043 \
             (ExprInSubqueryProjectionNotSupported), not a silent QueryExecutionFailed. \
             RED: check_sql_query does not recurse into q.having Predicate::InSubquery.subquery \
             — HAVING path is orthogonal to WHERE path (T21/T22); both q.where_ and q.having \
             must be covered by the fix. got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Test 24 (F-CSD-P7-001-T4): Negative control — nested WHERE-in-WHERE with
    // Predicate::InSubquery chains and NO projection-position InSubquery anywhere.
    //
    // SQL: SELECT det.detection_id FROM crowdstrike_detections det
    //      WHERE det.device_id IN (
    //          SELECT device_id FROM crowdstrike_devices
    //          WHERE device_id IN (SELECT device_id FROM armis_devices)
    //      )
    //
    // DESIRED: Ok with 1 row (populated tables; DataFusion executes natively).
    // Purpose: lock that the recursive fix does NOT over-reject supported query shapes.
    // Predicate::InSubquery chains are DataFusion-native (decorrelate_predicate_subquery).
    //
    // GREEN at HEAD and GREEN post-fix (negative control).
    //
    // Setup:
    //   crowdstrike_detections: [det-001/dev-A, det-002/dev-B, det-003/dev-C]
    //   crowdstrike_devices: [dev-A, dev-B]
    //   armis_devices: [dev-A]
    //
    // Step 1 (innermost): SELECT device_id FROM armis_devices → {dev-A}
    // Step 2 (middle):    SELECT device_id FROM crowdstrike_devices
    //                     WHERE device_id IN {dev-A} → {dev-A}
    // Step 3 (outer):     WHERE det.device_id IN {dev-A} → {det-001}
    // Result: 1 row.
    // -----------------------------------------------------------------------

    /// F-CSD-P7-001-T4 / BC-2.11.003 negative control: nested `Predicate::InSubquery`
    /// chains (WHERE-in-WHERE) with NO projection-position `Expr::InSubquery` anywhere
    /// and populated tables must execute successfully — NOT trigger E-QUERY-043.
    ///
    /// # Purpose
    ///
    /// Locks that the F-CSD-P7-001 fix (check_sql_query recursion into WHERE/HAVING
    /// Predicate::InSubquery.subquery) does NOT over-reject queries where the subquery
    /// contains only Predicate::InSubquery (predicate position — DataFusion-native) and
    /// no Expr::InSubquery in any SELECT projection, GROUP BY, or ORDER BY.
    ///
    /// # Gate boundary
    ///
    /// `Predicate::InSubquery` is DataFusion-native (handled by `decorrelate_predicate_subquery`
    /// optimizer). `Expr::InSubquery` in SELECT/GROUP BY/ORDER BY (expression position) is
    /// NOT supported. E-QUERY-043 only gates the latter. The fix MUST NOT conflate the two:
    /// recursing into WHERE.subquery to check for Expr::InSubquery positions (correct) is
    /// different from rejecting the WHERE Predicate::InSubquery itself (wrong).
    ///
    /// # GREEN at HEAD, GREEN post-fix
    ///
    /// - At HEAD: gate doesn't fire (no projection InSubquery even in outer query); DataFusion
    ///   executes the nested WHERE IN-subquery chain natively → Ok with 1 row.
    /// - Post-fix: gate recurses into WHERE subqueries to check for Expr::InSubquery in
    ///   select.items/group_by/order_by; finds none → gate still doesn't fire → Ok with 1 row.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P7_001_T4_nested_where_insubquery_no_projection_insubquery_executes_ok(
    ) {
        use prism_core::error::PrismError;

        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — 3 rows; device_id values: dev-A, dev-B, dev-C
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-C"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — 2 rows: dev-A and dev-B
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // armis_devices — 1 row: dev-A only (filters the middle subquery to {dev-A})
        let armis_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // Negative control shape: nested Predicate::InSubquery chain.
        // NO Expr::InSubquery in SELECT projection, GROUP BY, or ORDER BY anywhere.
        //   outer WHERE: Predicate::InSubquery { field: det.device_id, subquery: middle_q }
        //   middle_q WHERE: Predicate::InSubquery { field: device_id, subquery: armis_q }
        //   armis_q: plain SELECT device_id FROM armis_devices
        //
        // check_sql_query(outer_q): no InSubquery in outer select.items/group_by/order_by.
        // Post-fix: check_sql_query recurses into outer WHERE subquery (middle_q).
        //   middle_q: no InSubquery in select.items/group_by/order_by → ok.
        //   Post-fix recurse into middle_q WHERE subquery (armis_q).
        //   armis_q: no InSubquery anywhere → ok.
        // Gate returns false (correct) → no E-QUERY-043 → DataFusion executes.
        //
        // Confirm grammar reach: WHERE Predicate::InSubquery is established (T8/T14);
        // nested depth-2 pattern confirmed by T10 (empty tables variant).
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   WHERE det.device_id IN (\
                       SELECT device_id FROM crowdstrike_devices \
                       WHERE device_id IN (SELECT device_id FROM armis_devices)\
                   )";
        let ast = PrismQlParser::parse(sql)
            .expect("F-CSD-P7-001-T4: nested WHERE IN-subquery must parse (T10 confirmed)");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // Negative control assertion: E-QUERY-043 must NOT fire.
        // The fix recurses into WHERE subqueries but only checks for Expr::InSubquery
        // in expression positions — NOT for Predicate::InSubquery itself (predicate position).
        assert!(
            !matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P7-001-T4 negative control: nested WHERE Predicate::InSubquery chain \
             must NOT trigger E-QUERY-043 — Predicate::InSubquery is predicate-position and \
             DataFusion-native; the gate only fires for Expr::InSubquery in projection/group_by/order_by. \
             If this fires E-QUERY-043, the recursive fix over-rejects supported WHERE IN patterns. \
             got: {result:?}"
        );

        // Primary assertion: query must execute successfully with correct row count.
        assert!(
            result.is_ok(),
            "F-CSD-P7-001-T4 negative control: nested WHERE IN-subquery chain with populated \
             tables must return Ok (DataFusion decorrelate_predicate_subquery handles this \
             natively). got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();

        // Step 1: armis_devices → {dev-A}
        // Step 2: crowdstrike_devices WHERE device_id IN {dev-A} → {dev-A}
        // Step 3: crowdstrike_detections WHERE device_id IN {dev-A} → {det-001/dev-A} only
        // Expected: exactly 1 row.
        assert_eq!(
            total_rows, 1,
            "F-CSD-P7-001-T4: nested WHERE IN-subquery chain must return exactly 1 row \
             (det-001/dev-A matches; dev-B and dev-C filtered by the inner WHERE chain). \
             got {total_rows}"
        );
    }

    // =========================================================================
    // T25 / T26 / T27 — LOCAL pass-8 findings F-CSD-P8-001 (MED) and F-CSD-P8-002 (LOW)
    //
    // These tests drive paths that the existing 24 tests do NOT cover:
    //   T25/T26: the full `run_materialization_pipeline` path
    //            (existing tests all call `execute_against_session` directly)
    //   T27:     DML with `filter: Some(Predicate::InSubquery { subquery })` where the
    //            subquery's SELECT projection contains Expr::InSubquery
    //            (DML grammar cannot parse this shape; AST is constructed directly)
    // =========================================================================

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline-path adapter stubs for T25 / T26 (SID-1 compliant: in-process,
    // no DTU, no network). Pattern mirrors `RecordingAdapter` /
    // `StubCredentialResolver` in `armis_discriminator_wiring_seam_tests`
    // inside materialization.rs.
    // ─────────────────────────────────────────────────────────────────────────

    /// Zero-batch adapter: every `fetch` returns `Ok(vec![])`.
    ///
    /// Triggers `any_external_table_registered = false` in the pipeline's step-5
    /// loop → early-return at line 1068, bypassing `check_expr_insubquery_projection`.
    struct PipelineZeroAdapter {
        sensor_id: prism_core::SensorId,
    }

    #[async_trait::async_trait]
    impl prism_sensors::SensorAdapter for PipelineZeroAdapter {
        fn sensor_type(&self) -> prism_core::SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "t25-pipeline-zero-adapter"
        }

        async fn fetch(
            &self,
            _spec: &prism_sensors::adapter::SensorSpec,
            _params: &prism_sensors::adapter::QueryParams,
            _auth: &dyn prism_sensors::SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, prism_sensors::SensorError> {
            Ok(vec![])
        }
    }

    /// One-row adapter: every `fetch` returns a single-row batch.
    ///
    /// Triggers `any_external_table_registered = true` → pipeline proceeds past
    /// the early-return and calls `execute_against_session_with_registry` where
    /// `check_expr_insubquery_projection` fires.
    struct PipelineOneRowAdapter {
        sensor_id: prism_core::SensorId,
    }

    #[async_trait::async_trait]
    impl prism_sensors::SensorAdapter for PipelineOneRowAdapter {
        fn sensor_type(&self) -> prism_core::SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "t26-pipeline-one-row-adapter"
        }

        async fn fetch(
            &self,
            _spec: &prism_sensors::adapter::SensorSpec,
            _params: &prism_sensors::adapter::QueryParams,
            _auth: &dyn prism_sensors::SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, prism_sensors::SensorError> {
            Ok(vec![make_batch("device_id", &["stub-dev-001"])])
        }
    }

    /// Stub credential resolver: returns a test bearer token so `fan_out()` reaches
    /// the adapter's `fetch()` without a credential error.
    struct PipelineStubCreds;

    impl prism_sensors::CredentialResolver for PipelineStubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::SensorAuth>, prism_sensors::SensorError> {
            Ok(Box::new(prism_sensors::BearerStaticSensorAuth::new(
                "t25-t26-pipeline-test-token",
            )))
        }
    }

    /// Build a `MaterializationContext` wired with `adapter` for the `crowdstrike`
    /// sensor and a `PipelineStubCreds` credential resolver.
    fn make_crowdstrike_pipeline_context(
        adapter: Arc<dyn prism_sensors::SensorAdapter>,
    ) -> crate::materialization::MaterializationContext {
        let org_id = prism_core::OrgId::new();
        let mut registry = prism_sensors::AdapterRegistry::new();
        registry.register(org_id, adapter);
        crate::materialization::MaterializationContext::new_with_resolver(
            Arc::new(registry),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            10_000,
            Arc::new(PipelineStubCreds),
            None, // no OrgRegistry — test mode synthetic slug fallback
            None, // no resolved_spec_map — test mode
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 25 — F-CSD-P8-001 (MED) — RED
    //
    // `check_expr_insubquery_projection` is inside `execute_against_session_with_registry`,
    // which is only reached when `any_external_table_registered = true`. When ALL sensor
    // tables return 0 batches, step-5 never sets that flag, and the pipeline early-returns
    // at line 1068 WITHOUT running the gate — a plan-time error becomes data-dependent.
    //
    // RED: pipeline returns Ok(MaterializationOutput { batches: vec![] }).
    // GREEN (post-fix): gate is hoisted before the early-return (or invoked on the parsed
    // AST before step 4 fan-out) so it fires regardless of batch counts.
    // ─────────────────────────────────────────────────────────────────────────

    /// F-CSD-P8-001-T25 / BC-2.11.003: `run_materialization_pipeline` with a
    /// projection-position `Expr::InSubquery` query where ALL sensor tables return
    /// 0 batches must fire E-QUERY-043 (`ExprInSubqueryProjectionNotSupported`).
    ///
    /// Currently the gate is bypassed: the pipeline early-returns `Ok(empty)` at
    /// line 1068 because `PipelineZeroAdapter` causes `any_external_table_registered = false`.
    ///
    /// # Red→Green proof
    ///
    /// At HEAD: step-5 loop `if !batches.is_empty()` is never true → flag stays false
    /// → early-return fires → `execute_against_session_with_registry` is never called
    /// → `check_expr_insubquery_projection` at line 1177 is never reached → Ok(empty).
    ///
    /// Post-fix: gate runs before step-4 fan-out (or before the early-return check) →
    /// E-QUERY-043 fires regardless of batch counts → test turns GREEN.
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P8_001_T25_pipeline_zero_batch_all_tables_bypasses_e_query_043_gate(
    ) {
        use prism_core::error::PrismError;

        let mut mat_ctx = make_crowdstrike_pipeline_context(Arc::new(PipelineZeroAdapter {
            sensor_id: prism_core::SensorId::new("crowdstrike"),
        }));
        let session_ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for T25");

        // Projection-position InSubquery: both crowdstrike_detections and crowdstrike_devices
        // map to sensor_id "crowdstrike" via sensor_id_from_table_name (split on `_` prefix).
        // PipelineZeroAdapter returns Ok(vec![]) for every fetch call.
        let query = "SELECT device_id IN (SELECT device_id FROM crowdstrike_devices) \
                     AS is_known FROM crowdstrike_detections";
        let options = crate::engine::QueryOptions::default();

        let result = crate::materialization::run_materialization_pipeline(
            query,
            &options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await;

        // DESIRED (post-fix): E-QUERY-043 fires regardless of whether sensors return data.
        // Plan-time errors must be data-independent.
        //
        // RED observation (at HEAD):
        //   step-5: `any_external_table_registered` stays false (all batches empty)
        //   line 1068: `if !any_external_table_registered { return Ok(MaterializationOutput { batches: vec![], ... }) }`
        //   `execute_against_session_with_registry` never called
        //   `check_expr_insubquery_projection` never reached
        //   result = Ok(MaterializationOutput { batches: vec![] })
        //
        // F-CSD-P8-001 (MED): data-dependence in gate placement creates a silent bypass
        // for InSubquery-in-projection queries when sensors are empty at query time.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P8-001-T25 / BC-2.11.003: run_materialization_pipeline with \
             projection-position InSubquery and ALL-zero-batch sensors must return \
             E-QUERY-043 (ExprInSubqueryProjectionNotSupported). \
             RED: pipeline early-returns Ok(MaterializationOutput {{ batches: vec![] }}) at \
             line 1068 — step-5 never sets `any_external_table_registered = true` because \
             PipelineZeroAdapter returns Ok(vec![]) for all fetches. \
             `execute_against_session_with_registry` is never called and \
             `check_expr_insubquery_projection` (line 1177) never fires. \
             Fix: hoist gate before the `!any_external_table_registered` early-return. \
             got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 26 — F-CSD-P8-001 (GREEN consistency lock)
    //
    // Same query, same pipeline path, but with a populated adapter. Proves that
    // the gate IS reachable and fires correctly today when sensors return data.
    // The fix must not change this GREEN behavior for the populated-sensor path.
    // ─────────────────────────────────────────────────────────────────────────

    /// F-CSD-P8-001-T26 / BC-2.11.003 (GREEN lock): `run_materialization_pipeline`
    /// with a projection-position `Expr::InSubquery` query where at least ONE sensor
    /// table returns a non-empty batch must fire E-QUERY-043.
    ///
    /// This test proves the gate IS reachable today via the populated-sensor path,
    /// establishing the "surface is already data-INdependent when data exists"
    /// invariant. The fix must turn T25 GREEN without regressing this T26 GREEN lock.
    ///
    /// # Green proof
    ///
    /// `PipelineOneRowAdapter::fetch` returns `Ok(vec![make_batch(...)])`.
    /// step-5: `any_external_table_registered = true` → no early-return →
    /// `execute_against_session_with_registry` called → line 1177:
    /// `check_expr_insubquery_projection` fires before DataFusion planning → E-QUERY-043.
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P8_001_T26_pipeline_populated_batch_fires_e_query_043_gate_green_lock(
    ) {
        use prism_core::error::PrismError;

        let mut mat_ctx = make_crowdstrike_pipeline_context(Arc::new(PipelineOneRowAdapter {
            sensor_id: prism_core::SensorId::new("crowdstrike"),
        }));
        let session_ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for T26");

        // Identical query to T25 — only the adapter behavior changes.
        let query = "SELECT device_id IN (SELECT device_id FROM crowdstrike_devices) \
                     AS is_known FROM crowdstrike_detections";
        let options = crate::engine::QueryOptions::default();

        let result = crate::materialization::run_materialization_pipeline(
            query,
            &options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await;

        // GREEN lock: gate must fire on the populated-sensor path (this already works today).
        // If this regresses after the fix, the fix broke the populated path.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P8-001-T26 (GREEN lock) / BC-2.11.003: run_materialization_pipeline \
             with projection-position InSubquery and populated sensor batches must return \
             E-QUERY-043 — gate is reachable via execute_against_session_with_registry \
             line 1177 when any_external_table_registered = true. \
             If this fails, the T25 fix regressed the populated-sensor path. \
             got: {result:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 27 — F-CSD-P8-002 (LOW) — RED
    //
    // `check_expr_insubquery_projection`'s DML arm only walks `source_select`:
    //   `Ast::Sql(SqlStatement::Dml(dml)) => dml.source_select.as_ref().is_some_and(check_sql_query)`
    //
    // `dml.filter` is not walked. A DELETE/UPDATE WHERE clause containing
    // `Predicate::InSubquery { subquery }` whose `subquery.select.items` holds
    // `Expr::InSubquery` evades the gate entirely.
    //
    // The DML grammar (UPDATE/DELETE) uses `build_predicate_parser()` which does not
    // include `Predicate::InSubquery` (only `build_sql_predicate_parser()` adds it),
    // so this AST shape cannot be produced by parsing — it is constructed directly
    // (same strategy as T20 uses for INSERT-SELECT via parser reach).
    //
    // Sibling walker `check_temporal_literals` covers dml.filter as defense-in-depth
    // (F-P4-LOW-1, materialization.rs ~3415-3505). This test closes the equivalent
    // gap in `check_expr_insubquery_projection`.
    //
    // RED: `source_select = None` → `is_some_and` = false → gate Ok(()) →
    //      DML stub → Ok(vec![]).
    // GREEN (post-fix): gate walks dml.filter when it is
    //      Some(Predicate::InSubquery { subquery }) → check_sql_query(subquery) →
    //      finds Expr::InSubquery in subquery.select.items → E-QUERY-043.
    // ─────────────────────────────────────────────────────────────────────────

    /// F-CSD-P8-002-T27 / BC-2.11.003: DML `filter: Some(Predicate::InSubquery { subquery })`
    /// where `subquery`'s SELECT projection holds `Expr::InSubquery` must fire E-QUERY-043.
    ///
    /// # AST shape (constructed directly — not parseable via DELETE grammar)
    ///
    /// ```text
    /// DELETE FROM crowdstrike_detections
    /// WHERE device_id IN (
    ///   SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices)) AS flag
    ///   FROM crowdstrike_detections
    /// )
    /// ```
    ///
    /// The `device_id IN (SELECT device_id FROM crowdstrike_devices)` inside the
    /// WHERE-subquery's SELECT projection is `Expr::InSubquery`. The gate must
    /// walk `dml.filter → subquery.select.items` to detect it.
    ///
    /// # Red→Green proof
    ///
    /// At HEAD: `dml.source_select = None` → DML arm `is_some_and(check_sql_query)` = false
    /// → gate returns `Ok(())` without inspecting `dml.filter` → DML stub returns `Ok(vec![])`.
    ///
    /// Post-fix: gate's DML arm also checks `dml.filter`:
    /// `if let Some(Predicate::InSubquery { subquery, .. }) = &dml.filter { check_sql_query(subquery) }`
    /// → `check_sql_query` finds `Expr::InSubquery` in `subquery.select.items` → returns true
    /// → gate fires E-QUERY-043.
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P8_002_T27_dml_filter_insubquery_subquery_projection_evades_e_query_043_gate(
    ) {
        use crate::ast::{
            Ast, Expr, FieldPath, FromClause, Predicate, SelectClause, SelectItem, SourceRef,
            SqlQuery, SqlStatement,
        };
        use crate::write_ast::{DmlNode, DmlOperation};
        use prism_core::error::PrismError;

        let ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for T27");

        // Innermost subquery: `SELECT device_id FROM crowdstrike_devices`
        let innermost_q = SqlQuery::new(
            SelectClause::new(vec![SelectItem::Expr {
                expr: Expr::Field(FieldPath::new(["device_id"])),
                alias: None,
            }]),
            FromClause::new(SourceRef::from_raw("crowdstrike_devices")),
        );

        // WHERE-subquery: `SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices)) AS flag
        //                  FROM crowdstrike_detections`
        //
        // `Expr::InSubquery` is in `select.items` of this subquery — the shape the gate must
        // detect via `dml.filter` traversal.
        let where_subquery = SqlQuery::new(
            SelectClause::new(vec![SelectItem::Expr {
                expr: Expr::InSubquery {
                    field: FieldPath::new(["device_id"]),
                    subquery: Box::new(innermost_q),
                },
                alias: Some("flag".to_string()),
            }]),
            FromClause::new(SourceRef::from_raw("crowdstrike_detections")),
        );

        // DML node: `DELETE FROM crowdstrike_detections WHERE device_id IN (<where_subquery>)`
        //
        // `source_select = None` (DELETE, not INSERT-SELECT). The current DML arm:
        //   `dml.source_select.as_ref().is_some_and(check_sql_query)`
        // evaluates to `false` for None → gate never inspects `dml.filter`.
        //
        // This DmlNode is constructed directly because `build_delete_parser` uses
        // `build_predicate_parser()` (filter grammar) which does NOT include
        // `Predicate::InSubquery` — this shape cannot be produced by parsing.
        // `#[non_exhaustive]` on DmlNode does not restrict construction inside
        // the same crate (prism-query).
        let dml_node = DmlNode {
            operation: DmlOperation::Delete,
            target_table: "crowdstrike_detections".to_string(),
            columns: None,
            assignments: vec![],
            filter: Some(Predicate::InSubquery {
                field: FieldPath::new(["device_id"]),
                subquery: Box::new(where_subquery),
                negated: false,
            }),
            source_select: None,
        };
        let ast = Ast::Sql(SqlStatement::Dml(dml_node));

        let result = execute_against_session(
            &ctx,
            "synthetic-dml-t27-filter-interior",
            &ast,
            std::collections::HashMap::new(),
        )
        .await;

        // DESIRED (post-fix): E-QUERY-043 from gate traversal of dml.filter subquery projection.
        //
        // RED observation (at HEAD):
        //   `check_expr_insubquery_projection` DML arm:
        //     `Ast::Sql(SqlStatement::Dml(dml)) => dml.source_select.as_ref().is_some_and(check_sql_query)`
        //   `source_select = None` → `is_some_and` returns false → gate returns Ok(())
        //   DML execution stub: `_ => Ok(Vec::new())` → result = Ok(vec![])
        //
        // `dml.filter`'s `Predicate::InSubquery.subquery.select.items` contains
        // `Expr::InSubquery` but is NEVER inspected by the gate at HEAD.
        //
        // Sibling precedent (F-P4-LOW-1): `check_temporal_literals` already walks dml.filter
        // (materialization.rs ~3415-3505) as defense-in-depth. This closes the equivalent
        // gap in `check_expr_insubquery_projection`.
        assert!(
            matches!(
                &result,
                Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
            ),
            "F-CSD-P8-002-T27 / BC-2.11.003: DML with filter Predicate::InSubquery \
             whose subquery projection holds Expr::InSubquery must return E-QUERY-043. \
             RED: gate DML arm only walks source_select (= None for DELETE) — dml.filter \
             is not traversed → Expr::InSubquery in filter subquery projection is undetected \
             → result is Ok(vec![]) instead of E-QUERY-043. \
             Fix: extend DML arm to walk dml.filter when it is \
             Some(Predicate::InSubquery {{ subquery, .. }}) and call check_sql_query(subquery). \
             got: {result:?}"
        );
    }

    // =========================================================================
    // T28 — LOCAL pass-10 finding F-CSD-P10-001 (LOW):
    // JOIN ON × FuncCall-wrapped InSubquery — EMPIRICAL DETERMINATION
    //
    // Background: `descend_subquery_expr`'s FuncCall arm recurses with itself
    // (not `contains_insubquery`), so the shape
    //   `INNER JOIN t2 ON coalesce(field IN (SELECT ...))`
    // is NOT rejected by the E-QUERY-043 gate. The gate correctly rejects the
    // bare projection-position InSubquery (T9/T18) and the bare JOIN ON InSubquery
    // was verified DataFusion-plannable (T5 GREEN lock). But the FuncCall-wrapped
    // JOIN ON variant's DataFusion behavior was unverified at pass-10 time.
    //
    // This test runs the query empirically to determine the outcome. The assertion
    // below records the verified result. See the doc comment on the test fn for
    // the outcome rationale.
    //
    // Grammar reach: CONFIRMED (extends T5 + T18 precedents).
    //   - T5 confirmed: `JOIN t2 ON field IN (SELECT ...)` parses via join_clause
    //     using expr.clone() for ON condition.
    //   - T18 confirmed: `coalesce(field IN (SELECT ...))` parses as
    //     FuncCall::Scalar { func: Unknown("coalesce"), args: [InSubquery { ... }] }
    //     because scalar_call uses expr.clone() for args and in_subquery is part of expr.
    //   - Combined: `JOIN t2 ON coalesce(field IN (SELECT ...))` is grammar-valid.
    // =========================================================================

    /// F-CSD-P10-001 / BC-2.11.003: JOIN ON `coalesce(field IN (SELECT ...))` —
    /// EMPIRICAL DETERMINATION — GREEN LOCK (DataFusion executes successfully).
    ///
    /// # Finding summary (F-CSD-P10-001 LOW)
    ///
    /// `descend_subquery_expr`'s FuncCall arm calls `args.iter().any(descend_subquery_expr)`.
    /// When an arg is `Expr::InSubquery { subquery }`, `descend_subquery_expr` calls
    /// `check_sql_query(subquery)` — which checks the INNER subquery's projection positions
    /// (not the FuncCall-wrapping context). Since `SELECT device_id FROM armis_devices`
    /// has no InSubquery in its projections, `check_sql_query` returns false and the gate
    /// does NOT fire E-QUERY-043. The query reaches DataFusion.
    ///
    /// # Empirical determination (run 2026-07-10) — DOCUMENTED DATAFUSION CAPABILITY
    ///
    /// DataFusion EXECUTES SUCCESSFULLY. `coalesce(x IN (SELECT ...))` in JOIN ON position
    /// is plannable: DataFusion's optimizer decorrelates the IN-subquery even when wrapped
    /// in a `coalesce()` scalar function call. The query returns correct results:
    /// only detections where device_id matches the armis subquery set × all dev rows.
    ///
    /// # Conclusion: walker asymmetry is CORRECT BY DESIGN
    ///
    /// The `descend_subquery_expr` FuncCall arm recursing with itself (not `contains_insubquery`)
    /// is the CORRECT behavior for the JOIN ON position:
    ///   - `contains_insubquery` would reject the FuncCall-wrapped InSubquery as a
    ///     projection-position violation (E-QUERY-043), which would be WRONG — the shape
    ///     is DataFusion-plannable.
    ///   - `descend_subquery_expr` correctly recurses into the inner SqlQuery to check
    ///     whether the INNER subquery's SELECT projection contains InSubquery (which would
    ///     be a genuine violation). It does NOT reject the FuncCall wrapper itself.
    ///
    /// F-CSD-P10-001 is CLOSED as "documented DataFusion capability — no fix required".
    /// The walker asymmetry between `descend_subquery_expr` and `contains_insubquery`
    /// is load-bearing correctness, not a bug.
    ///
    /// # GREEN LOCK (BC-5.38.001)
    ///
    /// This test permanently locks the verified capability. Any future change to
    /// `descend_subquery_expr`'s FuncCall arm that causes this test to fail has
    /// OVER-REJECTED a DataFusion-plannable query shape. Regressing this test
    /// means the gate rejects valid JOIN ON expressions — a false positive.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P10_001_T28_join_on_funccall_wrapped_insubquery_datafusion_empirical(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data.
        // 3 rows: det-001/dev-A, det-002/dev-B, det-003/dev-C.
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002", "det-003"],
            "device_id",
            &["dev-A", "dev-B", "dev-C"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — INNER JOIN target with data.
        // 2 rows: dev-A, dev-B.
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // armis_devices — subquery table with data (populated).
        // 1 row: dev-A only.
        // Only det-001 (device_id = dev-A) matches the IN-subquery condition.
        let armis_batch = make_batch("device_id", &["dev-A"]);
        register_mem_table(&ctx, "armis_devices", vec![armis_batch])
            .expect("armis_devices registration must succeed");

        // FuncCall-wrapped InSubquery in JOIN ON position.
        // Grammar reach: CONFIRMED (extends T5 + T18 precedents).
        //   - T5 confirmed: join_clause uses expr.clone() for ON condition.
        //   - T18 confirmed: scalar_call uses expr.clone() for args; in_subquery is part of expr.
        //   - coalesce(single arg) is the same grammar shape as T18's SELECT projection test.
        //
        // descend_subquery_expr walk path for the JOIN ON FuncCall:
        //   descend_subquery_expr(FuncCall::Scalar { func: "coalesce", args: [InSubquery { ... }] })
        //     → args.iter().any(descend_subquery_expr)
        //     → descend_subquery_expr(InSubquery { subquery: SELECT device_id FROM armis_devices })
        //       → check_sql_query(subquery): no InSubquery in select.items/group_by/order_by
        //       → returns false
        //   → gate does NOT fire E-QUERY-043 → query reaches DataFusion.
        //
        // DataFusion EMPIRICALLY VERIFIED (2026-07-10): executes successfully.
        // coalesce(x IN (SELECT ...)) is plannable in JOIN ON position — the optimizer
        // decorrelates the IN-subquery even when wrapped in a scalar function.
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   INNER JOIN crowdstrike_devices dev \
                   ON coalesce(det.device_id IN (SELECT device_id FROM armis_devices))";

        let ast = PrismQlParser::parse(sql).expect(
            "F-CSD-P10-001 T28: JOIN ON coalesce(field IN (SELECT ...)) must parse \
             (grammar reach confirmed by T5 and T18 precedents: join_clause + scalar_call \
             both use expr.clone(); in_subquery is part of expr atom)",
        );

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // GREEN LOCK: DataFusion EXECUTES SUCCESSFULLY.
        // Empirically verified 2026-07-10: coalesce(x IN (SELECT ...)) in JOIN ON
        // is DataFusion-plannable. The query returns correct results.
        //
        // If this test FAILS after a code change, the change has OVER-REJECTED a
        // DataFusion-plannable JOIN ON shape. Do NOT silence by adjusting the assertion —
        // investigate whether descend_subquery_expr's FuncCall arm was incorrectly tightened
        // to route through contains_insubquery (which would falsely reject this valid shape).
        assert!(
            result.is_ok(),
            "F-CSD-P10-001 T28 GREEN LOCK / BC-2.11.003: JOIN ON coalesce(field IN (SELECT ...)) \
             must EXECUTE SUCCESSFULLY. DataFusion empirically verified (2026-07-10) to plan \
             and execute this shape via optimizer subquery decorrelation. \
             The walker asymmetry (descend_subquery_expr recurses with itself for FuncCall args, \
             not with contains_insubquery) is CORRECT BY DESIGN — it checks the inner subquery's \
             projection positions without rejecting the FuncCall wrapper. \
             If this regresses after a code change, descend_subquery_expr's FuncCall arm was \
             incorrectly tightened — revert the change to that arm. \
             got: {result:?}"
        );

        // Row-count sanity: armis_devices has dev-A only. INNER JOIN condition is
        // coalesce(det.device_id IN {dev-A}) → TRUE only when det.device_id = dev-A.
        // det-001 (dev-A) × crowdstrike_devices (dev-A, dev-B) = 2 combinations.
        // det-002 (dev-B), det-003 (dev-C) → condition FALSE → excluded.
        // Expected total rows: 2 (det-001 appears once per dev row that passes the INNER JOIN).
        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(
            total_rows, 2,
            "F-CSD-P10-001 T28: INNER JOIN ON coalesce(IN subquery) must return 2 rows \
             (det-001 paired with dev-A and dev-B, as the ON condition is TRUE for all \
             dev rows when det.device_id = dev-A matches the armis subquery set). \
             det-002 and det-003 are excluded (condition = FALSE). \
             got {total_rows}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T29 — F-CSD-P10-001 empty-table variant consistency lock
    //
    // T28 established the GREEN lock: DataFusion executes
    // `JOIN ON coalesce(field IN (SELECT ...))` successfully when all tables are
    // populated. T29 verifies the EMPTY-TABLE variant of the same shape behaves
    // consistently: with armis_devices having 0 batches and pre-registration
    // covering it, the query must still plan successfully and return 0 rows
    // (not an error due to armis_devices being absent from the DataFusion catalog).
    //
    // Pre-registration (BC-2.11.005 DEC-022 / F-CSD-P3-001 fix) must register
    // armis_devices from the IN-subquery position with a schema-only MemTable.
    // Without this pre-registration, DataFusion would fail with "table not found:
    // armis_devices" — but that is a pre-registration failure, not a gate/FuncCall
    // issue. This test specifically verifies the combined behavior.
    // ─────────────────────────────────────────────────────────────────────────

    /// F-CSD-P10-001 T29 / BC-2.11.003 + BC-2.11.005 empty-table consistency lock:
    /// JOIN ON `coalesce(field IN (SELECT ...))` with the subquery table having 0 batches
    /// must execute successfully (Ok with 0 rows) — consistent with the T28 GREEN lock.
    ///
    /// # Combined behavior
    ///
    /// Two subsystems interact here:
    ///   1. **E-QUERY-043 gate** (T28 established): does NOT fire for FuncCall-wrapped
    ///      InSubquery in JOIN ON position — walker asymmetry is correct by design.
    ///   2. **Pre-registration** (F-CSD-P3-001 fix): armis_devices in the IN-subquery
    ///      FROM position must be pre-registered with a schema-only MemTable when
    ///      it has 0 batches, so DataFusion can plan the subquery.
    ///
    /// The empty-table variant tests that these two subsystems compose correctly:
    /// the gate passes (as in T28) AND pre-registration covers the subquery table
    /// (so DataFusion can plan) → the query returns 0 rows (empty IN-set, INNER JOIN
    /// produces 0 matches).
    ///
    /// # Expected result
    ///
    /// `Ok` with 0 rows: armis_devices has 0 rows → IN-subquery set is empty →
    /// `coalesce(x IN {}) = coalesce(FALSE) = FALSE` for all detections →
    /// INNER JOIN ON FALSE → 0 result rows.
    ///
    /// # Relationship to existing tests
    ///
    /// This test is DISTINCT from T8-T11 (F-CSD-P3-001 series): those tests have bare
    /// `WHERE det.device_id IN (SELECT device_id FROM crowdstrike_devices)` WHERE
    /// predicates. This test has a FuncCall-wrapped InSubquery in JOIN ON position —
    /// a different AST path through pre_register_empty_tables.
    #[tokio::test]
    async fn test_BC_2_11_003_F_CSD_P10_001_T29_join_on_funccall_wrapped_insubquery_empty_table_consistent(
    ) {
        let ctx =
            build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");

        // crowdstrike_detections — outer FROM table with data.
        let det_batch = make_two_col_batch(
            "detection_id",
            &["det-001", "det-002"],
            "device_id",
            &["dev-A", "dev-B"],
        );
        register_mem_table(&ctx, "crowdstrike_detections", vec![det_batch])
            .expect("crowdstrike_detections registration must succeed");

        // crowdstrike_devices — INNER JOIN target with data.
        let dev_batch = make_batch("device_id", &["dev-A", "dev-B"]);
        register_mem_table(&ctx, "crowdstrike_devices", vec![dev_batch])
            .expect("crowdstrike_devices registration must succeed");

        // armis_devices — 0 batches.
        // register_mem_table skips registration (pre-fix behavior).
        // pre_register_empty_tables must register armis_devices from the IN-subquery
        // FROM position (F-CSD-P3-001 fix coverage) with a schema-only MemTable so
        // DataFusion can plan the subquery.
        register_mem_table(&ctx, "armis_devices", vec![])
            .expect("register_mem_table with empty batches must not error");

        // Identical SQL shape to T28 — only armis_devices batch count differs.
        let sql = "SELECT det.detection_id \
                   FROM crowdstrike_detections det \
                   INNER JOIN crowdstrike_devices dev \
                   ON coalesce(det.device_id IN (SELECT device_id FROM armis_devices))";

        let ast = PrismQlParser::parse(sql)
            .expect("F-CSD-P10-001 T29: same SQL as T28 — grammar reach confirmed by T28 parse.");

        let result =
            execute_against_session(&ctx, sql, &ast, std::collections::HashMap::new()).await;

        // CONSISTENCY LOCK: empty-table variant must execute successfully with 0 rows.
        //
        // Gate behavior (from T28): does NOT fire E-QUERY-043 for FuncCall-wrapped
        // InSubquery in JOIN ON — walker asymmetry is correct by design.
        //
        // Pre-registration behavior: armis_devices in the IN-subquery FROM position
        // must be covered by pre_register_empty_tables (F-CSD-P3-001 fix).
        // If this fails with QueryExecutionFailed citing armis_devices, the pre-registration
        // fix does NOT recurse into FuncCall-wrapped IN-subquery FROM positions —
        // report as a gap in F-CSD-P3-001 coverage.
        assert!(
            result.is_ok(),
            "F-CSD-P10-001 T29 (consistency lock) / BC-2.11.003 + BC-2.11.005: \
             JOIN ON coalesce(field IN (SELECT ...)) with 0-batch subquery table must \
             execute successfully (Ok with 0 rows). \
             Gate does NOT fire for this shape (T28 GREEN lock). \
             Pre-registration (F-CSD-P3-001) must cover armis_devices in the FuncCall-wrapped \
             IN-subquery FROM position. \
             If this fails with QueryExecutionFailed citing armis_devices, report as \
             F-CSD-P3-001 coverage gap: pre_register_empty_tables does not recurse into \
             FuncCall-wrapped IN-subquery FROM positions. \
             got: {result:?}"
        );

        let batches = result.unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        // armis_devices is empty → IN-subquery set {} → coalesce(FALSE) = FALSE for all
        // detections → INNER JOIN ON FALSE → 0 result rows.
        assert_eq!(
            total_rows, 0,
            "F-CSD-P10-001 T29: INNER JOIN ON coalesce(IN empty subquery) must return \
             0 rows (empty IN-set → condition always FALSE → no INNER JOIN matches). \
             got {total_rows}"
        );
    }
}
