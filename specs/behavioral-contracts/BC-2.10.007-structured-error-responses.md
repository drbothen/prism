---
document_type: behavioral-contract
level: L3
version: "1.15"
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
modified: "2026-07-14"  # v1.15: QueryDenylisted vector row corrected from nonexistent `query_hash` field to actual struct `{ failure_count, reason, expiry_ts }` (F-MCPRS-PRL6-MED-001). POL-29 full vector sweep: all other rows verified correct. RETRYABLE-503 adjudication added to §Implementer Code Follow-Up (spec correct, code fix needed). v1.14: §MED-001 safety arm added (SafetyContextContamination/SafetyDataExfiltration → category "safety", ec_code_override E-SAFETY-001/E-SAFETY-002, original_params_valid: true; map_prism_error returns "Internal error" per Rule 1 — ec_code_override via nested match required). §LOW-001: 3 missing LOW-002 test vectors added (QueryPlanFailed, QueryMaterializationLimitExceeded, QueryVirtualFieldFailed) + 2 safety vectors + catch-all regression guard. §LOW-002 tests-to-add extended (QueryMaterializationLimitExceeded, QueryVirtualFieldFailed). Taxonomy v2.47: E-SAFETY-001/002 descriptions corrected to match variant semantics. v1.13: §LOW-002 arm code corrected — per-variant ec_code_override via nested match (implementer-discovered: map_prism_error returns "Internal error" for all 6; Rule 1 redaction prevents code inference; v1.12 claim "ec_code_override: None" was incorrect). v1.12: 6 query-engine variants moved to dedicated 'internal' arm (F-MCPRS-PRL2-LOW-002). v1.11: Rule 1 AuditPersistenceFailed carve-out (MED-001) + McpSerializationError "internal" (OBS-002).
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

**Rule 1 — `message` (UNIVERSAL with one exhaustive exception):** For all -32000 variants, `message` MUST be the terse redacted form `"Internal error"` — NO audit-log pointer embedded in `message`, NO upstream detail interpolation (DI-006). The `message` field reaches the LLM-agent caller and MUST NOT leak operator-level detail. This rule applies to every variant in `prism_error_to_structured_call_result` that routes to `-32000`, including variants with dedicated VariantMeta arms (authentication variants such as `AuthTokenExpired`/`AuthTokenInvalid`, watchdog variants, sensor adapter variants, config/spec variants, `McpSerializationError`, etc.).

**Exception (EXHAUSTIVE — one variant only): `PrismError::AuditPersistenceFailed`.** This variant emits its taxonomy-verbatim Display as `message` — `"E-AUDIT-001: Audit emission failed; write operation blocked. Retry the operation. If the error persists, check tracing subscriber health."` — instead of the generic `"Internal error"`. Rationale: the message carries no sensitive detail (no credentials, no raw sensor text, no stack traces); the agent caller needs the error code and retry guidance to act on this transient, retryable fail-closed condition. Authority: `map_prism_error` `AuditPersistenceFailed` arm comment in `error_mapping.rs`; BC-2.05.001 DEC-014 fail-closed contract. **This exception list is EXHAUSTIVE** — no other -32000 variant emits a non-redacted `message`.

**Rule 2 — `suggestion` (CLASS-DIFFERENTIATED):** The suggestion content differs by which VariantMeta arm a variant lands in:

- **Prism-side infrastructure arm** — `suggestion` = `"Prism infrastructure failure. Contact Prism operator; see audit log for details."`: Variants grouped in the shared infrastructure VariantMeta arm: `PrismError::Internal`, `Io`, and all `Storage*` variants (`StorageOpenFailed`, `StorageWriteFailed`, `StorageReadFailed`, `StorageDomainNotFound`, `StorageKeyNotFound`, `StorageLockHeld`, `StorageHealthCheckFailed`, `SchemaMismatch`, `StorageBatchFailed`). This enumeration is **EXHAUSTIVE** for this suggestion string — only these variants land in this arm.

- **`_` non-exhaustive catch-all arm** — `suggestion` = `"See audit log for details."`: All `#[non_exhaustive]` variants without a dedicated VariantMeta arm fall here. This includes OCSF normalization variants, `WritePartialFailure`, scheduler/detection/case variants, `Infusion`, `Plugin`, IOC variants, credential variants, and any future `PrismError` additions. NOTE: `QueryExecutionFailed`, `QueryPlanFailed`, `QueryMaterializationLimitExceeded`, `QueryMemoryBudgetExceeded`, `QueryVirtualFieldFailed`, and `QueryDenylisted` are **EXCLUDED** from the catch-all from v1.12 onward — they have a dedicated query engine arm (see below). **`SafetyContextContamination` and `SafetyDataExfiltration` are EXCLUDED from the catch-all from v1.14 onward — they have a dedicated safety arm (see §MED-001).** The `_` catch-all arm in `prism_error_to_structured_call_result` is the **EXHAUSTIVE** definition of which variants receive exactly `"See audit log for details."` as suggestion.

