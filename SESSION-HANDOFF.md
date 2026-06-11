---
document_type: session-handoff
level: ops
version: "7.749"
status: current
timestamp: 2026-06-11T22:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **PRIORITY READ ORDER — D-1098 ROUNDS 9-11 RECORDED; QRY LOCAL CONVERGED; DELIVERY PHASE BEGINS (next session resumes ZERO-CONTEXT from this file). ZERO-CONTEXT RESUME SNAPSHOT.**
> Read §ACTIVE OBJECTIVE (North Star) FIRST, then §RESUME SNAPSHOT 2026-06-11-REVIEW-CYCLE-PAUSE-D1092 below (§BRANCH STATE updated by D-1098), then STATE.md frontmatter, then the task ledger (`.factory/objectives/multi-client-soc-demo-tasks.md`) before dispatching any agent.
> develop HEAD `c287b00d` (UNCHANGED — no review-cycle branch has merged). factory-artifacts PUSHED to origin/factory-artifacts (D-1066 standing authorization). STATE v7.749.

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
| 3 — **NEXT after review cycle** | **S-DEMO-DTU-LIVE-SCENARIO-001-B** | **draft v1.0** (T5 CURRENT — interrupted by 2026-06-10 review cycle before story-writer dispatch) | 7 | BC-2.06.019 + BC-2.06.020 (both draft) | S-DEMO-DTU-LIVE-SCENARIO-001-A (SATISFIED) | Story-writer materializes full impl spec from draft shell AFTER review cycle completes; remove-uncertainty before dispatch |
| 4 | **S-DEMO-004** | **registered** (STORY-INDEX row v2.342; T8 needs architect+PO: depends_on edge + AC-006 data-distinctness via real seeding; then story-writer + remove-uncertainty) | TBD | TBD (needs PO authorship) | S-DEMO-MULTI-TENANT-DTU-001 + data layer (001-A/B) | T8 architect+PO produce the formal story file |
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | **draft stub** (D-1029; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human launcher-lifecycle decision needed) | 0 stub (TBD) | -- | S-DEMO-003 (SATISFIED) | T11 story-writer materialization → T12 delivery |
| 6 — capstone | **Multi-client SOC-analyst narrative story** (not yet named or authored) | **not-authored** (no story file, no STORY-INDEX row; owner: product-owner + story-writer; after data layer + tooling exist) | TBD | TBD | Orders 3+4+5 complete | T13 → T14 demo recording; the demo's capstone deliverable |
| optional | **S-5.02** | not-started (wave 5) | 3 | 2 proxy | S-5.01 | MCP client targeting — capability discovery if narrative needs "show client's available sensors" |
| optional | **S-3.13** | not-started (wave 3) | 3 | 3 proxy | S-3.02, S-1.12 | Dynamic per-org table availability |
| optional | **S-5.04** | not-started (wave 5; depends_on updated S-5.04-FIX-001) | 5 | -- | S-5.03, S-DEMO-001 | Sensor health subsystem |

**NEXT CONCRETE ACTION: QRY pr-manager delivery (IN PROGRESS; QRY LOCAL CONVERGED head b3df3b16 35c) → after QRY merges: MCP pr-manager delivery (b0099308 CONVERGED) → DTU continues LOCAL cascade NEXT pass 19 (streak 0/3 head cd1c157b 25c) → DTU converges → DTU pr-manager delivery → PINNED merges QRY→MCP→DTU → register burst → RESUME T5 (Story B). See §BRANCH STATE for per-branch NEXT ACTIONS.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: T5 (interrupted by review cycle; D-1091..D-1098). T1+T2+T3+T4+T4-A DONE. ADR-036 v2.2. BC-INDEX v6.25. BC-3.4.003 v1.1. ARCH-INDEX v2.133. STORY-INDEX v2.348. error-taxonomy v1.76. policies v1.32. prd v1.12. BC-2.06.018 v1.6 ACTIVE. STATE v7.749.**

---

## §RESUME SNAPSHOT 2026-06-11-REVIEW-CYCLE-PAUSE-D1092

