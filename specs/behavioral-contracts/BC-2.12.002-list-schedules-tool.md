---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-017"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-017"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.002: `list_schedules` MCP Tool — List Active Schedules with Next Run Times

## Description

The `list_schedules` tool returns summaries of all persisted scheduled queries, including timing metadata (last_run, next_run) and epoch counters per client. It is a read-only tool, always visible in `tools/list`, not gated by write capabilities. Filtering by `client_id` restricts timing metadata to that client only. An audit entry is emitted per DI-004.

## Preconditions
- The `list_schedules` MCP tool is invoked
- Optional parameters: `client_id` (filter to schedules targeting a specific client), `enabled_only` (boolean, default true)

## Postconditions
- Returns an array of schedule summaries, each containing: `name`, `query` (original PrismQL string), `interval`, `splay_percent`, `enabled`, `snapshot`, `removed`, `clients` (targeted client IDs), `last_run` (map of client_id to last execution timestamp or null), `next_run` (map of client_id to next scheduled execution), `epoch` (map of client_id to current epoch counter), `created_at`
- If `client_id` is provided, only schedules that target that client are returned, and `last_run`/`next_run`/`epoch` maps contain only that client's entry
- Schedules are sorted by `next_run` (earliest first) for the first targeted client
- An audit entry is emitted for the tool invocation (DI-004)
- This is a read-only tool -- always visible in `tools/list`, not gated by write capabilities

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted
- DI-008: Client data separation -- when `client_id` is specified, only that client's timing metadata is returned

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `client_id` is not a valid configured client | Structured error with rejected value |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-004 | No schedules exist | Empty array, not an error |
| EC-12-005 | Schedule exists but is disabled | Included only if `enabled_only: false` |
| EC-12-006 | Schedule targets client that was removed from config since creation | Schedule listed with warning annotation; `next_run` for removed client shows `null` |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_schedules()` with 3 active schedules | Array of 3 summaries sorted by next_run | happy-path |
| `list_schedules(client_id="acme")` | Only schedules targeting acme; timing maps contain only acme | happy-path |
| `list_schedules(client_id="nonexistent")` | `Err(E-MCP-004)` | error |
| `list_schedules()` with no schedules | Empty array | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| — | No specific VP; covered by schedule cap enforcement at creation | — |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-017 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
