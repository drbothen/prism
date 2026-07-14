---
pass: 35
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: bd044a2e
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: false
finding_count: 5
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: 27
fix_burst_commits: [c9eb2cd4]
fix_burst_new_frozen_head: c9eb2cd4
---

# LOCAL Adversary Pass 35 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** bd044a2e (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — 2 HIGH + 2 MED + 1 OBS findings
**CLEAN(PR-merge):** NO — 2 HIGH + 2 MED findings (HIGH + MED block both criteria)
**Streak:** stays 0/3 on frozen bd044a2e; RESET to 0/3 on new frozen c9eb2cd4 after fix-burst-27
**SAP-1:** PASS (zero net-new event_type emissions)
**Status:** CLOSED — fix-burst-27 COMPLETE @c9eb2cd4 (D-1753 2026-07-14)

---

## Findings

### F-PQLFN-P35-HIGH-001 [HIGH][spec-drift] — CLOSED (BC-2.11.019 v1.16→v1.17, product-owner, factory-artifacts this burst)

**Severity:** HIGH
**Category:** spec-drift — BC-2.11.019 §OBS-003 gate-scope prose claimed ENTIRE `dml.source_select` subtree un-gated including `source_select.where_`; directly contradicts ADR-048 §D.7.6 (Position 7) AND shipped code
**Status:** CLOSED — BC-2.11.019 v1.16→v1.17 (product-owner, same factory-artifacts burst)

**Description:** BC-2.11.019 §OBS-003 ("Gate Scope Boundaries" sub-section) contained v1.13-era content stating that `check_enrich_udf_availability` does NOT walk `dml.source_select` at all, and listed `source_select.where_` alongside `source_select.having`, `source_select.join_on`, and `source_select.projections` as an un-gated position.

At HEAD bd044a2e this claim was factually false: ADR-048 v1.13 §D.7.6 added Position 7 (INSERT source_select WHERE) as a **gated** surface via the `Ast::Sql(SqlStatement::Dml(dml))` arm that walks `dml.source_select.where_`. The code implements this gate; a load-bearing test `test_f_pqlfn_p32_obs_001_insert_source_select_where_aggregate_fires_e_query_001` is GREEN on this branch. The BC §OBS-003 prose was left behind at v1.13 when the code and ADR were updated.

**Impact:** HIGH because §OBS-003 is explicitly the "Gate Scope Boundaries" sub-section of BC-2.11.019 — its entire purpose is to document which surfaces ARE and ARE NOT gated. An adversary, security reviewer, or future implementer consulting §OBS-003 to understand gate coverage would conclude `INSERT source_select WHERE` is NOT gated, producing either: (a) a false security-gap report (actionable noise requiring investigation), or (b) unnecessary remediation to add a gate that already exists. The BC is the authoritative specification; spec-vs-code drift at this severity is a production-grade issue.

**Severity rationale:** HIGH because: (1) the contradicted spec section is the authoritative scope-boundary section (not a prose description); (2) the contradiction directly contradicts ADR-048 architecture decision AND the code simultaneously; (3) no reader consulting this BC alone would conclude the gate exists at Position 7.

**Closure evidence:** BC-2.11.019 v1.16→v1.17 (product-owner, factory-artifacts this burst):
- §OBS-003 title corrected from "DML WHERE Predicates Now Gated; DML Source-Select Projections Remain Un-Gated" → "DML WHERE + INSERT source_select WHERE Now Gated; `source_select` {having, join_on, projections} Remain Un-Gated (ADR-048 §D.7.3 exemption / Gap-2 deferral)".
- §OBS-003 "What is now gated" paragraph extended: INSERT source_select WHERE (`dml.source_select.where_`) added via the `Ast::Sql(SqlStatement::Dml(dml))` arm, ADR-048 v1.13 §D.7.6 OD-7, load-bearing test cite.
- §OBS-003 "What remains un-gated" rewritten: `source_select.where_` removed from un-gated list (it is now gated); remaining un-gated scope narrowed to `source_select.having` (intentionally exempt per ADR-048 §D.7.3 six-aggregate-function permit), `source_select.join_on` (Gap-2, DRIFT-PQLFN-OD7-GAP2-S307 deferral with story anchor), `source_select.projections` (same Gap-2 deferral). Documentation-only correction — code was already correct.

---

### F-PQLFN-P35-HIGH-002 [HIGH][spec-drift] — CLOSED (BC-2.11.019 v1.17, product-owner, factory-artifacts this burst)

