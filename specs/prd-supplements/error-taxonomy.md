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
| E-CACHE-001 | broken | transient | "Cache invalidation failed during write for ({client_id}, {sensor_id}, {source_id}): {reason}" | No | Cache invalidation failed during a write operation. The write itself succeeded at the sensor, but the cache may contain stale data. Mutex poisoning triggers process exit with exit code 2 (per interface-definitions.md exit codes). Non-poisoning failures (e.g., serialization) are logged but do not terminate. |

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
| E-ALIAS-004 | broken | validation | "Invalid parameter for alias '{name}': {reason}" | No | Alias invoked with an unknown parameter name, or a parameter value fails type validation (not a simple literal). Note: all parameters must have defaults (enforced at creation time by BC-2.11.008), so "missing parameter without default" is not a reachable state at invocation time. |
| E-ALIAS-005 | broken | validation | "Alias '{name}' has dependent aliases: {dependents}" | No | Deletion blocked because other aliases reference this alias. Delete dependents first or use `force: true` for cascade deletion. |
| E-ALIAS-006 | broken | validation | "Alias name '{name}' conflicts with reserved {type}: '{conflicting_name}'" | No | Alias name matches a known OCSF field name or AxiQL keyword. Choose a different alias name that does not shadow reserved identifiers. |

## IO: Filesystem I/O Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-IO-001 | broken | configuration | "Filesystem write failure during atomic file operation: {path} ({reason})" | No | Filesystem write failure during atomic file operation (aliases.toml, credential files). Likely caused by insufficient permissions or disk full. The operation fails entirely with no partial state. |

## SAFETY: Prompt Injection Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-SAFETY-001 | cosmetic | safety | "Suspicious pattern detected in field '{field}' of {sensor} record" | No | Informational; added to safety_flags, not a blocking error |
| E-SAFETY-002 | broken | safety | "Safety regex compilation failed at startup: {error}" | No | Fatal startup error; regex patterns invalid |

## SCHED: Scheduled Query Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-SCHED-001 | broken | not_found | "Schedule '{schedule_id}' not found" | No | Referenced schedule does not exist or has been deleted |
| E-SCHED-002 | broken | validation | "Invalid interval '{interval}': minimum is 60s" | No | Schedule interval must be at least 60 seconds to prevent excessive API load |
| E-SCHED-003 | broken | validation | "Schedule name '{name}' already exists" | No | Schedule names must be unique; use a different name or delete the existing schedule first |
| E-SCHED-004 | degraded | transient | "Max concurrent schedule executions reached ({count}/16)" | Yes | At most 16 schedules may execute simultaneously; retry after current executions complete |
| E-SCHED-005 | degraded | transient | "Previous execution of schedule '{schedule_id}' still in-flight" | Yes | The schedule's prior run has not completed; wait for it to finish or investigate if it is stuck |

## DIFF: Differential Result Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-DIFF-001 | broken | not_found | "No previous results for schedule '{schedule_id}' client '{client_id}'" | No | Differential computation requires at least two completed runs; wait for the schedule to execute at least twice |
| E-DIFF-002 | broken | validation | "Diff computation exceeded record limit ({count} records, max {max})" | No | The result set is too large for in-memory differential computation; narrow the schedule's query scope |

## RULE: Detection Rule Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-RULE-001 | broken | validation | "Rule predicate parse error at position {pos}: {message}" | No | The AxiQL predicate in the rule definition cannot be parsed (syntax error, unknown field) |
| E-RULE-002 | broken | validation | "Rule validation failed: {reason}" | No | Rule exceeds structural limits (nesting depth, predicate size, regex complexity) or contains invalid references |
| E-RULE-003 | broken | not_found | "Rule '{rule_id}' not found" | No | Referenced rule does not exist or has been deleted |
| E-RULE-004 | broken | validation | "Rule name '{name}' conflicts with existing rule in scope '{scope}'" | No | Rule names must be unique within their scope; use a different name or delete the existing rule |
| E-RULE-005 | broken | validation | "Invalid correlation config: {reason}" | No | Correlation or sequence configuration is malformed (missing group_by, invalid window, threshold < 2, missing stages) |

## ALERT: Alert Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-ALERT-001 | broken | not_found | "Alert '{alert_id}' not found" | No | Referenced alert does not exist or has been purged |
| E-ALERT-002 | broken | validation | "Alert '{alert_id}' already acknowledged" | No | The alert has already been acknowledged; no further acknowledgment is needed |

## CASE: Case Management Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CASE-001 | broken | not_found | "Case '{case_id}' not found" | No | Referenced case does not exist or has been deleted |
| E-CASE-002 | broken | validation | "Invalid state transition: cannot move case from '{current}' to '{target}'" | No | Case status transitions follow a defined state machine; check valid transitions (open -> in_progress -> resolved -> closed) |
| E-CASE-003 | broken | validation | "Disposition required when resolving case '{case_id}'" | No | Cases must have a disposition (true_positive, false_positive, benign, inconclusive) before transitioning to 'resolved' |
| E-CASE-004 | broken | validation | "Case '{case_id}' is already in status '{status}'" | No | The case is already in the requested target status; no update performed |

## STORE: Storage Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-STORE-001 | broken | configuration | "RocksDB initialization failed: {reason}" | No | The embedded RocksDB instance could not be opened (corrupt WAL, missing directory, permissions). Requires manual intervention. |
| E-STORE-002 | broken | transient | "Domain write failed for {domain}: {reason}" | No | A write to the RocksDB storage layer failed. The operation was not persisted. Check disk space and permissions. |
| E-STORE-003 | degraded | transient | "Domain read failed for {domain}: {reason}" | Yes | A read from the RocksDB storage layer failed. May be transient (I/O contention) or permanent (corruption). |
| E-STORE-004 | broken | configuration | "Column family '{cf_name}' not found in RocksDB" | No | Expected column family is missing from the database. May indicate a schema migration issue or database corruption. |

## WATCHDOG: Watchdog Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-WATCHDOG-001 | broken | validation | "Query memory limit exceeded: {current_bytes} bytes (budget {budget_bytes})" | No | The query's memory consumption exceeded the watchdog budget. The query has been terminated and added to the denylist. Narrow the query scope or increase the memory budget. |
| E-WATCHDOG-002 | broken | validation | "Query denylisted: hash '{query_hash}' blocked since {added_at}" | No | This query (by content hash) has been placed on the denylist due to previous resource violations. Modify the query to change its hash, or clear the denylist via watchdog_status. |
