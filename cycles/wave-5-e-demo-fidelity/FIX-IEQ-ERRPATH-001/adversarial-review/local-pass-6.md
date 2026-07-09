---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [6]
feature_head_at_review: fdfa78f2
fix_burst_head: d983613b
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  MED: 2
  OBS: 1
  total: 3
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 6 — FIX-IEQ-ERRPATH-001

---

## Pass 6 (frozen fdfa78f2; fresh-context adversary; fix-PR IEQ non-existent column error path; streak candidate 1/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 (2 MED + 1 OBS), 1 code-behavior defect

**Code HEAD at review:** fdfa78f2 (frozen; D-1616 implementer compute_sqlpipe_head_binding() — AS aliases ∪ bare Field names ∪ GROUP BY names; SELECT * → None → raw-schema fallback)

**Fix-burst HEAD:** d983613b (implementer: MIXED-STAR branch (c) Option A precise union; 34/34 module GREEN; just check 5353/5353 GREEN; non-exhaustive 89/89)

**LOCAL 3-CLEAN(strict) streak after pass-6:** 0/3 (NOT CLEAN(strict); fix-burst dispatched; RESET by @d983613b push per DRIFT-ORCH-PRLEVEL-PUSH-001)

---

## Finding ADV-FIX-P6-MED-001 — Story body prose drift: §BC-table Key Clauses cell frozen at stale v1.6 semantics

**Severity:** MED

**Classification:** story body prose drift — POL-23 same-row-adjacent-cell gap (pin-sync rounds updated BC-version cells but not adjacent semantic cells)

**Affected files:** `S-DEMO-PRISMQL-ONBOARDING-001-B` §Behavioral Contracts table — Key Clauses cell

**BC reference:** POL-23 (story BC-table completeness); BC-2.11.016 v1.13 (current head-projection rule semantics)

**Finding:** The §Behavioral Contracts table in S-DEMO-PRISMQL-ONBOARDING-001-B carried the Key Clauses cell description frozen at v1.6-era semantics: "twelve positions across three AST modes (SQL, filter, pipe); gate fires on column-not-found at each position." At fdfa78f2 the BC is at v1.13, which defines a FOURTEEN-position gate with the DERIVED-COLUMN BINDING RULE, the SqlPipe HEAD-PROJECTION BINDING RULE (EC-11-059/060/061), and the full enumerated binding-context lifecycle. The v1.6 "twelve positions" prose predates seven subsequent BC amendments (v1.7 grammar fix, v1.8 14-position expansion + FP-001/DERIVED-COLUMN BINDING RULE, v1.9 sort-grammar, v1.10 union-path, v1.11 payload dichotomy, v1.12 agg-arg walk, v1.13 head-projection seeding). The Key Clauses cell survived 7 consecutive pin-sync bursts because those bursts updated only the BC-version cells in the table, not the adjacent Key Clauses prose column. A literal re-implementer of the story would build a 12-position gate without the derived-column binding rule, producing false E-QUERY-038 on all derived-column and head-projection query shapes.

This is a POL-23 violation: same-row-adjacent-cell staleness. The pin-sync round protocol only bumped the version cells; the semantics cell accumulated a gap of 7 BC versions.

**Closure:** CLOSED — story-writer S-DEMO-PRISMQL-ONBOARDING-001-B v2.8→v2.9: Key Clauses cell rewritten to reflect v1.14 semantics — fourteen-position gate, three AST modes, DERIVED-COLUMN BINDING RULE (running {available,suspended} context; Enrich union/fail-open; Stats replaces), SqlPipe HEAD-PROJECTION BINDING RULE (aliases ∪ bare selected ∪ GROUP BY refs seeded before stage walk; MIXED-STAR branch (c) schema_columns ∪ explicit-cols union); 2 BC-2.11.016 pins→v1.14. Closes ADV-FIX-P6-MED-001.

---

## Finding ADV-FIX-P6-MED-002 — FP-001 mixed-star violation: has_star short-circuit returns raw-schema fallback for SELECT *, expr AS alias heads

**Severity:** MED

**Classification:** code-behavior — FP-001 false positive (has_star short-circuit bypasses Option A precise union for MIXED-STAR queries; false E-QUERY-038 on downstream alias and bare-field references)

**Affected files:** `crates/prism-query/src/engine.rs` — `compute_sqlpipe_head_binding()` has_star short-circuit path

