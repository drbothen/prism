//! Red Gate tests for S-DEMO-PRISMQL-ONBOARDING-001-A — AC-007 through AC-010.
//!
//! Covers `prismql://reference` static resource (BC-2.10.014), `query_tutorial`
//! MCP Prompt (BC-2.10.009), and the L1 primer upgrade to the `query` tool
//! description (BC-2.10.009 §L1 primer).
//!
//! ALL tests in this file must FAIL against the todo!() stubs (Red Gate per BC-5.38.001).
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_10_014_reference_resource_sections | AC-007 | BC-2.10.014 |
//! | test_BC_2_10_014_reference_resource_static_invariant | AC-008 | BC-2.10.014 |
//! | test_BC_2_10_009_query_tutorial_prompt | AC-009 | BC-2.10.009 |
//! | test_BC_2_10_009_l1_primer_query_tool_description | AC-010 | BC-2.10.009 |

use prism_mcp::{
    prompts::{build_prompt_router, render_query_tutorial, PROMPT_QUERY_TUTORIAL},
    resources::{build_resource_list, schema::render_pql_reference_resource},
    server::PrismServer,
};

// ─── AC-007: prismql://reference registration and required sections ───────────

/// AC-007 (BC-2.10.014 — Resource registration, content required sections):
/// `resources/list` must include `prismql://reference` with `mimeType: "text/markdown"`.
/// `resources/read("prismql://reference")` must return content containing ALL 7 required
/// section headers AND error quick-reference rows for E-QUERY-001, E-QUERY-002,
/// E-QUERY-003, E-QUERY-037, E-QUERY-038.
///
/// Note: the resource LIST registration itself is already implemented (WIRING-EXEMPT
/// per stub comments), so the list-presence assertion passes without implementation.
/// The RED GATE is maintained by `render_pql_reference_resource()` which is `todo!()`.
///
/// RED GATE: Fails with todo!() panic from `render_pql_reference_resource()` in
/// `crates/prism-mcp/src/resources/schema.rs`.
#[test]
fn test_BC_2_10_014_reference_resource_sections() {
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

    // RED GATE: render_pql_reference_resource will todo!() panic here.
    let result = render_pql_reference_resource()
        .expect("BC-2.10.014 AC-007: render_pql_reference_resource must return Ok");

    // Extract content text.
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

    assert!(
        !content_text.is_empty(),
        "BC-2.10.014 AC-007: render_pql_reference_resource must return non-empty content"
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

// ─── AC-008: content authorship invariant ────────────────────────────────────

/// AC-008 (BC-2.10.014 — Content authorship invariant; EC-10-035, EC-10-036):
/// (a) No hardcoded vendor table names in `## Query Examples` section.
/// (b) Content length ≤ 3,000 tokens (~12KB).
/// (c) Content is identical on two successive reads (static invariant).
///
/// RED GATE: Fails with todo!() panic from `render_pql_reference_resource()`.
#[test]
fn test_BC_2_10_014_reference_resource_static_invariant() {
    // RED GATE: both calls will todo!() panic here.
    let result_1 = render_pql_reference_resource()
        .expect("BC-2.10.014 AC-008: first render_pql_reference_resource call must succeed");
    let result_2 = render_pql_reference_resource()
        .expect("BC-2.10.014 AC-008: second render_pql_reference_resource call must succeed");

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