- **Query engine arm** — `suggestion` = `"Prism query engine failure. Contact Prism operator; see audit log for details."`: Six DataFusion/query-engine variants that are Prism-internal failures (sensor dispatch has already completed or was never relevant): `PrismError::QueryPlanFailed` (E-QUERY-002), `QueryExecutionFailed` (E-QUERY-034), `QueryMaterializationLimitExceeded` (E-QUERY-005), `QueryMemoryBudgetExceeded` (E-WATCHDOG-001), `QueryVirtualFieldFailed` (E-QUERY-010), `QueryDenylisted` (E-QUERY-008). All carry `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`. Per-variant `ec_code_override` REQUIRED via nested match (see §Implementer Code Follow-Up §LOW-002). Reason: `map_prism_error` returns `"Internal error"` for all 6 per BC-2.10.007 Rule 1 message redaction — the code-inference logic extracts the E-code from the `map_prism_error` message string, which is `"Internal error"` (no E-prefix), not the variant Display string. Each variant's Display string DOES carry its E-QUERY-NNN / E-WATCHDOG-NNN prefix, but inference uses the redacted `map_prism_error` message, not the Display. Without explicit per-variant overrides, all 6 fall through to the `E-INT-001` fallback. This enumeration is **EXHAUSTIVE** for this suggestion string. Authority: F-MCPRS-PRL2-LOW-002 adjudication + implementer-discovered v1.13 correction, BC-2.10.007 v1.13.

- **Dedicated VariantMeta arm class** — category-appropriate variant-specific suggestions: Other -32000-returning variants that have their own named arms in `prism_error_to_structured_call_result` carry category-appropriate suggestions per the §Category decision rule. The `message = "Internal error"` Rule 1 still applies, but `suggestion` is NOT the generic audit-log pointer phrase. Canonical examples from the shipped `error_mapping.rs` VariantMeta arms: `AuthTokenExpired` → `"The auth token has expired. Re-authenticate and obtain a fresh token."` (`"authentication"` category); `AuthTokenInvalid` → `"The auth token is invalid. Re-authenticate and obtain a valid token."` (`"authentication"` category); `WatchdogKilled`/`WatchdogHeartbeatMissed`/`WatchdogRestartLimitExceeded` → `"Prism process supervision failure (memory or watchdog). Contact Prism operator; see audit log for details."` (`"internal"` category); sensor adapter variants (`SensorRateLimited`, `SensorHttpError`, `SensorTimeout`, `SensorResponseParse`) and config/spec variants also carry dedicated-arm suggestions appropriate to their category; `McpSerializationError` → `"Prism MCP serialization failure. Contact Prism operator; see audit log for details."` (`"internal"` category, `ec_code_override: Some("E-MCP-003")`) — see `prism_error_to_structured_call_result` VariantMeta arms in `error_mapping.rs` for authoritative suggestion strings.

This is the **message/suggestion split** cited in error-taxonomy v2.41 rows for E-INT-001, E-AUTH-010, E-AUTH-011, E-QUERY-034, E-WATCH-002, and the §INT narrative. This section codifies the split as an explicit postcondition; previously it was only implicit in the field description table above. Ratifying authority: DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] + error-taxonomy v2.40. Amended at v1.10 to clarify the UNIVERSAL message rule, the EXHAUSTIVE catch-all class enumeration, and the class-differentiated suggestion rule (F-MCPNULL-P6-OBS-003 2026-07-13). Amended at v1.11 to add `AuditPersistenceFailed` as the ONE EXHAUSTIVE exception to Rule 1 (MED-001), and to adjudicate `McpSerializationError` to `category: "internal"` with explicit suggestion and `ec_code_override: Some("E-MCP-003")` (OBS-002; DEFECT-MCP-ROWSHAPE-NULLS-001 pass-7 2026-07-13). Amended at v1.14 to EXCLUDE `SafetyContextContamination` and `SafetyDataExfiltration` from the catch-all — dedicated safety arm added (`category: "safety"`, per-variant `ec_code_override` via nested match; §MED-001). Safety arm `suggestion` = `"Do not retry; report to operator."` (BC §Category table, safety category); `original_params_valid: true` (safety layer detected malicious CONTENT, not malformed SHAPE); `upstream_message: null` (no upstream sensor contacted). Rule 1 message redaction (`"Internal error"`) for both variants is UNCHANGED (2026-07-13).

