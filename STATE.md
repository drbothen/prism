---
document_type: pipeline-state
level: ops
version: "8.841"
producer: state-manager
timestamp: 2026-08-26T10:48:00Z
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
develop_head: "3f1e66179"
# NOTE: D-2299 — SINGLE-COMMIT BURST: POL-41 parallelize_against_inflight_state_writes REGISTERED (human-directed 2026-08-24). policies v1.42→v1.43. SESSION-HANDOFF.md Rule 15 + v8.006→v8.007. CLAUDE.md mirror deferred. STATE v8.831→v8.832.
bc_index_version: "9.66"
# NOTE: D-2308 — BC-INDEX v9.65→v9.66: BC-2.16.015 pin v1.5→v1.6 (LOCAL cascade round-4 fix-burst). draft/active/total UNCHANGED (11/253/277). D-2306 NOTE archived.
vp_index_version: "2.22"
# NOTE: D-2054 — VP-INDEX v2.21→v2.22: VP-157 and VP-158 promoted to active (v1.1); ADR-056 v0.5 and ADR-057 v0.4 rows added. D-2053 NOTE archived.
story_index_version: "2.902"
# NOTE: D-2308 — STORY-INDEX v2.901→v2.902: story v1.8→v1.9; S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001 stub registered; total_stories 314→315. D-2307 NOTE archived.
arch_index_version: "2.334"
# NOTE: D-2308 — ARCH-INDEX v2.333→v2.334: ADR-058 v2.33→v2.34 + ADR-028 v1.30→v1.31 anchor_stories (SAC-2; F-VULNS-R4C-DEF-001). D-2286 NOTE archived.
workspace_test_count: 5816
# NOTE: D-2288 — workspace_test_count 5765→5816: S-ADR058-OCSF-ROUTING-001 MERGED develop@3f1e66179 (PR #242); 5816 tests on develop (merged content; pre-merge just check confirmed GREEN). STATE v8.821→v8.822.
vsdd_factory_version: "1.0.0-rc.22"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-2308 LOCAL cascade round-4 fix-burst (TD-VSDD-053) — 3 parallel passes A/B/C on frozen @551d18196+BC v1.5/story v1.8: 1 LOW (F-VULNS-R4C-LOW-001/F-R4A-LOW-001 id-absent null→absent spec precision, corroborated 2 passes) + 3 OBS/deferred (F-VULNS-R4C-OBS-001 TOML comment; F-R4A-OBS-001 engine WARN-on-absent-optional deferred→S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001; F-VULNS-R4C-DEF-001 SAC-2 ADR anchor_stories) — ALL RESOLVED: BC-2.16.015 v1.5→v1.6 + story v1.8→v1.9 + ADR-058 v2.34 + ADR-028 v1.31 + code @5aae6f0b3 PUSHED origin; ARCH-INDEX v2.334; BC-INDEX v9.66; STORY-INDEX v2.902 (315). BC-5.39.001 LOCAL streak RESET 0/3; new frozen HEAD @5aae6f0b3; 3-CLEAN round-5 pending. trajectory-tail →7→5→3→4. STATE v8.840→v8.841."
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── PARKED WORKTREES ──
worktree_status: "Main worktree on develop origin/develop @3f1e66179. ACTIVE: .worktrees/S-CLAROTY-VULNS-001 (feature/S-CLAROTY-VULNS-001 @5aae6f0b3; Wave A G1 LOCAL 3-CLEAN confirmation PENDING on frozen @5aae6f0b3; feature PUSHED origin). PARKED (2): S-3.09 @43c41389d KEEP-PARKED; W3-FIX-S307-001 @fcab8717c PARKED-DIRTY do-NOT-touch. PENDING-TEARDOWN: .worktrees/S-ADR058-OCSF-ROUTING-001 (D-2288; PR #242 merged; remote feature branch deleted). Torn down (D-2262): S-ADR058-OCSF-COERCION-001 (PR #240); S-CLAROTY-AUDITLOG-TIMEBOX-001 (PR #239); maint-clippy-1.98 (PR #241)."

# ── DTU + PIPELINE META ──
dtu_required: true
dtu_assessment: COMPLETE
dtu_assessment_approved: 2026-04-20
dtu_clones_built: in_progress
dtu_strategy: "Option 2 — DTU-first"
dtu_strategy_decided: 2026-04-20
active_objective: "v1 FIRST RELEASE: fully-working Claroty xDome sensor, end-to-end (D-2264 GOVERNING DECISION 2026-08-21). Validation: REAL Claroty xDome tenant (live API; AD-017 opaque). v1 scope: client+sensor onboarding → OCSF correctness (COERCION+ROUTING) → all query shapes → push-down → SOC-analyst Q&A loop → stability. Release gate: live xDome validation after ROUTING-001 merges. POST-v1 de-scoped: S-OCSF-FIDELITY-CROWDSTRIKE/CYBERINT/ARMIS-001 + S-ADR058-DTU-PARITY-MIGRATION-001."
task_ledger: ".factory/objectives/multi-client-soc-demo-tasks.md"
demo_scope_doc: ".factory/objectives/DEMO-SCOPE.md"
api_specs_reference: ".factory/reference/api-specs/"
user_directive_persistent: "No pragmatic convergence. Fix all issues before build."
user_directive_remove_uncertainty: "Run dclaude:remove-uncertainty on every implementation story BOTH immediately after story-writer materializes/writes it AND again before TDD delivery (D-1110 extension 2026-06-12)."
policy_registry_source_of_truth: .factory/policies.yaml
sprint_state_path: ".factory/stories/sprint-state.yaml"
historical_cycles: [phase-1-convergence, wave-3-multi-tenant, wave-4-operations, wave-0-plugin-prereqs]
current_cycle: wave-5-e-demo-fidelity

