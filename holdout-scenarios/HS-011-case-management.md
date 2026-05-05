---
document_type: holdout-scenario
level: L3
id: "HS-011"
category: "case-management"
must_pass: false
priority: P0
epic_id: "E-4"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-04T00:00:00Z
phase: 1b
inputs: []
input-hash: null
traces_to: prd.md
behavioral_contracts:
  - BC-2.14.001
  - BC-2.14.002
  - BC-2.14.003
  - BC-2.14.004
  - BC-2.14.005
  - BC-2.14.006
  - BC-2.14.007
  - BC-2.14.008
  - BC-2.14.009
  - BC-2.14.010
  - BC-2.14.012
lifecycle_status: active
introduced: cycle-1
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "Wave 4 holdout group — BC-anchored per D-216."
---

# HS-011: Case Management

**Group:** Case lifecycle (open/update/close), timeline entry idempotency via UUID v7 `timeline_entry_id`, case metrics aggregation per-org, multi-tenant case isolation, and multi-alert case linking for the S-4.06 + S-4.07 subsystem.
**Date:** 2026-05-04
**Priority:** P0

---

## Scenario

Five sub-scenarios covering the Wave 4 case management subsystem (S-4.06 Case Management, S-4.07 Case Metrics). These scenarios validate that cases progress through the 5-state machine (New → Acknowledged → Investigating → Resolved → Closed) with correct 12-transition enforcement (BC-2.14.002); that timeline entries carry a UUID v7 `timeline_entry_id` making case annotation replay-safe (BC-2.14.007); that the `case_metrics` MCP tool returns accurate per-org aggregates including open count, MTTR, and severity distribution (BC-2.14.008/010); that case CF keys are properly org-prefixed per ADR-008 to enforce multi-tenant isolation (BC-2.14.009); and that a single case can reference multiple `alert_ids` through the `acknowledge_alert` and case-linking mechanisms (BC-2.14.012).

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.14.001 | `create_case` MCP Tool — Create Case from One or More Alerts | HS-011-01, HS-011-05 |
| BC-2.14.002 | Case State Transitions — 5-State Machine with 12 Valid Transitions | HS-011-01 |
| BC-2.14.003 | `update_case` MCP Tool — Transition State, Set Disposition, Add Annotation | HS-011-01, HS-011-02 |
| BC-2.14.004 | `list_cases` MCP Tool — Filter by Status, Client, Severity, Assignee | HS-011-03, HS-011-04 |
| BC-2.14.005 | `get_case` MCP Tool — Full Case Detail with Timeline and Linked Alerts | HS-011-01, HS-011-05 |
| BC-2.14.006 | Disposition Assignment — Required on Resolved Transition | HS-011-01 |
| BC-2.14.007 | Timeline Annotations — 5 Types: Note, StatusChange, AlertLink, EvidenceLink, OtImpact | HS-011-02 |
| BC-2.14.008 | TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation | HS-011-03 |
| BC-2.14.009 | Case Persistence — RocksDB Domain for Case State, Timeline, Disposition, Metrics | HS-011-01, HS-011-04 |
| BC-2.14.010 | `case_metrics` MCP Tool — Aggregate MTTD/MTTR and Case Status Counts | HS-011-03 |
| BC-2.14.012 | `acknowledge_alert` MCP Tool — Mark Alert as Acknowledged (Idempotent) | HS-011-05 |

## Verification Approach

Sub-scenarios are driven by the DTU harness constructing cases via the MCP tool layer and asserting state transitions, timeline entries, and persistence in the RocksDB `cases` CF. Case state machine transitions are tested exhaustively against the 12-valid-transition table from BC-2.14.002, with specific focus on invalid backward transitions (Resolved → Acknowledged = rejected) and disposition requirement on the Investigating → Resolved transition (BC-2.14.006).

For HS-011-02 (timeline idempotency), the harness submits the same `update_case` timeline annotation with the same `timeline_entry_id` twice. The second call must be a no-op (idempotent), with the timeline containing exactly one entry for that `timeline_entry_id`. For HS-011-03 (metrics), the harness creates 5 cases across 3 severity levels, closes 3 of them, and asserts `case_metrics` returns accurate counts and MTTR values derived from actual state-transition timestamps.

