---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [18]
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
streak_after: 2/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 18 — FIX-IEQ-ERRPATH-001

---

## Pass 18 (frozen 35117a38; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 2/3 — ADVANCES to 2/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK ADVANCE: 1/3 → 2/3** (BC-5.39.001; second consecutive CLEAN(strict) on frozen 35117a38)

**Code HEAD at review:** 35117a38 (frozen; D-1628 fix-burst: EC-11-076 per-reference HEAD-JOIN scoping — replaced name-keyed bare_head_cols HashSet<String> with per-reference (name, is_bare) pairs; 5391/5391 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** YES (zero findings of any severity — streak advancement criterion satisfied)

**CLEAN(PR-merge):** YES (zero CRIT + HIGH + MED findings — PR-merge gate satisfied)

---

## Probe Coverage

All probes returned empty-handed. Full coverage:

### MATERIALIZATION.RS DELTA AUDIT

**check_temporal_literals mut-ast binding (1-line delta from prior passes):**

- Reviewed `materialization.rs` for the `check_temporal_literals` function: the mut-ast binding change is a pure refactor of the mutable AST traversal variable — no behavioral change to the temporal literal validation path. The DRIFT-IEQ interaction surface is zero: `check_temporal_literals` operates on the AST before `check_query_column_availability` runs; there is no code path where temporal literal position errors influence E-QUERY-038 availability checks or vice versa. No DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 interaction found. CLEAN.

### TABLE_REGISTRY.RS ~20-LINE DELTA AUDIT

**check_availability_gate dot-notation dispatch, columns_for_table 3-empty-case docs, column_type_for E-QUERY-041 gate:**

- `check_availability_gate`: dot-notation dispatch delegates to the correct registry lookup path. No new code paths added that bypass the E-QUERY-038 guard. BC-2.11.001 (table availability gating) and BC-2.11.016 (column availability gating) both correctly applied in the new dispatch arm.
- `columns_for_table` 3-empty-case docs: documentation-only change (no executable behavior change). The three empty-return cases (unregistered table, multi-tenant non-matching tenant, zero-column table) are correctly documented and the code behavior matches the doc. ADR-052 temporal coercion path is independent of this function.
- `column_type_for E-QUERY-041 gate`: the E-QUERY-041 (TemporalLiteralInvalidPosition) gate invoked via `column_type_for` follows the correct lookup chain. No interaction with E-QUERY-038 (non-existent column) — the two gates are sequentially independent in the execution pipeline. ALIGNED with BC-2.11.001, BC-2.11.016, ADR-052 §D4. CLEAN.

### BC_2_11_019_N1B_TEST.RS FIXTURE CROSS-CHECK

**4-column fixture — all test columns present, no E-QUERY-038 pre-emption:**

- Read `bc_2_11_019_n1b_test.rs`: all 4 columns referenced in the fixture (`severity_id`, `severity`, `device_name`, `timestamp`) are present in the test sensor schema registered for that test. No fixture column maps to a non-existent schema column. E-QUERY-038 is therefore not pre-empted by a legitimate column-not-found for fixture columns — any E-QUERY-038 emission in these tests is driven by the test's explicit typo injection, not accidental fixture drift. CONFIRMED CORRECT at @35117a38.

### T13-PREFLIGHT-AUDIT.PY G1–G8 EXTENSION (62→70 LINES) AUDIT

**G1–G8 load-bearing, runtime-derived expectations, honest WARN fallbacks, fail_count==0 gate:**

- Reviewed the G1–G8 extension in `scripts/t13-preflight-audit.py`. All 8 new gate checks (G1–G8) are load-bearing: each check verifies a distinct runtime-observable behavior (IEQ operator string emission, did_you_mean payload, Levenshtein bound, SEC-002 size guard, operator-type exclusions, SQL-mode E-QUERY-001 precedence, multi-tenant parity, FP-001 re-introduction shape) via actual query invocation against the live engine — not mocked or hardcoded.
- Runtime-derived expectations: G1–G8 derive expected values from the running system (via query execution), not from hardcoded literals. This prevents the test from passing vacuously on a wrong implementation.
- Honest WARN fallbacks: where runtime conditions are unavailable (e.g., no DTU endpoint reachable), the gates emit WARN (not PASS) and do NOT advance `pass_count`. This prevents false positives on incomplete environments.
- `fail_count==0` gate: the script's exit criterion is `fail_count == 0` (hard failures only). WARNs do not affect exit code. Gate semantics are correct.
- ALIGNED with T13 capstone audit requirements. CLEAN.

### INLINE TEST MODULE SAMPLE — 72 TESTS NO TAUTOLOGIES

**Full inline test module review (sample of 72 tests):**

- Sampled 72 tests from the inline `#[cfg(test)] mod tests` block in `prism-query`. No tautological assertions found (no `assert_eq!(expected, expected)`, no assertions that would pass regardless of implementation). Each test exercises a distinct behavioral point: specific operator, specific suspension rule, specific error payload field. Load-bearing criterion satisfied per TD-VSDD-059.
- No `#[ignore]` annotations on the load-bearing IEQ/IIN/INE error-path tests introduced in this cascade. All 6 EC-11-076 tests remain present and GREEN. CONFIRMED INTACT at @35117a38.

### SINGLE-VS-MULTI-TENANT PARITY — ALL SIX SUSPENSION RULES

**Parity across EC-11-054/055 (enrich), EC-11-070/071 (STAR-WITH-JOIN), EC-11-072 (STAGE-JOIN), EC-11-074/075 (HEAD-JOIN), EC-11-076 (PER-REFERENCE):**

- Each suspension rule reviewed for single-tenant vs multi-tenant parity:
  1. **EC-11-054/055 (enrich registry-wired/absent):** single and multi-tenant paths both thread the `InfusionRegistry` Option correctly. No parity gap.
  2. **EC-11-070/071 (STAR-WITH-JOIN):** STAR position suspension applies identically in both paths (no tenant-specific override). PARITY OK.
  3. **EC-11-072 (STAGE-JOIN):** stage-scoped schema lookup is tenant-aware via the registry; single-tenant fallback path preserved. PARITY OK.
  4. **EC-11-074/075 (HEAD-JOIN SUSPENSION):** bare-head suspension applies identically in both execution paths (execute/execute_scheduled). PARITY OK.
  5. **EC-11-076 (PER-REFERENCE):** per-reference (name, is_bare) extraction is schema-lookup-agnostic; the tenant context only affects which schema is consulted, not the extraction logic. PARITY OK.
- Zero parity gaps found across all six suspension rules. CLEAN.

### E-QUERY-038 PAYLOAD CORRECTNESS

**sort+dedup, did_you_mean Levenshtein ≤3 lex tie-break, DID_YOU_MEAN_MAX_NAME_BYTES=128 SEC-002 at all 3 sites:**

- **sort+dedup on `available_columns`:** Verified at all 3 emission sites in `check_query_column_availability`. The available_columns list passed to E-QUERY-038 is sorted + deduped before inclusion. No site skips this step. CONFIRMED.
- **did_you_mean Levenshtein ≤3 with lex tie-break:** The `levenshtein_distance` guard (`dist <= 3`) and lexicographic tie-break (`cmp` on equal distances) are applied consistently at all 3 E-QUERY-038 emission sites. No site uses a looser distance bound or omits the tie-break. CONFIRMED.
- **DID_YOU_MEAN_MAX_NAME_BYTES=128 SEC-002 guard:** The 128-byte cap on column names in the `did_you_mean` hint is present at all 3 emission sites. No site allows an unbounded column name to transit the error payload. SEC-002 log-injection guard: column names are truncated before inclusion in the structured error field. CONFIRMED at @35117a38.

### IEQ/IIN/INE OPERATOR-STRING EMISSION + VALID_OPERATORS_FOR_TYPE EXCLUSIONS

**operator-string emission accuracy and exclusion completeness:**

- `IEQ`, `IIN`, `INE` operator strings are emitted correctly in E-QUERY-002 (QueryTypeMismatch) payloads when type-pair validation rejects a non-String RHS. The operator strings match the canonical names defined in BC-2.11.024.
- `valid_operators_for_type` correctly EXCLUDES `IEQ`/`IIN`/`INE` from numeric, boolean, datetime, and JSON type classes — these operators are String-only per BC-2.11.024 §D1. No type class inadvertently includes case-insensitive operators. CONFIRMED.
- Conversely, `valid_operators_for_type` correctly INCLUDES `IEQ`/`IIN`/`INE` for String type. No over-exclusion. CONFIRMED.

### BC-2.11.024 SQL-MODE E-QUERY-001 PARSE-TIME PRECEDENCE

**SQL-mode IEQ on non-String operand → E-QUERY-001 (parse-time) takes precedence over E-QUERY-038 (plan-time):**

- In SQL-mode (non-pipe), an `IEQ` expression with a non-String RHS triggers E-QUERY-001 (UnknownOperator / parse-time rejection) before plan-time column checking reaches E-QUERY-038. The gate ordering is: parse-time type-pair validation → plan-time column availability. A mis-typed `IEQ` cannot mask a non-existent column by short-circuiting; conversely a non-existent column is not falsely reported as a type error. Precedence ordering is CORRECT per BC-2.11.024 §D4. CONFIRMED at @35117a38.

### FP-001 RE-INTRODUCTION SHAPE — DERIVED-SEEDED CORRECTLY

**JOIN + | stats count() as col | where col — DERIVED-seeded FP-001 shape:**

- Shape: `SELECT * FROM t1 JOIN t2 ON t1.id = t2.id | stats count() as col | where col = 'x'` — `col` in the WHERE stage is a DERIVED column from the stats stage. The E-QUERY-038 gate at the WHERE stage must consult the binding context seeded by the stats output (DERIVED seed), not the raw schema. At @35117a38, this path correctly seeds DERIVED bindings from the stats stage into the WHERE stage column-availability check. E-QUERY-038 does NOT fire on `col` (it is a known DERIVED binding). FP-001 re-introduction via DERIVED-seeded stats+JOIN shape: NEGATIVE RESULT (no false-positive). CONFIRMED CORRECT.

### POL-7/21/25/26/27/29/32 SPEC-LAYER CHECKS

All policy checks returned PASS:

| Policy | Check | Result |
|--------|-------|--------|
| POL-7 | BC frontmatter `lifecycle_status` field present (not deprecated `lifecycle`) | PASS |
| POL-21 | No new TODO/FIXME/HACK in spec artifacts at @35117a38 | PASS |
| POL-25 | BC version pins in carrier stories match current BC versions (BC-2.11.016 v1.21, BC-2.11.017 v1.9, BC-2.11.020 v1.14, BC-2.11.004 v1.26, error-taxonomy v2.34) | PASS |
| POL-26 | ADR superseded-by back-refs are bidirectional (ADR-052 §D4 Option A temporal typing; ADR-047 case-insensitive operators) | PASS |
| POL-27 | No `file.rs:NNN` line-number citations in narrative spec content (TD-VSDD-091) | PASS |
| POL-29 | Story version pins round-trip through all cascade carrier stories with zero stale live pins | PASS |
| POL-32 | BC status transitions consistent with POL-14 auto-promotion at merge | PASS |

### PRIOR-PASS FIX PROPAGATION VERIFICATION

**No stale v1.20 pins remaining (BC-2.11.016 v1.20 was the pre-EC-11-076 version superseded at D-1628):**

- Grepped all carrier story files (`S-DEMO-FIDELITY-REMEDIATION-001`, `S-DEMO-PRISMQL-ONBOARDING-001-B`, `S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001`, `S-PRISMQL-CASE-INSENSITIVE-001`) for `v1.20` references to BC-2.11.016. ZERO occurrences of stale `BC-2.11.016 v1.20` found. All 4 carrier stories correctly cite `v1.21`. Prior-pass fix propagation VERIFIED. CLEAN.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 35117a38. Five total emission sites verified: three `column_not_found.rejected` sites + two `reload.*` sites. All five catalog rows exist in BC-2.16.002 §Postconditions with full field schema, audit role, and recurrence policy. Zero new `event_type` assignments introduced at @35117a38 (the D-1628 fix-burst replaced HashSet<String> with per-reference pairs — a pure logic refactor; no new tracing emissions). SAP-1 coverage UNCHANGED from pass-17.

**SAP-2 N/A:** No sensor TOML spec modifications in this fix cascade.

**POL-24 N/A (this pass):** No new EC body authored in this burst (pass-18 is CLEAN; no fixes).

**Audit-script Section G arithmetic:** G1–G8 extension (62→70) reviewed; EC-11-076 (added D-1628) remains the latest EC in BC-2.11.016 v1.21; section G count 73 (pass-17 established); no additions this pass.

---

## Convergence Assessment

**Trajectory:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)→[0]→[0]

