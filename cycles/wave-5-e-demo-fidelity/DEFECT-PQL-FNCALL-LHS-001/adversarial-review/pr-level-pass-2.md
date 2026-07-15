---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [2]
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
streak_after: 2/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 2 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 2 (frozen 973aedcf; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 1/3 → 2/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

Streak: **2/3** (BC-5.39.001: two consecutive CLEAN(strict) passes on frozen HEAD 973aedcf; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; HEAD verified unchanged at 973aedcf (local = origin = PR headRefOid) before and after the pass; pass 3 next)

Cascade tally: **2 passes / 0 fix-bursts**

---

## Findings

None in-perimeter. Zero findings of any severity class (CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP) within the PR scope.

### Out-of-Scope Deferred Item (carried from LOCAL cascade; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade; not re-raised as a PR-LEVEL finding.

---

## Positive Verifications

- **SAP-1 PASS:** ~170 raw `event_type =` occurrences / ~35 distinct values verified against BC-2.16.002 v1.61 catalog (92 rows); ZERO net-new emissions added in this diff. `ast.rs` normalizer `tracing::warn!(target: "prism_query::normalizer", ...)` correctly annotated as SAP-1-non-applicable (no `event_type` field — does not contribute a catalog-tracked emission).

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code, not only pass reports)
  - `EnrichUdfNotFoundDetails` (error.rs: `#[non_exhaustive]`, `sanitize_for_log` at `new()`) exists at canonical definition site
  - `TemporalLiteralPosition` exists at canonical definition site
  - `check_enrich_udf_availability` (engine.rs: seven positions per ADR-048 §D.7.1, HAVING intentionally exempt)
  - `DATAFUSION_BUILTIN_AGGREGATE_NAMES` (+ `distinct_count` / `percentile` variants) verified at definition site
  - `DATAFUSION_BUILTIN_FUNCTION_NAMES` verified at definition site
  - `RESERVED_KEYWORDS` 20-keyword list verified byte-matching BC-2.11.004 v1.42 ratification at canonical 3 sites in BC-INDEX v8.25

- **POL-24 PASS:**
  - E-QUERY-039 template byte-verbatim match confirmed (error-taxonomy.md ↔ error.rs)
  - Aggregate-in-WHERE detail matches BC-2.11.004 v1.47 EC-11-082 vector
  - LOW-006 E-QUERY-001 keyword message locked by 11 assertions across 7 shared-parser surfaces

- **TD-VSDD-059 PASS:** All closures load-bearing:
  - Injection-safety construction test asserting U+0085/U+2028 stripped (load-bearing, not doc-comment-only)
  - Self-sorting `Display` test
  - Seven-position aggregate + E-QUERY-039 fold tests
  - HAVING-exemption GREEN lock

- **TD-VSDD-060 PASS:** Constructor callsites consistent; no signature changes in this diff; callers in engine.rs `execute()` gate chain verified for completeness.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **Gate ordering verified:** E-QUERY-001 → E-QUERY-041/042 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039 per BC-2.11.019 §Gate Ordering.

- **Spec versions verified:**
  - BC-2.11.019 v1.20 (injection-safety; load-bearing test rides THIS branch)
  - BC-2.11.004 v1.47 (EC-11-082 renumber structural; 3 live BC sites; BC-2.11.005 KEEPER sole live EC-11-013)
  - ADR-048 v1.15 §D.7.1–D.7.6 (seven-position walker; 13 live pins current)
  - error-taxonomy E-QUERY-039 template current (v2.52)

- **No secrets, no AI attribution, no `--no-verify` markers in diff.**

- **No new `unwrap()` / `expect()` in production code paths.**

- **`#[non_exhaustive]` on new pub types** — `EnrichUdfNotFoundDetails` (error.rs), `TemporalLiteralPosition` confirmed with attribute.

- **No `reqwest` changes** — ADR-050 rustls-tls constraint untouched.

- **Novelty: LOW** — fresh-context re-derivation corroborates convergence. Adversary independently arrives at the same load-bearing evidence set as pass-1 with no surprises. No new structural concerns surfaced.

---

## Convergence Status

- CLEAN(strict): YES — zero findings of any severity within perimeter
- CLEAN(PR-merge): YES
- Streak: **2/3** on frozen HEAD 973aedcf
- DRIFT-ORCH-PRLEVEL-PUSH-001: clean (HEAD verified unchanged at 973aedcf before and after pass)
- Novelty: LOW — spec has converged for this defect scope; independent re-derivation produces consistent evidence set across two consecutive CLEAN passes

---

## Next Step

PR-LEVEL pass 3 on SAME frozen HEAD 973aedcf (streak 2/3 → target 3/3). On 3/3 → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge).