# ── LOCKED ARCHITECTURAL DECISIONS ──
architectural_decisions_locked:
  - "1 LOCKED Option-A: TOML spec URLs ground against DTU clone routes (real-API canonical), NOT production Rust adapter URLs [SUPERSEDED by ADR-053 §D1 — grounding-order flip (OpenAPI→spec→DTU) EFFECTIVE]"
  - "2 LOCKED Option-B: Parity test loads reference OCSF from committed fixture JSON"
  - "3 LOCKED Option-A: Expand PLUGIN-MIGRATION-001-D scope to include SpecErrorCode::ESpec017 variant in prism-core + filename-stem validation"
  - "4 LOCKED Option-A: TOML auth_type declares REAL behavior [SUPERSEDED by ADR-053 §D3 — Cyberint dual-surface split EFFECTIVE]"
  - "5 LOCKED Path-A (D-747): ADR-028 §D2 supersedes ADR-026 §D3 partial [SUPERSEDED by ADR-053 §D2 — Armis token_exchange EFFECTIVE]"

# ── COMPACTION RECORD ──
pre_compact_snapshot: "See cycles/wave-5-e-demo-fidelity/: decisions-archive-D1789-D2199.md + decisions-archive-D2200-D2299.md + session-handoff-archive.md + drift-items-open.md. D-2305 compaction (2026-08-26): decisions D-2200..D-2299 (exhaustive) + Current Phase Steps D-2262..D-2299 (exhaustive) archived to cycle files; frontmatter NOTEs trimmed to 1 per field. D-2244+1 compaction (2026-08-19): convergence-trajectory.md CREATED (OCSF cascade passes 1-45). D-2237 compaction (2026-08-18): decisions D-1789..D-2199 (exhaustive) + D-2059..D-2159 steps archived. Last preserved decision inline: D-2300. Git history on factory-artifacts preserves all content."
pre_compact_snapshot_at: "2026-08-26"
---

