---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [4]
feature_head_at_review: 76c0fa60
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
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 4 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 4 (frozen 76c0fa60; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

Streak: **0/3** (BC-5.39.001 strict criterion: pass-4 has 1 LOW + 2 OBS findings within perimeter; CLEAN(strict) requires zero findings of any severity; streak stays 0/3; fix-burst-37 pushed two commits 3d89ae1e→72d8ed8d mid-cascade; DRIFT-ORCH-PRLEVEL-PUSH-001: re-gate on frozen HEAD 72d8ed8d for all subsequent passes)

Cascade tally (as of this pass): **4 passes / 2 fix-bursts**

CLEAN(strict): NO — 3 in-perimeter findings (1 LOW + 2 OBS)
CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; LOW/OBS non-blocking

---

## Findings

### F-PQLFN-PR4-LOW-001 — SqlPipe-head-HAVING interception arm has no load-bearing test

**Severity:** LOW
**Category:** test-coverage gap
**Routing:** test-writer (fix-burst-37; CLOSED this session)

**File/Anchor:** `crates/prism-query/tests/` — keyword gate / enrich-udf test suite; BC-2.11.019 v1.21 §Postconditions position-f interception; EC-11-086 (HAVING aggregate intercept)

**Defect:** The `check_enrich_udf_availability` function added a `spq.head.having` walk arm in the `Ast::SqlPipe` branch specifically to intercept `DATAFUSION_BUILTIN_AGGREGATE_NAMES` members in HAVING position before the infusion-registry lookup fires E-QUERY-039. The Sql(Select)-form tests (EC-11-086 mirror) exercise the `Ast::Sql` branch, not the `Ast::SqlPipe` branch. Reverting the `spq.head.having` walk from the SqlPipe arm would silently regress to pre-fix behavior (percentile in a SqlPipe HAVING fires E-QUERY-039 instead of E-QUERY-001) without causing any test failure. The coverage gap is not protected by the existing EC-11-086 tests because those drive `Ast::Sql` (plain SQL SELECT ... HAVING), not `Ast::SqlPipe` (SQL head + pipe tail).

**Failure scenario:** Future refactor removes or mis-conditions the `spq.head.having` walk arm in `check_enrich_udf_availability` for the `Ast::SqlPipe` branch → `SELECT ... | having percentile(latency, 0.95) = 5` produces E-QUERY-039 (registry path) instead of E-QUERY-001 (HAVING-intercept path) → no test failure.

**BC references:** BC-2.11.019 v1.21 §Postconditions position-f interception; EC-11-086; ADR-048 v1.16 §D.2

---

### F-PQLFN-PR4-OBS-001 — HAVING-interception guidance template hardcodes two-arg signature; no fail-loud enforcement of the percentile-specific invariant

**Severity:** OBS
**Category:** spec-contract gap / hardening
**Routing:** implementer + product-owner (fix-burst-37; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/engine.rs` — `check_enrich_udf_availability` HAVING-interception loop; BC-2.11.019 v1.21 §Postconditions position-f; ADR-048 v1.16 §D.2 canonical guidance template

**Defect:** The HAVING-interception guidance template `"E-QUERY-001: query parse error at offset {offset}: '{name}' is a PrismQL aggregate function; {NAME} is not directly supported in HAVING predicates — alias it in SELECT: SELECT {NAME}(field, p) AS alias ... HAVING alias > threshold (ADR-048 D.3 OD-2)"` hardcodes a two-argument `(field, p)` signature in the aliasing template. This is intentionally specific to `percentile` (the sole set-difference member: `AGGREGATE_NAMES ∖ FUNCTION_NAMES` reachable as `ScalarFunc::Unknown`). However, there is no fail-loud assertion verifying this invariant. If a future DataFusion version removes `percentile` from the DataFusion aggregate registry, or if a new `DATAFUSION_BUILTIN_AGGREGATE_NAMES` member is added that is not percentile, the HAVING-intercept loop would apply the percentile-specific two-arg template to a different function name — silently producing misleading guidance. The set-difference enumeration `{AGGREGATE_NAMES} ∖ {FUNCTION_NAMES}` equals exactly `{percentile}` at this codebase snapshot, but this invariant is undocumented and unenforced.

**Failure scenario:** `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gains a new member `approx_quantile` that is not in `DATAFUSION_BUILTIN_FUNCTION_NAMES` → `having approx_quantile(x, 0.9) = 5` fires E-QUERY-001 with the template `"approx_quantile(field, p)"` guidance → the two-arg template is wrong for a function with different arity → misleading LLM retry guidance.

**BC references:** BC-2.11.019 v1.21 §Postconditions; ADR-048 v1.16 §D.2; EC-11-086

---

### F-PQLFN-PR4-OBS-002 — `{name}` in guidance template is input-verbatim (uppercase echo); convention undocumented

**Severity:** OBS
**Category:** spec-contract gap / documentation
**Routing:** product-owner (fix-burst-37; CLOSED this session)

**File/Anchor:** BC-2.11.019 v1.21 §Postconditions position-f; ADR-048 v1.16 §D.2 canonical guidance template; `crates/prism-query/src/engine.rs` HAVING-interception guidance string

**Defect:** The guidance template uses `{name}` (input-verbatim) for the function name in the lower-case position and `{NAME}` (uppercased via `.to_uppercase()`) for the SELECT-alias example. An analyst writing `PERCENTILE(latency, 0.95) = 5` in HAVING would receive a message that echoes `'PERCENTILE'` (all-caps) in the first sentence but `PERCENTILE(field, p)` in the aliasing example — both internally consistent but the first appearance deviates from the canonical lowercase example in BC-2.11.019 §Postconditions. The input-verbatim echo behavior is deliberate and correct (it identifies the exact token the analyst wrote), but it is undocumented in BC-2.11.019. A future spec-reviewer or adversary might flag the uppercase echo as a defect when it is intentional behavior.

**Failure scenario:** Spec-review or adversary reads BC-2.11.019 §Postconditions HAVING-intercept template, observes the canonical lowercase example `'percentile'`, notes that code emits the input-verbatim uppercase form `'PERCENTILE'`, and files a correctness finding — when the behavior is intentional and compliant with the injection-safety requirement (no user-input sanitization needed for PrismQL keyword tokens that are normalized by the parser to lowercase before reaching the gate — but the HAVING path uses the ScalarFunc::Unknown name which may carry the original case).

**BC references:** BC-2.11.019 v1.21 §Postconditions position-f

---

### Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3; not re-raised as a PR-LEVEL finding.

---

## Fix-Burst-37 Closure

All three in-perimeter findings closed same-session. Fix-burst-37 produced two commits: test-writer 3d89ae1e + implementer 72d8ed8d. Branch pushed to origin; PR #223 HEAD is now 72d8ed8d.

### F-PQLFN-PR4-LOW-001 — SqlPipe-head-HAVING interception arm has no load-bearing test

**test-writer (commit 3d89ae1e):** 5 GREEN lock tests added in first-run: (1) `test_BC_2_11_019_ec11_086_sqlpipe_having_percentile_with_registry` — SqlPipe-head HAVING intercept fires E-QUERY-001 when registry contains entries (spq.head.having walk arm load-bearing); (2) `test_BC_2_11_019_ec11_086_sqlpipe_having_percentile_without_registry` — SqlPipe-head HAVING intercept fires E-QUERY-001 when registry is empty (registry-independence confirmed via SqlPipe arm); (3) `test_BC_2_11_019_null_input_verbatim_lower` — null() via lowercase input echoes `null` verbatim (low-case casing lock for F-PQLFN-PR4-OBS-002); (4) `test_BC_2_11_019_null_input_verbatim_upper` — NULL() via uppercase input echoes `NULL` verbatim (upper-case casing lock); (5) `test_BC_2_11_019_percentile_uppercase_input_verbatim` — PERCENTILE() via uppercase input echoes `PERCENTILE` verbatim in E-QUERY-001 message (validates input-verbatim behavior documented in §OBS-004 fix-burst-37 amendment). prism-query 1664/1664. Non-exhaustive 91/91.

### F-PQLFN-PR4-OBS-001 — HAVING-interception guidance template hardcodes two-arg signature

**implementer (commit 72d8ed8d):** `debug_assert_eq!` fail-loud guard added inside the HAVING-interception loop in `check_enrich_udf_availability` (`engine.rs`): asserts that the set of names reaching the interception arm equals exactly `{percentile}` (the sole set-difference member `AGGREGATE_NAMES ∖ FUNCTION_NAMES`); compiles out in release builds — no production panic path; validates the percentile-template invariant in debug/test builds. Code-comment pin sweep: BC-2.11.019 `v1.21` → `v1.22` at 5 comment-pin sites where `v1.21` appeared in HAVING-interception context (engine.rs 2 sites, filter_parser.rs 1 site, ast.rs 2 sites). prism-query 1664/1664. just check FULL WORKSPACE GREEN. Non-exhaustive 91/91.

**product-owner (BC-2.11.019 v1.21→v1.22, pre-edited):** §OBS-004 section amended to add: (a) input-verbatim convention note — the `{name}` token echoes the user-written case (e.g., `PERCENTILE` if analyst wrote uppercase); documented as deliberate behavior; analyst-facing message uses input-verbatim first occurrence + uppercased `.to_uppercase()` aliasing example; (b) percentile-template invariant note — the two-arg `(field, p)` template is specific to `percentile`, the sole current member of `DATAFUSION_BUILTIN_AGGREGATE_NAMES ∖ DATAFUSION_BUILTIN_FUNCTION_NAMES`; the `debug_assert_eq!` guard in `engine.rs` enforces this invariant at test-time; if the set grows, the guard fires before production deployment; (c) fail-loud assertion requirement — `debug_assert_eq!` on set-cardinality must remain in `check_enrich_udf_availability` HAVING-interception loop; removal without updating the template is a P1 finding. Changelog row added. BC-2.11.019 v1.22.

### F-PQLFN-PR4-OBS-002 — `{name}` input-verbatim echo undocumented

**product-owner (BC-2.11.019 v1.21→v1.22, same pre-edit as OBS-001):** §OBS-004 input-verbatim convention note covers this finding — the note documents that `{name}` echoes user-written case verbatim (deliberate; injection-safety requirement satisfied by parser normalization upstream of the HAVING gate; PrismQL keyword tokens normalized to lowercase before reaching engine.rs, but the ScalarFunc::Unknown name in HAVING preserves original case). No separate BC amendment required; the §OBS-004 convention note establishes the canonical behavior. CLOSED by same product-owner pre-edit as OBS-001.

**Post-fix verification:** Workspace 5600/5600 tests PASS. prism-query 1664/1664. non-exhaustive EXPECTED=91. just check pre-push FULL WORKSPACE GREEN. Branch pushed to origin/fix/DEFECT-PQL-FNCALL-LHS-001 at 72d8ed8d (2 commits over 76c0fa60: 3d89ae1e + 72d8ed8d). PR #223 HEAD updated to 72d8ed8d. CI PENDING on new HEAD.

---

## Positive Verifications

- **SAP-1 PASS:** 55 raw `event_type =` occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog (92 rows). ZERO net-new emissions added in fix-burst-37. Settled methodology carries from prior passes.

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code at 76c0fa60, not only pass reports)
  - `check_enrich_udf_availability` (engine.rs) HAVING-intercept arm verified at canonical definition site
  - `DATAFUSION_BUILTIN_AGGREGATE_NAMES` definition verified byte-matching BC-2.11.019 v1.21 at 3 canonical sites
  - `DATAFUSION_BUILTIN_FUNCTION_NAMES` verified at definition site
  - `spq.head.having` walk added in `Ast::SqlPipe` branch of `check_enrich_udf_availability` verified at definition site
  - Set-difference `{AGGREGATE_NAMES} ∖ {FUNCTION_NAMES}` = `{percentile}` verified by inspection of both sets at HEAD 76c0fa60
  - Five fix-burst-36 load-bearing tests verified GREEN at 76c0fa60 baseline

- **TD-VSDD-059 PASS:** F-PQLFN-PR4-LOW-001 closure is load-bearing: two SqlPipe-head-HAVING EC-11-086 mirror tests directly exercise the `spq.head.having` walk arm; reverting the arm causes both to fail. F-PQLFN-PR4-OBS-001 closure is load-bearing: `debug_assert_eq!` fires in debug/test builds if the set invariant breaks; the 5 casing-lock tests are load-bearing for F-PQLFN-PR4-OBS-002 (null/Null/PERCENTILE verbatim echo).

- **TD-VSDD-060 PASS:** No function signature changes in fix-burst-37 diff; callsite sweep clean.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **POL-23 sweep verified clean (at 76c0fa60):** `grep -r "BC-2.11.019 v1.21" .factory/` — ZERO live non-exempt pins. ADR-048 line-259 `BC-2.11.019 v1.21` citation was adjudicated by orchestrator as PROVENANCE citation (records which BC version drove the v1.16 amendment; historically immutable, exempt per same rationale as changelog rows). No other live v1.21 pins found.

- **13 adversarial bypass probes — NO BYPASS:**
  1. Arm separation: `Ast::SqlPipe` HAVING walk arm separate from `Ast::Sql` HAVING walk; removing SqlPipe arm would not affect Sql tests → LOW-001 correctly identified coverage gap
  2. Set-difference enumeration: `AGGREGATE_NAMES ∖ FUNCTION_NAMES` = exactly `{percentile}` verified at HEAD 76c0fa60; no other function reaches the interception arm
  3. Case-folding: `PERCENTILE(x, 0.95)` in HAVING → intercept fires via case-insensitive name check (`eq_ignore_ascii_case`) → E-QUERY-001 with verbatim uppercase echo; no bypass via case variant
  4. NULL grammar non-collision with IS NULL / =NULL / IN(NULL): `null(x) = 5` parses as fn_call_comparison (fn_call LHS) → RESERVED_KEYWORDS LOW-006 fires before HAVING intercept; no parser ambiguity with IS NULL or =NULL or IN(NULL) which parse via separate grammar arms
  5. Registry-independence ordering: HAVING intercept fires BEFORE infusion-registry lookup in both `Ast::Sql` and `Ast::SqlPipe` branches → E-QUERY-001 even with empty registry; registry-independence confirmed by both SqlPipe-no-registry tests
  6. SqlPipe head span absoluteness: span offsets in SqlPipe-head HAVING are absolute (relative to full query string, same as Sql HAVING) → E-QUERY-001 offset correct in both forms
  7. stddev TM-12 passthrough: `stddev` ∈ `DATAFUSION_BUILTIN_FUNCTION_NAMES` → excluded by `∉ FUNCTION_NAMES` guard → does NOT fire HAVING intercept → passes to DataFusion execution; TM-12 green lock intact
  8. Empty-arg form: `percentile() = 5` in HAVING → intercept fires on name `percentile` (name-based check only; arity not checked at this gate) → E-QUERY-001; DataFusion would catch arity mismatch downstream if intercept were bypassed
  9. Registy-independence with percentile registered as infusion: if `percentile` were added to `InfusionRegistry`, HAVING intercept still fires FIRST (before registry lookup); the `debug_assert_eq!` would NOT fire because the set invariant holds — `percentile` is in `AGGREGATE_NAMES ∖ FUNCTION_NAMES` regardless of its infusion-registry status
  10. Two-arg template mismatch with non-percentile: future `approx_quantile` member → `debug_assert_eq!` fails in debug builds → F-PQLFN-PR4-OBS-001 guard correctly catches the template drift before production deployment
  11. Uppercase echo in E-QUERY-001: `PERCENTILE(latency, 0.95) = 5` → message echoes `PERCENTILE` verbatim (input-verbatim per §OBS-004); no mismatch with "PERCENTILE" aliasing template (both uppercase) → internally consistent
  12. Null-case verbatim: `NULL(x) = 5` → RESERVED_KEYWORDS fires (NOT HAVING intercept) → E-QUERY-001 template from LOW-006 path, not HAVING path; verbatim echo of `NULL` is via LOW-006 gate, not HAVING intercept; separate paths confirmed
  13. `distinct_count` HAVING pass-through: `distinct_count` ∉ `DATAFUSION_BUILTIN_AGGREGATE_NAMES` → intercept does NOT fire → registry check proceeds → E-QUERY-039 if not registered; EC-11-087 GREEN lock intact (unchanged from fix-burst-36)

- **Spec versions verified at 76c0fa60 (pre-fix-burst-37):**
  - BC-2.11.019 v1.21 (HAVING intercept note + §OBS-004; load-bearing test suite rides THIS branch)
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords; 3 live BC sites)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.1 HAVING-exemption caveat; 13 live pins)
  - error-taxonomy E-QUERY-039 template current (v2.52)

