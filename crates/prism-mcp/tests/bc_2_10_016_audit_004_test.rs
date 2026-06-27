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
//! RED GATE: Current code has 7 dot-notation FROM instances across 4 functions.
//! The test counts them and asserts zero. It fails RED listing all occurrences found.
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
        "BC-2.10.016 AUDIT-004 RED GATE: found {} dot-notation FROM reference(s) \
         across render_* prompt functions. All must be replaced with underscore-qualified \
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
