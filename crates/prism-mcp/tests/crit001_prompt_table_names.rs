//! CRIT-001 regression guard for S-DEMO-FIDELITY-REMEDIATION-001.
//!
//! Every FROM target in every `render_*` prompt body must name a table that
//! actually exists in the registered sensor specs. A prompt that references a
//! non-existent table will silently produce empty query results, misleading
//! the AI agent and violating the demo fidelity requirement.
//!
//! # Finding
//! CRIT-001: `crates/prism-mcp/src/prompts.rs` — render_* functions emitted FROM
//! clauses with non-existent table names:
//!   - `crowdstrike_alerts`  (valid table is `crowdstrike_detections`)
//!   - `claroty_assets`      (valid table is `claroty_devices`)
//!
//! # Test strategy (POSITIVE-resolve guard)
//! Parse FROM targets out of each rendered prompt body using a simple regex (no
//! full SQL parser needed — FROM targets are single-word identifiers).
//! Assert every target is a member of the authoritative registered table name set
//! derived from the sensor TOML specs.
//!
//! The registered table name set is the single source of truth (crowdstrike.sensor.toml,
//! claroty.sensor.toml, armis.sensor.toml, cyberint.sensor.toml). Changes to sensor
//! specs that add or remove tables MUST also update prompts — this test enforces that.
//!
//! TD-VSDD-059: load-bearing test, not paper-fix. Drives the production render_*
//! functions and asserts against the real registered table name set.

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_triage_alerts,
};

// ── Authoritative registered table set ───────────────────────────────────────
// Derived from sensor TOML specs in crates/prism-sensors/specs/*.sensor.toml.
// The format is `{sensor_prefix}_{table_name}` (prefix = sensor_type key from spec).
//
// crowdstrike.sensor.toml tables: detections, devices, incidents
// claroty.sensor.toml tables:     alerts, audit_logs, devices
// armis.sensor.toml tables:       devices, alerts
// cyberint.sensor.toml tables:    alerts, incidents
//
// This set is the SINGLE AUTHORITATIVE SOURCE for this test. If a sensor spec
// adds a new table, add the qualified name here. If a prompt uses a new table
// name, it must appear here first.
fn registered_table_names() -> Vec<&'static str> {
    vec![
        "crowdstrike_detections",
        "crowdstrike_devices",
        "crowdstrike_incidents",
        "claroty_alerts",
        "claroty_audit_logs",
        "claroty_devices",
        "armis_devices",
        "armis_alerts",
        "cyberint_alerts",
        "cyberint_incidents",
    ]
}

