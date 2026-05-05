---
document_type: holdout-scenario
level: L3
id: "HS-010"
category: "detection-alert-pipeline"
must_pass: true
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
  - BC-2.12.005
  - BC-2.12.007
  - BC-2.13.001
  - BC-2.13.002
  - BC-2.13.003
  - BC-2.13.005
  - BC-2.13.006
  - BC-2.13.008
  - BC-2.13.011
  - BC-2.13.012
  - BC-2.13.013
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

# HS-010: Detection & Alert Pipeline

**Group:** Detection rule registration, diff pack flow, alert generation with UUID v7 idempotency keys, multi-tenant rule isolation, alert deduplication, and rule disable semantics for the S-4.02–S-4.05 pipeline.
**Date:** 2026-05-04
**Priority:** P0

---

## Scenario

Six sub-scenarios covering the Wave 4 detection and alert pipeline (S-4.02 Diff/Result Packs, S-4.03 Detection Rules, S-4.04 Detection Evaluation, S-4.05 Alert Generation). These scenarios validate that detection rules load and match correctly against query result packs (BC-2.13.001/002); that diff packs correctly surface added/removed records across consecutive result epochs (BC-2.12.005/007); that alerts are generated with `alert_id` (UUID v7) as the idempotency key making the pipeline replay-safe (BC-2.13.005/013); that detection rules and their results are scoped per org to prevent cross-tenant rule firing (BC-2.13.011/012); that the deduplication guard suppresses re-emissions on the same `alert_id` (BC-2.13.013); and that disabling a rule stops new alert generation without modifying existing alerts.

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.12.005 | Differential Result Computation — Hash Previous Results, Return Added/Removed | HS-010-02 |
| BC-2.12.007 | `get_diff_results` MCP Tool — Retrieve Differential Results for a Scheduled Query | HS-010-02 |
| BC-2.13.001 | Detection Rule Loading — Parse PrismQL Predicate, Validate at Load Time, Reject Invalid | HS-010-01 |
| BC-2.13.002 | Single-Event Detection — Evaluate Rule Predicate Against Each Differential Record | HS-010-01, HS-010-04 |
| BC-2.13.003 | Correlation Detection — Threshold Over Sliding Window with Group-By, Reset-After-Fire | HS-010-04 |
| BC-2.13.005 | Alert Generation — Interpolate Template, Persist Alert, Broadcast via MCP Notification | HS-010-03, HS-010-05, HS-010-06 |
| BC-2.13.006 | `create_rule` MCP Tool — Create Detection Rule with Scope | HS-010-01, HS-010-06 |
| BC-2.13.008 | `delete_rule` MCP Tool — Remove Rule (Confirmation for Global Rules) | HS-010-06 |
| BC-2.13.011 | Three-Scope Rule Resolution — Global Baseline + Per-Client Overrides + Analyst Ad-Hoc | HS-010-04 |
| BC-2.13.012 | Detection State Persistence — RocksDB Domain for Correlation Windows, Alert History | HS-010-04, HS-010-05 |
| BC-2.13.013 | Alert Deduplication — Per-Match-Mode Dedup Keys Prevent Duplicate Alerts | HS-010-05 |

## Verification Approach

Sub-scenarios are driven by the DTU harness feeding pre-fabricated query result packs (epoch N and epoch N+1) into the diff engine and detection pipeline. Alert emission is verified by asserting the presence of structured alert records in the `alerts` RocksDB CF and by inspecting MCP notification broadcasts. UUID v7 `alert_id` generation is verified using a UUID v7 parser asserting that the timestamp component advances monotonically.

For HS-010-04 (multi-tenant isolation), two org contexts run simultaneously. Org A's detection rules are scoped to `org_id: "org-a"` in the RocksDB `detection_state` CF; the harness verifies that Org B's differential results do not trigger Org A's rules, and vice versa. For HS-010-05 (deduplication), the same diff pack is fed to the detection engine twice with the same epoch contents; the harness asserts that a second `alert_id` for the same event is NOT written to the `alerts` CF.

## Evaluation Rubric

| Criterion | Weight | Pass Threshold |
|-----------|--------|----------------|
| Detection rule load + single-event matching correctness | 25% | 100% — must-pass |
| Diff pack flow (added/removed records per epoch) | 20% | 100% — must-pass |
| Alert UUID v7 idempotency key (alert_id is UUID v7; same event → same dedup key) | 20% | 100% — must-pass |
| Multi-tenant rule isolation (no cross-org rule firing) | 20% | 100% — must-pass |
| Alert deduplication (same alert_id not written twice) | 10% | ≥90% |
| Rule disable stops new alerts without touching existing | 5% | ≥80% |

