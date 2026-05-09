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

# BC-2.08.009: Diagnostic Resource Templates — `prism://diagnostics/*` MCP Resources

## Description

Prism exposes three MCP resource templates in the `prism://diagnostics/` namespace that
give the AI agent proactive read access to operational diagnostic state without requiring
explicit tool calls. These resources surface the same underlying data as the
`get_diagnostics` tool (BC-2.08.008) — they are a view, not a separate store. All three
apply the same injection-defense and credential-redaction invariants as the tool. Resources
never expose raw log files from disk; all data flows through the same redactor used by the
diagnostic tool.

## Preconditions

- The MCP server is running and has registered resource templates via the `resources/list`
  endpoint
- For `prism://diagnostics/{subsystem}`: the `{subsystem}` parameter is one of the 10
  valid subsystem names (same set as BC-2.08.008)
- For `prism://diagnostics/trace/{trace_id}`: `{trace_id}` is a valid UUID v7

## Postconditions

### Resource `list` (resources/list)

- `resources/list` returns a template entry for each of the three diagnostic resources:
  - `prism://diagnostics/summary` — cross-subsystem overview
  - `prism://diagnostics/{subsystem}` — per-subsystem detail (template, `{subsystem}` is
    the variable)
  - `prism://diagnostics/trace/{trace_id}` — full trace of a specific operation
    (template, `{trace_id}` is the variable)
- Each template includes `mimeType: "application/json"` and a description matching the
  observability.md resource documentation

### Resource `read` (`prism://diagnostics/summary`)

- Returns a condensed JSON summary covering all 10 diagnostic subsystems:
  ```json
  {
    "_meta": {
      "resource": "prism://diagnostics/summary",
      "trust_level": "internal",
      "safety_flags": []
    },
    "subsystems": {
      "scheduler": { "status": "ok", "active_schedules": 12, "recent_skips": 0 },
      "detection": { "status": "ok", "active_rules": 47, "alerts_last_hour": 3 },
      "actions": { "status": "ok", "retry_queue_depth": 0 },
      "config": { "status": "ok", "last_reload": "2026-04-15T09:45:00Z" },
      "plugins": { "status": "ok", "loaded_count": 2 },
      "infusions": { "status": "ok", "cache_hit_rate": 0.92 },
      "credentials": { "status": "degraded", "missing_count": 1 },
      "fanout": { "status": "ok", "error_rate_1h": 0.0 },
      "watchdog": { "status": "ok", "rss_mb": 187 },
      "storage": { "status": "ok", "total_cf_size_mb": 48 }
    }
  }
  ```
- `_meta.trust_level` is `"untrusted_external"` if any subsystem summary includes
  sensor-derived content

### Resource `read` (`prism://diagnostics/{subsystem}`)

- Returns the same data as `get_diagnostics(subsystem: "{subsystem}")` — identical JSON
  structure to the tool response data block, wrapped in the resource envelope
- `_meta.resource` is set to the full resource URI
- The same injection scanning, trust_level annotation, and credential redaction apply

### Resource `read` (`prism://diagnostics/trace/{trace_id}`)

- Returns all diagnostic log entries matching `trace_id` across all 10 subsystems,
  ordered by timestamp ascending
- Each entry includes: `timestamp`, `level`, `subsystem`, `message`, `trace_id`
- The same safety/redaction invariants as the per-subsystem resource apply

## Invariants

- INV-DIAG-005: Resources MUST resolve to the same underlying data as the
  `get_diagnostics` tool (single source of truth). A fresh call to `get_diagnostics`
  immediately after reading the resource MUST return data that is at most 1 second more
  stale than the resource data (both read from the same in-memory diagnostic state)
- INV-DIAG-006: Resource templates NEVER expose log files from disk directly. All data
  flows through the diagnostic aggregation layer and the credential redactor — no file
  `read()` syscall on raw log files is reachable from the resource handlers
