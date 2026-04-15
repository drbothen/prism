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
| E-AUDIT-002 | degraded | transient | "Vector endpoint unreachable for audit log forwarding" | Yes | External audit destination unavailable; entries accumulate in RocksDB buffer with exponential backoff retry |
| E-AUDIT-003 | degraded | transient | "Audit buffer approaching capacity ({count}/{max} entries)" | No | Buffer nearing 100K limit; oldest entries will be purged if limit exceeded |
| E-AUDIT-004 | broken | transient | "Audit buffer purge operation failed: {reason}" | Yes | RocksDB error during overflow purge; buffer continues growing; next purge cycle retries |

## QUERY: Query Engine Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-QUERY-001 | broken | validation | "Query parse error at position {pos}: {message}" | No | AxiQL query string cannot be parsed (syntax error, unknown keyword, unknown field) |
| E-QUERY-002 | broken | validation | "Type error: field '{field}' is {actual_type}, cannot use {operator}" | No | Type mismatch in query predicate (e.g., numeric comparison on string field) |
| E-QUERY-003 | broken | validation | "Security limit exceeded: {limit_name} is {actual} (max {max})" | No | Query exceeds a syntactic security limit (length, nesting depth, pipe stages, regex length) |
| E-QUERY-004 | degraded | transient | "Query timed out after {seconds}s" | Yes | Query execution exceeded the 30s timeout. Retryable with a narrower scope. |
| E-QUERY-005 | broken | validation | "Materialization limit exceeded: fetched {count} records (max 10000)" | No | Streaming record counter exceeded 10K during sensor fan-out fetch |
| E-QUERY-006 | broken | validation | "Query scope too broad: estimated {count} records across {sensor_count} sensors" | No | Query would produce more results than can be materialized; narrow by time range, client, sensor, or severity |
| E-QUERY-008 | broken | validation | "Query has been denylisted after {N} consecutive failures ({reason}). Denylist expires at {expiry}." | No | Query matches a denylisted hash due to previous resource violations. Modify the query to change its hash, or clear the denylist via watchdog_status. Use `force_execute: true` to override. |
| E-QUERY-009 | broken | validation | "Required column constraint violation for {sensor}: columns [{required_columns}] must be constrained in WHERE clause" | No | Query does not constrain a REQUIRED column for a target sensor. The sensor API requires certain parameters (e.g., a time range or entity ID) to prevent full-scan of unbounded remote APIs. Add the listed columns to the WHERE clause. See DI-021. |
| E-QUERY-010 | broken | validation | "Internal tables are read-only via AxiQL. Use the dedicated MCP tool: {tool_name}" | No | SQL write statement (INSERT/UPDATE/DELETE) targets an internal Prism table; mutations go through dedicated MCP tools |

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
| E-SCHED-006 | degraded | transient | "Query execution failed for client '{client_id}' on schedule '{schedule_id}': {reason}" | Yes | Query execution failed for a specific client; error recorded in history; schedule continues for other clients |
| E-SCHED-007 | degraded | transient | "Query exceeded watchdog limits on schedule '{schedule_id}': {reason}" | No | Scheduled query terminated by watchdog; error recorded; schedule remains active |
| E-SCHED-008 | broken | validation | "Maximum schedule count exceeded: {current_count}/{max_count}" | No | The configurable maximum number of active schedules has been reached (default 500, configurable via `[defaults.limits].max_schedules` in TOML). Delete unused schedules before creating new ones. |

## PACK: Query Pack Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-PACK-001 | broken | configuration | "Pack configuration parse error at {line}:{column}: {message}" | No | packs.toml parse failure; fatal startup error |
| E-PACK-002 | broken | validation | "Pack '{pack_id}' contains query that fails AxiQL parsing: {query_name}" | No | A query within the pack has invalid AxiQL syntax; entire pack is rejected |
| E-PACK-003 | degraded | validation | "Pack '{pack_id}' discovery query exceeds security limits" | No | Discovery query for pack activation is too complex; pack marked inactive |
| E-PACK-004 | broken | validation | "Pack name '{name}' already exists" | No | Pack names must be unique; use a different name or delete the existing pack |
| E-PACK-005 | broken | not_found | "Pack '{name}' not found" | No | Referenced pack does not exist or has been deleted |

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
| E-RULE-006 | broken | permission | "Rule scope 'global' requires 'detection.write.global' capability" | No | Creating a global-scope rule requires the elevated `detection.write.global` capability path |
| E-RULE-007 | broken | not_found | "Rule '{rule_id}' not found at scope '{scope}'" | No | No rule with the given ID exists at the specified scope |
| E-RULE-008 | cosmetic | validation | "Rule condition references field '{field}' not in OCSF schema or vendor extensions" | No | Advisory warning; compilation proceeds; field resolves to NULL at execution time |
| E-RULE-009 | degraded | validation | "Sequence rule too complex for SQL compilation (exceeds join depth)" | No | Fallback to interpretive evaluation with performance warning |
| E-RULE-010 | broken | validation | "Rule ID '{rule_id}' already exists at {scope} scope; analyst rules must use unique IDs" | No | Analyst-created rule ID conflicts with an existing rule at global or client scope |
| E-RULE-011 | broken | validation | "Maximum rule count exceeded: {current_count}/{max_count}" | No | The configurable maximum number of active detection rules has been reached (default 1000, configurable via `[defaults.limits].max_rules` in TOML). Delete unused rules before creating new ones. |

