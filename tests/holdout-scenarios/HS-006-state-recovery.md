# S-3.6.01 STUB — placeholder anchors, awaiting failing-test phase to validate the new Wave 3 BC mappings
---
document_type: holdout-scenario
level: L3
id: "HS-006"
category: "state-recovery"
must_pass: true
priority: P1
epic_id: "E-3.6"
version: "1.1"
status: draft
producer: stub-architect
timestamp: "2026-04-29T00:00:00Z"
phase: "3.A"
inputs: []
input-hash: null
traces_to: prd.md
# TODO(S-3.6.01 failing-test): replace [] with [BC-3.6.001, BC-3.6.002, BC-3.5.001] after Wave 3 BC anchor validation
behavioral_contracts: []
# TODO(S-3.6.01 failing-test): add closes_td: [TD-HOLDOUT-W2-002] after anchor validation
closes_td: []
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "STUB — S-3.6.01 Red Gate placeholder. Phase 1b BC references retired; Wave 3 anchors (BC-3.6.001, BC-3.6.002, BC-3.5.001) not yet written. Do not merge until failing-test phase installs correct anchors."
---

# HS-006: State Recovery Scenarios (STUB — Wave 3 re-anchor in progress)

**Group:** Multi-tenant harness restart without data loss; per-org state isolation preserved
**Date:** 2026-04-29
**Priority:** P1
**Status:** STUB — awaiting Wave 3 BC anchor validation (S-3.6.01 failing-test phase)

> **NOTE:** This file is the S-3.6.01 Red Gate stub. The sub-scenarios below are
> placeholder outlines only. They intentionally omit the Wave 3 BC IDs, harness
> module names (prism-dtu-harness), and `HarnessError::CloneCrashed` references
> that the acceptance criteria require. The failing-test phase will assert their
> absence (triggering Red Gate) and the implementation phase will fill them in.

---

## HS-006-01: Multi-Tenant Harness Restart (STUB)

**Title:** TODO — replace Phase 1b "Clean Restart with Cursor Resume" with RocksDB + harness restart scenario

**Preconditions:**
- TODO: N customers, each with persisted state in RocksDB
- TODO: harness process stopped; per-org dirty bits tracked

**Steps:**
1. TODO: harness restart procedure (Wave 3 module names)
2. TODO: per-org state loaded from RocksDB
3. TODO: polling resumes per-org

**Expected Outcome:**
- TODO: all org state recovered; no data re-fetch beyond last persisted batch
- TODO: cross-tenant integrity check (devices(OrgA) ∩ devices(OrgB) = ∅)
- TODO: cite BC-3.5.001 postcondition 2 (NOT YET WRITTEN — Red Gate)

**BC Anchors:** TODO — [BC-3.6.001, BC-3.6.002, BC-3.5.001] (stub: not yet installed)

---

## HS-006-02: Clone Crash Recovery (STUB)

**Title:** TODO — replace Phase 1b atomic file write crash with clone task panic mid-operation

**Preconditions:**
- TODO: org A clone task running; org B clone task running independently
- TODO: org A clone panics mid-operation

**Steps:**
1. TODO: clone task panic triggered
2. TODO: harness detects task exit
3. TODO: harness reports error for org A only
4. TODO: org B continues unaffected

**Expected Outcome:**
- TODO: HarnessError::CloneCrashed detected for org A (NOT YET WRITTEN — Red Gate)
- TODO: org B returns HTTP 200; no cross-tenant contamination
- TODO: cite BC-3.6.002 postconditions 1, 2, 3 (NOT YET WRITTEN — Red Gate)

**BC Anchors:** TODO — [BC-3.6.002] (stub: not yet installed)

---

## HS-006-03: Config Change Detection (STUB — Phase 1b content retained pending review)

**Title:** Changed polling config detected on restart, cursor reset to avoid stale data

