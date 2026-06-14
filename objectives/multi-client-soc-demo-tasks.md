---
document_type: task-ledger
objective: multi-client-soc-analyst-demo
level: ops
version: "1.25"
producer: state-manager
status: active
timestamp: 2026-06-14T00:00:00Z
related:
  - SESSION-HANDOFF.md §ACTIVE OBJECTIVE
  - .factory/STATE.md
---

# Task Ledger — Multi-Client SOC-Analyst Live Demo

> **SCOPE SOURCE OF TRUTH: `.factory/objectives/DEMO-SCOPE.md`** — the authoritative full demo scope narrative (what is built, what is in convergence, and what the honest gaps are). This ledger is the granular TASK tracker; DEMO-SCOPE.md is the NARRATIVE scope. Read DEMO-SCOPE.md first if you need to understand what the demo includes or where we stand overall.

## Objective

Deliver a multi-client SOC-analyst LIVE DEMO — multiple clients, different sensor combos, REAL per-client data with scenario progression, prism federation, MCP→Claude, end-to-end SOC investigation. TDE deferred.

## Scope Decisions (user, 2026-06-09)

- SOC-analyst demo FIRST. TDE workflow (detection rules, write/action-back containment) DEFERRED — requires `prism-operations` crate + dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).
- REAL per-client data segregation required — NOT just client-targeting/federation routing.

## Scope Expansion (user, 2026-06-09, D-1077)

**A. Continuous generation = SCENARIO PROGRESSION (unfolding attack).** Not just live-append of random events — each per-client scenario must EVOLVE through stages (e.g. CompromisedEndpoint: recon → lateral movement → exfil → containment), with new telemetry surfacing over time. MUST stay deterministic-over-time (same seed + same clock-offset → same timeline) for reproducibility. This is a NEW mechanism not present today (current generators produce one-shot static snapshots; HighChurn models churn as static tombstones, NOT a live feed).

**B. One larger story.** Fold static per-client seeding (BC-2.06.018 baseline) + continuous scenario-progression into a SINGLE larger story. S-DEMO-DTU-DATA-SEEDING-001 expands; will likely be renamed by story-writer to reflect live-scenario scope (e.g. S-DEMO-DTU-LIVE-SCENARIO-001). Anchors MULTIPLE BCs: BC-2.06.018 baseline + a new scenario-progression BC + a new enrichment-correlation BC (pending architect design + PO authorship).

**C. Enrichment DTUs in the live demo.** ThreatIntel + NVD (both static-fixture, NO generator today) must be included in the live demo with SCENARIO-CORRELATED data: the IOCs the progression introduces must resolve in ThreatIntel, and the CVEs on affected devices must resolve in NVD — so the SOC-analyst enrichment workflow is believable. PagerDuty + Jira = response/ticketing DTUs — OUT of current enrichment scope; adjacent to the deferred TDE write-back track per D-1072 (boundary preserved).

**Open obligation (D-1077):** E-DEMO-001 error code must be registered in `.factory/specs/prd-supplements/error-taxonomy.md` (new E-DEMO-NNN namespace; first entry; obligation belongs to error-taxonomy owner; tied to the data-seeding story delivery).

## Progress Summary

Foundations: COMPLETE (reused). Build: 8/15 tasks done; T4 DONE; T4-A DONE; T5 DONE; T6 DONE; T8 DONE; T9 DONE. **D-1110 PRE-TDD remove-uncertainty RE-RUN DONE/CLEAR (D-1161; S-DEMO-004 v1.6). D-1165 2026-06-14: S-DEMO-004 v1.6→v1.7 (LOCAL adversary O-01 spec fix); LAUNCHER ready v1.0 materialized; S-5.02 BCs locked (BC-2.10.004 v2.8/007 v1.5/011 v1.5); T16 ordering corrected (S-1.15 parallel NOT gating PIVOT-001; PIVOT-001 before S-1.14-REDO).** No open PRs. **CURRENT TASK: T10 (S-DEMO-004 ready v1.7; PRE-TDD CLEAR; parallel execution ACTIVE — lanes A-E in spec-prep).**

**~15 core stories in scope (3 merged; 1 delivery-ready; 1 script-ready; 1 not-authored narrative capstone; 4 capability-discovery REQUIRED per D-1162; 5 enrichment REQUIRED per D-1164) + prereq-verifications. See §Complete Story Roadmap below and §ENRICHMENT-REAL (D-1164).**

## CURRENT POINTER

**T10 IN PROGRESS: S-DEMO-004 delivery (ready v1.7; D-1165 spec-only O-01 fix; PRE-TDD CLEAR). PARALLEL EXECUTION ACTIVE (D-1165 2026-06-14): no fixed worktree cap; review-throughput is the practical limiter (target ~3 in LOCAL cascade + 1 at PR-level).** Lane A: S-5.02 (PO-done/finalize-next) → S-5.03 → S-5.04. Lane B: S-3.13 (finalize-needed). Lane C: PIVOT-001 (finalize/research) → [S-1.14-REDO ∥ PIVOT-002] → PIVOT-003 (CRITICAL PATH). Lane D: S-1.15. Lane E: LAUNCHER (T11, ready v1.0). Capstone LAST. Merge-coordination: S-3.13 ↔ PIVOT-001 on prism-query/engine.rs (land constructor-sig change first); infusion trio PIVOT-001→S-1.14-REDO serialize. T1–T9 ALL DONE. No open PRs. D-989+D-1090 autonomy ACTIVE.

## NEXT ACTION (verbatim, for cold resume)

**T10 — per-story 12-gate delivery of S-DEMO-004 (ready v1.6):**

PREREQUISITE (D-1110 standing directive): `dclaude:remove-uncertainty` RE-RUN — **DONE/CLEAR (D-1161 2026-06-14).** 5/6 prior fixes confirmed-correct; 1 dev-dep framing fix applied (prism-dtu-common MODIFY ["dtu"]→["dtu","fixture-gen"]); S-DEMO-004 v1.5→v1.6. Pre-TDD verdict CLEAR. Do NOT re-run.

Execute the 12-gate sequence:
1. `vsdd-factory:worktree-manage create S-DEMO-004`
2. `vsdd-factory:test-writer` — Red Gate stubs (AC-001/004/005/006 are Red Gate; AC-010 gated `#[ignore]` per spec)
3. `vsdd-factory:implementer` — TDD green (boot step 9A per-org adapter registration; MultiInstanceHarness::start(8 entries via HarnessEntry::new); write_overlay_temp_dir(tempdir.path()); content-level ids_org_a ∩ ids_org_c = ∅ assertion via new_with_seed per-org seeds)
4. LOCAL adversary 3-CLEAN strict (BC-5.39.001 D-779; SAP-1 tracing catalog sweep; SAP-2 DTU↔TOML parity)
5. `vsdd-factory:demo-recorder` per-AC
6. Push feature branch to origin (timeout: 600000 or run_in_background; pre-push gate ~14 min cold)
7. `vsdd-factory:pr-manager` — PR create
8. PR-LEVEL 3-CLEAN strict (adversary uses absolute worktree path + directory sanity-check guard) + pr-reviewer APPROVE + security-reviewer CLEAR
9. CI all green
10. Squash-merge to develop
11. Worktree cleanup
12. state-manager post-merge burst (POL-14: BC-3.2.001 if draft→active; develop_head update; STORY-INDEX S-DEMO-004 row → merged; task ledger T10 → done; CURRENT POINTER → T11)