> **START HERE.** This snapshot is self-contained. A fresh session with ZERO prior context can resume exactly here.
> _Previous snapshot (2026-06-10-REVIEW-CYCLE-CHECKPOINT-D1091; STATE v7.742) archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`. The D-1090 Story B NEXT-ACTION content remains valid and is preserved in §RESUME T5 below._
> _D-1092 pause checkpoint: the session ended after this burst. All three cascade branch heads in §BRANCH STATE are EXACT at checkpoint (no agents left in flight) — still verify `git log` on resume as standard discipline._

---

### FRESH-SESSION RESUME PROTOCOL (zero prior context)

1. Run `vsdd-factory:factory-worktree-health` (devops-engineer) — **BLOCKING**; do not read state until it passes.
2. Read §GOAL below (sequence to completion), then STATE.md frontmatter (`current_step`, D-1092 decision row).
3. **Verify branch state with `git log`.** For each of the three fix worktrees under `.worktrees/`, run `git -C .worktrees/<name> log --oneline -3` and compare against §BRANCH STATE below. The checkpoint heads (cf0dfe1e / 36e3fc7b / 7c1c2a5e) are EXACT at the D-1092 pause (session ended cleanly; no agents in flight).
4. Confirm develop HEAD `c287b00d` unchanged (`git log --oneline develop | head -1`) — no review branch has merged.
5. `gh pr list --state open` → expect exactly ONE: draft PR #182 (fix/review-2026-06-10-dtu-fleet — parked with custody note from the contained unauthorized-push incident; do NOT close, do NOT mark ready; pr-manager assumes it at DTU convergence; DTU merges LAST).
6. Apply the 7 review-cycle process-gap lessons (`cycles/wave-5-e-demo-fidelity/lessons.md` D-1091 entry, items a–g) — especially (e) push-guards on cascade worktrees, (f) no orphaned background gates, (g) exclusive worktree ownership + commit-early.
7. Execute the §GOAL sequence below: the immediate next actions are the three adversary passes listed per-branch in §BRANCH STATE (query-core pass 6; mcp-boot pass 6; dtu-fleet pass 8 — each streak 0/3).

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) — INTERRUPTED at T5 by user-directed full-codebase review (2026-06-10); PAUSED at D-1092 (session end 2026-06-10/11) |
| **develop HEAD** | `c287b00d` (unchanged; no review branch merged yet) |
| **STATE version** | v7.749 |
| **BC-INDEX version** | v6.25 (total 250; active 232; draft 5; retired 6; BC-3.4.003 v1.1 human-authorized per-clone recovery) |
| **STORY-INDEX version** | v2.348 (total_stories 194) |
| **VP-INDEX version** | v1.77 |
| **ARCH-INDEX version** | v2.133 |
| **error-taxonomy version** | v1.76 |
| **policies version** | v1.32 |
| **prd version** | v1.12 |
| **test-vectors version** | v2.9 |
| **Open PRs** | 1 — draft PR #182 (parked; custody note; see §BRANCH STATE branch 2) |
| **factory-artifacts** | PUSHED to origin/factory-artifacts (D-1066; D-1091/D-1092 LOCAL exception expired at D-1093) |

---

### 2. §GOAL (unchanged — do not override)

Active objective: **multi-client SOC-analyst live demo**. Interrupted at T5 by a user-directed full-codebase review (2026-06-10). Sequence to completion:

1. Finish the 3 fix-branch BC-5.39.001 LOCAL cascades to **3-CLEAN strict each** (see §BRANCH STATE).
2. pr-manager 9-step delivery per branch, **PINNED merge order: QRY → MCP → DTU**.
3. State-manager **register burst** (§REGISTER BURST CHECKLIST below).
4. **RESUME T5**: Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) materialize + remove-uncertainty + 12-gate delivery per task ledger CURRENT POINTER T5 / D-1090 autonomy envelope (§RESUME T5 below).

E2E sequence after register burst: **T5 Story B → T6 S-DEMO-MULTI-TENANT-DTU-001 → T8 S-DEMO-004.**

---

### 3. §REVIEW-CYCLE ORIGIN

User-approved **14-item adjudication package (2026-06-10)** from an **8-lane full-codebase review**:
- **2 CRIT:** DTU-01 (Cyberint source field), QRY-01 (E-QUERY taxonomy swap)
- **2 HIGH:** QRY-02 (dead cache), SNS-01 (dead write path)
- **CrowdStrike parity pass** added CS-01..04 CRITs, incl. demo-critical CS-04.

Human decisions recorded:
- **ADR-037 ACCEPTED** — customer-config retired (`specs/architecture/decisions/ADR-037-prism-customer-config-crate-retirement.md`; BC-3.3.001/002/003/004 retired in BC-INDEX v6.21).
- **ADR-038 ACCEPTED v1.1→v1.4** — E-CFG/E-QUERY namespace reconciliation (`specs/architecture/decisions/ADR-038-e-cfg-runtime-namespace-reconciliation.md`).
- **P1-03** — BC-2.07.003 **UNAMENDED**; hot-reload flush interim; spec compliance via story **S-CACHE-SPEC-COMPLIANCE-001** post-demo; `cache.rs` carries human-authorized deviation citation.

---

### 4. §BRANCH STATE (updated D-1098 — verify `git log` on resume as standard discipline)

All three worktrees under `.worktrees/`, branches over `develop@c287b00d`, NOT pushed except DTU. **MCP is CONVERGED** — no further adversary passes; awaits pr-manager delivery after QRY merge. **QRY LOCAL CONVERGED** — head b3df3b16 (35c; CLAUDE.md carry-forward docs-only commit rides PR; code surface converged at f721fb21); pr-manager delivery IN PROGRESS. DTU head cd1c157b (25c; P18-01 MED closed).

**Cascade protocol (all 3 branches):** BC-5.39.001 3-CLEAN **strict** per branch; report format CLEAN(strict)/CLEAN(PR-merge); fresh-context adversary each pass. Adjudicated **do-not-reflag lists** are recorded in each pass dispatch — key standing exclusions: P1-03 deviation (human-authorized), BC frontmatter timestamps (register burst), DEF-1 Claroty pagination (post-merge fix), denylist wiring (S-WATCHDOG-WIRING-001), scope params (S-QUERY-SCOPE-PARAMS-001), cross-branch artifacts (merge order resolves).

**1. `fix/review-2026-06-10-query-core`** — 35 commits, head `b3df3b16` (CLAUDE.md carry-forward docs-only; code surface converged at `f721fb21`). Gate exit 0. **LOCAL CASCADE CONVERGED 3/3** (16 passes total). **pr-manager delivery IN PROGRESS** (push → PR → PR-LEVEL 3-CLEAN strict + pr-reviewer APPROVE + security CLEAR + CI green → squash-merge to develop per pinned order QRY first).
- **E-QUERY-034 split LANDED** (commits `388be2cf` / `0c240683` / `cf0dfe1e`: prism-core variant, prism-query sweep, prism-mcp arm). Pass-6: P6-01 display stutter (`77b81345`); P6-02 E-QUERY-006/E-ALIAS-001 nesting in E-QUERY-034 (`da3b7f31`: NEW `PrismError::UnknownSourceTable { source_name }` / E-QUERY-036 at `resolve_source_refs`; EXISTING `PrismError::AliasNotFound` un-nesting at `explain.rs`; 959/959 green); P6-03 [process-gap] E-QUERY-009 LATENT confirmed, DEFER-EQUERY009-001 stands. Pass-7: P7-01 LOW stale comment — `3c7679f8`. Pass-8: P8-01 LOW Internal.detail convention — taxonomy v1.74 adjudication + `76bda665` (959/959); P8-02 OBS DEFER-EQUERY009-001 re-verification. Pass-9: P9-01 HIGH v1.74 sweep false — **closed commit `873b693f`** (`WriteToolOutcome` typed discriminant; 960/960). Taxonomy v1.74→v1.75. Pass-10: CLEAN(strict)=YES; streak 1/3. Pass-11: P11-01 CRIT VP-058 grace period proven-but-UNWIRED — **closed commit `0c553426`** (`consecutive_over_limit` param; 85/85 + 960/960). Streak reset 0/3. Pass-12: P12-01 HIGH WatchdogCheckState docs retired PER-QUERY model — **closed commit `c779ff09`** (docs process-RSS; phantom feature claim corrected; 85/85). Streak 0/3. Pass-13: F-P13-01 LOW E-WATCH-002 code collision — **closed commit `f721fb21`** (1-line comment fix `error_mapping.rs`; taxonomy v1.75→v1.76). Pass-14: CLEAN(strict)=YES; streak 1/3 (cross-branch: QueryLimitExceeded E-QUERY-007→033 flip present on MCP branch error.rs:616 — ADR-038 v1.2 cross-branch assignment holds). Pass-15: CLEAN(strict)=YES; streak 2/3. **Pass-16: CLEAN(strict)=YES; streak 3/3 — CONVERGED** (final pass: proven-but-unwired + sibling-divergence lenses + from-scratch E2E flow re-derivation; zero findings of ANY severity). CLAUDE.md carry-forward commit `b3df3b16` added (docs-only; rides PR per §ALSO NOTE resolution).
- **NEXT ACTION: pr-manager 9-step delivery for QRY** (QRY FIRST in pinned order QRY→MCP→DTU). Push branch → create PR → PR-LEVEL 3-CLEAN strict (fresh-context adversary) + pr-reviewer APPROVE + security CLEAR + CI green → squash-merge to develop. Do NOT pr-manage MCP until QRY develop merge is confirmed.
- Do-not-reflag additions since D-1097: passes 14/15/16 all CLEAN(strict)=yes — no new do-not-reflag entries needed. CLAUDE.md carry-forward (b3df3b16) is docs-only; PR-LEVEL adversary covers it.
- Full do-not-reflag cumulative: E-QUERY-034 spec set (taxonomy v1.71/72/73/74/75/76, BC-2.11.006 v1.19, BC-2.11.005 v1.7); POL-16 hot-reload test retired; P4-D1/D2 deferred; P6-01 stutter closure; P6-02 E-QUERY-036; P6-03 + P8-02 DEFER-EQUERY009-001; P7-01 historical-comment; P8-01 closures (taxonomy v1.74 INT/Internal.detail/E-CACHE-001 scope); P9-01 `WriteToolOutcome` discriminant closure (taxonomy v1.75); P11-01 grace-period wiring closure + list_capabilities no-op pre-existing; P12-01 WatchdogCheckState docs closure + stale feature claim fixed; F-P13-01 E-WATCH-002/003 ruling closure; E-WATCH-003 forward-defined zero-emitter row; passes 14/15/16 CLEAN(strict) — no additional do-not-reflag entries.

**2. `fix/review-2026-06-10-dtu-fleet`** — 25 commits, head `cd1c157b`, gate exit 0. Streak 0/3; NEXT pass 19.
- Pass-8: F-P8-01 HIGH (COMP-012 deps + doc-nuance) — `332c99bd`; candidate (c) OBS/harmless; register-burst item 19 ADJUDICATED. Pass-9: CLEAN(strict)=YES; streak 1/3. Pass-10: P10-01 OBS armis search key-presence discriminator NO-ACTION adjudication; P10-02 OBS ScenarioConfig deserialized-not-consumed = Story-A NIT-2 anchored to Story B; do-not-reflag list tightened. CLEAN(strict)=no; streak reset 0/3. Pass-11: CLEAN(strict)=YES; streak 1/3. Pass-12: P12-01 HIGH demo-critical — Armis NULL `os_name`/`risk_score` — **closed commit `0959e92f`** (deterministic RNG-free pools; null-intolerant tests; 310/310). CLEAN(strict)=no; streak reset 0/3. Pass-13: CLEAN(strict)=YES; streak 1/3. Pass-14: P14-01 HIGH demo-critical — BC-3.4.003 invariant 6 (`auth_outage` `recovery_after_calls` override) missing in Claroty+Armis (TD-VSDD-059 paper-fix class). **Closed commit `dee1f2a0`**: override-read ported to Claroty+Armis; EC-3.4.003-06 tests load-bearing; Claroty doc corrected; gate exit 0. PO ruling A (D-1096): invariant 6 generator-fixture-layer-only; Cyberint `AuthMode::Reject` route-layer mechanism = architecturally-equivalent-but-distinct — **exempt** from `GenOpts::overrides`. CLEAN(strict)=no; streak reset 0/3. Pass-15: P15-01 CRIT REFUTED by orchestrator ground-truth check (adversary read main-checkout/develop tree instead of worktree — lesson p; false positive, no streak effect); P15-02 MED CONFIRMED — BC-3.4.003 v1.0→v1.1 (human-authorized per-clone recovery representations; TV-04b/04c added); zero code/test changes. CLEAN(strict)=no; streak 0/3. Pass-16: CLEAN(strict)=YES; streak 1/3. Pass-17: CLEAN(strict)=YES; streak 2/3. **Pass-18: CLEAN(strict)=no — P18-01 MED:** cyberint `post_reset` doc falsely claimed token re-registration (contradicted by `reset_all` implementation + AC-8 tests; harness-sibling behavior wrongly described onto standalone route handler). **Closed commit `cd1c157b`** (doc corrected; 12-crate fleet sibling-sweep table — all other post_reset/reset_all handler docs verified CLEAN). DTU streak reset → 0/3; head cd1c157b (25c).
- **NEXT ACTION: launch adversary pass 19 (streak 0/3)** with corrected ground truth: 5 crates 3-way gated (common + claroty/armis/crowdstrike/cyberint); 7 two-way; COMP-012 in all 9 COMP-DTU arrays; Armis `os_name`/`risk_score`/`manufacturer` deterministic non-null; BC-3.4.003 v1.1 (per-clone recovery representations — crowdstrike explicit-200, claroty absence-of-call-record, armis absence-of-injection; ALL CORRECT — do NOT reflag); Cyberint invariant-6 exempt (route-layer `AuthMode::Reject` = architecturally distinct); P18-01 cyberint post_reset doc closure (cd1c157b — do NOT reflag). ALL code reads must be under the worktree absolute path (lesson p); verify `git -C <worktree> log -1` matches stated head cd1c157b before citing line numbers.
- **PUSHED to origin + DRAFT PR #182 exists** — artifact of a contained unauthorized-push incident (lesson (e)). PR is **parked draft with custody note**. pr-manager assumes it at convergence. **DTU merges LAST.**
- Do-not-reflag additions since D-1097: P18-01 closure (`cd1c157b` — cyberint post_reset doc corrected; 12-crate sibling-sweep all CLEAN; do NOT reflag post_reset/reset_all doc content as defect).
- Full do-not-reflag cumulative: F-P8-01 closure (module-decomposition v1.19); candidates (b) doc-nuance; candidate (c) threatintel/nvd fixture-gen OBS/harmless (register-burst item 19 ADJUDICATED); P10-01 NO-ACTION adjudication (human ratification requested at gate); P10-02 NIT-2 Story B anchor; P12-01 Armis null-value closure (`0959e92f`); cyberint/crowdstrike incidents-route gaps pre-existing/out-of-scope (register-burst item 4); P14-01 override-porting closure (`dee1f2a0`); BC-3.4.003 v1.0 ruling-A Cyberint exemption; P15-01 refutation (false CRIT worktree-path error; implementation confirmed correct); P15-02 closure (BC-3.4.003 v1.1 human-authorized per-clone recovery); passes 16/17 CLEAN; P18-01 closure (cd1c157b; cyberint post_reset doc corrected; 12-crate sibling-sweep CLEAN).

**3. `fix/review-2026-06-10-mcp-boot`** — **CONVERGED 3/3** at head `b0099308` (33 commits). No further adversary passes. Gate exit 0.
- Pass history: P5-02 fail-closed write audit (`ab2ab0ce`); P5-01 re-anchor (`5863dbc7`); P5-03 capability fields (`7c1c2a5e`). Pass-6: P6-01 OBS dead computation in `audit_emitter.rs` — commit `b4707e95`. Pass-7: CLEAN(strict)=YES; streak 1/3. Pass-8: P8-OBS-01 [process-gap] SAP-1 probe vs BC-2.16.002 crate scope — PO-half CLOSED (BC-2.16.002 v1.76); CLAUDE.md SAP-1 wording half PENDING HUMAN; streak reset 0/3. Pass-9: CLEAN(strict)=YES; streak 1/3. Pass-10: F-P10-01/02/03 LOW (stale docstring + tautological timeout test + Jira/PD/Slack network-mode parity); closed commits `487122e3` + `b0099308`; architect Decision B: LOGICAL-MODE-ONLY BY DESIGN; streak reset 0/3. Pass-11: CLEAN(strict)=YES; streak 2/3. **Pass-12: CLEAN(strict)=YES; streak 3/3 — CONVERGED.** Pass-13 (final verification pass): CLEAN(strict)=YES — zero findings of ANY severity; proven-but-unwired + sibling-divergence lenses applied.
- **NEXT ACTION: pr-manager delivery — 9-step PR cycle AFTER QRY fix-branch converges and merges.** Pinned merge order: QRY → MCP → DTU. PR #182 (parked draft) is the DTU PR; MCP needs a new PR. Do NOT pr-manage MCP until QRY develop merge is confirmed.
- Full do-not-reflag cumulative: two-class audit contract adjudicated; capability-fields carrier = tracing emission per dispatch; P6-01 dead-computation closure (`b4707e95`); P8-OBS-01 PO-half closed (BC-2.16.002 v1.76); F-P10-01/02/03 closures (`487122e3` + `b0099308`; Decision B logical-mode-only); SAP-1 CLAUDE.md wording PENDING HUMAN (adversary may note as process-gap; action requires human; not a code defect).

---

### 5. §REGISTER BURST CHECKLIST (state-manager — runs AFTER all 3 merges, BEFORE T5)

1. Close 32 validated TDs with PR evidence + TD-A-004 (ratification memo in `proposals/TD-S-PLUGIN-PREREQ-A-004-close-as-superseded-ratification.md`) + TD-VSDD-094.
2. Retire ghost TD-W2-SENSORS-FULL-001; re-file TD-ADR005-001 vs current security surface; file TD-S307-001 (E-QUERY-024/027/028); re-home TD-VSDD-019; relocate TD-VSDD-082/083/084 + TD-S305-001; surface per-file TDs (S305 family, S302-005) in main register.
3. NEW TD: cache normalized-vs-raw deviation, human-authorized 2026-06-10, anchor S-CACHE-SPEC-COMPLIANCE-001.
4. File CS-05 (CrowdStrike TOML column coverage incl. `containment_status`) as product-decision story candidate. **EXTENDED D-1095:** also file `cyberint incidents` + `crowdstrike incidents` TOML tables have NO DTU route — same family as CS-05 (pre-existing out-of-scope gaps surfaced during P12-01 sibling sweep; each has a TOML `[[tables]]` declaration with NO corresponding DTU route handler; requires product decision on DTU route scope per table).
5. Priority adjustments: S304-AUDIT-001→P2; WV15 set down (PR35-001/002→P3, PR36-001/PR40-001→P4); MED-003 merge into ULID-001; MUTATE-005 P2/P3 reconcile.
6. STATE.md count refresh (workspace_test_count, develop_head post-merges, version pins to CURRENT values at register-burst time — as of D-1092: taxonomy v1.72, BC-INDEX v6.23, STORY-INDEX v2.348, ARCH-INDEX v2.132, policies v1.32, prd v1.12) + STORY-INDEX frontmatter BC-count re-basis per BC-INDEX.
7. Register S-DEMO-004 follow-ups: n/a (row registered v2.342).
8. Anchor ~10 STATE drift promises (edition sync, tape paths, pagination validation, E-QUERY-009, orphan sensors dir, BC-3.5.002 cites).
9. Input-hash refresh sweep (corpus-wide; 313 stale reported by `compute-input-hash --scan specs` at D-1091 — burst-set artifacts already refreshed at D-1091).
10. POL-23 Direction-A remediation: normalize 4 deviant BC timestamps to creation dates (BC-2.07.003, BC-2.16.002, BC-2.06.018, BC-2.01.017) + fix 2 POL-27 modified-format violations (BC-2.16.002, BC-2.06.018) + POL-29 sweep.
11. POL-32 tombstone rows: BC-2.11.001 missing v1.4 row; BC-3.2.001 v0.9 row (reconstruct from BC-INDEX v5.79).
12. TD-S302-005 premise refresh (pipeline implemented; blocker = cold-start buffer routing per S-2.08 Rule 5).
13. Story-ID sweep: BC-2.11.001 v1.7 + BC-INDEX v6.19 generic citation → S-QUERY-SCOPE-PARAMS-001.
14. DEF-1: post-merge fix-pr for Claroty list_alerts/list_audit_logs ignoring POST-body offset/limit + Gap-CL-004 comment correction (**BEFORE demo**).
15. File taxonomy↔code symmetry-audit maintenance story (4 collisions found: E-CFG, E-QUERY-007 tombstone, E-QUERY-003 triple, E-QUERY-011 two-BC; first work items: E-QUERY-006 embedded-detail + E-QUERY-010 zero-emitter, per ADR-038 adjacent-findings).
16. S-MAINT-VERIFY-PIPELINE-001 scope doc exists in `proposals/S-MAINT-VERIFY-PIPELINE-001-scope.md` — story-writer materializes (absorbs TD-CICD-001 + KANI-001 + FUZZ-002/003).
17. ~~BC-INDEX row 220 stale "v1.70" cite for BC-2.16.002 → sync to current (v1.75).~~ **RESOLVED D-1094** — row-220 inline-pin synced to v1.76 (PO committed v1.76 directly; BC-INDEX v6.23→v6.24 records the POL-11 bump). _(added D-1092; resolved D-1094)_
18. Lessons addenda [process-gap] to `cycles/wave-5-e-demo-fidelity/lessons.md`: (h) architect micro used heredoc appends for 2 changelog sections — TD-FACTORY-HOOK-BYPASS-001 mechanism violation, self-reported, content correct; (i) MCP implementer committed BC-2.16.002 v1.75 to factory-artifacts directly (`8eacb6d1`) — state-manager ownership deviation, single-commit clean; (j) P6/P7 churn pattern — spec fix-bursts introduced new errors (wrong story range, unswept YAML cells, false verified-claims) caught only by subsequent passes; codify the P7 dispatch's verification discipline (verify IDs against indexes, sweep structured fields with prose, verify counts on disk, never write unexecuted "verified" claims) into fix-burst templates; **(k) SAP-1 wording/scope reconciliation [process-gap] — SAP-1 in CLAUDE.md does not clearly scope the event_type catalog probe to BC-2.16.002's declared crate (prism-spec-engine); capability_check and credential_access are correctly handled by BC-2.04.013 and BC-2.03.010 respectively; PENDING HUMAN CLAUDE.md edit to add crate-scope clarifier sentence; (l) do-not-reflag list completeness discipline — NIT-2 (ScenarioConfig deserialized-not-consumed) omission from DTU do-not-reflag list caused avoidable P10-02 reflag + DTU streak reset 1/3→0/3; story-B anchor was known from D-1090 Story B handoff; explicit NIT-N tracking in do-not-reflag lists is required going forward; (m) load-bearing string sentinels make naive convention sweeps regression traps — when a string literal is used both for display AND as a typed discriminant (e.g. `"E-INT-001:"` as `WriteToolOutcome` routing key in `invalidation.rs:512`), dropping the prefix-convention alone is insufficient; the sentinel must first be replaced by a typed construct (`WriteToolOutcome` enum variant) before the string site can be made prose-only; sweep regexes that only target the prefix pattern miss this class entirely (P9-01, lesson-j instance #2 this cascade); (n) key-presence/null-tolerant parity assertions are paper-fix enablers — parity tests that assert only that a key exists (`is_some()`) or allow null values (`is_null()||`) permit a generator to silently produce NULL on the seeded/live-demo path while the test passes; column-coverage tests MUST assert that values are non-null AND semantically populated (e.g., `os_name` is a non-empty string from the expected pool); SAP-2 step-2 should be extended to require non-null value assertions for all TOML-declared columns when testing the seeded path (P12-01; upstream SAP-2 extension candidate).** _(added D-1092; (k)+(l) added D-1094; (m)+(n) added D-1095)_
19. ~~threatintel/nvd unused fixture-gen feature-forwarding (observation, harmless) — adjudicate or note.~~ **ADJUDICATED D-1093:** OBS/harmless — unused `fixture-gen` feature forwarding in threatintel/nvd DTU crates is a cosmetic no-op; no emitters, no runtime effect. No action. _(added D-1092; adjudicated D-1093)_

**Lessons addenda (p)(q) — D-1097:**
(p) [process-gap] **adversary worktree-path read error → false CRIT P15-01** (pass 15 DTU). Adversary cited line numbers from the pre-fix `main`/`develop` checkout layout instead of the dispatched worktree `fix/review-2026-06-10-dtu-fleet`. Resulted in a false CRIT claiming a production-critical fix was a paper-fix. Near-miss: an orchestrator ground-truth check (`rg recovery_after_calls` directly in worktree) caught the error before a spurious fix-burst was dispatched. **Standing discipline codified:** (1) adversary dispatch instructions MUST include: "ALL code reads, `grep`/`rg` searches, and line-number citations MUST use the worktree absolute path (e.g. `/Users/jmagady/Dev/prism/.worktrees/<branch>/`) — do NOT read from the main checkout at `/Users/jmagady/Dev/prism/crates/`"; (2) instruct: "Before citing any line numbers, run `git -C <worktree-path> log -1 --format='%H %s'` and confirm the output matches the stated head SHA"; (3) **orchestrator discipline:** for any CRIT finding on a worktree branch, the orchestrator MUST run a ground-truth check (direct `rg`/`grep` in the worktree) before dispatching a fix-burst. Dispatching a fix-burst on a CRIT without a ground-truth verification is now explicitly prohibited.

(q) **Source-of-Truth rule-7 spec-amendment-to-match-code escalation correctly executed** (pass 15 DTU, P15-02 MED). The orchestrator surfaced "code already models correct per-clone behavior; spec is divergent" → ran AskUserQuestion → user selected "Authorize v1.1" → PO amended BC-3.4.003 v1.0→v1.1 under human authorization. First instance of §7 escalation in this cascade. Pattern validated: the §7 escalation gate is functioning correctly; no process change needed. The correct behaviour is: (1) adversary or orchestrator identifies code-vs-spec mismatch where code is provably correct by architecture; (2) orchestrator presents both options (amend spec / fix code) to human; (3) human authorizes; (4) PO amends spec; (5) zero code/test changes required when the code is already correct. This is distinct from the defer-pattern anti-pattern (which would have filed a TD and deferred).

**Lessons:** 7 review-cycle process-gap lessons (a)–(g) recorded at `cycles/wave-5-e-demo-fidelity/lessons.md` (D-1091 entry): CS-04 realistic-query blind spot; gate-script false-pass / exit-code-only checking; retirement sweeps stopping one boundary short (5+); parallel-burst race (stories anchor AFTER BC-family amendments settle); unauthorized push (constraints must bind re-invocations + cascade push-guards); orphaned-gate pattern (4×; foreground gates / commit-early); rogue git-reset edit-war (exclusive worktree ownership + commit-early).

---

### 6. §RESUME T5 (after register burst — preserved Story B NEXT ACTION from D-1090)

**T5 = S-DEMO-DTU-LIVE-SCENARIO-001-B materialize + deliver, full-autonomous per D-1090 envelope (run all gates A→merge autonomously; PAUSE only §7/product-business/Level-3/CLAUDE.md).**

(1) Dispatch `vsdd-factory:story-writer` to MATERIALIZE the full implementation spec for S-DEMO-DTU-LIVE-SCENARIO-001-B from its draft v1.0 shell (`.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`), against BC-2.06.019 (scenario progression) + BC-2.06.020 (enrichment correlation) + ADR-036 v2.2 (Deterministic Scenario Progression Engine).

**CONTRACT-COMPLETENESS FRONT-LOAD (Story-A P6 lesson — do this BEFORE locking the story spec):** verify the following mechanism is FULLY specified in BC-2.06.019/020 + ADR-036 before marking the story ready:
- Progression mechanism: deterministic-over-time timeline (same seed + clock-offset → same timeline; NOT random append)
- Stage masks: recon → lateral-movement → exfil → containment (all 4 stages enumerated with criteria)
- Enrichment correlation: IOCs the progression introduces must resolve in ThreatIntel; CVEs on affected devices must resolve in NVD
If any design gap exists, story-writer surfaces to orchestrator → architect/PO BEFORE locking. Do NOT proceed to TDD with an underspecified contract.

Fold in the 2 Story-A NIT follow-ups during materialization:
- NIT-1: E-DEMO-004 error message references `scenario.enabled` but Story A fires it on non-default fixture_set archetype + missing `org_id` — reconcile message/trigger when Story B wires `scenario.enabled` (BC-2.06.019).
- NIT-2: `ScenarioConfig` fields (`enabled`/`archetype`/`scenario_start_secs`/`stage_duration_secs`) deserialized-but-unconsumed in Story A — Story B consumes them (BC-2.06.019).

(2) Run `dclaude:remove-uncertainty` on the materialized spec (standing directive D-1061).

(3) Deliver via the canonical 12-gate per-story TDD sequence (worktree → test-writer Red Gate → implementer TDD green → LOCAL adversary 3-CLEAN strict → demo-recorder per-AC → push with `timeout: 600000`/background → pr-manager → PR-LEVEL 3-CLEAN strict + pr-reviewer APPROVE + security CLEAR (push-before-regate per DRIFT-ORCH-PRLEVEL-PUSH-001) → CI green → squash-merge → state-manager post-merge burst (POL-14: BC-2.06.019 + BC-2.06.020 draft→active)).

T6 (S-DEMO-MULTI-TENANT-DTU-001 ready v1.2; BC-2.06.017; remove-uncertainty COMPLETE D-1076) is independently deliverable after T5 starts.

---

### 7. §ALSO NOTE (durability items outside .factory/)

- **CLAUDE.md carry-forward RESOLVED (D-1098):** Main-checkout uncommitted CLAUDE.md review-cycle edits (6 doc fixes incl. EXPECTED=50, 19 CFs, `new_unchecked` mechanism, registry-gate concurrency framing, 26-crate header) copied onto QRY branch as commit `b3df3b16` (docs-only). Will land on develop when QRY PR squash-merges. Main-tree copy `CLAUDE.md` still shows as modified (`git status` `M CLAUDE.md`) — **leave it in place; reconcile at QRY merge** (the merge-squash will include this content; do not reset the main-tree copy or you lose it).
- prism memory file updated (19 CFs) — already durable.
- Carry-forward fix candidates (also pre-loaded as dtu-fleet P8 candidates, §BRANCH STATE branch 2): module-decomposition COMP-DTU `dependencies` arrays omit prism-core (architect); 3 lib.rs doc comments in claroty/cyberint/armis say 2-way while attribute is 3-way (implementer, DTU branch).
- E2E sequence after register burst: **T5 Story B → T6 S-DEMO-MULTI-TENANT-DTU-001 → T8 S-DEMO-004.**

> **This snapshot is current as of D-1098 (2026-06-11 rounds 9-11 recorded; QRY LOCAL CONVERGED; DELIVERY PHASE BEGINS; STATE v7.749). §BRANCH STATE updated D-1098. Prior snapshots (D-1090, D-1091) archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`.**

