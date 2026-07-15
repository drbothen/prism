---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [10]
feature_head_at_review: f715b0a5
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 10 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 10 (frozen f715b0a5; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3 — first merge-blocking finding)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

Streak: **0/3** (BC-5.39.001 strict criterion: pass-10 has 1 MED in-perimeter finding — first MERGE-BLOCKING result of the PR-LEVEL cascade; streak stays 0/3; fix-burst-41 pushed commit 74932b99; DRIFT-ORCH-PRLEVEL-PUSH-001: re-gate on frozen HEAD 74932b99 for all subsequent passes)

Cascade tally (as of this pass): **10 passes / 6 fix-bursts**

CLEAN(strict): NO — 1 in-perimeter finding (1 MED)
CLEAN(PR-merge): NO — 1 MED finding is MERGE-BLOCKING

**Attack angles for this pass:**
- (A) runtime-Display prefix-count consistency between plan-time gate messages and parse-time gate messages (fresh-context check: do keyword-rejection messages emitted at runtime have consistent prefix structure end-to-end through the Display chain?)
- (B) walker variant exhaustiveness under catch-all (independent re-verification of the predicate-position walker to confirm all expression variants are explicitly handled or correctly caught)

---

## Findings

### F-PQLFN-PR10-MED-001 — LOW-006 keyword-rejection runtime Display doubles "E-QUERY-001:" and "parse error at offset": sibling-parity gap survived 58 passes

**Severity:** MED (MERGE-BLOCKING)
**Category:** code behavior defect / output correctness / sibling-parity gap
**Routing:** test-writer + implementer (fix-burst-41; CLOSED this session)
**CLAUDE.md precedence rule 7 adjudication:** SPEC WINS — code de-prefixed (adversary option a)

**File/Anchor:** `crates/prism-query/src/filter_parser.rs` — LOW-006 keyword-rejection `.validate()` callback; `crates/prism-query/src/error_recovery.rs` — `shift_parse_error_offsets` / `ParseError::Display`; `crates/prism-query/src/materialization.rs` — `QueryParseFailed` detail builder; `crates/prism-query/src/sql_parser.rs` — `has_semantic_error`; BC-2.11.004 v1.48 EC-11-085 POL-24 byte-exact canonical + ADR-048 §D.7.2 de-prefix discipline

**Defect:** The LOW-006 keyword-rejection emit path in `filter_parser.rs` called `.validate()` with a message string that already embedded the `"E-QUERY-001: "` canonical prefix. This prefixed string then flowed through the full error propagation chain:

1. `.validate()` fires → `RichReason::Custom("E-QUERY-001: 'NOT' is a PrismQL keyword and cannot be used as a function call name")` is created — prefix embedded here
2. `ParseError::Display` impl (in `error_recovery.rs`) prepends `"parse error at offset {span}: "` → partial doubling at the display boundary
3. `QueryParseFailed` detail builder in `materialization.rs` reads the parse error and applies the canonical format template, which prepends `"E-QUERY-001: query parse error at offset {N}: "` — yielding a fully doubled prefix pair

**Resulting double-prefix in test output:**
```
E-QUERY-001: query parse error at offset 36: parse error at offset 36: E-QUERY-001: 'NOT' is a PrismQL keyword...
```

EC-11-085 requires byte-exact canonical format `"E-QUERY-001: ..."` — the doubled form violates this invariant (POL-24 byte-exact canonical).

**Why it survived 58 passes (49 LOCAL + 9 PR-LEVEL):**

Two compounding factors:

1. **All prior LOW-006 assertions were `contains`-based (substring tolerant of doubling).** From fix-burst-20 onward, LOW-006 tests asserted `error_detail.contains("E-QUERY-001:")` and `error_detail.contains("is a PrismQL keyword")`. These substring checks pass whether the output is singly or doubly prefixed — the doubled form still contains the expected substrings. No test verified the ABSENCE of a second prefix occurrence.

