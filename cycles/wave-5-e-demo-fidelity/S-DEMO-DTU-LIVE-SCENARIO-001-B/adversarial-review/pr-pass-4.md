---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 4
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "13efc875"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1109
---

# PR-LEVEL Adversary Pass 4 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head 13efc875 at review)
**Pass:** PR-LEVEL pass 4 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-3 Closure Verification

All three pass-3 findings verified sound (not paper-fixes):

- **BPRL-P3-01 MED** (CLAUDE.md EXPECTED=50 stale; count propagation): CLOSED at commits 2323cf37 + 13efc875. CLAUDE.md sentence updated 50→52; check-non-exhaustive.sh + struct_violations.rs doc-comment sites updated 50→52; workspace grep confirmed zero remaining live EXPECTED=50 sites. Gate confirmed `PASS: 52 (expected: 52)`. **VERIFIED LOAD-BEARING — count enforced by gate + all prose sites.**
- **BPRL-P3-OBS-1** (cyberint alerts.rs `unwrap_or("ip")` fail-open): CLOSED at commit 2323cf37. Replaced with fail-closed match; new test `test_BC_2_06_019_cyberint_ioc_value_without_ioc_type_withheld` added. **VERIFIED LOAD-BEARING — test exercises fail-closed path.**
- **BPRL-P3-OBS-2** (crowdstrike hosts.rs stage-projection undocumented precedence): CLOSED at commit 2323cf37. Doc comment added citing BC-2.06.019 PC-4 and by-design precedence rule. **VERIFIED LOAD-BEARING (doc comment explains by-design behavior in conjunction with pre-existing test coverage).**

## Adversary Pass 4 Findings

### BPRL-P4-01 — IOC-surface in `alerts.rs` filter matched synthetic `_ioc_value` only — production IOC fields never stamped

**Finding ID:** BPRL-P4-01
**Severity:** MEDIUM
**Category:** Design-faithful correctness (BC-2.06.019 PC-4 per-sensor IOC-surface matrix; production-inert path)

**Description:** BC-2.06.019 Postcondition-4 (v1.3, at time of review) specifies that the scenario progression engine stamps IOC fields for sensors with IOC-surface capability. The `alerts.rs` StageMask projection for cyberint applied an IOC filter (`ioc_value_filter`) matching the synthetic `_ioc_value` field only. The real Cyberint alert IOC fields (`ioc`, `iocs[]`, `alert_data` subfields) are never stamped by the scenario generator — the generator writes `_ioc_value` (a synthetic test-only discriminator) and the production IOC surface fields remain unpopulated. The IOC-filter test passes against `_ioc_value` in the test fixture, producing green coverage, but the production serving path for real IOC field exposure is inert. A SOC-analyst running `ioc_value = "some-ioc"` PrismQL queries against the live demo would retrieve zero records.

**Root cause (WO-D1109 §Q4):** The scenario engine was designed to prove the route-mask mechanism (correct), but the production IOC field stamping was out of scope for Story B. Story B's actual contract (BC-2.06.019 v1.3 PC-4) specifies the StageMask columns and route guard, not the IOC-surface production values. The generator's `_ioc_value` sentinel was a test-convenience discriminator, never a production IOC value.

**D-1109 Human Decision (2026-06-12 — DESIGN-FAITHFUL path):** User direction: "We want the Design-faithful path. Infusion is flagship feature that needs to be correct." IOC-surface work deferred OUT of PR #185 into the S-DEMO-ENRICHMENT-PIVOT-001/002/003 chain:

- **S-DEMO-ENRICHMENT-PIVOT-001** — infusion engine plugin-bridge prerequisites (forward-subset of S-1.14-REDO)
- **S-DEMO-ENRICHMENT-PIVOT-002** — ThreatIntel + NVD infusion specs and .prx plugins
- **S-DEMO-ENRICHMENT-PIVOT-003** — IOC-stamping and demo pivot query (per-sensor IOC-surface realization; anchors BC-2.06.019 v1.4 PC-4 per-sensor IOC-surface matrix)

**BC-2.06.019 v1.3→v1.4 amended by PO per D-1109:** Per-sensor IOC-surface matrix added to PC-4. H1 title UNCHANGED. Interim State clause added: "Story B (PR #185) delivers the StageMask column-projection and route guard. The production per-sensor IOC-surface stamping (Cyberint alert `ioc`/`iocs[]`/`alert_data`; CrowdStrike `behaviors[].ioc_*`) is deferred to S-DEMO-ENRICHMENT-PIVOT-003 per D-1109 human decision. The Interim State is: IOC filter on `_ioc_value` synthetic sentinel is production-inert but the route guard and StageMask machinery are correct." Route Coverage Table added to BC-2.06.019 (per new policy route_coverage_table_required_for_stagemask_changes; registered as POL-33).

**Deferral anchors:** BPRL-P4-01 is CLOSED-BY-DEFERRAL — the finding is intentionally deferred to S-DEMO-ENRICHMENT-PIVOT-003 with explicit BC-2.06.019 v1.4 Interim State clause + Route Coverage Table. **DO NOT REFLAG in pass 5 or later: this is an adjudicated deferral with explicit spec amendment.**

**Status: CLOSED-BY-DEFERRAL (BC-2.06.019 v1.4 Interim State clause + anchored to S-DEMO-ENRICHMENT-PIVOT-003; raising "IOC masking inert" again = re-raising an adjudicated deferral).**

---

### BPRL-P4-02 — CrowdStrike detections route served primary-device records at stage 0

