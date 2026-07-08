---
document_type: adversarial-review
scope: LOCAL
passes: [22]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 2de85b18
fix_burst_head: null
date: 2026-07-07
clean_strict: true
clean_pr_merge: true
finding_counts: {}
streak_after: 1/3
---

# LOCAL Adversary Pass 22 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 22 (frozen 2de85b18; delta ~39 files vs develop@ea714d14)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES. Zero findings.
**Findings:** NONE
**Code HEAD at review:** 2de85b18 (FROZEN — no fix-burst; no new commit)
**LOCAL 3-CLEAN(strict) streak after pass-22:** 1/3

---

## Finding Inventory

None. Zero findings of any severity.

---

## Verification Summary

### Functional correctness

**IEQ/IIN/INE grammar and AST:** `filter_parser.rs` combinator ordering confirmed correct — `IIN` parsed before `IN` (longest-match-first; RG-002 load-bearing); `case_insensitive: true` flag propagates through `Predicate::Compare` and `Predicate::In` to DataFusion `lower()` lowering in `pipe_sql_emitter.rs`. All 73 RGTs present and named in story v1.27 §Red Gate Tests.

**Predicate construction site audit (TD-VSDD-060 sibling-sweep):** All `Predicate::Compare{...}` and `Predicate::In{...}` struct-literal construction sites in the delta carry an explicit `case_insensitive` field (true or false). No site was found that could silently default to an incorrect value. 151 construction sites swept across `filter_parser.rs`, `ast.rs`, `pipe_sql_emitter.rs`, `explain.rs`, `pushdown.rs`, `normalized_pql.rs`, and test files.

**PRIMARY↔SECONDARY normalization parity:** `build_column_array` (prism-bin `spec_driven_adapter.rs`) and `normalize_with_mappers` (prism-ocsf `normalizer.rs`) share the single-source `OCSF_ENUM_LABEL_FIELDS` pub const from prism-ocsf. No divergence between PRIMARY and SECONDARY normalization paths. Empty-string passthrough parity verified (RG-047 PRIMARY + RG-054 SECONDARY mirror).

**Forbidden pattern grep — CLEAN:**
- No `prism_spec_engine::types::ColumnType::Int64` / `::Float64` / `::Timestamp` in delta
- No `lifecycle: active` BC frontmatter introduced
- No `reqwest::Client::new()` without `.timeout()` in delta
- No `println!` in production code in delta
- No `unwrap()` / `expect()` on `Result` in non-test paths in delta

**Explain path (pass-21 fix re-verified):** F-P21-OBS-001 closure confirmed at 2de85b18 — `predicate_to_exprs` guard arms `case_insensitive: true => vec![]` present for both `Predicate::Compare` and `Predicate::In`; RG-073 `test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable` GREEN. Runtime pushdown path `collect_equality_exprs` still correctly guards `case_insensitive: false` only. No regression.

**Pushdown correctness (pass-9 fix re-verified):** IEQ/IIN/INE predicates excluded from `collect_equality_exprs` (case_insensitive:false guard at pushdown.rs:299). InSubquery arm in `collect_ci_fields_inner` handles nested CI predicates. No regression.

**Domain entity coverage:** All required entities from the story's §Domain Model (PrismQL filter expressions, OCSF enum-label fields, prism describe output, round-trip normalizer, mode boundaries) exercised by RGTs. Story v1.27 entity table complete.

---

## SAP Probe Results (Pass 22, verified against 2de85b18)

**SAP-1 (tracing emission catalog completeness):** PASS — `rg 'event_type\s*=' crates/ --type rust` run against 2de85b18 delta. Only `event_type` value introduced in this story's delta is `ocsf.enum_label_unrecognized` (dual sites: `build_column_array` in `spec_driven_adapter.rs` + `normalize_with_mappers` in `normalizer.rs`). Both match BC-2.16.002 catalog row 91 (field schema, audit role, recurrence policy). Catalog count UNCHANGED 91. SEC-002 size-cap guard (BC-2.16.002 row 91 value_truncated_to field) verified present at both emission sites. No new `event_type` values introduced.

**SAP-2 (DTU↔TOML schema parity):** N/A — delta does not touch `.prism/specs/sensors/*.toml` or DTU clone types/routes.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 73 RGTs are non-`#[ignore]` unit tests within `#[cfg(test)]` module blocks; no external dependency required for any test.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BC anchors verified in story v1.27 body (BC-2.11.024, BC-2.02.013, BC-2.11.002, BC-2.11.004, BC-2.11.018, BC-2.02.002, BC-2.02.010, BC-2.10.012). E-QUERY-002 byte-verbatim error code verified present in error-taxonomy.md. All BC IDs resolve to files in `.factory/specs/behavioral-contracts/`.

**POL-22 Phase C (RGT inventory completeness):** PASS — all 73 RGT names (RG-001..RG-073) and all domain entities verified present in story v1.27 §Red Gate Tests table.

---

## Post-Pass State

- Feature HEAD: **2de85b18** (FROZEN — streak advancing; no new push)
- LOCAL 3-CLEAN(strict) streak: **1/3**
- 1406/1406 prism-query tests at 2de85b18 (from pass-21 gate; re-verify full workspace at 3/3)
- RG-001..073 GREEN
- Novelty: NONE
- NEXT ACTION: LOCAL adversary pass-23 on SAME frozen 2de85b18 (streak candidate 2/3)
