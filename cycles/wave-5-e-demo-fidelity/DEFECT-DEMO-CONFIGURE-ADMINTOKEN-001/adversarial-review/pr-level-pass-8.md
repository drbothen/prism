<!-- canonical pass-8, adversary=vsdd-factory:adversary fresh-context, frozen HEAD 828449de, 2026-07-17, 1 MED — streak 0/3 -->

# Adversarial Review — PR #225 PR-LEVEL Pass 8 (DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001)
**Finding count by severity:** CRIT 0 · HIGH 0 · MED 1 · LOW 0 · OBS 0 · PROCESS-GAP 0

## Verification statement
Frozen HEAD 828449de. Read: story v0.16 (full), PR body, evidence file, STORY-INDEX v2.703 (row L828 + registry rows), BC-INDEX rows, ARCH-INDEX lines 154/175, BC-3.6.001 v0.8, BC-2.06.017 v1.12, error-taxonomy v2.54 (§DEMO block + changelog), ADR-003 Amendment #5, as-built code. Known-accepted items 1–7 confirmed and excluded.

## F-ADMTOK-PR8-MED-001 — E-DEMO namespace preamble universally asserts "construction-time only / never at request-handling time," contradicted by E-DEMO-007 (runtime error) on all three sub-claims
- Severity MEDIUM, confidence HIGH. Artifact: error-taxonomy.md §DEMO preamble (lines 600–605) vs E-DEMO-007 row (line 615). Anchor: story AC-003 §Error Taxonomy Addition + T-11.
- Preamble: "All E-DEMO-NNN codes are construction-time errors that propagate through build_clone_pairs -> anyhow::Result<Vec<ClonePair>> and abort harness startup. They are never emitted at request-handling time (per INV-CONSTRUCTION-TIME-FAILURE-001 in BC-2.06.018 and INV-CONSTRUCTION-TIME-INJECTION-001 in BC-2.06.020)."
- E-DEMO-007 violates all three: (1) runtime error in cmd_configure (row's own Description; produced by resolve_configure_token multi_org_cmd.rs:1023–1029); (2) flows through resolve_configure_token -> anyhow::Result<String>, not build_clone_pairs; (3) emitted at configure-command time, not startup abort.
- Preamble predates E-DEMO-007 (written when 001..006 were all construction-time). Correct in-scope action: amend preamble to carve out E-DEMO-007 as the sole runtime code.
- Routing: product-owner (owns error-taxonomy.md). Taxonomy v2.54→v2.55. In-scope per AC-003/T-11.

## Version-Pin Lattice Audit
All CONSISTENT: story v0.16 == STORY-INDEX L828 (v2.703 records the sync) == PR body; BC pins v0.8/v1.12 match files + BC-INDEX; taxonomy v2.54; E-DEMO-007 template verbatim (POL-24). F-ADMTOK-PR7-MED-001 confirmed CLOSED.

## Anchor Verification (registry text)
SS-01 anchor CORRECT (ARCH-INDEX:154 lists prism-dtu-demo-server); SS-22 negative claim confirmed (:175). BC H1s match BC-INDEX + story tables; SS-01 subsystem matches; lifecycle_status accurate. No mis-anchoring.

## PR-Description Verification
Story-version 0.16 current; BC pins match; diff-stat matches; sweep totals self-consistent (65KB grep-output substitution honestly noted); convergence table consistent with frozen-HEAD rule. No overstated claims.

## POL-22 A/C
Phase A: ADR-003 Amendment #5 §Decision (story L107–111 == ADR L628–632) and §Implementation item 4 (L116–118 == L664–666) verbatim. Phase C: all entities resolve; no phantom anchors.

## SAP-1 / SAP-2
SAP-1 PASS (0 event_type matches on diff surface; no new emission). SAP-2 N/A (no sensor TOMLs).

## CI Status
All 44 checks pass; all 5 runs success at 828449de.

## Novelty Assessment
MEDIUM — new class (taxonomy preamble-vs-runtime-row contradiction) that survived LOCAL 3-CLEAN because those passes never re-read the section preamble against the newly-inserted row. All other axes clean; artifact otherwise near convergence.

## Dual Verdict
CLEAN (strict): no
CLEAN (PR-merge): no