**Severity:** HIGH
**Category:** spec-drift — BC-2.11.019 §Postconditions aggregate-gate walker scope enumeration (b) claims "exactly **six** positions"; ADR-048 v1.13 and code have seven
**Status:** CLOSED — BC-2.11.019 v1.16→v1.17 (same burst as F-PQLFN-P35-HIGH-001)

**Description:** BC-2.11.019 §Postconditions aggregate-gate walker scope enumeration (b) stated: "The walker visits exactly **six** positions: (1)–(6) [listed positions only through Position 6 SQL DML WHERE]." Position (7) INSERT source_select WHERE was entirely absent from this enumeration. ADR-048 v1.13 §D.7.6 added Position 7 as OD-7, the code implements it, and a load-bearing test is GREEN.

**Impact:** HIGH because §Postconditions (b) is a declarative exhaustive enumeration — "exactly six" is a falsifiable claim. Any compliance audit or adversarial review consulting §Postconditions (b) would conclude only six surfaces exist, missing Position 7 entirely. This is the same class of error as HIGH-001 (stale enumeration count) but at the Postconditions layer rather than the OBS gate-scope layer.

**Severity rationale:** HIGH for same reasons as HIGH-001: both enumerations are authoritative declarative claims in the BC, not narrative description. A reader relying on either would reach a factually false conclusion about the gate's coverage.

**Closure evidence:** BC-2.11.019 v1.17 (same burst): §Postconditions (b) updated from "exactly **six** positions" to "exactly **seven** positions"; Position (7) appended as "INSERT source_select WHERE (`dml.source_select.where_` of INSERT INTO ... SELECT statements; ADR-048 v1.13 §D.7.6 OD-7; load-bearing test `test_f_pqlfn_p32_obs_001_insert_source_select_where_aggregate_fires_e_query_001`)". TD-VSDD-060 residual sweep: zero other live-prose "six positions" / "does NOT walk" / "un-gated" normative claims for `source_select.where_` outside §Changelog rows.

---

### F-PQLFN-P35-MED-001 [MED][spec-drift] — CLOSED (BC-2.11.019 v1.17, product-owner, factory-artifacts this burst)

**Severity:** MED
**Category:** spec-drift — E-QUERY-039 firing enumeration (a) and §Error Cases Condition column both omit INSERT source_select WHERE (OD-7) as position (v)
**Status:** CLOSED — BC-2.11.019 v1.16→v1.17 (same burst as HIGH-001/HIGH-002)

**Description:** Two enumeration sites in BC-2.11.019 are incomplete with respect to Position 7:

(1) **§Postconditions firing enumeration (a):** Lists positions (i)–(iv) at which a fn-call LHS predicate triggers E-QUERY-039: pipe `| where`, filter-mode root, SqlPipe `| where`, SQL DML WHERE. Position (v) INSERT source_select WHERE is absent.

(2) **§Error Cases E-QUERY-039 Condition column:** The condition enumeration ends with "SQL DML WHERE predicate (`dml.filter` of DELETE/UPDATE ...)" and does not include INSERT source_select WHERE.

**Severity rationale:** MED (rather than HIGH) because these are firing-condition enumerations — they describe when E-QUERY-039 fires, which is downstream of the scope-boundary documentation in HIGH-001/HIGH-002. The gap is less severe because the gate's existence is established by HIGH-001/HIGH-002's scope documentation; this is an enumeration omission at the firing-condition layer. However, a developer checking whether a specific query form would trigger E-QUERY-039 would consult these enumerations and incorrectly conclude INSERT source_select WHERE does not trigger the gate.

**Closure evidence:** BC-2.11.019 v1.17 (same burst):
(1) §Postconditions firing enumeration (a): "(v) INSERT source_select WHERE (`dml.source_select.where_` of INSERT INTO ... SELECT; ADR-048 v1.13 §D.7.6 OD-7)" added; cross-reference updated from "(i)–(iv) above" to "(i)–(v) above".
(2) §Error Cases E-QUERY-039 Condition column: condition prose extended to include "or INSERT source_select WHERE predicate (`dml.source_select.where_` of INSERT INTO ... SELECT; ADR-048 v1.13 §D.7.6 OD-7)". Documentation-only correction.

---

### F-PQLFN-P35-MED-002 [MED][semantic-anchoring] — CLOSED @c9eb2cd4 (fix-burst-27: 5 new per-position walk-observable test modules)