<!-- STATE.md SIZE BUDGET: 193 lines (wc-l) | target 200 lines (soft) | hard-cap 500 | margin from soft-target: 7 | margin from actual: 307 | compact eligible: safe_to_compact: true -->

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Language** | Rust |
| **Mode** | brownfield |
| **Deploy** | per-analyst stdio (MCP) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-08-26 D-2308 LOCAL round-4 fix-burst — BC-2.16.015 v1.6/story v1.9; ADR-058 v2.34/ADR-028 v1.31 anchor_stories; engine-loglevel stub; code @5aae6f0b3 PUSHED; streak 0/3 frozen @5aae6f0b3; ARCH-INDEX v2.334/BC-INDEX v9.66/STORY-INDEX v2.902 (315). trajectory-tail →7→5→3→4. STATE v8.840→v8.841. |

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| 0: Codebase Ingestion | passed | 2026-04-13 | 2026-04-14 | human-approved | converged |
| 1a: Product Brief + Domain Spec | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1b: PRD + Behavioral Contracts | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1c: Architecture + VPs | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 1d: Adversarial Spec Review | passed | 2026-04-15 | 2026-04-15 | 33-pass convergence | 13→1 converged |
| 2: Story Decomposition | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 3: Waves 0-3 + Plugin Prereqs | COMPLETE | 2026-04-21 | 2026-05-27 | wave gates converged | PRs #1-161; 3711 tests; develop@af79f160 |
| 3: Post-Wave-3 DTU+Demo+PRs #162-241 | COMPLETE | 2026-05-27 | 2026-08-20 | all MERGED develop@362e4f85 | PR #241 squash-merged 2026-08-20; 5765 tests; workspace CI green |
| Wave-A spec-evolution LOCAL CASCADE | CONVERGED | 2026-07-23 | 2026-07-23 | BC-5.39.001 strict 3/3 | 47 passes / 36 fix-bursts. CLEAN(strict): 19/24/30/33/36/39/41/42/45/46/47. |
| DEFECT-ADAPTER-TLS-XDOME-LIVE-001 | FULLY VALIDATED | 2026-08-15 | 2026-08-15 | D-2166 AC-LIVE-001 SATISFIED; HS-008..011 CONSUMED | PR #237 squash-merged develop@3197e27a9 2026-08-15 |
| S-CLAROTY-AUDITLOG-TIMEBOX-001 | MERGED | 2026-08-16 | 2026-08-16 | PR #239 develop@69d821be 2026-08-16T22:51Z | LOCAL 9-pass 3-CLEAN + HOLDOUT PASS 4/4 + LIVE xDome PASS; PR-LEVEL 3-CLEAN on 8ae0b5d8 |
| OCSF-correctness claroty SPEC adversary cascade | CLOSED (substantive) | 2026-08-16 | 2026-08-19 | human decision 2026-08-19 | FINAL FROZEN: ADR-058 v2.24/BC-2.16.002 v2.29/BC-2.16.003 v1.19/ROUTING-001 v1.44/COERCION-001 v1.40 |
| D-2238..D-2243 (exhaustive) SPEC fix bursts | COMPLETE | 2026-08-18 | 2026-08-18 | state-manager | F-P33..P45 fix bursts: ADR-058 v2.17→v2.21; BC-2.16.003 v1.13→v1.15; ROUTING-001 v1.31→v1.37; COERCION-001 v1.30→v1.34. |
| D-2245..D-2252 (exhaustive) FB-46..FB-69 SPEC fix-bursts | COMPLETE | 2026-08-18 | 2026-08-19 | state-manager | Multiple spec fix-bursts: ADR-058 v2.21→v2.24; BC-2.16.003 v1.15→v1.19; ROUTING-001 v1.37→v1.44; COERCION-001 v1.34→v1.40. FINAL FROZEN D-2251. |
| S-ADR058-OCSF-COERCION-001 TDD + PR cycle | MERGED | 2026-08-20 | 2026-08-20 | PR #240 develop@362e4f85 2026-08-20 (human-authorized admin-merge) | LOCAL cascade CONVERGED (human admin override D-2259); HOLDOUT PASS 4/4 (HS-001..HS-004 real MCP stdio); demo COMPLETE; just check 5765 GREEN; FINAL FROZEN ADR-058 v2.26/BC-2.16.002 v2.32/BC-2.16.003 v1.21; active_contracts 252→253 |
| S-ADR058-OCSF-ROUTING-001 LOCAL+HOLDOUT+DEMO + PR-LEVEL FIX-BURSTS | MERGED | 2026-08-23 | 2026-08-23 | PR #242 SQUASH-MERGED to develop@3f1e66179 (D-2288) | LOCAL 3-CLEAN D-2283 @8aeaf06c4/fc0776dad; HOLDOUT PASS D-2285 HS-023 3/3 P0; DEMO @dc37a57a7 (21/21 ACs); FIX-BURST-1 D-2286 @2393470cd; FIX-BURST-2 D-2287 @5645c8506; PR-LEVEL 3-CLEAN CONVERGED; MERGED D-2288. |

_Historical Phase Progress rows archived to cycles/wave-5-e-demo-fidelity/burst-log.md (D-1794 + D-2237 + D-2244+1 + D-2261 compactions)._

## Convergence Status

