//! Red Gate tests for S-DEMO-PRISMQL-ONBOARDING-001-A — AC-007 through AC-010.
//!
//! Covers `prismql://reference` static resource (BC-2.10.014), `query_tutorial`
//! MCP Prompt (BC-2.10.009), and the L1 primer upgrade to the `query` tool
//! description (BC-2.10.009 §L1 primer).
//!
//! ALL tests in this file must FAIL against the current (defective) implementation
//! (Red Gate per BC-5.38.001). Tests were rewritten from the initial false-green
//! versions that called leaf renderers directly.
//!
//! # What was wrong with the original tests
//!
//! - AC-007 reference: called `render_pql_reference_resource` directly rather than
//!   `dispatch_read_resource("prismql://reference")` — bypassed the dispatch table,
//!   which does NOT route `prismql://reference`.
//! - AC-007 error codes: the pql_reference.md Error Code Quick-Reference table has
//!   WRONG canonical meanings (E-QUERY-001 and E-QUERY-037 are swapped/wrong).
//!   The previous tests only checked for the presence of the code strings — not the
//!   correct canonical meanings per error-taxonomy.md.
//! - AC-008 static invariant: same dispatch bypass issue.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_10_014_reference_resource_dispatch_routed | AC-007 | BC-2.10.014 |
//! | test_BC_2_10_014_reference_resource_sections | AC-007 | BC-2.10.014 |
//! | test_BC_2_10_014_reference_resource_canonical_error_code_meanings | AC-007 | BC-2.10.014 |
//! | test_BC_2_10_014_reference_resource_static_invariant | AC-008 | BC-2.10.014 |
//! | test_BC_2_10_009_query_tutorial_prompt | AC-009 | BC-2.10.009 |
//! | test_BC_2_10_009_l1_primer_query_tool_description | AC-010 | BC-2.10.009 |

use std::sync::Arc;

use prism_mcp::{
    context::PrismContext,
    prompts::{build_prompt_router, render_query_tutorial, PROMPT_QUERY_TUTORIAL},
    resources::{build_resource_list, dispatch_read_resource},
    server::PrismServer,
};

// ─── AC-007: prismql://reference dispatch routing ────────────────────────────