| Field | Rule |
|-------|------|
| `message` | `"Internal error"` — verbatim terse form for all -32000 variants. **ONE exhaustive exception:** `AuditPersistenceFailed` emits taxonomy-verbatim Display per BC-2.05.001 DEC-014 (no sensitive detail; agent needs code + retry guidance for transient fail-closed). All other -32000 variants MUST use `"Internal error"`. |
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
| `"internal"` | Prism-side infrastructure or invariant failure — sensor was NEVER reached; Prism's own storage, I/O, internal invariant, or query engine failed. Also covers watchdog-triggered query termination (see Watchdog note below), Prism MCP layer serialization failures, and DataFusion query engine failures. | Do not retry; escalate to Prism operator for infrastructure investigation | `PrismError::Internal`, `PrismError::Io`, `StorageOpenFailed`, `StorageWriteFailed`, `StorageReadFailed`, `StorageDomainNotFound`, `StorageKeyNotFound`, `StorageLockHeld`, `StorageHealthCheckFailed`, `SchemaMismatch`, `StorageBatchFailed`, **`WatchdogKilled`, `WatchdogHeartbeatMissed`, `WatchdogRestartLimitExceeded`**, **`McpSerializationError`**, **`QueryPlanFailed` (E-QUERY-002), `QueryExecutionFailed` (E-QUERY-034), `QueryMaterializationLimitExceeded` (E-QUERY-005), `QueryMemoryBudgetExceeded` (E-WATCHDOG-001), `QueryVirtualFieldFailed` (E-QUERY-010), `QueryDenylisted` (E-QUERY-008)** |

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
| `PrismError::QueryExecutionFailed { detail: "DataFusion execution error: ..." }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-QUERY-034"` | error (LOW-002 — query engine arm) |
| `PrismError::QueryDenylisted { failure_count: 3, reason: "timeout".to_string(), expiry_ts: 9_999_999 }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-QUERY-008"` | error (LOW-002 — query engine arm) |
| `PrismError::QueryMemoryBudgetExceeded { limit_mb: 200, used_mb: 210 }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-WATCHDOG-001"` | error (LOW-002 — query engine arm) |
| `PrismError::QueryPlanFailed { detail: "plan error".to_string() }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-QUERY-002"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."` | error (LOW-001 — missing LOW-002 vector) |
| `PrismError::QueryMaterializationLimitExceeded { count: 10001, max: 10000 }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-QUERY-005"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."` | error (LOW-001 — missing LOW-002 vector) |
| `PrismError::QueryVirtualFieldFailed { field: "device_id".to_string(), detail: "resolution failed".to_string() }` | `category: "internal"`, `original_params_valid: false`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-QUERY-010"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."` | error (LOW-001 — missing LOW-002 vector) |
| `PrismError::SafetyContextContamination { detail: "test contamination".to_string() }` | `category: "safety"`, `original_params_valid: true`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-SAFETY-001"`, `suggestion: "Do not retry; report to operator."` | error (MED-001 — safety boundary arm) |
| `PrismError::SafetyDataExfiltration { field: "api_key".to_string() }` | `category: "safety"`, `original_params_valid: true`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-SAFETY-002"`, `suggestion: "Do not retry; report to operator."` | error (MED-001 — safety boundary arm) |
| `PrismError::WritePartialFailure { .. }` (genuinely catch-all variant) | `category: "upstream_error"`, `suggestion: "See audit log for details."` — NOT `"safety"`; proves safety arm is correctly scoped to only the 2 safety variants | error (LOW-001 — catch-all-not-safety regression guard) |

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

## Implementer Code Follow-Up (F-4, OBS-1, OBS-2, MED-001, RETRYABLE-503)

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

### OBS-002 (pass-7 status: REQUIRED — implementer must apply after this BC commit)

**Change:** In `prism_error_to_structured_call_result`, update the `McpSerializationError` arm:

