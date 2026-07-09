---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [3]
feature_head_at_review: 39c8b134
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 2
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 3 — FIX-IEQ-ERRPATH-001

---

## Pass 3 (frozen 39c8b134; fresh-context adversary; PR-LEVEL cascade; streak candidate 2/3 — NOT ADVANCING — reset 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 3 total (0 CRIT / 0 HIGH / 0 MED / 1 LOW / 2 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — No new `event_type` values introduced; BC-2.16.002 v2.08 catalog complete.

**STREAK:** RESET 1/3 → 0/3 — CLEAN(strict)=NO on frozen 39c8b134. All 3 findings CLOSED same-burst; fix-burst pushed @8610ecd0. Per DRIFT-ORCH-PRLEVEL-PUSH-001 the push resets the streak. **Next: PR-LEVEL pass 4 on frozen 8610ecd0 (streak candidate 1/3).**

**Code HEAD at review:** 39c8b134 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 1 LOW + 2 OBS findings

**CLEAN(PR-merge):** YES — ZERO CRIT + HIGH + MED findings

---

## Findings

### ADV-PR-P3-LOW-001 — POL-22 Citation Drift: BC-2.11.016 §Implementation Location Table (LOW)

**Finding:** BC-2.11.016 v1.22 §Implementation location table cited pre-v1.21 extractor function names. The v1.21 PER-REFERENCE SCOPING fix replaced the name-keyed `HashSet<String>` mechanism with per-reference `(name, is_bare)` pairs and introduced new extractor functions (`extract_field_paths_with_bareness`, `extract_predicate_columns_with_bareness`). The implementation table still cited the retired function names at positions 1, 2, 3, 4, 5, and 6, leaving a POL-22 citation-accuracy defect visible to a fresh-context adversary reading spec-vs-code alignment.

**POL-22 Phase-C audit (expanded in-scope):** During the fix-burst, product-owner ran a full Phase-C audit of all 14 positions in the implementation table, finding 5 additional stale cells: positions 10–14 cited pre-BC-2.11.016 `check_availability_gate` / `check_enrich_columns` names; the correct extractor for positions 10–14 pipe stages is `extract_column_name_from_field_path` via `check_pipe_stage_columns`. Position 11 also contained a false HAVING-parenthetical in the column-check description (HAVING is a position-6 SQL-mode gate, not position-11 stats-by); removed.

**Status:** CLOSED

**Fix:** product-owner updated BC-2.11.016 v1.22 → **v1.23** (2026-07-09 D-1635):
- Positions 1/3/4/5: updated to `extract_field_paths_with_bareness`
- Positions 2/6: updated to `extract_predicate_columns_with_bareness`
- Positions 10–14: corrected to `extract_column_name_from_field_path` via `check_pipe_stage_columns`
- Position 11: false HAVING-parenthetical removed
- Sibling BC pins: BC-2.11.017 v1.10→**v1.11** (pin-only), BC-2.11.020 v1.15→**v1.16** (pin-only), BC-2.11.004 v1.27→**v1.28** (pin-only)
- Carrier story pin round: S-DEMO-PRISMQL-ONBOARDING-001-B v2.17→**v2.18**; S-DEMO-FIDELITY-REMEDIATION-001 v2.40→**v2.41**→**v2.42** (second bump: orchestrator-caught live present-tense mechanism prose in frontmatter comment + AC-M2 body updated to `_with_bareness` names; historical blockquotes preserved); S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.26→**v1.27**; S-PRISMQL-CASE-INSENSITIVE-001 v1.51→**v1.52**

---

### ADV-PR-P3-OBS-001 — Wasted Levenshtein Computation Under HEAD-JOIN Suspension (OBS)

