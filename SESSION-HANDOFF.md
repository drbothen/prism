---
document_type: session-handoff
level: ops
version: "7.741"
status: current
timestamp: 2026-06-10T06:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **PRIORITY READ ORDER — D-1090 DURABILITY HARDENING (zero-context Story B resume) + D-1089 STORY-A MERGED (PR #181 develop@c287b00d; BC-2.06.018 v1.6 active) + D-1082 COMPLETE STORY ROADMAP. ZERO-CONTEXT RESUME SNAPSHOT.**
> Read §ACTIVE OBJECTIVE (North Star) FIRST, then task ledger (`.factory/objectives/multi-client-soc-demo-tasks.md`), then STATE.md frontmatter + §RESUME SNAPSHOT before dispatching any agent.
> develop HEAD `c287b00d`. factory-artifacts LOCAL-ONLY (origin/factory-artifacts = 4d28cbc2; D-1090 burst not yet pushed — push on next state burst per D-1066). STATE v7.741.

---

## §ACTIVE OBJECTIVE — Multi-Client SOC-Analyst Live Demo (NORTH STAR)

> **READ THIS FIRST.** This section persists the current priority goal so fresh sessions never drift onto unrelated pipeline machinery (D-1072, user-directed 2026-06-09).

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
| 2 — **NEXT** | **S-DEMO-DTU-LIVE-SCENARIO-001-A** | **ready v1.1** (T4 DONE D-1079; remove-uncertainty CONFIRMED SOUND D-1080; ADR-036 v2.1) | 8 | BC-2.06.018 (draft) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 (SATISFIED) | **NEXT DELIVERY (T4-A)** — 12-gate TDD. Story B hard-depends on this merge |
| 3 | **S-DEMO-DTU-LIVE-SCENARIO-001-B** | **draft v1.0** (D-1079; blocked on Story A merge) | 7 | BC-2.06.019 + BC-2.06.020 (both draft) | S-DEMO-DTU-LIVE-SCENARIO-001-A (hard) | Story-writer materializes full impl spec from draft shell after A merges; remove-uncertainty before dispatch |
| 4 | **S-DEMO-004** | **draft / not-yet-authored in STORY-INDEX** (T8 not-started; needs architect+PO: add depends_on missing edge + AC-006 data-distinctness via real seeding; then story-writer + remove-uncertainty) | TBD | TBD (needs PO authorship) | S-DEMO-MULTI-TENANT-DTU-001 + data layer (001-A/B) | No STORY-INDEX row yet — T8 architect+PO produces the formal story file |
| 5 | **S-DEMO-LAUNCHER-CONSOLIDATION-001** | **draft stub** (D-1029; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human launcher-lifecycle decision needed) | 0 stub (TBD) | -- | S-DEMO-003 (SATISFIED) | T11 story-writer materialization → T12 delivery |
| 6 — capstone | **Multi-client SOC-analyst narrative story** (not yet named or authored) | **not-authored** (no story file, no STORY-INDEX row; owner: product-owner + story-writer; after data layer + tooling exist) | TBD | TBD | Orders 3+4+5 complete | T13 → T14 demo recording; the demo's capstone deliverable |
| optional | **S-5.02** | not-started (STORY-INDEX v2.332; wave 5) | 3 | 2 proxy | S-5.01 | MCP client targeting — capability discovery if narrative needs "show client's available sensors" |
| optional | **S-3.13** | not-started (STORY-INDEX v2.332; wave 3) | 3 | 3 proxy | S-3.02, S-1.12 | Dynamic per-org table availability |
| optional | **S-5.04** | not-started (STORY-INDEX v2.332; wave 5; depends_on updated S-5.04-FIX-001) | 5 | -- | S-5.03, S-DEMO-001 | Sensor health subsystem |

**NEXT CONCRETE ACTION (T5 — D-1090): Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) — story-writer materializes full impl spec → dclaude:remove-uncertainty → 12-gate delivery. See §RESUME SNAPSHOT 2026-06-10-STORY-B-DELIVERY-D1090 above for verbatim NEXT ACTION.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: T5 (D-1090). §Complete Story Roadmap = definitive per-story detail. T1+T2+T3+T4+T4-A DONE. ADR-036 v2.2. BC-INDEX v6.10. ARCH-INDEX v2.119. STORY-INDEX v2.338. error-taxonomy v1.64. BC-2.06.018 v1.6 ACTIVE. STATE v7.741.**

