---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 5
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "bc0f36c5"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1111
---

# PR-LEVEL Adversary Pass 5 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head bc0f36c5 — unchanged from pass 4; no code change since bc0f36c5)
**Pass:** PR-LEVEL pass 5 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-4 Closure Verification

All pass-4 findings verified sound (load-bearing or properly adjudicated):

- **BPRL-P4-01 MED** (IOC-surface in alerts.rs production-inert): CLOSED-BY-DEFERRAL per D-1109 human decision (design-faithful path). BC-2.06.019 v1.4 Interim State clause governs. Anchored to S-DEMO-ENRICHMENT-PIVOT-003. **VERIFIED — deferral is structurally complete: BC-2.06.019 v1.4 carries the per-sensor IOC-surface matrix, the Interim State clause enumerates exactly what is deferred and to which story, and the Route Coverage Table confirms the StageMask machinery (the Story B deliverable) is correct. This is not a paper-fix — the BC explicitly names the incomplete surface and the remediation anchor. CLOSED-BY-DEFERRAL stands.**
- **BPRL-P4-02 LOW** (CrowdStrike detections stage-0 guard; Armis alerts in:alerts guard): CLOSED at commit bc0f36c5. Stage guards verified load-bearing: `test_BC_2_06_019_detections_stage_0_returns_empty` exercises the stage-0 empty-list path; armis `in:alerts` guard similarly wired. **VERIFIED LOAD-BEARING — SID-1 unit test exercises the guard directly without external dependency.**
- **BPRL-P4-PG-01 process-gap** (no Route Coverage Table): CLOSED by BC-2.06.019 v1.4 Route Coverage Table + POL-33 registered. **VERIFIED — Route Coverage Table enumerates StageMask field × clone × route file × HTTP route × guard mechanism × status for all four DTU crates in scope.**

## Adversary Pass 5 Finding

### BPRL-P5-01 — BC-2.06.019 v1.4 Route Coverage Table row defects vs code

**Finding ID:** BPRL-P5-01
**Severity:** HIGH
**Category:** BC-code parity (BC-2.06.019 PC-4 Route Coverage Table; POL-33 route_coverage_table_required_for_stagemask_changes)

**Description:** The Route Coverage Table added to BC-2.06.019 v1.4 (per D-1109/POL-33) contained four defects when verified against the actual DTU router source at HEAD bc0f36c5:

1. **Phantom row — CrowdStrike `alerts_search.rs` / `GET /alerts/queries/alerts/v2`:** The table listed a "crowdstrike / alerts_search.rs / GET /alerts/queries/alerts/v2 / stage_idx >= 2 (lateral) / GUARDED" row. No such route file or route exists in `crates/prism-dtu-crowdstrike/src/routes/`. The CrowdStrike DTU clone serves detections and hosts routes, not an `/alerts/queries/alerts/v2` endpoint. This row is a phantom — it describes a non-existent route.

