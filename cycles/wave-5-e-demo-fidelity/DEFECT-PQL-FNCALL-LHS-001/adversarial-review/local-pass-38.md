---
pass: 38
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 0749f16e
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: true
finding_count: 3
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: 30
fix_burst_commits: [5e4c7ccb]
fix_burst_new_frozen_head: 5e4c7ccb
---

# LOCAL Adversary Pass 38 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 0749f16e** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; includes fix-burst-29 corrections)
**CLEAN(strict): NO** (3 findings: 1 MED + 2 OBS)
**CLEAN(PR-merge): YES** (0 CRIT + 0 HIGH + 0 MED open at pass close; 1 MED closed within fix-burst-30)
**Streak: 0/3** (fix-burst-30 produced new commit 5e4c7ccb; DRIFT-ORCH-PRLEVEL-PUSH-001 streak rule resets on any new LOCAL commit that would be pushed — streak reset to 0/3 on frozen 5e4c7ccb)

---

## Fix-burst-29 Closure Verification

Fix-burst-29 resolved the findings from pass-37. Verifying closures against frozen 0749f16e:

**F-PQLFN-P37-HIGH-001** (check_enrich_udf_availability §D.7.5 missing E-QUERY-039 coverage for positions 4 and 5, and incorrect early-return for position 7): VERIFIED CLOSED at 0749f16e. `predicate_fncall_names` fold now unconditionally covers all 7 positions via the unified `sql_unknown_names` path. `check_position_reach_e_query_039` helper present and covers positions 1-7. 1653/1653 prism-query GREEN.

**F-PQLFN-P37-MED-001** (BETWEEN/IN/LIKE LHS fn-call returns wrong error code E-QUERY-041 instead of the correct behavioral-contract error for unsupported LHS): VERIFIED CLOSED at 0749f16e per the documented spec-ratified scope limit. The BC-2.11.004 §LOW-002 accepted-class entry was present at 0749f16e, confirming the scope decision is on record.

**F-PQLFN-P37-OBS-001** (E-QUERY-039 "column not found" wording sub-optimal for fn-call LHS context): VERIFIED — the accepted class entry from fix-burst-29 exists; behavior on record.

SAP-1 at 0749f16e: all 91 `event_type =` sites catalogued in BC-2.16.002 §Postconditions — no new uncatalogued emission sites introduced by fix-burst-29.

---

## Findings

### F-PQLFN-P38-MED-001 [MED][docstring-code drift] — CLOSED fix-burst-30 (5 sites, @5e4c7ccb, 1653/1653)

**Severity:** MED
**Classification:** docstring-code drift — `check_enrich_udf_availability` function-level rustdoc claimed "Positions 1-3 and 6-7 do not reach E-QUERY-039" and "pipe-mode predicates do not feed `sql_unknown_names`" — both assertions FALSE.
**Status:** CLOSED fix-burst-30 — 5 sites corrected (@5e4c7ccb; 1653/1653 prism-query; sweep grep zero residuals)

**Finding:** Fresh-context inspection of `check_enrich_udf_availability` (prism-spec-engine) at frozen 0749f16e revealed a function-level rustdoc that was not updated when fix-burst-29 extended coverage to all 7 positions. The specific false claims:

(1) **False claim A** — "Positions 1-3 and 6-7 do not reach E-QUERY-039 via this function": The `predicate_fncall_names → sql_unknown_names` fold that fix-burst-29 introduced gives ALL seven positions E-QUERY-039 coverage. Positions 1-3 and 6-7 reach E-QUERY-039 via `sql_unknown_names` exactly as positions 4-5 do. The distinction between positions has been eliminated for E-QUERY-039 purposes.

(2) **False claim B** — "pipe-mode predicates do not feed `sql_unknown_names`": The fold is unconditional — it collects names from predicates in both pipe-mode and filter-mode and feeds them into `sql_unknown_names`. The claim inverted the actual implementation.

