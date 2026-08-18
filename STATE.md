---
document_type: pipeline-state
level: ops
version: "8.776"
producer: state-manager
timestamp: 2026-08-19T00:05:00Z
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
develop_head: "69d821be"
# NOTE: D-2202 — develop_head corrected 791b68c3→69d821be (fast-forward COMPLETE; develop == origin/develop == 69d821be confirmed). OCSF-mapping schema-validation COMPLETE (ADR-058 v2.4; 12 KF + CODE DEFECT §class_selector.rs); ARCH-INDEX v2.303→v2.304. STATE v8.732→v8.733. D-2201 NOTE archived.
# NOTE: D-2183 — develop advanced 3197e27a9→791b68c3: PR #238 chore(gitignore) AD-017 live-evidence SQUASH-MERGED to develop 2026-08-15. D-2171 NOTE archived.
# NOTE: D-2109 — develop advanced ef996a4c0→5d1a30ac7: PR #236 fix/claroty-live-api-fidelity SQUASH-MERGED; BC-5.39.001 PR-LEVEL 3-CLEAN CONVERGED (passes 8/9/10 on frozen 386df43c5); HS-014 PASS; worktrees CLAROTY-LIVE + FINDING-R removed. D-2102 NOTE archived.
bc_index_version: "9.37"
# NOTE: D-2243 — BC-INDEX v9.36→v9.37: P43/44/45 fix-burst — BC-2.16.003 pin v1.14→v1.15 (F-P44-OBS-001 raw_extensions ColumnDescriptor col_type=Json/nullable=true in §Interpretation A Tier-2 + EC-016-013-027). Counts UNCHANGED: active 252/draft 4/total 269. D-2242 NOTE archived.
# NOTE: D-2242 — BC-INDEX v9.35→v9.36: P40/41/42 fix-burst — BC-2.16.003 pin v1.13→v1.14 (§Interpretation A Tier-1/Tier-2 alignment; EC-016-013-027 NEW). Counts UNCHANGED: active 252/draft 4/total 269. D-2233 NOTE archived.
# NOTE: D-2233 — BC-INDEX v9.34→v9.35: pass-30 fix-burst (BC-2.16.003 pin v1.12→v1.13). BC-2.16.002 UNCHANGED v2.28. Counts UNCHANGED: active 252/draft 4/total 269. D-2232 NOTE archived.
# NOTE: D-2232 — BC-INDEX v9.33→v9.34: pass-28 fix-burst (BC-2.16.002 pin v2.27→v2.28). Counts UNCHANGED: active 252/draft 4/total 269. D-2229 NOTE archived.
# NOTE: D-2229 — BC-INDEX v9.32→v9.33: pass-25 fix-burst (BC-2.16.003 pin v1.11→v1.12). Counts UNCHANGED: active 252/draft 4/total 269. D-2228 NOTE archived.
# NOTE: D-2228 — BC-INDEX v9.31→v9.32: pass-24 fix-burst (BC-2.16.003 pin v1.10→v1.11). Counts UNCHANGED: active 252/draft 4/total 269. D-2221 NOTE archived.
# NOTE: D-2221 — BC-INDEX v9.30→v9.31: pass-17 fix-burst (BC-2.16.002 pin v2.11→v2.27 ascending→descending reorder). Counts UNCHANGED: active 252/draft 4/total 269. D-2220 NOTE archived.
# NOTE: D-2220 — BC-INDEX v9.29→v9.30: pass-16 fix-burst (BC-2.16.003 pin v1.9→v1.10). Counts UNCHANGED: active 252/draft 4/total 269. D-2209 NOTE archived.
# NOTE: D-2186 — BC-INDEX v9.14→v9.15: S-CLAROTY-AUDITLOG-TIMEBOX-001 LOCAL adversary pass-1 fix-burst — BC-2.01.013 pin v1.20→v1.21; BC-2.16.013 pin v1.38→v1.39. Counts UNCHANGED: active 252/draft 4/total 269. D-2178 NOTE archived.
# NOTE: D-2172 — BC-INDEX v9.12→v9.13: PR #237 MERGED (POL-14 auto-promotion) — BC-2.16.014 lifecycle_status draft→active (active 251→252; draft 5→4). Remaining 4 draft: BC-2.01.018+BC-2.06.011+BC-2.21.001+BC-2.02.014. D-2159 NOTE archived.
# NOTE: D-2082 — BC-INDEX v8.92→v8.93: F-WASE-P72-HIGH-002 RESOLVED. Counts: active 251/draft 5/total 269. D-2080 NOTE archived.
vp_index_version: "2.22"
# NOTE: D-2054 — VP-INDEX v2.21→v2.22: VP-157 and VP-158 promoted to active (v1.1); ADR-056 v0.5 and ADR-057 v0.4 rows added. D-2053 NOTE archived.
story_index_version: "2.858"
# NOTE: D-2243 — STORY-INDEX v2.857→v2.858: P43/44/45 fix-burst — ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34 (sibling-pin + input-hash update). total_stories 302 UNCHANGED. D-2242 NOTE archived.
# NOTE: D-2242 — STORY-INDEX v2.856→v2.857: P40/41/42 fix-burst — ADR-058 v2.19→v2.20; BC-2.16.003 v1.13→v1.14; ROUTING-001 v1.35→v1.36; COERCION-001 v1.32→v1.33. total_stories 302 UNCHANGED. D-2241 NOTE archived.
# NOTE: D-2241 — STORY-INDEX v2.855→v2.856: TD-VSDD-096 records-only micro-burst — F-P39-LOW-001 §Authority BC-2.16.002 date parenthetical corrected: ROUTING-001 v1.34→v1.35. COERCION-001/ADR-058/BC-INDEX/ARCH-INDEX UNCHANGED. total_stories 302 UNCHANGED. D-2240 NOTE archived.
# NOTE: D-2240 — STORY-INDEX v2.854→v2.855: TD-VSDD-096 records-only micro-burst — F-P36-LOW-001 §Changelog v1.12 volatile-line-cite token neutralized: ROUTING-001 v1.33→v1.34. COERCION-001/ADR-058/BC-INDEX/ARCH-INDEX UNCHANGED. total_stories 302 UNCHANGED. D-2239 NOTE archived.
# NOTE: D-2239 — STORY-INDEX v2.853→v2.854: F-P34 fix-burst — ADR-058 v2.18→v2.19; ROUTING-001 v1.32→v1.33; COERCION-001 v1.31→v1.32. total_stories 302 UNCHANGED. D-2238 NOTE archived.
# NOTE: D-2238 — STORY-INDEX v2.852→v2.853: F-P33-MED-001 fix-burst — ADR-058 v2.17→v2.18 (§D1 param-threading explicit; §I1 two-step form; AC-012/RG-024 DISCHARGED). ROUTING-001 v1.31→v1.32; COERCION-001 v1.30→v1.31. total_stories 302 UNCHANGED. D-2235 NOTE archived.
# NOTE: D-2235 — STORY-INDEX v2.851→v2.852: pass-32 fix-burst — ADR-058 v2.16→v2.17; ROUTING-001 v1.30→v1.31; COERCION-001 v1.29→v1.30. total_stories 302 UNCHANGED. D-2234 NOTE archived.
# NOTE: D-2233 — STORY-INDEX v2.849→v2.850: pass-30 fix-burst — BC-2.16.003 pin v1.13 propagated to both stories. total_stories 302 UNCHANGED. D-2232 NOTE archived.
# NOTE: D-2232 — STORY-INDEX v2.848→v2.849: pass-28 fix-burst — BC-2.16.002 pin v2.28 propagated; VPs cells VP-017/VP-016 populated. total_stories 302 UNCHANGED. D-2231 NOTE archived.
# NOTE: D-2203 — STORY-INDEX v2.821→v2.822: OCSF-correctness claroty story-decomposition burst — 3 NEW stub stories; total_stories 299→302. D-2200 NOTE archived.
# NOTE: D-2200 — STORY-INDEX v2.820→v2.821: S-CLAROTY-AUDITLOG-TIMEBOX-001 ready→merged (PR #239 @69d821be 2026-08-16T22:51Z). total_stories 299 UNCHANGED. D-2199 NOTE archived.
arch_index_version: "2.321"
# NOTE: D-2243 — ARCH-INDEX v2.320→v2.321: P43/44/45 fix-burst — ADR-058 pin v2.20→v2.21 (§I1 helper canonical home prism-spec-engine::column_mapping; §B2 item 1 None-columns no individual Arrow field; §G raw_extensions col_type/nullable; §Status heading corrected). D-2242 NOTE archived.
# NOTE: D-2242 — ARCH-INDEX v2.319→v2.320: P40/41/42 fix-burst — ADR-058 pin v2.19→v2.20 (§G Tier-1/Tier-2 describe model; §I5 intro 7→12; §I5(d) device-arm; §G finding_info.uid example). D-2239 NOTE archived.
# NOTE: D-2239 — ARCH-INDEX v2.318→v2.319: F-P34 fix-burst — ADR-058 pin v2.18→v2.19 (F-P34-LOW-002 §Status heading lag; F-P34-OBS-001 ocsf_field_to_arrow_name standardized). D-2238 NOTE archived.
# NOTE: D-2238 — ARCH-INDEX v2.317→v2.318: F-P33-MED-001 fix-burst — ADR-058 pin v2.17→v2.18. D-2235 NOTE archived.
# NOTE: D-2235 — ARCH-INDEX v2.316→v2.317: pass-32 fix-burst — ADR-058 v2.16→v2.17. D-2233 NOTE archived.
# NOTE: D-2221 — ARCH-INDEX v2.303→v2.315: Wave-A spec-evolution FB69..FB80 + DEFECT-ADAPTER-TLS-XDOME-LIVE-001 bursts. D-2220 NOTE archived.
workspace_test_count: 5743
# NOTE: D-2184 — workspace_test_count 5733→5741: S-CLAROTY-AUDITLOG-TIMEBOX-001 TDD-GREEN — RG-001..RG-005 pass. D-2161 NOTE archived.
# NOTE: D-2160 — 5731→5732: RG-020 added; just check GREEN 5732 at feature HEAD 21df2f6d4. D-2159 NOTE archived.
# NOTE: D-2155 — 5724→5730: pass-38 fix-burst CODE commit a5b61b35b (RG-017+RG-018; just check 5730 green). D-2125 NOTE archived.
# NOTE: D-2121 — 5703→5722: pass-8 CODE commit 490b5c831 (RG-008 fix; just check 5722 green). D-2109 NOTE archived.
vsdd_factory_version: "1.0.0-rc.22"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-2243 P43/44/45 fix-burst COMPLETE — F-P43-HIGH-001 §I1 ocsf_field_to_arrow_name canonical home relocated to prism-spec-engine::column_mapping; F-P43-MED-001 §Red Gate count 24→25; F-P44-OBS-001 raw_extensions ColumnDescriptor col_type=Json/nullable=true; F-P45-MED-001 phantom ADR-023 §D3 cite corrected to dependency-graph.md §Dependency Rules Rule 2. ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34. ARCH-INDEX v2.320→v2.321. BC-INDEX v9.36→v9.37. STORY-INDEX v2.857→v2.858. NEW FROZEN PERIMETER: ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34. RESUME: adversary SPEC passes 46/47/48 (parallel, maximally-skeptical) on new frozen HEAD (streak 0/3 re-gated). trajectory-tail →2→2→1→1 (p42→p43→p44→p45)"
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── PARKED WORKTREES ──
worktree_status: "Main worktree on develop origin/develop @69d821be (PR #239 squash-merged 2026-08-16T22:51Z). .worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001 @8ae0b5d8 PENDING teardown. PARKED (2): S-3.09 @43c41389d KEEP-PARKED; W3-FIX-S307-001 @fcab8717c PARKED-DIRTY do-NOT-touch."

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
pre_compact_snapshot: "See cycles/wave-5-e-demo-fidelity/: decisions-archive-D1789-D2199.md + session-handoff-archive.md + drift-items-open.md. D-2237 compaction (2026-08-18): decisions D-1789..D-2199 (exhaustive) + Phase Progress historical rows + Convergence Status + Concurrent Cycles + Current Phase Steps D-2059..D-2159 (exhaustive) + resolved/closed drift+blocking items + all open Drift Items archived. Last preserved decision in STATE.md: D-2200. Prior: D-1794 compaction (2026-07-16): frontmatter chains trimmed; D-1785..D-1788 (exhaustive) + archive stubs archived. Git history on factory-artifacts preserves all content."
pre_compact_snapshot_at: "2026-08-18"
---

