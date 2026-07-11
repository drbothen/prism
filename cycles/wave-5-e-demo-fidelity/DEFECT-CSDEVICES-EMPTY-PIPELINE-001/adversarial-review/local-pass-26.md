---
document_type: adversarial-review
scope: LOCAL
fix_defect: DEFECT-CSDEVICES-EMPTY-PIPELINE-001
passes: [26]
feature_head_at_review: 99719a7a
date: 2026-07-11
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 3
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: adversary
---

# LOCAL Adversary Pass 26 — DEFECT-CSDEVICES-EMPTY-PIPELINE-001

---

## Pass 26 (frozen 99719a7a; fresh-context adversary; LOCAL cascade; policy rubric + SAP-1 + SAP-2 + POL-22; streak candidate 1/3 — NOT ADVANCING — stays 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 3 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 3 OBS / 0 PROCESS-GAP)

**Adversary novelty assessment:** LOW — no new structural classes; all three findings are doc-prose sync, architectural durability (routed to architect), and a pre-existing SAP-2 TOML surface gap correctly anchored to existing deferral.

**SAP-1:** PASS — Catalog rows 179–183 verified against all `event_type =` emission sites; no new emissions without a BC-2.16.002 catalog row; catalog at v2.10 complete and unchanged.

**SAP-2:** PASS — No new CRITICAL findings; all 6 TOML-declared devices columns map to DTU fields. Pre-existing surface gap (os_version, containment_status, external_ip, local_ip, agent_version, standalone-only cid/agent_id) correctly anchored to DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (human-directed EXPAND-NOW queued, D-1666 decision 4).

**POL-22:** PASS — All cited entities resolve; no dangling cross-references.

**POL-33:** PASS — Route Coverage Table (BC-2.16.013 v1.28, 9 rows) verified.

**STREAK:** 0/3 — CLEAN(strict)=NO on frozen 99719a7a (3 OBS findings; OBS-002 architect adjudication dispatched). Fix commits @3202d80f (test-writer) + @9fe2d016 (implementer) close OBS-001 and OBS-002 doc items. New frozen HEAD for pass 27: **9fe2d016** (streak RESET → 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001 — any commit to branch resets streak).

**Code HEAD at review:** 99719a7a (frozen; just check FULL WORKSPACE 5476/5476 GREEN, 60 skipped; non-exhaustive 89/89; LOCAL-ONLY; develop baseline UNCHANGED @b9cf3f9b)

**CLEAN(strict):** NO — 3 OBS findings

**CLEAN(PR-merge):** YES — zero CRIT / HIGH / MED findings

---

## Findings

### F-CSD-P26-OBS-001 — `reference_content.rs` "3-tier" gate docstring stale — catalog now 4 tiers (LOW novelty; OBS)

**Finding:** The `build_reference_content` function in `crates/prism-mcp/src/resources.rs` contained an inline docstring and exhaustiveness-stub comment describing the E-QUERY-043 example gate as a "3-tier" construction (PositiveE043 / NegativeE043 / generic-fallback). The `ExampleKind::NegativeE043` variant added at pass-25 (@99719a7a fix-burst) extended the gate to 4 tiers, but the "3-tier" prose was not updated in the same commit. Additionally, the exhaustiveness-stub match arm in `negative_e043_parity_gate.rs` extended at pass-25 did not carry a comment explaining the new NegativeE043 arm's purpose relative to the 4-tier design.

**Status:** CLOSED

**Fix:** test-writer @3202d80f — `test_bc_2_11_022_ci_3tier_gate` renamed → `test_bc_2_11_022_ci_4tier_gate`; `has_negative_e043` assertion added to the gate test; exhaustiveness stub in `non-exhaustive-violation` crate extended with `ExampleKind::NegativeE043` match arm. TD-VSDD-060 cross-ref sibling sweep applied: `negative_e043_parity_gate.rs` adjacent match arm comments swept to reflect 4-tier design. implementer @9fe2d016 — "3-tier" stale prose replaced with "4-tier" design note in `build_reference_content` doc-annotation; emitter boundary comment verified accurate.

