---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T14:00:00
phase: 2-patch
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.008: `get_diagnostics` MCP Tool — Subsystem Diagnostic Query with Injection Defense

## Description

The `get_diagnostics` MCP tool returns aggregated operational diagnostics from one of
10 diagnostic subsystems. Results are read-only summaries — never raw log lines — and
are passed through `InjectionScanner::scan()` before inclusion in the response. Any
response containing content derived from external sensor data (e.g., detection
correlation group keys that are attacker-controlled field values) is annotated with
`_meta.safety_flags` and `trust_level: "untrusted_external"`. Credential values are
never included. The operation is idempotent: no state is mutated by calling this tool.

The tool accepts an optional `tenant_id` to narrow results to a single client's
operations. Without `tenant_id`, cross-client aggregates are returned (subject to
analyst permissions).

## Preconditions

- The MCP server is running and at least one subsystem has emitted diagnostic data
- The `subsystem` parameter, if provided, is one of the 10 valid subsystem names (see
  Invariants for the canonical list); omitting `subsystem` is valid and returns a
  cross-subsystem overview
- `since` is a valid ISO 8601 timestamp or relative duration string (e.g., `"1h"`, `"30m"`)
  if provided; omitting defaults to the last 1 hour
- `trace_id` is a valid UUID v7 string if provided
- `tenant_id` is a valid client_id string matching the pattern `^[a-z0-9_-]{1,64}$` if provided

## Postconditions

- A JSON response is returned with the following envelope:
  ```json
  {
    "_meta": {
      "tool": "get_diagnostics",
      "subsystem": "<subsystem | 'all'>",
      "period": "<since param or 'last 1h'>",
      "trust_level": "<'internal' | 'untrusted_external'>",
      "safety_flags": [],
      "trace_id": "<if requested>",
      "truncated": false
    },
    "data": { ... }
  }
  ```
- `_meta.trust_level` is `"untrusted_external"` if ANY field in `data` was derived from
  external sensor data (correlation group keys, alert titles, rule match field values,
  hostname strings from sensor API responses)
- `_meta.safety_flags` is populated (non-empty) if `InjectionScanner::scan()` detected
  a suspicious pattern in any sensor-derived field
- `_meta.truncated: true` and `_meta.truncated_at_bytes: 10485760` are set if the
  response payload exceeds 10 MB; the data is trimmed at the most recent complete
  record boundary before the limit
- No state is mutated: no rotation triggers, no log clearing, no write to any RocksDB
  column family
- Credential values are absent from all fields; `credentials` subsystem mode returns
  only `{client_id, sensor_id, status: "set"|"missing"|"error", source_type: "keyring"|"env"|"file"}`,
  never the secret value itself

## 10 Subsystem Query Modes

| Subsystem Name | Key Diagnostics Returned |
|----------------|--------------------------|
| `scheduler` | Active schedules, fire/skip counts, semaphore state, overruns, per-schedule timing |
| `detection` | Rule evaluation counts, match/fire/suppress counts, correlation window state, per-rule details |
| `actions` | Delivery success/failure counts, rate limit state, retry queue depth, per-action status |
| `config` | Last reload time/source/result, config source sync status, file watcher state, validation warnings |
| `plugins` | Loaded plugins, WASM compilation status, memory/fuel usage, per-plugin health |
| `infusions` | Loaded infusions, cache hit/miss rates, lookup counts, data file freshness |
| `credentials` | Per-client credential status (set/missing/source type — never values), resolution failures |
| `fanout` | Per-sensor request counts, latency percentiles (p50/p95/p99), error rates, rate limit state |
| `watchdog` | Current RSS, per-query memory usage, denylist entries, recent terminations |
| `storage` | RocksDB column family sizes, compaction state, dirty bit count |

## Invariants

- INV-DIAG-001: `get_diagnostics` MUST NOT return log lines containing unredacted
  credentials. The same credential-redaction logic used by the main log writer is applied
  to all diagnostic output before it enters the MCP response
- INV-DIAG-002: `_meta.safety_flags` MUST be populated for any entry whose content
  was derived from `untrusted_external` source (attacker-controlled sensor data).
  Omitting `safety_flags` when `trust_level: "untrusted_external"` is a contract
  violation
- INV-DIAG-003: Read-only invariant — calling `get_diagnostics` MUST NOT mutate any
  state. No audit entries are written, no config reloads triggered, no log files cleared
