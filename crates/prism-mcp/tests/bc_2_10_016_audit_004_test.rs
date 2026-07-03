//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-AUDIT-004 — BC-2.10.016 v1.2.
//!
//! Finding AUDIT-004: All five `render_*` prompt functions in `prism-mcp/src/prompts.rs`
//! contain SQL examples that use dot-notation FROM references (`FROM crowdstrike.alerts`,
//! `FROM claroty.assets`, etc.). These must be replaced with underscore-qualified names
//! (`FROM crowdstrike_alerts`, `FROM claroty_assets`, etc.) — matching the valid PrismQL
//! FROM syntax.
//!
//! # Dot-notation instances found in current code (source of this test's assertions)
//!
//! - `render_triage_alerts`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`, `FROM armis.alerts`
//! - `render_investigate_host`: `FROM crowdstrike.devices`, `FROM claroty.assets`, `FROM armis.devices`
//! - `render_client_overview`: `FROM crowdstrike.alerts`, `FROM claroty.alerts`
//! - `render_cross_client_status`: `FROM crowdstrike.alerts`
//! - `render_query_tutorial`: no dot-notation FROM (CLEAN — this test guards it stays clean)
//!
//! # Test strategy
//!
//! Call each `render_*` function with valid inputs, then scan the returned body text for
//! `FROM sensor.table` dot-notation patterns. Uses a manual token scan (no regex dep):
//! split by whitespace, find "FROM" tokens, check the next token for a `.` that is NOT
//! a URL scheme (i.e., not `://`). Assert zero violations.
//!
//! Load-bearing: the prompts.rs `render_*` functions have been updated to use
//! underscore-qualified names. Re-introducing dot-notation causes this test to fail
//! listing all violations found.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_10_016_audit_004_no_dot_notation_in_prompts | AUDIT-004 | BC-2.10.016 v1.2 |

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_query_tutorial, render_triage_alerts,
};

// ── TOML-derived registered-table helper ─────────────────────────────────────

/// Build the registered-table set by parsing `crates/prism-sensors/specs/*.sensor.toml`
/// at test runtime.
///
/// Each TOML spec has the structure:
/// ```toml
/// sensor_id = "crowdstrike"
/// ...
/// [[tables]]
/// table_name = "detections"
/// ```
///
/// This function returns a `HashSet<String>` of `"<sensor_id>_<table_name>"` entries
/// derived directly from the canonical source-of-truth TOML specs, so the test does
/// NOT drift when a table is added or renamed.
///
/// OBS-1 fix: replaces the hardcoded `registered: &[&str]` list that silently drifted
/// on table renames. Runtime-parsed set tracks the TOMLs automatically.
fn registered_tables_from_specs() -> std::collections::HashSet<String> {
    use std::collections::HashSet;

    // CARGO_MANIFEST_DIR is the prism-mcp crate directory.
    // The sensor specs live two levels up at crates/prism-sensors/specs/.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let specs_dir = std::path::Path::new(manifest_dir)
        .join("..")
        .join("prism-sensors")
        .join("specs");

    let mut registered = HashSet::new();

    // Parse each *.sensor.toml file in the specs directory.
    let read_dir = std::fs::read_dir(&specs_dir).unwrap_or_else(|e| {
        panic!(
            "OBS-1: cannot read sensor specs dir {:?}: {e} — \
             check that crates/prism-sensors/specs/ exists relative to CARGO_MANIFEST_DIR",
            specs_dir
        )
    });

    for entry in read_dir {
        let entry = entry.expect("OBS-1: directory entry read failed");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".sensor.toml"))
            .unwrap_or(false)
        {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("OBS-1: cannot read {path:?}: {e}"));

        // Minimal TOML extraction: parse sensor_id and [[tables]].table_name without
        // a full schema type (avoids coupling to the sensor TOML schema crate).
        let doc: toml::Value = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("OBS-1: cannot parse {path:?} as TOML: {e}"));

        let sensor_id = doc
            .get("sensor_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                panic!("OBS-1: {path:?} missing 'sensor_id' key — spec is malformed")
            });

        if let Some(tables) = doc.get("tables").and_then(|v| v.as_array()) {
            for table in tables {
                if let Some(table_name) = table.get("table_name").and_then(|v| v.as_str()) {
                    registered.insert(format!("{sensor_id}_{table_name}"));
                }
            }
        }
    }

    assert!(
        !registered.is_empty(),
        "OBS-1: registered_tables_from_specs() produced an empty set — \
         no *.sensor.toml files found or none have [[tables]] entries. \
         Specs dir: {:?}",
        specs_dir
    );

    registered
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// BC-2.10.016 v1.2 AUDIT-004 — Red Gate test.
///
/// Every `render_*` prompt function must use underscore-qualified FROM targets
/// (e.g., `FROM crowdstrike_alerts`) NOT dot-notation (e.g., `FROM crowdstrike.alerts`).
///
/// Dot-notation FROM references cause AI agents to emit invalid PrismQL queries
/// that produce silent E-SENSOR-030 partial failures (0 rows, no error surface).
///
/// The fix: replace every `FROM sensor.table` literal in `prompts.rs` with
/// `FROM sensor_table`.
///
/// # Red Gate failure
///
/// Current code has 7 dot-notation FROM instances:
/// - render_triage_alerts (3): crowdstrike.alerts, claroty.alerts, armis.alerts
/// - render_investigate_host (3): crowdstrike.devices, claroty.assets, armis.devices
/// - render_client_overview (2): crowdstrike.alerts, claroty.alerts
/// - render_cross_client_status (1): crowdstrike.alerts
///
/// This test asserts zero occurrences and FAILS RED because > 0 are found.
#[test]
fn test_bc_2_10_016_audit_004_no_dot_notation_in_prompts() {
    // ── Collect all prompt outputs ────────────────────────────────────────────

    // render_triage_alerts("acme")
    let triage = render_triage_alerts("acme")
        .expect("render_triage_alerts must not fail for valid client_id 'acme'");
    let triage_text = extract_text(&triage);

    // render_investigate_host("acme", "10.0.0.1")
    let investigate = render_investigate_host("acme", "10.0.0.1")
        .expect("render_investigate_host must not fail for valid inputs");
    let investigate_text = extract_text(&investigate);

    // render_client_overview("acme")
    let overview = render_client_overview("acme")
        .expect("render_client_overview must not fail for valid client_id 'acme'");
    let overview_text = extract_text(&overview);

    // render_cross_client_status(None)
    let cross = render_cross_client_status(None)
        .expect("render_cross_client_status must not fail with no time_range");
    let cross_text = extract_text(&cross);

    // render_query_tutorial("acme", None)
    let tutorial = render_query_tutorial("acme", None)
        .expect("render_query_tutorial must not fail for valid inputs");
    let tutorial_text = extract_text(&tutorial);

    // ── Scan each function's output for dot-notation FROM references ──────────

    let mut all_violations: Vec<String> = Vec::new();

    scan_for_violations(&triage_text, "render_triage_alerts", &mut all_violations);
    scan_for_violations(
        &investigate_text,
        "render_investigate_host",
        &mut all_violations,
    );
    scan_for_violations(
        &overview_text,
        "render_client_overview",
        &mut all_violations,
    );
    scan_for_violations(
        &cross_text,
        "render_cross_client_status",
        &mut all_violations,
    );
    scan_for_violations(&tutorial_text, "render_query_tutorial", &mut all_violations);

    // ── Assert zero violations ────────────────────────────────────────────────

    assert!(
        all_violations.is_empty(),
        "BC-2.10.016 AUDIT-004: found {} dot-notation FROM reference(s) \
         across render_* prompt functions. All must use underscore-qualified \
         names (e.g., FROM crowdstrike_alerts). \
         Violations found:\n{}",
        all_violations.len(),
        all_violations.join("\n")
    );
}

