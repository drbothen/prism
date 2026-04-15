---
document_type: prd-supplement
level: L3
section: "error-taxonomy"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
---

# Error Taxonomy for Prism

All Prism errors follow the code format `E-{CATEGORY}-{NNN}` and are surfaced as structured error responses (BC-2.10.007). Each error includes a severity level, retryability, and an actionable suggestion for the LLM agent.

## Severity Levels

| Severity | Definition | Example |
|----------|-----------|---------|
| **broken** | Feature completely unusable; requires human intervention or config change | Invalid credentials, config validation failure |
| **degraded** | Feature partially working; some data available but incomplete | Partial sensor failure in cross-client query, rate limiting |
| **cosmetic** | Minor issue; full functionality available with a workaround | Audit emission warning, unmapped OCSF field |

---

## AUTH: Authentication Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-AUTH-001 | broken | authentication | "OAuth2 token request failed for {sensor} on client '{client_id}'" | No | CrowdStrike OAuth2 client credentials flow failed (invalid client_id/secret) |
| E-AUTH-002 | degraded | authentication | "OAuth2 token expired mid-request for {sensor}; auto-refresh failed" | No | Token expired and automatic refresh also failed |
| E-AUTH-003 | broken | authentication | "Bearer token rejected by {sensor} for client '{client_id}' (HTTP {status})" | No | Claroty/Armis static bearer token is invalid or revoked |
| E-AUTH-004 | broken | authentication | "Cookie authentication failed for {sensor} on client '{client_id}'" | No | Cyberint cookie-based auth flow failed |
| E-AUTH-005 | broken | authentication | "Credentials not found for ({client_id}, {sensor_id})" | No | No credentials in keyring or file backend for the specified sensor |

## SENSOR: Sensor API Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-SENSOR-001 | degraded | upstream_error | "{sensor} API unreachable for client '{client_id}': {reason}" | Yes | HTTP connection refused, timeout, or DNS resolution failure |
| E-SENSOR-002 | degraded | upstream_error | "{sensor} API returned HTTP {status} for client '{client_id}'" | Yes | HTTP 5xx server error from sensor API |
| E-SENSOR-003 | degraded | upstream_error | "{sensor} API rate limit exceeded for client '{client_id}'" | Yes | HTTP 429; `retry_after_seconds` populated from Retry-After header |
| E-SENSOR-004 | degraded | upstream_error | "{sensor} API returned unexpected response format" | Yes | HTTP 200 but body is not valid JSON or missing structural fields |
| E-SENSOR-005 | degraded | upstream_error | "Partial results from {sensor}: query truncated after page {n}" | No | Sensor became unavailable mid-pagination; partial results returned |

## OCSF: Normalization Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-OCSF-001 | broken | configuration | "OCSF protobuf descriptor failed to load: {error}" | No | Fatal startup error; build-time issue |
| E-OCSF-002 | cosmetic | upstream_error | "OCSF field '{field}' unmappable from {sensor} record type '{type}'" | No | Field preserved in raw_extensions; OCSF message still valid |
| E-OCSF-003 | cosmetic | upstream_error | "Timestamp parse failed for {sensor} record; using fetch timestamp as fallback" | No | Record included with degraded timestamp accuracy |
| E-OCSF-004 | degraded | upstream_error | "OCSF protobuf encoding failed for record from {sensor}" | No | Record skipped; logged at error level |

## CRED: Credential Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CRED-001 | broken | configuration | "OS keyring unavailable: {platform_error}" | No | Keychain locked, libsecret not running, etc. |
| E-CRED-002 | broken | configuration | "Encrypted file backend key material missing" | No | Encryption key env var not set |
| E-CRED-003 | broken | authentication | "Credential decryption failed for ({client_id}, {sensor_id})" | No | Key material changed or file corrupted |
| E-CRED-004 | broken | validation | "Invalid credential name: '{name}' does not match [{pattern}]" | No | Path traversal prevention |

## FLAG: Feature Flag Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-FLAG-001 | broken | permission | "Write capability '{path}' not enabled for client '{client_id}'" | No | Runtime TOML denies the capability |
| E-FLAG-002 | broken | permission | "Write capability '{path}' not compiled (cargo feature absent)" | No | Compile-time feature gate missing |
| E-FLAG-003 | broken | permission | "Token expired for action '{action_summary}'" | No | Confirmation token TTL exceeded |
| E-FLAG-004 | broken | permission | "Token already consumed for action '{action_summary}'" | No | Single-use token reuse attempt |
| E-FLAG-005 | broken | permission | "Token action hash mismatch" | No | Confirmed action differs from original request |
| E-FLAG-006 | broken | permission | "Write operation with client_id: null not supported" | No | Write operations require an explicit client_id; cross-client writes are not permitted |
| E-FLAG-007 | degraded | permission | "Token store capacity reached (100 active tokens)" | Yes | Hard cap on active confirmation tokens; retry after existing tokens expire (up to 300s). Confirm or cancel pending actions to free capacity sooner. |
| E-FLAG-008 | broken | permission | "Confirmation token not found: '{token_id}'" | No | Token does not exist in the in-memory store (server may have restarted, or token_id is invalid). Agent must re-request via the original write tool. |

