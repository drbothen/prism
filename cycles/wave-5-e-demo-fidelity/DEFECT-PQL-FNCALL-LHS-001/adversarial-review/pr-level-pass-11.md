---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [11]
feature_head_at_review: 74932b99
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 4
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 2
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 3
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 11 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 11 (frozen 74932b99; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

Streak: **0/3** (BC-5.39.001 strict criterion: pass-11 has 1 MED in-perimeter finding — merge-blocking; fix-burst-42 pushed commit 9c99e54f; DRIFT-ORCH-PRLEVEL-PUSH-001: re-gate on frozen HEAD 9c99e54f for all subsequent passes)

Cascade tally (as of this pass): **11 passes / 7 fix-bursts** (PR-LEVEL); **58 passes / 42 fix-bursts** (total)

CLEAN(strict): NO — 1 in-perimeter finding (1 MED + 1 LOW + 2 OBS)
CLEAN(PR-merge): NO — MED finding is MERGE-BLOCKING

**Attack angles for this pass:**
- (A) Spec–code alignment: do the spec artifacts that governed fix-burst-41 accurately describe the shipped implementation? (ADR-048 §D.7.2 Form-B convention vs the semantic-flag normalization actually shipped)
- (B) Code correctness of the `semantic` flag propagation chain: are there any other sites where the flag would need to be consumed or set that were missed?
- (C) Green-lock attempt: write byte-exact Display lock tests exercising the in-perimeter security-limit paths (E-QUERY-003 / EC-004) against the semantic-flag change

---

## Findings

### F-PQLFN-PR11-MED-001 — ADR-048 §D.7.2 Form-B convention spec-code drift (closed by fix-burst-42)

**Severity:** MED (MERGE-BLOCKING)
**Category:** spec-code drift / CLAUDE.md rule-7 spec-wins adjudication
**Routing:** architect (ADR-048 v1.16→v1.17; spec retraction; CLOSED fix-burst-42 @16a6ec85)

**Defect:** ADR-048 v1.16 §D.7.2 still ratified the "Form-B two-form convention" (Chumsky double-nested prefix is load-bearing for recovery guard). Fix-burst-41 shipped a different mechanism: `ParseError.semantic: bool` discriminant replaces the starts_with string probe; `materialization.rs` uses `strip_prefix("E-QUERY-001: ")` on semantic errors; LOW-006 is de-prefixed at emit. The Form-B rationale ("prefix is load-bearing for recovery") no longer holds — the recovery guard was ported to `e.semantic`.

ADR-048 §D.7.2 declared the doubled-prefix form canonical, but the code now single-prefixes all outputs. This is a spec-leads-code conflict that must be resolved by retracting the Form-B rationale from spec (CLAUDE.md rule 7: spec wins → spec must be corrected to reflect the shipped design).

**Resolution:** architect retracted §D.7.2 Form-B convention; documented semantic-flag design as the normative mechanism. ADR-048 v1.16→v1.17.

**BC references:** ADR-048 v1.16 §D.7.2; BC-2.11.004 v1.48 EC-11-085 (byte-exact canonical); BC-5.39.001 3-CLEAN criterion

---

### F-PQLFN-PR11-LOW-001 — sql_parser `has_semantic_error` comment overstates de-prefix scope (closed by fix-burst-42)

**Severity:** LOW
**Category:** doc-accuracy / comment drift
**Routing:** implementer (comment fix in commit 9c99e54f)

**Defect:** The `has_semantic_error` function in `crates/prism-query/src/sql_parser.rs` carried a comment describing its probe as "checks if error message starts with E-QUERY-001:" — but fix-burst-41 changed the probe from a `starts_with("E-QUERY-001:")` string check to `e.semantic`. The comment described the old fragile string-probe mechanism, not the shipped semantic-flag mechanism. Any reader maintaining this code would be misled about the actual discriminator.

**Resolution:** comment sharpened to describe the `e.semantic` flag check. Committed at 9c99e54f.

---

### F-PQLFN-PR11-OBS-001 — mixed-code Display E-QUERY-003/EC-004 two-layer form (pre-existing; OBS adjudication; CLOSED fix-burst-42)

**Severity:** OBS (non-blocking per BC-5.39.001 strict criterion — see CLEAN(strict)/CLEAN(PR-merge) disambiguation)
**Category:** Display output correctness / canonical form adjudication
**Routing:** PO (BC-2.11.006 v1.19→v1.20; error-taxonomy v2.52→v2.53; CLOSED fix-burst-42 @16a6ec85)

**Finding:** The `regex_match` try_map callback (E-QUERY-003 length-limit check) and the `build_source_ref_parser` EC-004 path-traversal try_map callback both flow through the `materialization.rs` `QueryParseFailed` detail builder. With the fix-burst-41 `strip_prefix` logic, when these inner-code prefixed messages arrive (e.g., `"E-QUERY-003: regex too long"` or `"EC-004: illegal path segment"`), `strip_prefix("E-QUERY-001: ")` does NOT match — the detail builder falls through to `e.message` directly, producing the two-layer form:

```
E-QUERY-001: query parse error at offset {N}: E-QUERY-003: regex too long
```

Vector A examination: this two-layer form is **pre-existing behavior** for the E-QUERY-003 / EC-004 paths and is not a regression of fix-burst-41. Fix-burst-41 only changed the LOW-006 RESERVED_KEYWORDS path. The two-layer form is the intended security-limit surface: the outer layer identifies the parse context; the inner layer identifies the specific security constraint.

**Options presented to product-owner:**
- **Option A:** Generalize `strip_prefix` to strip any `"E-QUERY-{NNN}: "` prefix — normalizes to single-prefix but loses the security constraint identity in the outer display context.
- **Option B (RATIFIED):** Treat the two-layer form as canonical for in-perimeter security-limit errors (E-QUERY-003 / EC-004). Document in BC-2.11.006 and error-taxonomy. Byte-exact templates + test vectors added. The mixed-code form is informative: clients see both the parse-context error code and the security-constraint code.
- **Option C:** Restructure try_map to return a ParseError directly without going through the materialization layer — scope expansion beyond fix-burst-42.

**PO ratification:** Option B RATIFIED. BC-2.11.006 v1.19→v1.20 documents the two-layer Display as canonical for security-limit wrapped errors. error-taxonomy v2.52→v2.53 cross-references BC-2.11.006 v1.20.

**NOTE (pre-existing classification):** This finding applies to the state that existed BEFORE fix-burst-41 as well. It is OBS rather than MED because the two-layer form correctly communicates security constraint context and no client compatibility regression has been demonstrated.

---

### F-PQLFN-PR11-OBS-002 — ParseError lacked `#[non_exhaustive]`; public `semantic` field added (CLOSED fix-burst-42)

**Severity:** OBS (non-blocking)
**Category:** API surface hygiene / `#[non_exhaustive]` discipline
**Routing:** PO + implementer (BC-2.11.019 v1.23→v1.24 §OBS-005; gate 91→92; CLOSED fix-burst-42 @16a6ec85 + 9c99e54f)

**Finding:** Fix-burst-41 introduced `pub semantic: bool` as a new public field on `ParseError` in `crates/prism-query/src/error_recovery.rs`. `ParseError` is a `pub` type in `prism-query`. Under the project `#[non_exhaustive]` discipline (CLAUDE.md §Conventions), all public TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`. The `semantic` field addition without `#[non_exhaustive]` would allow downstream code to construct `ParseError` structs directly — a pub-API surface violation.

**Resolution:** `#[non_exhaustive]` added to `ParseError` by implementer at 9c99e54f. Compile-fail gate case added. Both `check-non-exhaustive.sh` and `check-non-exhaustive-per-symbol.py` updated: EXPECTED 91→92. BC-2.11.019 v1.23→v1.24 §OBS-005 checklist documents the gate advancement.

---

### Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1–10; not re-raised as a PR-LEVEL finding.

---

## Three Lock-Exposed Production Defects (distinct from the 4 review findings above)

The GREEN-lock writing discipline (Vector C) exposed **three production defects** that 11 passes of static analysis had not caught. These are recorded here because their discovery mechanism is itself a process-insight: byte-exact lock tests are a defect-discovery instrument, not merely a coverage instrument.

### CRIT-class — regex_match `try_map` at wrong span level (E-QUERY-003 length-limit error suppressed)

**Root cause:** The `regex_match` function in `crates/prism-query/src/filter_parser.rs` applied the length-limit `try_map` callback at the outer parser combinator span, not at the literal level. This caused Chumsky's `choice()` span-merging to resolve competing errors by selecting the `kw("IN")` error over the E-QUERY-003 length-limit error when both branches were attempted in the same input position. The effect: queries with a regex literal that exceeded the length limit would produce an `IN` keyword error instead of the E-QUERY-003 length-limit error.

**Fix:** `try_map` moved to the literal level (IEQ/INE pattern) so the E-QUERY-003 error is attached to the regex literal itself, not the outer span. Span-merging no longer suppresses it.

### HIGH-class — EC-004 path-traversal check structurally DEAD CODE in `build_source_ref_parser`

**Root cause:** The `build_source_ref_parser` grammar in `crates/prism-query/src/filter_parser.rs` restricted the segment grammar to `[a-zA-Z0-9._-]` via the `filter` combinator. The EC-004 path-traversal `try_map` that checked for `..` and `/` characters could therefore never fire — the grammar made those characters unparseable before the `try_map` was reached.

**Fix:** Grammar widened to accept arbitrary printable characters at parse time; EC-004 `try_map` moved downstream where it operates on the accepted string and fires correctly on path-traversal inputs.

### MED-class — structural (non-semantic) errors double-prefixed via materialization detail builder

**Root cause:** The `QueryParseFailed` detail builder in `materialization.rs` applied the canonical `"E-QUERY-001: query parse error at offset {N}: "` prefix to ALL errors regardless of the `e.semantic` flag. Structural parse errors (`ExpectedFound` reason) produced by Chumsky's choice parser already had a `"parse error at offset {N}: "` segment baked into their display form. The detail builder then prepended `"E-QUERY-001: query parse error at offset {N}: "` on top, yielding the double-prefix form.

**Fix:** Detail builder path updated: for semantic errors (`e.semantic == true`), use `e.message` directly (strip_prefix behavior for known-prefix semantic errors; pass-through for inner-code-prefixed security-limit errors). For structural errors, the `ParseError::Display` output is used as-is — already in the correct single-prefix form.

---

## Fix-Burst-42 Closure Trail

All in-perimeter findings closed same-session. Fix-burst-42 split across two commits: spec/index layer @16a6ec85 (architect + PO) and implementation layer @9c99e54f (implementer).

### Spec/Index layer (@16a6ec85, committed to fix/DEFECT-PQL-FNCALL-LHS-001)

- **Architect — ADR-048 v1.16→v1.17:** §D.7.2 Form-B two-form convention RETRACTED. Semantic-flag normalization documented as normative mechanism. `ParseError.semantic: bool` discriminant set via `matches!(err.reason(), RichReason::Custom(_))`; `materialization.rs` strips `"E-QUERY-001: "` from semantic message detail via `strip_prefix`; `sql_parser.rs` `has_semantic_error` updated to `e.semantic` flag check. Mixed emitter regime documented: LOW-006 de-prefixed at emit; IIN/IEQ/INE keep prefix per BC-2.11.024, normalized at strip. POL-23 sweep: BC-2.11.004 v1.16 live pins → v1.17 (23 code sites); BC-2.11.019 v1.16×4 live pins → v1.17 (44 sites). ARCH-INDEX cell updated.

- **Product-owner — BC-2.11.006 v1.19→v1.20:** F-PQLFN-PR11-OBS-001 Option B ratification. Two-layer wrapped Display canonical for in-perimeter try_map errors documented. Byte-exact templates + test vectors. BC-2.11.006 v1.20 cross-reference added to error-taxonomy.

- **Product-owner — error-taxonomy v2.52→v2.53:** BC-2.11.006 v1.20 cross-reference. E-QUERY-001 Description paragraph added documenting the two-layer Display form for security-limit wrapped errors.

- **Product-owner — BC-2.11.019 v1.23→v1.24:** §OBS-005 checklist entry added for `ParseError` `#[non_exhaustive]` gate advancement (EXPECTED 91→92; compile-fail gate entry; gate scripts both updated 91→92).

### Implementation layer (@9c99e54f, pushed to origin/fix/DEFECT-PQL-FNCALL-LHS-001)

**Implementer fixes (TDD green-lock):**

1. **`filter_parser.rs` — `regex_match` `try_map` moved to literal level:** IEQ/INE pattern — the length-limit `try_map` now attaches to the regex literal parser directly, not the outer combinator span. E-QUERY-003 error no longer suppressed by choice() span-merging.

2. **`filter_parser.rs` — `build_source_ref_parser` grammar widened + EC-004 `try_map` reachable:** Segment grammar expanded beyond `[a-zA-Z0-9._-]`; EC-004 `try_map` for `..` and `/` path-traversal moved downstream to operate on accepted strings — defect closed, gate now reachable.

3. **`materialization.rs` — detail builder uses `e.message` for semantic errors:** For `e.semantic == true` errors, the detail is taken directly from `e.message` (with `strip_prefix("E-QUERY-001: ")` applied for known-prefix semantic errors; pass-through for inner-code-prefixed security-limit errors like E-QUERY-003/EC-004). Structural errors use `ParseError::Display` as-is.

4. **Pin sweeps:** ADR-048 v1.16→v1.17 (23 code sites); BC-2.11.019 v1.23→v1.24 (44 sites). Both per POL-23/POL-29 sweep discipline.

5. **`ParseError` `#[non_exhaustive]`:** Added at `error_recovery.rs`. Compile-fail case added to `tests/external/non-exhaustive-violation/`. Both gate scripts updated: EXPECTED 91→92. gate 92/92 on branch.

6. **sql_parser `has_semantic_error` comment sharpened** (LOW-001 closure): comment updated to describe `e.semantic` flag check.

7. **3 new Display lock tests:**
   - 2 in `regression_tests.rs` — E-QUERY-003 length-limit lock (regex too long triggers error, not `IN` keyword error) + EC-004 path-traversal lock (illegal path segment triggers EC-004, not parse suppression)
   - 1 in structural lock (single-prefix structural parse error lock; verifies structural errors are NOT doubly prefixed)

**Post-fix:** 3/3 GREEN; full suite 1683/1683 prism-query (+3 over 1680); non-exhaustive 92/92; pre-push `just check` GREEN.

---

## Positive Verifications

- **Vector A fully addressed:** ADR-048 v1.17 retracts the Form-B two-form convention. §D.7.2 now documents the semantic-flag mechanism as normative. Any reintroduction of the double-prefix in the LOW-006 path will fail the all-21-keyword single-prefix count loop (from fix-burst-41 pass-10).

- **Vector B propagation chain — GREEN:** `ParseError.semantic` is set at construction (`error_recovery.rs`), propagated through `shift_parse_error_offsets` (fix-burst-41 existing), consumed at `materialization.rs` detail builder and `sql_parser.rs` `has_semantic_error`. No additional consumption sites found in the codebase grep at 74932b99. The flag is not consumed externally (the struct is `#[non_exhaustive]`; external match arms cannot destructure).

- **Vector C lock-writing — lock-exposed 3 production defects:** All three fixed at 9c99e54f. Three load-bearing tests enforce each fix. 11 passes of static adversarial review had not detected these defects because they required exercising the Display chain from first principles against byte-exact expected outputs — not reachable by static reasoning alone.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions in fix-burst-42. No production emission sites modified. Settled methodology carries from prior passes: 55 raw occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog; ZERO net-new emissions.

- **TD-VSDD-059 PASS (fix-burst-42):** All 4 closures (MED-001 ADR spec-retraction + LOW-001 comment + OBS-001 BC+taxonomy + OBS-002 gate) have load-bearing artifacts: ADR-048 v1.17 is structural retraction (not rename); BC-2.11.006 v1.20 byte-exact templates; gate 92/92 structurally fails on any removal of `#[non_exhaustive]` from `ParseError`.

- **TD-VSDD-060 PASS (fix-burst-42 implementer):** `ParseError` new `#[non_exhaustive]` — sibling sweep: `shift_parse_error_offsets` (propagates struct unchanged), `materialization.rs` detail builder (reads `e.semantic`), `sql_parser.rs` `has_semantic_error` (reads `e.semantic`). All 3 sites swept in-commit. `regex_match` try_map moved — `filter_parser.rs` single-site; no sibling callsites. `build_source_ref_parser` widened — single-site parser combinator; no external callsites.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **CLAUDE.md forbidden patterns clean:** No `println!` in production code paths. No new `unwrap()`/`expect()` in production code. No `reqwest` changes (ADR-050 rustls-tls untouched). No AI attribution in commits.

- **Spec versions at 74932b99 (pre-fix-burst-42):**
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords)
  - BC-2.11.006 v1.19 (pre-fix-burst-42; STALE — actual shipped v1.20 after @16a6ec85)
  - BC-2.11.019 v1.23 (pre-fix-burst-42; STALE — actual shipped v1.24 after @16a6ec85)
  - ADR-048 v1.16 (pre-fix-burst-42; STALE — actual shipped v1.17 after @16a6ec85)
  - error-taxonomy v2.52 (pre-fix-burst-42; STALE — actual shipped v2.53 after @16a6ec85)
  - policies.yaml v1.34 (POL-34 registered)

