---
document_type: pipeline-state
level: ops
version: "7.782"
producer: state-manager
timestamp: 2026-06-13T16:00:00Z
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: prism
mode: brownfield
phase: 3
status: in_progress
started: 2026-04-13
repos: [poller-cobra, poller-express, poller-bear, poller-coaster, serveMyAPI, tally, axiathon, ocsf-proto-gen, mcp-claroty-xdome]
safe_to_compact: true

# ── CANONICAL CURRENT-STATE VALUES (authoritative; do not drop in future compactions) ──
develop_head: "939f36ce"
story_b_head: "15bedc12"
bc_index_version: "6.43"
vp_index_version: "1.79"
story_index_version: "v2.369"
arch_index_version: "2.133"
error_taxonomy_version: "1.78"
total_stories: 200
active_contracts: 232
draft_contracts: 5
retired_contracts: 6
prd_version: "1.12"
policies_version: "1.33"
subsystem_count: 22
vp_count: 157
bc_count_corrected: 250
workspace_test_count: 4273
vsdd_factory_version: "1.0.0-rc.18"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-1133 — demo-scope durability burst. DEMO-SCOPE.md created (.factory/objectives/DEMO-SCOPE.md). Wired into SESSION-HANDOFF §ACTIVE OBJECTIVE + resume protocol step 2 + STATE frontmatter demo_scope_doc + task ledger header. Streak 0/3 UNCHANGED. PR-LEVEL pass 25 NEXT at 15bedc12. STATE v7.782."
wave5_phase_b_status: "COMPLETE — Lanes 1/2/3/4 + S-MAINT all merged. Lane 1: S-SPEC-HTTP-METHOD-VALIDATION-001 PR#172. Lane 2: S-DEMO-QUERY-PUSHDOWN-001 PR#173. Lane 3: OCSF-CLASS-MIGRATION-001 PR#174. Lane 4: S-DEMO-003 PR#176. S-MAINT-ECRED-TAXONOMY-SYNC-001 PR#175."
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── T5 CASCADE STATE (CURRENT) ──
t5_pr: "PR #185 (S-DEMO-DTU-LIVE-SCENARIO-001-B) OPEN"
t5_streak: "0/3"
t5_pass_next: 25
t5_bc_019: "BC-2.06.019 v1.7"
t5_bc_020: "BC-2.06.020 v1.6"
t5_story_b: "v2.16"
t5_pivot_003: "v1.8"
t5_adr_036: "v2.3"

# ── PARKED WORKTREES ──
worktree_status: "stale: S-3.09 (FROZEN) + W3-FIX-S307-001 (BLOCKED superseded) — leave alone"

# ── POL-14 STATE (last merge) ──
pol14_last_merge: "Fix-PR cycle (PRs #182/#183/#184) — POL-14 idempotent: fix-PRs carry no behavioral_contracts frontmatter BCs; active_contracts 232 / draft_contracts 5 UNCHANGED (D-1103 check). Last BC promotion: BC-2.06.018 v1.5→v1.6 draft→active (D-1089, PR #181)."

# ── DTU + PIPELINE META ──
dtu_required: true
dtu_assessment: COMPLETE
dtu_assessment_approved: 2026-04-20
dtu_clones_built: in_progress
dtu_strategy: "Option 2 — DTU-first"
dtu_strategy_decided: 2026-04-20
active_objective: "multi-client SOC-analyst live demo (real per-client data; TDE deferred) — see SESSION-HANDOFF §ACTIVE OBJECTIVE"
task_ledger: ".factory/objectives/multi-client-soc-demo-tasks.md"
demo_scope_doc: ".factory/objectives/DEMO-SCOPE.md"
user_directive_persistent: "No pragmatic convergence. Fix all issues before build."
user_directive_remove_uncertainty: "Run dclaude:remove-uncertainty on every implementation story BOTH immediately after story-writer materializes/writes it AND again before TDD delivery (D-1110 extension 2026-06-12)."
policy_registry_source_of_truth: .factory/policies.yaml
sprint_state_path: ".factory/stories/sprint-state.yaml"
historical_cycles: [phase-1-convergence, wave-3-multi-tenant, wave-4-operations, wave-0-plugin-prereqs]
current_cycle: wave-5-e-demo-fidelity

# ── LOCKED ARCHITECTURAL DECISIONS ──
architectural_decisions_locked:
  - "1 LOCKED Option-A: TOML spec URLs ground against DTU clone routes (real-API canonical), NOT production Rust adapter URLs"
  - "2 LOCKED Option-B: Parity test loads reference OCSF from committed fixture JSON"
  - "3 LOCKED Option-A: Expand PLUGIN-MIGRATION-001-D scope to include SpecErrorCode::ESpec017 variant in prism-core + filename-stem validation"
  - "4 LOCKED Option-A: TOML auth_type declares REAL behavior (cyberint=cookie_roundtrip, claroty=bearer_static)"
  - "5 LOCKED Path-A (D-747): ADR-028 §D2 supersedes ADR-026 §D3 partial"

