//! Red Gate test suite for DEFECT-CLAROTY-SORTBY-DETERMINISM-001
//! Claroty xDome sort_by determinism — 7 tables, 10 tests (RG-001..RG-010).
//!
//! Story: DEFECT-CLAROTY-SORTBY-DETERMINISM-001
//! BCs: BC-2.16.015 v2.0, BC-2.16.013 v1.43, BC-2.16.019 v1.3, BC-2.16.020 v1.3, BC-2.16.021 v1.3
//!
//! ## Red Gate invariant
//!
//! ALL 10 tests in this file MUST FAIL before the TOML implementation (Task 5).
//! Failure mode: `assert!` panics because `"sort_by"` is absent from every current
//! `body_template` in `claroty.sensor.toml`. Tests fail on assertion errors — not
//! compile errors. Any test that passes before Task 5 indicates the TOML was already
//! modified (investigate before proceeding).
//!
//! ## Red Gate tests
//!
//! | RG  | Test function                                          | Traces to BC / AC               |
//! |-----|--------------------------------------------------------|---------------------------------|
//! | 001 | test_rg_vulnerabilities_sort_by_in_request_body        | BC-2.16.015 v2.0 §Post §1       |
//! | 002 | test_rg_audit_logs_sort_by_in_request_body             | BC-2.16.013 v1.43 §Post §1      |
//! | 003 | test_rg_server_interfaces_sort_by_in_request_body      | BC-2.16.019 v1.3 §Post §1       |
//! | 004 | test_rg_organization_zones_sort_by_in_request_body     | BC-2.16.020 v1.3 §Post §1       |
//! | 005 | test_rg_organization_zone_policies_sort_by_in_request_body | BC-2.16.020 v1.3 §Post §2   |
//! | 006 | test_rg_organization_firewall_groups_sort_by_in_request_body | BC-2.16.021 v1.3 §Post §1 |
//! | 007 | test_rg_organization_firewall_policies_sort_by_in_request_body | BC-2.16.021 v1.3 §Post §2|
//! | 008 | test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field | BC-2.16.015 v2.0 §Post §1   |
//! | 009 | test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback   | BC-2.16.013 v1.43 §Post §1      |
//! | 010 | test_rg_server_interfaces_composite_key_both_present   | BC-2.16.019 v1.3 §Post §1       |
//!
//! BC-5.38.001 density check: 10 RGTs / 7 ACs = 1.43 (≥ 0.5 threshold). PASS.
//!
//! ## SAP-1 compliance
//!
//! No tracing emissions added — this is a spec-parse test file (TOML static assertions only).
//! SAP-1 status: N/A (no event_type = emissions in this file).
//!
//! ## SAP-2 compliance
//!
//! sort_by is a REQUEST-BODY parameter; it does not appear in DTU response structs.
//! SAP-2 (DTU↔TOML schema parity) does not apply to request-body fields.
//! SAP-2 status: N/A (request-body only; no response-shape impact).

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use prism_spec_engine::spec_parser::SpecLoader;

const CLAROTY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/claroty.sensor.toml"
));

// ── Helper ────────────────────────────────────────────────────────────────────
/// Extract the `sort_by` JSON array from a `body_template` string.
///
/// Locates the `"sort_by":` key in the template, finds the opening `[`, then
/// walks forward to find the matching `]` (handling nested brackets). Returns
/// the parsed array.
///
/// **Panics (test failure) if:**
/// - `"sort_by":` is absent from the body_template (the expected RED state
///   before the TOML edit — this panic IS the red gate)
/// - The extracted value is not a valid JSON array
fn extract_sort_by_array(body_template: &str) -> Vec<serde_json::Value> {
    let sort_by_marker = r#""sort_by":"#;
    let sort_by_pos = body_template.find(sort_by_marker).expect(
        "body_template must contain a \"sort_by\" key \
         (RED GATE: absent before Task-5 TOML edit; \
         BC-2.16.015/013/019/020/021 postcondition not yet implemented)",
    );
    let after_key = &body_template[sort_by_pos + sort_by_marker.len()..];
    let bracket_start = after_key
        .find('[')
        .expect("sort_by value must be a JSON array starting with '['");
    let array_str = &after_key[bracket_start..];

    // Walk to find the matching ']', respecting nested brackets.
    let mut depth = 0usize;
    let mut end = 0usize;
    for (i, c) in array_str.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(end > 0, "sort_by array must have a matching closing ']'");
    let json_array = &array_str[..end];
    serde_json::from_str(json_array).expect("sort_by value must be a valid JSON array")
}