| Metric | Value |
|--------|-------|
| BC-5.39.001 streak | **S-CLAROTY-VULNS-001 LOCAL 0/3** — pass-1 (2 CRIT, 2 HIGH, 5 MED, 2 OBS ALL FIXED @e2b779800+@3874f8624) + pass-2 (1 MED, 3 LOW, 1 OBS ALL FIXED @62f1c6379) + pass-3 (1 MED F-VULNS-ANCHOR-001 + 4 LOW + 1 OBS ALL FIXED @c8f21c4d2) + pass-4 (2 MED F-VULNS-ADV-001+F-VULNS-ADV-002 + 1 LOW F-VULNS-ADV-003 + 2 OBS ALL FIXED @cebeba3d6) + pass-5 (1 HIGH F-VULNS-P5-001 + 1 MED F-VULNS-P5-002 + 1 LOW F-VULNS-P5-003 + 2 OBS ALL FIXED @8f4c25c87) + diverse-lens batch (lens-1 CLEAN; lens-4 CLEAN; lens-2 F-L2-001/002/003 + lens-3 F-L3-003/004/005 ALL FIXED @4e525126b+story v1.6; 2 HIGH race-FP discarded) + round-2 fix-burst D-2306 (3 passes A/B/C; 2 MED+1 LOW+2 OBS ALL FIXED @fa35b09aa) + round-3 fix-burst D-2307 (3 passes A/B/C; 1 MED+1 LOW+1 OBS ALL FIXED @551d18196; story v1.8). Feature HEAD @551d18196 frozen for round-4 confirmation. |
| Active cascade | S-CLAROTY-VULNS-001 Wave A G1 LOCAL adversary cascade. Feature HEAD @551d18196 (PUSHED origin). BC-5.39.001 3-CLEAN confirmation round-4 PENDING. |
| Pass count | VULNS-001 LOCAL: 5 serial passes + 4-lens diverse-lens batch (D-2304; lens-1 CLEAN; lens-2 1M+2O; lens-3 2M+1L+1O; lens-4 CLEAN; 2H FP discarded; fix @4e525126b+story v1.6) + round-2 fix-burst D-2306 (3 passes A/B/C on frozen @4e525126b; 2 MED+1 LOW+2 OBS ALL FIXED @fa35b09aa) + round-3 fix-burst D-2307 (3 passes A/B/C on frozen @fa35b09aa; 1 MED+1 LOW+1 OBS ALL FIXED @551d18196). Full history: cycles/wave-5-e-demo-fidelity/convergence-trajectory.md |
| Last CLEAN(strict) | ROUTING-001 PR-LEVEL 3/3 CLEAN(strict)=YES CONVERGED (D-2288; MERGED develop@3f1e66179 2026-08-23). VULNS-001: no CLEAN pass yet (streak 0/3). |
| Finding trajectory | VULNS-001 LOCAL →(pass-1: 2C+2H+5M+2O)→(pass-2: 1M+3L+1O)→(pass-3: 1M+4L+1O)→(pass-4: 2M+1L+2O)→(pass-5: 1H+1M+1L+2O)→(diverse-lens: 4 lenses; real: 1M+2O+2M+1L+1O; 2H FP discarded; ALL FIXED @4e525126b)→(round-2: 3 passes A/B/C; 2M+1L+2O ALL FIXED @fa35b09aa)→(round-3: 3 passes A/B/C; 1M+1L+1O ALL FIXED @551d18196). trajectory-tail →5→7→5→3. 3-CLEAN confirmation round-4 PENDING. Full history: cycles/wave-5-e-demo-fidelity/convergence-trajectory.md |
| Frozen perimeter | ADR-058 v2.33 / BC-2.16.002 v2.35 / BC-2.16.003 v1.27 (active) / BC-2.11.016 v1.31 / error-taxonomy v2.82 / ROUTING-001 v1.57 (merged) / COERCION-001 v1.47 (merged) / BC-2.16.015 v1.5 (draft, VULNS-001 round-3 fixed) / story v1.8 — code @3f1e66179 (develop) / feature @551d18196 (PUSHED; VULNS-001 frozen for round-4) / ARCH-INDEX v2.333 / BC-INDEX v9.65 / STORY-INDEX v2.901 / HOLDOUT-INDEX v1.26 |

## Concurrent Cycles

_No concurrent cycles in progress. Current cycle: wave-5-e-demo-fidelity._

## Current Phase Steps

| Step | Agent | Date | Content |
|------|-------|------|---------|
| _D-735..D-2299 (exhaustive)_ | — | archived | cycles/wave-5-e-demo-fidelity/burst-log.md (compactions) |
| D-2304 | state-manager | 2026-08-25 | diverse-lens batch: 4 lenses; lens-1+lens-4 CLEAN; lens-2+lens-3 real 1M+2O+2M+1L+1O ALL FIXED @4e525126b+story v1.6; trajectory-tail →6→5→5→7 |
| D-2305 | state-manager | 2026-08-26 | compact-state burst: D-2200..D-2299 (exhaustive) + D-2262..D-2299 (exhaustive) CPS archived to cycle files; STATE v8.837→v8.838 |
| D-2306 | state-manager | 2026-08-26 | round-2 fix-burst: 2M+1L+2O ALL FIXED @fa35b09aa; story v1.6→v1.7; STORY-INDEX v2.900; trajectory-tail →5→5→7→5 |
| D-2307 | state-manager | 2026-08-26 | round-3 fix-burst: 1M+1L+1O ALL FIXED @551d18196; story v1.7→v1.8; STORY-INDEX v2.901; trajectory-tail →5→7→5→3 |

## Decisions Log

_D-1789..D-2199 (exhaustive) archived to `cycles/wave-5-e-demo-fidelity/decisions-archive-D1789-D2199.md` (D-2237 compaction). Prior archives: D-700..D-1788 (exhaustive) in earlier archive files._
_Rows D-2200..D-2299 (exhaustive) archived → cycles/wave-5-e-demo-fidelity/decisions-archive-D2200-D2299.md (D-2305 compaction 2026-08-26). Prior range D-001..D-1788 (exhaustive) in earlier cycle archives._

