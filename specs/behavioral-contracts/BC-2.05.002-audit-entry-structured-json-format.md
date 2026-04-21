---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "8bd996e"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.002: Audit Entries Use Structured JSON Format with Complete Fields

## Description

Every audit entry is a valid JSON object containing a complete set of required fields:
`timestamp` (ISO 8601 UTC), `trace_id` (unique per invocation), `client_id`, `tool_name`,
`parameters` (with secrets redacted), `user_identity`, `result_summary`, `capability_checks`,
and `safety_flags`. No field is omitted even when its value is empty or null. The `client_id`
field handles multi-client scenarios with defined sentinels: `"multi_client"` for fan-out
queries, `"all_clients"` for cross-client queries with `clients: null`, and `"cross_client"`
for `client_id: null` on non-query tools.

For query tool invocations, `parameters` includes both `original_query` and `expanded_query`
to enable audit trail reconstruction of alias expansion. User identity is resolved at startup
(TOML `analyst_id` → env var → OS username) and is immutable for the session lifetime.

## Preconditions
- An MCP tool invocation has been dispatched
- The audit middleware is constructing an `AuditEntry`

## Postconditions
- The emitted audit entry is valid JSON containing all required fields:
  - `timestamp` (ISO 8601 UTC)
  - `trace_id` (unique per invocation)
  - `client_id` (for single-client tools: the `TenantId` from the tool call; for query engine tools operating on multiple clients: `"multi_client"` with the full client list in `parameters`; for cross-client queries with `clients: null`: `"all_clients"`; for `client_id: null` on non-query tools: `"cross_client"`)
  - `tool_name` (the MCP tool name, e.g., `query_crowdstrike_alerts`)
  - `parameters` (the tool input parameters as JSON, with secrets redacted). For `query` tool invocations, the parameters field includes both `original_query` (the raw query string as submitted) and `expanded_query` (the query after alias resolution), enabling audit trail reconstruction of alias expansion.
  - `user_identity` (the analyst identity, resolved at startup via: (1) TOML config `analyst_id` field, (2) `PRISM_ANALYST_ID` env var, (3) OS username detection; first non-empty value wins; immutable for session lifetime)
  - `result_summary` (success/failure outcome)
  - `capability_checks` (array of capability evaluations, may be empty for read ops)
  - `safety_flags` (array of triggered prompt injection flags, may be empty)
- The JSON structure is compatible with the existing Vector pipeline for ingestion

## Invariants
- DI-004: Audit completeness -- no field is omitted even if the value is empty or null

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Missing `client_id` | Tool call lacks `client_id` entirely (malformed request) | Audit entry records `client_id: "MISSING"` and the tool returns `PrismError::InvalidInput` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-002 | Cross-client query (`client_id: null`) | Audit entry records `client_id: "cross_client"` to distinguish from single-client invocations |
| EC-05-003 | Tool name is a meta-tool (`list_capabilities`) | Audit entry is still emitted with the same field completeness requirements |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.002.

| Scenario | `client_id` in Tool Call | `client_id` in Audit Entry |
|----------|--------------------------|---------------------------|
| Single-client tool | `"acme"` | `"acme"` |
| Multi-client fan-out | `["acme", "beta"]` | `"multi_client"` (full list in `parameters`) |
| Cross-client query | `null` | `"all_clients"` |
| Non-query with null `client_id` | `null` | `"cross_client"` |
| Missing `client_id` | absent | `"MISSING"`; tool returns `PrismError::InvalidInput` |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify audit entry field completeness. Placeholder for future VP (proptest: all emitted entries are valid JSON with all required fields present).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
