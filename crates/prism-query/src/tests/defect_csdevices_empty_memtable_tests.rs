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
//! # Red Gate (BC-5.38.001)
//!
//! ALL three tests FAIL before the empty-MemTable fix lands in
//! `register_mem_table` / `run_materialization_pipeline`.
//! Failure mode: `result.expect_err()` in the test helper receives `Ok(0 rows)`
//! would PASS, but the assertion `result.is_ok()` fails with
//! `Err(QueryExecutionFailed { detail: "...crowdstrike_devices..." })`.

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::{
        array::StringArray,
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
}
