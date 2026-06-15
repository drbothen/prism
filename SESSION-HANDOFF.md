---
document_type: session-handoff
level: ops
version: "7.819"
status: current
timestamp: 2026-06-15T00:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-1176 BURST (2026-06-15) — RESUME-SESSION CASCADE ROUND: S-DEMO-004 MERGED (PR #188 develop@7241f5ef 2026-06-15T05:48:52Z; LOCAL 3/3 strict CONVERGED + PR-LEVEL 3/3 strict CONVERGED passes 5/6/7; CI 43/43; POL-14 all 9 BCs already active — idempotent). S-3.13 v1.13 + LAUNCHER v2.5 story edits committed. 4 drift items recorded. PIVOT-001 LOCAL 2/3. S-3.13 LOCAL 0/3 (duplicate rename IN-FLIGHT). LAUNCHER LOCAL 0/3 (re-pass IN-FLIGHT). S-5.02 BLOCKED on human CLAUDE.md 60→64 commit. STATE v7.819.**
>
> **PRIORITY READ ORDER:** Read §ACTIVE OBJECTIVE (North Star) FIRST, then §RESUME SNAPSHOT D-1176 below (contains RESTART PROTOCOL + TASK LEDGER), then STATE.md frontmatter. All prior D-1101..D-1175 notes SUPERSEDED.
> **SOURCE-OF-TRUTH FOR CURRENT PIPELINE POSITION:** STATE.md frontmatter (`develop_head`, `current_step`) + §RESUME SNAPSHOT D-1176 below are AUTHORITATIVE for current position and next action. `.factory/objectives/DEMO-SCOPE.md` is the demo SCOPE and NARRATIVE reference — its STATUS values track build progress but it is NOT the live pipeline position tracker.
> develop HEAD `7241f5ef` (PR #188 squash-merge 2026-06-15; D-1176 post-merge burst). factory-artifacts PUSHED to origin/factory-artifacts (D-1066 standing authorization; D-1176 burst). STATE v7.819.

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
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | **draft stub** (D-1029; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human launcher-lifecycle decision needed) | 0 stub (TBD) | -- | S-DEMO-003 (SATISFIED) | T11 story-writer materialization → T12 delivery |
| 6 — capstone | **Multi-client SOC-analyst narrative story** (not yet named or authored) | **not-authored** (no story file, no STORY-INDEX row; owner: product-owner + story-writer; after data layer + tooling exist) | TBD | TBD | Orders 3+4+5 complete | T13 → T14 demo recording; the demo's capstone deliverable |
| **D-1107 SCOPE-IN** | **S-5.02** | not-started (wave 5) | 3 | 2 proxy | S-5.01 | Tool Routing/Errors/Client Scoping — **OPTED IN (D-1107 2026-06-12)**; remove-uncertainty before TDD (D-1061) |
| **D-1107 SCOPE-IN** | **S-5.03** | not-started (wave 5) | -- | -- | S-5.02 | Resources and Prompts — hard dep of S-5.04; **OPTED IN (D-1107)**; remove-uncertainty before TDD |
| **D-1107 SCOPE-IN** | **S-5.04** | not-started (wave 5) | 5 | -- | S-5.03, S-DEMO-001 | Sensor Health Subsystem — **OPTED IN (D-1107)**; remove-uncertainty before TDD |
| **D-1107 SCOPE-IN** | **S-3.13** | not-started (wave 3) | 3 | 3 proxy (needs PO BCs) | S-3.02, S-1.12 | Dynamic Table Availability — **OPTED IN (D-1107)**; parallel after PO authors dedicated BCs; remove-uncertainty before TDD |

**NEXT CONCRETE ACTION: T11 — LAUNCHER v2.5 @d9098c1f LOCAL adversary re-pass (streak 0/3; IN-FLIGHT). PARALLEL: PIVOT-001 @e4d95d19 LOCAL pass-3 (streak 2/3 — one more clean pass → PR); S-3.13 @97148f90 implementer duplicate-test rename → LOCAL re-pass (streak 0/3); S-5.02 @8eaff098 BLOCKED on human CLAUDE.md 60→64 commit. T1–T10+T4-A ALL DONE. D-989+D-1090 autonomy grant remains active.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: T11 (LAUNCHER v2.5 @d9098c1f IN PROGRESS; LOCAL 0/3). T1+T2+T3+T4+T4-A+T5+T6+T8+T9+T10 DONE. PR #188 MERGED develop@7241f5ef (D-1176 T10 DONE). ADR-036 v2.3. BC-INDEX v6.58. ARCH-INDEX v2.133. STORY-INDEX v2.393. error-taxonomy v1.81. VP-INDEX v1.79 (158). policies v1.33. prd v1.12. BC-2.06.017 v1.10 ACTIVE. BC-2.06.018 v1.6 ACTIVE. BC-2.06.019 v1.7 ACTIVE. BC-2.06.020 v1.6 ACTIVE. STATE v7.819 (D-1176 cascade-round burst).**

---

## §RESUME SNAPSHOT — D-1176 (2026-06-15 — CASCADE ROUND CLOSE; T10 DONE; 4 active lanes; NO open PRs; STATE v7.819)

> **D-1176: CASCADE ROUND COMPLETE. T10 DONE — S-DEMO-004 PR #188 MERGED develop@7241f5ef 2026-06-15T05:48:52Z (LOCAL 3/3 strict CONVERGED + PR-LEVEL 3/3 strict CONVERGED passes 5/6/7 on frozen 89942715; CI 43/43; POL-14 all 9 BCs already active — idempotent no-ops). T11 IN PROGRESS — LAUNCHER v2.5 @d9098c1f LOCAL streak 0/3. PIVOT-001 @e4d95d19 LOCAL 2/3 (one more clean pass → PR). S-3.13 @97148f90 LOCAL 0/3 (v1.13; duplicate rename IN-FLIGHT). S-5.02 @8eaff098 BLOCKED on human CLAUDE.md 60→64 commit. CLAUDE.md 60→64 human edit STILL PENDING. All prior D-1101..D-1175 notes SUPERSEDED.**

---

### ZERO-CONTEXT RESTART PROTOCOL (run in this order; no prior context needed)

A fresh session with NO prior context runs these steps in order before taking any action.

**Step 0.** Read this snapshot first. It is authoritative. Do NOT act on any other prior context.

**Step 1.** Run `vsdd-factory:factory-worktree-health` (devops-engineer). **BLOCKING** — do not proceed until it passes.

**Step 2.** Verify develop HEAD:
```bash
git log --oneline -1 origin/develop
```
Expected: `7241f5ef` (or newer if the human's CLAUDE.md edit landed or a lane merged).

**Step 3.** Verify each worktree HEAD against PINNED STATE below:
```bash
git -C /Users/jmagady/Dev/prism/.worktrees/S-5.02 rev-parse --short HEAD
git -C /Users/jmagady/Dev/prism/.worktrees/S-DEMO-LAUNCHER-CONSOLIDATION-001 rev-parse --short HEAD
git -C /Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-001 rev-parse --short HEAD
git -C /Users/jmagady/Dev/prism/.worktrees/S-3.13 rev-parse --short HEAD
```
Expected values: see PINNED STATE table below. If any differ, the worktree has new commits since D-1176 — use live git as truth (PINNED STATE is the D-1176 baseline, not a lock).

**Step 4.** Check CLAUDE.md on develop for human CLAUDE.md edit status:
```bash
grep 'EXPECTED=6' /Users/jmagady/Dev/prism/CLAUDE.md
```
If output is `EXPECTED=60` (not 64), the human edit has NOT yet landed — Task L0 (S-5.02) remains BLOCKED. If output is `EXPECTED=64`, L0 is unblocked: dispatch devops-engineer to rebase feature/S-5.02 onto develop@7241f5ef, then LOCAL adversary re-pass.

**Step 5.** Apply lessons (a)–(z24) from `cycles/wave-5-e-demo-fidelity/lessons.md`. Lesson z24 (DRIFT-HOLLOW-FEATURE-INTEGRATION-001) is critical for all parallel lanes.

**Step 6.** Execute the TASK LEDGER below (ordered; L0 BLOCKED-ON-HUMAN first, then L1–L5 autonomously).

---

### PINNED STATE (concrete; from D-1176 burst)

| Artifact | Value | Notes |
|----------|-------|-------|
| develop HEAD | `7241f5ef` | PR #188 squash-merge 2026-06-15T05:48:52Z; S-DEMO-004 MERGED |
| factory-artifacts HEAD (D-1176) | run `git -C .factory log -1 --format='%h %s'` | Do not hard-code; git owns this |
| S-DEMO-004 worktree | REMOVED | feature/S-DEMO-004 DELETED — cleaned post-merge; PR #188 MERGED develop@7241f5ef |
| S-5.02 worktree HEAD | `8eaff098` | feature/S-5.02; non-exhaustive EXPECTED=64; BLOCKED on CLAUDE.md edit; rebase onto 7241f5ef NEEDED before unblock |
| S-3.13 worktree HEAD | `97148f90` | feature/S-3.13; story v1.13; duplicate/misnamed test RENAME IN-FLIGHT; LOCAL streak 0/3 |
| PIVOT-001 worktree HEAD | `e4d95d19` | feature/S-DEMO-ENRICHMENT-PIVOT-001; story v1.6; LOCAL strict streak 2/3 |
| LAUNCHER worktree HEAD | `d9098c1f` | feature/S-DEMO-LAUNCHER-CONSOLIDATION-001; story v2.5 (AC-004 /health→/dtu/health); LOCAL streak 0/3; re-pass IN-FLIGHT |
| All active worktrees | just-check-GREEN | All 4 remaining lanes pass `just check` |
| S-5.02 | strict streak 0/3; BLOCKED | BLOCKED on CLAUDE.md 60→64 human commit |
| S-3.13 | strict streak 0/3 | Duplicate test rename IN-FLIGHT; adversary re-pass NEXT after fix |
| PIVOT-001 | strict streak 2/3 | ONE more CLEAN(strict) → PR ready |
| LAUNCHER | strict streak 0/3 | AC-004 prose fix @d9098c1f; adversary re-pass IN-FLIGHT |
| CLAUDE.md on develop | EXPECTED=60 (stale) | Human edit to 64 STILL PENDING; gates L0/S-5.02 only |

---

### TASK LEDGER (durable; replaces orchestrator in-session task list)

Execute L0 first (human action); L1–L4 are autonomous and can run in parallel after Step 1–3 pass.

| ID | Status | Task | Agent | Notes |
|----|--------|------|-------|-------|
| **L0** | **BLOCKED-ON-HUMAN** | Human edits CLAUDE.md on develop: change §Conventions "60 types" → "64 types" and `ci.yml EXPECTED=60` → `EXPECTED=64`. Attribution comment: +StructuredErrorFields, CapabilityEntry, ResolutionStep, CapabilityStatus (S-5.02). Commit to develop. THEN: devops-engineer rebases feature/S-5.02 onto develop@7241f5ef → LOCAL adversary re-pass (expect CLEAN(strict) → streak 1/3). | human → devops-engineer → adversary | All S-5.02 code/gate already correct in worktree (EXPECTED=64, 64 violations detected); only blocker is the CLAUDE.md stale count on develop. |
| **L1** | READY — 2/3 | PIVOT-001 @e4d95d19: LOCAL adversary pass-3. ONE MORE CLEAN(strict) → LOCAL 3/3 CONVERGED → push feature branch → PR. Verify EC-001/EC-002 closed + boot-wiring OBS-1 adjudicated PIVOT-002/003 scope + E-INFUSE-007 present. | adversary | Story v1.6. Merge-coord: engine.rs + boot.rs shared with S-3.13 — land constructor-sig story first, rebase second. DO-NOT-REFLAG: E-INFUSE-007 IS present in error-taxonomy.md v1.81. |
| **L2** | IN-FLIGHT | LAUNCHER @d9098c1f: LOCAL adversary re-pass. Verify AC-004 /health→/dtu/health closure. Verify MED-A/MED-B still closed. Hollow-feature gate: start-multi wired. Toward strict 3-CLEAN. | adversary | Story v2.5 (AC-004 /health→/dtu/health fixed). |
| **L3** | IN-FLIGHT | S-3.13 @97148f90: FIRST implementer fixes duplicate/misnamed test (DRIFT-S313-DUPTEST-001 IN-FLIGHT), THEN LOCAL adversary re-pass. Verify S-5.03 re-scope (no over-claim), proxy test relabeled, explain-wrapper wired, RG count = 17. Toward strict 3-CLEAN. | implementer → adversary | Story v1.13 (ACs 7 / RG 17 after v1.12+v1.13 edits). Merge-coord: engine.rs + boot.rs with PIVOT-001. |
| **L4** | **DONE** | S-DEMO-004 PR #188: LOCAL 3/3 strict CONVERGED + PR-LEVEL 3/3 strict CONVERGED (passes 5/6/7 on frozen 89942715). CI 43/43 GREEN. POL-14: all 9 BCs already active — idempotent. MERGED develop@7241f5ef 2026-06-15T05:48:52Z. | — | **CLOSED. T10 DONE.** |
| **L5** | BLOCKED-ON-L1..L3 | Per lane, after each LOCAL strict 3-CLEAN: create PR + run PR-LEVEL cascade → merge. MERGE-COORD: S-3.13 + PIVOT-001 both touch prism-query/engine.rs + boot.rs (different zones) — land constructor-signature-changing story first, rebase the other onto develop@7241f5ef. S-5.03 depends_on S-3.13 (do S-3.13 first). | pr-manager → adversary | Convergence rule: 3 consecutive strict-CLEAN passes (BC-5.39.001 / D-779). Orchestrator-drives-cascade (pr-manager lacks Agent tool). |

**Convergence rule (all lanes):** every lane needs 3 CONSECUTIVE CLEAN(strict) passes. Any finding resets streak to 0/3. Orchestrator drives cascade (pr-manager lacks Agent tool).

**PATTERN NOTE for fresh session:** recent re-passes keep surfacing SMALL fix-introduced hygiene findings (body-version drift, line-pin decay, doc accuracy). Expect to verify-and-possibly-fix one or two tiny items per lane before a clean pass; this is normal strict-3-CLEAN tail behavior, not regression.

---

### DO-NOT-REFLAG / Adjudications Already Made

These items are CLOSED or DEFERRED-BY-HUMAN. A fresh session must NOT reopen them.

| Item | Ruling | Where Anchored |
|------|--------|---------------|
| S-3.13 AC-7 + AC-4-notification + Task6/7 | DEFERRED to S-5.03 (DRIFT-S313-S503-RESCOPING-001 EXECUTED D-1173). Do not raise as S-3.13 findings. | S-3.13 v1.13; S-5.03 v1.13 |
| PIVOT-001 boot.rs production wiring + hot_reload NullSource | PIVOT-002/003 + S-1.14-REDO scope. Do not raise as PIVOT-001 findings. | OBS-1 D-1173 adjudication |
| PIVOT-001 OBS-1 E-INFUSE-007 missing from error-taxonomy | DISMISSED — E-INFUSE-007 IS registered at error-taxonomy.md line 438 v1.81. DO-NOT-REFLAG. | D-1175 dismissal |
| DRIFT-PIVOT-PLUGINID-INFUSIONID-001 | Forward-concern tracked (D-1176). PluginInfusionSource::new uses spec.infusion_id but PluginRuntime keys by plugin_metadata.plugin_id. NOT a PIVOT-001 blocker — PIVOT-002/003 scope. | D-1176 drift items |
| S-DEMO-004 PR #188 — all closures | MERGED develop@7241f5ef. ALL findings closed. T10 DONE. DO-NOT-REFLAG any S-DEMO-004 finding. | PR #188 merged 2026-06-15 |
| DEFER-CLAUDEMD-NONEXHAUSTIVE-COUNT-001 (60→64) | Human-only; gates L0/S-5.02 only; no other lanes blocked | D-1173 / D-1175 |
| All T5 (S-DEMO-DTU-LIVE-SCENARIO-001-B) PR-LEVEL closures | FULLY CONVERGED (29 passes, merged PR #185). See §3 T5 STORY STATUS and cascade ledger below. | PR #185 merged develop@7fd35b77 |

---

### 4 ACTIVE LANES — Current SHAs + Phase + Next Action (D-1176 baseline)

> **IMPORTANT:** SHAs are the D-1176 baseline. Run RESTART PROTOCOL Step 3 to confirm actual HEAD before acting. If worktrees have advanced, live git is authoritative.

| Lane | Story | Worktree Path | Branch | HEAD (D-1176) | Phase / Streak | EXACT NEXT ACTION |
|------|-------|---------------|--------|---------------|----------------|-------------------|
| **T10 (DONE)** | S-DEMO-004 | REMOVED | `feature/S-DEMO-004 (DELETED)` | `89942715 (frozen merge-base)` | **MERGED PR #188 develop@7241f5ef 2026-06-15T05:48:52Z** | **CLOSED.** T10 DONE. |
| **Lane A** | S-5.02 | `/Users/jmagady/Dev/prism/.worktrees/S-5.02` | `feature/S-5.02` | `8eaff098` | LOCAL streak 0/3; BLOCKED on human CLAUDE.md commit | **L0 (human):** commit CLAUDE.md 60→64 → devops-engineer rebase feature/S-5.02 onto develop@7241f5ef → LOCAL adversary re-pass. Story v1.7. EXPECTED=64 in ci.yml. |
| **Lane B** | S-3.13 | `/Users/jmagady/Dev/prism/.worktrees/S-3.13` | `feature/S-3.13` | `97148f90` | LOCAL streak 0/3; duplicate test rename IN-FLIGHT | **L3:** implementer rename DRIFT-S313-DUPTEST-001 → then LOCAL adversary re-pass. Story v1.13 (ACs 7 / RG 17). Merge-coord: engine.rs + boot.rs with PIVOT-001. |
| **Lane C** | PIVOT-001 | `/Users/jmagady/Dev/prism/.worktrees/S-DEMO-ENRICHMENT-PIVOT-001` | `feature/S-DEMO-ENRICHMENT-PIVOT-001` | `e4d95d19` | LOCAL strict streak **2/3** | **L1:** LOCAL adversary pass-3 → ONE MORE CLEAN(strict) → PR. Story v1.6. Merge-coord: engine.rs + boot.rs with S-3.13. |
| **Lane D** | — | — | — | — | **CLOSED** (D-1168) | S-1.15 DROPPED. Permanently closed. |
| **Lane E** | LAUNCHER | `/Users/jmagady/Dev/prism/.worktrees/S-DEMO-LAUNCHER-CONSOLIDATION-001` | `feature/S-DEMO-LAUNCHER-CONSOLIDATION-001` | `d9098c1f` | LOCAL streak 0/3; re-pass IN-FLIGHT | **L2:** LOCAL adversary re-pass. Story v2.5. Verify AC-004 /health→/dtu/health + MED-A/MED-B still closed. Hollow-feature: start-multi wired. |

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
| S-5.03 | not-started v1.13 | S-5.02 MERGED + S-3.13 MERGED | Resources and Prompts; received AC-8/9/10 from S-3.13 re-scope; hard prereq of S-5.04 |
| S-5.04 | not-started | S-5.03 MERGED | Sensor Health Subsystem; 5 pts |
| PIVOT-002 | not-started | PIVOT-001 MERGED | Enrichment chain; parallel with S-1.14-REDO possible |
| S-1.14-REDO | not-started | PIVOT-001 MERGED | Forward-subset of PIVOT chain; serialized infusion trio |
| PIVOT-003 | not-started | PIVOT-002 + S-1.14-REDO MERGED | Final enrichment story; closes TD-PLUGIN-P0-002 P0 |
| T13 narrative capstone | not-authored | S-DEMO-004+LAUNCHER+S-5.02/03/04+S-3.13+PIVOT-001/002/003 | PO+story-writer; SOC-analyst workflow story |
| T14 demo recording | not-started | T13 MERGED | demo-recorder |

**Merge-coordination note (MERGE-COORD):** S-3.13 (Lane B) and PIVOT-001 (Lane C) both touch `prism-query/engine.rs` in different zones (table-availability plan-time vs enrich UDF registration). Sequence: land constructor-signature-changing story first, rebase second. S-5.03 depends_on S-3.13.

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

### INDEX VERSIONS (as of D-1176 snapshot)

| Artifact | Version | Notes |
|----------|---------|-------|
| STATE.md | v7.819 | This snapshot (D-1176 cascade-round close; S-DEMO-004 MERGED) |
| BC-INDEX | v6.58 | active 235 / draft 2 / retired 6; total 250 |
| STORY-INDEX | v2.393 | 200 stories; S-DEMO-004 → merged; S-3.13 → v1.13; LAUNCHER → v2.5 |
| error-taxonomy | v1.81 | E-INFUSE-007 (PIVOT-001 HIGH-1 UDF-registration failure); E-QUERY-037 boxed emitter + strsim |
| ARCH-INDEX | v2.133 | — |
| VP-INDEX | v1.79 | 158 registered |
| prd | v1.12 | — |
| policies | v1.33 | POL-33 route_coverage_table_required_for_stagemask_changes |
| prismql-grammar | v1.1 | enrich function-call form |
| develop HEAD | 7241f5ef | PR #188 squash-merge 2026-06-15T05:48:52Z; T10 DONE |
| Open PRs | NONE | PR #188 MERGED (T10 DONE). All prior PRs merged. |

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
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) — T1–T10+T4-A DONE. NEXT: T11 (LAUNCHER) + parallel lanes (PIVOT-001 2/3, S-3.13 fix-in-flight, S-5.02 BLOCKED). |
| **develop HEAD** | `7241f5ef` (PR #188 squash-merge 2026-06-15T05:48:52Z; D-1176 T10 DONE) |
| **STATE version** | v7.819 |
| **BC-INDEX version** | v6.58 (total 250; active 235; draft 2; retired 6; BC-2.06.017 v1.10 active; BC-2.06.018 v1.6 active; BC-2.06.019 v1.7 active; BC-2.06.020 v1.6 active) |
| **STORY-INDEX version** | v2.393 (total_stories 200) |
| **VP-INDEX version** | v1.79 (158 registered) |
| **ARCH-INDEX version** | v2.133 |
| **error-taxonomy version** | v1.81 (E-INFUSE-007 PIVOT-001 HIGH-1 UDF-registration failure; E-QUERY-037 boxed emitter + strsim) |
| **ADR-036 version** | v2.3 (time_anchor 5-arg ruling) |
| **policies version** | v1.33 (POL-33 route_coverage_table_required_for_stagemask_changes) |
| **prd version** | v1.12 |
| **Open PRs** | **NONE.** PR #188 MERGED develop@7241f5ef (T10; D-1176 2026-06-15). PR #185 MERGED develop@7fd35b77 (T5). PR #186 MERGED develop@f7400f83 (D-1143). PR #187 MERGED develop@664566e9 (T6; D-1158). |
| **T10 branch** | `feature/S-DEMO-004`; MERGED at develop@7241f5ef (2026-06-15); worktree+branch cleaned |
| **factory-artifacts** | PUSHED to origin/factory-artifacts (D-1066; D-1176 burst) |

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

# 2. Verify develop HEAD == 7241f5ef
git log --oneline origin/develop | head -1

# 3. Verify STATE.md version
grep '^version:' /Users/jmagady/Dev/prism/.factory/STATE.md
# Expected: version: "7.819"

# 4. Confirm active worktrees (S-DEMO-004 worktree REMOVED post-merge)
ls /Users/jmagady/Dev/prism/.worktrees/
# Expected: S-3.09 + W3-FIX-S307-001 (parked) + S-5.02 + S-3.13 + S-DEMO-ENRICHMENT-PIVOT-001 + S-DEMO-LAUNCHER-CONSOLIDATION-001

# 5. Confirm factory-artifacts pushed (expect D-1176 burst commit at HEAD)
git -C /Users/jmagady/Dev/prism/.factory log -1 --format='%h %s'

# 6. T10 DONE (D-1176 2026-06-15; PR #188 develop@7241f5ef; all 9 BCs idempotent)
# NEXT: T11 LAUNCHER @d9098c1f LOCAL re-pass + parallel lanes (PIVOT-001 L1 2/3; S-3.13 L3 fix-in-flight)
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
