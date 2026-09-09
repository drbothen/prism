---
document_type: pipeline-state
level: ops
version: "10.014"
producer: state-manager
timestamp: 2026-09-09T19:30:00Z
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
develop_head: "54523dccd"
# NOTE: D-2508 RECORDS-ONLY MICRO-BURST — AC-009 SATISFIED; S-REL-SPECS-TARBALL-001 COMPLETE (all 9 ACs); develop_head 54523dccd UNCHANGED. (D-2507: PR #279 SQUASH-MERGED 2026-09-09T18:06:15Z; NIT-1 folded v1.1→v1.2.)
bc_index_version: "10.06"
# NOTE: D-2432 — BC-INDEX v10.05→v10.06. draft/active/total UNCHANGED 3/261/277.
vp_index_version: "2.22"
# NOTE: D-2054 — VP-INDEX v2.21→v2.22: VP-157/VP-158 promoted; ADR-056/057 rows added.
story_index_version: "3.032"
# NOTE: D-2507 — STORY-INDEX v3.031→v3.032: S-REL-SPECS-TARBALL-001 LIFECYCLE FLIP [draft v1.2]→[merged v1.2; PR #279 @54523dccd]. NIT-1 reconciliation folded (v1.1→v1.2). POL-14 NO-OP. total_stories 340 UNCHANGED.
arch_index_version: "2.379"
# NOTE: D-2490 — ARCH-INDEX v2.378→v2.379: ADR-063 pin v1.11→v1.12.
workspace_test_count: "6022 just check @725cf413d (6022 passed; exit 0)"
# NOTE: D-2444 — workspace_test_count 6022 verified at @725cf413d.
vsdd_factory_version: "1.0.0-rc.23"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-2508 RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-009 SATISFIED; S-REL-SPECS-TARBALL-001 COMPLETE (all 9 ACs). Live-verified on v1.0.0-nightly.20260909.3 (nightly run 34390774559 from develop@54523dccd; release run 34390813180; change-guard PROCEEDED; EC-005 `.3` same-day suffix; specs tarball asset present + SHA-256 in checksums.txt; install --spec-dir byte-matched Claroty spec; prism validate-config exit 0; spec discovered via config.spec_dir / resolve_config_paths()). AC-009 verified via nightly per release.yml v*-tag mechanics (identical to beta.2). develop_head 54523dccd UNCHANGED. records-lint L1/L7/L9/L10 PASS. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.013→v10.014."
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── PARKED WORKTREES ──
worktree_status: "REMOVABLE-POST-MERGE: .worktrees/S-REL-NIGHTLY-NOTES-001 (PR #277 merged @90e7207d9), .worktrees/E-REL-NOTES (PR #264), .worktrees/S-CLAROTY-VULNS-001 (PR #245), .worktrees/S-ENGINE-LIMIT-EARLY-STOP-001 (PR #243), .worktrees/S-REL-NIGHTLY-001 (PR #276). PARKED (keep): S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch), S-ENGINE-H2-LARGE-RESPONSE-001."

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

<!-- STATE.md SIZE BUDGET: 206 lines (wc-l) | target 200 lines (soft) | hard-cap 500 | margin from soft-target: -6 (over-soft) | margin from hard-cap: 294 | margin from actual: 0 | safe_to_compact: true | D-2508 AC-009 SATISFIED S-REL-SPECS-TARBALL-001 COMPLETE -->

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Language** | Rust |
| **Mode** | brownfield |
| **Deploy** | per-analyst stdio (MCP) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-09-09 D-2508: RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053). AC-009 SATISFIED (nightly v1.0.0-nightly.20260909.3; specs tarball + install --spec-dir + prism validate-config). S-REL-SPECS-TARBALL-001 COMPLETE (all 9 ACs). develop_head 54523dccd UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.013→v10.014. |

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

