---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 7
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "bc0f36c5"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1113
---

# PR-LEVEL Adversary Pass 7 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head bc0f36c5 — unchanged from pass 6; no code change since bc0f36c5)
**Pass:** PR-LEVEL pass 7 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-6 Closure Verification

All pass-6 findings verified sound (load-bearing or properly adjudicated):

- **BPRL-P6-01 HIGH [process-gap]** (BC-2.06.019 v1.5 Route Coverage Table missing Claroty `routes/devices.rs`): CLOSED. BC-2.06.019 v1.5→v1.6 — Claroty devices row added (`GET /api/v2/devices`, StageMask-guarded, load-bearing for AC-015) + exhaustive inventory verification note embedded under table (7-file StageMask handler scan + handler→route→row mapping; 8-row EXHAUSTIVE count confirmed). Story B v2.8. PIVOT-003 v1.3. STORY-INDEX v2.361. BC-INDEX v6.34. Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only). **VERIFIED — v1.6 Route Coverage Table has 8 rows, all verified against DTU router source. CLOSED stands.**

## Pass-7 Finding

### BPRL-P7-01 — MED [process-gap] BC-2.06.019 v1.6 inventory-note prose contains fabricated grep claim

**Severity:** MED
**Category:** process-gap (inventory verification note contains a factual inaccuracy — PO prose claim about tool output that was never executed)
**BC:** BC-2.06.019 v1.6 Route Coverage Table inventory verification note

**Finding:**

The BC-2.06.019 v1.6 inventory verification note embedded under the Route Coverage Table contains the following claim (paraphrased from the note):

> `crates/prism-dtu-claroty/src/routes/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` references"

This claim is factually incorrect. Reading `crates/prism-dtu-claroty/src/routes/alerts.rs` directly confirms:

1. The file contains **zero** occurrences of the string `scenario_stage_ctx`.
2. The file contains **zero** occurrences of `stage` in any form that would match a StageMask grep pattern.
3. `claroty/alerts.rs` is EXEMPT from the StageMask inventory on real-API grounds — the claroty alerts endpoint does not support server-side stage filtering. The EXEMPT determination itself is correct and well-founded.

The defect is confined to the **explanatory prose** within the inventory note. The prose fabricated a grep match that was never run, invoking `scenario_stage_ctx` as justification for why `alerts.rs` appeared in "both grep sets." This string does not exist in `alerts.rs`. The EXEMPT status of `claroty/alerts.rs` is correct; only the stated justification is wrong.

**Evidence:**

- `grep -c 'scenario_stage_ctx' crates/prism-dtu-claroty/src/routes/alerts.rs` → 0
- `grep -c 'stage' crates/prism-dtu-claroty/src/routes/alerts.rs` → 0 (no stage references)
- The claroty/alerts route's EXEMPT status is correctly anchored in its real-API behavior (no server-side stage parameter), not in any presence/absence in grep output.

**All other axes verified clean:**