- INV-DIAG-004: The `credentials` subsystem mode MUST redact all secret material.
  Only `source_type` and credential status (`set`/`missing`/`error`) are returned

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `subsystem` is not one of the 10 valid names | Structured error: "Invalid subsystem '{value}'. Valid values: scheduler, detection, actions, config, plugins, infusions, credentials, fanout, watchdog, storage." |
| `E-MCP-004` | `since` is not a valid ISO 8601 timestamp or duration string | Structured error: "Invalid since value '{value}'. Use ISO 8601 (e.g. '2026-04-16T10:00:00Z') or relative duration (e.g. '1h', '30m')." |
| `E-MCP-004` | `tenant_id` fails client_id character-set validation | Structured error per BC-2.06.010 |
| `E-MCP-004` | `trace_id` is not a valid UUID v7 | Structured error: "Invalid trace_id format. Expected UUID v7." |
| `E-MCP-DIAG-001` | Response payload exceeds 10 MB after serialization | Response is returned with `_meta.truncated: true`; data trimmed to 10 MB at a complete record boundary. This is not an error — it is a normal size-limit postcondition. The message field includes: "Response truncated at 10 MB. Narrow the query with 'since' or 'subsystem'." |
| `E-CFG-001` | `tenant_id` refers to a client not in the configuration | Structured error: "Client '{tenant_id}' not found." |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-040 | `subsystem: "detection"`, `tenant_id: "acme"` — correlation group keys contain injection-pattern text (attacker-controlled hostname) | `_meta.safety_flags` non-empty; `_meta.trust_level: "untrusted_external"`; values included as-is with flags; host process unaffected |
| EC-08-041 | `subsystem: "credentials"`, all sensors healthy — response includes credential resolution failures | Response includes `{status: "error", source_type: "keyring"}` for the failing sensor; no credential value or redacted placeholder is present |
| EC-08-042 | `since: "72h"` — very long time window, response exceeds 10 MB | `_meta.truncated: true`, `_meta.truncated_at_bytes: 10485760`; oldest entries dropped first; most recent data retained |
| EC-08-043 | No `subsystem` specified — cross-subsystem overview requested | Returns a condensed summary for all 10 subsystems; `_meta.subsystem: "all"` |
| EC-08-044 | `trace_id` provided — only log lines matching that trace ID returned across all subsystems | Response filtered to matching trace; `_meta.subsystem: "all"` |
| EC-08-045 | `subsystem: "fanout"` — all sensors healthy, no errors in window | `recent_errors: []`, `recent_warnings: []`; summary shows successful request counts and latency percentiles |
| EC-08-046 | 100 concurrent `get_diagnostics` calls | All complete independently; no locks held beyond per-subsystem read access; no deadlock; no state mutation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `subsystem: "detection"` with injection-pattern hostname in correlation key | `_meta.safety_flags` non-empty; `trust_level: "untrusted_external"`; value preserved | happy-path + injection |
| `subsystem: "credentials"` | No credential values; only `status` and `source_type` per entry | happy-path |
| `since: "72h"` producing >10 MB response | `_meta.truncated: true`; data trimmed at 10 MB record boundary | edge-case |
| Invalid `subsystem` value | Structured error listing 10 valid subsystem names | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |
| VP-038 | Injection scanner: never panics on arbitrary input strings | fuzz |

## Related BCs

- BC-2.08.009 — Diagnostic Resource Templates (same underlying data, resource interface)
- BC-2.09.004 — Safety Flag Parallel Fields (injection defense pattern applied here)
- BC-2.09.005 — Trust-Level Metadata Per Response (trust_level annotation pattern)
- BC-2.08.005 — Health Check MCP Tool (health data is distinct from diagnostic data)
- BC-2.03.007 — Secret Redaction in Logs, Errors, and MCP Responses (same redaction pipeline)

## Architecture Anchors

- `specs/architecture/observability.md` §"MCP Tool: `get_diagnostics`" — 10 subsystem modes, injection scanning requirement, response shape
- `specs/architecture/observability.md` §"Security Constraints on Diagnostic Logs" — credential redaction, trust_level, safety_flags
- `specs/architecture/api-surface.md` — tool registry entry for `get_diagnostics`
- S-5.08 Task: `server/diagnostics.rs` — diagnostic tool handler, injection scanner integration, 10 subsystem query backends

## Story Anchor

S-5.08 — Diagnostics and `prism logs` CLI

## VP Anchors

Integration test: `tests/diagnostics_tests.rs` — "Simulate detection correlation group key containing injection pattern → verify safety_flags populated, trust_level=untrusted_external, value not stripped."

Integration test: `tests/diagnostics_tests.rs` — "Verify get_diagnostics with credentials subsystem — confirm no credential values in response for all source types."

Integration test: `tests/diagnostics_tests.rs` — "Verify get_diagnostics does not mutate any RocksDB CF state across 100 concurrent invocations."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-002, DI-004 |
| Story | S-5.08 |
| Priority | P1 |
| Interface | observability.md §get_diagnostics |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added missing lifecycle fields (deprecated, deprecated_by, replacement, retired, removed, removal_reason); added ## Canonical Test Vectors scaffolding; added ## Changelog. |
| 1.0 | phase-2-patch | 2026-04-16 | product-owner | Initial draft (phase 2-patch addition) |
