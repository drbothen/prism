---
document_type: pipeline-state
level: ops
version: "1.0"
producer: state-manager
timestamp: 2026-04-20T00:00:00
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: prism
mode: brownfield
phase: 3
status: in_progress
started: 2026-04-13
repos:
  - poller-cobra
  - poller-express
  - poller-bear
  - poller-coaster
  - serveMyAPI
  - tally
  - axiathon
  - ocsf-proto-gen
  - mcp-claroty-xdome
current_step: "PHASE 3 DTU Wave 0 ACTIVE — 3 parallel worktrees (S-6.06, S-6.14, S-6.15) off develop. F99-001, F99-003, F99-004 closed. v0.48 lint hooks (validate-state-index-status-coherence + validate-anchor-capabilities-union) installed."
awaiting: "Wave 0 red gate (test-writer stubs + failing tests)"
drift_rebaseline_complete: 2026-04-20
vsdd_factory_version: "v0.48.0 (F99-001 auto-detection active)"
adjacent_regression_streak: 9
structural_fix_in_flight: "2 new lint hooks in vsdd-factory plugin (off-repo); 5 previously-installed hooks landed 2026-04-21"
linters_installed: 2026-04-21
pre_build_sweep_waves_completed: 8
story_corpus_sweep_complete: 2026-04-20
full_corpus_sweep_complete: 2026-04-20
total_artifacts_swept: 427
bc_corpus_sweep_complete: 2026-04-20
pre_build_sweep_requested: 2026-04-19
recent_passes_summary: "p59:11→p60:6→p61:4→p62:1→p63:3→p64:3→p65:2→p66:1→p67:0✓→p68:0✓→p69:0✓ RE-CONVERGED →housekeeping RESET 3→0→p70:8→p71:7→p72:5→p73 reorder→p74:4→p75:6→p76:6→p77:6→p78:3→p79:3 (9-pass adjacent-regression; see convergence-trajectory.md) →drift-rebaseline(v0.47.0)→p80:9(1C+4H+3M+1L)→p81:10(1C+4H+4M+1L)→p81remediated(10 fixed)→p82:7(3H+3M+1L)→p82remediated(7fixed+1obs)→p83:6(4H+2M)→p83remediated(6 fixed)→p84:3(3H)→p84remediated(3fixed)→p85:4(1C+1H+2M)→p85remediated(4fixed+1obs)→p86:8(2C+4H+2M)→p86remediated(8fixed)→p87:6(3H+3M)→p87remediated(6fixed)→p88:12(3H+6M+2L)→p88remediated(12fixed)→p89:6(3H+2M+1L)→p89remediated(5fixed)→p90:5(1C+2H+2M)→p90remediated(5fixed)→p91:1(1H)→p91remediated(1fixed)→p92:7(4H+3M)→p92remediated(7fixed)→p93:2(2M)→p93remediated(2fixed)→p94:3(3H)→p94remediated(3fixed)→p95:1(1H)→p95remediated(1fixed)→p96:4(3H+1M)→p96remediated(4fixed)→p97:4(2H+2M)→p97remediated(4fixed)→p98:3(2H+1M)→p98remediated→p99:4(1H+2M+1L)→CONVERGED-user-override"
convergence_counter: 3
convergence_status: "PHASE_2_PATCH_CONVERGED_DTU_READY"
pre_build_sweep_re_converged: 2026-04-20
pre_build_sweep_total_passes: 11
dtu_required: true
dtu_assessment: COMPLETE
dtu_assessment_approved: 2026-04-20
dtu_clones_built: in_progress
dtu_strategy: "Option 2 — DTU-first"
dtu_strategy_decided: 2026-04-20
dtu_readiness_audit_complete: 2026-04-21
dtu_readiness_verdict: "READY — all 14 stories scope-complete, anchored, externally-referenced, cross-consistent"
dtu_critical_path: "S-6.06 dtu-common (4 days, 8 points, blocks 13 others)"
dtu_total_points: 72
dtu_estimated_hours: 470
dtu_calendar_estimate_4person: "~11 days"
dtu_calendar_estimate_1person: "~5-6 weeks"
dtu_known_gaps_nonblocking: "fixture capture process; ES 7.x/OpenSearch variants; OTLP proto version pin; holdout traceability"
policy_registry_source_of_truth: .factory/policies.yaml
current_cycle: phase-3-dtu-wave-0
f99_001_resolved: 2026-04-21
f99_003_resolved: 2026-04-21
f99_004_resolved: 2026-04-21
historical_cycles:
  - name: phase-1-convergence
    passes: 33
    archived: 2026-04-18
    final_trajectory: "13 → 1 finding (converged at pass-33)"
