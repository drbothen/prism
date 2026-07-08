---
document_type: adversarial-review
scope: LOCAL
passes: [24]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 2de85b18
fix_burst_head: 633c5fab
date: 2026-07-07
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 1, LOW: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 24 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 24 (frozen 2de85b18; fresh-context adversary; streak candidate 3/3 — NOT CLEAN)

**Pass result:** CLEAN(strict)=NO (1 MED + 1 LOW), CLEAN(PR-merge)=NO (MED finding blocks)
**Findings:** 2 (F-P24-MED-001 MED, F-P24-LOW-001 LOW — both CLOSED fix-in-scope)
**Code HEAD at review:** 2de85b18 (frozen)
**Fix-burst HEAD (new frozen candidate for pass-25):** 633c5fab
**Fix-burst commits on feature branch:** 633c5fab (implementer: `engine.rs` `valid_operators_for_type(ColumnType::String)` String arm 5→8 operators, siblings `e_query_pedagogical.rs` 5→8, `normalized_pql.rs` comments updated; RG-074 RED→GREEN; dead ALL-CAPS entries removed + comment corrected in `bc_2_10_016_audit_004_test.rs`; 1407/1407 prism-query + 447/447 prism-mcp GREEN)
**LOCAL 3-CLEAN(strict) streak after pass-24:** 0/3 (RESET by fix-burst push 633c5fab per DRIFT-ORCH-PRLEVEL-PUSH-001)
**Next:** LOCAL pass-25 on frozen 633c5fab

---

## Finding Inventory

### F-P24-MED-001 (MED) — engine.rs valid_operators_for_type(String) omitted IEQ/IIN/INE from E-QUERY-002 StructuredErrorFields contract

**Finding:** In `crates/prism-query/src/engine.rs`, the function `valid_operators_for_type` returns a `Vec<&'static str>` of operator strings used to populate the `valid_operators` field of `E-QUERY-002`'s `StructuredErrorFields` type. When `column_type == ColumnType::String`, the function returned only 5 operators: `["=", "!=", "LIKE", "IN", "NOT IN"]`. The three new case-insensitive operators `IEQ`, `IIN`, and `INE` were absent.

This means any agent or user receiving an E-QUERY-002 error (e.g., using a numeric column with IEQ) and parsing the `valid_operators` array from the structured error response would never learn that `IEQ`, `IIN`, and `INE` are valid string-column operators. The Display prose `SuggestedSuffix` (shown in `--` notes on structured error output) mentioned the CI operators correctly, but the machine-readable `valid_operators` array contradicted it and contradicted BC-2.11.024 v1.3 §Postconditions which mandates the array be authoritative.

The sibling functions in `e_query_pedagogical.rs` showed the same gap: the `required_string` operator list for pedagogical messages also had 5 operators, missing IEQ/IIN/INE. The `normalized_pql.rs` source comment listing valid string operators was also stale.

**Severity:** MED — machine-readable E-QUERY-002 structured error contract gives agents an incomplete operator list; pedagogical string-operator table stale; BC-2.11.024 v1.3 contract postcondition violated. Not a runtime data-corruption defect but a discovery contract defect that would cause LLM agents to under-utilize IEQ/IIN/INE after receiving an error.

**Closure:** CLOSED fix-in-scope per production-grade default. Implementer @633c5fab:
- `engine.rs` `valid_operators_for_type(ColumnType::String)` String arm expanded from 5 to 8 operators: `["=", "!=", "LIKE", "IN", "NOT IN", "IEQ", "IIN", "INE"]`. Note: negated IIN (`!IIN`) intentionally absent — not grammatically representable as a standalone operator name in the E-QUERY-002 array (the grammar handles negation via the `not` keyword; the operator string is `IIN`).
- `e_query_pedagogical.rs` `required_string` operator list updated 5→8 (same set).
- `normalized_pql.rs` code comments listing valid string operators updated.
- `error_mapping.rs` auto-tracks (derives `valid_operators` dynamically from `valid_operators_for_type` — no separate change required).
- Red Gate test RG-074 `test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators` added in `crates/prism-query/src/engine.rs` module `#[cfg(test)] mod tests`. Test asserts that `valid_operators_for_type(ColumnType::String)` returns all 8 operators including IEQ, IIN, INE. RED before commit, GREEN at 633c5fab. 1407/1407 prism-query tests GREEN. 447/447 prism-mcp tests GREEN.

