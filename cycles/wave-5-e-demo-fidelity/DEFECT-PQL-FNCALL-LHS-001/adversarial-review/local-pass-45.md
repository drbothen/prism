---
pass: 45
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 973aedcf
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: false
clean_pr_merge: true
finding_count: 1
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: fix-burst-34
fix_burst_pending: false
fix_burst_spec_only: true
fix_burst_bc: [BC-2.11.004]
---

# LOCAL Adversary Pass 45 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 973aedcf** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; UNCHANGED since fix-burst-33; first pass on this HEAD)
**CLEAN(strict): NO** (1 finding: 1 LOW — CLOSED by fix-burst-34)
**CLEAN(PR-merge): YES** (only LOW-severity finding; no CRIT/HIGH/MED)
**Streak: 0/3** (UNCHANGED — pass-45 NOT CLEAN(strict); BC-5.39.001 streak-reset rule; streak remains 0/3; pass-46 opens fresh on same frozen 973aedcf)
**Fix-Burst-34:** COMPLETE (spec-only @PO; no code change; HEAD 973aedcf UNCHANGED; pass-46 gates on same frozen 973aedcf)

---

## Pass-44 Closure Re-Verification

Pass-44 had 2 findings (F-PQLFN-P44-MED-001 SR-006 EC-11-013 namespace collision + F-PQLFN-P44-LOW-001 stale imperative comment), both CLOSED in fix-burst-33 (spec + code). Both closures VERIFIED SUBSTANTIVE:

- **F-PQLFN-P44-MED-001 (EC-11-013→EC-11-082 renumber; KEEPER=BC-2.11.005) — SUBSTANTIVE:** BC-2.11.004 v1.47 body has EC-11-082 at all 3 sites (§Postconditions, §Edge Cases ID cell, §Canonical Test Vectors category cell). BC-2.11.005 §Edge Cases EC-11-013 row is UNTOUCHED — KEEPER preserved. Code-comment sweep @973aedcf (4 crate sites: engine.rs:~6827, temporal_typing_tests.rs:~5375/~5402/~5413) verified: zero remaining EC-11-013 in prism-query crate. Namespace collision is structurally resolved.

- **F-PQLFN-P44-LOW-001 (engine.rs ~1587 stale imperative comment) — SUBSTANTIVE:** engine.rs ~1587 reads past-tense ADR-048 v1.4 retraction truth (TD-VSDD-059 paper-fix test: the comment is a doc-comment, not an assertion; the fix is the correct form — past-tense citation of a completed action replaces an imperative command for something already done). TD-VSDD-060 sibling site ~15128 also reworded @8389935c (2 sites confirmed). Implementer self-disclosure of risk severity NOT authoritative; adversary independently confirms load-bearing truth: ADR-048 v1.4 retraction IS documented in ADR-048 §Changelog; the past-tense citation is factually accurate.

---

## Findings

### F-PQLFN-P45-LOW-001 [LOW][POL-23 sweep miss] — OPEN → CLOSED (fix-burst-34)

**Affected artifact:** `stories/S-PRISMQL-CASE-INSENSITIVE-001-prismql-ieq-iin-ine-case-insensitive-operators.md` (4 sites), `specs/behavioral-contracts/BC-2.11.004-prismql-pipe-mode.md` (1 changelog row).

**Finding:** fix-burst-33's POL-23 story-sweep step grepped for the changed EC identifier (EC-11-013) rather than the BC version-pin string (BC-2.11.004 v1.46). POL-23 triggers on BC version bumps — its sweep target is the version-pin string, not the changed identifier inside the BC body. Because BC-2.11.004 was bumped v1.46→v1.47, POL-23 requires sweeping all story files for "BC-2.11.004 v1.46" citations and advancing them to v1.47. S-PRISMQL-CASE-INSENSITIVE-001 carries 4 live v1.46 pins (frontmatter BC status comment line ~74; §Behavioral Contracts body table version cell line ~171; §Token Budget row line ~189; AC-013b trace line ~407) — all missed.

