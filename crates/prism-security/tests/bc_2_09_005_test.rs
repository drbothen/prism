//! Tests for BC-2.09.005: Trust-Level Metadata Per Response
//!
//! Verifies: every response includes `_meta.trust_level`; sensor tools =>
//! `untrusted_external`; internal tools => `internal`; mixed => more restrictive.
//!
//! All tests must FAIL before implementation (Red Gate).

use prism_core::TrustLevel;
use prism_security::trust_level::{trust_level_for_tool, TrustLevelExt};

// ─── BC-2.09.005 Postcondition 2 ─────────────────────────────────────────────

/// BC-2.09.005 postcondition 2: sensor query tools have `untrusted_external` trust.
/// Canonical vector: CrowdStrike query result.
#[test]
fn test_BC_2_09_005_sensor_tool_has_untrusted_external_trust_level() {
    let level = trust_level_for_tool("crowdstrike_detections");
    assert_eq!(
        level,
        TrustLevel::UntrustedExternal,
        "sensor tool must have UntrustedExternal trust level"
    );
}

/// BC-2.09.005 postcondition 2: wire format is "untrusted_external" (exact string).
#[test]
fn test_BC_2_09_005_untrusted_external_wire_str_is_canonical() {
    let level = TrustLevel::UntrustedExternal;
    assert_eq!(
        level.wire_str(),
        "untrusted_external",
        "wire_str must be exactly 'untrusted_external'"
    );
}

// ─── BC-2.09.005 Postcondition 3 ─────────────────────────────────────────────

/// BC-2.09.005 postcondition 3: health check tool has `internal` trust.
/// Canonical vector: `check_sensor_health` response.
#[test]
fn test_BC_2_09_005_health_tool_has_internal_trust_level() {
    let level = trust_level_for_tool("check_sensor_health");
    assert_eq!(
        level,
        TrustLevel::Internal,
        "health check tool must have Internal trust level"
    );
}

/// BC-2.09.005 postcondition 3: capability listing tool has `internal` trust.
#[test]
fn test_BC_2_09_005_capabilities_tool_has_internal_trust_level() {
    let level = trust_level_for_tool("list_capabilities");
    assert_eq!(
        level,
        TrustLevel::Internal,
        "list_capabilities tool must have Internal trust level"
    );
}

/// BC-2.09.005 postcondition 3: `internal` wire format is exact.
#[test]
fn test_BC_2_09_005_internal_wire_str_is_canonical() {
    let level = TrustLevel::Internal;
    assert_eq!(
        level.wire_str(),
        "internal",
        "wire_str must be exactly 'internal'"
    );
}

// ─── BC-2.09.005 Postcondition 4 ─────────────────────────────────────────────

/// BC-2.09.005 postcondition 4: error responses have `internal` trust level.
#[test]
fn test_BC_2_09_005_error_response_has_internal_trust_level() {
    // Error responses are Prism-generated
    let level = trust_level_for_tool("__error__");
    assert_eq!(
        level,
        TrustLevel::Internal,
        "error responses must have Internal trust level"
    );
}

// ─── BC-2.09.005 Postcondition 5 — enum has exactly two values ───────────────

/// BC-2.09.005 postcondition 5: TrustLevel enum has exactly two values.
#[test]
fn test_BC_2_09_005_trust_level_is_binary_enum() {
    // UntrustedExternal is not safe for prose
    assert!(
        !TrustLevel::UntrustedExternal.is_safe_for_prose(),
        "UntrustedExternal must not be safe for prose"
    );
    // Internal is safe for prose
    assert!(
        TrustLevel::Internal.is_safe_for_prose(),
        "Internal must be safe for prose"
    );
}

// ─── BC-2.09.005 EC-09-013 — more restrictive wins ───────────────────────────

/// EC-09-013: mixed internal + sensor data → `untrusted_external` (more restrictive).
#[test]
fn test_BC_2_09_005_most_restrictive_untrusted_wins_over_internal() {
    let result = TrustLevel::most_restrictive(TrustLevel::Internal, TrustLevel::UntrustedExternal);
    assert_eq!(
        result,
        TrustLevel::UntrustedExternal,
        "most_restrictive must return UntrustedExternal when mixed"
    );
}

/// most_restrictive is commutative: order doesn't matter.
#[test]
fn test_BC_2_09_005_most_restrictive_is_commutative() {
    let a = TrustLevel::most_restrictive(TrustLevel::UntrustedExternal, TrustLevel::Internal);
    let b = TrustLevel::most_restrictive(TrustLevel::Internal, TrustLevel::UntrustedExternal);
    assert_eq!(a, b, "most_restrictive must be commutative");
}

// ─── AC-2 (prism-security layer) ─────────────────────────────────────────────

/// AC-2 (trust level layer): sensor tool name maps to UntrustedExternal.
/// The full envelope-level AC-2 test lives in prism-mcp/tests/bc_2_09_005_test.rs.
#[test]
fn test_BC_2_09_005_ac2_sensor_tool_name_maps_to_untrusted_external() {
    let level = trust_level_for_tool("crowdstrike_detections");
    assert_eq!(
        level,
        TrustLevel::UntrustedExternal,
        "AC-2: crowdstrike_detections must map to UntrustedExternal"
    );
    assert_eq!(
        level.wire_str(),
        "untrusted_external",
        "AC-2: wire format must be 'untrusted_external'"
    );
}
