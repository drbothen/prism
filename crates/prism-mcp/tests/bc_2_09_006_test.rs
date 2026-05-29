//! Tests for BC-2.09.006: Tool Description Security Warnings — registrar-level tests.
//!
//! Verifies: ToolDescriptionRegistrar appends required security sections to sensor tools;
//! sections survive re-registration (idempotent).
//!
//! IMP-5: also includes live ToolRouter test that verifies the production tool catalog
//! via the macro-generated ToolRouter::list_all() API (not a source-file scan).
//!
//! All tests pass (implementation complete).

use prism_mcp::tool_registry::{ToolDescriptionRegistrar, ToolRegistration};
use prism_security::provenance::ToolDescriptionTemplate;

/// BC-2.09.006: registrar appends all required 9 sections to a minimal sensor tool description.
#[test]
fn test_BC_2_09_006_registrar_appends_security_sections_to_minimal_description() {
    let registrar = ToolDescriptionRegistrar;
    let minimal = ToolRegistration::new(
        "crowdstrike_detections",
        "Retrieves CrowdStrike detections.",
        true,
        None,
    );

    let registered = registrar.register(minimal);

    assert!(
        ToolDescriptionTemplate::is_valid_sensor_tool_description(&registered.description),
        "registrar must append all required sections; missing: {:?}",
        ToolDescriptionTemplate::missing_sections(&registered.description)
    );
}

/// BC-2.09.006: security sections are idempotent — no duplication on re-registration.
#[test]
fn test_BC_2_09_006_registrar_security_sections_are_idempotent() {
    let registrar = ToolDescriptionRegistrar;
    let desc = "Retrieves CrowdStrike detections.".to_owned();

    let first = registrar.register(ToolRegistration::new(
        "crowdstrike_detections",
        desc.as_str(),
        true,
        None,
    ));

    let second = registrar.register(ToolRegistration::new(
        "crowdstrike_detections",
        first.description.as_str(),
        true,
        None,
    ));

    let security_note_count = second.description.matches("SECURITY NOTE:").count();
    assert_eq!(
        security_note_count, 1,
        "SECURITY NOTE: must appear exactly once after idempotent re-registration; got {}",
        security_note_count
    );
}

/// BC-2.09.006: non-sensor tool registration does NOT add security sections.
#[test]
fn test_BC_2_09_006_non_sensor_tool_not_given_security_sections() {
    let registrar = ToolDescriptionRegistrar;
    let health = ToolRegistration::new(
        "check_sensor_health",
        "Checks the health of all sensors.",
        false,
        None,
    );

    let registered = registrar.register(health);

    assert!(
        !registered.description.contains("DATA TRUST LEVEL:"),
        "non-sensor tool must not have DATA TRUST LEVEL section"
    );
    assert!(
        !registered.description.contains("SECURITY NOTE:"),
        "non-sensor tool must not have SECURITY NOTE section"
    );
}

/// IMP-5: live ToolRouter test — verifies the production tool catalog via
/// `PrismServer::production_tool_catalog()` (runtime catalog, not source-file scan).
///
/// For every tool whose description contains "DATA SOURCE:" (identified as a sensor tool),
/// all 9 required sections must be present (BC-2.09.006 postcondition 1).
///
/// LOAD-BEARING: if a sensor tool is added to the router without all 9 sections,
/// this test fails. Source-level scan (in tool_dispatch_tests) could miss tools
/// registered programmatically; this test catches those too.
#[test]
fn test_BC_2_09_006_live_tool_catalog_sensor_tools_have_9_sections() {
    use prism_mcp::server::PrismServer;

    let tools = PrismServer::production_tool_catalog();

    assert!(
        !tools.is_empty(),
        "IMP-5: tool_router().list_all() must return at least one tool; got empty list. \
         Check that #[tool_router] macro is applied correctly to PrismServer."
    );

    let sensor_tools: Vec<_> = tools
        .iter()
        .filter(|t| {
            t.description
                .as_deref()
                .unwrap_or("")
                .contains("DATA SOURCE:")
        })
        .collect();

    assert!(
        !sensor_tools.is_empty(),
        "IMP-5: expected at least one sensor tool in the catalog (with DATA SOURCE: section); \
         found none. Check #[tool_router] macro and tool description attributes."
    );

    let mut failures: Vec<String> = Vec::new();
    for tool in &sensor_tools {
        let desc = tool.description.as_deref().unwrap_or("");
        let missing = ToolDescriptionTemplate::missing_sections(desc);
        if !missing.is_empty() {
            failures.push(format!(
                "  tool '{}': missing sections: {:?}",
                tool.name, missing
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "BC-2.09.006 VIOLATION (live catalog): {} sensor tool(s) missing required sections:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
