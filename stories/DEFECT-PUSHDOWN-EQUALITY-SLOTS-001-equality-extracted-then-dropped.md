---
document_type: story
story_id: "DEFECT-PUSHDOWN-EQUALITY-SLOTS-001"
title: "Equality predicates extracted by pushdown planner but silently dropped — no spec slots exist to receive them"
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
behavioral_contracts:
  - BC-2.11.007
# BC status: BC-2.11.007 carries status: active, lifecycle_status: active.
# Per S-7.01 gate: behavioral_contracts is non-empty, so draft status is valid.
# BC authorship of new postconditions (if required for the "no-slot" case) must be
# confirmed by product-owner before status=ready.
# BC array propagation: AC↔BC bidirectional traces are deferred to specification time.
verification_properties: []
depends_on: []
blocks: []
origin_finding: "DEFECT-PUSHDOWN-EQUALITY-SLOTS-001 (G7)"
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

# DEFECT-PUSHDOWN-EQUALITY-SLOTS-001: Equality predicates extracted by pushdown planner but silently dropped — no spec slots exist to receive them

## Problem

The pushdown planner correctly extracts equality predicates from a WHERE clause. However,
after extraction, the predicates are silently dropped because no spec-level parameter
slots exist in the current `FetchStep` grammar to receive them. The extracted predicates
are neither pushed down to the sensor API parameters nor surfaced as a query warning —
they disappear from the execution path without any observable feedback.

The consequence is that a query author writing `WHERE sensor_field = 'value'` has no
indication that the equality predicate was extracted, found no pushdown slot, and was
therefore discarded. The query executes as a full upstream fetch followed by client-side
filtering, with the user believing (incorrectly) that the predicate may have been pushed
down.

This differs from DEFECT-PUSHDOWN-OPERATOR-CLASS-001 (G6), which concerns predicate
operator classes (`IN`, `!=`, `OR`, range) that are never extracted. G7 concerns equality
predicates that ARE extracted but then dropped due to a slot-availability gap.

The triage records coverage as NEW (no existing story or BC postcondition covers the
extracted-but-dropped case for equality predicates).

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G7 (MED).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "equality predicates are extracted and then dropped because no spec slots
exist to receive them; BC anchor: BC-2.11.007."

## Authority

**Behavioral contracts:**

| BC | Title | Frontmatter status | Governing clause |
|----|-------|-------------------|-----------------|
| BC-2.11.007 | Sensor Filter Push-Down | status: active; lifecycle_status: active | Governs how predicates from the WHERE clause are pushed down to sensor API parameters. The defect is a gap in this BC: it specifies pushdown for cases where a slot exists, but does not specify behavior when a slot is absent for an extracted equality predicate. |

No ADR governs the "extracted-but-no-slot" behavioral surface at present. Whether an
ADR amendment is needed is a question for architect adjudication (see §Routing).

## Routing

**Architect adjudication precedes story decomposition.**

Route: `architect` + `product-owner` → `story-writer`

The architect must adjudicate:
(a) whether the "no slot for extracted equality predicate" case should be surfaced as a
    query warning, logged as a diagnostic event, or handled silently with documentation,
(b) whether the `FetchStep` grammar should be extended to add slots for common equality
    pushdown parameters across sensor adapters, or
(c) whether a per-sensor slot-declaration mechanism should be added to the spec grammar.

The product-owner must:
(a) amend BC-2.11.007 to add a postcondition or invariant for the "extracted but no slot"
    case, or author a new BC covering this surface.

After both adjudications, the story-writer produces acceptance criteria.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect in the tracking system.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G7 |