/// BC-2.10.016 AUDIT-004 — Positive guard: EVERY `FROM <sensor>_<table>` target in the
/// rendered prompts must resolve to a registered table derived from the sensor TOML specs.
///
/// AC-AUDIT-004 requires not just the absence of dot-notation, but also that the
/// replacement underscore-qualified names are genuine registered tables. A prompt
/// that switches from `FROM crowdstrike.detections` to `FROM crowdstrike_detections`
/// satisfies the negative guard, but a prompt that switches to `FROM crowdstrike_phantom`
/// (non-existent) would pass the negative guard and violate this positive guard.
///
/// OBS-1 strengthening: the prior guard only checked that ≥1 FROM target resolved to a
/// registered table, against a HARDCODED list. This test:
///   1. Derives the registered-table set from `crates/prism-sensors/specs/*.sensor.toml`
///      at runtime (via `registered_tables_from_specs()`), so it cannot silently drift
///      on a table rename.
///   2. Asserts EVERY underscore-qualified FROM target across all render_* prompt bodies
///      resolves to a table in the spec-derived set — not just ≥1.
///
/// Registered table set is dynamically derived from sensor TOML specs:
///   (crowdstrike: detections, devices, incidents)
///   (claroty:     alerts, audit_logs, devices)
///   (armis:       devices, alerts)
///   (cyberint:    alerts, incidents)
/// Any addition or rename in those TOMLs is automatically picked up.
#[test]
fn test_bc_2_10_016_audit_004_prompt_from_targets_include_registered_table() {
    // Derive the registered-table set from the canonical sensor TOML specs.
    // This replaces the hardcoded &[&str] list (OBS-1 fix).
    let registered = registered_tables_from_specs();

    // Collect the combined text from all render_* prompt functions.
    let triage_text =
        extract_text(&render_triage_alerts("acme").expect("render_triage_alerts must not fail"));
    let investigate_text = extract_text(
        &render_investigate_host("acme", "10.0.0.1")
            .expect("render_investigate_host must not fail"),
    );
    let overview_text = extract_text(
        &render_client_overview("acme").expect("render_client_overview must not fail"),
    );
    let cross_text = extract_text(
        &render_cross_client_status(None).expect("render_cross_client_status must not fail"),
    );
    let tutorial_text = extract_text(
        &render_query_tutorial("acme", None).expect("render_query_tutorial must not fail"),
    );

    let all_text = format!(
        "{}\n{}\n{}\n{}\n{}",
        triage_text, investigate_text, overview_text, cross_text, tutorial_text
    );

    // Collect all `FROM <token>` targets that look like sensor-qualified table names
    // (contain '_' — underscore form is canonical post AUDIT-004 fix).
    let tokens: Vec<&str> = all_text.split_ascii_whitespace().collect();

    // OBS-1: track EVERY underscore FROM target and its resolution status.
    let mut found_registered: Vec<String> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if *token == "FROM" {
            if let Some(next) = tokens.get(i + 1) {
                let clean = next.trim_end_matches([',', ';', ')', '\n']);
                // Only consider underscore-qualified tokens (sensor_table form).
                if clean.contains('_') && !clean.contains("://") {
                    if registered.contains(clean) {
                        found_registered.push(clean.to_string());
                    } else {
                        // An underscore-form FROM target that is NOT in the registered set.
                        unresolved.push(clean.to_string());
                    }
                }
            }
        }
    }

    // Assert EVERY underscore FROM target resolves — no phantom table names.
    assert!(
        unresolved.is_empty(),
        "BC-2.10.016 AUDIT-004 POSITIVE GUARD FAILED: found {} underscore-form FROM target(s) \
         that do NOT resolve to any registered table in the sensor TOML specs.\n\
         Unresolved: {unresolved:?}\n\
         Registered (from sensor TOMLs): {registered:?}\n\
         Fix: update prompts.rs to use only real table names from crates/prism-sensors/specs/.",
        unresolved.len()
    );

    // Also assert ≥1 target resolved (guards against an empty prompt body).
    assert!(
        !found_registered.is_empty(),
        "BC-2.10.016 AUDIT-004 POSITIVE GUARD FAILED: the combined rendered prompt bodies \
         contain zero 'FROM <sensor>_<table>' references that resolve to a real registered \
         table. At least one FROM target must resolve to a registered table (e.g., \
         'FROM crowdstrike_detections'). This guard ensures prompts name real tables, not \
         just syntactically valid identifiers.\nRegistered (from sensor TOMLs): {registered:?}"
    );
}