layout_bootstrap_date: 2026-04-18
subsystem_count: 20
story_count: 75
bc_count_corrected: 200
cap_count: 34  # active; highest_cap_id: CAP-035
bc_index_version: "4.13"
vp_index_version: "v1.11"
story_index_version: "v1.41"
test_vectors_version: "2.6"
prd_version: "1.7"
error_taxonomy_version: "1.7"
holdout_index_version: "1.2"
capabilities_version: "1.5"
l2_index_version: "1.6"
module_decomposition_version: "1.2"
arch_index_version: "1.1"
verification_coverage_matrix_version: "1.10"
verification_architecture_version: "1.12"
deferred_items_count: 0
vp_count: 62
vp_tbd_resolution_complete: 2026-04-20
prd_supplements: [interface-definitions, error-taxonomy, nfr-catalog, test-vectors]
nfr_catalog_version: "1.5"
deployment_model: per-analyst-stdio
dtu_crate_count: 14
phase_0_approved: 2026-04-14
phase_1_converged: 2026-04-15
phase_2_started: 2026-04-15
phase_2_converged: 2026-04-15
phase_2_patch_converged: 2026-04-21
phase_2_patch_total_passes: 99
phase_2_patch_remediation_bursts: 20
phase_2_patch_findings_total_fixed: 95
phase_2_patch_convergence_rationale: "User override post pass-99. Semantic policies (4/5/6/7/8/9) all PASS at corpus scope. Remaining meta-doc drift (F99-001..004) deferred to 2 new lint hooks being built in vsdd-factory plugin (validate-state-index-status-coherence + validate-anchor-capabilities-union). Structural fix > adversarial iteration."
user_directive_persistent: "No pragmatic convergence. Fix all issues before build."
---

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Repository** | /Users/jmagady/Dev/prism |
| **Mode** | brownfield |
| **Language** | Rust |
| **Target Workspace** | per-analyst stdio (MCP server) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-04-21 |
| **Current Phase** | 3 (DTU Wave 0 ACTIVE) |
| **Current Step** | Phase 3 DTU Wave 0 ACTIVE. 3 parallel worktrees branched off develop (S-6.06, S-6.14, S-6.15). F99-001/003/004 closed. vsdd-factory v0.48.0 lint hooks installed. |

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| 0: Codebase Ingestion | passed | 2026-04-13 | 2026-04-14 | human-approved | converged |
| 1a: Product Brief + Domain Spec | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1b: PRD + Behavioral Contracts | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1c: Architecture + VPs | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 1d: Adversarial Spec Review | passed | 2026-04-15 | 2026-04-15 | 33-pass convergence | 13→1 converged |
| 2: Story Decomposition | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 2 Patch Cycle | CONVERGED-USER-OVERRIDE | 2026-04-16 | 2026-04-21 | user-override | …→0(58) counter=3/3 → reset by p59 → 11→6→4→1→3→3→2→1→0→0→0 RE-CONVERGED → housekeeping RESET counter=0/3 → p70:8 → p71:7 → p72:5 → p73 deterministic-reorder(132 BCs) → p74:4 (CRIT-002 9 VPs + 32 BCs) → p75:6 → p76:6 → p77:6+2OBS → p78:3+3OBS → p79:1+2MED+1OBS (9-pass adjacent-regression streak) → p80:9(1C+4H+3M+1L) → p80 remediated → p81:10 all fixed → p82:7 all fixed → p83:6 all fixed → p84:3 all fixed → p85:4(1C+1H+2M) all fixed → p86:8(regress) all fixed → p87:6 all fixed → p88:12(regress) all fixed → p89:6(3H+2M+1L) p89:5 fixed → p90:5(1C+2H+2M) all fixed → p91:1 fixed → p92:7(4H+3M) all fixed (+linters) → p93:2 fixed → p94:3(3H) fixed counter=0/3 → p95:1 fixed → p96:4(3H+1M) all fixed → p97:4(2H+2M) all fixed (PRD v1.7; STORY-INDEX v1.41) → p98:3(2H+1M) self-correcting burst → p99:4(final) → USER-OVERRIDE-CONVERGED |
| 3: TDD Implementation — DTU Wave 0 | ACTIVE | 2026-04-21 | — | — | Wave 0: 3 worktrees live (S-6.06, S-6.14, S-6.15); F99-001/003/004 closed |
| 4–7 | not-started | — | — | — | — |

