---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-18"
capability: "CAP-033"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "1e29f9d"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.005: Partial Report Failure — Failed Sections Include Error Note, Others Delivered

## Description

When a scheduled report action executes multiple PrismQL queries (one per report section),
and one or more queries fail (timeout, OOM, sensor error), the failing sections are replaced
with error notes in the assembled report. The remaining sections containing successful results
are delivered. The full report is always delivered — never silently suppressed due to
a single section failure. This is INV-ACTION-005.

## Preconditions

- A `trigger = "schedule"` action with `[action.destination.report]` contains multiple queries
- The schedule semaphore has been acquired (BC-2.18.004)
- One or more queries fail during execution (timeout, OOM, partial failure from sensor)

## Postconditions

- For each query in the report:
  - **Success:** Section rendered as: `# {title}\n{query results as HTML table or markdown}`
  - **Failure:** Section rendered as: `# {title}\n[Section '{title}' failed: {error_message}]`
- The assembled report contains all sections (successful + error note sections)
- The report is delivered to the destination (email or webhook) regardless of partial failure
- An audit event is emitted: `action_report_delivered` with `total_sections`, `failed_sections`,
  `delivery_status` (delivered/delivery_failed)
- The dirty bit pattern (BC-2.15.005) is applied per-query: dirty bit set before query execution,
  cleared after (including on failure — the dirty bit indicates in-flight, not failure)

## Invariants

- INV-ACTION-005: If one report query fails, include error note in that section; deliver remaining sections
- The report assembly is always complete (all sections present, some with error notes)
- No section is silently omitted — every section in `report.queries` appears in the output
- Report delivery proceeds even if ALL sections fail (full error-note report is delivered)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | Query times out (30s default) | Error note: `"[Section '{title}' failed: query timed out after 30s]"` |
| — | Query exceeds memory budget | Error note: `"[Section '{title}' failed: memory budget exceeded]"` |
| — | Sensor unavailable for one section's query | Error note with partial failure detail from query engine |
| `E-ACTION-008` | Report delivery (email/webhook) itself fails after assembly | At-least-once retry applies to the delivery step (BC-2.18.001); partial content is still delivered |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-015 | Report with 5 sections; all 5 queries fail | Report assembled with 5 error-note sections; still delivered to destination |
| EC-18-016 | Report with 1 section (single query); query fails | Report with 1 error-note section; delivered |
| EC-18-017 | Query succeeds but returns 0 rows | Section rendered with empty table and "No results" note; not treated as failure |
| EC-18-018 | Section order in delivered report | Sections are ordered as declared in `report.queries`; failed sections maintain their position |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-005-happy | 3-section report; all succeed | All 3 sections rendered with results; report delivered | Baseline |
| TV-18-005-partial | 3 sections; query 2 times out | Section 2 has error note; sections 1 and 3 have results; report delivered | AC-7 |
| TV-18-005-all-fail | 5 sections; all queries fail | Report with 5 error-note sections; still delivered | EC-18-015 |
| TV-18-005-zero-rows | Query succeeds with 0 results | Section rendered with empty table; no error note | EC-18-017 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | Partial-report assembly is a trivial Result→RenderedSection map; invariant verified by integration tests TV-18-005-partial and TV-18-005-all-fail in tests/action_tests.rs; no proof harness adds material confidence | — |

## Related BCs

- BC-2.18.004 — Schedule Semaphore (semaphore must be held during all section executions)
- BC-2.18.002 — Schedule Best-Effort Delivery (delivery guarantee tier for schedule triggers)
- BC-2.18.001 — At-Least-Once Delivery (applies to the report delivery step, not individual sections)

## Architecture Anchors

- AD-021: Actions — partial report failure handling
- `specs/architecture/actions.md` — report assembly, section error handling
- S-4.08 Task 8: `action/report.rs`

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-005, AC-7)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Scheduled report with 3 queries where query 2 times out → sections 1 and 3 contain results, section 2 contains error note, report delivered."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-005 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
