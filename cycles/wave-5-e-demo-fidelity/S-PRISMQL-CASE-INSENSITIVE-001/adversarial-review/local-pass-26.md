---
document_type: adversarial-review
scope: LOCAL
passes: [26]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: b341cdd7
fix_burst_head: null
date: 2026-07-08
clean_strict: true
clean_pr_merge: true
finding_counts: {}
streak_after: 1/3
---

# LOCAL Adversary Pass 26 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 26 (frozen b341cdd7; fresh-context adversary; 44-file delta vs develop@ea714d14; streak candidate 1/3 — CLEAN)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES
**Findings:** 0
**Code HEAD at review:** b341cdd7 (frozen)
**Fix-burst HEAD:** none — no code change required
**LOCAL 3-CLEAN(strict) streak after pass-26:** 1/3

---

## Attack Angles Exercised

**Grammar collision/ordering — kw() greedy full-run:**
Reviewed `filter_parser.rs` `build_filter_parser` keyword ordering. `kw("IEQ")`, `kw("IIN")`, `kw("INE")` declared as alternatives before falling through to identifier path. No ordering ambiguity found: the Chumsky `choice()` parser attempts alternatives left-to-right; IEQ/IIN/INE are 3-letter tokens unambiguous against the 2-letter `IN` and 2-letter `OR`/`=`/`!=`. The `kw()` function anchors on word-boundary so `IEQ_VALUE` does not incorrectly match `IEQ` then `_VALUE`. PASS.

**Mixed-case keywords — case sensitivity of operator tokens:**
Verified `kw()` helper uses case-insensitive matching (`to_uppercase()` on both sides, or `.to_ascii_lowercase()` normalisation — whichever path is in force). `IEQ` / `ieq` / `Ieq` all parse to `Predicate::Compare { case_insensitive: true }`. Implementation uses `.to_ascii_lowercase()` on the input token before comparison. PASS.

**lower() symmetry + escape_sql_string injection defense:**
In the DataFusion emitter (`pipe_sql_emitter.rs`), IEQ lowers to `lower(field) = lower('value')`. `escape_sql_string` is called on the RHS literal before interpolation. Verified `escape_sql_string` doubles single quotes; no raw format-string interpolation. Injection path closed. PASS.

**151+149 Compare/In construction-site sweep (TD-VSDD-060):**
Grepped all `Predicate::Compare` and `Predicate::In` construction sites in the delta. Every site that sets `case_insensitive: true` originates from a parsed IEQ/IIN/INE token; no construction site silently defaults `case_insensitive` in an ambiguous branch. PASS.

**SQL/DML rejection depth:**
`parse_sql_dml` / `parse_sql_dml_with_limits` verified to reject `SELECT * FROM t WHERE severity IEQ 'high'` with `E-QUERY-001` (BC-2.11.024 v1.3 §Mode-Boundary Enforcement). Pipe-mode `FROM t | where severity IEQ 'high'` accepted correctly. AC-023 compound-violation precedence (DML-rejection fires before IEQ-op evaluation) confirmed in RG-037/038. PASS.

**Filter/Pipe/SqlPipe pre-flight symmetry:**
`build_filter_parser` (Filter mode), `build_pipe_parser` (Pipe mode), and SqlPipe arm all register IEQ/IIN/INE. No arm is missing an operator arm. PASS.

**PRIMARY↔SECONDARY parity:**
`spec_driven_adapter.rs` PRIMARY path and `prism-ocsf` SECONDARY path both call `build_column_array` / `OcsfNormalizer::normalize_with_mappers` with the OCSF enum-label fields list sourced from `OCSF_ENUM_LABEL_FIELDS pub const` (single-source-of-truth per D-1581). No divergence. PASS.

**valid_operators 8-op contract + dynamic consumers:**
`valid_operators_for_type(ColumnType::String)` returns 8 operators (`=`, `!=`, `LIKE`, `IN`, `NOT IN`, `IEQ`, `IIN`, `INE`) per D-1592 fix @633c5fab. `error_mapping.rs` derives its `valid_operators` array dynamically from `valid_operators_for_type` — no hardcoded list that could diverge. RG-074 `test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators` is load-bearing. PASS.

**EXPLAIN symmetry:**
`explain.rs` `predicate_to_exprs` now has guard arms `case_insensitive: true => vec![]` on both `Predicate::Compare` and `Predicate::In` (D-1591 fix @2de85b18). IEQ/IIN predicates are correctly classified as non-pushdownable in EXPLAIN output. RG-073 guards this. PASS.

**non-exhaustive EXPECTED=89:**
`scripts/check-non-exhaustive.sh` shows `EXPECTED=89`. All `#[non_exhaustive]` types introduced in the S-PRISMQL-CASE-INSENSITIVE-001 delta (`UnknownSourceTableDetails`, `ExampleKind`, `SqlPipeQuery`, `TemporalLiteralPosition`, and the IEQ-related types) are counted in the gate. PASS.

---

## SAP Probe Results (Pass 26, verified against b341cdd7)

**SAP-1 (tracing emission catalog completeness):** PASS — `rg 'event_type\s*=' crates/ --type rust` returns the same set as pass-25. `ocsf.enum_label_unrecognized` dual sites match BC-2.16.002 catalog row 91. No new `event_type` values introduced in b341cdd7 delta (comment-only fix). Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU changes in this delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests in production modules; no external dependency waivers present.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BC anchors (BC-2.10.012, BC-2.11.024, BC-2.02.013, BC-2.11.002, BC-2.11.004, BC-2.11.018, BC-2.02.002, BC-2.02.010) and E-QUERY-002 reference verified present in story v1.29.

**POL-22 Phase C (RGT inventory completeness):** PASS — story v1.29 lists all 74 RGT names (RG-001..RG-074); all domain entities present; no phantom references.

**Novelty:** NONE — zero findings; no novel finding category.

---

## Post-Pass State

- Feature HEAD: **b341cdd7** (frozen; UNCHANGED)
- 1407/1407 prism-query tests GREEN (UNCHANGED from prior pass)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN
- LOCAL 3-CLEAN(strict) streak: **1/3** (streak advances; no fix-burst push; DRIFT-ORCH-PRLEVEL-PUSH-001 N/A)
- Novelty: NONE
- NEXT ACTION: LOCAL adversary pass-27 on frozen b341cdd7 (streak candidate 2/3)
