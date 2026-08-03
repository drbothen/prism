---
document_type: story
story_id: "S-DEMO-CLAROTY-TIME-001"
title: "Claroty time-window pushdown limitation — subsumed by DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001"
wave: tbd
epic_id: engine-defects
priority: P1
status: draft
version: "0.1"
severity: HIGH
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/stories/DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001-time-window-structurally-impossible.md
origin_finding: "S-DEMO-CLAROTY-TIME-001 (unauthored stub reference in triage table)"
origin_cascade: "D-1889 live-demo triage 2026-07-20"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending — no independent BCs are cited here because this story is
# subsumed. BC-2.01.013 (Datasource Trait Adapter Pattern, status: active,
# lifecycle_status: active) governs the parent defect and is cited there.
# Per S-7.01 gate: behavioral_contracts is empty — status MUST remain draft.
# This stub is a supersession-record artifact; no BC authorship is needed here.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
# tdd_mode: NOT SET — no acceptance criteria will be authored here. This stub
# records a supersession finding, not a specification.
---

# S-DEMO-CLAROTY-TIME-001: Claroty time-window pushdown limitation — subsumed

## Problem

This story ID was referenced in the D-1889 live-demo triage table as "an unauthored stub"
covering the Claroty half of the time-window pushdown structural impossibility. The triage
recorded it as a promise — a name designating work that should be tracked — but no file
was ever created. It existed only as a string in the triage table, not as a real artifact.

## Origin

D-1889 live-demo triage 2026-07-20. The triage entry reads: "time-window pushdown is
structurally impossible for claroty and cyberint; coverage PARTIAL — the claroty half is
an unauthored stub, S-DEMO-CLAROTY-TIME-001."

This registration creates the artifact the triage promised so that the ID is no longer a
dangling reference.

## Subsumed-vs-Distinct Verdict: SUBSUMED

**Verdict:** S-DEMO-CLAROTY-TIME-001 is fully subsumed by
`DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001` and carries no independent scope.

**Rationale:**

1. `DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001` was authored after the triage and
   explicitly absorbs this intent. Its Problem section states verbatim: "The current DEFECT
   stub subsumes that outstanding intent." The sibling writer was aware of this ID and
   made a deliberate design decision to consolidate.

2. The structural impossibility is shared: neither Claroty xDome nor Cyberint exposes a
   query-parameter slot at the `FetchStep` grammar level for time-window pushdown. The root
   cause, the architectural options (silent limitation flag vs. query warning vs. error), and
   the resolution path are identical for both sensors. Splitting them into two stories would
   force two parallel architect adjudications on the same mechanism question and two parallel
   BC amendments to BC-2.01.013 — creating needless coordination overhead.

3. No Claroty-specific behavior distinguishes this from the Cyberint half at the spec or
   implementation level. The defect is not "Claroty-specific time-window pushdown" — it is
   "time-window pushdown is architecturally unavailable for sensors without a time-filter
   parameter slot," and Claroty is one instance of that class. Isolating Claroty into a
   separate story would manufacture an artificial boundary.

**Recommendation:** Record this ID as superseded. The work it intended to hold is tracked
under `DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001`. Future references to
`S-DEMO-CLAROTY-TIME-001` should point to that defect stub.

This registration stub exists solely to prevent the ID from remaining a dangling promise.
Creating substantive scope here — acceptance criteria, ACs, a separate routing path —
would be manufacturing a distinction that does not reflect the actual defect structure.
An honest supersession record is more valuable than a fabricated split.

## Authority

This stub carries no independent authority surface. The governing behavioral contract and
routing for the subsumed work live in `DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001`:

| Artifact | Verbatim status | Role in parent defect |
|----------|-----------------|----------------------|
| BC-2.01.013 (Datasource Trait Adapter Pattern) | `status: active`, `lifecycle_status: active` | Governs `FetchStep` grammar and spec-driven adapter contract; covers the pushdown structural gap for both sensors |

## Routing

No routing action needed beyond the supersession record. All work routes through
`DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001`.

State-manager may record this stub's ID as superseded in STORY-INDEX.md when
`DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001` reaches `status: ready`.

## Scope — NOT YET SPECIFIED

No acceptance criteria, Red Gate tests, task decomposition, or story points will be
authored here. This stub is a supersession-record artifact. The scope for resolving the
underlying defect is fully contained in `DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001`.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage; records subsumed verdict with rationale; no ACs or scope — this ID is fully covered by DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001 |
