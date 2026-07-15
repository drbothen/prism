---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [1]
feature_head_at_review: 973aedcf
date: 2026-07-15
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 1 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 1 (frozen 973aedcf; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

Streak: **1/3** (BC-5.39.001: one CLEAN(strict) pass on frozen HEAD 973aedcf; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; HEAD unchanged since push; pass 2 next)

Cascade tally: **1 pass / 0 fix-bursts**

---

## Findings

None in-perimeter. Zero findings of any severity class (CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP) within the PR scope.

### Out-of-Scope Deferred Item (carried from LOCAL cascade; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade; not re-raised as a PR-LEVEL finding.

---

## Positive Verifications

- **Spec version alignment verified:**
  - BC-2.11.019 v1.20 (injection-safety; `§Injection-safety` load-bearing test at `test_enrich_udf_not_found_display.rs:162` asserting U+0085/U+2028 stripped; rides THIS branch per cross-branch note)
  - BC-2.11.004 v1.47 (EC-11-082 renumber structural; 3 live BC sites; BC-2.11.005 KEEPER sole live EC-11-013)
  - BC-2.11.005 v1.13 (KEEPER; EC-11-013 sole live citation)
  - ADR-048 v1.15 (seven-position walker §D.7.1–D.7.7; 13 live pins current)
  - ADR-052 v1.15 (current)
  - error-taxonomy v2.52 (current)
  - STORY-INDEX v2.686 (current)
  - BC-INDEX v8.25 (current; verbatim RESERVED_KEYWORDS at 3 sites grep-clean of fabricated residue)

- **Seven-position walker load-bearing (engine.rs 1930–2107):** positions 1–7 (SELECT, WHERE, GROUP BY, HAVING exempt-intentional, ORDER BY, LIMIT, DML WHERE) enumerated with boundary locks; DML match arm wildcard removed; HAVING intentionally exempt (aggregate-context gate separate from column-existence gate).

- **DATAFUSION_BUILTIN_AGGREGATE_NAMES gate load-bearing (engine.rs 2109–2134):** runs unconditionally; EC-11-082-compliant message; aggregate names set validated.

- **LOW-006 gate load-bearing (filter_parser.rs 1492–1508):** byte-exact E-QUERY-001 template; function-call-in-WHERE rejection enforced at parse layer.

- **Injection-safety load-bearing (error.rs 83–93 + test_enrich_udf_not_found_display.rs:162):** U+0085/U+2028/Cc stripping asserted via load-bearing test.

- **EC-11-082 renumber structural:** BC-2.11.004 renumber is a structural correction to BC-2.11.004's internal EC-ID namespace; BC-2.11.005 retains EC-11-013 as KEEPER sole live citation; 3 live BC sites (BC-2.11.004 body) now carry EC-11-082; no behavioral regression.

- **pipe_sql_emitter backstop (lines 943–948):** defense-in-depth backstop for function-call detection in pipe-SQL lowering path; no dead-code.

- **sql_parser.rs 243–263 recovery-path guard:** recovery path correctly surfaces parse errors rather than silently accepting malformed function-call LHS expressions.

- **SAP-1 PASS:** 55 raw `event_type =` occurrences / 12 distinct in-scope values verified against BC-2.16.002 v1.61 catalog (92 rows); settled methodology; zero net-new emissions added in this PR.

- **POL-22 Phase A+C PASS:** Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure); Phase C (all positive verifications cross-checked against code, not only pass reports).

- **TD-VSDD-059 PASS:** every claimed closure has a load-bearing test or structural code change; no doc-comment-only or rename-only closures.

- **TD-VSDD-091 PASS:** narrative spec content cites function names and behavioral anchors; no file:line volatile pins in live BC prose.

- **No secrets, no AI attribution, no `--no-verify` markers in diff.**

- **PR-body facts cross-verified vs pr-223-created.md:** DEFECT-PQL-FNCALL-LHS-001, HEAD 973aedcf, 13 files, 7490+/79-, base develop — consistent. Note: GitHub PR body not directly queryable by adversary at review time (read-only limitation); pr-223-created.md is the authoritative source-of-record for merge-gate asks.

- **CI re-run 87265140790 COMPLETED:** PR #223 now 43/43 checks PASS (verified 2026-07-15 via `gh pr checks 223`).

---

## Convergence Status

- CLEAN(strict): YES — zero findings of any severity within perimeter
- CLEAN(PR-merge): YES
- Streak: **1/3** on frozen HEAD 973aedcf
- DRIFT-ORCH-PRLEVEL-PUSH-001: clean (no pushes since HEAD was frozen at push)
- Novelty: LOW — spec has converged for this defect scope; adversary re-derives the same load-bearing evidence set with no surprises

---

## Next Step

PR-LEVEL pass 2 on SAME frozen HEAD 973aedcf (streak 1/3 → target 2/3).
