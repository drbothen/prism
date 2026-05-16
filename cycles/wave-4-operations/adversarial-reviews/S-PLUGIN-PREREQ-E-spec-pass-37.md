---
document_type: adversarial-review-pass
pass: 37
cycle: S-PLUGIN-PREREQ-E-spec
date: 2026-05-16
reviewer: adversary
predecessor_pass: 36
predecessor_burst: "FB28 D-645 SHA 29784b61"
verdict: BLOCKED
finding_count: { CRIT: 0, HIGH: 0, MED: 3, LOW: 0, OBS: 2 }
streak_status: "0/3 stays 0/3 — BLOCKED holds streak; 9th attempt at 3-CLEAN sequence"
fix_burst: FB29
fix_burst_committed: <state-manager records SHA after commit>
---

# S-PLUGIN-PREREQ-E Spec — Adversarial Review Pass 37

## §1 Summary
BLOCKED. 3 MED findings + 2 OBS observations. Streak 0/3 → 0/3 (BLOCKED holds at zero). 9th attempt at 3-CLEAN sequence.

## §2 Methodology
Loaded all 19 spec-package artifacts. Resolved actual paths (orchestrator's hint paths for ADRs / VPs / HS were stale — ADRs under `.factory/specs/architecture/decisions/`, VP files lowercase, holdouts at `.factory/holdout-scenarios/`). Applied 25 active policies (POL-1..16, 18, 20..27) + production-grade lens + TD-VSDD-059 paper-fix detection + TD-VSDD-060 sibling-sweep gate. POL-22 Phase A (lexical-vs-semantic anchor verification by grepping target docs) and Phase C (named-entity grep against canonical sources) applied throughout.

## §3 Findings

### F-LP37-MED-001 — AC-8 cites non-existent test name; within-FB28 sibling-sweep gap
- **Severity:** MEDIUM
- **Policy:** POL-25 (multi-cite propagation), CLAUDE.md TD-VSDD-060
- **File:** `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md` line 235
- **Evidence:** AC-8 prose cited singular `test_BC_2_16_012_spec_parser_behavioral_equivalence` which exists nowhere in Red Gate. FB28 expanded Red Gate Tests 7-10 to sensor-suffix names but did not propagate to AC-8.
- **Fix routing:** product-owner — replace with explicit enumeration of 4 canonical test names.
- **Closure:** FB29 PO stage — AC-8 line 235 rewritten with 4 canonical names. Story v1.13 → v1.14.

### F-LP37-MED-002 — Story Task 7 lists OnceLock<RwLock<...>> as valid alternative; ADR-026 D7 forbids
- **Severity:** MEDIUM
- **Policy:** POL-4 (semantic anchoring), CLAUDE.md Source-of-Truth Precedence rule #2
- **Files:** story line 170; ADR-026 line 246
- **Evidence:** Story Task 7 line 170 listed `OnceLock<RwLock<...>>` as valid alternative; ADR-026 D7 lines 246-259 explicitly forbid it.
- **Fix routing:** product-owner — strike parenthetical, cite ADR-026 §D7.
- **Closure:** FB29 PO stage — Task 7 line 170 rewritten with ADR-026 §D7 citation.

### F-LP37-MED-003 — VP-153 message templates diverge byte-for-byte from canonical error-taxonomy.md
- **Severity:** MEDIUM
- **Policy:** POL-24 (error_message_template_verbatim), CLAUDE.md Source-of-Truth Precedence rule #3
- **Files:** VP-153 lines 49-50, 57-58, 62-64; error-taxonomy.md lines ~384-386
- **Evidence:** VP-153 Rule A/B/C message-format quotations for E-SPEC-012/013/014 had divergent placeholders + prose vs canonical error-taxonomy.md v1.30. Pre-existing defect surviving 36 prior passes.
- **Fix routing:** architect — Option A (byte-verbatim sync from canonical).
- **Closure:** FB29 architect stage — Rule A/B/C rewritten byte-verbatim from error-taxonomy.md v1.30. VP-153 v0.5 → v0.6.

## §4 FB28 Paper-Fix Audit
All three FB28 closures verified load-bearing (TD-VSDD-059 audit passes):
1. F-LP36-MED-001 (AC-9 test name canonicalization) — verified: AC-9 line 239 cites `test_BC_2_16_012_003_write_tool_invalidation_runtime_register`; Red Gate Test 11 line 279 uses same name.
2. F-LP36-MED-002 (Red Gate Tests expanded to 4-sensor breadth) — verified: frontmatter `red_gate_tests: 11` matches body count; Tests 7-10 use canonical `_002_` segment per Test 7's behavioral-equivalence convention.
3. F-LP36-MED-003 (STORY-INDEX crates_touched col 3 adds prism-query) — verified: STORY-INDEX line 395 col 3 = `prism-sensors,prism-spec-engine,prism-query`; story frontmatter matches; ADR-027 v1.4 has SS-07.

## §5 Sibling-Sweep Audit (TD-VSDD-060)
- STORY-INDEX v2.117 row v1.13 ✓
- Story v1.13, updated 2026-05-16 ✓
- Frontmatter `crates_touched: [SS-01, SS-07, SS-16]` ✓
- No AC↔"Red Gate Test N" numbered back-references ✓ (renumber 8→11 non-disruptive)
- BC-2.16.002 v1.20 catalog row count = 33 ✓
- All 5 story-cited BCs versions in BC-INDEX match ✓
- VP-INDEX totals 156/122/34 ✓
- verification-coverage-matrix totals match ✓
- **Sibling-sweep gap detected (F-LP37-MED-001):** AC-8 test name was NOT updated during FB28 when Red Gate Tests 7-10 expanded to use sensor-suffix names.

## §6 Observations (non-blocking)

### OBS-LP37-001 — HS-PREREQ-E-001-03 line 128 "behaviorally unchanged" loose phrasing
- AC-2 + INV-AUTH-OPEN-002 say each impl gains one new method body (`auth_type_name()`); HS prose calls impls "behaviorally unchanged".
- Intent is "observable runtime behavior of EXISTING call sites is unchanged".
- LOW severity, non-blocking.

### OBS-LP37-002 [process-gap] — Story v1.13 changelog "BC-2.16.012 row 003" misnomer
- `_NNN_` segments in Red Gate test names are intra-test-set grouping numbers, NOT BC TV/EC/INV identifiers in BC-2.16.012 body.
- LOW severity. Folded into v1.14 changelog per PO stage.
- [process-gap] candidate: codify Red-Gate-test-segment naming convention separate from BC TV/EC numbering.

## §7 Convergence Trajectory
- Pass-37: BLOCKED (3 MED findings)
- Streak: 0/3 → 0/3 (BLOCKED holds; 9th attempt at 3-CLEAN sequence)
- Pattern observed: F-LP37-MED-001/002 are within-FB28 sibling-sweep gaps (11+ POL-23 manifestation pattern from changelog history). F-LP37-MED-003 is fresh-context surfacing of stale POL-24 violation surviving 36 prior passes — illustrates fresh-context compounding value.
- Recommendation: Dispatch FB29 with PO (MED-001/002) + architect (MED-003); state-manager runs last per POL-3. Pass-38 (2nd of 9th 3-CLEAN attempt) follows.