## Current Phase Steps

_Pre-drift-work passes 59-79: see [cycles/phase-2-patch/phase-steps-p59-p79.md](cycles/phase-2-patch/phase-steps-p59-p79.md)_

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| Drift re-baseline (2026-04-20) | state-manager | COMPLETE | 293+39 files re-baselined under v0.47.0; fixpoint in 3 passes; 5 plugin PRs shipped upstream; drift detection corpus-wide-meaningful for first time |
| Pass-80 adversarial review | adversary | COMPLETE | 9 findings; counter reset to 0/3; report: cycles/phase-2-patch/adversary-pass-80.md |
| Pass-80 remediation burst | ba + po + sw | COMPLETE | 5 SS-20 BCs (BC-2.20.001-005); CAP-035 created; 6 existing specs updated; 3 stories re-anchored |
| Pass-81 adversarial review | adversary | COMPLETE | 10 findings (1C+4H+4M+1L); report: cycles/phase-2-patch/adversary-pass-81.md |
| Pass-81 remediation burst | ba + po + sw + architect + state-manager | COMPLETE | 10 findings fixed; SS-20 drift resolved; VP-061/062 filed; CAP-035 promoted P0 |
| Pass-82 adversarial review | adversary | COMPLETE | 7 findings (3H+3M+1L); counter 0/3; report: cycles/phase-2-patch/adversary-pass-82.md |
| Pass-82 remediation burst | po + architect + sw + ba | COMPLETE | F82-001–007 + OBS-082-001–003 closed |
| Pass-83 adversarial review | adversary | COMPLETE | 6 findings; report: cycles/phase-2-patch/adversary-pass-83.md |
| Pass-83 remediation burst | sw + architect | COMPLETE | F83-001–006 closed; STORY-INDEX v1.34, verification-architecture v1.8 |
| Pass-84 adversarial review | adversary | COMPLETE | 3 findings (3H); report: cycles/phase-2-patch/adversary-pass-84.md |
| Pass-84 remediation | architect | COMPLETE | F84-001–003 closed; verification-architecture v1.9, verification-coverage-matrix v1.7 |
| Pass-85 adversarial review | adversary | COMPLETE | 4 findings (1C+1H+2M); report: cycles/phase-2-patch/adversary-pass-85.md |
| Pass-85 remediation | architect | COMPLETE | F85-001–004 + OBS-85-001 closed; VP-INDEX v1.10, verification-architecture v1.10, verification-coverage-matrix v1.7 |
| Pass-86 adversarial review | adversary | COMPLETE | 8 findings (2C+4H+2M); report: cycles/phase-2-patch/adversary-pass-86.md |
| Pass-86 remediation | architect | COMPLETE | F86-001–008 closed; verification-coverage-matrix v1.8 |
| Pass-87 adversarial review | adversary | COMPLETE | 6 findings (3H+3M, 1 pass-86 regression); report: cycles/phase-2-patch/adversary-pass-87.md |
| Pass-87 remediation | architect + story-writer | COMPLETE | F87-001–006 closed; VP-INDEX v1.10, verification-architecture v1.11, verification-coverage-matrix v1.9, STORY-INDEX v1.35 |
| Pass-88 adversarial review | adversary | COMPLETE | 12 findings (3H+6M+2L+1OBS); REGRESSION from p87; report: cycles/phase-2-patch/adversary-pass-88.md |
| Pass-88 remediation | story-writer + architect | COMPLETE | F88-001–012 all closed; STORY-INDEX v1.36 |
| Pass-89 adversarial review | adversary | COMPLETE | 6 findings (3H+2M+1L); report: cycles/phase-2-patch/adversary-pass-89.md |
| Pass-89 remediation | story-writer | COMPLETE | F89-002–006 closed; STORY-INDEX v1.37; F89-007 LOW deferred |
| Pass-90 adversarial review | adversary | COMPLETE | 5 findings (1C+2H+2M); report: cycles/phase-2-patch/adversary-pass-90.md |
| Pass-90 remediation | story-writer + architect | COMPLETE | F90-001–005 all closed; STORY-INDEX v1.38, VP-INDEX v1.11, verification-architecture v1.12, verification-coverage-matrix v1.10 |
| Pass-91 adversarial review | adversary | COMPLETE | 1 finding (1H); counter 0/3; report: cycles/phase-2-patch/adversary-pass-91.md |
| Pass-91 remediation (VP-inputs sweep) | story-writer | COMPLETE | F91-001: 21 VP paths added across 10 stories; STORY-INDEX v1.39 |
| Pass-92 adversarial review | adversary | COMPLETE | 7 findings (4H+3M); counter 0/3; report: cycles/phase-2-patch/adversary-pass-92.md |
| Pass-92 remediation | story-writer + product-owner | COMPLETE | F92-001–007 all closed; STORY-INDEX v1.40; 5 lint hooks installed |
| Pass-93 adversarial review | adversary | COMPLETE | 2 findings (2M); counter 0/3; report: cycles/phase-2-patch/adversary-pass-93.md |
| Pass-93 remediation | story-writer + product-owner | COMPLETE | F93-001 (S-5.09 BC note) + F93-002 (BC-2.17.005 dual-anchor CAP-030/032) — both closed; BC-INDEX v4.13 |
| Pass-94 adversarial review | adversary | COMPLETE | 3 findings (3H); counter 0/3; report: cycles/phase-2-patch/adversary-pass-94.md |
| Pass-94 remediation | story-writer + product-owner | COMPLETE | F94-001 (S-5.09 queue cap 10000→1000 + WARN per-drop); F94-002 (BC-2.16.008 traceability CAP-029+CAP-030); F94-003 (PRD §7 CAP-032 row + dual-anchor + total 206→207) — BC-2.16.008 v1.5, S-5.09 v1.12, PRD v1.4 |
| Pass-95 adversarial review | adversary | COMPLETE | 1 finding (1H); counter 0/3; report: cycles/phase-2-patch/adversary-pass-95.md |
| Pass-95 remediation (F95-001) | product-owner | COMPLETE | PRD §7 line 869 BC-2.17.005 CAP-030→CAP-030,CAP-032; PRD v1.4→v1.5 |
| Pass-96 adversarial review | adversary | COMPLETE | 4 findings (3H+1M); counter 0/3; report: cycles/phase-2-patch/adversary-pass-96.md |
| Pass-96 remediation | story-writer + product-owner | COMPLETE | F96-001 (S-1.15 +CAP-030); F96-002 (S-1.14 +CAP-030); F96-003 (S-5.06 +CAP-032); F96-004 (PRD §2 SS-19 singular→dual); PRD v1.6; S-5.06 v1.10 |
| Pass-97 adversarial review | adversary | COMPLETE | 4 findings (2H+2M); report: cycles/phase-2-patch/adversary-pass-97.md |
| Pass-97 remediation | po + story-writer + state-manager | COMPLETE | F97-001 (PRD §2 SS-10 CAP-008+CAP-015); F97-002 (STORY-INDEX pin v4.12→v4.13); F97-003 (INDEX.md backfilled — actually completed by state-manager in pass-98 self-correcting burst); F97-004 (convergence-trajectory.md backfilled through pass-98); PRD v1.7; STORY-INDEX v1.41 |
| Pass-98 adversarial review | adversary | COMPLETE | 3 findings (2H+1M); report: cycles/phase-2-patch/adversary-pass-98.md; all 3 are claim-vs-artifact drift from F97-003/004 |
| Pass-98 remediation (self-correcting burst) | state-manager | COMPLETE | F98-001 (INDEX.md status + rows updated); F98-002 (convergence-trajectory.md p97+p98 rows + details); F98-003 (STATE.md frontmatter reconciled) |
| Pass-99 adversarial review | adversary | COMPLETE | 4 findings (1H+2M+1L) — all meta-doc drift class; semantic policies (4/5/6/7/8/9) all PASS; report: cycles/phase-2-patch/adversary-pass-99.md |
| Phase 2 patch CONVERGED | orchestrator + human | COMPLETE | 20 passes, 95 findings remediated, semantic policies clean; meta-doc drift deferred to off-repo lint hooks |
| Cleanup — archive transient artifacts | state-manager | COMPLETE | 132 → 51 hot-path files; 81 archived under archive/; commit 35cc6e2 |
| Hash check (post-cleanup) | state-manager | COMPLETE | TOTAL=448 MATCH=399 STALE=0 UNCOMPUTED=0; commit a454f2f |
| DTU readiness audit | adversary (Explore) | COMPLETE | All 14 DTU stories READY; ~72 points / ~470 hours; no blockers; 4 minor non-blocking gaps |
| Session handoff snapshot | state-manager | COMPLETE | Comprehensive checkpoint written; commit follows |
| F99-001/003/004 remediation (v0.48 lint) | state-manager | COMPLETE | INDEX.md status synced (F99-001); adversary-pass-76.md frontmatter scrubbed (F99-003-A); INDEX.md table retargeted/stripped (F99-003-B); burst-log.md p80-p99 backfilled 20 entries (F99-004) |
| Phase 2 → Phase 3 transition | state-manager | COMPLETE | STATE.md phase 2→3; current_cycle phase-3-dtu-wave-0; DTU Wave 0 ACTIVE; dtu_clones_built in_progress |
| DTU Wave 0 worktrees branched | devops-engineer | COMPLETE | 3 worktrees off develop: S-6.06 dtu-common, S-6.14 dtu-threatintel, S-6.15 dtu-nvd under /Users/jmagady/dev/prism/.worktrees/ |

