---
document_type: behavioral-contract
level: L3
version: "2.8"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-14"  # v2.8: R1 reconciliation — lock 3-case client_id error taxonomy; validate_client_ids implementer note; canonical MCP error-code table
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.004: Client Scoping on Every Tool (Stateless Model)

**Note:** This file replaces BC-2.10.004 v1.0. With per-sensor read tools removed, client scoping follows two patterns: read tools use the `query` tool's `clients` array, while write tools and management tools use scalar `client_id`.

## Description

Every tool call carries explicit client scoping — there is no session-level "active client" context. Read tools use a `clients` array (null = all clients, array = specific clients). Write tools require a non-null scalar `client_id` (cross-client writes are not supported). Management tools vary: health and capabilities accept null for cross-client overview; credential listing requires non-null `client_id` per security policy. The `confirm_action` tool validates `client_id` against the token's embedded `client_id`, accepting the `__global__` sentinel for global-scope operations. All `client_id` values are validated against `[a-zA-Z0-9_-]+` before any processing per DI-008.

## Preconditions
- An MCP tool call is received by the server handler

## Postconditions

### Read Tools (via `query`)
- The `query` and `explain_query` tools use a `clients` array parameter:
  - `clients: null` -- query all configured clients (cross-client mode)
  - `clients: ["acme"]` -- query a single client
  - `clients: ["acme", "globex"]` -- query specific clients
- Each `client_id` in the array is validated against `[a-zA-Z0-9_-]+` before any processing
- The `clients` array is included in the tracing span and audit entry

### Read Management Tools
- `check_sensor_health` uses `client_id: Option<String>` (null for cross-client health overview)
- `list_capabilities` uses `client_id: Option<String>` (null for all clients)
- `list_credentials` uses `client_id: String` (required, non-null -- cross-client credential listing not supported per security policy)
- `list_aliases` uses `scope` filter (not `client_id`) -- aliases are scoped by `global` or `client:<id>`
- `explain_alias` uses `scope` filter (not `client_id`)

### Write Tools
- All write tools (`crowdstrike_contain_host`, `crowdstrike_acknowledge_alert`, `claroty_resolve_alert`, `claroty_device_action`, `cyberint_acknowledge_alert`, `cyberint_close_alert`, `armis_update_alert_status`, `armis_device_action`) require `client_id: String` as a non-null required parameter
- Cross-client write operations are not supported -- `client_id` must always identify a specific client

### Alias Mutation Tools
- `create_alias` and `delete_alias` use `scope` parameter (`global` or `client:<client_id>`) instead of `client_id`

### Credential Mutation Tools
- `configure_credential_source` and `delete_credential` require `client_id: String` (non-null, per-client scoped)

### Confirmation Tool
- `confirm_action` validates `client_id` against the token's embedded `client_id`, not against client configuration. The `__global__` sentinel is valid for `confirm_action` only -- it matches when the token was generated for a global-scope operation (aliases, schedules, packs, global-scope rules).

## Invariants
- DI-008: Client data separation -- client scoping is enforced on every tool call
- Stateless: there is no session-level "active client" context. Each tool call is self-contained.
- Write operations always require a specific (non-null) client_id

## Error Cases

Three distinct conditions govern client_id failures. All three produce `isError: true` MCP tool responses with the structured error shape from BC-2.10.007.

| Case | Condition | PrismError Variant | MCP Error Code | Message Format | `original_params_valid` |
|------|-----------|-------------------|---------------|----------------|------------------------|
| (a) Missing / empty | `client_id` field absent or `""` | (validated pre-dispatch; see implementer note below) | `E-MCP-001` | `"Invalid client_id format: ''"` | `false` |
| (b) Malformed / invalid characters | `client_id` present but fails `[a-zA-Z0-9_-]{1,64}` validation | (validated pre-dispatch; see implementer note below) | `E-MCP-001` | `"Invalid client_id format: '{value}'"` — value is the exact invalid string received | `false` |
| (c) Well-formed but not registered | `client_id` passes format validation but is unknown in runtime config | `PrismError::ClientNotFound { client_id }` | `E-CFG-100` | `"E-CFG-100: client '{client_id}' not found in configuration"` | `true` |