T7 DEPENDENCY NOTE: T7 (data-seeding story) was listed as a T10 blocker in the original ledger. T7 is effectively satisfied by the merged Story A (PR #181) + Story B (PR #185) which together implement BC-2.06.018 + BC-2.06.019 + BC-2.06.020 including new_with_seed per-org seeding. The formal T7 row is annotated as effectively-satisfied below.

Alternatives if T10 blocked: T11 (S-DEMO-LAUNCHER-CONSOLIDATION-001 story-writer), T13 (PO+story-writer: narrative capstone).

**USER AUTHORIZATION (D-1090 2026-06-10): full-autonomous delivery still active. Autonomy envelope: run all gates A→merge autonomously; PAUSE ONLY for §7 spec-to-match-code amendments / genuine product-business decisions / Level-3 escalation / CLAUDE.md edits. D-989 autonomy grant ACTIVE.**

T5 SEQUENCE:

**(1) story-writer MATERIALIZE** — dispatch vsdd-factory:story-writer for S-DEMO-DTU-LIVE-SCENARIO-001-B (Story B v1.0 draft shell at `.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`; scenario progression + enrichment correlation; 7pt; BC-2.06.019+020; ADR-036 v2.2) to materialize full implementation spec.

**CONTRACT-COMPLETENESS FRONT-LOAD (Story-A P6 lesson — REQUIRED before locking spec):** story-writer MUST verify the progression mechanism — deterministic-over-time timeline (same seed+clock-offset → same timeline; NOT random append), stage masks (recon→lateral-movement→exfil→containment), and enrichment correlation (IOCs the progression introduces resolve in ThreatIntel; CVEs on affected devices resolve in NVD) — is FULLY specified in BC-2.06.019/020 + ADR-036. If any design gap exists, surface to orchestrator to route to architect/PO BEFORE locking the story.

Also fold in the 2 Story-A NIT follow-ups during materialization:
- NIT-1 (E-DEMO-004 message reconcile): E-DEMO-004 fires on non-default fixture_set archetype + missing org_id in Story A; reconcile message/trigger when Story B wires `scenario.enabled` (BC-2.06.019 anchor).
- NIT-2 (ScenarioConfig field wiring): `ScenarioConfig` fields (`enabled`/`archetype`/`scenario_start_secs`/`stage_duration_secs`) deserialized-but-unconsumed in Story A — Story B consumes them (BC-2.06.019 anchor).

**(2) remove-uncertainty** — run `dclaude:remove-uncertainty` on materialized Story B spec (standing directive D-1061).

**(3) 12-gate delivery** — vsdd-factory:worktree-manage create S-DEMO-DTU-LIVE-SCENARIO-001-B → vsdd-factory:test-writer → vsdd-factory:implementer → LOCAL adversary 3-CLEAN strict (BC-5.39.001) → demo-recorder → push (timeout: 600000 or run_in_background; pre-push gate ~14 min cold) → pr-manager PR → PR-LEVEL 3-CLEAN strict (adversary uses absolute worktree path + directory sanity-check guard) + pr-reviewer APPROVE + security CLEAR → CI → squash-merge → state-manager post-merge burst (POL-14: BC-2.06.019+020 draft→active).

T6 (S-DEMO-MULTI-TENANT-DTU-001 ready v1.2; BC-2.06.017) is independently deliverable in parallel after T5 materialize step starts (remove-uncertainty already COMPLETE from D-1076).

---

## §ENRICHMENT-REAL (D-1164 — 2026-06-14, User-Directed Scope Decision)

> **USER DECISION D-1164 (2026-06-14):** FULL Option-A infusion framework is REQUIRED before demo recording. Enrichment (ThreatIntel/NVD) must flow through the REAL prism code path the same structural way sensors do. DTU clones are the ONLY substituted element. The demo-server-side `build_clone_pairs` pre-seeding (Story B / BC-2.06.020) is acknowledged as NOT sensor-parity-real — it is SUPERSEDED/COMPLETED by this work. **This block CLOSES TD-PLUGIN-P0-002 (P0 — infusion 100% `unimplemented!()`) upon merge.**

### The Real Enrichment Code Path (Required)

```
PrismQL  | enrich threat_intel(ioc_value)
             ↓
         DataFusion ScalarUDF (registered by prism-query)
             ↓
         InfusionRegistry (prism-spec-engine)
             ↓
         PluginInfusionSource::enrich_single
             ↓
         WASM plugin (.prx) via wasmtime
             ↓
         DTU HTTP endpoint (prism-dtu-threatintel / prism-dtu-nvd)
```

DTU clones are the ONLY substituted element — fully consistent with DTU-EVERYTHING invariant.

### WASM Toolchain Risk + Accepted Contingency

WASM toolchain risk ACCEPTED per D-1164. Contingency (human-directed per Canonical Principle Rule 3): if WASM blocks, `PluginInfusionSource::enrich_single` may fall back to a direct `reqwest` HTTP call to the DTU endpoint. This contingency is TD-anchored to S-1.14-REDO/S-1.15 for replacement at the point those stories complete.

### Required Stories (5 total, REQUIRED, demo-critical-path before T13/T14)

Dependency chain: S-1.15 and S-1.14-REDO are FOUNDATIONAL and can proceed in parallel with the capability-discovery block (T15a-d). PIVOT-001→002→003 is a strict linear chain that follows both foundational stories.

| Story | Role | Status | Points | Depends-on |
|-------|------|--------|--------|------------|
| **S-1.15** | WASM plugin runtime — `PluginInfusionSource` delegates to it | partial-merge (VP-040..043) | 6 | S-1.11 (SATISFIED) |
| **S-1.14-REDO** | Full infusion engine: InfusionLoader + 3-tier cache + all source types (MMDB/CSV/JSON + plugin) | draft/BLOCKED (awaits Wave 0+1 plugin foundation per D-333) | 8 (TBD) | S-WAVE5-PREP-01 (SATISFIED), S-3.02-FOLLOWUP-RUNTIME (SATISFIED) |
| **S-DEMO-ENRICHMENT-PIVOT-001** | plugin-type `InfusionLoader::parse` + `PluginInfusionSource` + DataFusion `ScalarUDF` registration in prism-query | draft v1.1 | 5 | S-1.14 (via S-1.14-REDO) |
| **S-DEMO-ENRICHMENT-PIVOT-002** | `threatintel.infusion.toml` + `nvd.infusion.toml` grounded vs DTU routes + `prism-threatintel-infusion` + `prism-nvd-infusion` WASM `.prx` crates | draft v1.1 | 8 | S-DEMO-ENRICHMENT-PIVOT-001 |
| **S-DEMO-ENRICHMENT-PIVOT-003** | real IOC/CVE field stamping in Cyberint/CrowdStrike DTU fixtures + canonical `\| enrich` pivot-query validation at scenario stage >= 3 | draft v1.8 | 8 | S-DEMO-ENRICHMENT-PIVOT-002 |

**Total enrichment scope: ~35 pts. All 5 stories are REQUIRED before T13 capstone and T14 demo recording.**

### Sequencing (within overall demo roadmap)

```
T10 S-DEMO-004 [CURRENT] → capability-discovery block (T15a-d) [REQUIRED]
                         → S-1.15 + S-1.14-REDO [FOUNDATIONAL; parallelizable with T15a-d]
                         → S-DEMO-ENRICHMENT-PIVOT-001 [after S-1.14-REDO]
                         → S-DEMO-ENRICHMENT-PIVOT-002 [after PIVOT-001]
                         → S-DEMO-ENRICHMENT-PIVOT-003 [after PIVOT-002]
                         → T11 launcher consolidation [parallelizable with enrichment chain]
                         → T13 capstone [LAST — after ALL enrichment stories merged]
                         → T14 demo recording
```

PIVOT-001→002→003 slots AFTER the capability-discovery block and AFTER S-1.14-REDO, parallelizable with the launcher (T11). PIVOT-001/002/003 depend on S-1.14-REDO but NOT on the capability-discovery stories — they can run in parallel once S-1.14-REDO merges.

### Pre-Enrichment Architect Planning Task (REQUIRED before enrichment delivery begins)

Because Full Option A is chosen, the architect MUST determine the exact build order and specifically whether PIVOT-001's subset scope folds into S-1.14-REDO (to avoid double-implementing the plugin-type loader). PIVOT-001 is formally registered as the `forward_subset_implemented_by` of S-1.14-REDO in STORY-INDEX. The architect must adjudicate whether to:
- Deliver S-1.14-REDO first (complete full infusion engine) and fold PIVOT-001 into it, OR
- Deliver PIVOT-001 as the minimal plugin-bridge prerequisite ahead of the full S-1.14-REDO redo

This planning task must complete BEFORE the enrichment delivery sequence begins.

### SESSION-HANDOFF Reconciliation Note

SESSION-HANDOFF.md §ACTIVE OBJECTIVE already lists the PIVOT chain "between capability-discovery and T11" as planned enrichment work. This ledger's §ENRICHMENT-REAL block now brings the ledger into agreement: the canonical placement is REQUIRED (not optional) before T13, sequenced after S-DEMO-004 (T10) + capability-discovery block (T15a-d), parallelizable with the launcher (T11), and dependent on S-1.14-REDO + S-1.15 foundational work completing first.

---

## Complete Story Roadmap

This is the complete set of stories for the multi-client SOC demo. The 12-gate per-story TDD sequence applies to each implementable story; remove-uncertainty runs before each delivery.

| Order | Story ID | Role in demo | Status | Pts | BCs | depends_on | Maps-to-task |
|-------|----------|-------------|--------|-----|-----|------------|-------------|
| 1 — **MERGED** | **S-DEMO-MULTI-TENANT-DTU-001** | Multi-address binding — per-instance distinct DTU sockets enable per-client overlay testing | **merged v1.14** (D-1158 2026-06-14: PR #187 develop@664566e9; LOCAL 11-pass 3-CLEAN strict + PR-LEVEL 10-pass 3-CLEAN strict; BC-2.06.017 v1.10 active per POL-14) | 8 | BC-2.06.017 (active — v1.10) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | T6 DONE |
| 2 — **MERGED** | **S-DEMO-DTU-LIVE-SCENARIO-001-A** | Baseline seeding retrofit — wire seeded generators into demo-server clones for per-client distinct data | **merged v1.5** (D-1089 2026-06-10: PR #181 develop@c287b00d; LOCAL 18-pass 3-CLEAN strict + PR-LEVEL 3-pass 3-CLEAN strict; BC-2.06.018 v1.6 active; ADR-036 v2.2; INV-DISTINCT-DATA-001 proven) | 8 | BC-2.06.018 (active — v1.6) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | T4-A (DONE) |
| 3 — **MERGED** | **S-DEMO-DTU-LIVE-SCENARIO-001-B** | Scenario progression (unfolding attack stages) + enrichment correlation (ThreatIntel/NVD IOC+CVE resolution) | **merged v2.16** (D-1139 2026-06-13: PR #185 squash-merged develop@7fd35b77; LOCAL 13-pass 3-CLEAN strict + PR-LEVEL 29-pass 3-CLEAN strict CONVERGED; BC-2.06.019 v1.7 + BC-2.06.020 v1.6 active per POL-14; ADR-036 v2.3; D-1117: SEC-001 + cyberint CVE↔NVD correlation) | 7 | BC-2.06.019 v1.7 (active) + BC-2.06.020 v1.6 (active) | SATISFIED | T5 DONE |
| 4 | **S-DEMO-004** | Multi-org isolation smoke test — 3-org × mixed-sensor per-client data-distinctness proof | **ready v1.6** (D-1161 2026-06-14: D-1110 PRE-TDD remove-uncertainty RE-RUN CLEAR — 1 dev-dep framing fix: prism-dtu-common MODIFY ["dtu"]→["dtu","fixture-gen"]; 5/6 prior fixes confirmed-correct; PRE-TDD verdict CLEAR. D-1160 2026-06-14: T8+T9 complete — architect+PO reconciled to real-seeding model; story-writer materialized to ready; remove-uncertainty applied; 7 BCs: BC-3.2.001, BC-2.06.014, BC-2.11.005, BC-2.01.013, BC-2.10.001, BC-2.06.017, BC-2.06.018; depends_on 6 edges all SATISFIED; acceptance_criteria_count=10; red_gate_tests=4; VP-148) | 8 | BC-3.2.001, BC-2.06.014, BC-2.11.005, BC-2.01.013, BC-2.10.001, BC-2.06.017, BC-2.06.018 | S-DEMO-001, S-DEMO-002, S-CONFIG-MULTI-TENANT-OVERRIDE-001, S-DEMO-MULTI-TENANT-DTU-001 (SATISFIED), S-DEMO-DTU-LIVE-SCENARIO-001-A (SATISFIED), S-DEMO-DTU-LIVE-SCENARIO-001-B (SATISFIED) | T10 (delivery, in-progress — worktree+test-writer NEXT) |
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | Demo tooling generalization — generalize demo-setup/demo-run/demo-teardown to N orgs; reconcile start-demo.sh vs demo-run.sh launcher overlap | **ready v1.0** (D-1165 2026-06-14: story-writer materialized; 10 ACs; tdd_mode facade; retire start-demo.sh; 5 BCs: BC-2.06.001/012/013/014/017; UX question N-org-config deferred to PR review; scan in flight) | 5 | BC-2.06.001, BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.017 | S-DEMO-003 (SATISFIED) | T11 (remove-uncertainty NEXT → delivery) → T12 |
| 6 (capstone) | **Multi-client SOC-analyst narrative story** (not yet named) | Multi-client SOC investigation walkthrough + demo storyline + demo-recorder evidence per persona — the demo's capstone deliverable | **not-authored** (no story file exists; owner: product-owner + story-writer; authorable after data layer + tooling exist) | TBD | TBD | S-DEMO-DTU-LIVE-SCENARIO-001-B + S-DEMO-004 + S-DEMO-LAUNCHER-CONSOLIDATION-001 | T13 (PO+story-writer, not-started) → T14 |
| 7 — **REQUIRED** | **S-5.02** | Tool routing, errors, and client scoping (MCP client targeting) — capability-discovery: "show client's available sensors" | **in-prep** (D-1165: PO BC reconciliation DONE — BC-2.10.004 v2.8/007 v1.5/011 v1.5 locked; story-writer finalize NEXT: server.rs deltas, Red Gate names, rmcp 1.7 compat, tri-state model scope expansion) | 3 | 3 (BC-2.10.004, BC-2.10.007, BC-2.10.011 — all active-lifecycle/draft-status) | S-5.01-FOLLOWUP-MCP-BOOT (SATISFIED — D-1163) | T15a (required; story-writer finalize-needed before TDD) |
| 8 — **REQUIRED** | **S-5.03** | Resources and Prompts — HARD prerequisite of S-5.04 (S-5.04 depends_on S-5.03 → S-5.02 → S-5.01) | **not-started** (STORY-INDEX v2.384; 4 pts; VP-050) | 4 | 2 (BC-2.08.005, BC-2.08.006) | S-5.02 | T15b (required per D-1162 2026-06-14 — transitive pull-in) |
| 9 — **REQUIRED** | **S-5.04** | Sensor health subsystem — capability-discovery surface for per-client sensor status | **not-started** (STORY-INDEX v2.384; 5 pts; depends_on fixed S-5.04-FIX-001 2026-05-29: S-2.07→S-DEMO-001) | 5 | -- | S-5.03, S-DEMO-001 | T15c (required per D-1162 2026-06-14) |
| 10 — **REQUIRED** | **S-3.13** | Dynamic per-org table availability — capability-discovery surface | **in-prep** (D-1165: needs PO new E-QUERY table-availability code [E-QUERY-001 is parse-error code, new code needed], architect edit-distance-vs-strsim decision, story-writer retarget planner.rs→engine.rs/scoping.rs; 3 pts proxy) | 3 | 3 (proxy BCs; BC-2.16.007/001/BC-2.11.001; PO authorship recommended before ready) | S-3.02 (SATISFIED), S-1.12 (SATISFIED via ConfigManager surface — D-1163) | T15d (required; finalize-needed before TDD; parallel with T15a-b-c) |

**Notes:**
- **D-1162 USER SCOPE DECISION (2026-06-14): Capability-discovery stories promoted optional→REQUIRED.** User stated the optional capability-discovery stories "are not optional." S-5.02, S-3.13, and S-5.04 are now REQUIRED core demo deliverables. S-5.03 is a transitive HARD prerequisite of S-5.04 (S-5.04 depends_on S-5.03 → S-5.02 → S-5.01) — pulled in mandatorily even though not named by the user. See STATE.md D-1162.
- **PREREQ-VERIFICATION — S-5.01 (REQUIRED before T15a):** STORY-INDEX shows S-5.01 ("Server Bootstrap and Tool Registration") as a Wave 5 story (7 pts; not-started in the formal story row). HOWEVER: S-5.01-FOLLOWUP-MCP-BOOT was MERGED (PR #163 develop@e898c3c9 2026-05-29) and delivered the full PrismServer/tool-router/injection-defense surface that S-5.01 originally specified. The formal S-5.01 story row in STORY-INDEX still shows "not-started" — it was NEVER formally closed/merged because S-5.01-FOLLOWUP-MCP-BOOT was the graduation vehicle. Before dispatching T15a (S-5.02 delivery), the orchestrator MUST verify: (a) does S-5.02's depends_on reference S-5.01 or S-5.01-FOLLOWUP-MCP-BOOT, and (b) is S-5.01-FOLLOWUP-MCP-BOOT's merged state recognized as satisfying S-5.02's dep? If S-5.01 formal story row status is still "not-started" but the content is effectively delivered by PR #163, a factory-only story-status correction may be needed (product-owner routes to story-writer). Do NOT assume S-5.01 is an open implementation blocker without verifying.
- **PREREQ-VERIFICATION — S-1.12 (REQUIRED before T15d):** STORY-INDEX shows S-1.12 as `[partial-merge]` (F-AUD-D1-07: HotReloadWatcher unimplemented; F-AUD-D3-01 inverted-polarity test). S-1.12-FOLLOWUP (HotReloadWatcher notify v7 graduation story) remains BLOCKED in STORY-INDEX (awaits Wave 0+1 plugin foundation per D-333; E-CLEANUP-02 class). Before dispatching T15d (S-3.13 delivery), the orchestrator MUST verify: does S-3.13's partial-merge dependency on S-1.12 actually block S-3.13's specific dynamic-table-availability behavior, or does S-3.13's implemented scope not depend on the HotReloadWatcher graduation work? If S-1.12's missing HotReloadWatcher does block S-3.13, then S-1.12-FOLLOWUP unblocking becomes a prerequisite of T15d — a significant scope expansion (the plugin-migration foundation work). The orchestrator should route this determination to the architect.
- **DELIVERY ORDERING (D-1162):**
  - Chain A (MCP capability): S-5.01 verify/satisfy → S-5.02 (T15a) → S-5.03 (T15b) → S-5.04 (T15c)
  - Chain B (query capability): S-3.02 SATISFIED; S-1.12 verify → S-3.13 (T15d, parallel with T15a-b-c after PO authors dedicated BCs)
  - Enrichment pivot (after capability-discovery): PIVOT-001 → PIVOT-002 → PIVOT-003
  - Launcher (independent): T11 S-DEMO-LAUNCHER-CONSOLIDATION-001 (parallel with capability-discovery block)
  - Capstone (LAST): T13 narrative story → T14 demo recording (after ALL prior delivered)
  - S-DEMO-004 (T10) FIRST — in progress; all above sequenced after T10 merge.
- **PREREQ-CONFIRMED (D-1163 2026-06-14 — architect-adjudicated prereq-confirmation verdicts):**
  - **S-5.01 → S-5.02 (T15a) GATE: SATISFIED via metadata reconciliation.** S-5.01's scope (PrismServer/rmcp/tool-router/client-scoping foundation) is on develop@664566e9 via S-5.01-FOLLOWUP-MCP-BOOT (merged PR #163 develop@e898c3c9 2026-05-29). The formal S-5.01 story row was never closed because S-5.01-FOLLOWUP-MCP-BOOT was the graduation vehicle. Verdict: dependency gate SATISFIED via metadata reconciliation — no new story. S-5.01 story file annotated with `effective_merged_by` field (v1.9→v1.10). S-5.02 `depends_on` repointed from `[S-5.01]` to `[S-5.01-FOLLOWUP-MCP-BOOT]` (v1.4→v1.5). T15a UNBLOCKED after this reconciliation.
  - **S-5.02 BC status (STALE NOTE CORRECTED):** S-5.02's BCs (BC-2.10.004, BC-2.10.007, BC-2.10.011) are ALREADY authored/active — the prior STORY-INDEX note "proxy BCs; no dedicated BC yet" was STALE (carried over from S-3.08–S-3.13 wave note; not applicable to S-5.02). BC-2.10.007 provides the full 7-field `ToolError` struct (retryable/retry_after_seconds/suggestion/original_params_valid). No PO BC-authorship needed before TDD for S-5.02.
  - **S-1.12 → S-3.13 (T15d) GATE: SATISFIED, no added scope.** S-3.13 does NOT require the unimplemented `HotReloadWatcher::start`. S-3.13 integrates via the MERGED `ConfigManager` surface (`register_swap_listener`, `process_spec_changes`, `store`/`load`, `add_sensor_spec`). AC-4/AC-5 hot-reload scenarios drive through `add_sensor_spec` + swap-listener callback, not the filesystem watcher. S-3.13's proxy BCs (BC-2.16.007/BC-2.16.001/BC-2.11.001) are sufficient for Red Gate tests. No PO BC-authorship needed before TDD for S-3.13 Red Gate pass (PO authorship recommended before status=ready per STORY-INDEX note, but not a TDD blocker). T15d UNBLOCKED.
  - **Net: core demo story count STAYS at 10.** Neither prereq adds a story. S-5.01 and S-1.12 do NOT become demo-scope stories.
- **DTU-EVERYTHING invariant (D-1163 2026-06-14 — user reaffirmation, binding).** Full text: `.factory/objectives/DEMO-SCOPE.md §Binding Demo Invariant — DTU-EVERYTHING`. Summary: ALL data sources in the live demo run on prism DTU behavioral clones (CrowdStrike/Armis/Claroty/Cyberint + ThreatIntel/NVD). NO real third-party API connections. All remaining demo stories MUST scope against DTU clones. Corollary: infusion/WASM plugin framework (`unimplemented!()`) is NOT on demo critical path — demo enrichment is DTU-clone-served (BC-2.06.020).
- **NIT-1 (Story-B anchor, D-1089):** E-DEMO-004 error message references `scenario.enabled` but Story A fires it on non-default fixture_set archetype + missing `org_id`. The message/trigger should be reconciled when Story B wires `scenario.enabled`. Anchor: S-DEMO-DTU-LIVE-SCENARIO-001-B / BC-2.06.019. Non-blocking for Story A merge.
- **NIT-2 (Story-B anchor, D-1089):** `ScenarioConfig` fields (`enabled`/`archetype`/`scenario_start_secs`/`stage_duration_secs`) are deserialized but unconsumed in Story A. Story B (scenario progression; BC-2.06.019) consumes them. This is a known stub-with-anchor, NOT a defect in Story A. Anchor: S-DEMO-DTU-LIVE-SCENARIO-001-B / BC-2.06.019.
- S-DEMO-LAUNCHER-CONSOLIDATION-001 has a STORY-INDEX row but pts=0 (stub); story-writer materialization at T11 will set real points.
- The narrative capstone story (Order 6) has no ID, no file, no STORY-INDEX row — it is the final authoring step before demo recording (T13).
- Delivery order is sequential for core chain: Orders 4→5+6(launcher)→7→8→9→10(parallel)→11→12→13; Orders 1/2/3 already MERGED.
- **S-DEMO-CYBERINT-INCIDENTS-SEEDING-001 (deferred follow-up, D-1083):** Draft stub registered in STORY-INDEX v2.333. Split off from Story A per LOCAL adversary Pass-2 F-P2-MED-001: Cyberint incidents table is intentionally non-generator-backed in Story A (generator covers alert/asm_asset/cve/ioc only; cyberint.sensor.toml EC-016-013-002). This story adds a Cyberint incidents generator surface + /api/v1/incidents DTU route so the incidents table becomes generator-backed and per-client distinct. Anchored to BC-2.06.018 §Scope Boundary. Depends on Story A merge. Story-writer materialization + PO BC authorship required before dispatch. Not part of the current demo critical path — included in §Complete Story Roadmap as a deferred item below the core 6.

---

## Already Satisfied (DO NOT rebuild — reused merged work)

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** — per-org config overlays
- **S-DEMO-001** — per-org adapters + boot step 9A (multi-org config boot)
- **S-DEMO-002** — single-org E2E + CI harness
- **S-DEMO-003** — demo scripts + prism credential set/delete CLI + operator runbook
- **S-6.20** — multi-clone demo-server
- **S-6.06 through S-6.10** — 4 DTU clones (CrowdStrike, Cyberint, Claroty, Armis)
- **S-3.1.x / S-3.3.x** — org isolation type system + multi-org config boot
- **S-DEMO-QUERY-PUSHDOWN-001** — query pushdown fidelity
- **Per-sensor route fidelity stories** — S-DEMO-CLAROTY-TRAILING-SLASH-001, S-DEMO-CLAROTY-SPEC-PROSE-FIX-001, S-DEMO-CLAROTY-PAGINATION-001, S-DEMO-HARNESS-CLONE-PARITY-001 (all merged Wave 5 Phase C)
- **MCP→Claude (R4)** — works today via stdio

---

## Session Work Already Completed (this session, 2026-06-08/09)

- PR #180 S-DEMO-HARNESS-CLONE-PARITY-001 merged (Phase C complete) — develop@64d34967. [done]
- Capability audit (multi-client-dtu-demo-capability-audit) — verdict partial-significant-gaps, CORE multi-tenant build-on not rebuild. [done]
- Backlog inventory of existing stories. [done]
- North-star objective persisted to STATE.md + SESSION-HANDOFF.md (D-1072). [done]
- Task ledger created (D-1073, this burst). [done]

---

## TASK LEDGER

Status vocab: `not-started` | `in-progress` | `blocked` | `done`

| Task | Status | Owner-agent | Depends-on | Story ref | Done-when |
|------|--------|-------------|------------|-----------|-----------|
| T1 | done | product-owner | — | S-DEMO-MULTI-TENANT-DTU-001 | New multi-address-binding BC authored (BC-2.06.017 Per-DTU-Instance Multi-Address Binding; draft; SS-01; CAP-036; P2; D-1074); Flag-2 resolved (no BC-2.06.014 amendment); EC-003 explicit-error; 5 invariants authored. BC-INDEX v6.01 registered. |
| T2 | done | architect | T1 | S-DEMO-MULTI-TENANT-DTU-001 | OQ-1 resolved: MultiInstanceConfig+InstanceEntry → prism-dtu-demo-server/src/multi_instance.rs; MultiInstanceHarness+HarnessEntry → prism-dtu-harness/src/multi_instance.rs; NOT prism-dtu-common (architect override D-1075). OQ-2 resolved: Box<dyn BehavioralClone>; HarnessError gains DuplicateKey+BindFailure; canonical signature start+socket_map; INV-PERIMETER-001 satisfied. OQ-3 resolved: behavioral_contracts:[BC-2.06.017]; BC-2.06.014 §References only. No ADR. Story defects for T3 flagged (server.rs→multi_instance.rs; [SS-17]→[SS-01]; behavioral_contracts []→[BC-2.06.017]). |
| T3 | done | story-writer | T1, T2 | S-DEMO-MULTI-TENANT-DTU-001 | Story finalized to status:ready v1.2 (D-1076 2026-06-09): behavioral_contracts:[BC-2.06.017]; subsystems:[SS-01]; §File Structure corrected server.rs→multi_instance.rs; §Architecture Mapping corrected; OQ-1/OQ-2 design directives reflected in body/ACs (Box<dyn BehavioralClone>; HarnessError DuplicateKey+BindFailure(Vec<BindError>); canonical signatures start+socket_map; crate placement per D-1075); dclaude:remove-uncertainty applied (8 uncertainties closed: 4 HIGH incl CRIT U-002, 3 MED, 1 LOW); BC-2.06.017 v1.0→v1.1 (2 PO amendments); S-7.01 gate CLEARED. |
| T4 | done | architect+PO+story-writer | — | S-DEMO-DTU-LIVE-SCENARIO-001-A + 001-B | **RECONCILED+COMPLETE (D-1079):** ADR-036 v2.0 substrate reconciliation complete (architect; two-phase retrofit; new_with_seed; generated_records; dual-path routes; canonical org_slug + device IDs; E-DEMO-004/005). BC-2.06.018/019/020 v1.1 corrections complete (PO; substrate reality + canonical IDs). E-DEMO-004/005 registered in error-taxonomy v1.64. Story split materialized (story-writer; user-authorized): original S-DEMO-DTU-LIVE-SCENARIO-001 SUPERSEDED → Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A; 8pt baseline retrofit; ready v1.0; BC-2.06.018; 14 ACs; depends_on S-CONFIG-MULTI-TENANT-OVERRIDE-001 SATISFIED; blocks 001-B) + Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B; 7pt progression+enrichment; draft v1.0; BC-2.06.019+BC-2.06.020; 16 ACs; depends_on A). |
| T4-A | done | per-story delivery | T4 | S-DEMO-DTU-LIVE-SCENARIO-001-A | Story A (baseline seeding retrofit; 8pt; BC-2.06.018; v1.5) implemented + merged PR #181 develop@c287b00d 2026-06-10. LOCAL 18-pass 3-CLEAN strict + PR-LEVEL 3-pass 3-CLEAN strict; security CLEAR; pr-reviewer APPROVE; CI GREEN. BC-2.06.018 v1.6 active (POL-14). INV-DISTINCT-DATA-001 proven. ADR-036 v2.2 full 8-archetype seeding. 21-pass total adversarial effort. |
| T5 | **done** | per-story delivery | T4-A (SATISFIED) | S-DEMO-DTU-LIVE-SCENARIO-001-B | **MERGED PR #185 develop@7fd35b77 2026-06-13.** LOCAL 13-pass 3-CLEAN strict + PR-LEVEL 29-pass 3-CLEAN strict CONVERGED. BC-2.06.019 v1.7 + BC-2.06.020 v1.6 draft→active per POL-14 (D-1139). Story B v2.16; ADR-036 v2.3. D-1117 enhancement arc: SEC-001 + cyberint CVE↔NVD correlation CLOSED. SEC LOW dispositions: SEC-006 do-not-reflag; SEC-007 anchored PIVOT-003; SEC-008 accepted. |
| T6 | **done** | orchestrator-driven per-story delivery | T3 (SATISFIED) + T4-A+T5 (SATISFIED) | S-DEMO-MULTI-TENANT-DTU-001 | **MERGED PR #187 develop@664566e9 2026-06-14 (D-1158).** LOCAL 11-pass 3-CLEAN strict + PR-LEVEL 10-pass 3-CLEAN strict CONVERGED. CI 43/43 green. HIGH F-PR3 routing-bypass paper-fix CLOSED (real prism-sensors E2E FanOutTarget routing test). SEC-001/002/006 CLOSED. TLS+brotli CI fixes. POL-14: BC-2.06.017 v1.10 draft→active. Story v1.14 / BC v1.10. T6 DONE. |
| T7 | done (effectively satisfied) | per-story delivery | T5, T6 | (data-seeding story; satisfied by Story A + Story B) | Per-client data seeding implemented + merged: Story A (BC-2.06.018 new_with_seed; PR #181 develop@c287b00d) + Story B (BC-2.06.019+020 scenario progression; PR #185 develop@7fd35b77) — both merged, both ACTIVE. INV-DISTINCT-DATA-001 proven. org A ≠ org C CrowdStrike ID sets by construction (seed_a ≠ seed_c). T7 formally satisfied by the merged capability stack that T4-A and T5 delivered. |
| T8 | **done** | architect + product-owner | — | S-DEMO-004 | **DONE D-1160 2026-06-14.** S-DEMO-004 reconciled v1.1→v1.3: (1) architect v1.1→v1.2 — 3 depends_on edges added (S-DEMO-MULTI-TENANT-DTU-001 SATISFIED, S-DEMO-DTU-LIVE-SCENARIO-001-A SATISFIED, S-DEMO-DTU-LIVE-SCENARIO-001-B SATISFIED); §DTU-multi-tenancy-scope rewritten to real-seeding model; §AC-006 Design Directive added; no ADR amendment; (2) PO v1.2→v1.3 — BC-2.06.017 + BC-2.06.018 added to behavioral_contracts array (now 7 BCs); §Behavioral Contracts table added; no BC amendment. |
| T9 | **done** | story-writer | T8 | S-DEMO-004 | **DONE D-1160 2026-06-14.** S-DEMO-004 materialized ready v1.3→v1.5: story-writer v1.3→v1.4 (AC-006/007/009 real-seeding bodies; Tasks/File Structure/risk_mitigations propagated; BC-2.10.001 trace gap closed; status draft→ready); remove-uncertainty v1.4→v1.5 (D-1110 first run: 6 fixes — 2 HIGH, 4 MEDIUM; device-ID false-green trap documented; Cargo.toml dev-deps enumerated in §File Structure). contract-completeness COMPLETE. |
| T10 | **in-progress** | per-story delivery | T6, T7 (SATISFIED), T9 (DONE) | S-DEMO-004 | 3-org × mixed-sensor isolation smoke test implemented + merged (P0 proof; per-tenant data verified via ids_org_a ∩ ids_org_c = ∅ content assertion). **D-1165 2026-06-14: S-DEMO-004 v1.6→v1.7 (LOCAL adversary PASS-1 O-01 spec fix; crates_touched + overlay_wiring.rs row). PRE-TDD CLEAR (D-1161; do NOT re-run). PARALLEL EXECUTION ACTIVE (D-1165): lanes A-E in spec-prep concurrently with T10. NEXT: vsdd-factory:worktree-manage create S-DEMO-004 → test-writer.** |
| T11 | **in-prep** | per-story delivery | — | S-DEMO-LAUNCHER-CONSOLIDATION-001 | **D-1165 2026-06-14: LAUNCHER ready v1.0 materialized** (story-writer; 10 ACs; 5 pts; tdd_mode facade; retire start-demo.sh; 5 BCs; scan in flight). remove-uncertainty NEXT → 12-gate TDD. Parallelizable with enrichment chain. |
| T12 | blocked | per-story delivery | T6, T7, T11 | (demo scripts story) | Multi-org demo scripts implemented + merged; `demo-run.sh` stands up N clients with distinct sensors+data. |
| T13 | not-started | product-owner + story-writer | — | (NEW narrative story) | New story authored: multi-client SOC-analyst investigation walkthrough (the demo storyline) to ready. |
| T14 | blocked | demo-recorder + technical-writer | T10, T12, T13 | (narrative story) | SOC investigation walkthrough recorded as demo evidence; DEMO-RUNBOOK.md updated for multi-client; per-persona evidence captured. |
| T15a | **not-started** (REQUIRED — D-1162) | story-writer + per-story delivery | S-5.01 verify (see §Notes PREREQ-VERIFICATION) | S-5.02 | S-5.02 delivered: MCP tool routing + errors + client scoping; remove-uncertainty first; 12-gate TDD. PO authors dedicated BCs before ready. |
| T15b | **not-started** (REQUIRED — D-1162) | story-writer + per-story delivery | T15a | S-5.03 | S-5.03 delivered: Resources and Prompts; HARD prereq of T15c (S-5.04 depends_on S-5.03); remove-uncertainty first; 12-gate TDD. |
| T15c | **not-started** (REQUIRED — D-1162) | story-writer + per-story delivery | T15b | S-5.04 | S-5.04 delivered: Sensor health subsystem (per-client sensor status); remove-uncertainty first; 12-gate TDD. |
| T15d | **not-started** (REQUIRED — D-1162) | story-writer + per-story delivery | S-1.12 verify (see §Notes PREREQ-VERIFICATION) | S-3.13 | S-3.13 delivered: Dynamic per-org table availability; PO authors dedicated BCs before ready; parallel with T15a-b-c after PO dispatch; remove-uncertainty first; 12-gate TDD. |
| **T16-ARCH-PLAN** | **not-started** (REQUIRED before enrichment delivery) | architect | T10, T15a-d (or parallel) | architect planning (no story file) | Architect determines exact build order for Full Option-A enrichment: adjudicate whether PIVOT-001 scope folds into S-1.14-REDO (avoid double-implementing plugin-type loader). Ruling MUST precede T16a dispatch. |
| **T16-FOUND-A** | **not-started** (REQUIRED — D-1164; CORRECTED D-1165) | per-story delivery | T16-ARCH-PLAN (PARALLEL with T15a-d, NOT a gate) | S-1.15 | **D-1165 ARCHITECT CORRECTION: S-1.15 is NOT a gate before PIVOT-001.** `PluginRuntime::enrich_single` is operational on develop; S-1.15's remaining work (TD-PLUGIN-P0-008 action-dispatch) runs IN PARALLEL with PIVOT-001, not before it. S-1.15 WASM plugin runtime delivered: `PluginInfusionSource` delegating to wasmtime; partial-merge unimplemented stubs replaced; VP-040/041/042/043 green; 12-gate TDD + remove-uncertainty. |
| **T16-FOUND-B** | **not-started** (REQUIRED — D-1164; CORRECTED D-1165) | per-story delivery | T16a (AFTER PIVOT-001, NOT before) | S-1.14-REDO | **D-1165 ARCHITECT CORRECTION: PIVOT-001 builds BEFORE S-1.14-REDO (forward-subset relationship).** S-1.14-REDO infusion engine delivered: InfusionLoader + InfusionRegistry + 3-tier cache + MMDB/CSV/JSON + plugin source types; closes TD-PLUGIN-P0-002 (P0) upon merge; 12-gate TDD + remove-uncertainty. |
| **T16a** | **not-started** (REQUIRED — D-1164; CORRECTED D-1165) | per-story delivery | T16-ARCH-PLAN (parallel with T16-FOUND-A) | S-DEMO-ENRICHMENT-PIVOT-001 | **D-1165 ORDERING: T16a (PIVOT-001) runs in parallel with T16-FOUND-A (S-1.15), BEFORE T16-FOUND-B (S-1.14-REDO).** DataFusion async-UDF execution research in flight; story-writer to fix 4 implementer-traps before TDD: removed post_return, enrich_single signature/PluginError, async-UDF registration path, NullSource wiring. PIVOT-001 delivered: plugin-type `InfusionLoader::parse` + `PluginInfusionSource` + DataFusion `ScalarUDF` registration in prism-query; 12-gate TDD + remove-uncertainty. |
| **T16b** | **not-started** (REQUIRED — D-1164) | per-story delivery | T16a | S-DEMO-ENRICHMENT-PIVOT-002 | PIVOT-002 delivered: `threatintel.infusion.toml` + `nvd.infusion.toml` grounded vs DTU route surfaces; `prism-threatintel-infusion` + `prism-nvd-infusion` WASM `.prx` plugin crates calling DTU HTTP endpoints; 12-gate TDD + remove-uncertainty. |
| **T16c** | **not-started** (REQUIRED — D-1164) | per-story delivery | T16b | S-DEMO-ENRICHMENT-PIVOT-003 | PIVOT-003 delivered: real IOC/CVE field stamping in Cyberint/CrowdStrike DTU fixtures + canonical `\| enrich threat_intel(ioc_value)` / `\| enrich nvd(device_cves_first)` pivot-query validation at demo server scenario stage >= 3; TD-PLUGIN-P0-002 CLOSED (all infusion code merged). 12-gate TDD + remove-uncertainty. |

---

## Resume Note

Cold-start agents: execute the **CURRENT POINTER's NEXT ACTION** above. After completing a task, state-manager flips its Status to `done`, advances CURRENT POINTER to the next unblocked task, and bumps this ledger's version.

Per-story delivery tasks (T6, T7, T10, T12) follow the canonical 12-gate per-story sequence:
1. `dclaude:remove-uncertainty` (standing directive — ALWAYS first)
2. `vsdd-factory:worktree-manage create <STORY-ID>`
3. `vsdd-factory:test-writer` — stubs + failing Red Gate tests
4. `vsdd-factory:implementer` — TDD green
5. LOCAL adversary 3-CLEAN strict (BC-5.39.001 D-779)
6. `vsdd-factory:demo-recorder` per-AC
7. Push feature branch to origin
8. `vsdd-factory:pr-manager` — PR create
9. PR-LEVEL adversary 3-CLEAN strict + pr-reviewer APPROVE + security-reviewer CLEAR
10. CI all green
11. Squash-merge to develop
12. Worktree cleanup + state-manager post-merge burst

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.25 | 2026-06-14 | state-manager | D-1165 CONSOLIDATION BURST — PARALLEL-EXECUTION KICKOFF + spec locks. (1) PARALLEL EXECUTION (D-1165): worktree cap lifted per user directive; review-throughput is practical limiter (~3 LOCAL + 1 PR-level). Canonical lanes: A (S-5.02→S-5.03→S-5.04), B (S-3.13), C (PIVOT-001→[S-1.14-REDO∥PIVOT-002]→PIVOT-003, CRITICAL), D (S-1.15), E (LAUNCHER). Capstone LAST. (2) T16 ORDERING CORRECTED (2 architect findings supersede D-1164 T16-FOUND-A gating): (a) S-1.15 is NOT a gate before PIVOT-001 — `PluginRuntime::enrich_single` is operational; S-1.15's TD-PLUGIN-P0-008 action-dispatch runs PARALLEL with PIVOT-001; (b) PIVOT-001 builds BEFORE S-1.14-REDO (forward-subset); T16a parallel with T16-FOUND-A; T16-FOUND-B after T16a. (3) Pre-TDD scan status: S-5.02 PO-done/finalize-next (server.rs deltas, Red Gate names, rmcp 1.7, tri-state scope); S-3.13 finalize-needed (PO new E-QUERY code, architect strsim decision, story-writer retarget engine.rs/scoping.rs); PIVOT-001 finalize/research (DataFusion async-UDF in flight; 4 implementer-traps to fix). (4) LAUNCHER ready v1.0 (scan in flight); S-1.15 lane owes scoping pass. (5) S-5.02 BC reconciliation DONE: BC-2.10.004 v2.8/007 v1.5/011 v1.5 locked. S-DEMO-004 v1.6→v1.7 (O-01 spec fix). §Complete Story Roadmap rows 5/7/10 updated; T10/T11/T16-FOUND-A/T16-FOUND-B/T16a rows corrected. CURRENT POINTER = T10 in-progress + parallel multi-lane. develop_head 664566e9 UNCHANGED. active_contracts 235 / draft_contracts 2 / total_stories 200 UNCHANGED. BC-INDEX v6.54→v6.55. STORY-INDEX v2.384→v2.385. Ledger v1.24→v1.25. |
| 1.24 | 2026-06-14 | state-manager | D-1164: USER SCOPE DECISION — FULL Option-A infusion framework REQUIRED before demo recording (real sensor-parity enrichment). (1) §ENRICHMENT-REAL block added: 5 Option-A stories (S-1.15 + S-1.14-REDO + PIVOT-001/002/003) enumerated as REQUIRED demo-critical-path before T13/T14; dependency structure, sequencing, WASM-risk/reqwest-contingency, and pre-enrichment architect planning task recorded. (2) Progress Summary updated: core demo story count 10 → ~15. (3) TASK LEDGER: T16-ARCH-PLAN + T16-FOUND-A + T16-FOUND-B + T16a + T16b + T16c added (all not-started, REQUIRED per D-1164). (4) SESSION-HANDOFF reconciliation: canonical placement recorded — enrichment-real block REQUIRED before T13, after T10+T15a-d, parallelizable with T11. (5) Ledger note: closes TD-PLUGIN-P0-002 (P0) upon PIVOT-003/T16c merge. CURRENT POINTER = T10 UNCHANGED. develop_head 664566e9 UNCHANGED. active_contracts/draft_contracts/total_stories UNCHANGED (stories already registered). BC-INDEX UNTOUCHED. Ledger version 1.23→1.24. |
| 1.23 | 2026-06-14 | state-manager | D-1163: PREREQ-CONFIRMED burst — (1) S-5.01→S-5.02 dep gate SATISFIED via metadata reconciliation: S-5.01 `effective_merged_by` S-5.01-FOLLOWUP-MCP-BOOT (PR #163 develop@e898c3c9) annotated in story file v1.9→v1.10; S-5.02 `depends_on` repointed to S-5.01-FOLLOWUP-MCP-BOOT (v1.4→v1.5); STORY-INDEX S-5.01+S-5.02 rows annotated; stale "proxy BCs; no dedicated BC yet" note on S-5.02 CORRECTED (BC-2.10.004/007/011 are dedicated authored/active BCs — no PO authorship needed before TDD). (2) S-1.12→S-3.13 dep gate SATISFIED — S-3.13 integrates via MERGED ConfigManager surface, not HotReloadWatcher; no added scope. (3) Net: core demo story count stays 10. (4) DTU-EVERYTHING binding invariant recorded: DEMO-SCOPE.md §Binding Demo Invariant added (v1.2→v1.3); cross-ref in this ledger §Notes. CURRENT POINTER = T10 UNCHANGED. develop_head 664566e9 UNCHANGED. active_contracts/draft_contracts/total_stories/BC-INDEX UNCHANGED. Ledger version 1.22→1.23.  |
| 1.22 | 2026-06-14 | state-manager | D-1162: USER SCOPE DECISION — capability-discovery stories promoted optional→REQUIRED (2026-06-14). User stated S-5.02/S-3.13/S-5.04 "are not optional." S-5.03 added as transitive HARD prereq of S-5.04 (S-5.04 depends_on S-5.03→S-5.02→S-5.01). §Complete Story Roadmap: S-5.02/S-5.03/S-5.04/S-3.13 moved from "optional" to REQUIRED rows 7-10. §TASK LEDGER: T15 replaced by T15a (S-5.02)/T15b (S-5.03)/T15c (S-5.04)/T15d (S-3.13) all REQUIRED. Delivery ordering recorded in §Notes: Chain A S-5.01-verify→S-5.02→S-5.03→S-5.04; Chain B S-1.12-verify→S-3.13 (parallel after PO BCs). PREREQ-VERIFICATION obligations recorded for S-5.01 (effective delivery via PR #163 S-5.01-FOLLOWUP-MCP-BOOT; formal story row still not-started) and S-1.12 (partial-merge; S-1.12-FOLLOWUP BLOCKED). Core demo stories 6→10 (S-DEMO-004+launcher+narrative+S-5.02+S-5.03+S-5.04+S-3.13). CURRENT POINTER = T10 UNCHANGED. develop HEAD UNCHANGED 664566e9. active_contracts/draft_contracts/total_stories UNCHANGED (no new stories authored — these stories already have STORY-INDEX rows). Ledger version 1.21→1.22. |
| 1.21 | 2026-06-14 | state-manager | D-1161: T10 PRE-TDD remove-uncertainty RE-RUN DONE/CLEAR (2026-06-14). D-1110 second run on S-DEMO-004 v1.5: 5 of 6 prior fixes CONFIRMED-CORRECT; 1 residual mis-framing caught — prism-bin Cargo.toml `prism-dtu-common` dev-dep framed as ADD with `["fixture-gen"]` but is ALREADY present with `["dtu"]` and must be MODIFY to `["dtu","fixture-gen"]` (independent features). Story-writer applied one-line correction v1.5→v1.6. PRE-TDD verdict CLEAR. T10 CURRENT POINTER sub-step advanced from "PRE-TDD re-run" to "vsdd-factory:worktree-manage create S-DEMO-004 → test-writer". T10 status not-started→in-progress. §Complete Story Roadmap row 4 updated to ready v1.6. Progress Summary updated (D-1110 PRE-TDD DONE/CLEAR note added). STORY-INDEX v2.383→v2.384. develop HEAD UNCHANGED 664566e9. active_contracts/draft_contracts UNCHANGED (235/2). Ledger version 1.20→1.21. |
| 1.20 | 2026-06-14 | state-manager | D-1160: T8+T9 DONE — S-DEMO-004 reconciled+materialized+remove-uncertainty to ready v1.5 (2026-06-14). T8: architect v1.1→v1.2 (3 depends_on edges added, all SATISFIED: S-DEMO-MULTI-TENANT-DTU-001 PR #187, 001-A PR #181, 001-B PR #185; §DTU-multi-tenancy-scope real-seeding; §AC-006 Design Directive; no ADR amendment). T8-PO v1.2→v1.3 (BC-2.06.017+BC-2.06.018 added to behavioral_contracts; §BC table added; 5→7 BCs; no BC amendment). T9-story-writer v1.3→v1.4 (AC-006/007/009 real-seeding bodies; File Structure+Tasks+risk_mitigations propagated; BC-2.10.001 trace gap closed; status draft→ready). remove-uncertainty v1.4→v1.5 (6 fixes: 2 HIGH, 4 MEDIUM). T7 annotated effectively-satisfied (Story A + Story B deliver the seeding substrate). CURRENT POINTER advanced to T10. §Complete Story Roadmap row 4 updated to ready v1.5. NEXT ACTION updated verbatim for T10 cold-resume including D-1110 PRE-TDD RE-RUN prerequisite. STORY-INDEX v2.382→v2.383. develop HEAD UNCHANGED 664566e9. active_contracts/draft_contracts UNCHANGED (235/2). Ledger version 1.19→1.20. |
| 1.19 | 2026-06-14 | state-manager | D-1158: T6 DONE — PR #187 squash-merged develop@664566e9 2026-06-14. LOCAL 11-pass 3-CLEAN strict + PR-LEVEL 10-pass 3-CLEAN strict CONVERGED. CI 43/43 green. HIGH F-PR3 routing-bypass CLOSED; SEC-001/002/006 CLOSED; TLS+brotli CI fixed. POL-14: BC-2.06.017 v1.10 draft→active. T6 status in-progress→done. CURRENT POINTER advanced to T8 (architect+PO: S-DEMO-004). §Complete Story Roadmap row 1 updated to merged. Progress Summary updated (6/15 tasks done). BC-INDEX v6.53→v6.54. STORY-INDEX v2.381→v2.382. develop HEAD f7400f83→664566e9. STATE v7.800→v7.801. Ledger version 1.18→1.19. |
| 1.18 | 2026-06-13 | state-manager | D-1144: T6 mandatory pre-TDD remove-uncertainty re-run DONE. S-DEMO-MULTI-TENANT-DTU-001 v1.2→v1.3 (status remains ready). U-RERUN-001 HIGH fixed (EXPECTED re-baselined from stale 49→56 to correct 52→59; +7 delta unchanged; story lines 609/677/735 corrected). U-RERUN-002 MED fixed (stale prism-dtu-* no-import claim reworded). All version pins + signatures confirmed unchanged. STORY-INDEX v2.370→v2.371. develop HEAD UNCHANGED f7400f83. CURRENT POINTER sub-step advanced to vsdd-factory:deliver-story. NEXT ACTION updated to deliver-story (no longer remove-uncertainty). Ledger version 1.17→1.18. |
| 1.17 | 2026-06-13 | state-manager | D-1143: PR #186 MERGED develop@f7400f83 (lefthook fail-closed docs-only pre-push; 43-green CI; 35/35 test cases; D-1134 bypass-exception remediation RESOLVED/CLOSED). No open PRs. T6 CURRENT POINTER updated to IN PROGRESS. Progress Summary and CURRENT POINTER updated. develop HEAD 7fd35b77→f7400f83. STATE v7.784→v7.785. Ledger version 1.16→1.17. |
| 1.16 | 2026-06-13 | state-manager | D-1139: T5 DONE — PR #185 squash-merged develop@7fd35b77. T5 status in-progress→done. T6 status blocked→in-progress. CURRENT POINTER advanced to T6 (S-DEMO-MULTI-TENANT-DTU-001; remove-uncertainty MANDATORY RE-RUN → TDD delivery). Story Roadmap row 3 updated to merged. Progress Summary updated (5/15 tasks done). BC-2.06.019 v1.7 + BC-2.06.020 v1.6 draft→active (POL-14). develop HEAD 939f36ce→7fd35b77. BC-INDEX v6.43→v6.44. STORY-INDEX v2.369→v2.370. STATE v7.783→v7.784. Ledger version 1.15→1.16. |
| 1.15 | 2026-06-13 | state-manager | D-1132: Zero-context resume hardening. T5 pointer updated (streak 0/3). No task status changes. Ledger version 1.14→1.15. |
| 1.14 | 2026-06-10 | state-manager | D-1091: Review-cycle mid-cycle checkpoint. T5 PAUSED — user-directed full-codebase review (2026-06-10) interrupted T5 before story-writer dispatch. CURRENT POINTER + NEXT ACTION gated on review-cycle completion (3 fix-branch cascades to 3-CLEAN strict → PINNED merges QRY→MCP→DTU → 16-item register burst per SESSION-HANDOFF §5). T5 sequence content UNCHANGED (resumes verbatim after register burst per D-1090 envelope). No task status changes. STATE v7.741→v7.742. Ledger version 1.13→1.14. |
| 1.13 | 2026-06-10 | state-manager | D-1090: Zero-context resume durability hardening for Story B (T5). (1) Local develop branch fast-forwarded to c287b00d confirmed (no SHA drift — note recorded for fresh sessions). (2) USER AUTHORIZATION recorded: full-autonomous materialize+deliver of T5 (Story B); autonomy envelope identical to Story A (D-989): run all gates A→merge autonomously; PAUSE only for §7/product-business/Level-3/CLAUDE.md. (3) NEXT ACTION augmented with contract-completeness front-loading step (story-writer must verify progression mechanism + stage masks + enrichment correlation fully specified in BC-2.06.019/020 + ADR-036 BEFORE locking spec; surface gaps to orchestrator for architect/PO routing). (4) Two Story-A NIT follow-ups folded into NEXT ACTION (NIT-1 E-DEMO-004 message reconcile; NIT-2 ScenarioConfig field wiring — both anchored to BC-2.06.019). (5) Operational lessons from cycles/wave-5-e-demo-fidelity/lessons.md folded into SESSION-HANDOFF resume protocol (3 Story-A process-gaps: adversary worktree-path-guard; push timeout 600s; exhaustive sibling-sweep). No task status changes; no code/spec/BC/count changes. STATE v7.740→v7.741. Ledger version 1.12→1.13. |
| 1.12 | 2026-06-10 | state-manager | D-1089: T4-A DONE — PR #181 squash-merged develop@c287b00d. BC-2.06.018 v1.6 active (POL-14). T4-A status in-progress→done. CURRENT POINTER advanced to T5 (S-DEMO-DTU-LIVE-SCENARIO-001-B UNBLOCKED). §Complete Story Roadmap Story A row updated to merged v1.5. §TASK LEDGER T4-A row updated to done. NEXT ACTION updated verbatim. Progress Summary updated (4/15 tasks done). Two NIT follow-up anchors added to §Notes (NIT-1 E-DEMO-004 message/trigger; NIT-2 ScenarioConfig stub). T5 unblocked: story-writer materializes Story B spec → remove-uncertainty → 12-gate delivery. BC-INDEX v6.09→v6.10. STORY-INDEX v2.337→v2.338. STATE v7.739→v7.740. Ledger version 1.11→1.12. |
| 1.11 | 2026-06-10 | state-manager | D-1088: T4-A LOCAL 3-CLEAN STRICT CONVERGENCE durability checkpoint. T4-A status note updated to "LOCAL CONVERGED (18-pass cascade; 3-CLEAN strict P16/17/18); demo+PR gates in progress". T4-A row status not-started→in-progress (done only at squash-merge per convention). CURRENT POINTER updated. NEXT ACTION updated verbatim. Progress Summary updated. Spec artifact versions UNCHANGED (ADR-036 v2.2, BC-2.06.018 v1.5, story v1.5, BC-INDEX v6.09, STORY-INDEX v2.337 — no bumps; state-narrative checkpoint only). STATE v7.738→v7.739. Ledger version 1.10→1.11. |
| 1.10 | 2026-06-09 | state-manager | D-1083: Mid-cascade spec-consistency burst. S-DEMO-CYBERINT-INCIDENTS-SEEDING-001 deferred follow-up added to §Notes (anchored to BC-2.06.018 §Scope Boundary; split off from Story A per LOCAL Pass-2 F-P2-MED-001; Cyberint incidents table intentionally non-generator-backed in Story A; draft stub registered in STORY-INDEX v2.333). BC-INDEX v6.05→v6.06. STORY-INDEX v2.332→v2.333. STATE v7.733→v7.734. Ledger version 1.9→1.10. |
| 1.9 | 2026-06-09 | state-manager | D-1082: Complete story roadmap enumerated (user-directed — "include all the stories we are going to work on"). §Complete Story Roadmap table added (9 stories: 6 core + 3 optional). Progress Summary updated to reference roadmap. SESSION-HANDOFF §ACTIVE OBJECTIVE Build Sequence augmented with full enumerated story list + source-of-truth pointer to §Complete Story Roadmap. STATE v7.732→v7.733 (D-1082 decision row). All statuses verified against STORY-INDEX v2.332: S-DEMO-MULTI-TENANT-DTU-001 ready v1.2 ✓; S-DEMO-DTU-LIVE-SCENARIO-001-A ready v1.1 ✓; S-DEMO-DTU-LIVE-SCENARIO-001-B draft v1.0 ✓; S-DEMO-004 not-in-STORY-INDEX (draft/stub per T8) ✓; S-DEMO-LAUNCHER-CONSOLIDATION-001 draft stub pts=0 ✓; narrative capstone not-authored ✓; S-5.02/S-3.13/S-5.04 not-started ✓. No story files authored, no BCs/counts changed — enumeration/durability only. Ledger version 1.8→1.9. |
| 1.8 | 2026-06-09 | state-manager | D-1081: Zero-context resume durability hardening (user-directed). No task-status changes — bookkeeping only. sprint-state.yaml current_story fixed to point at T4-A Story A. SESSION-HANDOFF snapshot refreshed + §7 checklist expected values corrected. Coherence sweep confirmed ledger agrees with STATE v7.732/develop 64d34967/BC counts 250/235/6/total_stories 188. Ledger version 1.7→1.8. |
| 1.7 | 2026-06-09 | state-manager | D-1080: Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A) re-validation via dclaude:remove-uncertainty CONFIRMED SOUND. ADR-036 v2.0 substrate design correct; all mechanism/wiring corrections applied (U-A-01..U-A-10): gen_seeded_rng symbol; CrowdStrike load_host_ids()/load_host_details() fallback; GenOpts::default() syntax; demo-server Cargo.toml deps; Armis fallible; non-exhaustive-violation crate dep; per-clone generate() divergence. ADR-036 v2.0→v2.1 (architect). Story A v1.0→v1.1 (story-writer). DRIFT-SLUG-FORMAT-BC34004-001 registered (non-blocking). T4-A status: validated + delivery-ready (remove-uncertainty COMPLETE). CURRENT POINTER updated: T4-A 12-gate TDD NEXT. NEXT ACTION updated verbatim. ARCH-INDEX v2.117→v2.118. STORY-INDEX v2.331→v2.332. STATE v7.730→v7.731. |
| 1.6 | 2026-06-09 | state-manager | D-1079: T4 RECONCILED+COMPLETE. ADR-036 v2.0 substrate reconciliation complete (architect; two-phase retrofit: new_with_seed + generated_records + dual-path routes; canonical org_slug=hex(org_id[0..4]); device ID dev-{8hex}-{seed}-{n}; CloneConfig.org_id; E-DEMO-004/005). BC-2.06.018/019/020 v1.0→v1.1 (PO; substrate reality corrections). error-taxonomy v1.63→v1.64 (E-DEMO-004+005 added). Story split materialized (story-writer; user-authorized): original S-DEMO-DTU-LIVE-SCENARIO-001 SUPERSEDED → Story A (001-A; 8pt; ready; BC-2.06.018; 14 ACs; blocks 001-B) + Story B (001-B; 7pt; draft; BC-2.06.019/020; 16 ACs; depends_on A). T4 status: in-progress→done. T4-A row added (Story A delivery; not-started; NEXT UNBLOCKED). T5 updated (Story B; blocked on T4-A). CURRENT POINTER advanced to T4-A. NEXT ACTION updated verbatim. STORY-INDEX v2.330→v2.331. BC-INDEX v6.04→v6.05. ARCH-INDEX v2.116→v2.117. total_stories 188. Progress: 3+T4/15 done; Story A delivery NEXT. |
| 1.5 | 2026-06-09 | state-manager | D-1078: T4 design substantially complete. ADR-036 confirmed in ARCH-INDEX v2.116. BC-2.06.019 (scenario progression, 5 invariants) + BC-2.06.020 (enrichment correlation, 6 invariants) registered in BC-INDEX v6.04. E-DEMO-001/002/003 confirmed in error-taxonomy v1.63. T4 CURRENT POINTER remains (story-writer assembles S-DEMO-DTU-LIVE-SCENARIO-001 retry pending — 2 transient socket drops). NEXT ACTION updated verbatim. T4 done-when updated. Progress: 3/15 done; design artifacts durable. |
| 1.4 | 2026-06-09 | state-manager | D-1077: User-directed scope expansion of multi-client SOC demo (A/B/C). BC-2.06.018 registered (draft; PO-authored; SS-01; CAP-036; P2). Scope Expansion section added. T4 status not-started→in-progress; T4 done-when updated to reflect EXPANDED scope (seeding DECIDED; progression + enrichment PENDING architect design). T5 done-when updated to single-larger-live-scenario-story. NEXT ACTION updated verbatim. E-DEMO-001 obligation recorded. CURRENT POINTER remains T4. Progress: 3/15 done. |
| 1.3 | 2026-06-09 | state-manager | D-1076: T3 done (story-writer finalized S-DEMO-MULTI-TENANT-DTU-001 to status:ready v1.2; dclaude:remove-uncertainty closed 8 uncertainties (4 HIGH incl CRIT U-002) before TDD; BC-2.06.017 v1.1 (2 PO amendments); architect reconciliation extend-D-1075 no-ADR; S-7.01 gate CLEARED); CURRENT POINTER advanced to T4 (product-owner + architect decide per-client data seeding approach). Progress: 3/15 done. |
| 1.2 | 2026-06-09 | state-manager | D-1075: T2 done (architect adjudication OQ-1/OQ-2/OQ-3 complete; no ADR); CURRENT POINTER advanced to T3 (story-writer finalizes S-DEMO-MULTI-TENANT-DTU-001 to status:ready). |
| 1.1 | 2026-06-09 | state-manager | D-1074: T1 done (BC-2.06.017 authored + registered in BC-INDEX v6.01); CURRENT POINTER advanced to T2 (architect adjudicates OQ-1/OQ-2/OQ-3). |
| 1.0 | 2026-06-09 | state-manager | Initial task ledger created (D-1073). |