// ── Column-validation extension (MED-1 process-gap closure) ──────────────────
//
// BC-2.10.016 v1.2 §Postconditions: "any analyst copying an embedded prompt
// example query and executing it MUST get a successful result."
//
// The FROM-target tests above prove the TABLE exists. These tests prove every
// COLUMN referenced in each query example (SELECT, WHERE, GROUP BY) also exists
// in that table's authoritative column set.
//
// Column sets are derived from crates/prism-sensors/specs/*.sensor.toml via
// raw TOML parsing (consistent with the existing table-name helper above).
// No SpecLoader dependency needed: we read sensor_id + [[tables]] inline.

/// Build a map from `{sensor_id}_{table_name}` → set of column names, by
/// parsing the sensor TOML specs directly.
fn build_column_sets_from_specs(
) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
    use std::collections::{HashMap, HashSet};

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let specs_dir = std::path::Path::new(manifest_dir)
        .join("..")
        .join("prism-sensors")
        .join("specs");

    let mut map: HashMap<String, HashSet<String>> = HashMap::new();

    let read_dir = std::fs::read_dir(&specs_dir).unwrap_or_else(|e| {
        panic!(
            "MED-1/column-validator: cannot read sensor specs dir {:?}: {e}",
            specs_dir
        )
    });

    for entry in read_dir {
        let entry = entry.expect("MED-1: directory entry read failed");
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".sensor.toml"))
            .unwrap_or(false)
        {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("MED-1: cannot read {path:?}: {e}"));

        let doc: toml::Value = toml::from_str(&content)
            .unwrap_or_else(|e| panic!("MED-1: cannot parse {path:?} as TOML: {e}"));

        let sensor_id = doc
            .get("sensor_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("MED-1: {path:?} missing sensor_id"));

        if let Some(tables) = doc.get("tables").and_then(|v| v.as_array()) {
            for table in tables {
                let table_name = table
                    .get("table_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| panic!("MED-1: table in {path:?} missing table_name"));

                let key = format!("{sensor_id}_{table_name}");

                let mut cols: HashSet<String> = HashSet::new();
                if let Some(columns) = table.get("columns").and_then(|v| v.as_array()) {
                    for col in columns {
                        if let Some(col_name) = col.get("name").and_then(|v| v.as_str()) {
                            cols.insert(col_name.to_string());
                        }
                    }
                }
                map.insert(key, cols);
            }
        }
    }

    assert!(
        !map.is_empty(),
        "MED-1/column-validator: build_column_sets_from_specs() produced empty map — \
         no *.sensor.toml files found"
    );

    map
}

