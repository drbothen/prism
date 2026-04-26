// S-2.03 — Internal table tests.
//
// Covers BC-2.15.011 (internal table registration) and BC-2.15.009 (virtual fields).
//
// Test naming convention: test_BC_S_SS_NNN_[assertion_name]
// All tests pass. AC-12 (`column_name()` tests) was green by design from the
// start because the pure data mapping was implemented immediately. See the TDD
// log for the rationale.
//
// AC map:
//   AC-7  → test_BC_2_15_011_get_descriptor_prism_alerts_fields
//   AC-8  → test_BC_2_15_011_get_descriptor_prism_audit_requires_audit_read
//   AC-9  → test_BC_2_15_011_check_table_access_audit_without_capability_denied
//   AC-10 → test_BC_2_15_011_check_table_access_alerts_any_caps_ok
//   AC-11 → test_BC_2_15_011_scan_limit_default (serial — env var)
//           test_BC_2_15_011_scan_limit_valid_numeric (serial — env var)
//           test_BC_2_15_011_scan_limit_invalid_string (serial — env var)
//   AC-12 → test_BC_2_15_009_virtual_field_column_names [GREEN-BY-DESIGN]
//   AC-13 → test_BC_2_15_011_all_descriptors_count_and_names
//   AC-14 → test_BC_2_15_011_diff_results_columns_metadata_only
//   EC-005 → test_BC_2_15_011_ec005_get_descriptor_unknown_table_returns_none

#![allow(non_snake_case)]

#[cfg(test)]
mod inner {
    use prism_core::{
        CapabilityEffect, CapabilityPath, ClientCapabilities, InternalColumnType, StorageDomain,
        VirtualField,
    };

    use crate::internal_tables::{all_descriptors, check_table_access, get_descriptor, scan_limit};

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Build `ClientCapabilities` that includes `audit.read = Allow`.
    fn caps_with_audit_read() -> ClientCapabilities {
        let mut caps = ClientCapabilities::new();
        let path =
            CapabilityPath::new("audit.read").expect("test helper: 'audit.read' is valid path");
        caps.grant(path, CapabilityEffect::Allow);
        caps
    }

