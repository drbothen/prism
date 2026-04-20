# Convergence Trajectory — phase-2-patch

## Extracted from STATE.md on 2026-04-19

## Finding Progression

| Pass | Date | Findings | CRIT | HIGH | MED | LOW | Novelty | Counter |
|------|------|----------|------|------|-----|-----|---------|---------|
| 1 | 2026-04-17 | 29 | 6 | 9 | 9 | 5 | HIGH | 0/3 |
| 2 | 2026-04-17 | 24 | 6 | 7 | 6 | 5 | HIGH | 0/3 |
| 3 | 2026-04-17 | 21 | 3 | 5 | 7 | 6 | HIGH | 0/3 |
| 4 | 2026-04-17 | 7 | 0 | 3 | 2 | 2 | MEDIUM | 0/3 |
| 5 | 2026-04-17 | 4 | 0 | 0 | 3 | 1 | MEDIUM | 0/3 |
| 6 | 2026-04-17 | 3 | 0 | 0 | 3 | 0 | MEDIUM | 0/3 |
| 7 | 2026-04-17 | 2 | 0 | 0 | 1 | 1 | MEDIUM | 0/3 |
| 8 | 2026-04-17 | 0 | 0 | 0 | 0 | 0 | CLEAN | 1/3 |
| 9 | 2026-04-17 | 0 | 0 | 0 | 0 | 2 | CLEAN | 2/3 |
| 12 (anchoring audit) | 2026-04-17 | 26 | 9 | 11 | 4 | 2 | COMPREHENSIVE | RESET→0/3 |
| 13 | 2026-04-17 | 8 | 4 | 4 | 0 | 0 | HIGH | 0/3 |
| 14 | 2026-04-17 | 4 | 0 | 2 | 2 | 1 | HIGH | 0/3 |
| 15 | 2026-04-17 | 2 | 0 | 0 | 2 | 2 | MEDIUM | 0/3 |
| 16 | 2026-04-17 | 1 | 0 | 0 | 1 | 3 | MEDIUM | 0/3 |
| 17 | 2026-04-17 | 1 | 0 | 0 | 0+2obs | 0 | MEDIUM | 0/3 |
| 18 | 2026-04-17 | 3 | 0 | 1 | 2 | 3 | MEDIUM | 0/3 |
| 19 | 2026-04-17 | 6 | 0 | 1 | 5 | 2 | MEDIUM | 0/3 |
| 20 | 2026-04-17 | 12 | 2 | 5 | 2 | 3 | HIGH | 0/3 |
| 21 | 2026-04-17 | 8 | 0 | 3 | 3 | 2 | MEDIUM | 0/3 |
| 22 | 2026-04-17 | 6 | 0 | 3 | 1 | 2 | MEDIUM | 0/3 |
| 23 | 2026-04-18 | 7 | 0 | 4 | 1 | 2 | HIGH | 0/3 |
| 24 | 2026-04-18 | 3 | 0 | 2 | 1 | 0 | MEDIUM | 0/3 |
| 25 | 2026-04-19 | 14 | 0 | 5 | 7 | 2 | MEDIUM-HIGH | 0/3 |
| 26 | 2026-04-19 | 15 | 0 | 7 | 6 | 2 | HIGH | 0/3 |
| 27 | 2026-04-19 | 9 | 2 | 4 | 2 | 1 | HIGH | 0/3 |
| 28 | 2026-04-19 | 5 | 0 | 2 | 2 | 1 | MEDIUM | 0/3 |
| 29 | 2026-04-19 | 5 | 0 | 2 | 2 | 1 | MEDIUM | 0/3 |
| 30 | 2026-04-19 | 4 | 0 | 0 | 3 | 1 | MEDIUM | 0/3 |
| 31 | 2026-04-19 | 6 | 0 | 1 | 4 | 1 | MEDIUM | 0/3 |
| 32 | 2026-04-19 | 2 | 0 | 0 | 1 | 1 | MEDIUM | 0/3 |
| 33 | 2026-04-19 | 3 | 0 | 1 | 2 | 0 | MEDIUM | 0/3 |
| 34 | 2026-04-19 | 3 | 0 | 1 | 2 | 0 | MEDIUM | 0/3 |
| 35 | 2026-04-19 | 12 | 2 | 6 | 3 | 0 | HIGH | 0/3 |
| 36 | 2026-04-19 | 4 | 0 | 2 | 1 | 1 | MEDIUM | 0/3 |
| 37 | 2026-04-19 | 3 | 0 | 1 | 1 | 1 | MEDIUM | 0/3 |
| 38 | 2026-04-19 | 3 | 0 | 1 | 0 | 0 | MEDIUM | 0/3 |
| 39 | 2026-04-19 | 8 | 0 | 5 | 2 | 0 | MEDIUM | 0/3 |
| 40 | 2026-04-19 | 4 | 0 | 2 | 1 | 1 | MEDIUM | 0/3 |
| 41 | 2026-04-19 | 3 | 0 | 1 | 1 | 0 | MEDIUM | 0/3 |
| 42 | 2026-04-19 | 0 | 0 | 0 | 0 | 0 | CLEAN | 1/3 |
| 43 | 2026-04-19 | 5 | 0 | 3 | 1 | 1 | MEDIUM-HIGH | RESET→0/3 |