## Decisions Log

| ID | Decision | Rationale | Phase | Date |
|----|----------|-----------|-------|------|
| D-001 | All sensor adapters ship as TOML spec files | Eat our own dog food | 1b | 2026-04-16 |
| D-002 | Un-retire BC-2.04.014/.06.009/.10.005 with Config-Reload semantics | Restores DI-003 tool-list notification enforcement | 2-patch | 2026-04-17 |
| D-003 | Deployment model: per-analyst stdio (not multi-tenant server) | Matches 1898 & Co MSSP analyst workflow | 0 | 2026-04-14 |
| D-004 | Credentials never transit AI context; reference-based model | AI-opaque credential security requirement | 1b | 2026-04-16 |
| D-005 | HIGH-003 resolved Case A: global `prism://sensors/health` | Per-analyst-stdio deployment makes `{client_id}` template redundant within process | 2-patch | 2026-04-19 |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Blocking Issues

No open blocking issues. See cycles/phase-2-patch/blocking-issues-resolved.md for closed items.

---

## Historical Content

Cycle files: [burst-log](cycles/phase-2-patch/burst-log.md) | [convergence-trajectory](cycles/phase-2-patch/convergence-trajectory.md) | [session-checkpoints](cycles/phase-2-patch/session-checkpoints.md) | [lessons](cycles/phase-2-patch/lessons.md) | [resolved-blockers](cycles/phase-2-patch/blocking-issues-resolved.md)

