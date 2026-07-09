---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [19]
feature_head_at_review: 35117a38
date: 2026-07-09
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 3/3
convergence: CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 19 — FIX-IEQ-ERRPATH-001

---

## Pass 19 (frozen 35117a38; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 3/3 — ADVANCES to 3/3 — LOCAL CASCADE CONVERGED)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK ADVANCE: 2/3 → 3/3 — LOCAL strict-3-CLEAN CONVERGED (BC-5.39.001; three consecutive CLEAN(strict) on frozen 35117a38: passes 17/18/19)**

**Code HEAD at review:** 35117a38 (frozen; D-1628 fix-burst: EC-11-076 per-reference HEAD-JOIN scoping — replaced name-keyed bare_head_cols HashSet<String> with per-reference (name, is_bare) pairs; 5391/5391 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** YES (zero findings of any severity — streak advancement criterion satisfied; 3-CLEAN COMPLETE)

**CLEAN(PR-merge):** YES (zero CRIT + HIGH + MED findings — PR-merge gate satisfied)

---

## LOCAL CASCADE CONVERGED

**BC-5.39.001 3-CLEAN criterion met:** Passes 17/18/19 all CLEAN(strict) on unchanged HEAD 35117a38. Frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001) satisfied — no commits to fix branch between passes 17 and 19.

**Trajectory final:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)→[0]→[0]→[0]. 19 passes total.

**NEXT ACTION:** Push fix branch to origin → pr-manager 9-step fix-PR cycle targeting develop (closes DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 + DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001 audit-script 62→70 extension; unblocks T13 capstone).

---

## Probe Coverage

All probes returned empty-handed. Full coverage — rotated angles not covered in passes 17/18:

### END-TO-END GATE-PRECEDENCE TRACES — FIVE QUERY JOURNEYS

**Five complete query execution traces through check_query_column_availability and execute():**

1. **Filter IEQ nonexistent column** (`SELECT * FROM crowdstrike_alerts WHERE severity_id IEQ 'High'` — `severity_id` absent from schema): Trace: parse-time operator validation passes (IEQ valid for String positions) → plan-time check_query_column_availability → E-QUERY-041 absent (temporal literal not present) → E-QUERY-037 absent (table registered) → E-QUERY-038 fired (severity_id not in schema) → did_you_mean Levenshtein ≤3 lex tie-break → sort+dedup → payload emitted. E-QUERY-038 fires BEFORE any IEQ type-check attempt. Gate ordering E-QUERY-041→037→038→039 CONFIRMED. No gate suppression by IEQ operator.

2. **SqlPipe `|where` IIN nonexistent column** (`SELECT * FROM crowdstrike_alerts | where unknown_col IIN ('a','b')`): Trace: SQL parse passes → pipe stage check_pipe_stage_columns → unknown_col not in binding context → E-QUERY-038 emitted. IIN does not mask the column-not-found check. Gate ordering CONFIRMED.

3. **EC-11-076 per-reference** — qualified reference sharing a bare name in positions 1–6: `SELECT t.col FROM t JOIN t2 ON t.id = t2.id WHERE t.col = 'x'` where `col` appears bare in position 3. Qualified `t.col` (segments.len()==2) is NOT suspended → E-QUERY-038 fires if `t.col` absent from `t`. Bare `col` at head position suspended per EC-11-076. Both judgements consistent with BC-2.11.016 v1.21. CONFIRMED CORRECT at @35117a38.

4. **EC-11-061 anonymous head aggregate** — `SELECT count(*) FROM t | stats count() as cnt | sort cnt`: DERIVED binding `cnt` seeded by stats stage into sort check; no E-QUERY-038 on `cnt`. FP-001 shape battery COVERED. CONFIRMED CORRECT.

5. **EC-11-072 stage-join suspension** — `SELECT * FROM t1 JOIN t2 ON t1.id = t2.id | where ...`: stage-join bare-head suspension in positions 1–6 behaves identically to pass-18 verification. No regression introduced by per-reference change. CONFIRMED.

### GATE ORDERING E-QUERY-041→037→038→039

**Verified incl. documented ADR-052 §D4 temporal masking:**

- E-QUERY-041 (TemporalLiteralInvalidPosition): checked first in temporal-literal path; does NOT interfere with non-temporal column-not-found gate. Temporal and column-availability checks are independent code paths (different function calls; no shared early-return). ADR-052 §D4 Option A confirmed non-masking. CONFIRMED.
- E-QUERY-037 (TableNotRegistered): fires before column-lookup (no schema to check). Gate ordering preserved. CONFIRMED.
- E-QUERY-038 (ColumnNotFound): fires after table registration confirmed, at column availability check. CONFIRMED.
- E-QUERY-039 (AggregateNotAllowed): fires on aggregate-in-WHERE position, independent of column-not-found. No interaction with IEQ/IIN/INE column path. CONFIRMED.

