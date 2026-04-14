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

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-AUTH-001 | broken | "OAuth2 token request failed for {sensor} on client '{client_id}'" | No | CrowdStrike OAuth2 client credentials flow failed (invalid client_id/secret) |
| E-AUTH-002 | degraded | "OAuth2 token expired mid-request for {sensor}; auto-refresh failed" | No | Token expired and automatic refresh also failed |
| E-AUTH-003 | broken | "Bearer token rejected by {sensor} for client '{client_id}' (HTTP {status})" | No | Claroty/Armis static bearer token is invalid or revoked |
| E-AUTH-004 | broken | "Cookie authentication failed for {sensor} on client '{client_id}'" | No | Cyberint cookie-based auth flow failed |
| E-AUTH-005 | broken | "Credentials not found for ({client_id}, {sensor_id})" | No | No credentials in keyring or file backend for the specified sensor |

## SENSOR: Sensor API Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-SENSOR-001 | degraded | "{sensor} API unreachable for client '{client_id}': {reason}" | Yes | HTTP connection refused, timeout, or DNS resolution failure |
| E-SENSOR-002 | degraded | "{sensor} API returned HTTP {status} for client '{client_id}'" | Yes | HTTP 5xx server error from sensor API |
| E-SENSOR-003 | degraded | "{sensor} API rate limit exceeded for client '{client_id}'" | Yes | HTTP 429; `retry_after_seconds` populated from Retry-After header |
| E-SENSOR-004 | degraded | "{sensor} API returned unexpected response format" | Yes | HTTP 200 but body is not valid JSON or missing structural fields |
| E-SENSOR-005 | degraded | "Partial results from {sensor}: query truncated after page {n}" | No | Sensor became unavailable mid-pagination; partial results returned |

## OCSF: Normalization Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-OCSF-001 | broken | "OCSF protobuf descriptor failed to load: {error}" | No | Fatal startup error; build-time issue |
| E-OCSF-002 | cosmetic | "OCSF field '{field}' unmappable from {sensor} record type '{type}'" | No | Field preserved in raw_extensions; OCSF message still valid |
| E-OCSF-003 | cosmetic | "Timestamp parse failed for {sensor} record; using fetch timestamp as fallback" | No | Record included with degraded timestamp accuracy |
| E-OCSF-004 | degraded | "OCSF protobuf encoding failed for record from {sensor}" | No | Record skipped; logged at error level |

## CRED: Credential Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-CRED-001 | broken | "OS keyring unavailable: {platform_error}" | No | Keychain locked, libsecret not running, etc. |
| E-CRED-002 | broken | "Encrypted file backend key material missing" | No | Encryption key env var not set |
| E-CRED-003 | broken | "Credential decryption failed for ({client_id}, {sensor_id})" | No | Key material changed or file corrupted |
| E-CRED-004 | broken | "Invalid credential name: '{name}' does not match [{pattern}]" | No | Path traversal prevention |

## FLAG: Feature Flag Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-FLAG-001 | broken | "Write capability '{path}' not enabled for client '{client_id}'" | No | Runtime TOML denies the capability |
| E-FLAG-002 | broken | "Write capability '{path}' not compiled (cargo feature absent)" | No | Compile-time feature gate missing |
| E-FLAG-003 | broken | "Token expired for action '{action_summary}'" | No | Confirmation token TTL exceeded |
| E-FLAG-004 | broken | "Token already consumed for action '{action_summary}'" | No | Single-use token reuse attempt |
| E-FLAG-005 | broken | "Token action hash mismatch" | No | Confirmed action differs from original request |
| E-FLAG-006 | broken | "Write operation with client_id: null not supported" | No | Write operations require an explicit client_id; cross-client writes are not permitted |
| E-FLAG-007 | broken | "Token store capacity reached (100 active tokens)" | No | Hard cap on active confirmation tokens; wait for expiry or confirm/cancel pending actions |

## STATE: Pagination/Cache State Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-STATE-001 | degraded | "Pagination cursor invalid or expired for {sensor}/{source}" | No | Ephemeral cursor not found in-memory (server restarted, expired, or corrupted). Start a new query. |
| E-STATE-002 | cosmetic | "Cache miss for {sensor}/{source} on client '{client_id}'; fetching from sensor" | No | Informational; cache entry evicted or never cached. Not a blocking error. |

## CFG: Configuration Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-CFG-001 | broken | "Client '{client_id}' not found in configuration" | No | Referenced client not in TOML |
| E-CFG-002 | broken | "Missing required field: {toml_path}" | No | TOML validation failure |
| E-CFG-003 | broken | "Invalid value for {toml_path}: expected {expected}, got {actual}" | No | TOML type or value validation failure |
| E-CFG-004 | broken | "Configuration file not found at {path}" | No | TOML file path does not exist |

## MCP: Protocol Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-MCP-001 | broken | "Invalid client_id format: '{value}'" | No | client_id validation failure |
| E-MCP-002 | broken | "Tool '{name}' not available for client '{client_id}'" | No | Tool hidden by feature flags, agent somehow invoked it |
| E-MCP-003 | degraded | "MCP transport error: {reason}" | Yes | Stdio pipe issue, transient |
| E-MCP-004 | broken | "Invalid parameter '{param}': {reason}" | No | Tool input validation failure |
| E-MCP-999 | broken | "Internal error during error formatting" | No | Fallback error when error construction itself fails |

## AUDIT: Audit Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-AUDIT-001 | broken | "Audit emission failed; write operation blocked" | Yes | Audit subscriber failed during a write operation; the write was not executed. Retry may succeed if the subscriber recovers. |

## SAFETY: Prompt Injection Errors

| Code | Severity | Message Format | Retryable | Description |
|------|----------|---------------|-----------|-------------|
| E-SAFETY-001 | cosmetic | "Suspicious pattern detected in field '{field}' of {sensor} record" | No | Informational; added to safety_flags, not a blocking error |
| E-SAFETY-002 | broken | "Safety regex compilation failed at startup: {error}" | No | Fatal startup error; regex patterns invalid |
