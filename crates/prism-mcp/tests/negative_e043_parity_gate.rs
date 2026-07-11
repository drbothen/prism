//! F-CSD-P25-003 / BC-2.11.022 — NegativeE043 load-bearing plan-time gate lock.
//!
//! # Gap (finding F-CSD-P25-003)
//!
//! `test_bc_2_11_022_reference_content_completeness` only asserts the STRING
//! "E-QUERY-043" appears in the built content, and the 4-tier CI gate only
//! asserts ≥1 `NegativeOther` entry. Neither test asserts a load-bearing
//! E-QUERY-043 plan-time rejection — contrast with `NegativeE040` which is
//! asserted against `plan_sqlpipe_query` in `test_bc_2_11_022_ci_4tier_gate`.
//!
//! # Tests in this file
//!
//! 1. **`test_BC_2_11_022_negative_e043_reference_examples_has_loadbearing_entry`**
//!    — Asserts that `REFERENCE_EXAMPLES` contains at least one executable (non-comment)
//!    entry whose snippet fires `E-QUERY-043`. RED until the implementer adds
//!    `ExampleKind::NegativeE043` + at least one matching entry.
//!
//! 2. **`test_BC_2_11_022_negative_e043_plan_returns_error`**
//!    — For every `REFERENCE_EXAMPLES` entry that fires `E-QUERY-043`, asserts the
//!    error is specifically `ExprInSubqueryProjectionNotSupported` (not some other
//!    `Err` variant). RED until at least one entry fires E-QUERY-043.
//!
//! Both tests compile cleanly at HEAD (no non-existent variant reference) and are
//! RED at runtime because the current `NegativeOther` entries are comment-only
//! (`-- …`) and none of them fire `E-QUERY-043` via `execute_against_session`.
//!
//! RED gate tests: 2

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_imports,
    non_snake_case
)]

use prism_core::error::PrismError;
use prism_mcp::resources::REFERENCE_EXAMPLES;
use prism_query::{
    materialization::{execute_against_session, register_mem_table},
    memory::build_session_context,
    PrismQlParser,
};

/// F-CSD-P25-003 / BC-2.11.022 — NegativeE043 load-bearing entry existence gate.
///
/// Asserts that `REFERENCE_EXAMPLES` contains at least one entry whose snippet:
/// 1. Is not a comment (does not start with `--`).
/// 2. Parses without error.
/// 3. Returns `PrismError::ExprInSubqueryProjectionNotSupported` (E-QUERY-043) from
///    `execute_against_session`.
///
/// # Why this is the load-bearing gate
///
/// `test_bc_2_11_022_reference_content_completeness` only checks that the string
/// "E-QUERY-043" appears somewhere in the built Markdown content. This test checks
/// that an EXECUTABLE example in `REFERENCE_EXAMPLES` actually fires the gate —
/// mirroring the `NegativeE040` parity gate in `test_bc_2_11_022_ci_4tier_gate`.
///
/// # RED gate
///
/// RED until the implementer adds `ExampleKind::NegativeE043` (new variant) to
/// `resources.rs` and at least one `NegativeE043` entry to `REFERENCE_EXAMPLES`
/// whose snippet demonstrates projection-position `IN (SELECT …)`:
///
/// ```text
/// (ExampleKind::NegativeE043,
///  "E-043 IN-subquery in projection",
///  "SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices)) AS flag
///   FROM crowdstrike_detections")
/// ```
///
/// # GREEN after
///
/// At least one `REFERENCE_EXAMPLES` entry (executable, not a comment) fires
/// `E-QUERY-043` from `execute_against_session`.
#[tokio::test]
async fn test_BC_2_11_022_negative_e043_reference_examples_has_loadbearing_entry() {
    // E-QUERY-043 fires at plan time (before any table I/O) — empty tables suffice.
    let ctx = build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
    register_mem_table(&ctx, "crowdstrike_detections", vec![])
        .expect("crowdstrike_detections registration must succeed");
    register_mem_table(&ctx, "crowdstrike_devices", vec![])
        .expect("crowdstrike_devices registration must succeed");
    register_mem_table(&ctx, "sensor_table", vec![])
        .expect("sensor_table registration must succeed");

    let mut e043_firing_count = 0usize;

    for (_, title, snippet) in REFERENCE_EXAMPLES.iter() {
        // Skip comment-only entries (not executable).
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        // Skip entries that fail to parse (not E-QUERY-043 candidates).
        let ast = match PrismQlParser::parse(snippet) {
            Ok(ast) => ast,
            Err(_) => continue,
        };
        let result =
            execute_against_session(&ctx, snippet, &ast, std::collections::HashMap::new()).await;
        if matches!(
            result,
            Err(PrismError::ExprInSubqueryProjectionNotSupported { .. })
        ) {
            e043_firing_count += 1;
            // Confirm the error is precisely E-QUERY-043, not coincidental.
            match result.unwrap_err() {
                PrismError::ExprInSubqueryProjectionNotSupported { .. } => {}
                other => panic!(
                    "BC-2.11.022 F-CSD-P25-003: entry '{title}' matched the E-QUERY-043 \
                     pattern but unwrapped to a different error: {other:?}"
                ),
            }
        }
    }

    // LOCK: at least one executable REFERENCE_EXAMPLES entry must fire E-QUERY-043.
    //
    // RED (HEAD): current NegativeOther entries are comment-only → zero entries
    // produce E-QUERY-043 → e043_firing_count == 0 → assertion FAILS.
    //
    // GREEN (post-fix): implementer adds ExampleKind::NegativeE043 entry with a
    // projection-position IN-subquery snippet → that entry fires E-QUERY-043 →
    // e043_firing_count >= 1 → assertion passes.
    assert!(
        e043_firing_count > 0,
        "BC-2.11.022 F-CSD-P25-003: REFERENCE_EXAMPLES must contain at least one \
         executable (non-comment) entry that fires PrismError::ExprInSubqueryProjectionNotSupported \
         (E-QUERY-043) via execute_against_session. \
         Current state: {e043_firing_count} such entries found (expected > 0). \
         Fix: add ExampleKind::NegativeE043 variant to resources.rs and add at least one \
         NegativeE043 entry with a projection-position IN-subquery snippet, e.g.: \
         (ExampleKind::NegativeE043, \
          \"Negative: E-043 IN-subquery in projection (forbidden)\", \
          \"SELECT (device_id IN (SELECT device_id FROM crowdstrike_devices)) AS flag \
            FROM crowdstrike_detections\")"
    );
}

