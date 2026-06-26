---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: PR-LEVEL-9
frozen_pr_head: 950c19be
base_develop: 903c8fcb
reviewed_by: vsdd-factory:adversary
date: 2026-06-25
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "NO"
findings_count: 3
findings_blocking: 1
streak_after: "RESET 0/3 on b65b4d0c"
---

# PR-LEVEL Pass 9 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**FROZEN PR HEAD reviewed:** 950c19be
**Base develop:** 903c8fcb

## Verdict

- CLEAN(strict): NO
- CLEAN(PR-merge): NO

## Findings

### MED-1 — Expr::InSubquery fold↔detect asymmetry (BLOCKING)

**Severity:** MED (blocking — same fold↔detect asymmetry family as F-P1-MED-001 / LOW-1 / F-P2-MED-001)

**Root cause:** `Expr::InSubquery` was un-swept in the prior fix-burst. Both `inject_now_expr` (fold) and `expr_has_unfolded_temporal` (detect) skipped it. NOW() in a value-position subquery expression (e.g., `SELECT (x IN (SELECT ... WHERE ts > NOW()-INTERVAL '1h')) AS flagged FROM ...`) stayed unfolded AND undetected — `normalize_expr` catch-all emitted empty string → malformed SQL (SOUL.md #4 violation).

**Parser reach:** SELECT/GROUP BY/ORDER BY/JOIN ON positions — parser-reachable.

**Status:** CLOSED by implementer (950c19be → b65b4d0c) via comprehensive fold↔detect symmetry sweep.

---

### OBS-1 — FuncCall scalar/aggregate args latent gap (non-blocking)

**Severity:** OBS (latent; same root cause as MED-1 — non-reachable today)

**Root cause:** FuncCall scalar/aggregate args — detect recursed into args, fold did not. Non-reachable today via the current grammar (no path constructs a FuncCall with a temporal sub-expression in the args position), but represents the same systemic fold↔detect asymmetry class. Same root cause as MED-1.

**Status:** CLOSED by implementer (950c19be → b65b4d0c) in the same comprehensive symmetry sweep.

---

### OBS-2 — DML WHERE temporal (out-of-scope, verified safe)

**Severity:** OBS (out-of-scope; verified no corruption path)

**Analysis:** DML WHERE temporal predicates never reach `PqlNormalizer` or DataFusion. The `execute_against_session` DML catch-all returns `Ok(empty)` — `normalize` is never called on DML. NOW() is not parser-supported in DML predicates by grammar. AC-004 / BC-2.11.021 scope temporal normalization to SELECT/Filter/Pipe read paths only.

**Status:** VERIFIED SAFE — no corruption path. Out of scope. No fix required.

---

## Fix Disposition

All blocking and latent findings (MED-1 + OBS-1) closed by implementer via a **comprehensive fold↔detect symmetry sweep**:

- `inject_now_expr` now recurses into **exactly** the variants the detection side (`expr_has_unfolded_temporal`) recurses into:
  - `Expr::InSubquery` (fold + detect both fixed — the MED-1 target)
  - `FuncCall::Scalar` / `FuncCall::Aggregate` args (fold + detect both fixed — OBS-1)
  - `SELECT` / `GROUP BY` / `ORDER BY` / `JOIN ON` clauses in `inject_now_sql_query`
- Full symmetry table verified: every `Expr` variant + every `SqlQuery` clause.
- Misleading "mutual-omission symmetry" comment removed.
- 4 load-bearing tests added:
  1. Value-context `Expr::InSubquery` in SELECT-projection
  2. Value-context `Expr::InSubquery` in ORDER BY
  3. Detect-fires unit test
  4. FuncCall-arg fold test
- `just check` EXIT 0 (4949 tests); non-exhaustive 87; SAP-1 clean.

**This ENDS the recurring per-variant asymmetry pattern. `inject_now` fold↔detect is now provably symmetric across all Expr variants and SqlQuery clauses.**

## Post-Fix State

- **New FROZEN PR HEAD:** b65b4d0c
- **3-CLEAN streak:** RESET 0/3 on b65b4d0c (fresh-HEAD rule per DRIFT-ORCH-PRLEVEL-PUSH-001)
- **just check:** EXIT 0 (4949 tests)
- **non-exhaustive:** 87
- **fmt:** canonical clean
- **BC-INDEX:** UNCHANGED v7.15
- **STORY-INDEX:** UNCHANGED v2.477
- **develop_head:** UNCHANGED 903c8fcb

## DO-NOT-FLAG (next pass)

- `inject_now` fold↔detect is **provably symmetric** across all `Expr` variants + `SqlQuery` clauses (`Expr::InSubquery`, `FuncCall` args, `SELECT`/`GROUP BY`/`ORDER BY`/`JOIN ON`). Do not re-flag as asymmetric.
- DML temporal is **out-of-scope-safe**: no `normalize` path; grammar rejects `NOW()` in DML predicates. Do not flag OBS-2.

## Next Step

PR-LEVEL adversary cascade: 3 consecutive CLEAN(strict) passes on UNCHANGED b65b4d0c → CI green → squash-merge (--admin per D-1337) → post-merge POL-14 BC promotion → pre-flight demo re-audit → T13 capstone → T14 recording.
