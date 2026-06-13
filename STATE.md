---
document_type: pipeline-state
level: ops
version: "7.788"
producer: state-manager
timestamp: 2026-06-13T22:00:00Z
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
develop_head: "f7400f83"
bc_index_version: "6.45"
vp_index_version: "1.79"
story_index_version: "v2.372"
arch_index_version: "2.133"
error_taxonomy_version: "1.78"
total_stories: 200
active_contracts: 234
draft_contracts: 3
retired_contracts: 6
prd_version: "1.12"
policies_version: "1.33"
subsystem_count: 22
vp_count: 157
bc_count_corrected: 250
workspace_test_count: 4273
vsdd_factory_version: "1.0.0-rc.18"

# ── WAVE-5 PHASE STATUS ──
current_step: "D-1145 — T6 API-gap D-1075-API-GAP-001 ADJUDICATED (2026-06-13). Architect adjudication Option A (production-grade, no deferral): start_instances return type amended HashMap<String,SocketAddr>→MultiInstanceServers lifecycle handle (#[non_exhaustive]) owning shutdown_tx + task_handles; servers.socket_map() + servers.shutdown() + Drop graceful drain (axum with_graceful_shutdown); mirrors MultiInstanceHarness. Story v1.3→v1.4, BC-2.06.017 v1.1→v1.2. EXPECTED re-baselined: implementer/test-writer MUST use EXPECTED=52→60 (+8 total: 7 prior arms from D-1144 + 1 new E0639 MultiInstanceServers struct arm). NEXT: implementer lands MultiInstanceServers + amended start_instances + new compile-fail arm + ci.yml EXPECTED 52→60; test-writer updates tests; then resume Step 4.5 LOCAL adversary. STATE v7.788."
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
pre_compact_snapshot: "Hygiene compaction 2026-06-13 — STATE v7.785→v7.786. Decisions D-1124..D-1138 archived to cycles/wave-5-e-demo-fidelity/decisions-archive-D1124-D1138.md. Phase Progress per-story rows (DTU Waves 0-2 through Wave-5 Phase-B story rows) archived to cycles/wave-5-e-demo-fidelity/burst-log.md. Closed/resolved Drift Items moved to cycles/wave-5-e-demo-fidelity/drift-items-resolved.md. Phase-5 Deferred Findings moved to cycles/wave-5-e-demo-fidelity/phase-5-deferred-findings.md. Open PRs table replaced with one-line (both merged). Prior compaction: D-1132 2026-06-13 — Decisions D-1055..D-1123 archived. Prior compaction: D-1056 2026-06-08 — Decisions D-700..D-1054 archived."
pre_compact_snapshot_at: "2026-06-13"
---
# VSDD Pipeline State — Prism

## Project Metadata

**Prism** | Rust | brownfield | per-analyst stdio (MCP) | Started 2026-04-13 | Last Updated 2026-06-13 (D-1145 T6 API-gap D-1075-API-GAP-001 adjudicated Option A; start_instances→MultiInstanceServers; story v1.4, BC-2.06.017 v1.2; EXPECTED 52→60; STATE v7.788)

## Active Objective (North Star)

**NORTH STAR: Multi-client SOC-analyst live demo — multiple DTU clients, per-client data, prism MCP wired into Claude (stdio), deterministic scenario progression, ThreatIntel+NVD enrichment.** Full detail: SESSION-HANDOFF.md §ACTIVE OBJECTIVE + `.factory/objectives/DEMO-SCOPE.md`. Task ledger: `.factory/objectives/multi-client-soc-demo-tasks.md` CURRENT POINTER = **T6 IN PROGRESS** (S-DEMO-MULTI-TENANT-DTU-001 v1.4; D-1145 API-gap adjudicated; NEXT: implementer+test-writer then LOCAL adversary). T1–T5+T4-A DONE.

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

## Current Phase Steps

