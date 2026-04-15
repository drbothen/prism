---
document_type: prd-supplement
level: L3
section: "nfr-catalog"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
---

# Non-Functional Requirements Catalog

## NFR-001: Sensor Query Latency

| Attribute | Value |
|-----------|-------|
| Category | Performance |
| Requirement | Single-sensor, single-client query tools must return an MCP response within 10 seconds per page under normal conditions (sensor API responsive, no rate limiting). The 10s budget is per-page, inclusive of all HTTP calls required for that page (e.g., CrowdStrike two-step fetch requires 2+ HTTP calls per page -- all are included in the budget). |
| Measurement | Wall-clock time from MCP request receipt to MCP response write, measured via tracing spans |
| Breakdown | MCP overhead: <50ms. Credential retrieval: <100ms. HTTP round-trips to sensor (including multi-step fetch): <8s. OCSF normalization: <200ms. Response construction: <50ms. Safety scanning: <100ms. |
| Degradation | If sensor API latency exceeds 8s, the tool still returns (up to 30s timeout), but health monitoring flags the sensor as slow |
| Traces to | CAP-001, FM-001 |

## NFR-002: OCSF Normalization Overhead

| Attribute | Value |
|-----------|-------|
| Category | Performance |
| Requirement | OCSF normalization adds no more than 5ms per record overhead on top of raw sensor data processing |
| Measurement | Per-record normalization time measured via tracing spans on `OcsfNormalizer::normalize()` |
| Breakdown | DynamicMessage construction: <2ms. Field mapping: <2ms. raw_extensions serialization: <1ms. |
| Traces to | CAP-003 |

## NFR-003: Cross-Client Query Latency

| Attribute | Value |
|-----------|-------|
| Category | Performance |
| Requirement | Cross-client queries (`client_id: null`) must fan out with a configurable concurrency limit (default 10, enforced via semaphore), not sequentially. Total latency bounded by the slowest individual client within each concurrency batch, not the sum. When client count exceeds the concurrency limit, fan-out proceeds in batches. |
| Measurement | Wall-clock time compared to single-client query time for the same sensor |
| Target | Cross-client latency <= 1.5x single-client latency (accounting for aggregation overhead) |
| Traces to | CAP-002 |

## NFR-004: Credential Encryption at Rest

| Attribute | Value |
|-----------|-------|
| Category | Security |
| Requirement | All credentials stored via the encrypted file backend must be encrypted with AES-256-GCM. Key derivation uses HKDF-SHA256 with a per-credential 32-byte random salt and a fixed application-specific info string (`"prism-credential-v1"`), producing a unique 256-bit AES key per credential. Key material must be external to the credential file (environment variable or K8s secret mount). |
| Verification | Unit tests verify ciphertext differs from plaintext. Integration tests verify decryption with correct key and failure with incorrect key. File permission tests verify 0600/0700. |
| Traces to | CAP-004, DI-002, DI-014, R-006 |

## NFR-005: OS Keyring as Primary Backend

| Attribute | Value |
|-----------|-------|
| Category | Security |
| Requirement | OS keyring (macOS Keychain, Windows Credential Vault, Linux libsecret) is the default credential backend. Hardware-backed encryption where available. Encrypted file is fallback only. |
| Platform Notes | macOS: requires Keychain Access permission prompt at startup (pre-auth probe pattern). Linux: requires libsecret/Secret Service. Windows: requires Credential Vault. |
| Traces to | CAP-004, R-009 |

## NFR-006: Audit Trail Completeness

| Attribute | Value |
|-----------|-------|
| Category | Security / Compliance |
| Requirement | Every MCP tool invocation produces exactly one AuditEntry in structured JSON via the `tracing` crate. Write operations additionally log capability check results. Credential access events are logged with client/sensor context but never credential values. |
| Compliance | SOC 2 Type II: immutable, append-only audit trail with timestamp, user, client, action, result. ISO 27001: access control logging, incident response audit trail. |
| Verification | Integration tests assert log output for every tool call path. Property tests verify audit entries are never missing. |
| Traces to | CAP-007, DI-004 |