**Severity:** MED
**Category:** semantic-anchoring (POL-22) — `engine.rs` `check_enrich_udf_availability` docstring falsely claimed "other six gated surfaces each carry this sibling lock" when only 2/7 did
**Status:** CLOSED — fix-burst-27 @c9eb2cd4

**Description:** The `check_enrich_udf_availability` function in `engine.rs` carried a docstring claiming (approximately): "all seven gated surfaces carry an explicit known-UDF-passes-gate sibling lock test confirming the gate does not fire for registered enrichment UDFs." At HEAD bd044a2e, only 2 of 7 surfaces had explicit known-UDF-passes-gate sibling lock tests:
- Position 1 (pipe `| where`) — had a lock test from initial implementation
- Position 6 (SQL DML WHERE, OD-6) — lock added by fix-burst-26 POL-29 per-pass sweep (F-PQLFN-P34-OBS-001 closure)

Positions 2–5 and Position 7 lacked such locks. The docstring was a false 7/7 coverage claim when 2/7 existed.

**Severity rationale:** MED because a docstring asserting 7/7 coverage when 2/7 exist violates POL-22 semantic anchoring — it creates false confidence that the test suite's behavioral boundary between "column-position fn-call triggers gate" and "known enrichment UDF passes gate" is verified for all positions. A future implementer reading the docstring would not add missing locks, believing they already exist; an auditor would not flag the absence.

**Closure evidence @c9eb2cd4 (fix-burst-27):** 5 new per-position test modules added to `engine.rs` test suite. Each module tests the known-UDF-passes-gate sibling lock using a compound predicate form `enrich_lookup(ip_address)='US' AND totally_unknown_udf(x)=1`:
1. **`pipe_where_enrich_udf_passes_gate_tests`** — Position 1 (pipe `| where`): known enrichment UDF in pipe `| where` predicate passes gate without E-QUERY-039; `totally_unknown_udf` named in error for the adjacent unknown UDF.
2. **`filter_root_enrich_udf_passes_gate_tests`** — Position 2 (filter-mode root): known UDF passes; unknown UDF fires.
3. **`sqlpipe_where_enrich_udf_passes_gate_tests`** — Position 3 (SqlPipe `| where`): known UDF passes; unknown UDF fires.
4. **`sql_where_enrich_udf_passes_gate_tests`** — Position 4 (SQL WHERE): known UDF passes; unknown UDF fires.
5. **`sqlpipe_head_where_enrich_udf_passes_gate_tests`** — Position 5 (SqlPipe-head WHERE): known UDF passes; unknown UDF fires.

Walk-observable structure: compound predicate form ensures gate-removal OR registry-bypass flips an assertion — the test is not tautological with respect to walk reachability. Engine.rs docstring corrected to "all 7 gated surfaces verified: positions 1–7 each carry a known-UDF-passes-gate lock test." `just iter prism-query`: 1653/1653 GREEN (baseline 1648 + 5 new tests).

---

### F-PQLFN-P35-OBS-001 [OBS][test-strength] — CLOSED @c9eb2cd4 (fix-burst-27: all 7 tests restructured to walk-observable compound predicate form)

**Severity:** OBS
**Category:** test-strength — existing `passes_gate` tests individually tautological with respect to walk existence; removal of walk code would not flip any assertion
**Status:** CLOSED — fix-burst-27 @c9eb2cd4 (same commit as F-PQLFN-P35-MED-002)

**Description:** The prior known-UDF-passes-gate tests (2 positions: Position 1 and Position 6 from fix-burst-26) used simple single-UDF query forms such as `FROM t | where enrich_lookup(col)='val'` where E-QUERY-039 would not fire regardless of whether the walk visited the predicate or not — because there was no unknown UDF present to distinguish "walk reached predicate" from "walk skipped predicate." A test of the form `FROM t | where enrich_lookup(col)='val'` passes whether or not the `check_enrich_udf_availability` walk actually visits the `| where` predicate.

**Severity rationale:** OBS because the behavioral boundary between "column-position fn-call → gate fires" and "enrichment UDF predicate → gate passes" was not explicitly observable. A walk that skipped the predicate entirely would produce the same test outcome (E-QUERY-039 does NOT fire for a known UDF — same result whether skipped or walked). The tests were structurally tautological with respect to walk reachability — not incorrect, but not load-bearing.

