---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-14"
capability: "CAP-022"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "ac6b633"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.005: `get_case` MCP Tool — Full Case Detail with Timeline and Linked Alerts

## Description

The `get_case` MCP tool returns the complete case record for a specific case, including
all core fields, the full ordered timeline, all annotations, alert summaries for every
linked alert, and on-demand computed metrics (TTD, TTI, TTR, time in current status).
It is a read-only tool always visible in `tools/list` and enforces client scoping.

Deleted alerts are gracefully handled by returning `deleted: true` in the alert summary
rather than producing an error, ensuring the tool remains useful even when alert records
are purged.

## Preconditions
- The `get_case` MCP tool is invoked with required parameters: `case_id` (UUID), `client_id`

## Postconditions
- Returns the full case record including:
  - **Core fields:** `id`, `client_id`, `title`, `description`, `status`, `severity`, `assignee`, `disposition` (with per-variant metadata), `created_at`, `updated_at`, `closed_at`
  - **Linked alerts:** full array of `source_alert_ids` with summary for each alert (rule_name, severity, created_at, title)
  - **Timeline:** complete array of timeline entries in chronological order, each with: `event_type`, `description`, `actor`, `timestamp`
  - **Annotations:** complete array of annotations, each with: `type`, `content`, `author`, `timestamp`
  - **Metrics:** `mttd` (mean time to detect, if calculable), `mttr` (mean time to resolve, if case is resolved/closed), `time_in_current_status` (duration since last status change)
- The case must belong to the specified `client_id`
- An audit entry is emitted (DI-004)
- This is a read-only tool -- always visible in `tools/list`

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- case must belong to the specified client_id

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-001` | Case does not exist | Structured error |
| `E-CASE-008` | Case belongs to a different client than specified | Structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-016 | Case with 100+ timeline entries | All entries returned; no pagination within a single case's timeline |
| EC-14-017 | Linked alert was deleted (orphaned reference) | Alert summary shows `deleted: true` with original alert_id; no error |
| EC-14-018 | Case in New status with no annotations, no linked alerts | All arrays empty; metrics show `mttd: null`, `mttr: null` |
| EC-14-019 | Resolved case | `mttr` calculated as `resolved_at - created_at`; `mttd` calculated if linked alerts have `created_at` earlier than case `created_at` |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — minimal case | `case_id=X, client_id=acme` (New, no alerts, no annotations) | Full record; empty arrays; mttd=null, mttr=null |
| Happy path — resolved case | case in Resolved state with linked alerts | mttr = resolved_at - created_at; alert summaries included |
| Orphaned alert | alert in source_alert_ids was deleted | Alert summary with `deleted: true`, no error |
| Case not found | `case_id` does not exist | `E-CASE-001` |
| Wrong client | `case_id` belongs to different client | `E-CASE-008` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Metric computation invariants covered by VP-054 (BC-2.14.008); orphaned-alert handling is integration behavior; both covered by integration tests in S-4.06 test suite. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
