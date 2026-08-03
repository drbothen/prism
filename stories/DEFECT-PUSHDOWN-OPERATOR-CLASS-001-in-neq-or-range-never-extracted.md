---
document_type: story
story_id: "DEFECT-PUSHDOWN-OPERATOR-CLASS-001"
title: "IN / != / OR / range predicates never extracted; classify_predicates not called in execution path"
wave: tbd
epic_id: engine-defects
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pushdown-audit.md
severity: MED
behavioral_contracts: []
# BC status: No BC anchor for this defect at triage time. The triage anchors it to
# ADR-022 §C (fan-out orchestration and per-sensor pushdown integration scope).
# behavioral_contracts is empty: per S-7.01 gate, status must remain draft and
# must not be promoted to ready until a product-owner authors a BC that covers this
# pushdown operator class gap. See § Authority below.
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
origin_finding: "DEFECT-PUSHDOWN-OPERATOR-CLASS-001 (G6)"
origin_cascade: "D-1889 live-demo triage 2026-07-20"
cycle: "v1.0.0-greenfield"
phase: 3
# tdd_mode: NOT SET — tdd_mode and Red Gate enumeration (RG-001..RG-NNN) are
# deferred to specification time per SAC-1. Setting tdd_mode: strict on a
# registration stub with no acceptance criteria would create an immediate SAC-1
# violation (no enumerated RG list, no BC-5.38.001 density check).
track: "Platform Engineering"
subsystems: [SS-11]
---

# DEFECT-PUSHDOWN-OPERATOR-CLASS-001: IN / != / OR / range predicates never extracted; classify_predicates not called in execution path

## Problem

The predicate pushdown layer handles equality predicates only. The following predicate
classes are never extracted from the WHERE clause and therefore never pushed down to
sensor API parameters:

- `IN` (membership test, e.g., `status IN ('open', 'closed')`)
- `!=` (not-equal, e.g., `severity != 'low'`)
- `OR` (logical disjunction across conditions)
- range predicates (e.g., `score BETWEEN 7 AND 10`, `ts > X AND ts < Y`)

**Symbol verification:** `classify_predicates` is defined as a public function in
`prism-query::pushdown`. The triage claims it is "never called." Verification finds:

- The function IS called from `prism-query::explain` for query-explain output, and from
  test modules and Kani proof harnesses.
- A comment in `pushdown.rs` at the `build_pushdown_params` call site documents an
  explicit deferral: "Per-sensor `classify_predicates` integration deferred to wave-5
  when ColumnSpec…". The comment is in the production execution path, not a test.
- The triage claim is therefore accurate with respect to the production query execution
  path: `classify_predicates` is NOT called during actual query execution to drive
  pushdown. Explain-mode and proof-harness calls are not substitutes for production
  wiring.

The result: IN/!=/OR/range predicates are evaluated post-fetch (client-side filtering
over the full upstream dataset), even when the sensor API supports equivalent filter
parameters.

The triage records coverage as WEAK because BC-2.11.007 covers equality pushdown but
the operator-class gap has no BC anchor at present.

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G6 (MED).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "IN/!=/OR/range predicates are never extracted; `classify_predicates` is
never called; ADR-022 §C."

## Authority

**Architecture decisions:**

| ADR | Title | Frontmatter status |
|-----|-------|-------------------|
| ADR-022 | Production Runtime Wiring — prism-bin Chassis | status: ACCEPTED |

ADR-022 §C governs fan-out orchestration and the integration scope for
per-sensor pushdown. The deferral of `classify_predicates` integration was recorded
in ADR-022 §C scope as a wave-5 item. The defect registration here is the
prerequisite for scheduling that integration work.

**No BC anchor for this defect exists at triage time.** The `behavioral_contracts`
array is intentionally empty (S-7.01 gate: status must remain `draft` until a
product-owner authors and anchors a BC). The architect adjudication step will
determine whether to anchor this to an amended BC-2.11.007 or to a new BC. See
§Routing.

## Routing

**Architect adjudication precedes story decomposition.**

Route: `architect` → `story-writer`

The architect must adjudicate:
(a) which operator classes (`IN`, `!=`, `OR`, range) are candidates for pushdown wiring
    against which sensor API parameter slots, given current `FetchStep` grammar
    capabilities,
(b) whether `classify_predicates` production-path wiring requires a new BC or an
    amendment to an existing one (e.g., BC-2.11.007),
(c) whether any ADR-022 §C amendment is required to update the wave-5 scope note to
    reflect the new story ID, and
(d) whether this story covers the full operator-class surface or only a subset.

After architect adjudication, the product-owner authors the BC anchor, and the
story-writer produces acceptance criteria.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect in the tracking system and document
the verified symbol-level finding.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G6; verified classify_predicates symbol and production-path deferral |
