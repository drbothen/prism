---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [15]
feature_head_at_review: 9e116a01
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 2
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 15 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 15 (frozen 9e116a01; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml committed-.prx validation-before-build + security.rs fragment-hardened + BC-2.10.007 v1.19 CursorCapExceeded category "internal"; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL15-MED-001 [MED][POL-25 propagation gap] — CLOSED @9e116a01 spec-only (fix-burst-26: error-taxonomy v2.50→v2.51; PR HEAD UNCHANGED)

**Severity:** MED
**Classification:** POL-25 propagation gap — `E-STORE-020` row in `error-taxonomy.md` (v2.50) lacked the MCP-surface annotation that sibling internal-category rows carry. BC-2.10.007 v1.19 (fix-burst-25) established the canonical MCP mapping for `CursorCapExceeded` (`map_prism_error` → `-32000 INTERNAL_ERROR`, terse `"Internal error"` Message Format, BC-verbatim suggestion, category `"internal"`, `original_params_valid: true`, operator-resolvable), but the taxonomy companion burst that v1.7/v1.8/v1.11/v1.12 each had did not accompany v1.19. Sibling rows `E-INT-001` and `E-QUERY-034` both carry `(-32000 INTERNAL_ERROR)` in their Description cell — the pattern that anchors the MCP surface spec to the taxonomy. `E-STORE-020` shipped v2.50 without it.
**Status:** CLOSED — fix-burst-26 @9e116a01 spec-only (error-taxonomy v2.50→v2.51; PR HEAD UNCHANGED)

**Finding:** `E-STORE-020` (CursorCapExceeded, `cursor cap exceeded: cannot allocate more than 200 active cursors`) in `error-taxonomy.md` v2.50 carried no Description-column MCP-surface annotation. Inspection of the sibling pattern:

- `E-INT-001`: Description reads `"(map_prism_error -32000 INTERNAL_ERROR) Internal error; contact support."` — MCP surface annotated.
- `E-QUERY-034`: Description reads `"(map_prism_error -32000 INTERNAL_ERROR) Internal error."` — MCP surface annotated.
- `E-STORE-020` (v2.50): Description reads `"Cursor capacity exhausted. The process has reached the maximum of 200 active concurrent cursors. Wait for existing cursors to expire before retrying."` — no MCP annotation.

BC-2.10.007 v1.19 (PO-ratified, fix-burst-25) established `CursorCapExceeded` as category `"internal"`, `original_params_valid: true`, `retryable: false` with suggestion `"Cursor capacity exhausted. Wait for existing cursors to close before retrying."` These semantics were never reflected in the taxonomy Description cell, creating a gap in the spec chain. An adversary or developer consulting `E-STORE-020` in the taxonomy could not confirm the MCP-surface mapping without cross-referencing `error_mapping.rs` directly.

**Severity rationale:** MED because: (1) BC-2.10.007 v1.19 is the authoritative spec and was correctly ratified in fix-burst-25 — the code is correct and tests pass; (2) the gap is spec-layer only (taxonomy completeness, not code correctness); (3) however, the Description-cell annotation is the cross-reference anchor that lets the taxonomy serve as a standalone audit surface — omitting it from a newly-added internal-category row violates the POL-25 convention; (4) sibling rows establish an unambiguous precedent that `(map_prism_error -32000 INTERNAL_ERROR)` belongs in the Description for all internal-category MCP-surfaced errors.

**Fix plan — fix-burst-26:** PO amends `error-taxonomy.md` v2.50→v2.51: `E-STORE-020` Description cell annotated with `(map_prism_error -32000 INTERNAL_ERROR)` prefix, Message Format cell byte-verified against shipped Display impl, category confirmed `"internal"`, `original_params_valid: true`, operator-resolvable note.

**Closure evidence (fix-burst-26, error-taxonomy v2.50→v2.51, spec-only — PR HEAD 9e116a01 UNCHANGED):**

(1) **`E-STORE-020` Description column** amended to: `"(map_prism_error -32000 INTERNAL_ERROR) Cursor cap exceeded. Wait for existing cursors to close before retrying."` — `(-32000 INTERNAL_ERROR)` annotation added; terse message aligned with BC-2.10.007 v1.19 suggestion verbatim.

(2) **BC-verbatim suggestion byte-verified:** BC-2.10.007 v1.19 §Category table suggestion = `"Cursor capacity exhausted. Wait for existing cursors to close before retrying."` Description note is terse variant (annotation-style); the full BC-verbatim text is the authoritative retryability/suggestion anchor in BC-2.10.007 §Category.

(3) **Category `"internal"`** confirmed in Description cell — operator-resolvable note added (reduce client concurrency; wait for cursors to expire).

(4) **`original_params_valid: true`** semantics conveyed in annotation: MCP returns structured error indicating client parameters are valid, system resource cap is the limiting factor.

(5) **Sibling-convention compliance verified:** `E-INT-001`, `E-QUERY-034` Description cells follow the same `(map_prism_error -32000 INTERNAL_ERROR)` annotation pattern — `E-STORE-020` v2.51 is now consistent.

**Spec-only note:** fix-burst-26 touched only `error-taxonomy.md` and `STATE.md`/`SESSION-HANDOFF.md` (state bookkeeping). No code or test changes. PR HEAD 9e116a01 is UNCHANGED — pass-16 gates on the same frozen HEAD. No streak-reset-by-push (DRIFT-ORCH-PRLEVEL-PUSH-001: spec-only burst does not push to the fix branch).

---

### F-MCPRS-PRL15-LOW-001 [LOW][POL-24 message-format drift] — CLOSED @9e116a01 spec-only (fix-burst-26: error-taxonomy v2.51 Message Format cell corrected; PR HEAD UNCHANGED)

**Severity:** LOW
**Classification:** POL-24 message-format drift — pre-existing byte-level drift between the taxonomy `Message Format` cell for `E-STORE-020` and the shipped `Display` implementation. Taxonomy v2.50 read `"cursor cap exceeded (200 active cursors); wait for existing cursors to expire..."`, while the shipped `Display` impl for `PrismError::CursorCapExceeded` emits `"cursor cap exceeded: cannot allocate more than 200 active cursors"`. Additionally, the v2.50 Message Format phrasing `"wait for existing cursors to expire"` implied retryability that contradicted the v1.19 `retryable: false` semantics.
**Status:** CLOSED — fix-burst-26 spec-only (error-taxonomy v2.51 Message Format cell updated; PR HEAD UNCHANGED)

**Finding:** Taxonomy `E-STORE-020` Message Format cell (v2.50): `"cursor cap exceeded (200 active cursors); wait for existing cursors to expire or close existing connections to free capacity"`. Shipped `Display` implementation for `PrismError::CursorCapExceeded`: `"cursor cap exceeded: cannot allocate more than 200 active cursors"`.

Discrepancies:
1. **Delimiter:** taxonomy uses `(200 active cursors)` with parentheses; shipped uses `: cannot allocate more than 200 active cursors` with colon + full phrase.
2. **Phrasing:** taxonomy `"wait for existing cursors to expire or close existing connections"` vs. shipped message (no such phrase in the Display impl itself; suggestion is carried in the structured MCP error via `map_prism_error`).
3. **Retryability conflict:** the phrase `"wait for existing cursors to expire"` implies eventual retryability with the same params — contradicting `retryable: false` established in BC-2.10.007 v1.19.

The mirror-code convention (established by `E-INT-001` and `E-QUERY-034` which match their Display strings verbatim) requires the Message Format cell to reproduce the shipped Display string byte-for-byte.

**Severity rationale:** LOW because: (1) this is spec-vs-spec drift (taxonomy Message Format vs. Display impl), not a behavioral defect; (2) the shipped behavior is correct per BC-2.10.007 v1.19; (3) the drift is pre-existing (not introduced by fix-burst-25); (4) the retryability conflict with v1.19 is an internal taxonomy inconsistency only, not a code-level issue.

**Fix plan — fix-burst-26:** PO corrects `E-STORE-020` Message Format cell to verbatim shipped Display: `"cursor cap exceeded: cannot allocate more than 200 active cursors"`. Removes the speculative "wait for existing cursors to expire" phrasing (retryability/action is conveyed by `retryable: false` + `map_prism_error` structured suggestion, not by the Display message).

**Closure evidence (fix-burst-26, error-taxonomy v2.51, spec-only — PR HEAD 9e116a01 UNCHANGED):**

(1) **`E-STORE-020` Message Format cell** corrected to: `"cursor cap exceeded: cannot allocate more than 200 active cursors"` — byte-for-byte match with shipped `PrismError::CursorCapExceeded` Display impl.

(2) **Mirror-code convention verified:** `E-INT-001` Message Format = `"Internal server error"` (matches Display); `E-QUERY-034` Message Format = `"internal error"` (matches Display); `E-STORE-020` v2.51 = `"cursor cap exceeded: cannot allocate more than 200 active cursors"` (matches Display). Convention-compliant maintenance.

(3) **Retryability conflict resolved:** speculative `"wait for existing cursors to expire"` phrasing removed from Message Format. The `retryable: false` field in BC-2.10.007 v1.19 and the structured `map_prism_error` suggestion are now the sole authoritative retryability/action signals.

(4) **Spec-only note:** No code change. The Display impl was already correct. This is pure taxonomy prose correction.

---

## SAP-1 Emission Catalog Probe

**PASS.** `crates/` `event_type =` emission sites at HEAD 9e116a01 sampled against BC-2.16.002 §Postconditions Canonical Structured Event Catalog — all catalogued. Fix-burst-26 changes (`error-taxonomy.md` v2.50→v2.51 prose-only + STATE.md/SESSION-HANDOFF.md bookkeeping) introduced zero new `event_type =` emissions. No BC-2.16.002 catalog row required.

---

## Positive Verifications

- **EC-11-079 single `with_explicit_nulls(true)` chokepoint:** `server.rs` `WriterBuilder` construction confirmed as sole `WriterBuilder` site in `prism-mcp/src/`; null-not-absent contract enforced at one gated location; no regression from fix-burst-26 (spec-only).
- **5 Red Gate tests:** `test_null_column_is_explicit_null`, `test_non_null_column_not_absent`, `test_absent_column_when_not_in_schema`, `test_all_null_values_explicit`, `test_mixed_null_non_null_row_shape` — all confirmed present and GREEN at 9e116a01 (no code change).
- **BC-2.10.007 v1.19 CursorCapExceeded category fields:** category `"internal"`, `original_params_valid: true`, `retryable: false`, `ec_code_override: Some("E-STORE-020")`, suggestion `"Cursor capacity exhausted. Wait for existing cursors to close before retrying."` — confirmed present at 9e116a01 from fix-burst-25 closure.
- **28-arm explicit VariantMeta groups:** 13 internal / 2 validation / 3 configuration / 10 upstream_error-explicit (as established post-fix-burst-25: CursorCapExceeded moved to internal group). Unchanged at 9e116a01.
- **117-variant sentinel coverage:** `CursorCapExceeded` E-STORE section arm comment reads `"internal"` — consistent with production mapping. No regression.
- **`test_BC_2_10_007_cursor_cap_exceeded_category_is_internal` (5 assertions):** `category == "internal"`, `original_params_valid == true`, `retryable == false`, `ec_code_override == Some("E-STORE-020")`, suggestion contains `"Cursor capacity exhausted"` — confirmed present and GREEN at 9e116a01.
- **480/480 prism-mcp; 261/261 prism-core** at 9e116a01 (fix-burst-25 baseline; fix-burst-26 spec-only — no test count change).
- **error-taxonomy v2.51 sibling-convention sweep:** `E-INT-001` and `E-QUERY-034` Description cells confirmed with `(map_prism_error -32000 INTERNAL_ERROR)` prefix — `E-STORE-020` v2.51 now consistent.

---

## Summary

**CLEAN(strict): NO** (1 MED + 1 LOW — no severity is zero-finding)
**CLEAN(PR-merge): NO** (1 MED finding — MED blocks both CLEAN(strict) and CLEAN(PR-merge) per BC-5.39.001)

Streak: **0/3** (streak cannot advance; MED finding present).

Both findings CLOSED via fix-burst-26 (spec-only — error-taxonomy v2.50→v2.51; PR HEAD 9e116a01 UNCHANGED):
- **F-MCPRS-PRL15-MED-001 CLOSED (fix-burst-26 spec-only):** `E-STORE-020` Description column annotated `(map_prism_error -32000 INTERNAL_ERROR)` + terse message + operator-resolvable note; category `"internal"`, `original_params_valid: true`; sibling-convention compliant; PR HEAD 9e116a01 unchanged.
- **F-MCPRS-PRL15-LOW-001 CLOSED (fix-burst-26 spec-only):** `E-STORE-020` Message Format cell updated to verbatim shipped Display `"cursor cap exceeded: cannot allocate more than 200 active cursors"`; speculative retryability phrasing removed; mirror-code convention satisfied; PR HEAD 9e116a01 unchanged.

**Spec-only note:** fix-burst-26 touched `error-taxonomy.md` only (plus state bookkeeping). PR HEAD remains 9e116a01. No streak-reset-by-push applies (DRIFT-ORCH-PRLEVEL-PUSH-001: no push to fix branch).

CASCADE TALLY: 35 passes / 26 fix-bursts. HEAD @9e116a01 UNCHANGED (still LOCAL-ONLY; push pending to origin/fix/DEFECT-MCP-ROWSHAPE-NULLS-001); next = PR-LEVEL pass 16 on frozen @9e116a01 (streak 0/3).
