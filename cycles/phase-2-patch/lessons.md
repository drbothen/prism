# Lessons Learned — phase-2-patch

## Extracted from STATE.md on 2026-04-19

---

## Patch Cycle Retrospective — Durable Lessons

Durable lessons from Phase 2 patch cycle for future VSDD factory runs:

### Agent-level

1. **Version-race pattern** — state-manager must run LAST in every burst. This caused regressions in pass 4/5/6 before being recognized as a pattern.
2. **Path-prefix doubling** — brief agents with `ls <dest-dir>` verification before first write. story-writer created `.factory/stories/stories/` in Burst 1.
3. **Context overflow** — bursts writing >8 new artifacts should split "create" and "integrate" sub-bursts.
4. **Retroactive anchor propagation** — new BCs must immediately anchor back to implementing stories in the SAME burst.

### Process-level

5. **Previously-converged does not equal correct** — 50-pass-converged Phase 3 had 19 gaps. Mandate fresh-context consistency audit at every phase-gate.
6. **DTU assessment must cover ALL external integrations** at Phase 1 — sensors, actions, infusions, log-forwarding, ingestion. Don't discover scope mid-patch.
7. **BC retirement depth** — retiring a BC touches ~5 artifacts (index removed section, matrix, story frontmatter, AC prose, replacement's Related BCs).
8. **Trajectory monotonicity is a quality signal** — if findings count increases pass-over-pass, investigate root cause before proceeding.
9. **Duplication creates drift** — STATE.md Wave Summary duplicates STORY-INDEX. Every duplicate is a drift opportunity. Establish ONE source-of-truth per metric.
10. **Semantic anchoring integrity** — the lesson that kicked off this retrospective. Mis-anchors hide behind syntactically-valid references. PRD §7 had CAP titles RENAMED to cover mis-anchors (Pass 12 P3P12-A4-001).

### Infrastructure-level

11. **Agent commit permission friction** — specialist agents lack Bash; orchestrator commits all their work. Cost minutes every burst.
12. **User-as-senior-architect catches things adversary does not** — CI/CD gap, DTU scope, taxonomy consistency, CAP-020 mis-anchor. Structure orchestrator to present "questions for human review" at every gate.
13. **Fresh-context review compounds** — adversary passes 7-12 all surfaced new real findings, not just refinements.
14. **ARCH-INDEX is authoritative for subsystem names.** BC-INDEX subsystem labels and BC file frontmatter `subsystem:` fields must match ARCH-INDEX canonical names. Pass 13 P3P13-A2-004 exposed 7-subsystem taxonomy drift; Burst 14 canonicalized via single-direction sync (ARCH wins).
15. **Rename sweeps must scan aggregation and derivation docs.** Burst 14's SS-NN rename updated BC-INDEX + 208 BC frontmatters + PRD §7 but missed SUBSYSTEMS-*-SUMMARY.md (3 files), PRD §5 BC inventory, and downstream story tables.
16. **Retirement is a transitive event.** When retiring a BC, also: (a) strikethrough in all aggregation docs, (b) update current-active count headers, (c) remove from invariant-enforcer tables, (d) sync any stale pre-retirement title to current BC file title.
17. **Completeness is an anchor-integrity axis.** Active-BC row presence in aggregation docs is itself an anchor; a missing row is semantic drift, not just cosmetic.
18. **LOW observations on anchor-like claims are actually MED.** The `semantic_anchoring_integrity` policy covers any claim-like structure (invariant-to-BC enforcer lists, BC-to-Story traceability, BC-to-CAP anchors), not just BC frontmatter.
19. **BC file H1 is authoritative for BC titles.** Policy-relevant enrichment that appears in downstream indexes must be moved into the BC H1 rather than left as index-only context. Two BCs in Burst 19 had outright H1↔index contradictions resolved by BC body reading (BC-2.09.004, BC-2.02.008) — in both cases, BC H1 was correct.
20. **Each adversarial scope expansion surfaces next-layer drift.** Trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 shows alternating decay/uptick. Convergence requires closing all axes until adversary genuinely finds nothing.
21. **User decisions unblock convergence.** When multiple semantic-equivalent options exist, auto-adjudication may thrash. Surface to user with pros/cons, let them decide, commit. Burst 21 illustrated: 3 BCs in ambiguous retired-but-active state for 3+ passes; user chose Option A (un-retire) in one interaction; Burst 21 then closed 12 findings in one burst.
22. **Frontmatter IS an anchor claim; body IS the commitment.** When frontmatter lists IDs (story `bcs:`, story `vps:`, BC `capability:`, BC Traceability `Story`), a derivation exists between frontmatter and body. New policy `bc_array_changes_propagate_to_body_and_acs` formalizes the propagation rule.
23. **Policy adoption retroactively elevates pre-existing drift.** Each policy adoption should trigger a one-shot corpus-wide sweep for compliance with the new rule, not just forward enforcement.
24. **VP-layer coherence is a distinct drift class.** VP-INDEX can evolve (new VPs added, module assignments corrected) without propagating to verification-architecture.md and verification-coverage-matrix.md. Policy 9 (`vp_index_is_vp_catalog_source_of_truth`) formalizes VP-INDEX as the single source of truth.
