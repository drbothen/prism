---
document_type: pipeline-state
level: ops
version: "10.005"
producer: state-manager
timestamp: 2026-09-08T18:55:00Z
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
develop_head: "d5d5fe7c"
# NOTE: D-2499 — POST-MERGE. PR #276 (S-REL-NIGHTLY-001) ADMIN SQUASH-MERGED @d5d5fe7c 2026-09-08. Nightly lane LIVE. Prior NOTE D-2493 archived.
bc_index_version: "10.06"
# NOTE: D-2432 — BC-INDEX v10.05→v10.06. draft/active/total UNCHANGED 3/261/277.
vp_index_version: "2.22"
# NOTE: D-2054 — VP-INDEX v2.21→v2.22: VP-157/VP-158 promoted; ADR-056/057 rows added.
story_index_version: "3.028"
# NOTE: D-2499 — STORY-INDEX v3.027→v3.028: S-REL-NIGHTLY-001 [ready v1.0]→[merged v1.0; PR #276 @d5d5fe7c]. POL-14 NO-OP (behavioral_contracts: []). total_stories 338 UNCHANGED.
arch_index_version: "2.379"
# NOTE: D-2490 — ARCH-INDEX v2.378→v2.379: ADR-063 pin v1.11→v1.12.
workspace_test_count: "6022 just check @725cf413d (6022 passed; exit 0)"
# NOTE: D-2444 — workspace_test_count 6022 verified at @725cf413d.
vsdd_factory_version: "1.0.0-rc.23"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-2499 SINGLE-COMMIT BURST (TD-VSDD-053) — PR #276 (S-REL-NIGHTLY-001) ADMIN SQUASH-MERGED @d5d5fe7c (explicit human auth; 4 review cycles CLEAN(PR-merge); CI 48/48 green). Nightly lane LIVE. develop_head 1724e727f→d5d5fe7c. S-REL-NIGHTLY-001 [ready v1.0]→[merged v1.0; PR #276 @d5d5fe7c]. POL-14 NO-OP (behavioral_contracts: []). story_index v3.027→v3.028. FOLLOW-UP TRACKED: S-REL-NIGHTLY-NOTES-001 (draft, own-track; git-cliff categorized notes). LIVE-VERIFY (workflow_dispatch nightly) IN PROGRESS (authorized Q3). Queued: S-REL-SPECS-TARBALL-001 (beta.2). records-lint L1/L7/L9/L10 PASS. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.004→v10.005."
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── PARKED WORKTREES ──
worktree_status: "TORN DOWN (D-2494): .worktrees/S-REL-BETA1-NOTES-001. REMOVABLE: .worktrees/E-REL-NOTES (D-2492), .worktrees/S-REL-AGENT-VERSION-001 (D-2473), .worktrees/S-REL-VERSION-IDENTITY (D-2465), .worktrees/S-CLAROTY-VULNS-001, .worktrees/S-ENGINE-H2-LARGE-RESPONSE-001, .worktrees/S-ENGINE-LIMIT-EARLY-STOP-001. PARKED (keep): S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch). All wave-5-e feature worktrees MERGED D-2387..D-2415 (exhaustive)."

