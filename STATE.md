---
document_type: pipeline-state
level: ops
version: "7.810"
producer: state-manager
timestamp: 2026-06-14T00:00:00Z
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
develop_head: "664566e9"
bc_index_version: "6.56"
vp_index_version: "1.79"
story_index_version: "v2.387"
arch_index_version: "2.133"
error_taxonomy_version: "1.79"
total_stories: 200
active_contracts: 235
draft_contracts: 2
retired_contracts: 6
prd_version: "1.12"
policies_version: "1.33"
subsystem_count: 22
vp_count: 157
bc_count_corrected: 250
workspace_test_count: 4273
vsdd_factory_version: "1.0.0-rc.18"

# ── WAVE-5 PHASE STATUS ──
current_step: "T10 in progress + PARALLEL EXECUTION ACTIVE (D-1165/D-1166/D-1167 2026-06-14): S-DEMO-004 v1.10 (pass-3 LOW-1 §File-Structure corrected); S-5.02 ready v1.6 (remove-uncertainty NEXT); PIVOT-001 ready v1.3 (BC-2.19.001 v1.4 + BC-2.19.003 v1.3 pinned); S-3.13 ready v1.8 (body propagated; remove-uncertainty NEXT); LAUNCHER ready v2.0 (Option-2 Rust; 8 pts; tdd; remove-uncertainty NEXT); S-1.15 relevance under architect review. NEXT: worktree-manage create S-DEMO-004 → test-writer → implementer → LOCAL 3-CLEAN → demo-recorder → push → pr-manager → PR-LEVEL 3-CLEAN → CI → squash-merge → state-manager burst."
wave5_autonomy_granted: "2026-06-04 D-989 — full autonomous A→B→C, strict convergence, auto-merge on objective gates; pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit"

# ── PARKED WORKTREES ──
worktree_status: "stale: S-3.09 (FROZEN) + W3-FIX-S307-001 (BLOCKED superseded) — leave alone"

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
pre_compact_snapshot: "Durability-hardening compaction 2026-06-14 — STATE v7.801→v7.802 (D-1159). D-1139..D-1157 T5-merge + T6 cascade archived to cycles/wave-5-e-demo-fidelity/burst-log.md. Current Phase Steps rows D-1154/D-1155/D-1156/D-1157 replaced with 2-row summary. DEMO-SCOPE.md staleness corrected. SESSION-HANDOFF read-order clarified. Prior compaction: D-1159 precursor — Hygiene compaction 2026-06-13 — STATE v7.785→v7.786 (D-1124..D-1138 archived). D-1132 2026-06-13 — D-1055..D-1123 archived. D-1056 2026-06-08 — D-700..D-1054 archived."
pre_compact_snapshot_at: "2026-06-14"
---
# VSDD Pipeline State — Prism

## Project Metadata

**Prism** | Rust | brownfield | per-analyst stdio (MCP) | Started 2026-04-13 | Last Updated 2026-06-14 (D-1167 SPEC CONSOLIDATION — S-3.13 v1.8 + LAUNCHER v2.0 (Option-2 Rust) + S-DEMO-004 v1.10 + PIVOT-001 v1.3 (BC-2.19.001 v1.4 + v1.3 pinned); STATE v7.810)

## Active Objective (North Star)

