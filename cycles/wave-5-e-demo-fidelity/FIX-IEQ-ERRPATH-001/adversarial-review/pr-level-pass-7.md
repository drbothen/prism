---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [7]
feature_head_at_review: 8610ecd0
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 7 — FIX-IEQ-ERRPATH-001

---

## Pass 7 (frozen 8610ecd0; fresh-context adversary; PR-LEVEL cascade; streak candidate 2/3 — NOT ADVANCING — 1/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 1 total (0 CRIT / 0 HIGH / 1 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` returned no new values; BC-2.16.002 v2.08 catalog complete and unchanged.

**STREAK:** RESET 1/3 → 0/3 — NOT CLEAN(strict) on frozen 8610ecd0 (1 MED finding). Per BC-5.39.001, streak resets to 0/3. Same-burst fix pushed @ddf852bc. Per DRIFT-ORCH-PRLEVEL-PUSH-001, streak restarts on new frozen HEAD ddf852bc. **Next: PR-LEVEL pass 8 on SAME frozen ddf852bc (streak candidate 1/3; NO push before pass 8 per DRIFT-ORCH-PRLEVEL-PUSH-001).**

**Code HEAD at review:** 8610ecd0 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**Code HEAD after fix-burst:** ddf852bc (pushed; streak restarts; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 1 MED finding

**CLEAN(PR-merge):** NO — 1 MED finding (>= MED severity)

---

## Findings

### ADV-PR-P7-MED-001 — Inert assertion in t13-preflight-audit.py check A6

**Severity:** MED
**Confidence:** HIGH
**Novelty:** Genuinely novel — first pass to audit A–F check bodies line-by-line; prior passes focused on code-correctness surfaces (injection, column suspension, spec-citation)

**Finding:** `scripts/t13-preflight-audit.py` check A6 "list_capabilities tri-state model fields" was functionally INERT. The check computed `has_enabled_count` (a count of entries with `status == "enabled"`) but never used this variable in any assertion or conditional. The check emitted `PASS` unconditionally on any error-free API call, regardless of whether the tri-state model fields (`status`, `resolution_chain`) were actually present or whether the legacy `not_implemented` field was absent. This is the same class of defect as TD-VSDD-057 / PR-127 F-PG-001 (inert preflight gate that reports PASS without verifying the claimed contract).

**Contract it was supposed to verify:** BC-2.10.011 v1.5 §Postconditions — single-client `list_capabilities` response must include per-capability `status` (one of `enabled`, `runtime_disabled`, `compile_time_disabled`) and `resolution_chain` for disabled capabilities. The legacy `not_implemented` field must be absent (replaced by tri-state model in BC-2.10.011 v1.5).

**Root cause:** `has_enabled_count` was computed for a comment describing what A6 should check, but the actual assertion block was never written. The check fell through to `PASS` without any load-bearing assertion.

**Status:** CLOSED (same-burst fix @ddf852bc)

---

## Closure — ADV-PR-P7-MED-001

**Implementer:** @ddf852bc (same-burst fix; pushed to PR #219)

**Fix applied:** A6 rewritten with load-bearing assertions per BC-2.10.011 v1.5 single-client tri-state model. The rewritten A6 FAILs on:
1. `capabilities` key absent from response or not a dict
2. `not_registered_tools` key absent from response
3. Legacy `not_implemented` key present (forbidden by BC-2.10.011 v1.5)
4. Any capability entry missing `status` or `resolution_chain` key
5. Any capability entry with `status` outside `{enabled, runtime_disabled, compile_time_disabled}`

**Class-closure sweep — ALL A–F check bodies audited line-by-line:**
- A1–A22: all GENUINE (each check has at least one load-bearing assertion or explicit FAIL path)
- B: GENUINE
- C1–C7: all GENUINE
- **C8 RENAMED:** C8 was titled "NOW() accessible" but its check body never called `NOW()` — it validated SQL mode execution via `SELECT 1` (the ADR-052 §D4 baseline path). Renamed to "SQL mode executes (ADR-052 §D4 baseline path)" with a pointer to G7 (which explicitly tests `NOW()`). No behavioral change — the check body was already correct; only the title string was inaccurate.
- D: GENUINE
- E: GENUINE
- F: GENUINE

**Verification:** py_compile OK; just check 5397/5397 GREEN (no Rust test changes; A6 fix is Python-only); non-exhaustive 89/89.

---

## Probe Summary

### Probe 1 — Full A–F check body audit (novel probe; first pass to execute this sweep)

Systematic line-by-line review of all preflight check bodies in `scripts/t13-preflight-audit.py`.

Methodology: for each check, identified (a) the claimed contract from the docstring/comment, (b) the actual assertion(s) present, and (c) whether the check would FAIL on a contract violation.

**A6 finding:** `has_enabled_count = sum(1 for cap in capabilities.values() if cap.get("status") == "enabled")` computed but never referenced in any `assert`, `if ... fail()`, or `return` conditional. The check body fell through to unconditional `PASS`.

**C8 finding (non-blocking, remediated proactively):** Title claimed "NOW() accessible" but body executed `SELECT 1` for SQL mode baseline test. Renamed for accuracy. G7 is the proper `NOW()` test.

**All other checks (A1–A5, A7–A22, B, C1–C7, D, E, F):** Each check has at least one load-bearing assertion that would trigger FAIL on a contract violation. No additional inert-assertion instances found.

### Probe 2 — SAP-1: Structured Event Catalog completeness

`rg 'event_type\s*=' crates/ --type rust` — no new `event_type` values in PR-LEVEL pass-7 scope. BC-2.16.002 v2.08 catalog complete and unchanged. **SAP-1 PASS.**

### Probe 3 — TD-VSDD-059 (paper-fix detection on A6 closure)

A6 rewrite verified load-bearing: FAIL-paths confirmed for all 5 contract violation cases. The fix is structural (assertion added), not a doc-comment rename. **TD-VSDD-059 PASS.**

### Probe 4 — TD-VSDD-060 (sibling-site sweep on A6 changes)

A6 is a self-contained check function with a single call site in the main check dispatch table. No additional A6-referencing sites to sweep. C8 rename affected only the title string (no behavior change). **TD-VSDD-060 PASS.**

---

## Version Summary

**No spec/story version changes this pass.** Pass-7 finding is confined to `scripts/t13-preflight-audit.py` (Python audit script). The fix is a behavioral correction to the script, not a spec or BC revision. All spec and story versions carry forward from D-1638:
- BC-2.11.016 v1.25 (UNCHANGED)
- BC-2.11.017 v1.13 (UNCHANGED)
- BC-2.11.020 v1.18 (UNCHANGED)
- BC-2.11.004 v1.30 (UNCHANGED)
- S-DEMO-FIDELITY-REMEDIATION-001 v2.44 (UNCHANGED)
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.20 (UNCHANGED)
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.29 (UNCHANGED)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.54 (UNCHANGED)
- error-taxonomy v2.35 (UNCHANGED)
- BC-2.16.002 v2.08 (UNCHANGED)

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 6 on frozen 8610ecd0: 0 findings (CLEAN(strict); streak 0/3 → 1/3)** → **PR-LEVEL pass 7 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN(strict); streak RESET 1/3 → 0/3]** → same-burst fix pushed @ddf852bc

**Novelty:** HIGH — This pass deployed a novel probe (full A–F check body audit) that no prior pass had attempted. ADV-PR-P7-MED-001 is a genuinely novel finding: the same class (inert-assertion false-green demo-preflight gate) as TD-VSDD-057 / PR-127 F-PG-001, but not a recurrence — the prior instance was in the G-check area, this is in the A-check area (tri-state model validation). The class-closure sweep confirmed no other instances exist.

**Pattern:** Finding is confined to the audit script surface (Python). The production Rust code and spec/story surfaces remain clean across all 7 passes (zero CRIT/HIGH code-behavior defects in the entire PR-LEVEL cascade). Decay signature: 3→0→3→1→3→0→[1]. The oscillation pattern broken by pass-7 deploying a novel probe category (script behavioral correctness).

**Streak status:** 0/3 — RESET by pass-7 finding. Fix pushed @ddf852bc. Per DRIFT-ORCH-PRLEVEL-PUSH-001, streak restarts on new frozen HEAD ddf852bc. **NEXT: PR-LEVEL adversary pass 8 on SAME frozen HEAD ddf852bc** (streak candidate 1/3; NO push before pass 8).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — `rg 'event_type\s*=' crates/ --type rust` finds no new `event_type` values in PR-LEVEL pass-7 scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — A6 rewrite is structural (load-bearing assertions added); not a doc-comment rename. Five concrete FAIL conditions verified in code.

**TD-VSDD-060 (sibling-site sweep):** PASS — A6 has a single call site. C8 rename is title-string only; no behavioral propagation sites.

**BC-5.39.001 (3-CLEAN streak):** 0/3 — pass-7 result NOT CLEAN(strict). Streak RESET 1/3 → 0/3. Fix-burst pushed @ddf852bc (new frozen HEAD). Next pass re-gates on ddf852bc (streak candidate 1/3).