# ── DTU + PIPELINE META ──
dtu_required: true
dtu_assessment: COMPLETE
dtu_assessment_approved: 2026-04-20
dtu_clones_built: in_progress
dtu_strategy: "Option 2 — DTU-first"
dtu_strategy_decided: 2026-04-20
active_objective: "v1 FIRST RELEASE: fully-working Claroty xDome sensor, end-to-end (D-2264 GOVERNING DECISION 2026-08-21). Validation: REAL Claroty xDome tenant (live API; AD-017 opaque). v1 scope: client+sensor onboarding → OCSF correctness → all query shapes → push-down → SOC-analyst Q&A loop → stability. Release gate: live xDome validation. POST-v1 de-scoped: S-OCSF-FIDELITY-CROWDSTRIKE/CYBERINT/ARMIS-001 + S-ADR058-DTU-PARITY-MIGRATION-001."
# NOTE: D-2443 — v1 is Claroty-xDome-only AND ships WITHOUT demo bundle. Demo bundle (S-REL-004) + DTU parity DEFERRED post-beta.1.
task_ledger: ".factory/objectives/multi-client-soc-demo-tasks.md"
demo_scope_doc: ".factory/objectives/DEMO-SCOPE.md"
api_specs_reference: ".factory/reference/api-specs/"
user_directive_persistent: "No pragmatic convergence. Fix all issues before build."
v1_release_merge_authority: "D-2400 blanket grant (2026-08-31) + D-2445 FULL AUTONOMOUS MERGE+TAG AUTHORITY (2026-09-04): ALL PRs autonomous on green gates (security CLEAN/PR-merge + pr-reviewer READY + CI green + stale-verdict exit 0). Force-push to any branch STILL requires explicit human approval."
user_directive_remove_uncertainty: "Run dclaude:remove-uncertainty on every implementation story BOTH immediately after story-writer materializes/writes it AND again before TDD delivery (D-1110 extension 2026-06-12)."
policy_registry_source_of_truth: .factory/policies.yaml
sprint_state_path: ".factory/stories/sprint-state.yaml"
historical_cycles: [phase-1-convergence, wave-3-multi-tenant, wave-4-operations, wave-0-plugin-prereqs]
current_cycle: wave-5-e-demo-fidelity

# ── LOCKED ARCHITECTURAL DECISIONS ──
architectural_decisions_locked:
  - "1 LOCKED Option-A: TOML spec URLs ground against DTU clone routes [SUPERSEDED by ADR-053 §D1 — grounding-order flip EFFECTIVE]"
  - "2 LOCKED Option-B: Parity test loads reference OCSF from committed fixture JSON"
  - "3 LOCKED Option-A: Expand PLUGIN-MIGRATION-001-D scope to include SpecErrorCode::ESpec017 variant in prism-core + filename-stem validation"
  - "4 LOCKED Option-A: TOML auth_type declares REAL behavior [SUPERSEDED by ADR-053 §D3 — Cyberint dual-surface split EFFECTIVE]"
  - "5 LOCKED Path-A (D-747): ADR-028 §D2 supersedes ADR-026 §D3 partial [SUPERSEDED by ADR-053 §D2 — Armis token_exchange EFFECTIVE]"

# ── COMPACTION RECORD ──
pre_compact_snapshot: "See cycles/wave-5-e-demo-fidelity/: decisions-archive-D1789-D2199.md + decisions-archive-D2200-D2299.md + decisions-archive-D2300-D2489.md + session-handoff-archive.md + drift-items-open.md. D-2494 compaction (2026-09-08): decisions D-2300..D-2489 (exhaustive) + Current Phase Steps D-2368..D-2489 archived; snapshots D-2321..D-2491 archived. D-2305 compaction (2026-08-26): D-2200..D-2299 archived. D-2237 compaction (2026-08-18): D-1789..D-2199 archived. Git history on factory-artifacts preserves all content."
pre_compact_snapshot_at: "2026-09-08"
---

<!-- STATE.md SIZE BUDGET: 205 lines (wc-l) | target 200 lines (soft) | hard-cap 500 | margin from soft-target: -5 (over-soft) | margin from hard-cap: 295 | margin from actual: 0 | compact eligible: safe_to_compact: true -->

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Language** | Rust |
| **Mode** | brownfield |
| **Deploy** | per-analyst stdio (MCP) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-09-08 D-2499: PR #276 (S-REL-NIGHTLY-001) MERGED @d5d5fe7c. Nightly lane LIVE. develop_head 1724e727f→d5d5fe7c. story_index v3.027→v3.028. trajectory-tail →8→0→1→2. STATE v10.004→v10.005. |

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
| S-ADR058-OCSF-COERCION-001 TDD + PR cycle | MERGED | 2026-08-20 | 2026-08-20 | PR #240 develop@362e4f85 2026-08-20 | LOCAL cascade CONVERGED (D-2259); HOLDOUT PASS 4/4 (HS-001..HS-004 real MCP stdio); demo COMPLETE; just check 5765 GREEN; active_contracts 252→253 |
| S-ADR058-OCSF-ROUTING-001 LOCAL+HOLDOUT+DEMO + PR-LEVEL FIX-BURSTS | MERGED | 2026-08-23 | 2026-08-23 | PR #242 SQUASH-MERGED to develop@3f1e66179 (D-2288) | LOCAL 3-CLEAN D-2283; HOLDOUT PASS D-2285 HS-023 3/3; DEMO 21/21 ACs; PR-LEVEL 3-CLEAN CONVERGED; MERGED D-2288. |

