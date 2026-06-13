---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 6
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "bc0f36c5"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1112
---

# PR-LEVEL Adversary Pass 6 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head bc0f36c5 — unchanged from pass 5; no code change since bc0f36c5)
**Pass:** PR-LEVEL pass 6 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-5 Closure Verification

All pass-5 findings verified sound (load-bearing or properly adjudicated):

- **BPRL-P5-01 HIGH** (BC-2.06.019 v1.4 Route Coverage Table defects): CLOSED. BC-2.06.019 v1.5 Route Coverage Table corrected four defects — (a) phantom crowdstrike `alerts_search.rs` row removed; (b) summaries method+path corrected to `POST /detects/entities/summaries/GET/v1`; (c) armis `search.rs` UNGUARDED row added; (d) guard wording updated from `stage_idx >= 2 (lateral)` to `mask.lateral_devices`. PC-4 5-arg constructor prose corrected. Story B v2.7 + PIVOT-003 v1.2 pin sweeps applied. **VERIFIED — BC v1.5 Route Coverage Table confirmed against DTU router source. Story B and PIVOT-003 pin sites both reflect v1.5. No stale present-tense pins remain. CLOSED stands.**

## Pass-6 Finding

### BPRL-P6-01 — HIGH [process-gap] Route Coverage Table incomplete: Claroty `routes/devices.rs` absent

**Severity:** HIGH
**Category:** process-gap (Route Coverage Table exhaustive-inventory miss — second consecutive table-completeness miss; POL-33 mandates route_coverage_table_required_for_stagemask_changes)
**BC:** BC-2.06.019 v1.5 Route Coverage Table

**Finding:**

The BC-2.06.019 v1.5 Route Coverage Table (added at D-1109, corrected at D-1111) lists StageMask-affected routes across all sensor DTU clones. However, Claroty's `routes/devices.rs` — which IS guarded by `StageMask` in the PR diff and is load-bearing for AC-015 (stage-based filtering on Claroty device data) — was absent from the table.

**Evidence:**

The PR diff at HEAD bc0f36c5 shows `crates/prism-dtu-claroty/src/routes/devices.rs` modified to apply `with_stage_mask_projection`. This is a StageMask-guarded route. POL-33 requires all StageMask-guarded routes to appear in the BC-2.06.019 Route Coverage Table. The v1.5 table carried 7 rows covering CrowdStrike, Armis, Cyberint, Claroty (alerts, audit_log, vulnerabilities), but no Claroty devices row.

All other 7 existing rows in the v1.5 table were verified accurate:
- CrowdStrike detections: guard correct
- CrowdStrike summaries: method+path corrected at v1.5 (POST /detects/entities/summaries/GET/v1)
- Armis alerts: guard correct
- Armis search: UNGUARDED row correct (search.rs by design)
- Cyberint alerts: guard correct
- Claroty alerts: guard correct
- Claroty audit_log: guard (UNGUARDED, pass-through) correct

**SAP-1 probe:** PASS — `rg 'event_type\s*=' crates/ --type rust` — no new `event_type` emissions in PR diff.
**POL-23 sweep:** PASS — BC-2.06.019 version pins in story B and PIVOT-003 correctly reflect v1.5 after D-1111.
**POL-22 A+C:** PASS — adversarial review completed; no structural concerns about code paths outside of the Route Coverage Table completeness miss.

**Root cause:**

Second consecutive table-completeness miss (pass 5 caught armis search.rs; pass 6 caught claroty devices.rs). Both passes-5 and -6 approached the table as a correction exercise (verify existing rows, add/remove known-incorrect rows) rather than as an exhaustive inventory of all StageMask-guarded route files in the PR diff. The exhaustive-inventory-with-embedded-evidence pattern (now mandated by BC-2.06.019 v1.6 note) is the correct response: enumerate ALL handler files in the diff that call `with_stage_mask_projection`, map each to a row, verify no handler is unrepresented.

**CLEAN(strict):** no (1 HIGH finding)
**CLEAN(PR-merge):** no (1 HIGH finding)
**Streak:** 0/3