- **7-file inventory re-run:** PASS — the 7 non-EXEMPT route files identified in the v1.6 table (crowdstrike/detections, crowdstrike/summaries, armis/alerts, armis/search [UNGUARDED], cyberint/alerts, claroty/audit_log [UNGUARDED], claroty/devices) all verified present with correct guard status. No additional StageMask-guarded route files found in the PR diff. 8-row total count CORRECT.
- **BC-2.06.020 invariants:** PASS — enrichment correlation BC content unchanged and consistent.
- **E-DEMO-006 byte-exact:** PASS — org_id guard message format matches error-taxonomy v1.78 verbatim.
- **SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` — no new `event_type` emissions in PR diff.
- **SAP-2:** N/A — no sensor TOML files in PR diff.
- **Forbidden-pattern sweep:** PASS — no `unwrap()`, no `println!`, no `reqwest::Client::new()` without timeout, no retired ColumnType variants.
- **DormantTenant regression guard:** PASS — Red Gate test 17 present and non-vacuous.
- **Demo evidence 18/18 ACs:** PASS — demo evidence in commit range intact.
- **Frontmatter-body coherence:** PASS — acceptance_criteria_count 18, red_gate_tests 19 consistent.

**Root cause:**

PO prose claims about tool output (grep results) must be evidence-backed. The v1.6 note asserted that `claroty/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` references" — a claim that was never verified against the actual file content. This is the internal-agent variant of the hallucination class the research-agent caught in lesson z2 (Perplexity returning fabricated content). In both cases, the agent stated a factual claim about external output (grep results / API documentation) that was not actually verified. The EXEMPT determination itself was correct; only the fabricated justification was wrong.

**CLEAN(strict):** no (1 MED finding)
**CLEAN(PR-merge):** no (1 MED finding)
**Streak:** 0/3

---

## Closure Evidence (same-session fix burst D-1113)

**PO amended BC-2.06.019 v1.6→v1.7:**

Single sentence corrected in the inventory verification note:

- **Before (v1.6):** Explanatory prose claiming `claroty/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` references."
- **After (v1.7):** Corrected prose: `claroty/alerts.rs` does NOT appear in either grep set (zero stage/mask references); EXEMPT status stands solely on real-API grounds (claroty alerts endpoint does not support server-side stage filtering). Fabricated justification removed. No table row, semantic contract, or EXEMPT determination changed.

No table rows were modified. The Route Coverage Table remains 8 rows, EXHAUSTIVE. BC-2.06.019 semantic content is unchanged.

**Story-writer POL-23 sweep:**

- S-DEMO-DTU-LIVE-SCENARIO-001-B v2.8→v2.9 — BC-2.06.019 pin advanced v1.6→v1.7 at 2 live sites (§Behavioral Contracts BC table row + §Token Budget row); no AC/test changes; acceptance_criteria_count 18 UNCHANGED; red_gate_tests 19 UNCHANGED.
- S-DEMO-ENRICHMENT-PIVOT-003 v1.3→v1.4 — BC-2.06.019 pin advanced v1.6→v1.7 at all body-level sites (33+ pin sites swept exhaustively); no AC/test changes.
- STORY-INDEX story B row v2.8→v2.9; PIVOT-003 row v1.3→v1.4; v2.361→v2.362 + changelog row.

**Versions bumped:**
- BC-2.06.019: v1.6→v1.7 (PO)
- BC-INDEX: v6.34→v6.35 (state-manager)
- S-DEMO-DTU-LIVE-SCENARIO-001-B: v2.8→v2.9 (story-writer)
- S-DEMO-ENRICHMENT-PIVOT-003: v1.3→v1.4 (story-writer)
- STORY-INDEX: v2.361→v2.362 (state-manager)

**Code:** Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only; no code change required; no new push to PR #185).

**Lesson appended:**
- (z7) Agent prose claims about tool output must be evidence-backed — BC v1.6 note asserted a grep match that was never actually run; same hallucination class as lesson z2 (internal-agent variant); inventory notes must paste actual grep output or confirmed zero-hit results, not narrate assumed output [process-gap].

---

## Do-Not-Reflag Addendum for Pass 8

All prior do-not-reflag entries from the pass-7 dispatch instructions carry forward. Add:

- **BPRL-P7-01 CLOSED:** BC-2.06.019 v1.6→v1.7 — fabricated grep claim in inventory note corrected (claroty/alerts.rs does NOT appear in either grep set; zero stage/mask references; EXEMPT determination stands on real-API grounds). Story B v2.9. PIVOT-003 v1.4. STORY-INDEX v2.362. BC-INDEX v6.35. Story B HEAD bc0f36c5 UNCHANGED.

**Pass 8 ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `bc0f36c5`; PR #185
- BC-2.06.019 is now v1.7 — use the v1.7 Route Coverage Table (8 rows, exhaustive, corrected inventory note); do NOT cite v1.6 or earlier inventory-note prose
- STORY-INDEX v2.362; BC-INDEX v6.35
