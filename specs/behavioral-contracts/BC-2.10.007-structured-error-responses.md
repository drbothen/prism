---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-034"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-16"  # v1.7: Add "internal" category (9th enum value) for Prism-side infrastructure failures (Internal, Io, Storage*) — distinct from "upstream_error" (sensor failures). Fixes F-4 semantic misclassification from PR #191.
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.007: Structured Error Responses

## Description

All `PrismError` variants map to a consistent MCP error response via `From<PrismError> for McpError`. Error responses include `isError: true`, a structured `error` object with code, category, retryability, suggestion, and source, and `_meta.trust_level: "internal"` (errors are Prism-generated). Upstream sensor error messages are isolated in `upstream_message` and never interpolated into prose `content[].text` per DI-006. No internal implementation details (stack traces, file paths) appear in error responses. Concurrent tool invocations are correlated via MCP message ID; shared mutable state is synchronized.

## Preconditions
- A tool invocation has encountered an error (sensor API failure, validation error, auth error, etc.)
- The `PrismError` has been mapped to an MCP error response

## Postconditions

### Wire shape (NESTED — not flat)

Error responses use the **nested** MCP structured content shape. The story-spec framing of a "flat 7-field object" is superseded by this BC; the implementer must produce the nested shape below.

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "ERROR: [{category}] - {message}. {suggestion}"
    }
  ],
  "structuredContent": {
    "error": {
      "code":                  "E-MCP-001",
      "message":               "Human-readable error description (sanitized, no raw sensor text)",
      "category":              "validation",
      "retryable":             false,
      "retry_after_seconds":   null,
      "suggestion":            "Actionable text guiding the LLM toward resolution",
      "source":                "prism_mcp",
      "original_params_valid": false,
      "upstream_message":      null
    },
    "_meta": {
      "trust_level": "internal"
    }
  }
}
```

### Complete field specification for `structuredContent.error`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `code` | `String` | Always | Error code from taxonomy (e.g., `"E-MCP-001"`, `"E-SENSOR-003"`, `"E-CFG-100"`) |
| `message` | `String` | Always | Human-readable error description. Upstream sensor messages MUST NOT be interpolated here (DI-006). |
| `category` | `String` | Always | One of: `"transient"`, `"authentication"`, `"validation"`, `"not_found"`, `"permission"`, `"upstream_error"`, `"configuration"`, `"safety"`, `"internal"` |
| `retryable` | `bool` | Always | `true` for transient errors (rate limit, timeout, network); `false` for permanent (invalid params, auth invalid) |
| `retry_after_seconds` | `u64 \| null` | Always | For `SensorRateLimited` errors: ALWAYS a non-null `u64` equal to `retry_after_ms / 1000` (the `PrismError::SensorRateLimited` variant carries `retry_after_ms: u64` — a required field, never `Option`). For non-retryable and non-rate-limit errors (validation, auth, internal, etc.): ALWAYS `null`. MUST be `null` (not absent) when no delay applies — consistent client-side handling requires the field to always be present in JSON. |
| `suggestion` | `String` | Always | Short actionable string, e.g., `"Check credential_ref in prism.toml"` |
| `source` | `String` | Always | Origin of the error: `"prism_mcp"` for MCP-layer validation failures; `"crowdstrike_falcon_api"`, `"claroty_api"`, `"armis_api"`, `"cyberint_api"` for sensor errors; `"prism_config"` for configuration errors |
| `original_params_valid` | `bool` | Always | `true` if the tool parameters were structurally valid (error was not caused by bad input shape — e.g., sensor unavailable); `false` if bad parameters caused the error |
| `upstream_message` | `String \| null` | Always | Raw upstream sensor error message placed here ONLY — never interpolated into `message` or `content[].text`. `null` when the error originates in Prism (not an upstream sensor). |

### `_meta` field

The `_meta` object in `structuredContent` MUST include `trust_level: "internal"` to indicate that error data is Prism-generated (not upstream sensor data).

### 429 rate-limit wiring — `retry_after_seconds` source and mapping contract

**`PrismError::SensorRateLimited` shape (canonical — see `crates/prism-core/src/error.rs`):**

```rust
SensorRateLimited { sensor: String, retry_after_ms: u64 }
```

The field is named `sensor` (NOT `sensor_id`) and `retry_after_ms` is a REQUIRED `u64` (NOT `Option<u64>`). A rate-limit error ALWAYS carries a retry value — there is no "unknown retry delay" path for this variant.

**`retry_after_seconds` population rules:**

| Error variant | `retry_after_seconds` in JSON |
|---------------|-------------------------------|
| `PrismError::SensorRateLimited { retry_after_ms, .. }` | `retry_after_ms / 1000` (always a non-null `u64`) |
| All other variants (non-rate-limit, non-retryable) | `null` (present-as-null, never absent) |

The `null` value for non-rate-limit errors is what exercises the "null-not-absent" invariant — `SensorRateLimited` always produces a numeric value, never `null`.

**`to_error_data_with_retry` helper contract:**

The helper function (or equivalent inline mapping in `error_response.rs`) MUST:
- Return `Some(retry_after_ms / 1000)` when the error is `PrismError::SensorRateLimited { retry_after_ms, .. }` — binding `retry_after_ms` directly (it is a plain `u64`, no `Option` unwrap needed).
- Return `None` for all other `PrismError` variants.
- The JSON serializer renders `Some(n)` as a `u64` JSON number and `None` as explicit JSON `null` (not field omission).

**Implementer note for S-5.02:** The `error_mapping.rs` currently maps `PrismError::SensorRateLimited` to opaque `-32000 INTERNAL_ERROR`. The implementer must:
- Add an explicit arm for `PrismError::SensorRateLimited { sensor, retry_after_ms }` in `map_prism_error` (bind both fields; `sensor` is used for the error `source` field, `retry_after_ms` is divided by 1000 for `retry_after_seconds`).
- The `error_response.rs` module (to be created) must accept `retry_after_seconds: Option<u64>` (from the helper above) and serialize `None` as JSON `null` — the field MUST NOT be omitted.
- For non-`SensorRateLimited` variants, pass `retry_after_seconds: None` → serializes as `null`.

### Category decision rule — canonical mapping per error origin

The `category` field communicates the ERROR ORIGIN and correct LLM-agent response strategy. Nine legal values:

| Category | When to use | LLM-agent strategy | Example PrismError variants |
|----------|-------------|--------------------|-----------------------------|
| `"transient"` | Retryable regardless of origin (rate limit, query timeout) | Retry after `retry_after_seconds` (if non-null) | `SensorRateLimited`, `QueryTimeout` |
| `"authentication"` | Credential invalid or identity validation failure | Re-authenticate; check credential_ref | `AuthTokenExpired`, `AuthTokenInvalid`, `InvalidOrgSlug`, `InvalidAnalystId`, `InvalidClientId` |
| `"validation"` | Caller-supplied parameters structurally or semantically invalid | Fix the tool call parameters | `QueryParseFailed`, `McpParameterInvalid`, `QueryLimitExceeded`, `QuerySecurityLimitExceeded`, `UnknownSourceTable`, `AliasNotFound` |
| `"not_found"` | Named resource does not exist (reserved for future use; currently expressed as validation) | Verify resource name | (future: dedicated not-found variants) |
| `"permission"` | Access denied by feature flag, capability check, token lifecycle, or safety boundary | Inspect permissions; use confirmation flow | `CapabilityDenied`, `FeatureFlagEvalError`, `Unauthorized`, `TokenExpired`, `TokenAlreadyConsumed`, `McpPromptInjectionDetected`, `WriteRequiresClientId` |
| `"upstream_error"` | Genuine sensor or third-party service failure (Prism reached the sensor; the sensor failed) | Investigate sensor health; try a different sensor or time range | `SensorHttpError`, `SensorTimeout`, `SensorResponseParse`, `OcsfNormalizationFailed` and related OCSF variants |
| `"configuration"` | Prism operator configuration issue (not the API caller's problem) | Escalate to operator to fix prism.toml / sensor spec | `ConfigNotFound`, `ConfigParseFailed`, `ConfigValidationFailed`, `ConfigSnapshotStale`, `SpecNotFound`, `SpecValidationFailed` |
| `"safety"` | Safety boundary violation (injection, exfiltration, contamination) | Do not retry; report to operator | `SafetyContextContamination`, `SafetyDataExfiltration` |
| `"internal"` | Prism-side infrastructure or invariant failure — sensor was NEVER reached; Prism's own storage, I/O, or internal invariant failed | Do not retry; escalate to Prism operator for infrastructure investigation | `PrismError::Internal`, `PrismError::Io`, `StorageOpenFailed`, `StorageWriteFailed`, `StorageReadFailed`, `StorageDomainNotFound`, `StorageKeyNotFound`, `StorageLockHeld`, `StorageHealthCheckFailed`, `SchemaMismatch`, `StorageBatchFailed` |

**Critical distinction — "internal" vs "upstream_error":**

- `"upstream_error"`: Prism successfully dispatched a request to the sensor API; the sensor or network between Prism and the sensor is the fault domain.
- `"internal"`: Prism itself failed before or independent of any sensor dispatch; the fault domain is Prism's own runtime (disk, memory, RocksDB, invariant violation). Telling an LLM agent that an `Io` or `StorageWriteFailed` error is an "upstream_error" is semantically incorrect — the sensor was never involved.

**Implementer note (F-4 code follow-up required):** The current `error_mapping.rs` maps `PrismError::Internal`, `PrismError::Io`, and all `PrismError::Storage*` variants to the JSON-RPC code `-32000` with a generic message. After this BC amendment, the structured error builder (`error_response.rs`) must set `category: "internal"` for these variants and `category: "upstream_error"` only for sensor-origin failures (`SensorHttpError`, `SensorTimeout`, `SensorResponseParse`, `OcsfNormalizationFailed` and related OCSF variants). See §Implementer Code Follow-Up below.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Sensor API returns 401 | `isError: true`; `category: "authentication"`, `retryable: false`, `source: "crowdstrike_falcon_api"` | error |
| Upstream error message contains injection payload | Payload in `upstream_message` only; `content[].text` has no injection content | error + injection |
| Expired confirmation token | `code: "E-FLAG-003"`, `category: "permission"`, `retryable: false` | edge-case |
| `PrismError::Internal { reason: "invariant violated" }` | `category: "internal"`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"` | error (F-4 — internal infra) |
| `PrismError::Io(std::io::Error)` | `category: "internal"`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"` | error (F-4 — internal infra) |
| `PrismError::StorageWriteFailed { .. }` | `category: "internal"`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"` | error (F-4 — internal infra) |
| `PrismError::SensorHttpError { .. }` (sensor returns 503) | `category: "upstream_error"`, `retryable: true`, `upstream_message: "<sensor 503 body>"` | error (upstream sensor) |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-004, DI-006 |
| L2 Edge Cases | DEC-009 |
| Priority | P0 |