- INV-DIAG-002 (shared with BC-2.08.008): `_meta.safety_flags` MUST be populated for
  any response containing `untrusted_external` content
- INV-DIAG-001 (shared with BC-2.08.008): Credential values MUST NOT appear in any
  resource response

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `{subsystem}` template variable is not one of the 10 valid names | Structured error: "Resource not found: prism://diagnostics/{value}. Valid subsystems: scheduler, detection, actions, config, plugins, infusions, credentials, fanout, watchdog, storage." |
| `E-MCP-004` | `{trace_id}` template variable is not a valid UUID v7 | Structured error: "Invalid trace_id in resource URI. Expected UUID v7." |
| `E-MCP-DIAG-001` | Resource response exceeds 10 MB | Response truncated with `_meta.truncated: true`; same 10 MB limit as BC-2.08.008 |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-047 | AI reads `prism://diagnostics/summary` before any schedules are configured | `scheduler.active_schedules: 0`; `status: "ok"` (zero is a valid healthy state) |
| EC-08-048 | AI reads `prism://diagnostics/detection` immediately after a config reload that added a new rule | The resource reflects the post-reload rule count; no stale data from the previous configuration |
| EC-08-049 | `prism://diagnostics/trace/{trace_id}` for a trace ID that has no matching log entries | Returns empty `entries: []` with `_meta.trace_id` set; not a 404 error |
| EC-08-050 | AI reads `prism://diagnostics/credentials` — one sensor has a missing credential | Response includes `{client_id: "acme", sensor_id: "cyberint", status: "missing"}`; no credential value; `_meta.trust_level: "internal"` (credential metadata is internal, not sensor-derived) |
| EC-08-051 | Concurrent `resources/list` and `resources/read` calls from 10 AI sessions | Each call is served from the same in-memory diagnostic state with read-only access; no race condition; no state mutation |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://diagnostics/summary` after startup | All 10 subsystems present; no credential values; `trust_level: "internal"` | happy-path |
| Read `prism://diagnostics/detection` then call `get_diagnostics(subsystem: "detection")` | Fields identical (same in-memory state, at most 1s stale) | happy-path |
| Read `prism://diagnostics/credentials` | No secret values; only `status` and `source_type` per entry | happy-path |
| `prism://diagnostics/{invalid_subsystem}` | Structured error listing 10 valid subsystem names | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-024 | Injection scanner: detects known injection patterns | proptest |
| VP-038 | Injection scanner: never panics on arbitrary input strings | fuzz |

## Related BCs

- BC-2.08.008 — `get_diagnostics` Tool (same underlying data, tool interface)
- BC-2.10.008 — MCP Resources for Client List and Sensor Inventory (same resource
  registration pattern)
- BC-2.09.004 — Safety Flag Parallel Fields (injection defense pattern applied here)
- BC-2.09.005 — Trust-Level Metadata Per Response

## Architecture Anchors

- `specs/architecture/observability.md` §"MCP Resource: `prism://diagnostics`" — three
  resource templates, URI structure, relationship to get_diagnostics tool
- `specs/architecture/api-surface.md` — resource template registry
- S-5.08 Task: `server/diagnostics.rs` — resource handler registration alongside tool handler

## Story Anchor

S-5.08 — Diagnostics and `prism logs` CLI

## VP Anchors

Integration test: `tests/diagnostics_tests.rs` — "Read prism://diagnostics/detection and call get_diagnostics(subsystem: 'detection') back-to-back — verify data fields are identical."

Integration test: `tests/diagnostics_tests.rs` — "Read prism://diagnostics/credentials — verify no secret values present for any source type."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-002, DI-004 |
| Story | S-5.08 |
| Priority | P1 |
| Interface | observability.md §prism://diagnostics |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added missing lifecycle fields (deprecated, deprecated_by, replacement, retired, removed, removal_reason); added ## Canonical Test Vectors scaffolding; added ## Changelog. |
| 1.0 | phase-2-patch | 2026-04-16 | product-owner | Initial draft (phase 2-patch addition) |
