---
document_type: task-ledger
objective: multi-client-soc-analyst-demo
level: ops
version: "1.8"
producer: state-manager
status: active
timestamp: 2026-06-09T12:00:00Z
related:
  - SESSION-HANDOFF.md §ACTIVE OBJECTIVE
  - .factory/STATE.md
---

# Task Ledger — Multi-Client SOC-Analyst Live Demo

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

Foundations: COMPLETE (reused). Build: 3/15 tasks done; T4 DONE; T4-A **VALIDATED + DELIVERY-READY** (D-1080: dclaude:remove-uncertainty on Story A CONFIRMED SOUND; ADR-036 v2.1; Story A v1.1; all U-A-01..U-A-10 corrections applied). **CURRENT TASK: T4-A — deliver Story A via 12-gate per-story TDD sequence NEXT.**

## CURRENT POINTER

**T4-A: S-DEMO-DTU-LIVE-SCENARIO-001-A VALIDATED + DELIVERY-READY (D-1080).** dclaude:remove-uncertainty re-validation CONFIRMED SOUND — ADR-036 v2.0 substrate design is correct; all mechanism/wiring corrections applied (U-A-01..U-A-10). ADR-036 v2.1. Story A v1.1. **NEXT: deliver Story A via 12-gate per-story TDD sequence. Story B after A merges.**

## NEXT ACTION (verbatim, for cold resume)