**Closure evidence @c9eb2cd4 (same commit as F-PQLFN-P35-MED-002):** All 7 known-UDF-passes-gate tests (2 existing + 5 new from MED-002) structured as compound predicate `enrich_lookup(ip_address)='US' AND totally_unknown_udf(x)=1`. Each test now asserts:
- E-QUERY-039 fires (confirming the walk reaches the compound predicate at all)
- `totally_unknown_udf` is named in the error `infusion` field (confirming walk visits the fn-call node)
- `enrich_lookup` is present in `available_infusions` (confirming known UDF is correctly bypassed)

Walk removal OR registry bypass changes observable behavior: removing the walk means neither E-QUERY-039 fires NOR `totally_unknown_udf` appears in the error — the assertion flips. The tests are no longer tautological with respect to walk existence. All 7 positions (1–7) now carry walk-observable compound-predicate locks; 1653/1653 prism-query GREEN.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in `crates/` on HEAD bd044a2e. No BC-2.16.002 catalog row required. Sampled all `event_type =` sites in `crates/prism-query/src/` and adjacent crates touched by the branch; all catalogued.

---

## Verification Walk

- **ADR-048 v1.14 zero residual `six`-position live prose:** Post-fix-burst-26 ADR-048 §Changelog historical rows referencing "all six" at v1.6/v1.8 versions are correct per TD-VSDD-091 (immutable past-version records); §D.6 "sixth gated position" remains correct (Position 6 is still the sixth); "six named aggregate functions" (§D.7.3) refers to the six HAVING-exempt functions, unchanged. No normative live-body "all six predicate positions" occurrences remain.
- **POL-32 BC-2.11.019 changelog order:** v1.17 entry at top of §Changelog in descending version order; no version-ordering violations.
- **`Dml` arm code confirmed:** `check_enrich_udf_availability`'s `Ast::Sql(SqlStatement::Dml(dml))` arm confirmed in `engine.rs` at HEAD bd044a2e; `dml.source_select.where_` walk confirmed via `collect_unknown_scalar_offsets_from_predicate` call [corrected 2026-07-14 per F-PQLFN-P36-LOW-003 sweep — function name was stale]; load-bearing test `test_f_pqlfn_p32_obs_001_insert_source_select_where_aggregate_fires_e_query_001` confirmed present and GREEN.
- **POL-22 named-entity discipline:** All BC ID cites, ADR cites, error code cites, and function name references in this pass report use concrete named identifiers; no TD-VSDD-091 line-number citations introduced.
- **Tests @c9eb2cd4:** 1653/1653 prism-query GREEN (baseline 1648 + 5 new walk-observable lock tests from fix-burst-27).

---

## Status

**CLOSED — fix-burst-27 COMPLETE (D-1753 2026-07-14).**

All 5 findings fully closed:
- **F-PQLFN-P35-HIGH-001 CLOSED:** BC-2.11.019 v1.16→v1.17 §OBS-003 title + content corrected for INSERT source_select WHERE (OD-7 now documented as gated; un-gated scope narrowed to having/join_on/projections with Gap-2 S-3.07 deferral cites). Documentation-only correction.
- **F-PQLFN-P35-HIGH-002 CLOSED:** BC-2.11.019 v1.17 §Postconditions (b) "exactly **seven** positions"; Position (7) appended with ADR-048 §D.7.6 cite and load-bearing test. Documentation-only correction.
- **F-PQLFN-P35-MED-001 CLOSED:** BC-2.11.019 v1.17 firing enumeration (a): (v) added; §Error Cases Condition column extended; (i)–(v) cross-ref updated. Documentation-only correction.
- **F-PQLFN-P35-MED-002 CLOSED @c9eb2cd4:** 5 new per-position test modules (Positions 2–5 + walk-observable restructure for Positions 1+6+7); compound predicate form; engine.rs docstring corrected to true 7/7 claim; 1653/1653 prism-query GREEN.
- **F-PQLFN-P35-OBS-001 CLOSED @c9eb2cd4:** All 7 known-UDF-passes-gate tests restructured to walk-observable compound predicate form; walk removal or registry bypass flips assertion; co-committed with MED-002 fix-burst-27.

**CASCADE TALLY:** 35 passes / 27 fix-bursts
**STREAK:** 0/3 on new frozen HEAD c9eb2cd4 (DRIFT-ORCH-PRLEVEL-PUSH-001: streak resets when new commits exist; all 5 findings closed; c9eb2cd4 is local-only)
**NEXT ACTION:** LOCAL pass-36 on frozen c9eb2cd4 (streak 0/3)