2. **Sibling-parity gap:** The aggregate-gate path (TM-08 single-prefix lock from fix-burst-36 / EC-11-087) had a byte-verbatim lock that would have failed on a doubled prefix. The LOW-006 keyword-rejection path had no analogous lock — the same rigor applied to the HAVING/aggregate gate was not applied to the keyword-rejection gate at the same pass. This is a sibling-parity gap (TD-VSDD-060 class applied to test-rigor rather than code).

Vector A (prefix-count consistency) was the first fresh-context probe to enumerate the end-to-end Display chain for the LOW-006 path, revealing that the `.validate()` message already carried the prefix before entering the Display chain that adds another prefix.

**IIN/IEQ/INE emitters:** The `filter_parser.rs` IIN/IEQ/INE emitters (`contains`-like operators) already embedded the canonical prefix per BC-2.11.024 field-level requirement, and are CORRECTLY left as-is. The de-prefix fix is scoped to the LOW-006 RESERVED_KEYWORDS rejection path only.

**BC references:** BC-2.11.004 v1.48 EC-11-085 (byte-exact canonical format); POL-24 (byte-exact canonical policy); ADR-048 §D.7.2 (de-prefix discipline); BC-5.39.001 (3-CLEAN criterion)

---

### Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3+4+5+6+7+8+9; not re-raised as a PR-LEVEL finding.

---

## Attack Vector B — Walker Variant Exhaustiveness (No Finding)

Fresh-context re-verification of the predicate-position walker in `crates/prism-query/src/engine.rs`. The walker handles `Expr` variants at WHERE/HAVING positions. Verified:

- All `Expr` arms explicitly handled or fall through to a semantically correct catch-all
- The catch-all arm (`_ => {}` or equivalent) covers Literal / ColumnRef / BinaryExpr / UnaryExpr / Between / InList / ILike / Like forms that are valid in filter position and should pass through
- The aggregate-detection arm (with `to_ascii_lowercase`) correctly intercepts named aggregate function calls regardless of case
- The `FuncCall::Scalar` arm correctly intercepts non-aggregate function calls (triggering E-QUERY-001 for keyword-named functions or passing through for non-keyword names depending on the RESERVED_KEYWORDS check)

**Result: GREEN — no exhaustiveness gap found. Vector B yields no finding.**

---

## Fix-Burst-41 Closure

All in-perimeter findings closed same-session. Fix-burst-41 produced two commits: test-writer a329c3c7 + implementer 74932b99. Branch pushed to origin; PR #223 HEAD is now 74932b99.

### F-PQLFN-PR10-MED-001 — Doubled E-QUERY-001 prefix in keyword-rejection Display chain

**test-writer (commit a329c3c7) — 9 RED tests, all FAILING with doubled-prefix evidence:**

Nine new tests in `test_f_pqlfn_pr10_med_001_*` family, verified FAILING before fix with doubled output:
```
E-QUERY-001: query parse error at offset 36: parse error at offset 36: E-QUERY-001: 'NOT' is a PrismQL keyword...
```

Test structure:
- `test_f_pqlfn_pr10_med_001_all_21_keywords_single_prefix_loop` — parameterized loop over all 21 `RESERVED_KEYWORDS`; for each keyword, verifies the error detail contains exactly ONE occurrence of `"E-QUERY-001:"` (using `assert_eq!(detail.matches("E-QUERY-001:").count(), 1)`). Kills doubled-prefix for all 21 keywords in one shot.
- 7 per-surface byte-exact locks:
  - `test_f_pqlfn_pr10_med_001_pipe_where_prefix_exact` — Pipe mode WHERE; byte-verbatim start-of-string assertion (`.starts_with("E-QUERY-001: 'NOT' is a PrismQL keyword")`)
  - `test_f_pqlfn_pr10_med_001_filter_prefix_exact` — Pipe filter surface; byte-verbatim
  - `test_f_pqlfn_pr10_med_001_sql_where_prefix_exact` — SQL WHERE; byte-verbatim
  - `test_f_pqlfn_pr10_med_001_sql_pipe_where_prefix_exact` — SqlPipe WHERE surface
  - `test_f_pqlfn_pr10_med_001_sql_having_prefix_exact` — SQL HAVING surface
  - `test_f_pqlfn_pr10_med_001_dml_where_prefix_exact` — DML WHERE surface
  - `test_f_pqlfn_pr10_med_001_insert_source_select_where_prefix_exact` — INSERT source_select WHERE surface