- **Lesson note — GREEN-lock verification as defect-discovery mechanism:**
  The three lock-exposed production defects (CRIT regex_match span suppression; HIGH EC-004 dead path; MED structural double-prefix) were missed by 11 consecutive passes of fresh-context static adversarial review. Each defect was discovered only when byte-exact lock tests were written against the actual Display output. This confirms that GREEN-lock test writing is not merely a coverage exercise — it is an independent defect-discovery instrument that complements static adversarial review. **Candidate for adversary-methodology codification at cycle close** [process-gap]. The standard adversary probe battery should include a mandatory "write byte-exact lock tests for the 3 highest-risk output paths per pass" step.

- **POL-22 Phase A+C PASS:**
  - Phase A: adversary independently derived all 4 findings from first principles; no reliance on implementer disclosure
  - Phase C: F-MED-001 fix is structural (spec retraction + semantic-flag); F-OBS-001 adjudication is documented; F-OBS-002 fix is structural (gate advancement); three lock-exposed fixes are load-bearing (moved try_map, widened grammar, changed detail-builder path)

- **Non-exhaustive gate: 92/92 on branch (@9c99e54f).** Both `check-non-exhaustive.sh EXPECTED=92` and `check-non-exhaustive-per-symbol.py` EXPECTED_COUNT=92 + `ParseError` appended to EXPECTED_SYMBOLS. CLAUDE.md sentence updated to reflect 92 (in worktree CLAUDE.md).