    /// Build `ClientCapabilities` with no rules (deny-by-default — no audit.read).
    fn caps_without_audit_read() -> ClientCapabilities {
        ClientCapabilities::new()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-7: get_descriptor("prism_alerts") fields
    // BC-2.15.011 postcondition — each queryable domain registered.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-7 (BC-2.15.011 postcondition): `get_descriptor("prism_alerts")` returns
    /// a descriptor with:
    /// - `table_name == "prism_alerts"`
    /// - `domain == Some(StorageDomain::Alerts)`
    /// - `requires_audit_read == false`
    /// - `columns[0] == ("alert_id", InternalColumnType::Text)`
    #[test]
    fn test_BC_2_15_011_get_descriptor_prism_alerts_fields() {
        let desc = get_descriptor("prism_alerts")
            .expect("AC-7 (BC-2.15.011): get_descriptor(\"prism_alerts\") must return Some");

        assert_eq!(
            desc.table_name, "prism_alerts",
            "AC-7 (BC-2.15.011): table_name must be \"prism_alerts\""
        );
        assert_eq!(
            desc.domain,
            Some(StorageDomain::Alerts),
            "AC-7 (BC-2.15.011): domain must be Some(StorageDomain::Alerts)"
        );
        assert!(
            !desc.requires_audit_read,
            "AC-7 (BC-2.15.011): prism_alerts must NOT require audit.read capability"
        );

        let first_col = desc
            .columns
            .first()
            .expect("AC-7 (BC-2.15.011): columns must not be empty");
        assert_eq!(
            first_col.0, "alert_id",
            "AC-7 (BC-2.15.011): first column name must be \"alert_id\""
        );
        assert_eq!(
            first_col.1,
            InternalColumnType::Text,
            "AC-7 (BC-2.15.011): first column type must be InternalColumnType::Text"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-8: get_descriptor("prism_audit") requires_audit_read == true
    // BC-2.15.011 error case E-QUERY-011.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-8 (BC-2.15.011 E-QUERY-011): `get_descriptor("prism_audit")` returns a
    /// descriptor where `requires_audit_read == true`.
    #[test]
    fn test_BC_2_15_011_get_descriptor_prism_audit_requires_audit_read() {
        let desc = get_descriptor("prism_audit")
            .expect("AC-8 (BC-2.15.011): get_descriptor(\"prism_audit\") must return Some");

        assert!(
            desc.requires_audit_read,
            "AC-8 (BC-2.15.011 E-QUERY-011): prism_audit descriptor must have \
             requires_audit_read == true"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-9: check_table_access denied when audit.read capability absent
    // BC-2.15.011 error case E-QUERY-011.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-9 (BC-2.15.011 E-QUERY-011): `check_table_access(audit_descriptor, caps)`
    /// where `caps` does NOT have `audit.read = Allow` returns
    /// `Err(PrismError::AuditTableAccessDenied)` and the error Display contains
    /// "audit.read capability".
    #[test]
    fn test_BC_2_15_011_check_table_access_audit_without_capability_denied() {
        let audit_desc = get_descriptor("prism_audit")
            .expect("AC-9 setup: get_descriptor(\"prism_audit\") must return Some");
        let caps = caps_without_audit_read();

        let result = check_table_access(audit_desc, &caps);

        assert!(
            result.is_err(),
            "AC-9 (BC-2.15.011 E-QUERY-011): check_table_access for prism_audit without \
             audit.read must return Err"
        );

        let err = result.unwrap_err();
        let display = err.to_string();
        assert!(
            display.contains("audit.read capability"),
            "AC-9 (BC-2.15.011 E-QUERY-011): error Display must contain \"audit.read capability\" \
             to surface typos in the Display impl; got: {display}"
        );

        // Also verify the variant — anchors the type, not just the string
        assert!(
            matches!(err, prism_core::PrismError::AuditTableAccessDenied),
            "AC-9 (BC-2.15.011 E-QUERY-011): error must be PrismError::AuditTableAccessDenied"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-10: check_table_access for non-audit table always returns Ok
    // BC-2.15.011 postcondition.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-10 (BC-2.15.011 postcondition): `check_table_access(alerts_descriptor, any_caps)`
    /// returns `Ok(())` — non-audit tables require no capability.
    #[test]
    fn test_BC_2_15_011_check_table_access_alerts_any_caps_ok() {
        let alerts_desc = get_descriptor("prism_alerts")
            .expect("AC-10 setup: get_descriptor(\"prism_alerts\") must return Some");

        // Verify with empty capabilities (deny-by-default)
        let empty_caps = caps_without_audit_read();
        assert_eq!(
            check_table_access(alerts_desc, &empty_caps),
            Ok(()),
            "AC-10 (BC-2.15.011): check_table_access for prism_alerts with empty caps \
             must return Ok(()) — non-audit tables are always accessible"
        );

        // Also verify with full audit.read capability (still Ok)
        let full_caps = caps_with_audit_read();
        assert_eq!(
            check_table_access(alerts_desc, &full_caps),
            Ok(()),
            "AC-10 (BC-2.15.011): check_table_access for prism_alerts with audit.read \
             must also return Ok(())"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-11: scan_limit() reads PRISM_MAX_INTERNAL_TABLE_SCAN from env
    // BC-2.15.011 — configurable scan limit with default 50_000.
    //
    // CRITICAL: These three tests mutate the PROCESS-GLOBAL env var.
    // They are grouped sequentially inside a single test function and guarded
    // with a static Mutex to prevent races with other tests.
    //
    // Alternative: add `serial_test` crate — but that requires a Cargo.toml
    // change. We use a module-level mutex here to avoid adding a dependency.
    // ─────────────────────────────────────────────────────────────────────────

    const SCAN_LIMIT_VAR: &str = "PRISM_MAX_INTERNAL_TABLE_SCAN";

    /// AC-11a (BC-2.15.011): when `PRISM_MAX_INTERNAL_TABLE_SCAN` is not set,
    /// `scan_limit()` returns the default value `50_000`.
    ///
    /// Uses a mutex guard to ensure this test does not run concurrently with
    /// the other scan-limit tests that also mutate the env var.
    #[test]
    fn test_BC_2_15_011_scan_limit_default() {
        let _guard = ENV_VAR_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: single-threaded under the mutex
        unsafe {
            std::env::remove_var(SCAN_LIMIT_VAR);
        }
        let result = scan_limit();
        assert_eq!(
            result, 50_000,
            "AC-11a (BC-2.15.011): scan_limit() with no env var must return the \
             spec default of 50_000 (not a constant from the stub)"
        );
    }

    /// AC-11b (BC-2.15.011): when `PRISM_MAX_INTERNAL_TABLE_SCAN = "1000"`,
    /// `scan_limit()` returns `1000`.
    #[test]
    fn test_BC_2_15_011_scan_limit_valid_numeric() {
        let _guard = ENV_VAR_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: single-threaded under the mutex
        unsafe {
            std::env::set_var(SCAN_LIMIT_VAR, "1000");
        }
        let result = scan_limit();
        unsafe {
            std::env::remove_var(SCAN_LIMIT_VAR);
        }
        assert_eq!(
            result, 1000,
            "AC-11b (BC-2.15.011): scan_limit() with env var \"1000\" must return 1000"
        );
    }

    /// AC-11c / EC-004 (BC-2.15.011): when `PRISM_MAX_INTERNAL_TABLE_SCAN = "abc"`,
    /// `scan_limit()` returns the default `50_000` — no panic.
    #[test]
    fn test_BC_2_15_011_scan_limit_invalid_string() {
        let _guard = ENV_VAR_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: single-threaded under the mutex
        unsafe {
            std::env::set_var(SCAN_LIMIT_VAR, "abc");
        }
        let result = scan_limit();
        unsafe {
            std::env::remove_var(SCAN_LIMIT_VAR);
        }
        assert_eq!(
            result, 50_000,
            "AC-11c / EC-004 (BC-2.15.011): scan_limit() with non-numeric env var must return \
             the spec default of 50_000 without panicking"
        );
    }

    /// Mutex ensuring the three env-var-touching scan_limit tests run sequentially.
    static ENV_VAR_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // ─────────────────────────────────────────────────────────────────────────
    // AC-12: VirtualField::column_name() mappings
    // BC-2.15.009 postcondition — underscore-prefixed queryable columns.
    //
    // GREEN-BY-DESIGN: `column_name()` was fully implemented in the
    // stub (pure data mapping, no todo!()). This test is retained for type-system
    // regression coverage. See TDD log for the decision record.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-12 (BC-2.15.009 postcondition — GREEN-BY-DESIGN):
    /// `VirtualField::column_name()` returns the correct underscore-prefixed
    /// column name for each variant.
    ///
    /// This test is expected to PASS because the stub author
    /// implemented `column_name()` as pure data (no `todo!()`). This is
    /// intentional — see the red-gate-log.md decision record.
    #[test]
    fn test_BC_2_15_009_virtual_field_column_names() {
        assert_eq!(
            VirtualField::Sensor.column_name(),
            "_sensor",
            "AC-12 (BC-2.15.009): VirtualField::Sensor must map to \"_sensor\""
        );
        assert_eq!(
            VirtualField::Client.column_name(),
            "_client",
            "AC-12 (BC-2.15.009): VirtualField::Client must map to \"_client\""
        );
        assert_eq!(
            VirtualField::SourceTable.column_name(),
            "_source_table",
            "AC-12 (BC-2.15.009): VirtualField::SourceTable must map to \"_source_table\""
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-13: all_descriptors() returns exactly 7 entries with the right names
    // BC-2.15.011 postcondition — all seven domains registered.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-13 (BC-2.15.011 postcondition): `all_descriptors()` returns exactly 7
    /// descriptors — one per BC-2.15.011 registered table.
    ///
    /// Expected names: prism_alerts, prism_cases, prism_rules, prism_schedules,
    /// prism_diff_results, prism_audit, prism_aliases.
    ///
    /// Count is asserted against the literal `7` (not `>= 7`) — the spec defines
    /// exactly seven internal tables for this release.
    #[test]
    fn test_BC_2_15_011_all_descriptors_count_and_names() {
        let descs = all_descriptors();

        assert_eq!(
            descs.len(),
            7,
            "AC-13 (BC-2.15.011 postcondition): all_descriptors() must return exactly 7 \
             entries — one per registered internal table. Got: {}",
            descs.len()
        );

        let names: Vec<&str> = descs.iter().map(|d| d.table_name).collect();

        let expected = [
            "prism_alerts",
            "prism_cases",
            "prism_rules",
            "prism_schedules",
            "prism_diff_results",
            "prism_audit",
            "prism_aliases",
        ];

        for expected_name in &expected {
            assert!(
                names.contains(expected_name),
                "AC-13 (BC-2.15.011): all_descriptors() must contain \"{expected_name}\"; \
                 got: {names:?}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-14: diff_results_columns() — metadata-only, no "previous_results" column
    // BC-2.15.011 postcondition.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-14 (BC-2.15.011 postcondition): `get_descriptor("prism_diff_results").columns`
    /// contains exactly the six metadata columns and does NOT contain any column
    /// named "previous_results" (the raw payload blob is never exposed as a
    /// queryable column per BC-2.15.011 dev notes).
    #[test]
    fn test_BC_2_15_011_diff_results_columns_metadata_only() {
        let desc = get_descriptor("prism_diff_results")
            .expect("AC-14 setup: get_descriptor(\"prism_diff_results\") must return Some");

        let col_names: Vec<&str> = desc.columns.iter().map(|(name, _)| name.as_str()).collect();

        // Required columns per spec
        let required = [
            "query_hash",
            "client_id",
            "previous_results_hash",
            "epoch",
            "counter",
            "last_diff_time",
        ];
        for col in &required {
            assert!(
                col_names.contains(col),
                "AC-14 (BC-2.15.011): prism_diff_results columns must include \"{col}\"; \
                 got: {col_names:?}"
            );
        }

        // Raw payload blob must NOT be a queryable column
        assert!(
            !col_names.contains(&"previous_results"),
            "AC-14 (BC-2.15.011 dev note): \"previous_results\" blob must NOT appear as a \
             queryable column in prism_diff_results — use get_diff_results MCP tool instead. \
             Got columns: {col_names:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-005: get_descriptor() with unknown table name returns None
    // BC-2.15.011 EC-005.
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-005 (BC-2.15.011): `get_descriptor("nonexistent_table")` returns `None`.
    /// Callers must handle this gracefully without panic.
    #[test]
    fn test_BC_2_15_011_ec005_get_descriptor_unknown_table_returns_none() {
        let result = get_descriptor("nonexistent_table");

        assert!(
            result.is_none(),
            "EC-005 (BC-2.15.011): get_descriptor for an unknown table name must return None \
             so callers can handle the miss gracefully without panic"
        );
    }
}