_Historical Phase Progress rows archived to cycles/wave-5-e-demo-fidelity/burst-log.md (D-1794 + D-2237 + D-2244+1 + D-2261 compactions)._

## Convergence Status

| Metric | Value |
|--------|-------|
| BC-5.39.001 streak | G1–G6 ALL MERGED; v1 Claroty xDome 14-table sensor COMPLETE. v1.0.0-beta.1 PUBLISHED 2026-09-08. NEXT: triage dependabot + post-beta.1 stories. |
| Active cascade | G1 MERGED (D-2387; PR #245 @6972ac2e). G2 MERGED (D-2401; PR #246 @3d724a069). G3 MERGED (D-2404; PR #247 @12cecb12). G4 MERGED (D-2407; PR #248 @157596490). G5 MERGED (D-2412; PR #249 @07e64f4e). G6 MERGED (D-2415; PR #250 @672b10b6). D-2396 CONVERGENCE-BAR satisfied. |
| Pass count | trajectory-tail →8→0→1→2. Full history: cycles/wave-5-e-demo-fidelity/convergence-trajectory.md |
| Last CLEAN(strict) | VULNS-001 PR-LEVEL pass-3 on frozen 73bea7c1c CLEAN(strict) (streak 1/3). |
| Frozen perimeter | ADR-058 v2.34 / ADR-059 v1.2 (WITHDRAWN) / ADR-060 v1.18 / ADR-061 v1.2 / BC-2.16.002 v2.54 / BC-2.11.001 v1.31 / BC-2.16.003 v1.27 / BC-2.11.016 v1.31 / error-taxonomy v2.82 / ROUTING-001 v1.57 (merged) / COERCION-001 v1.47 (merged). |

## Concurrent Cycles

_Current cycle: wave-5-e-demo-fidelity. No parallel cycles running._

## Current Phase Steps

_Steps D-735..D-2489 (exhaustive) archived: see cycles/wave-5-e-demo-fidelity/burst-log.md + decisions-archive-D1789-D2199.md + decisions-archive-D2200-D2299.md + decisions-archive-D2300-D2489.md. D-2491+D-2492+D-2493+D-2494 archived to burst-log (D-2496+D-2497+D-2498+D-2499 rotations). Showing last 5 steps._

| Step | Date | Summary |
|------|------|---------|
| D-2495 | 2026-09-08 | DEMO LIVE-ENV PIVOT + DEMO-SCOPE.md v2.0→v2.1. Human-directed: live demo → LIVE Claroty xDome (monroe); DTU fleet STALE/non-functional + OUT of demo path; D-1163 DTU-EVERYTHING SUPERSEDED; 4 packaging gaps (a–d) documented. STATE v10.000→v10.001. |
| D-2496 | 2026-09-07 | SINGLE-COMMIT BURST — live-validation-matrix.md v0.2→v0.3 + soc-analyst-qa-catalog.md v0.2→v0.3: 4 tables → 14 tables (106 items + 64 Q&A; 64 SOC questions). v1.0.0-beta.1 binary deployed to live SOC. Demo Phase-1 DONE. STATE v10.001→v10.002. |
| D-2497 | 2026-09-08 | SINGLE-COMMIT BURST — live-monroe-capstone-runbook.md v1.0 NEW (10 acts / 24 beats; all 14 Claroty tables; DTU-zero; SUPERSEDES T13 runbook). Demo Phase-1 prep COMPLETE. Nightly lane own track (feature/S-REL-NIGHTLY-001). STATE v10.002→v10.003. |
| D-2498 | 2026-09-08 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-NIGHTLY-001 REGISTERED (ready v1.0; E-REL; P2; facade; own INDEPENDENT track; NOT beta.2-blocking). story_index v3.026→v3.027; total_stories 337→338. AS-BUILT on feature/S-REL-NIGHTLY-001 @a2c7a8bce. STATE v10.003→v10.004. |
| D-2499 | 2026-09-08 | POST-MERGE BURST (TD-VSDD-053) — PR #276 (S-REL-NIGHTLY-001) ADMIN SQUASH-MERGED @d5d5fe7c (explicit human auth; 4 review cycles CLEAN(PR-merge); CI 48/48). Nightly lane LIVE. develop_head 1724e727f→d5d5fe7c. POL-14 NO-OP (behavioral_contracts: []). story_index v3.027→v3.028. FOLLOW-UP TRACKED: S-REL-NIGHTLY-NOTES-001 (draft, own-track; git-cliff notes). LIVE-VERIFY IN PROGRESS (authorized Q3). records-lint L1/L7/L9/L10 PASS. STATE v10.004→v10.005. |

## Decisions Log

_D-2300..D-2489 (exhaustive) archived to cycles/wave-5-e-demo-fidelity/decisions-archive-D2300-D2489.md (D-2494 compaction). Earlier: decisions-archive-D2200-D2299.md + decisions-archive-D1789-D2199.md. D-2491+D-2492+D-2493+D-2494 archived to burst-log (D-2496+D-2497+D-2498+D-2499 rotations). Showing last 5 decisions._

| ID | Agent | Date | Summary | Cycle | Committed |
|----|-------|------|---------|-------|-----------|
| D-2495 | state-manager | 2026-09-08 | SINGLE-COMMIT BURST (TD-VSDD-053) — DEMO LIVE-ENVIRONMENT PIVOT, human-directed 2026-09-08. Live demo runs against LIVE Claroty xDome tenant (monroe; live API; AD-017 opaque creds), NOT DTU clone fleet. DTU fleet STALE/non-functional; S-CLAROTY-DTU-PARITY-001 + S-REL-004 remain DEFERRED post-rc.1. SUPERSEDES D-1163 (DTU-EVERYTHING). DEMO-SCOPE.md v2.0→v2.1: D-1163 marked superseded (historical text preserved); stale SCOPED-NOT-BUILT section corrected to merged-status truth + 4 real packaging gaps (a–d); T13 DTU runbook flagged stale. Demo readiness: READY-WITH-GAPS. Human authorized full live-demo packaging. develop_head UNCHANGED 1724e727f; STATE v10.000→v10.001. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2496 | state-manager | 2026-09-07 | SINGLE-COMMIT BURST (TD-VSDD-053) — live-validation-matrix.md v0.2→v0.3 + soc-analyst-qa-catalog.md v0.2→v0.3: expanded from 4 tables to all 14 Claroty xDome tables. live-validation-matrix.md v0.3: 106 validation items + 64 Q&A. soc-analyst-qa-catalog.md v0.3: 64 SOC questions. Scoped to LIVE monroe tenant; customer-data-free (AD-017). enrich_nvd live-safe; enrich_threat_intel DTU-bound (not live-safe). Supersedes 4-table versions. Demo Phase-1 progress toward 14-table live Q&A run (Phase-2, human-in-loop). Factual: v1.0.0-beta.1 binary deployed to live SOC env (checksum-verified). records-lint L1/L7/L9/L10 PASS. develop_head UNCHANGED 1724e727f. STATE v10.001→v10.002. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2497 | state-manager | 2026-09-08 | SINGLE-COMMIT BURST (TD-VSDD-053) — Live-monroe capstone narrative runbook authored: .factory/objectives/live-monroe-capstone-runbook.md v1.0 (10 acts / 24 beats; all 14 Claroty tables exercised; enrich nvd only, enrich threat_intel prohibited as DTU-bound; DTU-zero; customer-data-free per AD-017; launcher test-soc/prism-live-mcp-wrapper.sh; read-only surface). SUPERSEDES stale T13-capstone-demo-runbook.md (DTU-based). Autonomous demo Phase-1 prep COMPLETE (14-table live-validation matrix + 64-question SOC Q&A catalog per D-2496 + this capstone narrative). Remaining demo work = Phase-2: live monroe Q&A run + recorded walkthrough (HUMAN-IN-LOOP; MCP trust-gate operator-approved). PLANNING NOTES: (1) nightly release lane own track (human-directed 2026-09-08), in progress on feature/S-REL-NIGHTLY-001 (nightly.yml cron 07:17 UTC + change-guard + vX.Y.Z-nightly.YYYYMMDD via RELEASE_PROMOTE_TOKEN → release.yml; keep-14 retention; edge-nightly pointer; RELEASE-CHANNELS §5 4-target doc fix). (2) S-REL-SPECS-TARBALL-001 TARGETED for v1.0.0-beta.2. (3) v1.0.0-beta.1 PUBLISHED (D-2494) + DEPLOYED to live SOC env (binary + matching 14-table Claroty spec). (4) SECURITY: plaintext API keys observed in test-soc/.mcp.json (Perplexity + Tavily) — operator advised to rotate + move to secret refs (values NOT recorded here per AD-017). records-lint L1/L7/L9/L10 PASS. develop_head UNCHANGED 1724e727f. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.026 / total_stories 337 UNCHANGED. STATE v10.002→v10.003. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2498 | state-manager | 2026-09-08 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-NIGHTLY-001 REGISTERED (ready v1.0; E-REL; P2; Wave F-A; tdd_mode: facade; subsystems [SS-22]; crates_touched []; behavioral_contracts [] POL-14 NO-OP; depends_on S-REL-001; OWN INDEPENDENT TRACK human-directed 2026-09-08; NOT beta.2-blocking; 10 ACs; AC-010 live workflow_dispatch human-in-loop verify; nightly.yml: cron 07:17 UTC + change-guard + vX.Y.Z-nightly.YYYYMMDD tag via RELEASE_PROMOTE_TOKEN → release.yml 4-target build + prerelease; keep-14 retention two-layer safety filter; edge-nightly mutable pointer delete+recreate non-v* tag). Implementation AS-BUILT on feature/S-REL-NIGHTLY-001 @a2c7a8bce (2 commits: c799847d5 nightly.yml + a2c7a8bce RELEASE-CHANNELS §2/§6/§7 status). RELEASE-CHANNELS §5 4-target already correct per D-2497. release.yml UNCHANGED. records-lint L1/L7/L9/L10 PASS. develop_head UNCHANGED 1724e727f. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. story_index v3.026→v3.027; total_stories 337→338. STATE v10.003→v10.004. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2499 | state-manager | 2026-09-08 | POST-MERGE BURST (TD-VSDD-053) — PR #276 (S-REL-NIGHTLY-001, nightly release lane) ADMIN SQUASH-MERGED to develop @d5d5fe7c under explicit in-session human authorization. Nightly lane now LIVE: nightly.yml (cron 07:17 UTC + workflow_dispatch; change-guard; vX.Y.Z-nightly.YYYYMMDD BASE-MATCH tag via RELEASE_PROMOTE_TOKEN → release.yml) + channel-aware release.yml (nightly canned summary body: '## Nightly build' + develop@sha + date + CHANGELOG-Unreleased link; stable/alpha/beta/rc curated-CHANGELOG hard-fail PRESERVED; exact-form nightly regex prevents hybrid-tag bypass) + keep-14 retention (two-layer nightly-only filter) + edge-nightly mutable pointer + LOW-12 SIGPIPE hardening across release-tag/promote/prep. RELEASE-CHANNELS §2 nightly PLANNED→IMPLEMENTED / §6 §7 PARTIALLY IMPLEMENTED. 4 review cycles CLEAN(PR-merge). CI 48/48 green. develop_head 1724e727f→d5d5fe7c. S-REL-NIGHTLY-001 [ready v1.0]→[merged v1.0; PR #276 @d5d5fe7c]. POL-14 NO-OP (behavioral_contracts: []). story_index v3.027→v3.028. Review: CRIT-1 (Option-A git-cliff unworkable in publish job: git-cliff not installed + shallow checkout) resolved via canned summary body (Option B). FOLLOW-UP TRACKED: S-REL-NIGHTLY-NOTES-001 (draft, own-track) — enhance nightly release notes from canned summary to git-cliff categorized changelog; requires git-cliff install + full-history checkout in release.yml nightly path; human-directed 2026-09-08 Q1 preference; story to be materialized by story-writer when picked up. LIVE-VERIFY (workflow_dispatch nightly) IN PROGRESS to publish first v1.0.0-nightly.YYYYMMDD prerelease + edge-nightly (authorized Q3). D-2494 rotated to burst-log archive. records-lint L1/L7/L9/L10 PASS. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. STATE v10.004→v10.005. | wave-5-e-demo-fidelity | factory-artifacts |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Blocking Issues

| Issue | Owner | Opened | Resolved | Notes |
|-------|-------|--------|----------|-------|
| ADR-058-SPEC-READINESS-FAIL-001 [D-2176; status: CLOSED] | architect + product-owner + story-writer | 2026-08-15 | 2026-08-20 | CLOSED — COERCION-001 MERGED (PR #240 @362e4f85 2026-08-20). |
| F-R11-CRIT-001 [status: RESOLVED 2026-08-26 D-2326] | architect + product-owner + implementer + test-writer | 2026-08-26 | 2026-08-26 | ADR-060 §D8.7 plan-shape gate implemented; BC-2.16.002 v2.39 + 9 RG-PSG-001..009 tests GREEN; just check 5836 GREEN. |
| F-R12-CRIT-001 [status: RESOLVED D-2328] | architect + implementer + test-writer | 2026-08-27 | 2026-08-27 | Fixed @1f1b06309: comprehensive plan-shape audit — ADR-060 v1.3 conditions A–J + conservative default. just check GREEN 5846. |
| F-R12-HIGH-001 [status: RESOLVED D-2328] | architect + PO + implementer | 2026-08-27 | 2026-08-27 | Fixed @1f1b06309: Condition H JOIN. just check GREEN 5846. |
| F-R13-CRIT-001 [status: RESOLVED D-2329] | architect + implementer + test-writer | 2026-08-27 | 2026-08-27 | Fixed @968e73f05: truncate_result_to_limit pre-cap REMOVED; engine.rs Step 6 caps+signals. just check GREEN 5847. |
| F-R15-LENSA-CRIT-001 [status: SPEC-REMEDIATED D-2332 (CODE-PENDING round-16)] | architect + PO + implementer + test-writer | 2026-08-27 | D-2332 SPEC | BC-2.16.002 v2.41 EC-01-030..031: is_pushed_temporal_predicate redesigned. Story v1.13 RG-PSG-021..023 RED tests written. CODE-PENDING: round-16. |
| F-R15-LENSA-HIGH-001 [status: SPEC-REMEDIATED D-2332 (CODE-PENDING round-16)] | architect + implementer + test-writer | 2026-08-27 | D-2332 SPEC | BC-2.16.002 v2.41 EC-01-032..033 + BC-2.11.001 v1.26 EC-11-092/093: early_stopped truncation-signal chain. Story v1.13 RG-PSG-024..025 RED tests written. CODE-PENDING: round-16. |
| F-R16-P1-CRIT-001 [status: SPEC-REMEDIATED D-2333 (CODE-PENDING round-16)] | architect + PO + implementer + test-writer | 2026-08-28 | D-2333 SPEC | ADR-061 §D1 cache-key identity invariant: relative-temporal PERMIT path gates on OrgRegistry::slug_for. RG-PSG-026..029 + RG-SLUG-001..006 RED uncommitted. |
| F-R16-P1-HIGH-001 [severity elevated; CWE-284/340/200; status: SPEC-REMEDIATED D-2333 (CODE-PENDING round-16)] | security-reviewer + architect + implementer | 2026-08-28 | D-2333 SPEC | ADR-061 v1.0 NEW: 3 defect sites closed; D2 skip-with-structured-warn fail-closed; RG-SLUG-001..006. |
| PROCESS-GAP [D-2092; status: OPEN] | Orchestrator | 2026-08-02 | — | version-field-sync ambiguity in dispatch brief; process improvement |
| PROCESS-GAP [D-2091; anchor: S-MAINT-BURST-COMMIT-COUNT-GATE-001; status: MITIGATED] | Orchestrator | 2026-08-02 | — | S-MAINT-BURST-COMMIT-COUNT-GATE-001 ARCH-QUES-001 pending |
| PROCESS-GAP [D-2368; status: OPEN; target: post-Monday 2026-08-31] | Orchestrator | 2026-08-30 | — | LIMIT story holdout scenarios absent at materialization time — authored retroactively as HS-030. Validator gate pending at S-MAINT-RG-LIST-GATE-001 scope. |
| PROCESS-GAP [D-2377; status: OPEN; target: post-Monday 2026-08-31] | Orchestrator | 2026-08-30 | — | BC-bump burst checklist must enumerate ALL traces_to/bcs-dependent stories via STORY-INDEX grep before declaring complete. Attach to S-MAINT-ANTIPIN-SWEEP-001 or new story. |
| PROCESS-GAP [D-2387; status: OPEN; target: cycle-close] | Orchestrator | 2026-08-31 | — | Auto-mode classifier blocked pr-manager merges (manufactured-auth false positive). Codify protocol: pr-manager dispatch brief must carry explicit human-consent token. |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md, convergence-trajectory.md, decisions-archive-D1789-D2199.md, decisions-archive-D2200-D2299.md, decisions-archive-D2300-D2489.md, session-handoff-archive.md, lessons.md, session-checkpoints.md. Prior cycles: wave-0-plugin-prereqs/, wave-3-multi-tenant/, wave-4-operations/.

## Session Resume Checkpoint (D-2499 — NIGHTLY-001-MERGED + LIVE-VERIFY-IN-PROGRESS; develop @d5d5fe7c; STATE v10.005) [supersedes D-2498]

### RESUME IN ONE BREATH
Phase 3, S-REL-NIGHTLY-001 MERGED (PR #276 @d5d5fe7c; D-2499). Nightly lane LIVE; live-verify (first v1.0.0-nightly.YYYYMMDD) IN PROGRESS (authorized Q3). story_index v3.028 / total_stories 338. develop_head d5d5fe7c; bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. v1.0.0-beta.1 PUBLISHED + deployed to live SOC. Demo Phase-1 COMPLETE. S-REL-NIGHTLY-NOTES-001 TRACKED (draft, own-track; git-cliff notes).

**NEXT ACTIONS (in order):**
0. **RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).
1. **Await live-verify:** First v1.0.0-nightly.YYYYMMDD prerelease + edge-nightly pointer on develop@d5d5fe7c (workflow_dispatch nightly, authorized Q3).
2. **Demo Phase-2 (human-in-loop):** Live monroe Q&A run against real xDome tenant (MCP trust-gate) + demo recording. Human must authorize each live API call.
3. **S-REL-NIGHTLY-NOTES-001 (own track, draft):** git-cliff categorized notes follow-up. Story-writer to materialize when picked up.
4. **Queued — S-REL-SPECS-TARBALL-001:** Standalone specs tarball + install/docs guidance, TARGETED for v1.0.0-beta.2.
5. **Post-demo triage:** 10 dependabot PRs #265–#274 (cargo+github-actions bumps; non-blocking). Close PR #255 (obsolete rc.1 CHANGELOG; human UI).

**HEADS (backup boundary):**
- `develop`: origin = `d5d5fe7c` (PR #276 S-REL-NIGHTLY-001 merged 2026-09-08; nightly lane LIVE).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053).
- `main`: `bdf24cec8` (stub, untouched; first main join deferred to stable v1.0.0).
- Open PRs: #255 (OBSOLETE rc.1 CHANGELOG — CLOSE, do NOT merge). Dependabot: #265–#274 (UNTRIAGED).
- WORKTREES PARKED (do not remove): S-3.09 (KEEP), W3-FIX-S307-001 (DIRTY do-NOT-touch).

**PENDING USER-APPROVED WORK:** Demo Phase-2 (live monroe Q&A run + recording) requires human-in-loop MCP trust gate. Force-push to any branch still requires explicit human approval.

**OPEN ITEMS:** Await live-verify (nightly, authorized Q3). Demo Phase-2 (live monroe run + recording; human-in-loop). 10 dependabot PRs #265–#274 (UNTRIAGED; non-blocking). PR #255 (close human UI). Post-beta.1: S-REL-HOLDOUT-HARNESS-001 (P2), S-REL-DOCS-CI-WIRE-001 (P2), S-REL-VBUMP-001 (F-B), S-REL-NIGHTLY-NOTES-001 (draft, own-track; git-cliff notes follow-up). S-REL-SPECS-TARBALL-001 (targeted beta.2).

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence / fix all issues (production-grade default). (b) D-989 + D-2445 autonomy grant in force (merge+tag on green gates; force-push any branch still needs human). (c) D-2410 DO NOT SAVE LIVE-TEST OUTPUT INTO REPO. (d) Live xDome validation: canonical runbook .factory/ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE + TAG (D-2445, 2026-09-04): all PRs autonomous on green gates; force-push STILL requires explicit human approval. (f) DEFECT-1 RESOLVED by PR #237. (g) POST-v1 FOLLOW-UPS: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) Sensor scope v1: Claroty xDome ONLY (D-2440/D-2443). (i) RELEASING.md at repo root; release-config quality_gates vsdd-partial. (j) No registry publish in v1 (DEF-REL-002/003/004; S-REL-008 future). (k) Demo bundle (S-REL-004) and Claroty DTU parity (S-CLAROTY-DTU-PARITY-001) DEFERRED post-beta.1 per D-2443. (l) BETA channel; v1.0.0-rc.1 = immutable never-published ghost (D-2452). (m) git-cliff 2.14.1 hybrid model (ADR-063); cargo-release 1.1.5 (ADR-064 §D3); PRISM_VERSION injection COMPLETE (ADR-064 §D4 + ADR-050 §D6); docs VERSION-AGNOSTIC; ADR-063 v1.12: DEVIATION-1/2/3 + §D3 catch-all skip parser + breaking block guard documented. (n) D-2495/D-2496/D-2497: LIVE demo direction — DEMO-SCOPE.md v2.1 authoritative; live-monroe-capstone-runbook.md v1.0 is the capstone execution guide; T14 recording against live monroe. (o) S-REL-DOCS-CI-WIRE-001 draft stub v0.1 (D-2456). (p) S-REL-HOLDOUT-HARNESS-001 draft v0.1 (D-2472). (q) D-2499: S-REL-NIGHTLY-001 MERGED (PR #276 @d5d5fe7c; nightly lane LIVE; live-verify IN PROGRESS authorized Q3); S-REL-NIGHTLY-NOTES-001 tracked follow-up (draft, own-track; git-cliff notes); S-REL-SPECS-TARBALL-001 targeted beta.2.