- `test_f_pqlfn_pr10_med_001_ec_11_085_null_variant_single_prefix` — EC-11-085 NULL keyword variant; single-prefix count lock

prism-query 1671/1671 (all FAILING at this commit). Non-exhaustive 91/91.

**implementer (commit 74932b99) — 5-layer structural fix:**

1. **`prism-query/src/error_recovery.rs` — `ParseError` gains `pub semantic: bool` field** (`#[serde(default)]` to preserve wire-format compatibility). The `semantic` flag distinguishes validator-injected custom errors (`.validate()` / `RichReason::Custom`) from structural parse errors (`RichReason::ExpectedFound`).

2. **`error_recovery.rs` — `semantic` flag set at error construction:** `semantic = matches!(err.reason(), RichReason::Custom(_))` — Chumsky's `.validate()` callback and `Rich::custom` paths produce `Custom` reason; `ExpectedFound` structural parse failures produce `ExpectedFound`. This replaces the prior fragile `starts_with("E-QUERY-001:")` discriminator.

3. **`crates/prism-query/src/filter_parser.rs` — LOW-006 emit de-prefixed:** The `.validate()` callback for RESERVED_KEYWORDS rejection now emits the bare message without the `"E-QUERY-001: "` prefix: `"'<keyword>' is a PrismQL keyword and cannot be used as a function call name"`. The canonical prefix is applied downstream by the `QueryParseFailed` detail builder (which correctly adds it for semantic errors). IIN/IEQ/INE emitters are UNCHANGED (they embed the prefix per BC-2.11.024 field-level requirement; their path through the detail builder does not double-prefix because the detail builder only applies the canonical format to non-prefixed semantic messages).

4. **`error_recovery.rs` — `semantic` flag propagated through `shift_parse_error_offsets`:** The offset-shifting function preserves the `semantic` flag through its transformation.

5. **`crates/prism-query/src/sql_parser.rs` — `has_semantic_error` keys on `e.semantic`:** Previously relied on `e.message.starts_with("E-QUERY-001:")` (fragile string probe on the raw message, before prefix injection). Now correctly checks `e.semantic`.

6. **`crates/prism-query/src/materialization.rs` — `QueryParseFailed` detail builder strips `"E-QUERY-001: "` prefix for semantic errors:** For semantic errors (`e.semantic == true`), the builder checks whether the raw message already carries the `"E-QUERY-001: "` prefix (IIN/IEQ/INE paths) and, if not, prepends it. For the now-de-prefixed LOW-006 path, the raw message is bare — the builder adds the prefix once, producing the correct `"E-QUERY-001: '<keyword>' is a PrismQL keyword..."` canonical form.

**Post-fix:** 9/9 GREEN; full suite 5616/5616 (+9 over 5607); non-exhaustive 91/91; pre-push `just check` GREEN (76s).

---

## Positive Verifications

- **Vector A fully killed:** The 9 new tests exercise all 21 keywords across 8 surfaces (pipe/filter/SQL WHERE/SqlPipe-where/SQL HAVING/DML WHERE/INSERT source_select WHERE + EC-11-085 NULL) with single-prefix count locks and byte-verbatim starts-with assertions. Any reintroduction of the doubled prefix fails at least the all-21-keyword count loop.

- **IIN/IEQ/INE emitter parity — GREEN:** Verified at 74932b99 that `filter_parser.rs` IIN/IEQ/INE emitters still embed the `"E-QUERY-001: "` prefix in their `.validate()` callbacks per BC-2.11.024 §PostConditions field-level requirement. These paths were NOT touched by fix-burst-41. Existing EC-11-085 byte-verbatim lock tests for IIN/IEQ/INE pass unchanged.