> TODO: review whether this Phase 1b scenario is still valid under Wave 3 module
> names (prism-dtu-harness). Retain behavioral assertion if Wave 3 also implements
> query fingerprint validation; update module references accordingly.

**Preconditions:**
- TODO: review for Wave 3 applicability

**Expected Outcome:**
- TODO: update to Wave 3 module names if retained; remove if superseded

**BC Anchors:** TODO (stub: not yet determined)

---

## HS-006-04: Forward Progress Invariant (STUB — Phase 1b content retained pending review)

**Title:** Prism rejects any attempt to move cursor backward

> TODO: review whether this Phase 1b scenario maps to a Wave 3 BC. Retain if
> applicable; update module references to prism-dtu-harness.

**Preconditions:**
- TODO: review for Wave 3 applicability

**Expected Outcome:**
- TODO: update to Wave 3 module names if retained

**BC Anchors:** TODO (stub: not yet determined)

---

## HS-006-05: Batch Receipt Audit Trail (STUB — Phase 1b content retained pending review)

**Title:** Batch receipts provide audit trail for delivered data across restarts

> TODO: review whether batch receipt patterns survive into Wave 3 RocksDB model.
> Remove or update to cite applicable Wave 3 BCs.

**Preconditions:**
- TODO: review for Wave 3 applicability

**Expected Outcome:**
- TODO: update or remove based on Wave 3 BC review

**BC Anchors:** TODO (stub: not yet determined)

---

## HS-006-06: Multi-Tenant State Recovery After System-Wide Restart (STUB)

**Title:** TODO — re-anchor to prism-dtu-harness N-customer restart (replace poller-bear/coaster refs)

**Preconditions:**
- TODO: N customers with established state in RocksDB
- TODO: system-wide harness process restart

**Steps:**
1. TODO: harness receives shutdown signal
2. TODO: per-org state persisted to RocksDB
3. TODO: harness restarts; all orgs' state loaded
4. TODO: polling resumes per-org

**Expected Outcome:**
- TODO: all orgs recover; no data loss beyond last persisted batch
- TODO: cross-tenant integrity (BC-3.5.001 postcondition 2) — NOT YET WRITTEN
- TODO: credential re-acquisition per Wave 3 model

**BC Anchors:** TODO — [BC-3.6.001, BC-3.6.002, BC-3.5.001] (stub: not yet installed)

---

## HS-006-07: Per-Org Failure Injection Triggers Crash Detection (STUB — new scenario)

**Title:** TODO — inject InternalError on org A; verify CloneCrashed for org A; org B unaffected

**Preconditions:**
- TODO: org A and org B clones running in harness
- TODO: failure injection layer available (BC-3.6.001 — NOT YET WRITTEN)

**Steps:**
1. TODO: call inject_failure(org_slug, dtu_type, FailureMode::InternalError) for org A
2. TODO: org A clone crashes
3. TODO: harness detects CloneCrashed for org A
4. TODO: org B continues returning HTTP 200

**Expected Outcome:**
- TODO: HarnessError::CloneCrashed for org A only (BC-3.6.002 — NOT YET WRITTEN)
- TODO: BC-3.6.001 per-org scoping guarantee verified (NOT YET WRITTEN)
- TODO: no cross-tenant effect

**BC Anchors:** TODO — [BC-3.6.001, BC-3.6.002] (stub: not yet installed)

---

## State Checkpoint (STUB)

```yaml
scenario_group: HS-006
title: State Recovery (Wave 3 re-anchor — STUB)
# TODO(S-3.6.01 failing-test): update scenarios count to 7 after HS-006-07 added
scenarios: 7
priority: P1
# TODO(S-3.6.01 failing-test): replace [] with [BC-3.6.001, BC-3.6.002, BC-3.5.001, BC-3.2.001, BC-3.2.003]
behavioral_contracts: []
# TODO(S-3.6.01 failing-test): replace [] with [TD-HOLDOUT-W2-002]
closes_td: []
status: stub
```