| ID | Author | Date | Decision | Cycle | Updated |
|----|--------|------|----------|-------|---------|
| D-2300 | state-manager | 2026-08-25 | **SINGLE-COMMIT BURST (TD-VSDD-053) — S-CLAROTY-VULNS-001 Wave A G1 LOCAL cascade pass-1/pass-2 fix-bursts RECORDED. pass-1: 2 CRIT, 2 HIGH, 5 MED, 2 OBS — ALL FIXED @e2b779800 + @3874f8624. pass-2: 1 MED, 3 LOW, 1 OBS — ALL FIXED @62f1c6379 + BC-2.16.015 v1.0→v1.1 (F-VULNS-P1-004: §4 SAP-2 DTU-parity mandate annotated D-2200/D-2264 deferral + anchor S-ADR058-DTU-PARITY-MIGRATION-001; TD-VSDD-097 Dim-3 DISCHARGED) + story v1.1→v1.2 (F-VULNS-P1-003: RG-003 split RG-003a/003b, RG-004b mock wire-shape added, density 10/8=1.25, SAC-1 restored). F-VULNS-011: crates_touched synced [prism-sensors, prism-spec-engine]→[prism-sensors, prism-bin] (feature diff @62f1c6379 zero prism-spec-engine files). Feature HEAD 62f1c6379 FROZEN for pass-3 re-cascade. just check GREEN: prism-sensors 199 / prism-bin 229 / prism-spec-engine 798. BC-5.39.001 LOCAL streak 0/3 (re-cascade pending). BC-INDEX v9.60→v9.61. STORY-INDEX v2.894→v2.895. TD-VSDD-097: Dim-1 CLEAR (no sibling pairs affected); Dim-2 CLEAR (BC-2.16.015 §4 is not a copy-source section); Dim-3 DISCHARGED (§4 MUST anchored to story S-ADR058-DTU-PARITY-MIGRATION-001). records-lint exit 0. STATE v8.832→v8.833.** | wave-5-e-demo-fidelity | 2026-08-25 |
| D-2301 | state-manager | 2026-08-25 | **SINGLE-COMMIT BURST (TD-VSDD-053) — S-CLAROTY-VULNS-001 Wave A G1 LOCAL cascade pass-3 fix-burst RECORDED. 1 MED (F-VULNS-ANCHOR-001: §Architecture Anchors spec_driven_adapter.rs crate prism-spec-engine→prism-bin) + 4 LOW (F-VER-001 BC-version pins, F-DOC-001 test docstring, F-EC004-001 advisory-title test, F-AC003-001 e2e class_uid/_sensor) + 1 OBS (F-EC009-001 id E-QUERY-038) — ALL FIXED: test-writer @c8f21c4d2 + product-owner BC-2.16.015 v1.1→v1.2 + story-writer story v1.2→v1.3. Feature code HEAD c8f21c4d2; just check GREEN prism-sensors 200 / prism-bin 230 / prism-spec-engine 798. BC-5.39.001 LOCAL streak 0/3 (fix-burst advanced frozen HEAD to c8f21c4d2; pass-4 re-cascade pending). BC-INDEX v9.61→v9.62; STORY-INDEX v2.895→v2.896. TD-VSDD-097: Dim-1 CLEAR (§Architecture Anchors in BC-2.16.015 is not a sibling-pair section); Dim-2 CLEAR (§Architecture Anchors is not a copy-source section); Dim-3 CLEAR (no new MUSTs introduced). records-lint exit 0. trajectory-tail →0→11→5→6. STATE v8.833→v8.834.** | wave-5-e-demo-fidelity | 2026-08-25 |
| D-2302 | state-manager | 2026-08-25 | **SINGLE-COMMIT BURST (TD-VSDD-053) — S-CLAROTY-VULNS-001 Wave A G1 LOCAL cascade pass-4 fix-burst RECORDED. 2 MED [F-VULNS-ADV-001 REQUIRED-semantics misattribution BC §Invariants + story AC-006/EC-001; F-VULNS-ADV-002 EC-007 uncovered E-SPEC-018 arm] + 1 LOW F-VULNS-ADV-003 [wire-test self-mirroring] + 2 OBS [RG-006 misnomer, EC-008] — ALL FIXED: test-writer @cebeba3d6 (EC-007 + EC-008 tests, wire-test prod-config gate, RG-006 note) + product-owner BC v1.2→v1.3 + story-writer story v1.3→v1.4. GOVERNING DECISION (human-approved Option A, 2026-08-25): E-SPEC-018 on a PRESENT unparseable datetime HARD-ERRORS (structured TimestampParseFailure) — canonical/professional-grade engine behavior (cross-sensor, live-validated); the spec was inaccurate (claimed demote-to-null) and was corrected to match. Resilient demote-to-null alternative (workspace-wide engine change) NOT adopted; potential future story only (not created speculatively). Feature code HEAD cebeba3d6; just check GREEN prism-sensors 201 / prism-bin 232 / prism-spec-engine 798. BC-5.39.001 LOCAL streak 0/3 (fix-burst advanced frozen HEAD to cebeba3d6; pass-5 re-cascade pending). BC-INDEX v9.62→v9.63; STORY-INDEX v2.896→v2.897. TD-VSDD-097 all dims CLEAR. records-lint exit 0. STATE v8.834→v8.835.** | wave-5-e-demo-fidelity | 2026-08-25 |
| D-2303 | state-manager | 2026-08-25 | **SINGLE-COMMIT BURST (TD-VSDD-053) — S-CLAROTY-VULNS-001 Wave A G1 LOCAL cascade pass-5 fix-burst RECORDED. 1 HIGH F-VULNS-P5-001 [doubled queryable name claroty_claroty_vulnerabilities; table_name claroty_vulnerabilities→vulnerabilities; registers as claroty_vulnerabilities per sibling convention; code @d37dcd97a + BC v1.4 + story v1.5] + 1 MED F-VULNS-P5-002 [atomic-fail: normalize post-accumulation → whole-result Err; §Error-Cases + EC-007/008 corrected; Option-A fail-fast] + 1 LOW F-VULNS-P5-003 [EC-008 body-excerpt assertion] + 2 OBS [F-P5-004 EC-006 null-datetime test, F-P5-005 stale version header] — ALL FIXED: implementer @d37dcd97a + test-writer @8f4c25c87 + product-owner BC v1.3→v1.4 + story-writer story v1.4→v1.5. Feature code HEAD 8f4c25c87; just check GREEN prism-sensors 202 / prism-bin 232 / prism-spec-engine 798. BC-5.39.001 LOCAL streak 0/3 (fix-burst advanced frozen HEAD to 8f4c25c87; pass-6 re-cascade pending). BC-INDEX v9.63→v9.64; STORY-INDEX v2.897→v2.898. TD-VSDD-097 all dims CLEAR. records-lint exit 0. STATE v8.835→v8.836.** | wave-5-e-demo-fidelity | 2026-08-25 |
| D-2304 | state-manager | 2026-08-25 | **SINGLE-COMMIT BURST (TD-VSDD-053) — S-CLAROTY-VULNS-001 DIVERSE-LENS ADVERSARY BATCH (4 parallel fresh-context lenses on frozen crates @8f4c25c87 + specs BC v1.4/story v1.5): lens-1 (correctness/spec-vs-code) CLEAN(strict); lens-4 (wire-shape/prod-grade) CLEAN(strict); lens-2 (test-coverage/SAP-3) 1 MED + 2 OBS fixed; lens-3 (naming/consistency) 2 MED + 1 LOW + 1 OBS fixed; 2 HIGH index findings DISCARDED (race false-positives; lens-3 read .factory/ mid-D-2303-commit; post-commit rows verified correct). ALL real findings FIXED: test-writer @4e525126b (+4 tests; prism-sensors 203/prism-bin 234/prism-spec-engine 798 GREEN, just check EXIT 0) + story-writer story v1.5→v1.6 (F-L3-003/004). Feature crates HEAD @4e525126b PUSHED origin. STORY-INDEX v2.898→v2.899. BC-5.39.001 LOCAL streak 0/3. SESSION WRAP: SESSION-HANDOFF.md D-2304 snapshot. TD-VSDD-097 all dims CLEAR. records-lint exit 0. trajectory-tail →6→5→5→7. STATE v8.836→v8.837.** | wave-5-e-demo-fidelity | 2026-08-25 |
| D-2305 | state-manager | 2026-08-26 | **compact-state burst: extracted D-2200..D-2299 decisions (exhaustive) + D-2262..D-2299 current-phase-steps (exhaustive) to cycles/wave-5-e-demo-fidelity/ archives; frontmatter NOTEs trimmed to 1 per field; STATE.md slimmed to <200 lines; canonical values preserved. trajectory-tail →6→5→5→7. STATE v8.837→v8.838.** | wave-5-e-demo-fidelity | 2026-08-26 |
| D-2306 | state-manager | 2026-08-26 | **LOCAL cascade round-2 fix-burst (TD-VSDD-053) — 3 parallel fresh-context passes A/B/C on frozen @4e525126b + BC v1.4/story v1.6: 2 MED (F-VULNS-PB-001/PA-L01 `id` `$.id` positive-extraction false-green, corroborated by 2 passes → RG-009 added; F-VULNS-PC-MED-001 BC §Description "18 Tier-2"→"17") + 1 LOW (F-VULNS-PC-LOW-002 test docstrings phantom EC-016-015-007/EC-009 → real §Error Cases E-SPEC-018 / E-QUERY-001 + story §Edge Cases anchors) + 2 OBS (F-VULNS-PA-O01 EC-016-015-006 null-datetime raw_extensions clarification; F-VULNS-PA-O02 §1 body_template TOML→text fence) — ALL FIXED: test-writer @fa35b09aa (RG-009 + 2 strengthened wire assertions + docstring anchors; just check GREEN 5838/5838) + product-owner BC-2.16.015 v1.4→v1.5 + story-writer story v1.6→v1.7 (RG-009 density 11/8=1.375, BC-pin propagation 7 sites, §Authority body_template/count sweep). 2 prior-session HIGH index findings RE-CONFIRMED race false-positives (pass C verified index rows twice on-disk). Feature HEAD @fa35b09aa PUSHED origin (fast-forward 4e525126b..fa35b09aa; pre-push just check GREEN). BC-5.39.001 LOCAL streak RESET 0/3; new frozen HEAD @fa35b09aa; 3-CLEAN confirmation round-3 pending. BC-INDEX v9.64→v9.65; STORY-INDEX v2.899→v2.900. TD-VSDD-097: Dim-a CLEAR; Dim-b DISCHARGED (story §Authority sync); Dim-c CLEAR. records-lint exit 0. trajectory-tail →5→7→5→(round-2). STATE v8.838→v8.839.** | wave-5-e-demo-fidelity | 2026-08-26 |
| D-2307 | state-manager | 2026-08-26 | **LOCAL cascade round-3 fix-burst (TD-VSDD-053) — 3 parallel passes A/B/C on frozen @fa35b09aa + BC v1.5/story v1.7: 1 MED (F-VULNS-PC3-MED-001 RG-009 enumeration propagation gap, corroborated 2 passes) + 1 LOW (wire-test §Tests-in-this-file table 3→8) + 1 OBS (RG-004 tautological class_uid arm annotated non-load-bearing) — ALL FIXED: story-writer story v1.7→v1.8 (RG-009 swept into 6 enumeration sites: §File Structure, crates_touched, Tasks 3/7/10) + test-writer @551d18196 (docstring density 10→11/1.25→1.375, wire-test table completed, class_uid annotated cites RG-004b; just check GREEN prism-sensors 14/14 + prism-bin 8/8). BC-2.16.015 UNCHANGED v1.5. Feature HEAD @551d18196 PUSHED origin. BC-5.39.001 LOCAL streak RESET 0/3; new frozen HEAD @551d18196; 3-CLEAN round-4 pending. STORY-INDEX v2.900→v2.901. TD-VSDD-097 downstream-copy dimension DISCHARGED (exhaustive RG-enumeration sweep). records-lint exit 0. trajectory-tail →5→7→5→3. STATE v8.839→v8.840.** | wave-5-e-demo-fidelity | 2026-08-26 |
| D-2308 | state-manager | 2026-08-26 | **LOCAL cascade round-4 fix-burst — 1 LOW (F-VULNS-R4C-LOW-001/F-R4A-LOW-001 id-absent "null"→"absent" precision, 2-pass corroborated) + 3 OBS/deferred (F-VULNS-R4C-OBS-001 TOML comment header→vulnerabilities; F-R4A-OBS-001 engine WARN-on-absent-key ruled to new stub S-ENGINE-SOURCE-PATH-ABSENT-KEY-LOGLEVEL-001; F-VULNS-R4C-DEF-001 SAC-2 ADR anchor_stories) ALL RESOLVED: BC-2.16.015 v1.5→v1.6 + story v1.8→v1.9 + ADR-058 v2.34/ADR-028 v1.31 anchor_stories + code @5aae6f0b3 (TOML comment + RG-008 test rename id_null_when_absent→id_absent_when_missing). Feature PUSHED origin. BC-5.39.001 LOCAL streak 0/3; frozen @5aae6f0b3; 3-CLEAN round-5 pending. ARCH-INDEX v2.334/BC-INDEX v9.66/STORY-INDEX v2.902 (total 315). TD-VSDD-097 all dims CLEAR. records-lint exit 0. STATE v8.840→v8.841.** | wave-5-e-demo-fidelity | 2026-08-26 |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Blocking Issues