**Finding:** Under the HEAD-JOIN SUSPENSION RULE (BC-2.11.016 v1.20+), when the gate reaches a bare-column reference in a SQL head position with a non-empty JOIN list, the gate suspends (fail-open, no E-QUERY-038 fired). However, the `did_you_mean` Levenshtein suggestion was still computed for the suspended reference before the fail-open return. This computation was immediately discarded (the `did_you_mean` value is only used when E-QUERY-038 is emitted), creating wasted CPU cycles on every suspended bare-head reference. Not a correctness defect — no user-visible behavior change — but a code hygiene OBS under the production-grade default.

**Status:** CLOSED

**Fix:** implementer @**8610ecd0** — threaded `compute_did_you_mean: bool` into `check_column_availability`; the HEAD-JOIN suspension arm passes `false` (skips Levenshtein computation, returns `did_you_mean: None` on the discarded path); 9 callsites swept per TD-VSDD-060 (no user-visible behavior change on any path).

---

### ADV-PR-P3-OBS-002 — Volatile Line Numbers in Audit Script G4 Comment (OBS)

**Finding:** `scripts/t13-preflight-audit.py` G4 item (the `check_sql_mode_rejection` predicate anchor introduced in the D-1632 fix-burst) carried a code comment citing a line number in the query engine source. Per TD-VSDD-091, line-number citations in non-test code decay on subsequent diffs and create false precision; the comment should use a durable function-name anchor instead.

**Status:** CLOSED

**Fix:** implementer @**8610ecd0** — replaced line-number citation with durable function-anchored citation (`check_query_column_availability` in `engine.rs`); `py_compile.compile` clean post-edit.

---

## Closure Summary

All 3 findings CLOSED in same-burst:
- ADV-PR-P3-LOW-001: spec-side only (BC-2.11.016 v1.23 + sibling pins + story pins; no code change)
- ADV-PR-P3-OBS-001: code-side @8610ecd0 (Levenshtein skip under suspension; TD-VSDD-060 sweep clean)
- ADV-PR-P3-OBS-002: code-side @8610ecd0 (durable citation; py_compile clean)

Branch pushed @8610ecd0. just check 5397/5397 GREEN; non-exhaustive 89/89. Streak RESET per DRIFT-ORCH-PRLEVEL-PUSH-001.

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0

**Novelty:** LOW — ADV-PR-P3-LOW-001 is a POL-22 citation-drift class finding that recurs when the implementation location table is not swept during extractor-rename fix-bursts. ADV-PR-P3-OBS-001 is a code-hygiene OBS (wasted computation with no user impact). ADV-PR-P3-OBS-002 is a TD-VSDD-091 violation class (volatile line number). None of these findings involve the security-critical injection-safety logic or the E-QUERY-038 gate correctness that were the primary subjects of passes 1 and 2.

**Pattern:** The 3 findings are spec-layer citation hygiene (LOW-001) + code efficiency (OBS-001) + doc-accuracy (OBS-002). No code-behavior defects. The 0 CRIT/HIGH/MED count confirms the security-critical surfaces (injection-safety chokepoint, MCP payload, emission catalog, gate correctness) are fully validated.

**Streak status:** 0/3 — RESET by fix-burst push @8610ecd0. **NEXT: PR-LEVEL adversary pass 4 on SAME frozen HEAD 8610ecd0** (streak candidate 1/3; BC-5.39.001; NO push before pass 4 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — No new `event_type` values in pass-3 review scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — ADV-PR-P3-OBS-001 fix threads `compute_did_you_mean: bool` through 9 callsites; the behavior change is observable (Levenshtein not computed on suspended path); no paper-fix.

**TD-VSDD-060 (sibling-site sweep):** PASS — `check_column_availability` signature change swept across 9 callsites per implementer TD-VSDD-060 discipline; no unswept site.

**POL-22 (citation accuracy):** FIXED — BC-2.11.016 v1.23 implementation location table updated; Phase-C audit complete (all 14 positions verified).

**BC-5.39.001 (3-CLEAN streak):** RESET — pass-3 result CLEAN(strict)=NO on frozen 39c8b134. Streak 1/3 → 0/3. Fix pushed @8610ecd0; next cascade gate on frozen 8610ecd0.