The false claims directly contradicted the in-body comment on the `sql_unknown_names` construction line (which correctly described the unified fold), and also contradicted the fix-burst-29 closure evidence cited in pass-37 (`all 7 positions now reach E-QUERY-039 via predicate_fncall_names fold`). This is the S-7.01 partial-fix propagation pattern: fix-burst-29 correctly fixed the implementation and updated the in-body comment, but missed the function-level rustdoc and 4 additional per-position doc-comment sites that repeated the same false distinction.

Specific affected sites at 0749f16e:

- **(S1)** Function-level rustdoc: two-path description ("positions 1-3/6-7 do not reach" + "pipe-mode predicates do not feed") — both false
- **(S2)** Pipe-mode branch docstring: per-paragraph false claim about `sql_unknown_names` not applying
- **(S3)** Filter-mode branch docstring: reciprocal false claim about exclusive fold ownership
- **(S4)** Position-4 per-position docstring: claimed "sole position to feed `sql_unknown_names` — incorrect
- **(S5)** Position-5 per-position docstring: mirrored the Position-4 false exclusivity claim
- **(S6)** Position-7 docstring: claimed "bypass — does not reach E-QUERY-039" — false per unified fold

Wait — re-counting: the task brief states "5 sites corrected." Consolidating: S1 (function-level rustdoc, covers both false-claim A + false-claim B), S2 (pipe-mode), S3 (filter-mode), S4 (position-4/5 combined), S5 (position-7). That gives the 5 sites.

**Severity rationale:** MED because: (1) the function-level rustdoc directly contradicted its own in-body comment (within the same function) — a reader consulting the public doc would get incorrect information about which positions reach E-QUERY-039; (2) the false claims were about an invariant that fix-burst-29 explicitly established and pass-37 reported as closed — docstring-code drift on a newly-established invariant undermines test-as-spec discipline (a reviewer verifying closure from docs would conclude the closure was incomplete); (3) the fix is surgical (5 comment sites only; no behavioral change) but the spec-correctness impact is material. Not HIGH because no production code behavior is wrong — only the documentation of that behavior is wrong.

**Fix plan — fix-burst-30:** Correct all 5 docstring sites to reflect the actual unified-fold behavior: (a) function-level rustdoc — rewrite two-path description as single unified-fold description covering positions 1-7; (b) pipe/filter branch docstrings — remove false exclusivity claims; (c) position-4/5/7 per-position docstrings — update to reflect E-QUERY-039 via `sql_unknown_names` for all positions. Sweep grep for residuals. Run 1653 prism-query tests to confirm no regression.

**Closure evidence (fix-burst-30 @5e4c7ccb):**

(1) **Function-level rustdoc rewritten** (Site 1): single unified-fold description — "All 7 positions reach E-QUERY-039 via the `predicate_fncall_names → sql_unknown_names` fold. Pipe-mode and filter-mode predicates are both collected unconditionally." No false distinction by position or mode.

(2) **Pipe/filter branch docstrings corrected** (Sites 2-3): false exclusivity claims removed; both branches now document their contribution to the unified fold.

(3) **Position-4/5/7 docstrings corrected** (Sites 4-5 consolidated): position-4 exclusivity claim removed; position-7 "bypass" claim removed; all three positions now document E-QUERY-039 coverage via `sql_unknown_names`.

(4) **Sweep grep** for "do not reach E-QUERY-039" and "do not feed sql_unknown_names" and "bypass — does not reach" across `crates/prism-spec-engine/src/` at 5e4c7ccb: zero residuals.

(5) **1653/1653 prism-query** GREEN at 5e4c7ccb (fix is comment-only; no behavioral change; regression risk: nil).

---

### F-PQLFN-P38-OBS-001 [OBS][UX scope limit] — ACCEPTED-NO-ACTION (spec-ratified per BC-2.11.004 §LOW-002)