**BC reference:** BC-2.11.016 v1.13 FP-001 invariant (DERIVED-COLUMN BINDING RULE); §Preconditions.2 HEAD-PROJECTION BINDING RULE

**Finding:** At fdfa78f2, `compute_sqlpipe_head_binding()` contains a short-circuit: when `has_star` is true (any head column is a star `*`), the function immediately returns `None`, causing `check_pipe_stage_columns` to fall back to raw source schema. For `SELECT * FROM t` (pure star), this is correct per BC-2.11.016 v1.13 EC-11-061 (star expands to schema-derived cols; raw-schema fallback is the right behavior). However, for mixed-star heads (`SELECT *, count(*) AS cnt`, `SELECT *, severity AS sev`, `SELECT *, severity`), the short-circuit discards explicitly enumerated output columns (aliases and bare field names) that should be added to the binding context on top of the schema-derived columns. A downstream pipe stage referencing the alias `cnt` or bare name `sev` on a mixed-star head will receive false E-QUERY-038 even though those columns are valid outputs.

Empirically confirmed: 3 RED tests constructed at commit @29155063 — `SELECT *, count(*) AS cnt ... | sort cnt` (alias ref), `SELECT *, severity ... | where severity = 'High'` (bare name ref), `SELECT *, severity AS sev ... | where sev = 'High'` (alias ref on bare alias) — all three fail with E-QUERY-038 on fdfa78f2 even though the queries are valid.

**Closure:** CLOSED — product-owner: BC-2.11.016 v1.13→v1.14 (D-1617): MIXED-STAR branch (c) added to SQLPIPE HEAD-PROJECTION BINDING RULE (Option A precise union): for `SELECT *, expr AS alias, bare_field, ...` head, binding = schema_columns ∪ {AS aliases} ∪ {bare un-aliased Field names} ∪ {GROUP BY names}; anonymous non-Field expressions → suspended fail-open per FP-001; EC-11-062 (mixed-star with alias — alias seeds binding; downstream pipe alias ref valid), EC-11-063 (mixed-star with bare field — bare name seeds binding; downstream pipe ref valid), EC-11-064 (SELECT * standalone — schema_columns fallback; downstream schema ref valid). BC-2.11.020 v1.7→v1.8 (branch enumeration restatement sync). BC-2.11.004 v1.19→v1.20 (pin-only). error-taxonomy v2.27→v2.28 (E-QUERY-038 row pins + MIXED-STAR note). BC-INDEX v7.65→v7.66. Implementer: RED @29155063 (EC-11-062..064 three failing mixed-star tests) → GREEN @d983613b (MIXED-STAR branch (c) in compute_sqlpipe_head_binding: when has_star, return Some(schema_columns ∪ aliases ∪ bare_names ∪ group_by_names) instead of None; pure-star with no additional columns → schema_columns set is the correct binding; 34/34 module GREEN; just check 5353/5353 GREEN; non-exhaustive 89/89). Closes ADV-FIX-P6-MED-002.

---

## Finding ADV-FIX-P6-OBS-001 — Process-gap: pin-sync rounds update BC-version cells but not adjacent semantic cells

**Severity:** OBS [process-gap]

**Classification:** protocol gap — POL-23 / POL-29 enforcement boundary (recurring class; see ADV-FIX-P6-MED-001 for manifestation)

**Finding:** ADV-FIX-P6-MED-001 is an instance of a recurring class: pin-sync rounds produced by story-writer historically bump the BC-version column in §Behavioral Contracts tables but leave adjacent semantic columns (Key Clauses, Description) unchanged. S-DEMO-PRISMQL-ONBOARDING-001-B accumulated 7 consecutive pin-sync bursts without a single semantic-cell update, allowing the Key Clauses cell to drift from v1.6 to v1.13 (7 BC amendments) undetected. In the same burst that MED-001 was found, all 4 cascade carrier stories were swept and MED-001 was the only remaining Key Clauses / Description cell gap — so the scope was bounded. However, the protocol gap that produced MED-001 remains: POL-23's verification steps and the story-writer pin-round procedure do not include a same-row adjacent-cell semantic currency check.

**Closure:** PROCESS — L22 appended to lessons.md (D-1617): codifies the same-row adjacent-cell drift class + codification-pending note for POL-23/POL-29 extension. MED-001 closed in-scope (story v2.9 Key Clauses cell refresh + semantic sweep of all 4 carrier stories). Full protocol codification (POL-23/POL-29 amendment) deferred per S-7.02 cycle-closing checklist before cascade cycle declared CLOSED.

