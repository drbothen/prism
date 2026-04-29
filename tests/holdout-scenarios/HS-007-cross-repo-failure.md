# S-3.6.02 STUB — placeholder anchors, awaiting failing-test phase to validate the new Wave 3 BC mappings
---
document_type: holdout-scenario
level: L3
id: "HS-007"
category: "cross-repo-failure"
must_pass: true
priority: P1
epic_id: "E-3.6"
version: "1.1"
status: draft
producer: stub-architect
timestamp: "2026-04-29T00:00:00Z"
phase: "1b"
inputs: []
input-hash: null
traces_to: prd.md
# TODO(S-3.6.02 failing-test): replace [] with [BC-3.6.001, BC-3.6.002, BC-3.5.002] after Wave 3 BC anchor validation
behavioral_contracts: []
# TODO(S-3.6.02 failing-test): add closes_td: [TD-HOLDOUT-W2-NNN] if applicable after anchor validation
closes_td: []
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "STUB — S-3.6.02 Red Gate placeholder. Phase 1b BC references retired; Wave 3 anchors (BC-3.6.001, BC-3.6.002, BC-3.5.002) not yet written. Do not merge until failing-test phase installs correct anchors."
---

# HS-007: Cross-Repo Failure Scenarios (STUB — Wave 3 re-anchor in progress)

**Group:** Per-customer DTU failure isolation; customer A fails, customer B unaffected
**Date:** 2026-04-29
**Priority:** P1
**Status:** STUB — awaiting Wave 3 BC anchor validation (S-3.6.02 failing-test phase)

> **NOTE:** This file is the S-3.6.02 Red Gate stub. The sub-scenarios below are
> placeholder outlines only. They intentionally omit the Wave 3 BC IDs, harness
> module names (`prism-dtu-harness`), `IsolationMode::Network`, and
> `HarnessError::CloneCrashed` references that the acceptance criteria require.
> The failing-test phase will assert their absence (triggering Red Gate) and the
> implementation phase will fill them in.

---

## HS-007-01: Cross-Customer Failure Isolation (STUB)

**Title:** TODO — inject per-org failure on org A; verify org B unaffected in Network harness

**Preconditions:**
- TODO: two orgs registered in `IsolationMode::Network` harness (BC-3.5.002)
- TODO: per-org `FailureLayerShared` state initialized

**Steps:**
1. TODO: call `inject_failure(org_slug_A, dtu_type, FailureMode::AuthReject)` for org A
2. TODO: org A DTU queries return HTTP 401
3. TODO: org B DTU queries return HTTP 200 with valid data
4. TODO: call `clear_failure(org_slug_A, dtu_type)`; org A resumes HTTP 200

**Expected Outcome:**
- TODO: org A isolated failure; org B `FailureLayerShared` state unchanged
- TODO: cite BC-3.6.001 postconditions 1, 2, 3 (NOT YET WRITTEN — Red Gate)

**BC Anchors:** TODO — [BC-3.6.001, BC-3.5.002] (stub: not yet installed)

---

## HS-007-02: Routing Bug Detection via Network Mode (STUB)

**Title:** TODO — wrong-org credentials to live clone endpoint return HTTP 401

**Preconditions:**
- TODO: org A and org B clones running on separate `SocketAddr`s (`IsolationMode::Network`)

**Steps:**
1. TODO: send org A credentials to org B `SocketAddr`
2. TODO: assert HTTP 401 response (routing bug detectable)

**Expected Outcome:**
- TODO: cross-process credential routing rejected (BC-3.5.002 postcondition 2 — NOT YET WRITTEN)

**BC Anchors:** TODO — [BC-3.5.002] (stub: not yet installed)

---

## HS-007-03: Crash Detection After Auth-Reject (STUB)

**Title:** TODO — inject InternalError; verify HarnessError::CloneCrashed for org A within 1s

**Preconditions:**
- TODO: org A and org B clones running in `prism-dtu-harness`

**Steps:**
1. TODO: inject `FailureMode::InternalError` on org A clone
2. TODO: org A clone panics
3. TODO: harness detects `HarnessError::CloneCrashed` for org A within 1s
4. TODO: org B continues returning HTTP 200

**Expected Outcome:**
- TODO: `HarnessError::CloneCrashed` for org A only (BC-3.6.002 — NOT YET WRITTEN)
- TODO: no cross-tenant effect on org B

**BC Anchors:** TODO — [BC-3.6.002] (stub: not yet installed)

---

## State Checkpoint (STUB)

```yaml
scenario_group: HS-007
title: Cross-Repo Failure (Wave 3 re-anchor — STUB)
# TODO(S-3.6.02 failing-test): update scenarios count to 3 after HS-007-01/02/03 finalized
scenarios: 3
priority: P1
# TODO(S-3.6.02 failing-test): replace [] with [BC-3.6.001, BC-3.6.002, BC-3.5.002]
behavioral_contracts: []
# TODO(S-3.6.02 failing-test): replace [] with [TD-HOLDOUT-W2-NNN] if applicable
closes_td: []
status: stub
```
