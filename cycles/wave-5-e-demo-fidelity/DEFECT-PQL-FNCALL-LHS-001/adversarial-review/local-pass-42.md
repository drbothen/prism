---
pass: 42
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 1/3
streak_after: 2/3
status: CLEAN
fix_burst: null
fix_burst_head_unchanged: null
fix_burst_spec_only: null
fix_burst_bc: null
---

# LOCAL Adversary Pass 42 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; unchanged from pass-41)
**CLEAN(strict): YES** (zero findings of any severity)
**CLEAN(PR-merge): YES** (zero CRIT + zero HIGH + zero MED)
**Streak: 2/3** (ADVANCES from 1/3 — second consecutive CLEAN(strict) pass; BC-5.39.001 streak-advance rule)

---

## Pass-41 Closure Re-Verification

Pass-41 had zero findings. No closures to re-verify; the pass was already CLEAN(strict).

Pass-42 confirms all pass-41 and earlier verifications remain stable at frozen HEAD 5e4c7ccb:
- BC-2.11.019 at v1.19 (at time of pass-42 review; v1.20 pre-edit pending fix-burst-32 attribution truth closure): seven-position walker coverage confirmed; incremental attribution present.
- E-QUERY-042 arm-4 fn-call LHS gate at materialization boundary confirmed live.
- push-down `collect_equality_exprs` non-Field-LHS rejection confirmed.
- Span-shift explicit-enumeration compile-protection confirmed (all seven spans enumerated).
- 1653/1653 prism-query GREEN state at 5e4c7ccb.

---

## Findings

**ZERO findings.** No CRIT, HIGH, MED, LOW, OBS, or PROCESS-GAP issues identified at frozen HEAD 5e4c7ccb.

---

## SAP-1 Result

**PASS.** `event_type =` emission sweep at frozen 5e4c7ccb: 232 total sites across `crates/`; 31 matches in `crates/prism-query/` (primary scope of this defect lane). All `event_type` values map to catalogued rows in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. Zero orphans. Fix-burst-31 (spec-only BC-2.11.019 doc correction) introduced no net-new `event_type =` emissions.

---

## Positive Verifications (Pass 42)

- **Grammar traced end-to-end — admission surface confirmed (LOW-001/004/005/006):** The `fn_call_comparison` production in `build_predicate_parser` admits only `literal | field_path` as fn-call arguments (LOW-001 scope limit). Fn-names must begin with ASCII alphabetic or `_` (LOW-004). Fn-call is not admitted on the RHS (LOW-005). Reserved keywords in fn-name position produce E-QUERY-001 (LOW-006 20-keyword case-insensitive exclusion list). All four scope limits line-verified at 5e4c7ccb; no regression.

- **Atom-choice ordering verified:** `IIN` is tried before `IN` in the Chumsky combinator chain (risk-mitigation from story frontmatter). Ordering confirmed correct at 5e4c7ccb — no parse-ambiguity regression introduced by the grammar extension.

- **Six-callers arithmetic confirmed:** `build_predicate_parser` is shared by six callers (filter-mode root predicate, pipe `| where` stage, SqlPipe pipe-stage `| where`, SQL WHERE, SQL HAVING, SQL DML WHERE via `build_delete_parser`/`build_update_parser`). The `fn_call_comparison` production therefore extends fn-call LHS grammar to all six positions. This matches BC-2.11.004 §Postconditions SHARED-PARSER SCOPE (F-PQLFN-P1-MED-001) and ADR-048 v1.15 §D.7. No under-count or over-count at 5e4c7ccb.

- **E-QUERY-001 — both forms byte-verbatim (POL-24):** Form A (parse-error with detail-only, no prefix in detail string) and Form B (double-prefix ratified v1.12) both confirmed present in their respective callsites at 5e4c7ccb. POL-24 byte-verbatim requirement satisfied; no message drift from prior passes.

- **E-QUERY-042 NonColumnLhsComparison byte-verbatim (em-dash):** The E-QUERY-042 NonColumnLhsComparison error message canonical form — including the em-dash separator — confirmed byte-verbatim at 5e4c7ccb. No character substitution (en-dash, hyphen) introduced by the grammar extension.

- **Emitter defense-in-depth confirmed:** `pipe_sql_emitter.rs` carries charset validation for fn-call names in `fn_call_comparison` lowering context. `Aggregate` and `Window` function variants are blocked upstream in the predicate walk (aggregate gate) before reaching the emitter; no bypass path at 5e4c7ccb.

- **No cross-stage interaction with sort/limit/stats:** The fn-call LHS grammar extension operates only within `build_predicate_parser`. `| sort`, `| head`, `| tail`, and `| stats` productions do not share this parser; no cross-stage regression at 5e4c7ccb.

- **Regression-to-develop assessment CLEAN:** The fold in `predicate_fncall_names` that chains `collect_unknown_scalar_offsets_from_predicate` across positions preserves the E-QUERY-039 chain correctly. Identifier/digit-leading field_path backtrack paths are unaffected by the grammar extension. No regression to develop observed.

- **LOW-006 keyword-list exhaustiveness vs atom productions:** The 20-keyword reserved-word exclusion list in `fn_call_comparison` (NOT/AND/OR/IN/IIN/IEQ/INE/IS/BETWEEN/LIKE/CIDR/MATCHES/HAS/MISSING/CONTAINS/ICONTAINS/STARTSWITH/ISTARTSWITH/ENDSWITH/IENDSWITH) [narrative-only correction per F-PQLFN-P46-MED-001; original list was not derived from grep] was verified against the full atom-choice ordering. No admission gap for keyword-shaped fn-names at 5e4c7ccb.

- **E-QUERY-042 arm-4 before E-QUERY-037 ordering architect-ratified:** Gate ordering E-QUERY-042 arm-4 (NonColumnLhsComparison) fires at plan time before E-QUERY-037 in the fn-call LHS path — this ordering was architect-ratified (ADR-052 §D4 v1.11); confirmed consistent at 5e4c7ccb.

---

## Status

```
CLEAN — pass 42 complete. Zero findings.

CASCADE TALLY: 42 passes / 31 fix-bursts (pass-43 in flight)

STREAK: 2/3 (ADVANCES from 1/3 — pass-42 CLEAN(strict); BC-5.39.001 streak-advance rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY;
no push since 5e4c7ccb; frozen HEAD unchanged; streak gates on unchanged frozen HEAD.

FINDINGS BREAKDOWN:
  Total: 0

CLEAN(strict): YES (zero findings of any severity)
CLEAN(PR-merge): YES (zero CRIT+HIGH+MED)

NEXT ACTION: LOCAL adversary pass 43 on frozen 5e4c7ccb (streak 2/3; feature HEAD UNCHANGED)
```