**NORTH STAR: Multi-client SOC-analyst live demo — multiple DTU clients, per-client data, prism MCP wired into Claude (stdio), deterministic scenario progression, ThreatIntel+NVD enrichment, capability-discovery (D-1162 REQUIRED).** Full detail: SESSION-HANDOFF.md §ACTIVE OBJECTIVE + `.factory/objectives/DEMO-SCOPE.md`. Task ledger: `.factory/objectives/multi-client-soc-demo-tasks.md` CURRENT POINTER = **T10 — S-DEMO-004 delivery (ready v1.10; PRE-TDD CLEAR; PARALLEL EXECUTION ACTIVE — lanes A-E per D-1165/D-1166/D-1167)** (T1–T9 ALL DONE; develop@664566e9). NEXT ACTION: vsdd-factory:worktree-manage create S-DEMO-004 → test-writer → deliver-story S-DEMO-004. Parallel: Lane A (S-5.02 ready v1.6 — remove-uncertainty NEXT), B (S-3.13 ready v1.8 — remove-uncertainty NEXT), C (PIVOT-001 ready v1.3 — remove-uncertainty NEXT), D (S-1.15 architect relevance review PENDING), E (LAUNCHER ready v2.0 — remove-uncertainty NEXT). After T10: capability-discovery T15a-d + enrichment T16a-c (all REQUIRED).

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
| 3: DTU + Spec stories (post-Wave-3) | COMPLETE | 2026-05-27 | 2026-06-08 | PRs #162–#176 | S-DEMO-001/002/003 + CLAROTY/ARMIS/CROWDSTRIKE DTU + Phase B all MERGED; develop@a42e3eaf |
| **5: Wave 5 Phase B** | **COMPLETE** | — | 2026-06-08 | All 4 lanes merged | develop@a42e3eaf |
| **5: Wave 5 Phase C** | **COMPLETE** | — | 2026-06-09 | PRs #177–#180 | develop@64d34967; all 4 lanes merged |
| **5: S-DEMO-DTU-LIVE-SCENARIO-001-A (T4-A)** | **MERGED** | 2026-06-09 | 2026-06-10 | PR #181 develop@c287b00d | LOCAL 18-pass 3-CLEAN strict + PR-LEVEL 3-pass 3-CLEAN strict; BC-2.06.018 v1.6 active |
| **Review cycle fix-PRs (#183/#184/#182)** | **MERGED** | 2026-06-10 | 2026-06-12 | PRs #183→#184→#182 | QRY develop@f88b10e3; MCP develop@c200d5a2; DTU develop@939f36ce; all CI 43/43 GREEN |
| **5: S-DEMO-DTU-LIVE-SCENARIO-001-B (T5)** | **MERGED** | 2026-06-12 | 2026-06-13 | PR #185 develop@7fd35b77 | LOCAL 13-pass 3-CLEAN; PR-LEVEL 29-pass 3-CLEAN strict CONVERGED; BC-2.06.019 v1.7 + BC-2.06.020 v1.6 active |
| **5: lefthook docs-only pre-push (PR #186)** | **MERGED** | 2026-06-13 | 2026-06-13 | PR #186 develop@f7400f83 | 35/35 test cases; 43-green CI; D-1134 bypass remediation RESOLVED |
| **5: S-DEMO-MULTI-TENANT-DTU-001 (T6)** | **MERGED** | 2026-06-13 | 2026-06-14 | PR #187 develop@664566e9 | LOCAL 11-pass 3-CLEAN + PR-LEVEL 10-pass 3-CLEAN strict; CI 43/43; BC-2.06.017 v1.10 active |

## Current Phase Steps

| Step | Agent | Status | Output |
|------|-------|--------|--------|
_D-735 through D-1164 archived to cycles/wave-5-e-demo-fidelity/burst-log.md and decisions-archive files. D-1161..D-1164 archived at D-1165 compaction 2026-06-14._
| D-1161..D-1164 | state-manager | 2026-06-14 | ARCHIVED — PRE-TDD CLEAR (D-1161); capability-discovery REQUIRED (D-1162); PREREQ-CONFIRMED+DTU-EVERYTHING (D-1163); FULL Option-A infusion framework REQUIRED/5 stories enumerated (D-1164). See burst-log.md. STATE v7.803→v7.807. |
| D-1165 | state-manager | 2026-06-14 | PARALLEL-LANE KICKOFF + spec consolidation. (1) S-DEMO-004 v1.6→v1.7 (story-writer O-01 closure: crates_touched [prism-bin]→[prism-bin, prism-dtu-harness]; overlay_wiring.rs MODIFY row). (2) LAUNCHER ready v1.0 materialized (story-writer; 10 ACs; 5 pts; facade; retire start-demo.sh; 5 BCs BC-2.06.001/012/013/014/017; scan in flight). (3) S-5.02 BC reconciliation: BC-2.10.004 v2.7→v2.8 / BC-2.10.007 v1.4→v1.5 / BC-2.10.011 v1.4→v1.5 (all active-lifecycle/draft-status; POL-14 at merge). (4) PARALLEL EXECUTION: no worktree cap; Lane A S-5.02→S-5.03→S-5.04; Lane B S-3.13; Lane C PIVOT-001→[S-1.14-REDO∥PIVOT-002]→PIVOT-003 (CRITICAL); Lane D S-1.15; Lane E LAUNCHER. (5) T16 ordering CORRECTED (2 architect findings): S-1.15 NOT a gate (enrich_single operational); PIVOT-001 BEFORE S-1.14-REDO (forward-subset). Pre-TDD: S-5.02 PO-done/finalize-next; S-3.13 finalize-needed; PIVOT-001 finalize/research in flight. BC-INDEX v6.54→v6.55. STORY-INDEX v2.384→v2.385. Ledger v1.24→v1.25. develop_head 664566e9 UNCHANGED. total_stories 200 / active_contracts 235 / draft_contracts 2 UNCHANGED. STATE v7.807→v7.808. |
| D-1166 | state-manager | 2026-06-14 | LANE SPECS FINALIZED burst. (1) S-5.02 story-writer finalized: ready v1.6; tri-state model; server.rs/error_mapping.rs deltas; 11 ACs; 13 Red Gate names; rmcp 1.7; 5 pts; tdd_mode strict. (2) PIVOT-001 story-writer corrected: ready v1.2; 4 implementer-traps fixed (post_return RETRACTED; enrich_single delegation signature; DataFusion 53.1 async-UDF path; NullSource-replacement task). BC-2.19.001 v1.4 PO amendment: plugin-type descriptors MUST carry Arc<PluginInfusionSource> not NullSource. (3) S-3.13 spec partially locked: error-taxonomy v1.79 (E-QUERY-037 TableNotAvailable — PrismError variant; map_prism_error -32602; engine.rs plan-time; strsim="0.11"); BC-2.11.001 v1.8 (table-availability postcondition + E-QUERY-037 error case); architect ratified. PENDING: story body propagation (E-QUERY-001→E-QUERY-037). (4) S-DEMO-004 v1.9: PO AC-009 sequential model (v1.7→v1.8; test renamed test_BC_2_11_005_sequential_org_queries_do_not_interfere); story-writer v1.8→v1.9 (M2-01 boot.rs stderr; O2-01 fixture path; O2-02 helper signatures; M2-02 regression-guard NOTE). (5) LAUNCHER Option-2 decision (user): start-multi Rust CLI subcommand wiring start_instances/MultiInstanceConfig; crates_touched adds prism-dtu-demo-server; tdd_mode facade→tdd; pts ~5→~7-8; story rework PENDING. Pending follow-ups: (a) S-3.13 body E-QUERY-001→E-QUERY-037; (b) PIVOT-001 BC-table version-pin; (c) LAUNCHER Option-2 rework. BC-INDEX v6.55→v6.56. STORY-INDEX v2.385→v2.386. error-taxonomy v1.78→v1.79. Ledger v1.25→v1.26. develop_head 664566e9 UNCHANGED. total_stories 200 / active_contracts 235 / draft_contracts 2 UNCHANGED. STATE v7.808→v7.809. |

## Decisions Log

_D-001..D-046 archived: `cycles/phase-3-dtu-wave-2/decisions-archive-d001-d032.md`. D-047..D-174: `cycles/wave-3-multi-tenant/decisions-archive-d047-d114.md`. D-175..D-188: `cycles/wave-3-multi-tenant/burst-log.md`. D-200..D-213: `cycles/wave-4-operations/burst-log.md`. D-432..D-699: `cycles/wave-0-plugin-prereqs/burst-log.md` (D-727 compaction). **D-214..D-320 LOST** — TD-VSDD-058. **D-700..D-1054: `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md`** (D-1056 compaction). **D-1055..D-1123: `cycles/wave-5-e-demo-fidelity/decisions-archive-D1055-D1123.md`** (D-1132 compaction). **D-1124..D-1138: `cycles/wave-5-e-demo-fidelity/decisions-archive-D1124-D1138.md`** (hygiene compaction 2026-06-13). **D-1139..D-1164: `cycles/wave-5-e-demo-fidelity/burst-log.md`** (D-1159 compaction 2026-06-14 — T5+T6 cascade; D-1165 compaction 2026-06-14 — D-1161..D-1164 archived)._

| ID | Decision | Rationale | Phase | Date |
|----|----------|-----------|-------|------|
| D-1167 | state-manager | 2026-06-14 | **SPEC CONSOLIDATION burst (TD-VSDD-053 single-commit).** (1) S-3.13 v1.7→v1.8: body propagation COMPLETE — E-QUERY-001→E-QUERY-037 at all body sites (AC-2/3/8, Objective, BC table, BC Linkage, Architecture Compliance, Edge Cases); planner.rs→engine.rs retarget; strsim="0.11" chosen; 15 Red Gate tests; HotReloadWatcher NOT needed (D-1163 ConfigManager surface). Status: ready for remove-uncertainty → TDD. (2) LAUNCHER v1.0→v2.0: Option-2 Rust executed — StartMulti Commands variant; cmd_start_multi wiring start_instances/MultiInstanceConfig; MultiOrgDemoConfig/OrgConfig structs; nested {org_slug:{sensor:url}} sidecar (.prism-dtu-demo-server.urls-multi.json); 13 ACs; 8 pts; tdd_mode tdd; 5 Red Gate tests; risk LOW→MEDIUM; crates_touched [prism-dtu-demo-server]. Status: ready. (3) S-DEMO-004 v1.9→v1.10: LOCAL adversary PASS-3 LOW-1 §File-Structure correction — regression-guard row renamed to bc_2_10_006_mcp_stdout_purity.rs CREATE; description cites BC-2.10.006 §Postconditions as stdout-purity invariant source. (4) PIVOT-001 v1.2→v1.3: BC version-pin sync (citation/frontmatter only) — BC-2.19.001 v1.3→v1.4 + BC-2.19.003 v?→v1.3 (PO-confirmed D-1166). (5) S-1.15 demo-relevance PENDING architect review (TD-PLUGIN-P0-008 action-dispatch may be deferred-TDE; no story change; Lane D HOLD). BC-INDEX v6.56 UNCHANGED. STORY-INDEX v2.386→v2.387. Ledger v1.26→v1.27. develop_head 664566e9 UNCHANGED. total_stories 200 / active_contracts 235 / draft_contracts 2 UNCHANGED. STATE v7.809→v7.810. | wave-5-e-demo-fidelity | 2026-06-14 |
| D-1166 | state-manager | 2026-06-14 | **LANE SPECS FINALIZED burst.** (1) S-5.02 story-writer finalized (ready v1.6; tri-state model; 11 ACs; 13 Red Gate names; rmcp 1.7; 5 pts; tdd_mode strict; server.rs/error_mapping.rs deltas). (2) PIVOT-001 story-writer corrected (ready v1.2; 4 implementer-traps: post_return RETRACTED; enrich_single delegation signature fixed; DataFusion 53.1 async-UDF path confirmed; NullSource-replacement task added). BC-2.19.001 v1.4 PO amendment: plugin-type InfusionUdfDescriptor MUST carry Arc<PluginInfusionSource> not NullSource (loading defect if NullSource; equivalent to E-INFUSE-003). (3) S-3.13 spec partially locked: E-QUERY-037 TableNotAvailable registered in error-taxonomy v1.79 (PrismError::TableNotAvailable{table,sensor,available_sensors,available_tables,did_you_mean}; MCP -32602; plan-time engine.rs emission; strsim="0.11"); BC-2.11.001 v1.8 (table-availability postcondition + E-QUERY-037 error case); architect ratified shape + map_prism_error arm. PENDING: S-3.13 story body propagation (body still cites E-QUERY-001; story-writer dispatch REQUIRED before TDD). (4) S-DEMO-004 v1.9: PO v1.8 (AC-009 sequential model — MCP-over-stdio serialized channel; test renamed test_BC_2_11_005_sequential_org_queries_do_not_interfere; not tokio::join!); story-writer v1.9 (M2-01 boot.rs step1_init_tracing/.with_writer(stderr); O2-01 fixture path tests/fixtures/→fixtures/; O2-02 helper write_multi_org_overlays signatures; M2-02 regression-guard NOTE). (5) LAUNCHER Option-2 (user decision): real Rust CLI subcommand demo-server start-multi wiring start_instances/MultiInstanceConfig; crates_touched adds prism-dtu-demo-server; tdd_mode facade→tdd; pts ~5→~7-8; story rework PENDING (architect/story-writer add start-multi ACs + MultiOrgConfig schema). Pending follow-ups: (a) S-3.13 body E-QUERY-001→E-QUERY-037; (b) PIVOT-001 BC-table version-pin (BC-2.19.001 v1.4/BC-2.19.003 v1.3); (c) LAUNCHER Option-2 rework. BC-INDEX v6.55→v6.56. STORY-INDEX v2.385→v2.386. error-taxonomy v1.78→v1.79. Ledger v1.25→v1.26. develop_head 664566e9 UNCHANGED. total_stories 200 / active_contracts 235 / draft_contracts 2 UNCHANGED. STATE v7.808→v7.809. | wave-5-e-demo-fidelity | 2026-06-14 |
| D-1165 | state-manager | 2026-06-14 | **PARALLEL-LANE KICKOFF + spec consolidation burst.** (1) S-DEMO-004 v1.6→v1.7: story-writer O-01 closure — crates_touched [prism-bin]→[prism-bin, prism-dtu-harness]; overlay_wiring.rs MODIFY row added to §File Structure (wiring-not-redesign; spec-only, no code). (2) LAUNCHER ready v1.0: story-writer materialized S-DEMO-LAUNCHER-CONSOLIDATION-001 (10 ACs; 5 pts; tdd_mode facade; retire start-demo.sh decision recorded; UX question N-org-config deferred to PR review; 5 BCs: BC-2.06.001/012/013/014/017; scan in flight). (3) S-5.02 BC reconciliation: BC-2.10.004 v2.7→v2.8 (3-case client_id taxonomy; E-MCP-001/E-CFG-100; E-AUTH-003 namespace-collision caveat) / BC-2.10.007 v1.4→v1.5 (nested 9-field structuredContent.error) / BC-2.10.011 v1.4→v1.5 (tri-state + hierarchical capability-path list_capabilities). All active-lifecycle/status:draft; POL-14 at merge. (4) PARALLEL EXECUTION: no worktree cap (user directive D-1165); practical limiter ~3 LOCAL + 1 PR-level. Lanes: A (S-5.02→S-5.03→S-5.04), B (S-3.13), C (PIVOT-001→[S-1.14-REDO∥PIVOT-002]→PIVOT-003 CRITICAL PATH), D (S-1.15), E (LAUNCHER). Merge-coordination: S-3.13↔PIVOT-001 on prism-query/engine.rs; infusion trio PIVOT-001→S-1.14-REDO serialize. (5) T16 ORDERING CORRECTED — 2 architect findings supersede D-1164 T16-FOUND-A gating: (a) S-1.15 NOT a gate before PIVOT-001 — `PluginRuntime::enrich_single` operational on develop; S-1.15 TD-PLUGIN-P0-008 action-dispatch runs PARALLEL with PIVOT-001, not before; (b) PIVOT-001 builds BEFORE S-1.14-REDO (forward-subset). Pre-TDD scan: S-5.02 PO-done/story-writer-finalize-next (server.rs deltas, Red Gate names, rmcp 1.7 compat, tri-state scope expansion); S-3.13 finalize-needed (PO new E-QUERY table-availability code, architect edit-distance-vs-strsim, story-writer retarget engine.rs/scoping.rs); PIVOT-001 finalize/research in flight (DataFusion async-UDF; 4 implementer-traps); LAUNCHER scan in flight; S-1.15 scoping pass owed. BC-INDEX v6.54→v6.55. STORY-INDEX v2.384→v2.385. Ledger v1.24→v1.25. develop_head 664566e9 UNCHANGED. total_stories 200 / active_contracts 235 / draft_contracts 2 UNCHANGED. STATE v7.807→v7.808. | wave-5-e-demo-fidelity | 2026-06-14 |

## Skip Log

| Step | Skipped? | Justification |
|------|----------|---------------|
| UX Spec | yes | CLI/stdio-only product; no UI surfaces |
| DTU clone build | deferred | dtu_clones_built: pending — awaiting Phase 3 start |

## Phase-5 Deferred Findings

_Moved to `cycles/wave-5-e-demo-fidelity/phase-5-deferred-findings.md`. Two findings: F-LP12-OBS-001 (E-PLUGIN-008 dual-semantic) + F-LP25-OBS-001 (BC-2.17.002 vacuously true). Both require PO adjudication at Phase-5 PO pass._

## Drift Items (S-7.02 Cycle-Close Checklist)

_Closed items: `cycles/wave-5-e-demo-fidelity/drift-items-resolved.md`. OQ-001 (2026-05-22): BC-5.39.001 elevation — track for next maintenance burst._

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
| DRIFT-D849-001 | ADR-031 `related_bcs` field missing BC-2.01.017 | Architect to amend ADR-031 frontmatter | next architect dispatch |
| DRIFT-D849-002 | VP-TBD No-HTTP-Call invariant during StaticCookieAuthProvider::acquire_token | Architect to assign VP-NNN | next formal-verifier or architect dispatch |
| DRIFT-D916-001 [process-gap, codified] | POL-14 story-status transition gap: BCs promoted but story-status not auto-set to merged | S-POL-14-STATUS-SYNC-001 filed (D-918) | — |
| DRIFT-D904-001 (JUSTIFIED DEFERRAL) | OBS-PR1-001 adversary diff-tooling limitation | Track in drbothen/vsdd-factory upstream | upstream |
| DRIFT-D904-002 (JUSTIFIED DEFERRAL) | OBS-PR2 worktree-path-resolution hazard | Track in drbothen/vsdd-factory upstream | upstream |
| DRIFT-D923-001 [architect-scope] | ADR-022 §B step 8 description accuracy gap | Architect to amend ADR-022 §B step 8 narrative | next architect dispatch |
| DRIFT-D923-002 [architect/PO-scope] | validate_sensor_spec not called from production spec-load path | Architect/PO adjudicate | separate disposition |
| DRIFT-D1000-001..005 [process-gap, JUSTIFIED DEFERRAL] | Five vsdd-factory engine process improvements | vsdd-factory upstream | upstream |
| DRIFT-D954-001 [cross-crate, REGISTERED] | BC-3.5.002 precondition 3 mis-cite in prism-dtu-armis (~40+ sites) + prism-dtu-slack (1 site) | S-MAINT-W3SEC-CITE-SWEEP-002 anchors this | S-MAINT-W3SEC-CITE-SWEEP-002 |
| DRIFT-D1016-SEC-007 [hardening-candidate] | QueryParams.start_time/end_time typed as Option<String>; TimestampString newtype proposed | Architect/PO adjudicate | architect/PO |
| DEFER-ORPHAN-SENSORS-DIR-001 [legacy-cleanup] | Orphaned top-level sensors/*.toml hardcode us-1 | S-MAINT-ORPHAN-SENSORS-DIR-001 to be authored | — |
| DEFER-SS22-LABEL-DRIFT-001 | ARCH-INDEX "Process Lifecycle" vs BC-INDEX/story "Binary Entrypoint" for SS-22 | Maintenance story; architect adjudicates | — |
| DEFER-CLAUDEMD-BC216002-MISLABEL-001 | SAP-1 probe + CLAUDE.md §Conventions cite BC-2.16.002 as "Structured Event Catalog" (wrong) | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DEFER-CI-WORKFLOW-SPEC-DRIFT-001 | Spec↔CI-workflow drift class; no existing policy/lint for workflow attribute drift | Cycle-close: consistency-validator rule improvement | cycle-close |
| DEFER-EQUERY009-001 | BC-2.11.007 DI-021 E-QUERY-009 enforcement absent from live path | Phase-5: PO/architect adjudicate | phase-5 |
| DEFER-POL7-EDEMO-TEMPLATE-001 | POL-7 §References step unsatisfiable for E-DEMO story-template family | Cycle-close: PO reconciles | cycle-close |
| DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001 [HUMAN-ONLY] | DRIFT-ORCH-PRLEVEL-PUSH-001 not yet mirrored into CLAUDE.md | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 [HUMAN-ONLY] | CLAUDE.md §Git Workflow states factory-artifacts local-only by default (outdated) | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DRIFT-PAGINATION-PAGESIZE-VALIDATION-001 [pre-existing] | `spec_parser` PaginationConfig lacks page_size>0 guard | PO/architect adjudicate | spec-engine validation story |
| DRIFT-EDITION-SYNC-001 [pre-existing, JUSTIFIED DEFERRAL] | prism-credentials/Cargo.toml edition=2021 vs workspace edition 2024 | S-MAINT-EDITION-SYNC-001 | S-MAINT-EDITION-SYNC-001 |
| DRIFT-SEC-TAPE-PATH-001 [security-LOW] | Hardcoded `/Users/<name>/...` absolute paths in `docs/demo-evidence/**/*.tape` files (CWE-200) | S-MAINT-TAPE-PATH-SWEEP-001 (maintenance wave) | maintenance wave |
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 [doc-staleness, HUMAN-ONLY] | CLAUDE.md §Conventions cites stale EXPECTED count | **HUMAN CLAUDE.md EDIT REQUIRED** | human at next checkpoint |
| DRIFT-SLUG-FORMAT-BC34004-001 [PO-harmonization] | BC-3.4.004+BC-3.5.001 carry arbitrary-string slug vs ADR-036 §2.2 canonical hex slug | PO reconciles BC-3.4.004/BC-3.5.001 test vectors | maintenance / Story-B-adjacent |
| DRIFT-RC1-PAGINATION-PARITY-001 [PO-harmonization] | BC-2.16.013 INV-HARNESS-ROUTE-PARITY does not explicitly define "route surface" | PO harmonization at next PO dispatch | next PO dispatch |
| DRIFT-D1151-CAP036-001 [process-gap, system-level] | capabilities.md §CAP-036 "Anchored BCs" list missing reverse-cite for BC-2.06.017/018/019/020 (4-BC cohort; PRE-EXISTING; OUT-OF-DIFF; BC→CAP direction correct; gap is CAP→BC reverse-cite only) | business-analyst system-level capabilities.md maintenance sweep | cycle-close |

## Blocking Issues

| ID | Description | Blocker Owner | Since | Status |
|----|-------------|---------------|-------|--------|
| TD-VSDD-005 | vsdd-factory:adversary runtime tool-binding bug — only Read bound at dispatch; general-purpose-as-adversary workaround required | vsdd-factory plugin maintainer | 2026-04-26 | OPEN — housekeeping pause before Wave 3 |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md · decisions-archive-D700-D1054.md · decisions-archive-D1055-D1123.md · **decisions-archive-D1124-D1138.md** · **drift-items-resolved.md** · **phase-5-deferred-findings.md** · frontmatter-cascade-archive.md · session-handoff-archive.md · convergence-trajectory.md · lessons.md. Prior cycles: wave-0-plugin-prereqs/ · wave-3-multi-tenant/ · wave-4-operations/.

_No open PRs on develop. Last merges: PR #185 develop@7fd35b77 (T5 DONE), PR #186 develop@f7400f83 (D-1134 bypass), PR #187 develop@664566e9 (T6 DONE — D-1158 BC-2.06.017 active). T8+T9 COMPLETE (D-1160). D-1110 PRE-TDD re-run CLEAR (D-1161 — S-DEMO-004 v1.6). NEXT: T10 worktree+test-writer → deliver-story S-DEMO-004._

## Session Resume Checkpoint (D-1167 — 2026-06-14; STATE v7.810)

**STATE v7.810. CURRENT POSITION: T10 in progress + PARALLEL EXECUTION ACTIVE (D-1165/D-1166/D-1167). S-DEMO-004 ready v1.10 (pass-3 LOW-1 §File-Structure corrected; PRE-TDD CLEAR — do NOT re-run). S-5.02 READY v1.6 (remove-uncertainty NEXT). S-3.13 READY v1.8 (body propagation COMPLETE; remove-uncertainty NEXT). PIVOT-001 READY v1.3 (BC-2.19.001 v1.4 + BC-2.19.003 v1.3 pinned; remove-uncertainty NEXT). LAUNCHER READY v2.0 (Option-2 Rust; 8 pts; tdd; 13 ACs; 5 Red Gate tests; remove-uncertainty NEXT). S-1.15 architect relevance review PENDING. T1–T9 ALL DONE. No open PRs. develop@664566e9. BC-INDEX v6.56 (active 235 / draft 2 / retired 6). STORY-INDEX v2.387 (200 stories). error-taxonomy v1.79. BC-2.06.017 v1.10 ACTIVE, BC-2.06.018 v1.6 ACTIVE, BC-2.06.019 v1.7 ACTIVE, BC-2.06.020 v1.6 ACTIVE.**

**RESUME PROTOCOL (run on fresh session start):**
0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star) + §RESUME SNAPSHOT (latest D-1167). Read `.factory/objectives/DEMO-SCOPE.md` (demo scope/narrative reference; authoritative pipeline position = STATE.md + SESSION-HANDOFF §RESUME SNAPSHOT).
1. `vsdd-factory:factory-worktree-health` (BLOCKING — must pass before reading any state).
2. Verify develop HEAD: `git log --oneline origin/develop | head -1` → expect `664566e9`.
3. Verify `grep "^version:" .factory/STATE.md` shows `"7.810"`.
4. Parked worktrees: `.worktrees/S-3.09` (FROZEN) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded) — leave alone.
5. Apply lessons (a)–(z23) from `cycles/wave-5-e-demo-fidelity/lessons.md`.
6. **NEXT ACTION: T10 — S-DEMO-004 v1.10 TDD delivery. PARALLEL lanes active: A (S-5.02 ready v1.6 — remove-uncertainty → TDD), B (S-3.13 ready v1.8 — remove-uncertainty → TDD), C (PIVOT-001 ready v1.3 — remove-uncertainty → TDD), D (S-1.15 architect relevance review PENDING), E (LAUNCHER ready v2.0 — remove-uncertainty → TDD). Execute T10: vsdd-factory:worktree-manage create S-DEMO-004 → test-writer → implementer → LOCAL 3-CLEAN strict → demo-recorder → push → pr-manager → PR-LEVEL 3-CLEAN strict + pr-reviewer + security → CI → squash-merge → state-manager post-merge burst. D-989+D-1090 autonomy grant ACTIVE. AFTER T10: T15a-d (capability-discovery) + T16a-c (enrichment) ALL REQUIRED (D-1162/D-1164). T16 ordering: T16-ARCH-PLAN → T16a (PIVOT-001) ∥ T16-FOUND-A (S-1.15 if not deferred) → T16-FOUND-B (S-1.14-REDO) → T16b → T16c.**
