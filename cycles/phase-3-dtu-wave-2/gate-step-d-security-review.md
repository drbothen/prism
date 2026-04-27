---
document_type: gate-step-report
gate_step: d
gate_step_name: security-review
cycle: phase-3-dtu-wave-2
gate: wave-2-integration-gate
scope: e45159b9..c239dd0b
reviewer: vsdd-factory:security-reviewer
date: 2026-04-26
verdict: APPROVED_WITH_CONDITIONS
total_findings: 8
critical: 0
high: 2
medium: 3
low: 3
---

# Wave 2 Integration Gate — Gate Step D: Security Review

**Scope:** e45159b9..c239dd0b
**Reviewer:** vsdd-factory:security-reviewer
**Date:** 2026-04-26
**Verdict:** APPROVED WITH CONDITIONS — 8 findings (0 CRITICAL, 2 HIGH, 3 MEDIUM, 3 LOW)

**Condition:** WGS-W2-001 + WGS-W2-002 must resolve before production deployment with real credentials. MEDIUM/LOW items tracked as TD.

---

## HIGH Findings

### WGS-W2-001 (HIGH, CWE-943, OWASP A03:2021): AQL Query Verbatim Forwarding to Armis API Without Sanitization

**File:line:** `crates/prism-sensors/src/auth/armis.rs:116-120`

**Description:** `ArmisAdapter::build_aql()` forwards the `aql_query` field from `spec.sensor_config` verbatim. Comment at line 119 explicitly states: "Return VERBATIM — no modification, sanitization, or injection prevention." In a multi-tenant MSSP environment, any path by which input can influence `aql_query` creates an injection vector against the Armis API. Cross-tenant data leak risk exists in shared Armis account models.

**Recommendation:** Add spec-parse-time validation layer enforcing an AQL operator allowlist and rejecting cross-scope constructs. Audit (`AuditRiskLevel=HIGH`) any explicit `aql_query` overrides for forensic traceability.

---

### WGS-W2-002 (HIGH, CWE-312, OWASP A02:2021): Derived Bearer Tokens Stored as Plain String in `ArmisAdapter` and `ClarotyAdapter`

**File:line:** `crates/prism-sensors/src/auth/armis.rs:82`, `crates/prism-sensors/src/auth/claroty.rs:146`, `crates/prism-sensors/src/auth/crowdstrike.rs:73` (`CachedToken::token`)

**Description:** Raw API keys correctly use `SecretString`. Derived bearer tokens — which are equally sensitive as actual auth material in `Authorization: Bearer` headers — are plain heap-allocated strings. They are not zeroed on drop; they are visible in heap dumps and core files; they are not guarded against accidental logging.

**Recommendation:** Wrap derived bearer tokens in `SecretString` with `expose_secret()` at the HTTP header injection site.

---

## MEDIUM Findings

### WGS-W2-003 (MEDIUM, CWE-306, OWASP A07:2021): DTU `POST /dtu/reset` Unauthenticated on PagerDuty and Jira Clones

**File:line:** `crates/prism-dtu-pagerduty/src/routes/dtu.rs:89`, `crates/prism-dtu-jira/src/routes/dtu.rs:58`, `crates/prism-dtu-slack/src/routes/dtu.rs:61`

