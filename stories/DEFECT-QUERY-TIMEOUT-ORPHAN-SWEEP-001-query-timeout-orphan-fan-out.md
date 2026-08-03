---
document_type: story
story_id: "DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001"
title: "Query timeout drops parent future while detached fan-out tasks continue hitting live tenant API"
wave: wave-c
epic_id: engine-defects
priority: P0
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pushdown-audit.md
severity: CRIT/HIGH
behavioral_contracts:
  - BC-2.11.006
# BC status: BC-2.11.006 carries status: draft, lifecycle_status: active.
# Per S-7.01 gate: behavioral_contracts is non-empty, so draft status is valid.
# BC authorship must be confirmed by product-owner before status=ready.
# BC array propagation: AC↔BC bidirectional traces are deferred to specification time.
verification_properties: []
depends_on: []
blocks: []
origin_finding: "DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001 (G4)"
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

# DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001: Query timeout drops parent future while detached fan-out tasks continue hitting live tenant API

## Problem

When a query timeout fires, the parent future is cancelled and the caller receives a
timeout error. However, fan-out tasks spawned via `tokio::spawn` in `fanout.rs` are
detached from the parent future's cancellation tree. These detached tasks continue
executing against the live tenant API after the parent has been dropped.

Two compounding behaviors follow from this:

1. **Resource exhaustion:** detached tasks hold HTTP semaphore permits and issue API
   calls against a tenant whose query has already been abandoned. Under repeated
   timeout-then-retry patterns this accumulates live API calls that never resolve to
   any query result.

2. **Offset corruption on retry:** the retry path re-sweeps from offset 0, because the
   pagination state accumulated by the detached (now-orphaned) tasks is not available
   to the new query execution. The effective result is that a timed-out query followed
   by a retry issues duplicate upstream requests and may return inconsistent data.

This defect was split from DEFECT-PAGINATION-ROW-BUDGET-001 (F12) during the D-1889
triage because it represents a distinct structural failure mode (task-scope leakage on
cancellation) rather than a row-budget accounting failure.

## Origin

D-1889 live-demo triage 2026-07-20 — Bucket B Engine Defects, finding G4 (CRIT/HIGH).
Source readings: `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
and `findings/prism-pushdown-audit.md`.

Triage entry: "query timeout drops parent future; detached `tokio::spawn` fan-out keeps
hitting live tenant API; retry re-sweeps offset 0."

## Authority

**Behavioral contracts:**

| BC | Title | Frontmatter status | Governing clause |
|----|-------|-------------------|-----------------|
| BC-2.11.006 | Query Security Limits | status: draft; lifecycle_status: active | Governs per-query timeout and resource-limit enforcement. The orphan fan-out constitutes a resource-limit violation: permits and upstream API capacity are consumed after the limit fires. |

**Architecture decisions:**

| ADR | Title | Frontmatter status |
|-----|-------|-------------------|
| ADR-022 | Production Runtime Wiring — prism-bin Chassis | status: ACCEPTED |

ADR-022 §C governs fan-out orchestration and the wiring contracts for async tasks. The
orphan-sweep defect is a violation of the cancellation discipline implied by §C.

**No other ADR governs cancellation scope for `tokio::spawn` tasks in this codebase at
present.** If the architect determines that a new ADR is required to formalize query
cancellation contracts, that decision is made during the architect adjudication step
(see §Routing).

## Routing

**Architect adjudication precedes implementation.**

Route: `architect` → `implementer`

The architect must adjudicate whether:
(a) the fix is structural wiring (replace bare `tokio::spawn` with structured concurrency,
    e.g., `JoinSet` or a cancellation token threaded into fan-out tasks), or
(b) a scope-guard mechanism in the fan-out orchestrator is sufficient, or
(c) an ADR amendment to ADR-022 §C is required to formalize the decision.

After architect adjudication, the story-writer produces acceptance criteria and the
implementer proceeds under TDD.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, architecture mapping, edge cases, and task breakdown are deferred to specification
time. This stub exists solely to register the defect in the tracking system and prevent
it from being silently lost.

`tdd_mode` and Red Gate enumeration will be set by the spec-authoring agent (per SAC-1)
when acceptance criteria are authored. The routing chain is:
`architect adjudication → product-owner (BC-2.11.006 amendment if needed) → story-writer
(AC authorship) → test-writer + implementer`.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage G4 |
