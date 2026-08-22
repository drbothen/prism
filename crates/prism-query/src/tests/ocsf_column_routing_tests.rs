//! Red Gate tests RG-Q-001..RG-Q-009 — OCSF column-name routing through the
//! E-QUERY-038 plan-time column gate (S-ADR058-OCSF-ROUTING-001).
//!
//! # Defect being locked in
//!
//! With `ocsf_column_naming = true` the Arrow schema field names emitted by
//! `pipeline_result_to_record_batch` (and returned by `SELECT *` / `prism_describe`)
//! use `ocsf_field_to_arrow_name(col.ocsf_field)` — e.g. `id` → `finding_info_uid`.
//! However, the column-resolution / validation layer (`check_column_availability` /
//! `TableRegistry::columns_for_table`) still stores and checks raw TOML `col.name` values.
//! Consequence: explicit `SELECT <ocsf_name>` and `WHERE <ocsf_name>` fail with
//! `E-QUERY-038 ColumnNotFound` even though the column IS present in the schema,
//! while raw `col.name` references that are no longer valid Arrow names succeed.
//!
//! # Red Gate summary (pre-fix)
//!
//! | Test | Expected post-fix | Red Gate failure mode |
//! |------|-------------------|-----------------------|
//! | RG-Q-001 | Ok | Err(ColumnNotFound "finding_info_uid") |
//! | RG-Q-002 | Ok | Err(ColumnNotFound "finding_info_title") |
//! | RG-Q-003 | Ok | Err(ColumnNotFound "message") |
//! | RG-Q-004 | Err(ColumnNotFound) | Ok (raw col.name found) |
//! | RG-Q-005 | Err(ColumnNotFound) | Ok (raw col.name found) |
//! | RG-Q-006 | Err(ColumnNotFound), wire available_columns = OCSF names | available_columns = raw TOML col.names |
//! | RG-Q-007 | Ok (green-lock) | stays green |
//! | RG-Q-008 | Ok + Err(ColumnNotFound w/ OCSF avail) (green-lock) | multi-tenant HEAD gate already fixed by Fix B |
//! | RG-Q-009 | Ok (pipe `message` ok) + Err (pipe `description` rejected) | FAIL: pipe-stage binding-seed still raw col.names |
//!
//! # SAP-3 compliance
//! All tests invoke `engine.execute()` from the public query engine surface (SQL or pipe
//! syntax), NOT synthetic AST.  No internal handler is called directly.
//!
//! # BC traceability
//! - BC-2.11.016 (E-QUERY-038 plan-time column gate)
//! - ADR-058 §D (ocsf_column_naming field) / §I6 (flag-gate for index column registration)
//!
//! Story: S-ADR058-OCSF-ROUTING-001 holdout gap (RG-Q-001..009, re-cascade P1).

// Test code — allow expect/unwrap per the project pattern for prism-query test files.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{collections::HashMap, sync::Arc};

use crate::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    table_registry::TableRegistry,
};
use prism_core::{error::PrismError, OrgSlug, SensorId};
use prism_spec_engine::{
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    ResolvedSensorSpec, ResolvedSpecKey,
};

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Minimal no-op credential store — identical pattern to `temporal_typing_tests`.
struct NoopCs;

