---
document_type: behavioral-contract
level: L3
version: "1.10"
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
modified: "2026-07-13"  # v1.10: Clarify §Internal-redacted split — UNIVERSAL message rule (ALL -32000 arms get "Internal error"), EXHAUSTIVE catch-all class enumeration, CLASS-DIFFERENTIATED suggestion rule (F-MCPNULL-P6-OBS-003; DEFECT-MCP-ROWSHAPE-NULLS-001 pass-6). Prior v1.9: Explicit postcondition for message/suggestion split (F-MCPNULL-P4-OBS-001 [process-gap], [H8b]).
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

### Internal-redacted error message/suggestion split (INTERNAL_ERROR class)

All `PrismError` variants that route through `map_prism_error` to MCP JSON-RPC code `-32000 INTERNAL_ERROR` are governed by **two rules — one universal, one class-differentiated**:

**Rule 1 — `message` (UNIVERSAL, ALL -32000 arms):** `message` MUST be the terse redacted form `"Internal error"` — NO audit-log pointer embedded in `message`, NO upstream detail interpolation (DI-006). The `message` field reaches the LLM-agent caller and MUST NOT leak operator-level detail. This rule applies to EVERY variant in `prism_error_to_structured_call_result` that routes to `-32000`, including variants with dedicated VariantMeta arms (authentication variants such as `AuthTokenExpired`/`AuthTokenInvalid`, watchdog variants, sensor adapter variants, config/spec variants, etc.).

**Rule 2 — `suggestion` (CLASS-DIFFERENTIATED):** The suggestion content differs by which VariantMeta arm a variant lands in:

- **Prism-side infrastructure arm** — `suggestion` = `"Prism infrastructure failure. Contact Prism operator; see audit log for details."`: Variants grouped in the shared infrastructure VariantMeta arm: `PrismError::Internal`, `Io`, and all `Storage*` variants (`StorageOpenFailed`, `StorageWriteFailed`, `StorageReadFailed`, `StorageDomainNotFound`, `StorageKeyNotFound`, `StorageLockHeld`, `StorageHealthCheckFailed`, `SchemaMismatch`, `StorageBatchFailed`). This enumeration is **EXHAUSTIVE** for this suggestion string — only these variants land in this arm.

- **`_` non-exhaustive catch-all arm** — `suggestion` = `"See audit log for details."`: All `#[non_exhaustive]` variants without a dedicated VariantMeta arm fall here. This includes OCSF normalization variants, `QueryExecutionFailed`, `WritePartialFailure`, scheduler/detection/case/safety variants, `Infusion`, `Plugin`, IOC variants, credential variants, and any future `PrismError` additions. The `_` catch-all arm in `prism_error_to_structured_call_result` is the **EXHAUSTIVE** definition of which variants receive exactly `"See audit log for details."` as suggestion.

- **Dedicated VariantMeta arm class** — category-appropriate variant-specific suggestions: Other -32000-returning variants that have their own named arms in `prism_error_to_structured_call_result` carry category-appropriate suggestions per the §Category decision rule. The `message = "Internal error"` Rule 1 still applies, but `suggestion` is NOT the generic audit-log pointer phrase. Canonical examples from the shipped `error_mapping.rs` VariantMeta arms: `AuthTokenExpired` → `"The auth token has expired. Re-authenticate and obtain a fresh token."` (`"authentication"` category); `AuthTokenInvalid` → `"The auth token is invalid. Re-authenticate and obtain a valid token."` (`"authentication"` category); `WatchdogKilled`/`WatchdogHeartbeatMissed`/`WatchdogRestartLimitExceeded` → `"Prism process supervision failure (memory or watchdog). Contact Prism operator; see audit log for details."` (`"internal"` category); sensor adapter variants (`SensorRateLimited`, `SensorHttpError`, `SensorTimeout`, `SensorResponseParse`) and config/spec variants also carry dedicated-arm suggestions appropriate to their category — see `prism_error_to_structured_call_result` VariantMeta arms in `error_mapping.rs` for authoritative suggestion strings.

This is the **message/suggestion split** cited in error-taxonomy v2.41 rows for E-INT-001, E-AUTH-010, E-AUTH-011, E-QUERY-034, E-WATCH-002, and the §INT narrative. This section codifies the split as an explicit postcondition; previously it was only implicit in the field description table above. Ratifying authority: DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] + error-taxonomy v2.40. Amended at v1.10 to clarify the UNIVERSAL message rule, the EXHAUSTIVE catch-all class enumeration, and the class-differentiated suggestion rule (F-MCPNULL-P6-OBS-003 2026-07-13).

