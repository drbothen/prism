---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [3]
feature_head_at_review: 973aedcf
date: 2026-07-15
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 2
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 3 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 3 (frozen 973aedcf; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 2/3 → 0/3 RESET)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

Streak: **0/3 RESET** (BC-5.39.001 strict criterion: pass-3 has 1 LOW + 2 OBS findings within perimeter; CLEAN(strict) requires zero findings of any severity; streak 2/3 → 0/3; new cascade must re-gate on newly-pushed HEAD 76c0fa60 per DRIFT-ORCH-PRLEVEL-PUSH-001 — fix-burst-36 pushed two commits aed17453→76c0fa60 mid-cascade)

Cascade tally (as of this pass): **3 passes / 1 fix-burst**

CLEAN(strict): NO — 3 in-perimeter findings (1 LOW + 2 OBS)
CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; LOW/OBS non-blocking

---

## Findings

### F-PQLFN-PR3-LOW-001 — HAVING percentile E-QUERY-039 leak

**Severity:** LOW
**Category:** code-behavior defect
**Routing:** product-owner + architect + implementer (fix-burst-36; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/engine.rs` — `check_enrich_udf_availability` HAVING-exempt branch; `crates/prism-query/src/ast.rs` — DATAFUSION_BUILTIN_AGGREGATE_NAMES definition; BC-2.11.019 §Postconditions; ADR-048 §D.2

**Defect:** The existing pass-2 Positive Verification confirmed `DATAFUSION_BUILTIN_AGGREGATE_NAMES` as the gate for aggregate function detection, and noted that `percentile` was absent from this set. However, ADR-048 v1.15 §D.2 stated the HAVING position was intentionally exempt from the E-QUERY-039 gate. This created an unresolved behavioral ambiguity: `percentile(x) = 5` in a HAVING clause was documented as producing E-QUERY-039 (via the outer registry path), but the registry check is deliberately bypassed in HAVING position. Under the production code path at 973aedcf, `percentile(x)` in a HAVING clause reaches E-QUERY-039 via the outer `check_enrich_udf_availability` registry call (position: pre-HAVING exempt check), not via the HAVING-position interception. The intended behavior — E-QUERY-001 for ALL aggregate function names in HAVING regardless of registry — was not mechanically enforced; it depended on registry non-membership. This is a latent correctness gap: any future registry entry for `percentile` (or any DATAFUSION_BUILTIN_AGGREGATE_NAMES member) would silently change the HAVING error surface from E-QUERY-001 to E-QUERY-039, violating BC-2.11.019 §Postconditions.

**Failure scenario:** `SELECT * FROM events | where | having percentile(latency, 0.95) = 5` at a registry that happens to contain `percentile` → E-QUERY-039 emitted from HAVING position → violates BC-2.11.019 §Postconditions intent (HAVING should produce E-QUERY-001 registry-independently for aggregate function names).

**BC references:** BC-2.11.019 v1.20 §Postconditions; ADR-048 v1.15 §D.2; EC-11-086

---

### F-PQLFN-PR3-OBS-001 — Missing dedicated LOW-006 SqlPipe-head-WHERE position-5 test

**Severity:** OBS
**Category:** test-coverage gap
**Routing:** test-writer (fix-burst-36; CLOSED this session)

**File/Anchor:** `crates/prism-query/tests/` — keyword gate test suite; BC-2.11.004 v1.47 LOW-006

**Defect:** The RESERVED_KEYWORDS gate (LOW-006: `name ∈ RESERVED_KEYWORDS → E-QUERY-001`) has lock tests for Pipe-mode WHERE and SQL-mode WHERE. The SqlPipe head-WHERE surface (position-5 in the seven-position walker: `Ast::SqlPipe` head clause) lacked a dedicated LOW-006 test. Pass-2 positive verifications confirmed the gate fires at all surfaces, but without a per-surface lock test the SqlPipe head-WHERE coverage is assertion-free at the test layer. A regression could silently remove the SqlPipe position-5 arm without a failing test.

**Failure scenario:** Future refactor removes SqlPipe head-WHERE arm from the RESERVED_KEYWORDS gate → `lower | where 5 = 3` in SqlPipe-head context no longer produces E-QUERY-001 → no test failure.

---

### F-PQLFN-PR3-OBS-002 — NULL absent from RESERVED_KEYWORDS

**Severity:** OBS
**Category:** spec-contract gap
**Routing:** product-owner (fix-burst-36; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/parser/filter_parser.rs` — RESERVED_KEYWORDS constant (20 keywords); BC-2.11.004 v1.47 LOW-006 reserved-keyword list

**Defect:** RESERVED_KEYWORDS contains 20 keywords (NOT/AND/OR/IN/IIN/IEQ/INE/IS/BETWEEN/LIKE/CIDR/MATCHES/HAS/MISSING/CONTAINS/ICONTAINS/STARTSWITH/ISTARTSWITH/ENDSWITH/IENDSWITH). `NULL` is not present. `NULL` is a PrismQL literal that when used as a function name would parse ambiguously (fn_call grammar at the filter parser level). Without NULL in RESERVED_KEYWORDS, `null(x) = 5` passes the LOW-006 gate and proceeds to the registry/aggregate checks where it would produce a less-informative error or pass silently if DataFusion accepts it as a scalar alias. The BC-2.11.004 v1.47 keyword list should include NULL for completeness of the reserved-word surface.

**Failure scenario:** User writes `null(device_id) = 'x'` → not blocked at LOW-006 gate → proceeds to registry lookup → produces confusing downstream error rather than the parse-time E-QUERY-001 canonical reserved-keyword message.

---

### Out-of-Scope Deferred Item (carried from LOCAL cascade; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2; not re-raised as a PR-LEVEL finding.

---

## Fix-Burst-36 Closure

All three in-perimeter findings closed same-session by the following agent trail. Fix-burst-36 produced two commits: test-writer aed17453 + implementer 76c0fa60. Branch pushed to origin; PR #223 HEAD is now 76c0fa60.

### F-PQLFN-PR3-LOW-001 — HAVING percentile E-QUERY-039 leak

**product-owner (BC-2.11.004 v1.47→v1.48):** Added EC-11-085 (`NULL(x)=5` → E-QUERY-001 parse-time; NULL ∈ RESERVED_KEYWORDS), EC-11-086 (`HAVING percentile(x)=5` → E-QUERY-001 plan-time registry-independent; HAVING-position `DATAFUSION_BUILTIN_AGGREGATE_NAMES` intercept fires before registry guard), EC-11-087 (`HAVING distinct_count(x)>0` → SUCCESS; distinct_count not in DATAFUSION_BUILTIN_AGGREGATE_NAMES → passes to registry → not found → E-QUERY-039). RESERVED_KEYWORDS 20→21 adding NULL. Test vectors added. BC-2.11.004 v1.48.

**product-owner (BC-2.11.019 v1.20→v1.21):** §Postconditions position-f interception note added: two-condition criterion `name ∈ DATAFUSION_BUILTIN_AGGREGATE_NAMES ∧ name ∉ DATAFUSION_BUILTIN_FUNCTION_NAMES` for HAVING-position interception (fires E-QUERY-001 registry-independently); §OBS-004 added documenting the deliberate asymmetry (distinct_count not in aggregate names set → reaches registry → E-QUERY-039 if not registered). BC-2.11.019 v1.21.

**architect (ADR-048 v1.15→v1.16):** §D.2 Mechanism 2 rewritten to describe the HAVING-position `DATAFUSION_BUILTIN_AGGREGATE_NAMES` intercept explicitly — fires E-QUERY-001 with HAVING-specific guidance (registry-independent); E-QUERY-039 outcome retracted for HAVING aggregate names; byte-verbatim canonical message documented. §D.7.1 HAVING-exemption caveat added: `DATAFUSION_BUILTIN_AGGREGATE_NAMES` members in HAVING fire E-QUERY-001 not E-QUERY-039; `distinct_count` unaffected. §D.7.3 percentile outcome updated: "registry-dependent E-QUERY-039" → "E-QUERY-001 (registry-independent, v1.16)". Stale test `test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt` noted for supersession per this v1.16 mechanism change. POL-23 sweep: 7 BC-2.11.004 v1.15 pins → v1.16 in BC-2.11.004; 5 BC-2.11.019 v1.15 pins → v1.16 in BC-2.11.019. ARCH-INDEX row updated in pre-edit. ADR-048 v1.16.

**story-writer (S-PRISMQL-CASE-INSENSITIVE-001 v1.72→v1.73):** POL-23 sweep of BC-2.11.004 v1.47→v1.48 pin sites in story (4 sites including bare-cell at Behavioral Contracts table — one additional site beyond PO flag); BC-2.11.019 v1.20→v1.21 pins (0 live story pins). Story v1.73; updated 2026-07-15.

### F-PQLFN-PR3-OBS-001 — Missing dedicated LOW-006 SqlPipe-head-WHERE position-5 test

**test-writer (commit aed17453):** 5 RED tests added: 2× EC-11-086 coverage (registry-present / registry-absent HAVING aggregate interception), 3× EC-11-085 coverage (null() via Pipe-WHERE / SQL-WHERE / filter surfaces). 2 GREEN locks added: EC-11-087 (distinct_count passes HAVING gate to registry → E-QUERY-039 registry-dependent), F-PQLFN-PR3-OBS-001 (SqlPipe-head-WHERE position-5 LOW-006 lock for `lower | where 5 = 3`). Stale test `test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt` replaced with updated test asserting E-QUERY-001 at HAVING position (superseded per ADR-048 v1.16 §D.2 mechanism change). Total: +7 tests, −1 stale replaced.

### F-PQLFN-PR3-OBS-002 — NULL absent from RESERVED_KEYWORDS

**implementer (commit 76c0fa60):** RESERVED_KEYWORDS array extended with `NULL` entry in `filter_parser.rs` `fn_call_comparison` (20→21 keywords); `having_fncall_names` collection added; two-condition HAVING interception added BEFORE registry guard in `engine.rs` `check_enrich_udf_availability` (fires E-QUERY-001 for DATAFUSION_BUILTIN_AGGREGATE_NAMES ∧ ¬DATAFUSION_BUILTIN_FUNCTION_NAMES names in HAVING position); comment-pin sweep (20-keyword→21-keyword, 4 sites). prism-query 1659/1659 GREEN; `just check` 5595/5595 GREEN; non-exhaustive 91/91.

**Post-fix verification:** Workspace 5595/5595 tests PASS. prism-query 1659/1659. non-exhaustive EXPECTED=91. `just check` pre-push GREEN. Branch pushed to origin/fix/DEFECT-PQL-FNCALL-LHS-001 at 76c0fa60 (2 commits over 973aedcf: aed17453 + 76c0fa60). PR #223 HEAD updated to 76c0fa60. CI PENDING on new HEAD.

---

## Positive Verifications

- **SAP-1 PASS:** ~55 raw `event_type =` occurrences / ~12 distinct values verified against BC-2.16.002 v1.61 catalog (92 rows); ZERO net-new emissions added in this diff. Settled methodology carries from LOCAL cascade.

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code, not only pass reports)
  - `check_enrich_udf_availability` (engine.rs) HAVING-exempt branch verified at canonical definition site
  - `DATAFUSION_BUILTIN_AGGREGATE_NAMES` definition verified byte-matching BC-2.11.004 v1.47 at 3 canonical sites
  - `DATAFUSION_BUILTIN_FUNCTION_NAMES` verified at definition site
  - `RESERVED_KEYWORDS` 20-keyword list verified byte-matching BC-2.11.004 v1.47 LOW-006 at 3 canonical sites in BC-INDEX v8.25
  - `EnrichUdfNotFoundDetails` (error.rs: `#[non_exhaustive]`, `sanitize_for_log` at `new()`) exists at canonical definition site
  - Seven-position walker confirmed HAVING intentionally exempt per ADR-048 §D.7.1

