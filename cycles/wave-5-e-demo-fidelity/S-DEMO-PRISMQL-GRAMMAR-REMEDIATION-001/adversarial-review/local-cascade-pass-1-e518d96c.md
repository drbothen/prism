---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
cascade: LOCAL
pass: 1
frozen_head: e518d96c
reviewer_perimeter: "per-story (story diff 903c8fcb..e518d96c + 8 BCs + ADR-043/044/045/046 + error-taxonomy)"
verdict_strict: "CLEAN(strict)=NO"
verdict_pr_merge: "CLEAN(PR-merge)=NO"
pass_outcome: "NOT CLEAN — 1 HIGH + 1 MED + 3 OBS; all 5 closed by fix-burst (code HEAD e518d96c→f03679b2) + PO BC corrections"
post_pass_head: f03679b2
streak_after: "0/3 on f03679b2 (cascade re-runs from Pass 1 on new HEAD)"
timestamp: 2026-06-25T00:00:00Z
---

# LOCAL Adversary Pass 1 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Story:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
**Cascade:** LOCAL
**Pass:** 1 (on HEAD e518d96c; cascade reset from prior pass on 329fa519)
**Frozen HEAD reviewed:** `e518d96c`
**Diff perimeter:** story diff `903c8fcb..e518d96c` + 8 BCs (BC-2.11.020/021/022/023, BC-2.10.015/016/017, BC-2.11.002) + ADR-043/044/045/046 + error-taxonomy

## Verdict

- **CLEAN(strict):** NO
- **CLEAN(PR-merge):** NO
- **Finding count:** 1 HIGH + 1 MED + 3 OBS = 5 total
- **All findings:** CLOSED (fix-burst + PO corrections)

---

## Findings

### F-PASS1-HIGH-1 — Filter-mode end-to-end tests not load-bearing (HIGH)

**Severity:** HIGH
**BC anchor:** BC-2.11.023 §D4/AC-011 — "verifies rows returned"
**Finding:** `filter_mode.rs` end-to-end tests used an empty `AdapterRegistry` that returned 0 rows on every query. The test assertions counted 0 rows against an expected 0, making the assertions trivially true regardless of whether the `Ast::Filter` execution arm was correctly wired. The docstring claimed "pre-seeded rows" that did not exist. `TODO` comment assertions were present but unfilled. BC-2.11.023 §D4 mandates that the test "executes `severity='HIGH'` as `Ast::Filter` against a mocked/DTU sensor source and verifies rows matching the predicate are returned" — the empty-registry pattern produced zero rows by construction, defeating the verification requirement.

**Status:** CLOSED by implementer commit `f03679b2`
**Fix:** Introduced `SeverityStubAdapter` with 2 HIGH rows + 3 LOW rows pre-seeded. Test now asserts exact row-count = 2 (HIGH matches), not 0. `Ast::Filter` execution path wired to invoke the adapter and materialize real rows. Stale TODO assertions and false docstring removed.

---

### F-PASS1-MED-1 — E-QUERY-040 Display message was a paraphrase, not verbatim template (MED / POL-24)

**Severity:** MED
**Governance anchor:** POL-24 (verbatim error-taxonomy template)
**Finding:** The `#[error(...)]` template on the `E-QUERY-040` (`RedundantRowLimit`) variant used a paraphrase of the pedagogical template from `error-taxonomy.md` rather than the exact verbatim text. POL-24 requires that error Display strings match the canonical template exactly so that MCP client error messages are predictable and auditable.

**Status:** CLOSED by implementer commit `8f6bb337`
**Fix:** `#[error]` template updated to verbatim canonical pedagogical template from `error-taxonomy.md`. Test strengthened to assert the full substring of the verbatim template text (not just the error code prefix).

---

### OBS-1 — `mode_bridge_normalized_pql` in wrong module (OBS)