// ── RG-001 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 v2.0 §Postconditions §1 sort-by postcondition — AC-001
/// EC-016-015-009 (offset pagination determinism).
///
/// The `fetch_vulnerabilities` body_template MUST contain a `"sort_by"` key
/// whose value includes `adjusted_vulnerability_score` (primary sort, desc) and
/// `name` (tiebreaker, asc) — the exact two-element array contracted by
/// BC-2.16.015 v2.0 §Post §1.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_vulnerabilities_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "vulnerabilities")
        .expect("vulnerabilities table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_vulnerabilities")
        .expect("fetch_vulnerabilities step must exist in vulnerabilities table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_vulnerabilities step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-001 RED GATE (BC-2.16.015 v2.0 §Post §1): \
         fetch_vulnerabilities body_template must contain a \"sort_by\" key \
         with [adjusted_vulnerability_score desc, name asc]; \
         currently absent — EC-016-015-009 offset pagination is non-deterministic"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("adjusted_vulnerability_score"),
        "sort_by must reference adjusted_vulnerability_score (primary sort field, BC-2.16.015 v2.0 §Post §1)"
    );
    assert!(
        body_template.contains(r#""order":"desc""#),
        "sort_by first element must use order:desc for adjusted_vulnerability_score \
         (highest-risk records survive DI-019 10K cap, BC-2.16.015 v2.0 §Post §1)"
    );
}

// ── RG-002 ────────────────────────────────────────────────────────────────────
/// BC-2.16.013 v1.43 §Postconditions §1 audit_logs sort-by postcondition — AC-002
/// EC-016-013-011 (offset pagination determinism).
///
/// The `fetch_audit_logs` body_template MUST contain a `"sort_by"` key with the
/// preferred value `[{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"filter_by": ${...}` is present before Task 5).
#[test]
fn test_rg_audit_logs_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "audit_logs")
        .expect("audit_logs table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_audit_logs")
        .expect("fetch_audit_logs step must exist in audit_logs table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_audit_logs step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-002 RED GATE (BC-2.16.013 v1.43 §Post §1): \
         fetch_audit_logs body_template must contain a \"sort_by\" key \
         with [timestamp asc, id asc] (preferred) or [timestamp asc] (fallback); \
         currently absent — EC-016-013-011 offset pagination is non-deterministic"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("timestamp"),
        "sort_by must reference timestamp field (primary sort, BC-2.16.013 v1.43 §Post §1)"
    );
    assert!(
        body_template.contains("\"id\""),
        "sort_by must reference id field (tiebreaker — preferred form; \
         BC-2.16.013 v1.43 §Post §1 id-tiebreaker caveat; \
         use fallback [timestamp only] if live xDome API rejects 'id')"
    );
}

// ── RG-003 ────────────────────────────────────────────────────────────────────
/// BC-2.16.019 v1.3 §Postconditions §1 sort-by postcondition — AC-003
/// EC-016-019-007 (offset pagination determinism, composite PK guarantee).
///
/// The `fetch_server_interfaces` body_template MUST contain a `"sort_by"` key
/// with value `[{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_server_interfaces_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "server_interfaces")
        .expect("server_interfaces table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_server_interfaces")
        .expect("fetch_server_interfaces step must exist in server_interfaces table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_server_interfaces step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-003 RED GATE (BC-2.16.019 v1.3 §Post §1): \
         fetch_server_interfaces body_template must contain a \"sort_by\" key \
         with [server_name asc, interface_name asc]; \
         currently absent — EC-016-019-007 offset pagination is non-deterministic \
         (server_name alone is non-unique without interface_name tiebreaker)"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("server_name"),
        "sort_by must reference server_name (primary sort field, BC-2.16.019 v1.3 §Post §1)"
    );
    assert!(
        body_template.contains("interface_name"),
        "sort_by must reference interface_name (composite PK tiebreaker, BC-2.16.019 v1.3 §Post §1)"
    );
}