## Trajectory Shorthand

29→24→21→7→4→3→2→CLEAN→26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→3→3→12→4→3→3→8→4→3→0→5

## Key Events

- **Pass 8**: First clean pass (convergence counter 1/3)
- **Pass 9**: Second clean pass (convergence counter 2/3)
- **Burst 11**: CAP-031 created; CAP-020 semantic mismatch corrected; counter RESET to 0 (spec change)
- **Pass 12**: Comprehensive semantic anchoring audit — PRD §7 CAP title cover-up discovered
- **Pass 23 (2026-04-18)**: VP-layer staleness surfaces as new drift class; Policy 9 adopted in Burst 24
- **Pass 27**: First CRIT in 16 passes — supplement fabrication root cause
- **Pass 29**: Whack-a-mole pattern confirmed — scripted sweep introduced in Burst 30
- **Pass 30**: Scripted sweep VERIFIED by adversary; zero 2-col title drifts confirmed
- **Pass 32**: Major decay (6→2); tool-naming drift axis surfaced
- **Burst 40**: All 7 deferred items closed (deferred_items_count: 0)

## Per-Pass Details (frontmatter extracted)

### Pass 1 (2026-04-17)
29 findings (6 CRIT, 9 HIGH, 9 MED, 5 LOW); convergence counter reset; fixes dispatched in Burst 4a (arch) + Burst 4b (po/sw/sm)

### Pass 2 (2026-04-17)
24 findings (6 CRIT, 7 HIGH, 6 MED, 5 LOW); convergence counter still at 0

### Pass 3 (2026-04-17)
21 findings (3 CRIT, 5 HIGH, 7 MED, 6 LOW); convergence counter still at 0

### Pass 4 (2026-04-17)
7 findings (0 CRIT, 3 HIGH, 2 MED, 2 LOW); convergence trajectory 29→24→21→7

### Pass 5 (2026-04-17)
4 findings (0 CRIT, 0 HIGH, 3 MED, 1 LOW); trajectory 29→24→21→7→4; CRIT/HIGH zero 2nd consecutive pass

### Pass 6 (2026-04-17)
3 findings (0 CRIT, 0 HIGH, 3 MED, 0 LOW); trajectory 29→24→21→7→4→3; CRIT/HIGH zero 3rd consecutive pass

### Pass 7 (2026-04-17)
2 findings (0 CRIT, 0 HIGH, 1 MED, 1 LOW); trajectory 29→24→21→7→4→3→2; CRIT/HIGH zero 4th consecutive pass

### Pass 8 (2026-04-17)
CLEAN — 0/0/0/0

### Pass 9 (2026-04-17)
CLEAN — 0/0/0/+2 LOW; counter 2/3; then reset by Burst 11

### Pass 12 (2026-04-17)
26 findings across 6 anchoring axes (9 CRIT, 11 HIGH, 4 MED, 2 LOW); BLOCK convergence (specialized comprehensive anchoring audit, not standard pass)

### Pass 13 (2026-04-17)
8 findings (4 CRIT, 4 HIGH, 0 MED, 0 LOW) across 4 anchoring axes; trajectory 26 → 8 = 69% decay; BLOCK counter at 0/3

### Pass 14 (2026-04-17)
4 findings (0 CRIT, 2 HIGH, 2 MED, 1 observation); trajectory 26 → 8 → 4 = 50% decay; BLOCK counter at 0/3

