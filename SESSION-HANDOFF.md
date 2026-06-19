---
document_type: session-handoff
level: ops
version: "7.875"
status: current
timestamp: 2026-06-19T09:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-1242 (2026-06-19): COMPREHENSIVE ZERO-CONTEXT RESTART SNAPSHOT. Both demo-blocking lanes MERGED (S-5.03 PR #194 + PIVOT-002 PR #195). PrismQL-onboarding DESIGN PACKAGE committed (ADR-041 v1.1 + 7 BCs + E-QUERY-038 + S-DEMO-PRISMQL-ONBOARDING-001 draft). develop_head UNCHANGED 9114e028. STATE v7.874→v7.875.**
>
> **PRIORITY READ ORDER:** Read §ACTIVE OBJECTIVE (North Star) FIRST, then **§RESUME SNAPSHOT D-1242** (authoritative zero-context restart protocol; supersedes D-1241). STATE.md frontmatter (`develop_head`, `current_step`) is the secondary authoritative source. All prior D-1101..D-1241 notes SUPERSEDED.
> **SOURCE-OF-TRUTH FOR CURRENT PIPELINE POSITION:** §RESUME SNAPSHOT D-1242 (below) + STATE.md frontmatter. `.factory/objectives/DEMO-SCOPE.md` is the demo SCOPE/NARRATIVE reference — not the live pipeline tracker.
> develop HEAD `9114e028` (PIVOT-002 squash @6c367356 + D-1178 CLAUDE.md count bump 76→79 @9114e028; 2026-06-19; D-1242 docs-only burst — develop UNCHANGED). factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'` (do not hard-code). STATE v7.875.

---

## §RESUME SNAPSHOT — D-1242 (2026-06-19 — COMPREHENSIVE ZERO-CONTEXT RESTART SNAPSHOT; develop_head 9114e028 UNCHANGED; STATE v7.875)

> **D-1242 burst (2026-06-19).** DOCS/STATE-ONLY burst — develop_head UNCHANGED at 9114e028. Both demo-blocking lanes MERGED: S-5.03 PR #194 develop@85ac7b06 (D-1238) + PIVOT-002 PR #195 develop@9114e028 (D-1240). PrismQL-onboarding DESIGN PACKAGE committed (D-1241): ADR-041 v1.1 + 7 BCs (BC-2.10.012/013/014 draft + BC-2.11.016/017/018 draft) + 2 BC amendments + E-QUERY-038 + S-DEMO-PRISMQL-ONBOARDING-001 draft (13 pts). Task ledger advanced to v1.36. D-1236..D-1241 SUPERSEDED by D-1242.

---

### ZERO-CONTEXT RESTART PROTOCOL D-1242 (run in this order; no prior context needed)

**Step 0.** Read this D-1242 snapshot first. It is authoritative. Do NOT act on any other prior context.

**Step 1.** Run `vsdd-factory:factory-worktree-health` (devops-engineer). **BLOCKING** — do not proceed until it passes.

**Step 2.** Verify develop HEAD:
```bash
git log --oneline -1 origin/develop
```
Expected: `9114e028` (PIVOT-002 squash @6c367356 + D-1178 CLAUDE.md count bump 76→79; 2026-06-19). If newer, use live HEAD as authoritative.

**Step 3.** Confirm open PRs:
```bash
gh pr list --state open --base develop
```
Expected: NO open PRs (S-5.03 PR #194 MERGED, PIVOT-002 PR #195 MERGED; no new PRs from docs-only D-1242 burst).

**Step 4.** Check worktree state:
```bash
git worktree list
```
Expected mounted worktrees: main repo + `.factory` (factory-artifacts) + `.worktrees/S-3.09` (FROZEN) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded). Note: `.worktrees/S-5.03` is a stale worktree (merged — devops-engineer may prune). Leave S-3.09 and W3-FIX-S307-001 alone.

**Step 5.** Read PINNED STATE table below and verify against live git. The table is self-consistent as of D-1242 authoring.

**Step 6.** Apply lessons (a)–(z25) + process-gap 1–3 from `cycles/wave-5-e-demo-fidelity/lessons.md`. Lesson z24 (DRIFT-HOLLOW-FEATURE-INTEGRATION-001) and z25 (implementer must NOT commit .factory directly) are critical. Process-gap 1 (CI clippy --all-targets), 2 (demo-evidence test-name sweep), 3 (comprehensive doc-accuracy sweep) are standing disciplines.

---

### PINNED STATE (D-1242 — verified 2026-06-19)

| Artifact | Value | Notes |
|----------|-------|-------|
| develop HEAD | `9114e028` | PIVOT-002 squash @6c367356 + D-1178 CLAUDE.md count bump 76→79; 2026-06-19 (D-1240). UNCHANGED through D-1242 (docs-only). |
| factory-artifacts HEAD | run `git -C .factory log -1 --format='%h %s'` | Do not hard-code; git owns this |
| Open PRs | **NONE** | S-5.03 PR #194 MERGED develop@85ac7b06. PIVOT-002 PR #195 MERGED develop@9114e028. No new PRs. |
| Parked worktrees | S-3.09 FROZEN + W3-FIX-S307-001 BLOCKED | Leave alone. S-5.03 worktree stale (merged); devops-engineer may prune. |
| ci.yml EXPECTED | `79` | After PIVOT-002 merge (+3 types: HttpLookupAuthType, HttpLookupCredentialConfig, HttpLookupConfig). Authority: `ci.yml`. |
| CLAUDE.md non-exhaustive count | `79` | Confirmed on develop@9114e028. |
| scripts/check-non-exhaustive.sh | `EXPECTED=79` | Confirmed. |
| STATE.md | v7.875 | D-1242 burst. |
| BC-INDEX | v6.82 | active 235 / draft 8 / retired 6 / total 256. +6 new BCs D-1241 (BC-2.10.012/013/014 + BC-2.11.016/017/018). 2 amendments (BC-2.10.009 v1.4, BC-2.11.001 v1.10). |
| STORY-INDEX | v2.434 | 204 stories. S-DEMO-PRISMQL-ONBOARDING-001 draft added D-1241. |
| ARCH-INDEX | v2.138 | ADR-041 v1.1 PROPOSED (4-layer LLM onboarding; D-1241). |
| error-taxonomy | v1.91 | E-QUERY-038 column-not-found plan-time gate (D-1241). |
| VP-INDEX | v1.79 | 157 registered. |
| prd | v1.12 | Unchanged. |
| policies | v1.33 | POL-33 route_coverage_table_required_for_stagemask_changes. |
| active_contracts | 235 | Unchanged through D-1242. |
| draft_contracts | 8 | BC-2.06.011, BC-2.21.001, BC-2.10.012, BC-2.10.013, BC-2.10.014, BC-2.11.016, BC-2.11.017, BC-2.11.018. |

---

### WHAT'S DONE THIS SESSION (D-1238..D-1241)

| Burst | What Was Done |
|-------|---------------|
| D-1238 (2026-06-19) | S-5.03 MERGED PR #194 develop@85ac7b06. ci.yml EXPECTED=76; CLAUDE.md=76. POL-14 BC-2.08.005/BC-2.08.006/BC-2.10.008/BC-2.10.009 draft→active. |
| D-1239 (2026-06-19) | error-taxonomy v1.90: E-INFUSE-013 (InvalidFieldSpec) added. BC-2.16.002 v1.83 (http_lookup_enrich_failed field-schema expanded). |
| D-1240 (2026-06-19) | PIVOT-002 MERGED PR #195 develop@9114e028. ci.yml EXPECTED=79; CLAUDE.md=79. POL-14 BC-2.19.001 idempotent (already active). ALL DEMO-BLOCKING LANES DONE. |
| D-1241 (2026-06-19) | PrismQL LLM-onboarding DESIGN PACKAGE: ADR-041 v1.1 (4-layer teaching surface; OPD-1 ADOPTED: normalized_pql field). 6 new BCs (BC-2.10.012/013/014 + BC-2.11.016/017/018 draft). 2 BC amendments (BC-2.10.009 v1.4, BC-2.11.001 v1.10). E-QUERY-038 in error-taxonomy v1.91. S-DEMO-PRISMQL-ONBOARDING-001 draft story (13 pts, 7 BCs, depends S-5.03+S-3.13, wave TBD). BC-INDEX v6.82. STORY-INDEX v2.434. develop UNCHANGED 9114e028. |

---

### WHAT'S NEXT — Demo Roadmap (durable task list)

| Order | Story / Task | Status | Pts | depends_on | Notes |
|-------|-------------|--------|-----|------------|-------|
| **NEXT-A (PICKABLE NOW)** | **S-DEMO-ENRICHMENT-PIVOT-003** | not-started | 8 | PIVOT-002 MERGED (SATISFIED) | Real IOC fields (ioc_value, ioc_type, ioc_severity) in Cyberint/CrowdStrike DTU fixtures + canonical end-to-end pivot proof. Closes BC-2.06.019 v1.7 §Interim State `_ioc_value` violation. Closes TD-PLUGIN-P0-002 P0. story-writer must author if not yet materialized; remove-uncertainty before TDD. |
| **NEXT-B (PICKABLE NOW)** | **S-5.04** | not-started | 5 | S-5.03 MERGED (SATISFIED) | Sensor Health Subsystem. remove-uncertainty before TDD per D-1110. DEMO-CRITICAL-PATH. |
| **NEXT-C (PICKABLE NOW, draft) — DEMO-BLOCKING D-1243** | **S-DEMO-PRISMQL-ONBOARDING-001** | draft (story file authored) | 13 | S-5.03 MERGED + S-3.13 MERGED (SATISFIED) | **DEMO-BLOCKING per user directive 2026-06-19 (D-1243).** 4-layer LLM auto-onboarding teaching surface. T13 capstone requires Claude to author PrismQL against per-client schemas — this teaching mechanism is prerequisite. ADR-041 v1.1 design package ready. remove-uncertainty before TDD per D-1110. |
| **T13 (BLOCKED)** | Multi-client SOC-analyst narrative capstone (not yet named) | not-authored | TBD | **PIVOT-003 (DEMO-BLOCKING) + S-5.04 (DEMO-BLOCKING) + S-DEMO-PRISMQL-ONBOARDING-001 (DEMO-BLOCKING D-1243) — all 3 must MERGE** | PO + story-writer. The demo's capstone deliverable. All 3 DEMO-BLOCKING stories are HARD gates. |
| **T14 (BLOCKED)** | Demo recording | not-started | — | T13 MERGED | demo-recorder. |

**Autonomy D-989+D-1090 active.** Pause only for §7 spec-to-match-code amend / genuine product-business decision / Level-3 escalation / CLAUDE.md edit.

---

### DO-NOT-REFLAG / Ratified Decisions (carry-forward — fresh adversary/session must NOT reopen)

| Item | Ruling | Anchor |
|------|--------|--------|
| ADR-040 v2.0 dual-path infusion | InfusionType::HttpLookup (NVD) + InfusionType::Plugin (ThreatIntel WASM) — LOCKED | D-1226 |
| OPD-1 ADOPTED: normalized_pql field | echo-PQL mechanism adopted in PrismQL LLM-onboarding design | D-1241 |
| All merged-PR closures (PRs #185..#195) | T5/T6/T10/T11/T12/T15a/T15d/S-1.14-REDO/S-5.03/PIVOT-002 MERGED and CLOSED — DO-NOT-REFLAG any | PRs merged 2026-06-13..2026-06-19 |
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 | RESOLVED-MECHANISM (D-1178): count bumps land per-story at merge under orchestrator authority | D-1178 |
| E-QUERY-034 reuse | Reuse ratified — no new error code needed for column-not-found beyond E-QUERY-034+E-QUERY-038 | D-1241 |
| Wave-5 autonomy grant | D-989 + D-1090: full A→merge autonomous; pause only §7/product-business/Level-3/CLAUDE.md | D-989+D-1090 |
| user_directive_persistent | No pragmatic convergence. Fix all issues before build. | STATE.md frontmatter |
| user_directive_remove_uncertainty | Run dclaude:remove-uncertainty on every impl story BOTH immediately after story-writer AND again before TDD delivery | D-1110 extension |
| DRIFT-ORCH-PRLEVEL-PUSH-001 (frozen-HEAD streak rule) | Pushing any commit mid-PR-LEVEL cascade resets streak to 0/3. Re-gate on pushed HEAD. | 2026-06-08 |
| BC-2.08.005 v1.7 two-phase ratification | Spec-only health probe (S-5.03) / null-source; live probe delivered in S-5.04. OBS-3 DEC-004 → product-owner S-5.04 (NOT S-5.03 blocker). | D-1226 |
| PIVOT-002 CRIT/HIGH closures | CRIT-1 (spawn_blocking), CRIT-2a (path traversal), CRIT-2b (loadall disclosure), HIGH-1 (UDF-name), HIGH-2 (config pub), HIGH-3 (SandboxViolation URL), HIGH-4 (PluginId vs InfusionId) — ALL VERIFIED CLOSED through merge. DO-NOT-REFLAG. | PRs #194+#195 |
| factory-artifacts push standing auth | D-1066: state-manager pushes factory-artifacts after every burst (no per-burst re-authorization) | D-1066 |

---

### CONVERGENCE PROTOCOL REMINDERS (standing — fresh session)

- **BC-5.39.001 3-CLEAN protocol:** CLEAN(strict) = ZERO findings of ANY severity. CLEAN(PR-merge) = ZERO CRIT+HIGH+MED (non-blocking). Adversary CLEAN reports MUST specify both.
- **Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001):** Any commit mid-cascade resets streak to 0/3. Re-gate on new HEAD.
- **TD-VSDD-005:** vsdd-factory:adversary tool-binding bug → use `general-purpose` agent as adversary (needs Grep/Bash for SAP-1/SAP-2 probes).
- **Per adversary dispatch:** inject policy rubric from `.factory/policies.yaml`; apply SAP-1 + SAP-2 + production-grade lens. Embed (worktree-abs-path, feature-HEAD-SHA, story-id, canonical-repo-root) tuple per DRIFT-ORCH-ADVERSARY-TUPLE-001.
- **Lesson z25:** Implementer must NOT commit .factory/ artifacts directly. BC/spec commits route via state-manager dispatch.
- **INDEPENDENT multi-pass rule (lesson z24):** Run 3 INDEPENDENT fresh-context streak passes. Sequential single-pass cascades missed F-SV-1 for 8 passes.

---

---

## §ACTIVE OBJECTIVE — Multi-Client SOC-Analyst Live Demo (NORTH STAR)

> **FULL DEMO SCOPE (authoritative — read this to know everything the demo includes, what is built, and what the honest gaps are): `.factory/objectives/DEMO-SCOPE.md`**

> **READ THIS FIRST.** This section persists the current priority goal so fresh sessions never drift onto unrelated pipeline machinery (D-1072, user-directed 2026-06-09). **UNCHANGED by the 2026-06-10 review cycle — the review is an interruption at T5, not a goal change.**

### Goal

Deliver a **MULTI-CLIENT SOC-ANALYST LIVE DEMO**:
- Stand up a collection of DTU clones representing MULTIPLE clients
- Each client has a DIFFERENT sensor combination AND genuinely different per-client data (real data segregation — NOT just client-targeting/federation)
- Each client's scenario **progresses over time** (staged unfolding attack — recon → lateral movement → exfil → containment) and is reproducible (deterministic-over-time; same seed + clock-offset → same timeline)
- Enrichment DTUs (ThreatIntel + NVD) serve scenario-correlated data (IOCs and CVEs introduced by the progression are resolvable)
- Prism federates into each client's DTUs
- Prism MCP wired into Claude (stdio)
- Demonstrate an end-to-end SOC-analyst investigation workflow

### Scope Decisions (user, 2026-06-09)

- **SOC-analyst demo FIRST.** Threat-Detection-Engineer (TDE) workflow — detection rules, write/action-back containment — is **DEFERRED** to a separate later track. Reason: requires building the absent `prism-operations` crate + wiring the dispatch-dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).
- **REAL per-client data segregation required.** Not just client-targeting/federation routing. Each client must serve distinct fixture data.

### Scope Expansion (user, 2026-06-09, D-1077)

**A. Continuous generation = SCENARIO PROGRESSION (unfolding attack).** Not just live-append of random events — each per-client scenario must EVOLVE through stages (e.g. CompromisedEndpoint: recon → lateral movement → exfil → containment), with new telemetry surfacing over time. MUST be deterministic-over-time (same seed + same clock-offset → same timeline) for reproducibility. This is a NEW mechanism not present today (current generators produce one-shot static snapshots; HighChurn models churn as static tombstones, NOT a live feed).

**B. One larger story.** Fold static per-client seeding (BC-2.06.018 baseline) + continuous scenario-progression into a SINGLE larger story (S-DEMO-DTU-DATA-SEEDING-001 expands; will likely be renamed by story-writer to reflect live-scenario scope, e.g. S-DEMO-DTU-LIVE-SCENARIO-001). It anchors MULTIPLE BCs: BC-2.06.018 baseline + a new scenario-progression BC + a new enrichment-correlation BC (pending architect design + PO authorship).

**C. Enrichment DTUs in the live demo.** ThreatIntel + NVD (both static-fixture, NO generator today) must be included in the live demo with SCENARIO-CORRELATED data: the IOCs the progression introduces must resolve in ThreatIntel, and the CVEs on affected devices must resolve in NVD — so the SOC-analyst enrichment workflow is believable. PagerDuty + Jira = response/ticketing DTUs — OUT of current enrichment scope; adjacent to the deferred TDE write-back track per D-1072 (boundary preserved).

**Open obligation (D-1077):** E-DEMO-001 error code must be registered in `.factory/specs/prd-supplements/error-taxonomy.md` (new E-DEMO-NNN namespace; first entry; obligation belongs to error-taxonomy owner; tied to the data-seeding story delivery). Do NOT author the taxonomy entry until the data-seeding story is in scope — record as an open obligation only.

### Capability Audit Verdict (2026-06-09)

`partial-significant-gaps` — but the CORE is genuinely multi-tenant; **build-on, not rebuild**.
- R4 (MCP→Claude): WORKS today
- R1/R2/R3/R5 (multi-client DTU / federation / per-client data / demo tooling): PARTIAL — gap = demo tooling + per-client data seeding
- R6 (TDE write-back): NO → deferred