/// Extract lines from a prompt body that contain a SQL SELECT example.
///
/// Matches lines of the form "- <sensor>: SELECT ... FROM ..." (prompt style).
/// Strips the "- sensor: " label prefix and returns the bare SQL fragment.
fn extract_example_sql_lines(body: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        // Lines like: "  - crowdstrike: SELECT * FROM crowdstrike_detections WHERE ..."
        // Find ": SELECT" or bare "SELECT " starts.
        let sql_start = if let Some(p) = trimmed.find(": SELECT") {
            Some(p + 2) // skip ": "
        } else if trimmed.to_ascii_uppercase().starts_with("SELECT ") {
            Some(0)
        } else {
            None
        };

        if let Some(start) = sql_start {
            let candidate = &trimmed[start..];
            let upper = candidate.to_ascii_uppercase();
            if upper.contains("SELECT ") && upper.contains(" FROM ") {
                result.push(candidate.to_string());
            }
        }
    }
    result
}

/// Extract the FROM table name from a SQL fragment.
///
/// Returns `{sensor_id}_{table_name}` (underscore form).
/// Ignores dot-notation targets (those are already caught by the dot-notation tests).
fn sql_from_table(sql: &str) -> Option<String> {
    let upper = sql.to_ascii_uppercase();
    let from_pos = upper.find(" FROM ")?;
    let after = &sql[from_pos + 6..];
    let end = after
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    let table = &after[..end];
    // Only process underscore-form table names (not dot-notation).
    if table.contains('_') && !table.contains('.') {
        Some(table.to_string())
    } else {
        None
    }
}

/// Extract column references from a SQL fragment.
///
/// Handles simple patterns used in prompt examples:
/// - SELECT col1, col2 (excludes *, COUNT(*), aggregate functions)
/// - WHERE col = 'val', col IN (...), col IS NOT NULL
/// - GROUP BY col
///
/// Returns only bare identifier column names (alphanumeric + underscore, no leading digit).
fn sql_column_refs(sql: &str) -> Vec<String> {
    let mut cols: Vec<String> = Vec::new();
    let upper = sql.to_ascii_uppercase();

    // ─── SELECT list ────────────────────────────────────────────────────────
    let from_pos = upper.find(" FROM ").unwrap_or(sql.len());
    if let Some(sel_pos) = upper.find("SELECT ") {
        let sel_content = &sql[sel_pos + 7..from_pos];
        for item in sel_content.split(',') {
            let item = item.trim();
            if item == "*" {
                continue;
            }
            if item.contains('(') {
                continue;
            } // COUNT(*), aggregate
            if item.parse::<f64>().is_ok() {
                continue;
            }
            if !item.is_empty()
                && item.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && item.chars().next().map_or(false, |c| !c.is_ascii_digit())
            {
                cols.push(item.to_string());
            }
        }
    }

    // ─── WHERE clause ───────────────────────────────────────────────────────
    let upper_after_from = if from_pos < upper.len() {
        &upper[from_pos..]
    } else {
        ""
    };
    if let Some(where_off) = upper_after_from.find(" WHERE ") {
        let where_abs = from_pos + where_off + 7;
        let where_content = &sql[where_abs..];
        let where_upper = where_content.to_ascii_uppercase();
        let where_end = [
            where_upper.find(" GROUP BY "),
            where_upper.find(" ORDER BY "),
            where_upper.find(" LIMIT "),
        ]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(where_content.len());

        let where_region = &where_content[..where_end];
        let predicates = split_predicates(where_region);
        for pred in predicates {
            let pred = pred.trim();
            let pred_upper = pred.to_ascii_uppercase();
            // Skip if starts with a SQL keyword
            if matches!(
                pred_upper.split_ascii_whitespace().next().unwrap_or(""),
                "AND" | "OR" | "NOT" | "IS" | "IN" | "NULL" | "TRUE" | "FALSE"
            ) {
                continue;
            }
            // LHS: identifier before first operator character or space
            let col_end = pred
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(pred.len());
            let col = &pred[..col_end];
            if !col.is_empty()
                && col.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && col.chars().next().map_or(false, |c| !c.is_ascii_digit())
                && !matches!(
                    col.to_ascii_uppercase().as_str(),
                    "AND" | "OR" | "NOT" | "IS" | "IN" | "NULL" | "TRUE" | "FALSE"
                )
            {
                cols.push(col.to_string());
            }
        }
    }

    // ─── GROUP BY clause ────────────────────────────────────────────────────
    if let Some(gb_pos) = upper.find(" GROUP BY ") {
        let gb_content = &sql[gb_pos + 10..];
        let gb_upper = gb_content.to_ascii_uppercase();
        let gb_end = [
            gb_upper.find(" ORDER BY "),
            gb_upper.find(" LIMIT "),
            gb_upper.find(" HAVING "),
        ]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(gb_content.len());

        for item in gb_content[..gb_end].split(',') {
            let item = item.trim();
            if item.contains('(') {
                continue;
            }
            if !item.is_empty()
                && item.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && item.chars().next().map_or(false, |c| !c.is_ascii_digit())
            {
                cols.push(item.to_string());
            }
        }
    }

    cols
}