| Issue | Owner | Opened | Resolved | Notes |
|-------|-------|--------|----------|-------|
| ADR-058-SPEC-READINESS-FAIL-001 [blocking-next-action; D-2176/observed 2026-08-15; severity: BLOCKING; status: CLOSED] | architect + product-owner + story-writer | 2026-08-15 | 2026-08-20 | CLOSED — COERCION-001 MERGED (PR #240 @362e4f85 2026-08-20). |
| PROCESS-GAP [D-2092/observed 2026-08-02; status: OPEN] | Orchestrator | 2026-08-02 | — | version-field-sync ambiguity in dispatch brief; process improvement |
| PROCESS-GAP [D-2091/observed 2026-08-02; anchor: S-MAINT-BURST-COMMIT-COUNT-GATE-001; status: MITIGATED] | Orchestrator | 2026-08-02 | — | S-MAINT-BURST-COMMIT-COUNT-GATE-001 ARCH-QUES-001 pending |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md, convergence-trajectory.md, decisions-archive-D1789-D2199.md, decisions-archive-D2200-D2299.md, session-handoff-archive.md, lessons.md, session-checkpoints.md. Prior cycles: wave-0-plugin-prereqs/, wave-3-multi-tenant/, wave-4-operations/.

## Session Resume Checkpoint (D-2303 -- 2026-08-25 -- VULNS-001 Wave A G1 LOCAL cascade pass-5 fixed @8f4c25c87; pass-6 re-cascade pending; STATE v8.835→v8.836) [supersedes D-2302]

### RESUME IN ONE BREATH
Prism Phase-3, v1 = live Claroty-xDome. S-CLAROTY-VULNS-001 Wave A G1 TDD GREEN. LOCAL adversary cascade: pass-1 (2 CRIT, 2 HIGH, 5 MED, 2 OBS ALL FIXED @e2b779800+@3874f8624) + pass-2 (1 MED, 3 LOW, 1 OBS ALL FIXED @62f1c6379) + pass-3 (1 MED F-VULNS-ANCHOR-001 + 4 LOW + 1 OBS ALL FIXED @c8f21c4d2) + pass-4 (2 MED F-VULNS-ADV-001+F-VULNS-ADV-002 + 1 LOW F-VULNS-ADV-003 + 2 OBS ALL FIXED @cebeba3d6) + pass-5 (1 HIGH F-VULNS-P5-001 + 1 MED F-VULNS-P5-002 + 1 LOW F-VULNS-P5-003 + 2 OBS ALL FIXED @8f4c25c87). Feature HEAD frozen @8f4c25c87 for pass-6 re-cascade. BC-5.39.001 LOCAL streak 0/3. BC-INDEX v9.64 (active 253/draft 11/total 277). STORY-INDEX v2.898 (314 stories). develop_head 3f1e66179; workspace_test_count 5816 (develop); just check GREEN feature @8f4c25c87 (prism-sensors 202/prism-bin 232/prism-spec-engine 798).

**RESUME NEXT-ACTION:** Dispatch LOCAL adversary pass-6 on frozen feature HEAD @8f4c25c87. BC-5.39.001 requires 3 consecutive CLEAN(strict) passes to converge. Pass-6 is pass 1-of-3 toward convergence after the pass-5 fix burst. Feature branch NOT pushed to origin (push after LOCAL 3-CLEAN then HOLDOUT then DEMO).

**CRITICAL CONVENTIONS:** xDome baseline = 4 tables (devices/alerts/audit_logs/device_alert_relations). OpenAPI is 4.4 MB — NEVER full-read. ColumnOptions::Required is push-down-eligibility option (pushdown.rs) NOT extraction null/error gate. No AI attribution in commits. prism-spec-engine has ZERO modified files in this story. E-SPEC-018 on a PRESENT unparseable datetime HARD-ERRORS (structured TimestampParseFailure) — human-approved Option A 2026-08-25. NAMING: table_name in TOML is `vulnerabilities` (F-VULNS-P5-001 fix); queryable name registers as `claroty_vulnerabilities` (sensor_id prefix prepended by SS-16 registry). Code uses vulnerabilities throughout; MCP query surface uses claroty_vulnerabilities.

**SPEC PERIMETER (develop@3f1e66179 + feature@8f4c25c87):** ADR-058 v2.33 / BC-2.16.002 v2.35 / BC-2.16.003 v1.27 (active) / BC-2.11.016 v1.31 / error-taxonomy v2.82 / ROUTING-001 v1.57 (merged) / COERCION-001 v1.47 (merged) / BC-2.16.015 v1.4 (draft, pass-1+pass-2+pass-3+pass-4+pass-5 fixed). Indexes: ARCH-INDEX v2.333 / BC-INDEX v9.64 / STORY-INDEX v2.898 / HOLDOUT-INDEX v1.26.

**HEADS:**
- `develop`: `3f1e66179` (local == origin; clean)
- `factory-artifacts`: run `git -C .factory log -1 --format='%H'`
- `.worktrees/S-CLAROTY-VULNS-001`: ACTIVE (feature/S-CLAROTY-VULNS-001 @8f4c25c87; Wave A G1 LOCAL pass-6 re-cascade PENDING; feature NOT PUSHED)
- `.worktrees/S-ADR058-OCSF-ROUTING-001`: PENDING-TEARDOWN (PR #242 merged; remote feature branch deleted)
- `.worktrees/S-3.09` @`43c41389d` KEEP-PARKED (LOCAL-ONLY AT RISK -- unpushed)
- `.worktrees/W3-FIX-S307-001` @`fcab8717c` PARKED-DIRTY do-NOT-touch

**HOLDOUT:** HS-024 (VULNS-001; 3 — AUTHORED, UNREAD; consume ONLY after LOCAL 3-CLEAN). HS-029 (ACLPOLICY-001; 3 — AUTHORED, UNREAD). HS-028 (ORGPOLICY-001; 4 — AUTHORED, UNREAD). HS-027 (SERVERS-001; 3 — AUTHORED, UNREAD). HS-026 (DEVVULNREL-001; 3 — AUTHORED, UNREAD). HS-025 (OT-EVENTS-001; 3 — AUTHORED, UNREAD). HS-022/023 CONSUMED.

**BACKUP BOUNDARY (D-2303):**
- PUSHED / safe: `origin/develop` `3f1e66179`; `factory-artifacts` (this burst commit).
- LOCAL-ONLY AT RISK: `.worktrees/S-CLAROTY-VULNS-001` @`8f4c25c87` (feature NOT PUSHED — awaiting LOCAL 3-CLEAN); `.worktrees/S-3.09` @`43c41389d` (unpushed); `.worktrees/W3-FIX-S307-001` @`fcab8717c` (unpushed, dirty).