**Finding ID:** BPRL-P4-02
**Severity:** LOW
**Category:** StageMask guard discipline (BC-2.06.019 PC-4; route coverage)

**Description:** `crates/prism-dtu-crowdstrike/src/routes/detections.rs` — the list and summaries handlers served primary device records at stage 0 (the "pre-compromise" stage) without a stage guard. BC-2.06.019 PC-4 specifies that detection-class records should only surface from stage 1 onwards (lateral movement begins). At stage 0, detections should be empty. The route served records without checking `stage_idx >= 1`.

**Sibling sweep (SAP-2 + Route Coverage Table discipline):** On investigation, the adversary verified:
- Armis alerts route: `in:alerts` guard absent for scenario-gated paths. Added in fix-burst.
- Claroty alerts route: no device_id emitted at stage 0; pre-existing guard behavior verified — EXEMPT (no device_id field in claroty alert schema at stage 0 path).
- CrowdStrike hosts route: stage-guard for containment progression — VERIFIED PRESENT (existing B-P1-01 closure from LOCAL cascade).

**Fix evidence:** Commit `bc0f36c5` (2026-06-12):
- `crates/prism-dtu-crowdstrike/src/routes/detections.rs`: list + summaries handlers: added `stage_idx >= 1` guard before serving records; returns empty list at stage 0.
- `crates/prism-dtu-armis/src/routes/alerts.rs`: `in:alerts` stage guard added per sibling sweep.
- SID-1 unit test added to prism-dtu-crowdstrike detections module: `test_BC_2_06_019_detections_stage_0_returns_empty` — exercises stage-0 path, asserts empty list returned; does not require external DTU (SID-1 compliance).
- `just check` PASS (930s approximate) confirmed at push.

**Status: CLOSED (commit bc0f36c5; stage guard load-bearing with new SID-1 unit test; armis sibling guard added; claroty exemption rationale documented).**

---

### [process-gap] BPRL-P4-PG-01 — No route×entity coverage matrix documented

**Finding ID:** BPRL-P4-PG-01
**Severity:** PROCESS-GAP
**Category:** Governance documentation (POL discipline)

**Description:** The PR diff introduces StageMask-relevant route handlers across four DTU crates (crowdstrike detections, armis alerts, cyberint alerts, claroty alerts) but no Route Coverage Table enumerating the StageMask field, clone, route file, HTTP route, guard mechanism, and status was documented in the governing BC or policy. Without a coverage table, future changes to any route can silently omit a guard without adversarial detection.

**Resolution:** BC-2.06.019 v1.4 (PO amendment, D-1109) adds the Route Coverage Table per WO-D1109 §Q4 ruling. POL-33 `route_coverage_table_required_for_stagemask_changes` registered: any BC/story/source commit introducing or modifying a StageMask-relevant DTU route handler MUST update the Route Coverage Table in the same artifact/commit. **CLOSED.**

**Status: CLOSED (BC-2.06.019 v1.4 Route Coverage Table; POL-33 registered D-1109).**

---

## Standing Probe Results

### SAP-1 (Tracing emission catalog completeness)

`rg 'event_type\s*=' crates/ --type rust` run against the Story B diff scope at HEAD 13efc875. Zero new `event_type =` emissions introduced by Story B changes reviewed in this pass. **SAP-1: PASS.**

### SAP-2 (DTU↔TOML schema parity)

No sensor TOML files modified in the PR diff at HEAD 13efc875. **SAP-2: N/A.**

### POL-22 Phase A+C (CLAUDE.md canonical principle self-audit)

No rationalization anti-patterns present. BPRL-P4-02 fix (bc0f36c5) verified load-bearing with SID-1 test. Pass-2/3 fixes previously verified sound — not paper-fixes. **POL-22 A+C: PASS.**

---

## Convergence Status

```
CLEAN (strict): no  — BPRL-P4-01 MED + BPRL-P4-02 LOW + BPRL-P4-PG-01 process-gap present at review HEAD (13efc875)
                       BPRL-P4-02 fixed in commit bc0f36c5 (pushed 2026-06-12)
                       BPRL-P4-01 CLOSED-BY-DEFERRAL (BC-2.06.019 v1.4 Interim State + anchored to S-DEMO-ENRICHMENT-PIVOT-003)
                       BPRL-P4-PG-01 CLOSED (BC-2.06.019 v1.4 Route Coverage Table + POL-33)
CLEAN (PR-merge): no  — BPRL-P4-01 MED and BPRL-P4-02 LOW were both blocking at review
```

**Streak after pass 4: 0/3** (BPRL-P4-01 MED finding resets streak per BC-5.39.001 D-779; BPRL-P4-01 closed-by-deferral does NOT constitute strict-CLEAN — a finding was present).

**Story B branch HEAD after fix-burst: `bc0f36c5` (= remote; pushed 2026-06-12).**

**Do-not-reflag additions for pass 5:**
- BPRL-P4-01 CLOSED-BY-DEFERRAL: IOC-surface production inertness is adjudicated per D-1109. BC-2.06.019 v1.4 Interim State clause governs. Anchored to S-DEMO-ENRICHMENT-PIVOT-003. Re-raising this = re-raising an adjudicated deferral.
- BPRL-P4-02 CLOSED: CrowdStrike detections stage-0 guard; Armis alerts in:alerts guard. Both at bc0f36c5.
- BPRL-P4-PG-01 CLOSED: Route Coverage Table in BC-2.06.019 v1.4; POL-33 registered.

**NEXT: PR-LEVEL pass 5** — dispatch fresh adversary on PR #185 at HEAD bc0f36c5.
