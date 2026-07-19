//! SAP-3 wire-level regression tests for DEFECT-T13-AUDIT-ECODE-EXPECTATIONS-001.
//!
//! The T13 pre-flight audit reported two false FAILs ([G4] and [H8]) caused by the
//! audit instrument reading the message-text regex scrape instead of the canonical
//! `structuredContent.error.code`.  These tests lock the wire-level behaviour that
//! the engine is **spec-correct** — they are regression coverage, NOT Red Gate tests.
//!
//! Both tests satisfy SAP-3 (spec-arm reachability) and SID-2 (composed-output
//! assertions) per CLAUDE.md §Standing Adversary Probes & Implementer Disciplines:
//!
//! - SAP-3: each test drives end-to-end through `prism_error_to_structured_call_result`
//!   (the MCP tool boundary), not a stub or internal helper, and asserts on the
//!   SERIALISED JSON output.
//! - SID-2: at least one assertion covers the FULL composed `content[].text` string,
//!   not only its component fields.
//!
//! Wire-shape discipline (CLAUDE.md §Conventions wire-shape assertion discipline):
//! Every assertion targets the SERIALISED JSON bytes — the exact envelope the LLM
//! agent consumes — via `serde_json::to_string` on `structured_content`.
//!
//! # Test → defect mapping
//!
//! | Test | Defect check | BC |
//! |------|--------------|----|
//! | test_sap3_sql_mode_ieq_rejection_wire_shape | [G4] | BC-2.11.017 AC-003 / BC-2.11.024 |
//! | test_sap3_head_join_bare_unknown_col_wire_shape | [H8] | BC-2.11.016 §FP-001 / BC-2.10.007 |

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use prism_core::error::PrismError;
use prism_mcp::error_mapping::prism_error_to_structured_call_result;
use prism_query::PrismQlParser;

// ── Helper: extract content[0].text from a CallToolResult ────────────────────

fn content_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.as_str().to_owned()))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Test A: [G4] SQL-mode IEQ rejection → E-QUERY-001 ────────────────────────

