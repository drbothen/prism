---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [8]
feature_head_at_review: 97cb070e
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

# PR-LEVEL Adversary Pass 8 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 8 (frozen 97cb070e; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3→1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

Streak: **1/3** (BC-5.39.001 strict criterion: pass-8 has ZERO findings of any severity; CLEAN(strict)=YES; streak advances 0/3→1/3 on frozen HEAD 97cb070e; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — HEAD verified 97cb070e local=origin=PR before and after)

Cascade tally (as of this pass): **8 passes / 4 fix-bursts**

CLEAN(strict): YES — ZERO in-perimeter findings
CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings

---

## Findings

No in-perimeter findings.

---

## Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3+4+5+6+7; not re-raised as a PR-LEVEL finding.

---

## Positive Verifications

- **Detail-builder spec-code-test triangle probe (spec-side internal-consistency audit) — NO BYPASS:** `having_aggregate_interception_detail(name: &str) -> String` verified byte-exact vs BC-2.11.019 v1.23 §OBS-004 canonical template for both branches: (1) `percentile` branch → byte-verbatim `(field, p)` canonical template locked by `test_having_aggregate_detail_percentile_canonical`; (2) else branch → signature-neutral `(...)` template locked by `test_having_aggregate_detail_generic_uses_ellipsis`. Three full-string unit tests lock both branches byte-verbatim at 97cb070e. Reverting either branch causes its lock test to fail — load-bearing confirmed. BC-2.11.019 v1.23 §OBS-004 template is internally consistent with the code implementation; spec-code alignment independently re-derived, not via implementer disclosure (POL-22 Phase A).

- **LOW-006 21-keyword .validate() guard uniformity probe (gate-adjacent load-bearing check) — UNIFORM:** `RESERVED_KEYWORDS` set (21 keywords including NULL added in fix-burst-36) verified present in the `.validate()` callback in `filter_parser.rs`. The guard fires uniformly for ALL shared-parser callers: SQL FROM-WHERE, Pipe WHERE, SqlPipe WHERE, SqlPipe head-WHERE. No caller-specific bypass path identified. `test_keyword_as_function_call_in_where_sql`, `test_keyword_as_function_call_in_where_pipe`, and SqlPipe-head variants all exercise the single `.validate()` gate path. One gate, all callers — load-bearing confirmed.

- **Seven-position walker completeness probe (gate-adjacent load-bearing check) — COMPLETE:** E-QUERY-038 predicate-position walker (engine.rs, 7 positions per BC-2.11.019 §Position-Coverage table) verified complete. HAVING position intentionally exempt — intercepted upstream by `DATAFUSION_BUILTIN_AGGREGATE_NAMES` guard per fix-burst-36; exemption deliberate and documented in BC-2.11.019 v1.23 §D.7.3 cross-note. All seven positions confirmed with load-bearing tests from the fix-burst cascade; no unwalked position identified in the predicate tree structure at 97cb070e.

- **Injection-safety sanitize-at-construction probe (gate-adjacent load-bearing check) — LOCKED:** `ColumnNotFoundDetails::new` chokepoint in `error.rs` verified: `sanitize_for_log` applied at construction time. All downstream formatters receive the sanitized value; no unsanitized code path to a user-facing format surface exists. `test_enrich_udf_not_found_display` locks the sanitized output. CWE-117 threat model: Bidi override characters in adversary-controlled column names sanitized before log emission. Construction-time sanitization is load-bearing — removing the `sanitize_for_log` call at `ColumnNotFoundDetails::new` causes the injection-safety test to fail.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions added in fix-burst-38 (confirmed from pass-6/pass-7 verification; HEAD 97cb070e unchanged since fix-burst-38). Settled methodology carries from prior passes: 55 raw occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog 92 rows; ZERO net-new emissions.

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code at 97cb070e, not only pass reports)
  - `having_aggregate_interception_detail` present at 97cb070e — grep confirmed; two-branch structure (`eq_ignore_ascii_case("percentile")` guard) verified
  - `debug_assert_eq!` ABSENT at 97cb070e in HAVING-interception loop — `rg 'debug_assert' crates/prism-query/src/engine.rs` returns ZERO results in the HAVING-interception loop context (removed in fix-burst-38)
  - EC-11-086 five tests (2 SqlPipe-head-HAVING mirrors + 3 casing-lock verbatim tests from fix-burst-37) verified GREEN at 97cb070e — behavioral output of HAVING interception unchanged

