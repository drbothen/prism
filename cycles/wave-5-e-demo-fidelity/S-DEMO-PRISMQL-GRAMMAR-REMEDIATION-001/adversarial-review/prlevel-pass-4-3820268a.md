---
document_type: adversarial-review
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: prlevel-4
pr: "#203"
pr_head: "3820268a"
base_develop: "903c8fcb"
result: NOT_CLEAN
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
findings_total: 2
findings_high: 1
findings_obs: 1
date: 2026-06-25
state_decision: D-1350
---

# PR-LEVEL Adversarial Pass 4 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**PR HEAD reviewed:** `3820268a`
**Base develop:** `903c8fcb`
**Date:** 2026-06-25
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**3-CLEAN streak:** RESET 0/3 on `216e19fa`

---

## Summary

Pass 4 on PR HEAD `3820268a` found 2 findings (1 HIGH + 1 OBS). Both CLOSED before streak advance.

- HIGH-1 (TD-VSDD-060 sibling-sweep / S-7.01 partial-fix regression): 10 of 11 demo VHS tapes referenced uncommitted `/tmp/run_*.sh` drivers — the prior OBS-2 fix at `3820268a` touched only AC-017/018's tape; 9 other tapes remained non-reproducible from a clean checkout. Additionally AC-002 GIF showed stale pre-v2.00 E-QUERY-040 wording. CLOSED by demo-recorder (feature HEAD `c5833b9d`→`216e19fa`): committed 10 driver scripts into evidence dir, repointed all 10 non-017/018 tapes to committed paths, re-recorded 2 stale GIFs (AC-001-003 + AC-002 with v2.00 neutral E-QUERY-040 wording), left 9 unchanged GIFs intact; all 11 tapes now reproducible.

- OBS-1 (pre-existing normalizer correctness, newly reachable via the AC-011 Filter arm): apostrophe-bearing string literal (e.g. `name = "O'Brien"`, valid via double-quoted PQL literal) mis-normalized to a DataFusion double-quoted IDENTIFIER instead of an escaped SQL string literal. CLOSED by implementer (code `3820268a`→`c5833b9d`): Filter arm switched from `normalize_predicate_pub` to `predicate_to_datafusion_sql` (uses `escape_sql_string` → `'O''Brien'`), unifying with the Pipe-mode SQL emitter; `normalize_predicate_pub`/`emit_quoted_string` left intact (correct for PQL round-trips per BC-2.11.018); load-bearing test `test_filter_mode_string_with_embedded_apostrophe_executes_correctly` added (real MemTable, asserts 1 matching row). `just check` EXIT=0 (4930 tests).

---

## Findings

### HIGH-1 — Demo evidence partial-fix regression: 10/11 tapes non-reproducible from committed artifacts (TD-VSDD-060 / S-7.01)

**Severity:** HIGH
**Status:** CLOSED — feature HEAD `c5833b9d`→`216e19fa` (demo-recorder)

**Finding:** The OBS-2 fix in Pass 3 (`3820268a`) committed only the AC-017/018 driver script. The remaining 10 demo VHS tapes (AC-001-003, AC-004, AC-005, AC-006, AC-007, AC-008, AC-009, AC-010, AC-011, AC-012..AC-016) still referenced `/tmp/run_*.sh` paths that are not committed to the repository. Any clean checkout would fail to reproduce these tapes — a direct violation of demo evidence reproducibility requirements (POL-10). Additionally, the AC-001-003 and AC-002 GIFs pre-dated the v2.00 error_taxonomy update and showed the old `| limit`-only E-QUERY-040 wording.

**Closure:** demo-recorder performed a full sibling sweep per TD-VSDD-060:
- Committed 10 driver scripts into `docs/demo-evidence/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001/` evidence directory
- Repointed all 10 tape configurations to committed script paths (no `/tmp/` references remain)
- Re-recorded AC-001-003 GIF (with corrected v2.00 `| limit`/`| tail` neutral wording) and AC-002 GIF (same correction)
- 9 other GIFs verified current; left unchanged
- All 11 tapes now fully reproducible from committed artifacts alone
- Feature HEAD after closure: `216e19fa`

---