// ── RG-004 ────────────────────────────────────────────────────────────────────
/// BC-2.16.020 v1.3 §Postconditions §1 zones sort-by postcondition — AC-004
/// EC-016-020-011 (offset pagination determinism).
///
/// The `fetch_organization_zones` body_template MUST contain a `"sort_by"` key
/// with value `[{"field":"zone_name","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_organization_zones_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("organization_zones table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_organization_zones")
        .expect("fetch_organization_zones step must exist in organization_zones table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_organization_zones step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-004 RED GATE (BC-2.16.020 v1.3 §Post §1): \
         fetch_organization_zones body_template must contain a \"sort_by\" key \
         with [zone_name asc]; currently absent — EC-016-020-011 \
         offset pagination is non-deterministic (API default: priority asc, non-unique)"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("zone_name"),
        "sort_by must reference zone_name (REQUIRED PK, BC-2.16.020 v1.3 §Post §1; \
         OrganizationZones__sortable_fields_enum)"
    );
}

// ── RG-005 ────────────────────────────────────────────────────────────────────
/// BC-2.16.020 v1.3 §Postconditions §2 zone_policies sort-by postcondition — AC-005
/// EC-016-020-012 (offset pagination determinism).
///
/// The `fetch_organization_zone_policies` body_template MUST contain a `"sort_by"` key
/// with value `[{"field":"policy_name","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_organization_zone_policies_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect("organization_zone_policies table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_organization_zone_policies")
        .expect(
            "fetch_organization_zone_policies step must exist in organization_zone_policies table",
        );
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_organization_zone_policies step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-005 RED GATE (BC-2.16.020 v1.3 §Post §2): \
         fetch_organization_zone_policies body_template must contain a \"sort_by\" key \
         with [policy_name asc]; currently absent — EC-016-020-012 \
         offset pagination is non-deterministic (API default: matching_devices asc, non-unique)"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("policy_name"),
        "sort_by must reference policy_name (REQUIRED PK, BC-2.16.020 v1.3 §Post §2; \
         OrganizationZonePolicies__sortable_fields_enum)"
    );
}

// ── RG-006 ────────────────────────────────────────────────────────────────────
/// BC-2.16.021 v1.3 §Postconditions §1 firewall_groups sort-by postcondition — AC-006
/// EC-016-021-011 (offset pagination determinism).
///
/// The `fetch_organization_firewall_groups` body_template MUST contain a `"sort_by"` key
/// with value `[{"field":"firewall_group_name","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_organization_firewall_groups_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_firewall_groups")
        .expect("organization_firewall_groups table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_organization_firewall_groups")
        .expect(
            "fetch_organization_firewall_groups step must exist in organization_firewall_groups table",
        );
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_organization_firewall_groups step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-006 RED GATE (BC-2.16.021 v1.3 §Post §1): \
         fetch_organization_firewall_groups body_template must contain a \"sort_by\" key \
         with [firewall_group_name asc]; currently absent — EC-016-021-011 \
         offset pagination is non-deterministic (API default: priority asc, non-unique)"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("firewall_group_name"),
        "sort_by must reference firewall_group_name (REQUIRED PK, BC-2.16.021 v1.3 §Post §1; \
         OrganizationFirewallGroups__sortable_fields_enum)"
    );
}

// ── RG-007 ────────────────────────────────────────────────────────────────────
/// BC-2.16.021 v1.3 §Postconditions §2 firewall_policies sort-by postcondition — AC-007
/// EC-016-021-012 (offset pagination determinism).
///
/// The `fetch_organization_firewall_policies` body_template MUST contain a `"sort_by"` key
/// with value `[{"field":"policy_name","order":"asc"}]`.
///
/// RED: `assert!` panics because `"sort_by"` is absent from the current
/// body_template (only `"fields": [...]` is present before Task 5).
#[test]
fn test_rg_organization_firewall_policies_sort_by_in_request_body() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_firewall_policies")
        .expect("organization_firewall_policies table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_organization_firewall_policies")
        .expect(
            "fetch_organization_firewall_policies step must exist in organization_firewall_policies table",
        );
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_organization_firewall_policies step must have a body_template");

    // RED GATE: sort_by is absent before Task-5 TOML edit.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-007 RED GATE (BC-2.16.021 v1.3 §Post §2): \
         fetch_organization_firewall_policies body_template must contain a \"sort_by\" key \
         with [policy_name asc]; currently absent — EC-016-021-012 \
         offset pagination is non-deterministic (API default: matching_devices asc, non-unique)"
    );

    // Additional field-presence assertions (reached only after RED gate passes at GREEN time).
    assert!(
        body_template.contains("policy_name"),
        "sort_by must reference policy_name (REQUIRED PK, BC-2.16.021 v1.3 §Post §2; \
         OrganizationFirewallGroupPolicies__sortable_fields_enum)"
    );
}