---

### F-CSD-P26-OBS-002 — `check_expr_insubquery_projection` `Ast::Pipe` vs `Ast::SqlPipe` stage-walk asymmetry — durability question (LOW novelty; OBS)

**Finding:** The E-QUERY-043 gate in `check_expr_insubquery_projection` dispatches `Ast::SqlPipe(spq) => { check_sql_query(&spq.head); walk spq.stages via PipeStage::Where → check_predicate }` (the pass-25 fix @99719a7a) but has no analogous `Ast::Pipe` arm that walks pipe stages. The adversary surfaced a durability question: if `Ast::Pipe` stage predicates could ever embed an `Expr::InSubquery` atom, the lack of an equivalent walk in the `Ast::Pipe` arm would constitute a gate gap.

**Routing:** Architect adjudication dispatched (Option B, 2026-07-11).

**Architect adjudication (Option B, 2026-07-11):** Asymmetry STANDS; T39 lock UNCHANGED. Key empirical correction to adversary claim: adversary stated that `Ast::Pipe`-mode `InSubquery` would reach DataFusion. This is INCORRECT. `pipe_sql_emitter::predicate_to_datafusion_sql` contains an explicit `Predicate::InSubquery => Err(QueryExecutionFailed("not yet supported"))` arm that BLOCKS before DataFusion — so `Ast::Pipe`-mode InSubquery is already blocked at the emitter layer before any DataFusion execution can occur. `Ast::SqlPipe` LACKS this emitter block, which is WHY the stage-walk added at pass-25 is necessary (the SQL emitter for SqlPipe does lower predicates to DataFusion SQL). E-QUERY-043 is semantically scoped to SQL-planning-reachable projection positions; flipping T39's assertion would produce a MISLEADING hint. The asymmetry is correct by design. Two-step condition documented for future gate extension: (1) grammar exposes `Predicate::InSubquery` in pipe stages AND (2) emitter lowers it. Neither condition is currently met. Doc-annotation-only work items executed by implementer.

**Status:** CLOSED — ARCHITECT ADJUDICATION (Option B, 2026-07-11)

**Fix (doc items):** test-writer @3202d80f — T39 `Pipe` sub-assertion comment updated with emitter-boundary rationale (why `Ast::Pipe` is exempt: `pipe_sql_emitter` has explicit `Predicate::InSubquery => Err` arm). implementer @9fe2d016 — `materialization.rs` wildcard-arm comment replaced with architect-ratified rationale (citing function-name-only per TD-VSDD-091: `pipe_sql_emitter::predicate_to_datafusion_sql` Err arm blocks InSubquery before DataFusion; SqlPipe stage-walk added pass-25 because SqlPipe emitter lowers predicates); emitter claim re-verified at `crates/prism-query/src/pipe_sql_emitter.rs`. just check FULL WORKSPACE GREEN; non-exhaustive 89/89.

---

### F-CSD-P26-OBS-003 — SAP-2 devices-table DTU-fields-not-in-TOML coverage gap — pre-existing; closed-by-existing-deferral (LOW novelty; OBS)

**Finding:** SAP-2 probe on the devices TOML spec identified that 6 DTU fields (os_version, containment_status, external_ip, local_ip, agent_version, and standalone-only cid/agent_id) are present in the `prism-dtu-crowdstrike` response struct but absent from `crowdstrike.sensor.toml`'s `[[tables]]` devices column list. Under SAP-2 classification rules this is a MEDIUM (field in DTU with no TOML column — missing coverage, not a runtime crash).

**Pre-existing determination:** This gap is NOT introduced by the CSDEVICES fix branch. It predates this branch. It was first surfaced at pass-14 as F-CSD-P14-010 (SAP-2 §4-class PENDING-HUMAN), formally registered as DRIFT-SAP2-DEVICES-TOML-SURFACE-001, and received a human-authorized EXPAND-NOW deferral with specific anchor (D-1666 decision 4: columns os_version, agent_version, external_ip, local_ip, containment_status queued after CSDEVICES merge).