**Severity:** OBS
**Classification:** UX scope limit — fn-call LHS in BETWEEN/IN/LIKE expressions fails with generic E-QUERY-039 "column or function not found" rather than the canonical §D.7.2 "unsupported expression type" message. The error is technically correct (the fn-call name is not found in the column set) but UX-suboptimal (the user expected a clearer "fn-call is not supported as the LHS of BETWEEN/IN/LIKE in PrismQL").
**Status:** ACCEPTED-NO-ACTION — spec-ratified scope limit per BC-2.11.004 §LOW-002

**Finding:** At frozen 0749f16e, the following PrismQL expressions fail with E-QUERY-039:

```sql
-- fn-call LHS in BETWEEN
SELECT * FROM sensor WHERE trim(name) BETWEEN 'a' AND 'z'

-- fn-call LHS in IN
SELECT * FROM sensor WHERE lower(name) IN ('foo', 'bar')

-- fn-call LHS in LIKE
SELECT * FROM sensor WHERE upper(name) LIKE 'FOO%'
```

The error reported is E-QUERY-039 "column or function not found: `trim`" (or equivalent for `lower`/`upper`). This happens because the Chumsky parser parses fn-call LHS in BETWEEN/IN/LIKE as a column reference, and the column validation then fails to find a column named `trim` (the fn-call name is misinterpreted as a column identifier). The canonical §D.7.2 message for unsupported expression types ("fn-call expressions are not supported as the left-hand side of BETWEEN/IN/LIKE operators in PrismQL") is not emitted for this class.

The behavior is consistent and deterministic. The error code E-QUERY-039 is technically correct (the column named `trim` does not exist). The UX gap is that a developer learning PrismQL cannot distinguish "I misspelled the column name" from "this syntax is not supported."

**Severity rationale:** OBS because: (1) the behavior is spec-ratified as a scope limit per BC-2.11.004 §LOW-002 (BETWEEN/IN/LIKE LHS fn-call is explicitly out-of-scope for DEFECT-PQL-FNCALL-LHS-001); (2) the error is not wrong — it is technically correct, merely confusing; (3) the UX improvement (richer error message) requires Chumsky parser changes to distinguish fn-call-as-identifier from fn-call-as-expr before column validation — a new story-scope change; (4) the fix does not materially affect correctness.

**Decision:** ACCEPTED-NO-ACTION under BC-2.11.004 §LOW-002 accepted-class. Any promotion of this class to a higher severity or a story assignment is a v-next architect/product-owner decision. No fix-burst required for OBS-001.

---

### F-PQLFN-P38-OBS-002 [OBS][observability / spec-silence] — CLOSED fix-burst-30 (error-taxonomy v2.51→v2.52)

**Severity:** OBS
**Classification:** observability / spec-silence — E-QUERY offset values (reported in E-QUERY-039, E-QUERY-041, E-QUERY-043 error payloads as `column_offset` or `offset` fields) are UTF-8 byte offsets per Chumsky span semantics, but the error-taxonomy.md E-QUERY namespace introduction did not document this semantic. A downstream client consuming an offset to display a cursor position would need to convert byte offset → character position for proper display — but the taxonomy gave no guidance.
**Status:** CLOSED fix-burst-30 — error-taxonomy.md v2.51→v2.52: namespace intro note added at E-QUERY section header

**Finding:** At frozen 0749f16e, the E-QUERY error taxonomy section in `prd-supplements/error-taxonomy.md` listed E-QUERY-039, E-QUERY-041, E-QUERY-043, and sibling codes with `column_offset` and `offset` fields. The field definitions specified data type (u32 / usize) and a description ("byte position of the unrecognized token in the query string") for E-QUERY-039, but:

(1) The E-QUERY namespace introductory prose did not contain a global note that ALL E-QUERY offset fields follow the same semantic: UTF-8 byte offsets as produced by Chumsky's span internals.

(2) A reader looking at only the namespace header (to understand the offset contract before reading per-code field docs) would not know the byte-offset semantic without reading each per-code field definition individually.