// ── RG-008 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 v2.0 §Postconditions §1 structural assertion — AC-001
/// EC-016-015-009 + DI-019 truncation rationale.
///
/// Parses the embedded `sort_by` JSON array from `fetch_vulnerabilities`
/// body_template and asserts the structural contract:
/// - Array has exactly 2 elements
/// - Element [0]: `{"field": "adjusted_vulnerability_score", "order": "desc"}` (primary)
/// - Element [1]: `{"field": "name", "order": "asc"}` (tiebreaker — provably unique CVE ID)
///
/// The `name` tiebreaker (CVE ID / advisory title) makes the sort order total,
/// ensuring deterministic page boundaries under DI-019 10K cap.
///
/// RED: `extract_sort_by_array` panics at the `.expect(...)` for the missing key
/// because `"sort_by"` is absent from the current body_template.
#[test]
fn test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "vulnerabilities")
        .expect("vulnerabilities table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_vulnerabilities")
        .expect("fetch_vulnerabilities step must exist in vulnerabilities table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_vulnerabilities step must have a body_template");

    // Presence check (earlier failure mode — cleaner message than the helper panic).
    assert!(
        body_template.contains(r#""sort_by""#),
        "RG-008 RED GATE (BC-2.16.015 v2.0 §Post §1): \
         fetch_vulnerabilities body_template must contain \"sort_by\" key (absent before Task 5)"
    );

    // Structural parse: extract and verify the sort_by array.
    let sort_by = extract_sort_by_array(body_template);

    assert_eq!(
        sort_by.len(),
        2,
        "BC-2.16.015 v2.0 §Post §1: fetch_vulnerabilities sort_by must have exactly 2 elements \
         [adjusted_vulnerability_score desc, name asc]; got {:?}",
        sort_by
    );

    // Element [0]: primary sort — adjusted_vulnerability_score desc.
    let primary = &sort_by[0];
    assert_eq!(
        primary.get("field").and_then(|f| f.as_str()),
        Some("adjusted_vulnerability_score"),
        "sort_by[0] field must be 'adjusted_vulnerability_score' (primary sort, \
         highest-risk records survive DI-019 10K cap; BC-2.16.015 v2.0 §Post §1); \
         got: {:?}",
        primary
    );
    assert_eq!(
        primary.get("order").and_then(|o| o.as_str()),
        Some("desc"),
        "sort_by[0] order must be 'desc' for adjusted_vulnerability_score \
         (highest-risk first; BC-2.16.015 v2.0 §Post §1); got: {:?}",
        primary
    );

    // Element [1]: tiebreaker — name asc (CVE ID, provably unique).
    // Confirmed member of Vulnerability__sortable_fields_enum (ValidatingSortClause__6).
    let tiebreaker = &sort_by[1];
    assert_eq!(
        tiebreaker.get("field").and_then(|f| f.as_str()),
        Some("name"),
        "sort_by[1] field must be 'name' (CVE ID tiebreaker, provably unique; \
         OCSF finding_info.title semantics; Vulnerability__sortable_fields_enum; \
         BC-2.16.015 v2.0 §Post §1 + EC-016-015-009); got: {:?}",
        tiebreaker
    );
    assert_eq!(
        tiebreaker.get("order").and_then(|o| o.as_str()),
        Some("asc"),
        "sort_by[1] order must be 'asc' for name tiebreaker \
         (BC-2.16.015 v2.0 §Post §1); got: {:?}",
        tiebreaker
    );

    // Verify desc-before-asc ordering: primary is desc, tiebreaker is asc.
    // This is the structural contract that proves non-trivial sort ordering.
    let primary_order = primary.get("order").and_then(|o| o.as_str()).unwrap_or("");
    let tiebreaker_order = tiebreaker
        .get("order")
        .and_then(|o| o.as_str())
        .unwrap_or("");
    assert_eq!(
        primary_order, "desc",
        "sort_by[0] (adjusted_vulnerability_score) must be 'desc' — \
         desc-before-asc ordering confirms DI-019 truncation rationale"
    );
    assert_eq!(
        tiebreaker_order, "asc",
        "sort_by[1] (name) must be 'asc' — \
         asc tiebreaker after desc primary confirms total sort order"
    );
}

