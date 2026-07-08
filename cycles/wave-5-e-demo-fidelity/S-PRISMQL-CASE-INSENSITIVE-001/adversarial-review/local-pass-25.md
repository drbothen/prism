---
document_type: adversarial-review
scope: LOCAL
passes: [25]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 633c5fab
fix_burst_head: b341cdd7
date: 2026-07-08
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 1, OBS: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 25 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 25 (frozen 633c5fab; fresh-context adversary; 44 files vs develop@ea714d14; streak candidate 1/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 MED + 1 OBS), CLEAN(PR-merge)=NO (MED finding blocks)
**Findings:** 2 (F-P25-MED-001 MED — CLOSED; F-P25-OBS-001 OBS — CLOSED)
**Code HEAD at review:** 633c5fab (frozen)
**Fix-burst HEAD (new frozen candidate for pass-26):** b341cdd7
**Fix-burst commits on feature branch:** b341cdd7 (implementer: volatile line-number pins replaced with function-name anchors — `explain.rs` ×3 comments `pushdown.rs:299` → `pushdown.rs::collect_equality_exprs`; `test_adapter_normalization.rs` ×3 comments `normalizer.rs:146` → `normalizer.rs::normalize_with_mappers`; comment-only, no logic change; 1407/1407 prism-query GREEN)
**LOCAL 3-CLEAN(strict) streak after pass-25:** 0/3 (RESET by fix-burst push b341cdd7 per DRIFT-ORCH-PRLEVEL-PUSH-001)
**Next:** LOCAL pass-26 on frozen b341cdd7

---

## Finding Inventory

### F-P25-MED-001 (MED) — Citation error: bc_2_10_016_audit_004_test.rs path cited prism-query instead of prism-mcp (POL-22 / POL-4)

**Finding:** Two artifacts cited the non-existent path `crates/prism-query/tests/bc_2_10_016_audit_004_test.rs`:

1. **Story v1.28 narrative (~line 936):** The narrative described the F-P24-LOW-001 closure referring to `bc_2_10_016_audit_004_test.rs` under `crates/prism-query/tests/`. The file lives at `crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs` — it is an integration test in the `prism-mcp` crate, not `prism-query`. The `prism-query` crate has no `bc_2_10_016_audit_004_test.rs` file.

2. **local-pass-24.md (~line 56):** Same wrong path in the F-P24-LOW-001 finding body: "In `crates/prism-query/tests/bc_2_10_016_audit_004_test.rs`...". The F-P24-LOW-001 finding itself was valid and correctly closed; only the path citation was wrong.

Additionally, the story v1.28 narrative's vocabulary-list claim mislisted the removed entries: it described `'OPEN'` as a removed entry (no such entry exists in the test's ALL-CAPS block) and omitted `'MEDIUM'` and `'LOW'` which were also removed. The actual ALL-CAPS entries removed were `'HIGH'`, `'CRITICAL'`, `'MEDIUM'`, `'LOW'` — consistent with a typical severity vocabulary.

**Severity:** MED — POL-22 Phase B file-path citation integrity violation; POL-4 spec accuracy. The wrong path cites a file that does not exist in `prism-query/tests/`; any agent attempting to locate the file would fail. The finding is documentation-integrity, not a runtime defect.

**Closure:** CLOSED — story v1.29 (story-writer, this burst): path corrected to `crates/prism-mcp/tests/bc_2_10_016_audit_004_test.rs` at ~line 936; vocabulary-entry list corrected from (`'HIGH'`, `'CRITICAL'`, `'OPEN'`, etc.) to (`'HIGH'`, `'CRITICAL'`, `'MEDIUM'`, `'LOW'`). local-pass-24.md line 56: path corrected with inline note `(path corrected pass-25 F-P25-MED-001)`.

---

### F-P25-OBS-001 (OBS) — Six volatile line-number pins in comments (TD-VSDD-091)

**Finding:** The branch introduced six `file.rs:NNN` line-number pins in code comments — a pattern explicitly forbidden by TD-VSDD-091 (narrative spec content must cite function names + behavioral anchors, not line numbers):

**In `crates/prism-query/src/explain.rs`** (3 sites):
- Three comments each citing `pushdown.rs:299` as the runtime counterpart to the explain-classification path. Line 299 of `pushdown.rs` is `collect_equality_exprs`, not a stable anchor — any preceding insertion shifts the reference.

**In `crates/prism-mcp/tests/test_adapter_normalization.rs`** (3 sites):
- Three comments each citing `normalizer.rs:146` as the location of `normalize_with_mappers`. Same decay risk.

All six pins appeared in comments added by the branch (not pre-existing develop baseline); all point at production functions whose line numbers will shift with future insertions.

**Severity:** OBS — TD-VSDD-091 compliance; no runtime impact; future diff-decay risk only.

**Closure:** CLOSED fix-in-scope implementer @b341cdd7 — all six comments updated to cite function names:
- `explain.rs` ×3: `pushdown.rs:299` → `pushdown.rs::collect_equality_exprs`
- `test_adapter_normalization.rs` ×3: `normalizer.rs:146` → `normalizer.rs::normalize_with_mappers`

Comment-only changes; no logic altered. 1407/1407 prism-query GREEN.

---

## SAP Probe Results (Pass 25, verified against 633c5fab)

**SAP-1 (tracing emission catalog completeness):** PASS — `ocsf.enum_label_unrecognized` dual sites match BC-2.16.002 catalog row 91. No new `event_type` values introduced in the 633c5fab delta. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU changes in this delta.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — all 74 Red Gate tests are non-`#[ignore]` unit tests; no external dependency waivers present.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BC anchors and E-QUERY-002 reference verified present in story v1.29.

**POL-22 Phase C (RGT inventory completeness):** PASS-with-1-MED (the F-P25-MED-001 citation) — story v1.29 corrects path; all 74 RGT names (RG-001..RG-074) verified present post-correction. All domain entities present.

**TD-VSDD-059 (paper-fix detection — F-P24-MED-001 re-verification):** PASS — RG-074 `test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators` is load-bearing (asserts the 8-operator set); `error_mapping.rs` derives `valid_operators` dynamically from `valid_operators_for_type` — fix is structural, not a doc-comment or rename.

**Novelty:** MEDIUM — citation integrity finding (POL-22 path discipline) is novel to this pass and not a recurrence of a prior-pass category.

---

## Fix-Burst Commit Log (feature/S-PRISMQL-CASE-INSENSITIVE-001)

| Commit | Author | Change |
|--------|--------|--------|
| b341cdd7 | implementer | `explain.rs` ×3 comments `pushdown.rs:299` → `pushdown.rs::collect_equality_exprs`; `test_adapter_normalization.rs` ×3 comments `normalizer.rs:146` → `normalizer.rs::normalize_with_mappers`; comment-only TD-VSDD-091 volatile-pin sweep; 1407/1407 prism-query GREEN; F-P25-OBS-001 CLOSED |

---

## Post-Fix-Burst State

- Feature HEAD: **b341cdd7** (new frozen candidate for pass-26)
- 1407/1407 prism-query tests GREEN
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by fix-burst push b341cdd7 per DRIFT-ORCH-PRLEVEL-PUSH-001)
- Novelty: MEDIUM (citation integrity — POL-22 path discipline finding)
- NEXT ACTION: LOCAL adversary pass-26 on frozen b341cdd7