### HEAD-JOIN × E-QUERY-002 INTERACTION — FAIL-OPEN ANALYSIS

**RwLock-poison TOCTOU analysis of two-lock sequence; fail-open safe:**

- The HEAD-JOIN suspension rule (EC-11-074/075/076) and E-QUERY-002 (QueryTypeMismatch) are evaluated in sequence: column-availability gate runs before type-pair gate. If column-not-found fires (E-QUERY-038), execution does NOT proceed to E-QUERY-002 for that position. Fail-open in EC-11-074/075: if head-join bare-ref suspension fires, the column is skipped (no E-QUERY-038 AND no E-QUERY-002). No TOCTOU between the two-check sequence — each check is a pure function over the binding context snapshot; no shared mutable state between E-QUERY-038 and E-QUERY-002 evaluation. SAFE.

### EXTRACTOR BARENESS SYMMETRY — ALL 12 PUSH-SITES

**extract_field_paths_with_bareness vs _from_expr; collect_predicate_columns_with_bareness vs _columns — all 12 call sites:**

- Verified all 12 push-sites that call bare-aware extraction functions. No site mixes bare-aware and bare-unaware extractors for the same semantic purpose. Per-reference (name, is_bare) pairs flow consistently from extraction through suspension rule evaluation. No asymmetry introduced at @35117a38. CLEAN.

### COMPUTE_SQLPIPE_HEAD_BINDING BRANCHES A/B/C

**Three branch paths through compute_sqlpipe_head_binding:**

- Branch A (SELECT with bare cols): binds bare column names to schema type; bareness flag preserved. CORRECT.
- Branch B (SELECT with qualified cols): binds as QUALIFIED; not suspended by EC-11-076. CORRECT.
- Branch C (STAR): all schema columns seeded; no bareness issue (STAR suspension covered by EC-11-070/071). CORRECT.
- No fourth branch introduced at @35117a38. EXHAUSTIVE.

### DERIVED-PROVENANCE E-QUERY-002 SKIP

**DERIVED-seeded binding skips type-pair validation on first-pass column-presence check:**

- In the stats-seeded binding context, DERIVED bindings (cnt, sev) are recorded with their DERIVED provenance. The check_pipe_stage_columns implementation skips E-QUERY-002 type-pair validation for columns that are DERIVED (their type is inferred, not schema-locked). This is correct: a stats-derived alias is a new binding, not a schema column — type-checking against schema types is inapplicable. FP-001 DERIVED-seeded re-introduction shape confirmed NEGATIVE (no false-positive E-QUERY-038 on DERIVED bindings). CONFIRMED at @35117a38.

### IEQ/IIN/INE OPERATOR EMISSION + VALID_OPERATORS_FOR_TYPE

**Operator string emission accuracy and exclusion completeness — fresh-angle sample:**

- Sampled `valid_operators_for_type` for Float and Boolean type classes: neither includes IEQ/IIN/INE. Float permits EQ/NE/LT/LE/GT/GE only. Boolean permits EQ/NE only. Case-insensitive operators correctly excluded. No over-inclusion. CONFIRMED.
- Sampled `valid_operators_for_type` for String: includes IEQ, IIN, INE alongside EQ/NE/IN/NIN. No over-exclusion. CONFIRMED.

### TIMESTAMPARITHMETIC GRAMMAR CLOSURE (NOW-BASED ONLY)

**ADR-052 §D4 temporal dispatch — NOW-based arithmetic only, no constant-fold path:**

- TimestampArithmetic grammar closure confirmed: NOW() ± INTERVAL expressions form the only temporal-literal arithmetic path that reaches E-QUERY-041. Constant-literal timestamps (ISO-8601 strings, numeric epoch) go through the standard column-type coercion path (check_temporal_literals), not the arithmetic path. No interaction with E-QUERY-038 column-not-found gate. ADR-052 §D4 temporal masking analysis COMPLETE. CONFIRMED.

### INSUBQUERY OUTER-FIELD-ONLY SCOPE

**InSubquery outer-field-only scope at @35117a38:**

- Subquery column references are resolved against the outer query's schema only (no implicit cross-join from subquery). E-QUERY-038 fires on outer-scope non-existent columns in subquery position. No subquery-specific false-negative path introduced at @35117a38. CONFIRMED.

### CLIENT_ID PAYLOAD INJECTION-SAFETY (SEC-002)

**OrgSlug charset + SEC-002 at E-QUERY-038 emission sites:**