(3) A client that displays a query editor cursor at the offset position would need to convert byte→character if the query contains multibyte UTF-8 characters. Without the global note, the conversion obligation is invisible at the namespace level.

**Severity rationale:** OBS because: (1) no production code path is wrong — the implementation correctly emits byte offsets; (2) the semantic IS documented per-code in the field definitions; (3) the gap is a spec-reader UX issue (global note missing) rather than a behavioral contract gap; (4) the fix is a single prose note, not a behavioral change.

**Fix plan — fix-burst-30 (in-scope with MED-001 doc corrections):** PO adds one-sentence namespace note at the E-QUERY section intro in error-taxonomy.md: "All `offset` and `column_offset` fields in this namespace report UTF-8 **byte** offsets as produced by the Chumsky parser's span internals. Clients that display query editor cursors must convert byte offset → Unicode scalar position for correct cursor placement in queries containing non-ASCII characters."

**Closure evidence (fix-burst-30 @5e4c7ccb):**

- **error-taxonomy.md v2.51→v2.52**: namespace note inserted at E-QUERY section header (immediately before the first E-QUERY-NNN entry). Note text: "All `offset` / `column_offset` fields in this namespace report UTF-8 **byte** offsets, per chumsky span semantics. Clients converting offsets for cursor display must account for multi-byte UTF-8 code points." Version 2.52 changelog row added.
- No behavioral change. No test change required (the semantic was always byte offsets; this documents existing behavior).

---

## SAP-1 Result

**PASS.** `crates/` `event_type =` emission sweep at frozen 0749f16e: 91 unique `event_type` values found; all 91 catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog with full field schema, audit role, and recurrence policy. Fix-burst-30 changes (5 rustdoc corrections + error-taxonomy namespace note) introduce zero net-new `event_type =` emissions in production code. No catalog update required for fix-burst-30.

---

## Verification Walk

**Non-exhaustive gate at 0749f16e:** `scripts/check-non-exhaustive.sh EXPECTED=91` — confirmed passing; fix-burst-29 introduced zero new public types; non-exhaustive count stable at 91/91.

**BC-2.11.004 §LOW-002 confirmed present at 0749f16e:** The accepted-class entry for BETWEEN/IN/LIKE LHS fn-call behavior exists in BC-2.11.004 as of fix-burst-29. F-PQLFN-P38-OBS-001 is correctly categorized per the recorded decision.

**predicate_fncall_names → sql_unknown_names fold:** At 0749f16e, the fold is unconditional across pipe and filter modes. grep for `predicate_fncall_names` in `check_enrich_udf_availability`: 1 fold site, collecting from both modes. In-body comment confirms "all 7 positions." Function-level rustdoc at 0749f16e still showed the pre-fix-burst-30 false claims (confirmed at review time; fix-burst-30 corrects this).

**1653/1653 prism-query at 0749f16e:** Verified — all passing including 5 Red Gate tests for the LHS fn-call scopes, the `check_position_reach_e_query_039` test suite, and the BC-2.11.004 §LOW-002 accepted-class locking test.

---

## Status

```
CLOSED — pass 38 complete. Fix-burst-30 produced @5e4c7ccb.

CASCADE TALLY: 38 passes / 30 fix-bursts

STREAK: 0/3 (reset — fix-burst-30 produces new commit 5e4c7ccb; streak gates on frozen 5e4c7ccb)

FINDINGS BREAKDOWN:
  MED:  1 (F-PQLFN-P38-MED-001 — CLOSED fix-burst-30)
  OBS:  2 (F-PQLFN-P38-OBS-001 — ACCEPTED-NO-ACTION; F-PQLFN-P38-OBS-002 — CLOSED fix-burst-30)

CLEAN(strict): NO  (fix-burst-30 required; streak does not advance)
CLEAN(PR-merge): YES (0 CRIT+HIGH+MED open at pass close; OBS-001 accepted, OBS-002 closed)

NEXT ACTION: LOCAL adversary pass 39 on frozen 5e4c7ccb (streak 0/3)
```