- **No secrets, no AI attribution, no `--no-verify` markers in diff.**

- **No new `unwrap()` / `expect()` in production code paths.**

- **`#[non_exhaustive]` on new pub types** — no new pub types introduced in fix-burst-37.

- **No `reqwest` changes** — ADR-050 rustls-tls constraint untouched.

- **Novelty: LOW** — findings are tightly scoped to test-coverage (LOW) and two observational spec gaps (OBS) around the HAVING-interception template and input-verbatim convention. No new structural concerns surfaced. Cascade is very close to convergence; all substantive dimensions have been CLEAN for multiple passes.

---

## CI Note

CI on 76c0fa60 initially reported Test (x86_64-unknown-linux-musl) FAILED. Root cause: runner disk exhaustion during `mold` link step (`mold: Disk full` + `rustc-LLVM ERROR: No space left on device`). This is a pure infrastructure flake — the disk failure occurred during build compilation, before any test binary executed. The identical job re-run succeeded 43/43. No code changes were required or made for the re-run. This infra flake does not affect the frozen HEAD 76c0fa60 or the pass-4 review findings; the review was conducted on the frozen HEAD per DRIFT-ORCH-PRLEVEL-PUSH-001 discipline.

---

## Orchestrator Adjudication — ADR-048 line-259 Provenance Citation Exemption