## Evaluation Rubric

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Case lifecycle (5-state machine, 12 transitions enforced, invalid transitions rejected) | 30% | ≥95% — core behavior |
| Timeline entry idempotency (same timeline_entry_id = no duplicate annotation) | 20% | ≥95% |
| Case metrics accuracy (open count, severity distribution, MTTR within 1% tolerance) | 20% | ≥90% |
| Multi-tenant CF key isolation (no cross-org case leakage) | 20% | 100% — must-check |
| Multi-alert case linking (one case → multiple alert_ids, all linked correctly) | 10% | ≥80% |

Total: 100%. Overall PASS threshold: aggregate weighted score ≥85%. (must_pass: false — no single criterion is 100% mandatory, though multi-tenant check is near-mandatory at 100%.)

## Edge Conditions

- Invalid state transition (e.g., Resolved → New): must return a structured error with `valid_transitions` list; state must remain Resolved (BC-2.14.002).
- Disposition not set when transitioning to Resolved: must return E-CASE-003 or equivalent structured error (BC-2.14.006).
- `case_metrics` with zero cases: must return `{ open: 0, closed: 0, mttd: null, mttr: null }` without panicking (BC-2.14.010).
- Timeline entry for a non-existent case_id: must return 404/not-found error, not a panic.
- Case linked to a non-existent `alert_id`: must return a structured error; case creation should fail gracefully without orphaning the case record.

## Failure Guidance

If HS-011-01 fails (invalid transition not rejected): verify the 12-transition state machine enforcement in S-4.06 and confirm the transition table from BC-2.14.002 is implemented exhaustively. If HS-011-02 fails (duplicate timeline entry): check that `timeline_entry_id` is checked against existing entries in the `cases` CF before insertion (idempotency guard). If HS-011-03 fails (MTTR inaccuracy): verify that TTD/TTI/TTR timestamps are derived from case state-transition events stored in the timeline (BC-2.14.008) rather than wall-clock approximations. If HS-011-04 fails (cross-org leakage): confirm all `cases` CF keys carry `{org_id}:` prefix per ADR-008. Open TDs: none for Wave 4 case management (S-4.06/S-4.07 are clean-sheet).

## Category: architectural-invariant

Source: BC-2.14.002 (5-state machine), BC-2.14.007 (UUID v7 timeline_entry_id), BC-2.14.008 (MTTR computation), ADR-008 (org-prefixed CF keys). Must-pass: false (partial PASS acceptable).

---

## HS-011-01: Case Lifecycle — Open, Update, and Close with State Machine Enforcement

**Title:** A case progresses through New → Acknowledged → Investigating → Resolved → Closed with disposition requirement enforced.

**Preconditions:**
- Prism server initialized with RocksDB `cases` CF
- Alert UUID-A1 exists in the `alerts` CF for OrgA
- OrgA's session active

**Steps:**
1. Call `create_case` with `{ alert_ids: [UUID-A1], severity: "HIGH", org_id: "org-a" }`. Capture `case_id`.
2. Assert case state = New; `get_case` returns full case detail with `linked_alerts: [UUID-A1]`.
3. Call `update_case` to transition to Acknowledged. Assert state = Acknowledged; `StatusChange` timeline entry added.
4. Call `update_case` to transition to Investigating. Assert state = Investigating.
5. Attempt transition to Resolved WITHOUT setting disposition. Assert E-CASE-003 error returned; state remains Investigating.
6. Call `update_case` to set `disposition: "TRUE_POSITIVE"`. Then transition to Resolved. Assert state = Resolved; disposition stored.
7. Attempt backward transition to Acknowledged. Assert rejected with structured error listing valid transitions.
8. Transition to Closed. Assert state = Closed.
9. Assert RocksDB `cases` CF entry reflects Closed state with full timeline (5 StatusChange entries).

**Expected Outcome:**
- All 4 forward transitions succeed in sequence.
- Backward transition (Resolved → Acknowledged) rejected with structured error.
- Disposition required before Resolved transition enforced (E-CASE-003 on attempt without disposition).
- Timeline contains one `StatusChange` entry per valid transition.
- `get_case` returns full detail including all timeline entries and `linked_alerts`.
- Audit log contains `case_state_transition` entries for each successful transition.