### Reuse (already merged — DO NOT rebuild)

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (per-org overlays), S-DEMO-001 (per-org adapters / boot step 9A), S-DEMO-002 (single-org E2E + CI harness), S-DEMO-003 (demo scripts + credential CLI + runbook), S-6.20 (multi-clone demo-server), S-6.06–6.10 (4 DTU clones), S-3.1.x / S-3.3.x (org isolation type system + multi-org config boot), S-DEMO-QUERY-PUSHDOWN-001 + per-sensor route-fidelity stories (live-query realism).

### Build Sequence — Complete Story Roadmap

> **Source of truth for the full story list:** `.factory/objectives/multi-client-soc-demo-tasks.md §Complete Story Roadmap`. The table below is the resume-snapshot mirror. Always reconcile against the ledger if detail is needed.

| Order | Story ID | Status | Pts | BCs | depends_on | Notes |
|-------|----------|--------|-----|-----|------------|-------|
| 1 — **DONE** | **S-DEMO-MULTI-TENANT-DTU-001** | **MERGED** (T6 DONE — PR #187 develop@664566e9 2026-06-14; LOCAL 11-pass 3-CLEAN strict + PR-LEVEL 10-pass 3-CLEAN strict; CI 43/43; BC-2.06.017 v1.10 active per POL-14 D-1158) | 8 | BC-2.06.017 (active — v1.10) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | T6 DONE |
| 2 — DONE | **S-DEMO-DTU-LIVE-SCENARIO-001-A** | **MERGED** (T4-A; PR #181 develop@c287b00d 2026-06-10; BC-2.06.018 v1.6 active) | 8 | BC-2.06.018 (active) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | T4-A DONE — Story B unblocked |
| 3 — **MERGED** | **S-DEMO-DTU-LIVE-SCENARIO-001-B** | **MERGED** (T5 DONE — PR #185 squash-merged develop@7fd35b77 2026-06-13; 3/3 strict CONVERGED passes 27/28/29; BC-2.06.019 v1.7 + BC-2.06.020 v1.6 ACTIVE per POL-14 D-1139) | 7 | BC-2.06.019 v1.7 (active) + BC-2.06.020 v1.6 (active) | SATISFIED | DONE |
| 4 — **DONE** | **S-DEMO-004** | **MERGED** (T10 DONE — PR #188 develop@7241f5ef 2026-06-15T05:48:52Z; LOCAL 3/3 strict CONVERGED + PR-LEVEL 3/3 strict CONVERGED passes 5/6/7; CI 43/43; POL-14 all 9 BCs idempotent; D-1176) | 8 | BC-3.2.001, BC-2.06.014, BC-2.11.005, BC-2.01.013, BC-2.06.017, BC-2.06.018, BC-2.10.001, BC-2.22.001, BC-2.09.008 (all active) | S-DEMO-MULTI-TENANT-DTU-001 + data layer (001-A/B) | T10 DONE |
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | **MERGED** (T11 DONE — PR #190 develop@c3ecf6c8 2026-06-16; PR-LEVEL 3/3 strict; POL-14 all 5 BCs idempotent) | 8 | BC-2.06.001, BC-2.06.012, BC-2.06.013, BC-2.06.014, BC-2.06.017 | S-DEMO-003 (SATISFIED) | T11 DONE |
| 6 — capstone | **Multi-client SOC-analyst narrative story** (not yet named or authored) | **not-authored** (no story file, no STORY-INDEX row; owner: product-owner + story-writer; after enrichment chain + capability-discovery + S-1.14-REDO all merged) | TBD | TBD | S-1.14-REDO + PIVOT-003 + S-5.04 all MERGED | T13 → T14 demo recording; the demo's capstone deliverable |
| **D-1107 SCOPE-IN** | **S-5.02** | **MERGED** (PR #191 develop@bec894a2 2026-06-17; PR-LEVEL 3/3 strict; POL-14 BC-2.10.011 status:draft→active) | 5 | BC-2.10.004, BC-2.10.007, BC-2.10.011 (all active) | S-5.01-FOLLOWUP-MCP-BOOT (SATISFIED) | T15a DONE |
| **D-1107 SCOPE-IN** | **S-5.03** | **IN PROGRESS — 3-CLEAN NEEDED** @5a444a5f | 4 | BC-2.08.005, BC-2.08.006 | S-5.02 MERGED + S-3.13 MERGED (SATISFIED) | CLEAN(PR-merge)=yes; fix 3 strict-only OBS (F-OBS-1 userinfo strip + separator cosmetic + stale comment) → freeze HEAD → 3-CLEAN; OBS-3 DEC-004 → product-owner S-5.04/targeted story (out of scope); **DEMO-CRITICAL-PATH** |
| **D-1107 SCOPE-IN** | **S-5.04** | not-started | 5 | -- | S-5.03, S-DEMO-001 | Sensor Health Subsystem — remove-uncertainty before TDD; **DEMO-CRITICAL-PATH** |
| **D-1107 SCOPE-IN** | **S-3.13** | **MERGED** (PR #192 develop@60249ccc 2026-06-16; PR-LEVEL 3/3 strict; POL-14 BC-2.16.007 status:draft→active) | 3 | BC-2.16.007, BC-2.16.001, BC-2.11.001 (all active) | S-3.02, S-1.12 (SATISFIED) | T15d DONE |
| **D-1164 REQUIRED** | **S-DEMO-ENRICHMENT-PIVOT-001** | **MERGED** (T12 DONE — PR #189 develop@1b2e9a31 2026-06-16; PR-LEVEL 3/3 strict; BC-2.19.001+BC-2.19.003 active) | 5 | BC-2.19.001 (active), BC-2.19.003 (active) | S-1.14 (via S-1.14-REDO) | T12 DONE |
| **D-1205 DEMO-BLOCKING** | **S-DEMO-ENRICHMENT-PIVOT-002** | **IN PROGRESS — BLOCKED on rebase** @0f958261 | 8 | BC-2.19.001 | PIVOT-001 MERGED; CRIT-1/2a/2b/HIGH-1+all HIGH/MED closed; BLOCKED on rebase onto S-1.14-REDO (F-SV-1 dep; ThreatIntel prod enrichment requires infusion_load_step→PluginRuntime wiring) | MERGES AFTER S-1.14-REDO; rebase → verify prod enrichment → 3-CLEAN → merge |
| **D-1205 DEMO-BLOCKING** | **S-1.14-REDO** | **IN PROGRESS — 3-CLEAN NEEDED** @2020dbf0 (CLEAN; D-1227) | 8 | -- | S-WAVE5-PREP-01+S-3.02-FOLLOWUP-RUNTIME (SATISFIED) | **MERGES FIRST (D-1221)** — all fixes committed (just check 4499/4499 green; non-exhaustive gate 67); dispatch 3 INDEPENDENT fresh-context adversary passes on frozen 2020dbf0 → LOCAL 3-CLEAN → PR-LEVEL 3-CLEAN → merge; EXPECTED 66→67 |
| **D-1205 DEMO-BLOCKING** | **S-DEMO-ENRICHMENT-PIVOT-003** | not-started | 8 | -- | PIVOT-002 MERGED | Real IOC fields in DTU fixtures + canonical pivot proof; closes BC-2.06.019 §Interim State _ioc_value violation; closes TD-PLUGIN-P0-002 P0 |

**NEXT CONCRETE ACTION (D-1221..D-1227): 3 lanes frozen — CASCADE ROUND-2 complete + D-1227 HEAD correction. (a) S-1.14-REDO @2020dbf0: worktree CLEAN; dispatch 3 INDEPENDENT fresh-context adversary passes on frozen HEAD 2020dbf0 → LOCAL 3-CLEAN → PR-LEVEL 3-CLEAN → MERGE FIRST; EXPECTED 66→67. (b) S-5.03 @5a444a5f: fix 3 strict-only OBS (F-OBS-1 userinfo strip + separator cosmetic + stale comment) → freeze HEAD → LOCAL 3-CLEAN → PR-LEVEL 3-CLEAN → merge (INDEPENDENT; no sequencing dep); EXPECTED →72. (c) PIVOT-002 @0f958261: AWAIT S-1.14-REDO merge → orchestrator dispatches devops-engineer to rebase → verify production ThreatIntel enrichment → 3-CLEAN → merge; EXPECTED →76. OBS-3 DEC-004 → product-owner routes to S-5.04/targeted story (DO-NOT-REFLAG as S-5.03 blocker). T13 capstone (not-authored; PO+story-writer) after all three merged. D-989+D-1090 autonomy grant remains active.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: L-POST (D-1205 SCOPING RESOLVED — all prior lanes CLOSED; develop@60249ccc). T1+T2+T3+T4+T4-A+T5+T6+T8+T9+T10+T11+T12 DONE. NO OPEN PRs. develop@60249ccc (D-1204 S-3.13 merge). BC-INDEX v6.69. ARCH-INDEX v2.135. STORY-INDEX v2.420. error-taxonomy v1.86. VP-INDEX v1.79 (157). policies v1.33. prd v1.12. BC-2.19.001 v1.7 ACTIVE. BC-2.06.017 v1.10 ACTIVE. BC-2.06.018 v1.6 ACTIVE. BC-2.06.019 v1.7 ACTIVE. BC-2.06.020 v1.6 ACTIVE. STATE v7.848 (D-1205 scope-decision burst).**

---

## §RESUME SNAPSHOT — D-1241 (2026-06-19 — PrismQL LLM-onboarding DESIGN PACKAGE; develop_head 9114e028 UNCHANGED; STATE v7.874)

> **D-1241 burst (2026-06-19).** SPEC-ONLY burst — develop_head UNCHANGED at 9114e028. ADR-041 v1.1 authored (4-layer LLM auto-onboarding teaching surface; OPD-1 RESOLVED: normalized_pql field ADOPTED). 6 new BCs (BC-2.10.012/013/014 draft + BC-2.11.016/017/018 draft) + 2 BC amendments (BC-2.10.009 v1.4, BC-2.11.001 v1.10). E-QUERY-038 column-not-found plan-time gate registered in error-taxonomy v1.91. S-DEMO-PRISMQL-ONBOARDING-001 draft story authored (13 pts, 7 BCs, depends S-5.03+S-3.13, wave TBD). BC-INDEX v6.82 (250→256 total; 2→8 draft). STORY-INDEX v2.434 (203→204). ARCH-INDEX v2.138 (ADR-041 v1.0→v1.1 PROPOSED). D-1240 SUPERSEDED by D-1241.

---

### ZERO-CONTEXT RESTART PROTOCOL D-1241 (run in this order)

**Step 1.** `vsdd-factory:factory-worktree-health` — confirm .factory/ worktree on factory-artifacts branch.

**Step 2.** `git log --oneline -1 origin/develop` → expect `9114e028`.

**Step 3.** `gh pr list --state open --base develop` → expect NO open demo-blocking PRs (all prior lanes merged).

**Step 4.** Apply lessons (a)–(z25) + process-gap 1–3 from `cycles/wave-5-e-demo-fidelity/lessons.md`.

**Step 5.** Execute PIVOT-003 lane (NEXT DEMO-BLOCKING impl story):
- PIVOT-003 adds real IOC fields (ioc_value, ioc_type, ioc_severity) to Cyberint/CrowdStrike DTU fixtures + canonical end-to-end pivot proof.
- Closes TD-PLUGIN-P0-002 P0. Closes BC-2.06.019 v1.7 §Interim State `_ioc_value` violation.
- story-writer must author PIVOT-003 spec if not yet materialized; remove-uncertainty before TDD.

**Step 6.** S-DEMO-PRISMQL-ONBOARDING-001 is now **DEMO-BLOCKING** (D-1243 user directive 2026-06-19). It gates T13 alongside PIVOT-003 and S-5.04. All three are PICKABLE NOW. Not a sequencing blocker for PIVOT-003 or S-5.04 (all three are independent/parallel), but all three must merge before T13 can begin.

**After PIVOT-003 MERGED + S-5.04 MERGED + S-DEMO-PRISMQL-ONBOARDING-001 MERGED (all 3 DEMO-BLOCKING per D-1243):** T13 capstone narrative (PO+story-writer; not-authored) → T14 demo recording.

**DO-NOT-REFLAG (PIVOT-002 closures; all verified pre-rebase and through merge):** CRIT-1 (sync-WASM SEC-001 spawn_blocking), CRIT-2a (path traversal DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001), CRIT-2b (load disclosure DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001), HIGH-1 (UDF-name validation DRIFT-PIVOT-UDFNAME-VALIDATION-001), HIGH-2 (config pub field DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001), HIGH-3 (SandboxViolation URL log DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001), HIGH-4 (PluginId vs InfusionId DRIFT-PIVOT-PLUGINID-INFUSIONID-001).

**Autonomy D-989+D-1090 active.** Pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit.

---

## §RESUME SNAPSHOT — D-1240 (2026-06-19 — PIVOT-002 MERGED; develop_head 9114e028; ALL DEMO-BLOCKING LANES DONE; STATE v7.873)

> **D-1240 burst (2026-06-19).** PIVOT-002 MERGED PR #195 develop@9114e028. ci.yml EXPECTED=79 on develop@9114e028. CLAUDE.md non-exhaustive count=79. POL-14 BC-2.19.001 already active (lifecycle_status:active; idempotent). BC-INDEX v6.80 UNCHANGED. error-taxonomy v1.90 UNCHANGED. **ALL DEMO-BLOCKING LANES DONE.** D-1236/D-1237/D-1238 SUPERSEDED by D-1240.

---

### ZERO-CONTEXT RESTART PROTOCOL D-1240 (run in this order)

**Step 1.** `vsdd-factory:factory-worktree-health` — confirm .factory/ worktree on factory-artifacts branch.

**Step 2.** `git log --oneline -1 origin/develop` → expect `9114e028`.

**Step 3.** `gh pr list --state open --base develop` → expect NO open PIVOT-002 PR (merged).

**Step 4.** Apply lessons (a)–(z25) + process-gap 1–3 from `cycles/wave-5-e-demo-fidelity/lessons.md`.

**Step 5.** Confirm PIVOT-002 worktree state:
```
git -C .worktrees/S-DEMO-ENRICHMENT-PIVOT-002 log --oneline -1  # should reference merged commit
```
Worktree may be pruned by devops-engineer.

**Step 6.** Execute PIVOT-003 lane (NEXT DEMO-BLOCKING):
- PIVOT-003 adds real IOC fields (ioc_value, ioc_type, ioc_severity) to Cyberint/CrowdStrike DTU fixtures + canonical end-to-end pivot proof.
- Closes TD-PLUGIN-P0-002 P0. Closes BC-2.06.019 v1.7 §Interim State `_ioc_value` violation.
- story-writer must author PIVOT-003 spec if not yet materialized; remove-uncertainty before TDD.

**After PIVOT-003:** S-5.04 (Sensor Health Subsystem) → T13 capstone narrative → T14 demo recording.

**DO-NOT-REFLAG (PIVOT-002 closures; all verified pre-rebase and through merge):** CRIT-1 (sync-WASM SEC-001 spawn_blocking), CRIT-2a (path traversal DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001), CRIT-2b (load disclosure DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001), HIGH-1 (UDF-name validation DRIFT-PIVOT-UDFNAME-VALIDATION-001), HIGH-2 (config pub field DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001), HIGH-3 (SandboxViolation URL log DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001), HIGH-4 (PluginId vs InfusionId DRIFT-PIVOT-PLUGINID-INFUSIONID-001).

**DESIGN TRACK (separate burst — do NOT include in next PIVOT-003 state burst):** ADR-041 PrismQL-LLM-onboarding + `.factory/research/llm-query-dsl-onboarding-2026-06-19.md` + ARCH-INDEX update are pending a dedicated design burst. These were present in the working tree but explicitly excluded from D-1240 per scoping instruction.

**Autonomy D-989+D-1090 active.** Pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit.

---

## §RESUME SNAPSHOT — D-1238 (2026-06-19 — S-5.03 MERGED + PIVOT-002 SPEC-FIX; develop_head 85ac7b06; 1 DEMO-BLOCKING LANE; STATE v7.872)

> **D-1238/D-1239 burst (2026-06-19).** S-5.03 MERGED PR #194 develop@85ac7b06. ci.yml EXPECTED=76 on develop@85ac7b06. CLAUDE.md non-exhaustive count=76. POL-14 BC promotions: BC-2.08.005/BC-2.08.006/BC-2.10.008/BC-2.10.009 status draft→active (lifecycle_status was already active; no count change). BC-2.16.007 idempotent. error-taxonomy v1.90 (E-INFUSE-013 added). BC-2.16.002 v1.83. **1 DEMO-BLOCKING LANE:** PIVOT-002 @0f958261 branch feature/S-DEMO-ENRICHMENT-PIVOT-002 — clean (pre-rebase); story v1.4; UNBLOCKED (F-SV-1 dep satisfied by S-1.14-REDO merge; spec-fix committed in this burst); devops-engineer rebase onto develop@85ac7b06 NEXT; verify EXPECTED count post-rebase (76 + PIVOT-002 types, target ~79); LOCAL 3-CLEAN(strict) on rebased frozen HEAD → push → PR-LEVEL 3-CLEAN → merge. D-1236/D-1237 SUPERSEDED by D-1238.

---

### ZERO-CONTEXT RESTART PROTOCOL D-1238 (run in this order)

**Step 1.** `vsdd-factory:factory-worktree-health` — confirm .factory/ worktree on factory-artifacts branch.

**Step 2.** `git log --oneline -1 origin/develop` → expect `85ac7b06` (D-1178 CLAUDE.md count bump 70→76; S-5.03 merge @7fc1afef).

**Step 3.** `gh pr list --state open --base develop` → expect NO open S-5.03 PR. PIVOT-002 PR not yet open (pre-rebase).

**Step 4.** Apply lessons (a)–(z25) + process-gap 1–3 from `cycles/wave-5-e-demo-fidelity/lessons.md`.

**Step 5.** Confirm PIVOT-002 worktree state:
```
git -C .worktrees/S-DEMO-ENRICHMENT-PIVOT-002 log --oneline -1  # expect 0f958261
git -C .worktrees/S-DEMO-ENRICHMENT-PIVOT-002 status            # expect clean
```

**Step 6.** Execute PIVOT-002 lane:
- **STEP C1:** devops-engineer rebases `feature/S-DEMO-ENRICHMENT-PIVOT-002` onto develop@`85ac7b06`.
- **STEP C2:** Verify production ThreatIntel enrichment uses real InfusionSource (not NullSource) after rebase.
- **STEP C3:** Reconcile non-exhaustive EXPECTED: develop=76; determine PIVOT-002 additions; update ci.yml + scripts/check-non-exhaustive.sh + CLAUDE.md.
- **STEP C4:** LOCAL 3-CLEAN(strict) adversary passes on rebased frozen HEAD. DO-NOT-REFLAG CRIT-1/2a/2b/HIGH-1/2/3/4 — ALL CLOSED pre-rebase.
- **STEP C5:** Push → PR-LEVEL 3-CLEAN(strict) → pr-manager squash-merge.
- **STEP C6 (POST-MERGE BURST):** state-manager post-merge: develop_head update; PIVOT-002 POL-14 BC promotions; STATE advance.

**After PIVOT-002:** PIVOT-003 → S-5.04 → T13 capstone → T14 recording.

**DO-NOT-REFLAG (PIVOT-002 pre-rebase closures; all verified):** CRIT-1 (sync-WASM SEC-001 spawn_blocking), CRIT-2a (path traversal DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001), CRIT-2b (load disclosure DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001), HIGH-1 (UDF-name validation DRIFT-PIVOT-UDFNAME-VALIDATION-001), HIGH-2 (config pub field DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001), HIGH-3 (SandboxViolation URL log DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001), HIGH-4 (PluginId vs InfusionId DRIFT-PIVOT-PLUGINID-INFUSIONID-001).

**Autonomy D-989+D-1090 active.** Pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit.

---

## §RESUME SNAPSHOT — D-1236 (2026-06-19 — COMPREHENSIVE ZERO-CONTEXT RESTART SNAPSHOT; develop_head 5c747549; PR #194 OPEN; 2 DEMO-BLOCKING LANES; STATE v7.870)

> **D-1237 SHA CORRECTION applied: feature/S-5.03 @ 14189f22 PUSHED; PR #194 remote==local==14189f22; CI running on 14189f22 (stale-base, EXPECTED=72 self-consistent) — MOOT pending develop rebase. D-1236: COMPREHENSIVE ZERO-CONTEXT DURABILITY SNAPSHOT. develop_head 5c747549 (feat(S-1.14-REDO): Full Infusion Engine — Loader, Registry, Runtime, Cache Integration; PR #193 squash-merged 2026-06-19). ci.yml EXPECTED=70 on develop@5c747549 (CONFIRMED; S-1.14-REDO added 4 types). CLAUDE.md non-exhaustive count=70 (CONFIRMED). 2 DEMO-BLOCKING LANES: (1) S-5.03 @14189f22 (worktree) / PR #194 OPEN HEAD 14189f22 PUSHED — branch feature/S-5.03; story v1.22; needs rebase onto develop@5c747549 (EXPECTED 72→76 reconciliation); r5 PR-LEVEL 3-CLEAN(strict) on rebased+pushed frozen HEAD NEXT; INDEPENDENT (no sequencing dep on PIVOT-002). (2) PIVOT-002 @0f958261 branch feature/S-DEMO-ENRICHMENT-PIVOT-002 — clean (pre-rebase); story v1.4; UNBLOCKED (F-SV-1 dep satisfied by S-1.14-REDO merge); devops-engineer rebase onto develop@5c747549 NEXT; verify production ThreatIntel uses real InfusionSource post-rebase; EXPECTED →82. Merge B/C independent. D-1234+D-1235 SUPERSEDED by D-1236. D-1236 snapshot corrected by D-1237 (SHA update only).**

---

### ZERO-CONTEXT RESTART PROTOCOL (run in this order; no prior context needed)

A fresh session with NO prior context runs these steps in order before taking any action.

**Step 0.** Read this D-1236 snapshot first. It is authoritative. Do NOT act on any other prior context.

**Step 1.** Run `vsdd-factory:factory-worktree-health` (devops-engineer). **BLOCKING** — do not proceed until it passes.

**Step 2.** Verify develop HEAD:
```bash
git log --oneline -1 origin/develop
```
Expected: `5c747549` (feat(S-1.14-REDO): Full Infusion Engine; PR #193 squash-merged 2026-06-19). If newer, use live HEAD as authoritative.

**Step 3.** Confirm open PRs:
```bash
gh pr list --state open --base develop
```
Expected: PR #194 OPEN (feature/S-5.03; HEAD 14189f22 or newer if additional commits pushed).

**Step 4.** Check worktree state:
```bash
git worktree list
```
Expected mounted worktrees: main repo + `.factory` (factory-artifacts) + `.worktrees/S-5.03` + `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002` + `.worktrees/S-3.09` (FROZEN) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded). `.worktrees/S-1.14-REDO` is STALE (merged) — safe for devops-engineer to prune. Leave S-3.09 and W3-FIX-S307-001 alone.

**Step 5.** Verify active worktree states:
```bash
git -C .worktrees/S-5.03 log --oneline -1 && git -C .worktrees/S-5.03 status
git -C .worktrees/S-DEMO-ENRICHMENT-PIVOT-002 log --oneline -1 && git -C .worktrees/S-DEMO-ENRICHMENT-PIVOT-002 status
```
Expected: S-5.03 HEAD `14189f22` (docs: comprehensive PR-diff doc-accuracy sweep), CLEAN. PIVOT-002 HEAD `0f958261` (pre-rebase), CLEAN.

**Step 6.** Apply lessons (a)–(z25) + process-gap 1–3 from `cycles/wave-5-e-demo-fidelity/lessons.md`. Lesson z24 (DRIFT-HOLLOW-FEATURE-INTEGRATION-001) and lesson z25 (implementer must NOT commit .factory directly) are both critical. Process-gap 1 (CI clippy --all-targets), 2 (demo-evidence test-name sweep), 3 (comprehensive doc-accuracy sweep) are new D-1236 additions.

**Step 7.** Execute EXACT NEXT ACTIONS per ACTIVE LANES table below. S-5.03 and PIVOT-002 are independent (no merge sequencing constraint between them).

---

### PINNED STATE (concrete; D-1236 authoritative)

| Artifact | Value | Notes |
|----------|-------|-------|
| develop HEAD | `5c747549` | feat(S-1.14-REDO): Full Infusion Engine; PR #193 squash-merged 2026-06-19 (D-1235). |
| factory-artifacts HEAD | run `git -C .factory log -1 --format='%h %s'` | Do not hard-code; git owns this |
| Open PRs | **PR #194 OPEN** | feature/S-5.03; HEAD 14189f22 PUSHED (remote==local==14189f22; CI running on 14189f22 stale-base MOOT pending rebase); awaiting rebase onto develop@5c747549 + EXPECTED 72→76 + r5 PR-LEVEL 3-CLEAN(strict). |
| S-1.14-REDO worktree | STALE (MERGED) | branch feature/S-1.14-REDO MERGED; worktree `.worktrees/S-1.14-REDO` STALE — devops-engineer may prune |
| S-5.03 worktree | ACTIVE | `.worktrees/S-5.03` — branch feature/S-5.03 — HEAD 14189f22 (comprehensive doc-accuracy sweep) — clean; PR #194 OPEN HEAD 14189f22 PUSHED (remote==local==14189f22; D-1237 corrected from 3c4252d5) |
| PIVOT-002 worktree | ACTIVE (UNBLOCKED) | `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002` — branch feature/S-DEMO-ENRICHMENT-PIVOT-002 — HEAD 0f958261 (pre-rebase); devops-engineer rebase onto develop@5c747549 NEXT |
| S-3.09 worktree | FROZEN | `.worktrees/S-3.09` — leave alone |
| W3-FIX-S307-001 worktree | BLOCKED/superseded | `.worktrees/W3-FIX-S307-001` — leave alone |
| ci.yml EXPECTED (develop) | `70` | S-1.14-REDO merged: 66 + 4 new non-exhaustive types = 70. Updated from 66 per D-1235. |
| CLAUDE.md non-exhaustive count | `70` | Arrives at 70 via S-1.14-REDO merge diff. Updated from 66. |
| S-1.14-REDO story | v1.6 merged | PR #193 squash-merged develop@5c747549 2026-06-19 (D-1235). |
| BC-2.19.001 | v2.1 active | E-INFUSE-012 MAX_SOURCE_FILE_BYTES=100 MiB; InfusionError::SourceFileTooLarge{path,size,limit}. Status: active. |
| BC-2.19.002 | v1.3 active | POL-14 status: draft→active D-1235 (legacy-field sync; lifecycle_status was already active). |
| BC-2.19.003 | v1.3 active | Already active per D-1192; idempotent. |
| BC-2.19.004 | v1.4 active | POL-14 status: draft→active D-1235 (legacy-field sync; lifecycle_status was already active). |
| BC-2.19.005 | v1.3 active | POL-14 status: draft→active D-1235 (legacy-field sync; lifecycle_status was already active). |
| error-taxonomy | v1.90 | E-INFUSE-013 (InvalidFieldSpec; D-1239); prior v1.89 = E-INFUSE-012 (SourceFileTooLarge). |
| BC-INDEX | v6.80 | active 235 / draft 2 / retired 6 / total 250. Updated from v6.79 per D-1238/D-1239. |
| STORY-INDEX | v2.432 | total_stories 203. Updated from v2.431 per D-1238. |
| ARCH-INDEX | v2.137 | ADR-040 v2.0 (dual-path infusion; D-1226) |
| VP-INDEX | v1.79 | 157 registered |
| prd | v1.12 | — |
| policies | v1.33 | — |

---

### ACTIVE LANES — Current SHAs + Phase + EXACT NEXT ACTION (D-1236)

> **SEQUENCING (D-1236):** S-1.14-REDO MERGED develop@5c747549 (DONE). S-5.03 (PR #194 OPEN) and PIVOT-002 are INDEPENDENT (no merge sequencing constraint). Each merge: orchestrator bumps CLAUDE.md count + ci.yml EXPECTED.

| Lane | Story | Worktree | Branch | HEAD | Phase / Streak | EXACT NEXT ACTION |
|------|-------|----------|--------|------|----------------|-------------------|
| **L-POST-A — DONE** | S-1.14-REDO (Full Infusion Engine) | `.worktrees/S-1.14-REDO` (STALE) | `feature/S-1.14-REDO` (MERGED) | **5c747549** (squash commit on develop) | **MERGED PR #193** — POL-14 complete; S-1.14 graduated ADR-020 | **DONE.** devops-engineer may prune `.worktrees/S-1.14-REDO`. |
| **L-POST-B — DONE** | S-5.03 (MCP Resources & Prompts) | `.worktrees/S-5.03` (STALE) | `feature/S-5.03` (MERGED) | **85ac7b06** (squash commit on develop @7fc1afef; CLAUDE.md count bump @85ac7b06) | **MERGED PR #194** — POL-14 BC-2.08.005/BC-2.08.006/BC-2.10.008/BC-2.10.009 draft→active; EXPECTED=76. | **DONE.** devops-engineer may prune `.worktrees/S-5.03`. |
| **L-POST-C — DONE** | PIVOT-002 (ThreatIntel/NVD dual-path) | `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002` (STALE) | `feature/S-DEMO-ENRICHMENT-PIVOT-002` (MERGED) | **9114e028** (squash @6c367356 + CLAUDE.md count bump @9114e028) | **MERGED PR #195** — POL-14 BC-2.19.001 idempotent (already active); EXPECTED=79. D-1240. | **DONE.** devops-engineer may prune `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002`. NEXT: PIVOT-003. |

---

### ARCHITECTURE DECISIONS RATIFIED (D-1226 — locked; fresh adversary MUST NOT re-litigate)

| Decision | Ruling | Anchor |
|----------|--------|--------|
| DUAL-PATH INFUSION (ADR-040 v2.0) | InfusionType::HttpLookup (declarative TOML; reuses pipeline.rs executor) for NVD. InfusionType::Plugin (WASM via wasmtime) for ThreatIntel polymorphic IOC. prism-nvd-infusion crate REMOVED — NVD uses HttpLookup path. | ADR-040 v2.0 |
| BC-2.10.008 v1.12 | org-no-overlay returns 0 sensors + EC-10-017. Option B. | D-1226 |
| BC-2.08.005 v1.7 | Two-phase health probe: S-5.03 spec-only/null-source, S-5.04 delivers live probe. | D-1226 |
| BC-2.16.002 v1.83 | http_lookup_enrich_failed catalog field-schema expanded (D-1239); prior v1.82 = HttpLookup event names per ADR-040 §D8.7. | D-1239 |
| error-taxonomy v1.90 | E-INFUSE-013 (InvalidFieldSpec; D-1239); prior v1.89 = E-INFUSE-009/010/011 (HttpLookup lifecycle events) + E-INFUSE-012 (SourceFileTooLarge, CWE-400 guard). | D-1239 |
| AC-10 scope | Relocated S-5.03 → S-5.08 (BC-2.08.009 v1.4). | D-1226 |
| OBS-3 BC-2.08.005 DEC-004 zero-sensor message | Routed to product-owner for S-5.04 or targeted story. NOT a S-5.03 blocker. | D-1223 |
| pipe-mode | Deferred to S-3.01 scope. NOT a S-1.14-REDO finding. | Ratified scope |
| E-INFUSE-004 http_lookup valid-types message sync | Obligation recorded in PIVOT-002 story v1.4. NOT a S-1.14-REDO blocker. | E-INFUSE-004 |
| output_columns adds_columns subset | output_columns must be a subset of adds_columns — ratified scope. | Ratified scope |
| infusion_name == infusion_id | Invariant: infusion_name is always equal to infusion_id. | Ratified scope |

---

### DO-NOT-REFLAG — Closures Verified (D-1236 — fresh adversary MUST NOT reopen)

**S-1.14-REDO closures (all verified via production-path tracing; all CLOSED as of 4133d186):**

| Finding | Status | Closure Note |
|---------|--------|-------------|
| CRIT-1 InfusionSource types non-functional | CLOSED | Real MMDB/CSV/JSON/HttpLookup source implementations |
| CRIT-2 Tier-3 cache (real RocksDB boot) | CLOSED | Production boot wires RocksDB-backed tier-3 |
| HIGH-A source_column in TOML not propagated | CLOSED | Propagated through InfusionLoader to descriptor |
| E-INFUSE-008 redaction gap | CLOSED | Redacted per error-taxonomy v1.88+ |
| OBS-1 (original) load_spec real source not null | CLOSED | load_spec_with_runtime uses real InfusionSource |
| F-LOCAL-1/2/3 (cache eviction, TTL, registry) | CLOSED | Three-tier cache fully wired |
| F-SV-1 plugin boot-wiring (hollow-feature) | CLOSED | infusion_load_step → PluginRuntime wired in boot.rs; CONFIRMED by independent multi-pass |
| F-SV-2 InfusionRegistry not registered in engine | CLOSED | Registered at engine init |
| F-TTL-1 per-descriptor TTL non-default | CLOSED | Per-descriptor TTL wired; non-default-TTL test load-bearing |
| F-2REV-LOW-1 refresh_interval_secs doc comment | CLOSED | doc→inert/reserved comment; committed 2020dbf0 |
| OBS-1 InfusionLruCache::new(NonZeroUsize) via const | CLOSED | Uses const; committed 2020dbf0 |
| OBS-2 AC-7 integration test sentinel-source | CLOSED | Test strengthened; committed 2020dbf0 |
| NonZeroUsize cache ctor | CLOSED | Uses const NonZeroUsize; committed 2020dbf0 |
| cache-key ':' guard | CLOSED | Delimiter guarded |
| descriptor #[non_exhaustive] | CLOSED | Applied |
| AC-7 integration test | CLOSED | Strengthened with sentinel-source |
| SEC-001/E-INFUSE-012 source-file 100 MiB guard | CLOSED | fs::metadata size guard in CsvSource/JsonLookupSource/MmdbSource + 4 load-bearing tests; BC-2.19.001 v2.1; error-taxonomy v1.89 (E-INFUSE-012 InfusionError::SourceFileTooLarge) |
| F-HOTRELOAD-OVERSIZE-1 load_spec/hot_reload propagate SourceFileTooLarge as Err | CLOSED | load_spec/load_spec_with_runtime/hot_reload return Err(SourceFileTooLarge) — reject+preserve pattern; EC-19-004 missing-file degrade preserved; committed 41a4d3bd |
| OBS-1 InfusionError #[non_exhaustive] (gate 69→70, v70 enum_violation) | CLOSED | InfusionError #[non_exhaustive] added; CI gate bumped 69→70; committed f819a70d |
| Windows spec-discovery (test-fixture TOML backslash-path) | CLOSED | Test-fixture TOML paths use '/' (forward slash) only; production loader.rs unchanged; 2 regression tests added; committed 4133d186 |

**Ratified scope (NOT findings — fresh adversary must NOT raise as blockers):**
- pipe-mode → S-3.01 scope (not S-1.14-REDO)
- E-INFUSE-004 http_lookup valid-types message → PIVOT-002 (obligation recorded in story v1.4)
- output_columns must be a subset of adds_columns — ratified scope boundary
- infusion_name == infusion_id — ratified invariant

**S-5.03 closures (LOCAL + PR-LEVEL r1–r4; all verified at 14189f22 / PR #194 HEAD 14189f22 PUSHED; D-1237 corrected from 3c4252d5):**

| Finding | Status | Closure Note |
|---------|--------|-------------|
| per-org DI-008 resources + check_sensor_health | CLOSED | Per-org DI wired |
| Spec-only health probe / null-source | CLOSED | BC-2.08.005 v1.7 two-phase ratified |
| reload notification wiring | CLOSED | Wired |
| Schema DI-006 | CLOSED | Fixed |
| Prompt-arg sanitization OBS-1 | CLOSED | DI-006 parity applied; v1.17 label sweep |
| Mutex poison-tolerance | CLOSED | Handled |
| Non-exhaustive types | CLOSED | +6 types registered |
| display_name field | CLOSED | Human-approved addition; story v1.22 |
| reload-wipe HIGH | CLOSED | Fixed |
| keyed-object health | CLOSED | Fixed |
| resource_pressure null | CLOSED | Fixed |
| cross-client isolation | CLOSED | Removed cross-client sharing |
| EC-004/005 retired-status | CLOSED | v1.22 correction |
| ci.yml tally reconciled | CLOSED | v1.22 |
| r1 SEC-001 (MED DI-006 URI echo) | CLOSED | Generic "Unknown or unsupported resource URI" — no echo |
| r1 SEC-002 (LOW) | CLOSED | validate_hostname allowlist [a-zA-Z0-9._:-] |
| r1 SEC-003 (LOW) | CLOSED | sanitize_display_name (128-cap + ctrl-strip) |
| r1 OBS-A (LOW) | CLOSED | zero-clients test sentinel-only + EC-08-013 citation removed |
| r2 dead org_registry test lines | CLOSED | Removed |
| r2 validate_time_range doc | CLOSED | None-contract removed; accurate printable-ASCII; decoupled from validate_hostname |
| r2 uri-echo scrub on config-mgr-not-wired branches | CLOSED | Scrubbed |
| r2 evidence stale HEAD/count/renamed-test | CLOSED | Updated |
| r3 validate_time_range None-contract-doc | CLOSED | Removed |
| r3 evidence phantom-test-citation (IMP-8) | CLOSED | Fixed; 33/33 tests confirmed present |
| r4 ConfigSnapshot non-exhaustive rationale | CLOSED | Corrected — struct-literal not "MCP wire type" |
| r4 validate_snapshot doc | CLOSED | Updated |
| r4 render_sensors_health stale-flag doc | CLOSED | Updated response-root stale-flag doc |
| r4 enabled_sensors emits sensor-IDs | CLOSED | BC-2.10.008 v1.12 postcondition 1 satisfied |
| unused_mut in test_SEC_003 (test-code lint) | CLOSED | Fixed in comprehensive sweep at 14189f22 |

**S-5.03 named deferrals (Canonical Principle Rule 3 — NOT blockers):**
- OBS-S503-1: reload_config.rs diff_snapshots DOT-vs-underscore → taxonomy-sweep anchor → deferred to S-5.11 (BC-2.16.007 v1.5 changelog; maintenance story)
- OBS-3 DEC-004 BC-2.08.005 zero-sensor message → product-owner for S-5.04 or targeted story
- OBS-B check_sensor_health content-text JSON-stringification = CONFORMANT (BC met; not a finding)
- validate_time_range logic-tightening — DEFERRED (not an active hole; printable-ASCII already rejects newlines/control)

**PIVOT-002 closures (CRIT/HIGH/MED — prior to blocked-on-rebase state):**

| Finding | Status | Closure Note |
|---------|--------|-------------|
| CRIT-1 WASM guest integration | CLOSED | Real wit_bindgen WASM guest |
| CRIT-2a validate_plugin_path | CLOSED | Path validation with canonicalize + allowlist |
| CRIT-2b real .prx end-to-end fixture | CLOSED | End-to-end fixture test |
| HIGH-1 identity (infusion_id == plugin_id invariant) | CLOSED | Invariant enforced |
| HIGH-2/3 event catalog + SSRF | CLOSED | Events registered; SSRF mitigated |
| HIGH-4 | CLOSED | Fixed |

**Named deferrals (Canonical Principle Rule 3 — specific future-story anchors):**
- S-5.11: column-delta MCP notifications (draft; OBS-S503-1 DOT-vs-underscore taxonomy-sweep anchor)
- S-5.12: add_sensor_spec MCP notifications (draft)
- E-INFUSE-004 http_lookup valid-types message → PIVOT-002 (recorded in story v1.4)

---

### DO-NOT-REFLAG / Adjudications Carried Forward

These items are CLOSED or DEFERRED-BY-HUMAN. A fresh session must NOT reopen them.

| Item | Ruling | Where Anchored |
|------|--------|---------------|
| S-3.13 AC-7 + AC-4-notification + Task6/7 | DEFERRED to S-5.03 (DRIFT-S313-S503-RESCOPING-001 EXECUTED D-1173). Do not raise as S-3.13 findings. | S-3.13 v1.13; S-5.03 v1.13 |
| PIVOT-001 boot.rs production wiring + hot_reload NullSource | PIVOT-002/003 + S-1.14-REDO scope. Do not raise as PIVOT-001 findings. | OBS-1 D-1173 adjudication |
| PIVOT-001 OBS-1 E-INFUSE-007 missing from error-taxonomy | DISMISSED — E-INFUSE-007 IS registered at error-taxonomy.md line 438 v1.81. DO-NOT-REFLAG. | D-1175 dismissal |
| DRIFT-S313-DUPTEST-001 | CLOSED (D-1177). | D-1177 |
| DRIFT-MCP-INTERNAL-CODE-GRANULARITY-001 | ADJUDICATED SPEC-ACCEPTED (D-1196). | D-1196 |
| All T5/T6/T10/T11/T12 PR closures | MERGED and CLOSED. DO-NOT-REFLAG any. | PRs #185/187/188/189/190 |
| S-5.02 PR #191 closures | MERGED develop@bec894a2. DO-NOT-REFLAG. | PR #191 |
| S-3.13 PR #192 closures | MERGED develop@60249ccc. DO-NOT-REFLAG. | PR #192 |
| DRIFT-PIVOT-PLUGINPATH-TRAVERSAL-001 (PIVOT-002 latent) | NOT a PIVOT-001 PR blocker. Deferred to PIVOT-002. | D-1179 |
| DRIFT-PIVOT-LOADALL-PATH-DISCLOSURE-001 (PIVOT-002 latent) | NOT a PIVOT-001 PR blocker. Deferred to PIVOT-002. | D-1179 |
| BC-2.10.011 E-CFG-100 self-contradiction | FIXED in v1.6 (D-1180). DO-NOT-REFLAG. | D-1180 |
| S-DEMO-004 PR #188 — all closures | MERGED develop@7241f5ef. DO-NOT-REFLAG. | PR #188 merged 2026-06-15 |
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 | RESOLVED-MECHANISM (D-1178): orchestrator CLAUDE.md authority granted; count bumps per-story at merge. | D-1178 |
| LAUNCHER PR #190 — all closures | MERGED develop@c3ecf6c8 (T11 DONE). DO-NOT-REFLAG. | PR #190 merged 2026-06-16 |
| PIVOT-001 PR #189 — all closures | MERGED develop@1b2e9a31 (T12 DONE). DO-NOT-REFLAG. | PR #189 merged 2026-06-16 |

---

### CONVERGENCE PROTOCOL REMINDERS (fresh session)

**BC-5.39.001 3-CLEAN protocol:**
- CLEAN(strict) = ZERO findings of ANY severity (CRIT+HIGH+MED+LOW+OBS+PROCESS-GAP). Required for streak advancement.
- CLEAN(PR-merge) = ZERO CRIT+HIGH+MED (non-blocking threshold only). Does NOT advance the 3-CLEAN streak.
- Adversary CLEAN reports MUST specify both: `CLEAN(strict): yes/no` + `CLEAN(PR-merge): yes/no`.
- Orchestrator dispatch uses STRICT criterion for fix-bursts.

**Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001):** Pushing any commit mid-PR-LEVEL cascade resets streak to 0/3. Re-gate on the pushed HEAD. The D-1234 PR-LEVEL streak was RESET by the CI-fix push 4133d186 (Windows path separator fix; pushed after f819a70d converged 3/3). Fresh streak on frozen 4133d186.

**TD-VSDD-005:** vsdd-factory:adversary tool-binding bug → use `general-purpose` agent as adversary (needs Grep/Bash for SAP-1/SAP-2 probes).

**Per adversary dispatch:** inject policy rubric from `.factory/policies.yaml`; apply SAP-1 (event_type↔BC-2.16.002 §Postconditions Canonical Structured Event Catalog) + SAP-2 (DTU↔TOML parity) + production-grade lens. Embed (worktree-abs-path, feature-HEAD-SHA, story-id, canonical-repo-root) tuple per DRIFT-ORCH-ADVERSARY-TUPLE-001.

**Lesson z25 (implementer discipline — bc/spec commits):** Implementer must NOT commit .factory/ artifacts directly. BC/spec commits route via state-manager dispatch.

**INDEPENDENT multi-pass rule (from lesson z24):** Run 3 INDEPENDENT fresh-context streak passes. Sequential single-pass cascades missed F-SV-1 hollow-feature for 8 passes. Each streak pass must be a fresh subagent with no prior context from earlier passes.

---

### QUEUED STORIES (after 3 active lanes converge)

| Story | Status | Depends On | Notes |
|-------|--------|------------|-------|
| S-5.04 | not-started | S-5.03 MERGED | Sensor Health Subsystem — remove-uncertainty before TDD; 5 pts; DEMO-CRITICAL-PATH |
| PIVOT-003 | not-started DEMO-BLOCKING | PIVOT-002 MERGED | Real IOC fields + canonical pivot proof; closes BC-2.06.019 §Interim State _ioc_value violation; closes TD-PLUGIN-P0-002 P0 |
| T13 narrative capstone | not-authored | S-1.14-REDO + PIVOT-003 + S-5.04 all MERGED | PO+story-writer; SOC-analyst workflow story |
| T14 demo recording | not-started | T13 MERGED | demo-recorder |
| S-5.11 | draft | post-demo | column-delta MCP notifications; OBS-S503-1 anchor |
| S-5.12 | draft | post-demo | add_sensor_spec MCP notifications |
| S-MAINT-EAUTH-COLLISION-001 | draft v1.0 | post-demo | E-AUTH-001/002 collision renumber; off critical path |

---

### TASK LEDGER (D-1234 — L-POST)

| ID | Status | Task | Notes |
|----|--------|------|-------|
| **L-A through L-T12** | **CLOSED** | S-5.02 / S-3.13 / T10 / T11 / T12 — all merged. | DONE. |
| **L-POST-A** | **DONE — MERGED PR #193** | S-1.14-REDO MERGED develop@5c747549 2026-06-19 (D-1235). POL-14 BC-2.19.002/004/005 draft→active. S-1.14 graduated ADR-020. ci.yml EXPECTED=70. | DONE |
| **L-POST-B** | **DONE — MERGED PR #194** | S-5.03 MERGED develop@85ac7b06 2026-06-19 (D-1238). POL-14 BC-2.08.005/BC-2.08.006/BC-2.10.008/BC-2.10.009 draft→active. CLAUDE.md non-exhaustive=76; ci.yml EXPECTED=76. | DONE |
| **L-POST-C** | **DONE — MERGED PR #195** | PIVOT-002 MERGED develop@9114e028 2026-06-19 (D-1240). POL-14 BC-2.19.001 idempotent (already active). CLAUDE.md non-exhaustive=79; ci.yml EXPECTED=79. | DONE |
| **L-POST-D** | not-started | S-5.04: remove-uncertainty → 12-gate TDD; depends S-5.03 MERGED. | After L-POST-B |
| **L-POST-E** | not-started | PIVOT-003: 12-gate TDD; depends PIVOT-002 MERGED. | **UNBLOCKED** (PIVOT-002 DONE) |
| **T13** | not-started (not-authored) | Capstone SOC-analyst narrative story: PO + story-writer dispatch after all above merged. | After L-POST-A/B/C/D/E |
| **T14** | blocked | Demo recording; depends T13 merged. | After T13 |

**Autonomy grant (D-989+D-1090):** FULL autonomous A→merge. Pause ONLY for: §7 spec-to-match-code amend / genuine product-business decision / Level-3 escalation / CLAUDE.md edit.

**Convergence rule:** every lane needs 3 CONSECUTIVE CLEAN(strict) passes. Any finding resets streak to 0/3. Orchestrator drives cascade (pr-manager lacks Agent tool).

---

### SYSTEMIC LESSON (z24 + DRIFT-HOLLOW-FEATURE-INTEGRATION-001)

**Lesson z24 — DRIFT-HOLLOW-FEATURE-INTEGRATION-001 (hollow-feature wiring class; 2026-06-14):**

Three stories in the current parallel batch (PIVOT-001, S-3.13, S-5.02) each shipped TDD-green + unit-tested in isolation but were NOT wired into the production boot path / engine call site. The pattern: implementer adds a new capability (new function, error code, UDF registration), writes unit tests against the new function, all Red Gate tests pass — but the production `main()` / `engine.rs` / `boot.rs` never calls the new entry point. A fresh LOCAL adversary with production-path tracing caught each one.

**Required gate (not yet in formal TDD flow):** After TDD green, before LOCAL adversary dispatch: "feature wired into production boot/engine AND real end-to-end path test exists (not just unit test of the new function in isolation)."

---

### INDEX VERSIONS (updated through D-1241)

| Artifact | Version | Notes |
|----------|---------|-------|
| STATE.md | v7.874 | D-1241 PrismQL LLM-onboarding design package; develop_head 9114e028 UNCHANGED (spec-only burst) |
| BC-INDEX | v6.82 | active 235 / draft 8 / retired 6; total 256; +6 new BCs (D-1241); 2 amendments (BC-2.10.009 v1.4, BC-2.11.001 v1.10) |
| STORY-INDEX | v2.434 | 204 stories; S-DEMO-PRISMQL-ONBOARDING-001 draft added (D-1241) |
| error-taxonomy | v1.91 | E-QUERY-038 column-not-found plan-time gate (D-1241); E-INFUSE-013 (D-1239) |
| ARCH-INDEX | v2.138 | ADR-041 v1.1 PROPOSED (4-layer LLM onboarding; D-1241) |
| VP-INDEX | v1.79 | 157 registered |
| prd | v1.12 | — |
| policies | v1.33 | POL-33 route_coverage_table_required_for_stagemask_changes |
| develop HEAD | `9114e028` | PIVOT-002 squash @6c367356 + D-1178 CLAUDE.md count bump 76→79 @9114e028 2026-06-19 (D-1240); UNCHANGED through D-1241 (spec-only) |
| Open PRs | none | S-5.03 PR #194 MERGED; PIVOT-002 PR #195 MERGED; no new PRs from spec-only burst |


### 2 ACTIVE LANES — Current SHAs + Phase + Next Action (D-1196 baseline)

> **IMPORTANT:** Verify against PINNED STATE table and live git before acting. If worktrees have advanced beyond these SHAs, live git is authoritative.

| Lane | Story | Worktree Path | Branch | HEAD (D-1224) | Phase / Streak | EXACT NEXT ACTION |
|------|-------|---------------|--------|---------------|----------------|-------------------|
| **T10 (DONE)** | S-DEMO-004 | REMOVED | `feature/S-DEMO-004 (DELETED)` | — | **MERGED PR #188 develop@7241f5ef** | **CLOSED.** |
| **T11 (DONE)** | LAUNCHER | STALE (cleanup pending) | `feature/S-DEMO-LAUNCHER-CONSOLIDATION-001 (DELETED)` | — | **MERGED PR #190 develop@c3ecf6c8** | **CLOSED.** T11 DONE. |
| **T12 (DONE)** | PIVOT-001 | STALE (cleanup pending) | `feature/S-DEMO-ENRICHMENT-PIVOT-001 (DELETED)` | — | **MERGED PR #189 develop@1b2e9a31** | **CLOSED.** T12 DONE. |
| **Lane A (CLOSED)** | S-5.02 | STALE (cleanup pending) | `feature/S-5.02 (DELETED)` | — | **MERGED PR #191 develop@bec894a2** | **CLOSED.** D-1202 2026-06-17. POL-14 BC-2.10.011 status:draft→active. EXPECTED=64. |
| **Lane B (CLOSED)** | S-3.13 | STALE (cleanup pending) | `feature/S-3.13 (DELETED)` | — | **MERGED PR #192 develop@60249ccc** | **CLOSED.** D-1204 2026-06-16. POL-14 BC-2.16.007 status:draft→active. EXPECTED=66. |
| **Lane D** | — | — | — | — | **CLOSED** (D-1168) | S-1.15 DROPPED. Permanently closed. |
| **L-POST-A (MERGES FIRST)** | S-1.14-REDO | `.worktrees/S-1.14-REDO` | `feature/S-1.14-REDO` | **2020dbf0** (CLEAN — D-1227) | IN PROGRESS — 0/3 streak | Adversary passes on frozen 2020dbf0 → LOCAL 3-CLEAN → PR-LEVEL 3-CLEAN → merge; EXPECTED 66→67 |
| **L-POST-B (INDEPENDENT)** | S-5.03 | `.worktrees/S-5.03` | `feature/S-5.03` | **5a444a5f** (clean) | IN PROGRESS — 0/3 streak | Fix 3 strict-only OBS → freeze HEAD → LOCAL 3-CLEAN → PR-LEVEL 3-CLEAN → merge; EXPECTED →72; OBS-3 → product-owner |
| **L-POST-C (BLOCKED on rebase)** | PIVOT-002 | `.worktrees/S-DEMO-ENRICHMENT-PIVOT-002` | `feature/S-DEMO-ENRICHMENT-PIVOT-002` | **0f958261** (current; pre-rebase) | BLOCKED — 0/3 streak | Await S-1.14-REDO merge → rebase → verify prod ThreatIntel enrichment → 3-CLEAN → merge; EXPECTED →76 |

---

### PR #188 CASCADE VERDICTS (S-DEMO-004 — CONVERGED; PR MERGED develop@7241f5ef)

| Pass / Review | Verdict | Key Findings | Status |
|---------------|---------|-------------|--------|
| pr-reviewer | **APPROVE POSTED** | 3 NITs (non-blocking). | Complete |
| security-reviewer | **CLEAR** | 0 CRIT/HIGH/MED. 6 LOW mitigated/accepted. | Complete — MAY PROCEED |
| PR-LEVEL adversary pass-1 | CLEAN(PR-merge)=YES / CLEAN(strict)=NO | LOW-1 demo-doc prose (claroty_assets → claroty_alerts); OBS-1 self-ref (no-action). | FIXED |
| PR-LEVEL adversary pass-2 | CLEAN(PR-merge)=NO / CLEAN(strict)=NO | MED-1: BC-2.22.001+BC-2.09.008 gap (frontmatter + body + AC traces). | FIXED (story v1.12) |
| PR-LEVEL adversary pass-3 | CLEAN(PR-merge)=NO / CLEAN(strict)=NO | F-PR3-MED-001: AC-003 claroty_alerts/SQL form; F-PR3-MED-002: AC-004 test name BC prefix. | FIXED (story v1.14) |
| PR-LEVEL adversary pass-5 | CLEAN(PR-merge)=YES / CLEAN(strict)=YES | Zero findings. Streak 1/3. | CONVERGED 1/3 |
| PR-LEVEL adversary pass-6 | CLEAN(PR-merge)=YES / CLEAN(strict)=YES | Zero findings. Streak 2/3. | CONVERGED 2/3 |
| PR-LEVEL adversary pass-7 | CLEAN(PR-merge)=YES / CLEAN(strict)=YES | Zero findings. Streak 3/3. | **CONVERGED 3/3** |
| CI | GREEN | 43/43 checks PASS | MERGED |
| **SQUASH-MERGE** | **develop@7241f5ef** | PR #188 merged 2026-06-15T05:48:52Z | **T10 DONE** |

---

### QUEUED STORIES (after current lanes converge)

| Story | Status | Depends On | Notes |
|-------|--------|------------|-------|
| S-5.03 | **IN PROGRESS — 3-CLEAN NEEDED** @5a444a5f v1.17 | S-5.02 MERGED + S-3.13 MERGED (SATISFIED) | CLEAN(PR-merge)=yes; fix 3 strict-only OBS → freeze HEAD → strict-clean 3-CLEAN; OBS-3 DEC-004 → product-owner S-5.04; EXPECTED →72; hard prereq of S-5.04 |
| S-5.04 | not-started | S-5.03 MERGED | Sensor Health Subsystem; 5 pts |
| PIVOT-002 | **IN PROGRESS — BLOCKED on rebase** @0f958261 v1.3 **DEMO-BLOCKING (D-1205)** | PIVOT-001 MERGED; BLOCKED on rebase onto S-1.14-REDO post-merge (F-SV-1 dep) | CRIT-1/2a/2b/HIGH-1+all HIGH/MED closed; security gates folded in as AC-007..012; EXPECTED →76; MERGES AFTER S-1.14-REDO |
| S-1.14-REDO | **IN PROGRESS — 3-CLEAN NEEDED** @2020dbf0 v1.2 CLEAN (D-1227) **DEMO-BLOCKING (D-1205) — MERGES FIRST** | S-WAVE5-PREP-01+S-3.02-FOLLOWUP-RUNTIME SATISFIED | all fixes committed (just check 4499 green; non-exhaustive gate 67); adversary passes on frozen 2020dbf0; EXPECTED 66→67 |
| PIVOT-003 | not-started **DEMO-BLOCKING (D-1205)** | PIVOT-002 MERGED | Real IOC fields + canonical pivot proof; closes BC-2.06.019 §Interim State _ioc_value violation; closes TD-PLUGIN-P0-002 P0 |
| T13 narrative capstone | not-authored | S-1.14-REDO + PIVOT-003 + S-5.04 all MERGED | PO+story-writer; SOC-analyst workflow story |
| T14 demo recording | not-started | T13 MERGED | demo-recorder |
| S-MAINT-EAUTH-COLLISION-001 | draft v1.0 | demo-capstone T13-T14 (post-demo) | E-AUTH-001/002 collision renumber (SpecEngineError OAuth2 → E-AUTH-008/009); off critical path |

**Merge-coordination note (MERGE-COORD D-1221):** S-1.14-REDO merges FIRST (EXPECTED 66→67). S-5.03 is INDEPENDENT (EXPECTED →72; no sequencing dep on S-1.14-REDO). PIVOT-002 BLOCKED on rebase onto S-1.14-REDO post-merge; after rebase: verify production ThreatIntel enrichment → 3-CLEAN → merge (EXPECTED →76). Each merge: orchestrator bumps CLAUDE.md count + ci.yml EXPECTED. S-5.03 depends_on S-3.13(SATISFIED).

---

### USER AUTHORIZATIONS (confirmed active — fresh session does NOT re-ask)

| Authorization | Decision | Date | Scope |
|---------------|----------|------|-------|
| Full Option-A enrichment framework REQUIRED | D-1164 | 2026-06-14 | 4 enrichment stories (S-1.14-REDO + PIVOT-001/002/003) REQUIRED before T13/T14 demo recording |
| Option-2 Rust launcher (start-multi CLI subcommand) | D-1166 | 2026-06-14 | S-DEMO-LAUNCHER-CONSOLIDATION-001 uses Rust start_multi path; crates_touched includes prism-dtu-demo-server |
| Parallel execution CAP LIFTED | D-1165 | 2026-06-14 | No fixed worktree cap; review-throughput is practical limiter (~3 in LOCAL cascade + 1 at PR-level simultaneously) |
| factory-artifacts push authorized (standing) | D-1066 | 2026-06-08 | State-manager PUSHES factory-artifacts to origin/factory-artifacts as FINAL step of every state burst |
| Full autonomous A→merge | D-989 + D-1090 | 2026-06-04 + 2026-06-10 | Pause ONLY for: §7 spec-to-match-code amend / genuine product-business decision / Level-3 escalation / CLAUDE.md edit |

---

### SYSTEMIC LESSON (z24 + DRIFT-HOLLOW-FEATURE-INTEGRATION-001)

**Lesson z24 — DRIFT-HOLLOW-FEATURE-INTEGRATION-001 (hollow-feature wiring class; 2026-06-14):**

Three stories in the current parallel batch (PIVOT-001, S-3.13, S-5.02) each shipped TDD-green + unit-tested in isolation but were NOT wired into the production boot path / engine call site. The pattern: implementer adds a new capability (new function, error code, UDF registration), writes unit tests against the new function, all Red Gate tests pass — but the production `main()` / `engine.rs` / `boot.rs` never calls the new entry point. A fresh LOCAL adversary with production-path tracing caught each one.

**Implication for all 4 parallel lanes:** Before declaring LOCAL adversary dispatch ready on any of these lanes, the orchestrator MUST instruct the adversary to explicitly trace the production invocation path from the real entry point (engine.rs / boot.rs / MCP server handler) to the new feature surface. Unit-test-only coverage is NOT sufficient to satisfy hollow-feature detection.

**Required gate (not yet in formal TDD flow — add per session-reviewer cycle-close):** After TDD green, before LOCAL adversary dispatch: "feature wired into production boot/engine AND real end-to-end path test exists (not just unit test of the new function in isolation)."

---

### INDEX VERSIONS (updated through D-1204)

| Artifact | Version | Notes |
|----------|---------|-------|
| STATE.md | v7.861 | D-1227 HEAD correction; develop_head 60249ccc UNCHANGED; 3 lanes: S-1.14-REDO@2020dbf0 CLEAN/S-5.03@5a444a5f/PIVOT-002@0f958261; merge sequencing locked (D-1221) |
| BC-INDEX | v6.69 | active 235 / draft 2 / retired 6; total 250; BC-2.16.007 v1.6 (D-1204 POL-14 draft→active legacy-field sync) |
| STORY-INDEX | v2.420 | 201 stories; S-3.13 v1.20 MERGED PR #192; S-5.02 v1.13 MERGED PR #191 |
| error-taxonomy | v1.86 | E-QUERY-037 org-scoped enumeration (D-1203 F-S313-PRL-MED-001 CLOSED) |
| ARCH-INDEX | v2.135 | ADR-039 v1.1 (D-1198) |
| VP-INDEX | v1.79 | 157 registered |
| prd | v1.12 | — |
| policies | v1.33 | POL-33 route_coverage_table_required_for_stagemask_changes |
| prismql-grammar | v1.1 | enrich function-call form |
| develop HEAD | `60249ccc` | feat(S-3.13): dynamic table availability; D-1204 merge |
| Open PRs | NONE | PR #191 MERGED (S-5.02 develop@bec894a2 D-1202). PR #192 MERGED (S-3.13 develop@60249ccc D-1204). All lanes CLOSED. |

### MERGE COORDINATION (COMPLETE — D-1204)

- **S-5.02** MERGED PR #191 develop@bec894a2 (D-1202 2026-06-17). **S-3.13** MERGED PR #192 develop@60249ccc (D-1204 2026-06-16). BOTH lanes CLOSED.
- **Cumulative #[non_exhaustive] count RECONCILED** (develop@60249ccc): EXPECTED=66. ci.yml EXPECTED=66 CONFIRMED. CLAUDE.md reads '66 types...EXPECTED=66' CONFIRMED. S-5.02 +4 (StructuredErrorFields, CapabilityEntry, ResolutionStep, CapabilityStatus) + S-3.13 +2 (TableNotAvailableDetails + TableRegistry) = 66 total. Consistent across ci.yml / CLAUDE.md on develop@60249ccc.

### POST-LANE ROADMAP (North Star = multi-client SOC-analyst live demo)

After all 4 active lanes merge, the queued sequence (per `.factory/objectives/multi-client-soc-demo-tasks.md`):

| Story | Depends On | Status | Notes |
|-------|-----------|--------|-------|
| S-5.03 (Resources/Prompts) | S-5.02 MERGED + S-3.13 MERGED (SATISFIED) | **IN PROGRESS — 3-CLEAN NEEDED** @5a444a5f v1.17 | CLEAN(PR-merge)=yes; fix 3 strict-only OBS → freeze HEAD → 3-CLEAN; OBS-3 DEC-004 → product-owner S-5.04; EXPECTED →72; INDEPENDENT |
| S-5.04 (Sensor Health) | S-5.03 MERGED | not-started | 5 pts; remove-uncertainty before TDD |
| PIVOT-002 | PIVOT-001 MERGED | **IN PROGRESS — BLOCKED on rebase** @0f958261 v1.3 **DEMO-BLOCKING (D-1205)** | BLOCKED on rebase onto S-1.14-REDO post-merge (F-SV-1 dep); security gates addressed (AC-007..012); EXPECTED →76; MERGES AFTER S-1.14-REDO |
| S-1.14-REDO | S-WAVE5-PREP-01+S-3.02-FOLLOWUP-RUNTIME SATISFIED | **IN PROGRESS — 3-CLEAN NEEDED** @2020dbf0 v1.2 CLEAN (D-1227) **DEMO-BLOCKING (D-1205) — MERGES FIRST** | all fixes committed (just check 4499 green); adversary passes on frozen 2020dbf0; EXPECTED 66→67 |
| PIVOT-003 | PIVOT-002 MERGED | not-started **DEMO-BLOCKING (D-1205)** | Adds real IOC fields to Cyberint/CrowdStrike DTU fixtures + canonical pivot proof; closes BC-2.06.019 §Interim State _ioc_value violation; closes TD-PLUGIN-P0-002 P0 |
| T13 capstone (SOC-analyst narrative) | All above MERGED | not-authored | PO + story-writer; the demo's capstone deliverable |
| T14 demo recording | T13 MERGED | not-started | demo-recorder |

---

### PARKED WORKTREES (leave alone)

| Worktree | Status | Action |
|----------|--------|--------|
| `.worktrees/S-3.09` | FROZEN | Leave alone |
| `.worktrees/W3-FIX-S307-001` | BLOCKED/superseded | Leave alone |

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) — T1–T10+T4-A DONE. T11 (LAUNCHER) PR #190 OPEN (PR-LEVEL 0/3 re-pass IN FLIGHT). PIVOT-001 PR #189 OPEN (PR-LEVEL 0/3 re-pass IN FLIGHT). S-3.13 v1.15 LOCAL CONVERGED 3/3. S-5.02 @ea06ff52 LOCAL CONVERGED 3/3. |
| **develop HEAD** | `1b2e9a31` (feat(S-DEMO-ENRICHMENT-PIVOT-001) 2026-06-16 D-1192; UNCHANGED through D-1194) |
| **STATE version** | v7.825 |
| **BC-INDEX version** | v6.60 (total 250; active 235; draft 2; retired 6; BC-2.19.001 v1.7 D-1181 two-phase fix; BC-2.10.011 v1.6; BC-2.06.017 v1.10 active; BC-2.06.018 v1.6 active; BC-2.06.019 v1.7 active; BC-2.06.020 v1.6 active) |
| **STORY-INDEX version** | v2.398 (total_stories 200; S-3.13 v1.15/rg22 LOCAL CONVERGED 3/3 @31b4d147; PIVOT-001 v1.10 @e87e44ea validators wired; LAUNCHER v2.8 @8e183f03 SEC-001 closed; S-5.02 v1.8 @ea06ff52 BC-2.10.011 v1.6 LOCAL CONVERGED 3/3) |
| **VP-INDEX version** | v1.79 (158 registered) |
| **ARCH-INDEX version** | v2.133 |
| **error-taxonomy version** | v1.81 (E-INFUSE-007 PIVOT-001 HIGH-1 UDF-registration failure; E-QUERY-037 boxed emitter + strsim) |
| **ADR-036 version** | v2.3 (time_anchor 5-arg ruling) |
| **policies version** | v1.33 (POL-33 route_coverage_table_required_for_stagemask_changes) |
| **prd version** | v1.12 |
| **Open PRs** | **PR #189 OPEN** (feature/S-DEMO-ENRICHMENT-PIVOT-001 @e87e44ea; pr-reviewer APPROVE + security CLEAR; PR-LEVEL adversary 0/3 re-pass IN FLIGHT). **PR #190 OPEN** (feature/S-DEMO-LAUNCHER-CONSOLIDATION-001 @037c44f3; pr-reviewer APPROVE + security CLEAR; SEC-001 CLOSED; PR-LEVEL 0/3 re-pass IN FLIGHT). PR #188 MERGED develop@7241f5ef (T10; D-1176 2026-06-15). PR #185 MERGED develop@7fd35b77 (T5). PR #186 MERGED develop@f7400f83 (D-1143). PR #187 MERGED develop@664566e9 (T6; D-1158). |
| **T10 branch** | `feature/S-DEMO-004`; MERGED at develop@7241f5ef (2026-06-15); worktree+branch cleaned |
| **factory-artifacts** | PUSHED to origin/factory-artifacts (D-1066; D-1181 burst) |

---

### 2. §GOAL (unchanged — do not override)

Active objective: **multi-client SOC-analyst live demo**. T5 in PR-LEVEL cascade. E2E sequence to completion:

1. ~~Review cycle (3 fix-PRs #183/#184/#182)~~ DONE (D-1103).
2. ~~Register burst (25 items)~~ DONE (D-1103).
3. ~~T5 story-writer materialize + remove-uncertainty + LOCAL 3-CLEAN strict (13 passes)~~ DONE.
4. ~~T5 PR-LEVEL pass 1~~ DONE (BPRL-P1-01 LOW stale 3-guard comment; closed 45323267).
5. ~~T5 PR-LEVEL pass 2~~ DONE (BPRL-P2-01 MED cyberint alerts StageMask projection; closed 4eadb027).
6. ~~T5 PR-LEVEL pass 3~~ DONE (BPRL-P3-01 MED CLAUDE.md 50→52 in-PR; BPRL-P3-OBS-1/OBS-2 fixed; closed 2323cf37+13efc875; D-1108).
7. ~~T5 PR-LEVEL pass 4~~ DONE (BPRL-P4-01 MED CLOSED-BY-DEFERRAL D-1109; BPRL-P4-02 LOW fixed bc0f36c5; BPRL-P4-PG-01 closed POL-33; BC-2.06.019 v1.4; D-1109/D-1110).
8. ~~T5 PR-LEVEL pass 12~~ DONE (BPRL-P12-01 MED closed D-1118 — VP-020-K false-green replaced with genuine demo-server integration test; cyberint membership duplicate removed; feature HEAD 7ddc0a51).
9. ~~T5 PR-LEVEL pass 13~~ DONE (D-1119: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; streak 1/3).
9. ~~T5 PR-LEVEL pass 14~~ DONE (D-1120: BPRL-P14-01 HIGH SPEC-ONLY closed — BC-2.06.020 v1.4 PC-9 + story B AC-019 RNG range literal 0..100000→0..10000; code UNCHANGED 7ddc0a51; streak RESET 0/3).
9. ~~T5 PR-LEVEL pass 15~~ DONE (D-1121: BPRL-P15-01 MED SPEC-ONLY closed — story B Phase-6 gate instruction "19 RGTs"→"23 RGTs"; story B v2.13; code UNCHANGED 7ddc0a51; streak 0/3).
9. ~~T5 PR-LEVEL pass 16~~ DONE (D-1122: CLEAN(strict)=YES; CLEAN(PR-merge)=YES — zero findings; exhaustive spec-consistency audit; sub-threshold item story line-47 "~16 tests" tilde-qualified estimate dispositioned below OBS; streak 0/3→1/3).
9. ~~T5 PR-LEVEL pass 17~~ DONE (D-1123: CLEAN(strict)=YES; CLEAN(PR-merge)=YES — zero findings; full behavioral trace 5 stages x 6 clones + cross-BC + wiring + SAP-1 + S-7.01; novelty LOW; streak 1/3→2/3).
9. ~~T5 PR-LEVEL pass 20~~ DONE (D-1126: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; zero findings; BPRL-P19-01 closure verified; streak 0/3→1/3).
9. ~~T5 PR-LEVEL pass 21~~ DONE (D-1127: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; zero findings; 8 independent re-derivation axes all PASS; novelty LOW; streak 1/3→2/3).
9. ~~T5 PR-LEVEL pass 22~~ DONE (D-1128: BPRL-P22-01 MED SPEC-ONLY VP-Anchors prose A..H/8→A..L/12; BC-2.06.020 v1.5; story B v2.14; orchestrator caught+reverted catalog-format regression; CLEAN(strict)=no; streak RESET 2/3→0/3).
9. ~~T5 PR-LEVEL pass 23~~ DONE (D-1130: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; zero findings; DRIFT-2/3 re-derivation + VP-020-K case-sound + PC-8/PC-9 + E-DEMO-002 + SAP-1; novelty LOW; streak 0/3→1/3).
9. ~~T5 PR-LEVEL pass 24~~ DONE (D-1131: BPRL-P24-01 LOW [process-gap] — false-coverage INV-PERIMETER-COMPLIANCE-001 enforcement mechanism; DTU perimeter IS Cargo-structural; user-directed prose-correction; BC-2.06.020 v1.6; story B v2.16; code 15bedc12; streak RESET 1/3→0/3).
9. ~~T5 PR-LEVEL pass 25~~ DONE (D-1135: BPRL-P25-01 MED — propagation gap from P24; threatintel test module comment + rustdoc + evidence-report line-195 still cited `perimeter-violation/`; corrected 6f6e744e + 69c53cb9; streak RESET 0/3).
9. ~~T5 PR-LEVEL pass 26~~ DONE (D-1136: BPRL-P26-01 MED/process-gap — sibling-site miss; evidence-report lines 74+155 + tape line 9 still "perimeter gate passes"; demo-recorder dd84c76c; one-time no-verify D-1134; exhaustive grep ZERO residual; streak RESET 0/3).
9. ~~T5 PR-LEVEL pass 27~~ DONE (D-1137: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; zero findings; 9 novel non-perimeter angles all PASS; perimeter-prose fully converged 6 surfaces / 3 passes; streak 0/3→1/3).
9. ~~T5 PR-LEVEL passes 28/29~~ DONE (CONVERGED 3/3 strict — zero findings in all 3 clean passes).
9. ~~pr-reviewer APPROVE + security-reviewer MAY PROCEED~~ DONE (re-run at dd84c76c).
9. ~~CI green → squash-merge~~ DONE (PR #185 merged develop@7fd35b77 2026-06-13).
9. ~~post-merge burst (POL-14)~~ DONE (D-1139: BC-2.06.019+BC-2.06.020 draft→active; active 232→234).
10. **CURRENT: T6 — S-DEMO-MULTI-TENANT-DTU-001** (ready v1.3; BC-2.06.017 draft; 8 pts; remove-uncertainty DONE D-1144; EXPECTED re-baselined 52→59; NEXT: vsdd-factory:deliver-story).
11. T8 → capability-discovery block (D-1107) → S-DEMO-ENRICHMENT-PIVOT-001/002/003 chain → T11 → T13 capstone.

---

### 3. §T5 STORY STATUS

**S-DEMO-DTU-LIVE-SCENARIO-001-B** — scenario progression + enrichment correlation live demo. **MERGED PR #185 develop@7fd35b77 2026-06-13.**

| Field | Value |
|-------|-------|
| **Story version** | v2.16 MERGED (19 ACs / 23 Red Gate tests; D-1131 BPRL-P24-01: AC-016 + Architecture Compliance row + Phase-6 gate item + RGT row 16 corrected to structural Cargo/E0432 enforcement; BC-2.06.020 pin v1.5→v1.6; code 15bedc12; counts UNCHANGED) |
| **BC-2.06.019** | v1.7 **ACTIVE** (D-1139 POL-14 draft→active; D-1113 fabricated inventory-note prose corrected; D-1112 Claroty devices Route Coverage row + exhaustive inventory note; v1.5: D-1111 Route Coverage Table corrected + PC-4 5-arg prose; v1.4: D-1109 per-sensor IOC-surface matrix + Interim State clause + Route Coverage Table; PRE-6 org_id guard per PO OBS-1 ruling) |
| **BC-2.06.020** | v1.6 **ACTIVE** (D-1139 POL-14 draft→active; D-1131 BPRL-P24-01: INV-PERIMETER-COMPLIANCE-001 body + Architecture Anchors corrected — `tests/external/perimeter-violation/` covers prism-query BC-2.11.006 only; DTU perimeter enforced STRUCTURALLY via Cargo/E0432; v1.5: D-1128 BPRL-P22-01 SPEC-ONLY VP Anchors prose A..H/8→A..L/12; v1.4: D-1120 SPEC-ONLY PC-9 0..100000→0..10000; v1.3: D-1117 PC-8+PC-9+INV-CYBERINT-ALERT-CVE-CORRELATION-001+VP-020-I..L) |
| **ADR-036** | v2.3 (time_anchor 5-arg ruling) |
| **Demo evidence** | 19/19 ACs COMPLETE (commit f75f3159; VHS; docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/; AC-019 re-recorded 0863184a: both crate commands; all 4 VP-020 tests demonstrated) |
| **LOCAL cascade** | CONVERGED 3/3 strict (13 passes at pre-D-1117 code; D-1117 adds 3 commits; D-1118 adds 2 more commits) |
| **PR-LEVEL streak** | **3/3 CONVERGED** (passes 27/28/29 all CLEAN(strict)=YES; full pass ledger: passes 1-26 with BPRL closures listed in cascade ledger below; pass 27 D-1137; passes 28/29 CLEAN strict; CONVERGED at 29 — PR merged develop@7fd35b77) |
| **Branch HEAD** | dd84c76c = final PR HEAD (D-1136: 2 demo-evidence files corrected); squash-merged develop@7fd35b77 |

**T5 CASCADE LEDGER (complete pass-by-pass history — for zero-context reconstruction):**

| Pass | Type | Result | Streak | Key Event |
|------|------|--------|--------|-----------|
| L-1 | LOCAL | 4 findings | 0/3 | B-P1-01 CRIT route-projection + B-P1-02 CRIT vacuous tests |
| L-2 | LOCAL | 1 finding | 0/3 | B-P2-01 HIGH Claroty dev-join key split |
| L-3 | LOCAL | 1 finding | 0/3 | B-P3-01 stage-0 guard (BC stage-0 tension ruling) |
| L-4 | LOCAL | 1+2obs | 0/3 | B-P4-01 E-DEMO-003 hoist; obs: E-DEMO-006 org_id guard (PO OBS-1 ruling → PRE-6) |
| L-5 | LOCAL | 5+2obs | 0/3 | B-P5 set: renumber, signature alignment, UUID canonicalization, Arc-threading |
| L-6 | LOCAL | 0 (CLEAN) | 1/3 | — |
| L-7 | LOCAL | 2 findings | 0/3 | B-P7-01 rustdoc accuracy; B-P7-02 BC-2.06.020 pin alignment |
| L-8 | LOCAL | 0 (CLEAN) | 1/3 | — |
| L-9 | LOCAL | 1 finding | 0/3 | B-P9-01 [[test]] required-features |
| L-10 | LOCAL | 1 finding | 0/3 | F-P10-01 required-features DTU-conditional tests |
| L-11 | LOCAL | 0 (CLEAN) | 1/3 | — |
| L-12 | LOCAL | 0 (CLEAN) | 2/3 | — |
| L-13 | LOCAL | 0 (CLEAN) | **3/3 CONVERGED** | LOCAL 3-CLEAN strict. Then D-1117 added 3 code commits. |
| PRL-1 | PR-LEVEL | 1 finding LOW | 0/3 | BPRL-P1-01 stale 3-guard comment; closed 45323267 |
| PRL-2 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P2-01 cyberint alerts StageMask; closed 4eadb027 |
| PRL-3 | PR-LEVEL | 1 MED + 2 OBS | 0/3 | BPRL-P3-01 CLAUDE.md 50→52 in-PR (human decision D-1108); OBS-1/2 fixed 2323cf37+13efc875 |
| PRL-4 | PR-LEVEL | 1 MED+1 LOW+1 PG | 0/3 | BPRL-P4-01 CLOSED-BY-DEFERRAL (IOC masking → PIVOT-003); BPRL-P4-02 bc0f36c5; BPRL-P4-PG-01 POL-33 |
| PRL-5 | PR-LEVEL | 1 finding HIGH | 0/3 | BPRL-P5-01 Route Coverage Table defects; BC-2.06.019 v1.5 |
| PRL-6 | PR-LEVEL | 1 finding HIGH | 0/3 | BPRL-P6-01 Claroty devices row missing; BC-2.06.019 v1.6 |
| PRL-7 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P7-01 fabricated grep-claim; BC-2.06.019 v1.7 |
| PRL-8 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P8-01 BC-INDEX row-120 stale pin v2.4→v2.9 |
| PRL-9 | PR-LEVEL | 0 (CLEAN) | 1/3 | — |
| PRL-10 | PR-LEVEL | 0 (CLEAN) | 2/3 | — |
| PRL-11 | PR-LEVEL | 0 (CLEAN) | **3/3** [INVALIDATED D-1117] | D-1117 code change followed: SEC-001 fix + cyberint CVE↔NVD correlation. Streak 3/3→0/3. |
| — | D-1117 | CODE CHANGE | 0/3 | SEC-001 CVE-9999-{:05}; CyberintClone new_with_scenario gains &catalog; BC-2.06.020 v1.3 PC-8+PC-9+INV-CYBERINT-ALERT-CVE-CORRELATION-001+VP-020-I..L; error-taxonomy v1.78 E-DEMO-006; code advances 13efc875→f0b6b8c7 |
| PRL-12 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P12-01 VP-020-K false-green; genuine demo-server integration test 9219ce76; dedup 7ddc0a51 |
| PRL-13 | PR-LEVEL | 0 (CLEAN) | 1/3 | — |
| PRL-14 | PR-LEVEL | 1 finding HIGH | 0/3 | BPRL-P14-01 SPEC-ONLY PC-9 0..100000→0..10000; BC-2.06.020 v1.4; story B v2.12 |
| PRL-15 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P15-01 SPEC-ONLY Phase-6 gate "19 RGTs"→"23 RGTs"; story B v2.13 |
| PRL-16 | PR-LEVEL | 0 (CLEAN) | 1/3 | Exhaustive D-1117 audit; story line-47 "~16 tests" adjudicated below-OBS |
| PRL-17 | PR-LEVEL | 0 (CLEAN) | 2/3 | Full behavioral trace 5 stages x 6 clones + cross-BC + SAP-1 + S-7.01 |
| PRL-18 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P18-01 AC-019 evidence 3 fabricated/inverted BC anchors; demo-recorder 5d5484d0 |
| PRL-19 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P19-01 AC-019 tape omitted VP-020-K; re-recorded 0863184a (both crate commands) |
| PRL-20 | PR-LEVEL | 0 (CLEAN) | 1/3 | BPRL-P19-01 closure verified; core invariants + SAP-1 + EXPECTED=52 |
| PRL-21 | PR-LEVEL | 0 (CLEAN) | 2/3 | 8-axis independent re-derivation; novelty LOW — "genuinely converged" |
| PRL-22 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P22-01 SPEC-ONLY VP Anchors prose A..H/8→A..L/12; BC-2.06.020 v1.5; story B v2.14 |
| — | D-1129 | CONSISTENCY-SWEEP | 0/3 | DRIFT-1/2/3: STORY-INDEX PIVOT-003 pin v1.5; Cyberint 6-arg in §Tasks/FSR/build_clone_pairs; story B v2.15 |
| PRL-23 | PR-LEVEL | 0 (CLEAN) | 1/3 | DRIFT-2/3 re-derivation; VP-020-K case-sound; PC-8/PC-9; E-DEMO-002; SAP-1 |
| PRL-24 | PR-LEVEL | 1 finding LOW | 0/3 | BPRL-P24-01 [process-gap] DTU perimeter prose corrected to Cargo/E0432; BC-2.06.020 v1.6; story B v2.16; code 15bedc12 |
| PRL-25 | PR-LEVEL | 1 finding MED | 0/3 | BPRL-P25-01 propagation gap: threatintel test module comment + rustdoc + evidence-report line-195 still cited `perimeter-violation/`; 6f6e744e + 69c53cb9 |
| PRL-26 | PR-LEVEL | 1 finding MED/PG | 0/3 | BPRL-P26-01 sibling-site miss: evidence-report lines 74+155 + tape line 9 "perimeter gate passes"; dd84c76c; one-time no-verify push D-1134; exhaustive grep ZERO residual |
| PRL-27 | PR-LEVEL | 0 (CLEAN) | 1/3 | Zero findings; 9 novel non-perimeter angles all PASS; perimeter-prose fully converged 6 surfaces / 3 passes |
| PRL-28 | PR-LEVEL | 0 (CLEAN) | 2/3 | Zero findings; streak 1/3→2/3 |
| PRL-29 | PR-LEVEL | 0 (CLEAN) | **3/3 CONVERGED** | Zero findings; streak 2/3→3/3. PR #185 merged develop@7fd35b77. |

**Pass report files:** `cycles/wave-5-e-demo-fidelity/S-DEMO-DTU-LIVE-SCENARIO-001-B/adversarial-review/pr-pass-NN.md` (NN = pass number).

**LOCAL cascade trajectory (13 passes at pre-D-1117 code):** P1:4 → P2:1 → P3:1 → P4:1+2obs → P5:5+2obs → P6:0 → P7:2 → P8:0 → P9:1 → P10:1 → P11:0 → P12:0 → P13:0

**Key LOCAL closures (do not reflag):**
- B-P1-01 CRIT: route projection missing (all sensor routes now explicitly call `with_stage_mask_projection`)
- B-P1-02 CRIT: vacuous tests (bare TOML stubs not reaching guarded path; per lesson v class)
- B-P2-01 HIGH: Claroty dev- join key (split org_id + sensor_id from raw dev-key)
- B-P3-01: EC-019-012 contradiction guard (stage_idx > 0 guard; BC stage-0 tension ruling)
- B-P4-01: guard order E-003 hoist (E-DEMO-003 before stage_mask compute)
- E-DEMO-006: org_id guard — PO OBS-1 ruling; BC-2.06.019 v1.2 PRE-6 added
- B-P5 set: renumber, signature alignment, UUID canonicalization, Arc-threading
- B-P7-01: rustdoc accuracy; B-P7-02: BC-2.06.020 pin alignment in story body
- B-P9-01/F-P10-01: `[[test]]` required-features for DTU-conditional tests

**PR-LEVEL pass history:**
- Pass 1: 4 directed probes PASS; BPRL-P1-01 LOW (stale 3-guard comment in cyberint_client.rs) closed 45323267. CLEAN(strict)=no; CLEAN(PR-merge)=yes. Streak 0/3.
- Pass 2: BPRL-P2-01 MED (cyberint alerts route — StageMask projection unimplemented; §FSR + BC-2.06.019 PC-4 required it; wrong LOCAL-P2 adjudication had exempted cyberint citing "§Tasks only" — spec-wins per Source-of-Truth rule-7). HTTP-level IOC-filter tests Red→Green. Threats route adjudicated static-fixture out-of-PC-4-scope. Doc comments corrected. Closed 4eadb027. CLEAN(strict)=no; CLEAN(PR-merge)=yes. Streak 0/3.
- Pass 3: BPRL-P3-01 MED (CLAUDE.md #[non_exhaustive] count stale 50 vs EXPECTED=52; D-1108 human decision ratified in-PR delivery — supersedes D-1106 post-merge plan); BPRL-P3-OBS-1 (cyberint fail-closed ioc_type; new test); BPRL-P3-OBS-2 (crowdstrike containment-precedence doc comment). SAP-1 PASS; SAP-2 N/A. Fixed commits 2323cf37+13efc875; sibling sweep 13efc875. CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 4: BPRL-P4-01 MED (IOC-surface production-inert; generator only stamps `_ioc_value` synthetic sentinel; real Cyberint/CrowdStrike IOC fields unpopulated). CLOSED-BY-DEFERRAL per D-1109 human decision (design-faithful path; BC-2.06.019 v1.3→v1.4 per-sensor IOC-surface matrix + Interim State clause + Route Coverage Table; deferred to S-DEMO-ENRICHMENT-PIVOT-003). BPRL-P4-02 LOW (CrowdStrike detections served primary-device records at stage 0; sibling sweep added armis alerts in:alerts guard). CLOSED commit bc0f36c5. BPRL-P4-PG-01 process-gap (no Route Coverage Table). CLOSED BC-2.06.019 v1.4 + POL-33. SAP-1 PASS; POL-22 A+C PASS; pass-2/3 fixes verified load-bearing. CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 5: BPRL-P5-01 HIGH (BC-2.06.019 v1.4 Route Coverage Table row defects vs code: phantom crowdstrike alerts_search.rs row; wrong method+path for summaries — real `POST /detects/entities/summaries/GET/v1`; missing armis search.rs `GET /api/v1/search` UNGUARDED row; `stage_idx >= 2` wording vs real `mask.lateral_devices` field). LOW note: PC-4 prose stale 4-arg constructor. All CLOSED — BC-2.06.019 v1.4→v1.5 (PO); story B v2.6→v2.7 (2 pins); PIVOT-003 v1.1→v1.2 (30 pins exhaustive). BC-INDEX v6.33. STORY-INDEX v2.360. Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only). SAP-1 PASS; SAP-2 N/A; POL-22 A+C PASS. CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 6: BPRL-P6-01 HIGH [process-gap] (BC-2.06.019 v1.5 Route Coverage Table missing Claroty `routes/devices.rs` / `GET /api/v2/devices` — StageMask-guarded in PR diff; load-bearing for AC-015; second consecutive table-completeness miss — passes 5 and 6 both verified existing rows without exhaustive inventory of all guarded route files in diff). All 7 existing v1.5 rows verified accurate; only Claroty devices row was absent. CLOSED — BC-2.06.019 v1.5→v1.6 (PO: Claroty devices row added + exhaustive inventory verification note embedded under table; 7-file StageMask handler scan; 8-row EXHAUSTIVE); story B v2.7→v2.8 (2 pins); PIVOT-003 v1.2→v1.3 (all body-level pins). BC-INDEX v6.34. STORY-INDEX v2.361. Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only). SAP-1 PASS. CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 7: BPRL-P7-01 MED [process-gap] (BC-2.06.019 v1.6 inventory verification note contained fabricated grep claim — prose asserted claroty/alerts.rs "appears in both grep sets due to `scenario_stage_ctx` references" but zero stage/mask references exist in that file; EXEMPT determination itself correct on real-API grounds). All other axes verified clean: 7-file inventory re-run PASS; BC-2.06.020 invariants PASS; E-DEMO-006 byte-exact PASS; SAP-1 PASS; forbidden-pattern sweep PASS; DormantTenant regression PASS; demo evidence 18/18 ACs PASS; frontmatter-body coherence PASS. CLOSED — BC-2.06.019 v1.6→v1.7 (PO: single fabricated sentence corrected; claroty/alerts.rs does NOT appear in either grep set; EXEMPT stands solely on real-API grounds; no table/semantic change); story B v2.8→v2.9 (2 pins); PIVOT-003 v1.3→v1.4 (33+ pins). BC-INDEX v6.35. STORY-INDEX v2.362. Story B HEAD bc0f36c5 UNCHANGED (BC-side fix only). CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 8: BPRL-P8-01 MED [process-gap] (BC-INDEX row 120 (BC-2.06.020) carried stale `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.4 (B-P5-03 2026-06-12)` — D-1113 burst updated row 119 (BC-2.06.019) correctly but missed sibling row 120; story B current version v2.9). All code axes verified clean (code unchanged since pass 4). Exhaustive annotation sweep: VP-INDEX/ARCH-INDEX carry no version pins; PIVOT-001/002/003 BC-INDEX rows have no version pins — zero additional stale hits. CLOSED — BC-INDEX row-120 `ready v2.4`→`ready v2.9 (D-1114 2026-06-12)` (state-manager: index-row annotation only; no BC semantic change); BC-INDEX v6.36. Story B HEAD bc0f36c5 UNCHANGED (index-row annotation only; no code change). Lesson z8 appended (shared-anchor-story index rows must be swept as a CLASS). CLEAN(strict)=no; CLEAN(PR-merge)=no. Streak 0/3.
- Pass 9: ZERO findings of any severity. 15 verification axes all PASS: BC-INDEX rows 119/120 both `ready v2.9` CURRENT; BC-2.06.019 v1.7 8-row EXHAUSTIVE Route Coverage Table PASS; BC-2.06.020 v1.2 invariants PASS; E-DEMO-006 verbatim taxonomy↔BC↔code PASS; SAP-1 PASS; SAP-2 N/A; forbidden-pattern sweep PASS; DormantTenant Red Gate test 17 PASS; demo evidence 18/18 PASS; frontmatter-body coherence PASS; story B HEAD bc0f36c5 = remote (diff unchanged) PASS; BC pin consistency PASS; cross-record index consistency PASS. Novelty LOW. PR fully spec-sanctioned and production-grade. CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak 1/3.
- Pass 10: ZERO findings of any severity. Novel angles probed: scenario-state concurrency (Arc<IncidentTimeline>, no Mutex, pure-function engine — PASS); repeated-construction determinism (seeded RNG, no thread_rng() in prod — PASS); stage-boundary saturation arithmetic vs ADR-036/TV vectors — PASS; Cargo.lock chrono unification (workspace-resolved, no fragmentation — PASS); required-features test isolation (DTU-conditional tests correctly gated — PASS); rustdoc text-fenced non-doctest (all new code blocks marked `text`/`ignore` — PASS). BC H1↔INDEX↔subsystem↔version sync verified both BCs. All BPRL-P1 through BPRL-P9 do-not-reflag items confirmed still closed. CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak 2/3.
- Pass 17: ZERO findings of any severity. Full holdout-style behavioral trace (5 attack stages x 6 clones: Armis/Claroty/CrowdStrike/Cyberint/NVD/ThreatIntel) + cross-BC consistency (BC-2.06.019 v1.7 ↔ BC-2.06.020 v1.4 shared catalog/seed) + build_clone_pairs wiring (guard order E-DEMO-002→006→003→004, all 6 constructors) + POL-12 stub-residue PASS + SAP-1 PASS + S-7.01 SEC-001 sibling-drift PASS (CVE-9999-{:05} in all 6 clones; no sibling using old CVE-202x-* format) + forbidden patterns PASS. All BPRL-P1 through BPRL-P16 do-not-reflag items confirmed closed. Novelty LOW. CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak 1/3→2/3.
- Pass 18: BPRL-P18-01 MED — AC-019 demo-evidence artifacts (`AC-019-cyberint-cve-pivot.tape` header + `evidence-report.md` line 89) contained 3 fabricated/inverted BC anchors: (1) PC-8↔PC-9 labels inverted (PC-8=scenario catalog / PC-9=baseline namespace — canonical; evidence had them reversed); (2) fabricated invariant `INV-CYBERINT-CVE-PIVOT-001` (canonical: `INV-CYBERINT-ALERT-CVE-CORRELATION-001`); (3) fabricated type `CveCorrelationCatalog` (canonical: `ScenarioEntityCatalog`). All code/tests/ACs/BCs/story CORRECT. All convergence-positive checks PASS. CLEAN(strict)=NO; CLEAN(PR-merge)=YES. Streak RESET 2/3→0/3. Closed: demo-recorder commit 5d5484d0.

---

### 4. §T6 DISPATCH INSTRUCTIONS (S-DEMO-MULTI-TENANT-DTU-001)

**T5 DONE. T6 IN PROGRESS. PR #185 merged develop@7fd35b77 (T5). PR #186 merged develop@f7400f83 (D-1134 remediation; no open PRs). D-1144: remove-uncertainty re-run DONE. Dispatch sequence:**

**REMOVE-UNCERTAINTY STATUS: COMPLETE (D-1144 2026-06-13).** Story v1.3 (was v1.2). Key corrections: (1) EXPECTED gate re-baselined from stale `49→56` to correct `52→59` (+7 delta: 6 E0639 struct arms + 1 E0004 enum arm; ci.yml ground-truth EXPECTED=52 post-001-A AC-014 + 001-B growth). (2) Stale "does NOT import prism-dtu-*" claim reworded (now imports prism-dtu-common per Story A). Story file: `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (ready v1.3; BC-2.06.017 draft v1.1; 8 pts; SS-01; CAP-036).

**CRITICAL FOR IMPLEMENTER/TEST-WRITER:** ci.yml line `EXPECTED=52` is the current count. This story adds 7 new `#[non_exhaustive]` arms (6 E0639 struct arms from MultiInstanceConfig/InstanceEntry/MultiInstanceHarness/HarnessEntry/HarnessError variants + 1 E0004 enum arm). Update ci.yml to `EXPECTED=59` at the commit that adds those types. Also update `scripts/check-non-exhaustive.sh` and `tests/external/non-exhaustive-violation/src/struct_violations.rs` in the same commit per the prior pattern (D-1076 U-006, Story A commit pattern).

**STEP: `vsdd-factory:deliver-story S-DEMO-MULTI-TENANT-DTU-001`** — full 12-gate per-story TDD delivery (remove-uncertainty already done; start from worktree-manage).

---

_ARCHIVED: T5 PR-LEVEL pass 28 dispatch instructions (now superseded — cascade converged). The do-not-reflag list for T5 closures is preserved in the cascade ledger above in §3._

**[Historical reference only — T5 pass 27 ground truth at dd84c76c]:**

**Ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `dd84c76c`; PR #185
- Adversary MUST re-materialize PR diff via `gh pr diff 185` — use `dd84c76c` as the HEAD; same diff as pass 27 but must re-materialize to avoid stale cached diffs
- ALL code reads use `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B/` absolute path
- Verify `git -C .worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H %s'` matches `dd84c76c` before citing any line numbers
- BC-2.06.019 is v1.7 — use the v1.7 Route Coverage Table (8 rows, exhaustive); do NOT cite v1.6 or earlier inventory-note prose
- BC-2.06.020 is v1.6 — use v1.6; includes INV-PERIMETER-COMPLIANCE-001 + Architecture Anchors corrected to structural Cargo/E0432 enforcement (`tests/external/perimeter-violation/` covers prism-query BC-2.11.006 only — stated explicitly); VP Anchors `VP-020-A through VP-020-L` / `all 12 VPs`; PC-8, PC-9 (range `0..10000`); do NOT cite v1.5 or earlier enforcement-mechanism prose
- BC-INDEX rows 119/120 anchor story pin is `ready v2.16 (D-1131 2026-06-13)` — do NOT cite v2.15 or earlier annotations
- Story B is v2.16 — BC-2.06.020 pin is v1.6; AC-016 describes structural Cargo/E0432 enforcement for DTU perimeter (NOT perimeter-violation gate); Phase-6 gate instruction reads "all 23 Red Gate tests pass"; Cyberint new_with_scenario is 6-arg in §Tasks/FSR/build_clone_pairs; RGT row 16 type is `unit` (NOT `unit+compile-fail`)
- Streak is 1/3; this is pass 28
- IMPORTANT do-not-raise: catalog `gen_device_cves` uses `CVE-9999-{:05}` (5-digit); Cyberint baseline generator uses `CVE-9999-{:04}` (4-digit) — these are TWO DISTINCT GENERATORS by design; the digit-width difference is intentional; DO NOT raise this as an inconsistency
- IMPORTANT do-not-raise: evidence-report.md and AC-013-014-016-enrichment-correlation.tape (dd84c76c) now use structural Cargo/E0432 framing for the DTU perimeter — perimeter-prose is FULLY CONVERGED; DO NOT re-raise ANY perimeter-prose framing finding (see perimeter-prose ruling below in do-not-reflag list)

**Full do-not-reflag list for pass 28 and all subsequent passes (do NOT raise these as new findings):**

All LOCAL closures listed in §3 above, plus:
- BPRL-P1-01 closed (stale 3-guard comment; 45323267)
- BPRL-P2-01 closed (cyberint alerts StageMask; 4eadb027)
- BPRL-P3-01 closed (CLAUDE.md 50→52 in-PR per D-1108; 2323cf37+13efc875)
- BPRL-P3-OBS-1 closed (cyberint fail-closed ioc_type match; new test; 2323cf37)
- BPRL-P3-OBS-2 closed (crowdstrike containment-precedence doc comment; 2323cf37)
- EXPECTED sibling sweep closed (scripts/check-non-exhaustive.sh + struct_violations.rs 50→52; 13efc875)
- BPRL-P4-01 CLOSED-BY-DEFERRAL: IOC-surface production inertness (generator stamps `_ioc_value` synthetic sentinel only; real Cyberint/CrowdStrike IOC fields unpopulated). ADJUDICATED per D-1109 human decision. BC-2.06.019 v1.4 Interim State clause governs. Anchored to S-DEMO-ENRICHMENT-PIVOT-003. **Raising "IOC masking inert" again = re-raising an adjudicated deferral — DO NOT REFLAG.**
- BPRL-P4-02 closed (CrowdStrike detections stage-0 guard; Armis alerts in:alerts guard; bc0f36c5)
- BPRL-P4-PG-01 closed (Route Coverage Table in BC-2.06.019 v1.4; POL-33 registered)
- BPRL-P5-01 closed: BC-2.06.019 v1.4→v1.5 Route Coverage Table corrected. Story B v2.7. PIVOT-003 v1.2.
- BPRL-P6-01 closed: BC-2.06.019 v1.5→v1.6 Claroty devices row + exhaustive inventory note. Story B v2.8. PIVOT-003 v1.3.
- BPRL-P7-01 closed: BC-2.06.019 v1.6→v1.7 fabricated inventory-note prose corrected. Story B v2.9. PIVOT-003 v1.4.
- BPRL-P8-01 closed: BC-INDEX row-120 story-version pin sync v2.4→v2.9 (D-1114); updated to v2.10 (D-1117); now updated to v2.11 (D-1118). **DO NOT REFLAG.**
- BPRL-P9: pass 9 had ZERO findings — no do-not-reflag entries.
- BPRL-P10: pass 10 had ZERO findings — no do-not-reflag entries. Novel angles (scenario-state concurrency, RNG determinism, stage-boundary saturation, Cargo.lock unification, required-features isolation, rustdoc text-fenced non-doctest) all verified PASS — do NOT re-probe as fresh findings.
- BPRL-P11: pass 11 had ZERO findings — no do-not-reflag entries (pass INVALIDATED by D-1117 code change; closures still stand).
- **SEC-001 CLOSED (D-1117):** `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs` — CVE-202x-* format collision with real NVD namespace. CLOSED: changed to `CVE-9999-{:05}` (year 9999 sentinel; never used by real NVD). **Raising "synthetic CVE IDs could collide with NVD" again = closed finding — DO NOT REFLAG.**
- **D-1117 cyberint CVE↔NVD correlation CLOSED:** `CyberintClone::new_with_scenario` (f0b6b8c7) gained `&catalog` parameter; `generate_cves` draws `cve_id` values from `catalog.device_cves` (cyclic assignment) — scenario-mode Cyberint CVEs now correlate end-to-end to NVD. BC-2.06.020 v1.3 PC-8+PC-9+INV-CYBERINT-ALERT-CVE-CORRELATION-001. 4 new unit tests. AC-019 (evidence f75f3159). **Raising "cyberint alert CVEs don't resolve in NVD" or "no end-to-end CVE pivot chain" again = closed finding — DO NOT REFLAG.**
- Historical 001-A evidence-report EXPECTED=50 citations — point-in-time records; do NOT reflag
- Evidence-report cites recording-HEAD 785adc4b — benign (demo recordings before final fixups; evidence matches spec intent)
- `Arc::try_unwrap` documented panic — intentional (only unwrap after exclusive ownership; documented)
- B-P1-04 fallback behavior — by design per ADR-036
- Test 12 by-design (static fixture test for threats route; out of PC-4 scope per adjudication)
- `tracing::error!` in scenario engine — exempt from SAP-1 structured event catalog (non-audit diagnostic; per PO OBS-1 ruling)
- "2 leaky" (low-level DTU fixture gen output) — pre-existing; not a Story B change
- HashMap determinism — fixed-order iteration via sorted keys (closed in LOCAL cascade)
- Cargo.toml:15 locator — version-pin format lint; not a behavioral finding
- Pre-existing harness no-features lib errors — pre-existing build config; not a Story B regression
- BC stage-0 tension (`stage_idx > 0` guard) — by-design per PO ruling
- Armis key-presence discriminator (P10-01) — NO-ACTION adjudication; human ratification requested
- Baseline (non-scenario) `CVE-9999-{:04}` format — intentionally non-pivotable (PC-9 by-design; Cyberint baseline doesn't correlate to NVD; ONLY scenario-mode is required to correlate per PC-8)
- **BPRL-P12-01 CLOSED (D-1118):** VP-020-K / TV-020-013 was a false-green (test in wrong crate `prism-dtu-cyberint`; never called `NvdState::lookup_and_count`; doc comment cited nonexistent demo-server test). Genuine integration test now at `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs::test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` (9219ce76). Cyberint membership duplicate removed (7ddc0a51). **DO NOT re-raise "cyberint resolves test is only membership", "duplicate test name `_resolves_in_nvd`", or "no demo-server integration test for NVD pivot" — all CLOSED.**
- **PASS-13 COSMETIC NIT (D-1119 adjudication) — DO NOT REFLAG:** Stale doc-comment in `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs` (~lines 16–20) references the now-deleted same-named cyberint test ("the prism-dtu-cyberint copy ... is a Half-B membership guard"). The membership coverage now lives under VP-020-J (`test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog`). Pass-13 adversary adjudicated this as cosmetic; no behavioral impact; CLEAN strict unaffected. Anchored as opportunistic cleanup to S-DEMO-ENRICHMENT-PIVOT-003. **DO NOT raise as a finding in pass 15 or any subsequent pass.**
- **BPRL-P14-01 CLOSED (D-1120 SPEC-ONLY):** BC-2.06.020 PC-9 implementer directive + story B AC-019 carried `rng.gen_range(0..100000)` (5-digit upper bound), contradicting the spec's own `^CVE-9999-\d{4}$` invariant + TV-020-011. Shipped code was correct (`0..10000`). BC-2.06.020 v1.4 PC-9 directive now reads `0..10000`; story B AC-019 literal is `0..10000`; invariant, TV-020-011, and code are all consistent. **DO NOT re-raise "RNG range `0..100000` contradicts `\d{4}` regex", "AC-019 range literal inconsistent with format invariant", or "spec-self-contradiction in PC-9 range" — CLOSED.**
- **BPRL-P15-01 CLOSED (D-1121 SPEC-ONLY):** Story B Phase-6 gate instruction (line ~581) carried "all 19 Red Gate tests pass" — stale count from pre-D-1117 revision. Story B v2.13 Phase-6 gate instruction now reads "all 23 Red Gate tests pass". Exhaustive sweep confirmed this was the sole stale gate-count prose; all other `19` occurrences are AC count (correct) or RGT row-index labels. **DO NOT re-raise "gate instruction says 19", "Phase-6 gate skips cyberint-correlation RGTs", or "red_gate_tests count mismatch in gate instruction" — CLOSED.**
- **PASS-16 SUB-THRESHOLD DISPOSITION (D-1122) — DO NOT REFLAG:** Story line ~47 points-justification comment contains "Red Gate test suite (~16 tests, FAIL-first): 1 pt" — a tilde-qualified estimate in the FROZEN 7-point breakdown rationale. `points: 7` has never changed. Live RGT count (23) is consistent across: frontmatter `red_gate_tests: 23`, 23-row RGT table, Phase-6 gate instruction "all 23 Red Gate tests pass", STORY-INDEX. The `~16 tests` annotation is NOT a count-of-record surface; it is a tilde-qualified effort estimate in frozen authoring-time rationale, analogous to historical changelog prose. Pass-16 adversary adjudicated this below-OBS threshold. Anchored as opportunistic cleanup to S-DEMO-ENRICHMENT-PIVOT-003. **DO NOT raise as a finding in pass 21 or any subsequent pass.**
- **BPRL-P18-01 CLOSED (D-1124):** AC-019 demo-evidence artifacts (`AC-019-cyberint-cve-pivot.tape` header + `evidence-report.md` line 89) contained 3 fabricated/inverted BC anchors: PC-8↔PC-9 labels inverted (canonical: PC-8=scenario catalog assignment / PC-9=baseline namespace isolation); fabricated invariant `INV-CYBERINT-CVE-PIVOT-001` (canonical: `INV-CYBERINT-ALERT-CVE-CORRELATION-001`); fabricated type `CveCorrelationCatalog` (canonical: `ScenarioEntityCatalog`). All 3 corrected in demo-recorder commit 5d5484d0. `rg` confirms fabricated names gone, canonical present. NO re-render (anchors were header-comment-only). **DO NOT re-raise "inverted PC-8/PC-9 labels in evidence", "INV-CYBERINT-CVE-PIVOT-001 not found", or "CveCorrelationCatalog not found" — CLOSED.**
- **BPRL-P19-01 CLOSED (D-1125):** AC-019 tape command (`-p prism-dtu-cyberint` only) omitted VP-020-K after BPRL-P12-01 relocated `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` to `prism-dtu-demo-server`. Evidence-report claimed all 4 VP-020 tests pass but tape only showed 3 (VP-020-I/J/L). Re-recorded in demo-recorder commit 0863184a: both commands — `-p prism-dtu-cyberint` (VP-020-I/J/L; 3 PASS) + `-p prism-dtu-demo-server -E test(cyberint_alert_cve_resolves_in_nvd)` (VP-020-K; 1 PASS). VHS re-render succeeded; evidence-report corrected to two-crate split (cyberint=3 VP-020-I/J/L, demo-server=10 incl VP-020-K). **DO NOT re-raise "AC-019 tape command only runs cyberint tests", "VP-020-K not shown in evidence", or "evidence-report overstates VP-020 coverage as 4/4 when tape only shows 3" — CLOSED.**
- **BPRL-P22-01 CLOSED (D-1128 SPEC-ONLY):** BC-2.06.020 §VP Anchors prose had `VP-020-A through VP-020-H` / `all 8 VPs` — stale from before D-1117 added VP-020-I..L. Corrected in BC-2.06.020 v1.5: `VP-020-A through VP-020-L` / `all 12 VPs`. Story B v2.14 (BC-2.06.020 pin v1.4→v1.5; 3 sites). PIVOT-003 v1.7 (2 sites). **DO NOT re-raise "VP Anchors prose says VP-020-H", "only 8 VPs listed in VP Anchors", or "prose says all 8 VPs but table has 12" — CLOSED. ALSO DO NOT raise "catalog uses {:05} vs {:04} inconsistency" — these are two DISTINCT generators by design (gen_device_cves catalog=5-digit per mod.rs:449+SEC-001+TV-020-012; Cyberint baseline=4-digit per generator.rs:389+PC-9+TV-020-011) — the digit-width difference is INTENTIONAL.**
- **DRIFT-1 CLOSED (D-1129 consistency-sweep):** STORY-INDEX PIVOT-003 row trailing `2 BCs:` inline annotation carried `BC-2.06.020 v1.3` — corrected to `v1.5`. **DO NOT re-raise "PIVOT-003 STORY-INDEX inline annotation says v1.3" or "STORY-INDEX row lists v1.3 but BC is v1.5" — CLOSED.**
- **DRIFT-2/3 CLOSED (D-1129 consistency-sweep):** Story B §Tasks Phase-2 Cyberint task prose, §FSR clone.rs row, and §Tasks Phase-4 build_clone_pairs Cyberint call carried stale 5-arg `new_with_scenario`. Corrected to 6-arg per AC-019 + BC-2.06.020 PC-8 + shipped code 0863184a (code was always correct). Full sweep: Cyberint=6-arg (3 sites); other operational clones=5-arg; ThreatIntel/NVD=1-arg — all correct. **DO NOT re-raise "story B §Tasks shows 5-arg Cyberint constructor", "FSR clone.rs row shows 5-arg", or "build_clone_pairs Cyberint call shows 5-arg" — CLOSED.**
- **PASS-23 D-1130 RE-CONFIRMATIONS (do NOT re-raise as new findings in pass 25):** All BPRL-P1..P22 closures re-confirmed independently. VP-020-K case-sound by double-uppercase-normalize in registry+lookup. PC-8 cyclic catalog assignment (`catalog.device_cves[i % len]`). PC-9 baseline `CVE-9999-{:04}` non-pivotable per design. E-DEMO-002 prescan guard order confirmed. SAP-1 PASS. Artifact cluster assessed as fully converged.
- **BPRL-P24-01 CLOSED (D-1131 prose-correction):** INV-PERIMETER-COMPLIANCE-001 enforcement-mechanism claim was false across 4 surfaces (AC-016 body / BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 body / BC Architecture Anchors bullet / threatintel test comment): all cited `tests/external/perimeter-violation/` as enforcing the DTU perimeter. That gate covers the **prism-query pub-API perimeter only** (BC-2.11.006) and has zero dependency on any DTU crate. The DTU perimeter (`prism-dtu-threatintel` and `prism-dtu-nvd` must not import `prism-spec-engine`, `prism-sensors`, or `prism-query`) IS held STRUCTURALLY: the forbidden deps simply don't appear in `Cargo.toml`, so any violation is a standard E0432 compile error in the workspace build. User decision: prose-correction (structural framing), NOT a new compile-fail gate. All 4 surfaces corrected: BC-2.06.020 v1.6 (PO); story B v2.16 (story-writer); test comment (implementer commit 15bedc12). **DO NOT re-raise "DTU perimeter gate missing", "no compile-fail test for DTU perimeter", "tests/external/perimeter-violation/ should cover DTU crates", or "INV-PERIMETER-COMPLIANCE-001 not enforced by a dedicated gate" — user ratified structural Cargo/E0432 enforcement as adequate; prose-correction over gate-build.**
- **BPRL-P25-01 CLOSED (D-1135 propagation fix):** Partial-fix propagation gap from D-1131. Two residual false-framing sites after P24 fix: (1) `crates/prism-dtu-threatintel/tests/bc_2_06_020_enrichment_correlation.rs` module comment + rustdoc still cited `tests/external/perimeter-violation/` as enforcing the DTU perimeter (propagation miss from D-1131 which only corrected threatintel test comment, not module-level comment/rustdoc); (2) `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/evidence-report.md` line ~195 perimeter table row still said "perimeter gate passes". Corrected: implementer commit `6f6e744e` (threatintel test module comment + rustdoc → structural Cargo/E0432 framing) + demo-recorder commit `69c53cb9` (evidence-report line-195 → structural framing). **DO NOT re-raise "threatintel test module comment cites perimeter-violation gate", "threatintel rustdoc cites perimeter-violation gate", or "evidence-report perimeter row says 'gate passes'" — CLOSED.**
- **BPRL-P26-01 CLOSED (D-1136 sibling-site sweep):** Pass-26 adversary found that the P25 fix (D-1135) missed two sibling sites in `evidence-report.md` (lines 74 and 155) and one in `AC-013-014-016-enrichment-correlation.tape` (line 9): all three still used "perimeter gate passes" / compile-fail framing. Demo-recorder commit `dd84c76c` (2 files: evidence-report.md + AC-013-014-016-enrichment-correlation.tape). User authorized ONE-TIME `git push --no-verify` exception for this docs-only commit (D-1134); durable remediation: PR #186 (D-1138). Orchestrator ran exhaustive grep across all demo-evidence files confirming ZERO residual `perimeter.*gate passes` or `compile-fail gate` false-framing. Only legitimate "gate passes" expression remaining: `ci.yml EXPECTED=52` at line 192 (structural; correct). Spec layer (story B v2.16 / BC-2.06.020 v1.6 / PIVOT-003 v1.8) was already-correct — no version bump required. **DO NOT re-raise "evidence-report lines 74/155 say perimeter gate passes", "tape line 9 says compile-fail gate", or any perimeter-prose framing finding in evidence-report or tape files — CLOSED.**
- **PERIMETER-PROSE FULLY CONVERGED — DO NOT RE-RAISE ANY PERIMETER-PROSE FRAMING FINDING (pass 27 D-1137 ruling):** The perimeter-prose surface has been corrected across ALL 6 surfaces over 3 passes: (P24 D-1131: AC-016 body / BC-2.06.020 INV-PERIMETER-COMPLIANCE-001 body / BC Architecture Anchors / threatintel test comment); (P25 D-1135: threatintel test module comment + rustdoc / evidence-report line-195); (P26 D-1136: evidence-report lines 74+155 / tape line 9). Pass 27 adversary confirmed ZERO residual false-framing across all demo-evidence surfaces. The only legitimate "gate passes" reference remaining in demo-evidence is `ci.yml EXPECTED=52` at line 192 (structural; correct). **DO NOT re-raise ANY perimeter-prose framing finding — "DTU perimeter gate missing", "no compile-fail test for DTU perimeter", "perimeter-violation/ should cover DTU crates", "INV-PERIMETER-COMPLIANCE-001 not enforced by dedicated gate", "evidence says gate passes", "tape says compile-fail gate", or any variant. The perimeter enforcement is STRUCTURAL (Cargo/E0432). All prose surfaces are CORRECTED. Convergence is complete. Only legitimate cite: ci.yml EXPECTED=52 line 192.**

**Post-convergence sequence (after 3-CLEAN strict at dd84c76c or later head):**
1. pr-reviewer RE-RUN → APPROVE (MUST re-run — code changed via D-1117/PRL-12/PRL-14/PRL-15/PRL-18/PRL-19/PRL-24/PRL-25-26 since pass-11 reviews on bc0f36c5; prior reviews are stale)
2. security-reviewer RE-RUN → MAY PROCEED (MUST re-run — same reason; prior security review on bc0f36c5 is stale)
3. CI green on final head (verify `gh pr checks 185`)
4. Admin squash-merge to develop
5. Post-merge state-manager burst: POL-14 (BC-2.06.019 v1.7 + BC-2.06.020 v1.6 draft→active); STORY-INDEX status update; STATE bump. (CLAUDE.md EXPECTED 50→52 is DONE — merged in-PR per D-1108; no post-merge human edit needed.)

---

### 5. §D-1107 CAPABILITY-DISCOVERY SCOPE-IN

**D-1107 USER DECISION (2026-06-12):** capability-discovery block opted INTO demo scope.

| Story | Status | Depends on | Notes |
|-------|--------|------------|-------|
| S-5.02 | not-started | S-5.01 | Tool Routing/Errors/Client Scoping |
| S-5.03 | not-started | S-5.02 | Resources and Prompts (hard dep of S-5.04) |
| S-5.04 | not-started | S-5.03, S-DEMO-001 | Sensor Health Subsystem |
| S-3.13 | not-started | S-3.02, S-1.12 | Dynamic Table Availability (parallel after PO authors BCs) |

**Updated build sequence:** T5 DONE → T6 IN PROGRESS (ready v1.3; remove-uncertainty DONE D-1144; NEXT: vsdd-factory:deliver-story) → T8 (architect+PO reconcile first) → S-5.02 → S-5.03 → S-5.04 (+ S-3.13 parallel after PO BCs) → T11 (pending launcher-lifecycle decision) → T13 capstone.

**Pre-delivery requirement (D-1061):** run `dclaude:remove-uncertainty` on EVERY one of the 4 opted-in stories before TDD.

---

### 6. §ALSO NOTE (durability items)

- **PENDING HUMAN items (non-blocking for T5 delivery; required before later stories):**
  - CLAUDE.md SAP-1 wording clarification (register-burst item 18k)
  - Armis key-presence discriminator ratification (P10-01 do-not-reflag; human sign-off needed for final ratification)
  - T11 S-DEMO-LAUNCHER-CONSOLIDATION-001 launcher-lifecycle decision (required before T11/T12)
  - ~~CLAUDE.md EXPECTED 50→52 sentence~~ DONE in-PR (D-1108; commits 2323cf37+13efc875)
  - CLAUDE.md DEFER-CLAUDEMD-BC216002-MISLABEL-001, DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001, DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 (all human-only edits)

---

### 7. §PARKED WORKTREES

| Worktree | Status | Action |
|----------|--------|--------|
| `.worktrees/S-3.09` | FROZEN | Leave alone |
| `.worktrees/W3-FIX-S307-001` | BLOCKED/superseded | Leave alone |
| `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B` | MERGED — may be cleaned up | PR #185 merged develop@7fd35b77 2026-06-13 |

---

### 8. §RESUME PROTOCOL COMMANDS

```bash
# 1. Factory worktree health (BLOCKING preflight)
# Use: vsdd-factory:factory-worktree-health skill

# 2. Verify develop HEAD == 1b2e9a31 (or newer if a lane merged)
git log --oneline -1 origin/develop

# 3. Verify STATE.md version
grep '^version:' /Users/jmagady/Dev/prism/.factory/STATE.md
# Expected: version: "7.837"

# 4. Confirm active worktrees (S-DEMO-004 worktree REMOVED post-merge)
ls /Users/jmagady/Dev/prism/.worktrees/
# Expected: S-3.09 + W3-FIX-S307-001 (parked) + S-5.02 + S-3.13 + S-DEMO-ENRICHMENT-PIVOT-001 + S-DEMO-LAUNCHER-CONSOLIDATION-001

# 5. Verify each worktree HEAD against D-1182 PINNED STATE
git -C /Users/jmagady/Dev/prism/.worktrees/S-5.02 rev-parse --short HEAD           # expect ea06ff52
git -C /Users/jmagady/Dev/prism/.worktrees/S-3.13 rev-parse --short HEAD           # expect 31b4d147
git -C /Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-001 rev-parse --short HEAD  # expect e87e44ea
git -C /Users/jmagady/Dev/prism/.worktrees/S-DEMO-LAUNCHER-CONSOLIDATION-001 rev-parse --short HEAD  # expect 8e183f03

# 6. Confirm factory-artifacts pushed (expect D-1182 burst commit at HEAD)
git -C /Users/jmagady/Dev/prism/.factory log -1 --format='%h %s'

# 7. Check open PRs
gh pr view 189  # PIVOT-001 — expect OPEN, head e87e44ea
gh pr view 190  # LAUNCHER — expect OPEN, head 8e183f03

# 8. Apply lessons and execute TASK LEDGER in §RESUME SNAPSHOT D-1182
# NEXT: L1 evidence-report rg5→rg7 fix (demo-recorder) → adversary re-pass; L2 adversary re-pass; L0 demo evidence → PR; L3 demo evidence → PR after PIVOT
```

---

### 9. Where Extracted History Lives

| Content | Archive Location |
|---------|-----------------|
| D-1103 §RESUME SNAPSHOT (superseded by D-1106) | `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md` |
| Per-story cascade pass tracking (STATE.md YAML frontmatter keys for 25+ stories) | `cycles/wave-5-e-demo-fidelity/frontmatter-cascade-archive.md` |
| Decision rows D-700..D-1054 | `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md` |
| Burst narratives (D-735..D-1084) | `cycles/wave-5-e-demo-fidelity/burst-log.md` |
| Lessons learned (incl. lessons a–y) | `cycles/wave-5-e-demo-fidelity/lessons.md` |
| Wave-0 history | `cycles/wave-0-plugin-prereqs/` |
| Wave-3 history | `cycles/wave-3-multi-tenant/` |
| Wave-4 history | `cycles/wave-4-operations/` |

Full pre-compaction STATE.md and SESSION-HANDOFF.md are preserved in git history on the `factory-artifacts` branch.

---

## §Standing Orchestrator Process Rules

These rules are canonical in CLAUDE.md and SESSION-HANDOFF.md. Listed here for reference.

1. **BC-5.39.001 3-CLEAN strict convergence (D-779).** CLEAN(strict) = zero findings of ANY severity. CLEAN(PR-merge) = zero CRIT+HIGH+MED. Streak advances ONLY on CLEAN(strict). Adversary CLEAN reports MUST specify both criteria.

2. **Single-commit-per-burst (TD-VSDD-053).** Each logical burst → ONE commit in `.factory/`. Multi-commit chains trigger MULTI_COMMIT_CHAIN_NOT_ALLOWED. No Stage-1/Stage-2/backfill chains.

3. **Anti-volatile-pin (TD-VSDD-091).** Narrative spec content must cite function names + behavioral anchors, NOT `file.rs:NNN` line numbers. Justified citations (Red Gate test tables, AC source-of-truth tables, pass-report changelogs) excepted.

4. **Paper-fix detection (TD-VSDD-059).** Adversary must verify every claimed closure has a load-bearing test or assertion, not just doc-comment or rename.

5. **Sibling-site sweep (TD-VSDD-060).** When changing a function signature, constant, or canonical identifier, grep ALL callsites in the same crate (and adjacent crates if pub) before committing.

6. **AD-017 credential opaqueness.** Credentials never transit AI context; reference-based model with CLI/env/vault paths. OrgSlug::new_unchecked is test-helpers-feature-gated.

7. **Source-of-Truth Precedence.** Later/more-specific artifact wins. Story spec supersedes BC for implementation scope. ADR supersedes earlier ADR. Code vs spec: SPEC WINS (Standing Rule for VSDD). Only human can authorize spec amendment to match code (§7).

8. **POL-14 auto-promotion.** When a story's PR merges, BCs in `behavioral_contracts` frontmatter auto-promote draft→active. State-manager runs this transition.

9. **D-989 autonomy scope.** Full autonomous Wave-5 execution. Pause only for §7 amend / product-business decision / Level-3 escalation / CLAUDE.md edit.

10. **factory-artifacts PUSH-AFTER-EACH-BURST (user-authorized D-1066, 2026-06-08).** The state-manager PUSHES factory-artifacts to origin/factory-artifacts as the FINAL step of every state burst (off-machine durability). Push is `git -C .factory push origin factory-artifacts` (normal push, NOT force-push, NOT to main/develop).

11. **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001, D-1065).** After ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix-commit causes the adversary to review stale code.

12. **Review-cycle pinned merge order (D-1091, updated D-1101).** QRY MERGED. MCP merge-reconciliation COMPLETE (head 08fdc38c) — pr-manager delivery NEXT. DTU last because PR #182 custody + DTU cascade must run to LOCAL CONVERGED first.

13. **Worktree-path read discipline (D-1097, lesson p).** Adversary dispatches MUST instruct "ALL code reads, grep/rg searches, and line-number citations MUST use the worktree absolute path." Orchestrator MUST run ground-truth check (direct rg in worktree) before dispatching any fix-burst on a CRIT claim.

14. **Long-gate discipline (D-1099, lesson r).** Long gates (pre-push `just check`, CI, PR review waits) run harness-tracked in orchestrator context or via Monitor-equipped agents. Sub-agents MUST NOT be dispatched to wait on long gates.

---