---

### 8. Standing Authorizations and Rules

**D-1090 STORY B (T5) AUTONOMY GRANT — ACTIVE (user-authorized 2026-06-10; resumes after review cycle completes)**
- Full autonomous Story B materialize + deliver (same envelope as Story A / D-989)
- Story-writer materializes spec → remove-uncertainty → 12-gate TDD → auto-merge to develop ONLY when objective gates met: LOCAL 3-CLEAN strict + PR-LEVEL 3-CLEAN strict + security MAY PROCEED + pr-reviewer APPROVE + all CI PASS
- **PAUSE-AND-SURFACE for 4 hard exceptions (do NOT auto-handle):**
  1. Source-of-Truth §7 spec-to-match-code amendments (only human authorizes)
  2. Genuine product/business decision not derivable from existing specs/ADRs
  3. Level-3 escalation: missing prerequisite, genuinely-red CI, convergence not reached after reasonable retries
  4. CLAUDE.md edits (human-only per Pipeline Authority)

**D-989 AUTONOMY GRANT — ACTIVE (granted 2026-06-04, extended to Story B by D-1090)**
- Full autonomous Wave-5 A→B→C execution
- Auto-advance phases + auto-merge to develop ONLY when objective gates met: LOCAL 3-CLEAN strict + PR-LEVEL 3-CLEAN strict + security MAY PROCEED + pr-reviewer APPROVE + all CI PASS
- Same 4 hard exceptions as D-1090 above

