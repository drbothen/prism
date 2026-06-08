# SESSION-HANDOFF Archive — Superseded Resume Snapshots

**Cycle:** wave-5-e-demo-fidelity
**Compacted from:** SESSION-HANDOFF.md D-1056 compaction burst (2026-06-08)
**Active SESSION-HANDOFF.md:** contains only the latest §RESUME SNAPSHOT 2026-06-08-S-DEMO-003-MERGED + §Standing Orchestrator Process Rules
**This file contains:** all superseded resume snapshots from SESSION-HANDOFF.md versions prior to D-1056

## Archive Index

The following superseded snapshots are archived here (chronological order, most recent first):

| Snapshot | D-NNN | Date | STATE version |
|----------|-------|------|---------------|
| §RESUME SNAPSHOT 2026-06-07-D1047-S-MAINT-ECRED-MERGED-S-DEMO-003-REBASELINE-PAUSED | D-1047 | 2026-06-07 | v7.698 |
| RESUME SNAPSHOT 2026-06-07 — D-1046 / S-MAINT-ECRED-TAXONOMY-SYNC-001-MERGED-PR175 / S-DEMO-003-PAUSED | D-1046 | 2026-06-07 | v7.698 |
| RESUME SNAPSHOT 2026-06-07 — D-1045 / S-MAINT-ECRED-TAXONOMY-SYNC-001-LOCAL-CONVERGED | D-1045 | 2026-06-07 | v7.697 |
| RESUME SNAPSHOT 2026-06-06 — D-1044 / DURABLE-CLEAR-CHECKPOINT | D-1044 | 2026-06-06 | v7.695 |
| RESUME SNAPSHOT 2026-06-06 — D-1030 / PHASE-B-LANE-3-LOCAL-PASS-9-NEXT | D-1030 | 2026-06-06 | v7.674 |
| RESUME SNAPSHOT 2026-06-05 — POST-MERGE PR#172 / CASCADE CLOSED | D-1000 | 2026-06-05 | v7.650 |
| §RESUME SNAPSHOT 2026-06-04-WAVE5-DUAL-MERGE-COMPLETE-PARALLEL-PLAN-READY | D-988/D-989 | 2026-06-04 | v7.638 |
| §PRE-IMPLEMENTATION RESUME SNAPSHOT (D-545 — 2026-05-14) | D-545 | 2026-05-14 | v7.260 |
| §RESUME SNAPSHOT 2026-05-17 (Session FB53-FB75) | — | 2026-05-17 | — |
| §RESUME SNAPSHOT 2026-05-19 (Session FB-IMPL-1..10 + PR #151 In Flight) | — | 2026-05-19 | — |
| §RESUME SNAPSHOT 2026-05-20 (PR #151 Merge + PR #152 Maintenance) | — | 2026-05-20 | — |
| §RESUME SNAPSHOT 2026-05-20-EVE (PLUGIN-MIGRATION-001-D Cascade Pass-4 Decisions Locked) | — | 2026-05-20 | — |
| §RESUME SNAPSHOT 2026-05-21 (PLUGIN-MIGRATION-001-D LOCAL Spec Cascade CONVERGED) | — | 2026-05-21 | — |
| §RESUME SNAPSHOT 2026-05-22 (PLUGIN-MIGRATION-001-D LOCAL Cascade EXIT) | — | 2026-05-22 | — |
| §RESUME SNAPSHOT 2026-05-22-PLUGIN-E (pass-4 CLEAN streak 1/3) | — | 2026-05-22 | — |
| §RESUME SNAPSHOT 2026-05-23-PLUGIN-E-CONVERGED | — | 2026-05-23 | — |
| §RESUME SNAPSHOT 2026-05-23-MULTI-TENANT-OVERLAY-DESIGN | — | 2026-05-23 | — |
| §RESUME SNAPSHOT 2026-05-23-PATH-C-DUAL-WORKTREE | — | 2026-05-23 | — |
| §RESUME SNAPSHOT 2026-05-24-S-CONFIG-OPTION-B-EXIT | — | 2026-05-24 | — |
| §RESUME SNAPSHOT 2026-05-24-CLEAR-CHECKPOINT-BOTH-PRS | D-823 | 2026-05-24 | — |
| §RESUME SNAPSHOT 2026-05-25-PRE-RESTART-SYSPOLICYD | D-824 | 2026-05-25 | — |
| Additional earlier snapshots (Wave 4 through Wave 3) | various | 2026-04..05 | — |

**Full content of all superseded snapshots** is in git history on the `factory-artifacts` branch — recover with:
```bash
git -C .factory log --oneline | head -20  # find the pre-compaction commit SHA
git -C .factory show <pre-compaction-sha>:SESSION-HANDOFF.md > /tmp/session-handoff-full.md
```

## Most Recent Superseded Snapshot (D-1047)

The most recently superseded snapshot is reproduced below for quick reference:

---

### §RESUME SNAPSHOT 2026-06-07-D1047-S-MAINT-ECRED-MERGED-S-DEMO-003-REBASELINE-PAUSED

**STATE v7.698. D-1047 — S-MAINT-ECRED-TAXONOMY-SYNC-001 MERGED PR #175 develop@c603741d 2026-06-07. DRIFT-ECRED-TAXONOMY-001 RESOLVED. ADR-035 v1.2 canonical E-CRED-001..010 namespace. S-DEMO-003 re-baseline PAUSED — awaiting user go-ahead for rebase onto develop@c603741d.**

**Pipeline status:**
- Mode: brownfield | Phase: 3 | Wave-5 Phase B in progress
- develop HEAD: `c603741d`
- STATE: v7.698 | BC-INDEX: v5.94 | STORY-INDEX: v2.315

**What just completed:** S-MAINT-ECRED-TAXONOMY-SYNC-001 MERGED. ADR-035 v1.2 established canonical E-CRED-001..010 namespace. error-taxonomy.md v1.61→v1.62. prism-core CredentialStoreError variants renamed. S-DEMO-003 needs re-baseline: resolution.rs E-CRED-005→E-CRED-008.

**NEXT:** User go-ahead to re-baseline S-DEMO-003 onto develop@c603741d + run E-CRED re-baseline fix.

**Standing Authorization D-989:** Full autonomous Wave-5 A→B→C active.

---

## §RESUME SNAPSHOT 2026-06-04-WAVE5-DUAL-MERGE-COMPLETE-PARALLEL-PLAN-READY (Key Content)

> This was the major planning checkpoint that established Phase B parallel execution. See key decisions:

**D-989 AUTONOMY GRANT (2026-06-04):** Full autonomous Wave-5 execution. Auto-advance phases + auto-merge on objective gates (LOCAL 3-CLEAN + PR-LEVEL 3-CLEAN + security MAY PROCEED + pr-reviewer APPROVE + all CI PASS). PAUSE for §7 spec-to-match-code amend / product-business decision / Level-3 escalation / CLAUDE.md edit. Standing rules NEVER waived.

**Phase B lanes (as planned at D-988):**
- S-DEMO-QUERY-PUSHDOWN-001 ← prism-query push-down
- S-DEMO-003 ← credential CLI + runbook
- S-SPEC-HTTP-METHOD-VALIDATION-001 ← validation.rs
- OCSF-CLASS-MIGRATION-001 ← sensor TOMLs

**Phase C (Claroty cluster):** S-DEMO-CLAROTY-PAGINATION-001 → S-DEMO-CLAROTY-TRAILING-SLASH-001 → S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 → S-DEMO-HARNESS-CLONE-PARITY-001

**All Phase B lanes subsequently merged as recorded in STATE.md Phase Progress table.**

---

## D-512 POST-CONVERGENCE ROADMAP + STANDING DIRECTIVES (Historical Reference)

_Archived from SESSION-HANDOFF.md §D-512. These standing orchestrator process rules were adopted 2026-05-09 and have been incorporated into CLAUDE.md. Kept here for historical completeness only — canonical versions are in CLAUDE.md._

**Standing Orchestrator Process Rules (adopted D-321 follow-up):**
- POL-29 axis recording on every adversary pass
- BC-5.39.001 3-CLEAN strict convergence (amended D-779 per CLEAN disambiguation)
- Single-commit-per-burst TD-VSDD-053
- Anti-volatile-pin TD-VSDD-091
- AD-017 credential opaqueness
- Correct-agent routing per CLAUDE.md routing table
- No pragmatic convergence — fix all issues before build (user_directive_persistent)

_Full historical content (Wave 3 backward) available in git history on factory-artifacts branch._