<!-- STATE.md SIZE BUDGET: 274 lines (wc-l) | target 200 lines (soft) | hard-cap 500 | margin from soft-target: +74 | margin from actual: -226 | compact eligible: safe_to_compact: true -->

# VSDD Pipeline State — Prism

## Project Metadata

| Field | Value |
|-------|-------|
| **Product** | Prism |
| **Language** | Rust |
| **Mode** | brownfield |
| **Deploy** | per-analyst stdio (MCP) |
| **Started** | 2026-04-13 |
| **Last Updated** | 2026-08-18 D-2243 P43/44/45 SPEC fix-burst COMPLETE — F-P43-HIGH-001 §I1 ocsf_field_to_arrow_name canonical home prism-spec-engine::column_mapping; F-P44-OBS-001 raw_extensions ColumnDescriptor col_type=Json/nullable=true; ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34; ARCH-INDEX v2.320→v2.321; BC-INDEX v9.36→v9.37; STORY-INDEX v2.857→v2.858 trajectory-tail →2→2→1→1 (p42→p43→p44→p45) |

## Active Objective (North Star)

**NORTH STAR: Multi-client SOC-analyst live demo** — multiple DTU clients, per-client data, prism MCP wired into Claude (stdio), deterministic scenario progression. Full detail: SESSION-HANDOFF.md §ACTIVE OBJECTIVE + `.factory/objectives/DEMO-SCOPE.md`. Task ledger: `.factory/objectives/multi-client-soc-demo-tasks.md`.