**TD-VSDD-060 sibling sweep:** 3 sites required update: `engine.rs` (primary), `e_query_pedagogical.rs` (sibling string-operator list), `normalized_pql.rs` (sibling code comment). `error_mapping.rs` auto-tracks. All sites updated atomically in 633c5fab.

---

### F-P24-LOW-001 (LOW) — stale "prompt uses IIN" comment + dead ALL-CAPS vocabulary entries in bc_2_10_016_audit_004_test.rs

**Finding:** In `crates/prism-query/tests/bc_2_10_016_audit_004_test.rs`, two categories of dead/stale content existed:

1. A code comment described the test prompt as using `IIN` to query severity. The `SENSOR_SEVERITY_VOCABULARY` allowlist was removed in pass-18 fix-burst (implementer e8b25d67). As part of that pass-10 fix, the armis triage prompt was corrected from `IIN ('HIGH','CRITICAL')` to `IN ('High','Critical')` (BC-2.11.024 mode-boundary contract: IIN not valid in raw SQL). However, the comment in `bc_2_10_016_audit_004_test.rs` was not swept at that time — it still said "prompt uses IIN" when the production code now uses case-sensitive `IN ('High','Critical')`.

2. The test data contained ALL-CAPS vocabulary entries (e.g., `"HIGH"`, `"CRITICAL"`, `"MEDIUM"`) in a vocabulary definition array. These entries were dead data — after pass-18 polish, the SENSOR_SEVERITY_VOCABULARY allowlist mechanism was replaced by the post-normalization `IEQ` example gate. The ALL-CAPS entries were never reached by any code path and represented the pre-normalization wrong state that the story explicitly fixes.

**Severity:** LOW — no runtime behavior impact; dead data and stale comment; misleads future readers about what the test actually validates.

**Closure:** CLOSED fix-in-scope per production-grade default. Same commit 633c5fab:
- Dead ALL-CAPS vocabulary entries removed from the test data array.
- Comment rewritten to describe the current state: the test verifies that the prompt uses Title-case values consistent with post-normalization OCSF state, not IIN.
- No new Red Gate test required (behavioral change is purely cleanup of dead/stale test data; the test's core assertion behavior is unchanged).

---

## SAP Probe Results (Pass 24, verified against 2de85b18)

**SAP-1 (tracing emission catalog completeness):** PASS — `ocsf.enum_label_unrecognized` dual sites match BC-2.16.002 catalog row 91. No new event_type values introduced in 2de85b18 delta. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — no sensor TOML or DTU changes.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — RG-074 is a non-`#[ignore]` unit test within `engine.rs` `#[cfg(test)] mod tests`; no external dependency required.

**POL-22 Phase A (ID/anchor integrity):** PASS — all 8 BC anchors and E-QUERY-002 reference verified.

**POL-22 Phase C (RGT inventory completeness):** PASS — story v1.28 adds RG-074 to the §Red Gate Tests table. All 74 RGT names (RG-001..RG-074) verified present. All domain entities present.

---

## Fix-Burst Commit Log (feature/S-PRISMQL-CASE-INSENSITIVE-001)

| Commit | Author | Change |
|--------|--------|--------|
| 633c5fab | implementer | `engine.rs` `valid_operators_for_type(ColumnType::String)` String arm 5→8 operators (=, !=, LIKE, IN, NOT IN, IEQ, IIN, INE); `e_query_pedagogical.rs` required_string 5→8; `normalized_pql.rs` comment updated; `error_mapping.rs` auto-tracks; RG-074 `test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators` (engine.rs mod tests) RED→GREEN; dead ALL-CAPS entries removed from `bc_2_10_016_audit_004_test.rs` + comment rewritten; 1407/1407 prism-query GREEN; 447/447 prism-mcp GREEN; F-P24-MED-001 + F-P24-LOW-001 CLOSED |

---

## Post-Fix-Burst State

- Feature HEAD: **633c5fab** (new frozen candidate for pass-25)
- 1407/1407 prism-query tests GREEN; 447/447 prism-mcp tests GREEN
- non-exhaustive: 89/89 UNCHANGED
- RG-001..074 GREEN
- LOCAL 3-CLEAN(strict) streak: **0/3** (RESET by fix-burst push 633c5fab per DRIFT-ORCH-PRLEVEL-PUSH-001)
- Novelty: LOW (E-QUERY-002 structured contract completeness; test-data dead-entry cleanup)
- NEXT ACTION: LOCAL adversary pass-25 on frozen 633c5fab
