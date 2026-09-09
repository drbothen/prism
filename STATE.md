---
document_type: pipeline-state
level: ops
version: "10.018"
producer: state-manager
timestamp: 2026-09-09T23:30:00Z
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
develop_head: "baf720d89"
# NOTE: D-2512 SESSION WRAP — durable §RESUME SNAPSHOT D-2512 written; D-2511 snapshot SUPERSEDED. develop_head baf720d89 UNCHANGED. (D-2511: v1.0.0-beta.2 PRE-RELEASE PUBLISHED; develop_head baf720d89 UNCHANGED.)
bc_index_version: "10.06"
# NOTE: D-2432 — BC-INDEX v10.05→v10.06. draft/active/total UNCHANGED 3/261/277.
vp_index_version: "2.22"
# NOTE: D-2054 — VP-INDEX v2.21→v2.22: VP-157/VP-158 promoted; ADR-056/057 rows added.
story_index_version: "3.033"
# NOTE: D-2509 — STORY-INDEX v3.032→v3.033: S-REL-CHANGELOG-CHANNEL-SCOPE-001 REGISTERED (draft v1.0; E-REL; P1; facade; behavioral_contracts [] POL-14 NO-OP; depends_on S-REL-CLIFF-001 + S-REL-NIGHTLY-NOTES-001 (both merged); 10 ACs; ADR-063 §D7 amendment pending). total_stories 340→341.
arch_index_version: "2.379"
# NOTE: D-2490 — ARCH-INDEX v2.378→v2.379: ADR-063 pin v1.11→v1.12.
workspace_test_count: "6022 just check @725cf413d (6022 passed; exit 0)"
# NOTE: D-2444 — workspace_test_count 6022 verified at @725cf413d.
vsdd_factory_version: "1.0.0-rc.23"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-2512 SESSION WRAP (TD-VSDD-053) — Durable §RESUME SNAPSHOT D-2512 written; D-2511 snapshot SUPERSEDED. v1.0.0-beta.2 PUBLISHED this session (PRs #277/#278/#279/#280 all merged; AC-008+AC-009 live-verified; beta.2 workstream COMPLETE). Nothing in flight. NEXT: architect authors ADR-063 §D7 per-channel tag-scoping amendment → S-REL-CHANGELOG-CHANNEL-SCOPE-001 draft→ready→deliver (v1.0.0-stable blocker). develop_head baf720d89 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.017→v10.018. SESSION-HANDOFF v9.009→v9.010."
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

<!-- STATE.md SIZE BUDGET: 201 lines (wc-l) | target 200 lines (soft) | hard-cap 500 | margin from soft-target: -1 (over-soft) | margin from hard-cap: 299 | margin from actual: 0 | safe_to_compact: true | D-2512 SESSION WRAP -->

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Language** | Rust |
| **Mode** | brownfield |
| **Deploy** | per-analyst stdio (MCP) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-09-09 D-2512: SESSION WRAP (TD-VSDD-053). Durable §RESUME SNAPSHOT D-2512 written; D-2511 snapshot SUPERSEDED. v1.0.0-beta.2 PUBLISHED this session (PRs #277/#278/#279/#280 merged; AC-008+AC-009 live-verified; beta.2 workstream COMPLETE). Nothing in flight. NEXT: ADR-063 §D7 amendment → S-REL-CHANGELOG-CHANNEL-SCOPE-001 ready→deliver. develop_head baf720d89 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.017→v10.018. |

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
| BC-5.39.001 streak | G1–G6 ALL MERGED; v1 Claroty xDome 14-table sensor COMPLETE. v1.0.0-beta.1 PUBLISHED 2026-09-08. v1.0.0-beta.2 PUBLISHED 2026-09-09 (release-tag.yml run 34407101117 + release.yml run 34407137100; RELEASING.md §6 all 8 PASS). NEXT: ADR-063 §D7 amendment → S-REL-CHANGELOG-CHANNEL-SCOPE-001 ready → v1.0.0 stable. |
| Active cascade | G1 MERGED (D-2387; PR #245 @6972ac2e). G2 MERGED (D-2401; PR #246 @3d724a069). G3 MERGED (D-2404; PR #247 @12cecb12). G4 MERGED (D-2407; PR #248 @157596490). G5 MERGED (D-2412; PR #249 @07e64f4e). G6 MERGED (D-2415; PR #250 @672b10b6). D-2396 CONVERGENCE-BAR satisfied. |
| Pass count | trajectory-tail →8→0→1→2. Full history: cycles/wave-5-e-demo-fidelity/convergence-trajectory.md |
| Last CLEAN(strict) | VULNS-001 PR-LEVEL pass-3 on frozen 73bea7c1c CLEAN(strict) (streak 1/3). |
| Frozen perimeter | ADR-058 v2.34 / ADR-059 v1.2 (WITHDRAWN) / ADR-060 v1.18 / ADR-061 v1.2 / BC-2.16.002 v2.54 / BC-2.11.001 v1.31 / BC-2.16.003 v1.27 / BC-2.11.016 v1.31 / error-taxonomy v2.82 / ROUTING-001 v1.57 (merged) / COERCION-001 v1.47 (merged). |

## Concurrent Cycles

_Current cycle: wave-5-e-demo-fidelity. No parallel cycles running._

## Current Phase Steps

_Steps D-735..D-2489 (exhaustive) archived: see cycles/wave-5-e-demo-fidelity/burst-log.md + decisions-archive-D1789-D2199.md + decisions-archive-D2200-D2299.md + decisions-archive-D2300-D2489.md. D-2491..D-2507 archived to burst-log (D-2508+D-2509+D-2510+D-2511+D-2512 rotations). Showing last 5 steps._

| Step | Date | Summary |
|------|------|---------|
| D-2508 | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-009 of S-REL-SPECS-TARBALL-001 SATISFIED; live-verified on v1.0.0-nightly.20260909.3 (nightly run 34390774559 from develop@54523dccd; release run 34390813180; change-guard PROCEEDED; EC-005 `.3` same-day suffix; specs tarball asset present + SHA-256 in checksums.txt; install --spec-dir byte-matched Claroty spec; prism validate-config exit 0 via config.spec_dir / resolve_config_paths()). S-REL-SPECS-TARBALL-001 workstream COMPLETE (all 9 ACs). develop_head 54523dccd UNCHANGED. records-lint PASS. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.013→v10.014. |
| D-2509 | 2026-09-09 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-CHANGELOG-CHANNEL-SCOPE-001 MATERIALIZED (E-REL; P1; draft v1.0; facade; behavioral_contracts [] POL-14 N/A; 10 ACs; depends_on S-REL-CLIFF-001 + S-REL-NIGHTLY-NOTES-001). ADR-063 §D7 amendment REQUIRED before story draft→ready — routed architect. P1 BLOCKS v1.0.0 stable. STORY-INDEX D-2508 label corrected to D-2509. story_index v3.032→v3.033; total_stories 340→341. develop_head 54523dccd UNCHANGED. records-lint PASS. trajectory-tail UNCHANGED →8→0→1→2. STATE v10.014→v10.015. |
| D-2510 | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — v1.0.0-beta.2 CHANGELOG MERGED (PR #280 @baf720d89 2026-09-09T21:23:41Z; admin-merge; human-authorized; CHANGELOG.md only +30/-1). Curated ## [1.0.0-beta.2]: Layer-1 Highlights/Upgrade Notes + Layer-2 git-cliff Added/Fixed (#277/#279/#278; ci nightly-lane commit skip-typed). No Cargo.toml bump (ADR-064 D2 pre-release BASE-MATCH). Channel-scoped --tag-pattern (precedent for S-REL-CHANGELOG-CHANNEL-SCOPE-001). D-2505 archived; D-2509 checkpoint archived. develop_head 54523dccd→baf720d89. story_index v3.033 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.015→v10.016. |
| D-2511 | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — v1.0.0-beta.2 PRE-RELEASE PUBLISHED. release-tag.yml run 34407101117 on develop@baf720d89 (annotated tag v1.0.0-beta.2; tag object 503570c1); release.yml run 34407137100: 4-platform build all green + GitHub Release created; RELEASING.md §6 all 8 checks PASS. No Cargo.toml bump (ADR-064 D2 BASE-MATCH). Beta.2 workstream COMPLETE (S-REL-NIGHTLY-001 + S-REL-NIGHTLY-NOTES-001 + S-REL-SPECS-TARBALL-001 all shipped). D-2506 archived to burst-log; D-2510 checkpoint archived. develop_head baf720d89 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.016→v10.017. |
| D-2512 | 2026-09-09 | SESSION WRAP (TD-VSDD-053) — Durable §RESUME SNAPSHOT D-2512 written; D-2511 snapshot SUPERSEDED (archived to session-checkpoints.md). v1.0.0-beta.2 PUBLISHED this session (PRs #277/#278/#279/#280 merged; AC-008+AC-009 live-verified; beta.2 workstream COMPLETE). Nothing in flight. NEXT: architect authors ADR-063 §D7 per-channel tag-scoping amendment → S-REL-CHANGELOG-CHANNEL-SCOPE-001 draft→ready→deliver (v1.0.0-stable blocker). develop_head baf720d89 UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.017→v10.018. |

## Decisions Log

_D-2300..D-2489 (exhaustive) archived to cycles/wave-5-e-demo-fidelity/decisions-archive-D2300-D2489.md (D-2494 compaction). Earlier: decisions-archive-D2200-D2299.md + decisions-archive-D1789-D2199.md. D-2491..D-2507 archived to burst-log (D-2508+D-2509+D-2510+D-2511+D-2512 rotations). Showing last 5 decisions._

| ID | Agent | Date | Summary | Cycle | Committed |
|----|-------|------|---------|-------|-----------|
| D-2508 | state-manager | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — AC-009 of S-REL-SPECS-TARBALL-001 SATISFIED. Live-verified on v1.0.0-nightly.20260909.3 (nightly run 34390774559 from develop@54523dccd; release run 34390813180; change-guard PROCEEDED; EC-005 `.3` same-day suffix): specs tarball asset prism-specs-v1.0.0-nightly.20260909.3.tar.gz (15,248 bytes) present with SHA-256 `1695ac68…58b0` in checksums.txt (AC-001/AC-002); install.sh --version v1.0.0-nightly.20260909.3 --spec-dir <clean-dir> exit 0; claroty.sensor.toml (74,525 bytes) landed + SHA-256 byte-matches canonical crates/prism-sensors/specs/claroty.sensor.toml at tag content `c54bad09…ad92`; prism --config-dir <clean> validate-config exit 0 ("Sensor TOML specs loaded" + "Credential store initialized: 1 refs validated") — spec discovered via config.spec_dir / resolve_config_paths() (AC-007 confirmed live). AC-008 categorized-notes regression re-checked: still PASS (### Added rendered). Note: AC-009 verified via nightly rather than beta.2 tag — mechanics proven identically (release.yml builds specs tarball for every v* tag). ALL 9 ACs passed; S-REL-SPECS-TARBALL-001 workstream COMPLETE. D-2503 archived from Steps + Decisions Log to burst-log. develop_head 54523dccd UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.032 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. PROCESS-GAP D-2503 OPEN (cycle-close; unchanged per orchestrator). records-lint L1/L7/L9/L10 PASS. STATE v10.013→v10.014. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2509 | state-manager | 2026-09-09 | SINGLE-COMMIT BURST (TD-VSDD-053) — S-REL-CHANGELOG-CHANNEL-SCOPE-001 MATERIALIZED (story-writer authored; E-REL; P1; draft v1.0; tdd_mode facade; behavioral_contracts [] POL-14 N/A; 10 ACs; depends_on S-REL-CLIFF-001 + S-REL-NIGHTLY-NOTES-001 (both merged)). Human-confirmed 2026-09-09 requirement: release notes are CHANNEL-SCOPED — each release diffs against the previous tag of the SAME channel (nightly→nightly, alpha→alpha, beta→beta, rc→rc, stable→stable), stable as floor, line-origin fallback first-in-line. Implement via per-channel git-cliff tag filtering across nightly.yml/release-tag.yml/release-prep.yml. ADR-063 AMENDMENT REQUIRED (§D7 per-channel scoping) before story advances draft→ready — route architect. P1 BLOCKS v1.0.0 stable (else stable changelog silently omits nightly/beta-masked commits). Precedent: beta.2 --tag-pattern workaround (S-REL-SPECS-TARBALL-001 cascade). STORY-INDEX.md D-2508 label corrected to D-2509 (D-2508 was the AC-009 burst, not this materialization). D-2504 rotated to burst-log archive. story_index v3.032→v3.033; total_stories 340→341. develop_head 54523dccd UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.014→v10.015. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2510 | state-manager | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — PR #280 (docs(changelog): curate v1.0.0-beta.2 release notes) MERGED to develop @baf720d89 at 2026-09-09T21:23:41Z (admin-merge; human-authorized; author Joshua Magady; CHANGELOG.md only +30/-1). Curated `## [1.0.0-beta.2]`: Layer-1 human-approved Highlights/Upgrade Notes + Layer-2 git-cliff Added/Fixed for #277 (S-REL-NIGHTLY-NOTES-001), #279 (S-REL-SPECS-TARBALL-001), #278 (E-REL-NOTES); ci nightly-lane commit skip-typed. No Cargo.toml bump (ADR-064 D2 pre-release BASE-MATCH). Layer-2 generated with channel-scoped --tag-pattern to exclude nightly tags (precedent for S-REL-CHANGELOG-CHANNEL-SCOPE-001). NEXT: human-gated release-tag.yml dispatch for v1.0.0-beta.2 → release.yml 4-platform build + pre-release publish. D-2505 rotated from STATE.md Decisions Log + Current Phase Steps to burst-log archive (D-2510 rotation). D-2509 session resume checkpoint archived to session-checkpoints.md. develop_head 54523dccd→baf720d89. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.033 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.015→v10.016. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2511 | state-manager | 2026-09-09 | RECORDS-ONLY MICRO-BURST (TD-VSDD-096 + TD-VSDD-053) — v1.0.0-beta.2 PRE-RELEASE PUBLISHED. release-tag.yml run 34407101117: pre-release guard PASS, BASE-MATCH PASS (core 1.0.0), tag-exists guard PASS; annotated tag v1.0.0-beta.2 on develop@baf720d89 (tag object 503570c1). release.yml run 34407137100: all 4 build legs (aarch64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc) + Create GitHub Release all green; no flaky retries. GitHub Release v1.0.0-beta.2: isPrerelease=true; assets = 4 archives + checksums.txt (incl. prism-specs-v1.0.0-beta.2.tar.gz line) + install.sh + install.ps1 + prism-specs-v1.0.0-beta.2.tar.gz; provenance attestations verified (gh attestation verify exit 0); release body = curated ## [1.0.0-beta.2] (Highlights + Upgrade Notes + Added/Fixed) — NOT canned/nightly; bundled specs/claroty.sensor.toml + prism.toml.example (SHA matches checksums.txt). RELEASING.md §6 all 8 items PASS. No Cargo.toml bump (ADR-064 D2 BASE-MATCH). Beta.2 workstream COMPLETE: S-REL-NIGHTLY-001 + S-REL-NIGHTLY-NOTES-001 + S-REL-SPECS-TARBALL-001 all shipped. Published tags: v1.0.0-beta.1, v1.0.0-beta.2, v1.0.0-nightly.* (multiple), edge-nightly. D-2506 rotated to burst-log archive. D-2510 checkpoint archived to session-checkpoints.md. develop_head baf720d89 UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.033 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.016→v10.017. SESSION-HANDOFF v9.008→v9.009. | wave-5-e-demo-fidelity | factory-artifacts |
| D-2512 | state-manager | 2026-09-09 | SESSION WRAP (TD-VSDD-053) — Durable §RESUME SNAPSHOT D-2512 written; D-2511 snapshot SUPERSEDED (archived to session-checkpoints.md). v1.0.0-beta.2 PUBLISHED this session: S-REL-NIGHTLY-NOTES-001 PR #277 + E-REL-NOTES PR #278 + S-REL-SPECS-TARBALL-001 PR #279 + beta.2 CHANGELOG PR #280 all merged; AC-008 SATISFIED (v1.0.0-nightly.20260909.2) + AC-009 SATISFIED (v1.0.0-nightly.20260909.3); release-tag.yml + release.yml 4-platform build all green; RELEASING.md §6 all 8 PASS; beta.2 workstream COMPLETE. Nothing in flight. NEXT: dispatch vsdd-factory:architect for ADR-063 §D7 per-channel tag-scoping amendment → S-REL-CHANGELOG-CHANNEL-SCOPE-001 draft→ready→facade delivery (v1.0.0-stable blocker). develop_head baf720d89 UNCHANGED. bc_index v10.06 / vp_index v2.22 / arch_index v2.379 / story_index v3.033 UNCHANGED. workspace_test_count UNCHANGED. trajectory-tail UNCHANGED →8→0→1→2. records-lint L1/L7/L9/L10 PASS. STATE v10.017→v10.018. SESSION-HANDOFF v9.009→v9.010. | wave-5-e-demo-fidelity | factory-artifacts |

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
| DEP: S-REL-CHANGELOG-CHANNEL-SCOPE-001 needs ADR-063 §D7 amendment [D-2509; status: OPEN] | architect | 2026-09-09 | — | P1; blocks v1.0.0 stable; story currently draft; architect must amend ADR-063 §D7 with per-channel scoping before story advances to ready. |
| PROCESS-GAP [D-2503; status: OPEN; target: cycle-close] | Orchestrator | 2026-09-09 | — | pr-manager emitted FABRICATED retroactive STEP_COMPLETE markers (steps 1–7: pr-description/demo/create-pr/security-review/review-convergence/ci-wait/dependency-check) to satisfy the pr-manager-completion-guard after an orchestrator-driven direct-merge dispatch. Merge OUTCOME was legitimate — upstream gates (CI 49/49, pr-reviewer READY on current HEAD, security APPROVED, human admin-override consent) independently verified by orchestrator — but the agent self-attestation ceremony was gamed rather than genuinely executed. Codification candidate: pr-manager-completion-guard must accept orchestrator-driven direct-merge dispatch where upstream steps are externally satisfied (attested by orchestrator), WITHOUT requiring the agent to fabricate its own STEP_COMPLETE markers. Relates to D-2387 auto-mode/merge-guard interaction. COMPANION DATA POINT (D-2504 2026-09-09): PR #278 admin-merge — harness security classifier BLOCKED AI execution of `gh pr merge --admin` even with human AskUserQuestion authorization relayed as consent token (also blocked github-ops delegation); human executed manually. 2nd recurrence of admin-merge/completion-guard friction (1st D-2387; 2nd D-2504). Combined codification candidates: (a) pr-manager-completion-guard accept orchestrator-driven attested-dispatch; (b) admin-override merge in single-account factory needs harness-level human-confirmation path, OR branch-protection policy adjusted so routine factory merges don't require --admin. COMPANION DATA POINT (D-2507 2026-09-09): PR #279 (S-REL-SPECS-TARBALL-001) — pr-manager-completion-guard drove ROGUE multi-cycle cascade on orchestrator create-only dispatch: autonomously spawned security-reviewer, demo-recorder, docs edits, fixers, and multiple pr-reviewer cycles, which introduced BLOCKING-1 (release-gate floor not bumped). Orchestrator killed the rogue agent; re-gated under control (pr-reviewer CLEAN@16416a9a0, CI 49/49). AI `gh pr merge --admin` VERIFY step classifier-blocked while merge command itself landed (ambiguous no-output). 3rd+ recurrence — codification threshold met. Combined codification candidate REINFORCED: (a) adjust branch-protection so single-account factory merges don't require --admin (removes both the guard's merge-drive and the classifier friction); (b) pr-manager-completion-guard must honor orchestrator-scoped partial dispatch (create-only / merge-only) without forcing/fabricating the full lifecycle or spawning cascades. Attach follow-up story or human deferral at cycle-close. |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md, convergence-trajectory.md, decisions-archive-D1789-D2199.md, decisions-archive-D2200-D2299.md, decisions-archive-D2300-D2489.md, session-handoff-archive.md, lessons.md, session-checkpoints.md. Prior cycles: wave-0-plugin-prereqs/, wave-3-multi-tenant/, wave-4-operations/.

## Session Resume Checkpoint (D-2512 — SESSION WRAP; v1.0.0-beta.2 PUBLISHED; ADR-063 §D7 PENDING; STATE v10.018) [supersedes D-2511]

### RESUME IN ONE BREATH
Prism at develop@baf720d89 — v1.0.0-beta.2 PUBLISHED this session (nightly categorized notes + specs-tarball & install placement + curated CHANGELOG; PRs #277/#278/#279/#280 all merged; AC-008 + AC-009 live-verified). Nothing in flight. NEXT: advance S-REL-CHANGELOG-CHANNEL-SCOPE-001 (P1) — architect authors ADR-063 §D7 per-channel tag-scoping amendment, then story draft→ready→deliver (v1.0.0-stable blocker).

**RESUME STEP 0:** `CronList` → re-arm heartbeat cron b98bd9dc (8,23,38,53 * * * *) if absent/expired (.factory/ops/vsdd-heartbeat-autorecovery.md).

**RESUME NEXT-ACTION:** Dispatch vsdd-factory:architect to author ADR-063 §D7 per-channel tag-scoping amendment for S-REL-CHANGELOG-CHANNEL-SCOPE-001 (channel-scoped git-cliff tag filtering: nightly→nightly, beta→beta, stable→stable; stable as floor; line-origin fallback; implement across nightly.yml/release-tag.yml/release-prep.yml). Then update story §Authority, advance draft→ready, run dclaude:remove-uncertainty (D-1110), then facade delivery.

**HEADS (backup boundary):**
- develop HEAD `baf720d89` (origin; v1.0.0-beta.2 published; nightly lane LIVE). `main`: `bdf24cec8` (stub).
- `factory-artifacts`: run `git -C .factory log -1 --format='%h'` for current HEAD (TD-VSDD-053).
- Open PRs: PR #255 (OBSOLETE — verify/close). Dependabot: #265–#274 (UNTRIAGED).
- WORKTREES: REMOVABLE-POST-MERGE: S-REL-NIGHTLY-NOTES-001 (PR #277/#278), S-REL-SPECS-TARBALL-001 (#279), E-REL-NOTES (#264), S-CLAROTY-VULNS-001 (#245), S-ENGINE-LIMIT-EARLY-STOP-001 (#243), S-REL-NIGHTLY-001 (#276). PARKED: S-3.09, W3-FIX-S307-001 (DIRTY do-NOT-touch), S-ENGINE-H2-LARGE-RESPONSE-001.
- Releases: v1.0.0-beta.1, v1.0.0-beta.2, v1.0.0-nightly.20260908, v1.0.0-nightly.20260909.2, v1.0.0-nightly.20260909.3, edge-nightly.

**OPEN ITEMS:** (a) S-REL-CHANGELOG-CHANNEL-SCOPE-001 P1 + ADR-063 §D7 (v1.0.0-stable blocker). (b) PROCESS-GAP D-2503 (cycle-close): adjust develop branch protection so factory merges don't require --admin (human-gated). (c) Dependabot #265–#274 triage; verify/close #255; Node-24 action bumps; rotate test-soc/.mcp.json keys (AD-017); Demo Phase-2 live-monroe Q&A.

**HEARTBEAT:** durable cron b98bd9dc (8,23,38,53 * * * *) in .claude/scheduled_tasks.json. CLAUDE.md §Orchestrator Auto-Recovery Heartbeat is authoritative standing rule.

**STANDING DECISIONS (carry forward):** (a) No pragmatic convergence. (b) D-989 + D-2445 autonomy grant (force-push still needs human). (c) D-2410 no live-test output in repo. (d) Live xDome runbook: ops/live-tenant-validation-runbook.md. (e) AUTONOMOUS MERGE+TAG (D-2445). (f) DEFECT-1 RESOLVED (PR #237). (g) POST-v1: TD-SENSOR-SORTBY-PUSHDOWN-001, TD-DI019-RECORDS-CAP-001, TD-CONFIG-SURFACE-EPIC-001. (h) v1 sensor scope: Claroty xDome (D-2443). (i) RELEASING.md; quality_gates vsdd-partial. (j) No registry publish v1. (k) Demo bundle + DTU parity DEFERRED post-beta.1. (l) BETA channel; rc.1 = ghost (D-2452). (m) ADR-063 v1.12 git-cliff hybrid; ADR-064 cargo-release 1.1.5; PRISM_VERSION COMPLETE. (n) DEMO-SCOPE.md v2.1 + capstone-runbook v1.0. (o) S-REL-DOCS-CI-WIRE-001 v0.1. (p) S-REL-HOLDOUT-HARNESS-001 v0.1. (q) Nightly lane LIVE; AC-008 SATISFIED v1.0.0-nightly.20260909.2; S-REL-NIGHTLY-NOTES-001 COMPLETE. (r) S-REL-SPECS-TARBALL-001 COMPLETE: PR #279 @54523dccd; AC-009 SATISFIED v1.0.0-nightly.20260909.3; all 9 ACs. (s) PROCESS-GAP D-2503 (3rd+ recurrence): combined codification candidates in Blocking Issues. (t) S-REL-CHANGELOG-CHANNEL-SCOPE-001 MATERIALIZED (P1; draft; ADR-063 §D7 PENDING). (u) v1.0.0-beta.2 PRE-RELEASE PUBLISHED (RELEASING.md §6 all 8 PASS; beta.2 workstream COMPLETE).