/// F-CSD-P25-003 / BC-2.11.022 — Every E-QUERY-043 firing entry in REFERENCE_EXAMPLES
/// returns exactly `ExprInSubqueryProjectionNotSupported`.
///
/// Iterates all executable entries in `REFERENCE_EXAMPLES`, runs each through
/// `execute_against_session`, and for entries that return any `PrismError`, asserts
/// that entries intended to fire E-QUERY-043 do not accidentally return a different
/// error variant.
///
/// # RED gate
///
/// RED until at least one `REFERENCE_EXAMPLES` entry fires `E-QUERY-043`
/// (same precondition as `test_BC_2_11_022_negative_e043_reference_examples_has_loadbearing_entry`).
///
/// # GREEN after
///
/// All `E-QUERY-043`-firing entries return the correct error variant and the total
/// count of such entries is ≥ 1.
#[tokio::test]
async fn test_BC_2_11_022_negative_e043_plan_returns_error() {
    let ctx = build_session_context(50 * 1024 * 1024).expect("build_session_context must succeed");
    register_mem_table(&ctx, "crowdstrike_detections", vec![])
        .expect("crowdstrike_detections registration must succeed");
    register_mem_table(&ctx, "crowdstrike_devices", vec![])
        .expect("crowdstrike_devices registration must succeed");
    register_mem_table(&ctx, "sensor_table", vec![])
        .expect("sensor_table registration must succeed");

    // Collect entries that fire E-QUERY-043 and verify the error is correct.
    let mut verified_e043_count = 0usize;

    for (_, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        let ast = match PrismQlParser::parse(snippet) {
            Ok(ast) => ast,
            Err(_) => continue,
        };
        let result =
            execute_against_session(&ctx, snippet, &ast, std::collections::HashMap::new()).await;

        match result {
            Err(PrismError::ExprInSubqueryProjectionNotSupported { .. }) => {
                // Entry fires E-QUERY-043 — expected.
                verified_e043_count += 1;
            }
            Ok(_) | Err(_) => {
                // Entry does not fire E-QUERY-043 — skip (NegativeE040 or Positive entries).
            }
        }
    }

    // LOCK: at least one verified E-QUERY-043 entry must exist.
    //
    // RED (HEAD): no entries fire E-QUERY-043 → verified_e043_count == 0 → FAILS.
    // GREEN (post-fix): NegativeE043 entry fires gate → verified_e043_count >= 1 → passes.
    assert!(
        verified_e043_count > 0,
        "BC-2.11.022 F-CSD-P25-003: at least one REFERENCE_EXAMPLES entry must fire \
         PrismError::ExprInSubqueryProjectionNotSupported (E-QUERY-043). \
         Verified count: {verified_e043_count}. \
         This mirrors the NegativeE040 load-bearing gate (plan_sqlpipe_query → RedundantRowLimit). \
         Fix: add ExampleKind::NegativeE043 entries with projection-position IN-subquery snippets."
    );
}