**Repos Tested:** prism-operations (S-4.06 case state machine, BC-2.14.001/002/003/005/006), prism-storage (cases CF, BC-2.14.009)

---

## HS-011-02: Timeline Entry Idempotency via timeline_entry_id UUID v7

**Title:** Submitting the same timeline annotation twice with the same `timeline_entry_id` produces exactly one entry (idempotent).

**Preconditions:**
- Case "case-1" in Investigating state for OrgA
- `timeline_entry_id = UUID-T1` (UUID v7) pre-generated

**Steps:**
1. Call `update_case` to add a `Note` annotation with `{ timeline_entry_id: UUID-T1, content: "Initial triage complete", type: "Note" }`.
2. Assert one `Note` timeline entry with `timeline_entry_id = UUID-T1` appears in `get_case` response.
3. Call `update_case` again with the SAME payload (`timeline_entry_id: UUID-T1`, same content).
4. Assert the second call returns success (idempotent — no error).
5. Assert `get_case` returns exactly ONE timeline entry with `timeline_entry_id = UUID-T1` (no duplicate).
6. Assert the `timeline` array length for "case-1" is unchanged after the second call.

**Expected Outcome:**
- First annotation: written to `cases` CF timeline array; UUID-T1 dedup key inserted.
- Second call: idempotent no-op; response indicates success (or `already_exists: true`).
- `get_case` timeline contains exactly one `Note` entry with UUID-T1.
- UUID-T1 is a valid UUID v7 (version bits = 0b0111).
- No audit log entry for the suppressed duplicate annotation.

**Repos Tested:** prism-operations (S-4.06 timeline annotation, BC-2.14.003/007), prism-storage (cases CF idempotency guard, BC-2.14.009)

---

## HS-011-03: Case Metrics Aggregation — Open Count, MTTR, Severity Distribution per Org

**Title:** `case_metrics` returns accurate aggregates for OrgA based on actual state-transition timestamps.

**Preconditions:**
- OrgA has 5 cases:
  - C1: Closed, severity=CRITICAL, created T0, resolved T0+3600s (MTTR=1h)
  - C2: Closed, severity=HIGH, created T0, resolved T0+7200s (MTTR=2h)
  - C3: Closed, severity=HIGH, created T0, resolved T0+1800s (MTTR=0.5h)
  - C4: Investigating, severity=MEDIUM
  - C5: New, severity=LOW
- Timestamps stored in RocksDB `cases` CF as state-transition events (BC-2.14.008)

**Steps:**
1. Call `case_metrics` for `{ org_id: "org-a" }`.
2. Assert response contains:
   - `open_count: 2` (C4 + C5 are non-Closed)
   - `closed_count: 3` (C1 + C2 + C3)
   - `mttd: <value>` (computed from creation to first alert acknowledgment; populated from timeline)
   - `mttr: ~4200s` (mean of 3600, 7200, 1800 = 4200s ±1% tolerance)
   - `severity_distribution: { CRITICAL: 1, HIGH: 2, MEDIUM: 1, LOW: 1 }`
3. Assert all values derived from RocksDB stored timestamps (not wall-clock approximations).
4. Call `case_metrics` for `{ org_id: "org-b" }` (OrgB has zero cases). Assert `{ open: 0, closed: 0, mttd: null, mttr: null }`.

**Expected Outcome:**
- `open_count`, `closed_count`, `severity_distribution` are exact.
- `mttr` within 1% of 4200s (floating-point precision tolerance).
- `mttd` is null if no `AlertLink` timeline entries exist (no detection timestamps recorded).
- OrgB call returns null aggregates without error (zero-case edge condition).
- All metric computations use timestamps from the case timeline CF, not wall clock.

**Repos Tested:** prism-operations (S-4.07 case metrics, BC-2.14.008/010), prism-storage (cases CF timestamp retrieval, BC-2.14.009)

---

## HS-011-04: Multi-Tenant Case Isolation — case_dedup_idx CF Org-Prefixed

**Title:** OrgA's cases are invisible to OrgB; `case_dedup_idx` CF keys carry correct org prefix per ADR-008.