/// Split a WHERE predicate region into individual conditions (split by AND / OR).
fn split_predicates(where_region: &str) -> Vec<String> {
    let mut result = Vec::new();
    let upper = where_region.to_ascii_uppercase();
    let mut start = 0;
    let mut pos = 0;
    while pos < upper.len() {
        if upper[pos..].starts_with(" AND ") {
            result.push(where_region[start..pos].to_string());
            start = pos + 5;
            pos = start;
        } else if upper[pos..].starts_with(" OR ") {
            result.push(where_region[start..pos].to_string());
            start = pos + 4;
            pos = start;
        } else {
            pos += 1;
        }
    }
    result.push(where_region[start..].to_string());
    result
}

/// MED-1 column-validity guard (BC-2.10.016 v1.2 §Postconditions process-gap closure).
///
/// Every column referenced in a SELECT, WHERE predicate, or GROUP BY clause of each
/// render_* prompt example query must exist in the authoritative column set for the
/// table it queries. Column sets are derived from the sensor TOML specs.
///
/// This is the load-bearing test that closes the [process-gap] from the MED-1 adversary
/// finding: the prior test suite verified FROM targets but not column names.
#[test]
fn test_bc_2_10_016_audit_004_column_refs_resolve_to_real_columns() {
    let column_sets = build_column_sets_from_specs();
    let client_id = "acme";
    let hostname = "10.0.0.1";

    let prompts: Vec<(&str, String)> = vec![
        (
            "render_triage_alerts",
            extract_text(
                &render_triage_alerts(client_id).expect("render_triage_alerts must succeed"),
            ),
        ),
        (
            "render_investigate_host",
            extract_text(
                &render_investigate_host(client_id, hostname)
                    .expect("render_investigate_host must succeed"),
            ),
        ),
        (
            "render_client_overview",
            extract_text(
                &render_client_overview(client_id).expect("render_client_overview must succeed"),
            ),
        ),
        (
            "render_cross_client_status",
            extract_text(
                &render_cross_client_status(None).expect("render_cross_client_status must succeed"),
            ),
        ),
    ];

    let mut failures: Vec<String> = Vec::new();
    // OBS-5: track totals for fail-closed vacuous-pass guards.
    let mut total_columns_checked: usize = 0;
    let mut total_sql_lines_found: usize = 0;

    for (prompt_name, body) in &prompts {
        let sql_lines = extract_example_sql_lines(body);
        total_sql_lines_found += sql_lines.len();

        for sql in &sql_lines {
            let Some(table_key) = sql_from_table(sql) else {
                continue;
            };

            let Some(valid_cols) = column_sets.get(&table_key) else {
                // Unknown table — FROM-target tests catch this separately.
                continue;
            };

            let col_refs = sql_column_refs(sql);
            total_columns_checked += col_refs.len();
            for col in col_refs {
                if !valid_cols.contains(&col) {
                    failures.push(format!(
                        "{prompt_name}: column '{col}' in SQL [{sql}] does NOT exist in \
                         table '{table_key}'. Valid columns: {valid_cols:?}"
                    ));
                }
            }
        }
    }

    // OBS-5: fail-closed guards — ensure the test is not vacuously passing when the
    // prompt format drifts and extract_example_sql_lines() stops finding SQL lines.
    assert!(
        total_sql_lines_found >= 4,
        "OBS-5 vacuous-pass guard: extracted {total_sql_lines_found} SQL example line(s) from \
         render_* prompts, expected >= 4. The prompt format may have drifted — \
         extract_example_sql_lines() no longer finds SQL examples in the prompt bodies."
    );
    assert!(
        total_columns_checked > 0,
        "OBS-5 vacuous-pass guard: checked 0 column references across all SQL example lines. \
         This means no SQL examples contain column references in SELECT/WHERE/GROUP BY. \
         The prompt format may have drifted — the test would pass vacuously without this guard."
    );

    assert!(
        failures.is_empty(),
        "BC-2.10.016 v1.2 MED-1 column-validity: prompt(s) reference invalid columns:\n\n{}\n\n\
         Fix: update render_* functions in prompts.rs to use only columns declared in \
         crates/prism-sensors/specs/*.sensor.toml for the referenced table.",
        failures.join("\n")
    );
}

/// Smoke test: verify the column-set builder loaded columns for all expected tables.
#[test]
fn test_bc_2_10_016_audit_004_column_sets_loaded_for_all_sensor_tables() {
    let column_sets = build_column_sets_from_specs();

    let expected = [
        (
            "claroty_alerts",
            vec![
                "id",
                "alert_type_name",
                "category",
                "status",
                "detected_time",
                "updated_time",
                "devices_count",
                "description",
            ],
        ),
        (
            "claroty_devices",
            vec![
                "uid",
                "asset_id",
                "device_category",
                "device_type",
                "risk_score",
                "retired",
            ],
        ),
        (
            "armis_devices",
            vec![
                "device_id",
                "name",
                "ip_address",
                "mac_address",
                "risk_score",
            ],
        ),
        (
            "armis_alerts",
            vec!["alert_id", "name", "severity", "status"],
        ),
        (
            "crowdstrike_detections",
            vec![
                "detection_id",
                "status",
                "severity",
                "device_id",
                "tactic",
                "technique",
            ],
        ),
        (
            "crowdstrike_devices",
            vec!["device_id", "hostname", "status"],
        ),
    ];

    for (table_key, sample_cols) in &expected {
        let col_set = column_sets
            .get(*table_key)
            .unwrap_or_else(|| panic!("MED-1 smoke: table '{table_key}' missing from column sets"));
        for col in sample_cols {
            assert!(
                col_set.contains(*col),
                "MED-1 smoke: expected column '{col}' missing from '{table_key}' column set. \
                 Got: {col_set:?}"
            );
        }
    }
}