## Phase Progress

| Phase | Status | Started | Completed | Gate | Finding Progression |
|-------|--------|---------|-----------|------|---------------------|
| 0: Codebase Ingestion | passed | 2026-04-13 | 2026-04-14 | human-approved | converged |
| 1a: Product Brief + Domain Spec | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1b: PRD + Behavioral Contracts | passed | 2026-04-14 | 2026-04-15 | human-approved | converged |
| 1c: Architecture + VPs | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 1d: Adversarial Spec Review | passed | 2026-04-15 | 2026-04-15 | 33-pass convergence | 13→1 converged |
| 2: Story Decomposition | passed | 2026-04-15 | 2026-04-16 | human-approved | converged |
| 3: Waves 0–3 + Plugin Prereqs | COMPLETE | 2026-04-21 | 2026-05-27 | wave gates converged | PRs #1–161; 3711 tests; develop@af79f160 |
| 3: Post-Wave-3 DTU+Demo+PRs #162–239 | COMPLETE | 2026-05-27 | 2026-08-16 | all MERGED develop@69d821be | PR #239 squash-merged 2026-08-16T22:51Z; 5743 tests; workspace CI green |
| Wave-A spec-evolution LOCAL CASCADE | CONVERGED | 2026-07-23 | 2026-07-23 | BC-5.39.001 strict 3/3 | 47 passes / 36 fix-bursts. CLEAN(strict): 19/24/30/33/36/39/41/42/45/46/47. |
| DEFECT-ADAPTER-TLS-XDOME-LIVE-001 | FULLY VALIDATED | 2026-08-15 | 2026-08-15 | D-2166 AC-LIVE-001 SATISFIED; HS-008..011 CONSUMED | PR #237 squash-merged develop@3197e27a9 2026-08-15 |
| S-CLAROTY-AUDITLOG-TIMEBOX-001 | MERGED | 2026-08-16 | 2026-08-16 | PR #239 develop@69d821be 2026-08-16T22:51Z | LOCAL 9-pass 3-CLEAN + HOLDOUT PASS 4/4 + LIVE xDome PASS; PR-LEVEL 3-CLEAN on 8ae0b5d8 |
| OCSF-correctness claroty SPEC adversary cascade | IN PROGRESS | 2026-08-16 | — | pass-45 COMPLETE (D-2243); streak 0/3 re-gated new perimeter | p29(CLEAN)[1/3]→p30(1MED)→p31(1LOW)→p32(2MED)→p33(1MED)→p34(4 findings)→p35(CLEAN 1/3)→p36(1LOW)→p37(CLEAN 1/3)→p38(CLEAN 2/3)→p39(1LOW)→p40(1HIGH)→p41(2MED)→p42(1HIGH+1LOW)→p43(2[1H+1M])→p44(1OBS)→p45(1MED). F-P43/44/45 FIXED-D2243. NEW FROZEN: ADR-058 v2.21/BC-2.16.003 v1.15/BC-2.16.002 v2.28/ROUTING-001 v1.37/COERCION-001 v1.34 |
| D-2238 F-P33-MED-001 fix burst | COMPLETE | 2026-08-18 | 2026-08-18 | D-2238 (exhaustive) | ADR-058 v2.17→v2.18; ROUTING-001 v1.31→v1.32; COERCION-001 v1.30→v1.31 trajectory-tail →1→1→2→1 |
| D-2239 F-P34 fix burst | COMPLETE | 2026-08-18 | 2026-08-18 | D-2239 (exhaustive) | ADR-058 v2.18→v2.19; ROUTING-001 v1.32→v1.33; COERCION-001 v1.31→v1.32 trajectory-tail →1→2→1→4 |
| D-2242 P40/41/42 SPEC fix burst | COMPLETE | 2026-08-18 | 2026-08-18 | D-2242 (exhaustive) | ADR-058 v2.19→v2.20; BC-2.16.003 v1.13→v1.14; ROUTING-001 v1.35→v1.36; COERCION-001 v1.32→v1.33 trajectory-tail →1→1→2→2 (p39→p40→p41→p42) |
| D-2243 P43/44/45 SPEC fix burst | COMPLETE | 2026-08-18 | 2026-08-18 | D-2243 (exhaustive) | ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34 trajectory-tail →2→2→1→1 (p42→p43→p44→p45) |