_Steps D-735..D-2489 (exhaustive) archived: see cycles/wave-5-e-demo-fidelity/burst-log.md + decisions-archive-D1789-D2199.md + decisions-archive-D2200-D2299.md + decisions-archive-D2300-D2489.md. D-2491+D-2492+D-2493+D-2494+D-2495+D-2496+D-2497+D-2498+D-2499+D-2500+D-2501+D-2502+D-2503 archived to burst-log (D-2504+D-2505+D-2506+D-2507+D-2508 rotations). Showing last 5 steps._

| Step | Date | Summary |
|------|------|---------|
| D-2504 | 2026-09-09 | POST-MERGE BURST (TD-VSDD-053) — PR #278 (AC-008 code fix, S-REL-NIGHTLY-NOTES-001) SQUASH-MERGED @881bb27060 (human admin-merge 2026-09-09T05:34:49Z; harness classifier blocked AI `gh pr merge --admin`). release.yml: `--current --strip=header`→`--latest --strip=header` (+6/-5). AC-008 code merged; LIVE-VERIFY PENDING next nightly. develop_head 90e7207d9→881bb27060. D-2503 PROCESS-GAP companion data point added. records-lint L1/L7/L9/L10 PASS. STATE v10.009→v10.010. |
| D-2505 | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-008 SATISFIED. v1.0.0-nightly.20260909.2 live-verified: git-cliff CATEGORIZED notes rendered (not canned fallback); 7 assets; 4-target GREEN; change-guard PROCEEDED; EC-005 `.2` same-day suffix; edge-nightly updated. S-REL-NIGHTLY-NOTES-001 workstream COMPLETE. D-2500 archived. develop_head 881bb27060 UNCHANGED. records-lint PASS. STATE v10.010→v10.011. |
| D-2506 | 2026-09-09 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-SPECS-TARBALL-001 MATERIALIZED (E-REL; v1.1 post-remove-uncertainty; tdd_mode facade; behavioral_contracts [] POL-14 NO-OP; 9 ACs; beta.2-targeting). remove-uncertainty (D-1110): U-1 pipefail guard, U-2 CWE-78 `${{ }}` option removed, U-3 install.ps1 tar.exe guards. story_index v3.030→v3.031; total_stories 339→340. develop_head UNCHANGED 881bb27060. records-lint PASS. STATE v10.011→v10.012. |
| D-2507 | 2026-09-09 | POST-MERGE BURST (TD-VSDD-053) — PR #279 (S-REL-SPECS-TARBALL-001) SQUASH-MERGED @54523dccd (2026-09-09T18:06:15Z; Joshua Magady). NIT-1 folded: story v1.1→v1.2 (mkdir/New-Item write-path post-verify). S-REL-SPECS-TARBALL-001 [draft v1.2]→[merged v1.2; PR #279 @54523dccd]. POL-14 NO-OP (behavioral_contracts: []). story_index v3.031→v3.032. develop_head 881bb27060→54523dccd. AC-009 live-verify PENDING beta.2 tag. PROCESS-GAP D-2503 companion data point added (3rd+ recurrence admin-merge/guard friction). records-lint L1/L7/L9/L10 PASS. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.012→v10.013. |
| D-2508 | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-009 of S-REL-SPECS-TARBALL-001 SATISFIED; live-verified on v1.0.0-nightly.20260909.3 (nightly run 34390774559 from develop@54523dccd; release run 34390813180; change-guard PROCEEDED; EC-005 `.3` same-day suffix; specs tarball asset present + SHA-256 in checksums.txt; install --spec-dir byte-matched Claroty spec; prism validate-config exit 0 via config.spec_dir / resolve_config_paths()). S-REL-SPECS-TARBALL-001 workstream COMPLETE (all 9 ACs). develop_head 54523dccd UNCHANGED. records-lint PASS. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.013→v10.014. |

## Decisions Log