**Preconditions:**
- OrgA has 3 cases (C1, C2, C3) in various states
- OrgB has 2 cases (D1, D2) in New state
- Both orgs' cases persisted to RocksDB `cases` and `case_dedup_idx` CFs

**Steps:**
1. OrgA calls `list_cases`. Assert exactly 3 cases returned (C1, C2, C3).
2. OrgB calls `list_cases`. Assert exactly 2 cases returned (D1, D2).
3. Directly enumerate all `cases` CF keys. Assert OrgA keys begin with `"org-a:"` and OrgB keys begin with `"org-b:"`. No keyless or cross-prefixed entries.
4. Directly enumerate `case_dedup_idx` CF keys. Assert same prefix discipline.
5. Attempt cross-org `get_case` for OrgA's C1 via OrgB session. Assert 404 / not-found.
6. Assert OrgA's C1 `case_id` is not visible in OrgB's `list_cases` response.

**Expected Outcome:**
- `list_cases` returns only calling org's cases.
- All `cases` CF and `case_dedup_idx` CF keys carry correct `{org_id}:` prefix per ADR-008.
- Cross-org `get_case` returns not-found; no case metadata leaked.
- `case_dedup_idx` CF entries are org-isolated (dedup index cannot be queried cross-org).

**Repos Tested:** prism-operations (S-4.06 org-scoped case retrieval, BC-2.14.004/005), prism-storage (cases CF + case_dedup_idx CF ADR-008 key prefix, BC-2.14.009)

---

## HS-011-05: Case Linking to Alerts — One Case References Multiple alert_ids

**Title:** A single case can be linked to multiple alert IDs; all linked alerts appear in `get_case` response.

**Preconditions:**
- Three alerts exist for OrgA: UUID-A1, UUID-A2, UUID-A3 (all in `alerts` CF)
- `acknowledge_alert` tool available (BC-2.14.012)

**Steps:**
1. Call `create_case` with `{ alert_ids: [UUID-A1, UUID-A2], severity: "HIGH", org_id: "org-a" }`. Capture `case_id`.
2. Assert `get_case` returns `linked_alerts: [UUID-A1, UUID-A2]`.
3. Call `acknowledge_alert` with `{ alert_id: UUID-A1, case_id: case_id }`. Assert idempotent (UUID-A1 already linked).
4. Call `acknowledge_alert` with `{ alert_id: UUID-A3, case_id: case_id }` (add third alert).
5. Assert `get_case` returns `linked_alerts: [UUID-A1, UUID-A2, UUID-A3]`.
6. Assert each alert in the `alerts` CF carries `case_id` reference linking back to the case.
7. Assert timeline contains `AlertLink` entry for UUID-A3 with `timeline_entry_id` (UUID v7).

**Expected Outcome:**
- `create_case` accepts multiple `alert_ids` in initial creation.
- `acknowledge_alert` is idempotent for already-linked alerts (UUID-A1 → no duplicate link).
- After adding UUID-A3, `get_case` lists all 3 linked alert IDs.
- Each alert record in `alerts` CF updated with `case_id` back-reference.
- `AlertLink` timeline entry for UUID-A3 carries a UUID v7 `timeline_entry_id`.
- Alert `acknowledge_alert` audit entry present for UUID-A3 (`alert_acknowledged` event with `case_id`).

**Repos Tested:** prism-operations (S-4.06 multi-alert case linking, BC-2.14.001/005/012), prism-storage (alerts CF case_id back-reference, cases CF linked_alerts array, BC-2.14.009)

---

## State Checkpoint

```yaml
scenario_group: HS-011
title: Case Management
scenarios: 5
priority: P0
must_pass: false
wave: 4
stories_covered: [S-4.06, S-4.07]
bcs_anchored:
  - BC-2.14.001
  - BC-2.14.002
  - BC-2.14.003
  - BC-2.14.004
  - BC-2.14.005
  - BC-2.14.006
  - BC-2.14.007
  - BC-2.14.008
  - BC-2.14.009
  - BC-2.14.010
  - BC-2.14.012
key_invariants:
  - 5-state-machine-12-transitions
  - timeline-entry-id-uuid-v7-idempotency
  - disposition-required-before-resolved
  - ADR-008-org-prefixed-cases-cf
status: draft
introduced: cycle-1
```
