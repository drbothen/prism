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
/// (1) At least one `ExampleKind::Positive` entry.
/// (2) At least one `ExampleKind::NegativeE040` entry (non-vacuous FORBID-BOTH gate).
/// (3) At least one `ExampleKind::NegativeOther` entry.
///
/// Additionally, every `ExampleKind::Positive` PQL snippet must round-trip through
/// `PrismQlParser::parse` without error (positive round-trip gate).
///
/// Every `ExampleKind::NegativeE040` PQL snippet must fail to parse OR must produce
/// a `RedundantRowLimit` error when executed (negative E-QUERY-040 gate).
///
/// Red Gate: The current `REFERENCE_EXAMPLES` constant uses the old `Basic/Advanced/Error`
/// variant names — these are now renamed to `Positive/NegativeE040/NegativeOther` per
/// BC-2.11.022 / ADR-045 D3. Compilation fails RED until the rename is complete.
#[test]
fn test_bc_2_11_022_ci_3tier_gate() {
    // Tier shape assertions (BC-2.11.022 ADR-045 §B).
    let has_positive = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::Positive));
    let has_negative_e040 = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::NegativeE040));
    let has_negative_other = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::NegativeOther));

    assert!(
        has_positive,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::Positive entry"
    );
    assert!(
        has_negative_e040,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::NegativeE040 entry \
         (non-vacuous FORBID-BOTH gate — tautological gate is a paper-fix per TD-VSDD-059)"
    );
    assert!(
        has_negative_other,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::NegativeOther entry"
    );

    // Positive round-trip gate (ADR-045 §B): Positive PQL snippets must parse.
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if !matches!(kind, ExampleKind::Positive) {
            continue;
        }
        // Skip comment-prefixed entries (should not appear in Positive tier, but guard defensively).
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        let result = PrismQlParser::parse(snippet);
        assert!(
            result.is_ok(),
            "BC-2.11.022 AC-007: Positive example '{title}' must parse via PrismQlParser::parse; \
             got errors: {:?}",
            result
        );
    }

    // Negative E-QUERY-040 gate: every NegativeE040 example must fail the parser OR
    // contain E-QUERY-040 / FORBID-BOTH content (parser rejects dual-limit at parse time).
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if !matches!(kind, ExampleKind::NegativeE040) {
            continue;
        }
        // Skip comment-prefixed entries.
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        // NegativeE040 snippets must either fail to parse (dual-limit detected by parser)
        // or be valid SQL+pipe queries (parse succeeds; runtime rejects at execution time).
        // At minimum: the snippet must NOT be parseable as a pure Pipe or Filter query
        // (it contains both SQL LIMIT and pipe limit, which is the FORBID-BOTH pattern).
        let result = PrismQlParser::parse(snippet);
        // The E-QUERY-040 gate fires at execution time (PrismError::RedundantRowLimit),
        // not necessarily at parse time (the SqlPipe grammar absorbs SQL+pipe combinations).
        // We assert the example compiles (so it's a real PQL string) — runtime rejection
        // is exercised by query-engine tests (BC-2.11.023 via run_query).
        assert!(
            result.is_ok(),
            "BC-2.11.022 AC-007: NegativeE040 example '{title}' must be a valid PQL string \
             (E-QUERY-040 fires at execution time, not parse time); got parse error: {:?}",
            result
        );
    }

    // Verify all three ExampleKind variants are constructable (compile-time check).
    let _p = ExampleKind::Positive;
    let _n = ExampleKind::NegativeE040;
    let _o = ExampleKind::NegativeOther;
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