Additionally, BC-2.11.004 v1.47 §Changelog contains an inaccurate sentence in the story-sweep rationale that describes the PO grepping EC-11-013 rather than the version-pin string. This sentence mischaracterizes the sweep that should have been run; it was corrected in place (changelog row content corrected; BC version remains v1.47 — no semantic delta, correction-of-authoring-narrative only per POL-29 changelog-correction exception).

**Root cause:** POL-23 sweep rationale error — the sweep grepper targeted the changed identifier (EC-11-013) rather than the version-pin string (BC-2.11.004 v1.46). The version-pin string is always the correct POL-23 sweep target; BC body identifier changes are swept by POL-29, which is a distinct sweep.

**Fix (fix-burst-34, product-owner; spec-only):**

1. BC-2.11.004 v1.47 §Changelog story-sweep sentence corrected in place: replaced the EC-11-013 grep rationale with the version-pin sweep rationale. BC version REMAINS v1.47 (changelog prose correction only; contract semantics unchanged).

2. S-PRISMQL-CASE-INSENSITIVE-001 v1.71→v1.72: 4 live BC-2.11.004 v1.46 pin sites updated to v1.47:
   - (1) frontmatter BC status comment line ~74
   - (2) §Behavioral Contracts body table version cell line ~171 (bare-cell form `| v1.46 |` → `| v1.47 |`)
   - (3) §Token Budget row line ~189
   - (4) AC-013b trace line ~407
   Exhaustive per-variant grep evidence (post-fix): Form A (`BC-2.11.004 v1.46`, space form): 0 live pins; Form B (`\| v1.46 \|`, bare cell form): 0 live pins; Form C (`v1.46` anywhere): remaining hits are historical changelog rows only. EC-11-013/EC-11-082 sibling sweep (POL-29): zero EC-11-013 or EC-11-082 references in this story file — no adjacent prose update needed. AC semantics UNCHANGED.

3. STORY-INDEX.md: version v2.684→v2.685; S-PRISMQL-CASE-INSENSITIVE-001 row updated to v1.72.

**Precedents:** POL-23 sweep rationale: the sweep target is always the BC version-pin string (e.g., "BC-2.11.004 v1.46") — not the changed identifier inside the BC body. BC body identifier changes are covered by POL-29. The version-pin string and the identifier are orthogonal sweep axes. (Source: this finding; codified as Lesson 56.)

**Status:** CLOSED — fix-burst-34 COMPLETE. BC-2.11.004 v1.47 changelog correction in place; story v1.72 4-site pin sweep complete; STORY-INDEX v2.685. HEAD 973aedcf UNCHANGED (spec-only). Pass-46 gates on same frozen 973aedcf.

---

## SAP-1 Result

**PASS.** Adversary ran `rg 'event_type\s*=' crates/ --type rust` against frozen HEAD 973aedcf. Total ripgrep output: 232 occurrences across 55 files in `crates/prism-query/` including doc-comments, test-assertions, and 2 SQL query-text strings. **Counting methodology note:** the stable adversary metric is distinct-live-emission-value count, not raw ripgrep count. The 232 raw hits decompose to: (a) ~35 files containing doc-comments/test-assertion occurrences (not production emission sites), (b) 2 SQL query-text string matches (not emission code), and (c) the 12 distinct live production emission values in `crates/prism-query/src/` at active `tracing::*!` call sites. All 12 distinct live values are catalogued in BC-2.16.002 §Postconditions (Canonical Structured Event Catalog). No new or removed emission sites in this pass window. No `event_type =` emission changes at frozen HEAD 973aedcf.

---

## Positive Verifications

- **ADR-048 v1.15 pin currency (13 sites) verified:** All 13 live normative ADR-048 citations in BC-2.11.004 v1.47 body reference v1.15. No stale v1.13 or earlier pin remains. Changelog chain verified monotonic v1.0–v1.15.