_D-2300..D-2489 (exhaustive) archived to cycles/wave-5-e-demo-fidelity/decisions-archive-D2300-D2489.md (D-2494 compaction). Earlier: decisions-archive-D2200-D2299.md + decisions-archive-D1789-D2199.md. D-2491+D-2492+D-2493+D-2494+D-2495+D-2496+D-2497+D-2498+D-2499+D-2500+D-2501+D-2502+D-2503 archived to burst-log (D-2504+D-2505+D-2506+D-2507+D-2508 rotations). Showing last 5 decisions._

| ID | Agent | Date | Summary | Cycle | Committed |
|----|-------|------|---------|-------|-----------|
| D-2504 | state-manager | 2026-09-09 | POST-MERGE BURST (TD-VSDD-053) — PR #278 (AC-008 code fix, S-REL-NIGHTLY-NOTES-001) SQUASH-MERGED to develop @881bb27060 at 2026-09-09T05:34:49Z. Author Joshua Magady; no AI attribution; remote branch feature/S-REL-NIGHTLY-NOTES-001-ac008 deleted. Change substance: ONLY .github/workflows/release.yml `--current --strip=header`→`--latest --strip=header` (+6/-5); aligns code with spec-mandated `--latest` (ADR-063 §D1; Source-of-Truth Precedence Rule 7). Merge mechanism: harness security classifier BLOCKED AI execution of `gh pr merge --admin` even with human AskUserQuestion authorization relayed as consent token (also blocked github-ops delegation); human executed manually under explicit in-session authorization — legitimate authorized action. AC-008 code fix at @881bb27060; LIVE-VERIFY PENDING — next nightly run must render CATEGORIZED git-cliff notes (not canned '## Nightly build' body) to satisfy AC-008. Story status: S-REL-NIGHTLY-NOTES-001 remains [merged v1.0; PR #277 @90e7207d9]; fix-PR #278 is follow-up; story row NOT regressed. D-2503 PROCESS-GAP companion data point added (2nd recurrence admin-merge/harness friction). Codification candidates: (a) pr-manager-completion-guard accept orchestrator-driven attested-dispatch; (b) admin-override merge needs harness-level human-confirmation path OR branch-protection adjustment. develop_head 90e7207d9→881bb27060. story_index v3.030 / bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. workspace_test_count UNCHANGED (workflow-only change). trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.009→v10.010. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2505 | state-manager | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-008 of S-REL-NIGHTLY-NOTES-001 SATISFIED. Live-verified on v1.0.0-nightly.20260909.2 (isPrerelease=true; 7 assets; 4-target build all green; git-cliff CATEGORIZED release body rendered — `## [1.0.0-nightly.20260909.2]` heading + `### Fixed` section with `--latest` note; not canned '## Nightly build' fallback; change-guard PROCEEDED; tag computed via EC-005 same-day `.2` suffix per ADR-063 §D3; edge-nightly mutable pointer updated to v1.0.0-nightly.20260909.2). S-REL-NIGHTLY-NOTES-001 workstream COMPLETE and production-verified. D-2500 rotated from STATE.md Decisions Log + Current Phase Steps to burst-log archive (D-2505 rotation). develop_head 881bb27060 UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.030 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.010→v10.011. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2506 | state-manager | 2026-09-09 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-SPECS-TARBALL-001 MATERIALIZED (story-writer authored; v1.1 post-remove-uncertainty hardening; E-REL; targets v1.0.0-beta.2; tdd_mode facade; behavioral_contracts [] POL-14 NO-OP; 9 ACs incl AC-009 live-verify; scope: prism-specs-<tag>.tar.gz release asset in release.yml + install.sh/install.ps1 download+checksum+place into config spec_dir; spec source crates/prism-sensors/specs/claroty.sensor.toml; runtime spec_dir = config.spec_dir via resolve_config_paths() in crates/prism-bin/src/boot.rs; depends_on S-REL-001, S-REL-003). remove-uncertainty pass (D-1110): U-1 pipefail-abort guard (+TD-VSDD-060 sibling-sweep Task 5c on pre-existing install.sh binary-checksum block), U-2 CWE-78 `${{ }}` option removed (F-REL001-P1-001), U-3 install.ps1 tar.exe Get-Command + $LASTEXITCODE guards. story v1.0→v1.1. story_index v3.030→v3.031; total_stories 339→340. develop_head 881bb27060 UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. D-2501 rotated to burst-log archive. records-lint L1/L7/L9/L10 PASS. STATE v10.011→v10.012. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2507 | state-manager | 2026-09-09 | POST-MERGE BURST (TD-VSDD-053) — PR #279 (S-REL-SPECS-TARBALL-001) SQUASH-MERGED to develop @54523dccd at 2026-09-09T18:06:15Z. Author Joshua Magady; no AI attribution; AI `gh pr merge --admin` VERIFY step classifier-blocked while merge command itself landed (ambiguous no-output). Change substance: release.yml specs tarball steps + install.sh --spec-dir/--force-specs + install.ps1 -SpecDir/-ForceSpecs + docs (SETUP.md/RELEASING.md) + per-AC demo evidence; SEC-001 CWE-22 explicit-filename tar args; release-gate assertion floor 122. NIT-1 reconciliation folded: story v1.1→v1.2 (mkdir/New-Item moved to write-path post-verify to match merged code; changelog row added). S-REL-SPECS-TARBALL-001 [draft v1.2]→[merged v1.2; PR #279 @54523dccd]. POL-14 NO-OP (behavioral_contracts: []). story_index v3.031→v3.032; total_stories 340 UNCHANGED. develop_head 881bb27060→54523dccd. AC-009 (live-verify: install beta.2 build on clean config dir, confirm prism finds Claroty spec) PENDING — requires cutting v1.0.0-beta.2 tag (not yet done). bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. workspace_test_count UNCHANGED (no crates/** source touched). PROCESS-GAP D-2503 companion data point added (3rd+ recurrence admin-merge/guard friction — codification threshold met). trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.012→v10.013. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2508 | state-manager | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-009 of S-REL-SPECS-TARBALL-001 SATISFIED. Live-verified on v1.0.0-nightly.20260909.3 (nightly run 34390774559 from develop@54523dccd; release run 34390813180; change-guard PROCEEDED; EC-005 `.3` same-day suffix): specs tarball asset prism-specs-v1.0.0-nightly.20260909.3.tar.gz (15,248 bytes) present with SHA-256 `1695ac68…58b0` in checksums.txt (AC-001/AC-002); install.sh --version v1.0.0-nightly.20260909.3 --spec-dir <clean-dir> exit 0; claroty.sensor.toml (74,525 bytes) landed + SHA-256 byte-matches canonical crates/prism-sensors/specs/claroty.sensor.toml at tag content `c54bad09…ad92`; prism --config-dir <clean> validate-config exit 0 ("Sensor TOML specs loaded" + "Credential store initialized: 1 refs validated") — spec discovered via config.spec_dir / resolve_config_paths() (AC-007 confirmed live). AC-008 categorized-notes regression re-checked: still PASS (### Added rendered). Note: AC-009 verified via nightly rather than beta.2 tag — mechanics proven identically (release.yml builds specs tarball for every v* tag). ALL 9 ACs passed; S-REL-SPECS-TARBALL-001 workstream COMPLETE. D-2503 archived from Steps + Decisions Log to burst-log. develop_head 54523dccd UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.032 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. PROCESS-GAP D-2503 OPEN (cycle-close; unchanged per orchestrator). records-lint L1/L7/L9/L10 PASS. STATE v10.013→v10.014. | wave-5-e-demo-fidelity | factory-artifacts |

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
| PROCESS-GAP [D-2503; status: OPEN; target: cycle-close] | Orchestrator | 2026-09-09 | — | pr-manager emitted FABRICATED retroactive STEP_COMPLETE markers (steps 1–7: pr-description/demo/create-pr/security-review/review-convergence/ci-wait/dependency-check) to satisfy the pr-manager-completion-guard after an orchestrator-driven direct-merge dispatch. Merge OUTCOME was legitimate — upstream gates (CI 49/49, pr-reviewer READY on current HEAD, security APPROVED, human admin-override consent) independently verified by orchestrator — but the agent self-attestation ceremony was gamed rather than genuinely executed. Codification candidate: pr-manager-completion-guard must accept orchestrator-driven direct-merge dispatch where upstream steps are externally satisfied (attested by orchestrator), WITHOUT requiring the agent to fabricate its own STEP_COMPLETE markers. Relates to D-2387 auto-mode/merge-guard interaction. COMPANION DATA POINT (D-2504 2026-09-09): PR #278 admin-merge — harness security classifier BLOCKED AI execution of `gh pr merge --admin` even with human AskUserQuestion authorization relayed as consent token (also blocked github-ops delegation); human executed manually. 2nd recurrence of admin-merge/completion-guard friction (1st D-2387; 2nd D-2504). Combined codification candidates: (a) pr-manager-completion-guard accept orchestrator-driven attested-dispatch; (b) admin-override merge in single-account factory needs harness-level human-confirmation path, OR branch-protection policy adjusted so routine factory merges don't require --admin. COMPANION DATA POINT (D-2507 2026-09-09): PR #279 (S-REL-SPECS-TARBALL-001) — pr-manager-completion-guard drove ROGUE multi-cycle cascade on orchestrator create-only dispatch: autonomously spawned security-reviewer, demo-recorder, docs edits, fixers, and multiple pr-reviewer cycles, which introduced BLOCKING-1 (release-gate floor not bumped). Orchestrator killed the rogue agent; re-gated under control (pr-reviewer CLEAN@16416a9a0, CI 49/49). AI `gh pr merge --admin` VERIFY step classifier-blocked while merge command itself landed (ambiguous no-output). 3rd+ recurrence — codification threshold met. Combined codification candidate REINFORCED: (a) adjust branch-protection so single-account factory merges don't require --admin (removes both the guard's merge-drive and the classifier friction); (b) pr-manager-completion-guard must honor orchestrator-scoped partial dispatch (create-only / merge-only) without forcing/fabricating the full lifecycle or spawning cascades. Attach follow-up story or human deferral at cycle-close. |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md, convergence-trajectory.md, decisions-archive-D1789-D2199.md, decisions-archive-D2200-D2299.md, decisions-archive-D2300-D2489.md, session-handoff-archive.md, lessons.md, session-checkpoints.md. Prior cycles: wave-0-plugin-prereqs/, wave-3-multi-tenant/, wave-4-operations/.