## NFR-007: Prompt Injection Defense Layers

| Attribute | Value |
|-----------|-------|
| Category | Security |
| Requirement | Four defense layers must be active on all sensor data responses: (1) structural separation, (2) provenance framing, (3) suspicious pattern detection, (4) trust-level metadata. No single layer is sufficient; all four operate simultaneously. |
| Acceptance | Integration tests with known injection payloads verify: payloads appear only in structuredContent fields, never in prose; safety flags are generated; trust_level is "untrusted_external". |
| Limitation | No sanitization is 100% effective against adversarial LLM prompts. The human analyst is the ultimate safety boundary. |
| Traces to | CAP-010, DI-006, R-005 |

## NFR-008: Ephemeral Pagination and Cache Durability

| Attribute | Value |
|-----------|-------|
| Category | Reliability |
| Requirement | Pagination tokens are ephemeral and in-memory only; they do not survive process restarts. This is acceptable because Prism is an interactive MCP tool (not a background poller) and queries can be re-issued. Response cache entries are also in-memory with LRU eviction. Cache loss on restart is acceptable; the next query re-fetches from the sensor API. |
| Verification | Integration tests verify: (1) pagination tokens are invalid after server restart, (2) cache entries are evicted under memory pressure, (3) re-issued queries succeed without cached state. |
| Traces to | CAP-011, R-008 |

## NFR-009: Graceful Shutdown

| Attribute | Value |
|-----------|-------|
| Category | Reliability |
| Requirement | On SIGTERM, SIGINT, or client disconnect, Prism completes shutdown within 5 seconds: cancel in-flight tasks, flush state writes, close HTTP clients, flush log subscribers. Force-exit after 5s timeout. |
| Verification | Integration tests send SIGTERM during active sensor queries and verify: process exits within 6s, state files are valid, no partial writes on disk. |
| Traces to | BC-2.10.010, FM-011 |

## NFR-010: Structured Logging

| Attribute | Value |
|-----------|-------|
| Category | Observability |
| Requirement | All log output uses `tracing` + `tracing-subscriber` with JSON formatter writing to stderr. Log level controlled by `PRISM_LOG_LEVEL` environment variable. All spans include `tenant`, `sensor`, and `source` fields where applicable. |
| Format | JSON lines to stderr. Compatible with Vector pipeline for centralized aggregation. |
| Sensitive Data | Credential values never appear in any log field. Sensor API response bodies logged at debug level only, truncated to 4KB. |
| Traces to | CAP-007 |

## NFR-011: Distributed Tracing

| Attribute | Value |
|-----------|-------|
| Category | Observability |
| Requirement | Every MCP tool invocation creates a tracing span with a unique `trace_id`. Child spans for credential retrieval, sensor API call, OCSF normalization, and response construction are nested under the root span. Span fields follow the `skip_all` + named fields pattern. |
| Traces to | CAP-007 |

## NFR-012: Cross-Platform Binary

| Attribute | Value |
|-----------|-------|
| Category | Compatibility |
| Requirement | Prism compiles and runs on Linux (x86_64, aarch64), macOS (aarch64), and Windows (x86_64). CI produces binaries for all targets. |
| Platform Variations | Credential backend defaults: macOS=Keychain, Windows=CredentialVault, Linux=libsecret (with encrypted file fallback if libsecret unavailable). File permissions: Unix 0600/0700; Windows ACL equivalent. |
| Traces to | R-009 |

## NFR-013: OCSF Schema Version Pinning

| Attribute | Value |
|-----------|-------|
| Category | Compatibility |
| Requirement | OCSF schema version is pinned per Prism release (currently v1.7.0). Proto field numbers are NOT stable across OCSF versions. Schema version is encoded in the proto package path (`ocsf.v1_7_0`). Generated `.proto` files are committed to detect drift. |
| Upgrade Path | OCSF version upgrades require a Prism release with regenerated proto files, updated field mappings, and regression tests against all sensor adapters. |
| Traces to | CAP-003, DI-005, R-004 |

