---
document_type: session-handoff
level: ops
version: "7.772"
status: current
timestamp: 2026-06-13T05:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-1123 BURST (2026-06-13) — pass 17 CLEAN(strict)=YES; CLEAN(PR-merge)=YES. Streak 2/3. Full behavioral trace (5 stages x 6 clones) + cross-BC + wiring + SAP-1 + S-7.01 zero findings; novelty LOW. Code UNCHANGED 7ddc0a51. PASS 18 (convergence pass) NEXT. If clean → 3/3 → pr-reviewer re-run + security re-run on 7ddc0a51 → CI → merge → POL-14. STATE v7.772.**
>
> **PRIORITY READ ORDER:** Read §ACTIVE OBJECTIVE (North Star) FIRST, then §RESUME SNAPSHOT D-1123 below, then STATE.md frontmatter. D-1101/D-1102/D-1103/D-1108/D-1109/D-1110/D-1111/D-1112/D-1113/D-1114/D-1115/D-1116/D-1117/D-1118/D-1119/D-1120/D-1121/D-1122 notes SUPERSEDED — D-1109/D-1110 snapshot archived to cycles/wave-5-e-demo-fidelity/session-handoff-archive.md.
> develop HEAD `939f36ce` (unchanged). Story B HEAD `7ddc0a51` = remote = PR #185 latest push (D-1118: 2 commits; D-1119/D-1120/D-1121/D-1122/D-1123: NO new commits). factory-artifacts PUSHED to origin/factory-artifacts (D-1066 standing authorization). STATE v7.772.

---