**Description:** The `POST /dtu/reset` endpoint on all three DTU clones has no authentication gate. The sibling endpoint `POST /dtu/configure` is protected by the `X-Admin-Token` header (ADR-003 Amendment #5). The reset endpoint — which reverts the DTU to unconfigured state — is equally sensitive and should receive the same protection.

**Recommendation:** Apply the same `X-Admin-Token` gate to `POST /dtu/reset` as is applied to `POST /dtu/configure`.

---

### WGS-W2-004 (MEDIUM, CWE-20+CWE-74): Event Buffer Key Injection via `table_name` and `client_id` Containing `/`

**File:line:** `crates/prism-sensors/src/event_buffer.rs:157-175` (`write_events`), `46-48` (`scope_prefix`)

**Description:** The `scope_prefix` function constructs a RocksDB key prefix by concatenating `table_name` and `client_id` with `/` separators. The existing `sensor_id` validation (slash rejection) is not applied to `table_name` or `client_id`. An attacker-controlled value in either field containing `/` could craft keys that traverse into adjacent scopes in the RocksDB namespace.

**Recommendation:** Add slash rejection for `table_name` and `client_id` identical to the existing `sensor_id` check.

---

### WGS-W2-005 (MEDIUM, CWE-209, OWASP A09:2021): `SensorError::HttpError { body }` Propagates Raw API Response Bodies into Error Messages

**File:line:** `crates/prism-sensors/src/adapter.rs:77` + all adapter implementations

**Description:** When a sensor API returns an error response, the full HTTP body is captured in `SensorError::HttpError { body }`. This body is then propagated through the error chain and potentially surfaces in logs, MCP responses, or audit records. Sensor API error responses may contain authentication challenge details, internal system identifiers, or other sensitive information.

**Recommendation:** Truncate and sanitize the body before `HttpError` construction (e.g., first 200 bytes, sensitive-key redaction). Log raw body at `TRACE` level only.

---

## LOW Findings

### WGS-W2-006 (LOW, CWE-532): `emit_credential_event` Logs `parameters` JSON via `tracing::info!` + Does NOT Persist to Audit Buffer

Related to WGC-W2-001. The `parameters` JSON blob logged at `info!` level may contain credential names, sensor IDs, and access types. This information flows into structured logs and is not subject to the redaction pipeline that guards production event data. Until the persistence path is implemented, the only record of credential audit events is in plain-text logs.

### WGS-W2-007 (LOW, CWE-362): `unsafe impl Sync for RocksDbBackend` With Noted Race Condition Risk

The `unsafe impl Sync for RocksDbBackend` in `prism-storage` is explicitly documented with a safety invariant comment and DEV-004 tracking. The documentation is correct and the risk is acknowledged. Flagged for completeness; the tracking issue should be resolved before high-concurrency production deployment.

### WGS-W2-008 (LOW, CWE-778): `token_events.rs` Emitters Log `token_id` at `tracing::info!` Level Without Durable Persistence

Related to WGC-W2-001. Token IDs logged at `info!` level in `emit_token_generated`/`emit_token_consumed`/`emit_token_expired` represent compliance-relevant events (SOC 2 CC6.1 token lifecycle). Until persistence is implemented, these records exist only in transient logs.

---

## Positive Security Patterns Observed

- `secrecy::SecretString` for raw credentials (`CrowdStrikeAuth::client_secret`, `CyberintAuth::api_key`, `ArmisAuth::secret_key`, `ClarotyAuth::password`) — correct application of credential protection.
- Sealed trait `SensorAuth` via private module pattern — prevents unauthorized adapter bypass.
- Redaction sentinel covers exact-match keys + suffix patterns (`_key`, `_secret`, `_token`, `_password`) — comprehensive surface coverage.
- Fail-closed audit for write tools (`PrismError::AuditPersistenceFailed` aborts execution) — correct default for compliance-critical paths.
- Dual semaphore DoS resistance (per-query 10 + global 200, 30s timeout) — aligns with BC-2.01.012.
- `sensor_id` slash rejection in `EventBuffer` — correct namespace isolation for RocksDB keys.
- OAuth2 token cache with read-lock fast path + 401-refresh-retry-once — standard industry pattern correctly implemented.
- DTU `/dtu/configure` admin-gated with `uuid::new_v4` (122 bits entropy) — ADR-003 Amendment #5 correctly implemented.
- RocksDB domain isolation via `StorageDomain` — prevents cross-domain key collision.
- `unsafe impl Sync` explicitly documented with safety invariant + DEV-004 tracking.