// ── MED-2 value-validation test (process-gap closure) ─────────────────────────
//
// BC-2.10.016 v1.2 §Postconditions: "any analyst copying an embedded prompt
// example query and executing it MUST get a successful result."
//
// The column-validity test proves columns exist. This test proves the WHERE/IN
// literal VALUES for status and severity columns also match what the DTU
// generators actually emit — a value mismatch produces 0 rows with no error.
//
// Canonical per-sensor value sets (verified against DTU generator source):
//
// crowdstrike:
//   status  ∈ {"new", "deleted"}
//     Source: prism-dtu-crowdstrike/src/generator.rs make_detection_with_ioc()
//             json!({..., "status": "new", ...}) (line ~766), tombstone "deleted"
//   severity ∈ {"Low", "Medium", "High", "Critical"}  (Title-case)
//     Source: make_detection_with_ioc() match severity_id (lines ~724-728):
//             1=>"Low", 2=>"Medium", 3=>"High", _=>"Critical"
//
// claroty:
//   status  ∈ {"Unresolved", "tombstone", "online"}
//     Source: prism-dtu-claroty/src/generator.rs make_alert() (line ~179):
//             "status": "Unresolved"
//
// armis:
//   status  ∈ {"UNHANDLED"}
//     Source: prism-dtu-armis/src/generator.rs build_alert() (line ~789):
//             "status": "UNHANDLED"
//   severity ∈ {"HIGH", "CRITICAL", "MEDIUM", "LOW"}  (UPPER-case)
//     Source: build_alert() severity parameter values (lines ~288-297 generate_compromised_endpoint)
//             "HIGH", "CRITICAL", "MEDIUM"
//
// cyberint:
//   severity ∈ {"low", "medium", "high", "critical"}  (lowercase)
//     Source: prism-dtu-cyberint/src/generator.rs
//             let severities = ["low", "medium", "high", "critical"];
//     Note: Added to SENSOR_SEVERITY_VOCABULARY in F-PHL2-MED-001 (Pass-H).
//           The cyberint severity vocabulary is validated by
//           test_f_phl2_med001_cyberint_alerts_severity_uses_lower_case in prism_describe.rs.
//           No render_* prompt emits a cyberint_alerts query, so cyberint_alerts has no
//           row in SENSOR_COLUMN_VOCABULARIES below (F-PIL4-LOW-001 fix).

/// Canonical per-sensor vocabulary for status and severity columns.
///
/// Derived from the DTU generator source files (see comment above for citations).
/// If a DTU generator adds new values, update this const to match.
///
/// Format: `(table_key, column_name, &[valid_values])`.
/// A WHERE/IN literal in a prompt example query must be a member of the
/// corresponding valid_values slice.
const SENSOR_COLUMN_VOCABULARIES: &[(&str, &str, &[&str])] = &[
    // crowdstrike_detections.status: DTU emits "new" ONLY for live detection records.
    // Source: generator.rs make_detection_with_ioc() "status": "new" (line ~766).
    // NOTE: "deleted" is a DEVICE-tombstone status (make_tombstone() "status": "deleted"),
    //       NOT a detection status. Tombstone records are device surface records and are
    //       never returned by the detections route. Removed to prevent over-broad vocabulary
    //       that would allow a prompt using status='deleted' on crowdstrike_detections to
    //       pass the MED-2 guard while returning 0 rows against live DTU data.
    // NOTE: "contained" is also a DEVICE status (gen_compromised_endpoint sets
    //       dev["status"] = json!("contained")), never a detection status.
    // F-PJL2-LOW-001 fix (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade).
    ("crowdstrike_detections", "status", &["new"]),
    // crowdstrike_detections.severity: Title-case from severity_id mapping.
    // Source: generator.rs make_detection_with_ioc() 1=>"Low", 2=>"Medium", 3=>"High", _=>"Critical"
    (
        "crowdstrike_detections",
        "severity",
        &["Low", "Medium", "High", "Critical"],
    ),
    // claroty_alerts.status: DTU emits "Unresolved" for all alert records.
    // Source: generator.rs make_alert() "status": "Unresolved"
    // NOTE: "tombstone" is a DEVICE status (gen_high_churn() sets device records
    //       to "status": "tombstone"), never an alert status. Removed to prevent
    //       over-broad vocabulary on the alerts table.
    ("claroty_alerts", "status", &["Unresolved"]),
    // armis_alerts.status: DTU emits "UNHANDLED" for all alert records.
    // Source: generator.rs build_alert() "status": "UNHANDLED"
    ("armis_alerts", "status", &["UNHANDLED"]),
    // armis_alerts.severity: UPPER-case from generate_compromised_endpoint() severity assignments.
    // Source: generator.rs build_alert() severity param, assigned as "HIGH", "CRITICAL", "MEDIUM", "LOW"
    (
        "armis_alerts",
        "severity",
        &["HIGH", "CRITICAL", "MEDIUM", "LOW"],
    ),
    // cyberint_alerts is intentionally absent.
    // No render_* prompt emits a cyberint_alerts SELECT, so any cyberint entry here
    // would be dead data (the MED-2 loop never matches `FROM cyberint_alerts`).
    // The cyberint severity vocabulary (lowercase) is validated separately by
    // test_f_phl2_med001_cyberint_alerts_severity_uses_lower_case in prism_describe.rs.
    // F-PIL4-LOW-001 fix (S-DEMO-FIDELITY-REMEDIATION-001 Pass-I).
];

