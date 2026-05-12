# Adversarial Review — S-PLUGIN-PREREQ-C — LOCAL Pass 5

**HEAD:** 651bbb64
**Reviewer:** adversary (fresh-context, read-only profile)
**Date:** 2026-05-12
**Subject:** Fix-burst-4 closure verification + new-finding sweep
**Streak attempt:** 2/3 → target 3/3 **(LOCK ATTEMPT)**

---

## Closure Verification Table

| Pass-4 Finding | Closure Status | Evidence |
|---|---|---|
| F-LP4-MED-001 (sub-table arithmetic 29 vs 30) | **REAL** | Story v1.3 "Total: 30 types audited"; sub-tables 9+3+8+8+2=30. STORY-INDEX v2.64 row cites fix-burst-4 + types::SensorSpec coverage. |
| F-LP4-MED-002 (types::SensorSpec violator coverage) | **REAL** | struct_violations.rs defines `v30_types_sensor_spec()` importing `prism_spec_engine::types::SensorSpec as TypesSensorSpec` (correctly disambiguated from spec_parser::SensorSpec). ci.yml `EXPECTED=30`, scripts/check-non-exhaustive.sh `EXPECTED=30`. Paper-fix mental revert: removing annotation from types::SensorSpec drops count to 29 → CI fails. Real coverage. |
| F-LP4-LOW-001 (check-ci wiring) | **REAL** | Justfile contains `@scripts/check-non-exhaustive.sh` inside `check-ci` recipe AND `check` recipe. Both gates invoke. |
| F-LP4-LOW-002 (DtuMode footnote) | **REAL** | Story v1.3 contains footnote citing BC-3.2.005 source authority + explicit AC-5 scope exclusion. |
| F-LP4-OBS-001 (just check ordering) | **DEFERRED (acceptable)** | Justfile unchanged — cosmetic placement after fmt/clippy/nextest/doctests/crate-layout. |

**Closure: 4 REAL, 1 deferred-OBS (acceptable). Zero paper-fixes.**

---

## CI Regression-Detector Positive-Coverage Audit (POL-11)

- ci.yml positive-coverage log includes computed counts + explicit type enumeration spanning all 30 types including `types::SensorSpec`
- Runtime-computed via grep counts; not hardcoded
- `--color=never` neutralises ANSI escape codes
- scripts/check-non-exhaustive.sh uses JSON `--message-format=json` (dodges rustc per-file caps)
- `timeout-minutes: 12` adequate for cold cache
- Reachability grep guard confirms job presence
- No false-green vectors detected

---

## New Findings

### F-LP5-LOW-001 — main.rs documentation header still says "29 types" (S-7.01 partial-fix propagation gap)
- **Severity:** LOW
- **Category:** documentation-drift / S-7.01 partial-fix discipline
- **Confidence:** HIGH
- **Subject:** `tests/external/non-exhaustive-violation/src/main.rs` doc-header was not updated by fix-burst-4. Doc-header lines 17, 18, 20 still cite "29 types" / ">=29 errors" / "29 — AC-5 original 14 + fix-burst-2 sibling sweep 15". The bullet list enumerates 29 entries — missing entry "30. types::SensorSpec — struct, types.rs". The struct_violations.rs `v30_types_sensor_spec` function is genuine but undocumented in the file-level overview.
- **Why fresh-context caught it:** Fix-burst-4 commit touched struct_violations.rs + ci.yml + script + story; did not touch main.rs doc-header. Sibling-sweep discipline (S-7.01 case (b)) catches "fix applied to one surface, sibling co-located surface not updated".
- **Blast radius:** 1 file (main.rs); internal documentation only. Demoted to LOW.
- **Recommended fix (post-LOCK acceptable):** Three line touches + 1 line append to main.rs:
  - Line 17: `29` → `30`
  - Line 18: `29` → `30`
  - Line 20: `29` → `30`; "(AC-5 original 14 + fix-burst-2 sibling sweep 15)" → "(AC-5 original 14 + fix-burst-2 sibling sweep 15 + fix-burst-4 types::SensorSpec)"
  - Add: `//!   30. types::SensorSpec    — struct, types.rs`