**Implementer note — `validate_client_ids` wiring required for S-5.02:**
The merged `validate_client_ids` in `crates/prism-mcp/src/server.rs` (lines 1215–1231) correctly enforces format cases (a) and (b) but emits a raw `rmcp::ErrorData` with no `E-MCP-001` prefix in the message string. The implementer must bring this up to the ratified contract:
1. `validate_client_ids` message MUST include the `E-MCP-001` error code prefix: `"E-MCP-001: invalid client_id format: '{id}'"`.
2. The structured error response emitted for cases (a) and (b) MUST follow BC-2.10.007's `structuredContent.error` shape with `code: "E-MCP-001"`, `category: "validation"`, `retryable: false`, `original_params_valid: false`.
3. Case (c) is handled by `PrismError::ClientNotFound` → `error_mapping.rs` → `-32602 INVALID_PARAMS` with display `"E-CFG-100: client '{client_id}' not found in configuration"`. The BC-2.10.007 structured error shape must carry `code: "E-CFG-100"`, `category: "configuration"`, `retryable: false`, `original_params_valid: true` (params were structurally valid — the client just does not exist).

**Error-taxonomy alignment:**
- `E-MCP-001` maps to "Invalid client_id format: '{value}'" per error-taxonomy.md §MCP. This is the canonical code for cases (a) and (b).
- `E-CFG-100` maps to "E-CFG-100: client '{client_id}' not found in configuration" per error-taxonomy.md §CFG-100..106. This is the canonical code for case (c).
- `PrismError::InvalidClientId { reason }` in prism-core displays as `"E-AUTH-003: invalid client ID: {reason}"`. This code is NOT used by `validate_client_ids` and must NOT be used here — `E-AUTH-003` in error-taxonomy.md is the sensor-layer bearer-token rejection error, creating a namespace collision in the Rust display. The `validate_client_ids` MCP-layer path correctly bypasses `PrismError::InvalidClientId` and must be kept on the `E-MCP-001` path.

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-005 | Cross-client query where some clients lack the target sensor | Clients without the sensor are silently skipped; `sensor_errors` reports them |
| DEC-003 | Cross-client query where one client has expired credentials | Partial results returned; `sensor_errors` array in response |
| EC-10-007 | `client_id` is an empty string | Treated as invalid input (fails `[a-zA-Z0-9_-]+` validation) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Write tool with valid `client_id` | Tool executes; client scoped correctly | happy-path |
| `client_id: ""` (empty string) | `E-MCP-001` validation error | error |
| Cross-client query where one client lacks the sensor | Partial results; `sensor_errors` lists missing-sensor client | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-001 | OrgSlug rejects invalid characters (formerly TenantId; ADR-006) | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008, DI-033 |
| L2 Edge Cases | DEC-003, DEC-005 |
| Replaces | BC-2.10.004 v1.0 (universal client_id on every tool) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 2.8 | S-5.02-pre-TDD-reconciliation | 2026-06-14 | product-owner | R1 reconciliation: Replaced single-row Error Cases table with canonical 3-case taxonomy table covering (a) missing/empty client_id → E-MCP-001 / original_params_valid=false, (b) malformed/invalid-char client_id → E-MCP-001 / original_params_valid=false, (c) well-formed but unregistered client_id → E-CFG-100 / original_params_valid=true. Added implementer note on validate_client_ids message wiring (must prefix E-MCP-001), BC-2.10.007 structured shape requirements per case, and E-AUTH-003 namespace-collision warning (InvalidClientId Rust Display must NOT be used at MCP layer). Aligns with error-taxonomy.md §MCP E-MCP-001 and §CFG-100..106 E-CFG-100. |
| 2.7 | ADR-038-D6-sweep | 2026-06-10 | product-owner | ADR-038 D6 number sweep: Error Cases row "Non-null `client_id` not found in config" code `E-CFG-001` → `E-CFG-100` (ClientNotFound). The BC froze the pre-v1.8 number; error-taxonomy v1.8 renumbered the client-not-found condition to E-CFG-100, and ADR-037 tombstoned the low number for the retired customer-config semantics. Error-column variant label corrected `PrismError::Config` (nonexistent variant) → `PrismError::ClientNotFound` (the ADR-038 D3 variant whose doc comment cites this BC). Condition text and response shape otherwise unchanged. Per ADR-038 (error-taxonomy v1.66). |
| 2.5 | pass-15-remediation | 2026-04-27 | product-owner | VP-001 label updated TenantId → OrgSlug (ADR-006); DI-033 added to L2 Invariants. |
| 2.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 2.3 | pass-63-fix | 2026-04-20 | product-owner | P3P63-A-OBS-001: Quoted `capability` frontmatter value per corpus convention. Corrected row 2.2 from 5-column to canonical 4-column schema. |
| 2.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref. |
| 2.1 | Burst 43 | 2026-04-19 | product-owner | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` in Credential Mutation Tools section |
| 2.0 | Phase 1 | 2026-04-14 | product-owner | Stateless model; per-sensor reads removed |
