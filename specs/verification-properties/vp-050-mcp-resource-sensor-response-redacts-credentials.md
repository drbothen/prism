---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.10.008
input-hash: "1e29f9d"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.10.008
module: prism-mcp
priority: P0
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-050: MCP Sensor Resource Response Redacts Credentials and Full API URLs

## Property Statement

`render_sensor_inventory_resource(config: &ClientSensorConfig) -> SensorInventoryResponse`
given a `ClientSensorConfig` containing full API base URLs and credential-like values
produces a response JSON where: (a) no string matching an API key pattern (UUID-format
token, `Bearer` prefix, base64 32+ characters) appears anywhere in the serialized output;
and (b) any API base URL field contains only the host+port component, not the full URL
path, query string, or embedded credentials.

## Source Contract

- **Anchor Story:** `S-5.03`
- **Source BC:** BC-2.10.008 — MCP Resources for Client List and Sensor Inventory
- **Module:** prism-mcp
- **Category:** Security / Credential Safety

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — generates ClientSensorConfig with fabricated API keys, bearer tokens, full URLs | All credential-pattern types and URL formats |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_mcp::resources::render_sensor_inventory_resource
//
// proptest!(|(config in arb_client_sensor_config_with_credentials())| {
//     let response = render_sensor_inventory_resource(&config);
//     let json = serde_json::to_string(&response).unwrap();
//
//     // (a) No API key patterns in output
//     // UUID-format token: 8-4-4-4-12 hex, all lowercase/uppercase
//     let uuid_pattern = regex::Regex::new(
//         r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
//     ).unwrap();
//     prop_assert!(!uuid_pattern.is_match(&json),
//         "response must not contain UUID-format API key tokens");
//
//     // Bearer token pattern
//     prop_assert!(!json.contains("Bearer "),
//         "response must not contain Bearer token prefix");
//
//     // Base64 strings of 32+ characters (API key heuristic)
//     let b64_pattern = regex::Regex::new(r"[A-Za-z0-9+/]{32,}={0,2}").unwrap();
//     prop_assert!(!b64_pattern.is_match(&json),
//         "response must not contain long base64 strings (API key heuristic)");
//
//     // (b) API base URL contains only host+port
//     if let Some(api_url_field) = response.api_base_url() {
//         let parsed = url::Url::parse(api_url_field).unwrap();
//         prop_assert!(parsed.path() == "/" || parsed.path().is_empty(),
//             "api_base_url must contain only host+port, not path '{}'", parsed.path());
//         prop_assert!(parsed.query().is_none(),
//             "api_base_url must not contain query string");
//         prop_assert!(parsed.password().is_none(),
//             "api_base_url must not contain embedded password");
//     }
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | proptest generates configs with arbitrary credential patterns; regex assertions are deterministic |
| Tool support? | Full | proptest + serde_json + regex; pure render function is ideal proptest target |
| Execution time budget | <60 seconds for 5000 cases | Serialization is cheap; regex matching is fast |
| Assumptions required | `render_sensor_inventory_resource` is a pure function (no I/O, no database calls) that accepts ClientSensorConfig and returns serializable SensorInventoryResponse | Must be extractable from async MCP handler |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.10.008. P0 security property: MCP resource response must never expose API key patterns or full credential-bearing URLs to the AI context. |
