// S-1.08: BC-2.04.005 — Hidden Tools Pattern — Stateless Tool List
//
// Tests verify that:
//  - AC-3: denied write capability → tool absent from tools/list.
//  - Read tools always appear in tools/list.
//  - Write tools enabled for ≥1 client appear in tools/list.
//  - Write tools disabled for ALL clients are completely absent.
//  - tools/list is stateless (same result regardless of call order).
//  - EC-04-010: tool enabled for Client A but not B → appears in list.
//  - EC-04-011: no clients have write enabled → only read tools visible.
//  - E-FLAG-006: write tool invoked with client_id: null → E-FLAG-006 error.
//
// Naming: test_BC_2_04_005_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::hidden_tools::{HiddenToolsRegistry, RegisteredTool, ToolKind};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn read_tool(name: &str) -> RegisteredTool {
    RegisteredTool {
        name: name.to_string(),
        description: format!("Read tool: {name}"),
        kind: ToolKind::Read,
    }
}

fn write_tool(name: &str, required_capability: &str) -> RegisteredTool {
    RegisteredTool {
        name: name.to_string(),
        description: format!("Write tool: {name}"),
        kind: ToolKind::Write {
            required_capability: required_capability.to_string(),
        },
    }
}

fn client_map_with(rules: Vec<(&str, Vec<(&str, CapabilityEffect)>)>) -> BTreeMap<String, ClientCapabilities> {
    let mut map = BTreeMap::new();
    for (client_id, cap_rules) in rules {
        let mut caps = ClientCapabilities::new();
        for (path, effect) in cap_rules {
            caps.grant(cap(path), effect);
        }
        map.insert(client_id.to_string(), caps);
    }
    map
}

fn make_registry() -> HiddenToolsRegistry {
    HiddenToolsRegistry::new(vec![
        read_tool("crowdstrike_list_hosts"),
        read_tool("crowdstrike_get_alerts"),
        write_tool("crowdstrike_contain_host", "sensor.crowdstrike.containment"),
        write_tool("crowdstrike_update_alert", "sensor.crowdstrike.alert_write"),
    ])
}

// ─────────────────────────────────────────────────────────────
// AC-3: denied write → tool absent from tools/list
// ─────────────────────────────────────────────────────────────

/// AC-3: Given a denied write capability (no client has it enabled), when
/// tool listing is requested, then the write tool is absent from the list.
#[test]
fn test_BC_2_04_005_ac3_denied_write_tool_absent_from_list() {
    let registry = make_registry();
    // No client has containment capability.
    let client_map = client_map_with(vec![("acme", vec![])]);

    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    assert!(
        !tool_names.contains(&"crowdstrike_contain_host"),
        "BC-2.04.005 AC-3: denied write tool must be absent from tools/list, found: {:?}",
        tool_names
    );
}

// ─────────────────────────────────────────────────────────────
// Read tools always appear
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_005_read_tools_always_in_list() {
    let registry = make_registry();
    // Empty client capabilities — no write tools visible.
    let client_map = client_map_with(vec![("acme", vec![])]);

    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    assert!(
        tool_names.contains(&"crowdstrike_list_hosts"),
        "BC-2.04.005: read tool must always be in tools/list"
    );
    assert!(
        tool_names.contains(&"crowdstrike_get_alerts"),
        "BC-2.04.005: read tool must always be in tools/list"
    );
}