---

## Pass PASS Notes

**SAP-1:** PASS — 3 `column_not_found.rejected` emission sites all cataloged in BC-2.16.002 v2.07 §Canonical Structured Event Catalog. No new `event_type` emissions in fdfa78f2 delta.

**POL-24 byte-for-byte:** PASS — `compute_sqlpipe_head_binding()` semantics verified against BC-2.11.016 v1.13 normative prose. No divergence between spec and implementation at fdfa78f2 (MIXED-STAR branch gap is an OMISSION, not a divergence).

**POL-23 sibling-sweep:** PASS (excluding MED-001 Key Clauses cell gap) — BC-2.11.016 POL-25 sibling BCs (BC-2.11.004 v1.19, BC-2.11.020 v1.7) are current. No other adjacent-cell drift detected in other carrier stories at fdfa78f2 (S-DEMO-FIDELITY-REMEDIATION-001, S-PRISMQL-CASE-INSENSITIVE-001, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Key Clauses cells verified current).

**Story pins:** PASS — 4 carrier stories (S-PRISMQL-CASE-INSENSITIVE-001 v1.43, S-DEMO-FIDELITY-REMEDIATION-001 v2.31, S-DEMO-PRISMQL-ONBOARDING-001-B v2.8, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.18) all current for BC-2.11.016 v1.13 / BC-2.11.004 v1.19 / BC-2.11.020 v1.7 / taxonomy v2.27 as pinned in D-1616. No multi-hop stale pins outside cascade perimeter at fdfa78f2.

---

## Fix-burst Summary

**Chain:** fdfa78f2 (frozen pass-6 HEAD — compute_sqlpipe_head_binding(); has_star short-circuit FP-001 on MIXED-STAR) → 29155063 (test-writer: RED EC-11-062..064 — mixed-star alias ref, mixed-star bare name ref, mixed-star alias-on-bare; 3 failing tests confirmed) → d983613b (implementer: MIXED-STAR branch (c) in compute_sqlpipe_head_binding — has_star path returns Some(schema_cols ∪ aliases ∪ bare_names ∪ group_by_names) instead of None; pure-star handled by schema_cols alone; 34/34 module GREEN; just check 5353/5353 GREEN; non-exhaustive 89/89)

**Spec closures:**

- BC-2.11.016 v1.13→v1.14 (product-owner): MIXED-STAR branch (c) Option A precise union; EC-11-062/063/064; closes ADV-FIX-P6-MED-002
- BC-2.11.020 v1.7→v1.8 (product-owner): branch enumeration restatement sync; BC anchor pins→v1.14
- BC-2.11.004 v1.19→v1.20 (state-manager): pin-only; BC anchor→v1.14
- error-taxonomy v2.27→v2.28 (product-owner): E-QUERY-038 row pins + MIXED-STAR note
- BC-INDEX v7.65→v7.66 (state-manager)

**Story closures (pin+semantic round — story-writer):**

- S-DEMO-PRISMQL-ONBOARDING-001-B v2.8→v2.9: Key Clauses cell rewritten to v1.14 semantics + 2 BC-2.11.016 pins→v1.14; closes ADV-FIX-P6-MED-001
- S-DEMO-FIDELITY-REMEDIATION-001 v2.31→v2.32: 6 BC-2.11.016 pins→v1.14; AC semantics verified current
- S-PRISMQL-CASE-INSENSITIVE-001 v1.43→v1.44: 4 BC-2.11.004 pins→v1.20; 1 error-taxonomy pin→v2.28
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.18→v1.19: 3 BC-2.11.020 pins→v1.8; EC-002b/FAMILY semantics verified current

**Lesson L22:** appended to lessons.md — process-gap: POL-23 pin-sync rounds update BC-version cells only, not adjacent semantic cells; codification pending.

**STORY-INDEX v2.637→v2.638. BC-INDEX v7.65→v7.66.**

**LOCAL 3-CLEAN(strict) streak after pass-6:** 0/3 (NOT CLEAN(strict) per BC-5.39.001). **NEXT:** freeze d983613b → LOCAL pass 7. If passes 7/8/9 CLEAN(strict) → LOCAL CONVERGED → push → PR → PR-LEVEL cascade.
