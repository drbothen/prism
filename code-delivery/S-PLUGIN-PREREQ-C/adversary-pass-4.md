# Adversarial Review — S-PLUGIN-PREREQ-C — LOCAL Pass 4

**HEAD:** 68c8b62d
**Reviewer:** adversary (fresh-context, read-only profile)
**Date:** 2026-05-12
**Subject:** Fix-burst-3 closure verification of pass-3's 5 findings + new-finding sweep
**Streak attempt:** 1/3 → target 2/3

---

## Closure Verification Table

| Pass-3 Finding | Closure Status | Evidence |
|---|---|---|
| F-LP3-MED-001 (story v1.1→v1.2 AC-5 narrative reconciliation to 29 types) | **REAL** | Frontmatter version 1.2. Changelog cites fix-burst-3 + ci.yml EXPECTED=29. AC-5 contains enumeration tables for 5 files. "8 types" remaining only as legitimate sub-section header for infusion/mod.rs. STORY-INDEX v2.63 row reflects v1.2. |
| F-LP3-MED-002 (11 MCP-wire types AC-5 scope exclusion) | **REAL** | grep returns 11 doc-comments with "AC-5 scope exclusion" in types.rs. Each cites MCP protocol stability. |
| F-LP3-LOW-001 (WriteStep::new + WriteEndpointSpec::new) | **REAL** | grep returns 2 occurrences of "NOT forward-compatible by itself" in write_endpoint.rs. Both include ..Default::default() example. |
| F-LP3-OBS-001 (just check wiring) | **REAL** | Justfile check recipe includes @scripts/check-non-exhaustive.sh. Standalone check-non-exhaustive recipe preserved. |
| F-LP3-OBS-002 (timeout monitor) | **DEFERRED-LOW (acceptable)** | ci.yml timeout-minutes: 12 unchanged. |

**Closure: 4 REAL, 1 deferred-LOW (acceptable). Zero paper-fixes.**

---

## CI Regression-Detector Positive-Coverage Audit

Reviewed ci.yml non-exhaustive-violation-compile-fail job + scripts/check-non-exhaustive.sh:
- Positive-coverage log line ✓
- Runtime-computed count via JSON parse ✓
- JSON dodges rustc per-file cap ✓
- Cargo failure-mode handled (CARGO_RC=0 fails) ✓
- timeout-minutes 12 (adequate for cold cache) ✓
- Reachability grep guard in verify-workflow-structure ✓

No false-green vectors detected in the AC-5 regression detector.

---

## New Findings

### F-LP4-MED-001 — Story AC-5 sub-table arithmetic: sums to 30 but body claims 29
- **Severity:** MEDIUM
- **Category:** spec-consistency / POL-7
- **Subject:** Story v1.2 states "Total: 29 types audited" but AC-5 sub-tables enumerate: spec_parser.rs 9 + write_endpoint.rs 3 + infusion 8 + types.rs 8 + column.rs 2 = 30.
- **Recommended fix:** Either (a) drop types::SensorSpec from story AC-5 table (sum = 29), OR (b) add v30_types_sensor_spec to struct_violations.rs and bump EXPECTED=30. Recommend (b) — types::SensorSpec is a config-input type.

### F-LP4-MED-002 — types::SensorSpec has #[non_exhaustive] but no compile-fail regression coverage
- **Severity:** MEDIUM
- **Category:** ci-as-code / regression-coverage (POL-11)
- **Subject:** types::SensorSpec at types.rs is annotated, but tests/external/non-exhaustive-violation/src/struct_violations.rs does NOT exercise it. Future PR removing annotation would not trip CI EXPECTED=29.
- **Recommended fix:** Add v30_types_sensor_spec() to struct_violations.rs; bump CI EXPECTED=30 and script EXPECTED=30. Co-resolves F-LP4-MED-001.

### F-LP4-LOW-001 — check-non-exhaustive not wired into just check-ci (sibling-sweep partial-fix)
- **Severity:** LOW
- **Category:** dev-experience / ci-as-code `[process-gap]`
- **Subject:** Justfile check-ci recipe (CI-equivalent local target) does NOT call @scripts/check-non-exhaustive.sh. Only just check (line 26) wires it.
- **Recommended fix:** Add @scripts/check-non-exhaustive.sh to check-ci recipe.

### F-LP4-LOW-002 — DtuMode is #[non_exhaustive] but not in story AC-5 table (pending intent)
- **Severity:** LOW (pending intent verification)
- **Category:** spec-completeness
- **Subject:** types.rs DtuMode is annotated with #[non_exhaustive] but doc-comment cites BC-3.2.005 (pre-existing). Story AC-5 table does NOT list DtuMode.
- **Recommended fix:** Either add footnote to story AC-5 sub-table listing DtuMode as "pre-existing annotation per BC-3.2.005 — not in AC-5 audit scope", OR confirm DtuMode IS in AC-5 scope and add to table + violator.

### F-LP4-OBS-001 — check-non-exhaustive terminal placement in just check (cosmetic)
- **Severity:** OBS (LOW)
- **Category:** dev-experience
- **Subject:** Justfile check recipe places check-non-exhaustive AFTER fmt+clippy+nextest+doctests+crate-layout. Cold-cache run takes 5-8 min before reaching non-exhaustive check. Fail-fast ordering (after clippy) would surface regressions faster.
- **Recommended fix:** No immediate action.

---

## Total Findings by Severity

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 2 | F-LP4-MED-001, F-LP4-MED-002 |
| LOW (OBS) | 3 | F-LP4-LOW-001, F-LP4-LOW-002, F-LP4-OBS-001 |
| **Total** | **5** | |

---

## Trajectory

PREREQ-C: 18 → 8 → 5 → **5 (pass-4)**.
- CRIT: 3 → 1 → 0 → **0**
- HIGH: 8 → 2 → 0 → **0**
- MED: 0 → 2 → 2 → **2** (NEW MEDs; pass-3 MEDs closed)
- LOW/OBS: 7 → 3 → 3 → **3**

Novelty: MEDIUM. F-LP4-MED-002 is a fresh-context find (annotation vs. violator-crate coverage cross-check). F-LP4-LOW-001 is sibling-sweep (S-7.01).

---

## 3-CLEAN Streak Status

**2/3.** Zero CRITICAL + zero HIGH findings. Streak advances 1/3 → 2/3.

MEDs non-blocking for streak per BC-5.39.001. Fix-burst-4 recommended to address F-LP4-MED-001 + F-LP4-MED-002 (joint resolution) + F-LP4-LOW-001 before pass-5 final lock attempt.

---

## Verdict

**CLEAN — streak advances 1/3 → 2/3.**

Fix-burst-3 closed all in-scope pass-3 findings cleanly. CI regression detector for AC-5 is robust. Two co-resolvable MED findings + 1 sibling-sweep LOW newly surfaced.

**Recommended next-step:** Fix-burst-4 addresses MED-001 + MED-002 + LOW-001 (3 file edits: struct_violations.rs, ci.yml, scripts/check-non-exhaustive.sh, Justfile, story v1.2→v1.3). Then pass-5 expecting streak 2/3 → 3/3 lock.