**Pattern:** Pass 18 probes the @35117a38 fix-branch HEAD with rotated angles not covered in passes 16/17: materialization.rs delta isolation, table_registry.rs gate dispatch, bc_2_11_019_n1b_test.rs fixture integrity, t13-preflight-audit.py G1–G8 extension review, 72-test inline module tautology audit, single-vs-multi-tenant parity across all 6 suspension rules, E-QUERY-038 payload completeness (sort+dedup, Levenshtein ≤3, DID_YOU_MEAN_MAX_NAME_BYTES=128), IEQ/IIN/INE operator emission + exclusion completeness, BC-2.11.024 SQL-mode parse-time precedence, FP-001 DERIVED-seeded re-introduction shape, POL-7/21/25/26/27/29/32 spec-layer audit, and prior-pass fix propagation. All probes returned empty-handed.

**Novelty assessment:** LOW — Pass 18 rotated to angles complementary to pass-17 (which covered FP-001 shape battery, extractor symmetry, dead-code residue, callsite symmetry). No new vulnerability surface found. The per-reference (name, is_bare) implementation at @35117a38 is consistent with BC-2.11.016 v1.21 semantics across all probed dimensions.

**Streak status:** 2/3 (BC-5.39.001). VERY NEXT ACTION: freeze 35117a38 → LOCAL adversary pass 19 (fresh context, strict; streak candidate 3/3). No commits to fix branch per DRIFT-ORCH-PRLEVEL-PUSH-001. Three consecutive CLEAN(strict) passes on unchanged HEAD (passes 17/18/19) → then push branch + open fix-PR via pr-manager.