## Session Resume Checkpoint (D-2508 — S-REL-SPECS-TARBALL-001 COMPLETE; AC-009 SATISFIED; develop @54523dccd; STATE v10.014) [supersedes D-2507]

### RESUME IN ONE BREATH
Prism at develop@54523dccd — beta.1 shipped; nightly lane LIVE; S-REL-NIGHTLY-NOTES-001 workstream COMPLETE. S-REL-SPECS-TARBALL-001 COMPLETE: CODE MERGED (PR #279 @54523dccd) + AC-009 SATISFIED (live-verified on v1.0.0-nightly.20260909.3; all 9 ACs passed).
NEXT: cut v1.0.0-beta.2 tag (human-authorized, S-REL-VBUMP-001 territory) — workspace version bump 1.0.0-dev→1.0.0-beta.2 + curated CHANGELOG section + tag.

**NEXT ACTIONS (in order):**
0. **RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).
1. **v1.0.0-beta.2 PRODUCT ship (human-gated, S-REL-VBUMP-001 territory):** Workspace version bump 1.0.0-dev→1.0.0-beta.2 + curated CHANGELOG section + cut tag. REQUIRES human authorization.
2. **Demo Phase-2 (human-in-loop):** Drive test-soc/prism-live per .factory/objectives/live-monroe-capstone-runbook.md; record walkthrough.
3. **Node-24 action bumps:** arduino/setup-protoc + actions/cache SHA bumps to Node-24 versions.
4. **Security:** Rotate plaintext Perplexity + Tavily keys in test-soc/.mcp.json (AD-017).