Total: 100%. Overall PASS threshold: all must-pass criteria at 100%, aggregate weighted score ≥85%.

## Edge Conditions

- Detection rule with invalid PrismQL predicate: must be rejected at `create_rule` time with a structured parse error (BC-2.13.001 load-time validation).
- Same event matching two different single-event rules: two distinct alerts generated (different `rule_id`, different `alert_id`); deduplication is per-rule, not per-event (EC-13-017 per BC-2.13.013).
- Dedup index read failure from RocksDB: fail-open — alert is persisted despite inability to check dedup (E-DETECT-010 per BC-2.13.013).
- Correlation rule `reset_after_fire: true` with window bucket rollover: group key fires once per window bucket; next window bucket fires again (BC-2.13.003 reset-after-fire semantics).
- Empty diff pack (no added, no removed records): detection engine evaluates zero records; no alerts generated; epoch still incremented.

## Failure Guidance

If HS-010-01 fails (rule not matching): verify the rule-to-SQL compilation path (BC-2.13.009) produces a valid DataFusion WHERE clause and that the column family names in the diff result schema match the rule's field references. If HS-010-03 fails (alert_id not UUID v7): check alert generation in S-4.05 and confirm `uuid::Uuid::now_v7()` is used rather than v4. If HS-010-04 fails (cross-org rule firing): verify `three-scope rule resolution` (BC-2.13.011) applies org_id scoping at the rule lookup layer and that detection state CF keys are prefixed with `{org_id}:` per ADR-008. If HS-010-05 fails (duplicate alerts): check dedup key insertion and TTL logic in BC-2.13.013 implementation (S-4.04). Open TDs: none for Wave 4 detection engine.

## Category: architectural-invariant

Source: BC-2.13.005 (alert_id UUID v7 idempotency), BC-2.13.013 (deduplication), ADR-008 (org-scoped CF keys), BC-2.13.011 (three-scope rule resolution). Must-pass: true.

---

## HS-010-01: Detection Rule Registration and Matching Against Query Result Packs

**Title:** A detection rule registered via `create_rule` correctly fires when its predicate matches a differential record.

**Preconditions:**
- Prism detection engine initialized; RocksDB `detection_state` CF ready
- OrgA's session active
- One pack "alerts-pack" associated with schedule "s1"

**Steps:**
1. Call `create_rule` with `{ name: "high-severity-alert", predicate: "severity >= 8", scope: "org", org_id: "org-a" }`. Capture `rule_id`.
2. Assert `create_rule` returns success and the rule is parseable (BC-2.13.001 load-time validation). Assert `rule_id` is returned.
3. Feed a diff pack to the detection engine containing one record with `severity = 9` (above threshold) and one with `severity = 5` (below threshold).
4. Assert exactly one alert is generated, corresponding to the `severity = 9` record.
5. Assert the alert record in the `alerts` CF contains `rule_id` and `alert_id` (UUID v7).
6. Assert no alert is generated for the `severity = 5` record.

**Expected Outcome:**
- `create_rule` succeeds; rule is persisted to RocksDB `detection_state` CF.
- Single-event matching evaluates each diff record against the rule predicate (BC-2.13.002).
- Exactly one alert generated for the threshold-crossing record; none for the sub-threshold record.
- Alert record contains `rule_id`, `alert_id` (UUID v7), `org_id: "org-a"`, and `matched_fields`.
- Audit log contains `rule_created` entry with `rule_id` and `scope: "org"`.

**Repos Tested:** prism-operations (S-4.03 detection rule loading, S-4.04 evaluation, BC-2.13.001/002), prism-storage (detection_state CF, BC-2.13.012)

---

## HS-010-02: Diff Pack Flow — Detect Changes Between Consecutive Query Result Packs

**Title:** The diff engine correctly computes added and removed records between epoch N and epoch N+1 result packs.

**Preconditions:**
- Schedule "s1" has completed epoch 1 with 3 records: [R1, R2, R3] (fingerprinted and stored)
- Epoch 2 result pack contains [R2, R3, R4] (R1 removed, R4 added)
- `get_diff_results` MCP tool available

