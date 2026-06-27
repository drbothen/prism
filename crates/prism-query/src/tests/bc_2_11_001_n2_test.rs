//! Red Gate tests for S-DEMO-FIDELITY-REMEDIATION-001 AC-N2 — BC-2.11.001 v1.14.
//!
//! Finding N2 (EC-11-067): `FROM cyberint.alerts` (dot-notation) in a FROM target
//! position routes to the sensor fan-out as a dot-notation string, producing a silent
//! E-SENSOR-030 partial failure (0 rows returned, isError=false). It MUST instead
//! return `PrismError::TableNotAvailable` (E-QUERY-037) at plan time with
//! `did_you_mean: "cyberint_alerts"`.
//!
//! Root cause: `check_availability_gate` in `table_registry.rs` converts `External {
//! sensor, table }` AST nodes to `"{sensor}_{table}"` (underscore form) and then checks
//! if the UNDERSCORE form is registered. If `cyberint_alerts` IS registered, the gate
//! PASSES — so dot-notation queries silently route to fan-out. The fix: when an AST
//! source is `External { sensor, table }` (i.e., user wrote `FROM sensor.table`), the
//! gate MUST reject it as an invalid FROM target with E-QUERY-037, regardless of whether
//! the underscore form exists — because `FROM sensor.table` is not the valid PrismQL
//! FROM syntax (only `FROM sensor_table` is valid for FROM targets).
//!
//! # Regression guard
//!
//! BC-2.11.023 / ADR-046 filter-mode dot-notation MUST NOT regress:
//! `crowdstrike_detections | severity='HIGH'` (filter mode uses `<table_name> |
//! <predicate>` syntax with underscore-qualified table names) must continue to work.
//! The TableRegistry check applies only to FROM-target resolution, not filter-mode
//! source refs.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_001_n2_dot_notation_from_target_e_query_037 | AC-N2 | BC-2.11.001 v1.14 EC-11-067 |
//! | test_bc_2_11_001_n2_filter_mode_underscore_no_regression | regression guard | BC-2.11.023 / ADR-046 |

use crate::table_registry::TableRegistry;
use prism_core::error::PrismError;
use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

// ── Test fixture helper ───────────────────────────────────────────────────────

