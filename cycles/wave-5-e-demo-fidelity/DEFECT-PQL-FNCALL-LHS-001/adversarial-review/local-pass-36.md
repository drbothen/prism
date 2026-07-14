---
pass: 36
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: c9eb2cd4
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: true
finding_count: 3
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: 28
fix_burst_commits: [9745372c]
fix_burst_new_frozen_head: 9745372c
---

# LOCAL Adversary Pass 36 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** c9eb2cd4 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — 3 LOW findings
**CLEAN(PR-merge):** YES — 3 LOW findings (LOW does not block PR-merge gate per BC-5.39.001 §CLEAN(PR-merge))
**Streak:** stays 0/3 on frozen c9eb2cd4; RESET to 0/3 on new frozen 9745372c after fix-burst-28
**SAP-1:** PASS (zero net-new event_type emissions; D-765 ?-propagation)
**Status:** CLOSED — fix-burst-28 COMPLETE @9745372c (D-1754 2026-07-14)

---

## Fix-burst-27 Closure Verification

Before enumerating new findings, this pass verified all five fix-burst-27 closures against frozen c9eb2cd4:

- **BC↔ADR seven-position parity:** BC-2.11.019 v1.17 §OBS-003, §Postconditions (b) "exactly seven positions" + Position (7), firing enumeration (a) (v), §Error Cases Condition column (v) — all five documentation corrections confirmed consistent with ADR-048 v1.14 §D.7.1 (all seven predicate positions) and the shipped code gate.
- **7/7 walk-observable locks spot-verified:** All 7 known-UDF-passes-gate test modules (Positions 1–7) present in `engine.rs` test suite; compound predicate form `enrich_lookup(ip_address)='US' AND totally_unknown_udf(x)=1` confirmed in each; walk-observable structure verified (removal of walk code changes observable assertion output).
- **`engine.rs` docstring 7/7 claim true:** The corrected docstring "all 7 gated surfaces verified: positions 1–7 each carry a known-UDF-passes-gate lock test" confirmed accurate against the 7 test module count at c9eb2cd4.
- **`sanitize_for_log` + test present:** `EnrichUdfNotFoundDetails::new` sanitization site confirmed present (from BC-2.11.019 v1.14 pass-11 fix); load-bearing test at `infusion_udf.rs::invoke_async_with_args` confirmed GREEN.
- **E-QUERY-001 message byte-parity:** The `totally_unknown_udf` fn-call name correctly appears in the `infusion` field of the E-QUERY-039 structured error; no regression in message format from fix-burst-27 restructuring.

All five fix-burst-27 closures VERIFIED REAL. No false-close recurrence.

---

## Findings

### F-PQLFN-P36-LOW-001 [LOW][POL-4] — CLOSED @9745372c (fix-burst-28: Position-N citation sweep ~30 sites)

**Severity:** LOW
**Category:** POL-4 (semantic-anchoring / nomenclature precision) — test comments, assert strings, section headers, doc lines, and enumeration blocks throughout `engine.rs` and adjacent test files referred to predicate-position walk surfaces using ordinal labels ("Position 1", "Position 2", …) that were conflated with ADR-048 locked-decision IDs (OD-1, OD-2, OD-3, OD-4, OD-5, OD-6, OD-7). OD-1 through OD-5 are locked-decision IDs for options chosen in ADR-048 §D.2 through §D.6; they are NOT positional ordinals. Only OD-6 (DML WHERE = Position 6) and OD-7 (INSERT source_select WHERE = Position 7) coincide numerically with position ordinals.
**Status:** CLOSED — fix-burst-28 @9745372c

**Finding:** Approximately 30 sites in `engine.rs` and associated test files used `OD-N` identifiers as if they were interchangeable with "Position N" ordinals. Examples of the conflation:

- Test function comments: `// Position 4: ADR-048 OD-4 SQL WHERE` (OD-4 is a locked option for the aggregate-function whitelist approach; it is not the SQL WHERE position locked-decision)
- Assert strings: `"OD-5 SqlPipe-head WHERE: ..."` (OD-5 is the locked decision for E-QUERY-001 as the gate-trigger error code; it is not the SqlPipe-head WHERE position)
- Enumeration blocks: listing `OD-1..OD-5` as position names where Positions 1–5 have no corresponding OD-N identifiers in ADR-048 §D (OD-1 through OD-5 are the general-design option decisions, not positional walk decisions)

Positions 1–5 are named by their grammar surface only: (1) pipe `| where`, (2) filter-mode root, (3) SqlPipe `| where`, (4) SQL WHERE, (5) SqlPipe-head WHERE. OD-6 and OD-7 are the actual positional ODs (DML WHERE as sixth gated position; INSERT source_select WHERE as seventh gated position).