- **Cross-mode error-message consistency matrix probe (cross-mode error-message consistency audit) — CONSISTENT:** All four defect-class error paths verified consistent across SQL, Pipe, SqlPipe, and SqlPipe-head modes:
  - **keyword-as-fn** (e.g., `distinct(x)` in WHERE position): E-QUERY-001 template fires via LOW-006 `.validate()` gate for all modes; error message structure consistent (function name, RESERVED_KEYWORDS list rendered identically).
  - **aggregate-in-WHERE** (e.g., `count(x) > 0` in WHERE position): E-QUERY-001 fires via the seven-position walker aggregate-detection arm for all modes; consistent error message (aggregate function name, position label).
  - **unknown-UDF** (e.g., `enrich unknown_udf(x)` in WHERE): E-INFUSE-002 fires via enrich-UDF registry lookup for all modes; consistent error message (UDF name, registry miss, `having_aggregate_interception_detail` is NOT invoked on this path — correct).
  - **percentile-HAVING** (e.g., `HAVING percentile(x, 0.95)`): E-QUERY-001 fires via `DATAFUSION_BUILTIN_AGGREGATE_NAMES` intercept with two-branch detail-builder for all modes supporting HAVING; `having_aggregate_interception_detail` produces the consistent `(field, p)` template. No cross-mode inconsistency detected. The 3 casing-lock verbatim tests (EC-11-085/086/087 group) confirm byte-identical output across `PERCENTILE`, `percentile`, `Percentile` variants.

- **TD-VSDD-059 PASS:** All fix-burst-38 closures have load-bearing tests (per pass-6/pass-7 verification; HEAD unchanged at 97cb070e). Three unit tests lock `having_aggregate_interception_detail` behavior structurally; reverting the two-branch fix causes `test_having_aggregate_detail_generic_uses_ellipsis` to fail. No paper-fix class (rename-only, doc-comment-only, assert-only) detected at 97cb070e.

- **TD-VSDD-060 PASS:** `having_aggregate_interception_detail` is private (`fn`, not `pub fn`). Single production call site in HAVING-interception loop. No external callsites exist to sweep; private-function scope bound confirmed.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose. BC-2.11.019 v1.23, BC-2.11.004 v1.48, and ADR-048 v1.16 verified free of volatile line-number citations in their live normative sections. Justified citations (Red Gate test tables, AC source-of-truth tables, pass-report changelogs) remain present and are excepted.

- **Disclosed merge-gate items honored as ratified-and-disclosed — NOT RE-RAISED:** Four items carried from prior passes and disclosed in the PR #223 body (F-PQLFN-PR7-OBS-001 LOW-006 DataFusion-name collision cost + CONVERGENCE HANDOFF items 1–4: DRIFT-PQLFN-OD7 Gap-1/Gap-2, BC-2.11.019 §Injection-safety cross-branch, D-PQLFN-P47-OBS-001 S-3.09 deferral, LOW-006 adjudication feature-decision) are verified present in the PR body. These items are NOT re-raised as pass-8 findings — they are ratified-and-disclosed design decisions awaiting human review at the merge gate. Reporting discipline enforced: verification notes for disclosed items appear in §Positive Verifications only.

- **Commit-history hygiene via file evidence clean:** No AI attribution (`Co-Authored-By: Claude` or robot emoji) present in any commit on `fix/DEFECT-PQL-FNCALL-LHS-001`. No `--no-verify` bypass residue in commit messages or CI override configuration. Standard hygiene checks PASS: no secrets detected, no new `unwrap()`/`expect()` in production code paths, no `reqwest` changes (ADR-050 rustls-tls constraint untouched), non-exhaustive EXPECTED=91 unchanged at 97cb070e.

- **Spec versions verified at 97cb070e:**
  - BC-2.11.019 v1.23 (two-branch detail-builder; debug_assert REMOVED; DML scope cross-note; §OBS-004 updated)
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords including NULL; 3 live BC sites)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.3 HAVING-exemption caveat; 13 live pins)
  - error-taxonomy E-QUERY-039 template current (v2.52)
  - policies.yaml v1.34 (POL-34 registered)

- **Novelty: ZERO** — pass-8 verifies a clean HEAD with no new spec or code changes since fix-burst-38 (same frozen HEAD 97cb070e as passes 6 and 7). All structural concerns from passes 1–7 are closed or disclosed. Remaining cascade is convergence-only (streak 1/3; two more CLEAN(strict) passes needed on unchanged frozen 97cb070e).

---

## Convergence Status

- CLEAN(strict): YES — ZERO in-perimeter findings
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings
- Streak: **1/3** (BC-5.39.001 strict criterion MET; streak advances 0/3→1/3; frozen HEAD 97cb070e UNCHANGED since fix-burst-38; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; NO commits pushed between pass-7 close and pass-8)
- Frozen HEAD: **97cb070e** (PR #223 HEAD; CI 43/43 PASS)
- DRIFT-ORCH-PRLEVEL-PUSH-001: NO commits pushed since fix-burst-38; all subsequent passes must use 97cb070e as the frozen HEAD; passes taken before fix-burst-38 on earlier frozen HEADs do NOT count toward the 97cb070e streak

---

## Next Step

PR-LEVEL pass-9 on SAME frozen 97cb070e (streak 1/3 → target 2/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; NO pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 97cb070e → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge + LOW-006 keyword-list adjudication merge-gate feature-decision).

**Reporting discipline note (carried from pass-7):** Verification notes that confirm prior closures are placed in §Positive Verifications only. Items previously disclosed in the PR body (F-PQLFN-PR7-OBS-001 merge-gate feature-decision + CONVERGENCE HANDOFF items 1–4) are NOT re-raised as findings — they are ratified-and-disclosed awaiting human review. Fresh-context adversary independently re-derived evidence (POL-22 Phase A); no reliance on implementer disclosure.
