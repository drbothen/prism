---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-21T00:00:00Z
phase: 2-patch
origin: greenfield
subsystem: "SS-20"
capability: "CAP-035"
lifecycle_status: active
introduced: cycle-1-pass-80
modified: 2026-04-21
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/architecture/observability.md"
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[md5]"
traces_to: ["CAP-035"]
extracted_from: ".factory/specs/architecture/observability.md"
---

# BC-2.20.004: Log Forwarder Credential Resolution — AD-017 Opaque Reference Model at Forward Time

## Description

Credentials used by external log forwarders (API keys, bearer tokens, HEC tokens, SMTP
passwords) must never appear as inline values in TOML configuration, log entries, MCP
responses, or error messages. Per the AI-opaque credential reference model (AD-017),
forwarder TOML specs reference credential names resolved from environment variables or
the OS keyring at forward time — specifically at the moment a delivery attempt is made,
not at configuration load time. This ensures credential values are never in the process
address space longer than necessary and never transit AI/LLM context.

## Preconditions

- At least one `[[server.log_forward]]` destination is configured
- The destination's credential field uses the reference form:
  `{ source = "env", key = "DD_API_KEY" }` or `{ source = "keyring", key = "..." }`
- The Prism process attempts a delivery batch to the destination

## Postconditions

- At delivery time, the forwarder resolves the credential reference by reading from the
  declared source (env var or OS keyring) immediately before constructing the HTTP request
- The resolved credential value is used only for the duration of the HTTP request and is
  not stored in any heap-allocated struct beyond that scope
- The resolved credential value NEVER appears in:
  - Any log entry at any level (trace/debug/info/warn/error)
  - Any MCP response or error message
  - Any structured error payload
  - Any RocksDB value or diagnostic state
- If credential resolution fails (env var missing, keyring unavailable):
  - The delivery attempt is skipped for this batch
  - A WARN is emitted to the LOCAL sink: `"[log-forwarder/{name}] credential resolution failed for key '{key}' from source '{source}' — skipping batch delivery"`
  - The credential key NAME is included in the warning; the credential VALUE is never included
  - The batch is returned to the front of the queue for retry

## Invariants

- `{ source = "env", key = "..." }` form is the only accepted inline form; literal credential
  values in TOML are rejected at config load time
- Credential resolution is lazy (at delivery time), not eager (at config load)
- AD-017 applies to ALL forwarder types (datadog, splunk_hec, elasticsearch, otlp, webhook, plugin)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| Env var missing | `DD_API_KEY` not set at delivery time | WARN with key name; batch skipped; retry scheduled; credential value NOT in warning |
| Keyring unavailable | OS keyring locked or inaccessible | WARN with key name; batch skipped; retry scheduled |
| Inline literal in TOML | `api_key = "sk-live-abc123"` (not reference form) | Config validation error at load time with `E-CFG-NNN: "Credential value must use reference form {source = ..., key = ...}"` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-20-016 | Credential rotated at runtime (env var updated while Prism running) | Next delivery batch reads new value; no restart required; old value NOT cached |
| EC-20-017 | Two destinations use the same env var key | Each resolves independently at delivery time; no shared cached value |
| EC-20-018 | WASM plugin forwarder requests credential via `host.get-config()` | Plugin receives the reference string only; host resolves and passes resolved value via a credential-safe host call; resolved value is not logged by host |
| EC-20-019 | Error response from destination endpoint contains credential echo (misconfigured server) | The raw response body is NOT logged at any level; only the HTTP status code is logged |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-20-004-happy | `api_key = { source = "env", key = "DD_API_KEY" }`; env var set | Delivery succeeds; no credential value in any log line | AD-017 |
| TV-20-004-missing | `api_key = { source = "env", key = "DD_API_KEY" }`; env var unset | WARN with `key = "DD_API_KEY"`; no value in WARN; batch retried | EC-20 error case |
| TV-20-004-inline | `api_key = "sk-live-abc123"` in TOML | Config load rejects with `E-CFG-NNN`; forwarder not registered | Inline literal rejected |
| TV-20-004-rotate | Env var updated at T+5m; delivery at T+6m | New value used for delivery at T+6m; no cached stale value | EC-20-016 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-TBD-20-004 | No log line at any level contains the resolved credential value; credential key name may appear | Integration test (set known credential value; capture all log output; assert value string absent) |

## Related BCs

- BC-2.03.006 — Credential Resolution at Sensor Query Time (same AD-017 model; analogous for sensors)
- BC-2.03.007 — Secret Redaction in Logs, Errors, and MCP Responses
- BC-2.18.007 — Action Credentials Must Use AI-Opaque Reference Model (same pattern for action delivery)
- BC-2.20.001 — Recursive Prevention (credential resolution errors route to local sink only)

## Architecture Anchors

- `specs/architecture/observability.md` §Forwarding Guarantees — "Credential safety — forwarder credentials use the same AI-opaque reference model (AD-017)"
- `specs/architecture/observability.md` §External Log Forwarding — TOML examples showing `{ source = "env", key = "..." }` form

## Story Anchor

S-5.09 — prism-mcp: External Log Forwarding Subsystem

## VP Anchors

TBD — integration test in `tests/log_forwarding_tests.rs`

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-035 (Diagnostic Log Forwarding) |
| ADR | AD-017 (AI-Opaque Credentials), observability.md §Forwarding Guarantees |
| Story | S-5.09 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pass-80-follow-on | 2026-04-21 | product-owner | Re-anchored CAP-025 → CAP-035 (business-analyst created CAP-035 post-hoc per pass-80 F80-002 follow-on); removed Capability Anchor Note; updated inputs (capabilities.md already present) |
| 1.0 | pass-80-remediation | 2026-04-21 | product-owner | Initial contract — F80-002 gap closure |
