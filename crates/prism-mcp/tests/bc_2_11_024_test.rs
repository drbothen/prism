//! Red Gate test for S-PRISMQL-CASE-INSENSITIVE-001 LOCAL-pass-10 F-CRIT-1.
//!
//! Finding F-CRIT-1: every PrismQL query embedded in a rendered MCP prompt MUST
//! parse via `PrismQlParser::parse`.
//!
//! The pass-9 fix introduced `severity IIN ('High', 'Critical')` into the
//! `render_triage_alerts` armis leg. `IIN` is the PQL case-insensitive IN
//! operator and is only valid in the filter-expression parse mode. When the
//! parser sees a SELECT-mode query it routes to the SQL parser, which does NOT
//! recognize `IIN` and rejects it with E-QUERY-001. String-containment tests
//! (RG-044 / bc_2_02_013_prompt_casing_test.rs) cannot catch this class of
//! defect because they check for stale casing literals, not for parse validity.
//!
//! # How queries are extracted
//!
//! Each embedded query appears on a single line, preceded by a sensor name
//! prefix such as `- crowdstrike: SELECT …`. The extractor finds all lines
//! that contain the substring `"SELECT "` (case-sensitive; uppercase only,
//! as used in all current prompts) and slices from that word to the end of the
//! trimmed line. This is robust for the current prompt format and degrades
//! gracefully (produces zero queries) for prompts like `query_tutorial` that
//! contain no embedded SQL.
//!
//! # Test → BC mapping
//!
//! | Test | BC | Finding |
//! |------|----|---------|
//! | test_BC_2_11_024_all_prompt_embedded_queries_parse | BC-2.11.024 | F-CRIT-1 |

use prism_mcp::prompts::{
    render_client_overview, render_cross_client_status, render_investigate_host,
    render_query_tutorial, render_triage_alerts,
};
use prism_query::PrismQlParser;

// ── Helper: extract all text from a GetPromptResult ────────────────────────

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

// ── Helper: extract embedded PrismQL queries from a rendered prompt body ───
//
// Finds all lines that contain `SELECT ` and returns the substring starting
// from `SELECT` to end-of-trimmed-line.  Handles multi-sensor prompt bodies
// like triage_alerts (three sensor legs) and single-sensor bodies like
// cross_client_status.  Returns an empty Vec for prompts with no embedded SQL
// (e.g. query_tutorial, which references commands and resources but embeds no
// runnable PQL).
fn extract_queries(prompt_name: &str, text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            line.find("SELECT ")
                .map(|pos| (prompt_name.to_string(), line[pos..].trim_end().to_string()))
        })
        .collect()
}

// ── F-CRIT-1 / BC-2.11.024 ─────────────────────────────────────────────────

/// F-CRIT-1 / BC-2.11.024
///
/// Every PrismQL query embedded in a rendered MCP prompt MUST parse via
/// `PrismQlParser::parse` (i.e., `parse` returns `Ok`).
///
/// A prompt that embeds a syntactically invalid query will cause the agent to
/// receive E-QUERY-001 at runtime, producing a silent zero-result triage flow
/// that is indistinguishable from a healthy empty result set.
///
/// # Red Gate failure (HEAD 18c65590)
///
/// `render_triage_alerts` armis leg reads:
///
/// ```text
///   - armis: SELECT * FROM armis_alerts WHERE severity IIN ('High', 'Critical') AND status = 'UNHANDLED'
/// ```
///
/// `IIN` is a PQL filter-mode case-insensitive IN operator.  In SELECT-mode
/// (`PrismQlParser::parse` routes on leading `SELECT`) the SQL sub-parser does
/// NOT know `IIN` and rejects the query with a parse error.  This test FAILS
/// at HEAD because the armis leg produces `Err([ParseError { … }])`.
///
/// # Fix target
///
/// Change `IIN ('High', 'Critical')` → `IN ('High', 'Critical')` in the armis
/// leg of `render_triage_alerts` (post-normalization Title-case per the fn
/// docstring §F-P9-MED-3).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_11_024_all_prompt_embedded_queries_parse() {
    // Render all five prompt functions with representative valid arguments.
    // `render_investigate_host` uses "10.0.0.1" as the hostname so the
    // interpolated queries contain a valid IP literal that the SQL parser accepts.
    let prompt_bodies: Vec<(&str, String)> = vec![
        (
            "render_triage_alerts(\"acme\")",
            extract_text(
                &render_triage_alerts("acme")
                    .expect("render_triage_alerts must succeed for valid client_id 'acme'"),
            ),
        ),
        (
            "render_investigate_host(\"acme\", \"10.0.0.1\")",
            extract_text(
                &render_investigate_host("acme", "10.0.0.1")
                    .expect("render_investigate_host must succeed for valid inputs"),
            ),
        ),
        (
            "render_client_overview(\"acme\")",
            extract_text(
                &render_client_overview("acme")
                    .expect("render_client_overview must succeed for valid client_id 'acme'"),
            ),
        ),
        (
            "render_cross_client_status(None)",
            extract_text(
                &render_cross_client_status(None)
                    .expect("render_cross_client_status must succeed with no time_range"),
            ),
        ),
        (
            "render_query_tutorial(\"acme\", None)",
            extract_text(
                &render_query_tutorial("acme", None)
                    .expect("render_query_tutorial must succeed for valid inputs"),
            ),
        ),
    ];

    // Extract all embedded queries across all prompts.
    let all_queries: Vec<(String, String)> = prompt_bodies
        .iter()
        .flat_map(|(name, text)| extract_queries(name, text))
        .collect();

    // Sanity: at least the triage/investigate/overview/cross_client prompts each
    // embed at least one SELECT query.  If extraction produced zero queries, the
    // extractor is broken and the test is vacuous.
    assert!(
        !all_queries.is_empty(),
        "BC-2.11.024 F-CRIT-1: query extractor produced zero queries across all prompts — \
         the extractor is broken. Check that 'SELECT ' (with trailing space) appears in \
         the rendered prompt bodies."
    );

    // Parse each extracted query and collect failures.
    let mut failures: Vec<String> = Vec::new();
    for (prompt_name, query) in &all_queries {
        if let Err(errs) = PrismQlParser::parse(query) {
            failures.push(format!(
                "  prompt={prompt_name}\n  query={query:?}\n  errors={errs:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "BC-2.11.024 F-CRIT-1: {}/{} embedded PrismQL queries FAILED to parse. \
         Prompt-embedded queries must be valid PQL so agents can execute them without \
         E-QUERY-001. At HEAD the armis leg of render_triage_alerts contains `IIN` \
         which is only valid in PQL filter mode, not SQL SELECT mode. \
         Fix: change `IIN` → `IN` in the armis leg per F-P9-MED-3 adjudication.\n\
         Failing queries ({} total):\n{}",
        failures.len(),
        all_queries.len(),
        failures.len(),
        failures.join("\n"),
    );
}