// ── RG-009 ────────────────────────────────────────────────────────────────────
/// BC-2.16.013 v1.43 §Postconditions §1 audit_logs structural assertion — AC-002
/// EC-016-013-011 (offset pagination determinism) + EC-002 (filter_by coexistence guard).
///
/// Asserts on the `fetch_audit_logs` body_template:
/// 1. BOTH `"filter_by"` AND `"sort_by"` substrings are present (coexistence guard —
///    EC-002: sort_by MUST NOT replace filter_by; the 7-day time window comes from
///    filter_by; removing it causes unbounded queries).
/// 2. The sort_by JSON array contains a `timestamp` entry.
/// 3. Either the preferred form (id tiebreaker present) OR the fallback form
///    (timestamp only) is used — both are acceptable per BC-2.16.013 §id-tiebreaker-caveat.
///
/// Note: `audit_logs` body_template uses a variable substitution
/// `${query.filter._claroty_audit_filter_by}` which is NOT valid JSON. The whole
/// body_template cannot be parsed as JSON. Instead: (a) substring checks for coexistence,
/// (b) string extraction to isolate the literal `sort_by` JSON array, then JSON parse.
///
/// RED: assertion 2 panics because `"sort_by"` is absent from the current body_template.
/// (Assertion 1, `"filter_by"` presence, passes even in RED state — this is expected;
/// the RED gate fires on the sort_by absence.)
#[test]
fn test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "audit_logs")
        .expect("audit_logs table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_audit_logs")
        .expect("fetch_audit_logs step must exist in audit_logs table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_audit_logs step must have a body_template");

    // 1. Coexistence guard: filter_by must STILL be present (not replaced).
    //    This assertion PASSES even at RED time (filter_by exists in current template).
    assert!(
        body_template.contains(r#""filter_by""#),
        "EC-002 COEXISTENCE: fetch_audit_logs body_template must retain \"filter_by\" key — \
         the 7-day time-window enforcement (S-CLAROTY-AUDITLOG-TIMEBOX-001) depends on it; \
         removing filter_by would cause unbounded queries (BC-2.16.013 §2.2 risk note)"
    );

    // 2. sort_by presence: RED GATE — this assertion FAILS in RED state.
    assert!(
        body_template.contains(r#""sort_by""#),
        "AC-002 RED GATE (BC-2.16.013 v1.43 §Post §1): \
         fetch_audit_logs body_template must contain a \"sort_by\" key \
         alongside \"filter_by\" (currently absent — EC-016-013-011 offset pagination \
         is non-deterministic without an explicit sort order)"
    );

    // 3. Structural parse of the sort_by array (JSON literal portion only).
    //    Note: the full body_template is NOT valid JSON (contains variable placeholder).
    //    extract_sort_by_array isolates the literal JSON array value for "sort_by".
    let sort_by = extract_sort_by_array(body_template);

    // 3a. timestamp entry must be present (BC-2.16.013 §Post §1: primary sort field).
    let has_timestamp = sort_by
        .iter()
        .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("timestamp"));
    assert!(
        has_timestamp,
        "sort_by must contain a 'timestamp' entry (primary sort field, \
         BC-2.16.013 v1.43 §Post §1, EC-016-013-011); got: {:?}",
        sort_by
    );

    // 3b. Either preferred form (id tiebreaker) or fallback form (timestamp only).
    //     Both forms are contractually acceptable per BC-2.16.013 §id-tiebreaker-caveat.
    //     Preferred: [timestamp asc, id asc] — use if live xDome accepts id as sort key.
    //     Fallback:  [timestamp asc]          — use if live xDome rejects id (AC-002 caveat).
    let has_id_tiebreaker = sort_by
        .iter()
        .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("id"));
    let is_timestamp_only = sort_by.len() == 1;
    assert!(
        has_id_tiebreaker || is_timestamp_only,
        "sort_by must be the preferred form [timestamp asc, id asc] \
         OR the fallback form [timestamp asc] (if id is rejected by live xDome API); \
         BC-2.16.013 v1.43 §Post §1 id-tiebreaker caveat + EC-016-013-011; \
         got: {:?}",
        sort_by
    );
}

