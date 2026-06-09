---
document_type: session-handoff
level: ops
version: "7.724"
status: current
timestamp: 2026-06-09T01:00:00Z
---

# Session Handoff — Prism VSDD Pipeline

> **PRIORITY READ ORDER — D-1073 TASK LEDGER CREATED + D-1072 NORTH-STAR OBJECTIVE PERSISTED + D-1071 PHASE-C COMPLETE + D-1066 FACTORY-PUSH-POLICY-CHANGE. ZERO-CONTEXT RESUME SNAPSHOT.**
> Read §ACTIVE OBJECTIVE (North Star) FIRST, then task ledger (`.factory/objectives/multi-client-soc-demo-tasks.md`), then STATE.md frontmatter + §RESUME SNAPSHOT before dispatching any agent.
> develop HEAD `64d34967`. factory-artifacts PUSHED to origin after each burst (user-authorized D-1066). STATE v7.724.

---

## §ACTIVE OBJECTIVE — Multi-Client SOC-Analyst Live Demo (NORTH STAR)

> **READ THIS FIRST.** This section persists the current priority goal so fresh sessions never drift onto unrelated pipeline machinery (D-1072, user-directed 2026-06-09).

### Goal

Deliver a **MULTI-CLIENT SOC-ANALYST LIVE DEMO**:
- Stand up a collection of DTU clones representing MULTIPLE clients
- Each client has a DIFFERENT sensor combination AND genuinely different per-client data (real data segregation — NOT just client-targeting/federation)
- Prism federates into each client's DTUs
- Prism MCP wired into Claude (stdio)
- Demonstrate an end-to-end SOC-analyst investigation workflow

### Scope Decisions (user, 2026-06-09)

- **SOC-analyst demo FIRST.** Threat-Detection-Engineer (TDE) workflow — detection rules, write/action-back containment — is **DEFERRED** to a separate later track. Reason: requires building the absent `prism-operations` crate + wiring the dispatch-dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).
- **REAL per-client data segregation required.** Not just client-targeting/federation routing. Each client must serve distinct fixture data.

### Capability Audit Verdict (2026-06-09)

`partial-significant-gaps` — but the CORE is genuinely multi-tenant; **build-on, not rebuild**.
- R4 (MCP→Claude): WORKS today
- R1/R2/R3/R5 (multi-client DTU / federation / per-client data / demo tooling): PARTIAL — gap = demo tooling + per-client data seeding
- R6 (TDE write-back): NO → deferred

### Reuse (already merged — DO NOT rebuild)

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (per-org overlays), S-DEMO-001 (per-org adapters / boot step 9A), S-DEMO-002 (single-org E2E + CI harness), S-DEMO-003 (demo scripts + credential CLI + runbook), S-6.20 (multi-clone demo-server), S-6.06–6.10 (4 DTU clones), S-3.1.x / S-3.3.x (org isolation type system + multi-org config boot), S-DEMO-QUERY-PUSHDOWN-001 + per-sensor route-fidelity stories (live-query realism).

### Build Sequence

| Step | Story / Action | Status | Notes |
|------|---------------|--------|-------|
| 1 | **S-DEMO-MULTI-TENANT-DTU-001** | draft P2 — BLOCKED (behavioral_contracts empty) | **NEXT ACTION: PO authors multi-address-binding BC** — resolves OQ-1 (which crate owns MultiInstanceConfig → recommend prism-dtu-common), OQ-2 (Box<dyn BehavioralClone> harness API), OQ-3 (canonical BC IDs). Then architect adjudicates. |
| 2 | Per-instance distinct data seeding | gap — needs story | `CloneConfig.seed` / `fixture_set` ignored by `build_clone_pairs` today (`crates/prism-dtu-demo-server/src/harness.rs`). Wire per-instance seeds (or POST /dtu/configure). New story OR scope-add to step 1. Closes real-per-client-data requirement. |
| 3 | **S-DEMO-004** | draft P0 | Add `depends_on: [S-DEMO-MULTI-TENANT-DTU-001]` (edge currently MISSING). Decide AC-006 data-distinctness via real seeding (NOT port-binding-only shortcut). |
| 4 | Demo tooling generalization | draft stub S-DEMO-LAUNCHER-CONSOLIDATION-001 | Generalize `demo-setup.sh` / `demo-run.sh` / `demo-teardown.sh` to loop over N orgs (today single-org demo-org). Story-writer materialization + human launcher-lifecycle decision needed. |
| 5 | Multi-client SOC-analyst narrative story | NEW — none exists | Multi-client SOC-analyst investigation walkthrough + demo-recorder evidence per persona. |
| 6 (optional) | S-5.02 / S-3.13 / S-5.04 | draft | MCP client targeting, dynamic per-org table availability, sensor health. All optional capability discovery for the narrative. |