_Historical Phase Progress rows (Wave-A spec-evolution passes 28–47, individual story rows PRs #162–#235) archived to cycles/wave-5-e-demo-fidelity/burst-log.md (D-2237 compaction)._

## Convergence Status

| Metric | Value |
|--------|-------|
| BC-5.39.001 streak | 0/3 — re-gated on new frozen HEAD post D-2243 fix-burst (perimeter changed) |
| Active cascade | OCSF-correctness claroty SPEC adversary |
| Pass count | 45 complete; pass-46 pending |
| Last CLEAN(strict) | pass-38 (2/3 reached; p39–p45 all had findings) |
| Finding trajectory | trajectory-tail →2→2→1→1 (p42→p43→p44→p45) |
| Frozen perimeter | ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34 |

## Concurrent Cycles

_No concurrent cycles in progress. Current cycle: wave-5-e-demo-fidelity._

## Current Phase Steps

| Step | Agent | Status | Output |
|------|-------|--------|--------|
| _D-735..D-2237 (exhaustive)_ | — | archived | cycles/wave-5-e-demo-fidelity/burst-log.md (D-1794 + D-2237 compactions) |
| D-2239 | state-manager | 2026-08-18 | F-P34 fix-burst COMPLETE — ADR-058 v2.18→v2.19; ROUTING-001 v1.32→v1.33; COERCION-001 v1.31→v1.32; ARCH-INDEX v2.318→v2.319; STORY-INDEX v2.853→v2.854. STATE v8.769→v8.770. |
| D-2240 | state-manager | 2026-08-18 | TD-VSDD-096 pass-36 records-only micro-burst COMPLETE — F-P36-LOW-001 §Changelog v1.12 volatile-line-cite token neutralized: ROUTING-001 v1.33→v1.34. Anti-whack-a-mole sweep ADR-058+COERCION-001: CLEAR. STORY-INDEX v2.854→v2.855. ZERO content/mechanism changes. STATE v8.772→v8.773. |
| D-2241 | state-manager | 2026-08-18 | TD-VSDD-096 pass-39 records-only micro-burst COMPLETE — F-P39-LOW-001 §Authority BC-2.16.002 date parenthetical corrected: ROUTING-001 v1.34→v1.35. Comprehensive perimeter records-hygiene audit ADR-058+COERCION-001: CLEAR. STORY-INDEX v2.855→v2.856. ZERO content/mechanism changes. STATE v8.773→v8.774. |
| D-2242 | state-manager | 2026-08-18 | P40/41/42 SPEC fix-burst COMPLETE — ADR-058 v2.19→v2.20 (§G Tier-1/Tier-2 describe model; §I5 intro 7→12; §I5(d) device-arm; §G finding_info.uid example). BC-2.16.003 v1.13→v1.14 (§Interpretation A alignment; EC-016-013-027 NEW). ROUTING-001 v1.35→v1.36 (AC-006/AC-007 rewritten; RG-025 NEW; density 25/12=2.08). COERCION-001 v1.32→v1.33 (sibling pin). ARCH-INDEX v2.319→v2.320. BC-INDEX v9.35→v9.36. STORY-INDEX v2.856→v2.857. STATE v8.774→v8.775. |
| D-2243 | state-manager | 2026-08-18 | P43/44/45 SPEC fix-burst COMPLETE — F-P43-HIGH-001 §I1 ocsf_field_to_arrow_name canonical home relocated to prism-spec-engine::column_mapping; F-P43-MED-001 §Red Gate count 24→25; F-P44-OBS-001 raw_extensions ColumnDescriptor col_type=Json/nullable=true; F-P45-MED-001 phantom ADR-023 §D3 corrected to dependency-graph.md §Dependency Rules Rule 2. ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34. ARCH-INDEX v2.320→v2.321. BC-INDEX v9.36→v9.37. STORY-INDEX v2.857→v2.858. STATE v8.775→v8.776. |

## Decisions Log

_D-1789..D-2199 (exhaustive) archived to `cycles/wave-5-e-demo-fidelity/decisions-archive-D1789-D2199.md` (D-2237 compaction). Prior archives: D-700..D-1788 (exhaustive) in earlier archive files._

| ID | Author | Date | Decision | Cycle | Updated |
|----|--------|------|----------|-------|---------|
| D-2200 | state-manager | 2026-08-16 | **S-CLAROTY-AUDITLOG-TIMEBOX-001 story status ready→merged (squash-merged PR #239 @69d821be 2026-08-16T22:51Z). POL-14 auto-promotion: BC-2.01.013 + BC-2.16.013 both lifecycle_status already active — no promotion; active_contracts 252/draft_contracts 4/total 269 UNCHANGED. workspace_test_count 5743 UNCHANGED. total_stories 299 UNCHANGED. STATE v8.730→v8.731.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2201 | state-manager | 2026-08-16 | **OCSF-mapping schema validation COMPLETE — ADR-058 v2.3→v2.4 (§K mapping correctness sweep: KF-05 audit_logs.id→activity_uid drop-to-raw_extensions; KF-06 devices.device_type→device.type_label vendor-extend). ARCH-INDEX v2.302→v2.303 (ADR-058 row). CODE DEFECT registered: class_selector.rs returns wrong class for claroty device types → OCSF correctness stories required (ROUTING-001 + COERCION-001). STATE v8.731→v8.732.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2202 | state-manager | 2026-08-16 | **develop_head corrected 791b68c3→69d821be (fast-forward COMPLETE; develop == origin/develop == 69d821be confirmed). ARCH-INDEX v2.303→v2.304 (develop_head correction). STATE v8.732→v8.733.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2203 | state-manager | 2026-08-16 | **OCSF-correctness claroty story-decomposition burst — S-ADR058-OCSF-COERCION-001 v1.1→v1.2; S-ADR058-OCSF-ROUTING-001 v1.2→v1.3 (AC-005 rewrite; AC-009/AC-010 NEW; 15 RGTs / density 1.5); 3 NEW stub stories registered. total_stories 299→302. BC-2.16.003 pin v1.4→v1.5. BC-INDEX v9.23→v9.24. STORY-INDEX v2.821→v2.822. ARCH-INDEX v2.304→v2.305. STATE v8.733→v8.734.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2204 | state-manager | 2026-08-16 | **OCSF-correctness claroty CONSISTENCY FIX-BURST — S-ADR058-OCSF-ROUTING-001 v1.3→v1.4; S-ADR058-OCSF-COERCION-001 v1.2→v1.3; 3 S-OCSF-FIDELITY stubs v0.1→v0.2. STORY-INDEX v2.822→v2.823. STATE v8.734→v8.735.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2205 | state-manager | 2026-08-16 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-1 fix-burst. ADR-058 v2.5→v2.6; BC-2.16.002 v2.26→v2.27; BC-2.16.003 v1.5→v1.6; ROUTING-001 v1.4→v1.5; COERCION-001 v1.3→v1.4; BC-INDEX v9.25→v9.26; STORY-INDEX v2.823→v2.824. STATE v8.735→v8.736.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2206 | state-manager | 2026-08-16 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-2 fix-burst. ADR-058 UNCHANGED v2.6; BC-2.16.003 v1.6→v1.7; ROUTING-001 v1.5→v1.6; COERCION-001 v1.4→v1.5; BC-INDEX v9.26→v9.27; STORY-INDEX v2.824→v2.825. STATE v8.736→v8.737.** | wave-5-e-demo-fidelity | 2026-08-16 |
| D-2207 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-3 fix-burst. ROUTING-001 v1.6→v1.7; COERCION-001 v1.5→v1.6; STORY-INDEX v2.825→v2.826. STATE v8.737→v8.738.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2208 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-4 fix-burst. BC-2.16.003 v1.7→v1.8; ROUTING-001 v1.7→v1.8; COERCION-001 v1.6→v1.7; BC-INDEX v9.27→v9.28; STORY-INDEX v2.826→v2.827. STATE v8.738→v8.739.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2209 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-5 fix-burst. BC-2.16.003 v1.8→v1.9; ROUTING-001 v1.8→v1.9; COERCION-001 v1.7→v1.8; BC-INDEX v9.28→v9.29; STORY-INDEX v2.827→v2.828. STATE v8.739→v8.740.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2210 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-6 fix-burst. ROUTING-001 v1.9→v1.10; COERCION-001 v1.8→v1.9; STORY-INDEX v2.828→v2.829. STATE v8.740→v8.741.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2211 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-7 fix-burst. ROUTING-001 v1.10→v1.11; COERCION-001 v1.9→v1.10; STORY-INDEX v2.829→v2.830. STATE v8.741→v8.742.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2212 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-8 CLEAN(strict)=YES — streak 1/3. FROZEN PERIMETER POST-PASS-8: ADR-058 v2.6 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.11 / COERCION-001 v1.10. STATE v8.742→v8.743.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2213 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-9 CLEAN(strict)=YES — streak 2/3. FROZEN PERIMETER POST-PASS-9: ADR-058 v2.6 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.11 / COERCION-001 v1.10. STATE v8.743→v8.744.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2214 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-10 fix-burst. ROUTING-001 v1.13→v1.14; COERCION-001 v1.12→v1.13; STORY-INDEX v2.832→v2.833. BC-5.39.001 streak RESET 2/3→0/3. STATE v8.744→v8.745.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2215 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-11 CLEAN(strict)=YES — streak 1/3. FROZEN: ADR-058 v2.7 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.14 / COERCION-001 v1.13. STATE v8.745→v8.746.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2216 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-12 fix-burst. ADR-058 v2.12→v2.13; ROUTING-001 v1.15→v1.16; COERCION-001 v1.14→v1.15; STORY-INDEX v2.834→v2.835. BC-5.39.001 streak RESET 1/3→0/3. STATE v8.746→v8.747.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2217 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC passes 13+14 CLEAN(strict)=YES — streak 2/3 after pass-14. FROZEN: ADR-058 v2.13 / BC-2.16.003 v1.9 / BC-2.16.002 v2.27 / ROUTING-001 v1.16 / COERCION-001 v1.15. STATE v8.747→v8.748.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2218 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053; TD-VSDD-096) — OCSF-correctness claroty adversary SPEC pass-15 records-only micro-burst. ROUTING-001 v1.16→v1.17; STORY-INDEX v2.835→v2.836. BC-5.39.001 streak RESET 2/3→0/3. STATE v8.748→v8.749.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2219 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-16 fix-burst. BC-2.16.003 v1.9→v1.10; ROUTING-001 v1.17→v1.18; COERCION-001 v1.15→v1.16; BC-INDEX v9.29→v9.30; STORY-INDEX v2.836→v2.837. STATE v8.749→v8.750.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2220 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-17 fix-burst. BC-2.16.002 pin promoted v2.11→v2.27; COERCION-001 v1.16→v1.17; BC-INDEX v9.30→v9.31; STORY-INDEX v2.837→v2.838. STATE v8.750→v8.751.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2221 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-18 fix-burst. ROUTING-001 v1.18→v1.19; COERCION-001 v1.17→v1.18; ARCH-INDEX v2.303→v2.315; STORY-INDEX v2.838→v2.839. STATE v8.751→v8.752.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2222 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-19 CLEAN(strict)=YES — streak 1/3. FROZEN: ADR-058 v2.13 / BC-2.16.003 v1.10 / BC-2.16.002 v2.27 / ROUTING-001 v1.19 / COERCION-001 v1.18. STATE v8.752→v8.753.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2223 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-20 CLEAN(strict)=YES — streak 2/3. FROZEN: ADR-058 v2.13 / BC-2.16.003 v1.10 / BC-2.16.002 v2.27 / ROUTING-001 v1.19 / COERCION-001 v1.18. STATE v8.753→v8.754.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2224 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-21 fix-burst. ROUTING-001 v1.21→v1.22; COERCION-001 v1.20→v1.21; STORY-INDEX v2.841→v2.842. BC-5.39.001 streak RESET 2/3→0/3. STATE v8.754→v8.755.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2225 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-22 fix-burst. ADR-058 v2.13→v2.14; ROUTING-001 v1.22→v1.23; COERCION-001 v1.21→v1.22; STORY-INDEX v2.842→v2.843. STATE v8.755→v8.756.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2226 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-23 records-only micro-burst. ROUTING-001 v1.23→v1.24; COERCION-001 v1.22→v1.23; STORY-INDEX v2.843→v2.844. STATE v8.756→v8.757.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2227 | state-manager | 2026-08-17 | **OCSF-correctness claroty adversary SPEC pass-24 fix-burst. BC-2.16.003 v1.10→v1.11; COERCION-001 v1.23→v1.24; ROUTING-001 v1.24→v1.25; BC-INDEX v9.31→v9.32; STORY-INDEX v2.844→v2.845. STATE v8.757→v8.758.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2228 | state-manager | 2026-08-17 | **DRIFT-DTU-PARITY-STALE-001 REGISTERED (MEDIUM, OPEN). Scope DTU-PARITY OUT of cascade passes going forward; verify only §Authority ADR-058 cite for SAC-2 link validity. STATE v8.758→v8.759.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2229 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-25 fix-burst. ADR-058 v2.14→v2.15; BC-2.16.003 v1.11→v1.12; COERCION-001 v1.24→v1.25; ROUTING-001 v1.25→v1.26; ARCH-INDEX v2.314→v2.315; BC-INDEX v9.32→v9.33; STORY-INDEX v2.845→v2.846. TD-VSDD-097 all dims CLEAR. FROZEN: ADR-058 v2.15 / BC-2.16.003 v1.12 / BC-2.16.002 v2.27 / ROUTING-001 v1.26 / COERCION-001 v1.25. STATE v8.759→v8.760.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2230 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-26 fix-burst. ADR-058 v2.15→v2.16; COERCION-001 v1.25→v1.26; ROUTING-001 v1.26→v1.27; ARCH-INDEX v2.315→v2.316; STORY-INDEX v2.846→v2.847. TD-VSDD-097 all dims CLEAR. FROZEN: ADR-058 v2.16 / BC-2.16.003 v1.12 / BC-2.16.002 v2.27 / ROUTING-001 v1.27 / COERCION-001 v1.26. STATE v8.760→v8.761.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2231 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-27 fix-burst. COERCION-001 v1.26→v1.27 (§Mandate Anchor #2 downstream-copy sweep); ROUTING-001 UNCHANGED v1.27; STORY-INDEX v2.847→v2.848. FROZEN: ADR-058 v2.16 / BC-2.16.003 v1.12 / BC-2.16.002 v2.27 / ROUTING-001 v1.27 / COERCION-001 v1.27. STATE v8.761→v8.762.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2232 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-28 fix-burst. BC-2.16.002 v2.27→v2.28; COERCION-001 v1.27→v1.28; ROUTING-001 v1.27→v1.28; BC-INDEX v9.33→v9.34; STORY-INDEX v2.848→v2.849. TD-VSDD-097 all dims CLEAR. FROZEN: ADR-058 v2.16 / BC-2.16.003 v1.12 / BC-2.16.002 v2.28 / ROUTING-001 v1.28 / COERCION-001 v1.28. STATE v8.762→v8.763.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2233 | state-manager | 2026-08-17 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-30 fix-burst. BC-2.16.003 v1.12→v1.13; COERCION-001 v1.28→v1.29; ROUTING-001 v1.28→v1.29; BC-INDEX v9.34→v9.35; STORY-INDEX v2.849→v2.850. BC-5.39.001 streak RESET 1/3→0/3. FROZEN: ADR-058 v2.16 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.29 / COERCION-001 v1.29. STATE v8.763→v8.764.** | wave-5-e-demo-fidelity | 2026-08-17 |
| D-2234 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053; TD-VSDD-096) — OCSF-correctness claroty adversary SPEC pass-31 records-only micro-burst. ROUTING-001 v1.29→v1.30; STORY-INDEX v2.850→v2.851. BC-5.39.001 CLEAN(strict)=NO (1 LOW). FROZEN: ADR-058 v2.16 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.30 / COERCION-001 v1.29. STATE v8.764→v8.765.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2235 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053) — OCSF-correctness claroty adversary SPEC pass-32 fix-burst. ADR-058 v2.16→v2.17; COERCION-001 v1.29→v1.30; ROUTING-001 v1.30→v1.31; ARCH-INDEX v2.316→v2.317; STORY-INDEX v2.851→v2.852. BC-5.39.001 CLEAN(strict)=NO (2 MED). FROZEN: ADR-058 v2.17 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.31 / COERCION-001 v1.30. STATE v8.765→v8.766.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2236 | state-manager | 2026-08-18 | **SESSION WRAP — pass-33 returned F-P33-MED-001 (MEDIUM): pipeline_result_to_record_batch signature gap — sensor_spec undefined free variable in ROUTING-001 v1.31; ADR-058 §D1/§I1 inaccurate. CLEAN(strict)=NO / CLEAN(PR-merge)=NO. Streak 0/3. FROZEN UNCHANGED: ADR-058 v2.17 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.31 / COERCION-001 v1.30. STATE v8.766→v8.767.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2237 | state-manager | 2026-08-18 | **compact-state burst COMPLETE — STATE.md v8.767→v8.768 (1,058 lines compacted). Archived to cycles/wave-5-e-demo-fidelity/: decisions-archive-D1789-D2199.md + burst-log.md + drift-items-resolved.md + blocking-issues-resolved.md + drift-items-open.md. TD-VSDD-097 all dims CLEAR. records-lint exit 0. STATE v8.767→v8.768.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2238 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053) — F-P33-MED-001 fix-burst COMPLETE. ADR-058 v2.17→v2.18 (§D1 param-threading explicit; §I1 two-step form). ROUTING-001 v1.31→v1.32 (AC-012+RG-024; free-var fix 5 loci; density 24/12=2.00). COERCION-001 v1.30→v1.31 (sibling pin). ARCH-INDEX v2.317→v2.318. STORY-INDEX v2.852→v2.853. BC-INDEX UNCHANGED v9.35. TD-VSDD-097 all dims CLEAR. BC-5.39.001 streak RESET 0/3 (perimeter changed). NEW FROZEN: ADR-058 v2.18 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.32 / COERCION-001 v1.31. records-lint exit 0. STATE v8.768→v8.769.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2239 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053) — F-P34 fix-burst COMPLETE. ADR-058 v2.18→v2.19 (F-P34-LOW-002 §Status heading lag corrected v2.17→v2.19; F-P34-OBS-001 helper symbol standardized to ocsf_field_to_arrow_name in §B2/§C4/§G). ROUTING-001 v1.32→v1.33 (story-writer: AC-012 call-site enumeration corrected; threading expr &self.sensor_spec.spec; ADR pin v2.19). COERCION-001 v1.31→v1.32 (state-manager: sibling ADR-058 pin v2.18→v2.19; no content change). ARCH-INDEX v2.318→v2.319. STORY-INDEX v2.853→v2.854. BC-INDEX UNCHANGED v9.35. TD-VSDD-097 all dims CLEAR: (1) ROUTING-001 amended same burst; (2) §Authority pin is terminal; (3) no new MUST blocks. BC-5.39.001 streak RESET 0/3 (perimeter changed). NEW FROZEN: ADR-058 v2.19 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.33 / COERCION-001 v1.32. records-lint exit 0. STATE v8.769→v8.770.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2240 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053; TD-VSDD-096) — pass-36 records-only micro-burst COMPLETE. F-P36-LOW-001 [LOW, TD-VSDD-091] §Changelog v1.12 row contained a quoted volatile-line-cite token ('line ~227') in record-tier text; rephrased F3 description to remove the bare line number while preserving meaning. ROUTING-001 v1.33→v1.34 (§v1.34 Amendment Sweep added). Anti-whack-a-mole sweep ADR-058 + COERCION-001 §Changelog + §TD-VSDD-097 + §Authority: CLEAR. COERCION-001/ADR-058/BC-INDEX/ARCH-INDEX UNCHANGED. STORY-INDEX v2.854→v2.855. TD-VSDD-097: (1) COERCION-001 §Changelog swept: CLEAR; (2) No downstream copies of §Changelog rows: CLEAR; (3) No new MUSTs introduced: N/A. ZERO content/mechanism changes. Self-certified. BC-5.39.001 streak 0/3 (F-P36-LOW-001 reset on p36). NEW FROZEN: ADR-058 v2.19 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.34 / COERCION-001 v1.32. records-lint exit 0. STATE v8.772→v8.773.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2241 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053; TD-VSDD-096) — pass-39 records-only micro-burst COMPLETE. F-P39-LOW-001 [LOW, POL-37/TD-VSDD-060 date-sync] §Authority BC-2.16.002 citation date parenthetical in ROUTING-001 v1.34 read "(modified 2026-08-16)" but BC-2.16.002 frontmatter modified: 2026-08-17. Corrected to "(modified 2026-08-17)". Comprehensive perimeter-wide records-hygiene audit: COERCION-001 §Authority all parentheticals ACCURATE; ADR-058 no date parentheticals in body; all version pins across all three perimeter artifacts ACCURATE; volatile line-cite tokens CLEAR; changelog L1/L7 PASS for all three artifacts. ROUTING-001 v1.34→v1.35 (§v1.35 Amendment Sweep added; TD-VSDD-097 all dims CLEAR). COERCION-001/ADR-058/BC-INDEX/ARCH-INDEX UNCHANGED. STORY-INDEX v2.855→v2.856. ZERO content/mechanism changes. Self-certified. BC-5.39.001 streak 0/3 (F-P39-LOW-001 reset from p37 CLEAN+p38 CLEAN). NEW FROZEN: ADR-058 v2.19 / BC-2.16.003 v1.13 / BC-2.16.002 v2.28 / ROUTING-001 v1.35 / COERCION-001 v1.32. records-lint exit 0. STATE v8.773→v8.774.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2242 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053) — P40/41/42 SPEC fix-burst COMPLETE. F-P40-HIGH-001≡F-P42-HIGH-001 (§G prism_describe ↔ raw_extensions contradiction: §G advertised phantom per-column names for ocsf_field==None columns). F-P41-MED-001 (§I5 intro 7→12). F-P41-MED-002 (§I5(d) "device" arm mislabeled). F-P42-LOW-001 (§G finding_info.uid example). HUMAN DECISION: aggregate raw_extensions model RATIFIED (Option 1). ADR-058 v2.19→v2.20 (§G Tier-1/Tier-2 describe model; §I5 intro 7→12; §I5(d) device-arm; §G finding_info.uid example). BC-2.16.003 v1.13→v1.14 (§Interpretation A Tier-1/Tier-2 alignment; EC-016-013-027 NEW). ROUTING-001 v1.35→v1.36 (AC-006/AC-007 rewritten; RG-025 NEW; §Authority ADR-058 pin v2.19→v2.20 + BC-2.16.003 pin v1.13→v1.14; density 25/12=2.08). COERCION-001 v1.32→v1.33 (state-manager: sibling ADR-058 pin v2.19→v2.20 + BC-2.16.003 pin v1.13→v1.14; no content change). ARCH-INDEX v2.319→v2.320. BC-INDEX v9.35→v9.36. STORY-INDEX v2.856→v2.857. TD-VSDD-097 all dims CLEAR: (1) ROUTING-001/COERCION-001 amended same burst; (2) §Authority pins terminal; (3) EC-016-013-027 anchored to AC-006/AC-007/RG-025. BC-5.39.001 strict streak REMAINS 0/3; perimeter changed → pass-43 re-gates on new frozen HEAD. NEW FROZEN: ADR-058 v2.20 / BC-2.16.003 v1.14 / BC-2.16.002 v2.28 / ROUTING-001 v1.36 / COERCION-001 v1.33. records-lint exit 0. STATE v8.774→v8.775.** | wave-5-e-demo-fidelity | 2026-08-18 |
| D-2243 | state-manager | 2026-08-18 | **SINGLE-COMMIT BURST (TD-VSDD-053) — P43/44/45 SPEC fix-burst COMPLETE (state-manager index/version/STATE leg). F-P43-HIGH-001 §I1 ocsf_field_to_arrow_name canonical home relocated to prism-spec-engine::column_mapping per dependency-graph.md §Dependency Rules Rule 2 (forbidden import cycle prism-mcp→prism-bin eliminated; both consumers import from prism_spec_engine::column_mapping). F-P43-MED-001 §Red Gate count 24→25 (RG-025 NEW in story-writer leg). F-P44-OBS-001 raw_extensions ColumnDescriptor col_type=Json/nullable=true (§Interpretation A Tier-2 + EC-016-013-027 §Expected Shape + §Canonical Test Vectors + §Traceability; ROUTING-001 AC-009/RG-016 extended). F-P45-MED-001 phantom ADR-023 §D3 cite corrected to dependency-graph.md §Dependency Rules Rule 2 at 4 loci. ADR-058 v2.20→v2.21. BC-2.16.003 v1.14→v1.15. ROUTING-001 v1.36→v1.37. COERCION-001 v1.33→v1.34 (sibling pin; input-hash 759227b→4cdc61e). ARCH-INDEX v2.320→v2.321. BC-INDEX v9.36→v9.37. STORY-INDEX v2.857→v2.858. TD-VSDD-097: (1) ROUTING-001/COERCION-001 amended same burst; (2) §Authority pins terminal; (3) EC-016-013-027 anchored AC-009/RG-016. BC-5.39.001 strict streak REMAINS 0/3; perimeter changed → pass-46 re-gates on new frozen HEAD. NEW FROZEN: ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34. records-lint exit 0. STATE v8.775→v8.776.** | wave-5-e-demo-fidelity | 2026-08-18 |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Phase-5 Deferred Findings