- DID_YOU_MEAN_MAX_NAME_BYTES=128 guard confirmed at all 3 E-QUERY-038 emission sites (fresh-angle verification complementary to pass-18). OrgSlug charset: alphanumeric + hyphens only; no injection-capable characters admitted by OrgSlug::new() validation. SEC-002 log-injection guard: column names truncated and sanitized before inclusion in structured error field. No new E-QUERY-038 emission site introduced at @35117a38. CONFIRMED.

### TEST-INTEGRITY SPOT-CHECK (POL-16/TAUTOLOGY)

**14-position spec-code alignment vs BC-2.11.016 v1.21; audit-script COVERAGE_MATRIX structural review:**

- Spot-checked 6 EC-11-076 tests (pass-19 fresh sample): test_bc_2_11_016_ec11_076_per_reference_{1..4} + 2 negative controls. Each test exercises a distinct behavioral point (qualified ref not suspended vs bare ref suspended). No tautological assertion pattern. No POL-16 violation. Load-bearing criterion satisfied per TD-VSDD-059. CONFIRMED.
- Audit-script COVERAGE_MATRIX structural review: G1–G8 column headers match BC-2.11.024 operator surface. No structural gap introduced at @35117a38. CONFIRMED.

### CHANGED-FILE DIFF-SURFACE ANCHORING

**Diff surface at @35117a38 vs develop@f935edb6 — fresh-angle review:**

- Changed files at @35117a38 relative to develop: `crates/prism-query/src/materialization.rs` (main implementation); `crates/prism-query/src/lib.rs` (function signature updates); `crates/prism-query/tests/` (6 EC-11-076 RED→GREEN test files + prior fix-burst test accumulation); `.factory/specs/behavioral-contracts/bc-2-11-016.md` v1.21; supporting carrier story files. No changed file outside this perimeter. Diff surface anchoring complete — no stale or orphaned change detected at @35117a38. CONFIRMED.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 35117a38. Five total emission sites verified: three `column_not_found.rejected` sites + two `reload.*` sites. All five catalog rows exist in BC-2.16.002 §Postconditions with full field schema, audit role, and recurrence policy. Zero new `event_type` assignments introduced at @35117a38. SAP-1 coverage UNCHANGED from passes 17/18.

**SAP-2 N/A:** No sensor TOML spec modifications in this fix cascade.

**POL-24 N/A (this pass):** No new EC body authored in this burst (pass-19 is CLEAN; no fixes).

**Audit-script Section G arithmetic:** G1–G8 extension (62→70) coverage UNCHANGED from pass-18. EC-11-076 (added D-1628) remains the latest EC in BC-2.11.016 v1.21; section G count 73 (pass-17 established); no additions this pass.

---

## Convergence Assessment

**Trajectory:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)→[0]→[0]→[0]

**Pattern:** Pass 19 probed @35117a38 with rotated angles not covered in passes 17/18: end-to-end gate-precedence traces through execute() for 5 query journeys (Filter IEQ nonexistent col; SqlPipe |where IIN; EC-11-076 per-reference; EC-11-061 anonymous head aggregate; EC-11-072 stage-join); gate ordering E-QUERY-041→037→038→039 verified incl. ADR-052 §D4 temporal masking; HEAD-JOIN × E-QUERY-002 interaction fail-open; extractor bareness symmetry all 12 push-sites; RwLock-poison TOCTOU analysis; compute_sqlpipe_head_binding branches a/b/c; DERIVED-provenance E-QUERY-002 skip; IEQ/IIN/INE operator emission + valid_operators_for_type; TimestampArithmetic grammar closure (NOW-based only); InSubquery outer-field-only scope; client_id payload injection-safety (OrgSlug charset + SEC-002); test-integrity spot-check (no POL-16/tautology); 14-position spec-code alignment vs BC-2.11.016 v1.21; audit-script COVERAGE_MATRIX structural review; changed-file diff-surface anchoring. All probes returned empty-handed.

**Novelty assessment:** LOW — Pass 19 rotated to angles complementary to passes 17/18. No new vulnerability surface found. The per-reference (name, is_bare) implementation at @35117a38 is consistent with BC-2.11.016 v1.21 semantics across all probed dimensions, including the newly-probed gate-ordering and diff-surface anchoring angles.

**Streak status:** 3/3 — **LOCAL strict-3-CLEAN CONVERGED** (BC-5.39.001). Passes 17/18/19 all CLEAN(strict) on unchanged HEAD 35117a38. Frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001) satisfied. NEXT: push fix branch to origin → pr-manager 9-step fix-PR cycle (closes DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 + DRIFT-AUDIT-SCRIPT-UNCOMMITTED-001 audit-script 62→70 extension; unblocks T13 capstone).