**Severity rationale:** LOW because: (1) the test assertions and positional comments are not authoritative specification content — they are test-infrastructure prose; (2) no behavioral defect results from the nomenclature conflation (the tests gate the correct surfaces regardless of the naming); (3) however, a future adversary reviewing the test suite would receive a false signal that ADR-048 OD-N IDs correspond to positional ordinals for all 7 positions, which could lead to incorrect cross-referencing when auditing ADR-048.

**Closure evidence @9745372c (fix-burst-28):**

Position-N citation sweep across ~30 sites: section headers, doc lines, trace lines, assert strings, and enumeration blocks. Corrections applied:
- Positions 1–5 comments: `"Position N: [surface description]"` without OD-N attribution (no locked OD exists for Positions 1–5)
- Positions 4/5 that cited OD-4/OD-5: corrected to `"Position 4: SQL WHERE"` / `"Position 5: SqlPipe-head WHERE"` (OD-4/OD-5 belong to aggregate-gate option decisions, not positional locks)
- OD-6 and OD-7 citations RETAINED where they appear with their correct positional correspondence (Position 6 = OD-6 DML WHERE; Position 7 = OD-7 INSERT source_select WHERE — these coincide correctly)
- Post-fix residual grep `OD-[1-5]` across `crates/prism-query/src/engine.rs` and test files: 2 legitimate locked-decision references only (non-positional context; correct)

---

### F-PQLFN-P36-LOW-002 [LOW][doc-completeness] — CLOSED @9745372c (fix-burst-28: 7-row walk table + corrected path sections)

**Severity:** LOW
**Category:** doc-completeness — `check_enrich_udf_availability` function docstring at the top of `engine.rs` omitted three walked-surface arms from its enumeration: (1) the DML arm (`Ast::Sql(SqlStatement::Dml(dml))`) walking `dml.filter` / `dml.source_select.where_`; (2) the `PipeStage::Where` arm; (3) the `Ast::Filter` arm. Additionally, function name references in the docstring used stale identifiers (pre-fix nomenclature).
**Status:** CLOSED — fix-burst-28 @9745372c

**Finding:** The `check_enrich_udf_availability` docstring enumerated only a subset of the predicate-position surfaces the function actually walks. At frozen c9eb2cd4:
- The DML arm (`Ast::Sql(SqlStatement::Dml(dml))` walking `dml.filter` for DELETE/UPDATE and `dml.source_select.where_` for INSERT) was absent from the docstring surface list
- `PipeStage::Where` was absent (the pipe stage walk is nested inside the main pipe walk but is a distinct code path)
- `Ast::Filter` was absent (filter-mode root position)
- Function name references in "Implementation notes" sections cited stale identifiers that no longer matched the implemented function names post-fix-burst-21/22/23/24/25/26/27

**Severity rationale:** LOW because: (1) the missing surfaces are documented in ADR-048 §D.7.1 (the authoritative seven-position walk table); (2) a developer reading the function docstring to understand gate coverage would miss three surfaces; (3) an adversary reviewing the function would flag the mismatch as a potential gate escape when it is actually correct code with an incomplete docstring.

**Closure evidence @9745372c (fix-burst-28):**

"# Predicate-position walks (ADR-048 §D.7.1)" 7-row table added to the `check_enrich_udf_availability` docstring:

| Position | Grammar Surface | Walk Path | Gate Function |
|----------|----------------|-----------|--------------|
| 1 | Pipe `\| where` | `Ast::Pipe` → `PipeStage::Where` | `collect_unknown_scalar_offsets_from_predicate` |
| 2 | Filter-mode root | `Ast::Filter` | `collect_unknown_scalar_offsets_from_predicate` |
| 3 | SqlPipe `\| where` | `Ast::SqlPipe` → `SqlPipeStage::Where` | `collect_unknown_scalar_offsets_from_predicate` |
| 4 | SQL WHERE | `Ast::Sql(SqlStatement::Select)` → `select.where_` | `collect_unknown_scalar_offsets_from_predicate` |
| 5 | SqlPipe-head WHERE | `Ast::SqlPipe` → head `where_` | `collect_unknown_scalar_offsets_from_predicate` |
| 6 | SQL DML WHERE (DELETE/UPDATE) | `Ast::Sql(SqlStatement::Dml(dml))` → `dml.filter` | `collect_unknown_scalar_offsets_from_predicate` |
| 7 | INSERT source_select WHERE | `Ast::Sql(SqlStatement::Dml(dml))` → `dml.source_select.where_` | `collect_unknown_scalar_offsets_from_predicate` |

Path-detection sections corrected; stale function name references updated.

---