- **TD-VSDD-059 PASS:** All closures load-bearing: injection-safety construction test (U+0085/U+2028 stripped), self-sorting Display test, seven-position aggregate + E-QUERY-039 fold tests, HAVING-exemption GREEN lock, LOW-006 keyword gate lock.

- **TD-VSDD-060 PASS:** No function signature changes in this diff; callsite sweep clean.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **Gate ordering verified:** E-QUERY-001 → E-QUERY-041/042 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039 per BC-2.11.019 §Gate Ordering; HAVING-position DATAFUSION_BUILTIN_AGGREGATE_NAMES check fires inside this ordering at HAVING arm position.

- **10 adversarial bypass probes — NO BYPASS:**
  1. Case-normalization: `LOWER(device_id) = 'x'` at HAVING → E-QUERY-001 aggregate-member intercept fires; no bypass via case variant
  2. Unicode confusables: homoglyph-substituted function name → blocked at fn_call grammar (identifier-start constraint)
  3. Nested fn-call args: `sum(lower(x)) = 5` in WHERE → non-aggregate outer fn-name → registry lookup → E-QUERY-039 correct path
  4. IN-subquery at HAVING: `count(*) IN (SELECT ...)` → documented fail-open (S-3.07 deferred) per ADR-048 §D.7.3 Gap-2; not a regression
  5. SqlPipe span shifting: spans shift when SqlPipe head clause grows → LOW-006 test locks against position-5 regressing
  6. Double-nested HAVING + WHERE: nested predicate with aggregate in both clauses → only HAVING arm fires on HAVING aggregate; WHERE arm fires on WHERE aggregate; no conflation
  7. NULL in HAVING: `null(x) = 5` in HAVING → RESERVED_KEYWORDS LOW-006 fires BEFORE aggregate intercept; correct ordering
  8. DATAFUSION_BUILTIN_FUNCTION_NAMES overlap: any function in both aggregate and function registries → two-condition criterion `∈ AGGREGATE ∧ ∉ FUNCTION_NAMES` correctly passes to registry (E-QUERY-039 path); no spurious E-QUERY-001
  9. distinct_count in HAVING: `having distinct_count(x) > 0` → distinct_count not in DATAFUSION_BUILTIN_AGGREGATE_NAMES → intercept does NOT fire → registry check proceeds → E-QUERY-039 if not registered; EC-11-087 GREEN lock confirms
  10. Stale test supersession: `test_BC_2_11_016_tm_having_percentile_not_e_query_001_having_exempt` correctly replaced; no ghost test asserting old behavior