**Status:** CLOSED-BY-EXISTING-DEFERRAL (DRIFT-SAP2-DEVICES-TOML-SURFACE-001; D-1666 decision 4; human-authorized; queued post-CSDEVICES-merge)

**Deferral linkage:** DRIFT-SAP2-DEVICES-TOML-SURFACE-001 — human-directed EXPAND-NOW after CSDEVICES branch merges; STATE.md §PENDING USER-APPROVED WORK item 1; SESSION-HANDOFF.md §RESUME SNAPSHOT D-1674 item 1. No new work required at this pass.

---

## Fix-Burst Summary

| Agent | Commit | Change |
|-------|--------|--------|
| test-writer | @3202d80f | `test_bc_2_11_022_ci_3tier_gate` → `test_bc_2_11_022_ci_4tier_gate`; `has_negative_e043` assertion added; exhaustiveness-stub extended with `ExampleKind::NegativeE043` match arm in non-exhaustive-violation; T39 emitter-boundary comment updated with architect-ratified `pipe_sql_emitter::predicate_to_datafusion_sql` Err-arm rationale (F-CSD-P26-OBS-001 + F-CSD-P26-OBS-002 doc items) |
| implementer | @9fe2d016 | `build_reference_content` "3-tier" stale prose → "4-tier" design note; `materialization.rs` wildcard-arm comment replaced with architect-ratified emitter-boundary rationale citing `pipe_sql_emitter::predicate_to_datafusion_sql` (function-name-only per TD-VSDD-091); emitter claim re-verified; just check FULL WORKSPACE GREEN; non-exhaustive 89/89 (F-CSD-P26-OBS-001 + F-CSD-P26-OBS-002 doc items); OBS-003 CLOSED-BY-EXISTING-DEFERRAL (no code change) |

**just check FULL WORKSPACE:** 5476 @9fe2d016, just check GREEN (renames only; no new tests this burst)
**Non-exhaustive gate:** 89/89 UNCHANGED
**SAP-1:** PASS — zero new `event_type =` emissions introduced
**SAP-2:** PASS — no new TOML↔DTU divergence; pre-existing gap correctly anchored to DRIFT-SAP2-DEVICES-TOML-SURFACE-001

---

## Architect Adjudication Memo — OBS-002 (2026-07-11)

**Decision ID:** D-1675

**Question posed:** Is the `Ast::Pipe` vs `Ast::SqlPipe` stage-walk asymmetry in `check_expr_insubquery_projection` a gate gap, or is it correct by design?

**Key facts established:**
1. `pipe_sql_emitter::predicate_to_datafusion_sql` has an explicit `Predicate::InSubquery => Err(QueryExecutionFailed("not yet supported"))` arm.
2. This arm blocks `Ast::Pipe`-mode `Predicate::InSubquery` BEFORE it reaches DataFusion. No DataFusion execution occurs.
3. `Ast::SqlPipe` uses a different emitter path that DOES lower predicates to DataFusion SQL; therefore the pass-25 stage-walk (`spq.stages` via `PipeStage::Where → check_predicate`) is necessary and correct.
4. E-QUERY-043 semantic scope: SQL-planning-reachable projection positions only.

**Decision (Option B):** Asymmetry STANDS. The `Ast::Pipe` arm requires no stage-walk because the emitter blocks before DataFusion. T39 lock assertion is correct and must NOT be flipped. Future gate extension requires BOTH: (a) grammar exposing `Predicate::InSubquery` in pipe stages AND (b) emitter lowering it to DataFusion SQL. Neither condition is met; gate is coherent.

**Rationale for T39 stability:** Flipping T39 to assert E-QUERY-043 on `Ast::Pipe` subqueries would produce a misleading error code: the block occurs at the emitter layer, not the plan-time gate layer. The gate's purpose is plan-time rejection of SQL-planning-reachable positions; emitter-layer rejection is a separate enforcement mechanism.

---

## New FROZEN HEAD for Pass 27

**9fe2d016** (LOCAL-ONLY; fix/csdevices-empty-pipeline; streak RESET → 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001)

Cascade now: 26 passes. Pass 27 NEXT on frozen 9fe2d016.