- **Vector B walker exhaustiveness — GREEN:** All `Expr` arms in the predicate-position walker explicitly handled; catch-all semantically correct; aggregate-detection `to_ascii_lowercase` arm present at f715b0a5 (confirmed via `rg 'to_ascii_lowercase' crates/prism-query/src/engine.rs`).

- **SAP-1 PASS:** Zero net-new `event_type =` emissions added in fix-burst-41. No production emission sites modified. Settled methodology carries from prior passes: 55 raw occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog; ZERO net-new emissions.

- **POL-22 Phase A+C PASS:**
  - Phase A: adversary independently derived the doubled-prefix defect from first principles via Display chain enumeration; no reliance on implementer disclosure
  - Phase C: fix verified structurally (5-layer discriminator replacement, not just de-prefix at one site); the `semantic` flag provides a load-bearing discriminator rather than a string probe

- **TD-VSDD-059 PASS (fix-burst-41):** All 9 closures have load-bearing tests: single-prefix count loop structurally fails on any reintroduction of doubled prefix; byte-verbatim starts-with assertions fail if prefix order changes. Structural fix (semantic field) provides independent discriminator from the prior fragile string probe. No paper-fix (rename-only / doc-comment-only).

- **TD-VSDD-060 PASS:** `ParseError` gains a new field `semantic: bool` — sibling-site sweep: `shift_parse_error_offsets` (propagates flag), `has_semantic_error` in `sql_parser.rs` (updated to use flag), detail builder in `materialization.rs` (reads flag). All 3 call-sites swept and updated in-commit.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **POL-14 vehicle confirmed:** BC-2.11.019 draft→active auto-promotion fires on PR #223 merge per POL-14. No new BCs added in fix-burst-41.

- **CLAUDE.md forbidden patterns clean:** No `println!` in production code paths. No new `unwrap()`/`expect()` in production code. No `reqwest` changes (ADR-050 rustls-tls untouched). No AI attribution in commits. No `--no-verify` bypass.

- **Spec versions verified at f715b0a5 (pre-fix-burst-41):**
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords)
  - BC-2.11.019 v1.23 (two-branch detail-builder; debug_assert REMOVED; DML scope cross-note)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.3 HAVING-exemption caveat)
  - error-taxonomy E-QUERY-039 template current (v2.52)
  - policies.yaml v1.34 (POL-34 registered)

- **Novelty: MEDIUM** — 1 code-behavior defect (doubled prefix in runtime Display chain); root cause is a sibling-parity gap in test rigor between the LOW-006 keyword-rejection path and the adjacent aggregate-gate path; survived 58 passes via `contains`-tolerant assertions. Fix is structural (semantic discriminator field), not superficial.

---

## Convergence Status

- CLEAN(strict): NO — 1 MED in-perimeter finding; strict criterion requires zero findings of any severity
- CLEAN(PR-merge): NO — 1 MED finding is MERGE-BLOCKING (CRIT/HIGH/MED threshold per BC-5.39.001 §Strict-vs-PR-Merge disambiguation)
- Streak: **0/3** (BC-5.39.001 strict criterion failed; fix-burst-41 pushed new commits a329c3c7 + 74932b99; DRIFT-ORCH-PRLEVEL-PUSH-001: streak must re-gate on frozen HEAD 74932b99)
- New frozen HEAD: **74932b99** (PR #223 HEAD after fix-burst-41; CI PENDING on 74932b99)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-41 push mid-cascade resets streak; all pass-11+ must use 74932b99 as the frozen HEAD; passes 1–10 on prior frozen HEADs do NOT count toward 74932b99 streak

---

## Next Step

CI green on 74932b99 (PR #223 new HEAD) → PR-LEVEL pass-11 on frozen 74932b99 (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 74932b99 → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge + LOW-006 keyword-list adjudication merge-gate feature-decision).