/// Extract string literal values from a SQL WHERE predicate or IN list.
///
/// Scans for single-quoted string literals in the SQL fragment.
/// Returns all values enclosed in single quotes.
fn extract_string_literals(sql: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            let mut val = String::new();
            // Collect until closing quote (no escape handling needed for these simple prompts)
            for inner in chars.by_ref() {
                if inner == '\'' {
                    break;
                }
                val.push(inner);
            }
            if !val.is_empty() {
                literals.push(val);
            }
        }
    }
    literals
}

/// MED-2: value-validation test — WHERE/IN literals match DTU-emitted values.
///
/// For each render_* prompt example query, cross-references the WHERE/IN literal
/// values for status and severity columns against the canonical per-sensor vocabulary
/// derived from the DTU generators (see SENSOR_COLUMN_VOCABULARIES above).
///
/// A value NOT in the vocabulary would cause the prompt query to return 0 rows
/// against live DTU data with no error, silently producing an empty demo.
///
/// Vocabulary constants are documented with their generator source citations so
/// a future DTU generator change fails this test (value-regression detection).
///
/// OBS-5 fix applied inline: also asserts total_columns_checked > 0 and
/// sql_lines_found >= EXPECTED_SQL_LINES so the test fails-closed if the prompt
/// format drifts (vacuous-pass prevention).
#[test]
fn test_bc_2_10_016_med2_prompt_filter_values_match_dtu_vocabulary() {
    let client_id = "acme";
    let hostname = "10.0.0.1";

    let prompts: Vec<(&str, String)> = vec![
        (
            "render_triage_alerts",
            extract_text(
                &render_triage_alerts(client_id).expect("render_triage_alerts must succeed"),
            ),
        ),
        (
            "render_investigate_host",
            extract_text(
                &render_investigate_host(client_id, hostname)
                    .expect("render_investigate_host must succeed"),
            ),
        ),
        (
            "render_client_overview",
            extract_text(
                &render_client_overview(client_id).expect("render_client_overview must succeed"),
            ),
        ),
        (
            "render_cross_client_status",
            extract_text(
                &render_cross_client_status(None).expect("render_cross_client_status must succeed"),
            ),
        ),
    ];

    let column_sets = build_column_sets_from_specs();

    let mut failures: Vec<String> = Vec::new();
    let mut total_columns_checked: usize = 0;
    let mut total_sql_lines: usize = 0;

    for (prompt_name, body) in &prompts {
        let sql_lines = extract_example_sql_lines(body);
        total_sql_lines += sql_lines.len();

        for sql in &sql_lines {
            let Some(table_key) = sql_from_table(sql) else {
                continue;
            };

            // Only validate tables we have vocabulary for — skip if not in SENSOR_COLUMN_VOCABULARIES.
            let vocab_entries: Vec<(&str, &[&str])> = SENSOR_COLUMN_VOCABULARIES
                .iter()
                .filter(|(tbl, _, _)| *tbl == table_key.as_str())
                .map(|(_, col, vals)| (*col, *vals))
                .collect();

            if vocab_entries.is_empty() {
                // No vocabulary registered for this table — skip (not a failure).
                continue;
            }

            // Check whether this SQL references any of the vocabulary columns.
            // Only run value-check if the column_sets confirms the column exists in this table.
            let valid_cols_for_table = column_sets.get(&table_key);

            for (vocab_col, valid_values) in &vocab_entries {
                // Skip if this column doesn't appear in the SQL (no WHERE/IN clause for it).
                let col_upper = vocab_col.to_ascii_uppercase();
                if !sql.to_ascii_uppercase().contains(&col_upper) {
                    continue;
                }

                // Confirm the column is real in this table (guard against test drift).
                if let Some(valid_cols) = valid_cols_for_table {
                    if !valid_cols.contains(*vocab_col) {
                        // Column not in spec for this table — skip (column test catches this).
                        continue;
                    }
                }

                total_columns_checked += 1;

                // Extract string literals from the WHERE region NEAR this specific column.
                //
                // Strategy: find the column name in the WHERE region (case-insensitive),
                // then extract only string literals that appear in the predicate fragment
                // starting at that column reference (up to the next AND/OR keyword or end).
                // This prevents cross-predicate false positives (e.g., severity literals
                // being checked against the status vocabulary from the same WHERE clause).
                let upper = sql.to_ascii_uppercase();
                let where_start = upper.find(" WHERE ").map(|p| p + 7).unwrap_or(sql.len());
                let where_region = &sql[where_start.min(sql.len())..];
                let where_upper = where_region.to_ascii_uppercase();

                // Find the column reference in the WHERE region.
                let col_pos = match where_upper.find(&col_upper) {
                    Some(p) => p,
                    None => continue, // column not actually in WHERE clause — skip
                };

                // Extract the predicate fragment from the column position to the next
                // AND / OR keyword (or end of WHERE region). This isolates the predicate
                // `col = 'val'` or `col IN ('v1', 'v2')` belonging to this column.
                let predicate_region = &where_region[col_pos..];
                let predicate_upper = predicate_region.to_ascii_uppercase();
                let pred_end = [predicate_upper.find(" AND "), predicate_upper.find(" OR ")]
                    .into_iter()
                    .flatten()
                    .min()
                    .unwrap_or(predicate_region.len());
                let predicate = &predicate_region[..pred_end];

                let literals = extract_string_literals(predicate);

                for lit in &literals {
                    if !valid_values.contains(&lit.as_str()) {
                        failures.push(format!(
                            "{prompt_name} table={table_key} col={vocab_col}: literal value \
                             '{lit}' is NOT in the DTU-emitted vocabulary {valid_values:?}. \
                             A query with this value returns 0 rows against DTU data. \
                             Fix: update prompts.rs to use an exact emitted value."
                        ));
                    }
                }
            }
        }
    }

    // OBS-5: fail-closed guards — test must not go vacuous if prompt format drifts.
    // We expect at least 4 SQL example lines across the four prompts (one per sensor per prompt
    // that has SQL examples — triage_alerts has 3, client_overview has 2, etc.)
    const EXPECTED_MIN_SQL_LINES: usize = 4;
    assert!(
        total_sql_lines >= EXPECTED_MIN_SQL_LINES,
        "MED-2 OBS-5 vacuous-pass guard: extracted {total_sql_lines} SQL example line(s) from \
         render_* prompts, expected >= {EXPECTED_MIN_SQL_LINES}. If this fails, the prompt \
         format drifted and extract_example_sql_lines() no longer finds SQL lines."
    );

    assert!(
        total_columns_checked > 0,
        "MED-2 OBS-5 vacuous-pass guard: checked 0 column vocabulary entries across all SQL \
         lines. This means no SQL examples reference the status/severity columns we have \
         vocabulary for. Either the prompts changed format or SENSOR_COLUMN_VOCABULARIES is \
         misconfigured. The test must not pass vacuously."
    );

    assert!(
        failures.is_empty(),
        "BC-2.10.016 MED-2 value-validation: prompt(s) use filter values NOT emitted by DTU \
         generators:\n\n{}\n\n\
         Fix: update render_* functions in prompts.rs so WHERE/IN literals match the exact \
         values the DTU generators emit. Canonical vocabulary is in SENSOR_COLUMN_VOCABULARIES \
         (citing generator source lines).",
        failures.join("\n")
    );
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Extract the text content from a `GetPromptResult`.
///
/// Concatenates text from all `PromptMessage` entries in the result.
fn extract_text(result: &rmcp::model::GetPromptResult) -> String {
    result
        .messages
        .iter()
        .filter_map(|msg| match &msg.content {
            rmcp::model::PromptMessageContent::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Scan `text` for `FROM sensor.table` dot-notation patterns without the `regex` crate.
///
/// Algorithm: split the text into whitespace-delimited tokens, find "FROM" tokens (case
/// sensitive — PrismQL uses uppercase FROM in examples), then check the next token.
/// If the next token contains a `.` and the `.` is NOT part of a `://` URL scheme
/// (e.g., `prismql://schema/...` is NOT a FROM target), record as a violation.
///
/// The sensor names in prompts are always lowercase (`crowdstrike`, `claroty`, etc.),
/// so we check that the token looks like `word.word` (no slash after the dot).
fn scan_for_violations(text: &str, fn_name: &str, violations: &mut Vec<String>) {
    // Tokenize by splitting on ASCII whitespace (spaces, newlines, tabs).
    let tokens: Vec<&str> = text.split_ascii_whitespace().collect();

    for (i, token) in tokens.iter().enumerate() {
        // Look for "FROM" keyword (SQL convention: uppercase in prompt examples).
        if *token == "FROM" {
            if let Some(next) = tokens.get(i + 1) {
                // Strip trailing punctuation (comma, semicolon, newline) from the token.
                let clean = next.trim_end_matches([',', ';', ')']);

                // Check for dot-notation: must contain '.' and NOT contain '://'
                // (to exclude URL references like prismql://reference).
                if clean.contains('.') && !clean.contains("://") {
                    violations.push(format!(
                        "  [{fn_name}] 'FROM {clean}' — must be 'FROM {}'",
                        clean.replacen('.', "_", 1)
                    ));
                }
            }
        }
    }
}