_Moved to `cycles/wave-5-e-demo-fidelity/phase-5-deferred-findings.md`. Two findings: F-LP12-OBS-001 (E-PLUGIN-008 dual-semantic) + F-LP25-OBS-001 (BC-2.17.002 vacuously true). Both require PO adjudication at Phase-5 PO pass._

## Drift Items (S-7.02 Cycle-Close Checklist)

_All open Drift Items archived to `cycles/wave-5-e-demo-fidelity/drift-items-open.md` (D-2237 compaction — 137 OPEN items). Closed items: `cycles/wave-5-e-demo-fidelity/drift-items-resolved.md`. Deferred (PIVOT-002/v1.0.0-greenfield): `cycles/wave-5-e-demo-fidelity/drift-items-deferred.md` (18 items, D-1368 compaction 2026-06-26)._

## Blocking Issues

| Issue | Owner | Opened | Resolved | Notes |
|-------|-------|--------|----------|-------|
| ADR-058-SPEC-READINESS-FAIL-001 [blocking-next-action; D-2176/observed 2026-08-15; owner: architect + product-owner + story-writer; severity: BLOCKING; status: OPEN] | architect + product-owner + story-writer | 2026-08-15 | — | OPEN — spec-evolution cycle required; COERCION-001 + ROUTING-001 NOT READY-FOR-TDD |
| PROCESS-GAP [D-2092/observed 2026-08-02; status: OPEN — version-field-sync ambiguity in dispatch brief] | Orchestrator | 2026-08-02 | — | process improvement; no blocker |
| PROCESS-GAP [D-2091/observed 2026-08-02; anchor: S-MAINT-BURST-COMMIT-COUNT-GATE-001; status: MITIGATED] | Orchestrator | 2026-08-02 | — | S-MAINT-BURST-COMMIT-COUNT-GATE-001 ARCH-QUES-001 pending |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md · decisions-archive-D1789-D2199.md · decisions-archive-D700-D1054.md · decisions-archive-D1055-D1123.md · decisions-archive-D1124-D1138.md · decisions-archive-D1165-D1352.md · drift-items-resolved.md · drift-items-open.md · drift-items-deferred.md · blocking-issues-resolved.md · phase-5-deferred-findings.md · frontmatter-cascade-archive.md · session-handoff-archive.md · convergence-trajectory.md · lessons.md. Prior cycles: wave-0-plugin-prereqs/ · wave-3-multi-tenant/ · wave-4-operations/.