The POL-23 sweep at 76c0fa60 found `BC-2.11.019 v1.21` cited at ADR-048 line-259. The orchestrator adjudicated this as a **provenance citation** (historically immutable): line-259 records which BC version drove the ADR-048 v1.16 amendment at D-1778. This citation is exempt from the POL-23 live-pin update requirement by the same rationale that applies to changelog rows and decisions-log entries — it captures a historical record of what drove a past change, not a forward reference to the current spec version. No update to ADR-048 line-259 is required for BC-2.11.019 v1.22.

This adjudication is recorded here per TD-VSDD-091 audit-trail discipline.

---

## Convergence Status

- CLEAN(strict): NO — 3 in-perimeter findings (1 LOW + 2 OBS); strict criterion requires zero findings of any severity
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED; LOW/OBS non-blocking for PR merge
- Streak: **0/3** (BC-5.39.001 strict criterion failed; fix-burst-37 pushed new commits 3d89ae1e + 72d8ed8d; DRIFT-ORCH-PRLEVEL-PUSH-001: streak must re-gate on frozen HEAD 72d8ed8d)
- New frozen HEAD: **72d8ed8d** (PR #223 HEAD after fix-burst-37; CI PENDING)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-37 push mid-cascade resets streak; all pass-5+ must use 72d8ed8d as the frozen HEAD; passes 1/2/3/4 on prior frozen HEADs do NOT count toward 72d8ed8d streak

---

## Next Step

CI green on 72d8ed8d (PR #223 new HEAD) → PR-LEVEL pass-5 on frozen 72d8ed8d (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 72d8ed8d → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge).
