---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: prlevel-pass-7
frozen_pr_head: 9eab936b
base_develop: 903c8fcb
reviewer: vsdd-factory:adversary
date: 2026-06-25
verdict_strict: "CLEAN(strict)=NO"
verdict_pr_merge: "CLEAN(PR-merge)=YES"
findings_count: 2
findings_crit: 0
findings_high: 0
findings_med: 0
findings_low: 2
all_closed: true
new_frozen_pr_head: 737f4ed8
---

# PR-LEVEL Pass 7 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**FROZEN PR HEAD reviewed:** `9eab936b`  
**Base develop:** `903c8fcb`  
**Verdict:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES  
**Findings:** 2 LOW — all CLOSED  
**New FROZEN PR HEAD:** `737f4ed8`

---

## Substantive Surface Audit (all PASS)

- **SqlPipe gate-sweep complete:** All 11 plan-time AST match sites cover `Ast::SqlPipe` including `PipeStage::Join` sources at all 4 extractor arms (materialization.rs, table_registry.rs, explain.rs) — TD-VSDD-060 PASS.
- **FORBID-BOTH dual-limit rule:** `E-QUERY-040` fires for `| limit` and `| tail` with neutral wording; `REFERENCE_EXAMPLES` contains `| tail` negative example.
- **Temporal grammar:** `NOW()` + `INTERVAL` plan-time constant injection verified; time-window push-down operative for `SqlPipe` head.
- **Verbatim errors:** All BC-2.11.023 §D1 + §D2 verbatim message substrings present with load-bearing tests; AC-009/AC-027 PASS.
- **SAP-1 (tracing emission catalog):** All `event_type` values present in BC-2.16.002 §Postconditions — NO missing rows.
- **MCP guard-reorder:** `emit_tool_audit` await guard confirmed before NYA fast-fail; 10 handlers with input-validation guards (validate_text_field/validate_id_field) confirmed audited; accurate NYA coverage.
- **Prompt-hang real-test coverage:** `test_BC_2_10_016_prompts_fast_return_within_5s` drives full rmcp transport `get_prompt` dispatch with 5s `tokio::time::timeout`; AC-015 PASS.
- **OrgSlug DI:** `OrgRegistry::slug_exists(&OrgSlug)` wired via `Arc<OrgRegistry>` injection; no placeholder construct.
- **AC-016 named Red Gate test:** `test_bc_2_10_016_missing_required_arg_fast_error` drives full rmcp transport dispatch with 5s timeout; renders prompt with `(unknown)` substitution and returns Ok.
- **BC-2.10.016 internal consistency:** v1.1 reconciles §Error Cases + EC-10-016-003 + INV-PROMPT-REQUIRED-ARGS to reflect option-(a) `(unknown)` substitution returning Ok; no-hang guarantee preserved.
- **just check EXIT 0:** 4942 tests; non-exhaustive gate 87; fmt-canonical PASS.

---

## Findings

### OBS-1 (LOW) — Ast::SqlPipe arms missing in normalizer pre-checks

**Severity:** LOW  
**Category:** SOUL.md #4 defensive parity  
**Source:** `crates/prism-query/src/materialization.rs` — `ast_has_unfolded_temporal_expr` and `ast_has_both_quote_string`

**Description:** Both pre-check functions fell through `_ => false` for `Ast::SqlPipe`. No live defect today (the execution arm wraps `spq.head` as `Ast::Sql` before calling `normalize`; no production path routes a raw `Ast::SqlPipe` through `normalize`). However, if a future path routes raw `SqlPipe` through `normalize`, both guards would silently report false, producing silent-corruption behavior (temporal constants not injected; FORBID-BOTH quota not checked).

**CLOSED by implementer (9eab936b → 737f4ed8):**
- Added explicit `Ast::SqlPipe` arms (checking `head` + iterating `stages`) to both `ast_has_unfolded_temporal_expr` and `ast_has_both_quote_string`
- Corrected scope comment on the `_ => false` fallthrough arms to reflect actual semantics
- 4 load-bearing tests added: `ast_has_both_quote_string` true for SqlPipe head containing both-quote string; `ast_has_both_quote_string` true for SqlPipe stage containing both-quote string; `ast_has_unfolded_temporal_expr` true for SqlPipe head with `NOW()`; `ast_has_unfolded_temporal_expr` true for SqlPipe stage with `NOW()` — all verify `normalize` returns `None` (defensive guard fires)

### OBS-2 (LOW) — Two sub-issues: hollow AC-016 test + BC-2.10.016 internal contradiction

**Severity:** LOW  
**Category:** (a) test hollow / (b) spec internal contradiction  

**OBS-2a — AC-016 named Red Gate test was hollow:**  
`test_bc_2_10_016_missing_required_arg_fast_error` previously performed a pure sync `render_investigate_host` call (does not exercise the dispatch machinery or MCP transport). The AC-016 Red Gate obligation requires driving the full `prompts/get` dispatch via rmcp transport to verify no hang on missing required arg.

**CLOSED by implementer (9eab936b → 737f4ed8):** Test rewrote to drive full rmcp transport `get_prompt` dispatch with 5s `tokio::time::timeout`; asserts Ok returned with `(unknown)` substituted in rendered output within timeout. Canonical test name `test_bc_2_10_016_missing_required_arg_fast_error` preserved; no story rename required.

**OBS-2b — BC-2.10.016 internal contradiction:**  
BC-2.10.016 v1.0 §Error Cases row "Required argument missing" stated "Returns structured MCP error" and EC-10-016-003 stated "Returns structured MCP error; MUST NOT hang." The invariant `INV-PROMPT-REQUIRED-ARGS` already sanctioned option-(a) `(unknown)` substitution returning Ok — creating a direct contradiction within the same document (invariant vs Error Cases / Edge Case).

**CLOSED by product-owner (BC-2.10.016 v1.0 → v1.1):** §Error Cases updated to describe option-(a) behavior (substitutes `(unknown)`, returns Ok within 5s, no structured MCP error). EC-10-016-003 updated to match. INV-PROMPT-REQUIRED-ARGS prose clarified to confirm option-(a) is the shipped implementation. No-hang / within-5s guarantee preserved.

---

## Post-Fix Verification

- `just check` EXIT 0 on `737f4ed8`: 4942 tests; non-exhaustive 87; SAP-1 clean
- FROZEN PR HEAD: `737f4ed8`
- 3-CLEAN streak RESET 0/3 on `737f4ed8` (code HEAD moved by OBS-1 fix)
- CI re-running on `737f4ed8` push (background)
- develop_head UNCHANGED: `903c8fcb`

---

## NEXT

PR-LEVEL adversary cascade: 3 consecutive CLEAN(strict) passes on UNCHANGED `737f4ed8` → CI green → squash-merge (--admin D-1337) → post-merge POL-14 BC promotion → pre-flight demo re-audit → T13 capstone → T14 recording.

**DO-NOT-FLAG carry-forward (this pass):**
- `Ast::SqlPipe` arms in both `ast_has_unfolded_temporal_expr` and `ast_has_both_quote_string` (head + stages) — defensive parity added by OBS-1 fix; load-bearing tests confirm behavior
- AC-016 named Red Gate test now full rmcp transport dispatch with 5s timeout — canonical test name preserved
- BC-2.10.016 v1.1 §Error Cases / EC-10-016-003 describe option-(a) `(unknown)` substitution (no-hang preserved; no structured MCP error)