1. `category: "upstream_error"` → `category: "internal"` (rationale: Prism's own MCP response serialization layer failed; sensor was never involved; fault domain is Prism-internal)
2. `suggestion:` → `"Prism MCP serialization failure. Contact Prism operator; see audit log for details."`
3. `ec_code_override: None` → `ec_code_override: Some("E-MCP-003")` (required: without this pin, the structured error falls through to code "E-INT-001" via the catch-all; `PrismError::McpSerializationError` Display prefix is `"E-MCP-003:"` per `prism-core/src/error.rs` `#[error]` attribute)
4. Remove the stale deferral comment: `// not a sensor failure ... remains catch-all pending future BC amendment`

**Exact implementer strings (implementation-ready):**
```rust
PrismError::McpSerializationError { .. } => VariantMeta {
    category: "internal",
    suggestion: "Prism MCP serialization failure. Contact Prism operator; see audit log for details.",
    retryable: false,
    retry_after_seconds: None,
    original_params_valid: true,
    source_override: None,
    upstream_message: None,
    owned_suggestion: None,
    ec_code_override: Some("E-MCP-003"),
    // ... (other fields None)
},
```

**Tests to add:** `test_BC_2_10_007_mcp_serialization_error_category_is_internal` asserting:
- `prism_error_to_structured_call_result(PrismError::McpSerializationError { detail: "test".to_string() })` produces `category: "internal"`, `code: "E-MCP-003"`, `suggestion: "Prism MCP serialization failure. Contact Prism operator; see audit log for details."`, `retryable: false`

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

### LOW-002 (status: REQUIRED — implementer must apply in DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst)

**Change:** In `prism_error_to_structured_call_result`, add a dedicated arm for the six query engine variants, giving them `category: "internal"` instead of the catch-all `category: "upstream_error"`.

**The six variants currently fall to the `_ =>` catch-all with `category: "upstream_error"`.** This is semantically incorrect: these are DataFusion/Prism query engine failures, not sensor boundary failures. The sensor was NOT the fault domain.

**Exact implementer code (implementation-ready, v1.13-corrected):**
```rust
// ── Query engine failures → category "internal" ─────────────────────────────
// BC-2.10.007 v1.12/v1.13 §LOW-002: Six DataFusion/query-engine variants. The sensor
// dispatch has completed (data is in MemTables) or was never relevant; the failure
// is in Prism's own query planning/execution/materialization/virtual-field/denylist
// layer. Category "internal" is correct. "upstream_error" (catch-all default) was
// semantically wrong — it directed LLM agents to investigate sensor health when the
// fault domain is Prism's own query engine.
//
// ec_code_override per variant: map_prism_error returns "Internal error" for ALL six
// (message field MUST be "Internal error" per BC-2.10.007 Rule 1; Display strings are
// NOT used as the message). Without per-variant pins, the code inference would fall
// through to "E-INT-001" for all six. The nested match binds the correct taxonomy code
// to each variant directly. (v1.12 incorrectly specified ec_code_override: None with
// the claim that Display-prefix inference fires — that was wrong: inference reads the
// map_prism_error message string, which is "Internal error", not the variant Display.)
//
// original_params_valid: false — the caller's query triggered the engine failure
// in all six cases (plan failure, execution failure, row limit, memory limit, virtual
// field reference, or denylist match). This signals to the LLM agent that reformulating
// the query might be warranted before escalating to the operator.
PrismError::QueryPlanFailed { .. }
| PrismError::QueryExecutionFailed { .. }
| PrismError::QueryMaterializationLimitExceeded { .. }
| PrismError::QueryMemoryBudgetExceeded { .. }
| PrismError::QueryVirtualFieldFailed { .. }
| PrismError::QueryDenylisted { .. } => {
    let ec_code: &'static str = match &err {
        PrismError::QueryPlanFailed { .. } => "E-QUERY-002",
        PrismError::QueryExecutionFailed { .. } => "E-QUERY-034",
        PrismError::QueryMaterializationLimitExceeded { .. } => "E-QUERY-005",
        PrismError::QueryMemoryBudgetExceeded { .. } => "E-WATCHDOG-001",
        PrismError::QueryVirtualFieldFailed { .. } => "E-QUERY-010",
        PrismError::QueryDenylisted { .. } => "E-QUERY-008",
        _ => unreachable!("outer OR-pattern guarantees only the six query-engine variants"),
    };
    VariantMeta {
        category: "internal",
        suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details.",
        retryable: false,
        retry_after_seconds: None,
        original_params_valid: false,
        source_override: None,
        upstream_message: None,
        owned_suggestion: None,
        ec_code_override: Some(ec_code),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
    }
}
```

**Tests to add:**
- `test_BC_2_10_007_query_execution_failed_category_is_internal`: asserts `QueryExecutionFailed { detail: "DataFusion execution error".to_string() }` → `category: "internal"`, `original_params_valid: false`, `retryable: false`, `code: "E-QUERY-034"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."`.
- `test_BC_2_10_007_query_plan_failed_category_is_internal`: asserts `QueryPlanFailed { detail: "plan error".to_string() }` → `category: "internal"`, `code: "E-QUERY-002"`.
- `test_BC_2_10_007_query_denylisted_category_is_internal`: asserts `QueryDenylisted { failure_count: 3, reason: "timeout".to_string(), expiry_ts: 9_999_999 }` → `category: "internal"`, `code: "E-QUERY-008"`.
- `test_BC_2_10_007_query_memory_budget_exceeded_category_is_internal`: asserts `QueryMemoryBudgetExceeded { limit_mb: 200, used_mb: 210 }` → `category: "internal"`, `code: "E-WATCHDOG-001"`.
- `test_BC_2_10_007_query_materialization_limit_exceeded_category_is_internal` (LOW-001 gap): asserts `QueryMaterializationLimitExceeded { count: 10001, max: 10000 }` → `category: "internal"`, `original_params_valid: false`, `retryable: false`, `code: "E-QUERY-005"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."`.
- `test_BC_2_10_007_query_virtual_field_failed_category_is_internal` (LOW-001 gap): asserts `QueryVirtualFieldFailed { field: "device_id".to_string(), detail: "resolution failed".to_string() }` → `category: "internal"`, `original_params_valid: false`, `retryable: false`, `code: "E-QUERY-010"`, `suggestion: "Prism query engine failure. Contact Prism operator; see audit log for details."`.

**Regression guard to update:** The test `test_CRIT_B_catch_all_category_is_upstream_error` (if it uses any of these 6 variants to exercise the catch-all) must be updated to use a genuinely unmapped variant. Verify by grepping the test body for `QueryExecutionFailed`, `QueryPlanFailed`, etc.

**SensorHttpError regression guard remains unchanged:** The existing `test_BC_2_10_007_sensor_http_error_category_is_upstream_error` (or equivalent) MUST continue to assert `SensorHttpError → category: "upstream_error"`. This verifies that the new query engine arm does NOT incorrectly capture genuine upstream sensor failures.

### MED-001 (status: REQUIRED — implementer must apply in DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst)

**Finding:** F-MCPRS-PRL3-MED-001 (pass-3 2026-07-13). `SafetyContextContamination` and `SafetyDataExfiltration` fell to the `_ =>` catch-all in `prism_error_to_structured_call_result` with `category: "upstream_error"` and `ec_code: "E-INT-001"`. The BC's own §Category table (line ~158) already enumerates both variants under `category: "safety"` — this was spec-code drift. LLM-agent impact: safety boundary violations received "retry a different sensor" guidance instead of "do not retry; report to operator."

**Root cause:** `map_prism_error` returns `(INTERNAL_ERROR, "Internal error")` for both variants per Rule 1 redaction (correct, must not change). Code inference in `prism_error_to_structured_call_result` extracts the E-code from the `map_prism_error` message string — which is `"Internal error"` with no E-prefix — so without `ec_code_override`, both fall through to `E-INT-001`. Identical mechanism to §LOW-002 query engine arm; same nested-match fix pattern required.

**Rule 1 compliance (MANDATORY):** `map_prism_error` MUST continue to return `"Internal error"` for `SafetyContextContamination` and `SafetyDataExfiltration`. The Rule 1 message redaction is correct and must not be changed. The fix is VariantMeta-side only (`ec_code_override` + `category` + `suggestion`).

**Exact VariantMeta arm (implementation-ready):**

```rust
// ── Safety boundary violations → category "safety" ──────────────────────────
// BC-2.10.007 v1.14 §MED-001 (F-MCPRS-PRL3-MED-001): SafetyContextContamination
// and SafetyDataExfiltration previously fell to the `_ =>` catch-all with
// category: "upstream_error" and ec_code: "E-INT-001". This was semantically wrong:
// these are Prism-side safety boundary detections, not upstream sensor failures.
//
// map_prism_error returns INTERNAL_ERROR/"Internal error" for BOTH variants per
// BC-2.10.007 Rule 1 redaction (error_mapping.rs ~lines 316-321). Code inference
// reads the map_prism_error message ("Internal error"), not the variant Display.
// Without ec_code_override, both fall to "E-INT-001". Per-variant ec_code_override
// required via nested match (same pattern as §LOW-002 query engine arm).
//
// original_params_valid: true — the tool call parameters were structurally valid
// (well-formed query, valid tool invocation); the safety boundary detected malicious
// CONTENT, not malformed SHAPE. Analogous to CapabilityDenied (category "permission",
// original_params_valid: true). LLM-agent strategy: do not retry; report to operator.
//
// upstream_message: null — safety violations are detected by Prism's own safety
// layer; no upstream sensor was contacted. DI-006: raw detection detail suppressed.
//
// RULE 1 INVARIANT: map_prism_error MUST continue to return "Internal error" for
// both variants. This is CORRECT per Rule 1 redaction. The message field in the
// structured error stays "Internal error". Only ec_code_override, category, and
// suggestion are addressed here. Do NOT change map_prism_error for these variants.
PrismError::SafetyContextContamination { .. }
| PrismError::SafetyDataExfiltration { .. } => {
    let ec_code: &'static str = match &err {
        PrismError::SafetyContextContamination { .. } => "E-SAFETY-001",
        PrismError::SafetyDataExfiltration { .. } => "E-SAFETY-002",
        _ => unreachable!("outer OR-pattern guarantees only the two safety variants"),
    };
    VariantMeta {
        category: "safety",
        suggestion: "Do not retry; report to operator.",
        retryable: false,
        retry_after_seconds: None,
        original_params_valid: true,
        source_override: None,
        upstream_message: None,
        owned_suggestion: None,
        ec_code_override: Some(ec_code),
        near_text: None,
        reference_pointer: None,
        valid_operators_for_type: None,
        how_to_fix: None,
        available_columns: None,
        did_you_mean: None,
        normalized_pql: None,
    }
}
```

**Per-variant VariantMeta summary (for implementer routing):**

| Field | `SafetyContextContamination` | `SafetyDataExfiltration` | Rationale |
|-------|------------------------------|--------------------------|-----------|
| `category` | `"safety"` | `"safety"` | BC §Category table: safety boundary violations |
| `ec_code_override` | `Some("E-SAFETY-001")` | `Some("E-SAFETY-002")` | `map_prism_error` returns `"Internal error"` (Rule 1); no E-prefix inference; codes from `#[error]` attributes in `prism-core/src/error.rs` |
| `suggestion` | `"Do not retry; report to operator."` | `"Do not retry; report to operator."` | BC §Category table exact agent-strategy string for `"safety"` category |
| `retryable` | `false` | `false` | Safety violations are permanent, not transient |
| `original_params_valid` | `true` | `true` | Params structurally valid; safety layer detected malicious CONTENT, not malformed SHAPE; analogous to `CapabilityDenied` (`original_params_valid: true`) |
| `upstream_message` | `None` (null) | `None` (null) | Prism's own safety layer detected this; no upstream sensor contacted; DI-006 |
| `source_override` | `None` | `None` | Defaults to `"prism_mcp"` via standard wiring |
| `message` (Rule 1) | `"Internal error"` (UNCHANGED) | `"Internal error"` (UNCHANGED) | Rule 1 redaction — map_prism_error MUST NOT change for these variants |

**Tests to add:**
- `test_BC_2_10_007_safety_context_contamination_category_is_safety`: asserts `SafetyContextContamination { detail: "test contamination".to_string() }` → `category: "safety"`, `original_params_valid: true`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-SAFETY-001"`, `suggestion: "Do not retry; report to operator."`.
- `test_BC_2_10_007_safety_data_exfiltration_category_is_safety`: asserts `SafetyDataExfiltration { field: "api_key".to_string() }` → `category: "safety"`, `original_params_valid: true`, `retryable: false`, `upstream_message: null`, `source: "prism_mcp"`, `code: "E-SAFETY-002"`, `suggestion: "Do not retry; report to operator."`.
- `test_BC_2_10_007_catch_all_category_is_not_safety_regression_guard` (LOW-001 regression guard): asserts a variant that genuinely falls to the `_ =>` catch-all (e.g., `PrismError::Infusion(..)` or `PrismError::WritePartialFailure { .. }`) → `category: "upstream_error"` (NOT `"safety"`); proves the safety arm is correctly scoped to only the 2 safety variants and no genuine catch-all variants are captured by it.

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

### RETRYABLE-503 (status: REQUIRED — pre-existing drift; dispatch to implementer in separate fix-burst)

**Finding (F-MCPRS-PRL6 pass-6 out-of-scope disposition 1, 2026-07-14):** Pre-existing spec-code drift not introduced by DEFECT-MCP-ROWSHAPE-NULLS-001 PR. §Canonical Test Vectors row for `PrismError::SensorHttpError { .. }` (sensor returns 503) specifies `retryable: true`. `error_mapping.rs` ships `retryable: false` for ALL `SensorHttpError` status codes — the entire `_ =>` arm sets `retryable: false` regardless of HTTP status.

**Adjudication (BC owner 2026-07-14): SPEC IS CORRECT — code fix required.**

Rationale: BC §Complete field specification for `structuredContent.error` defines `retryable` as "`true` for transient errors (rate limit, timeout, network); `false` for permanent (invalid params, auth invalid)." HTTP 503 Service Unavailable is an explicitly transient HTTP status — the upstream service is temporarily unavailable and the caller may retry after some delay. HTTP 401 and 403 are non-transient authentication/authorization failures and correctly map to `retryable: false`. All other HTTP status codes for `SensorHttpError` (5xx service failures, 408 request timeout, etc.) are transient and must map to `retryable: true`.

**Code change required in `error_mapping.rs` `SensorHttpError` arm:**
```rust
// BC-2.10.007 v1.15 §RETRYABLE-503: authentication failures (401/403) are non-retryable
// (permanent credential failure). All other HTTP status codes are transient and retryable
// (5xx server errors, 408 timeout, etc.). Pre-existing gap: prior arm set retryable: false
// unconditionally for all non-401/403 statuses. Spec correction per RETRYABLE-503 adjudication.
let retryable = !matches!(status, 401 | 403);
```

Note: the dedicated `SensorRateLimited` variant already handles 429 with explicit `retry_after_seconds` wiring and `category: "transient"`. The `SensorHttpError` 429 path (if a sensor sends 429 without triggering the rate-limit variant) should also be retryable via this change.

**Tests to add/update:**
- `test_BC_2_10_007_sensor_http_error_503_retryable_is_true` (new): asserts `SensorHttpError { sensor: "crowdstrike".to_string(), status: 503, body: "Service Unavailable".to_string() }` → `retryable: true`, `category: "upstream_error"`.
- Update any existing test that asserts `SensorHttpError` with non-401/403 status → `retryable: false` to assert `retryable: true`.
- `test_BC_2_10_007_sensor_http_error_401_retryable_is_false` (retain/add): asserts `SensorHttpError { status: 401, .. }` → `retryable: false`, `category: "authentication"`.

**Routing (orchestrator):** This is a pre-existing spec-code drift confirmed at DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL pass-6. The PR HEAD is frozen; this fix MUST be dispatched to the implementer in a new fix-burst after pass-6 closes.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.15 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB18-F-MCPRS-PRL6-MED-001 | 2026-07-14 | product-owner | **F-MCPRS-PRL6-MED-001 closure + POL-29 full vector sweep + out-of-scope dispositions.** **(1) MED-001 — QueryDenylisted vector row struct-truth correction.** §Canonical Test Vectors row for QueryDenylisted cited nonexistent field `query_hash: "abc123"`. Actual `PrismError::QueryDenylisted` struct (prism-core/src/error.rs): `{ failure_count: u32, reason: String, expiry_ts: u64 }`. Fixed to `{ failure_count: 3, reason: "timeout".to_string(), expiry_ts: 9_999_999 }`. **(2) POL-29 full sweep.** §LOW-002 Tests-to-add entry for `test_BC_2_10_007_query_denylisted_category_is_internal` also cited `query_hash` — corrected to the same real struct. All other vector rows verified against error.rs: `SensorNotRegisteredForOrg { sensor_id, org_slug }` ✓; `QueryExecutionFailed { detail }` ✓; `QueryMemoryBudgetExceeded { limit_mb, used_mb }` ✓; `QueryPlanFailed { detail }` ✓; `QueryMaterializationLimitExceeded { count, max }` ✓; `QueryVirtualFieldFailed { field, detail }` ✓; `SafetyContextContamination { detail }` ✓; `SafetyDataExfiltration { field }` ✓. **(3) RETRYABLE-503 adjudication (out-of-scope disposition 1).** New §Implementer Code Follow-Up subsection added. Pre-existing spec-code drift: §Canonical Test Vectors row for `SensorHttpError { .. }` (503) correctly specifies `retryable: true`; `error_mapping.rs` ships `retryable: false` for ALL `SensorHttpError` status codes. Adjudication: **spec is correct** — BC field semantics define `retryable: true` for "transient errors (rate limit, timeout, network)"; HTTP 503 is transient. Code fix required via new fix-burst (frozen HEAD constraint). |
| 1.14 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB16-F-MCPRS-PRL3-MED-001-LOW-001 | 2026-07-13 | product-owner | **F-MCPRS-PRL3-MED-001 + F-MCPRS-PRL3-LOW-001 closure (pass-3 spec side).** **(1) MED-001 — SafetyContextContamination/SafetyDataExfiltration safety-category arm.** Both variants fell to the `_ =>` catch-all with `category: "upstream_error"` and `ec_code: "E-INT-001"` — spec-code drift from BC §Category table. Root cause: `map_prism_error` returns `"Internal error"` for both per Rule 1 redaction (correct, unchanged); code inference cannot recover E-SAFETY codes without `ec_code_override`. Fix: dedicated safety arm added to §Implementer Code Follow-Up (same nested-match pattern as §LOW-002). Changes: (a) catch-all arm description: `"scheduler/detection/case/safety variants"` → `"scheduler/detection/case variants"` + EXCLUDED NOTE for both safety variants from v1.14; (b) §Implementer Code Follow-Up: §MED-001 section added with exact VariantMeta arm (per-variant ec_code via nested match, `category: "safety"`, `suggestion: "Do not retry; report to operator."`, `original_params_valid: true`, `upstream_message: None`); (c) header updated to `(F-4, OBS-1, OBS-2, MED-001)`; (d) Rule 1 compliance explicitly stated: `map_prism_error` MUST NOT change for safety variants. **(2) LOW-001 — §Canonical Test Vectors completed.** v1.13 §Canonical Test Vectors carried only 3 of 6 LOW-002 query engine vectors; QueryPlanFailed (E-QUERY-002), QueryMaterializationLimitExceeded (E-QUERY-005), and QueryVirtualFieldFailed (E-QUERY-010) were missing. Added those 3 plus 2 safety variant vectors + catch-all regression guard: 6 new rows total. §LOW-002 "Tests to add" extended with 2 missing tests (`QueryMaterializationLimitExceeded`, `QueryVirtualFieldFailed`). **Companion:** error-taxonomy v2.47 — E-SAFETY-001/002 descriptions corrected from stale Phase 1 placeholders to match `SafetyContextContamination`/`SafetyDataExfiltration` `#[error]` attributes in `prism-core/src/error.rs`. **No code changes required by this BC alone** — implementation follows via separate implementer fix-burst routing. |
| 1.13 | DEFECT-MCP-ROWSHAPE-NULLS-001-FB15-EC_CODE_OVERRIDE | 2026-07-13 | product-owner | **F-MCPRS-PRL2-LOW-002 closure refinement (implementer-discovered) — §LOW-002 arm code corrected.** v1.12 §LOW-002 specified `ec_code_override: None` with rationale "No `ec_code_override` required — each variant's Display carries its E-QUERY-NNN / E-WATCHDOG-NNN prefix, which the code extraction logic resolves correctly." This was incorrect. Root cause: v1.12 conflated the variant's Display string (which DOES carry E-QUERY-NNN prefix) with the `map_prism_error` message string (which returns `"Internal error"` for all 6 per Rule 1 redaction). Code inference in `prism_error_to_structured_call_result` extracts the E-code from the `map_prism_error` message string — not the variant Display. For all 6 variants, `map_prism_error` returns `(INTERNAL_ERROR, "Internal error")`; the message has no E-prefix; inference falls through to `E-INT-001` for all without explicit overrides. **Fix:** §LOW-002 arm code updated to shipped mechanism: OR-pattern arm wraps a nested `let ec_code: &'static str = match &err { ... }` computing per-variant E-codes (E-QUERY-002/034/005/010/008, E-WATCHDOG-001); `VariantMeta` uses `ec_code_override: Some(ec_code)`. §LOW-002 comment corrected with explanation of WHY inference cannot fire (Rule 1 → `"Internal error"` message → no E-prefix → E-INT-001 fallback). §Internal-redacted split "Query engine arm" sub-class rationale corrected from "No `ec_code_override` required" to the explicit nested-match requirement with explanation. No semantic change to category/suggestion/retryable/original_params_valid decisions — those v1.12 adjudications stand. |
| 1.12 | DEFECT-MCP-ROWSHAPE-NULLS-001-PRL2-LOW-002 | 2026-07-13 | product-owner | **F-MCPRS-PRL2-LOW-002 closure — §Category "internal" enumeration aligned with semantic rule for 6 query engine variants.** Finding: `QueryPlanFailed`, `QueryExecutionFailed`, `QueryMaterializationLimitExceeded`, `QueryMemoryBudgetExceeded`, `QueryVirtualFieldFailed`, and `QueryDenylisted` all fell to the `_ =>` catch-all in `prism_error_to_structured_call_result` with `category: "upstream_error"`, contradicting the BC's own semantic rule ("Prism itself failed before or independent of any sensor dispatch"). Construction site analysis confirmed: ALL 6 variants are DataFusion/Prism query engine failures constructed exclusively in `materialization.rs`, `memory.rs`, `internal_tables.rs`, and `internal_tables.rs` — no sensor API call appears in any construction path. **Per-variant adjudication (all `category: "internal"`):** (1) `QueryPlanFailed` (E-QUERY-002) — DataFusion query planning failure, pre-execution; (2) `QueryExecutionFailed` (E-QUERY-034) — DataFusion execution failure; sensor data was already loaded into MemTables; (3) `QueryMaterializationLimitExceeded` (E-QUERY-005) — Prism-internal row-limit enforcement during materialization; (4) `QueryMemoryBudgetExceeded` (E-WATCHDOG-001) — DataFusion memory pool exhaustion; (5) `QueryVirtualFieldFailed` (E-QUERY-010) — Prism-side virtual field computation failure; (6) `QueryDenylisted` (E-QUERY-008) — Prism-side denylist rejection. All six: `original_params_valid: false` (the caller's query triggered the failure), `retryable: false`, `upstream_message: null`. No `ec_code_override` needed: each variant's Display carries its E-QUERY-NNN/E-WATCHDOG-NNN prefix. **Changes:** §Category "internal" row extended with the 6 variants; §Internal-redacted split: `_` catch-all updated to exclude the 6 (with explicit NOTE); new "Query engine arm" sub-class added with suggestion `"Prism query engine failure. Contact Prism operator; see audit log for details."` (EXHAUSTIVE for this string); §Canonical Test Vectors: 3 vectors added for LOW-002; §Implementer Code Follow-Up: §LOW-002 section added with exact VariantMeta arm code. **Companion:** none (error-taxonomy rows for E-QUERY-002/034/005/010/008 and E-WATCHDOG-001 required no update — their existing rows did not document MCP category; that is BC-2.10.007's domain). |
| 1.11 | DEFECT-MCP-ROWSHAPE-NULLS-001-P7-MED-001-OBS-002 | 2026-07-13 | product-owner | **F-MCPNULL-P7-MED-001 + F-MCPNULL-P7-OBS-002 closure.** **(1) MED-001 — Rule 1 universality overclaim corrected.** The v1.10 Rule 1 claimed `message = "Internal error"` is "(UNIVERSAL, ALL -32000 arms)" without exception, but `PrismError::AuditPersistenceFailed` is an intentional exception: `map_prism_error` emits the full taxonomy-verbatim Display (`"E-AUDIT-001: Audit emission failed; write operation blocked. Retry the operation. If the error persists, check tracing subscriber health."`) as `message`, NOT `"Internal error"`. Rationale: carries no sensitive detail; agent caller needs code + retry guidance for this transient retryable fail-closed condition. Authority: `map_prism_error` `AuditPersistenceFailed` arm comment + BC-2.05.001 DEC-014. Rule 1 heading changed from "(UNIVERSAL, ALL -32000 arms)" to "(UNIVERSAL with one exhaustive exception)"; explicit carve-out paragraph added; exception list stated EXHAUSTIVE. The v1.10 changelog row's claim "the code already implements this split correctly" was accurate for the message/suggestion split behavior in general, but v1.10 Rule 1 prose overstated universality — this v1.11 entry corrects the spec-only overclaim. No code change required: `map_prism_error` arm was already correct. **(2) OBS-002 — McpSerializationError category adjudication.** The `McpSerializationError` arm in `prism_error_to_structured_call_result` carried `category: "upstream_error"` with a deferred-amendment comment "not a sensor failure ... remains catch-all pending future BC amendment". This IS that amendment. Ruling: Prism MCP response serialization failure is Prism-internal (Prism's own serialization layer failed; sensor was never involved) → `category: "internal"`. Exact implementer strings: `category: "internal"`, `suggestion: "Prism MCP serialization failure. Contact Prism operator; see audit log for details."`, `ec_code_override: Some("E-MCP-003")` (required to prevent E-INT-001 fallback; `McpSerializationError` Display prefix is `"E-MCP-003:"` per prism-core `error.rs` `#[error]` attribute). Added `McpSerializationError` to §Category decision rule "internal" row. Added `McpSerializationError` to §Internal-redacted split Rule 2 canonical examples. Added §Implementer Code Follow-Up OBS-002 (pass-7). Companion: error-taxonomy v2.42 (E-AUDIT-001 message contract annotation + E-MCP-003 row update). |
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