### F-PQLFN-P36-LOW-003 [LOW][ADR nomenclature] — CLOSED (architect ADR-048 v1.14→v1.15 + PO BC-2.11.019 v1.17→v1.18 + story NO-OP disposition)

**Severity:** LOW
**Category:** ADR nomenclature — ADR-048 §D.7.1 position walk table cited `collect_unknown_scalar_from_predicate` for the predicate-walk gate function in 7 cells; the runtime gate function actually used is `collect_unknown_scalar_offsets_from_predicate` (the offsets variant, which returns `Vec<(String, usize)>` = fn-call name + offset into predicate string, threading positional data into `QueryParseFailed`). Both functions exist: `collect_unknown_scalar_offsets_from_predicate` is the gate-positions function (returns offsets); `collect_unknown_scalar_from_predicate` is the SQL scalar walk function used in enrichment gate tests (returns `Vec<String>` without offsets).
**Status:** CLOSED — architect ADR-048 v1.14→v1.15 + PO BC-2.11.019 v1.17→v1.18 + story-writer S-DEMO-FIDELITY-REMEDIATION-001 NO-OP disposition (D-1754, POL-25 routing)

**Finding:** ADR-048 §D.7.1 is the authoritative table for predicate-position walks and gate functions. All 7 cells in the "Gate Function" column cited `collect_unknown_scalar_from_predicate`. This is the SQL scalar walk function — it returns `Vec<String>` (function names only, no offset data). The function actually used by the plan-time aggregate gate is `collect_unknown_scalar_offsets_from_predicate` — it returns `Vec<(String, usize)>` (function name + byte offset), which threads the offset into `QueryParseFailed` for accurate error location reporting. An adversary or implementer consulting §D.7.1 would trace to the wrong function and be unable to find where offsets are generated.

**Two-function disambiguation:** The non-offset variant `collect_unknown_scalar_from_predicate` serves the enrichment gate SQL scalar walk (walking SQL expressions for `EnrichStage` UDF calls returning function names). It is actively used in tests (`engine.rs` `check_enrich_udf_availability` scalar path). It is NOT used by the plan-time aggregate gate. The two functions have different signatures, purposes, and callers — the distinction is load-bearing.