**Steps:**
1. Trigger epoch 2 execution (tick fires, query returns [R2, R3, R4]).
2. The diff engine computes: `added = [R4]`, `removed = [R1]` (BC-2.12.005 hash-based diff).
3. Assert `get_diff_results` returns `{ epoch: 2, added: [R4], removed: [R1], unchanged: [R2, R3] }`.
4. Assert R1 is in `removed` (present in epoch 1 fingerprint, absent in epoch 2).
5. Assert R4 is in `added` (absent in epoch 1 fingerprint, present in epoch 2).
6. Assert R2 and R3 are neither in `added` nor `removed`.
7. Assert the diff result is persisted to the `diff_results` CF under key `diff:s1:org-a:2`.

**Expected Outcome:**
- Diff computation correctly identifies added and removed records using SHA-256 fingerprints.
- `get_diff_results` MCP tool returns the correct epoch 2 diff payload.
- RocksDB `diff_results` CF key follows the pattern `diff:{schedule_name}:{client_id}:{epoch}` (BC-2.12.010).
- Fingerprints from epoch 1 are updated to epoch 2 fingerprints after diff computation.
- Diff results retained for the configured `diff_retention_period` (default 7 days).

**Repos Tested:** prism-operations (S-4.02 diff engine, BC-2.12.005/007), prism-storage (diff_results CF persistence, BC-2.12.010)

---

## HS-010-03: Alert Generation with alert_id UUID v7 as Idempotency Key — Replay-Safe

**Title:** Alerts carry a UUID v7 `alert_id` as the idempotency key; replaying the same trigger does not create a duplicate.

**Preconditions:**
- Detection rule "rule-1" active for OrgA with `predicate: "score > 90"`
- Diff pack fed twice with identical contents (simulating replay or restart)
- Alert deduplication index active (BC-2.13.013)

**Steps:**
1. Feed diff pack (epoch 2) containing record M1 with `score = 95` to the detection engine.
2. Rule fires; alert generated with `alert_id = UUID-A1` (UUID v7). Assert UUID version = 7.
3. Assert `alert_id = UUID-A1` written to `alerts` CF with full alert payload.
4. Assert MCP notification broadcast emitted containing `alert_id = UUID-A1`.
5. Feed the SAME diff pack a second time (replay scenario).
6. Assert NO second alert is written to `alerts` CF for record M1 (dedup key `(rule_id, event_uid)` already exists per BC-2.13.013).
7. Assert total alert count for OrgA remains 1.

**Expected Outcome:**
- `alert_id` is a valid UUID v7 (version bits = 0b0111; timestamp component monotonically increasing).
- First feed: alert written, MCP notification broadcast, dedup key inserted.
- Second feed (replay): alert suppressed by dedup guard; no second write; no second broadcast.
- Dedup key `(rule-1, M1_event_uid)` present in RocksDB `alerts` domain dedup index.
- Debug-level log entry emitted for the suppressed duplicate (BC-2.13.013 suppression log).

**Repos Tested:** prism-operations (S-4.05 alert generation, BC-2.13.005/013), prism-storage (alerts CF, dedup index)

---

## HS-010-04: Detection Rule Evaluation Under Multi-Tenant Isolation

**Title:** Detection rules are org-scoped; OrgA's rules do not fire on OrgB's diff results, and vice versa.

**Preconditions:**
- OrgA has rule "rule-a" with `predicate: "severity >= 8"`, `scope: "org"`, `org_id: "org-a"`
- OrgB has rule "rule-b" with `predicate: "score > 50"`, `scope: "org"`, `org_id: "org-b"`
- Both orgs have active schedules; diff packs generated independently per org
- Three-scope rule resolution (BC-2.13.011) active

**Steps:**
1. Feed OrgA's diff pack (containing record with `severity = 9`) to the detection engine in OrgA context.
2. Assert "rule-a" fires and generates alert for OrgA. Assert OrgB's "rule-b" does NOT fire.
3. Feed OrgB's diff pack (containing record with `score = 75`) to the detection engine in OrgB context.
4. Assert "rule-b" fires and generates alert for OrgB. Assert OrgA's "rule-a" does NOT fire.
5. Assert OrgA's alert count = 1; OrgB's alert count = 1; zero cross-org alerts.
6. Assert RocksDB `detection_state` CF keys for both orgs carry distinct `{org_id}:` prefixes (ADR-008).

**Expected Outcome:**
- Rule evaluation is fully org-scoped; no cross-org rule firing occurs.
- Three-scope rule resolution (BC-2.13.011) correctly applies org-level scope to filter rules by org_id.
- `detection_state` CF keys follow ADR-008 prefix convention (verified by direct key enumeration).
- Zero cross-org alerts generated in any direction.
- Alert payloads carry `org_id` field matching the owning org.