## DETECT: Detection Evaluation Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-DETECT-001 | cosmetic | validation | "Field type mismatch in rule '{rule_id}': numeric comparison on string field '{field}'" | No | Predicate evaluates to false; warning logged; evaluation continues for other predicates |
| E-DETECT-002 | cosmetic | validation | "CIDR parse failure in rule '{rule_id}' at evaluation time: '{value}'" | No | Predicate evaluates to false; warning logged; should not occur if validated at load time |
| E-DETECT-003 | cosmetic | validation | "Group-by field is null for record in correlation rule '{rule_id}'" | No | Record excluded from correlation tracking for this rule; warning logged; no alert generated for this record |
| E-DETECT-004 | degraded | transient | "Window state deserialization failure for correlation rule '{rule_id}' on startup" | No | Affected correlation windows are reset to empty; warning logged; correlation detection resumes from clean state |
| E-DETECT-005 | cosmetic | validation | "Key field is null for record in sequence rule '{rule_id}'" | No | Record excluded from sequence tracking for this rule; warning logged |
| E-DETECT-006 | degraded | transient | "Sequence state deserialization failure for rule '{rule_id}' on startup" | No | Affected trackers reset to step 0; warning logged; detection resumes from clean state |
| E-DETECT-010 | degraded | transient | "Dedup index read failure from RocksDB for rule '{rule_id}'" | No | Alert is persisted (fail-open for dedup — better to have a duplicate than miss an alert); warning logged |

## ALERT: Alert Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-ALERT-001 | broken | not_found | "Alert '{alert_id}' not found" | No | Referenced alert does not exist or has been purged |
| E-ALERT-002 | broken | validation | "Alert '{alert_id}' already acknowledged" | No | The alert has already been acknowledged; no further acknowledgment is needed |

## CASE: Case Management Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CASE-001 | broken | not_found | "Case '{case_id}' not found" | No | Referenced case does not exist or has been deleted |
| E-CASE-002 | broken | validation | "Invalid state transition: cannot move case from '{current}' to '{target}'" | No | Case status transitions follow a defined state machine; check valid transitions per DI-025 |
| E-CASE-003 | broken | validation | "Disposition required when resolving case '{case_id}'" | No | Cases must have a disposition (true_positive, false_positive, benign, inconclusive) before transitioning to 'resolved' |
| E-CASE-004 | broken | validation | "Invalid state transition: cannot move case from '{current}' to '{target}'. Valid targets: [{valid_targets}]" | No | The requested state transition is not one of the 12 valid transitions in the case state machine |
| E-CASE-005 | broken | validation | "Case '{case_id}' is already in status '{status}'" | No | Self-transition attempted; the case is already in the requested target status |
| E-CASE-006 | broken | validation | "Disposition is required before resolving case '{case_id}'. Set disposition via update_case first." | No | Transition to Resolved requires a disposition to be set first |
| E-CASE-008 | broken | validation | "Case '{case_id}' belongs to client '{actual_client}', not '{requested_client}'" | No | Case belongs to a different client than specified in the request |
| E-CASE-009 | broken | validation | "Invalid filter value: '{value}' is not a valid {field_type}" | No | Invalid status or severity value in list_cases filter |
| E-CASE-010 | broken | validation | "Invalid disposition variant: '{name}'. Valid: TruePositive, FalsePositive, Benign, Inconclusive" | No | Unrecognized disposition variant name |
| E-CASE-011 | broken | validation | "Invalid annotation type: '{type}'. Valid user-created types: note, evidence_link, ot_impact" | No | Invalid annotation type; status_change and alert_link are system-generated only |
| E-CASE-012 | broken | validation | "Annotation content is empty or exceeds 10000 characters (got {length})" | No | Annotation content length constraint violation |
| E-CASE-013 | broken | validation | "Annotation type '{type}' is system-generated and cannot be created manually" | No | User attempted to create a status_change or alert_link annotation; these are auto-generated by the system |