- **Does this block LOCK?** **NO.** Per BC-5.39.001 streak gating, LOW findings do not reset streak.

---

## Other Sweep Results (all clean)

- Workspace 29/30 consistency: only one stale "29" remaining (main.rs above)
- Annotation count cross-check: in-scope count = 30 matches CI/story/script
- Paper-fix test on v30: real coverage (qualified import path disambiguates types::SensorSpec from spec_parser::SensorSpec)
- TD-VSDD-091 volatile-pin scan: no citations in story body
- POL-7 H1/source-of-truth: clean
- POL-11 STORY-INDEX consistency: clean
- POL-12 stub residue: unimplemented!() confined to infusion/* stubs (out of PREREQ-C scope)
- POL-13 frontmatter↔index: clean
- POL-14 BC promotion: BC-2.16.002 + BC-2.01.013 already active
- CI EXPECTED management: lower-bound semantic correct, comments in both ci.yml + script

---

## Total Findings by Severity

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 0 | — |
| LOW (OBS) | 1 | F-LP5-LOW-001 |
| **Total** | **1** | |

---

## Trajectory

PREREQ-C: 18 → 8 → 5 → 5 → **1 (pass-5)**.
- CRIT: 3 → 1 → 0 → 0 → **0**
- HIGH: 8 → 2 → 0 → 0 → **0**
- MED: 0 → 2 → 2 → 2 → **0** (both fix-burst-4 MEDs closed; no new MEDs)
- LOW/OBS: 7 → 3 → 3 → 3 → **1**

Novelty: **LOW** — refinements only. F-LP5-LOW-001 is genuine fresh-context (no prior pass flagged main.rs) but is a doc-drift consequence of fix-burst-4 itself, not a substantive contract gap. Spec has converged on substantive content.

---

## 3-CLEAN Streak Status

**3/3 — LOCAL CONVERGED.**

Zero CRITICAL + zero HIGH + zero MEDIUM findings. Per BC-5.39.001, MEDIUM and below are non-blocking for streak; this pass advances streak 2/3 → **3/3** and locks LOCAL CONVERGENCE.

The single LOW (F-LP5-LOW-001) is documentation-drift only, fixable in cleanup or absorbed into PR-LEVEL pass. It does not gate LOCK.

---

## Verdict

**LOCAL CONVERGED — streak 3/3 LOCKED.**

Fix-burst-4 closed all in-scope pass-4 MEDIUM findings cleanly and propagated 29→30 to all load-bearing surfaces (CI workflow, script, story body, sub-tables, AC-5 narrative, demo evidence pointer, file structure table). The CI regression detector for AC-5 remains robust. F-LP5-LOW-001 is a sibling-sweep doc-drift in main.rs internal header — non-blocking, single-file blast radius, recommended cleanup in next maintenance touch.

**Recommended next-step:** Cleanup F-LP5-LOW-001 (3 line touches + 1 line append) bundled into PR preparation OR absorbed into first PR-LEVEL fix-burst. Otherwise: proceed to PR creation via per-story-delivery step 5 (demo-recorder) → step 6 (rebase + pr-manager 9-step).

---

## Self-Validation Notes

- Evidence: file:line citations preserved as evidence anchors (test-file documentation is appropriate context, not volatile-pin in production code)
- Actionability: F-LP5-LOW-001 specifies exact line numbers + insertions
- Duplication: F-LP5-LOW-001 does not overlap with any pass-4 finding
- Novelty decay: findings 5→1 genuine convergence
- Fresh-context confirmation: this pass derived F-LP5-LOW-001 by independently sampling `\b29\b` workspace references (not from prior-pass reports). Compounding value confirmed.