---

## §RESUME SNAPSHOT 2026-06-10-STORY-B-DELIVERY-D1090

> **START HERE.** This snapshot is self-contained. A fresh session with ZERO prior context can resume exactly here.
> _Previous snapshot (2026-06-09-COMPLETE-ROADMAP-D1082; STATE v7.733) archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`._

---

### FRESH-SESSION RESUME PROTOCOL (zero prior context)

0. **Read §ACTIVE OBJECTIVE (North Star) above** — the current priority is the multi-client SOC demo; the per-story VSDD pipeline SERVES this goal. Do NOT drift into unrelated single-story pipeline machinery. Then **read `.factory/objectives/multi-client-soc-demo-tasks.md`**, find CURRENT POINTER = T5, and execute its NEXT ACTION. The task ledger is the granular, status-tracked resume source-of-truth (D-1073).
1. Run `vsdd-factory:factory-worktree-health` (devops-engineer) — **BLOCKING**; do not read state until it passes.
2. Read STATE.md frontmatter + this §RESUME SNAPSHOT.
3. Verify `git log --oneline develop | head -1` shows `c287b00d` (develop_head). **NOTE:** local develop was fast-forwarded to c287b00d in the 2026-06-10 session (main checkout was stale at 64d34967; synced to c287b00d = PR #181 squash-merge). This is the CORRECT HEAD — no SHA drift. If a fresh checkout shows `64d34967`, fetch + fast-forward before dispatching.
4. Confirm no open PRs (`gh pr list`) — expect NONE. Story A merged PR #181; Story B not yet started.
5. Confirm parked worktrees (`.worktrees/S-3.09` FROZEN, `.worktrees/W3-FIX-S307-001` BLOCKED) are left alone.
6. **Apply the 3 operational lessons from `cycles/wave-5-e-demo-fidelity/lessons.md` (Story-A process-gaps) before dispatching any sub-agent:**
   - **(a) Adversary/reviewer worktree-path-guard:** any adversary or review agent dispatched on a feature worktree MUST receive the worktree's ABSOLUTE PATH in dispatch instructions AND must run a directory sanity-check guard as its FIRST act (`git -C <worktree_root> rev-parse HEAD`). Grep/Glob/Read tools do NOT inherit bash `cd` — dispatches without absolute paths will silently examine the wrong tree and produce false findings.
   - **(b) Feature-branch pushes run the ~14-min `just check` pre-push gate:** push commands for feature branches MUST be run with `run_in_background: true` OR `timeout: 600000` (600s). Plain unadorned `git push` in a normal agent turn will timeout before the lefthook gate completes.
   - **(c) Sentinel/count/value changes require EXHAUSTIVE per-route/per-doc inventory (TD-VSDD-060):** when a value, sentinel, or count must propagate across multiple routes or document surfaces, enumerate ALL surfaces first (read `impl Router::build_router()` and count ALL arms; grep for exact old-count string across ALL CI-relevant files) before patching any of them. Apply to ALL in a single commit. Targeted/incremental sweeps cause streak resets.
7. Execute NEXT ACTION below.

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) |
| **Wave-5 Phase B** | **COMPLETE** — all 4 lanes + S-MAINT merged |
| **Wave-5 Phase C** | **COMPLETE** — all 4 lanes merged (TRAILING-SLASH, SPEC-PROSE-FIX, PAGINATION-001, HARNESS-CLONE-PARITY-001) |
| **Story A (T4-A)** | **MERGED** — PR #181 squash-merged develop@c287b00d 2026-06-10; BC-2.06.018 v1.6 active |
| **develop HEAD** | `c287b00d` |
| **STATE version** | v7.741 |
| **BC-INDEX version** | v6.10 |
| **STORY-INDEX version** | v2.338 |
| **VP-INDEX version** | v1.76 |
| **ARCH-INDEX version** | v2.119 |
| **Active BCs** | 236 |
| **Draft BCs** | 5 (BC-2.06.011 + BC-2.06.017 + BC-2.06.019 + BC-2.06.020 + BC-2.21.001) |
| **Total stories** | 189 |
| **Open PRs** | NONE |
| **factory-artifacts** | LOCAL-ONLY this burst (D-1090); push to origin on next state burst per D-1066 policy |

---

### 2. What Just Completed

**D-1090 Zero-Context Resume Durability Hardening — Story B (T5) authorization + SESSION-HANDOFF refresh (user-directed, 2026-06-10)**

User directive 2026-06-10: proceed with full-autonomous materialize + deliver of Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B / T5). Autonomy envelope identical to Story A (D-989): run all gates A→merge autonomously; pause only for §7/product-business/Level-3/CLAUDE.md. Local develop branch fast-forwarded to c287b00d confirmed (main checkout was stale at 64d34967; now correct — no SHA drift). SESSION-HANDOFF snapshot refreshed from 2026-06-09-COMPLETE-ROADMAP-D1082 (STATE v7.733) to 2026-06-10-STORY-B-DELIVERY-D1090 (STATE v7.741). Task ledger v1.12→v1.13: NEXT ACTION augmented with contract-completeness front-loading step + Story-A NIT follow-ups + autonomy authorization. Operational lessons pointer added (3 Story-A process-gap lessons). D-1090. STATE v7.740→v7.741.

**D-1089 T4-A (Story A) MERGED — PR #181 squash-merged develop@c287b00d (2026-06-10)**

S-DEMO-DTU-LIVE-SCENARIO-001-A DONE. LOCAL 18-pass 3-CLEAN strict (BC-5.39.001 P16/17/18) + PR-LEVEL 3-pass 3-CLEAN strict (P1/2/3) + security CLEAR + pr-reviewer APPROVE + CI green. ADR-036 v2.2 full 8-archetype seeding. `fixture_gen_seeded` sentinel wired. Per-client distinct data proven (INV-DISTINCT-DATA-001). 21-pass total adversarial effort. POL-14: BC-2.06.018 v1.5→v1.6 draft→active. active_contracts 235→236, draft_contracts 6→5. BC-INDEX v6.10. STORY-INDEX v2.338. T5 UNBLOCKED.

---

### 3. Exact Next Steps — Multi-Client SOC Demo (North Star)

> **T4-A (Story A) is MERGED. CURRENT POINTER = T5 (Story B). The pipeline serves the ACTIVE OBJECTIVE: multi-client SOC-analyst live demo.**

**VERBATIM NEXT ACTION (D-1090 — T5: Story B materialize + deliver):**

(0) Run `vsdd-factory:factory-worktree-health` (BLOCKING) per resume protocol above before dispatching any agent.

(1) Dispatch `vsdd-factory:story-writer` to MATERIALIZE the full implementation spec for S-DEMO-DTU-LIVE-SCENARIO-001-B from its draft v1.0 shell (`.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`), against BC-2.06.019 (scenario progression) + BC-2.06.020 (enrichment correlation) + ADR-036 v2.2 (Deterministic Scenario Progression Engine).

**CONTRACT-COMPLETENESS FRONT-LOAD (Story-A P6 lesson — do this BEFORE locking the story spec):** The story-writer MUST verify the following mechanism is FULLY specified in BC-2.06.019/020 + ADR-036 before marking the story ready:
- Progression mechanism: deterministic-over-time timeline (same seed + clock-offset → same timeline; NOT random append)
- Stage masks: recon → lateral-movement → exfil → containment (all 4 stages enumerated with criteria)
- Enrichment correlation: IOCs the progression introduces must resolve in ThreatIntel; CVEs on affected devices must resolve in NVD
If any design gap exists in BC-2.06.019/020 or ADR-036, the story-writer surfaces it to the orchestrator to route to architect/PO BEFORE locking the story. Do NOT proceed to TDD with an underspecified contract.

Also fold in the 2 Story-A NIT follow-ups during materialization (non-blocking anchors — Story B wires these):
- NIT-1: E-DEMO-004 error message references `scenario.enabled` but Story A fires it on non-default fixture_set archetype + missing `org_id` — reconcile message/trigger when Story B wires `scenario.enabled` (BC-2.06.019).
- NIT-2: `ScenarioConfig` fields (`enabled`/`archetype`/`scenario_start_secs`/`stage_duration_secs`) deserialized-but-unconsumed in Story A — Story B consumes them (BC-2.06.019).

(2) Run `dclaude:remove-uncertainty` on the materialized Story B spec (standing directive D-1061 — ALWAYS before TDD dispatch).

(3) Deliver Story B via the 12-gate per-story TDD sequence:
`vsdd-factory:worktree-manage create S-DEMO-DTU-LIVE-SCENARIO-001-B` → `vsdd-factory:test-writer` (Red Gate tests FAIL-first) → `vsdd-factory:implementer` (TDD green) → LOCAL adversary 3-CLEAN strict (BC-5.39.001 D-779) → `vsdd-factory:demo-recorder` per-AC → push feature branch (with `timeout: 600000` or `run_in_background: true` — pre-push `just check` ~14 min cold) → `vsdd-factory:pr-manager` PR create → PR-LEVEL adversary 3-CLEAN strict + `vsdd-factory:pr-reviewer` APPROVE + `vsdd-factory:security-reviewer` CLEAR (push any fix commits before re-gating per DRIFT-ORCH-PRLEVEL-PUSH-001) → CI all green → squash-merge → state-manager post-merge burst (POL-14: BC-2.06.019 + BC-2.06.020 draft→active).

T6 (S-DEMO-MULTI-TENANT-DTU-001 ready v1.2; BC-2.06.017) is independently deliverable and may be parallelized after T5 starts (same 12-gate sequence; remove-uncertainty already COMPLETE from T3 D-1076).

**AUTONOMY:** Full-autonomous to merge. Pause only for §7 spec-to-match-code amendments / genuine product-business decisions / Level-3 escalation / CLAUDE.md edits. User-authorized D-1090 2026-06-10.

| Priority | Story / Action | Status | Notes |
|----------|---------------|--------|-------|
| **P0 — CURRENT (T5)** | Story B: S-DEMO-DTU-LIVE-SCENARIO-001-B (scenario progression + enrichment; 7pt; BC-2.06.019/020) | **UNBLOCKED** — draft v1.0 shell; story-writer materializes full impl spec + contract-completeness check | Contract-completeness front-load REQUIRED before locking spec. remove-uncertainty after materialization. 12-gate delivery. |
| P1 (parallel) | S-DEMO-MULTI-TENANT-DTU-001 (T6) | ready v1.2 — independently deliverable | remove-uncertainty COMPLETE (D-1076). Start after T5 materialize step begins. |
| P2 | S-DEMO-004 — add depends_on + data-distinctness AC | draft — needs architect/PO update (T8) | Not on current critical path |
| P3 | S-DEMO-LAUNCHER-CONSOLIDATION-001 | draft stub (T11) | story-writer materialization + human review needed |
| P4 | Multi-client SOC-analyst narrative story | not-authored (T13) | After data layer + tooling exist |
| optional | S-5.02 / S-3.13 / S-5.04 | not-started (T15) | Capability discovery for narrative if needed |

#### Per-Story Delivery Step Ledger

Per-story delivery follows the canonical 12-gate sequence:
1. `dclaude:remove-uncertainty` (standing directive — ALWAYS first)
2. `vsdd-factory:worktree-manage create <STORY-ID>` (worktree setup)
3. `vsdd-factory:test-writer` — stubs + failing Red Gate tests
4. `vsdd-factory:implementer` — TDD green (one failing test → minimum code → micro-commit)
5. LOCAL adversary 3-CLEAN strict (BC-5.39.001 D-779; CLEAN(strict) = zero findings ANY severity)
6. `vsdd-factory:demo-recorder` per-AC (POL-10)
7. Push feature branch to `origin/feature/<story-id>` — **REQUIRED before PR create; use `timeout: 600000` or `run_in_background: true` (pre-push gate ~14 min cold — Story-A lesson)**
8. `vsdd-factory:pr-manager` — PR create
9. PR-LEVEL adversary 3-CLEAN strict + `vsdd-factory:pr-reviewer` APPROVE + `vsdd-factory:security-reviewer` CLEAR — **push any fix commits BEFORE re-running PR-LEVEL cascade (DRIFT-ORCH-PRLEVEL-PUSH-001); adversary dispatch MUST use absolute worktree path + directory sanity-check guard (Story-A lesson)**
10. CI all green
11. Squash-merge to develop
12. Worktree cleanup + state-manager post-merge burst (POL-14 BC promotions + sprint-state.yaml update)

**Active story pointer:**
- **Story B (S-DEMO-DTU-LIVE-SCENARIO-001-B) — T5 CURRENT. UNBLOCKED (Story A merged PR #181 develop@c287b00d 2026-06-10). draft v1.0 shell → story-writer materializes full impl spec.** `sprint-state.yaml current_story.story_id = S-DEMO-DTU-LIVE-SCENARIO-001-B`.

**Mid-cascade restart note:** If a fresh session finds an in-flight worktree/branch/open-PR for Story B, cross-reference `sprint-state.yaml` `current_story.delivery_step` + `gh pr list` + `.worktrees/` to determine the exact resume step before dispatching.

> **This snapshot is current as of D-1090 (2026-06-10 durability hardening; STATE v7.741). Prior snapshot (D-1082) archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`.**