2. **Wrong method + path for CrowdStrike summaries:** The table listed `GET /detects/entities/summaries/v1` for the summaries handler. The actual summaries handler issues `POST /detects/entities/summaries/GET/v1` (CrowdStrike's unconventional POST-as-GET pattern for batch ID retrieval). The method and path in the BC table were wrong.

3. **Missing Armis `routes/search.rs` / `GET /api/v1/search` row:** The Armis search route handler at `crates/prism-dtu-armis/src/routes/search.rs` serves `GET /api/v1/search`. This route is not StageMask-gated (Armis search results are served unrestricted; only `alerts.rs` carries the `in:alerts` guard). The table omitted this route entirely. Completeness requires all routes to be enumerated with their status (including UNGUARDED, where deliberate).

4. **Lateral wording vs real mechanism — `stage_idx >= 2` vs `mask.lateral_devices`:** Several table rows described the guard mechanism as `stage_idx >= 2 (lateral)`. The actual StageMask implementation does not compare `stage_idx` numerically; it checks `mask.lateral_devices` (a boolean field on the StageMask struct corresponding to the LateralMovement stage). The narrative "stage_idx >= 2" is an approximation; the code field name `mask.lateral_devices` is the load-bearing identifier.

**Root cause (lesson z4):** BC-2.06.019 v1.4 was authored by PO in the same session that produced the U20 uncertainty-scan fixes in PIVOT-003 story. The PIVOT-003 U20 fix correctly updated the story's route-coverage table with accurate route shapes grounded against DTU source. However, the BC-2.06.019 v1.4 Route Coverage Table was authored from the same assumed-route claims that the U20 scan caught as wrong — the BC table was written before the scan results were available and used pre-scan assumptions. The U20 fix swept the story copy but the BC copy (authored concurrently, pre-scan) kept the errors. This is a POL-25 dual-carrier propagation gap: when a table/claim exists in BOTH a BC and a story, a correction to one MUST sweep the other in the same burst.

**Code verification (SAP-2 style):** The following were verified against `crates/prism-dtu-crowdstrike/src/routes/` and `crates/prism-dtu-armis/src/routes/` at HEAD bc0f36c5:
- `detections.rs` — `list` handler: POST /detects/queries/detects/v1; `summaries` handler: POST /detects/entities/summaries/GET/v1. Both guarded `stage_idx >= 1`.
- No `alerts_search.rs` exists in prism-dtu-crowdstrike routes directory.
- `alerts.rs` (armis) — guarded with `in:alerts` mask check (added BPRL-P4-02 bc0f36c5).
- `search.rs` (armis) — `GET /api/v1/search`; no stage guard (search is unrestricted by design).

**Closure (same day — D-1111):** PO amended BC-2.06.019 v1.4→v1.5 with corrected 7-row Route Coverage Table verbatim-verified vs actual router source: phantom crowdstrike alerts_search row REMOVED; summaries method+path corrected to `POST /detects/entities/summaries/GET/v1`; armis search.rs `GET /api/v1/search` row added with UNGUARDED (by design); lateral-wording rows updated to cite `mask.lateral_devices` boolean field as the guard mechanism. Additionally, PC-4 prose PC-4 constructor row updated: "4-arg per-clone return types" stale reference corrected to the actual 5-arg `new_with_scenario(seed, archetype, org_id, Arc<timeline>, time_anchor)` signature with per-clone fallibility (CrowdStrike/Claroty -> Self; Armis/Cyberint/NVD -> anyhow::Result<Self>; ThreatIntel -> Self) per ADR-036 v2.3 ruling.

Story-writer applied POL-23 sweep: story B BC-2.06.019 pin advanced v1.4→v1.5 at both live sites (§Behavioral Contracts BC table row + §Token Budget row); PIVOT-003 BC-2.06.019 pin advanced v1.4→v1.5 at both live sites (story body §Behavioral Contracts table + §Token Budget row). PIVOT-001/002 carry zero pins to BC-2.06.019 (verified zero-hit grep). POL-29 sweep: `grep -rn 'BC-2.06.019 v1\.[0-4]'` in .factory/ — 0 remaining stale present-tense pins after sweep.

Story B advances v2.6→v2.7 (2 pins). PIVOT-003 advances v1.1→v1.2 (30 pins — exhaustive sweep per lesson from OCSF-CLASS-MIGRATION-001 cite-pin recurrence; full story file read top-to-bottom, 30 BC-2.06.019 present-tense pin sites confirmed v1.5, 0 stale). BC-INDEX row 119 annotated v1.5; STORY-INDEX rows updated; BC-INDEX v6.32→v6.33; STORY-INDEX v2.359→v2.360.

**Status: CLOSED (BC-2.06.019 v1.5 corrected Route Coverage Table + PC-4 5-arg prose; story B v2.7 + PIVOT-003 v1.2 pin sweeps; BC-INDEX v6.33; STORY-INDEX v2.360; D-1111).**

**Adversary LOW note (non-blocking):** BC-2.06.019 v1.4 PC-4 prose reference to "4-arg per-clone constructor" was stale (ADR-036 v2.3 ruling: 5-arg). Closed as part of the v1.5 amendment above.

---

## Standing Probe Results

### SAP-1 (Tracing emission catalog completeness)

`rg 'event_type\s*=' crates/ --type rust` run against the Story B diff scope at HEAD bc0f36c5. Zero new `event_type =` emissions introduced by Story B changes. **SAP-1: PASS.**

### SAP-2 (DTU↔TOML schema parity)

No sensor TOML files modified in the PR diff at HEAD bc0f36c5. **SAP-2: N/A.**

### POL-22 Phase A+C (CLAUDE.md canonical principle self-audit)

BPRL-P5-01 HIGH describes a real BC-code parity defect (table rows contradicting actual routes). Closure is load-bearing: the corrected BC-2.06.019 v1.5 Route Coverage Table accurately describes the actual DTU router behavior. No rationalization anti-patterns present. **POL-22 A+C: PASS.**

---

## Convergence Status

```
CLEAN (strict): no  — BPRL-P5-01 HIGH present at review HEAD (bc0f36c5)
                       CLOSED same day: BC-2.06.019 v1.5 corrected table + PC-4 5-arg prose;
                       story B v2.7 + PIVOT-003 v1.2 pin sweeps; BC-INDEX v6.33; STORY-INDEX v2.360
CLEAN (PR-merge): no  — BPRL-P5-01 HIGH was blocking at review
```

**Streak after pass 5: 0/3** (BPRL-P5-01 HIGH finding resets streak per BC-5.39.001 D-779; BC-side amendment closed same day but finding was present — strict-CLEAN requires zero findings of ANY severity).

**Story B branch HEAD: bc0f36c5 (unchanged — finding was BC-side only; no code change required; no new push).**

**Do-not-reflag additions for pass 6:**
- BPRL-P5-01 CLOSED: BC-2.06.019 v1.5 corrected Route Coverage Table (phantom row removed; summaries method+path corrected; armis search.rs row added; mask.lateral_devices wording). PC-4 5-arg prose corrected. Story B v2.7 + PIVOT-003 v1.2 pin sweeps. BC-INDEX v6.33. STORY-INDEX v2.360.
- Adversary LOW PC-4 4-arg prose stale: CLOSED as part of v1.5 amendment.

**NEXT: PR-LEVEL pass 6** — dispatch fresh adversary on PR #185 at HEAD bc0f36c5 (no code change since pass 5 — diff identical).