## §ACTIVE OBJECTIVE — Multi-Client SOC-Analyst Live Demo (NORTH STAR)

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
| 1 — parallel/independent | **S-DEMO-MULTI-TENANT-DTU-001** | **ready v1.2** (T1+T2+T3 DONE D-1076; remove-uncertainty 8 closed; S-7.01 CLEARED) | 8 | BC-2.06.017 (draft) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | READY FOR TDD DELIVERY at T6 — deliverable independent of Story A/B; deliver after T4-A+T5 complete |
| 2 — DONE | **S-DEMO-DTU-LIVE-SCENARIO-001-A** | **MERGED** (T4-A; PR #181 develop@c287b00d 2026-06-10; BC-2.06.018 v1.6 active) | 8 | BC-2.06.018 (active) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | T4-A DONE — Story B unblocked |
| 3 — **CURRENT** | **S-DEMO-DTU-LIVE-SCENARIO-001-B** | **PR-LEVEL cascade** (T5 CURRENT — PR #185 OPEN; HEAD=REMOTE=7ddc0a51 CODE UNCHANGED; LOCAL 3/3 strict; PR-LEVEL streak 1/3, pass 17 NEXT; D-1122) | 7 | BC-2.06.019 v1.7 + BC-2.06.020 v1.4 (both draft) | S-DEMO-DTU-LIVE-SCENARIO-001-A (SATISFIED) | After 3-CLEAN strict → pr-reviewer APPROVE → security → CI → squash-merge → post-merge burst (POL-14; CLAUDE.md 50→52 DONE in-PR D-1108) |
| 4 | **S-DEMO-004** | **registered** (STORY-INDEX row v2.342; T8 needs architect+PO: depends_on edge + AC-006 data-distinctness via real seeding; then story-writer + remove-uncertainty) | TBD | TBD (needs PO authorship) | S-DEMO-MULTI-TENANT-DTU-001 + data layer (001-A/B) | T8 architect+PO produce the formal story file |
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | **draft stub** (D-1029; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human launcher-lifecycle decision needed) | 0 stub (TBD) | -- | S-DEMO-003 (SATISFIED) | T11 story-writer materialization → T12 delivery |
| 6 — capstone | **Multi-client SOC-analyst narrative story** (not yet named or authored) | **not-authored** (no story file, no STORY-INDEX row; owner: product-owner + story-writer; after data layer + tooling exist) | TBD | TBD | Orders 3+4+5 complete | T13 → T14 demo recording; the demo's capstone deliverable |
| **D-1107 SCOPE-IN** | **S-5.02** | not-started (wave 5) | 3 | 2 proxy | S-5.01 | Tool Routing/Errors/Client Scoping — **OPTED IN (D-1107 2026-06-12)**; remove-uncertainty before TDD (D-1061) |
| **D-1107 SCOPE-IN** | **S-5.03** | not-started (wave 5) | -- | -- | S-5.02 | Resources and Prompts — hard dep of S-5.04; **OPTED IN (D-1107)**; remove-uncertainty before TDD |
| **D-1107 SCOPE-IN** | **S-5.04** | not-started (wave 5) | 5 | -- | S-5.03, S-DEMO-001 | Sensor Health Subsystem — **OPTED IN (D-1107)**; remove-uncertainty before TDD |
| **D-1107 SCOPE-IN** | **S-3.13** | not-started (wave 3) | 3 | 3 proxy (needs PO BCs) | S-3.02, S-1.12 | Dynamic Table Availability — **OPTED IN (D-1107)**; parallel after PO authors dedicated BCs; remove-uncertainty before TDD |

**NEXT CONCRETE ACTION: T5 — PR-LEVEL pass 18 (convergence pass) for PR #185 (S-DEMO-DTU-LIVE-SCENARIO-001-B) at HEAD 7ddc0a51 (diff UNCHANGED — reuse /tmp/pr185-pass13.diff or re-materialize via `gh pr diff 185`; NO CI push needed since D-1119/D-1120/D-1121/D-1122/D-1123 had no code commits). Streak 2/3 (pass 17 CLEAN(strict)=YES D-1123). If pass 18 CLEAN(strict)=YES → 3/3 PR-LEVEL CONVERGENCE → post-convergence sequence (see §RESUME SNAPSHOT D-1123 §4). D-1090 autonomy grant still active.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: T5 (PR-LEVEL cascade; PR #185 OPEN; streak 2/3; pass 18 (convergence) NEXT — D-1123). T1+T2+T3+T4+T4-A DONE. ADR-036 v2.3. BC-INDEX v6.40. BC-3.4.003 v1.1. BC-3.6.001 v0.8. BC-3.5.002 v0.5. ARCH-INDEX v2.133. STORY-INDEX v2.366. error-taxonomy v1.78. VP-INDEX v1.79 (158). policies v1.33. prd v1.12. BC-2.06.018 v1.6 ACTIVE. BC-2.06.019 v1.7. BC-2.06.020 v1.4. STATE v7.772.**

---

## §RESUME SNAPSHOT — D-1123 (2026-06-13 — pass 17 CLEAN(strict)=YES; streak 2/3; pass 18 (convergence) next; STATE v7.772)

> **D-1123: T5 PR-LEVEL cascade in progress. S-DEMO-DTU-LIVE-SCENARIO-001-B in PR-LEVEL cascade. PR #185 OPEN; HEAD=REMOTE=7ddc0a51 (CODE UNCHANGED — no new commits in D-1119, D-1120, D-1121, D-1122, or D-1123). Pass 17 CLEAN(strict)=YES; CLEAN(PR-merge)=YES — ZERO findings. Full holdout-style behavioral trace (5 attack stages x 6 clones: Armis/Claroty/CrowdStrike/Cyberint/NVD/ThreatIntel) + cross-BC consistency (BC-2.06.019 v1.7 ↔ BC-2.06.020 v1.4 shared catalog/seed) + build_clone_pairs wiring (guard order E-DEMO-002→006→003→004, all 6 constructors) + POL-12 stub-residue PASS + SAP-1 PASS + S-7.01 SEC-001 sibling-drift PASS. Novelty LOW. Streak 1/3→2/3. STATE v7.772. NEXT = PR-LEVEL pass 18 (convergence pass) at HEAD 7ddc0a51 (diff UNCHANGED — reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; NO CI push needed since no code changed). If CLEAN(strict)=YES → 3/3 PR-LEVEL CONVERGENCE → post-convergence sequence below.**
> _D-1101/D-1102/D-1103/D-1108/D-1109/D-1110/D-1111/D-1112/D-1113/D-1114/D-1115/D-1116/D-1117/D-1118/D-1119/D-1120/D-1121/D-1122 notes SUPERSEDED — D-1109/D-1110 snapshot archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`._

---

### FRESH-SESSION RESUME PROTOCOL (zero prior context)

1. Run `vsdd-factory:factory-worktree-health` (devops-engineer) — **BLOCKING**; do not read state until it passes.
2. Read §ACTIVE OBJECTIVE (North Star) FIRST. Then read STATE.md frontmatter (`current_step`, D-1122 decision row).
3. Verify develop HEAD: `git log --oneline origin/develop | head -1` → expect `939f36ce`.
4. Verify story B: `git -C .worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H'` → expect `7ddc0a51` (= remote; last pushed D-1118; D-1119/D-1120/D-1121/D-1122 had NO code commits).
5. `gh pr checks 185` — confirm CI status on 7ddc0a51.
6. Verify STATE version: `grep "^version:" .factory/STATE.md` → expect `"7.772"`.
7. Worktree status: `.worktrees/S-3.09` (FROZEN — leave alone) + `.worktrees/W3-FIX-S307-001` (BLOCKED/superseded — leave alone). Story B worktree: `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B` (ACTIVE — PR-LEVEL cascade in progress).
8. Apply lessons (a)–(z12) from `cycles/wave-5-e-demo-fidelity/lessons.md`.
9. **NEXT ACTION:** PR-LEVEL pass 18 (convergence pass) at HEAD 7ddc0a51 (streak 2/3; pass 17 CLEAN D-1123; diff UNCHANGED — reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; do-not-reflag: stale doc-comment in bc_2_06_020_cyberint_nvd_pivot.rs = cosmetic; BPRL-P14-01 + BPRL-P15-01 CLOSED; story line-47 "~16 tests" tilde-qualified estimate = below OBS threshold — DO NOT raise; see §4 below). If CLEAN(strict)=YES → 3/3 convergence → post-convergence sequence in §4.

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) — T5 PR-LEVEL cascade (PR #185 OPEN, streak 2/3; pass 18 (convergence) NEXT) |
| **develop HEAD** | `939f36ce` (DTU PR #182 squash-merged 2026-06-12T05:18Z; unchanged since D-1103) |
| **STATE version** | v7.772 |
| **BC-INDEX version** | v6.40 (total 250; active 232; draft 5; retired 6; BC-2.06.019 v1.7; BC-2.06.020 v1.4; rows 119/120 story pin v2.13) |
| **STORY-INDEX version** | v2.366 (total_stories 200) |
| **VP-INDEX version** | v1.79 (158 registered) |
| **ARCH-INDEX version** | v2.133 |
| **error-taxonomy version** | v1.78 (E-DEMO-006 new) |
| **ADR-036 version** | v2.3 (time_anchor 5-arg ruling) |
| **policies version** | v1.33 (POL-33 route_coverage_table_required_for_stagemask_changes) |
| **prd version** | v1.12 |
| **Open PRs** | 1 — PR #185 (S-DEMO-DTU-LIVE-SCENARIO-001-B) OPEN; PR-LEVEL cascade pass 18 (convergence) NEXT; streak 2/3 (pass 17 CLEAN D-1123; code unchanged 7ddc0a51) |
| **Story B branch** | `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; HEAD = REMOTE = `7ddc0a51` (CODE UNCHANGED since D-1118) |
| **factory-artifacts** | PUSHED to origin/factory-artifacts (D-1066; D-1120 burst) |

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
9. **CURRENT: T5 PR-LEVEL pass 18 (convergence pass)** — streak 2/3; diff UNCHANGED at 7ddc0a51 (reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; NO CI push needed); do-not-reflag list in §4 below. If CLEAN(strict)=YES → 3/3 PR-LEVEL CONVERGENCE.
9. After 3-CLEAN strict → pr-reviewer re-run APPROVE on 7ddc0a51 (MUST re-run — code changed via D-1117 since pass-11 reviews on bc0f36c5) → security-reviewer re-run MAY PROCEED on 7ddc0a51 (MUST re-run same reason) → CI green → squash-merge → post-merge burst (POL-14: BC-2.06.019 + BC-2.06.020 draft→active; CLAUDE.md EXPECTED 50→52 DONE in-PR D-1108 — no post-merge edit needed).
10. T6 → T8 → capability-discovery block (D-1107) → S-DEMO-ENRICHMENT-PIVOT-001/002/003 chain → T11 → T13 capstone.

---

### 3. §T5 STORY STATUS

**S-DEMO-DTU-LIVE-SCENARIO-001-B** — scenario progression + enrichment correlation live demo.

| Field | Value |
|-------|-------|
| **Story version** | v2.13 (19 ACs / 23 Red Gate tests; D-1121 BPRL-P15-01 SPEC-ONLY: Phase-6 gate instruction "19 RGTs"→"23 RGTs"; CODE UNCHANGED 7ddc0a51) |
| **BC-2.06.019** | v1.7 (D-1113 fabricated inventory-note prose corrected; D-1112 Claroty devices Route Coverage row + exhaustive inventory note; v1.5: D-1111 Route Coverage Table corrected + PC-4 5-arg prose; v1.4: D-1109 per-sensor IOC-surface matrix + Interim State clause + Route Coverage Table; PRE-6 org_id guard per PO OBS-1 ruling) |
| **BC-2.06.020** | v1.4 (D-1120 SPEC-ONLY: PC-9 implementer directive 0..100000→0..10000; v1.3: D-1117: PC-8 catalog-ID assignment for scenario Cyberint CVEs; PC-9 baseline non-pivotable CVE-9999-{:04}; INV-CYBERINT-ALERT-CVE-CORRELATION-001; VP-020-I..L; EC-020-012..015; TV-020-011..015; draft) |
| **ADR-036** | v2.3 (time_anchor 5-arg ruling) |
| **Demo evidence** | 19/19 ACs COMPLETE (commit f75f3159; VHS; docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/) |
| **LOCAL cascade** | CONVERGED 3/3 strict (13 passes at pre-D-1117 code; D-1117 adds 3 commits; D-1118 adds 2 more commits) |
| **PR-LEVEL streak** | 2/3 (pass 1 CLEAN; pass 2 BPRL-P2-01 MED closed 4eadb027; pass 3 BPRL-P3-01 MED + 2 OBS closed; pass 4 BPRL-P4-01 CLOSED-BY-DEFERRAL + BPRL-P4-02 closed + BPRL-P4-PG-01 closed; pass 5 BPRL-P5-01 HIGH closed BC-2.06.019 v1.5; pass 6 BPRL-P6-01 HIGH closed BC-2.06.019 v1.6; pass 7 BPRL-P7-01 MED closed BC-2.06.019 v1.7; pass 8 BPRL-P8-01 MED closed BC-INDEX row-120 sync; pass 9 CLEAN(strict)=YES streak 1/3 [INVALIDATED D-1117]; pass 10 CLEAN(strict)=YES streak 2/3 [INVALIDATED D-1117]; pass 11 CLEAN(strict)=YES streak 3/3 [INVALIDATED D-1117]; pass 12 BPRL-P12-01 MED closed D-1118 (VP-020-K false-green); pass 13 CLEAN(strict)=YES streak 1/3 D-1119; pass 14 BPRL-P14-01 HIGH SPEC-ONLY closed D-1120 (BC-2.06.020 v1.4 RNG range literal); pass 15 BPRL-P15-01 MED SPEC-ONLY closed D-1121 (story B Phase-6 gate "19 RGTs"→"23 RGTs"); pass 16 CLEAN(strict)=YES streak 1/3 D-1122 (exhaustive audit zero findings); pass 17 CLEAN(strict)=YES streak 2/3 D-1123 (behavioral trace 5 stages x 6 clones + cross-BC + wiring + SAP-1 + S-7.01; novelty LOW); **streak 2/3; pass 18 (convergence) NEXT at 7ddc0a51**) |
| **Branch HEAD** | 7ddc0a51 = remote |

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

---

### 4. §PR-LEVEL PASS 17 — DISPATCH INSTRUCTIONS

**Dispatch fresh adversary for PR-LEVEL pass 18 (convergence pass; streak 2/3; pass 17 CLEAN D-1123; diff UNCHANGED at 7ddc0a51 — reuse /tmp/pr185-pass13.diff or `gh pr diff 185`; NO CI push needed).**

**Ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `7ddc0a51`; PR #185
- Adversary reads PR diff via `gh pr diff 185` (diff UNCHANGED since pass 13 — reuse /tmp/pr185-pass13.diff or re-materialize)
- ALL code reads use `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B/` absolute path
- Verify `git -C .worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H %s'` matches `7ddc0a51` before citing any line numbers
- BC-2.06.019 is v1.7 — use the v1.7 Route Coverage Table (8 rows, exhaustive); do NOT cite v1.6 or earlier inventory-note prose
- BC-2.06.020 is v1.4 — use v1.4; includes PC-8, PC-9 (range `0..10000`), INV-CYBERINT-ALERT-CVE-CORRELATION-001, VP-020-I..L; do NOT cite v1.3 range `0..100000`
- BC-INDEX rows 119/120 anchor story pin is `ready v2.13 (D-1121 2026-06-13)` — do NOT cite v2.12 or earlier annotations
- Story B is v2.13 — Phase-6 gate instruction reads "all 23 Red Gate tests pass"; do NOT cite "19 Red Gate tests" in the gate instruction
- Streak is 2/3; this is pass 18 (convergence pass)

**Full do-not-reflag list for pass 17 (do NOT raise these as new findings):**

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
- **PASS-16 SUB-THRESHOLD DISPOSITION (D-1122) — DO NOT REFLAG:** Story line ~47 points-justification comment contains "Red Gate test suite (~16 tests, FAIL-first): 1 pt" — a tilde-qualified estimate in the FROZEN 7-point breakdown rationale. `points: 7` has never changed. Live RGT count (23) is consistent across: frontmatter `red_gate_tests: 23`, 23-row RGT table, Phase-6 gate instruction "all 23 Red Gate tests pass", STORY-INDEX. The `~16 tests` annotation is NOT a count-of-record surface; it is a tilde-qualified effort estimate in frozen authoring-time rationale, analogous to historical changelog prose. Pass-16 adversary adjudicated this below-OBS threshold. Anchored as opportunistic cleanup to S-DEMO-ENRICHMENT-PIVOT-003. **DO NOT raise as a finding in pass 17 or any subsequent pass.**

**Post-convergence sequence (after 3-CLEAN strict):**
1. pr-reviewer → APPROVE
2. security-reviewer → MAY PROCEED
3. CI green on final head
4. squash-merge to develop
5. Post-merge state-manager burst: POL-14 (BC-2.06.019 + BC-2.06.020 draft→active); STORY-INDEX status update; STATE bump. (CLAUDE.md EXPECTED 50→52 is DONE — merged in-PR per D-1108; no post-merge edit needed.)

---

### 5. §D-1107 CAPABILITY-DISCOVERY SCOPE-IN

**D-1107 USER DECISION (2026-06-12):** capability-discovery block opted INTO demo scope.

| Story | Status | Depends on | Notes |
|-------|--------|------------|-------|
| S-5.02 | not-started | S-5.01 | Tool Routing/Errors/Client Scoping |
| S-5.03 | not-started | S-5.02 | Resources and Prompts (hard dep of S-5.04) |
| S-5.04 | not-started | S-5.03, S-DEMO-001 | Sensor Health Subsystem |
| S-3.13 | not-started | S-3.02, S-1.12 | Dynamic Table Availability (parallel after PO authors BCs) |

**Updated build sequence:** T5 (CURRENT) → T6 (ready v1.2; remove-uncertainty DONE D-1076) → T8 (architect+PO reconcile first) → S-5.02 → S-5.03 → S-5.04 (+ S-3.13 parallel after PO BCs) → T11 (pending launcher-lifecycle decision) → T13 capstone.

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
| `.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B` | ACTIVE (PR-LEVEL cascade) | PR-LEVEL pass 18 (convergence) NEXT; streak 2/3 (pass 17 CLEAN D-1123; pass 16 CLEAN D-1122; pass 15 BPRL-P15-01 SPEC-ONLY D-1121); HEAD 7ddc0a51 (CODE UNCHANGED) |

---

### 8. §RESUME PROTOCOL COMMANDS

```bash
# 1. Factory worktree health (BLOCKING preflight)
# Use: vsdd-factory:factory-worktree-health skill

# 2. Verify develop HEAD == 939f36ce
git log --oneline origin/develop | head -1

# 3. Verify story B HEAD == 7ddc0a51
git -C /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H %s'

# 4. Check CI on PR #185
gh pr checks 185

# 5. Verify STATE.md version
grep '^version:' /Users/jmagady/Dev/prism/.factory/STATE.md
# Expected: version: "7.772"

# 6. Confirm parked worktrees
ls /Users/jmagady/Dev/prism/.worktrees/
# Expected: S-3.09 + W3-FIX-S307-001 + S-DEMO-DTU-LIVE-SCENARIO-001-B

# 7. Confirm factory-artifacts pushed
git -C /Users/jmagady/Dev/prism/.factory log -1 --format='%h %s'
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
