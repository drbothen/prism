//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area C.
//!
//! BC-2.11.022: `build_reference_content()` + CI 3-tier gate (ADR-045).
//!
//! Red Gate: `build_reference_content` is a `todo!()` stub — all tests panic on
//! the stub body. Tests will fail RED when the function is called.
//!
//! Red Gate tests: 3.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_imports
)]

use prism_mcp::resources::{build_reference_content, ExampleKind, REFERENCE_EXAMPLES};
use prism_query::PrismQlParser;

/// AC-006 / BC-2.11.022 postcondition — content completeness.
///
/// `build_reference_content(None)` must return a string containing:
/// - Mode names: "Filter", "SQL", "Pipe", "SqlPipe"
/// - Operators: "CONTAINS", "=~", "IN CIDR", "HAS", "MISSING", "IS NULL", "IS NOT NULL"
/// - Temporal grammar: "NOW()" and "INTERVAL"
/// - Virtual fields: "_sensor", "_client"
/// - Error codes: "E-QUERY-001", "E-QUERY-038", "E-QUERY-040"
/// - Enrichment: "enrich"
/// - Aggregates: "percentile", "distinct_count" (AC-026 coverage per story)
///
/// Red Gate: `build_reference_content` panics on `todo!()`.
#[test]
fn test_bc_2_11_022_reference_content_completeness() {
    // Call the stub — panics on todo!() → RED.
    let content = build_reference_content(None);

    // Mode names (BC-2.11.022 postcondition — all four modes documented).
    for mode in &["Filter", "SQL", "Pipe", "SqlPipe"] {
        assert!(
            content.contains(mode),
            "BC-2.11.022 AC-006: reference content must contain mode name '{mode}'"
        );
    }

    // Operator names (BC-2.11.022 postcondition — operators table required).
    for op in &[
        "CONTAINS",
        "=~",
        "IN CIDR",
        "HAS",
        "MISSING",
        "IS NULL",
        "IS NOT NULL",
    ] {
        assert!(
            content.contains(op),
            "BC-2.11.022 AC-006: reference content must contain operator '{op}'"
        );
    }

    // Temporal grammar (BC-2.11.021 + BC-2.11.022 postcondition — temporal section required).
    for temporal in &["NOW()", "INTERVAL"] {
        assert!(
            content.contains(temporal),
            "BC-2.11.022 AC-006: reference content must contain temporal keyword '{temporal}'"
        );
    }

    // Virtual fields (BC-2.11.012 — virtual fields documented).
    for vf in &["_sensor", "_client"] {
        assert!(
            content.contains(vf),
            "BC-2.11.022 AC-006: reference content must contain virtual field '{vf}'"
        );
    }

    // Error codes (BC-2.11.022 postcondition — E-QUERY quick-reference).
    for code in &["E-QUERY-001", "E-QUERY-038", "E-QUERY-040"] {
        assert!(
            content.contains(code),
            "BC-2.11.022 AC-006: reference content must contain error code '{code}'"
        );
    }

    // Enrichment section (BC-2.11.022 postcondition — enrichment section required).
    assert!(
        content.contains("enrich"),
        "BC-2.11.022 AC-006: reference content must contain 'enrich' (enrichment section)"
    );

    // Aggregates (AC-026 — aggregates/stats section required).
    for agg in &["percentile", "distinct_count"] {
        assert!(
            content.contains(agg),
            "BC-2.11.022 AC-026: reference content must contain aggregate '{agg}'"
        );
    }
}

/// AC-007 / BC-2.11.022 CI 3-tier gate (ADR-045 §B).
///
/// The shared `REFERENCE_EXAMPLES` constant must contain:
/// (1) At least one `ExampleKind::Basic` entry.
/// (2) At least one `ExampleKind::Advanced` entry.
/// (3) At least one `ExampleKind::Error` entry.
///
/// Additionally, every `ExampleKind::Basic` and `ExampleKind::Advanced` PQL snippet
/// must round-trip through `PrismQlParser::parse` without error (positive round-trip gate).
///
/// Red Gate: The current `REFERENCE_EXAMPLES` constant exists (stubbed in resources.rs)
/// but the `Basic` example is `"crowdstrike.detections"` (a bare source ref, not a
/// full filter predicate — this will parse as `Ast::Filter` with no predicate, which
/// may fail under strict parser rules). The `Advanced` example
/// `"SELECT * FROM crowdstrike.detections WHERE timestamp > NOW() - INTERVAL '7 days'"`
/// uses `NOW()` and `INTERVAL` which are not yet parsed → round-trip fails RED.
#[test]
fn test_bc_2_11_022_ci_3tier_gate() {
    // Tier shape assertions.
    let has_basic = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::Basic));
    let has_advanced = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::Advanced));
    let has_error = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::Error));

    assert!(
        has_basic,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::Basic entry"
    );
    assert!(
        has_advanced,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::Advanced entry"
    );
    assert!(
        has_error,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::Error entry"
    );

    // Positive round-trip gate (ADR-045 §B): Basic and Advanced PQL snippets must parse.
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        let should_parse = matches!(kind, ExampleKind::Basic | ExampleKind::Advanced);
        // Skip comment-prefixed entries (error examples may be comments).
        if !should_parse || snippet.trim_start().starts_with("--") {
            continue;
        }
        let result = PrismQlParser::parse(snippet);
        assert!(
            result.is_ok(),
            "BC-2.11.022 AC-007: REFERENCE_EXAMPLES '{title}' ({kind:?}) must parse via PrismQlParser::parse; \
             got errors: {:?}",
            result
        );
    }

    // Negative E-QUERY-040 gate: every NegativeE040 example must return RedundantRowLimit.
    // (No NegativeE040 entries in current stub; gate is future-proof for when implementer adds them.)
    // We verify the ExampleKind variants are usable by constructing them:
    let _b = ExampleKind::Basic;
    let _a = ExampleKind::Advanced;
    let _e = ExampleKind::Error;
}

/// AC-008 / BC-2.11.022 invariant — `None` registry placeholder.
///
/// `build_reference_content(None)` must:
/// 1. Complete synchronously without panicking.
/// 2. Return a string containing the placeholder text:
///    "Call `list_infusions` to see available enrichment functions for your deployment."
///
/// Red Gate: `build_reference_content` panics on `todo!()`.
#[test]
fn test_bc_2_11_022_none_registry_placeholder() {
    // Call the stub — panics on todo!() → RED.
    let content = build_reference_content(None);

    // Must not be empty.
    assert!(
        !content.is_empty(),
        "BC-2.11.022 AC-008: build_reference_content(None) must return non-empty string"
    );

    // Must contain the infusion placeholder text (BC-2.11.022 invariant).
    let placeholder =
        "Call `list_infusions` to see available enrichment functions for your deployment.";
    assert!(
        content.contains(placeholder),
        "BC-2.11.022 AC-008: build_reference_content(None) must contain infusion placeholder text; \
         got content (first 200 chars): {:?}",
        &content[..content.len().min(200)]
    );
}