**Repos Tested:** prism-operations (S-4.03 three-scope rule resolution, S-4.04 org-scoped evaluation, BC-2.13.011/012), prism-storage (ADR-008 key prefix, detection_state CF)

---

## HS-010-05: Alert Deduplication via Idempotency Key (Same alert_id = No Duplicate Emission)

**Title:** Dedup guard prevents emission of a second alert when the same triggering condition re-evaluates within the dedup TTL window.

**Preconditions:**
- Rule "rule-corr" is a correlation rule: `threshold: 3 events within 5 minutes, group_by: host_id`
- Host "host-99" has triggered 3 events; correlation rule fired; alert UUID-C1 generated
- Dedup key `(rule-corr, group_by_hash(host-99), window_bucket_N)` inserted with TTL=24h (BC-2.13.013)

**Steps:**
1. Assert alert UUID-C1 present in `alerts` CF for OrgA.
2. Inject 3 more events for "host-99" within the same window bucket (simulating window replay or restart).
3. The correlation engine re-evaluates the threshold for window_bucket_N.
4. Assert the dedup guard computes key `(rule-corr, group_by_hash(host-99), window_bucket_N)`.
5. Assert the key already exists in the dedup index (TTL not expired).
6. Assert NO second alert is written for host-99 in window_bucket_N. Total alert count stays at 1.
7. Advance virtual time to window_bucket_N+1. Inject 3 more events. Assert a NEW alert UUID-C2 is generated (different window bucket = new dedup key).

**Expected Outcome:**
- Dedup guard suppresses duplicate for the same `(rule_id, group_by_value_hash, window_bucket)` key.
- New window bucket produces new dedup key → new alert allowed.
- Total alerts: UUID-C1 (bucket N) and UUID-C2 (bucket N+1). No duplicates within bucket.
- Dedup key TTL (24h default) prevents unbounded index growth.
- RocksDB dedup index entries verified present for both keys.

**Repos Tested:** prism-operations (S-4.04 correlation detection, S-4.05 dedup guard, BC-2.13.003/013), prism-storage (dedup index CF, BC-2.13.012)

---

## HS-010-06: Detection Rule Disable Retains Existing Alerts but Stops New Generation

**Title:** Disabling a detection rule via `delete_rule` (or disable API) stops new alerts without modifying previously generated alerts.

**Preconditions:**
- Rule "rule-d" active for OrgA; 2 alerts already generated (UUID-D1, UUID-D2) in `alerts` CF
- Third diff pack available that would normally trigger "rule-d"

**Steps:**
1. Assert UUID-D1 and UUID-D2 present in `alerts` CF.
2. Call `delete_rule` (or disable equivalent) for "rule-d" with `{ rule_id: <id>, confirmation: true }`.
3. Assert `delete_rule` returns success and the rule is removed from the active rule registry.
4. Feed the third diff pack to the detection engine.
5. Assert NO new alert is generated for "rule-d" (rule absent from registry).
6. Assert UUID-D1 and UUID-D2 are STILL present in `alerts` CF (existing alerts unmodified).
7. Assert audit log contains `rule_deleted` entry with `rule_id` and `org_id: "org-a"`.

**Expected Outcome:**
- `delete_rule` removes the rule from active evaluation; no further alerts generated.
- Existing alerts (UUID-D1, UUID-D2) remain in `alerts` CF unmodified.
- Third diff pack evaluation: rule-d absent from rule set → zero alerts from rule-d.
- Audit log confirms `rule_deleted` event is present with correct metadata.
- Other active rules (if any) continue to evaluate normally on the same diff pack.

**Repos Tested:** prism-operations (S-4.03 rule lifecycle, BC-2.13.006/008), prism-storage (detection_state CF, alerts CF)

---

## State Checkpoint

```yaml
scenario_group: HS-010
title: Detection & Alert Pipeline
scenarios: 6
priority: P0
must_pass: true
wave: 4
stories_covered: [S-4.02, S-4.03, S-4.04, S-4.05]
bcs_anchored:
  - BC-2.12.005
  - BC-2.12.007
  - BC-2.13.001
  - BC-2.13.002
  - BC-2.13.003
  - BC-2.13.005
  - BC-2.13.006
  - BC-2.13.008
  - BC-2.13.011
  - BC-2.13.012
  - BC-2.13.013
key_invariants:
  - alert-id-uuid-v7-idempotency
  - deduplication-per-match-mode
  - three-scope-rule-resolution
  - ADR-008-org-scoped-detection-state
status: draft
introduced: cycle-1
```
