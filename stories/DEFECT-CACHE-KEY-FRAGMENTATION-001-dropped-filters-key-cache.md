---
document_type: story
story_id: "DEFECT-CACHE-KEY-FRAGMENTATION-001"
title: "Dropped filters still contribute to cache key, causing duplicate fetches"
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
  - BC-2.07.005
# BC status: BC-2.07.005 carries status: draft, lifecycle_status: active.
# Per S-7.01 gate: behavioral_contracts is non-empty, so draft status is valid.
# BC authorship must be confirmed by product-owner before status=ready.
# BC array propagation: AC↔BC bidirectional traces are deferred to specification time.
verification_properties: []
depends_on: []
blocks: []
origin_finding: "DEFECT-CACHE-KEY-FRAGMENTATION-001 (G5)"
origin_cascade: "D-1889 live-demo triage 2026-07-20"
cycle: "v1.0.0-greenfield"
phase: 3
# tdd_mode: NOT SET — tdd_mode and Red Gate enumeration (RG-001..RG-NNN) are
# deferred to specification time per SAC-1. Setting tdd_mode: strict on a
# registration stub with no acceptance criteria would create an immediate SAC-1
# violation (no enumerated RG list, no BC-5.38.001 density check).
track: "Platform Engineering"
subsystems: [SS-07, SS-11]
---

# DEFECT-CACHE-KEY-FRAGMENTATION-001: Dropped filters still contribute to cache key, causing duplicate fetches

## Problem

When the pushdown planner drops a filter (because the sensor API has no slot to receive
it or the predicate class is not supported for pushdown), the dropped filter expression
still participates in the cache key derivation. As a result, two queries that are
semantically equivalent from the sensor API's perspective — one with a dropped filter
and one without — produce different cache keys and trigger separate upstream fetches,
bypassing cache hits.

This fragments the cache unnecessarily: a query with a filter the sensor cannot push
down will never reuse results cached by a prior equivalent query, even when the sensor
returns the same data for both. Under repeated or parallel queries with varied filter
expressions that collapse to the same upstream request, this causes redundant API calls
against live tenant infrastructure.

The triage records this as ADJACENT coverage because BC-2.07.005 (Cache Key Derivation)
exists but does not currently specify the behavior for dropped-filter contributions to
the key.

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G5 (MED).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "dropped filters still contribute to the cache key, causing duplicate
fetches; BC anchor: BC-2.07.005 (draft)."

## Authority

**Behavioral contracts:**

| BC | Title | Frontmatter status | Governing clause |
|----|-------|-------------------|-----------------|
| BC-2.07.005 | Cache Key Derivation | status: draft; lifecycle_status: active | Governs how cache keys are derived from query parameters. The defect is that dropped (non-pushed-down) predicates participate in key derivation when they should be excluded from the key because they do not affect the upstream fetch. |

No ADR governs cache key derivation policy at present. If the product-owner determines
that BC-2.07.005 requires a new postcondition or amendment to specify the dropped-filter
exclusion rule, that authorship occurs during the routing chain below.

## Routing

Route: `product-owner` → `implementer`

The product-owner must:
(a) amend BC-2.07.005 (or author a new postcondition) to specify that predicates dropped
    by the pushdown planner must be excluded from the upstream cache key, and
(b) confirm whether the fix applies to the full cache key surface or only to the
    upstream-fetch portion of the key.

After product-owner authorship, the implementer proceeds under TDD.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect in the tracking system.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G5 |