**NEXT CONCRETE ACTION: PO authors S-DEMO-MULTI-TENANT-DTU-001 multi-address-binding BC (per-instance multi-address binding + no-cross-tenant-leakage invariant + single-instance backward-compat) and resolves OQ-1/OQ-2/OQ-3. Architect adjudicates after.**

**Task ledger (granular, status-tracked, resume source-of-truth): `.factory/objectives/multi-client-soc-demo-tasks.md` — CURRENT TASK = T1 (D-1073).**

---

## §RESUME SNAPSHOT 2026-06-08-DURABILITY-HARDENING-D1065

> **START HERE.** This snapshot is self-contained. A fresh session with ZERO prior context can resume exactly here.

---

### FRESH-SESSION RESUME PROTOCOL (zero prior context)

0. **Read §ACTIVE OBJECTIVE (North Star) above** — the current priority is the multi-client SOC demo; the per-story VSDD pipeline SERVES this goal. Do NOT drift into unrelated single-story pipeline machinery. Then **read `.factory/objectives/multi-client-soc-demo-tasks.md`**, find the CURRENT POINTER, and execute its NEXT ACTION. The task ledger is the granular, status-tracked resume source-of-truth (D-1073).
1. Run `vsdd-factory:factory-worktree-health` (devops-engineer) — **BLOCKING**; do not read state until it passes.
2. Read STATE.md frontmatter + this §RESUME SNAPSHOT.
3. Verify `git rev-parse origin/develop` == `64d34967` (develop_head). If drift, reconcile before dispatching.
4. Confirm no open PRs (`gh pr list`) and parked worktrees (S-3.09 FROZEN, W3-FIX-S307-001 BLOCKED) are left alone.
5. Pick the next action from §3 Exact Next Steps. Honor §4 Standing Rules (incl. remove-uncertainty-per-story + D-989 autonomy).

---

### 1. Pipeline Status

| Field | Value |
|-------|-------|
| **Mode** | brownfield |
| **Phase** | 3 (Wave 5 — wave-5-e-demo-fidelity) |
| **Wave-5 Phase B** | **COMPLETE** — all 4 lanes + S-MAINT merged |
| **Wave-5 Phase C** | **COMPLETE** — all 4 lanes merged (TRAILING-SLASH, SPEC-PROSE-FIX, PAGINATION-001, HARNESS-CLONE-PARITY-001) |
| **develop HEAD** | `64d34967` |
| **STATE version** | v7.724 |
| **BC-INDEX version** | v6.00 |
| **STORY-INDEX version** | v2.329 |
| **VP-INDEX version** | v1.76 |
| **ARCH-INDEX version** | v2.115 |
| **Active BCs** | 235 |
| **Draft BCs** | 2 (BC-2.06.011 + BC-2.21.001) |
| **Total stories** | 185 |
| **Open PRs** | NONE |
| **factory-artifacts** | pushed to origin after each burst (user-authorized D-1066 2026-06-08) |

---

### 2. What Just Completed

**D-1073 Task Ledger Burst — Durable task ledger created + wired (2026-06-09)**

Created `.factory/objectives/multi-client-soc-demo-tasks.md` — 15-task ordered ledger (T1–T15) with status tracking, owner-agent, dependencies, story refs, done-when criteria. CURRENT TASK: T1. Wired into SESSION-HANDOFF §ACTIVE OBJECTIVE, §FRESH-SESSION RESUME PROTOCOL step 0, §7 Resume Checklist, and STATE.md frontmatter `task_ledger` key. No code/spec/BC/VP/count changes. D-1073. STATE v7.723→v7.724.

**D-1072 North-Star Persistence Burst — ACTIVE OBJECTIVE written (2026-06-09)**

Multi-client SOC-analyst live demo objective persisted to STATE.md + SESSION-HANDOFF.md §ACTIVE OBJECTIVE. No code/spec/BC/VP changes. D-1072. STATE v7.722→v7.723.

**D-1071 S-DEMO-HARNESS-CLONE-PARITY-001 MERGED — PR #180 squash-merged develop@64d34967 (2026-06-09)**

Wave 5 Phase C COMPLETE — all 4 lanes merged. F-P6-DEFER-001 + F-P10-LOW-001 CLOSED. CI 43/43 GREEN.

**D-1064 S-DEMO-CLAROTY-PAGINATION-001 MERGED — PR #179 squash-merged develop@9ca7e7d7 2026-06-08**