- **EnrichUdfNotFoundDetails `sanitize_for_log` load-bearing at construction:** BC-2.11.004 §Postconditions references `sanitize_for_log` at the `EnrichUdfNotFoundDetails` construction site. Verified in codebase: `EnrichUdfNotFoundDetails::new()` calls `sanitize_for_log` on the udf_name field at construction. Load-bearing (TD-VSDD-059): the sanitization occurs before the value is ever stored or propagated, ensuring the injection-safety invariant is structurally enforced at the entry point. Not a doc-comment-only annotation.

- **`fn_call_comparison` ordering preserved:** BC-2.11.004 §Canonical Test Vectors ordering row for `fn_call_comparison` verified present and accurate post-EC-11-082 rename. The EC-ID rename (EC-11-013→EC-11-082) did not alter the behavioral content of the ordering constraint. Implementation in `engine.rs` verified consistent with the renamed EC.

- **`DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate live:** BC-2.11.004 §Edge Cases EC-11-080 and the corresponding `DATAFUSION_BUILTIN_AGGREGATE_NAMES` set in `engine.rs` verified structurally consistent. The reserved-keyword gate for aggregate function names (EC-11-080, fix-burst-2) is present as a load-bearing `HashSet` check, not a doc-comment. `DATAFUSION_BUILTIN_AGGREGATE_NAMES` is defined and used in the gate arm per BC-2.11.004 §Edge Cases EC-11-080.

- **BC-2.11.005 KEEPER preserved:** BC-2.11.005 §Edge Cases EC-11-013 row (aggregate-pushdown cache-hit path; original senior allocator) is UNTOUCHED by all fix-bursts in this cascade. EC-11-013 in BC-2.11.005 remains the sole live allocation. The renumber was applied exclusively to BC-2.11.004 (the collider). Namespace collision fully resolved.

- **POL-22 Phase A+C pass:** BC-2.11.004 v1.47 code-truth check: §Postconditions behavioral promises cross-referenced against `crates/prism-query/src/engine.rs` gate arm structure. Phase A: BC prose → code match (fn_call gate present; EC-11-082 behavior accurate). Phase C: code path → BC coverage (no undocumented gate arms in the fn_call_comparison branch). Pass confirmed.

---

## Status

```
NOT CLEAN(strict) — pass 45 complete. 1 finding (1 LOW). CLOSED fix-burst-34.

CASCADE TALLY: 45 passes / 34 fix-bursts

STREAK: 0/3 (UNCHANGED — pass-45 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY.
Frozen HEAD for pass-46: 973aedcf UNCHANGED (fix-burst-34 spec-only; no code commit).

FIX-BURST-34 STATUS (COMPLETE; spec-only):
  F-PQLFN-P45-LOW-001: CLOSED — BC-2.11.004 v1.47 changelog sentence corrected (PO; in-place
                                 correction; BC version unchanged). Story v1.71→v1.72:
                                 4 BC-2.11.004 v1.46→v1.47 pin sites updated (frontmatter
                                 comment; §BC body table version cell; §Token Budget row;
                                 AC-013b trace). STORY-INDEX v2.684→v2.685.

FINDINGS BREAKDOWN:
  LOW: 1 (F-PQLFN-P45-LOW-001 POL-23 sweep miss: EC-ID grep vs version-pin-string grep) — CLOSED
  Total: 1

CLEAN(strict): NO (pass-45 finding drove fix-burst-34; pass-46 gates on same frozen HEAD)
CLEAN(PR-merge): YES (zero CRIT/HIGH/MED findings)

NEXT ACTION: LOCAL pass-46 on SAME frozen HEAD 973aedcf (streak 0/3; BC-2.11.004 v1.47 live;
             HEAD UNCHANGED — fix-burst-34 spec-only; Lesson 56 appended)
```