| Field | Rule |
|-------|------|
| `message` | `"Internal error"` — verbatim terse form; MUST NOT contain audit-log pointer or upstream detail. **UNIVERSAL:** applies to ALL -32000 arms without exception. |
| `suggestion` | **CLASS-DIFFERENTIATED:** infrastructure arm → `"Prism infrastructure failure. Contact Prism operator; see audit log for details."`; `_` catch-all arm → `"See audit log for details."`; dedicated-arm variants → category-appropriate variant-specific suggestion per §Category rule (e.g., `AuthTokenExpired` → `"The auth token has expired. Re-authenticate and obtain a fresh token."`). |

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
| `"permission"` | Access denied by feature flag, capability check, token lifecycle, safety boundary, or org-scoping isolation. `original_params_valid: true` for all permission denials — the request parameters were structurally correct; access was refused. | Inspect permissions; use confirmation flow; for org-scoping errors, verify the sensor is registered under the target org | `CapabilityDenied`, `FeatureFlagEvalError`, `Unauthorized`, `TokenExpired`, `TokenAlreadyConsumed`, `McpPromptInjectionDetected`, `WriteRequiresClientId`, **`SensorNotRegisteredForOrg` (E-QUERY-032)** |
| `"upstream_error"` | Genuine sensor or third-party service failure (Prism reached the sensor; the sensor failed) | Investigate sensor health; try a different sensor or time range | `SensorHttpError`, `SensorTimeout`, `SensorResponseParse`, `OcsfNormalizationFailed` and related OCSF variants |
| `"configuration"` | Prism operator configuration issue (not the API caller's problem) | Escalate to operator to fix prism.toml / sensor spec | `ConfigNotFound`, `ConfigParseFailed`, `ConfigValidationFailed`, `ConfigSnapshotStale`, `SpecNotFound`, `SpecValidationFailed` |
| `"safety"` | Safety boundary violation (injection, exfiltration, contamination) | Do not retry; report to operator | `SafetyContextContamination`, `SafetyDataExfiltration` |
| `"internal"` | Prism-side infrastructure or invariant failure — sensor was NEVER reached; Prism's own storage, I/O, or internal invariant failed. Also covers watchdog-triggered query termination (see Watchdog note below). | Do not retry; escalate to Prism operator for infrastructure investigation | `PrismError::Internal`, `PrismError::Io`, `StorageOpenFailed`, `StorageWriteFailed`, `StorageReadFailed`, `StorageDomainNotFound`, `StorageKeyNotFound`, `StorageLockHeld`, `StorageHealthCheckFailed`, `SchemaMismatch`, `StorageBatchFailed`, **`WatchdogKilled`, `WatchdogHeartbeatMissed`, `WatchdogRestartLimitExceeded`** |

**Critical distinction — "internal" vs "upstream_error":**

- `"upstream_error"`: Prism successfully dispatched a request to the sensor API; the sensor or network between Prism and the sensor is the fault domain.
- `"internal"`: Prism itself failed before or independent of any sensor dispatch; the fault domain is Prism's own runtime (disk, memory, RocksDB, invariant violation, or watchdog termination). Telling an LLM agent that an `Io` or `StorageWriteFailed` error is an "upstream_error" is semantically incorrect — the sensor was never involved.

**Pinned adjudication — OBS-1 (PR #191): E-QUERY-032 / SensorNotRegisteredForOrg category = "permission", original_params_valid = true:**

The adversary flagged the current implementation's mapping of `SensorNotRegisteredForOrg` to `category: "validation" / original_params_valid: false` as semantically imprecise. This BC pins the production-grade-correct mapping:

- **Category: `"permission"`** — Cross-org sensor isolation is a scoping/permission denial, not a parameter validation failure. The analyst's org slug and sensor name are both structurally valid. The sensor exists and is registered — just not under the requesting org. The fault domain is the org-scoping permission boundary, not the caller's parameter shape. Parallel to `CapabilityDenied`: the capability (sensor access for that org) was denied. The LLM-agent strategy for "permission" errors — "inspect permissions; verify sensor is registered under the target org" — is exactly right for this error. The "validation" strategy — "fix the tool call parameters" — is wrong and misleading, because the parameters *are* correct.
- **original_params_valid: true** — The org slug and sensor name were syntactically and structurally valid. The request was denied at the scoping layer, not at parameter validation. Parallel to `ClientNotFound` (configuration/original_params_valid:true) and all other permission variants. "validation"/false would tell the LLM agent to fix its parameters, when the correct action is to check sensor registration for that org.
- **No change to the error code (E-QUERY-032) or JSON-RPC code (-32602 INVALID_PARAMS)**: The JSON-RPC code remains -32602 and the error code remains E-QUERY-032 as defined in error-taxonomy.md and enforced by BC-3.2.001 postcondition 5. The BC-2.10.007 `category` field and `original_params_valid` field are STRUCTURED CONTENT fields, not JSON-RPC protocol fields — they can differ from the JSON-RPC code. (-32602 INVALID_PARAMS is the closest JSON-RPC code for a scoping denial; -32002 FORBIDDEN would be semantically closer but the taxonomy and BC-3.2.001 have already fixed the JSON-RPC code at -32602. The `category` field is the primary semantic signal for LLM agents.)

**This changes the current implementation**, which maps `SensorNotRegisteredForOrg` to `category: "validation" / original_params_valid: false` (grouped with `QueryParseFailed`, `McpParameterInvalid`, etc. in the validation arm of `prism_error_to_structured_call_result`). See §Implementer Code Follow-Up (OBS-1) below.

**Pinned adjudication — OBS-2 (PR #191): Watchdog* variants — reachability and category:**

The adversary flagged that `WatchdogKilled`, `WatchdogHeartbeatMissed`, and `WatchdogRestartLimitExceeded` fall to the catch-all `"upstream_error"` in `prism_error_to_structured_call_result`.

**Reachability verdict: WatchdogKilled IS reachable on user-visible MCP tool paths.** Verified by reading `crates/prism-storage/src/watchdog.rs::check_query()` (the production emission site) and the MCP tool dispatch chain. The background watchdog monitor cancels query `CancellationToken` instances; the `?` propagation in the query execution path surfaces this as `PrismError::WatchdogKilled` to the MCP tool handler, which routes it through `prism_error_to_structured_call_result`. A tool call that triggers memory-watchdog termination will produce a user-visible structured error with this variant.

**WatchdogHeartbeatMissed and WatchdogRestartLimitExceeded:** No production emission sites found in the worktree outside of `prism-storage/src/watchdog.rs` itself (the spawn_monitor task) and tests. These are process-supervision signals that currently do not propagate into the MCP tool response path. However, since they share the same E-WATCH-* taxonomy and the same "Prism-side process supervision failure" fault domain, they are categorized identically to `WatchdogKilled` for forward compatibility.

**Category decision: `"internal"`, original_params_valid: true.** Watchdog termination is a Prism-side process supervision failure — the query was killed because Prism's own memory budget was exceeded, not because the sensor failed. Mapping these to `"upstream_error"` is semantically wrong: it directs the LLM agent to investigate sensor health when the problem is Prism's own process memory pressure. `"internal"` correctly signals "Prism infrastructure failure; escalate to operator." The catch-all `"upstream_error"` is not appropriate here because the fault domain is known and Prism-internal.

**This changes the current implementation** (catch-all `"upstream_error"` for Watchdog*). The existing test `test_CRIT_B_catch_all_category_is_upstream_error` uses `WatchdogKilled` to exercise the catch-all; after the implementer adds an explicit arm for Watchdog* variants, that test's assertion (`"upstream_error"`) must be updated to `"internal"`, and the catch-all test should use a genuinely unmapped future variant (or be repurposed). See §Implementer Code Follow-Up (OBS-2) below.

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
| `PrismError::SensorNotRegisteredForOrg { sensor_id: "claroty", org_slug: "demo-org-a" }` | `category: "permission"`, `original_params_valid: true`, `retryable: false`, `source: "prism_mcp"` | error (OBS-1 — org-scoping permission denial) |
| `PrismError::WatchdogKilled { budget_bytes: 512_000_000 }` | `category: "internal"`, `original_params_valid: true`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"` | error (OBS-2 — watchdog process supervision) |

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

## Implementer Code Follow-Up (F-4, OBS-1, OBS-2)

**This section records required implementer actions resulting from BC amendments. The orchestrator must route these to the implementer after the BC amendment is committed.**

File to change: `crates/prism-mcp/src/error_mapping.rs` (specifically `prism_error_to_structured_call_result` and its test module).

### F-4 (status: DONE in S-5.02 feature branch)

The F-4 mapping changes (Internal/Io/Storage* → "internal"; SensorHttpError/SensorTimeout/SensorResponseParse/OCSF* → "upstream_error") are already implemented in the S-5.02 feature branch. The category decision rule table in §Category above now documents this as the canonical mapping.

| PrismError variants | Old category (incorrect) | New category (correct) | Rationale |
|--------------------|--------------------------|------------------------|-----------|
| `Internal { .. }` | `"upstream_error"` (fallback) | `"internal"` | Prism invariant failure; sensor not reached |
| `Io(_)` | `"upstream_error"` (fallback) | `"internal"` | Prism I/O failure; sensor not reached |
| `StorageOpenFailed { .. }`, `StorageWriteFailed { .. }`, `StorageReadFailed { .. }`, `StorageDomainNotFound { .. }`, `StorageKeyNotFound { .. }`, `StorageLockHeld { .. }`, `StorageHealthCheckFailed { .. }`, `SchemaMismatch { .. }`, `StorageBatchFailed { .. }` | `"upstream_error"` (fallback) | `"internal"` | RocksDB / storage layer failure; sensor not reached |
| `SensorHttpError { .. }`, `SensorTimeout { .. }`, `SensorResponseParse { .. }` | `"upstream_error"` (correct, no change) | `"upstream_error"` | Sensor boundary — Prism dispatched to sensor and sensor failed |
| OCSF normalization variants (`OcsfField*`, `OcsfProtobuf*`, `OcsfNormalizationFailed`, etc.) | `"upstream_error"` (correct, no change) | `"upstream_error"` | Normalization of sensor-origin data; effectively upstream failure |

### OBS-1 (status: REQUIRED — implementer follow-up needed)

**Change:** In `prism_error_to_structured_call_result`, move `PrismError::SensorNotRegisteredForOrg { .. }` OUT of the "validation" arm (lines 846-872 in the S-5.02 branch) and into the "permission" arm (lines 920-942), with `original_params_valid: true`.

The "validation" arm currently has:
```rust
PrismError::SensorNotRegisteredForOrg { .. }
```
grouped with `QueryParseFailed`, `McpParameterInvalid`, etc., and sets `original_params_valid: false`.

After this change, `SensorNotRegisteredForOrg` must be added to the permission arm:
```rust
PrismError::CapabilityDenied { .. }
| PrismError::FeatureFlagEvalError { .. }
// ... (existing permission variants)
| PrismError::SensorNotRegisteredForOrg { .. }  // ← ADD HERE
```
with `original_params_valid: true` (already set by the permission arm).

The suggestion text in the permission arm should be updated (or a dedicated sub-arm created) to include org-scoping guidance: "Check sensor registration for the target org. Verify the sensor is configured under the requested org slug in prism.toml."

**JSON-RPC code (-32602) and error code (E-QUERY-032) are UNCHANGED** — only the structured content `category` and `original_params_valid` fields change.

**Tests to add:** A new test `test_BC_2_10_007_sensor_not_registered_for_org_category_is_permission` asserting:
- `prism_error_to_structured_call_result(PrismError::SensorNotRegisteredForOrg { sensor_id: "claroty", org_slug: "demo-org-a" })` produces `category: "permission"`, `original_params_valid: true`, `retryable: false`.

**Tests to update:** If any existing test asserts `category: "validation"` for `SensorNotRegisteredForOrg`, update to assert `category: "permission"`.

### OBS-2 (status: REQUIRED — implementer follow-up needed)

**Change:** In `prism_error_to_structured_call_result`, add an explicit arm for `WatchdogKilled`, `WatchdogHeartbeatMissed`, and `WatchdogRestartLimitExceeded` producing `category: "internal"`, `original_params_valid: true`, `retryable: false`. These must NOT fall to the catch-all.

```rust
// ── Process-supervision watchdog failures → category "internal" ─────────
// BC-2.10.007 v1.8 §OBS-2: Watchdog variants are Prism-side process supervision
// failures. WatchdogKilled is reachable on user-visible MCP tool paths via the
// query execution path (prism-storage::watchdog::check_query → ? propagation →
// tool handler → prism_error_to_structured_call_result). Category "internal"
// is correct: the fault domain is Prism's own memory budget, not a sensor failure.
// Catch-all "upstream_error" was semantically wrong — it directed LLM agents to
// investigate sensor health for a Prism-internal resource constraint.
PrismError::WatchdogKilled { .. }
| PrismError::WatchdogHeartbeatMissed { .. }
| PrismError::WatchdogRestartLimitExceeded { .. } => VariantMeta {
    category: "internal",
    suggestion: "Prism process supervision failure (memory or watchdog). Contact Prism operator; see audit log for details.",
    retryable: false,
    retry_after_seconds: None,
    original_params_valid: true,
    source_override: None,
    upstream_message: None,
    ec_code_override: None,
},
```

**Tests to add:**
- `test_BC_2_10_007_watchdog_killed_category_is_internal`: asserts `WatchdogKilled { budget_bytes: 512_000_000 }` → `category: "internal"`, `retryable: false`, `upstream_message: null`.

**Tests to update:**
- `test_CRIT_B_catch_all_category_is_upstream_error` currently uses `WatchdogKilled` to exercise the catch-all path and asserts `category: "upstream_error"`. After the explicit arm is added, this test will break. Update it to: (a) use a genuinely unmapped variant (the `PrismError::Infusion(_)` catch-all delegating variant works, or any newly-added future variant if available), and (b) change the assertion to confirm the new explicit arm maps `WatchdogKilled` to `"internal"` (or retire the WatchdogKilled assertion from the catch-all test and add the new dedicated test above instead).

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.10 | DEFECT-MCP-ROWSHAPE-NULLS-001-P6-F-MCPNULL-P6-OBS-003 | 2026-07-13 | product-owner | **F-MCPNULL-P6-OBS-003 closure — §Internal-redacted split clarified: UNIVERSAL message rule + EXHAUSTIVE catch-all enumeration + CLASS-DIFFERENTIATED suggestion rule.** The v1.9 §Internal-redacted error message/suggestion split postcondition parenthetical "(the catch-all class: PrismError::Internal, Io, all Storage* variants, WatchdogKilled, WatchdogHeartbeatMissed, WatchdogRestartLimitExceeded, and any future catch-all arms)" was ambiguous: it listed ALL variants the rule applied to, implying all received `"See audit log for details."` as suggestion — but WatchdogKilled/HeartbeatMissed/RestartLimitExceeded have dedicated VariantMeta arms with `"Prism process supervision failure (memory or watchdog). Contact Prism operator; see audit log for details."` and AuthTokenExpired/AuthTokenInvalid also have dedicated authentication-category arms with re-auth suggestions. **Three clarifications added:** (1) **UNIVERSAL message rule** — `message = "Internal error"` applies to ALL -32000 arms including those with dedicated VariantMeta arms (no exception); (2) **EXHAUSTIVE catch-all enumeration** — the §Internal-redacted section now defines three named classes: (a) Prism-side infrastructure arm (Internal/Io/Storage*) → `"Prism infrastructure failure. Contact Prism operator; see audit log for details."` [EXHAUSTIVE]; (b) `_` non-exhaustive catch-all arm → `"See audit log for details."` [EXHAUSTIVE by definition]; (c) dedicated VariantMeta arm class → category-appropriate variant-specific suggestions per §Category rule; (3) **canonical examples** from shipped `error_mapping.rs` VariantMeta arms included. Updated citation to error-taxonomy v2.41. Companion: error-taxonomy v2.41 (F-MCPNULL-P6-MED-001 — 4 rows corrected). No semantic code change — the code already implements this split correctly; this amendment closes the spec-ambiguity gap. |
| 1.9 | DEFECT-MCP-ROWSHAPE-NULLS-001-P4-OBS-001 | 2026-07-13 | product-owner | Explicit postcondition for internal-redacted message/suggestion split (F-MCPNULL-P4-OBS-001 [process-gap]). error-taxonomy v2.40 rows for E-INT-001, E-AUTH-010, E-AUTH-011, E-QUERY-034, E-WATCH-002, and the §INT narrative all cite "BC-2.10.007 message/suggestion split" but the BC body never codified the split as an explicit postcondition — the rule was only implicit in the `message`/`suggestion` field description table. Added §Internal-redacted error message/suggestion split postcondition section: for all INTERNAL_ERROR-class variants (map_prism_error catch-all: Internal, Io, Storage*, WatchdogKilled, WatchdogHeartbeatMissed, WatchdogRestartLimitExceeded), `message` MUST be `"Internal error"` (terse, redacted; no audit-log pointer, no upstream detail per DI-006); `suggestion` MUST carry the audit-log pointer (`"See audit log for details."` or variant-specific). Ratifying authority: DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] + error-taxonomy v2.40. No semantic change to code behavior — the code already implements this split; this amendment closes the spec gap. |
| 1.8 | PR-191-OBS1-OBS2-adjudication | 2026-06-16 | product-owner | Semantic pin (OBS-1, OBS-2, PR #191): (1) OBS-1 — pinned E-QUERY-032/SensorNotRegisteredForOrg to `category: "permission"` / `original_params_valid: true`. Previous impl used `"validation"` / `false`, which was semantically wrong: cross-org sensor access is a scoping/permission denial, not a parameter format error. The org slug and sensor name are structurally valid; access was refused at the org-scoping boundary. Updated §Category decision rule "permission" row to include SensorNotRegisteredForOrg explicitly. Added OBS-1 canonical test vector. Added §Implementer Code Follow-Up (OBS-1) with exact match-arm change and new test. (2) OBS-2 — pinned WatchdogKilled/WatchdogHeartbeatMissed/WatchdogRestartLimitExceeded to `category: "internal"` / `original_params_valid: true`. Verified reachability: WatchdogKilled IS reachable on user-visible MCP tool paths via query execution → watchdog::check_query() → ? propagation → tool handler → prism_error_to_structured_call_result. Category "internal" is correct (Prism-side process supervision fault, not sensor failure). Updated §Category "internal" row to enumerate Watchdog* variants. Added OBS-2 canonical test vector. Added §Implementer Code Follow-Up (OBS-2) with explicit arm code and test_CRIT_B_catch_all_category_is_upstream_error update instructions. Both OBS-1 and OBS-2 require implementer follow-up in the S-5.02 feature branch. |
| 1.7 | PR-191-F4-adjudication | 2026-06-16 | product-owner | Semantic fix (F-4, PR #191): added `"internal"` as 9th legal category value for Prism-side infrastructure/invariant failures (`PrismError::Internal`, `Io`, `Storage*`). "upstream_error" is now reserved for genuine sensor/third-party boundary failures only. Added canonical category decision rule table (9 rows, LLM-agent strategy column). Added 4 test vectors covering Internal/Io/Storage→internal and SensorHttpError→upstream_error. Added §Implementer Code Follow-Up (F-4) specifying exact mapping changes required in `error_mapping.rs` / `error_response.rs`. This is a semantic contract change (enum grows by one value); existing implementations emitting "upstream_error" for Prism-internal failures must be updated. |
| 1.6 | S-5.02-red-gate-clarification | 2026-06-14 | product-owner | Contract clarification (not a semantic change): aligned `retry_after_seconds` wiring with actual `PrismError::SensorRateLimited` shape (`{ sensor: String, retry_after_ms: u64 }` — required u64, not `Option<u64>`; field `sensor` not `sensor_id`). Replaced v1.5's `Option<u64>` / `None` framing with explicit table: SensorRateLimited ALWAYS produces non-null `retry_after_ms / 1000`; all other variants produce JSON `null`. Added `to_error_data_with_retry` helper contract. Updated `retry_after_seconds` field-spec row to distinguish rate-limit vs non-rate-limit cases. External JSON contract unchanged (field always present, null-not-absent). No code change required — the code shape was already correct; the BC was the imprecise artifact. |
| 1.5 | S-5.02-pre-TDD-reconciliation | 2026-06-14 | product-owner | R3 reconciliation: Locked canonical ToolError shape as NESTED (not flat) with complete 9-field `structuredContent.error` object specification. Fields: code, message, category, retryable, retry_after_seconds (always-present, null-not-absent), suggestion, source, original_params_valid, upstream_message. Added `_meta.trust_level: "internal"` spec. Added 429 wiring note: SensorError::RateLimited → PrismError::SensorRateLimited { retry_after_ms } → retry_after_seconds (ms/1000); implementer must add explicit SensorRateLimited arm in map_prism_error and wire retry_after_ms through error_response.rs. Story-spec flat-7-field framing superseded by this BC's nested shape. |
| 1.4 | MCP-cascade-pass-1 | 2026-06-10 | product-owner | Version bump at review cycle; no content change from 1.3. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