---

## Closure Evidence (same-session fix burst D-1112)

**PO amended BC-2.06.019 v1.5→v1.6:**

Claroty `routes/devices.rs` row added to Route Coverage Table:
- Clone: claroty
- Route file: `routes/devices.rs`
- HTTP route: `GET /api/v2/devices`
- Guard mechanism: `with_stage_mask_projection` (StageMask-guarded)
- Status: GUARDED — stage-aware

Additionally, PO embedded an exhaustive inventory verification note directly under the Route Coverage Table. The note records:
1. A 7-file grep scan of all StageMask handler files in the PR diff: `crates/prism-dtu-{crowdstrike,armis,cyberint,claroty}/src/routes/*.rs` — grep for `with_stage_mask_projection`
2. Handler-to-route-to-row mapping: each handler file maps to exactly one HTTP route, each route maps to exactly one table row
3. Result: 8 rows total (7 existing + 1 Claroty devices added) — EXHAUSTIVE per the scan

This embedded-inventory pattern (v1.6 note) mandates that future table amendments must include the inventory evidence inline so the adversary can verify completeness without a separate re-derivation scan.

**Story-writer POL-23 sweep:**

- S-DEMO-DTU-LIVE-SCENARIO-001-B v2.7→v2.8 — BC-2.06.019 pin advanced v1.5→v1.6 at 2 live sites (§Behavioral Contracts BC table row + §Token Budget row); no AC/test changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED.
- S-DEMO-ENRICHMENT-PIVOT-003 v1.2→v1.3 — BC-2.06.019 pin advanced v1.5→v1.6 at all body-level sites (frontmatter comment block, §Narrative, §Architecture Compliance Rules, §Acceptance Criteria AC traces, §Token Budget, §Tasks, §Forbidden Dependencies, §Edge Cases; historical changelog entries preserved as-is); no AC/test changes.
- STORY-INDEX PIVOT-003 row: `anchors BC-2.06.019 v1.5 PC-4` → `v1.6`; both story rows annotated with new version.
- POL-29 sweep: `grep -rn 'BC-2.06.019 v1\.[0-5]'` in .factory/ — 0 remaining stale present-tense pins after sweep (excluding historical changelog entries, which are immutable point-in-time records per TD-VSDD-091).

**Versions bumped:**
- BC-2.06.019: v1.5→v1.6 (PO)
- BC-INDEX: v6.33→v6.34 (state-manager)
- S-DEMO-DTU-LIVE-SCENARIO-001-B: v2.7→v2.8 (story-writer)
- S-DEMO-ENRICHMENT-PIVOT-003: v1.2→v1.3 (story-writer)
- STORY-INDEX: v2.360→v2.361 (story-writer; verified correct — see D-1112 STORY-INDEX reconciliation note)

**Code:** Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only; no code change required; no new push to PR #185).

**Lessons appended:**
- (z5) Route-table completeness requires exhaustive inventory WITH embedded evidence — two consecutive single-row fixes (P5: armis search; P6: claroty devices) each verified existing rows but missed an inventory sweep; v1.6 embedded-inventory pattern is the fix [process-gap].
- (z6) Story-writer index-edit protocol deviation — reinforce dispatch wording.

---

## Do-Not-Reflag Addendum for Pass 7

All prior do-not-reflag entries from the pass-6 dispatch instructions carry forward. Add:

- **BPRL-P6-01 CLOSED:** BC-2.06.019 v1.5→v1.6 — Claroty devices row added + exhaustive inventory verification note embedded under table. Story B v2.8. PIVOT-003 v1.3. STORY-INDEX v2.361. BC-INDEX v6.34. Story B HEAD bc0f36c5 UNCHANGED.

**Pass 7 ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `bc0f36c5`; PR #185
- BC-2.06.019 is now v1.6 — use the v1.6 Route Coverage Table (8 rows, exhaustive); do NOT cite v1.5 or earlier
- STORY-INDEX v2.361; BC-INDEX v6.34