---

### 4. Standing Authorizations and Rules

**D-1090 STORY B (T5) AUTONOMY GRANT — ACTIVE (user-authorized 2026-06-10)**
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
- **factory-artifacts is PUSHED to origin/factory-artifacts after each state burst (off-machine durability; user-authorized 2026-06-08, D-1066). The orchestrator/state-manager pushes factory-artifacts as the final step of every state burst. (This supersedes the prior LOCAL-ONLY default.)**
- Single-commit-per-burst (TD-VSDD-053) — no Stage-2/backfill chains
- BC-5.39.001 3-CLEAN strict (per D-779 disambiguation): streak advances ONLY on CLEAN(strict)=zero findings of ANY severity
- Fix-in-scope — no defer-pattern for AI-found AI-generated defects
- TD-VSDD-091 — no volatile line-number pins in .factory/ narrative; use function anchors
- **remove-uncertainty-per-story:** run `dclaude:remove-uncertainty` on EVERY implementation story before TDD delivery (user standing directive 2026-06-08, D-1061). Applies to all remaining Phase C stories and future waves.
- **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001):** after ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix means the adversary reviews stale code. Verify `git rev-parse origin/feature/<branch>` == local worktree HEAD before re-gating. (D-1065, codified 2026-06-08; evidence: PR #179 PR-LEVEL SEC-001 fix committed locally but unpushed — adversary reviewed stale remote.) DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001: this rule should also be mirrored into CLAUDE.md §Standing rules — **HUMAN-ONLY CLAUDE.md edit** (Pipeline Authority); non-blocking.

---

### 5. Parked Worktrees

| Worktree | Status | Action |
|----------|--------|--------|
| `.worktrees/S-3.09` | FROZEN | Leave alone |
| `.worktrees/W3-FIX-S307-001` | BLOCKED/superseded | Leave alone |
| All wave-5 story worktrees | CLEANED | Removed at merge |

---

### 6. Open Follow-Ups and Drift Items

**CLAUDE.md edit needed (HUMAN ONLY — non-blocking):**
- DEFER-CLAUDEMD-BC216002-MISLABEL-001: SAP-1 probe cites BC-2.16.002 as "Structured Event Catalog" but that BC is "Multi-Step Fetch Pipeline"; catalog lives in BC-2.05.005/BC-2.03.010. Human-mandated CLAUDE.md edit required.

**Active open drift items (non-blocking unless noted):**
- DRIFT-D850-001: **RESOLVED D-1059 2026-06-08** — BC-2.16.002 v1.70 POST-vs-GET pagination clause authored; S-DEMO-CLAROTY-PAGINATION-001 BC gap CLOSED. Story now materializable by story-writer.
- DRIFT-D954-001: BC-3.5.002 precondition 3 mis-cite in prism-dtu-armis (~40+) + prism-dtu-slack (1) — S-MAINT-W3SEC-CITE-SWEEP-002 anchored; story-writer materialization needed.
- DRIFT-D1016-SEC-007: QueryParams.start_time/end_time as Option<String>; TimestampString newtype candidate — architect/PO adjudication.
- DEFER-EQUERY009-001: BC-2.11.007 DI-021 E-QUERY-009 enforcement absent from live path — phase-5 adjudication.
- S-DEMO-LAUNCHER-CONSOLIDATION-001: draft stub; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human review of script-lifecycle question needed.

**Pre-existing maintenance stories (wave-independent):**
- S-MAINT-W3SEC-CITE-SWEEP-002 (armis+slack cite sweep)
- S-MAINT-ORPHAN-SENSORS-DIR-001 (top-level sensors/*.toml cleanup)
- S-MAINT-EDITION-SYNC-001 (workspace edition 2024 migration)
- S-POL-14-STATUS-SYNC-001 (BC promotion + story-status sync; maintenance wave)
- S-DEMO-MULTI-TENANT-DTU-001 (3 open OQs; needs story-writer materialization)

---

### 7. Resume Protocol Checklist

Run these commands at start of a fresh session to verify state:

```bash
# 0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star)
# The current priority is the multi-client SOC demo.
# Do NOT drift into unrelated single-story pipeline machinery.

# 1. Factory worktree health (BLOCKING preflight)
# Use: vsdd-factory:factory-worktree-health skill

# 2. Verify develop HEAD == c287b00d
git log --oneline develop | head -1
# Expected: c287b00d ...
# NOTE: local develop was fast-forwarded to c287b00d in session 2026-06-10 (was stale at 64d34967).
# This is correct — no SHA drift. If showing 64d34967, fetch + fast-forward before dispatching.

# 3. Verify STATE.md version
grep '^version:' .factory/STATE.md
# Expected: version: "7.741"

# 4. Verify no open PRs
gh pr list --state open
# Expected: (empty) — Story A merged PR #181; Story B not yet started.

# 5. Confirm factory-artifacts local vs remote
git -C .factory log -1 --format='%h %s'
git -C .factory rev-parse origin/factory-artifacts 2>/dev/null || echo "no remote yet"
# D-1090 burst is LOCAL-ONLY. origin/factory-artifacts = 4d28cbc2. Push on next state burst per D-1066.

# 6. Read task ledger → CURRENT POINTER + NEXT ACTION
cat .factory/objectives/multi-client-soc-demo-tasks.md | grep -A5 'CURRENT POINTER'
# Expected: CURRENT POINTER = T5 (S-DEMO-DTU-LIVE-SCENARIO-001-B UNBLOCKED).
# Full story list: §Complete Story Roadmap in task ledger (9 stories: 6 core + 3 optional; source of truth).

# 7. Read §RESUME SNAPSHOT 2026-06-10-STORY-B-DELIVERY-D1090 in this file
# Confirm develop_head = c287b00d, STATE version = 7.741, north-star next action = T5 Story B materialize.

# 8. Confirm active story + delivery step
grep -A4 '^current_story:' .factory/stories/sprint-state.yaml
# Expected: story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B, delivery_step: not-started
# (If delivery_step is past not-started, check gh pr list + .worktrees/ to find exact resume point)
```

---

### 8. Where Extracted History Lives

This compaction (D-1056) archived the following:

| Content | Archive Location |
|---------|-----------------|
| Per-story cascade pass tracking (STATE.md YAML frontmatter keys for 25+ stories) | `cycles/wave-5-e-demo-fidelity/frontmatter-cascade-archive.md` |
| Decision rows D-700..D-1054 | `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md` |
| Superseded SESSION-HANDOFF resume snapshots (D-1047 through D-988 + all earlier) | `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md` |
| Burst narratives (D-735..D-1055) | `cycles/wave-5-e-demo-fidelity/burst-log.md` |
| Lessons learned | `cycles/wave-5-e-demo-fidelity/lessons.md` |
| Wave-0 history | `cycles/wave-0-plugin-prereqs/` |
| Wave-3 history | `cycles/wave-3-multi-tenant/` |
| Wave-4 history | `cycles/wave-4-operations/` |

Full pre-compaction STATE.md and SESSION-HANDOFF.md are preserved in git history on the `factory-artifacts` branch at the commit immediately preceding the D-1056 compaction commit.

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

10. **factory-artifacts PUSH-AFTER-EACH-BURST (user-authorized D-1066, 2026-06-08).** The state-manager PUSHES factory-artifacts to origin/factory-artifacts as the FINAL step of every state burst (off-machine durability). This supersedes the prior LOCAL-ONLY default. Push is `git -C .factory push origin factory-artifacts` (normal push, NOT force-push, NOT to main/develop). If the remote branch does not yet exist, first push with `-u` flag: `git -C .factory push -u origin factory-artifacts`. DEFER-CLAUDEMD-FACTORY-PUSH-POLICY-001 tracks the corresponding CLAUDE.md §Git Workflow mirror (human-only edit).

11. **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001, D-1065).** After ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix-commit causes the adversary to review stale code. Verify `git rev-parse origin/feature/<branch>` == local worktree HEAD before re-gating.

---

_End of SESSION-HANDOFF.md. Superseded snapshots archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`._