Additionally, BC-2.11.019 §D.7.1 table (mirrored from ADR-048) and §D.7.5 code block contained the same stale function name, requiring PO amendment. Story S-DEMO-FIDELITY-REMEDIATION-001 contains 6 citation sites referencing the enrichment-gate scalar-walk function — all 6 are factually accurate (they describe the `collect_unknown_scalar_from_expr` / non-offset path for that story's enrichment scope) and required NO update.

**Severity rationale:** LOW because: (1) the function name error is in spec documentation (ADR and BC), not in the runtime code (which correctly uses the offsets variant); (2) the behavioral invariant is correct — the aggregate gate fires on the right function; (3) the discrepancy only affects readability and cross-reference traceability for developers/auditors consulting §D.7.1; (4) the finding is doc-level only with zero runtime impact.

**Closure evidence (POL-25 routing — architect + PO + story-writer):**

**(1) Architect — ADR-048 v1.14→v1.15 (pre-edit, included in this burst's factory-artifacts commit):**
- §D.7.1 table: all 7 "Gate Function" cells corrected from `collect_unknown_scalar_from_predicate` → `collect_unknown_scalar_offsets_from_predicate`
- Two-variant footnote added to §D.7.1: "`collect_unknown_scalar_offsets_from_predicate` (returns `Vec<(String, usize)>`) = predicate-position gate; `collect_unknown_scalar_from_predicate` (returns `Vec<String>`) = SQL scalar walk + enrichment gate tests (see §D.7.5 Note). Both functions are active; only the offsets variant feeds the aggregate gate."
- §D.7.5 code block corrected: stale `collect_unknown_scalar_from_predicate` call example replaced with `collect_unknown_scalar_offsets_from_predicate` (the gate-positions function)
- Historical v1.6 note added to §D.7.5: "Prior to v1.6, the non-offset variant was the gate function; offsets were introduced when `QueryParseFailed` offset threading was added."

**(2) PO — BC-2.11.019 v1.17→v1.18 (pre-edit, included in this burst's factory-artifacts commit):**
- 5 gate-position cite sites in BC-2.11.019 corrected: §D.7.1 mirror table cells + §D.7.5 reference sites updated to `collect_unknown_scalar_offsets_from_predicate` (7 total sites, per POL-25 routing sweep result)
- `collect_unknown_scalar_from_expr` CORRECTLY UNTOUCHED: BC-2.11.019 enrichment-scalar-walk citations (where the non-offset function is factually correct for that context) unchanged
- POL-32 verified: v1.18 changelog row prepended to §Changelog in descending version order

**(3) Story-writer — S-DEMO-FIDELITY-REMEDIATION-001 = NO-OP (documented disposition):**
All 6 story citation sites examined: they reference `collect_unknown_scalar_from_expr` or the non-offset scalar-walk variant in the context of the enrichment gate (story scope), where the non-offset function is the correct gate function. The stories are historically and factually accurate descriptions of their own scope — the enrichment gate SQL scalar walk does NOT use the offsets variant. No story version churn warranted. Disposition documented in story audit log.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in `crates/` on HEAD c9eb2cd4. Fix-burst-28 changes are comment/doc/OD-citation sweeps only (no `event_type` emission sites modified). No BC-2.16.002 catalog row required. D-765 ?-propagation precedent applies: fix-burst-27 replaced `tracing::warn!` emit sites with `?` propagation — no new catalog row required for those removed emissions.

---

## Verification Walk

- **Seven-position parity (BC-2.11.019 v1.17 ↔ ADR-048 v1.14 ↔ code ↔ tests):** All three artifacts confirm seven predicate surfaces gated. BC §OBS-003 "gated" and "un-gated" scopes align with ADR-048 §D.7.1 and code walks. No residual six-position live-prose claims.
- **7/7 walk-observable lock tests verified:** Compound predicate form `enrich_lookup(ip_address)='US' AND totally_unknown_udf(x)=1` confirmed in all 7 test modules; each asserts E-QUERY-039 fires + `totally_unknown_udf` appears in `infusion` field + `enrich_lookup` appears in `available_infusions`. Walk-observable structure: removal of walk code changes all three assertions simultaneously.
- **`engine.rs` docstring 7/7 claim confirmed true:** 7 test modules present at c9eb2cd4; docstring claim accurate.
- **ADR-048 §D.7.1 current-version function name (v1.14):** `collect_unknown_scalar_from_predicate` — confirmed stale (pre-v1.15 correction). v1.15 pre-edit included in this burst corrects to `collect_unknown_scalar_offsets_from_predicate`.
- **`collect_unknown_scalar_from_predicate` vs. `collect_unknown_scalar_offsets_from_predicate` disambiguation confirmed:** Both functions present in `crates/prism-query/src/engine.rs`; offsets variant returns `Vec<(String, usize)>` (gate positions); non-offset variant returns `Vec<String>` (scalar walk). Gate entry point uses offsets variant exclusively.
- **E-QUERY-001 message byte-parity:** `test_BC_2_11_019_cursor_cap_exceeded_category_is_internal` (added by fix-burst-25 on MCP lane) is unrelated to PQL lane; PQL E-QUERY-001 format confirmed unchanged from fix-burst-27. `totally_unknown_udf` correctly named in error `infusion` field.
- **Finding-severity trajectory:** HIGH(pass-34) → HIGH+MED(pass-35) → LOW×3(pass-36). Severity decay confirms implementation correctness is converging; remaining findings are documentation drift only.
- **1653/1653 prism-query:** confirmed GREEN at c9eb2cd4 baseline (established by fix-burst-27; unchanged by fix-burst-28 doc/comment sweep).

---

## Status

**CLOSED — fix-burst-28 COMPLETE (D-1754 2026-07-14).**

All 3 findings fully closed:
- **F-PQLFN-P36-LOW-001 CLOSED @9745372c:** OD citation sweep ~30 sites; Position-N labels corrected; OD-6/OD-7 retained (correct coincidence); residual OD-[1-5] grep = 2 legitimate locked-decision refs.
- **F-PQLFN-P36-LOW-002 CLOSED @9745372c:** `check_enrich_udf_availability` docstring 7-row walk table added (ADR-048 §D.7.1 mirror); path-detection sections corrected; stale function name references updated.
- **F-PQLFN-P36-LOW-003 CLOSED:** ADR-048 v1.14→v1.15 (architect, 7 §D.7.1 cells + two-variant footnote + §D.7.5 code block + v1.6 historical note); BC-2.11.019 v1.17→v1.18 (PO, 5 gate-position cite sites corrected; `collect_unknown_scalar_from_expr` correctly untouched); S-DEMO-FIDELITY-REMEDIATION-001 NO-OP (story-writer documented disposition — all 6 sites factually accurate for their scope).

**CASCADE TALLY:** 36 passes / 28 fix-bursts
**STREAK:** 0/3 on new frozen HEAD 9745372c (DRIFT-ORCH-PRLEVEL-PUSH-001: streak resets when new commits exist; all 3 findings closed; 9745372c is local-only)
**NEXT ACTION:** LOCAL pass-37 on frozen 9745372c (streak 0/3)