- **Spec versions verified at 973aedcf (pre-fix-burst-36):**
  - BC-2.11.019 v1.20 (injection-safety; load-bearing test rides THIS branch)
  - BC-2.11.004 v1.47 (EC-11-082 renumber structural; 3 live BC sites; BC-2.11.005 KEEPER sole live EC-11-013)
  - ADR-048 v1.15 §D.7.1–D.7.6 (seven-position walker; 13 live pins)
  - error-taxonomy E-QUERY-039 template current (v2.52)

- **No secrets, no AI attribution, no `--no-verify` markers in diff.**

- **No new `unwrap()` / `expect()` in production code paths.**

- **`#[non_exhaustive]` on new pub types** — no new pub types introduced in fix-burst-36.

- **No `reqwest` changes** — ADR-050 rustls-tls constraint untouched.

- **Novelty: LOW** — findings are tightly scoped to the HAVING-position behavioral gap (LOW-001) and two observational gaps (test coverage, keyword completeness). No new structural concerns surfaced. Independent adversary re-derivation corroborates the LOCAL cascade convergence evidence trail.

---

## Convergence Status

- CLEAN(strict): NO — 3 in-perimeter findings (1 LOW + 2 OBS); strict criterion requires zero findings of any severity
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED; LOW/OBS non-blocking for PR merge
- Streak: **0/3 RESET** (BC-5.39.001 strict criterion failed; fix-burst-36 pushed new commits aed17453 + 76c0fa60; DRIFT-ORCH-PRLEVEL-PUSH-001: streak must re-gate on frozen HEAD 76c0fa60)
- New frozen HEAD: **76c0fa60** (PR #223 HEAD after fix-burst-36; CI PENDING)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-36 push mid-cascade resets streak; all pass-4+ must use 76c0fa60 as the frozen HEAD; passes 1/2/3 on 973aedcf do NOT count toward 76c0fa60 streak

---

## Next Step

CI green on 76c0fa60 (PR #223 new HEAD) → PR-LEVEL pass-4 on frozen 76c0fa60 (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 76c0fa60 → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge).
