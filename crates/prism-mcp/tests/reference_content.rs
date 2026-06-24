//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area C.
//!
//! BC-2.11.022: `build_reference_content()` + CI 3-tier gate (ADR-045).
//!
//! All test bodies call `todo!()` — the implementer writes the assertions.
//! Compilation confirms `build_reference_content`, `ExampleKind`, and `REFERENCE_EXAMPLES`
//! are accessible from the public surface of `prism_mcp::resources`.
//!
//! Red Gate tests: 3.

use prism_mcp::resources::{build_reference_content, ExampleKind, REFERENCE_EXAMPLES};

/// AC-006 / BC-2.11.022 postcondition — `build_reference_content(None)` returns a string
/// that contains all of the following required phrases:
/// - mode names: "Filter", "SQL", "Pipe", "SqlPipe"
/// - operator names: "CONTAINS", "STARTSWITH"
/// - `NOW()` and `INTERVAL` (temporal grammar, ADR-044)
/// - virtual field names: `_sensor`, `_client`
/// - error codes: `E-QUERY-001`, `E-QUERY-038`
#[test]
fn test_bc_2_11_022_reference_content_completeness() {
    todo!(
        "BC-2.11.022 AC-006: call build_reference_content(None) and assert required phrases; \
         implementer implements the function body"
    )
}

/// AC-007 / BC-2.11.022 CI 3-tier gate — assert:
/// (1) `REFERENCE_EXAMPLES` contains at least one `ExampleKind::Basic` entry,
/// (2) `REFERENCE_EXAMPLES` contains at least one `ExampleKind::Advanced` entry,
/// (3) `REFERENCE_EXAMPLES` contains at least one `ExampleKind::Error` entry.
///
/// This is the CI gate that must pass before PR merge (ADR-045 §B).
#[test]
fn test_bc_2_11_022_ci_3tier_gate() {
    // Verify ExampleKind is accessible — compile-time shape check.
    let _ = ExampleKind::Basic;
    let _ = ExampleKind::Advanced;
    let _ = ExampleKind::Error;
    // The actual tier-gate assertion is implemented by the TDD implementer.
    todo!(
        "BC-2.11.022 AC-007: assert REFERENCE_EXAMPLES contains ≥1 entry per ExampleKind tier; \
         implementer ensures ADR-045 §B 3-tier gate passes"
    )
}

/// AC-008 / BC-2.11.022 invariant — `build_reference_content(None)` completes
/// synchronously without panicking and the returned string contains the infusion
/// placeholder text when `infusion_registry` is `None`.
#[test]
fn test_bc_2_11_022_none_registry_placeholder() {
    // Shape-check: the function must be callable — compile-time confirmation.
    let _ = REFERENCE_EXAMPLES;
    todo!(
        "BC-2.11.022 AC-008: assert build_reference_content(None) does not panic and \
         contains infusion placeholder text; implementer implements no-registry path"
    )
}