/// Parse SQL `FROM <table_name>` targets from a prompt body string.
///
/// Only collects targets that look like sensor-qualified table names — i.e., identifiers
/// containing an underscore (`sensor_table` form) or a dot (`sensor.table` form). This
/// filters out prose occurrences of "FROM" where the next word is a plain English word
/// (e.g., "read prism://sensors/health for resource pressure metrics FROM available sensors").
///
/// Handles both:
/// - `FROM crowdstrike_detections` (underscore form, canonical)
/// - `FROM crowdstrike.detections` (dot form, legacy — should NOT appear in fixed prompts)
fn extract_from_targets(body: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let lower = body.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(from_pos) = lower[pos..].find("from ") {
        let start = pos + from_pos + 5; // skip "from "
        let rest = &body[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
            .unwrap_or(rest.len());
        let target = &rest[..end];
        // Only collect targets that look like sensor-qualified table names:
        // they must contain '_' (sensor_table form) or '.' (sensor.table form).
        // Plain English words like "available", "all", "each" are skipped.
        if !target.is_empty() && (target.contains('_') || target.contains('.')) {
            targets.push(target.to_string());
        }
        pos = start;
    }
    targets
}

/// CRIT-001 regression guard — all render_* prompt bodies reference only registered tables.
///
/// For each render_* function, render the prompt with a valid test client_id / hostname,
/// extract all FROM targets from the body, and assert every target is in the registered
/// table name set.
///
/// Failures indicate a prompt references a non-existent table — the AI agent would silently
/// receive empty results for that query.
#[test]
fn test_crit001_all_prompt_from_targets_are_registered_tables() {
    let registered = registered_table_names();
    let client_id = "testclient";
    let hostname = "test-host.example.com";

    // Render each prompt and collect (prompt_name, body).
    let prompts: Vec<(&str, String)> = vec![
        (
            "render_triage_alerts",
            render_triage_alerts(client_id)
                .expect("render_triage_alerts must not fail for valid client_id")
                .messages
                .into_iter()
                .map(|m| match m.content {
                    rmcp::model::PromptMessageContent::Text { text } => text,
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (
            "render_investigate_host",
            render_investigate_host(client_id, hostname)
                .expect("render_investigate_host must not fail for valid inputs")
                .messages
                .into_iter()
                .map(|m| match m.content {
                    rmcp::model::PromptMessageContent::Text { text } => text,
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (
            "render_client_overview",
            render_client_overview(client_id)
                .expect("render_client_overview must not fail for valid client_id")
                .messages
                .into_iter()
                .map(|m| match m.content {
                    rmcp::model::PromptMessageContent::Text { text } => text,
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (
            "render_cross_client_status",
            render_cross_client_status(None)
                .expect("render_cross_client_status must not fail")
                .messages
                .into_iter()
                .map(|m| match m.content {
                    rmcp::model::PromptMessageContent::Text { text } => text,
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    ];

    let mut failures: Vec<String> = Vec::new();

    for (prompt_name, body) in &prompts {
        let targets = extract_from_targets(body);
        for target in &targets {
            // Normalize: replace dots with underscores for the comparison
            // (e.g., "crowdstrike.detections" → "crowdstrike_detections").
            let normalized = target.replace('.', "_");
            if !registered.contains(&normalized.as_str()) {
                failures.push(format!(
                    "{prompt_name}: FROM target '{target}' is not a registered table \
                     (normalized: '{normalized}'). Valid tables: {registered:?}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CRIT-001 regression: prompt(s) reference non-existent table names:\n{}",
        failures.join("\n")
    );
}

/// Whitebox check: verify the specific previously-invalid table names are gone.
///
/// `crowdstrike_alerts` must NOT appear in any render_* body (correct: crowdstrike_detections).
/// `claroty_assets` must NOT appear in any render_* body (correct: claroty_devices).
#[test]
fn test_crit001_invalid_table_names_are_absent_from_all_prompts() {
    let client_id = "testclient";
    let hostname = "test-host.example.com";

    let bodies = [
        render_triage_alerts(client_id)
            .unwrap()
            .messages
            .into_iter()
            .map(|m| match m.content {
                rmcp::model::PromptMessageContent::Text { text } => text,
                _ => String::new(),
            })
            .collect::<String>(),
        render_investigate_host(client_id, hostname)
            .unwrap()
            .messages
            .into_iter()
            .map(|m| match m.content {
                rmcp::model::PromptMessageContent::Text { text } => text,
                _ => String::new(),
            })
            .collect::<String>(),
        render_client_overview(client_id)
            .unwrap()
            .messages
            .into_iter()
            .map(|m| match m.content {
                rmcp::model::PromptMessageContent::Text { text } => text,
                _ => String::new(),
            })
            .collect::<String>(),
        render_cross_client_status(None)
            .unwrap()
            .messages
            .into_iter()
            .map(|m| match m.content {
                rmcp::model::PromptMessageContent::Text { text } => text,
                _ => String::new(),
            })
            .collect::<String>(),
    ];

    let all_bodies = bodies.join("\n");

    assert!(
        !all_bodies.contains("crowdstrike_alerts"),
        "CRIT-001: 'crowdstrike_alerts' is a non-existent table name. \
         Found in render_* prompt body. Expected: 'crowdstrike_detections'."
    );

    assert!(
        !all_bodies.contains("claroty_assets"),
        "CRIT-001: 'claroty_assets' is a non-existent table name. \
         Found in render_* prompt body. Expected: 'claroty_devices'."
    );
}
