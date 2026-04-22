//! Tests for BC-2.09.006: Tool Description Security Warnings — registrar-level tests.
//!
//! Verifies: ToolDescriptionRegistrar appends required security sections to sensor tools;
//! sections survive re-registration (idempotent).
//!
//! All tests must FAIL before implementation (Red Gate).

use prism_mcp::tool_registry::{ToolDescriptionRegistrar, ToolRegistration};
use prism_security::provenance::ToolDescriptionTemplate;

/// BC-2.09.006: registrar appends all required 9 sections to a minimal sensor tool description.
#[test]
fn test_BC_2_09_006_registrar_appends_security_sections_to_minimal_description() {
    let registrar = ToolDescriptionRegistrar;
    let minimal = ToolRegistration {
        name: "crowdstrike_detections".to_owned(),
        description: "Retrieves CrowdStrike detections.".to_owned(),
        is_sensor_tool: true,
        output_schema: None,
    };

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

    let first = registrar.register(ToolRegistration {
        name: "crowdstrike_detections".to_owned(),
        description: desc.clone(),
        is_sensor_tool: true,
        output_schema: None,
    });

    let second = registrar.register(ToolRegistration {
        name: "crowdstrike_detections".to_owned(),
        description: first.description.clone(),
        is_sensor_tool: true,
        output_schema: None,
    });

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
    let health = ToolRegistration {
        name: "check_sensor_health".to_owned(),
        description: "Checks the health of all sensors.".to_owned(),
        is_sensor_tool: false,
        output_schema: None,
    };

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
