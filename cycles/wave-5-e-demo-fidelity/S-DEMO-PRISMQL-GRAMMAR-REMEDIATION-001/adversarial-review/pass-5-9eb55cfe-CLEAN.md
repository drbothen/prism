---
document_type: adversarial-review-pass
pass: 5
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 9eb55cfe
story_version: v1.6
clean_strict: true
clean_pr_merge: true
finding_count: 0
cascade_streak_after: 1
reviewer: vsdd-factory:adversary
timestamp: 2026-06-25T00:00:00Z
---

# LOCAL Adversary Pass 5 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen HEAD:** `9eb55cfe`
**Story version read:** v1.6 (TD-VSDD-091 engine.rs location-pin remediation applied)
**Diff reviewed:** `903c8fcb..9eb55cfe`
**CLEAN(strict):** YES
**CLEAN(PR-merge):** YES
**Finding count:** 0

---

## Adversary Verdict

**CLEAN(strict): YES — ZERO findings of any severity (CRIT / HIGH / MED / LOW / OBS / PROCESS-GAP).**
**CLEAN(PR-merge): YES.**

3-CLEAN streak: **1 / 3** (streak began on this pass against frozen HEAD `9eb55cfe`; frozen-HEAD streak rule DRIFT-ORCH-PRLEVEL-PUSH-001 applies — any push resets to 0/3).

---

## Directed Checks (All PASS)

### 1. SAP-1 — Tracing emission catalog completeness

Grepped `event_type =` across `crates/` workspace. All emission sites present in
the BC-2.16.002 §Postconditions Canonical Structured Event Catalog:

- `filter.sql_lowering` — row present (added in Pass 3 / D-1340 PO burst, BC-2.16.002 v1.90)
- `filter.sql_planning_error` — row present (added same burst)
- `pipe.sql_lowering` — row present (pre-existing)
- `pipe.sql_planning_error` — row present (pre-existing)

No new emission sites added in 9eb55cfe diff. **SAP-1 PASS.**

### 2. SAP-2 — DTU↔TOML schema parity

No `.prism/specs/sensors/*.toml` or DTU clone files modified in this diff.
**SAP-2 N/A.**

### 3. SID-1 — No-ignored-test rationalization prohibition

All acceptance-criteria tests in this diff are non-`#[ignore]` unit tests in
`#[cfg(test)] mod tests` blocks, driving real production code paths. No deferred
integration stubs observed. **SID-1 PASS.**

### 4. BC-2.11.023 D1 mode-bridge message (AC-009)

`filter_parser.rs` SELECT-arm: verbatim BC-2.11.023 §D1 message verified in-place
with all three mandated substrings:
- `(enrich, where, limit, sort, stats, dedup, fields)` — present
- `1. SQL+pipe composition:` / `2. Pipe mode only:` numbered alternatives — present
- `See prismql://reference for the complete grammar.` — present

AC-009 test asserts all three substrings with explicit negative control (no raw
Chumsky token list). **PASS.**

### 5. BC-2.11.023 D2 mode-bridge diagnostic (AC-027)

`rewrite_d2_sql_keyword_in_pipe_position` in `error_recovery.rs` — present and
wired via `pipe_parser.rs`. Verbatim BC-2.11.023 §D2 message verified. Test
`test_bc_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position` confirmed
non-`#[ignore]`, load-bearing. WHERE/LIMIT uppercase correctly excluded.
**PASS.**

### 6. AC-023 IS-NOT-NULL JSON-list semantics (BC-2.11.022)

`build_reference_content` in `resources.rs` — verified verbatim IS-NOT-NULL-on-JSON-list
semantics note present (added in D-1342 fix-burst closure). Test
`test_bc_2_11_022_ac023_json_list_is_not_null_note` confirmed present and
non-`#[ignore]`, asserting actual note text. JSON-list materialization path
(`column_type_to_arrow` + `build_column_array` in `spec_driven_adapter.rs`): empty
list `[]` → Arrow Utf8 non-null string → IS NOT NULL = true; JSON null → Arrow null
→ false. Note accurately describes this behavior. **PASS.**

### 7. TD-VSDD-091 function-name anchors (story v1.6)

Story v1.6 §Architecture Mapping: five stale `engine.rs` file-location hints
replaced with function-name anchors (`inject_now`, `plan_sqlpipe_query`,
`run_materialization_pipeline`, `execute_against_session`). Verified against
shipped code — `inject_now` and `plan_sqlpipe_query` live in `lib.rs`,
`execute_against_session` in `materialization.rs`. No volatile line-number
citations remain in non-exempt narrative content. `engine.rs::normalize_pql`
reference at AC-010 and Previous Story Intelligence preserved (accurate).
**PASS.**

### 8. Anti-tautology CI gate

CI `EXPECTED=83` non-exhaustive compile-fail gate unchanged (no new public types
added in 9eb55cfe diff that lack `#[non_exhaustive]`). **PASS.**

### 9. VERIFIED-CLEAN carry-forward (do NOT reflag)

All items verified CLEAN in prior passes; confirmed no regression in 9eb55cfe diff:
- Temporal NOW()/INTERVAL production wiring — all 4 AST arms present
- FORBID-BOTH 0-row hoist Step 1b
- Filter-mode load-bearing tests
- E-QUERY-036 / E-QUERY-040 verbatim messages
- SAP-1 catalog complete (filter.* + pipe.* rows)
- Arc-DI BC-2.10.015 (OrgSlug::new + slug_exists)
- BC-2.10.016/017 fast-fail guard order
- SID-1 PASS
- SAP-2 N/A

---

## Implementation Quality Note

Implementation on HEAD `9eb55cfe` is unusually disciplined. No paper-fixes detected
(TD-VSDD-059). All behavioral fixes have load-bearing assertions with explicit
negative controls. Sibling-site sweep (TD-VSDD-060) followed correctly
(pipe_parser.rs + error_recovery.rs wired together). Anti-tautology gates explicit.
BC-2.11.023 §D1/§D2 verbatim message fidelity verified byte-level.

---

## Summary

**ZERO findings. This pass is CLEAN(strict) and CLEAN(PR-merge).**

3-CLEAN streak: **1 / 3** against frozen HEAD `9eb55cfe`.

> Note: This pass was retroactively counted as clean pass #1 toward the streak
> on `9eb55cfe`. The streak was subsequently reset to 0/3 by the fix-burst
> that closed pass-6 MED-1 (code HEAD advanced `9eb55cfe → 64d91111`).
> Per DRIFT-ORCH-PRLEVEL-PUSH-001, the streak restarts against the new HEAD.