- **What it delivers:** OffsetLimit POST-body pagination for Claroty. POST fetch steps inject offset+limit into request body (merged into interpolated body_template); GET steps keep URL query params. EC-002 malformed-body → SpecEngineError (no panic, no CWE-209 body-value leak). Gap-CL-004 CLOSED.
- **remove-uncertainty ROI:** v1.2 (C-1..C-5) caught 2 HIGH before TDD — wrong body-injection target (issue_request_with_retry→build_request) + missing offset/page_size plumbing across both build_request call sites (TD-VSDD-060). Concrete ROI for the standing remove-uncertainty-per-story directive.
- **Cascade stats:** LOCAL 3-CLEAN strict (P1 FB-001 MED EC-002-test-gap fixed; P2/P3/P4 CLEAN; BC-5.39.001 D-779). PR-LEVEL 3-CLEAN strict re-cascade (P1 OBS stale-SHA + security SEC-001 MED CWE-209 + adversary caught push-before-regate gap (DRIFT-ORCH-PRLEVEL-PUSH-001 codified); all fixed; re-cascade P1/2/3 CLEAN on pushed head fc8df590). security SECURITY-CLEAR-TO-MERGE (SEC-001 CLOSED + regression-guarded). pr-reviewer APPROVE. CI all green (42 checks).
- **POL-14:** BC-2.16.002+BC-2.16.013+BC-2.01.013 all already lifecycle_status: active — idempotent no-op. No BC-INDEX count change (active=235, draft=2 UNCHANGED).
- **Phase C Lane 3 COMPLETE.**
- **Drift items registered this burst:** DRIFT-ORCH-PRLEVEL-PUSH-001 [process-gap] — PR-LEVEL fix-bursts MUST be pushed to origin/feature before re-running PR-LEVEL cascade (lessons.md appended); DRIFT-D904-002 recurrence noted (recurrence #3); DRIFT-PAGINATION-PAGESIZE-VALIDATION-001 (D-1063) remains open (SEC-002 LOW; spec-engine scope; PO/architect adjudication).

**Also completed (Phase C Lanes 1+2 — for context):**
| Story | PR | SHA | Lane |
|---|---|---|---|
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | #177 | `5c5d240d` | Phase C Lane 1 (ADR-031 §D8-b Gap-CL-001 CLOSED) |
| S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 | #178 | `763e0ade` | Phase C Lane 2 (Gap-CL-006 CLOSED) |

**Also completed this cycle (Phase B):**
| Story | PR | SHA | Lane |
|---|---|---|---|
| S-SPEC-HTTP-METHOD-VALIDATION-001 | #172 | `752e407a` | Phase B Lane 1 |
| S-DEMO-QUERY-PUSHDOWN-001 | #173 | `9447671f` | Phase B Lane 2 |
| OCSF-CLASS-MIGRATION-001 | #174 | `0e89789a` | Phase B Lane 3 |
| S-MAINT-ECRED-TAXONOMY-SYNC-001 | #175 | `c603741d` | S-MAINT |
| S-DEMO-003 | #176 | `a42e3eaf` | Phase B Lane 4 |

---

### 3. Exact Next Steps — Multi-Client SOC Demo (North Star)

> **Wave 5 Phase C is COMPLETE. The pipeline now serves the ACTIVE OBJECTIVE: multi-client SOC-analyst live demo. See §ACTIVE OBJECTIVE above for full build sequence.**

**NEXT CONCRETE ACTION (D-1072, user-directed):**
Dispatch `vsdd-factory:product-owner` to author the S-DEMO-MULTI-TENANT-DTU-001 multi-address-binding BC. The BC must cover: per-instance multi-address binding, no-cross-tenant-leakage invariant, single-instance backward-compat. Resolve OQ-1 (MultiInstanceConfig crate — recommend `prism-dtu-common`), OQ-2 (Box<dyn BehavioralClone> harness API), OQ-3 (canonical BC IDs). Then dispatch architect to adjudicate.

| Priority | Story / Action | Status | Notes |
|----------|---------------|--------|-------|
| **P0 — NEXT** | PO authors S-DEMO-MULTI-TENANT-DTU-001 BC | BLOCKED (OQ-1/OQ-2/OQ-3 unresolved) | Dispatch: `vsdd-factory:product-owner` |
| P1 | Per-instance data seeding gap | needs story | Wire `CloneConfig.seed`/`fixture_set` in `build_clone_pairs`; new story or scope-add to #1 |
| P1 | S-DEMO-004 — add depends_on + data-distinctness AC | draft P0 — needs architect/PO update | Add depends_on edge; decide AC-006 via real seeding |
| P2 | S-DEMO-LAUNCHER-CONSOLIDATION-001 | draft stub | story-writer materialization + human review needed |
| P3 | Multi-client SOC-analyst narrative story | NEW — none exists | Multi-client SOC walkthrough + demo-recorder evidence per persona |
| optional | S-5.02 / S-3.13 / S-5.04 | draft | Capability discovery for narrative |

**Wave 5 Phase C — COMPLETE:**
| Story | PR | SHA | Status |
|---|---|---|---|
| ~~S-DEMO-CLAROTY-TRAILING-SLASH-001~~ | #177 | `5c5d240d` | MERGED (D-1060) |
| ~~S-DEMO-CLAROTY-SPEC-PROSE-FIX-001~~ | #178 | `763e0ade` | MERGED (D-1062) |
| ~~S-DEMO-CLAROTY-PAGINATION-001~~ | #179 | `9ca7e7d7` | MERGED (D-1064) |
| ~~S-DEMO-HARNESS-CLONE-PARITY-001~~ | #180 | `64d34967` | MERGED (D-1071) |

#### Per-Story Delivery Step Ledger

Per-story delivery follows the canonical 12-gate sequence (per orchestrator per-story-delivery reference):
1. `dclaude:remove-uncertainty` (standing directive — ALWAYS first)
2. `vsdd-factory:worktree-manage create <STORY-ID>` (worktree setup)
3. `vsdd-factory:test-writer` — stubs + failing Red Gate tests
4. `vsdd-factory:implementer` — TDD green (one failing test → minimum code → micro-commit)
5. LOCAL adversary 3-CLEAN strict (BC-5.39.001 D-779; CLEAN(strict) = zero findings ANY severity)
6. `vsdd-factory:demo-recorder` per-AC (POL-10)
7. Push feature branch to `origin/feature/<story-id>` — **REQUIRED before PR create**
8. `vsdd-factory:pr-manager` — PR create
9. PR-LEVEL adversary 3-CLEAN strict + `vsdd-factory:pr-reviewer` APPROVE + `vsdd-factory:security-reviewer` CLEAR — **push any fix commits BEFORE re-running PR-LEVEL cascade (DRIFT-ORCH-PRLEVEL-PUSH-001)**
10. CI all green
11. Squash-merge to develop
12. Worktree cleanup + state-manager post-merge burst (POL-14 BC promotions + sprint-state.yaml update)

**Active story pointer:**
- **No story in-flight.** Wave 5 Phase C COMPLETE. North-star next action: PO authors S-DEMO-MULTI-TENANT-DTU-001 BC.

**Mid-cascade restart note:** If a fresh session finds an in-flight worktree/branch/open-PR for a story, cross-reference `sprint-state.yaml` `current_story.delivery_step` + `gh pr list` + `.worktrees/` to determine the exact resume step before dispatching.

> **Previous snapshot (2026-06-08-DURABILITY-HARDENING-D1065; STATE v7.716) remains the base; this section updated in place for D-1071+D-1072+D-1073 completion. Prior snapshots archived to `cycles/wave-5-e-demo-fidelity/session-handoff-archive.md`.**

---

### 4. Standing Authorizations and Rules

**D-989 AUTONOMY GRANT — ACTIVE (granted 2026-06-04)**
- Full autonomous Wave-5 A→B→C execution
- Auto-advance phases + auto-merge to develop ONLY when objective gates met: LOCAL 3-CLEAN strict + PR-LEVEL 3-CLEAN strict + security MAY PROCEED + pr-reviewer APPROVE + all CI PASS
- **PAUSE-AND-SURFACE for 4 hard exceptions (do NOT auto-handle):**
  1. Source-of-Truth §7 spec-to-match-code amendments (only human authorizes)
  2. Genuine product/business decision not derivable from existing specs/ADRs
  3. Level-3 escalation: missing prerequisite, genuinely-red CI, convergence not reached after reasonable retries
  4. CLAUDE.md edits (human-only per Pipeline Authority)

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

# 2. Verify develop HEAD == 64d34967
git log --oneline develop | head -1
# Expected: 64d34967 ...

# 3. Verify STATE.md version
grep '^version:' .factory/STATE.md
# Expected: version: "7.724"

# 4. Verify no open PRs
gh pr list --state open
# Expected: (empty)

# 5. Confirm factory-artifacts is in sync with remote (user-authorized push policy D-1066)
git -C .factory rev-parse HEAD && git -C .factory rev-parse origin/factory-artifacts
# Expected: both SHAs match (HEAD == origin/factory-artifacts). If they differ, run:
#   git -C .factory push origin factory-artifacts

# 6. Read task ledger → CURRENT TASK
cat .factory/objectives/multi-client-soc-demo-tasks.md | grep -A3 'CURRENT POINTER'
# Expected: T1 — PO authors multi-address-binding BC for S-DEMO-MULTI-TENANT-DTU-001

# 7. Read this snapshot (you are here)
# Confirm develop_head, STATE version, north-star next action

# 8. Confirm active story + delivery step
grep -A4 '^current_story:' .factory/stories/sprint-state.yaml
# Expected: no story in-flight (Phase C COMPLETE)
# (If delivery_step != not-started, check gh pr list + .worktrees/ to find resume point)
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