### OBS-1 — Filter-arm apostrophe string mis-normalized to DataFusion IDENTIFIER (pre-existing, newly reachable)

**Severity:** OBS (pre-existing normalizer correctness gap, newly reachable via AC-011 Filter arm)
**Status:** CLOSED — code `3820268a`→`c5833b9d` (implementer)

**Finding:** A double-quoted PQL string literal containing an apostrophe — e.g. `name = "O'Brien"` (valid PQL; double-quoted literal per grammar) — was processed by `normalize_predicate_pub` in the Filter arm's SQL emission path. `normalize_predicate_pub`/`emit_quoted_string` wraps in DataFusion double-quote syntax (`"O'Brien"`), which DataFusion interprets as a column IDENTIFIER, not a string literal. The correct SQL emission is `'O''Brien'` (single-quoted with escaped apostrophe). This is a correctness bug: a query filtering on an apostrophe-bearing string would either produce incorrect results (wrong column reference) or a DataFusion parse error.

**Root cause:** The Filter arm (introduced by AC-011) called `normalize_predicate_pub` for SQL emission instead of `predicate_to_datafusion_sql`. The Pipe-mode SQL emitter already used `predicate_to_datafusion_sql` (which calls `escape_sql_string`); the Filter arm was inconsistent.

**Closure:** Implementer switched the Filter arm from `normalize_predicate_pub` to `predicate_to_datafusion_sql`, unifying Filter and Pipe-mode SQL emission:
- `normalize_predicate_pub` and `emit_quoted_string` left intact — correct for PQL round-trip normalization per BC-2.11.018; they are not SQL emitters
- `predicate_to_datafusion_sql` now handles Filter-arm SQL emission (consistent with Pipe-mode)
- Load-bearing test `test_filter_mode_string_with_embedded_apostrophe_executes_correctly` added: real MemTable with `name` column, query `SELECT * FROM t WHERE name = "O'Brien"`, asserts exactly 1 matching row returned
- `just check` EXIT=0 (4930 tests — count bumped by +1 new test)
- Feature HEAD after closure: `c5833b9d`

---

## OBS-2 — D2 fires on bare ORDER without BY

**Severity:** OBS (carry-forward DO-NOT-FLAG)
**Status:** NOT A DEFECT — intentional helpful superset of BC §D2 semantics. Benign; no action. Per D-1349 standing adjudication.

---

## Probes Passing

All other standing adversary probes PASS on `3820268a`:
- AC-001..AC-016 grammar acceptance criteria: PASS
- AC-017/018 driver scripts: PASS (committed at `3820268a`; full sweep now at `216e19fa`)
- AC-019 BLOCKER-001 deferral (D-1326): DO-NOT-FLAG per standing exemption
- AC-020 runbook v1.4 satisfied: PASS
- AC-021..AC-027: PASS
- FORBID-BOTH / E-QUERY-040 trigger (Limit + Tail): PASS
- Temporal plain-string handling (D-1335): DO-NOT-FLAG
- E-QUERY-036/037 label distinction: PASS
- SAP-1 tracing emission catalog: PASS
- AD-017 credential redaction: PASS
- Non-exhaustive EXPECTED=87 (pre-4930 check): PASS
- fmt-canonical: PASS
- `just check` EXIT=0 on `c5833b9d`/`216e19fa` (4930 tests): PASS

---

## Closure Summary

| Finding | Severity | Closed By | Commit |
|---------|----------|-----------|--------|
| HIGH-1 10/11 tapes non-reproducible + 2 stale GIFs | HIGH | demo-recorder | `c5833b9d`→`216e19fa` |
| OBS-1 Filter-arm apostrophe mis-normalized | OBS | implementer | `3820268a`→`c5833b9d` |

**New FROZEN PR HEAD after all closures:** `216e19fa`
**`just check` on `c5833b9d`:** EXIT=0 (4930 tests)
**non-exhaustive:** 87 (unchanged; no new pub types added)
**fmt-canonical:** CLEAN
**3-CLEAN streak RESET:** 0/3 on `216e19fa` (code HEAD moved by fix commits)
**CI:** re-runs on `216e19fa` push (in progress)