## Session Resume Checkpoint (D-2243 — 2026-08-18 — P43/44/45 SPEC fix-burst COMPLETE; STATE v8.775→v8.776) [supersedes D-2242]

### RESUME IN ONE BREATH
(1) Prism Phase-3, OCSF-correctness CLAROTY workstream — SPEC adversarial cascade (BC-5.39.001 3-CLEAN) at strict streak 0/3. D-2243 P43/44/45 SPEC fix-burst COMPLETE: F-P43-HIGH-001 (§I1 ocsf_field_to_arrow_name canonical home prism-spec-engine::column_mapping) + F-P43-MED-001 (§Red Gate count 24→25) + F-P44-OBS-001 (raw_extensions ColumnDescriptor col_type=Json/nullable=true) + F-P45-MED-001 (phantom ADR-023 §D3 cite corrected to dependency-graph.md §Dependency Rules Rule 2). ADR-058 v2.20→v2.21; BC-2.16.003 v1.14→v1.15; ROUTING-001 v1.36→v1.37; COERCION-001 v1.33→v1.34. NEW FROZEN PERIMETER: ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34.
(2) VERY NEXT ACTION: adversary SPEC passes 46/47/48 (parallel, maximally-skeptical) on new frozen HEAD (ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34). Re-gates BC-5.39.001 streak at 0/3 on new perimeter. NOTE: dispatch 46/47/48 in parallel to maintain multi-perspective pressure on the updated perimeter.
(3) OPEN PROCESS-GAP (6+ recurrences D-2222+): RED-gate command/enumeration coherence + test-location-coherence — carry-forward for cycle-close self-improvement story.
(4) DRIFT-DTU-PARITY-STALE-001: scope DTU-PARITY internals OUT of cascade passes — verify only §Authority ADR-058 cite for SAC-2 link validity.
(5) PENDING HUMAN DECISION: orchestrator OFFERED to pause at 3-CLEAN spec-convergence before implementing COERCION/ROUTING — awaiting user decision; do NOT auto-proceed to TDD implementation without it.