---

## Session Resume Checkpoint (2026-04-21-evening) — PHASE-3-DTU-WAVE-0-ACTIVE

_Previous checkpoint (PHASE-2-PATCH-CONVERGED/DTU-READY) archived: see [cycles/phase-2-patch/session-checkpoints.md](cycles/phase-2-patch/session-checkpoints.md)_

**STATUS:** Phase 3 DTU Wave 0 ACTIVE as of 2026-04-21. F99-001/003/004 closed (F99-002 was already resolved). vsdd-factory v0.48.0 lint hooks installed (validate-state-index-status-coherence + validate-anchor-capabilities-union). 3 parallel worktrees branched off develop for Wave 0.

**What just completed:**
- F99-001: INDEX.md status synced to PHASE-2-PATCH-CONVERGED-DTU-READY.
- F99-003-A: adversary-pass-76.md frontmatter retired refs scrubbed (adversarial-reviews/ → INDEX.md).
- F99-003-B: INDEX.md table: 28 broken archived-pass rows stripped; summary row added for archive.
- F99-004: burst-log.md p80-p99 backfilled (20 entries appended).
- Phase 2 → Phase 3 transition complete in STATE.md.
- DTU Wave 0: 3 worktrees branched off develop (S-6.06, S-6.14, S-6.15).