**Standing rules NEVER waived:**
- No `--no-verify` (lefthook hooks)
- No force-push to main/develop without explicit human authorization
- **factory-artifacts is PUSHED to origin/factory-artifacts after each state burst (off-machine durability; user-authorized 2026-06-08, D-1066).** The D-1091/D-1092 LOCAL exception (review-cycle custody caution) EXPIRED at D-1093 — factory-artifacts PUSHED at D-1093 (normal push, not force).
- Single-commit-per-burst (TD-VSDD-053) — no Stage-2/backfill chains
- BC-5.39.001 3-CLEAN strict (per D-779 disambiguation): streak advances ONLY on CLEAN(strict)=zero findings of ANY severity
- Fix-in-scope — no defer-pattern for AI-found AI-generated defects
- TD-VSDD-091 — no volatile line-number pins in .factory/ narrative; use function anchors
- **remove-uncertainty-per-story:** run `dclaude:remove-uncertainty` on EVERY implementation story before TDD delivery (user standing directive 2026-06-08, D-1061). Applies to all remaining Phase C stories and future waves.
- **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001):** after ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix means the adversary reviews stale code. Verify `git rev-parse origin/feature/<branch>` == local worktree HEAD before re-gating. (D-1065, codified 2026-06-08; evidence: PR #179 PR-LEVEL SEC-001 fix committed locally but unpushed — adversary reviewed stale remote.) DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001: this rule should also be mirrored into CLAUDE.md §Standing rules — **HUMAN-ONLY CLAUDE.md edit** (Pipeline Authority); non-blocking.
- **Review-cycle worktree discipline (D-1091, lessons (e)/(f)/(g)):** cascade worktrees carry push-guards until convergence (no push, no PR creation without orchestrator authorization); constraints bind re-invocations of the same agent; exclusive worktree ownership per active agent; commit-early; no turn ends with uncommitted work pending a background gate.

---

### 9. Parked Worktrees and Review Worktrees

| Worktree | Status | Action |
|----------|--------|--------|
| `.worktrees/S-3.09` | FROZEN | Leave alone |
| `.worktrees/W3-FIX-S307-001` | BLOCKED/superseded | Leave alone |
| `.worktrees/` review fix worktrees ×3 (dtu-fleet / query-core / mcp-boot) | ACTIVE — in-flight cascades (§BRANCH STATE) | Verify git log on resume; exclusive ownership; push-guard until convergence |
| All wave-5 story worktrees | CLEANED | Removed at merge |

---

### 10. Open Follow-Ups and Drift Items

> Many open drift/TD items are consolidated into the §REGISTER BURST CHECKLIST above (runs after the 3 review merges). Items below remain individually tracked.

**CLAUDE.md edit needed (HUMAN ONLY — non-blocking):**
- DEFER-CLAUDEMD-BC216002-MISLABEL-001: SAP-1 probe cites BC-2.16.002 as "Structured Event Catalog" but that BC is "Multi-Step Fetch Pipeline"; catalog lives in BC-2.05.005/BC-2.03.010. Human-mandated CLAUDE.md edit required.
- NOTE: the uncommitted develop-tree CLAUDE.md edits (§ALSO NOTE) already carry 6 review-cycle doc fixes — reconcile this bucket when that commit lands.

**Active open drift items (non-blocking unless noted):**
- DRIFT-D954-001: BC-3.5.002 precondition 3 mis-cite in prism-dtu-armis (~40+) + prism-dtu-slack (1) — S-MAINT-W3SEC-CITE-SWEEP-002 anchored; story-writer materialization needed.
- DRIFT-D1016-SEC-007: QueryParams.start_time/end_time as Option<String>; TimestampString newtype candidate — architect/PO adjudication.
- DEFER-EQUERY009-001: BC-2.11.007 DI-021 E-QUERY-009 enforcement absent from live path — register-burst item 8 anchors this.
- S-DEMO-LAUNCHER-CONSOLIDATION-001: draft stub; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human review of script-lifecycle question needed.

**Pre-existing maintenance stories (wave-independent):**
- S-MAINT-W3SEC-CITE-SWEEP-002 (armis+slack cite sweep)
- S-MAINT-ORPHAN-SENSORS-DIR-001 (top-level sensors/*.toml cleanup)
- S-MAINT-EDITION-SYNC-001 (workspace edition 2024 migration)
- S-POL-14-STATUS-SYNC-001 (BC promotion + story-status sync; maintenance wave)
- S-MAINT-VERIFY-PIPELINE-001 (scope doc in proposals/; register-burst item 16)

**New review-cycle stories (authored 2026-06-10; registered at STORY-INDEX v2.346):**
- S-CACHE-SPEC-COMPLIANCE-001 (P1-03 anchor; post-demo)
- S-QUERY-SCOPE-PARAMS-001 (scope params; cascade standing exclusion)
- S-WATCHDOG-WIRING-001 (denylist wiring; cascade standing exclusion)
- S-WATCHDOG-CONFIG-PROFILE-001 (graduated watchdog config profiles)

---

### 11. Resume Protocol Checklist

Run these commands at start of a fresh session to verify state:

```bash
# 0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star) + §RESUME SNAPSHOT D-1092
# The review cycle must complete (cascades → QRY→MCP→DTU merges → register burst) BEFORE T5 resumes.

# 1. Factory worktree health (BLOCKING preflight)
# Use: vsdd-factory:factory-worktree-health skill

# 2. Verify develop HEAD == c287b00d (unchanged — no review branch merged)
git log --oneline develop | head -1

# 3. Verify STATE.md version
grep '^version:' .factory/STATE.md
# Expected: version: "7.749"

# 4. Verify open PRs — expect exactly ONE parked draft
gh pr list --state open
# Expected: #182 draft (fix/review-2026-06-10-dtu-fleet) — PARKED with custody note. Do not close/ready it.

# 5. Verify the three review fix worktrees (D-1097 heads)
git -C .worktrees/review-2026-06-10-dtu-fleet log --oneline -1 2>/dev/null || ls .worktrees/
# query-core == b3df3b16 (35 commits; CONVERGED; pr-manager IN PROGRESS) | dtu-fleet == cd1c157b (25 commits; NEXT p19 streak 0/3) | mcp-boot == b0099308 (33 commits CONVERGED; awaits QRY merge)
# (worktree directory names may differ — enumerate .worktrees/ and match branches fix/review-2026-06-10-*)

# 6. Confirm factory-artifacts local vs remote — should be PUSHED (D-1097 normal push per D-1066)
git -C .factory log -1 --format='%h %s'
git -C .factory rev-parse origin/factory-artifacts 2>/dev/null || echo "no remote yet"

# 7. Confirm main-tree CLAUDE.md uncommitted edits still present (§ALSO NOTE — do NOT lose)
git -C . status --porcelain CLAUDE.md
# Expected: " M CLAUDE.md"

# 8. Task ledger pointer (resumes AFTER register burst)
grep -A5 'CURRENT POINTER' .factory/objectives/multi-client-soc-demo-tasks.md
# Expected: CURRENT POINTER = T5 (resumes after review cycle + register burst)
```

---

### 12. Where Extracted History Lives

| Content | Archive Location |
|---------|-----------------|
| Per-story cascade pass tracking (STATE.md YAML frontmatter keys for 25+ stories) | `cycles/wave-5-e-demo-fidelity/frontmatter-cascade-archive.md` |
| Decision rows D-700..D-1054 | `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md` |
| Superseded SESSION-HANDOFF resume snapshots (incl. D-1082, D-1090, D-1091) | `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md` |
| Burst narratives (D-735..D-1084) | `cycles/wave-5-e-demo-fidelity/burst-log.md` |
| Lessons learned (incl. D-1091 review-cycle lessons a–g) | `cycles/wave-5-e-demo-fidelity/lessons.md` |
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

10. **factory-artifacts PUSH-AFTER-EACH-BURST (user-authorized D-1066, 2026-06-08).** The state-manager PUSHES factory-artifacts to origin/factory-artifacts as the FINAL step of every state burst (off-machine durability). This supersedes the prior LOCAL-ONLY default. Push is `git -C .factory push origin factory-artifacts` (normal push, NOT force-push, NOT to main/develop). If the remote branch does not yet exist, first push with `-u` flag. DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 tracks the corresponding CLAUDE.md §Git Workflow mirror (human-only edit). D-1091/D-1092 EXCEPTION: the 2026-06-10 review-cycle checkpoint burst and the 2026-06-10/11 pause-checkpoint burst stay LOCAL per explicit user direction; push resumes on a subsequent state burst.

11. **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001, D-1065).** After ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix-commit causes the adversary to review stale code. Verify `git rev-parse origin/feature/<branch>` == local worktree HEAD before re-gating.

12. **Review-cycle pinned merge order (D-1091).** The three 2026-06-10 review fix branches merge in PINNED order QRY → MCP → DTU (cross-branch artifact conflicts resolve in this order; DTU last because PR #182 custody + DTU cascade depends on sibling merges).

---