## NFR-014: Safety Scanning Performance

| Attribute | Value |
|-----------|-------|
| Category | Performance / Security |
| Requirement | Suspicious pattern regex scanning must add no more than 1ms per record to response construction time. Regex patterns are compiled once at startup and reused for all scans. |
| Measurement | Per-record scan time measured via tracing spans |
| Traces to | CAP-010, BC-2.09.003 |

## NFR-015: Memory Usage

| Attribute | Value |
|-----------|-------|
| Category | Performance |
| Requirement | Prism's resident memory usage should remain under 512MB during normal operation (single analyst session, up to 50 configured clients). Sensor API responses are processed page-by-page, not buffered entirely in memory. Per-query memory budget is 200MB at the normal watchdog level, enforced by the resource watchdog (CAP-024, BC-2.15.006). |
| Memory Budget Breakdown | Base process + runtime: ~30MB. RocksDB (block cache, memtables, indexes): ~50-100MB. Cache (50 clients x 4 sensors x 50 entries x ~10KB): ~100MB. Detection state (correlation windows, sequence trackers): ~10-50MB. Confirmation tokens + cursors: ~5MB. Query materialization: ~50MB per query (10K records in Arrow columnar format with two-tier storage), capped at 200MB per-query by the resource watchdog. Concurrent queries each get their own 10K record budget but share the 512MB process limit. Under typical single-analyst usage, at most 1-2 queries are materialized concurrently (~100MB for materialization). Headroom: ~27-67MB for transient allocations, HTTP buffers, OCSF schema, and normalization intermediates. |
| Concurrent Query Note | The 10K record limit (DI-019) is per-query. Concurrent queries compete for the shared 512MB process memory. The per-query memory budget of 200MB (normal watchdog level) ensures no single query can consume more than ~40% of the process budget. If concurrent materialization would exceed the memory budget, the later query receives `E-QUERY-005` (retryable: true) with a suggestion to wait for the active query to complete or narrow the query scope. |
| Traces to | R-010 |

## NFR-016: Rate Limit Respect

| Attribute | Value |
|-----------|-------|
| Category | Reliability |
| Requirement | Prism must respect `Retry-After` headers from sensor APIs. Internal rate tracking prevents immediate re-requests after HTTP 429. Per-sensor rate budget warnings are exposed via health monitoring. Backoff follows exponential pattern: 2s base, 30s max. |
| Traces to | CAP-008, FM-008, R-010 |

## NFR-017: Cache Bounds

| Attribute | Value |
|-----------|-------|
| Category | Performance / Reliability |
| Requirement | The in-memory response cache is bounded per client per sensor. Maximum entries per `(client_id, sensor_id)` pair: 50 (configurable, per DI-018). When the cache exceeds the bound, LRU (Least Recently Used) eviction removes the oldest entries. Active pagination tokens are never evicted; only completed query result cache entries are eligible for eviction. The default of 50 scales to: 50 clients x 4 sensors x 50 entries x ~10KB avg = ~100MB (within NFR-015 memory budget). |
| Measurement | Cache hit/miss ratio and eviction count tracked via metrics. Total cache memory consumption monitored. |
| Verification | Unit tests verify LRU eviction behavior. Integration tests verify cache bounds under concurrent multi-client queries. |
| Traces to | CAP-011, NFR-015 |

## NFR-018: Token Store Cap

| Attribute | Value |
|-----------|-------|
| Category | Security / Reliability |
| Requirement | The in-memory confirmation token store enforces a hard cap of 100 active (non-expired) tokens. Expired tokens are proactively cleaned up on each new token creation request. If the cap is reached after cleanup, token creation fails with `E-FLAG-007`. This prevents memory exhaustion from unbounded token accumulation. |
| Measurement | Active token count tracked via metrics. Cap-reached events logged. |
| Verification | Unit tests verify cap enforcement. Integration tests verify cleanup behavior and error response when cap is reached. |
| Traces to | CAP-006, BC-2.04.009 |