# ── COMPACTION RECORD ──
pre_compact_snapshot: "D-1132 2026-06-13 — zero-context resume hardening compaction. STATE v7.780→v7.781. Decisions D-1055..D-1123 archived to cycles/wave-5-e-demo-fidelity/decisions-archive-D1055-D1123.md. Session Resume Checkpoint updated to D-1132 (pass 25 next at 15bedc12). SESSION-HANDOFF hardened with full do-not-reflag list (40+ items) + T5 cascade ledger. Prior compaction: D-1056 2026-06-08 — Decisions D-700..D-1054 archived to decisions-archive-D700-D1054.md. Historical frontmatter keys archived to frontmatter-cascade-archive.md."
pre_compact_snapshot_at: "2026-06-13"
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
| **Last Updated** | 2026-06-13 (D-1133 — demo-scope durability burst: DEMO-SCOPE.md created, wired into resume protocol + STATE frontmatter + task ledger; streak 0/3 unchanged; STATE v7.781→v7.782) |
| **Current Phase** | Wave 5 (wave-5-e-demo-fidelity) — T5 PR-LEVEL cascade (PR #185, streak 0/3, pass 25 NEXT at 15bedc12). Phase B + Phase C COMPLETE. Review cycle COMPLETE. T4-A DONE (PR #181). |
| **Current Step** | T5 PR-LEVEL pass 25 at HEAD 15bedc12 (diff CHANGED — re-materialize via `gh pr diff 185`). Streak 0/3 (BPRL-P24-01 LOW prose-correction D-1131). STATE v7.781. |

## Active Objective (North Star)

**NORTH STAR: Deliver a MULTI-CLIENT SOC-ANALYST LIVE DEMO** — multiple DTU clients, each with a different sensor combination and genuinely different per-client data; prism federates into each client's DTUs; prism MCP wired into Claude (stdio); end-to-end SOC-analyst investigation workflow demonstrated. Scenario progresses deterministically over time (same seed + clock-offset → same timeline). Enrichment DTUs (ThreatIntel + NVD) serve scenario-correlated data.

**Full detail: SESSION-HANDOFF.md §ACTIVE OBJECTIVE. Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md §CURRENT POINTER` = T5 (PR-LEVEL cascade; PR #185 OPEN; streak 0/3; pass 25 NEXT at 15bedc12 — D-1131). T1+T2+T3+T4+T4-A DONE.**

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| 0: Codebase Ingestion | passed | 2026-04-13 | 2026-04-14 | human-approved | converged |
| 1a: Product Brief + Domain Spec | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1b: PRD + Behavioral Contracts | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1c: Architecture + VPs | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 1d: Adversarial Spec Review | passed | 2026-04-15 | 2026-04-15 | 33-pass convergence | 13→1 converged |
| 2: Story Decomposition | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 3: DTU Waves 0–2 | COMPLETE | 2026-04-21 | 2026-04-27 | wave gates converged | PRs #1–72; 1505 tests |
| 3: Wave 3 (3.A+3.B+3.C) | COMPLETE | 2026-04-27 | 2026-05-02 | 43 PRs merged | 2363 tests; develop@ba3b10c7 |
| 3: Wave 3 Tier-3 + FOLLOWUP | COMPLETE | 2026-05-06 | 2026-05-10 | PRs #127–#135 + #141 | S-3.01..S-3.07 + S-3.02-FOLLOWUP; develop@c6dd6602 |
| 3: PLUGIN-PREREQ-A through E | COMPLETE | 2026-05-10 | 2026-05-19 | PRs #143–#151 | PREREQ-A/B/C/D/E MERGED; develop@80ebe794 |
| 3: PLUGIN-MIGRATION-001-D | MERGED | 2026-05-21 | 2026-05-22 | PR #153 develop@3f2de889 | LOCAL 25 passes 3-CLEAN; PR-LEVEL CONVERGED |
| 3: PLUGIN-MIGRATION-001-E through G | MERGED | 2026-05-23 | 2026-05-27 | PRs #154–#160 | Wave 2 COMPLETE; develop@2dda655f |
| 3: S-SPEC-TYPE-UNIFICATION-001 + S-PLUGIN-CI-001 | MERGED | 2026-05-27 | 2026-05-27 | PRs #159–#161 | develop@af79f160; 3711 tests |
| 3: S-3.02-FOLLOWUP-RUNTIME + S-5.01-FOLLOWUP-MCP-BOOT | MERGED | 2026-05-27 | 2026-05-29 | PRs #162–#163 | develop@e898c3c9; 3718 tests |
| 3: S-DTU-CYBERINT-AUTH-FIDELITY-001 | MERGED | 2026-05-30 | 2026-05-31 | PR #164 develop@e798e67c | LOCAL 17 passes + PR-LEVEL 15 passes 3-CLEAN CONVERGED |
| 5: S-SPEC-ENV-VAR-001 | MERGED | 2026-05-31 | 2026-06-01 | PR #165 develop@4feec93a | LOCAL 5 + PR-LEVEL 5 passes 3-CLEAN CONVERGED |
| 5: S-DEMO-001 | MERGED | 2026-05-31 | 2026-06-01 | PR #166 develop@5dd3df02 | LOCAL 3/3 + PR-LEVEL 3/3 CONVERGED; BC-2.11.005 active |
| 5: S-DEMO-CLAROTY-AUDIT-DTU-001 | MERGED | 2026-06-01 | 2026-06-02 | PR #167 develop@eb3416d1 | LOCAL 3/3 + PR-LEVEL 3/3 CONVERGED (11 passes) |
| 5: S-DEMO-ARMIS-AQL-001 | MERGED | 2026-06-01 | 2026-06-02 | PR #168 develop@eb3416d1 | LOCAL 3/3 + PR-LEVEL 3/3 CONVERGED |
| 5: S-MAINT-W3SEC-CITE-SWEEP-001 | MERGED | 2026-06-02 | 2026-06-02 | PR #169 develop@b38c1abc | LOCAL 3/3 + PR-LEVEL 3/3 CONVERGED; DRIFT-D943-001 CLOSED |
| 5: S-DEMO-CROWDSTRIKE-MULTIREGION-001 | MERGED | 2026-06-03 | 2026-06-03 | PR #170 develop@cd4a2211 | LOCAL 3/3 + PR-LEVEL 3/3 (passes 7/8/9) CONVERGED |
| 5: S-DEMO-002 | MERGED | 2026-05-31 | 2026-06-04 | PR #171 develop@fdd12251 | LOCAL 3/3 + PR-LEVEL 3/3 (passes 12/13/14) CONVERGED; E2E smoke GREEN |
| 5: S-SPEC-HTTP-METHOD-VALIDATION-001 | MERGED | 2026-05-31 | 2026-06-05 | PR #172 develop@752e407a | PR-LEVEL 3/3 (passes 12/13/14); 41 CI GREEN; Phase B Lane 1 COMPLETE |
| 5: S-DEMO-QUERY-PUSHDOWN-001 | MERGED | 2026-06-05 | 2026-06-06 | PR #173 develop@9447671f | LOCAL 3/3 + PR-LEVEL 3/3 (passes 17/18/19); 41 CI GREEN; Phase B Lane 2 COMPLETE |
| 5: OCSF-CLASS-MIGRATION-001 | MERGED | 2026-06-06 | 2026-06-06 | PR #174 develop@0e89789a | LOCAL 3/3 (11 passes) + PR-LEVEL 3/3; 39 CI GREEN; Phase B Lane 3 COMPLETE |
| 5: S-MAINT-ECRED-TAXONOMY-SYNC-001 | MERGED | 2026-06-07 | 2026-06-07 | PR #175 develop@c603741d | LOCAL 3/3 + PR-LEVEL 3/3; ADR-035 v1.2 |
| **5: S-DEMO-003** | **MERGED** | 2026-06-07 | 2026-06-08 | PR #176 develop@a42e3eaf | LOCAL 3/3 (19 passes) + PR-LEVEL 3/3; Phase B Lane 4 COMPLETE |
| **5: Wave 5 Phase B** | **COMPLETE** | — | 2026-06-08 | All 4 lanes merged | develop@a42e3eaf |
| **5: Wave 5 Phase C** | **COMPLETE** | — | 2026-06-09 | PRs #177–#180 | develop@64d34967; all 4 lanes merged |
| **5: S-DEMO-DTU-LIVE-SCENARIO-001-A (T4-A)** | **MERGED** | 2026-06-09 | 2026-06-10 | PR #181 develop@c287b00d | LOCAL 18-pass 3-CLEAN strict + PR-LEVEL 3-pass 3-CLEAN strict; BC-2.06.018 v1.6 active |
| **Review cycle fix-PRs (#183/#184/#182)** | **MERGED** | 2026-06-10 | 2026-06-12 | PRs #183→#184→#182 | QRY #183 develop@f88b10e3; MCP #184 develop@c200d5a2; DTU #182 develop@939f36ce; all CI 43/43 GREEN |
| **5: S-DEMO-DTU-LIVE-SCENARIO-001-B (T5)** | **IN PROGRESS** | 2026-06-12 | — | PR #185 OPEN (streak 0/3; pass 25 NEXT at 15bedc12) | LOCAL 13-pass 3-CLEAN; PR-LEVEL passes 1–24 complete; BPRL-P24-01 closed D-1131 |

## Current Phase Steps

<!-- Keep last 5 rows only. Archive older rows to cycles/wave-5-e-demo-fidelity/burst-log.md. -->

| Step | Agent | Status | Output |
|------|-------|--------|--------|
_D-735 through D-1127 archived to cycles/wave-5-e-demo-fidelity/burst-log.md and decisions-archive-D1055-D1123.md._
| D-1129 | state-manager | 2026-06-13 | CONSISTENCY-SWEEP CLOSURE — DRIFT-1/2/3 closed. STORY-INDEX PIVOT-003 v1.5. Story B §Tasks/FSR/build_clone_pairs Cyberint 6-arg corrected. Lesson z16. |
| D-1130 | state-manager | 2026-06-13 | PASS-23 CLEAN(strict)=YES streak 1/3. DRIFT-2/3 re-derivation + VP-020-K case-sound + PC-8/PC-9 + E-DEMO-002 guard order + SAP-1. Novelty LOW. |
| D-1131 | state-manager | 2026-06-13 | BPRL-P24-01 LOW [process-gap] CLOSED — DTU perimeter `tests/external/perimeter-violation/` prose corrected to structural Cargo/E0432. BC-2.06.020 v1.6. Story B v2.16. Code 15bedc12. Streak RESET 0/3. Pass 25 NEXT. |
| D-1132 | state-manager | 2026-06-13 | ZERO-CONTEXT RESUME HARDENING (D-1132). STATE compacted (214KB→slim): decisions D-1055..D-1123 archived to decisions-archive-D1055-D1123.md. SESSION-HANDOFF hardened: full do-not-reflag list (40+ items) consolidated + T5 cascade ledger (LOCAL 1-13 + PR-LEVEL 1-24) + post-3/3 sequence verbatim. Task ledger T5 pointer updated. Lesson z18 appended. State v7.780→v7.781. Streak 0/3 UNCHANGED. |
| D-1133 | state-manager | 2026-06-13 | DEMO-SCOPE DURABILITY BURST (D-1133). DEMO-SCOPE.md created at `.factory/objectives/DEMO-SCOPE.md` — authoritative full demo scope (6-sensor fleet, unfolding-attack T5, enrichment correlation + honest in-prism-enrich gap, PIVOT chain, build sequence). Wired: SESSION-HANDOFF §ACTIVE OBJECTIVE pointer + resume protocol step 2; STATE frontmatter `demo_scope_doc`; task ledger scope-source header. Lesson z19 appended. STATE v7.781→v7.782. Streak 0/3 UNCHANGED. |

## Decisions Log

_D-001..D-046 archived: `cycles/phase-3-dtu-wave-2/decisions-archive-d001-d032.md`. D-047..D-174: `cycles/wave-3-multi-tenant/decisions-archive-d047-d114.md`. D-175..D-188: `cycles/wave-3-multi-tenant/burst-log.md`. D-200..D-213: `cycles/wave-4-operations/burst-log.md`. D-432..D-699: `cycles/wave-0-plugin-prereqs/burst-log.md` (D-727 compaction). **D-214..D-320 LOST** — TD-VSDD-058. **D-700..D-1054: `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md`** (D-1056 compaction). **D-1055..D-1123: `cycles/wave-5-e-demo-fidelity/decisions-archive-D1055-D1123.md`** (D-1132 compaction)._

| ID | Decision | Rationale | Phase | Date |
|----|----------|-----------|-------|------|
| D-1133 | state-manager | 2026-06-13 | **DEMO-SCOPE DURABILITY BURST (D-1133).** DEMO-SCOPE.md created at `.factory/objectives/DEMO-SCOPE.md` as the single source of truth for the full demo narrative scope (6-sensor fleet status, unfolding-attack T5 center-piece, enrichment correlation at data layer + honest gap: in-prism `enrich` pivot not yet wired, PIVOT-001/002/003 chain designed, build sequence T1→T14). Wired into: SESSION-HANDOFF.md §ACTIVE OBJECTIVE (top pointer) + FRESH-SESSION RESUME PROTOCOL step 2; STATE.md frontmatter `demo_scope_doc`; task ledger header (scope-source-of-truth reference). Lesson z19: demo/project SCOPE must live in ONE authoritative durable artifact referenced from resume protocol — task ledger lists TASKS but does not convey the demo NARRATIVE + gaps a zero-context restart needs. No spec/code/count change. Streak 0/3 UNCHANGED. STATE v7.781→v7.782. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1132 | state-manager | 2026-06-13 | **ZERO-CONTEXT RESUME HARDENING (D-1132).** STATE compacted (214KB→slim). Decisions D-1055..D-1123 archived to decisions-archive-D1055-D1123.md. SESSION-HANDOFF §RESUME SNAPSHOT hardened with (1) full consolidated do-not-reflag list (40+ items verbatim), (2) T5 cascade ledger (LOCAL passes 1-13 converged; PR-LEVEL passes 1-24 with outcome), (3) post-3/3 convergence sequence verbatim, (4) exact next action (pass 25 at 15bedc12). Task ledger CURRENT POINTER updated: T5 = PR #185 PR-LEVEL cascade streak 0/3 pass 25 NEXT at 15bedc12; D-1117 enhancement arc recorded; D-1107 capability-discovery block + PIVOT-001/002/003 chain recorded. Lesson z18 appended: proactively run zero-context-resume hardening burst at deep cascade depth (20+ passes, large do-not-reflag list, bloated STATE). Streak 0/3 UNCHANGED. STATE v7.780→v7.781. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1131 | state-manager | 2026-06-13 | **PASS-24 CLOSURE BURST (D-1131).** BPRL-P24-01 LOW [process-gap] closed — `tests/external/perimeter-violation/` false-coverage prose corrected across 4 surfaces to structural Cargo/E0432 framing. BC-2.06.020 v1.5→v1.6. Story B v2.15→v2.16. Implementer 15bedc12. PIVOT-003 v1.7→v1.8. BC-INDEX v6.42→v6.43. STORY-INDEX v2.368→v2.369. Lesson z17. Streak RESET 1/3→0/3. Pass 25 NEXT at 15bedc12 (diff changed). STATE v7.779→v7.780. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1130 | state-manager | 2026-06-13 | **PASS-23 CLEAN(strict)=YES streak 0/3→1/3.** Zero findings. 8-axis re-derivation: DRIFT-2/3 independent re-confirmation (Cyberint 6-arg); VP-020-K case-sound; PC-8 cyclic; PC-9 baseline CVE-9999-{:04}; E-DEMO-002 guard order; SAP-1 PASS; all BPRL-P1..P22 + DRIFT-1/2/3 re-confirmed. Novelty LOW. CODE UNCHANGED 0863184a. STATE v7.778→v7.779. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1129 | state-manager | 2026-06-13 | **CONSISTENCY-SWEEP CLOSURE BURST (D-1129).** DRIFT-1/2/3 closed: STORY-INDEX PIVOT-003 pin v1.3→v1.5; story B §Tasks/FSR/build_clone_pairs Cyberint 5-arg→6-arg. BC-INDEX v6.41→v6.42. Lesson z16. Streak 0/3 UNCHANGED. CODE UNCHANGED 0863184a. STATE v7.777→v7.778. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1128 | state-manager | 2026-06-13 | **PASS-22 CLOSURE BURST (D-1128).** BPRL-P22-01 MED SPEC-ONLY — BC-2.06.020 VP Anchors prose A..H/8→A..L/12. BC-2.06.020 v1.4→v1.5. Story B v2.13→v2.14. PIVOT-003 v1.6→v1.7. Orchestrator caught+reverted catalog-format regression during PO sweep (catalog=CVE-9999-{:05} / baseline=CVE-9999-{:04} are DISTINCT generators by design). BC-INDEX v6.40→v6.41. STORY-INDEX v2.366→v2.367. Lesson z15. Streak RESET 2/3→0/3. STATE v7.776→v7.777. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1127 | state-manager | 2026-06-13 | **PASS-21 CLEAN(strict)=YES streak 1/3→2/3.** Zero findings. 8-axis independent re-derivation all PASS: stage-timing TVs, 5×6 StageMask, E-DEMO-001..006, ADR-036 v2.3 constructor-sig all 6 clones, BC frontmatter crates:, AC-019 cyclic-catalog, SAP-1, all BPRL-P1..P20. Novelty LOW. CODE UNCHANGED 0863184a. STATE v7.775→v7.776. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1126 | state-manager | 2026-06-13 | **PASS-20 CLEAN(strict)=YES streak 0/3→1/3.** Zero findings. BPRL-P19-01 closure verified (both crate commands; all 4 VP-020 tests demonstrated). Core invariants re-confirmed. SAP-1 PASS. EXPECTED=52 PASS. CODE UNCHANGED 0863184a. STATE v7.774→v7.775. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1124 | state-manager | 2026-06-13 | **BPRL-P18-01 MED CLOSED (D-1124).** AC-019 demo-evidence artifacts: 3 fabricated/inverted BC anchors corrected. PC-8↔PC-9 labels corrected (PC-8=scenario catalog, PC-9=baseline namespace). INV-CYBERINT-CVE-PIVOT-001→INV-CYBERINT-ALERT-CVE-CORRELATION-001. CveCorrelationCatalog→ScenarioEntityCatalog. Demo-recorder commit 5d5484d0. Lesson z13. Streak RESET 2/3→0/3. STATE v7.772→v7.773. | wave-5-e-demo-fidelity | 2026-06-13 |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Phase-5 Deferred Findings (D-571 cycle-close)

| Finding ID | Description | Rationale | Phase-5 Scope | Recorded |
|------------|-------------|-----------|---------------|---------|
| F-LP12-OBS-001 | E-PLUGIN-008 dual-semantic reuse — BC-2.17.005 hot-reload vs BC-2.17.006 initial-load use same error code with different meanings | Genuine architectural adjudication gap | Phase-5 product-owner error namespace adjudication | D-571 2026-05-15 |
| F-LP25-OBS-001 | BC-2.17.002 v1.5 EC-17-007 vacuously true under Vec<String> contract | PO semantic choice required | Phase-5 product-owner BC-2.17.002 review agenda | D-571 2026-05-15 |

## Drift Items (S-7.02 Cycle-Close Checklist)

**Open Structural Questions:**

**OQ-001 (2026-05-22)** — BC-5.39.001 standalone-file elevation. Inline as CLAUDE.md §Operational Discipline TDs bullet. Non-blocking; track for next maintenance burst.

| ID | Summary | Required Action | Due |
|----|---------|-----------------|-----|
| DRIFT-OBS-LP87-003 | POL-29 growth-complexity asymptote concern | session-reviewer cycle-close assessment | v1.0.0-greenfield |
| DRIFT-OBS-LP73-001 | BC-2.01.005/006/007/008 stale DI-012 labels | PREREQ-F PO+architect sweep | PREREQ-F cycle |
| DRIFT-OBS-LP73-002 | POL-2 DI amendment missing sibling-CLASS sweep | POL-2 amendment step 5 | v1.0.0-greenfield |
| DRIFT-OBS-LP72-001/002 | POL-29 registry classes (d) title-anchor 3-way sync + (e) schema-integrity sibling-CLASS sweep | POL-29 or POL-26 amendment | v1.0.0-greenfield |
| DRIFT-OBS-LP71-001/002 | HS-007 STUB title drift + HS-001..012 abbreviated-title predating POL-7 | Architect adjudication | v1.0.0-greenfield |
| DRIFT-OBS-LP70-001 | POL-29 step 8a in-cell bookkeeping-marker scope gap | POL-29 amendment + content-cell grep | v1.0.0-greenfield |
| DRIFT-OBS-LP69-001/002 | POL-26 §Changelog 8 recurrences (lint_hook null) + mixed ADR ordering | hooks/check-changelog-monotonic.sh | v1.0.0-greenfield |
| DRIFT-OBS-LP68-001 | POL-29 step 3a (a) historical-citation pin exception clause needed | POL-29 extension | v1.0.0-greenfield |
| DRIFT-OBS-LP67-001 | POL-29 v1.16+ `lint_hook: null` — no tooling validator | hooks/validate-pol-29-variant-form-registry.sh | v1.0.0-greenfield |
| DRIFT-D850-001 **[RESOLVED D-1059]** | BC-2.16.002 missing explicit postcondition for POST-body vs GET-URL OffsetLimit pagination dispatch | RESOLVED: BC-2.16.002 v1.70 POST-vs-GET pagination clause authored | CLOSED |
| DRIFT-D849-001 | ADR-031 `related_bcs` field missing BC-2.01.017 | Architect to amend ADR-031 frontmatter | next architect dispatch |
| DRIFT-D849-002 | VP-TBD No-HTTP-Call invariant during StaticCookieAuthProvider::acquire_token | Architect to assign VP-NNN | next formal-verifier or architect dispatch |
| DRIFT-D916-001 [process-gap, codified] | POL-14 story-status transition gap: BCs promoted but story-status not auto-set to merged | S-POL-14-STATUS-SYNC-001 filed (D-918) | — |
| DRIFT-D904-001 (JUSTIFIED DEFERRAL) | OBS-PR1-001 adversary diff-tooling limitation | Track in drbothen/vsdd-factory upstream | upstream |
| DRIFT-D904-002 (JUSTIFIED DEFERRAL) | OBS-PR2 worktree-path-resolution hazard | Track in drbothen/vsdd-factory upstream | upstream |
| DRIFT-D923-001 [architect-scope] | ADR-022 §B step 8 description accuracy gap | Architect to amend ADR-022 §B step 8 narrative | next architect dispatch |
| DRIFT-D923-002 [architect/PO-scope] | validate_sensor_spec not called from production spec-load path | Architect/PO adjudicate | separate disposition |
| DRIFT-D1000-001..005 [process-gap, JUSTIFIED DEFERRAL] | Five vsdd-factory engine process improvements | vsdd-factory upstream | upstream |
| DRIFT-D954-001 [cross-crate, REGISTERED] | BC-3.5.002 precondition 3 mis-cite in prism-dtu-armis (~40+ sites) + prism-dtu-slack (1 site) | S-MAINT-W3SEC-CITE-SWEEP-002 anchors this | S-MAINT-W3SEC-CITE-SWEEP-002 |
| DRIFT-D943-001 **[CLOSED D-958]** | BC-3.5.002 mis-cite in prism-dtu-crowdstrike + prism-dtu-cyberint | RESOLVED: S-MAINT-W3SEC-CITE-SWEEP-001 MERGED PR #169 | CLOSED |
| DRIFT-D926-001 **[CLOSED D-1000]** | HTTP-method whitelist validation for env-resolved step.method field | RESOLVED: S-SPEC-HTTP-METHOD-VALIDATION-001 MERGED PR #172 | CLOSED |
| DRIFT-D1016-SEC-007 [hardening-candidate] | QueryParams.start_time/end_time typed as Option<String>; TimestampString newtype proposed | Architect/PO adjudicate | architect/PO |
| DEFER-ORPHAN-SENSORS-DIR-001 [legacy-cleanup] | Orphaned top-level sensors/*.toml hardcode us-1 | S-MAINT-ORPHAN-SENSORS-DIR-001 to be authored | — |
| DEFER-SS22-LABEL-DRIFT-001 | ARCH-INDEX "Process Lifecycle" vs BC-INDEX/story "Binary Entrypoint" for SS-22 | Maintenance story; architect adjudicates | — |
| DEFER-CLAUDEMD-BC216002-MISLABEL-001 | SAP-1 probe + CLAUDE.md §Conventions cite BC-2.16.002 as "Structured Event Catalog" (wrong) | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DEFER-CI-WORKFLOW-SPEC-DRIFT-001 | Spec↔CI-workflow drift class; no existing policy/lint for workflow attribute drift | Cycle-close: consistency-validator rule improvement | cycle-close |
| DEFER-EQUERY009-001 | BC-2.11.007 DI-021 E-QUERY-009 enforcement absent from live path | Phase-5: PO/architect adjudicate | phase-5 |
| DEFER-POL7-EDEMO-TEMPLATE-001 | POL-7 §References step unsatisfiable for E-DEMO story-template family | Cycle-close: PO reconciles | cycle-close |
| DRIFT-ECRED-TAXONOMY-001 **[RESOLVED D-1046]** | prism-core E-CRED variant semantics misaligned with error-taxonomy.md | RESOLVED: S-MAINT-ECRED-TAXONOMY-SYNC-001 MERGED PR #175 | CLOSED |
| DRIFT-EDITION-SYNC-001 [pre-existing, JUSTIFIED DEFERRAL] | prism-credentials/Cargo.toml edition=2021 vs workspace edition 2024 | S-MAINT-EDITION-SYNC-001 | S-MAINT-EDITION-SYNC-001 |
| DRIFT-SEC-TAPE-PATH-001 [security-LOW] | Hardcoded `/Users/<name>/...` absolute paths in `docs/demo-evidence/**/*.tape` files (CWE-200) | S-MAINT-TAPE-PATH-SWEEP-001 (maintenance wave) | maintenance wave |
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 [doc-staleness, HUMAN-ONLY] | CLAUDE.md §Conventions cites stale EXPECTED count | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DRIFT-SLUG-FORMAT-BC34004-001 [PO-harmonization] | BC-3.4.004+BC-3.5.001 carry arbitrary-string slug vs ADR-036 §2.2 canonical hex slug | PO reconciles BC-3.4.004/BC-3.5.001 test vectors | maintenance / Story-B-adjacent |
| DRIFT-RC1-PAGINATION-PARITY-001 [PO-harmonization] | BC-2.16.013 INV-HARNESS-ROUTE-PARITY does not explicitly define "route surface" | PO harmonization at next PO dispatch | next PO dispatch |
| DRIFT-ORCH-PRLEVEL-PUSH-001 **[APPLIED D-1065]** | PR-LEVEL fix-bursts MUST be pushed before re-gating | APPLIED — SESSION-HANDOFF §4 + rule #11; DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001 registered | DONE |
| DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001 [HUMAN-ONLY] | DRIFT-ORCH-PRLEVEL-PUSH-001 not yet mirrored into CLAUDE.md | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 [HUMAN-ONLY] | CLAUDE.md §Git Workflow states factory-artifacts local-only by default (outdated) | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DRIFT-PAGINATION-PAGESIZE-VALIDATION-001 [pre-existing] | `spec_parser` PaginationConfig lacks page_size>0 guard | PO/architect adjudicate | spec-engine validation story |

## Blocking Issues

| ID | Description | Blocker Owner | Since | Status |
|----|-------------|---------------|-------|--------|
| TD-VSDD-005 | vsdd-factory:adversary runtime tool-binding bug — only Read bound at dispatch; general-purpose-as-adversary workaround required | vsdd-factory plugin maintainer | 2026-04-26 | OPEN — housekeeping pause before Wave 3 |

## Historical Content

All historical cycle files:
- Burst history (wave-5): `cycles/wave-5-e-demo-fidelity/burst-log.md`
- Decisions archive (D-700..D-1054): `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md`
- **Decisions archive (D-1055..D-1123): `cycles/wave-5-e-demo-fidelity/decisions-archive-D1055-D1123.md`** (NEW — D-1132 compaction)
- Frontmatter cascade archive (per-story pass data): `cycles/wave-5-e-demo-fidelity/frontmatter-cascade-archive.md`
- Session handoff archive (superseded snapshots): `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`
- Convergence trajectory: `cycles/wave-5-e-demo-fidelity/convergence-trajectory.md`
- Lessons learned: `cycles/wave-5-e-demo-fidelity/lessons.md`
- Wave 0 cycle: `cycles/wave-0-plugin-prereqs/`
- Wave 3 cycle: `cycles/wave-3-multi-tenant/`
- Wave 4 cycle: `cycles/wave-4-operations/`

---

## Session Resume Checkpoint (2026-06-13 — D-1133: demo-scope durability; pass 25 NEXT at 15bedc12; STATE v7.782)

_Previous checkpoint (D-1132; STATE v7.781) superseded by D-1133 burst. SESSION-HANDOFF.md §ACTIVE OBJECTIVE updated with DEMO-SCOPE.md pointer._

**STATE v7.782. CURRENT POSITION: T5 (S-DEMO-DTU-LIVE-SCENARIO-001-B) — PR-LEVEL cascade in progress. PR #185 OPEN; HEAD=REMOTE=15bedc12. Pass 24 BPRL-P24-01 LOW [process-gap] CLOSED. Streak 0/3. NEXT: PR-LEVEL adversarial pass 25 at 15bedc12 (diff CHANGED — re-materialize via `gh pr diff 185`; do NOT reuse any cached diff from passes 20-24). develop HEAD: 939f36ce. Story B v2.16. BC-INDEX v6.43. STORY-INDEX v2.369 (200). VP-INDEX v1.79 (158). policies v1.33. error-taxonomy v1.78. demo_scope_doc: .factory/objectives/DEMO-SCOPE.md (NEW D-1133).**

**Full do-not-reflag list and cascade ledger: see SESSION-HANDOFF.md §4 and §3.**

**RESUME PROTOCOL (run on fresh session start):**
0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star) + §RESUME SNAPSHOT (latest D-1133). Read `.factory/objectives/DEMO-SCOPE.md` (authoritative full demo scope).
1. `vsdd-factory:factory-worktree-health` (BLOCKING — must pass before reading any state).
2. Verify develop HEAD: `git log --oneline origin/develop | head -1` → expect `939f36ce`.
3. Verify story B: `git -C .worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H'` → expect `15bedc12`.
4. Verify `gh pr checks 185` — confirm CI status on 15bedc12.
5. Verify `grep "^version:" .factory/STATE.md` shows `"7.782"`.
6. Parked worktrees: `.worktrees/S-3.09` (FROZEN) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded) — leave alone.
7. Apply lessons (a)–(z18) from `cycles/wave-5-e-demo-fidelity/lessons.md`.
8. **NEXT ACTION: PR-LEVEL pass 25** — fresh adversary on PR #185 at HEAD 15bedc12; diff CHANGED (re-materialize via `gh pr diff 185`; do NOT reuse /tmp/pr185-pass20.diff); streak 0/3; do-not-reflag list = all closures BPRL-P1..P24 + DRIFT-1/2/3 + D-1117 enhancements. See SESSION-HANDOFF.md §4 for full verbatim list.