/// AC-007 (BC-2.10.014 — Dispatch routing):
/// `dispatch_read_resource("prismql://reference", ...)` must NOT return the
/// generic 404 "Unknown or unsupported resource URI". It must route to
/// `render_pql_reference_resource` and return the content.
///
/// RED GATE: Fails because `dispatch_read_resource` in `resources.rs` has no
/// handler for the `prismql://reference` URI — it falls through to the 404 return.
///
/// Note: `build_resource_list()` already lists `prismql://reference` (WIRING-EXEMPT
/// registration is already done). The dispatch routing is the missing piece.
#[tokio::test]
async fn test_BC_2_10_014_reference_resource_dispatch_routed() {
    let context = Arc::new(PrismContext::new());

    // Drive dispatch_read_resource for the prismql://reference URI.
    let result = dispatch_read_resource(
        "prismql://reference",
        &context,
        None, // no query_engine needed
        None, // no config_manager needed (static resource)
    )
    .await;

    // The dispatch must NOT return Err with the 404-equivalent message.
    // Currently falls through to: Err("Unknown or unsupported resource URI")
    assert!(
        result.is_ok(),
        "BC-2.10.014 AC-007: dispatch_read_resource('prismql://reference') must \
         return Ok — not a 404 error. The dispatch table does NOT yet have a handler \
         for 'prismql://reference'. \
         Got Err: {:?}",
        result.err()
    );

    let read_result = result.unwrap();

    // Must return non-empty text content.
    let content_text: String = read_result
        .contents
        .iter()
        .filter_map(|c| {
            if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = c {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    assert!(
        !content_text.is_empty(),
        "BC-2.10.014 AC-007: dispatch result for prismql://reference must be non-empty"
    );
}

// ─── AC-007: prismql://reference registration and required sections ───────────

/// AC-007 (BC-2.10.014 — Resource registration, content required sections):
/// `resources/list` must include `prismql://reference` with `mimeType: "text/markdown"`.
/// `resources/read("prismql://reference")` must return content containing ALL 7 required
/// section headers AND error quick-reference rows for E-QUERY-001, E-QUERY-002,
/// E-QUERY-003, E-QUERY-037, E-QUERY-038.
///
/// Note: The resource list registration is already done (build_resource_list passes).
/// The RED GATE is driven by `dispatch_read_resource` — not by `render_pql_reference_resource`
/// directly. This ensures the implementer wires the dispatch, not just the leaf function.
#[tokio::test]
async fn test_BC_2_10_014_reference_resource_sections() {
    // Verify prismql://reference is registered in the resource list.
    let resource_list = build_resource_list();
    let has_reference = resource_list
        .resources
        .iter()
        .any(|r| r.uri.as_str() == "prismql://reference");
    assert!(
        has_reference,
        "BC-2.10.014 AC-007: 'prismql://reference' must appear in list_resources response"
    );

    // Verify mimeType is text/markdown (or text/plain, both acceptable per story spec).
    let reference_resource = resource_list
        .resources
        .iter()
        .find(|r| r.uri.as_str() == "prismql://reference")
        .expect("BC-2.10.014 AC-007: prismql://reference must be registered");
    let mime = reference_resource.mime_type.as_deref().unwrap_or("");
    assert!(
        mime == "text/markdown" || mime == "text/plain",
        "BC-2.10.014 AC-007: prismql://reference must have mimeType 'text/markdown' or \
         'text/plain'; got: {:?}",
        mime
    );

    // RED GATE: drive via dispatch_read_resource to ensure routing is wired.
    // (NOT by calling render_pql_reference_resource directly — that bypasses dispatch.)
    let context = Arc::new(PrismContext::new());
    let dispatch_result = dispatch_read_resource("prismql://reference", &context, None, None)
        .await
        .expect(
            "BC-2.10.014 AC-007: dispatch_read_resource('prismql://reference') must succeed; \
             currently returns 404 (dispatch not wired for prismql:// URIs)",
        );

    let content_text: String = dispatch_result
        .contents
        .iter()
        .filter_map(|c| {
            if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = c {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    assert!(
        !content_text.is_empty(),
        "BC-2.10.014 AC-007: dispatch result must return non-empty content"
    );

    // BC-2.10.014: all 7 required section headers must be present.
    let required_sections = [
        "## What is PrismQL",
        "## Clause Grammar (BNF)",
        "## Operators and Types",
        "## Datetime Arithmetic",
        "## Error Code Quick-Reference",
        "## Query Examples",
        "## Self-Correction Workflow",
    ];

    for section in &required_sections {
        assert!(
            content_text.contains(section),
            "BC-2.10.014 AC-007: prismql://reference content must contain required section \
             header {:?}; not found. Content snippet: {:?}",
            section,
            &content_text[..content_text.len().min(500)]
        );
    }

    // BC-2.10.014: error quick-reference table must have rows for specific error codes.
    let required_error_codes = [
        "E-QUERY-001",
        "E-QUERY-002",
        "E-QUERY-003",
        "E-QUERY-037",
        "E-QUERY-038",
    ];
    for code in &required_error_codes {
        assert!(
            content_text.contains(code),
            "BC-2.10.014 AC-007: error quick-reference section must contain row for {}; \
             not found in content",
            code
        );
    }
}

// ─── AC-007: canonical error code meanings per error-taxonomy.md ─────────────

/// AC-007 (BC-2.10.014 — Error code canonical meanings):
/// The `## Error Code Quick-Reference` table in `prismql://reference` must map
/// each E-QUERY-NNN code to the CANONICAL meaning per `.factory/specs/prd-supplements/
/// error-taxonomy.md` and BC-2.10.014.
///
/// Canonical meanings (from error-taxonomy.md and prism_core::error):
/// - E-QUERY-001: query PARSE/SYNTAX error (query parse error at offset N: detail)
/// - E-QUERY-002: query PLANNING failed / denylist violation (SELECT-only enforcement)
/// - E-QUERY-037: TABLE NOT AVAILABLE — sensor not configured (table X not available)
/// - E-QUERY-038: COLUMN NOT FOUND / normalized PQL validation failure
///
/// RED GATE: Fails because pql_reference.md has SWAPPED/WRONG meanings:
/// - E-QUERY-001 row says "Unknown table name in FROM clause" (should be parse/syntax)
/// - E-QUERY-037 row says "Query syntax error" (should be table not available/sensor)
/// These are inverted and will cause AI agents to self-correct incorrectly.
///
/// This test drives via dispatch_read_resource (real path) to ensure both dispatch
/// routing AND content correctness are verified in one test.
#[tokio::test]
async fn test_BC_2_10_014_reference_resource_canonical_error_code_meanings() {
    let context = Arc::new(PrismContext::new());

    let result = dispatch_read_resource("prismql://reference", &context, None, None)
        .await
        .expect(
            "BC-2.10.014 AC-007: dispatch_read_resource('prismql://reference') must succeed; \
             dispatch routing must be wired",
        );

    let content_text: String = result
        .contents
        .iter()
        .filter_map(|c| {
            if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = c {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    // BC-2.10.014: E-QUERY-001 must map to parse/syntax errors.
    // Canonical meaning from error-taxonomy.md: "E-QUERY-001: query parse error at offset N"
    // The table row for E-QUERY-001 MUST contain "parse" or "syntax" — NOT "table name" or
    // "FROM clause" (which is E-QUERY-037's territory).
    let eq001_line = extract_table_row_for_code(&content_text, "E-QUERY-001");
    assert!(
        eq001_line.to_lowercase().contains("parse") || eq001_line.to_lowercase().contains("syntax"),
        "BC-2.10.014 AC-007: E-QUERY-001 row in Error Code Quick-Reference MUST describe \
         parse/syntax errors (canonical: 'query parse error'). \
         Current row says: {:?}. \
         The current pql_reference.md has swapped E-QUERY-001 and E-QUERY-037 meanings.",
        eq001_line
    );

    // E-QUERY-001 must NOT claim it is a "table" or "FROM" clause error.
    assert!(
        !eq001_line.to_lowercase().contains("table name")
            && !eq001_line.to_lowercase().contains("from clause"),
        "BC-2.10.014 AC-007: E-QUERY-001 row MUST NOT describe 'table name' or 'FROM clause' \
         errors — that is E-QUERY-037. Current row (wrong): {:?}",
        eq001_line
    );

    // BC-2.10.014: E-QUERY-037 must map to table-not-available / sensor-not-configured.
    // Canonical meaning from error-taxonomy.md: "E-QUERY-037: table X is not available —
    // sensor Y is not configured"
    let eq037_line = extract_table_row_for_code(&content_text, "E-QUERY-037");
    assert!(
        eq037_line.to_lowercase().contains("table")
            || eq037_line.to_lowercase().contains("sensor")
            || eq037_line.to_lowercase().contains("not available")
            || eq037_line.to_lowercase().contains("not configured"),
        "BC-2.10.014 AC-007: E-QUERY-037 row MUST describe table-not-available / \
         sensor-not-configured errors (canonical: 'table X not available, sensor not configured'). \
         Current row says: {:?}. \
         The current pql_reference.md has swapped E-QUERY-037 and E-QUERY-001 meanings.",
        eq037_line
    );

    // E-QUERY-037 must NOT claim it is a "syntax" or "parse" error.
    assert!(
        !eq037_line.to_lowercase().contains("syntax error")
            && !eq037_line.to_lowercase().contains("unexpected token"),
        "BC-2.10.014 AC-007: E-QUERY-037 row MUST NOT describe 'syntax error' or \
         'unexpected token' — those are E-QUERY-001. Current row (wrong): {:?}",
        eq037_line
    );

    // BC-2.10.014: E-QUERY-002 must map to type errors / planning failures / denylist.
    // Canonical meaning: "E-QUERY-002: query planning failed" (denylist violation,
    // e.g., non-SELECT statements rejected)
    let eq002_line = extract_table_row_for_code(&content_text, "E-QUERY-002");
    assert!(
        eq002_line.to_lowercase().contains("type")
            || eq002_line.to_lowercase().contains("plan")
            || eq002_line.to_lowercase().contains("operator")
            || eq002_line.to_lowercase().contains("column")
            || eq002_line.to_lowercase().contains("select"),
        "BC-2.10.014 AC-007: E-QUERY-002 row MUST describe type/planning/operator errors \
         (canonical: 'query planning failed'). Current row says: {:?}",
        eq002_line
    );
}

/// Helper: extract the table row for a given error code from the content text.
/// Returns the line containing the code, or an empty string if not found.
fn extract_table_row_for_code(content: &str, code: &str) -> String {
    content
        .lines()
        .find(|line| line.contains(code))
        .unwrap_or("")
        .to_string()
}

// ─── AC-008: content authorship invariant ────────────────────────────────────

/// AC-008 (BC-2.10.014 — Content authorship invariant; EC-10-035, EC-10-036):
/// (a) No hardcoded vendor table names in `## Query Examples` section.
/// (b) Content length ≤ 3,000 tokens (~12KB).
/// (c) Content is identical on two successive reads (static invariant).
///
/// RED GATE: Driven via dispatch_read_resource to ensure dispatch routing is wired.
/// The previous version called render_pql_reference_resource directly — that bypassed
/// the dispatch table and could pass even when dispatch was broken.
#[tokio::test]
async fn test_BC_2_10_014_reference_resource_static_invariant() {
    let context = Arc::new(PrismContext::new());

    // RED GATE: both calls drive via dispatch to ensure routing is wired.
    let result_1 = dispatch_read_resource("prismql://reference", &context, None, None)
        .await
        .expect(
            "BC-2.10.014 AC-008: first dispatch_read_resource('prismql://reference') must succeed",
        );
    let result_2 = dispatch_read_resource("prismql://reference", &context, None, None)
        .await
        .expect(
            "BC-2.10.014 AC-008: second dispatch_read_resource('prismql://reference') must succeed",
        );

    let extract_text = |r: &rmcp::model::ReadResourceResult| -> String {
        r.contents
            .iter()
            .filter_map(|c| {
                if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = c {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let text_1 = extract_text(&result_1);
    let text_2 = extract_text(&result_2);

    // EC-10-036: content is identical between successive reads (static invariant).
    assert_eq!(
        text_1,
        text_2,
        "BC-2.10.014 AC-008: prismql://reference content must be identical on two successive \
         reads (static invariant — content must NOT be dynamically generated). \
         First read length: {}, second: {}",
        text_1.len(),
        text_2.len()
    );

    // EC-10-035: content length ≤ 3000 tokens (~12,000 bytes at 4 bytes/token).
    // Using 12,000 bytes as a conservative upper bound for ≤3000 tokens.
    const MAX_BYTES: usize = 12_000;
    assert!(
        text_1.len() <= MAX_BYTES,
        "BC-2.10.014 AC-008: prismql://reference content must be ≤3000 tokens (~12KB); \
         got {} bytes ({}KB). Reduce content or use shorter section prose.",
        text_1.len(),
        text_1.len() / 1024
    );

    // EC-10-035: no hardcoded vendor table names in the ## Query Examples section.
    // Extract just the ## Query Examples section for targeted vendor-name check.
    let examples_start = text_1
        .find("## Query Examples")
        .expect("BC-2.10.014 AC-008: ## Query Examples section must be present");
    // Find the next ## section after ## Query Examples (or end of string).
    let examples_end = text_1[examples_start + 1..]
        .find("\n## ")
        .map(|pos| examples_start + 1 + pos)
        .unwrap_or(text_1.len());
    let examples_section = &text_1[examples_start..examples_end];

    let forbidden_vendor_prefixes = ["crowdstrike_", "claroty_", "armis_", "cyberint_"];
    for prefix in &forbidden_vendor_prefixes {
        assert!(
            !examples_section.contains(prefix),
            "BC-2.10.014 AC-008: '## Query Examples' section MUST NOT contain hardcoded \
             vendor table name starting with '{}'. Use '<sensor_table>' or generic \
             placeholders instead. Found in examples section: {:?}",
            prefix,
            &examples_section[..examples_section.len().min(300)]
        );
    }
}

// ─── AC-009: query_tutorial MCP Prompt structural elements ───────────────────

/// AC-009 (BC-2.10.009 — query_tutorial prompt structural elements):
/// - `prompts/list` includes at least 5 prompts, including `query_tutorial`.
/// - Without `goal` argument: message contains Steps 1–4, Step 5 absent.
/// - With `goal: "find critical detections"`: message additionally contains Step 5.
///
/// RED GATE: Fails with todo!() panic from `render_query_tutorial()` in
/// `crates/prism-mcp/src/prompts.rs`.
#[test]
fn test_BC_2_10_009_query_tutorial_prompt() {
    // AC-009: prompts/list includes at least 5 prompts including query_tutorial.
    let prompt_router = build_prompt_router();
    let all_prompts = prompt_router.list_all();
    assert!(
        all_prompts.len() >= 5,
        "BC-2.10.009 AC-009: prompts/list must include at least 5 prompts (triage_alerts, \
         investigate_host, client_overview, cross_client_status, query_tutorial); \
         got only {} prompts: {:?}",
        all_prompts.len(),
        all_prompts
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
    );

    let has_query_tutorial = all_prompts
        .iter()
        .any(|p| p.name.as_str() == PROMPT_QUERY_TUTORIAL);
    assert!(
        has_query_tutorial,
        "BC-2.10.009 AC-009: prompts/list must include 'query_tutorial'; \
         found only: {:?}",
        all_prompts
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
    );

    // RED GATE: render_query_tutorial will todo!() panic here.
    // AC-009 part 1: without goal — Steps 1–4 present, Step 5 absent.
    let no_goal_result = render_query_tutorial("acme", None)
        .expect("BC-2.10.009 AC-009: render_query_tutorial('acme', None) must return Ok");

    assert!(
        !no_goal_result.messages.is_empty(),
        "BC-2.10.009 AC-009: render_query_tutorial must return at least one message; \
         got empty messages list"
    );

    // Extract message text from all messages.
    let no_goal_text: String = no_goal_result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // BC-2.10.009 Step 1: prism_describe call instruction.
    assert!(
        no_goal_text.contains("prism_describe"),
        "BC-2.10.009 AC-009: query_tutorial Step 1 must include 'prism_describe' call \
         instruction. Got text: {:?}",
        &no_goal_text[..no_goal_text.len().min(500)]
    );

    // BC-2.10.009 Step 2: reference to prismql://reference resource.
    assert!(
        no_goal_text.contains("prismql://reference"),
        "BC-2.10.009 AC-009: query_tutorial Step 2 must reference 'prismql://reference' \
         for PQL authoring. Got text: {:?}",
        &no_goal_text[..no_goal_text.len().min(500)]
    );

    // BC-2.10.009 Step 3: E-QUERY error self-correction with named fields.
    // Must mention: near_text, available_columns, did_you_mean, valid_operators_for_type, how_to_fix.
    let step3_fields = [
        "near_text",
        "available_columns",
        "did_you_mean",
        "valid_operators_for_type",
        "how_to_fix",
    ];
    for field in &step3_fields {
        assert!(
            no_goal_text.contains(field),
            "BC-2.10.009 AC-009: query_tutorial Step 3 must include named E-QUERY \
             self-correction field '{}'. Got text: {:?}",
            field,
            &no_goal_text[..no_goal_text.len().min(800)]
        );
    }

    // BC-2.10.009 Step 4: DI-006 security reminder about untrusted sensor data.
    // Must mention security or trust level of sensor data.
    assert!(
        no_goal_text.to_lowercase().contains("untrusted")
            || no_goal_text.to_lowercase().contains("security")
            || no_goal_text.to_lowercase().contains("di-006")
            || no_goal_text.to_lowercase().contains("sensor data"),
        "BC-2.10.009 AC-009: query_tutorial Step 4 must include a DI-006 security reminder \
         about untrusted sensor data. Got text: {:?}",
        &no_goal_text[..no_goal_text.len().min(800)]
    );

    // BC-2.10.009 Step 5: must NOT be present when no goal is given.
    assert!(
        !no_goal_text.contains("find critical detections"),
        "BC-2.10.009 AC-009: query_tutorial without 'goal' must NOT contain Step 5 \
         goal contextualization. Got text: {:?}",
        &no_goal_text[..no_goal_text.len().min(500)]
    );

    // RED GATE: this call also todo!() panics.
    // AC-009 part 2: with goal — Step 5 must now be present.
    let with_goal_result = render_query_tutorial("acme", Some("find critical detections")).expect(
        "BC-2.10.009 AC-009: render_query_tutorial('acme', Some('find critical detections')) \
                 must return Ok",
    );

    let with_goal_text: String = with_goal_result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // BC-2.10.009 Step 5: "Your query goal: find critical detections." must be present.
    assert!(
        with_goal_text.contains("find critical detections"),
        "BC-2.10.009 AC-009: query_tutorial with goal='find critical detections' must \
         include Step 5: 'Your query goal: find critical detections.' \
         Got text: {:?}",
        &with_goal_text[..with_goal_text.len().min(800)]
    );
}

// ─── SEC-001: goal argument length bounding (F-PR197-RG-P2-MED-001) ─────────

/// SEC-001 / F-PR197-RG-P2-MED-001 — `goal` free-text argument must be bounded.
///
/// All free-text prompt arguments must be length-bounded before interpolation to
/// prevent DoS via unbounded memory allocation (F-PR163-IMP-7 / SEC-001 standing rule).
/// Sibling free-text args (time_range, name, description) are all bounded; `goal` was
/// the only one that was not.
///
/// Precondition: `render_query_tutorial("acme", Some(<257-byte string>))` must return
/// `Err(ErrorData::invalid_params(...))`. The error must NOT echo the raw payload
/// (DI-006 — avoids log-injection / AI-prompt-injection vector).
///
/// Valid goal (≤256 bytes) must still succeed (AC-009 regression check).
#[test]
fn test_BC_2_10_009_goal_argument_length_bounded_sec001() {
    // Over-length goal (257 bytes): must be rejected.
    let over_length_goal = "a".repeat(257);
    let result = render_query_tutorial("acme", Some(over_length_goal.as_str()));
    assert!(
        result.is_err(),
        "SEC-001 / F-PR197-RG-P2-MED-001: render_query_tutorial with a 257-byte 'goal' argument \
         must return Err (length-bounded per F-PR163-IMP-7); got Ok. \
         'goal' is a free-text field — all free-text fields must be length-bounded before use."
    );
    let err = result.unwrap_err();
    // DI-006: error must NOT echo the raw payload.
    assert!(
        !err.message.to_string().contains(&over_length_goal),
        "SEC-001 / DI-006: error message must NOT echo the raw 'goal' payload \
         (prompt-injection defense). Got message: {:?}",
        err.message
    );

    // Regression check: valid goal (≤256 bytes) must still succeed (AC-009 invariant).
    let valid_goal = "a".repeat(256);
    let ok_result = render_query_tutorial("acme", Some(valid_goal.as_str()));
    assert!(
        ok_result.is_ok(),
        "SEC-001 regression: render_query_tutorial with a 256-byte 'goal' must still return Ok \
         (boundary is 256 bytes, not 255); got Err: {:?}",
        ok_result.err()
    );
}

// ─── AC-010: query tool description L1 primer ────────────────────────────────

/// AC-010 (BC-2.10.009 §L1 primer — query tool description upgrade):
/// The `query` tool description must contain:
/// - The DSL declaration: "PrismQL (PQL) is a custom DSL"
/// - The clause vocabulary pattern with `SELECT ... FROM`
/// - The pipe-mode hint `|`
/// - All three schema-agnostic skeleton queries using `<table>` placeholder
/// - The discovery pointer phrase "Call `prism_describe`"
///
/// The description MUST NOT contain vendor table names (crowdstrike_, claroty_, armis_, cyberint_)
/// within the skeleton section.
///
/// RED GATE: The current query tool description does NOT contain the L1 primer content.
/// Fails on the first assert checking for "PrismQL (PQL) is a custom DSL" because the
/// `query` tool description in `server.rs` has not been updated with the L1 primer.
#[test]
fn test_BC_2_10_009_l1_primer_query_tool_description() {
    // Get the production tool catalog.
    let catalog = PrismServer::production_tool_catalog();

    // Find the "query" tool.
    let query_tool = catalog.iter().find(|t| t.name.as_ref() == "query").expect(
        "BC-2.10.009 AC-010: 'query' must be in production_tool_catalog; \
             not found. Available tools: see catalog",
    );

    let description = query_tool.description.as_deref().unwrap_or("");

    assert!(
        !description.is_empty(),
        "BC-2.10.009 AC-010: 'query' tool must have a non-empty description"
    );

    // BC-2.10.009 §L1 primer: DSL declaration must be present.
    assert!(
        description.contains("PrismQL (PQL) is a custom DSL"),
        "BC-2.10.009 AC-010: 'query' tool description must contain the DSL declaration \
         'PrismQL (PQL) is a custom DSL'; not found. \
         Current description (first 500 chars): {:?}",
        &description[..description.len().min(500)]
    );

    // BC-2.10.009 §L1 primer: clause vocabulary pattern.
    assert!(
        description.contains("SELECT") && description.contains("FROM"),
        "BC-2.10.009 AC-010: 'query' tool description must contain clause vocabulary \
         pattern 'SELECT ... FROM'; not found."
    );

    // BC-2.10.009 §L1 primer: pipe-mode hint must be present.
    assert!(
        description.contains("|"),
        "BC-2.10.009 AC-010: 'query' tool description must contain pipe-mode hint '|'; \
         not found in description"
    );

    // BC-2.10.009 §L1 primer: schema-agnostic skeletons with <table> placeholder.
    // Story spec requires 3 skeleton queries using <table>, not vendor-specific names.
    assert!(
        description.contains("<table>"),
        "BC-2.10.009 AC-010: 'query' tool description must use '<table>' placeholder \
         in skeleton queries (schema-agnostic, not vendor-specific); \
         '<table>' not found in description"
    );

    // BC-2.10.009 §L1 primer: discovery pointer.
    assert!(
        description.contains("Call `prism_describe`") || description.contains("prism_describe"),
        "BC-2.10.009 AC-010: 'query' tool description must include the discovery pointer \
         phrase 'Call `prism_describe`'; not found in description"
    );

    // Count <table> placeholder occurrences — must have at least 3 skeleton queries.
    let skeleton_count = description.matches("<table>").count();
    assert!(
        skeleton_count >= 3,
        "BC-2.10.009 AC-010: 'query' tool description must have at least 3 skeleton \
         queries using '<table>' placeholder; found only {} occurrences",
        skeleton_count
    );

    // BC-2.10.009 §L1 primer: no vendor table names in skeleton section.
    let forbidden_vendor_prefixes = ["crowdstrike_", "claroty_", "armis_", "cyberint_"];
    for prefix in &forbidden_vendor_prefixes {
        assert!(
            !description.contains(prefix),
            "BC-2.10.009 AC-010: 'query' tool description MUST NOT contain hardcoded \
             vendor table name starting with '{}'; use '<table>' placeholder instead",
            prefix
        );
    }
}