Deliver Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A, ready v1.1, 8pt, BC-2.06.018) via the 12-gate per-story TDD sequence: vsdd-factory:worktree-manage create → test-writer (14 Red Gate tests, FAIL-first) → implementer TDD across 8 crates (prism-dtu-common scenario stub + per-clone new_with_seed + dual-path routes + build_clone_pairs seed/org_id + Cargo/feature/ci.yml) → LOCAL adversary 3-CLEAN strict (BC-5.39.001) → demo-recorder → push origin/feature → pr-manager PR → PR-LEVEL 3-CLEAN strict + pr-reviewer APPROVE + security CLEAR → CI green → squash-merge → state-manager post-merge (POL-14 BC-2.06.018 draft→active). Then Story B (progression+enrichment) once A merges.

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
| T4-A | not-started | per-story delivery | T4 | S-DEMO-DTU-LIVE-SCENARIO-001-A | Story A (baseline seeding retrofit; 8pt; BC-2.06.018; v1.1 VALIDATED D-1080) implemented + merged via 12-gate per-story sequence. **NEXT UNBLOCKED DELIVERY — remove-uncertainty COMPLETE (D-1080); proceed directly to 12-gate TDD.** |
| T5 | blocked | per-story delivery + story-writer | T4-A | S-DEMO-DTU-LIVE-SCENARIO-001-B | Story B (scenario progression + enrichment correlation; 7pt; BC-2.06.019+020; draft v1.0) finalized to ready after Story A merges (story-writer materializes full implementation spec from draft shell); remove-uncertainty applied; then 12-gate delivery. |
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
| 1.8 | 2026-06-09 | state-manager | D-1081: Zero-context resume durability hardening (user-directed). No task-status changes — bookkeeping only. sprint-state.yaml current_story fixed to point at T4-A Story A. SESSION-HANDOFF snapshot refreshed + §7 checklist expected values corrected. Coherence sweep confirmed ledger agrees with STATE v7.732/develop 64d34967/BC counts 250/235/6/total_stories 188. Ledger version 1.7→1.8. |
| 1.7 | 2026-06-09 | state-manager | D-1080: Story A (S-DEMO-DTU-LIVE-SCENARIO-001-A) re-validation via dclaude:remove-uncertainty CONFIRMED SOUND. ADR-036 v2.0 substrate design correct; all mechanism/wiring corrections applied (U-A-01..U-A-10): gen_seeded_rng symbol; CrowdStrike load_host_ids()/load_host_details() fallback; GenOpts::default() syntax; demo-server Cargo.toml deps; Armis fallible; non-exhaustive-violation crate dep; per-clone generate() divergence. ADR-036 v2.0→v2.1 (architect). Story A v1.0→v1.1 (story-writer). DRIFT-SLUG-FORMAT-BC34004-001 registered (non-blocking). T4-A status: validated + delivery-ready (remove-uncertainty COMPLETE). CURRENT POINTER updated: T4-A 12-gate TDD NEXT. NEXT ACTION updated verbatim. ARCH-INDEX v2.117→v2.118. STORY-INDEX v2.331→v2.332. STATE v7.730→v7.731. |
| 1.6 | 2026-06-09 | state-manager | D-1079: T4 RECONCILED+COMPLETE. ADR-036 v2.0 substrate reconciliation complete (architect; two-phase retrofit: new_with_seed + generated_records + dual-path routes; canonical org_slug=hex(org_id[0..4]); device ID dev-{8hex}-{seed}-{n}; CloneConfig.org_id; E-DEMO-004/005). BC-2.06.018/019/020 v1.0→v1.1 (PO; substrate reality corrections). error-taxonomy v1.63→v1.64 (E-DEMO-004+005 added). Story split materialized (story-writer; user-authorized): original S-DEMO-DTU-LIVE-SCENARIO-001 SUPERSEDED → Story A (001-A; 8pt; ready; BC-2.06.018; 14 ACs; blocks 001-B) + Story B (001-B; 7pt; draft; BC-2.06.019/020; 16 ACs; depends_on A). T4 status: in-progress→done. T4-A row added (Story A delivery; not-started; NEXT UNBLOCKED). T5 updated (Story B; blocked on T4-A). CURRENT POINTER advanced to T4-A. NEXT ACTION updated verbatim. STORY-INDEX v2.330→v2.331. BC-INDEX v6.04→v6.05. ARCH-INDEX v2.116→v2.117. total_stories 188. Progress: 3+T4/15 done; Story A delivery NEXT. |
| 1.5 | 2026-06-09 | state-manager | D-1078: T4 design substantially complete. ADR-036 confirmed in ARCH-INDEX v2.116. BC-2.06.019 (scenario progression, 5 invariants) + BC-2.06.020 (enrichment correlation, 6 invariants) registered in BC-INDEX v6.04. E-DEMO-001/002/003 confirmed in error-taxonomy v1.63. T4 CURRENT POINTER remains (story-writer assembles S-DEMO-DTU-LIVE-SCENARIO-001 retry pending — 2 transient socket drops). NEXT ACTION updated verbatim. T4 done-when updated. Progress: 3/15 done; design artifacts durable. |
| 1.4 | 2026-06-09 | state-manager | D-1077: User-directed scope expansion of multi-client SOC demo (A/B/C). BC-2.06.018 registered (draft; PO-authored; SS-01; CAP-036; P2). Scope Expansion section added. T4 status not-started→in-progress; T4 done-when updated to reflect EXPANDED scope (seeding DECIDED; progression + enrichment PENDING architect design). T5 done-when updated to single-larger-live-scenario-story. NEXT ACTION updated verbatim. E-DEMO-001 obligation recorded. CURRENT POINTER remains T4. Progress: 3/15 done. |
| 1.3 | 2026-06-09 | state-manager | D-1076: T3 done (story-writer finalized S-DEMO-MULTI-TENANT-DTU-001 to status:ready v1.2; dclaude:remove-uncertainty closed 8 uncertainties (4 HIGH incl CRIT U-002) before TDD; BC-2.06.017 v1.1 (2 PO amendments); architect reconciliation extend-D-1075 no-ADR; S-7.01 gate CLEARED); CURRENT POINTER advanced to T4 (product-owner + architect decide per-client data seeding approach). Progress: 3/15 done. |
| 1.2 | 2026-06-09 | state-manager | D-1075: T2 done (architect adjudication OQ-1/OQ-2/OQ-3 complete; no ADR); CURRENT POINTER advanced to T3 (story-writer finalizes S-DEMO-MULTI-TENANT-DTU-001 to status:ready). |
| 1.1 | 2026-06-09 | state-manager | D-1074: T1 done (BC-2.06.017 authored + registered in BC-INDEX v6.01); CURRENT POINTER advanced to T2 (architect adjudicates OQ-1/OQ-2/OQ-3). |
| 1.0 | 2026-06-09 | state-manager | Initial task ledger created (D-1073). |