## STORE: Storage Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-STORE-001 | broken | configuration | "RocksDB initialization failed: {reason}" | No | The embedded RocksDB instance could not be opened (corrupt WAL, missing directory, permissions). Requires manual intervention. |
| E-STORE-002 | broken | transient | "Domain write failed for {domain}: {reason}" | No | A write to the RocksDB storage layer failed. The operation was not persisted. Check disk space and permissions. |
| E-STORE-003 | degraded | transient | "Domain read failed for {domain}: {reason}" | Yes | A read from the RocksDB storage layer failed. May be transient (I/O contention) or permanent (corruption). |
| E-STORE-004 | broken | configuration | "Column family '{cf_name}' not found in RocksDB" | No | Expected column family is missing from the database. May indicate a schema migration issue or database corruption. |
| E-STORE-005 | broken | configuration | "Database lock held by another process at '{path}'" | No | Another Prism instance is using the RocksDB data directory; single-process invariant (DI-017) |
| E-STORE-006 | broken | configuration | "Database corruption detected at '{path}'" | No | RocksDB detected corruption; attempts automatic repair; if repair fails, requires manual re-initialization |
| E-STORE-007 | broken | configuration | "Insufficient disk space at '{path}': available {available_mb}MB, required {required_mb}MB" | No | Fatal startup error; free disk space or change state_dir path |
| E-STORE-008 | degraded | transient | "I/O error during RocksDB read for domain '{domain}': {os_error}" | Yes | Read operation failed; may be transient I/O contention or permanent disk issue |
| E-STORE-009 | degraded | transient | "Dirty bit write failed for operation '{op}'" | No | Crash recovery disabled for this operation; warning logged; operation proceeds |
| E-STORE-010 | degraded | transient | "Recovery action failed on startup for dirty bit '{key}'" | No | Warning logged; dirty bit NOT cleared; recovery retried on next startup |

## CONFIRM: Confirmation Token Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-CONFIRM-001 | broken | permission | "Confirmation token expired or invalid for action '{action_summary}'" | No | The confirmation token has expired (300s TTL) or does not exist. Agent must re-request via the original write tool. |

## UDF: User-Defined Function Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-UDF-001 | cosmetic | validation | "ioc_match references unknown pattern set: '{name}'" | No | Returns false; warning logged; check IOC pattern set configuration |
| E-UDF-002 | cosmetic | validation | "time_window received invalid duration: '{value}'" | No | Returns false; warning logged; use format like '24h', '7d', '30m' |
| E-UDF-003 | cosmetic | validation | "subnet_contains received malformed CIDR: '{value}'" | No | Returns false; warning logged; use valid CIDR notation (e.g., '10.0.0.0/8') |

## WATCH: Watchdog Configuration Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-WATCH-001 | broken | configuration | "Invalid watchdog level: '{value}'. Valid: normal, restrictive, permissive" | No | Fatal startup error; check `watchdog.level` in TOML config |
| E-WATCH-002 | cosmetic | configuration | "Watchdog override value below safe minimum: {param}={value} (minimum {min})" | No | Value clamped to minimum (64 MB memory, 5s timeout, 1000 records); warning logged |

## DECOR: Context Decorator Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-DECOR-001 | cosmetic | transient | "Periodic decorator refresh failed: {reason}" | Yes | Stale cached values used; warning logged; retry on next refresh interval |
| E-DECOR-002 | cosmetic | configuration | "Config-time decorator references missing config field: '{field}'" | No | Decorator value set to null; warning logged |

## STATE: Pagination State Errors (additional)

Additional state errors beyond E-STATE-001 and E-STATE-002 (defined in the STATE section above):

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-STATE-003 | degraded | transient | "RocksDB domain '{domain}' is corrupted or unreadable during table scan" | No | Internal table registration failed for the affected domain; structured error with recovery suggestion (restart, check state_dir) |

## WATCHDOG: Watchdog Errors

| Code | Severity | Category | Message Format | Retryable | Description |
|------|----------|----------|---------------|-----------|-------------|
| E-WATCHDOG-001 | broken | validation | "Query memory limit exceeded: {current_bytes} bytes (budget {budget_bytes})" | No | The query's memory consumption exceeded the watchdog budget. The query has been terminated and added to the denylist. Narrow the query scope or increase the memory budget. |
| E-WATCHDOG-002 | broken | validation | "Query denylisted: hash '{query_hash}' blocked since {added_at}" | No | This query (by content hash) has been placed on the denylist due to previous resource violations. Modify the query to change its hash, or clear the denylist via watchdog_status. |
