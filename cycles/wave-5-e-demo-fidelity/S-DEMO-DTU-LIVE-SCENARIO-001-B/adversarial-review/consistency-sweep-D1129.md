# Consistency Sweep — D-1129 (2026-06-13)

**Type:** Proactive consistency-validator sweep (NOT an adversary pass — streak UNCHANGED at 0/3)
**Trigger:** Orchestrator-directed sweep of D-1117 spec cluster after pass 22 and before pass 23
**Rationale:** After 3 consecutive D-1117 prose-propagation findings (P14/P15/P22), the orchestrator ran a dedicated consistency sweep to flush remaining drift in one pass instead of dribbling one-per-adversary-pass. Codified as lesson z16.
**Code HEAD:** 0863184a (UNCHANGED — these are spec-text drifts; AC-019 + code were always correct 6-arg)
**State Version:** v7.777 → v7.778

---

## Findings

### DRIFT-1 — STORY-INDEX PIVOT-003 inline `BC-2.06.020 v1.3` stale (CLOSED)

**Severity:** MAJOR (spec-text drift; misdirects implementer to believe BC version is v1.3 when current is v1.5)
**Location:** `STORY-INDEX.md` line 517, PIVOT-003 row, trailing `2 BCs:` inline annotation
**Stale text:** `2 BCs: BC-2.06.019+BC-2.06.020 v1.3`
**Correct text:** `2 BCs: BC-2.06.019+BC-2.06.020 v1.5`

**Root cause:** The v1.3→v1.4 and v1.4→v1.5 BC-2.06.020 version-advance sweeps (D-1120, D-1128) targeted §Behavioral Contracts BC table row and §Token Budget BC context row in the PIVOT-003 story file and in the STORY-INDEX PIVOT-003 narrative changelog annotation. They did NOT sweep the trailing inline `2 BCs:` annotation at the end of the PIVOT-003 Full Story List row. This is a distinct location class from the narrative changelog annotations.

**Closure:** STORY-INDEX.md PIVOT-003 row trailing annotation corrected `v1.3` → `v1.5` in this burst.

---

### DRIFT-2 — Story B §Tasks Phase-2 Cyberint task: stale 5-arg `new_with_scenario` (CLOSED)

**Severity:** MAJOR (implementation-misdirecting; implementer following the task prose would call `new_with_scenario(config, seed, org_id, time_anchor, state)` — the pre-D-1117 5-arg signature — missing the catalog parameter added by D-1117 f0b6b8c7)
**Location:** `S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`, §Tasks, Phase-2, Cyberint task
**Root cause:** D-1117 added `catalog: &ScenarioEntityCatalog` as the 6th argument to `CyberintClone::new_with_scenario` (f0b6b8c7) and updated AC-019 + BC-2.06.020 PC-8. The §Tasks prose describing the Cyberint implementation step was not swept for the constructor signature update.

**Closure:** Story B v2.14→v2.15: Phase-2 Cyberint task prose corrected to 6-arg `new_with_scenario(config, seed, org_id, time_anchor, state, catalog)`.

---

### DRIFT-3 — Story B §Tasks Phase-2 FSR `clone.rs` row + Phase-4 `build_clone_pairs` Cyberint call: stale 5-arg (CLOSED)

**Severity:** MAJOR (implementation-misdirecting — same class as DRIFT-2)
**Location:** `S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`, §File Structure Reference clone.rs row and §Tasks Phase-4 `build_clone_pairs` Cyberint call site
**Root cause:** Same as DRIFT-2 — D-1117 6-arg change propagated to AC-019 + BC-2.06.020 PC-8 but not to the FSR row description or the build_clone_pairs call illustration in Phase-4.

**Closure:** Story B v2.14→v2.15: FSR clone.rs row description and Phase-4 build_clone_pairs Cyberint call corrected to 6-arg.

**Full `new_with_scenario` sweep result (story-writer confirms):**
- Cyberint clone: `new_with_scenario(config, seed, org_id, time_anchor, state, catalog)` — 6-arg at 3 sites (task prose, FSR, build_clone_pairs call) — all CORRECT after closure
- Armis, Claroty, CrowdStrike clones: `new_with_scenario(config, seed, org_id, time_anchor, state)` — 5-arg (no catalog; these clones do not require catalog per BC-2.06.020 PC-8 scope) — CORRECT
- ThreatIntel, NVD clones: `new_with_scenario(config, seed, org_id)` — 1-arg (static fixture enrichment clones; no temporal progression) — CORRECT

---

## Clean Confirmations

The following checks were run and confirmed CLEAN — do NOT raise as fresh findings:

| Check | Result |
|-------|--------|
| BC-2.06.020 PC-1..9 counts (9 PCs) | CLEAN — correct at v1.5 |
| BC-2.06.019/020 INV count (7 invariants) | CLEAN |
| TV count (15 test vectors) | CLEAN |
| EC count (15 edge cases) | CLEAN |
| VP-020 count (12 VPs, A..L) | CLEAN — v1.5 prose is `VP-020-A through VP-020-L` / `all 12 VPs` |
| VP-019 count (9 VPs per BC-2.06.019) | CLEAN |
| Story B acceptance_criteria_count 19 | CLEAN — consistent with 19-row AC table |
| Story B red_gate_tests 23 | CLEAN — consistent with 23-row RGT table and Phase-6 gate instruction |
| BC-2.06.019 H1 ↔ BC-INDEX row 119 title | CLEAN |
| BC-2.06.020 H1 ↔ BC-INDEX row 120 title | CLEAN |
| BC version pins (story B, PIVOT-001, PIVOT-002, PIVOT-003) | CLEAN after DRIFT-1 closure |
| Catalog `{:05}` vs Cyberint-baseline `{:04}` | CLEAN — intentionally distinct (two generators; DO NOT raise) |
| AC-019 code 6-arg Cyberint constructor | CLEAN — shipped code 0863184a was always correct 6-arg |

---

## Index Updates

| File | Change |
|------|--------|
| STORY-INDEX.md | v2.367→v2.368; PIVOT-003 row trailing pin v1.3→v1.5; story B row v2.14→v2.15; changelog row added |
| BC-INDEX.md | v6.41→v6.42; rows 119/120 anchor story pin v2.14→v2.15; changelog row added |

---

## Convergence Impact

**This is a consistency gate, NOT an adversary pass.**
- Streak UNCHANGED: 0/3 (pass 22 closed BPRL-P22-01; pass 23 NEXT)
- Feature HEAD: 0863184a (CODE UNCHANGED)
- PR #185: diff UNCHANGED (reuse /tmp/pr185-pass20.diff or `gh pr diff 185`)
- PR-LEVEL pass 23 is the next action

---

## Lesson z16

Proactive consistency-validator sweep after a multi-burst spec-churn cycle flushes drift in ONE pass instead of dribbling one-per-adversary-pass. After 3 consecutive D-1117 prose-propagation findings (P14/P15/P22), this dedicated sweep found 3 MORE (including 2 implementation-misdirecting 5-vs-6-arg task drifts) at once. Codified: orchestrator SHOULD run a consistency-validator sweep over the changed-spec cluster after any cycle with >= 3 spec-amendment bursts, BEFORE resuming/continuing the adversary 3-CLEAN cascade.
