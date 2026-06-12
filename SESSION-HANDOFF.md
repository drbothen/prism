---
document_type: session-handoff
level: ops
version: "7.753"
status: current
timestamp: 2026-06-12T03:45:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **D-1102 UPDATE (2026-06-12T03:45Z) — SEE STATE.md D-1102 FOR FULL DETAIL.** MCP PR #184 squash-merged → develop@`c200d5a2`. DTU LOCAL CONVERGED 3/3 strict at pass 33 (head `80749dbb`). NEXT: DTU reconciliation vs c200d5a2 → un-park PR #182 → PR-LEVEL cascade → merge LAST → register burst (22 items) → RESUME T5. BC-INDEX v6.26. STATE v7.753. Full SESSION-HANDOFF rewrite deferred to register burst after DTU merge.
>
> **PRIORITY READ ORDER — 2026-06-12-REVIEW-CYCLE-CHECKPOINT-D1101 base snapshot below still valid for branch context except where superseded by D-1102 note above.** Read §ACTIVE OBJECTIVE (North Star) FIRST, then §RESUME SNAPSHOT below, then STATE.md frontmatter.
> develop HEAD NOW `c200d5a2` (MCP PR #184 squash-merged; D-1102). DTU LOCAL CONVERGED head `80749dbb`. factory-artifacts PUSHED to origin/factory-artifacts (D-1066 standing authorization). STATE v7.753.

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

**NEXT CONCRETE ACTION: MCP push branch → PR → PR-LEVEL 3-CLEAN strict (directed probe: wire_config_swap_cache_flush cache-flush listener chain; verify boot.rs wire_config_swap_cache_flush present + invoked + integration tests green in pass-1) → pr-reviewer → security → CI → squash-merge SECOND. PARALLEL: DTU LOCAL cascade continues (NEXT pass 23, streak 1/3, head 0ed1f976, 30c over c287b00d/f88b10e3) → if pass 23+24 CLEAN(strict) → LOCAL CONVERGED → merge-base reconciliation vs f88b10e3 → push to PR #182 → un-park → PR-LEVEL cascade → merge LAST. See §BRANCH STATE for per-branch NEXT ACTIONS.**

**Task ledger (granular, status-tracked, source of truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT POINTER: T5 (interrupted by review cycle; D-1091..D-1101). T1+T2+T3+T4+T4-A DONE. ADR-036 v2.2. BC-INDEX v6.25. BC-3.4.003 v1.1. BC-3.6.001 v0.8. BC-3.5.002 v0.5. ARCH-INDEX v2.133. STORY-INDEX v2.348. error-taxonomy v1.76. VP-INDEX v1.78 (157/144). policies v1.32. prd v1.12. BC-2.06.018 v1.6 ACTIVE. STATE v7.752.**

---

## §RESUME SNAPSHOT — D-1102 NOTE (2026-06-12T03:45Z — supersedes branch state in D-1101 snapshot below)

> **D-1102: MCP MERGED + DTU LOCAL CONVERGED.** See STATE.md D-1102 for full detail.
> - develop HEAD: `c200d5a2` (PR #184 fix/review-2026-06-10-mcp-boot squash-merged 2026-06-12T03:37Z)
> - BC-INDEX: v6.26 (BC-2.05.001 v1.4, BC-2.16.002 v1.77 updated in MCP cascade)
> - DTU LOCAL: CONVERGED 3/3 strict at pass 33 — head `80749dbb`; PR #182 PARKED (merges LAST)
> - QRY worktree + local branch: CLEANED (remote already auto-deleted)
> - Register-burst checklist: 22 items (items 20/21/22 added this session)
> - **NEXT ACTION:** DTU merge-reconciliation vs develop@c200d5a2 → pr-manager → PR-LEVEL 3-CLEAN strict → squash-merge → register burst → RESUME T5 (S-DEMO-DTU-LIVE-SCENARIO-001-B)
> _Full SESSION-HANDOFF zero-context rewrite deferred to register burst after DTU merge. D-1101 base snapshot below remains accurate for all context EXCEPT branch state superseded above._

---

## §RESUME SNAPSHOT 2026-06-12-REVIEW-CYCLE-CHECKPOINT-D1101

> **START HERE (base snapshot — superseded for branch state by D-1102 note above).** This snapshot is self-contained for context. A fresh session should read the D-1102 note above first for current branch state, then use this snapshot for full context.
> _Previous snapshot (2026-06-11-REVIEW-CYCLE-CHECKPOINT-D1100; STATE v7.751) supersedes and archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`. The D-1090 Story B NEXT-ACTION content remains valid and is preserved in §RESUME T5 below._
> _D-1101 pause checkpoint: user physically relocating; session pausing. All branch heads in §BRANCH STATE are EXACT at checkpoint — superseded by D-1102 note above for current heads._

---

### FRESH-SESSION RESUME PROTOCOL (zero prior context)

1. Run `vsdd-factory:factory-worktree-health` (devops-engineer) — **BLOCKING**; do not read state until it passes.
2. Read §GOAL below (sequence to completion), then STATE.md frontmatter (`current_step`, D-1101 decision row).
3. **Verify branch state with `git log`.** Worktree dirs are under `.worktrees/` (names: `FIX-REVIEW-DTU-2026-06-10`, `FIX-REVIEW-MCP-2026-06-10`, `FIX-REVIEW-QRY-2026-06-10`). Run `git -C .worktrees/<dir> log --oneline -3` and compare against §BRANCH STATE below. The D-1101 checkpoint heads are EXACT (session paused cleanly; no agents in flight).
4. Confirm develop HEAD `f88b10e3` (`git log --oneline develop | head -1`) — QRY PR #183 squash-merged last session.
5. `gh pr list --state open` → expect exactly ONE: draft PR #182 (fix/review-2026-06-10-dtu-fleet — parked with custody note; merges LAST; do NOT close, do NOT mark ready; pr-manager assumes it at DTU convergence).
6. Confirm main-tree CLAUDE.md is CLEAN (`git status --porcelain CLAUDE.md` → nothing; carry-forward resolved at QRY merge). Apply the read-discipline rule for adversary dispatches (lesson p — ALL code reads, grep/rg, and line-number citations MUST use the worktree absolute path; verify `git -C <worktree-path> log -1` matches stated head before citing anything).
7. Long gates (pre-push `just check`, multi-minute CI waits) run harness-tracked in orchestrator context or via Monitor-equipped agents — do NOT dispatch sub-agents to wait on long gates (lesson r; 4 agent terminations this session).
8. **QRY worktree cleanup PENDING:** `worktree-manage cleanup FIX-REVIEW-QRY-2026-06-10` (or equivalent) + `git push origin --delete fix/review-2026-06-10-query-core`. Do this at start of next session before dispatching any new agents.

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) — INTERRUPTED at T5 by user-directed full-codebase review (2026-06-10); PAUSED at D-1101 (user physically relocating 2026-06-12) |
| **develop HEAD** | `f88b10e3` (QRY PR #183 squash-merged 2026-06-11; was c287b00d) |
| **STATE version** | v7.752 |
| **BC-INDEX version** | v6.25 (total 250; active 232; draft 5; retired 6; BC-3.4.003 v1.1 human-authorized per-clone recovery; BC-3.6.001 v0.8 PagerDuty-403 carve-out + Invariant 5 AuthReject column; BC-3.5.002 v0.5 Decision-B network-mode scope) |
| **STORY-INDEX version** | v2.348 (total_stories 194) |
| **VP-INDEX version** | v1.78 (157 registered, 144 active; VP-157 added D-1099 — BC-3.6.001 unsupported-mode 400 guard) |
| **ARCH-INDEX version** | v2.133 |
| **error-taxonomy version** | v1.76 |
| **policies version** | v1.32 |
| **prd version** | v1.12 |
| **test-vectors version** | v2.9 |
| **Open PRs** | 1 — draft PR #182 (parked; custody note; see §BRANCH STATE branch 2) |
| **factory-artifacts** | PUSHED to origin/factory-artifacts (D-1066; D-1100 burst pushed) |

---

### 2. §GOAL (unchanged — do not override)

Active objective: **multi-client SOC-analyst live demo**. Interrupted at T5 by a user-directed full-codebase review (2026-06-10). Sequence to completion:

1. MCP pr-manager 9-step delivery (SECOND in pinned order; merge-reconciliation COMPLETE head 08fdc38c; push branch → PR → PR-LEVEL 3-CLEAN strict with directed probe on wire_config_swap_cache_flush chain → pr-reviewer APPROVE → security MAY PROCEED → CI green → squash-merge).
2. DTU LOCAL cascade to **3-CLEAN strict** (NEXT pass 23, streak 1/3, head 0ed1f976) → pr-manager 9-step delivery LAST via PR #182.
3. State-manager **register burst** (§REGISTER BURST CHECKLIST below — 19 items as amended through D-1100).
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

### 4. §BRANCH STATE (updated D-1101 — verify `git log` on resume as standard discipline)

Worktree directories: `.worktrees/FIX-REVIEW-QRY-2026-06-10` (MERGED — cleanup pending), `.worktrees/FIX-REVIEW-MCP-2026-06-10` (CONVERGED), `.worktrees/FIX-REVIEW-DTU-2026-06-10` (cascade in flight). Branches were cut from develop@c287b00d. QRY branch squash-merged to f88b10e3 this session.

**Cascade protocol (active branches):** BC-5.39.001 3-CLEAN **strict** per branch; report format CLEAN(strict)/CLEAN(PR-merge); fresh-context adversary each pass. Adjudicated **do-not-reflag lists** are recorded per branch — key standing exclusions: P1-03 deviation (human-authorized), BC frontmatter timestamps (register burst), DEF-1 Claroty pagination (post-merge fix), denylist wiring (S-WATCHDOG-WIRING-001), scope params (S-QUERY-SCOPE-PARAMS-001).

**1. `fix/review-2026-06-10-query-core`** — **MERGED** (PR #183 squash-merged to develop@f88b10e3 2026-06-11). **LOCAL cascade CONVERGED 3/3** (16 passes); **PR-LEVEL cascade CONVERGED 3/3** (passes 1–3 strict). pr-reviewer APPROVED (261b98d9 polish + PR-body count fix). Security MAY PROCEED. All CI green. Branch MERGED and feature branch should be deleted.
- **NEXT ACTION: CLEANUP** — `worktree-manage cleanup FIX-REVIEW-QRY-2026-06-10` + `git push origin --delete fix/review-2026-06-10-query-core`. Do this FIRST at next session start.
- Out-of-diff OBS from PR-LEVEL pass 3 (do-not-reflag in DTU/MCP passes): **PRL3-01** E-QUERY-010/QueryVirtualFieldFailed zero-emitter collision → register-burst item 15 EXTENDED (alongside E-QUERY-009 and Internal.detail class-sweep).
- CLAUDE.md carry-forward RESOLVED: commit 261b98d9 landed on develop via QRY merge. Main-tree `M CLAUDE.md` staged edits were verified-subsumed by the worktree commit and discarded from main tree. Main checkout is clean.

**2. `fix/review-2026-06-10-dtu-fleet`** — 30 commits over c287b00d (= 30 commits over f88b10e3, same base; DTU was branched from c287b00d before QRY merged), head `0ed1f976`. Streak 1/3; **NEXT pass 23**.

Pass history since D-1099 (passes 20–22):
- **Pass 20 CLEAN(strict)=no:** `d58af213` (P20-01/02 — BC-3.6.001 v0.6: Postcondition 5 `{"error":"unsupported_failure_mode","mode":"<name>"}` body + VP-157 anchor corrections replacing erroneously-cited VP-131 sites); `c46f3944` (P20-03 RCA: `136497b4` had wrongly ported Claroty's Tower-layer 401 to PagerDuty's route-level path — PagerDuty MUST return 403 via `PagerDutyState.auth_reject` in `enqueue.rs` to match real Events API; restored contractually); `050fa46d` (P20-04 real-crate `apply_config` unprocessable arms ×3: Jira `state.rs:299-337`, PagerDuty `state.rs:186-231`, Slack `state.rs:107-151`; closing work-order from BC-3.6.001 v0.7). Streak reset 0/3.
- **Pass 21 CLEAN(strict)=no:** P21-01 HIGH (BC-3.6.001 Postcondition 1 residue — still said "HTTP 401 on every request" universally; contradicted by PagerDuty route-level 403 restored in c46f3944 and by Description prose "auth-reject (401/403)"). Closed: BC v0.7→v0.8 (Postcondition 1 PagerDuty-403 carve-out; Invariant 5 AuthReject status-code column per-clone verified codes; TV-10 new; TV-1/TV-9/EC-005 clarified). P21-02 OBS (stale versioned BC-3.6.001 cites in harness test header — 10 locations). Closed commit `0ed1f976` (versioned cites made version-free; AuthReject per-clone header note added). Streak reset 0/3.

- **Pass 22 CLEAN(strict)=YES:** BC v0.8 triangle contradiction-free (Postcondition 1 PagerDuty-403 carve-out + Invariant 5 AuthReject column vs route-level 403 in enqueue.rs internally consistent), P20-04/P21-02 closures verified (real-crate unprocessable arms + version-free harness cites), SAP-1/SAP-2 clean. Streak **1/3**. Head 0ed1f976 unchanged.

- **NEXT ACTION: launch adversary pass 23 (streak 1/3)** with corrected ground truth:
  - 5 crates 3-way gated (common + claroty/armis/crowdstrike/cyberint); 7 two-way
  - COMP-012 in all 9 COMP-DTU arrays (332c99bd); Armis `os_name`/`risk_score`/`manufacturer` deterministic non-null (0959e92f)
  - BC-3.4.003 v1.1 (per-clone recovery representations — crowdstrike explicit-200, claroty absence-of-call-record, armis absence-of-injection; ALL CORRECT — do NOT reflag)
  - Cyberint invariant-6 exempt (route-layer `AuthMode::Reject` = architecturally distinct from GenOpts::overrides)
  - BC-3.6.001 v0.8 now in effect: PagerDuty `auth_mode="reject"` → 403 via route-level `PagerDutyState.auth_reject` (enqueue.rs); ALL OTHER clones → 401 Tower layer; do NOT reflag this distinction
  - Real-crate unprocessable arms closed (050fa46d — Jira/PagerDuty/Slack `apply_config` now handles `"unprocessable"`)
  - P21-02 stale-cite closure (0ed1f976 — version-free harness test cites + AuthReject header note)
  - ALL code reads MUST use the worktree absolute path `/Users/jmagady/Dev/prism/.worktrees/FIX-REVIEW-DTU-2026-06-10/` — do NOT read from `/Users/jmagady/Dev/prism/crates/`
  - Verify `git -C /Users/jmagady/Dev/prism/.worktrees/FIX-REVIEW-DTU-2026-06-10 log -1 --format='%H %s'` matches `0ed1f976` before citing any line numbers

- **PUSHED to origin + DRAFT PR #182 exists** (artifact of a contained unauthorized-push incident, lesson e). PR is **parked draft with custody note**. pr-manager assumes it at convergence. **DTU merges LAST.** Merge-base assessment: c287b00d (= f88b10e3 merge base); DTU touches `crates/prism-dtu-*`; QRY/MCP touches `crates/prism-query/`, `crates/prism-mcp/` — EXPECTED CLEAN (verify at pr-manager time before pushing).

- Full do-not-reflag cumulative: F-P8-01 closure (module-decomposition v1.19); candidate (b) doc-nuance; candidate (c) threatintel/nvd fixture-gen OBS/harmless (register-burst item 19 ADJUDICATED); P10-01 NO-ACTION adjudication (Armis key-presence discriminator — human ratification requested); P10-02 NIT-2 Story B anchor; P12-01 Armis null-value closure (0959e92f); cyberint/crowdstrike incidents-route gaps pre-existing/out-of-scope (register-burst item 4); P14-01 override-porting closure (dee1f2a0); BC-3.4.003 v1.0 ruling-A Cyberint exemption; P15-01 refutation (false CRIT worktree-path error; implementation confirmed correct); P15-02 closure (BC-3.4.003 v1.1 human-authorized per-clone recovery); passes 16/17 CLEAN; P18-01 closure (cd1c157b; cyberint post_reset doc corrected); P19-OBS closure (136497b4; BC-3.6.001 v0.5; ops-clone failure-mode port + 400 guard); P20-01/02 closure (d58af213; BC-3.6.001 v0.6 Postcondition-5 body + VP-157 anchor corrections); P20-03 closure (c46f3944; PagerDuty AuthReject 403 restoration); P20-04 closure (050fa46d; real-crate unprocessable arms ×3); P21-01 closure (BC-3.6.001 v0.8; PagerDuty-403 carve-out in Postcondition 1 + Invariant 5 per-clone AuthReject column); P21-02 closure (0ed1f976; stale versioned cites → version-free); **pass 22 CLEAN(strict)=YES** (no new findings; triangle contradiction-free verified). PRL3-01 (QRY PR-LEVEL): E-QUERY-010 zero-emitter collision noted → register-burst item 15 EXTENDED; do-not-reflag for DTU pass 23 (out-of-diff observation from QRY delivery; no DTU code changes required).

**3. `fix/review-2026-06-10-mcp-boot`** — **MERGE-RECONCILIATION COMPLETE** at head `08fdc38c` (34+ commits; develop@f88b10e3 merged in via merge commit 1f5c1a06 + post-merge fix 08fdc38c WriteExecutor::new 6th CacheInvalidator param in test helper; gate 4140/4140, 45 skipped, EXPECTED=50 pass). Worktree: `.worktrees/FIX-REVIEW-MCP-2026-06-10`.
- Pass history: P5-02 fail-closed write audit (ab2ab0ce); P5-01 re-anchor (5863dbc7); P5-03 capability fields (7c1c2a5e). Pass-6: P6-01 OBS dead computation in `audit_emitter.rs` — commit b4707e95. Pass-7: CLEAN(strict)=YES; streak 1/3. Pass-8: P8-OBS-01 [process-gap] SAP-1 probe vs BC-2.16.002 crate scope — PO-half CLOSED (BC-2.16.002 v1.76); CLAUDE.md SAP-1 wording half PENDING HUMAN; streak reset 0/3. Pass-9: CLEAN(strict)=YES; streak 1/3. Pass-10: F-P10-01/02/03 LOW (stale docstring + tautological timeout test + Jira/PD/Slack network-mode parity); closed commits `487122e3` + `b0099308`; architect Decision B: LOGICAL-MODE-ONLY BY DESIGN; streak reset 0/3. Pass-11: CLEAN(strict)=YES; streak 2/3. **Pass-12: CLEAN(strict)=YES; streak 3/3 — CONVERGED.** Pass-13 (final verification pass): CLEAN(strict)=YES — zero findings of ANY severity.
- **NEXT ACTION: pr-manager delivery — 9-step PR cycle (SECOND in pinned order MCP→DTU after QRY already merged).** Merge-reconciliation DONE. Push branch (head 08fdc38c) → create PR targeting develop → PR-LEVEL 3-CLEAN strict (fresh-context adversary; **directed probe pass-1: verify wire_config_swap_cache_flush cache-flush listener chain survived merge intact — boot.rs wire_config_swap_cache_flush present + invoked + integration tests green**) + pr-reviewer APPROVE + security CLEAR + CI green → squash-merge to develop. PR-LEVEL seed do-not-reflag list from LOCAL list below.
- Full do-not-reflag cumulative for PR-LEVEL adversary: two-class audit contract adjudicated; capability-fields carrier = tracing emission per dispatch; P6-01 dead-computation closure (b4707e95); P8-OBS-01 PO-half closed (BC-2.16.002 v1.76); F-P10-01/02/03 closures (487122e3 + b0099308; Decision B logical-mode-only); SAP-1 CLAUDE.md wording PENDING HUMAN (adversary may note as process-gap; action requires human; not a code defect; do not flag as blocker); PRL3-01 E-QUERY-010 (out-of-diff QRY observation; not in MCP scope).

---

### 5. §REGISTER BURST CHECKLIST (state-manager — runs AFTER all merges, BEFORE T5)

1. Close 32 validated TDs with PR evidence + TD-A-004 (ratification memo in `proposals/TD-S-PLUGIN-PREREQ-A-004-close-as-superseded-ratification.md`) + TD-VSDD-094.
2. Retire ghost TD-W2-SENSORS-FULL-001; re-file TD-ADR005-001 vs current security surface; file TD-S307-001 (E-QUERY-024/027/028); re-home TD-VSDD-019; relocate TD-VSDD-082/083/084 + TD-S305-001; surface per-file TDs (S305 family, S302-005) in main register.
3. NEW TD: cache normalized-vs-raw deviation, human-authorized 2026-06-10, anchor S-CACHE-SPEC-COMPLIANCE-001.
4. File CS-05 (CrowdStrike TOML column coverage incl. `containment_status`) as product-decision story candidate. **EXTENDED D-1095:** also file `cyberint incidents` + `crowdstrike incidents` TOML tables have NO DTU route — same family as CS-05 (pre-existing out-of-scope gaps surfaced during P12-01 sibling sweep; each has a TOML `[[tables]]` declaration with NO corresponding DTU route handler; requires product decision on DTU route scope per table).
5. Priority adjustments: S304-AUDIT-001→P2; WV15 set down (PR35-001/002→P3, PR36-001/PR40-001→P4); MED-003 merge into ULID-001; MUTATE-005 P2/P3 reconcile.
6. STATE.md count refresh (workspace_test_count, develop_head post-merges recorded as f88b10e3 already updated, version pins to CURRENT values at register-burst time — error-taxonomy v1.76, BC-INDEX v6.25, STORY-INDEX v2.348, ARCH-INDEX v2.133, policies v1.32, prd v1.12, BC-3.6.001 v0.8, BC-3.5.002 v0.5) + STORY-INDEX frontmatter BC-count re-basis per BC-INDEX.
7. Register S-DEMO-004 follow-ups: n/a (row registered v2.342).
8. Anchor ~10 STATE drift promises (edition sync, tape paths, pagination validation, E-QUERY-009, orphan sensors dir, BC-3.5.002 cites).
9. Input-hash refresh sweep (corpus-wide; 313 stale reported by `compute-input-hash --scan specs` at D-1091 — burst-set artifacts already refreshed at D-1091).
10. POL-23 Direction-A remediation: normalize 4 deviant BC timestamps to creation dates (BC-2.07.003, BC-2.16.002, BC-2.06.018, BC-2.01.017) + fix 2 POL-27 modified-format violations (BC-2.16.002, BC-2.06.018) + POL-29 sweep.
11. POL-32 tombstone rows: BC-2.11.001 missing v1.4 row; BC-3.2.001 v0.9 row (reconstruct from BC-INDEX v5.79).
12. TD-S302-005 premise refresh (pipeline implemented; blocker = cold-start buffer routing per S-2.08 Rule 5).
13. Story-ID sweep: BC-2.11.001 v1.7 + BC-INDEX v6.19 generic citation → S-QUERY-SCOPE-PARAMS-001.
14. DEF-1: post-merge fix-pr for Claroty list_alerts/list_audit_logs ignoring POST-body offset/limit + Gap-CL-004 comment correction (**BEFORE demo**).
15. File taxonomy↔code symmetry-audit maintenance story (4 collisions found: E-CFG, E-QUERY-007 tombstone, E-QUERY-003 triple, E-QUERY-011 two-BC; work items: E-QUERY-006 embedded-detail + E-QUERY-009 LATENT enforcement + **E-QUERY-010 zero-emitter collision** (PRL3-01 QRY PR-LEVEL pass 3) + Internal.detail class-sweep note, per ADR-038 adjacent-findings).
16. S-MAINT-VERIFY-PIPELINE-001 scope doc exists in `proposals/S-MAINT-VERIFY-PIPELINE-001-scope.md` — story-writer materializes (absorbs TD-CICD-001 + KANI-001 + FUZZ-002/003).
17. ~~BC-INDEX row 220 stale "v1.70" cite for BC-2.16.002 → sync to current (v1.75).~~ **RESOLVED D-1094** — row-220 inline-pin synced to v1.76. _(resolved D-1094)_
18. Lessons addenda [process-gap] to `cycles/wave-5-e-demo-fidelity/lessons.md`: (h) architect micro heredoc appends = TD-FACTORY-HOOK-BYPASS-001 mechanism violation; (i) MCP implementer committed BC-2.16.002 v1.75 to factory-artifacts directly; (j) P6/P7 churn pattern; **(k) SAP-1 wording/scope reconciliation [process-gap] — PENDING HUMAN CLAUDE.md edit; (l) do-not-reflag list completeness discipline; (m) load-bearing string sentinels make naive convention sweeps regression traps; (n) key-presence/null-tolerant parity assertions are paper-fix enablers; (o) [related to n] SAP-2 extension candidate — non-null value assertions for all TOML-declared columns on seeded path; (p) adversary worktree-path read error → false CRIT P15-01; standing discipline: ALL code reads under worktree absolute path; (q) Source-of-Truth rule-7 spec-amendment-to-match-code escalation correctly executed (first instance this cascade); (r) sub-agents terminate awaiting long gates (4 instances: 2 implementer + 2 pr-manager) — long gates must run harness-tracked in orchestrator context; codify in dispatch templates.**
19. ~~threatintel/nvd unused fixture-gen feature-forwarding — adjudicate or note.~~ **ADJUDICATED D-1093:** OBS/harmless. _(resolved D-1093)_

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

### 7. §ALSO NOTE (durability items)

- **CLAUDE.md carry-forward RESOLVED (D-1098/D-1100):** Main-checkout uncommitted CLAUDE.md review-cycle edits landed on develop via QRY PR #183 (commit 261b98d9 → squash to f88b10e3). Main-tree staged `M CLAUDE.md` edits verified-subsumed by worktree commit and discarded. Main checkout is now clean (`git status` should show no `M CLAUDE.md`).
- **QRY WORKTREE CLEANUP PENDING:** `.worktrees/FIX-REVIEW-QRY-2026-06-10` + remote branch `fix/review-2026-06-10-query-core` — delete at next session start.
- **DTU REBASE-OR-MERGE ASSESSMENT:** DTU branch was cut from c287b00d. f88b10e3 (QRY squash) has the same merge-base. Expected CLEAN (DTU touches `crates/prism-dtu-*`; QRY/MCP touch `crates/prism-query/`, `crates/prism-mcp/`). Before pr-manager delivery, verify: `git -C .worktrees/FIX-REVIEW-DTU-2026-06-10 merge-base HEAD f88b10e3` = c287b00d (confirms no divergence). If clean, pr-manager can push directly.
- **MCP MERGE-RECONCILIATION COMPLETE (D-1101):** develop@f88b10e3 merged INTO fix/review-2026-06-10-mcp-boot — merge commit 1f5c1a06 + post-merge fix 08fdc38c. Union-resolution verified: E-QUERY-007 sweep ZERO (033 flip survives), E-INT-002/003 ZERO, prism-customer-config ZERO. Gate 4140/4140 green. CAUTION: PR-LEVEL pass-1 MUST include directed probe verifying wire_config_swap_cache_flush chain (merge agent initially dropped it during conflict resolution; re-inserted from develop — verify survival).
- **PENDING HUMAN items:**
  - CLAUDE.md SAP-1 wording clarification (register-burst item 18k — CLAUDE.md edit required; HUMAN-ONLY per Pipeline Authority; non-blocking for delivery)
  - Armis key-presence discriminator ratification (P10-01 do-not-reflag adjudication noted; human sign-off needed for final ratification)
  - T11 S-DEMO-LAUNCHER-CONSOLIDATION-001 launcher-lifecycle decision (story-writer materialization requires human answer on script lifecycle before T11/T12)

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
- **factory-artifacts is PUSHED to origin/factory-artifacts after each state burst (off-machine durability; user-authorized 2026-06-08, D-1066).** Reinstated D-1093 (D-1091/D-1092 LOCAL exception expired).
- Single-commit-per-burst (TD-VSDD-053) — no Stage-2/backfill chains
- BC-5.39.001 3-CLEAN strict (per D-779 disambiguation): streak advances ONLY on CLEAN(strict)=zero findings of ANY severity
- Fix-in-scope — no defer-pattern for AI-found AI-generated defects
- TD-VSDD-091 — no volatile line-number pins in .factory/ narrative; use function anchors
- **remove-uncertainty-per-story:** run `dclaude:remove-uncertainty` on EVERY implementation story before TDD delivery (user standing directive 2026-06-08, D-1061). Applies to all remaining Phase C stories and future waves.
- **PR-LEVEL push-before-regate (DRIFT-ORCH-PRLEVEL-PUSH-001):** after ANY PR-LEVEL fix-burst, PUSH the fix commits to `origin/feature/<branch>` BEFORE re-running the PR-LEVEL adversary cascade. LOCAL passes review the local worktree (no push needed); PR-LEVEL passes review the REMOTE PR (`gh pr diff`) — an unpushed local fix means the adversary reviews stale code. Verify `git rev-parse origin/feature/<branch>` == local worktree HEAD before re-gating.
- **Review-cycle worktree discipline (D-1091, lessons (e)/(f)/(g)):** cascade worktrees carry push-guards until convergence; constraints bind re-invocations; exclusive worktree ownership; commit-early; no turn ends with uncommitted work pending a background gate.
- **Worktree-path read discipline (D-1097, lesson p):** adversary dispatches MUST instruct "ALL code reads, grep/rg searches, and line-number citations MUST use the worktree absolute path — do NOT read from the main checkout at /Users/jmagady/Dev/prism/crates/". Orchestrator MUST run ground-truth check before dispatching any fix-burst on a CRIT claim.
- **Long-gate discipline (D-1099, lesson r):** long gates (pre-push `just check`, CI, PR review waits) run harness-tracked in orchestrator context or via Monitor-equipped agents. Sub-agents MUST NOT be dispatched to wait on long gates. No background-gate orphaning.
- **Review-cycle pinned merge order (D-1091, updated D-1100):** QRY ALREADY MERGED. Remaining order: MCP → DTU. DTU last because PR #182 custody + DTU cascade depends on cross-branch sibling artifact alignment.

---

### 9. Parked Worktrees and Review Worktrees

| Worktree | Status | Action |
|----------|--------|--------|
| `.worktrees/S-3.09` | FROZEN | Leave alone |
| `.worktrees/W3-FIX-S307-001` | BLOCKED/superseded | Leave alone |
| `.worktrees/FIX-REVIEW-QRY-2026-06-10` | MERGED — cleanup pending | Delete at next session start via `worktree-manage cleanup` |
| `.worktrees/FIX-REVIEW-MCP-2026-06-10` | MERGE-RECONCILIATION COMPLETE (08fdc38c) — push → PR → PR-LEVEL NEXT | Exclusive ownership; directed cache-flush probe in PR-LEVEL pass-1 |
| `.worktrees/FIX-REVIEW-DTU-2026-06-10` | CASCADE IN FLIGHT — pass 23 next (streak 1/3) | Exclusive ownership; push-guard until LOCAL convergence |

---

### 10. Open Follow-Ups and Drift Items

> Many open drift/TD items are consolidated into the §REGISTER BURST CHECKLIST above (runs after the merges). Items below remain individually tracked.

**CLAUDE.md edit needed (HUMAN ONLY — non-blocking):**
- DEFER-CLAUDEMD-BC216002-MISLABEL-001: SAP-1 probe cites BC-2.16.002 as "Structured Event Catalog" but that BC is "Multi-Step Fetch Pipeline"; catalog lives in BC-2.05.005/BC-2.03.010. Human-mandated CLAUDE.md edit required (register-burst item 18k).
- DEFER-CLAUDEMD-PRLEVEL-PUSH-RULE-001: PR-LEVEL push-before-regate rule should be mirrored into CLAUDE.md §Standing rules — HUMAN-ONLY CLAUDE.md edit.

**Active open drift items (non-blocking unless noted):**
- DRIFT-D954-001: BC-3.5.002 precondition 3 mis-cite in prism-dtu-armis (~40+) + prism-dtu-slack (1) — S-MAINT-W3SEC-CITE-SWEEP-002 anchored; story-writer materialization needed.
- DRIFT-D1016-SEC-007: QueryParams.start_time/end_time as Option<String>; TimestampString newtype candidate — architect/PO adjudication.
- DEFER-EQUERY009-001: BC-2.11.007 DI-021 E-QUERY-009 enforcement absent from live path — register-burst item 8 anchors this; register-burst item 15 now also includes E-QUERY-010 zero-emitter (PRL3-01).
- S-DEMO-LAUNCHER-CONSOLIDATION-001: draft stub; depends_on S-DEMO-003 SATISFIED; story-writer materialization + human review of script-lifecycle question needed (T11).

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
# 0. Read SESSION-HANDOFF.md §ACTIVE OBJECTIVE (North Star) + §RESUME SNAPSHOT D-1101
# The review cycle must complete (cascades → MCP→DTU merges → register burst) BEFORE T5 resumes.

# 1. Factory worktree health (BLOCKING preflight)
# Use: vsdd-factory:factory-worktree-health skill

# 2. Verify develop HEAD == f88b10e3 (QRY merged; MCP and DTU not yet)
git log --oneline develop | head -1

# 3. Verify STATE.md version
grep '^version:' .factory/STATE.md
# Expected: version: "7.752"

# 4. Verify open PRs — expect exactly ONE parked draft
gh pr list --state open
# Expected: #182 draft (fix/review-2026-06-10-dtu-fleet) — PARKED with custody note. Do not close/ready it.

# 5. Verify the two remaining review fix worktrees (QRY should be cleaned up)
git -C .worktrees/FIX-REVIEW-MCP-2026-06-10 log --oneline -1
# Expected: 08fdc38c (MCP merge-reconciliation COMPLETE — NEXT push → PR → PR-LEVEL 3-CLEAN)
git -C .worktrees/FIX-REVIEW-DTU-2026-06-10 log --oneline -1
# Expected: 0ed1f976 (DTU NEXT pass 23 streak 1/3)

# 6. Confirm factory-artifacts local vs remote — should be PUSHED (D-1100 burst)
git -C .factory log -1 --format='%h %s'
git -C .factory rev-parse origin/factory-artifacts 2>/dev/null || echo "no remote yet"

# 7. Confirm main-tree CLAUDE.md is clean (carry-forward resolved at QRY merge; should be clean since D-1100)
git -C . status --porcelain CLAUDE.md
# Expected: nothing (clean; carry-forward landed in PR #183 commit 261b98d9)

# 8. QRY worktree cleanup (do this FIRST if worktree still exists)
ls .worktrees/FIX-REVIEW-QRY-2026-06-10 2>/dev/null && echo "NEEDS CLEANUP"
# If present: worktree-manage cleanup FIX-REVIEW-QRY-2026-06-10
# Then: git push origin --delete fix/review-2026-06-10-query-core

# 9. Task ledger pointer (resumes AFTER register burst)
grep -A5 'CURRENT POINTER' .factory/objectives/multi-client-soc-demo-tasks.md
# Expected: CURRENT POINTER = T5 (resumes after review cycle + register burst)
```

---

### 12. Where Extracted History Lives

| Content | Archive Location |
|---------|-----------------|
| Per-story cascade pass tracking (STATE.md YAML frontmatter keys for 25+ stories) | `cycles/wave-5-e-demo-fidelity/frontmatter-cascade-archive.md` |
| Decision rows D-700..D-1054 | `cycles/wave-5-e-demo-fidelity/decisions-archive-D700-D1054.md` |
| Superseded SESSION-HANDOFF resume snapshots (incl. D-1082, D-1090, D-1091, D-1092) | `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md` |
| Burst narratives (D-735..D-1084) | `cycles/wave-5-e-demo-fidelity/burst-log.md` |
| Lessons learned (incl. D-1091 review-cycle lessons a–g; D-1095/D-1097/D-1099 addenda m,n,o,p,q,r) | `cycles/wave-5-e-demo-fidelity/lessons.md` |
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