## STATE: Pagination State Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-STATE-001 | degraded | transient | "Pagination cursor invalid or expired for {sensor}/{source}" | No | Ephemeral cursor not found in-memory (server restarted, expired, or corrupted). Start a new query. |
| E-STATE-002 | degraded | transient | "Active cursor cap ({cap}) reached for {sensor}/{source}" | Yes | Maximum active pagination cursors reached after lazy cleanup of expired cursors. Complete or abandon existing pagination sessions before starting new ones. Retryable because cursors expire (600s TTL). |

## CACHE: Response Cache Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CACHE-001 | broken | transient | "Cache invalidation failed during write for ({client_id}, {sensor_id}, {source_id}): {reason}" | No | Cache invalidation failed during a write operation. The write itself succeeded at the sensor, but the cache may contain stale data. Server should terminate if caused by mutex poisoning. |

## CFG: Configuration Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CFG-001 | broken | configuration | "Client '{client_id}' not found in configuration" | No | Referenced client not in TOML |
| E-CFG-002 | broken | configuration | "Missing required field: {toml_path}" | No | TOML validation failure |
| E-CFG-003 | broken | configuration | "Invalid value for {toml_path}: expected {expected}, got {actual}" | No | TOML type or value validation failure |
| E-CFG-004 | broken | configuration | "Configuration file not found at {path}" | No | TOML file path does not exist |

## MCP: Protocol Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-MCP-001 | broken | validation | "Invalid client_id format: '{value}'" | No | client_id validation failure |
| E-MCP-002 | broken | permission | "Tool '{name}' not available for client '{client_id}'" | No | Tool hidden by feature flags, agent somehow invoked it |
| E-MCP-003 | degraded | transient | "MCP transport error: {reason}" | Yes | Stdio pipe issue, transient |
| E-MCP-004 | broken | validation | "Invalid parameter '{param}': {reason}" | No | Tool input validation failure |
| E-MCP-999 | broken | transient | "Internal error during error formatting" | No | Fallback error when error construction itself fails |

## AUDIT: Audit Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-AUDIT-001 | broken | transient | "Audit emission failed; write operation blocked" | Yes | Audit subscriber failed during a write operation; the write was not executed. Retry may succeed if the subscriber recovers. |

## QUERY: Query Engine Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-QUERY-001 | broken | validation | "Query parse error at position {pos}: {message}" | No | AxiQL query string cannot be parsed (syntax error, unknown keyword, unknown field) |
| E-QUERY-002 | broken | validation | "Type error: field '{field}' is {actual_type}, cannot use {operator}" | No | Type mismatch in query predicate (e.g., numeric comparison on string field) |
| E-QUERY-003 | broken | validation | "Security limit exceeded: {limit_name} is {actual} (max {max})" | No | Query exceeds a syntactic security limit (length, nesting depth, pipe stages, regex length) |
| E-QUERY-004 | degraded | transient | "Query timed out after {seconds}s" | Yes | Query execution exceeded the 30s timeout. Retryable with a narrower scope. |
| E-QUERY-005 | broken | validation | "Materialization limit exceeded: fetched {count} records (max 10000)" | No | Streaming record counter exceeded 10K during sensor fan-out fetch |
| E-QUERY-006 | broken | validation | "Query scope too broad: estimated {count} records across {sensor_count} sensors" | No | Query would produce more results than can be materialized; narrow by time range, client, sensor, or severity |

## ALIAS: Alias Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-ALIAS-001 | broken | validation | "Alias '{name}' not found in scope '{scope}'" | No | Referenced alias does not exist in any applicable scope (global or per-client) |
| E-ALIAS-002 | broken | validation | "Alias cycle detected: {chain}" | No | Creating or updating the alias would introduce a circular reference |
| E-ALIAS-003 | broken | validation | "Alias composition depth exceeded: {chain} (max 3)" | No | Alias references other aliases beyond the maximum nesting depth of 3 |
| E-ALIAS-004 | broken | validation | "Parameter '{param}' missing for alias '{name}'" | No | Parameterized alias invoked without a required parameter that has no default, or invoked with an unknown parameter name |
| E-ALIAS-005 | broken | validation | "Alias '{name}' has dependent aliases: {dependents}" | No | Deletion blocked because other aliases reference this alias. Delete dependents first or use `force: true` for cascade deletion. |

## SAFETY: Prompt Injection Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-SAFETY-001 | cosmetic | safety | "Suspicious pattern detected in field '{field}' of {sensor} record" | No | Informational; added to safety_flags, not a blocking error |
| E-SAFETY-002 | broken | safety | "Safety regex compilation failed at startup: {error}" | No | Fatal startup error; regex patterns invalid |
