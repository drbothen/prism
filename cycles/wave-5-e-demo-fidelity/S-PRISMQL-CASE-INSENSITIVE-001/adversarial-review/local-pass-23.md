---
document_type: adversarial-review
scope: LOCAL
passes: [23]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 2de85b18
fix_burst_head: null
date: 2026-07-07
clean_strict: true
clean_pr_merge: true
finding_counts: {}
streak_after: 2/3
---

# LOCAL Adversary Pass 23 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 23 (frozen 2de85b18; fresh-context adversary; independent of pass-22)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES. Zero findings.
**Findings:** NONE
**Code HEAD at review:** 2de85b18 (FROZEN — no fix-burst; no new commit)
**LOCAL 3-CLEAN(strict) streak after pass-23:** 2/3

---

## Finding Inventory

None. Zero findings of any severity.

---

## Attack Angle Coverage

This pass exercised adversarial angles not emphasized in prior passes, to provide maximum coverage diversity before declaring 3/3:

### Angle 1 — Turkish-I and Unicode case folding

**Probe:** Does `lower()` (DataFusion's case fold function) correctly handle non-ASCII input? Is there a risk of Turkish-I (`I` → `ı`, not `i`) or similar Unicode collation hazards?

**Verdict:** Not a defect. The OCSF enum-label fields (`severity`, `status`, `activity_name`, `disposition`) use ASCII-only caption strings per the OCSF specification. `enum_map.rs` builds its lookup map using Rust `str::to_lowercase()` (Unicode-aware), and DataFusion's `lower()` SQL function applies the same Unicode-aware lowercase to the query literal before comparison. For ASCII-only enum labels the two implementations are identical in behavior. A Turkish-locale user writing `WHERE severity IEQ 'HIGH'` gets the same result as a US-locale user — both paths produce `'high'` after lowercasing, matching the lowercase-indexed `OcsfEnumMap` entry. No collation hazard for ASCII-only OCSF captions.

### Angle 2 — NULL 3VL via DataFusion lower()

**Probe:** DataFusion's `lower()` function propagates NULL — if a row has a NULL `severity` column, does `lower(severity) = lower('High')` return NULL (correct 3VL) or accidentally match?

**Verdict:** Not a defect. DataFusion's SQL semantics treat NULL correctly: `lower(NULL) = lower('high')` evaluates to NULL, which is neither TRUE nor FALSE, so the row is excluded from a WHERE clause filter. This is standard SQL 3VL behavior. No production code special-cases NULL in the IEQ path; the DataFusion planner handles it correctly via its native expression evaluation. This is the desired behavior per the story spec (IEQ is semantically equivalent to `lower(lhs) = lower(rhs)` for non-NULL values).

### Angle 3 — SQL injection via escape_sql_string (pass-21 SEC-001 re-verification)

**Probe:** Re-verify that `escape_sql_string` (single-quote doubling) correctly handles adversarial inputs: embedded single quotes, double quotes, null bytes, Unicode control characters.

**Verdict:** PASS re-verified. The `escape_sql_string` function replaces `'` with `''` (standard SQL escaping), which neutralizes the primary SQL injection vector for string literals in DataFusion's SQL string representation. No null byte or Unicode control character can break the DataFusion SQL parser's string literal scanning. The RG-049 parse-lock test (7 shapes) exercises this path. SEC-001 closure from pass-21 re-verified intact at 2de85b18.

### Angle 4 — Compound-violation precedence (per story AC-023)

**Probe:** What happens when a query uses IEQ in a SQL-mode SELECT (mode-boundary violation) AND references a non-existent column? Does the mode-boundary rejection take priority over the column-not-found error?

**Verdict:** Not a defect. AC-023 explicitly documents that the SQL-mode rejection (`E-QUERY-001`) fires at parse time, before column resolution occurs. The test `test_bc_2_11_024_sql_mode_ieq_rejected_e_query_001` verifies this precedence. No regression.

### Angle 5 — Cross-mode CI-typecheck parity (Filter/Pipe/SqlPipe)

**Probe:** The `check_ci_column_types` function gates IEQ/IIN/INE on ColumnType::String. Does this gate apply identically across Filter mode, Pipe mode, and SqlPipe mode?

**Verdict:** PASS verified. All three paths call `check_ci_column_types` before dispatching to the DataFusion lower() lowering. Pass-16 (D-1586) added this parity guard. RG-065/066 (check_ci_column_types guard tests) GREEN at 2de85b18. The `check_ci_column_types` function is the single shared check point — no path bypasses it.

### Angle 6 — Prior-closure propagation (F-P20 and earlier fixes intact)

**Probe:** Re-verify that significant prior findings remain closed at 2de85b18: F-P20-HIGH-001 (RG-071 PRIMARY GROUP BY cross-sensor), F-P20-LOW-001 (has_severity ColumnType::String guard), F-P17-HIGH-1 (server.rs ENUM CASING CONTRACT), F-P11-OBS-001 (OCSF_ENUM_LABEL_FIELDS single-source), F-P10-CRIT-1 (armis triage IIN→IN ('High','Critical')).

**Verdict:** All CLOSED. RG-071 at `bc_2_11_024_primary_group_by_test.rs` confirmed present. has_severity gate at `prism-mcp/src/server.rs` confirmed ColumnType::String guard. ENUM CASING CONTRACT confirmed in description text. OCSF_ENUM_LABEL_FIELDS confirmed as `pub const` in prism-ocsf only. Armis triage uses `IN ('High','Critical')`. No regressions.

### Angle 7 — 151 Compare/In construction sites sweep

**Probe:** Are there any `Predicate::Compare{...}` or `Predicate::In{...}` struct-literal construction sites that omit the `case_insensitive` field (which would default via struct-update syntax or be a compile error)?

**Verdict:** PASS. Rust does not allow struct-update syntax (` ..Default::default()`) for structs without a `Default` impl. All construction sites must either provide `case_insensitive` explicitly or fail to compile. All 151 construction sites verified to carry `case_insensitive: false` (production non-CI paths) or `case_insensitive: true` (CI grammar path). No site found that could produce an incorrect default.

---

## SAP Probe Results (Pass 23, verified against 2de85b18)

**SAP-1 (tracing emission catalog completeness):** PASS — same as pass-22. `ocsf.enum_label_unrecognized` dual sites match BC-2.16.002 catalog row 91. SEC-002 caps verified. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU changes.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 73 RGTs non-`#[ignore]` unit tests.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BCs, E-QUERY-002, all references verified.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 73 RGT names (RG-001..RG-073) verified present in story v1.27 §Red Gate Tests table. All domain entities present.

---

## Post-Pass State

- Feature HEAD: **2de85b18** (FROZEN — streak at 2/3; no new push)
- LOCAL 3-CLEAN(strict) streak: **2/3**
- 1406/1406 prism-query tests at 2de85b18
- RG-001..073 GREEN
- Novelty: NONE
- NEXT ACTION: LOCAL adversary pass-24 on SAME frozen 2de85b18 (streak candidate 3/3 — if CLEAN, 3-CLEAN achieved)