/// SAP-3 wire-level regression: SQL-mode IEQ rejection emits `code == "E-QUERY-001"` in
/// `structuredContent.error`, while `content[].text` carries NO E-code (BC-2.10.007
/// message/suggestion split — the pedagogical code lives in structured content only).
///
/// Defect: T13 audit check [G4] was a false FAIL because `parse_envelope` read the
/// regex-scraped code from message text ("PrismQL parse error: ...") which contains
/// no E-code, yielding "UNKNOWN".  The canonical code lives in
/// `structuredContent.error.code == "E-QUERY-001"` (via `ec_code_override` in
/// `error_mapping.rs`).
///
/// SAP-3: drives from the PUBLIC MCP surface — the real `PrismQlParser::parse` is
/// called first to confirm the query IS rejected, then `prism_error_to_structured_call_result`
/// (the actual MCP tool boundary) is exercised.  The assertion is on the SERIALISED
/// JSON envelope.
///
/// SID-2: the full composed `content[].text` is asserted (not only the `code` field).
///
/// BC-2.11.017 AC-003 / BC-2.11.024 / ADR-047.
#[test]
fn test_sap3_sql_mode_ieq_rejection_wire_shape() {
    // ── Step 1: confirm the parser rejects IEQ in SQL WHERE (SAP-3 reachability) ──
    let query = "SELECT severity, count(*) FROM cyberint_alerts WHERE severity IEQ 'high' GROUP BY severity";
    let parse_result = PrismQlParser::parse(query);
    assert!(
        parse_result.is_err(),
        "SAP-3 [G4]: IEQ in SQL WHERE must be rejected by PrismQlParser::parse; \
         got Ok (parser regression)"
    );
    let parse_errors = parse_result.unwrap_err();
    let first = parse_errors
        .first()
        .expect("SAP-3 [G4]: parse must return at least one error");

    // ── Step 2: build the MCP error via the production path ──────────────────
    // ADR-048 §D.7.2 / materialization.rs de-prefix discipline: production code strips
    // "E-QUERY-001: " from parse error messages before injecting into QueryParseFailed.detail
    // to prevent doubling by the `#[error]` template.  Mimic that here so the test
    // exercises the same wire shape the live audit sees.
    let detail = first
        .message
        .strip_prefix("E-QUERY-001: ")
        .unwrap_or(&first.message)
        .to_string();
    let err = PrismError::QueryParseFailed {
        query: query.to_string(),
        offset: first.offset,
        detail,
    };
    let result = prism_error_to_structured_call_result(err);

    // ── Step 3: wire-level assertion on SERIALISED JSON ──────────────────────
    let sc = result
        .structured_content
        .as_ref()
        .expect("SAP-3 [G4]: structuredContent must be present (BC-2.10.007)");

    // Serialise to JSON — this is the exact envelope the LLM agent receives.
    let serialized =
        serde_json::to_string(sc).expect("SAP-3 [G4]: structured_content must serialise");

    // 3a. structuredContent.error.code MUST be "E-QUERY-001" (ec_code_override pin).
    let code = sc
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|v| v.as_str())
        .expect("SAP-3 [G4]: structuredContent.error.code must be present");
    assert_eq!(
        code, "E-QUERY-001",
        "SAP-3 [G4]: structuredContent.error.code must be 'E-QUERY-001' (BC-2.11.017 AC-003 \
         ec_code_override); got {code:?}. \
         Serialised structuredContent: {serialized}"
    );

    // 3b. The message text must contain mode-boundary pedagogy naming the operator.
    //     The detail from parse_sql carries "not supported in SQL mode".
    assert!(
        serialized.contains("E-QUERY-001"),
        "SAP-3 [G4]: 'E-QUERY-001' must appear in serialised structuredContent (code field); \
         serialised: {serialized}"
    );

    // ── Step 4: content[].text wire assertions (SID-2 composed-output) ───────
    let text = content_text(&result);

    // 4a. content[].text MUST NOT contain "E-QUERY-001" — the E-code belongs in
    //     structuredContent only (BC-2.10.007 message/suggestion split).
    assert!(
        !text.contains("E-QUERY-001"),
        "SAP-3 [G4]: content[].text must NOT contain 'E-QUERY-001' — the E-code lives in \
         structuredContent.error.code, not in the human-readable content text \
         (BC-2.10.007 message/suggestion split). Got text: {text:?}"
    );

    // 4b. content[].text MUST contain the mode-boundary message fragment.
    //     The SQL parser emits "not supported in SQL mode" in the error detail.
    assert!(
        text.contains("not supported in SQL mode"),
        "SAP-3 [G4]: content[].text must contain 'not supported in SQL mode' \
         (pedagogical mode-boundary message per sql_parser.rs); got: {text:?}"
    );

    // 4c. SID-2: the full composed content[].text must contain "IEQ" (operator pedagogy).
    assert!(
        text.to_uppercase().contains("IEQ"),
        "SAP-3 [G4]: content[].text must name the IEQ operator (mode-boundary pedagogy \
         per BC-2.11.024 / ADR-047); got: {text:?}"
    );

    // 4d. SID-2: the full content[].text must start with "ERROR: [" (BC-2.10.007 format).
    assert!(
        text.starts_with("ERROR: ["),
        "SAP-3 [G4]: content[].text must start with 'ERROR: [' (BC-2.10.007 content_text \
         format 'ERROR: [{{category}}] - ...'). Got: {:?}",
        &text[..text.len().min(60)]
    );
}

// ── Test B: [H8] HEAD-JOIN bare unknown col → E-QUERY-034, NOT E-QUERY-038 ───

