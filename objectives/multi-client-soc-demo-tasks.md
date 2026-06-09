---
document_type: task-ledger
objective: multi-client-soc-analyst-demo
level: ops
version: "1.3"
producer: state-manager
status: active
timestamp: 2026-06-09T05:00:00Z
related:
  - SESSION-HANDOFF.md §ACTIVE OBJECTIVE
  - .factory/STATE.md
---

# Task Ledger — Multi-Client SOC-Analyst Live Demo

## Objective

Deliver a multi-client SOC-analyst LIVE DEMO — multiple clients, different sensor combos, REAL per-client data, prism federation, MCP→Claude, end-to-end SOC investigation. TDE deferred.

## Scope Decisions (user, 2026-06-09)

- SOC-analyst demo FIRST. TDE workflow (detection rules, write/action-back containment) DEFERRED — requires `prism-operations` crate + dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).
- REAL per-client data segregation required — NOT just client-targeting/federation routing.

## Progress Summary

Foundations: COMPLETE (reused). Build: 3/15 tasks done; CURRENT TASK **T4**.

## CURRENT POINTER

**T4** — Product-owner + architect decide the per-client data seeding approach for the multi-client SOC demo.

## NEXT ACTION (verbatim, for cold resume)

Dispatch `vsdd-factory:product-owner` + `vsdd-factory:architect` to decide the per-client data seeding approach for the multi-client SOC demo: today `build_clone_pairs` (`crates/prism-dtu-demo-server/src/harness.rs`) ignores `CloneConfig.seed`/`fixture_set` so every client serves identical data. Decide: wire per-instance seeds into `build_clone_pairs`, OR a POST /dtu/configure runtime-seeding path. Author or scope-add a story delivering REAL distinct per-client data (closes the real-per-client-data requirement). This is the data-distinctness backbone for S-DEMO-004 (T8) and the SOC narrative (T13).

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
| T4 | not-started | product-owner + architect | — | (per-client data seeding — NEW or scope-add) | Seeding approach decided (wire CloneConfig.seed/fixture_set in build_clone_pairs vs POST /dtu/configure); story authored/scoped to deliver REAL distinct per-client data. |
| T5 | blocked | story-writer | T4 | (data-seeding story) | Data-seeding story finalized to ready; remove-uncertainty applied. |
| T6 | blocked | orchestrator-driven per-story delivery | T3 | S-DEMO-MULTI-TENANT-DTU-001 | Multi-instance bind implemented + merged (test-writer → implementer → LOCAL 3-CLEAN strict → demo → PR → PR-LEVEL 3-CLEAN strict → merge). |
| T7 | blocked | per-story delivery | T5, T6 | (data-seeding story) | Per-client data seeding implemented + merged; org A != org B data verified. |
| T8 | not-started | architect + product-owner | — | S-DEMO-004 | S-DEMO-004 reconciled: add depends_on S-DEMO-MULTI-TENANT-DTU-001 (missing edge); AC-006 data-distinctness committed to REAL seeding (not OQ-2 port-binding-only). |
| T9 | blocked | story-writer | T8 | S-DEMO-004 | S-DEMO-004 finalized to ready; remove-uncertainty applied. |
| T10 | blocked | per-story delivery | T6, T7, T9 | S-DEMO-004 | 3-org x mixed-sensor isolation smoke test implemented + merged (P0 proof; per-tenant data verified). |
| T11 | not-started | story-writer | — | S-DEMO-LAUNCHER-CONSOLIDATION-001 + demo scripts | Multi-org demo-script story materialized (generalize demo-setup/demo-run/demo-teardown to N orgs; launcher reconcile) to ready; remove-uncertainty applied. |
| T12 | blocked | per-story delivery | T6, T7, T11 | (demo scripts story) | Multi-org demo scripts implemented + merged; `demo-run.sh` stands up N clients with distinct sensors+data. |
| T13 | not-started | product-owner + story-writer | — | (NEW narrative story) | New story authored: multi-client SOC-analyst investigation walkthrough (the demo storyline) to ready. |
| T14 | blocked | demo-recorder + technical-writer | T10, T12, T13 | (narrative story) | SOC investigation walkthrough recorded as demo evidence; DEMO-RUNBOOK.md updated for multi-client; per-persona evidence captured. |
| T15 | optional, not-started | story-writer + per-story delivery | (as needed) | S-5.02 / S-3.13 / S-5.04 | Capability-discovery surface (MCP client targeting / dynamic per-org table availability / sensor health) if the narrative needs "show this client's available sensors/tables." |

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
| 1.3 | 2026-06-09 | state-manager | D-1076: T3 done (story-writer finalized S-DEMO-MULTI-TENANT-DTU-001 to status:ready v1.2; dclaude:remove-uncertainty closed 8 uncertainties (4 HIGH incl CRIT U-002) before TDD; BC-2.06.017 v1.1 (2 PO amendments); architect reconciliation extend-D-1075 no-ADR; S-7.01 gate CLEARED); CURRENT POINTER advanced to T4 (product-owner + architect decide per-client data seeding approach). Progress: 3/15 done. |
| 1.2 | 2026-06-09 | state-manager | D-1075: T2 done (architect adjudication OQ-1/OQ-2/OQ-3 complete; no ADR); CURRENT POINTER advanced to T3 (story-writer finalizes S-DEMO-MULTI-TENANT-DTU-001 to status:ready). |
| 1.1 | 2026-06-09 | state-manager | D-1074: T1 done (BC-2.06.017 authored + registered in BC-INDEX v6.01); CURRENT POINTER advanced to T2 (architect adjudicates OQ-1/OQ-2/OQ-3). |
| 1.0 | 2026-06-09 | state-manager | Initial task ledger created (D-1073). |