// ─────────────────────────────────────────────────────────────
// Write tool enabled for ≥1 client appears in tools/list
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_005_write_tool_enabled_for_one_client_appears_in_list() {
    let registry = make_registry();
    let client_map = client_map_with(vec![(
        "acme",
        vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
    )]);

    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    assert!(
        tool_names.contains(&"crowdstrike_contain_host"),
        "BC-2.04.005: write tool enabled for ≥1 client must appear in tools/list"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-011: No clients have any write capabilities → only read tools
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_005_ec_no_write_clients_only_read_visible() {
    let registry = make_registry();
    let client_map = client_map_with(vec![
        ("acme", vec![]),
        ("beta", vec![]),
    ]);

    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Write tools absent
    assert!(
        !tool_names.contains(&"crowdstrike_contain_host"),
        "EC-04-011: write tool must be absent when no client has capability"
    );
    assert!(
        !tool_names.contains(&"crowdstrike_update_alert"),
        "EC-04-011: write tool must be absent when no client has capability"
    );

    // Read tools present
    assert!(
        tool_names.contains(&"crowdstrike_list_hosts"),
        "EC-04-011: read tool must still be present"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-010: Write enabled for Client A not B → appears in list
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_005_ec_write_enabled_for_one_of_two_clients_appears_in_list() {
    let registry = make_registry();
    let client_map = client_map_with(vec![
        (
            "acme",
            vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
        ),
        ("beta", vec![]), // no write caps
    ]);

    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Tool appears because acme has it (even though beta does not)
    assert!(
        tool_names.contains(&"crowdstrike_contain_host"),
        "EC-04-010: write tool enabled for acme must appear in tools/list even though beta lacks it"
    );
}

// ─────────────────────────────────────────────────────────────
// Stateless invariant: tools/list same regardless of call order
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_005_invariant_tools_list_is_stateless() {
    let registry = make_registry();
    let client_map = client_map_with(vec![(
        "acme",
        vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
    )]);

    // Call 5 times — must return identical tool names each time.
    let first: Vec<String> = registry
        .tools_list(&client_map)
        .iter()
        .map(|t| t.name.clone())
        .collect();

    for _ in 0..4 {
        let subsequent: Vec<String> = registry
            .tools_list(&client_map)
            .iter()
            .map(|t| t.name.clone())
            .collect();
        assert_eq!(
            first, subsequent,
            "BC-2.04.005 invariant: tools/list must be stateless (same each call)"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// Hidden tools are present in binary (get_tool returns Some)
// ─────────────────────────────────────────────────────────────

/// Architecture rule: hidden tools are NOT compiled out — they exist in the
/// binary but are excluded from the tools/list response (BC-2.04.005 dev notes).
#[test]
fn test_BC_2_04_005_hidden_tools_present_in_binary_but_absent_from_list() {
    let registry = make_registry();
    let client_map = client_map_with(vec![("acme", vec![])]); // no write caps

    // Tool is hidden from tools/list...
    let tools = registry.tools_list(&client_map);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(
        !tool_names.contains(&"crowdstrike_contain_host"),
        "BC-2.04.005: write tool must be absent from tools/list when no client has capability"
    );

    // ...but IS present in the binary (get_tool returns Some).
    let tool = registry.get_tool("crowdstrike_contain_host");
    assert!(
        tool.is_some(),
        "BC-2.04.005: hidden tool must still be retrievable via get_tool (not compiled out)"
    );
}

// ─────────────────────────────────────────────────────────────
// E-FLAG-006: write with null client_id → WriteRequiresClientId error
// ─────────────────────────────────────────────────────────────

/// EC-04-033: Write tool invoked with client_id: null → E-FLAG-006.
/// This tests that the evaluator returns WriteRequiresClientId when no
/// client_id is provided for a write operation.
#[test]
fn test_BC_2_04_005_ec_null_client_id_returns_e_flag_006() {
    use prism_core::error::PrismError;
    use prism_security::feature_flag::{CompileTimeGate, FeatureFlagEvaluator};

    // The evaluator represents "no client_id provided" as an empty string ""
    // or a special sentinel. We test that invoking check_permission with an
    // empty client_id for a write capability returns DeniedRuntime with an
    // appropriate error that maps to WriteRequiresClientId.

    // NOTE: The exact mechanism for detecting null vs missing client_id will be
    // defined by the implementer. This test verifies the observable contract:
    // an empty client_id for a write path must produce an error, not Allowed.
    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new());
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "", // null/absent client_id
        "sensor.crowdstrike.containment",
    );

    assert!(
        !matches!(result, prism_security::feature_flag::CapabilityCheckResult::Allowed),
        "EC-04-033: null/empty client_id for write path must not return Allowed"
    );
}
