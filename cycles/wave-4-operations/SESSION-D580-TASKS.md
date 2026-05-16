---
document_type: session-tasks
version: "1.3"
status: active
related_burst: D-585
predecessor_state: D-584
timestamp: 2026-05-16T01:00:00Z
---

# Session Task List — D-580 Durable Snapshot

This file persists the task list from the session covering D-570..D-579 (85 consecutive single-commit bursts).
Intended audience: orchestrator at next session start. Read alongside STATE.md + SESSION-HANDOFF.md + S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md.

## Task Status Table

| # | Status | Description | Blocking / Blocked-by |
|---|--------|-------------|----------------------|
| 1 | DONE | Step 9 worktree cleanup S-PLUGIN-PREREQ-D (.worktrees/ removed + local branch deleted) | — |
| 2 | DONE | D-570 closure burst (STATE+HANDOFF v7.274→v7.275; 76th single-commit) | — |
| 3 | DONE | Session-reviewer cycle-close for S-PLUGIN-PREREQ-D (D-571): 31 candidates → 18 codified + 9 subsumed + 2 Phase-5 deferred + 4 downgraded-immediate; 6 new POLs (21/23/24/25/26/27) + POL-7 amended + POL-22 added; policies.yaml v1.10→v1.11 | — |
| 4 | DONE | PREREQ-E spec authoring (D-574): PO+architect parallel; 3 BCs + 2 ADRs + 4 VPs + 3 HS + 1 story; BC-INDEX v4.81→v4.82; ARCH-INDEX v2.44→v2.45; STORY-INDEX v2.108→v2.109 | — |
| 5 | BLOCKED | PLUGIN-MIGRATION Wave 0 stories (PLUGIN-MIGRATION-001-A/B/C/D/E/F/G/H) | Blocked on PREREQ-E Phase 1d 3-CLEAN convergence + PREREQ-E implementation |
| 6 | DONE | OBS-LP35-001: verification-architecture.md:282 + ADR-023:732-733 Vec<String> rewrite (D-572; architect 2-site fix) | — |
| 7 | DONE | OBS-LP36-002: BC-INDEX workspace enumeration + count correction (D-572; SURPRISE — active count was 235→225 since v4.54 miscount; state-manager+PO) | — |
| 8 | DONE | F-LP16 prism-bin edition 2021→2024 maintenance fix-PR (D-573; PR #150 squash a5ab742c) | — |
| 9 | DONE | F-LP22 PluginError #[non_exhaustive] + ci.yml EXPECTED 30→31 maintenance fix-PR (D-573; same PR #150) | — |
| 10 | DONE | D-573 Step 9 cleanup MAINT-F-LP16-F-LP22 + post-merge burst (STATE+HANDOFF v7.277→v7.278; develop 95d46be2→a5ab742c; 79th single-commit) | — |
| 11 | DONE | D-574 PREREQ-E spec draft burst committed (STATE+HANDOFF v7.278→v7.279; 80th single-commit) | — |
| 12 | IN-PROGRESS | PREREQ-E Phase 1d adversarial cascade (passes 1–6 DONE + fix-bursts 1–5 DONE; fix-burst-6 NEXT; streak 0/3; trajectory 14→9→8→9→10→10) | Blocked on tasks 18+19 (fix-burst-6 + pass-7) |
| 13 | PENDING | PREREQ-E human approval gate (Phase 1d → Phase 2 transition) | Blocked on task 12 (3-CLEAN convergence) |
| 14 | PENDING | PREREQ-E per-story-delivery 8-step cycle (test-writer → implementer → LOCAL adversary 3-CLEAN → demo-recorder → push → pr-manager → squash-merge → post-merge state burst) | Blocked on tasks 12+13 |
| 15 | DONE | PREREQ-E fix-burst-3 (D-577): 8 findings; Path B chosen for auth_type_name(); 83rd single-commit | — |
| 16 | DONE | PREREQ-E pass-5 + fix-burst-5 (D-579): 10 findings; trajectory regression 9→10 (bookkeeping class); 85th single-commit | — |
| 17 | DONE | PREREQ-E pass-6 dispatched + report persisted (D-581; pass-6 BLOCKED 10 findings; report at cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-6.md; 87th single-commit) | — |
| 18 | DONE | PREREQ-E fix-burst-6 (10 findings closed): architect D-582 `bae9c46f` (8 closures: CRIT-001+HIGH-001/003+MED-001/002/003/004+LOW-002) + story-writer D-583 `422b7dec` (CRIT-001 propagation) + state-manager D-584 (HIGH-002 STORY-INDEX v2.109→v2.110). 3 OBS queued cycle-close. | — |
| 19 | DONE | PREREQ-E pass-7 dispatched + report persisted (D-585; pass-7 BLOCKED 8 in-scope (4H+4M) + 4 OBS; trajectory DECREASE to 8; report at cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-7.md; 91st single-commit) | — |
| 22 | BLOCKED | PREREQ-E fix-burst-7 (8 findings: F-LP7-HIGH-001/002/003/004 + F-LP7-MED-001/002/003/004) — architect (HIGH-001/004+MED-002/003) + product-owner (HIGH-002+MED-004) parallel; state-manager (HIGH-003+MED-001) | Awaiting fix-burst-7 dispatch |
| 23 | PENDING | PREREQ-E pass-8 (fresh-context) | Blocked on Task #22 (fix-burst-7 completion) |
| 24 | PENDING | If pass-8 CLEAN: streak 1/3 → continue cascade toward 3-CLEAN convergence | Blocked on Task #23 verdict |

## Strategic Options — RESOLVED (D-581)

**User chose Option 1 — Continue Cascade (Production-Grade Default).** Pass-6 dispatched and BLOCKED (10 findings). Fix-burst-6 is NEXT. The three original options are recorded below for historical completeness.

### Option 1 — Continue Cascade (Production-Grade Default)
**Action:** Dispatch adversary pass-6 with fresh context, policies.yaml 27-POL rubric, and extended sweep template.
**Rationale:** BC-5.39.001 3-CLEAN protocol requires 3 consecutive CLEAN passes. Pass-5 showed regression (9→10) — bookkeeping class, but streak stays 0/3. Production-grade default per CLAUDE.md.
**Adversary estimate:** 3–5 more passes to reach 3-CLEAN. The cascade is finding genuine quality issues each pass.
**What to dispatch:** `vsdd-factory:adversary` fresh-context against all 18 PREREQ-E artifacts at versions pinned in §Artifact Pin below.

### Option 2 — Accept Current Spec + Human Review Checkpoint
**Action:** Pause cascade; human architect reviews current spec package manually; if satisfied, accept as Phase 1d CONVERGED with explicit human override.
**Rationale:** PREREQ-D spec took 43 passes; PREREQ-E cascade is showing declining novelty but flat count. Human review may be faster than 3–5 more automated passes.
**Risk:** Bypasses BC-5.39.001 3-CLEAN requirement. Requires explicit user direction to override (user_directive_persistent in STATE.md mandates "No pragmatic convergence").
**What to do:** User reviews S-PLUGIN-PREREQ-E-spec-pass-5.md findings, then signals ACCEPT or CONTINUE.

### Option 3 — Methodology Shift: Codify POL-28 Before Pass-6
**Action:** Dispatch session-reviewer to codify POL-28 (extension of POL-25 — enumerate ALL citation surfaces + index files + ADR frontmatter as mandatory sweep targets) before running pass-6.
**Rationale:** Many pass-5 findings were POL-25 enforcement gaps at surfaces not yet enumerated in POL-25. Codifying POL-28 first prevents pass-6 from finding the same class of gaps, accelerating convergence.
**Sequencing:** session-reviewer POL-28 codification (policies.yaml v1.11→v1.12) → state-manager burst → adversary pass-6 with updated rubric.
**Note:** This option is queued for cycle-close per Canonical Principle Rule 4 (lessons/codifications at cycle-close, not mid-cycle). However, a single-POL codification targeted at accelerating convergence may be appropriate mid-cycle if user judges the benefit > cost.

## Artifact Pin — PREREQ-E Spec Package (All 18 Items)

| Artifact | Current Version | Type |
|----------|----------------|------|
| S-PLUGIN-PREREQ-E story | v1.7 | story (draft; 10 ACs; 3 pts; deps PREREQ-F+A) — updated at D-583 FB6 |
| BC-2.01.016 | v1.3 | BC draft (SensorAuth open trait) |
| BC-2.16.011 | v1.3 | BC draft (CustomAdapter retirement) — updated at D-582 FB6 |
| BC-2.16.012 | v1.6 | BC draft (PluginRegistry dispatch migration) |
| ADR-026 | v1.8 PROPOSED | ADR (SensorAuth un-sealing) — updated at D-582 FB6 |
| ADR-027 | v1.4 PROPOSED | ADR (CustomAdapter deprecation/removal) — updated at D-582 FB6 |
| VP-153 | v0.5 draft | VP (proptest P0; cross-composition prevention) |
| VP-154 | v0.6 draft | VP (integration_test P1; behavioral equivalence) |
| VP-155 | v0.4 draft | VP (integration_test P0; no public API) — updated at D-582 FB6 |
| VP-156 | v0.5 draft | VP (proptest P1; register_write_tool uniqueness) — updated at D-582 FB6 |
| HS-001 (PREREQ-E) | v1.2 | Holdout scenario |
| HS-002 (PREREQ-E) | v1.1 | Holdout scenario |
| HS-003 (PREREQ-E) | v1.3 | Holdout scenario |
| error-taxonomy | v1.27 | PRD supplement (E-SPEC-012/013/014 + E-PLUGIN-012/020 authored; E-SPEC-008 RETIRED annotated) |
| ARCH-INDEX | v2.49 | Architecture index — updated at D-582 FB6 |
| VP-INDEX | v1.42 | VP index — updated at D-582 FB6 |
| STORY-INDEX | v2.110 | Story index (PREREQ-E draft row v1.7, 5 BCs) — updated at D-584 FB6 |
| BC-INDEX | v4.83 | BC index (active 225, draft 5, total 239) — updated at D-582 FB6 |

## Resume Reading Order (Next Session)

1. `.factory/STATE.md` — current_step (D-580 frontmatter + §RESUME PROTOCOL)
2. `.factory/SESSION-HANDOFF.md` — §POST-D580 DURABLE RESUME SNAPSHOT
3. `.factory/cycles/wave-4-operations/SESSION-D580-TASKS.md` — this file (task list + strategic options)
4. `.factory/cycles/wave-4-operations/S-PLUGIN-PREREQ-E-CYCLE-SNAPSHOT.md` — full cascade history + §D580 DURABLE SNAPSHOT section
5. `.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-{1..7}.md` — per-pass finding context if needed

## Standing DO-NOT Directives (carry-forward, all intact)

- DO NOT push `factory-artifacts` to remote (orchestrator policy: local-only; 80+ commit divergence is expected correct state)
- DO NOT use `--no-verify` on any git command (TD-FACTORY-HOOK-BYPASS-001 P0)
- DO NOT add Claude attribution to commits (user explicit directive for prism)
- DO NOT dispatch PLUGIN-MIGRATION-001-A/B/C/D before PREREQ-E Phase 1d converges (3-CLEAN) and implementation lands
- DO NOT add entries to tech-debt-register without explicit human direction + concrete future dependency + specific story anchor (Canonical Principle Rule 3)
- DO NOT introduce the retired two-commit Stage-1/Stage-2/backfill chain (TD-VSDD-053; single-commit-per-burst only)
- DO NOT bypass git hooks or use `--no-verify` (POL-3)
- DO NOT commit files using Python/sed/echo bypass for .factory/ mutations (TD-FACTORY-HOOK-BYPASS-001; Edit/Write tools only)
- DO NOT run adversary passes on S-PLUGIN-PREREQ-D spec (closed; 43 passes converged 2026-05-14)
- DO NOT clean up sibling worktrees (S-3.09 + S-PLUGIN-PREREQ-B + S-PLUGIN-PREREQ-C + W3-FIX-S307-001 remain by design)
- DO NOT directly edit policies.yaml without session-reviewer codification workflow at cycle-close
- DO NOT run PREREQ-E implementation TDD before Phase 1d 3-CLEAN spec convergence
- DO NOT declare convergence without meeting BC-5.39.001 (3 consecutive CLEAN passes required)
- DO NOT merge to develop without explicit user authorization (Standing Rule — user-auth-required-for-merges)