- **Novelty: HIGH** — Three production defects discovered by lock-writing that survived 11 adversarial passes; root causes are structural (span-level try_map placement; grammar precondition mismatch; materialization path condition). Mechanism: adversary GREEN-lock test writing as defect-discovery instrument.

---

## Convergence Status

- CLEAN(strict): NO — 1 MED + 1 LOW + 2 OBS in-perimeter findings; strict criterion requires zero findings of any severity
- CLEAN(PR-merge): NO — 1 MED finding is MERGE-BLOCKING (CRIT/HIGH/MED threshold per BC-5.39.001 §Strict-vs-PR-Merge disambiguation)
- Streak: **0/3** (BC-5.39.001 strict criterion failed; fix-burst-42 pushed new commit 9c99e54f; DRIFT-ORCH-PRLEVEL-PUSH-001: streak must re-gate on frozen HEAD 9c99e54f)
- New frozen HEAD: **9c99e54f** (PR #223 HEAD after fix-burst-42; CI PENDING on 9c99e54f)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-42 push mid-cascade resets streak; all pass-12+ must use 9c99e54f as the frozen HEAD; passes 1–11 on prior frozen HEADs do NOT count toward 9c99e54f streak

---

## Next Step

CI green on 9c99e54f (PR #223 new HEAD) → PR-LEVEL pass-12 on frozen 9c99e54f (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 9c99e54f → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge + LOW-006 keyword-list adjudication merge-gate feature-decision).
