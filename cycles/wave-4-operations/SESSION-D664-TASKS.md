---
document_type: session-tasks
version: "1.8"
status: active
related_burst: D-672
predecessor_state: D-664
predecessor_session_tasks: SESSION-D644-TASKS.md
timestamp: 2026-05-17T03:30:00Z
---

# Session Task List — D-664 Durable Pre-/Clear Snapshot

This file persists the cascade state from the session covering D-645 through D-663 (cascade passes 37 through 54; 16 fix-bursts FB28-FB43; ~12M tokens consumed across pass-37 through pass-54 fresh-context cycle).

**Intended audience:** orchestrator at next session start AFTER /clear. Read alongside:
- `.factory/STATE.md` v7.351 (this burst bumps; `current_step` + `prereq_e_adversary_streak` + `pre_compact_snapshot` fields)
- `.factory/SESSION-HANDOFF.md` v7.351 (this burst bumps; §POST-D664 DURABLE RESUME SNAPSHOT section added)
- `.factory/cycles/wave-4-operations/S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md` (full cascade history through D-664)
- `.factory/cycles/wave-4-operations/SESSION-D644-TASKS.md` (prior session task list, D-645 through D-663 era)

## Cascade State Summary (as of D-664)

**Cascade progress:**
- **18 adversary passes completed this session** (pass-37 through pass-54; pass-1 through pass-36 in prior sessions D-580/D-644 era)
- **54 adversary passes total** across all sessions
- **16 fix-bursts closed this session** (FB28 through FB43)
- **27 fix-bursts total** (FB1-FB27 in prior sessions; FB28-FB43 this session)
- **170 consecutive single-commit bursts** (D-664 is the 170th; restoring TD-VSDD-053 discipline after FB43 two-commit deviation — see Known Issue below)
- **3 CLEAN passes this session**: pass-39 (1st), pass-43 (2nd), pass-51 (3rd)
- **8 CLEAN passes total** across all sessions: pass-9, pass-19, pass-23, pass-25, pass-26, pass-29, pass-30, pass-35 (prior sessions) + pass-39, pass-43, pass-51 (this session)
- **Current streak:** 0/3 (pass-62 BLOCKED at D-672; streak unchanged; 26 passes consumed in restart-9 sequence; F-LP62-MED-001 AC-5 mechanism orphan + OBS-LP62-001 HS-002 parenthetical + OBS-LP62-002 17-site D7 sweep; **POL-29 NOW ACTIVE** — structural intervention codified at FB50 D-672; pass-63 begins under POL-29 discipline)

