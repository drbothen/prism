---
document_type: behavioral-contract
level: L3
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Server & Transport"
capability: CAP-009
---

# BC-2.10.004: Client Scoping on Every Tool (Stateless Model)

**Note:** This file replaces BC-2.10.004 v1.0. With per-sensor read tools removed, client scoping follows two patterns: read tools use the `query` tool's `clients` array, while write tools and management tools use scalar `client_id`.

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
- `set_credential` and `delete_credential` require `client_id: String` (non-null, per-client scoped)

### Confirmation Tool
- `confirm_action` validates `client_id` against the token's embedded `client_id`, not against client configuration. The `__global__` sentinel is valid for `confirm_action` only -- it matches when the token was generated for a global-scope operation (aliases, schedules, packs, global-scope rules).

## Invariants
- DI-008: Client data separation -- client scoping is enforced on every tool call
- Stateless: there is no session-level "active client" context. Each tool call is self-contained.
- Write operations always require a specific (non-null) client_id

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` or `clients` entry contains invalid characters | Structured error: `code: "E-MCP-001"`, `message: "Invalid client_id format"`, `allowed_pattern: "[a-zA-Z0-9_-]+"` |
| `PrismError::Config` | Non-null `client_id` not found in config | Structured error: `code: "E-CFG-001"`, `message: "Client '{id}' not found"`, `suggestion: "Check TOML config for available clients"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-005 | Cross-client query where some clients lack the target sensor | Clients without the sensor are silently skipped; `sensor_errors` reports them |
| DEC-003 | Cross-client query where one client has expired credentials | Partial results returned; `sensor_errors` array in response |
| EC-10-007 | `client_id` is an empty string | Treated as invalid input (fails `[a-zA-Z0-9_-]+` validation) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-008 |
| L2 Edge Cases | DEC-003, DEC-005 |
| Replaces | BC-2.10.004 v1.0 (universal client_id on every tool) |
| Priority | P0 |