**Severity:** OBS
**Anchor:** BC-2.11.023 Architecture Anchors + Story File Structure
**Finding:** `mode_bridge_normalized_pql` lived in `prism-mcp/src/error_mapping.rs`. BC-2.11.023 Architecture Anchors and the story File Structure section mandate that this function lives in `prism-query/src/error_recovery.rs` (spec-wins, CLAUDE.md precedence rule #7). The MCP crate depends on the query crate — `prism-mcp` calling a function from `prism-query/error_recovery.rs` is the correct dependency direction. Having it in `prism-mcp` also meant `find_first_unquoted_pipe` (a pure query-string utility) lived in the MCP layer.

**Status:** CLOSED by implementer commit `616864d0`
**Fix:** `mode_bridge_normalized_pql` + `find_first_unquoted_pipe` relocated into `prism-query/src/error_recovery.rs`. `prism-mcp/src/error_mapping.rs` calls them via the public path. Dependency direction now correct.

---

### OBS-2 — BC-2.11.023 §D1 body still cited `ParseErrorDetails` (OBS)

**Severity:** OBS
**Anchor:** BC-2.11.023 §D1 prose (post-v1.1 residual)
**Finding:** BC-2.11.023 v1.1 changelog described the `ParseErrorDetails → StructuredErrorFields` rename as applied "throughout" but one occurrence in §D1 body prose was missed. The v1.1 update correctly fixed the majority of citations, but the §D1 body paragraph retained the stale `ParseErrorDetails` type name, creating a one-sentence inconsistency within the same contract.

**Status:** CLOSED by product-owner (BC-2.11.023 v1.1 → v1.2)
**Fix:** PO corrected the stale §D1 occurrence to `StructuredErrorFields`. No behavioral semantics changed.

---

### OBS-3 — BC-2.10.015 Architecture Anchor cited non-existent `OrgRegistry::contains` (OBS)

**Severity:** OBS
**Anchor:** BC-2.10.015 Architecture Anchors
**Finding:** BC-2.10.015 §Architecture Anchors cited `OrgRegistry::contains(client_id: &str) -> bool` as the method name. This method does not exist in the codebase. The real API is `OrgRegistry::slug_exists(&OrgSlug) -> bool` (confirmed by story spec and implementation). The story spec and code already used `slug_exists` correctly; only the BC anchor text was stale.

**Status:** CLOSED by product-owner (BC-2.10.015 v1.0 → v1.1)
**Fix:** PO corrected Architecture Anchor to `OrgRegistry::slug_exists(&OrgSlug) -> bool`. No behavioral semantics changed.

---

## Verified CLEAN (Do NOT Reflag)

The following areas were verified clean on the reviewed HEAD (`e518d96c`) and MUST NOT be reflagged in subsequent passes:

- **Temporal NOW()/INTERVAL plan-time wiring** in `execute_against_session` / `SqlPipe` head / `Ast::Sql`: D-1333 Option A plain-string `'<iso>'` form is correct per BC-2.11.021/ADR-044 D4 (OCSF Datetime = Arrow `Utf8`). Constant-fold inject_now is production-grade. The D-1335 SPEC-FIRST adjudication confirmed this form; do not re-derive from first principles.
- **FORBID-BOTH / E-QUERY-040** fires on 0-row returns: hoisted `plan_sqlpipe_query` Step 1b before fan-out in the execution path. Verified by load-bearing tests.
- **PqlNormalizer emits executable SQL**: SQL→Pipe composition path produces valid DataFusion-compatible SQL output (TIMESTAMP literal form correct; PrismQL-only operators not re-emitted).
- **SAP-1** (tracing emission catalog completeness): no uncatalogued `event_type` values found across `crates/**/*.rs`.
- **SAP-2** (DTU↔TOML schema parity): N/A — no TOML sensor specs touched in this story diff.
- **SID-1** (no ignore-rationalized deferrals): no `#[ignore]`'d tests used to rationalize missing behavior.
- **Non-exhaustive gate:** 87/87 (ci.yml EXPECTED=87; worktree value consistent with non-exhaustive gate at e518d96c; NOTE: develop is at 84 from PR #202 — the 87 count in the worktree reflects 3 additional `#[non_exhaustive]` types added in this story's diff; EXPECTED in ci.yml on the feature branch = 87, which is correct).
- **AC-019 + reset_token_cache**: OrgRegistry slug_exists wiring verified; reset_token_cache correctly limited to test/wasm32 scope.

---

## Post-Pass State

- **Fix-burst code HEAD:** `e518d96c` → `f03679b2` (implementer fix-burst: SeverityStubAdapter, verbatim E-QUERY-040 template, module relocation)
- **BC corrections:** BC-2.11.023 v1.1 → v1.2 (PO); BC-2.10.015 v1.0 → v1.1 (PO)
- **3-CLEAN streak:** 0/3 on `f03679b2` (streak resets on every HEAD change per BC-5.39.001 + FROZEN-HEAD-STREAK-RULE DRIFT-ORCH-PRLEVEL-PUSH-001)
- **NEXT:** LOCAL adversary Pass 1 (re-run) on `f03679b2`
- **just check at f03679b2:** GREEN (non-exhaustive 87/87; prism-query 1117/1117)
