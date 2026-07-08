---
document_type: adversarial-review
scope: PR-LEVEL
passes: [8]
story: S-PRISMQL-CASE-INSENSITIVE-001
pr: 217
feature_head_at_review: 36a094d6
base_develop_head: 7b1f6c51
closure_head: 36a094d6
date: 2026-07-08
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
streak_after: 3/3
convergence: CONVERGED
authored_by: orchestrator-relay from adversary pass-8 output
---
# PR-LEVEL Adversarial Review — Pass 8
## S-PRISMQL-CASE-INSENSITIVE-001

**Frozen HEAD:** 36a094d6 (feature/S-PRISMQL-CASE-INSENSITIVE-001)
**Base:** develop@7b1f6c51
**Date:** 2026-07-08
**Authored by:** orchestrator-relay from adversary pass-8 output

---

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) | **yes** |
| CLEAN (PR-merge) | **yes** |

**Finding summary:** 0 findings total. Zero CRIT, HIGH, MED, LOW, OBS, PROCESS-GAP.

**Novelty:** LOW — hardest-angle probes (security injection, NULL semantics, concurrency, unreachable-variant coverage) all returned clean. No unexplored defect surface remains.

**Streak status:** **3/3 → BC-5.39.001 3-CLEAN(strict) CONVERGED.** Three consecutive CLEAN(strict) passes (6/7/8) taken against unchanged frozen HEAD 36a094d6. No pushes occurred between any of the three passes (DRIFT-ORCH-PRLEVEL-PUSH-001 satisfied).

---

## Findings

None.

---

## Probe Results

### CWE-89 — SQL injection via `escape_sql_string`

**Result: CLEAN**

The IEQ/INE/IIN operators construct a DataFusion SQL `lower()` comparison using the parsed string literal. The probe verified the full path from PrismQL input → Chumsky parser → AST node → DataFusion logical plan construction. Single-quoted string literals are extracted from the CST as Rust `String` values by the parser; they never reach SQL text again as raw strings (DataFusion's logical plan API takes typed `Expr::Literal(ScalarValue::Utf8(...))` values, not raw SQL text). Therefore escape injection via the string value is structurally impossible — there is no SQL string-building step where the literal value could inject SQL syntax. CWE-89 does not apply to this code path.

### NULL 3-valued logic (3VL) semantics probe

**Result: CLEAN**

The probe examined whether `column IEQ 'value'` behaves consistently with SQL NULL 3VL: `NULL IEQ 'high'` should yield NULL (not FALSE), consistent with `lower(column) = 'high'` on a NULL column. Verified: the case-insensitive operators lower-and-compare using DataFusion's `Expr::Like` / `Expr::BinaryExpr` plan construction, which inherits DataFusion's standard NULL propagation. `NULL IEQ 'anything'` yields NULL, consistent with spec (AC-009 covers NULL behavior). RG-031 (`test_rg031_case_insensitive_eq_null_column`) confirms NULL propagation.

### GROUP BY / HAVING with normalized labels probe

**Result: CLEAN**

`GROUP BY severity` + `HAVING count(*) > 1` with IEQ-normalized values: verified that GROUP BY operates on the normalized (lowercased) column value in the OCSF-normalized table, not on the raw API value. The `sanitize_enum_label_for_log` normalization happens at the OCSF normalization layer before DataFusion sees the data; GROUP BY grouping keys therefore use the normalized form. No asymmetry between the grouped key and the IEQ comparison value. No defect.

### OnceLock enum-map concurrency probe

**Result: CLEAN**

`OcsfEnumMap` is initialized via `OnceLock::get_or_init()` in `shared_enum_map()` (D-1586 consolidation). The `OnceLock` contract guarantees that exactly one initialization closure runs; subsequent callers receive a reference to the same initialized value. No race condition possible: the enum-map data is read-only after initialization. Concurrent query tasks all share a single `&'static OcsfEnumMap` reference. Correct.

### NOT-wrapped CI predicates probe

**Result: CLEAN**

`NOT (column IEQ 'value')` generates the DataFusion plan `NOT (lower(column) = lower('value'))`, which is semantically equivalent to `column INE 'value'` for non-NULL values. The probe examined whether the `NOT` wrapper could bypass the case-insensitive normalization. Verified: the `NOT` operator wraps the entire lowercased comparison expression, not just the column reference. The lowercasing happens inside the comparison; `NOT` is applied after. Correct semantics. RG-043 covers NOT-wrapped CI predicates.

### Emitter guards for all parser-unreachable variants probe

**Result: CLEAN**

The case-insensitive operator normalization logic has several match arms for parser-unreachable AST variants (present as defensive guards per the pattern codified in D-1598/D-1597). The probe verified that all such arms have been either:
(a) covered by a `#[allow(unreachable_patterns)]`-annotated arm with a doc-comment explaining why the pattern is unreachable per the grammar (e.g., `IEQ` with non-String RHS is rejected at parse time), OR
(b) covered by a test that exercises the guard via mock input (e.g., the `to_logical_plan_err` path for malformed AST nodes).
No emitter guard is silently `unreachable!()` without a doc comment or test. Correct.

### SAP-1 — Tracing emission catalog completeness

**Result: CLEAN** — Same as passes 6 and 7. No new emission sites. All existing sites catalog-covered.

### SAP-2 — DTU↔TOML schema parity

**Result: N/A** — No TOML sensor spec changes.

### POL-22 — Phase A+C gates

**Result: CLEAN** — Verified at 36a094d6. Same as passes 6 and 7.

### Paper-fix audit

**Result: none** — No code changes since the D-1605 fix-burst. All load-bearing tests in place.

---

## Convergence Trajectory (PR-LEVEL)

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Streak |
|------|------------|---------------|-----------------|----------|--------|
| 1    | a2fc8940   | no            | no              | 2 MED + 2 LOW + 2 OBS (total 6) | 0/3 reset |
| 2    | 1172b15a   | no            | yes             | 1 LOW (total 1)                 | 0/3 (push resets) |
| 3    | dcb37099   | no            | yes             | 2 OBS (total 2)                 | 0/3 (push resets) |
| 4    | fab7df00   | yes           | yes             | 0 (total 0)                     | 1/3 |
| 5    | fab7df00   | no            | yes             | 3 OBS (total 3)                 | 0/3 RESET |
| 6    | 36a094d6   | yes           | yes             | 0 (total 0)                     | 1/3 |
| 7    | 36a094d6   | yes           | yes             | 0 (total 0)                     | 2/3 |
| 8    | **36a094d6** | **yes**     | **yes**         | 0 (total 0)                     | **3/3 CONVERGED** |

**Final trajectory:** 6 → 1 → 2 → 0 → 3 → 0 → 0 → 0

---

## Post-Pass Action

**PR-LEVEL 3-CLEAN(strict) CONVERGED per BC-5.39.001.** Total PR-LEVEL passes: 8. Combined with LOCAL 3-CLEAN (35 passes, D-1599), story is fully converged.

CI on 36a094d6: 40 pass / 3 pending (fuzz smoke ×2 + E2E smoke) / 0 fail. MERGEABLE per D-989 autonomy gate.

**VERY NEXT ACTION:** Await final 3 CI checks → pr-manager steps 8-9 squash-merge PR #217 (D-989 full-autonomous authorization) → post-merge burst: POL-14 BC-2.11.024 + BC-2.02.013 draft→active; story status → merged; POL-13 index sync; devops worktree cleanup.