/// Build a `TableRegistry` with `cyberint_alerts` and `crowdstrike_detections` registered.
fn make_registry_with_cyberint_crowdstrike() -> TableRegistry {
    let registry = TableRegistry::new();

    let cyberint_spec = SensorSpec::new(
        "cyberint",
        "Cyberint sensor",
        AuthType::ApiKey,
        "https://api.cyberint.com",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&cyberint_spec)
        .expect("register cyberint must not fail");

    let crowdstrike_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![TableSpec::new_point_in_time(
            "detections",
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    registry
        .register_sensor(&crowdstrike_spec)
        .expect("register crowdstrike must not fail");

    registry
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.11.001 v1.14 EC-11-067 — AC-N2 Red Gate test.
///
/// `FROM cyberint.alerts` (pipe mode) and `SELECT * FROM crowdstrike.detections` (SQL mode)
/// are dot-notation FROM targets. The `TableRegistry` only stores underscore-qualified names
/// (`cyberint_alerts`, `crowdstrike_detections`). Dot-notation in FROM target position MUST
/// return `PrismError::TableNotAvailable` (E-QUERY-037) at plan time with:
/// - `table: "cyberint.alerts"` (the name as written)
/// - `did_you_mean: "cyberint_alerts"` (the correct underscore form)
///
/// RED GATE: `check_availability_gate` currently converts `External { sensor, table }` AST
/// nodes to `"{sensor}_{table}"` (e.g., `"cyberint_alerts"`) BEFORE checking registration.
/// Since `cyberint_alerts` IS registered, the gate returns Ok(()) and the dot-notation
/// string routes to the fan-out, producing a silent E-SENSOR-030 partial failure.
/// After the fix: `External { sensor, table }` nodes in FROM target position must be
/// rejected with E-QUERY-037 (the dot-notation itself is the error).
#[test]
fn test_bc_2_11_001_n2_dot_notation_from_target_e_query_037() {
    let registry = make_registry_with_cyberint_crowdstrike();

    // ── Pipe mode: FROM cyberint.alerts ──────────────────────────────────────
    //
    // `cyberint.alerts` is classified as External { sensor: "cyberint", table: "alerts" }.
    // Current gate converts this to `cyberint_alerts` and finds it registered → Ok(()).
    // After fix: dot-notation in FROM position → E-QUERY-037 with did_you_mean.
    let pipe_result =
        registry.check_availability_gate("FROM cyberint.alerts | limit 10", None, None);

    match &pipe_result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            // After fix: table field must be the dot-notation string as written.
            assert_eq!(
                details.table, "cyberint.alerts",
                "BC-2.11.001 AC-N2: TableNotAvailable.table must be 'cyberint.alerts' \
                 (the name as written). Got: {:?}",
                details.table
            );
            // After fix: did_you_mean must be non-empty and contain the underscore form.
            // did_you_mean is a pre-formatted String: "" (no suggestion) or
            // " Did you mean: 'cyberint_alerts'?" (when Levenshtein ≤ 3 match found).
            assert!(
                details.did_you_mean.contains("cyberint_alerts"),
                "BC-2.11.001 AC-N2: TableNotAvailable.did_you_mean must contain 'cyberint_alerts'. \
                 Got: {:?}",
                details.did_you_mean
            );
        }
        Ok(()) => panic!(
            "BC-2.11.001 AC-N2 PIPE RED GATE: FROM cyberint.alerts must return \
             Err(PrismError::TableNotAvailable) — current code converts dot-notation to \
             underscore form and passes the gate. Got Ok(())."
        ),
        Err(other) => panic!(
            "BC-2.11.001 AC-N2 PIPE: expected Err(PrismError::TableNotAvailable), \
             got different error: {other:?}"
        ),
    }

    // ── SQL mode: SELECT * FROM crowdstrike.detections ───────────────────────
    //
    // EC-11-067 covers all modes (SQL, Pipe, SqlPipe).
    let sql_result = registry.check_availability_gate(
        "SELECT * FROM crowdstrike.detections LIMIT 5",
        None,
        None,
    );

    match &sql_result {
        Err(PrismError::TableNotAvailable(ref details)) => {
            assert_eq!(
                details.table, "crowdstrike.detections",
                "BC-2.11.001 AC-N2 SQL: TableNotAvailable.table must be 'crowdstrike.detections'. \
                 Got: {:?}",
                details.table
            );
            assert!(
                details.did_you_mean.contains("crowdstrike_detections"),
                "BC-2.11.001 AC-N2 SQL: did_you_mean must contain 'crowdstrike_detections'. \
                 Got: {:?}",
                details.did_you_mean
            );
        }
        Ok(()) => panic!(
            "BC-2.11.001 AC-N2 SQL RED GATE: SELECT * FROM crowdstrike.detections must return \
             Err(PrismError::TableNotAvailable) — current code converts dot-notation to \
             underscore form and passes the gate. Got Ok(())."
        ),
        Err(other) => panic!(
            "BC-2.11.001 AC-N2 SQL: expected Err(PrismError::TableNotAvailable), \
             got different error: {other:?}"
        ),
    }
}

/// BC-2.11.023 / ADR-046 regression guard — filter-mode underscore-qualified table names.
///
/// A filter-mode query `crowdstrike_detections | severity='HIGH'` uses underscore-qualified
/// table names (NOT dot-notation in FROM target position). The `TableRegistry` check must
/// NOT regress this valid query.
///
/// BC-2.11.023 / ADR-046: filter mode uses `<table_name> | <predicate>` syntax where
/// `<table_name>` is always an underscore-qualified name. The dot-notation rejection fix
/// targets FROM-target position only and must not affect filter mode.
///
/// This test PASSES before the fix and MUST continue to PASS after the fix.
/// If this test FAILS after the fix, the fix over-eagerly rejects valid filter-mode queries.
#[test]
fn test_bc_2_11_001_n2_filter_mode_underscore_no_regression() {
    let registry = make_registry_with_cyberint_crowdstrike();

    // Filter mode: table is underscore-qualified, no dot-notation in FROM position.
    // This must return Ok(()) — the table is registered.
    let result =
        registry.check_availability_gate("crowdstrike_detections | severity='HIGH'", None, None);

    assert!(
        result.is_ok(),
        "BC-2.11.023 / ADR-046 regression guard: filter-mode query \
         'crowdstrike_detections | severity=HIGH' must pass the availability gate \
         (crowdstrike_detections IS registered). \
         Got Err: {result:?}"
    );
}