**RESUME NEXT-ACTION: adversary SPEC passes 46/47/48 (parallel, maximally-skeptical) on new frozen HEAD. NEW FROZEN PERIMETER: ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34. (streak 0/3 re-gated on new perimeter)**

**HEADS:**
- `develop`: `69d821be` (LOCAL == origin/develop; pushed/clean). PR #239 squash-merged 2026-08-16T22:51Z.
- `factory-artifacts`: run `git -C .factory log -1 --format='%H'` for current HEAD (D-2243 fix-burst commit)
- `.worktrees/S-3.09` @`43c41389d` [feature/S-3.09] KEEP-PARKED (LOCAL-ONLY AT RISK — unpushed)
- `.worktrees/W3-FIX-S307-001` @`fcab8717c` [feature/W3-FIX-S307-001] PARKED-DIRTY do-NOT-touch (LOCAL-ONLY AT RISK — unpushed, 1 dirty test file)
- No open PRs. No agents in flight.

**GOVERNING DECISION (alongside D-2109):** DTU work DEFERRED to POST-FIRST-RELEASE per human decision 2026-08-16 — S-ADR058-DTU-PARITY-MIGRATION-001 AND DRIFT-DTU-CLAROTY-AUDITLOG-FILTERBODY-001 both PARKED until after v1 ships. Do NOT pick up DTU-fidelity work before v1.