## Implementer Code Follow-Up (F-4)

**This section records a required implementer action resulting from this BC amendment (F-4 from PR #191 review). The orchestrator must route this to the implementer after the BC amendment is committed.**

File to change: `crates/prism-mcp/src/error_mapping.rs` (and the structured error builder in `error_response.rs`).

Required mapping changes — add `category` field population (currently the code emits no `category` into structured content for most arms; the structured error builder must read category from the mapping output):

| PrismError variants | Old category (incorrect) | New category (correct) | Rationale |
|--------------------|--------------------------|------------------------|-----------|
| `Internal { .. }` | `"upstream_error"` (fallback) | `"internal"` | Prism invariant failure; sensor not reached |
| `Io(_)` | `"upstream_error"` (fallback) | `"internal"` | Prism I/O failure; sensor not reached |
| `StorageOpenFailed { .. }`, `StorageWriteFailed { .. }`, `StorageReadFailed { .. }`, `StorageDomainNotFound { .. }`, `StorageKeyNotFound { .. }`, `StorageLockHeld { .. }`, `StorageHealthCheckFailed { .. }`, `SchemaMismatch { .. }`, `StorageBatchFailed { .. }` | `"upstream_error"` (fallback) | `"internal"` | RocksDB / storage layer failure; sensor not reached |
| `SensorHttpError { .. }`, `SensorTimeout { .. }`, `SensorResponseParse { .. }` | `"upstream_error"` (correct, no change) | `"upstream_error"` | Sensor boundary — Prism dispatched to sensor and sensor failed |
| OCSF normalization variants (`OcsfField*`, `OcsfProtobuf*`, `OcsfNormalizationFailed`, etc.) | `"upstream_error"` (correct, no change) | `"upstream_error"` | Normalization of sensor-origin data; effectively upstream failure |

Additionally, add a test in `crates/prism-mcp/src/error_mapping.rs` `#[cfg(test)] mod tests` that asserts:
- `map_prism_error(PrismError::Internal { reason: "test".into() })` produces category `"internal"` (not `"upstream_error"`)
- `map_prism_error(PrismError::Io(std::io::Error::new(std::io::ErrorKind::Other, "disk")))` produces category `"internal"`
- `map_prism_error(PrismError::StorageWriteFailed { .. })` produces category `"internal"`
- `map_prism_error(PrismError::SensorHttpError { .. })` produces category `"upstream_error"` (regression guard)

Note: the `category` field is in the structured error builder, not in `map_prism_error` directly. The implementer must thread category derivation through `error_response.rs` using the same variant match, or extract category as a second return value alongside `(code, message)` from `map_prism_error`.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | PR-191-F4-adjudication | 2026-06-16 | product-owner | Semantic fix (F-4, PR #191): added `"internal"` as 9th legal category value for Prism-side infrastructure/invariant failures (`PrismError::Internal`, `Io`, `Storage*`). "upstream_error" is now reserved for genuine sensor/third-party boundary failures only. Added canonical category decision rule table (9 rows, LLM-agent strategy column). Added 4 test vectors covering Internal/Io/Storage→internal and SensorHttpError→upstream_error. Added §Implementer Code Follow-Up (F-4) specifying exact mapping changes required in `error_mapping.rs` / `error_response.rs`. This is a semantic contract change (enum grows by one value); existing implementations emitting "upstream_error" for Prism-internal failures must be updated. |
| 1.6 | S-5.02-red-gate-clarification | 2026-06-14 | product-owner | Contract clarification (not a semantic change): aligned `retry_after_seconds` wiring with actual `PrismError::SensorRateLimited` shape (`{ sensor: String, retry_after_ms: u64 }` — required u64, not `Option<u64>`; field `sensor` not `sensor_id`). Replaced v1.5's `Option<u64>` / `None` framing with explicit table: SensorRateLimited ALWAYS produces non-null `retry_after_ms / 1000`; all other variants produce JSON `null`. Added `to_error_data_with_retry` helper contract. Updated `retry_after_seconds` field-spec row to distinguish rate-limit vs non-rate-limit cases. External JSON contract unchanged (field always present, null-not-absent). No code change required — the code shape was already correct; the BC was the imprecise artifact. |
| 1.5 | S-5.02-pre-TDD-reconciliation | 2026-06-14 | product-owner | R3 reconciliation: Locked canonical ToolError shape as NESTED (not flat) with complete 9-field `structuredContent.error` object specification. Fields: code, message, category, retryable, retry_after_seconds (always-present, null-not-absent), suggestion, source, original_params_valid, upstream_message. Added `_meta.trust_level: "internal"` spec. Added 429 wiring note: SensorError::RateLimited → PrismError::SensorRateLimited { retry_after_ms } → retry_after_seconds (ms/1000); implementer must add explicit SensorRateLimited arm in map_prism_error and wire retry_after_ms through error_response.rs. Story-spec flat-7-field framing superseded by this BC's nested shape. |
| 1.4 | MCP-cascade-pass-1 | 2026-06-10 | product-owner | Version bump at review cycle; no content change from 1.3. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