| Step | Agent | Status | Output |
|------|-------|--------|--------|
_D-735 through D-1129 archived to cycles/wave-5-e-demo-fidelity/burst-log.md and decisions-archive files._
| D-1132 | state-manager | 2026-06-13 | ZERO-CONTEXT RESUME HARDENING (D-1132). STATE compacted: decisions D-1055..D-1123 archived. SESSION-HANDOFF hardened. Task ledger T5 pointer updated. Lesson z18 appended. State v7.780→v7.781. |
| D-1133 | state-manager | 2026-06-13 | DEMO-SCOPE DURABILITY BURST (D-1133). DEMO-SCOPE.md created at `.factory/objectives/DEMO-SCOPE.md`. Wired into SESSION-HANDOFF + STATE + task ledger. Lesson z19. STATE v7.781→v7.782. |
| D-1138 | state-manager | 2026-06-13 | T5 PR-LEVEL PASSES 25-27 CHECKPOINT. Hook-bypass governance (D-1134). BPRL-P25-01 MED CLOSED (D-1135). BPRL-P26-01 MED/PG CLOSED (D-1136). Pass 27 CLEAN(strict)=YES (D-1137). Streak 0/3→1/3. PR #186 OPEN. STATE v7.782→v7.783. |
| D-1139 | state-manager | 2026-06-13 | POST-MERGE BURST — PR #185 squash-merged develop@7fd35b77. T5 PR-LEVEL CONVERGED 3/3 strict (passes 27/28/29). POL-14: BC-2.06.019 v1.7 + BC-2.06.020 v1.6 draft→active (active 232→234, draft 5→3). SEC-006/007/008 dispositions. T6 pointer advanced. Lesson z20. STATE v7.783→v7.784. |
| D-1144 | state-manager | 2026-06-13 | **T6 REMOVE-UNCERTAINTY RE-RUN COMPLETE (D-1144).** Mandatory pre-TDD `dclaude:remove-uncertainty` re-run on S-DEMO-MULTI-TENANT-DTU-001. 1 HIGH U-RERUN-001 + 1 MED U-RERUN-002 found and fixed; EXPECTED re-baselined 52→59; story v1.2→v1.3. STORY-INDEX v2.370→v2.371. STATE v7.786→v7.787. |
| D-1145 | state-manager | 2026-06-13 | **T6 API-GAP ADJUDICATION D-1075-API-GAP-001 (D-1145).** `start_instances` zombie-server/port-leak gap found during TDD. Architect adjudicated Option A: return type amended HashMap→`MultiInstanceServers` (#[non_exhaustive] lifecycle handle; socket_map()/shutdown()/Drop graceful drain). Story v1.3→v1.4; BC-2.06.017 v1.1→v1.2; EXPECTED re-baselined 59→60 (+1 MultiInstanceServers E0639 arm; total 8 arms = 7 E0639 + 1 E0004). STORY-INDEX v2.371→v2.372. BC-INDEX v6.44→v6.45. develop_head UNCHANGED f7400f83. NEXT: implementer+test-writer land code; then LOCAL adversary Step 4.5. STATE v7.787→v7.788. | wave-5-e-demo-fidelity | 2026-06-13 |

## Decisions Log

_D-001..D-046 archived: `cycles/phase-3-dtu-wave-2/decisions-archive-d001-d032.md`. D-047..D-174: `cycles/wave-3-multi-tenant/decisions-archive-d047-d114.md`. D-175..D-188: `cycles/wave-3-multi-tenant/burst-log.md`. D-200..D-213: `cycles/wave-4-operations/burst-log.md`. D-432..D-699: `cycles/wave-0-plugin-prereqs/burst-log.md` (D-727 compaction). **D-214..D-320 LOST** — TD-VSDD-058. **D-700..D-1054: `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md`** (D-1056 compaction). **D-1055..D-1123: `cycles/wave-5-e-demo-fidelity/decisions-archive-D1055-D1123.md`** (D-1132 compaction). **D-1124..D-1138: `cycles/wave-5-e-demo-fidelity/decisions-archive-D1124-D1138.md`** (hygiene compaction 2026-06-13)._

| ID | Decision | Rationale | Phase | Date |
|----|----------|-----------|-------|------|
| D-1145 | state-manager | 2026-06-13 | **T6 API-GAP ADJUDICATION D-1075-API-GAP-001 (D-1145).** During T6 TDD implementation, `start_instances` (architect-locked D-1075 API) was found to have no external graceful-shutdown mechanism: returned only `HashMap<String,SocketAddr>`, leaking demo-server instances as detached tasks until process exit (zombie-server/port leak). Red Gate test `test_BC_2_06_017_demo_server_multi_instance_shutdown_clean` was unsatisfiable under the locked API. Architect adjudicated D-1075-API-GAP-001 → **Option A** (production-grade, no deferral): amend locked API so `start_instances` returns new `#[non_exhaustive] MultiInstanceServers` lifecycle handle owning single shared `shutdown_tx` + `task_handles`, with `servers.socket_map()` accessor + `servers.shutdown()` + Drop graceful drain (axum `with_graceful_shutdown`). Mirrors already-correct MultiInstanceHarness pattern. Eliminates the leak; makes the shutdown test satisfiable. User explicitly affirmed adding graceful shutdown. Spec amendments: (A) story-writer: S-DEMO-MULTI-TENANT-DTU-001 v1.3→v1.4 (`start_instances` return type HashMap→MultiInstanceServers; new type block in §Locked API; AC-001/AC-002/Story-Level-Goal → `servers.socket_map()`; EXPECTED re-baselined 59→60 [+1 MultiInstanceServers E0639 arm; now 8 arms = 7 E0639 + 1 E0004]; status remains ready). (B) product-owner: BC-2.06.017 v1.1→v1.2 (Postcondition 1 amended to Ok(MultiInstanceServers) lifecycle handle + shutdown/Drop semantics; EC-017-005 generalized to both handles; Postconditions 2-7 + invariants unchanged). Code + test changes are NEXT (implementer lands MultiInstanceServers + amended start_instances + compile-fail arm; ci.yml EXPECTED 52→60; test-writer updates tests). STORY-INDEX v2.371→v2.372. BC-INDEX v6.44→v6.45. develop_head UNCHANGED f7400f83. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1144 | state-manager | 2026-06-13 | **T6 REMOVE-UNCERTAINTY RE-RUN COMPLETE (D-1144).** Mandatory pre-TDD `dclaude:remove-uncertainty` re-run on S-DEMO-MULTI-TENANT-DTU-001 (D-1110 extension + user_directive_remove_uncertainty). 1 HIGH U-RERUN-001: stale EXPECTED count — story said bump `49→56`; ground-truth ci.yml EXPECTED=52 (grew via S-DEMO-DTU-LIVE-SCENARIO-001-A AC-014 + 001-B since 2026-06-09); re-baselined to `52→59` (+7 delta unchanged: 6 E0639 struct arms + 1 E0004 enum arm); FIXED at story lines 609, 677, 735. 1 MED U-RERUN-002: stale "crate currently does NOT import any prism-dtu-* crate" claim — now imports prism-dtu-common (Story A); REWORDED. CONFIRMED UNCHANGED: all version pins (axum 0.7, tokio 1/full, tempfile 3, anyhow 1, reqwest 0.12); BehavioralClone::start_on no-tls signature; HarnessError #[non_exhaustive]; new files absent; DemoHarness present. Story bumped v1.2→v1.3 (status remains ready; CLEARED FOR TDD DELIVERY). STORY-INDEX v2.370→v2.371. develop_head UNCHANGED f7400f83. No code change (story spec only). | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1143 | state-manager | 2026-06-13 | **PR #186 MERGED + T6 START (D-1143).** `maintenance/lefthook-docs-only-pre-push` squash-merged to develop at `f7400f83` (2026-06-13T21:38:48Z). lefthook.yml only — fail-closed docs-only pre-push skip for `just check` gate. pr-reviewer found 3 fail-closed holes (B1/B2/B3); devops-engineer fixed all 3 (commit 7990965a; 35/35 hardened test cases); pr-reviewer re-review APPROVE; human-approved; CI 43-green; squash-merged. D-1134 bypass-exception remediation RESOLVED/CLOSED. No open PRs. T6 IN PROGRESS: S-DEMO-MULTI-TENANT-DTU-001 (ready v1.2; BC-2.06.017 draft; 8 pts). Mandatory first step: `dclaude:remove-uncertainty` re-run (D-1110 extension). develop_head 7fd35b77→f7400f83. STATE v7.784→v7.785. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1142 | state-manager | 2026-06-13 | SEC-008: ThreatIntelClone poisoned-mutex `.expect` LOW — ACCEPTED (clippy-allowed startup pattern). | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1141 | state-manager | 2026-06-13 | SEC-007: `CrowdstrikeClone::new_with_scenario` missing `#[cfg(feature="fixture-gen")]` LOW — compiler-enforced-safe; anchored to S-DEMO-ENRICHMENT-PIVOT-003. | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1140 | state-manager | 2026-06-13 | SEC-006: `Arc::try_unwrap` panic LOW — DO-NOT-REFLAG; pre-adjudicated INTENTIONAL (SESSION-HANDOFF §4). | wave-5-e-demo-fidelity | 2026-06-13 |
| D-1139 | state-manager | 2026-06-13 | **PR #185 SQUASH-MERGED (D-1139).** T5 PR-LEVEL 3/3 strict CONVERGED (passes 27/28/29). POL-14: BC-2.06.019 v1.7 + BC-2.06.020 v1.6 draft→active (active 232→234). BC-INDEX v6.43→v6.44. STORY-INDEX v2.369→v2.370. develop_head 939f36ce→7fd35b77. Lesson z20. | wave-5-e-demo-fidelity | 2026-06-13 |

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

## Blocking Issues

| ID | Description | Blocker Owner | Since | Status |
|----|-------------|---------------|-------|--------|
| TD-VSDD-005 | vsdd-factory:adversary runtime tool-binding bug — only Read bound at dispatch; general-purpose-as-adversary workaround required | vsdd-factory plugin maintainer | 2026-04-26 | OPEN — housekeeping pause before Wave 3 |

## Historical Content

Current cycle `cycles/wave-5-e-demo-fidelity/`: burst-log.md · decisions-archive-D700-D1054.md · decisions-archive-D1055-D1123.md · **decisions-archive-D1124-D1138.md** · **drift-items-resolved.md** · **phase-5-deferred-findings.md** · frontmatter-cascade-archive.md · session-handoff-archive.md · convergence-trajectory.md · lessons.md. Prior cycles: wave-0-plugin-prereqs/ · wave-3-multi-tenant/ · wave-4-operations/.

_No open PRs. Last merges: PR #185 develop@7fd35b77 (T5 DONE), PR #186 develop@f7400f83 (D-1134 bypass remediation RESOLVED). D-1145: T6 API-gap D-1075-API-GAP-001 adjudicated Option A — MultiInstanceServers; story v1.4, BC-2.06.017 v1.2, EXPECTED 52→60._

## Session Resume Checkpoint (D-1145 — 2026-06-13; STATE v7.788)

**STATE v7.788. CURRENT POSITION: T6 — S-DEMO-MULTI-TENANT-DTU-001 v1.4 in progress; API-gap D-1075-API-GAP-001 adjudicated (D-1145). EXPECTED gate re-baselined: implementer/test-writer MUST use `EXPECTED=52→60` (+8 total: 7 E0639 struct arms + 1 E0004 enum arm) per D-1145. `MultiInstanceServers` (#[non_exhaustive]) is the new `start_instances` return type; `servers.socket_map()` replaces HashMap access; `servers.shutdown()` + Drop graceful drain required. develop HEAD: f7400f83 (UNCHANGED — spec only). BC-INDEX v6.45 (active 234 / draft 3 / retired 6). STORY-INDEX v2.372 (200 stories). VP-INDEX v1.79 (158). policies v1.33. error-taxonomy v1.78. demo_scope_doc: .factory/objectives/DEMO-SCOPE.md. No open PRs.**

**RESUME PROTOCOL (run on fresh session start):**
0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star) + §RESUME SNAPSHOT (latest D-1145). Read `.factory/objectives/DEMO-SCOPE.md` (authoritative full demo scope).
1. `vsdd-factory:factory-worktree-health` (BLOCKING — must pass before reading any state).
2. Verify develop HEAD: `git log --oneline origin/develop | head -1` → expect `f7400f83`.
3. Verify `grep "^version:" .factory/STATE.md` shows `"7.788"`.
4. Parked worktrees: `.worktrees/S-3.09` (FROZEN) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded) — leave alone. Story B worktree `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B` may be cleaned up (merged).
5. Apply lessons (a)–(z20) from `cycles/wave-5-e-demo-fidelity/lessons.md`.
6. **NEXT ACTION: T6 — implementer lands `MultiInstanceServers` + amended `start_instances` + new E0639 compile-fail arm + `ci.yml EXPECTED 52→60`; test-writer updates `test_BC_2_06_017_demo_server_multi_instance_shutdown_clean` and all callers of `start_instances` to use `servers.socket_map()`; then LOCAL adversary Step 4.5.** KEY: story v1.4 (D-1145); BC-2.06.017 v1.2; EXPECTED=60 at merge (NOT 59).