// ── RG-010 ────────────────────────────────────────────────────────────────────
/// BC-2.16.019 v1.3 §Postconditions §1 structural assertion — AC-003
/// EC-016-019-007 (offset pagination determinism, composite PK uniqueness guarantee).
///
/// Parses the embedded `sort_by` JSON array from `fetch_server_interfaces`
/// body_template and asserts the structural contract:
/// - Array has exactly 2 elements
/// - Element [0]: `{"field": "server_name", "order": "asc"}` (primary)
/// - Element [1]: `{"field": "interface_name", "order": "asc"}` (tiebreaker)
///
/// The composite `(server_name, interface_name)` is the unique PK per
/// BC-2.16.019 §Postconditions §3, making the sort order total.
/// `server_name` alone is non-unique (a server has multiple interfaces).
///
/// RED: `extract_sort_by_array` panics at the `.expect(...)` for the missing key
/// because `"sort_by"` is absent from the current body_template.
#[test]
fn test_rg_server_interfaces_composite_key_both_present() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "server_interfaces")
        .expect("server_interfaces table must exist in claroty.sensor.toml");
    let step = table
        .steps
        .iter()
        .find(|s| s.name == "fetch_server_interfaces")
        .expect("fetch_server_interfaces step must exist in server_interfaces table");
    let body_template = step
        .body_template
        .as_deref()
        .expect("fetch_server_interfaces step must have a body_template");

    // Presence check (earlier failure mode — cleaner message than the helper panic).
    assert!(
        body_template.contains(r#""sort_by""#),
        "RG-010 RED GATE (BC-2.16.019 v1.3 §Post §1): \
         fetch_server_interfaces body_template must contain \"sort_by\" key (absent before Task 5)"
    );

    // Structural parse: extract and verify the sort_by array.
    let sort_by = extract_sort_by_array(body_template);

    assert_eq!(
        sort_by.len(),
        2,
        "BC-2.16.019 v1.3 §Post §1: fetch_server_interfaces sort_by must have exactly 2 elements \
         [server_name asc, interface_name asc]; composite PK guarantee; got {:?}",
        sort_by
    );

    // Element [0]: server_name asc (primary sort).
    let primary = &sort_by[0];
    assert_eq!(
        primary.get("field").and_then(|f| f.as_str()),
        Some("server_name"),
        "sort_by[0] field must be 'server_name' (primary sort; \
         ServerInterfaces__sortable_fields_enum; BC-2.16.019 v1.3 §Post §1); \
         got: {:?}",
        primary
    );
    assert_eq!(
        primary.get("order").and_then(|o| o.as_str()),
        Some("asc"),
        "sort_by[0] order must be 'asc' for server_name \
         (BC-2.16.019 v1.3 §Post §1); got: {:?}",
        primary
    );

    // Element [1]: interface_name asc (tiebreaker — completes composite PK uniqueness).
    // interface_name is Tier-2 (raw_extensions) but is a PK element per BC-2.16.019 §Post §3.
    let tiebreaker = &sort_by[1];
    assert_eq!(
        tiebreaker.get("field").and_then(|f| f.as_str()),
        Some("interface_name"),
        "sort_by[1] field must be 'interface_name' (composite PK tiebreaker; \
         ServerInterfaces__sortable_fields_enum; BC-2.16.019 v1.3 §Post §1 + §Post §3); \
         got: {:?}",
        tiebreaker
    );
    assert_eq!(
        tiebreaker.get("order").and_then(|o| o.as_str()),
        Some("asc"),
        "sort_by[1] order must be 'asc' for interface_name \
         (BC-2.16.019 v1.3 §Post §1); got: {:?}",
        tiebreaker
    );
}