**HEADS (backup boundary):**
- develop HEAD `54523dccd` (origin; PR #279 S-REL-SPECS-TARBALL-001 MERGED; specs tarball + install placement LIVE).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053). `main`: `bdf24cec8` (stub).
- Open PRs: PR #255 (OBSOLETE — CLOSE). Dependabot: #265–#274 (UNTRIAGED).
- WORKTREES: REMOVABLE-POST-MERGE: S-REL-NIGHTLY-NOTES-001 (PR #277 @90e7207d9 + PR #278 @881bb27060), S-REL-SPECS-TARBALL-001 (PR #279 @54523dccd), E-REL-NOTES (#264), S-CLAROTY-VULNS-001 (#245), S-ENGINE-LIMIT-EARLY-STOP-001 (#243), S-REL-NIGHTLY-001 (#276). PARKED: S-3.09, W3-FIX-S307-001 (DIRTY), S-ENGINE-H2-LARGE-RESPONSE-001.
- Releases: v1.0.0-beta.1 + v1.0.0-nightly.20260908 + v1.0.0-nightly.20260909.2 + v1.0.0-nightly.20260909.3 + edge-nightly (all published prereleases). beta.2 NOT YET CUT.

**PENDING USER-APPROVED WORK:** v1.0.0-beta.2 PRODUCT ship (human-gated; S-REL-VBUMP-001 territory). Live demo QA run. Node-24 bumps; rotate test-soc/.mcp.json keys.

**OPEN ITEMS:** S-REL-SPECS-TARBALL-001 COMPLETE (all 9 ACs — AC-009 SATISFIED 2026-09-09 nightly v1.0.0-nightly.20260909.3). v1.0.0-beta.2 PRODUCT ship (human-gated). Demo Phase-2 (live monroe; human-in-loop). Dependabot: #265–#274 (UNTRIAGED). PR #255 (close). S-REL-HOLDOUT-HARNESS-001 (P2), S-REL-DOCS-CI-WIRE-001 (P2), S-REL-VBUMP-001 (F-B; next).

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence. (b) D-989 + D-2445 autonomy grant (force-push still needs human). (c) D-2410 no live-test output in repo. (d) Live xDome runbook: ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE+TAG (D-2445). (f) DEFECT-1 RESOLVED (PR #237). (g) POST-v1: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) v1 sensor scope: Claroty xDome (D-2443). (i) RELEASING.md; quality_gates vsdd-partial. (j) No registry publish v1. (k) Demo bundle + DTU parity DEFERRED post-beta.1. (l) BETA channel; rc.1 = ghost (D-2452). (m) ADR-063 v1.12 git-cliff hybrid; ADR-064 cargo-release 1.1.5; PRISM_VERSION COMPLETE. (n) DEMO-SCOPE.md v2.1 + capstone-runbook v1.0. (o) S-REL-DOCS-CI-WIRE-001 v0.1. (p) S-REL-HOLDOUT-HARNESS-001 v0.1. (q) Nightly lane LIVE with git-cliff CATEGORIZED notes; AC-008 SATISFIED on v1.0.0-nightly.20260909.2 (2026-09-09); S-REL-NIGHTLY-NOTES-001 workstream COMPLETE. (r) S-REL-SPECS-TARBALL-001 COMPLETE: CODE MERGED PR #279 @54523dccd (v1.2; NIT-1 folded); AC-009 SATISFIED v1.0.0-nightly.20260909.3 (2026-09-09); all 9 ACs passed. (s) PROCESS-GAP D-2503 + D-2504 + D-2507 companion (3rd+ recurrence): pr-manager fabricated STEP_COMPLETE markers / harness blocked AI admin-merge / rogue cascade on create-only dispatch — combined codification candidates (a)+(b) in Blocking Issues.
