//! Red Gate tests for S-DEMO-ENRICHMENT-PIVOT-003 — CrowdStrike IOC stamping.
//!
//! Covers:
//!   Test 6: `test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns`
//!
//! NOTE: Test 5 (`test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped`)
//! exercises `make_detection_with_ioc()` which is `pub(crate)` and cannot be called
//! from an external test file. That test is written as an in-crate `#[cfg(test)]` unit
//! test in `crates/prism-dtu-crowdstrike/src/generator.rs`.
//!
//! Story: S-DEMO-ENRICHMENT-PIVOT-003
//! Traces to:
//!   BC-2.06.019 PC-4 — CrowdStrike detections IOC stamp in behaviors[] array
//!   SAP-2 — CrowdStrike detection TOML columns must match generator.rs key set
//!   AC-005 — CrowdStrike detections TOML spec declares behaviors[] IOC columns
//!
//! FAIL mode (Red Gate):
//!   Test 6: `crowdstrike.sensor.toml` does NOT yet contain behaviors[] IOC columns
//!     → assertion fails.
//!
//! Run:
//!   cargo test -p prism-dtu-crowdstrike --features fixture-gen \
//!       --test bc_2_06_019_ioc_stamping -- --nocapture

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

// ---------------------------------------------------------------------------
// Test 6 — CrowdStrike detection TOML spec has behaviors[] IOC columns
// ---------------------------------------------------------------------------

/// Test 6 — BC-2.06.019 PC-4 + SAP-2: the CrowdStrike sensor TOML spec must
/// declare columns for the `behaviors[]` IOC fields with post-ENRICH-1 clean SQL identifiers
/// + source_path values, matching JSON keys stamped by `make_detection_with_ioc()`.
///
/// Expected columns (post-ENRICH-1 names, not bracket-in-name form):
///   name="behaviors_ioc_type",        source_path="$.behaviors[*].ioc_type"
///   name="behaviors_ioc_value",       source_path="$.behaviors[*].ioc_value"
///   name="behaviors_ioc_source",      source_path="$.behaviors[*].ioc_source"
///   name="behaviors_ioc_description", source_path="$.behaviors[*].ioc_description"
///
/// SAP-2 parity note (U19): CrowdStrike detection records are untyped `serde_json::Value`.
/// Column names match JSON keys in `make_detection_with_ioc()` in `src/generator.rs`
/// (wire keys: "ioc_type", "ioc_value", "ioc_source", "ioc_description" in behaviors[0]).
/// This test verifies the TOML spec side of the parity.
///
/// FAIL mode (load-bearing): the test PARSES the TOML and asserts the actual
///   [[tables.columns]] `name` and `source_path` values. If any column reverts to the
///   old bracket-in-name form (e.g., "behaviors[].ioc_value") or lacks source_path, the
///   assertion fails — providing real regression protection rather than comment-text matching.
///
/// This test was rewritten from a vacuous substring-match (HIGH-001 adversary finding):
///   OLD: content.contains("behaviors[].ioc_type")  ← matched COMMENT text in migrated spec
///   NEW: parse TOML, assert actual name + source_path field values
///
/// BC-2.06.019 PC-4, AC-005, SAP-2.
/// Red Gate test plan #6 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_crowdstrike_detection_toml_spec_has_ioc_columns() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let toml_paths = [workspace_root.join("crates/prism-sensors/specs/crowdstrike.sensor.toml")];

    // (expected_name, expected_source_path) — post-ENRICH-1 values.
    // Column names match JSON keys stamped by make_detection_with_ioc() in src/generator.rs.
    // FAIL if any column uses the old bracket-in-name form (e.g., "behaviors[].ioc_value")
    // or lacks source_path. The old bracket form never resolves nested arrays at runtime.
    let required_ioc_columns: &[(&str, &str)] = &[
        ("behaviors_ioc_type", "$.behaviors[*].ioc_type"),
        ("behaviors_ioc_value", "$.behaviors[*].ioc_value"),
        ("behaviors_ioc_source", "$.behaviors[*].ioc_source"),
        (
            "behaviors_ioc_description",
            "$.behaviors[*].ioc_description",
        ),
    ];

    for toml_path in &toml_paths {
        let content = std::fs::read_to_string(toml_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read crowdstrike sensor TOML at {:?}: {e}. \
                 The file must exist after this story ships.",
                toml_path
            )
        });

        // Parse the TOML so assertions check actual field values, not raw text.
        let parsed: toml::Value = content.parse().unwrap_or_else(|e| {
            panic!(
                "Failed to parse crowdstrike sensor TOML at {:?}: {e}",
                toml_path
            )
        });

        // Collect all [[tables.columns]] entries across all tables.
        let tables = parsed
            .get("tables")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| {
                panic!(
                    "crowdstrike sensor TOML at {:?} must have a [[tables]] section",
                    toml_path
                )
            });

        let all_columns: Vec<(String, Option<String>)> = tables
            .iter()
            .flat_map(|table| {
                table
                    .get("columns")
                    .and_then(|c| c.as_array())
                    .map(|cols| {
                        cols.iter()
                            .map(|col| {
                                let name = col
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_owned();
                                let source_path = col
                                    .get("source_path")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_owned());
                                (name, source_path)
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect();

        // Assert each required IOC column is present with the correct name AND source_path.
        for (expected_name, expected_source_path) in required_ioc_columns {
            let found = all_columns.iter().find(|(name, _)| name == expected_name);

            match found {
                None => {
                    let names: Vec<&str> = all_columns.iter().map(|(n, _)| n.as_str()).collect();
                    panic!(
                        "BC-2.06.019 PC-4 / AC-005 / SAP-2 / HIGH-001: \
                         crowdstrike.sensor.toml at {:?} must declare a column with \
                         name='{}' (post-ENRICH-1 clean identifier matching generator.rs \
                         wire key). \
                         If the name is still the old bracket form (e.g., 'behaviors[].ioc_value'), \
                         that is a regression — ENRICH-1 renamed it. \
                         SAP-2: column in TOML with no generator equivalent = P1 CRITICAL; \
                         generator key with no TOML column = MEDIUM (missing coverage). \
                         Actual column names found: {:?}",
                        toml_path,
                        expected_name,
                        names
                    );
                }
                Some((_, actual_source_path)) => {
                    assert_eq!(
                        actual_source_path.as_deref(),
                        Some(*expected_source_path),
                        "BC-2.06.019 PC-4 / SAP-2 / HIGH-001: crowdstrike.sensor.toml at {:?} \
                         column '{}' must have source_path='{}' (post-ENRICH-1 JSONPath). \
                         Actual source_path: {:?}",
                        toml_path,
                        expected_name,
                        expected_source_path,
                        actual_source_path
                    );
                }
            }
        }

        // Regression guard: verify the OLD bracket-in-name forms are ABSENT as actual column names.
        let forbidden_old_names = [
            "behaviors[].ioc_type",
            "behaviors[].ioc_value",
            "behaviors[].ioc_source",
            "behaviors[].ioc_description",
        ];
        for old_name in &forbidden_old_names {
            let reverted = all_columns.iter().any(|(name, _)| name == old_name);
            assert!(
                !reverted,
                "BC-2.06.019 PC-4 / HIGH-001 regression guard: crowdstrike.sensor.toml at {:?} \
                 must NOT contain a column with the old bracket-in-name form '{}'. \
                 This name was retired by ENRICH-1 and replaced with a clean SQL identifier + source_path. \
                 A reversion to the old form means nested array resolution will silently fail at runtime.",
                toml_path,
                old_name
            );
        }
    }
}