#[async_trait::async_trait]
impl prism_credentials::CredentialStore for NoopCs {
    async fn get(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, PrismError> {
        Ok(None)
    }
    async fn set(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
        _v: secrecy::SecretString,
    ) -> Result<(), PrismError> {
        Ok(())
    }
    async fn delete(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
    async fn list(
        &self,
        _t: &prism_core::OrgSlug,
    ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError> {
        Ok(vec![])
    }
    async fn exists(
        &self,
        _t: &prism_core::OrgSlug,
        _s: &str,
        _n: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
}

/// Build a `SensorSpec` that mirrors the Claroty alerts table with
/// `ocsf_column_naming = true`.
///
/// Column mapping (TOML col.name → ocsf_field → Arrow name):
///   - `id`            → `finding_info.uid`       → `finding_info_uid`   (Tier-1)
///   - `alert_type_name` → (no ocsf_field)         → `raw_extensions`     (Tier-2)
///   - `status`        → `status`                  → `status`             (Tier-1)
///   - `detected_time` → `time`                    → `time`               (Tier-1)
///   - `updated_time`  → `finding_info.modified_time` → `finding_info_modified_time` (Tier-1)
///   - `description`   → `message`                 → `message`            (Tier-1)
///   - `alert_name`    → `finding_info.title`      → `finding_info_title` (Tier-1)
///
/// The full registered table name is `claroty_alerts`
/// (`{sensor_id}_{table_name}` = `"claroty"_"alerts"`).
///
/// Registered via `TableRegistry::register_sensor` — the same path used at
/// production boot.  The ocsf_column_naming flag must cause the registry to
/// store OCSF-flattened names rather than raw col.names once the fix lands.
fn make_claroty_alerts_spec() -> SensorSpec {
    use prism_core::ColumnType;
    let mut spec = SensorSpec::new(
        "claroty",
        "Claroty xDome (OCSF routing test fixture)",
        AuthType::BearerStatic,
        "https://claroty.invalid",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "detection_finding",
            vec![
                // Tier-1 columns: ocsf_field present → post-fix Arrow name =
                //   ocsf_field_to_arrow_name(ocsf_field)
                ColumnSpec::new(
                    "id",
                    ColumnType::String,
                    Some("finding_info.uid".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "status",
                    ColumnType::String,
                    Some("status".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "detected_time",
                    ColumnType::Datetime,
                    Some("time".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "updated_time",
                    ColumnType::Datetime,
                    Some("finding_info.modified_time".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "description",
                    ColumnType::String,
                    Some("message".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "alert_name",
                    ColumnType::String,
                    Some("finding_info.title".to_string()),
                    vec![],
                ),
                // Tier-2 column: no ocsf_field → post-fix aggregates into `raw_extensions`
                ColumnSpec::new("alert_type_name", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    // ocsf_column_naming is a pub field; SensorSpec::new() defaults to false.
    // Set true to mirror the production claroty.sensor.toml (ADR-058 §D2 / AC-001).
    spec.ocsf_column_naming = true;
    spec
}

/// Build a `QueryEngine` wired with the Claroty alerts spec.
///
/// Uses the single-tenant path (`with_table_registry` only; no resolved_spec_map)
/// so that `check_column_availability` falls through to the registry fallback —
/// the path where the ocsf_column_naming gap lives (engine.rs §M1 fix path).
fn make_claroty_engine() -> QueryEngine {
    let spec = make_claroty_alerts_spec();
    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("RG fixture: register claroty sensor must not fail");

    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

/// Build a `QueryEngine` wired with a CrowdStrike spec (ocsf_column_naming = false).
///
/// Used by RG-Q-007 to verify that the raw col.name path still works for sensors
/// that do NOT opt into OCSF column naming.  `SensorSpec::new()` defaults
/// `ocsf_column_naming` to `false` — no override needed.
fn make_crowdstrike_engine() -> QueryEngine {
    use prism_core::ColumnType;
    let spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike Falcon (OCSF routing regression guard)",
        AuthType::ApiKey,
        "https://api.crowdstrike.invalid",
        vec![TableSpec::new_point_in_time(
            "detections",
            "security_finding",
            vec![
                ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
                ColumnSpec::new("status", ColumnType::String, None, vec![]),
                ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    // Confirm ocsf_column_naming is false (the default — this is the green-lock).
    assert!(
        !spec.ocsf_column_naming,
        "RG-Q-007 fixture: crowdstrike spec must have ocsf_column_naming=false (the default)"
    );

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("RG-Q-007 fixture: register crowdstrike sensor must not fail");

    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_table_registry(registry)
}

// ── RG-Q-001 ─────────────────────────────────────────────────────────────────

/// RG-Q-001 — `SELECT finding_info_uid FROM claroty_alerts` must return `Ok`.
///
/// Post-fix: `table_registry.columns_for_table("claroty_alerts")` must contain
/// `"finding_info_uid"` (the OCSF-flattened Arrow name for `id` /
/// `finding_info.uid`), so `check_column_availability` passes.
///
/// Red Gate failure (pre-fix): the registry stores raw col.name `"id"`.
/// `"finding_info_uid"` is absent → `E-QUERY-038 ColumnNotFound`.
///
/// SAP-3: query enters via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_001_ocsf_select_finding_info_uid_passes_e_query_038() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT finding_info_uid FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_ok(),
        "RG-Q-001 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT finding_info_uid FROM claroty_alerts` must return Ok after the fix — \
         the E-QUERY-038 gate must recognise 'finding_info_uid' as a valid column \
         (ocsf_field_to_arrow_name(\"finding_info.uid\") = \"finding_info_uid\"). \
         Got Err: {:?}",
        result.err()
    );
}

// ── RG-Q-002 ─────────────────────────────────────────────────────────────────

/// RG-Q-002 — `SELECT finding_info_title FROM claroty_alerts` must return `Ok`.
///
/// Post-fix: the registry must contain `"finding_info_title"` (OCSF-flattened Arrow
/// name for `alert_name` / `finding_info.title`, dot → underscore per ADR-058 §I1).
///
/// Red Gate failure (pre-fix): only raw col.name `"alert_name"` is stored → `E-QUERY-038`.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_002_ocsf_select_finding_info_title_passes_e_query_038() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT finding_info_title FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_ok(),
        "RG-Q-002 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT finding_info_title FROM claroty_alerts` must return Ok after the fix — \
         multi-segment dot→underscore flattening: ocsf_field_to_arrow_name(\"finding_info.title\") \
         = \"finding_info_title\". \
         Got Err: {:?}",
        result.err()
    );
}

// ── RG-Q-003 ─────────────────────────────────────────────────────────────────

/// RG-Q-003 — `SELECT * FROM claroty_alerts WHERE message = 'test'` must return `Ok`.
///
/// Post-fix: the registry must contain `"message"` (OCSF-flattened Arrow name for
/// `description` / `message`).  `SELECT *` bypasses the SELECT-position column check;
/// `WHERE message = 'test'` exercises the WHERE-position column check.
///
/// Red Gate failure (pre-fix): only raw col.name `"description"` stored → `"message"` not
/// found → `E-QUERY-038 ColumnNotFound` from the WHERE gate.
///
/// SAP-3: SQL with WHERE clause via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_003_ocsf_where_message_passes_e_query_038() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT * FROM claroty_alerts WHERE message = 'test'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_ok(),
        "RG-Q-003 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT * FROM claroty_alerts WHERE message = 'test'` must return Ok — \
         WHERE-position column validation must accept OCSF flattened name 'message' \
         (ocsf_field_to_arrow_name(\"message\") = \"message\"). \
         Got Err: {:?}",
        result.err()
    );
}

// ── RG-Q-004 ─────────────────────────────────────────────────────────────────

/// RG-Q-004 — `SELECT description FROM claroty_alerts` must return `Err(ColumnNotFound)`.
///
/// `description` is the raw TOML `col.name`; the post-fix Arrow schema uses OCSF-flattened
/// names, so `description` is NOT a valid column name — the correct name is `message`.
/// The `ColumnNotFoundDetails.available_columns` MUST contain `"message"` and MUST NOT
/// contain `"description"`.
///
/// This is a **Stage-2 breaking-change gate**: the raw col.name must stop being
/// accepted after the fix.
///
/// Red Gate failure (pre-fix): the registry stores raw col.name `"description"` →
/// `check_column_availability` returns `Ok` → test assertion on `is_err()` fails.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_004_raw_colname_description_rejected_post_ocsf_fix() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT description FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-Q-004 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT description FROM claroty_alerts` must return Err(ColumnNotFound) after the fix — \
         raw TOML col.name 'description' is not a valid OCSF-mode Arrow column name \
         (the Arrow name is 'message'). Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(&err, PrismError::ColumnNotFound(_)),
        "RG-Q-004: error must be PrismError::ColumnNotFound; got: {err:?}"
    );

    if let PrismError::ColumnNotFound(ref d) = err {
        assert_eq!(
            d.column, "description",
            "RG-Q-004: ColumnNotFoundDetails.column must be 'description'; got: '{}'",
            d.column
        );
        assert!(
            d.available_columns.contains(&"message".to_string()),
            "RG-Q-004: available_columns must contain 'message' (the OCSF flattened name); \
             got: {:?}",
            d.available_columns
        );
        assert!(
            !d.available_columns.contains(&"description".to_string()),
            "RG-Q-004: available_columns must NOT contain raw col.name 'description'; \
             got: {:?}",
            d.available_columns
        );
    }
}

// ── RG-Q-005 ─────────────────────────────────────────────────────────────────

/// RG-Q-005 — `SELECT alert_type_name FROM claroty_alerts` must return `Err(ColumnNotFound)`.
///
/// `alert_type_name` is a raw Tier-2 `col.name` (no `ocsf_field`).  Under
/// `ocsf_column_naming = true`, Tier-2 columns aggregate into `raw_extensions`.
/// After the fix the registry must NOT contain `"alert_type_name"` as an individual
/// column; `available_columns` MUST contain `"raw_extensions"` and MUST NOT
/// contain `"alert_type_name"`.
///
/// Red Gate failure (pre-fix): the registry stores raw col.name `"alert_type_name"` →
/// `check_column_availability` returns `Ok` → test assertion on `is_err()` fails.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_005_tier2_raw_colname_alert_type_name_rejected_post_ocsf_fix() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT alert_type_name FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-Q-005 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT alert_type_name FROM claroty_alerts` must return Err(ColumnNotFound) after the fix — \
         raw Tier-2 col.name 'alert_type_name' is not a valid OCSF-mode Arrow column \
         (Tier-2 columns aggregate into 'raw_extensions'). Got Ok."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(&err, PrismError::ColumnNotFound(_)),
        "RG-Q-005: error must be PrismError::ColumnNotFound; got: {err:?}"
    );

    if let PrismError::ColumnNotFound(ref d) = err {
        assert_eq!(
            d.column, "alert_type_name",
            "RG-Q-005: ColumnNotFoundDetails.column must be 'alert_type_name'; got: '{}'",
            d.column
        );
        assert!(
            d.available_columns.contains(&"raw_extensions".to_string()),
            "RG-Q-005: available_columns must contain 'raw_extensions' \
             (Tier-2 columns aggregate target); got: {:?}",
            d.available_columns
        );
        assert!(
            !d.available_columns.contains(&"alert_type_name".to_string()),
            "RG-Q-005: available_columns must NOT contain raw col.name 'alert_type_name'; \
             got: {:?}",
            d.available_columns
        );
    }
}

// ── RG-Q-006 ─────────────────────────────────────────────────────────────────

/// RG-Q-006 — `SELECT nonexistent_col FROM claroty_alerts` — wire-shape assertion on
/// `available_columns` JSON payload.
///
/// Post-fix, `ColumnNotFoundDetails.available_columns` must contain ONLY the OCSF-flattened
/// Arrow column names (and pseudo-columns `class_uid`, `_sensor`), NOT any raw TOML
/// `col.name`.  This test asserts on the **serialized JSON wire payload** of
/// `available_columns` per CLAUDE.md wire-shape discipline (2026-07-13).
///
/// Expected available columns (sorted, OCSF-mode):
///   `_sensor`, `class_uid`, `finding_info_modified_time`, `finding_info_title`,
///   `finding_info_uid`, `message`, `raw_extensions`, `status`, `time`
///
/// NOT expected in available_columns (raw TOML col.names):
///   `alert_name`, `alert_type_name`, `description`, `detected_time`,
///   `id`, `updated_time`
///
/// Red Gate failure (pre-fix): `available_columns` contains raw TOML col.names →
/// the wire-shape assertions fail.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
/// SID-2: asserts on the full serialized JSON string, not only component fields.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_006_ocsf_error_available_columns_wire_shape() {
    let engine = make_claroty_engine();

    let result = engine
        .execute(
            "SELECT nonexistent_col FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-Q-006 (S-ADR058-OCSF-ROUTING-001): \
         `SELECT nonexistent_col FROM claroty_alerts` must return Err(ColumnNotFound). \
         Got Ok."
    );

    let err = result.unwrap_err();
    let PrismError::ColumnNotFound(ref d) = err else {
        panic!("RG-Q-006: error must be PrismError::ColumnNotFound; got: {err:?}");
    };

    // Wire-shape assertion (CLAUDE.md §Conventions wire-shape discipline 2026-07-13):
    // Serialize available_columns to the JSON array string the MCP error_mapping layer
    // would emit — `error_obj["available_columns"]` in `prism_error_to_structured_call_result`.
    let available_json = serde_json::to_string(&d.available_columns)
        .expect("RG-Q-006: available_columns must be JSON-serializable");

    // --- Positive assertions: OCSF-flattened names MUST be present ---
    for expected_col in &[
        "finding_info_uid",
        "status",
        "time",
        "finding_info_modified_time",
        "message",
        "finding_info_title",
        "raw_extensions",
    ] {
        assert!(
            d.available_columns.contains(&(*expected_col).to_string()),
            "RG-Q-006: available_columns must contain OCSF name '{}'; \
             got wire JSON: {}",
            expected_col,
            available_json
        );
    }

    // --- Negative assertions: raw TOML col.names must NOT appear ---
    for forbidden_col in &[
        "id",
        "alert_name",
        "description",
        "detected_time",
        "updated_time",
        "alert_type_name",
    ] {
        assert!(
            !d.available_columns.contains(&(*forbidden_col).to_string()),
            "RG-Q-006: available_columns must NOT contain raw TOML col.name '{}'; \
             got wire JSON: {}",
            forbidden_col,
            available_json
        );
    }

    // SID-2: assert on the full serialized JSON wire payload (not only component fields).
    // Verify the JSON array contains at least the required OCSF names.
    assert!(
        available_json.contains("finding_info_uid"),
        "RG-Q-006 (wire): serialized available_columns JSON must contain 'finding_info_uid'; \
         got: {}",
        available_json
    );
    assert!(
        available_json.contains("raw_extensions"),
        "RG-Q-006 (wire): serialized available_columns JSON must contain 'raw_extensions'; \
         got: {}",
        available_json
    );
    // Verify none of the forbidden raw col.names appear in the JSON wire output.
    for forbidden_col in &["\"id\"", "\"description\"", "\"alert_name\""] {
        assert!(
            !available_json.contains(forbidden_col),
            "RG-Q-006 (wire): serialized available_columns JSON must NOT contain raw col.name {}; \
             got: {}",
            forbidden_col,
            available_json
        );
    }
}

// ── RG-Q-007 ─────────────────────────────────────────────────────────────────

/// RG-Q-007 — Green-lock regression guard: raw col.name on a non-OCSF sensor
/// (`ocsf_column_naming = false`) still returns `Ok`.
///
/// CrowdStrike uses `ocsf_column_naming = false` (the default).  Querying
/// `SELECT detection_id FROM crowdstrike_detections` must succeed BOTH before
/// AND after the fix.  This test MUST remain green throughout.
///
/// Rationale: the fix must be scoped to sensors with `ocsf_column_naming = true`.
/// The gate must not regress for the majority of sensors that remain in raw-name mode.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_007_non_ocsf_sensor_raw_colname_still_passes_green_lock() {
    let engine = make_crowdstrike_engine();

    let result = engine
        .execute(
            "SELECT detection_id FROM crowdstrike_detections",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_ok(),
        "RG-Q-007 (S-ADR058-OCSF-ROUTING-001 green-lock): \
         `SELECT detection_id FROM crowdstrike_detections` must return Ok — \
         CrowdStrike has ocsf_column_naming=false; raw col.name 'detection_id' \
         must remain valid. This test must stay green before AND after the fix. \
         Got Err: {:?}",
        result.err()
    );
}

// ── Multi-tenant fixture helper ───────────────────────────────────────────────

/// Build a `(ResolvedSpecKey, ResolvedSensorSpec)` pair for the Claroty sensor with
/// `ocsf_column_naming = true`, scoped to `org`.
///
/// Pattern mirrors `make_resolved` in `e_query_pedagogical.rs` and
/// `make_sec003_resolved_spec_map` in `explain_tests.rs`.
fn make_claroty_resolved(org: &str) -> (ResolvedSpecKey, ResolvedSensorSpec) {
    use prism_core::ColumnType;

    let mut spec = SensorSpec::new(
        "claroty",
        "Claroty xDome (multi-tenant OCSF fixture)",
        AuthType::BearerStatic,
        "https://claroty.invalid",
        vec![TableSpec::new_point_in_time(
            "alerts",
            "detection_finding",
            vec![
                ColumnSpec::new(
                    "id",
                    ColumnType::String,
                    Some("finding_info.uid".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "status",
                    ColumnType::String,
                    Some("status".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "detected_time",
                    ColumnType::Datetime,
                    Some("time".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "updated_time",
                    ColumnType::Datetime,
                    Some("finding_info.modified_time".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "description",
                    ColumnType::String,
                    Some("message".to_string()),
                    vec![],
                ),
                ColumnSpec::new(
                    "alert_name",
                    ColumnType::String,
                    Some("finding_info.title".to_string()),
                    vec![],
                ),
                // Tier-2 column: no ocsf_field → aggregates into raw_extensions
                ColumnSpec::new("alert_type_name", ColumnType::String, None, vec![]),
            ],
            vec![],
        )],
        None,
        "1.0.0",
        vec![],
    );
    spec.ocsf_column_naming = true;

    let overlay_toml = format!("extends = \"claroty\"\ninstance_id = \"claroty@{org}\"");
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .expect("RG-Q-008/009 fixture: SensorInstanceOverlay TOML must parse");
    let org_slug = OrgSlug::new(org);
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
    let key: ResolvedSpecKey = (org_slug, SensorId::new("claroty"));
    (key, resolved)
}

/// Build a multi-tenant `QueryEngine` with the Claroty OCSF spec wired via
/// `with_resolved_spec_map` + `with_table_registry`.
///
/// The registry is populated from the same spec so E-QUERY-037 (table gate) passes.
/// Engine executes queries on behalf of org "acme".
fn make_claroty_multitenant_engine() -> QueryEngine {
    let (key, resolved) = make_claroty_resolved("acme");
    let sensor_spec = resolved.spec.clone();

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&sensor_spec)
        .expect("RG-Q-008/009 fixture: register claroty sensor must not fail");

    let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
    spec_map.insert(key, resolved);

    QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCs),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(crate::scoping::ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        crate::cache::CacheConfig::default(),
    )
    .with_resolved_spec_map(Arc::new(spec_map))
    .with_table_registry(registry)
}

// ── RG-Q-008 ─────────────────────────────────────────────────────────────────

/// RG-Q-008 — Multi-tenant HEAD gate: OCSF-flattened SELECT passes; raw col.name rejected.
///
/// This is a **green-lock** test for the multi-tenant HEAD gate path
/// (`check_column_availability` with `resolved_spec_map = Some`) which was already
/// repaired by Fix B (re-cascade P1 HIGH-001 coverage).  It MUST PASS.
///
/// Sub-case A (must Ok): `SELECT finding_info_uid FROM claroty_alerts` via multi-tenant engine.
///   The multi-tenant `check_column_availability` path (Fix B) now returns OCSF-flattened
///   names; `finding_info_uid` must be found.
///
/// Sub-case B (must Err ColumnNotFound): `SELECT id FROM claroty_alerts` — raw col.name.
///   After Fix B the multi-tenant path returns OCSF names only; raw `id` is absent.
///   `available_columns` must contain `finding_info_uid` and NOT contain `id`.
///
/// If this test unexpectedly fails, it signals Fix B regressed in this engine path.
///
/// SAP-3: SQL SELECT via `engine.execute()` public surface.
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_008_multitenant_ocsf_head_projection() {
    let engine = make_claroty_multitenant_engine();

    // Sub-case A: OCSF-flattened name must pass E-QUERY-038 on the multi-tenant HEAD gate.
    let result_a = engine
        .execute(
            "SELECT finding_info_uid FROM claroty_alerts",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result_a.is_ok(),
        "RG-Q-008A (S-ADR058-OCSF-ROUTING-001 green-lock, re-cascade P1 HIGH-001): \
         `SELECT finding_info_uid FROM claroty_alerts` (MULTI-TENANT) must return Ok — \
         Fix B wired OCSF-flattened names into check_column_availability multi-tenant path. \
         Unexpected failure signals Fix B regression. Got Err: {:?}",
        result_a.err()
    );

    // Sub-case B: raw col.name must be rejected on the multi-tenant HEAD gate.
    let result_b = engine
        .execute("SELECT id FROM claroty_alerts", QueryOptions::default())
        .await;

    assert!(
        result_b.is_err(),
        "RG-Q-008B (S-ADR058-OCSF-ROUTING-001 green-lock): \
         `SELECT id FROM claroty_alerts` (MULTI-TENANT) must return Err(ColumnNotFound) — \
         raw TOML col.name 'id' is not a valid OCSF-mode Arrow column. Got Ok."
    );

    if let Err(PrismError::ColumnNotFound(ref d)) = result_b {
        assert_eq!(
            d.column, "id",
            "RG-Q-008B: ColumnNotFoundDetails.column must be 'id'; got: '{}'",
            d.column
        );
        assert!(
            d.available_columns
                .contains(&"finding_info_uid".to_string()),
            "RG-Q-008B: available_columns must contain OCSF name 'finding_info_uid'; \
             got: {:?}",
            d.available_columns
        );
        assert!(
            !d.available_columns.contains(&"id".to_string()),
            "RG-Q-008B: available_columns must NOT contain raw col.name 'id'; \
             got: {:?}",
            d.available_columns
        );
    }
}

// ── RG-Q-009 ─────────────────────────────────────────────────────────────────

/// RG-Q-009 — Multi-tenant pipe-stage binding-seed: OCSF gap in
/// `get_initial_available_columns` (re-cascade P1 MED-002, the RED test).
///
/// `get_initial_available_columns` (engine.rs multi-tenant branch) seeds the pipe-stage
/// binding context with `c.name.clone()` — raw TOML col.names — instead of
/// OCSF-flattened Arrow names.  This means:
///
///   - `FROM claroty_alerts | where message = 'x'` — OCSF name `message` is not in
///     the binding seed (`description` is) → plan-time rejection → Err(ColumnNotFound).
///     **Post-fix this MUST return Ok.**
///
///   - `FROM claroty_alerts | where description = 'x'` — raw col.name `description` IS
///     in the binding seed (wrongly) → plan-time acceptance → Ok.
///     **Post-fix this MUST return Err(ColumnNotFound).**
///
/// Pre-fix state (RED):
///   - pipe `message` query → Err(ColumnNotFound) when it SHOULD be Ok
///   - pipe `description` query → Ok when it SHOULD be Err
///
/// Both sub-cases fail together before the fix; both must pass after.
///
/// SAP-3: pipe-mode query via `engine.execute()` public surface (FROM … | where syntax).
#[tokio::test]
async fn test_BC_2_11_016_RG_Q_009_multitenant_ocsf_pipe_stage() {
    let engine = make_claroty_multitenant_engine();

    // Sub-case A: pipe WHERE with OCSF-flattened name must return Ok post-fix.
    // Pre-fix: `message` absent from get_initial_available_columns → Err(ColumnNotFound). RED.
    let result_pipe_ocsf = engine
        .execute(
            "FROM claroty_alerts | where message = 'x'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result_pipe_ocsf.is_ok(),
        "RG-Q-009A (S-ADR058-OCSF-ROUTING-001 RED, re-cascade P1 MED-002): \
         `FROM claroty_alerts | where message = 'x'` (MULTI-TENANT pipe) must return Ok — \
         `get_initial_available_columns` multi-tenant branch must seed the pipe binding \
         context with OCSF-flattened names so 'message' (= ocsf_field_to_arrow_name(\"message\")) \
         is valid. Pre-fix: Err(ColumnNotFound) because binding seed still uses raw col.names. \
         Got Err: {:?}",
        result_pipe_ocsf.err()
    );

    // Sub-case B: pipe WHERE with raw col.name must return Err(ColumnNotFound) post-fix.
    // Pre-fix: `description` IS in get_initial_available_columns (raw seed) → Ok. RED.
    let result_pipe_raw = engine
        .execute(
            "FROM claroty_alerts | where description = 'x'",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result_pipe_raw.is_err(),
        "RG-Q-009B (S-ADR058-OCSF-ROUTING-001 RED, re-cascade P1 MED-002): \
         `FROM claroty_alerts | where description = 'x'` (MULTI-TENANT pipe) must return \
         Err(ColumnNotFound) — raw TOML col.name 'description' must be rejected in OCSF mode \
         (the Arrow column is 'message'). \
         Pre-fix: Ok because binding seed still carries raw col.name 'description'. \
         Got Ok."
    );

    if let Err(PrismError::ColumnNotFound(ref d)) = result_pipe_raw {
        assert_eq!(
            d.column, "description",
            "RG-Q-009B: ColumnNotFoundDetails.column must be 'description'; got: '{}'",
            d.column
        );
        assert!(
            d.available_columns.contains(&"message".to_string()),
            "RG-Q-009B: available_columns must contain OCSF name 'message'; \
             got: {:?}",
            d.available_columns
        );
        assert!(
            !d.available_columns.contains(&"description".to_string()),
            "RG-Q-009B: available_columns must NOT contain raw col.name 'description'; \
             got: {:?}",
            d.available_columns
        );
    }
}