**OCSF WORKSTREAM STATE:** D-2243 fix-burst COMPLETE — ADR-058 v2.20→v2.21 (§I1 ocsf_field_to_arrow_name canonical home prism-spec-engine::column_mapping; §B2 item 1 None-ocsf_field columns no individual Arrow field; §G raw_extensions col_type=Json/nullable=true; §Status heading corrected). BC-2.16.003 v1.14→v1.15 (§Interpretation A Tier-2 raw_extensions col_type/nullable; EC-016-013-027 updated). ROUTING-001 v1.36→v1.37 (§I1 canonical home; §Red Gate count 24→25; AC-009/RG-016 extended; density 25/12=2.08). COERCION-001 v1.33→v1.34 (sibling pin; input-hash updated). NEW FROZEN PERIMETER: ADR-058 v2.21 / BC-2.16.003 v1.15 / BC-2.16.002 v2.28 / ROUTING-001 v1.37 / COERCION-001 v1.34. Cascade ledger: p29(CLEAN 1/3)→p30(1MED)→p31(1LOW)→p32(2MED)→p33(1MED F-P33-MED-001 FIXED-D2238)→p34(4 findings FIXED-D2239)→p35(CLEAN 1/3)→p36(1LOW F-P36-LOW-001 FIXED-D2240)→p37(CLEAN 1/3)→p38(CLEAN 2/3)→p39(1LOW F-P39-LOW-001 FIXED-D2241)→p40(1HIGH)→p41(2MED)→p42(1HIGH+1LOW FIXED-D2242)→p43(2[1H+1M])→p44(1OBS)→p45(1MED FIXED-D2243). BC-5.39.001 streak 0/3 re-gated on new frozen HEAD.

**CROSS-SENSOR WORKSTREAM (parked, draft):** S-OCSF-FIDELITY-{CROWDSTRIKE,CYBERINT,ARMIS}-001 — schedule after claroty ships.

**PENDING HOUSEKEEPING (next session):** (1) Worktree teardown: `.worktrees/S-CLAROTY-AUDITLOG-TIMEBOX-001` (PR #239 merged; remote branch deleted). (2) Register self-improvement story for RED-gate/test-location-coherence [process-gap] (6+ recurrences) at cycle-close.

**BACKUP BOUNDARY (D-2243):**
- PUSHED / safe: `origin/develop` `69d821be` (PR #239 merged 2026-08-16T22:51Z); `factory-artifacts` (D-2243 fix-burst commit — run `git -C .factory log -1 --format='%H'`).
