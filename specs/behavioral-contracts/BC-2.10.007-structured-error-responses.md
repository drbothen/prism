---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "MCP Interface"
capability: "CAP-034"
---

# BC-2.10.007: Structured Error Responses

## Preconditions
- A tool invocation has encountered an error (sensor API failure, validation error, auth error, etc.)
- The `PrismError` has been mapped to an MCP error response

## Postconditions
- Error responses include `isError: true` on the MCP tool result
- The `content[].text` follows the pattern: `"ERROR: [{category}] - {message}. {suggestion}"`
- The `structuredContent.error` object contains:
  - `code`: Error code from the taxonomy (e.g., `"E-SENSOR-001"`)
  - `message`: Human-readable error description
  - `category`: Error category string (`"transient"`, `"authentication"`, `"validation"`, `"not_found"`, `"permission"`, `"upstream_error"`, `"configuration"`, `"safety"`)
  - `retryable`: boolean indicating whether the same request might succeed on retry
  - `retry_after_seconds`: Optional integer (present only when `retryable: true` and a specific delay is known)
  - `suggestion`: Actionable text guiding the LLM toward resolution
  - `source`: Origin of the error (e.g., `"crowdstrike_falcon_api"`, `"prism_config"`)
  - `original_params_valid`: boolean indicating whether the tool parameters were the cause
- Error responses include `_meta.trust_level: "internal"` (errors are Prism-generated)
- Upstream sensor error messages are included in `upstream_message` field but never interpolated into the prose `content[].text`
- No internal implementation details (stack traces, function names, Prism file paths) appear in error responses

## Invariants
- DI-004: Audit completeness -- error responses still generate an AuditEntry with the error code and category
- DI-006: Upstream error messages treated as untrusted data (placed in structured fields, not prose)
- Concurrency note: MCP tool invocations may be pipelined (multiple concurrent requests). Error responses must be correlated with the correct request via the MCP message ID. Shared mutable state (token store, cache) must be accessed with appropriate synchronization.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | This BC defines the error response format itself | All PrismError variants map to this format via `From<PrismError> for McpError` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-012 | Error during error response construction | Fallback to minimal error: `{"error": {"code": "E-MCP-999", "message": "Internal error during error formatting"}}` |
| EC-10-013 | Sensor API error message contains prompt injection payload | Payload appears only in `structuredContent.error.upstream_message`, never in prose text |
| DEC-009 | Expired confirmation token | `code: "E-FLAG-003"`, `category: "permission"`, `retryable: false`, `suggestion` includes original tool name |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-004, DI-006 |
| L2 Edge Cases | DEC-009 |
| Priority | P0 |