/// SAP-3 wire-level regression: `QueryExecutionFailed` (the HEAD-JOIN fail-open variant)
/// emits `code == "E-QUERY-034"` in `structuredContent.error`, while `content[].text`
/// carries the redacted "Internal error" form with NO E-code (Rule-1 redaction, BC-2.10.007).
///
/// Defect: T13 audit check [H8] was a false FAIL because `parse_envelope` read the
/// regex-scraped code from message text ("ERROR: [internal] - Internal error. ...") which
/// contains no E-code, yielding "UNKNOWN".  The canonical code lives in
/// `structuredContent.error.code == "E-QUERY-034"` (via `ec_code_override` in the
/// six-variant query-engine arm of `error_mapping.rs`).
///
/// SAP-3: drives through `prism_error_to_structured_call_result` (the MCP tool boundary)
/// and asserts on the SERIALISED JSON envelope.
///
/// SID-2: the full composed `content[].text` is asserted (not only the `code` field).
///
/// BC-2.11.016 §HEAD-JOIN SUSPENSION RULE / BC-2.10.007 §LOW-002.
#[test]
fn test_sap3_head_join_bare_unknown_col_wire_shape() {
    // ── Step 1: construct the production error that HEAD-JOIN fail-open yields ──
    // `QueryExecutionFailed` is the variant that `DataFusion` produces for schema
    // errors at execution time (e.g. unknown column in a cross-sensor JOIN).
    // error_mapping.rs pins it to `ec_code_override = Some("E-QUERY-034")`.
    let err = PrismError::QueryExecutionFailed {
        detail: "DataFusion plan execution: schema error: field 'totally_unknown_col' not found"
            .to_owned(),
    };
    let result = prism_error_to_structured_call_result(err);

    // ── Step 2: wire-level assertion on SERIALISED JSON ──────────────────────
    let sc = result
        .structured_content
        .as_ref()
        .expect("SAP-3 [H8]: structuredContent must be present (BC-2.10.007)");

    // Serialise to JSON — this is the exact envelope the LLM agent receives.
    let serialized =
        serde_json::to_string(sc).expect("SAP-3 [H8]: structured_content must serialise");

    // 2a. structuredContent.error.code MUST be "E-QUERY-034" (ec_code_override pin).
    //     NOT "E-QUERY-038" — E-QUERY-038 is for plan-time ColumnNotFound, not
    //     execution-time QueryExecutionFailed (BC-2.11.016 §HEAD-JOIN SUSPENSION RULE).
    let code = sc
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|v| v.as_str())
        .expect("SAP-3 [H8]: structuredContent.error.code must be present");
    assert_eq!(
        code, "E-QUERY-034",
        "SAP-3 [H8]: structuredContent.error.code must be 'E-QUERY-034' for \
         QueryExecutionFailed (HEAD-JOIN fail-open; BC-2.11.016 §FP-001 / BC-2.10.007 §LOW-002); \
         got {code:?}. Serialised structuredContent: {serialized}"
    );

    // 2b. Must NOT be E-QUERY-038 — belt-and-suspenders: E-QUERY-038 is plan-time
    //     ColumnNotFound; HEAD-JOIN fail-open must not regress to it.
    assert_ne!(
        code, "E-QUERY-038",
        "SAP-3 [H8]: HEAD-JOIN fail-open must NOT produce E-QUERY-038 (plan-time \
         ColumnNotFound); that would violate BC-2.11.016 §HEAD-JOIN SUSPENSION RULE \
         which mandates fail-open to E-QUERY-034 or controlled rejection"
    );

    // 2c. E-QUERY-034 must appear in serialised structuredContent (code field).
    assert!(
        serialized.contains("E-QUERY-034"),
        "SAP-3 [H8]: 'E-QUERY-034' must appear in serialised structuredContent; \
         serialised: {serialized}"
    );

    // ── Step 3: content[].text wire assertions (SID-2 composed-output) ───────
    let text = content_text(&result);

    // 3a. content[].text MUST NOT contain "E-QUERY-034" — Rule-1 redaction (BC-2.10.007):
    //     the E-code belongs in structuredContent.error.code, not in the human-readable
    //     message text (prevents E-code leakage into LLM agent context).
    assert!(
        !text.contains("E-QUERY-034"),
        "SAP-3 [H8]: content[].text must NOT contain 'E-QUERY-034' (Rule-1 redaction, \
         BC-2.10.007 message/suggestion split); got text: {text:?}"
    );

    // 3b. content[].text MUST contain "Internal error" — the Rule-1 terse redaction form.
    assert!(
        text.contains("Internal error"),
        "SAP-3 [H8]: content[].text must contain 'Internal error' (Rule-1 redaction for \
         QueryExecutionFailed; BC-2.10.007 §LOW-002); got: {text:?}"
    );

    // 3c. SID-2: "audit log" must appear exactly ONCE in the full composed content[].text
    //     (no duplication between message and suggestion — BC-2.10.007 [H8b] invariant).
    let audit_log_count = text.to_lowercase().matches("audit log").count();
    assert_eq!(
        audit_log_count, 1,
        "SAP-3 [H8]: 'audit log' must appear exactly once in content[].text \
         (BC-2.10.007 [H8b] no-duplication invariant); found {audit_log_count} times. \
         Got text: {text:?}"
    );

    // 3d. SID-2: the full content[].text must start with "ERROR: [internal]" (BC-2.10.007).
    assert!(
        text.starts_with("ERROR: [internal]"),
        "SAP-3 [H8]: content[].text must start with 'ERROR: [internal]' for \
         QueryExecutionFailed (category='internal', BC-2.10.007 §LOW-002 / BC-2.10.007 \
         content_text format). Got: {:?}",
        &text[..text.len().min(60)]
    );
}