**Trajectory novel-finding count (this session):** 3(pass-36 carry-forward)→pass-37:BLOCKED(3M)→pass-38:BLOCKED(1M+1L)→pass-39:CLEAN★→pass-40:BLOCKED(1M+1L)→pass-41:BLOCKED(1L)→pass-42:BLOCKED(1M+1L)→pass-43:CLEAN★→pass-44:BLOCKED(2M)→pass-45:BLOCKED(1M+1L+2OBS)→pass-46:BLOCKED(1H+1M)→pass-47:BLOCKED(1H+3M+1L)→pass-48:BLOCKED(1H+3M)→pass-49:BLOCKED(1H+4M+1L)→pass-50:BLOCKED(2M+1L)→pass-51:CLEAN★→pass-52:BLOCKED(1H)→pass-53:BLOCKED(2M)→pass-54:BLOCKED(1H+2OBS)→pass-55:CLEAN★(0 findings; 2 non-blocking OBS; novelty ZERO)→pass-56:BLOCKED(1H; F-LP56-HIGH-001 production call-graph defect; novelty HIGH; architect Option A)→pass-57:BLOCKED(2H+1M+1OBS; F-LP57-HIGH-001 runtime_deliverables + F-LP57-HIGH-002 SS-22 + F-LP57-MED-001 tracing-test + OBS-LP57-001 Path A; novelty HIGH; FB45 architect+PO+SM multi-agent)→pass-58:BLOCKED(2H+3M+1OBS; F-LP58-HIGH-001 ADR-027 title-vs-§D1 contradiction + F-LP58-HIGH-002 HS-003-05 Step 1 vs AC-9 gate FB45-sibling-sweep + F-LP58-MED-001/002/003 + OBS-LP58-001; novelty HIGH; FB46 multi-agent closure) →pass-59:BLOCKED(2H+1M+1OBS; F-LP59-HIGH-001 CAP-029 mis-anchor + F-LP59-HIGH-002 risk_mitigations renumbering drift + F-LP59-MED-001 deprecation framing 5-site sibling-sweep + OBS-LP59-001; novelty HIGH; POL-29 #16+; FB47 corrective multi-agent) →pass-60:BLOCKED(1H+1L+1OBS; F-LP60-HIGH-001 BC-2.16.012 §Changelog ASCENDING-at-top 4th POL-26 recurrence + F-LP60-LOW-001 §risk_mitigations AC-7..8 path-citation ambiguity Option (a) + OBS-LP60-001 BC-INDEX schema cycle-close; POL-29 #17+; FB48 multi-agent) →pass-61:BLOCKED(1H+1M; F-LP61-HIGH-001 story §Changelog 5th POL-26 monotonic-ordering recurrence sibling-class-missed-by-FB48 + F-LP61-MED-001 §risk_mitigations AC-4..6 behavioral-equivalence misattribution Tests-6-7-verify-absence; POL-29 #18+; FB49 multi-agent) →pass-62:BLOCKED(0H+1M+2L; F-LP62-MED-001 AC-5 mechanism orphan to AC-7..8 entry + OBS-LP62-001 HS-002 §Failure-conditions parenthetical stale + OBS-LP62-002 17-site D7 pin sweep Interpretation #2 chosen; **POL-29 CODIFIED at FB50 D-672**; trend signal 0 HIGH first since pass-55; FB50 architect+PO+SM multi-agent; 178th single-commit)

**User directive:** Option 1 (continue cascade) chosen at D-664 checkpoint. Pass-55 begins next session.

## Task Status Table

| # | Status | Description |
|---|--------|-------------|
| 82 | **DONE** | **PREREQ-E fix-burst-28 CLOSED** — D-645; story v1.13; STORY-INDEX v2.117; 151st single-commit |
| 83 | **DONE** | **PREREQ-E pass-37 BLOCKED + FB29 CLOSED** — D-646; story v1.14; VP-153 v0.6; VP-INDEX v1.48; STORY-INDEX v2.118; 152nd single-commit |
| 84 | **DONE** | **PREREQ-E pass-38 BLOCKED + FB30 CLOSED** — D-647; story v1.15; STORY-INDEX v2.119; 153rd single-commit |
| 85 | **DONE** | **PREREQ-E pass-39 CLEAN★** — D-648; streak 0/3 → 1/3; 154th single-commit |
| 86 | **DONE** | **PREREQ-E pass-40 BLOCKED + FB31 CLOSED** — D-649; BC-2.01.016 v1.6; HS-002 v1.2; BC-INDEX v4.94; streak 1/3 → 0/3; 155th single-commit |
| 87 | **DONE** | **PREREQ-E pass-41 BLOCKED + FB32 CLOSED** — D-650; HS-002 v1.3; 156th single-commit |
| 88-89 | **DONE** | **PREREQ-E pass-42 BLOCKED + FB33 CLOSED** — D-651; ADR-027 v1.7; ARCH-INDEX v2.56; 157th single-commit |
| 90 | **DONE** | **PREREQ-E pass-43 CLEAN★** — D-652; streak 0/3 → 1/3; 158th single-commit |
| 91-92 | **DONE** | **PREREQ-E pass-44 BLOCKED + FB34 CLOSED** — D-653; story v1.16; VP-153 v0.7; BC-2.01.016 v1.7; streak 1/3 → 0/3; 159th single-commit |
| 93-94 | **DONE** | **PREREQ-E pass-45 BLOCKED + FB35 CLOSED** — D-654; story v1.17; STORY-INDEX v2.121; 160th single-commit |
| 95-96 | **DONE** | **PREREQ-E pass-46 BLOCKED + FB36 CLOSED** — D-655; HS-002 v1.4; story v1.18; STORY-INDEX v2.122; 161st single-commit |
| 97-98 | **DONE** | **PREREQ-E pass-47 BLOCKED + FB37 CLOSED** — D-656; story v1.19; BC-2.16.012 v1.16; BC-2.16.002 v1.21; HS-003 v1.6; BC-INDEX v4.96; STORY-INDEX v2.123; 162nd single-commit |
| 99-100 | **DONE** | **PREREQ-E pass-48 BLOCKED + FB38 CLOSED** — D-657; ADR-026 v1.13; story v1.20; error-taxonomy v1.31; ARCH-INDEX v2.57; STORY-INDEX v2.124; 163rd single-commit |
| 101-102 | **DONE** | **PREREQ-E pass-49 BLOCKED + FB39 CLOSED** — D-658; ADR-026 v1.14; VP-153 v0.8; story v1.21; HS-001 v1.4; +3 ACs; +3 Red Gate tests; ARCH-INDEX v2.58; VP-INDEX v1.50; STORY-INDEX v2.125; 164th single-commit |
| 103-104 | **DONE** | **PREREQ-E pass-50 BLOCKED + FB40 CLOSED** — D-659; story v1.22; VP-153 v0.9; VP-INDEX v1.51; STORY-INDEX v2.126; verification-architecture v1.41; verification-coverage-matrix v1.38; 165th single-commit |
| 105 | **DONE** | **PREREQ-E pass-51 CLEAN★** — D-660; streak 0/3 → 1/3; 166th single-commit |
| 106-107 | **DONE** | **PREREQ-E pass-52 BLOCKED + FB41 CLOSED** — D-661; BC-2.16.002 v1.22; BC-INDEX v4.97; streak 1/3 → 0/3; 167th single-commit |
| 108-109 | **DONE** | **PREREQ-E pass-53 BLOCKED + FB42 CLOSED** — D-662; F-LP53-HIGH-001 REJECTED Fork B; 2 MED cycle-snapshot fixes; POL-30 established; 168th single-commit |
| 110 | **DONE** | **PREREQ-E pass-54 BLOCKED + FB43 CLOSED** — D-663; BC-2.16.002 v1.23; BC-INDEX v4.98; first pass under Fork B surfaced Fork-A residual; 169th single-commit |
| 111 | **DONE** | **D-664 DURABLE PRE-/CLEAR RESUME SNAPSHOT** — this burst; STATE.md v7.350→v7.351; SESSION-HANDOFF.md v7.350→v7.351; SESSION-D664-TASKS.md created; CYCLE-SNAPSHOT §D-664 appended; SESSION-D644-TASKS.md v1.19→v1.20 close-out; 170th consecutive single-commit (restoring TD-VSDD-053 discipline) |
| 112 | **DONE** | **PREREQ-E pass-55 CLEAN★** — D-665; streak 0/3 → 1/3 (4th CLEAN advance); 2 non-blocking OBS; novelty ZERO; 9th 3-CLEAN sequence attempt begins |
| 113 | **DONE** | **D-665 state-manager bookkeeping burst** — 171st consecutive single-commit; OBS-LP55-001 dispatch-table fix (line 67 v1.23→v1.22); OBS-LP55-002 [process-gap] queued as Codification Queue item 11; STATE+HANDOFF v7.351→v7.352; SESSION-D664-TASKS.md v1.20→v1.21; pass-55 report persisted; CYCLE-SNAPSHOT §D-665 appended |
| 114 | **DONE — BLOCKED** | PREREQ-E pass-56 (0C+1H+0M+0L+0OBS; F-LP56-HIGH-001 production call-site for mark_query_phase_started() unspecified + Architecture Compliance Rule forbade only viable call site; structural call-graph defect; novelty HIGH; streak 1/3→0/3 10th reset; FB44 closed in-scope D-666) |
| 115 | **DONE** | D-666 FB44 SINGLE-COMMIT CLOSURE — architect Option A adjudication: boot.rs MAY ONE designated insertion; ADR-026 v1.15 + BC-2.16.012 v1.17 + VP-156 v0.9 + story v1.23 + STORY-INDEX v2.127 + BC-INDEX v4.99 + ARCH-INDEX v2.59 + VP-INDEX v1.52; 172nd consecutive single-commit (TD-VSDD-053 STABLE) |
| 116 | **DONE — BLOCKED** | PREREQ-E pass-57 (0C+2H+1M+0L+1OBS; F-LP57-HIGH-001 runtime_deliverables + F-LP57-HIGH-002 SS-22 + F-LP57-MED-001 tracing-test + OBS-LP57-001 Path A; novelty HIGH; FB45 closed in-scope D-667) |
| 117 | **DONE** | D-667 FB45 SINGLE-COMMIT CLOSURE — architect+PO multi-agent; ADR-026 v1.16 + ADR-022 v1.4 + BC-2.16.012 v1.18 + VP-156 v0.10 + story v1.24 + STORY-INDEX v2.128 + BC-INDEX v5.00 + ARCH-INDEX v2.60 + VP-INDEX v1.53; 173rd consecutive single-commit (TD-VSDD-053 STABLE) |
| 118 | **DONE — BLOCKED** | PREREQ-E pass-58 (2nd pass of restart-9 sequence; BLOCKED 2 HIGH + 3 MED + 1 OBS; F-LP58-HIGH-001 ADR-027 title-vs-§D1 contradiction + F-LP58-HIGH-002 HS-003-05 Step 1 vs AC-9 gate FB45-sibling-sweep-gap-#15+ + F-LP58-MED-001/002/003 + OBS-LP58-001; novelty HIGH; FB46 closed all in-scope D-668) |
| 119 | **DONE** | D-668 FB46 MULTI-AGENT CLOSURE — 174th consecutive single-commit; architect ADR-027 v1.8 + ARCH-INDEX v2.61; PO story v1.25 + HS-PREREQ-E-003 v1.7 + STORY-INDEX v2.129; state-manager pass-58 report + STATE+HANDOFF v7.354→v7.355 + SESSION-D664-TASKS.md v1.3→v1.4 + CYCLE-SNAPSHOT §D-668 appended |
| 120 | **DONE — BLOCKED** | PREREQ-E pass-59 (3rd pass of restart-9 sequence; BLOCKED 2 HIGH + 1 MED + 1 OBS; F-LP59-HIGH-001 CAP-029 mis-anchor + F-LP59-HIGH-002 risk_mitigations FB39-renumbering-drift + F-LP59-MED-001 ADR-027 "deprecation" framing 5-site sibling-sweep + OBS-LP59-001 cosmetic; novelty HIGH; 3 of 4 self-introduced by FB46; FB47 closed all in-scope D-669) |
| 121 | **DONE** | D-669 FB47 MULTI-AGENT CORRECTIVE CLOSURE — 175th consecutive single-commit; architect ADR-026 v1.17 + ARCH-INDEX v2.62; PO story v1.26 + BC-2.16.011 v1.7 + HS-PREREQ-E-002 v1.5 + STORY-INDEX v2.130; state-manager pass-59 report + BC-INDEX v5.01 + STATE+HANDOFF v7.355→v7.356 + SESSION-D664-TASKS.md v1.4→v1.5 + CYCLE-SNAPSHOT §D-669 appended |
| 122 | **DONE — BLOCKED** | PREREQ-E pass-60 (4th pass of restart-9 sequence; BLOCKED 1 HIGH F-LP60-HIGH-001 BC-2.16.012 §Changelog ASCENDING-at-top 4th POL-26 recurrence + 1 LOW F-LP60-LOW-001 §risk_mitigations AC-7..8 path-citation ambiguity + 1 OBS-LP60-001 BC-INDEX schema asymmetry; novelty MEDIUM-HIGH; FB48 closed in-scope D-670) |
| 123 | **DONE** | D-670 FB48 SINGLE-COMMIT CLOSURE — 176th consecutive single-commit; state-manager BC-2.16.012 v1.19 §Changelog row reorder (POL-26 bookkeeping) + BC-INDEX v5.02; PO story v1.27 §risk_mitigations AC-7..8 disambiguation + STORY-INDEX v2.131; pass-60 report persisted; STATE+HANDOFF v7.356→v7.357; SESSION-D664-TASKS.md v1.5→v1.6; CYCLE-SNAPSHOT §D-670 appended; OBS-LP60-001 cycle-close queue item 12 added |
| 124 | **DONE — BLOCKED** | PREREQ-E pass-61 (5th pass of restart-9 sequence; BLOCKED 1 HIGH F-LP61-HIGH-001 story §Changelog v1.23 out-of-position 5th POL-26 recurrence + 1 MED F-LP61-MED-001 §risk_mitigations AC-4..6 behavioral-equivalence misattribution; novelty MEDIUM; POL-29 #18+; FB49 closed in-scope D-671) |
| 125 | **DONE** | D-671 FB49 MULTI-AGENT CLOSURE — state-manager story §Changelog v1.23 row reorder (POL-26-COROLLARY bookkeeping) + story v1.27→v1.28 + STORY-INDEX v2.131→v2.132; PO §risk_mitigations AC-4..6 Option (a) disambiguation; pass-61 report persisted; STATE+HANDOFF v7.357→v7.358; SESSION-D664-TASKS.md v1.6→v1.7; CYCLE-SNAPSHOT §D-671 appended; 177th consecutive single-commit (TD-VSDD-053 STABLE) |
| 126 | **DONE — BLOCKED** | PREREQ-E pass-62 (6th pass of restart-9 sequence; BLOCKED 0 HIGH + 1 MED + 2 LOW; F-LP62-MED-001 AC-5 mechanism orphan + OBS-LP62-001 HS-002 parenthetical + OBS-LP62-002 17-site D7 pin sweep Interpretation #2; novelty MEDIUM; POL-29 candidate evidence #19+#20; FB50 closed in-scope D-672) |
| 127 | **DONE** | D-672 FB50 MULTI-AGENT CLOSURE + POL-29 CODIFICATION — architect 4 files (BC-2.16.012 v1.20 + VP-156 v0.11 + ADR-022 v1.5 + BC-2.16.002 v1.24; 9 D7 pin sweep) + PO 3 files (story v1.29 + HS-002 v1.6 + HS-003 v1.8; 8 D7 pin sweep) + SM INDEX cascade (BC-INDEX v5.03 + VP-INDEX v1.54 + ARCH-INDEX v2.63 + STORY-INDEX v2.133) + policies.yaml v1.12 (POL-29 ACTIVE); 17-site total D7 pin sweep Interpretation #2; 178th consecutive single-commit (TD-VSDD-053 STABLE); cycle-close queue item 9 retired |
| 128 | **PENDING** | PREREQ-E pass-63 (7th pass of restart-9 sequence; under POL-29 active discipline going forward) |

## Pinned Artifact Versions (post-FB50, as of D-672)

| Artifact | Version |
|----------|---------|
| Story S-PLUGIN-PREREQ-E | v1.29 |
| BC-2.01.016 | v1.7 |
| BC-2.16.011 | v1.7 |
| BC-2.16.012 | v1.20 |
| BC-2.16.002 | v1.24 |
| ADR-026 | v1.17 |
| ADR-022 | v1.5 |
| ADR-027 | v1.8 |
| VP-153 | v0.9 |
| VP-154 | v0.6 |
| VP-155 | v0.5 |
| VP-156 | v0.11 |
| HS-PREREQ-E-001 | v1.4 |
| HS-PREREQ-E-002 | v1.6 |
| HS-PREREQ-E-003 | v1.8 |
| error-taxonomy | v1.31 |
| ARCH-INDEX | v2.63 |
| VP-INDEX | v1.54 |
| STORY-INDEX | v2.133 |
| BC-INDEX | v5.03 |
| policies.yaml | v1.12 |
| verification-architecture | v1.41 |
| verification-coverage-matrix | v1.38 |

## Story Spec Expansion (this session)

- Story AC count: 10 → 13 (added AC-3b, AC-3c, AC-11; added per FB39 PO dispatch D-658)
- Red Gate tests: 11 → 14 (added Tests 4, 5, 14 for new ACs; added per FB39 D-658)
- Story crates_touched: SS-17 added (PluginRuntime/WASM Plugin Runtime; added per FB37 architect D-656)
- Story architectural_decisions: ADR-022 added (per FB37 D-656)
- Story §References subsections: ADR-022 added + new Holdout Scenarios subsection (per FB39 D-658)

## Fork B Canonical Rule (POL-30, established FB42 D-662)

**Rule statement:** The §Postconditions Canonical Structured Event Catalog bullet-version-label `(vN.MM)` tracks **catalog-content-version** — the version at which the events table CONTENT last changed. BC frontmatter `version:` tracks **BC document version** — bumped on ANY BC change. These two version counters bump **independently** per their respective change-trigger semantics and MUST NOT be force-synchronized.

**Under Fork B:** BC-2.16.002 bullet label v1.21 + frontmatter v1.23 + 8 cite-pin sites at v1.21 is INTERNALLY CONSISTENT. The FB41 sync (v1.20→v1.21 in bullet label) was a misdiagnosis-induced fix; result was harmless by coincidence.

**POL-25 mandatory workspace-wide grep on every BC version bump per POL-29 candidate** is the discipline that prevents future sibling-sweep gaps.

**Evidence record:** 9-recurrence catalog-bullet sub-class (F-LP32 through F-LP52 sub-class instances) retrospectively closed as misdiagnosis-induced. Adversary applying frontmatter-version rule to catalog-content-version label.

## Known Issue — TD-VSDD-053 FB43 Two-Commit Deviation

FB43 state-manager dispatch produced TWO commits to factory-artifacts:
- Primary commit (169th): 7 files, 159 insertions — BC-2.16.002 v1.23 corrective row + BC-INDEX v4.98 corrective row + STATE.md + SESSION-HANDOFF.md + SESSION-D644-TASKS.md + pass-54 report + cycle-snapshot
- Second commit: pass-54 report `fix_burst_committed:` SHA-pin (1 line) — post-primary metadata fill

The second commit was a "fill in the SHA placeholder" commit. The factory-dispatcher hook did NOT flag (no "backfill"/"Stage 1"/"Stage 2" keywords in commit subjects) but the spirit of TD-VSDD-053 single-commit-per-burst is violated.

**Status:** One-time historical deviation. Documented here as known issue.

**Go-forward convention (effective D-664):** Use `fix_burst_committed: see-git-log` placeholder in adversary report YAML to avoid post-commit SHA-placeholder fills. This eliminates the two-commit pattern at the source.

**Triage options for future reference:**
- (A) Accept as known deviation; document going forward — SELECTED DEFAULT
- (B) Soft-reset HEAD~1 to combine into single atomic commit (factory-artifacts is local-only per orchestrator policy)
- (C) Use `see-git-log` placeholder convention going forward — ALSO SELECTED (primary prevention)

**D-664 (this commit) is the 170th consecutive single-commit, restoring TD-VSDD-053 discipline.**

## Cycle-Close Codification Queue (10 items — unchanged from D-663)

These items are non-blocking for pass-55 dispatch. All deferred to cycle-close per S-7.02.

1. OBS-LP38-001 [process-gap] — VP-INDEX narrative POL-9/POL-11 templated phrasing
2. OBS-LP41-001 (long-standing) — BC-2.22.001 modified-field format heterogeneity (out-of-perimeter)
3. F-LP41-OUT-OF-PERIMETER-001 — test-vectors.md:94 TD-VSDD-091 workspace candidate
4. F-LP41-OUT-OF-PERIMETER-002 — error-taxonomy.md:456,458 TD-VSDD-091 workspace candidate
5. F-LP42-WORKSPACE-001 — ADR-023:87-88 TD-VSDD-091 workspace candidate
6. F-LP42-WORKSPACE-002 — ADR-023:375 TD-VSDD-091 workspace candidate
7. F-LP42-WORKSPACE-003 — ADR-023:978-979 TD-VSDD-091 workspace candidate
8. F-LP42-WORKSPACE-004 — ADR-023:1030-1031 TD-VSDD-091 workspace candidate
9. ~~POL-29 candidate~~ — **CODIFIED as POL-29 at FB50 D-672** (policies.yaml v1.12; cycle-close queue item RETIRED)
10. POL-30 candidate — Fork B independent-versioning rule (operational FB42 onward; needs formal codification in .factory/policies.yaml)
11. **OBS-LP55-002 [process-gap]** — VP-template `proof_method:` + `verification_method:` field-duplication (benign template artifact; cycle-close VP-template review — either consolidate to single field or document intent of duplication; affects VP-153/154/155/156)
12. **OBS-LP60-001 [process-gap]** — BC-INDEX header column-count vs row column-count schema asymmetry (10 of 217 BC-INDEX rows carry a 7th "Version" column; header declares only 6 columns; pre-existing 59-pass-surviving pattern across non-PREREQ-E BCs)

**Plus non-blocking observations (queued from this session):**
- OBS-LP45-001: E-SPEC-012/013 variant non-canonicalized (test-writer-deferred)
- OBS-LP45-002: harness file-name scope note (pre-existing convention)
- OBS-LP54-001: FB42 "misdiagnosis-induced" recharacterization debate (non-blocking)
- OBS-LP54-002: story risk_mitigations enumeration incompleteness for AC-3b/3c/10/11 (non-blocking)

## Standing DO-NOT Directives (carry-forward, all intact)

- DO NOT push `factory-artifacts` to remote (orchestrator policy: local-only)
- DO NOT use `--no-verify` on any git command (TD-FACTORY-HOOK-BYPASS-001 P0)
- DO NOT add Claude attribution to commits (user explicit directive for prism)
- DO NOT dispatch PLUGIN-MIGRATION-001-A/B/C/D before PREREQ-E Phase 1d converges (3-CLEAN) and implementation lands
- DO NOT add entries to tech-debt-register without explicit human direction + concrete future dependency + specific story anchor
- DO NOT introduce the retired two-commit Stage-1/Stage-2/backfill chain (TD-VSDD-053 single-commit-per-burst only); use `fix_burst_committed: see-git-log` convention to avoid post-commit metadata fills
- DO NOT bypass git hooks or use `--no-verify` (POL-3)
- DO NOT commit files using Python/sed/echo bypass for .factory/ mutations (TD-FACTORY-HOOK-BYPASS-001; Edit/Write tools only)
- DO NOT clean up sibling worktrees (S-3.09 + S-PLUGIN-PREREQ-B + S-PLUGIN-PREREQ-C + W3-FIX-S307-001 remain by design)
- DO NOT directly edit policies.yaml without session-reviewer codification workflow at cycle-close
- DO NOT run PREREQ-E implementation TDD before Phase 1d 3-CLEAN spec convergence
- DO NOT declare convergence without meeting BC-5.39.001 (3 consecutive CLEAN passes required)
- DO NOT merge to develop without explicit user authorization
- DO NOT force-sync BC bullet-version-labels with frontmatter (Fork B POL-30: independent versioning)
- DO NOT modify v1.22/v4.97 changelog rows or any prior immutable changelog rows (POL-26 monotonic append; corrective rows ADD, never EDIT)
- DO NOT touch ADR-023 / ADR-022 / BC-2.16.004 / VP-155 / HS-PREREQ-E-002-05 / test-vectors.md / error-taxonomy.md for TD-VSDD-091 cleanup until coordinated workspace-wide cycle-close pass

## Resume Reading Order (Next Session After /Clear)

1. `.factory/STATE.md` (v7.351 post-D-664) — current_step + RESUME PROTOCOL + pre_compact_snapshot pointer
2. `.factory/SESSION-HANDOFF.md` (v7.351) — §POST-D664 DURABLE RESUME SNAPSHOT section
3. `.factory/cycles/wave-4-operations/SESSION-D664-TASKS.md` — this file
4. `.factory/cycles/wave-4-operations/S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md` — D-663 + D-664 sections
5. `.factory/cycles/wave-4-operations/SESSION-D644-TASKS.md` — prior session task list (D-645 through D-663 era; SUPERSEDED by this file)
6. `.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-{37..54}.md` — per-pass finding context if needed

## Session Continuation Behavior

At next session start after /clear:
1. Orchestrator MUST run factory-worktree-health (BLOCKING) before reading STATE.md
2. Orchestrator MUST read STATE.md → SESSION-HANDOFF.md → SESSION-D664-TASKS.md in that order
3. Orchestrator MUST verify HEAD on factory-artifacts is the 170th-single-commit SHA produced by D-664 (`git -C .factory log -1 --format='%H'`)
4. Orchestrator MUST verify TD-VSDD-053 stable (170 consecutive single-commits acknowledging FB43 one-time deviation)
5. Orchestrator MAY proceed directly to pass-55 dispatch per user "Option 1 continue cascade" directive — NO clarification needed
6. Orchestrator MUST inject Fork B canonical rule (POL-30) into the pass-55 adversary dispatch under "Project Policy Rubric"
7. Orchestrator MUST use `fix_burst_committed: see-git-log` in any new adversary report YAML going forward (no post-commit SHA-placeholder fills)

## Suggested Pass-55 Vector Rotation (Mandatory — Do Not Re-Use Exhausted Vectors)

Exhausted vectors (pass-37 through pass-54) are documented at length in STATE.md frontmatter `pass_trajectory` field. The following vectors are suggested for pass-55:

1. **FB43 close-watch** — BC-2.16.002 v1.23 corrective row + BC-INDEX v4.98 corrective row content semantic verification (do corrective rows accurately reframe the v1.22/v4.97 changelog under Fork B?)
2. **Fork B independent-versioning validation under POL-30** — verify NO artifact post-FB43 carries Fork-A-aligned phrasing ("synced with frontmatter" / "9th POL-23 catalog-bullet-label sub-class" type language)
3. **POL-25 workspace grep for retired Fork-A phrasings** across ALL artifacts (final sweep; BC-2.16.002 v1.23 + BC-INDEX v4.98 + STATE.md + SESSION-HANDOFF.md + cycle-snapshot + SESSION-D664-TASKS.md)
4. **Cross-doc Fork B reference coherence** — 6+ surfaces (STATE.md, SESSION-HANDOFF.md, cycle-snapshot, SESSION-D664-TASKS.md, pass-54 report, BC-2.16.002 v1.23, BC-INDEX v4.98) all reference Fork B canonical rule consistently
5. **TD-VSDD-053 FB43 deviation acknowledgment** — verify SESSION-D664-TASKS.md records the known issue; pass-55 does NOT raise FB43 two-commit as a new finding (it is a documented known deviation)
6. **AC traceability chain Phase A** — every AC in story body must trace to at least one Red Gate test + one BC postcondition; reverse-verify every BC postcondition has at least one AC trace
7. **HS body cross-reference completeness** — for each HS sub-scenario, verify ALL cited artifacts (BCs, ADRs, error codes, AC IDs) exist in canonical sources
8. **CLAUDE.md production-grade lens** — search for "TODO" / "FIXME" / "deferred" in story / BCs / VPs / HSs / ADRs body content (changelog rows exempt)
9. **ARCH-INDEX subsystem dependency completeness** — each SS-NN ↔ its prism-* crate ↔ which Tier — coherent across ARCH-INDEX + ADRs + stories
10. **POL-26 §Changelog FB43 new rows cell-count audit** — BC-2.16.002 v1.23 + BC-INDEX v4.98 + STATE.md D-663 + others schema compliance
