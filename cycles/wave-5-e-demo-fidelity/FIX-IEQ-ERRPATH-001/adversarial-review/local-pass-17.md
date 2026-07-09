---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [17]
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
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 17 — FIX-IEQ-ERRPATH-001

---

## Pass 17 (frozen 35117a38; fresh-context adversary; rotated angles; fix-PR IEQ non-existent column error path; streak candidate 1/3 — ADVANCES to 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK ADVANCE: 0/3 → 1/3** (BC-5.39.001; first CLEAN(strict) on frozen 35117a38)

**Code HEAD at review:** 35117a38 (frozen; D-1628 fix-burst: EC-11-076 per-reference HEAD-JOIN scoping — replaced name-keyed bare_head_cols HashSet<String> with per-reference (name, is_bare) pairs; 5391/5391 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**CLEAN(strict):** YES (zero findings of any severity — streak advancement criterion satisfied)

**CLEAN(PR-merge):** YES (zero CRIT + HIGH + MED findings — PR-merge gate satisfied)

---

## Probe Coverage

All probes returned empty-handed. Full coverage:

### FP-001 SHAPE BATTERY (16 shapes)

**All 6 EC-11-076 shapes (pass-16 fix-burst, per-reference HEAD-JOIN scoping):**

1. **Qualified SELECT with typo:** `SELECT alias.typo_col FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — E-QUERY-038 fires on `alias.typo_col` (qualified typo, NOT suspended). CORRECT at @35117a38.

2. **Table-qualified SELECT with typo:** `SELECT t1.typo_col FROM crowdstrike_alerts AS t1 JOIN some_other_table ON t1.id = some_other_table.id WHERE col = 'x'` — E-QUERY-038 fires on `t1.typo_col`. CORRECT.

3. **Qualified agg-arg with typo:** `SELECT sum(alias.typo_severity) FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — E-QUERY-038 fires on `alias.typo_severity`. CORRECT.

4. **SqlPipe form:** Same per-reference scoping applies in SqlPipe head; qualified typo fires E-QUERY-038 regardless of bare WHERE col in positions 1–6. CORRECT.

5. **Bare WHERE ref correctly suspended (negative control):** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id WHERE col = 'high'` — bare `col` in WHERE head position is fail-open per EC-11-074. CORRECT — not changed by @35117a38.

6. **Qualified present-col correctly passes gate (negative control):** `SELECT alias.severity FROM crowdstrike_alerts AS alias JOIN some_other_table ON alias.id = some_other_table.id WHERE col = 'x'` — `alias.severity` exists in schema; no E-QUERY-038 fired. CORRECT.

**EC-11-074/075 regression check (pass-15 fix-burst, HEAD-JOIN SUSPENSION RULE):**

7. **Bare head SELECT with JOIN:** `SELECT col FROM t1 JOIN t2 ON t1.id = t2.id` — bare `col` in SELECT head suspended (fail-open) per EC-11-074. CONFIRMED INTACT at @35117a38.

8. **Bare head WHERE with JOIN:** `SELECT count(*) FROM t1 JOIN t2 ON t1.id = t2.id WHERE col = 'x'` — bare `col` in WHERE head suspended per EC-11-075. CONFIRMED INTACT.

**STAR-WITH-JOIN / STAGE-JOIN independence (passes 12-14):**

9. **STAR-WITH-JOIN:** `SELECT * FROM t1 JOIN t2 ON t1.id = t2.id WHERE t1.nonexistent = 'x'` — star in head; qualified `t1.nonexistent` outside star position fires E-QUERY-038; star itself does not trigger the suspension. CONFIRMED INTACT at @35117a38.

10. **STAGE-JOIN independence:** SqlPipe stage-join (STAGE-JOIN SUSPENSION RULE, EC-11-072) behavior independent of HEAD-JOIN SUSPENSION RULE. Gate operates on stage-scoped schema, not head schema. CONFIRMED INTACT.

**JOIN ON position-5 bare + unknown-qualifier refs:**

11. **JOIN ON bare ref (position 5):** Bare ref in ON clause does not inhibit E-QUERY-038 on unrelated qualified SELECT ref. The suspension is scoped to head positions 1–6 only; ON clause is not a head position per BC-2.11.016 v1.21. CONFIRMED CORRECT.

12. **Unknown-qualifier ref in SELECT with JOIN:** `SELECT unknown_alias.col FROM t1 JOIN t2 ON t1.id = t2.id` — `unknown_alias` is not a registered alias; E-QUERY-038 or equivalent fires. Not suppressed by HEAD-JOIN SUSPENSION. CONFIRMED CORRECT.

**FROM-alias==column-name pathological case:**

13. **Alias name collides with column name:** `SELECT col FROM t1 AS col JOIN t2 ON t1.id = t2.id WHERE col = 'x'` — the alias is `col`; bare `col` in head is suspended per EC-11-074/075; no spurious qualified-ref suppression. `t2.col` (if t2.col exists) fires E-QUERY-038 normally. CONFIRMED CORRECT — per-reference scoping handles the alias==column-name case cleanly.

**E-QUERY-002 gate interaction:**

14. **E-QUERY-002 + JOIN:** When a non-existent column resolves via E-QUERY-002 (QueryTypeMismatch) rather than E-QUERY-038, the HEAD-JOIN SUSPENSION RULE does not interact with E-QUERY-002 logic (they are separate gates). CONFIRMED — no interaction gap at @35117a38.

15. **E-QUERY-002 alone (no JOIN):** Standard behavior preserved. CONFIRMED INTACT.

16. **IEQ on non-existent column with JOIN:** The original WARN-2 trigger `severity_id IEQ 'x'` on a table with JOIN — E-QUERY-038 fires correctly with did_you_mean hint; not suppressed. This was the FIX-IEQ-ERRPATH-001 root motivation, now verified intact through all 17 passes. CONFIRMED.

### EXTRACTOR SYMMETRY AUDIT

**match-arm-by-match-arm review of extractor pairs at @35117a38:**

- `extract_field_paths_with_bareness` vs `extract_field_paths_from_expr`: `extract_field_paths_with_bareness` is the NEW function introduced at @35117a38; returns `(name, is_bare)` pairs. `extract_field_paths_from_expr` is the RETAINED function for non-suspension contexts (returns bare name only). Both arms cover the same AST node variants (FieldPath, AliasedExpr, FunctionCall, etc.); no arm present in one but absent in the other for structurally equivalent nodes. SYMMETRIC.

- `collect_predicate_columns_with_bareness` vs `collect_predicate_columns`: same analysis; `collect_predicate_columns_with_bareness` returns `(name, is_bare)` pairs; `collect_predicate_columns` returns bare names only. Both handle Eq, Lt, Gt, In, Like, IsNull, And, Or, Not at equivalent depth. No asymmetric gap found. SYMMETRIC.

### DEAD-CODE RESIDUE CHECK

**`bare_head_cols` / `collect_bare_field_names_from_expr` / `collect_bare_pred_field_names` (removed at @35117a38):**

Searched `crates/prism-query/src/` for any surviving usage of `bare_head_cols`, `collect_bare_field_names_from_expr`, `collect_bare_pred_field_names`:

- Live source files: ZERO occurrences. All three identifiers were cleanly removed at @35117a38.
- Historical test docstrings and `// Previously:` comments: found in 2 test file comment blocks (non-executable; these reference the old approach for archaeological context). These are NOT dead code — they are comments. No `bare_head_cols` survives as a live binding anywhere. CLEAN.

### CALLSITE SYMMETRY

**Both callsites (`execute` and `execute_scheduled`) in `prism-query/src/execute*.rs`:**

- `execute`: calls `check_query_column_availability` with the full `InfusionRegistry` argument; receives `(name, is_bare)` extraction paths. VERIFIED SYMMETRIC.
- `execute_scheduled`: same call signature; same code path. VERIFIED SYMMETRIC.

No asymmetric callsite found. Both callsites received the D-1628 refactor identically.

### POL-23/25/29 SIBLING + CARRIER CURRENCY

BC and story version currency verified at @35117a38 fix-branch state:

| Artifact | Expected version | Status |
|----------|-----------------|--------|
| BC-2.11.016 | v1.21 (PER-REFERENCE SCOPING) | CURRENT |
| BC-2.11.017 | v1.9 (pin-only D-1628) | CURRENT |
| BC-2.11.020 | v1.14 (prose+pin D-1628) | CURRENT |
| BC-2.11.004 | v1.26 (pin-only D-1628) | CURRENT |
| error-taxonomy | v2.34 (D-1628) | CURRENT |
| S-DEMO-FIDELITY-REMEDIATION-001 | v2.39 (D-1628 pin round) | CURRENT |
| S-DEMO-PRISMQL-ONBOARDING-001-B | v2.16 (D-1628 pin round) | CURRENT |
| S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 | v1.25 (D-1628 pin round) | CURRENT |
| S-PRISMQL-CASE-INSENSITIVE-001 | v1.50 (D-1628 pin round) | CURRENT |

Zero stale live pins detected across all 4 carrier stories. POL-23/25/29 PASS.

### POL-16 CLEAN

No `#[allow(...)]` suppressions introduced at @35117a38 or any prior fix-burst commit in this cascade that were not already present. POL-16 CLEAN.

### TEST INTEGRITY LOAD-BEARING

All 6 EC-11-076 tests introduced at @5ef1a1a8 and made GREEN at @35117a38 remain present and GREEN. Each test exercises the production code path without `#[ignore]`. Confirmed load-bearing (not doc-comment / rename-only closure). TD-VSDD-059 PASS.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace at frozen 35117a38. Five total emission sites verified: three `column_not_found.rejected` sites + two `reload.*` sites. All five catalog rows exist in BC-2.16.002 §Postconditions with full field schema, audit role, and recurrence policy. Zero new `event_type` assignments introduced at @35117a38 (the D-1628 fix-burst replaced HashSet<String> with per-reference pairs — a pure logic refactor; no new tracing emissions). SAP-1 coverage UNCHANGED from pass-16.

**SAP-2 N/A:** No sensor TOML spec modifications in this fix cascade.

**POL-24 N/A (this pass):** No new EC body authored in this burst (pass-17 is CLEAN; no fixes).

**Audit-script Section G arithmetic:** UNCHANGED from D-1628 — EC-11-076 (added D-1628) remains the latest EC in BC-2.11.016 v1.21; section G count 73; no additions this pass.

---

## Convergence Assessment

**Trajectory:** 6→3→3→2→1→[0]→2→[0]→4(low/obs)→1(med)→1(med)→[0]

**Pattern:** Pass 17 probes the @35117a38 fix-branch HEAD with all prior FP-001 shape battery (16 shapes), extractor-symmetry audit, dead-code residue check, callsite symmetry, and full sibling currency sweep. All probes returned empty-handed. The per-reference (name, is_bare) implementation at @35117a38 is sound under all probed angles.

**Novelty assessment:** LOW-to-ZERO — Pass 17 covered the full FP-001 battery including all 6 EC-11-076 shapes (angle set from pass-16's finding), the EC-11-074/075 regression checks, STAR-WITH-JOIN and STAGE-JOIN independence, JOIN ON position-5 shapes, FROM-alias==column-name pathological case, E-QUERY-002 gate interaction, extractor symmetry, dead-code residue, and callsite symmetry. No new vulnerability surface was found. The implementation is consistent with BC-2.11.016 v1.21 semantics throughout.

**Streak status:** 1/3 (BC-5.39.001). VERY NEXT ACTION: freeze 35117a38 → LOCAL adversary pass 18 (fresh context, strict; streak candidate 2/3). No commits to fix branch per DRIFT-ORCH-PRLEVEL-PUSH-001. Three consecutive CLEAN(strict) passes on unchanged HEAD required (passes 17/18/19) → then push branch + open fix-PR via pr-manager.