**What's next:** Wave 0 red gate — test-writer dispatched to stub failing tests for S-6.06, S-6.14, S-6.15 per TDD protocol.

**Worktrees (Wave 0):**
- `/Users/jmagady/dev/prism/.worktrees/S-6.06-dtu-common` (critical path, blocks 13 others)
- `/Users/jmagady/dev/prism/.worktrees/S-6.14-dtu-threatintel`
- `/Users/jmagady/dev/prism/.worktrees/S-6.15-dtu-nvd`

**Recent commits (factory-artifacts branch):**
```
[see: git -C /Users/jmagady/dev/prism/.factory log -5 --oneline]
```

**Corpus inventory snapshot (unchanged from Phase 2 convergence):**
- BC-INDEX v4.13: 200 active BCs | VP-INDEX v1.11: 62 VPs | STORY-INDEX v1.41: 75 stories
- CAP registry: 34 active (CAP-035 highest) | DI registry: 28 active
- 7 lint hooks total: 5 installed Phase 2 + 2 new v0.48 (validate-state-index-status-coherence, validate-anchor-capabilities-union)
- policies.yaml v1.1 (9 policies)

**Resume instructions for next session:**
1. Read `/Users/jmagady/dev/prism/.factory/STATE.md` (this file)
2. Run `git -C /Users/jmagady/dev/prism/.factory log -5 --oneline` to see recent state
3. Check worktree status: `git -C /Users/jmagady/dev/prism worktree list`
4. Wave 0 protocol: test-writer → implementer → demo-recorder → pr-manager per story
5. DTU stories: `/Users/jmagady/dev/prism/.factory/stories/S-6.{06,14,15}-dtu*.md`
6. Master DTU spec: `/Users/jmagady/dev/prism/.factory/specs/architecture/dtu-assessment.md`

**Corpus version reference:** BC-INDEX v4.13 | STORY-INDEX v1.41 | VP-INDEX v1.11 | capabilities v1.5 | L2-INDEX v1.6 | ARCH-INDEX v1.1 | prd.md v1.7 | error-taxonomy v1.7 | holdout-index v1.2 | verification-coverage-matrix v1.10 | verification-architecture v1.12 | test-vectors v2.6 | nfr-catalog v1.5 | policies.yaml v1.1

**User directives (carry forward):**
- "No pragmatic convergence. Fix all issues before build."
- DTU-first strategy (Option 2 approved 2026-04-20)
- 7 linters now installed (v0.48.0 complete)

**Key files:**
[burst-log](cycles/phase-2-patch/burst-log.md) | [convergence-trajectory](cycles/phase-2-patch/convergence-trajectory.md) | [adversary-pass-99.md](cycles/phase-2-patch/adversary-pass-99.md) | [INDEX.md](cycles/phase-2-patch/INDEX.md)

## Agent Routing Quick Reference

| Task | Agent |
|------|-------|
| BC body/frontmatter/error-taxonomy/supplements | `vsdd-factory:product-owner` |
| Story body/frontmatter/AC edits, STORY-INDEX | `vsdd-factory:story-writer` |
| Architecture docs, VP files, ARCH-INDEX, VP-INDEX | `vsdd-factory:architect` |
| STATE.md / INDEX.md / commits / pushes | `vsdd-factory:state-manager` |
| Adversarial review (read-only, fresh context) | `vsdd-factory:adversary` |
| Cross-document validation | `vsdd-factory:consistency-validator` |
