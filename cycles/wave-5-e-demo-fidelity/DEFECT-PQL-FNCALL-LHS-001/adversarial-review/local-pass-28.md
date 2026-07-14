---
pass: 28
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: ef8b9bb3
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: true
finding_count: 3
streak_before: 0/3
streak_after: 0/3
---

# LOCAL Adversary Pass 28 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** ef8b9bb3 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES (zero CRIT/HIGH/MED findings)
**Streak:** 0/3 (unchanged — LOW findings prevent strict-CLEAN advancement)

---

## Findings

### F-PQLFN-P28-LOW-001 [LOW][doc-accuracy]

**Location:** `sql_parser` guard comment (fn_call_comparison gate)
**Description:** The guard comment pseudo-quoted a phrase not present in ADR-048. The comment attributed a rationale to ADR-048 using quotation-style phrasing that did not match ADR-048's actual language, creating a false citation.
**Severity rationale:** Doc-accuracy only; no behavioral impact. Production-grade default requires truthful citations.
**Status:** OPEN (dispatched to fix-burst-22)

---

### F-PQLFN-P28-LOW-002 [LOW][doc-accuracy]

**Location:** `InSubquery` no-op arms rationale comment
**Description:** The no-op arms carried a structurally-false rationale: "SqlQuery is not an Expr" — which is grammatically incorrect (SqlQuery IS a variant path). The true rationale is grammar-unreachability: InSubquery subqueries cannot currently contain a function call as the LHS comparison target because the grammar does not admit that form. The false rationale masked the real reason for the no-op.
**Severity rationale:** Doc-accuracy / correctness of comment. The false "not an Expr" claim would mislead future maintainers attempting to extend the grammar. FUTURE-EXTENSION NOTE obligation identified: if grammar extends to allow InSubquery LHS function calls, these arms must be revisited.
**Status:** OPEN (dispatched to fix-burst-22)

---

### F-PQLFN-P28-OOS-001 [MED][out-of-scope → FIXED IN SCOPE by orchestrator decision]

**Location:** `SqlPipe` rewriter — stage-error path
**Description:** Pre-existing asymmetry: 2 of 3 pipe-error rewriters (rewrite_temporal_literal_in_pipe_key_position) were absent from the SqlPipe stage-error path, violating BC-2.11.023 AC-025 parity. The third rewriter (`rewrite_fn_call_comparison_in_pipe_key_position`) was present. This OOS finding was authorized for in-scope fix by orchestrator decision because: (a) parity gap was load-bearing (users of SqlPipe temporal and other key-position rewrites would receive generic errors instead of actionable ADR-052 §D4 messages), and (b) the fix was bounded (single helper addition with clear staging semantics).
**BC reference:** BC-2.11.023 AC-025
**Status:** FIXED IN SCOPE by orchestrator decision (fix-burst-22)

---

## SAP-1 Check (Tracing Emission Catalog)

`rg 'event_type\s*=' crates/ --type rust` — no new `event_type` values introduced in ef8b9bb3 relative to ef8b9bb3 baseline.
**SAP-1: PASS**

---

## Policy Rubric Summary

- Phase A (grammar alternatives): 7 compare_op alternatives — PASS
- Phase C (20 RESERVED_KEYWORDS list, 14 non-compose productions): PASS
- 6-position gate coverage: PASS (all 6 gate positions verified)
- 3-rewriter parity: PARTIAL (asymmetry found — OOS-001 dispatched for fix)
- ADR-048 citation accuracy: FAIL (LOW-001 pseudo-quote)
- Comment truth: FAIL (LOW-002 false rationale)

**Novelty:** LOW

---

## Disposition

- CLEAN(strict): NO — 2 LOW findings prevent streak advancement
- CLEAN(PR-merge): YES — zero CRIT/HIGH/MED
- Streak: STAYS 0/3
- Action: fix-burst-22 dispatched (LOW-001, LOW-002, OOS-001)