### Pass 15 (2026-04-17)
2 findings (0 CRIT, 0 HIGH, 2 MED, 2 LOW observations); trajectory 26 → 8 → 4 → 2 = 50% decay; CRIT/HIGH zero 2nd consecutive; BLOCK at 0/3 on MED anchor-integrity

### Pass 16 (2026-04-17)
1 finding (0 CRIT, 0 HIGH, 1 MED, 3 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 = 50% decay; CRIT/HIGH zero 3rd consecutive; BLOCK at 0/3 on MED anchor-integrity

### Pass 17 (2026-04-17)
3 findings (0 CRIT, 1 HIGH, 0 MED + 2 LOW observations elevated to MED per semantic_anchoring_integrity policy); trajectory 26 → 8 → 4 → 2 → 1 → 1 (stable at 1); BLOCK at 0/3

### Pass 18 (2026-04-17)
3 findings (0 CRIT, 1 HIGH, 2 MED, 3 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 (scope-expansion uptick); BLOCK at 0/3

### Pass 19 (2026-04-17)
6 findings (0 CRIT, 1 HIGH, 5 MED, 2 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 (scope-expansion continuing); BLOCK at 0/3

### Pass 20 (2026-04-17)
12 findings (2 CRIT, 5 HIGH, 2 MED, 3 LOW obs); trajectory ...→ 6 → 12; BLOCK at 0/3

### Pass 21 (2026-04-17)
8 findings (0 CRIT, 3 HIGH, 3 MED, 2 LOW obs); trajectory ...→ 12 → 8 (decay + no new axes); BLOCK at 0/3

### Pass 22 (2026-04-17)
6 findings (0 CRIT, 3 HIGH, 1 MED, 2 LOW obs); trajectory ...→ 8 → 6 (decay, new policy-8 surfacing); BLOCK at 0/3

### Pass 23 (2026-04-18)
7 findings (0 CRIT, 4 HIGH, 1 MED, 2 LOW); trajectory ...→ 6 → 7 (uptick — new drift class: architecture-layer staleness after VP-INDEX updates); BLOCK at 0/3; novelty HIGH

### Pass 24 (2026-04-18)
3 findings (0 CRIT, 2 HIGH, 1 MED, 0 LOW); trajectory ...→7→3 (decay resumed post-23 uptick); CRIT=0 for 13th consecutive pass; Policy 9 first substantive surfacing; BLOCK at 0/3; novelty MEDIUM

### Pass 25 (2026-04-19)
14 findings (0 CRIT, 5 HIGH, 7 MED, 2 LOW); trajectory ...3→14; BLOCK at 0/3; novelty MEDIUM-HIGH — fresh-scope sampling (STORY-INDEX frontmatter, BC-INDEX status column, PRD narrative, Wave 4 story body titles, S-5.09 stdio mis-anchor, DI-017 orphan)

### Pass 26 (2026-04-19)
15 findings (0 CRIT, 7 HIGH, 6 MED, 2 LOW); trajectory ...14→15; BLOCK at 0/3; novelty HIGH — Burst 26 regression (S-4.06 marker) + systematic Wave-4 BC title drift + PRD §5b Test Vectors supplement absent + 7 orphan domain invariants + SS-16 BC non-standard format

### Pass 27 (2026-04-19)
9 findings (2 CRIT, 4 HIGH, 2 MED, 1 LOW); trajectory ...15→9; BLOCK at 0/3; novelty HIGH — 6 of 9 findings in new test-vectors.md supplement including TV-002 wrong token TTL/UUID/removed-error-code + TV-006 wrong case states/error-code + TV-010 mis-attributed DI-031; plus S-1.14/.15 non-canonical BC table schema, S-1.09 E-FLAG-002/003 drift, S-2.01 BC-2.15.002 title drift, BC-2.16.001/.009 body-vs-index priority drift

### Pass 28 (2026-04-19)
5 findings (0 CRIT, 2 HIGH, 2 MED, 1 LOW); trajectory ...9→5; strong decay; CRIT=0 streak restored; novelty MEDIUM

### Pass 29 (2026-04-19)
5 findings (0 CRIT, 2 HIGH, 2 MED, 1 LOW); trajectory flatlined ...5→5; novelty MEDIUM — same-class drift as Burst 28 closed findings; key HIGH: S-1.10 BC-2.09.004 has factually-wrong title explicitly documented as wrong in BC-INDEX v4.6 changelog

### Pass 30 (2026-04-19)
4 findings (0 CRIT, 0 HIGH, 3 MED, 1 LOW); trajectory ...5→5→4; no HIGH first time this cycle; novelty MEDIUM — scripted sweep verified (0 drifts in 2-col); new drift axes: 3-col schema descriptions, Policy 8 bidirectional AC gaps, Task 4 stale prose

### Pass 31 (2026-04-19)
6 findings (0 CRIT, 1 HIGH pattern, 4 MED, 1 LOW); trajectory ...4→6 uptick due to first comprehensive Policy 8 sweep across all 73 stories; novelty MEDIUM — H-001 13 BC-level AC-trace gaps across 6 stories; M-101 S-1.05 Task 6 three-tier propagation miss

### Pass 32 (2026-04-19)
2 findings (0 CRIT, 0 HIGH, 1 MED, 1 LOW obs); trajectory ...6→2 major decay; novelty MEDIUM — M-101 tool-naming drift axis; L-101 interface-definitions.md supplement missing Phase 3-patch tools (observation only)

### Pass 33 (2026-04-19)
3 findings (0 CRIT, 1 HIGH, 2 MED, 0 LOW); trajectory 3→3 flat; novelty MEDIUM — H-001 CAP-033 action.execute vs canonical action.write 17:3; M-001 test-vectors.md 5 stale execute_action refs; M-002 PRD 16 NFRs vs catalog 18

### Pass 34 (2026-04-19)
3 findings (0 CRIT, 1 HIGH, 2 MED); trajectory 3→3 flat; novelty MEDIUM — H-001 CAP-022 4 non-existent case tools; M-001 error-taxonomy missing 18 rows; M-002 api-surface missing 8 of 12 S-5.06 tools

### Pass 35 (2026-04-19)
12 findings (2 CRIT regressions, 6 HIGH, 3 MED, 1 OBS); Burst 35 introduced 2 CRIT regressions and 6 HIGH findings

### Pass 36 (2026-04-19)
4 findings (0 CRIT, 2 HIGH, 1 MED, 1 LOW); counter stays 0/3

### Pass 37 (2026-04-19)
3 findings (0 CRIT, 1 HIGH, 1 MED, 1 OBS); counter stays 0/3

### Pass 38 (2026-04-19)
3 findings (0 CRIT, 1 HIGH, 0 MED, 0 LOW, 2 OBS); HIGH-001 Wave 5 BC sum not propagated from Burst 38 S-5.06 0→4 update

### Pass 39 (2026-04-19)
8 findings (0 CRIT, 5 HIGH Policy 8 propagation + 2 MED + 1 OBS); Burst 40 cleanup validation surfaced Policy 8 propagation gaps

### Pass 59 (2026-04-20)
11 findings (3 HIGH, 4 MED, 3 LOW, 1 OBS); counter RESET 2→0; pre-build sweep introduced 3 HIGH + 4 MED + 3 LOW + 1 OBS. Root causes: Wave 1-8 anchor population wrong semantics; Step 5 inputs-format miss; 13 DTU stories referenced non-existent dtu-strategy.md.

### Pass 60 (2026-04-20)
6 findings (1 HIGH, 3 MED, 2 LOW); counter stays 0/3; HIGH-001 scope expansion (5 additional stories); MED-001 changelog version monotonicity violation across 70 stories; MED-002 subsumed by MED-001; MED-003 subsystems:[] contradicts anchor_subsystem: in 3 stories; LOW-001 manifest gap; LOW-002 observational.

### Pass 61 (2026-04-20)
4 findings (1 HIGH, 2 MED-class, 1 LOW-obs); counter stays 0/3; HIGH-001 scope-expansion pattern (S-4.07 File Structure table line 248); MED-001/002/003 duplicate-changelog extending to BCs and VPs (7 tombstone BCs + BC-2.03.005 + VP-014/015/021/030); LOW-001 22 BCs VP-TBD accepted as Phase 3 tech debt.

### Pass 62 (2026-04-20)
1 finding (0 CRIT, 0 HIGH, 1 MED, 0 LOW); counter stays 0/3; MED-001 BC-2.12.011 (status=retired) had duplicate 1.0 Changelog rows — scope gap from pass-61 Track B's removed-only filter. BC-2.12.012 verified clean. Trajectory: 11→6→4→1 (strong decay).
